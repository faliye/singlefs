# rules-compress-residents-r2 辩方腿报告

日期：2026-11-16。立场：辩方，复核第一轮判决 `research/prompts/rules-compress-residents-r1-main-verification.md`，替第一版合稿与被判不采纳的腿各辩一次。没写模型；核对用的是只读文本脚本（下面「复核方式」），不起项目代码，没往任务服务交任务。

引文写法：原件 = 仓里 `.claude/…`；第一版 = `research/prompts/rules-compress-r1-merged/dot-claude/…`；第二版 = `research/prompts/rules-compress-r2-merged/dot-claude/…`。行号都是这一轮现查的。

## 复核方式与基线

- 开工核了清单：`sha256sum -c research/prompts/rules-compress-residents-r2-snapshot/judged-sha256.txt` 在仓根跑，35 份原件全部 OK（不是 OK 的 0 行）；第一版、第二版合稿树各按快照里的 `merged-tree-sha256.txt` 核过，0 份对不上。
- 第一轮两份腿报告的引文：`python3 scripts/agents/cite-check.py research/prompts/rules-compress-residents-r1-opus-output.md research/prompts/rules-compress-residents-r1-sonnet-output.md --background research/prompts/_rules-compress-residents-r1-body.md`，末行：`✓ 核了 2175 处引文、2 处裸行号，对上 2175 处，没判 2 处`（没判的两处是攻方报告第 57、66 行引合稿总表第 120、46 行的裸行号，我读了那两行，与攻方说的一致）。
- 草稿目录 `~/.local/share/singlefs/work/rules-compress-residents-r2-sonnet/`：`ptr.py`（找 `上游规则文件`加括号节名的指针并核节标题）、`pairs.py`（把 `diff-from-r1.patch` 里的换括号改动与别的改动分开）、`res.py`（全树「文件」「节」指针解析），sha256 见文末。三份都只读文本。

## 一、判决逐条复核

「站得住」= 这一条打中成立；「站不住」= 其实不成立或打偏。每条先写替第一版合稿能辩的最强理由，再写它撑不撑得住。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 1 指到验证规则「一个门禁：两种跑法」的指针落空 | 站得住（最强辩词只能说「差一跳」，撑不住判据字面） | 第一版 `rules/verification.md` 第 26 到 47 行就是那一节（下一个标题在第 48 行），把第 26 到 92 行（含后面各节）灌进 `grep -nE 'lkmm|litmus|领卡|核账|九格'` 也是 0 行（量的）。指到它的：第一版 `rules/fs-design.md:112`「工具是门禁 lkmm（每份 litmus 一条判定，…「一个门禁：两种跑法」）」、`skills/crash-test/commit-stages.md:24`「各片与合并各判什么」、`skills/crash-test/SKILL.md:18`「各格判什么在 …verification.md」。话在参考件：第二版 `rules/verification-reference.md:42`「lkmm 每份 litmus」、`:18`「合并那一步照常判九格、crash-case-replay 格核账不现判」、`:21`「要上卡的三处」。辩词：第一版 `verification.md:44` 与 `:99` 有两条「先读 verification-reference.md…」「逐行在 verification-reference.md「门禁管哪一半」」，两跳到得了。撑不住：判据格问的是「指到的地方有没有那句话」，一跳到不了；且 `:44` 只点名参考件里的「一个门禁：两种跑法」，不指 lkmm 在哪。原件同一话在 `.claude/rules/verification.md:42`（那一节里），是压缩拆文件带来的。 |
| 2 证红的变异清单写死成 crates/mutations.tsv | 站得住 | 原件泛称：`.claude/rules/show-me-test-by-review.md:12`「做成**存进仓里的变异清单**」；第一版 `rules/show-me-test-by-review.md:7`「做成存进仓里的变异行（`crates/mutations.tsv`）交给门禁反复复跑」。仓里两条路：`.claude/agents/experiment-runner.md:35`「入库装置那一支不写 research 的变异表…变异行照 `crates/mutations.tsv` 第 1 行表头的六段格式追加」与「`research/` 那一支写变异表 `scripts/mutation/tables/e<号>_<英文名>.tsv`」；门禁 `.claude/gate.d/code-source-discipline.sh:1271`「research 那侧：scripts/experiment-apparatus/ 下每个 *.rs 在 scripts/mutation/tables/ 下有同名 .tsv」。辩词：用户定案第 4 条只说「变异行之外 commit message 也写」，文件名是顺手写的。撑不住：写成泛称的是原件，第一版把射程读窄成一张表，对 `scripts/experiment-apparatus/` 那一支做不到。（腿报告第一版判「两条腿都报」属实：攻方 二-1、正推 格二 1。） |
| 3 新加的「两处说同一件事以上游为准」 | 站得住 | 原件 `.claude/rules/code-discipline-by-review.md` 没有开头段，`grep -c '以上游为准'` 输出 0；仓里 `.claude/rules/*.md` 与 `.claude/singlefs-ai-sop/rules/*.md` 里 `grep -rn '以上游为准\|同射程'` 也 0 行。第一版 `rules/code-discipline-by-review.md:4`「…同射程，只写上游没写的；两处说同一件事以上游为准。」。辩词：是个说明句不是判据。撑不住：它规定了两处冲突时谁赢，是新判据，且总表里没登。只有正推腿报了这一条（判决没写「两条腿」，对得上）。 |
| 4 四处半句没有落点 | 站得住，四处都是原件原话；一处实做法，三处小 | 四处在原件与第二版的原话逐字对得上（见第二节表）：`等改动停下再跑，或者用 --staged` 是整句做法；「引的时候引那一行」「即值 − 1」「先问它是哪个参数的函数，再写进正文」是括注与次序半句。攻方 `rules-compress-residents-r1-opus-output.md:105`、`:106` 把后两处之一判成「同一节上一条已写出结局后引判定那一行」「是说明不是判据」，这是最强辩词：第一版 `rules/evidence-discipline.md:53` 已有「结局出来、读过判定的那一行，再改成结论并引那一行」。撑不住的地方：被取消的那一次运行引哪一行（调度它的一方给的最终判定），第 53 行的「判定的那一行」指不到；放回不改做法、不引新词。判「站得住」，严重度低。 |
| 5 并行交错那一句被处置成「规则里没有对应的节」 | 站得住 | 第一版 `rules/fs-design.md:112`「怎么证明它的判定分得出差别、怎么把写下的模型绑到代码上，规则里还没有对应的节，记在欠账表」；原件 `.claude/rules/implementation-workflow.md:77`「`litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符」；`.claude/scripts/lkmm.sh:34` 与 `:40` 是「会失败的检查」第 3、4 条（每条 Never 自己的对照组、每条 Never 绑到代码）。处置当时的依据对（原件 `exhaustive-interleaving.md:16` 指到的 `verification.md` 里 `grep -c 对照组` 是 0），错在从「那个文件里没有」推到「规则里没有」。（腿：攻方 五-1、正推 丢-6。） |
| 6 检出工具可自行终止那一支被删而标成只改措辞 | 站得住 | 原件 `.claude/rules/command-safety-by-review.md:22`「报告的一方（人、agent 或检出的工具）可以终止它」；第一版 `rules/command-safety-by-review.md:17`「报告的人或 agent 可以终止它」。辩词：删得与 `.claude/main-agent.md:76`「看门狗只报、不停子 agent 也不停进程」和 `scripts/agents/agent-watch.py:22`「它看、它报，不停子 agent、不停进程」一致，删的是空话。这条辩词是删的理由，不是「只改措辞」的理由：标签错了是事实。用户定案第 21 条定「不放回」，与判决一致。 |
| 7 指到上游节名的指针：0 命中，改成指到文件并把判据句留在项目一侧 | 站得住，但判决只写了一处；修补改了四十三处，范围比判决大（第三节） | `grep -c '射程与例外' .claude/singlefs-ai-sop/rules/communication-discipline.md` 输出 0；上游标题是 `:2 沟通纪律`、`:9 不写客套`、`:15 转述别处的做法，带出处或原文`、`:27`、`:34`、`:42`；话在开头段 `:5`「写给第三方读的产出（邮件、公告、对外文档）按它的读者写，不在射程里。」。第一版 `rules/writing-discipline-by-review.md:56`「射程的例外照它的「射程与例外」」落空：那个节名是原件本地 `.claude/rules/communication-discipline-by-review.md:14` 的标题，套到了上游头上。 |

## 二、第二版修补够不够得着（逐条）

| 判决条目 | 修补落在哪 | 够得着吗；有没有把对的改错 | 证据 |
|---|---|---|---|
| 1 | 第二版 `rules/fs-design.md:112`；`skills/crash-test/commit-stages.md:24`；`skills/crash-test/SKILL.md:18` | 够得着，没改错。三处都改指参考件里真有那句话的节：`fs-design.md:112`「`.claude/rules/verification-reference.md`「各道的判定单位」」（该节 `:38` 起，`:42` 有「lkmm 每份 litmus」）；`commit-stages.md:24`「照 `.claude/rules/verification-reference.md`「门禁 crash-point-replay 怎么跑」与门禁文件头…」（该节 `:14` 起，`:18`、`:21` 有九格核账与三处领卡）；`SKILL.md:18`「各格判什么在 `.claude/rules/verification-reference.md`「门禁管哪一半」」（`:140` 起，`:146` 是崩溃点重放那一行）。这一组文件里别的指到那一节的（`session-wrapup.md:37`、`crash-test/SKILL.md:12`、`mutation-sampling.md:69`、`test-script-discipline.md:26`）要的是「提交之外」那一条或跑法表，第二版 `verification.md:29` 起的那一节里有（`:35` 表、`:38`「提交之外」）。 |
| 2 | 第二版 `rules/show-me-test-by-review.md:7`；`rules/implementation-workflow.md:9` | 够得着；留一处窄的新二义（不是改错）。两处现写「（`crates/` 的测试进 `crates/mutations.tsv`，实验装置的进 `scripts/mutation/tables/` 下它那一张表）」。仓里「实验装置」有两支：`scripts/experiment-apparatus/` 那一支确是 `scripts/mutation/tables/` 下的表（`code-source-discipline.sh:1271`），入库装置（`crates/singlefs-verification/src/bin/`）的变异行进 `crates/mutations.tsv`（`.claude/agents/experiment-runner.md:35`）。「实验装置」不加限定会被读成包括入库装置，与 `crates/` 的测试那一半重叠。建议写成「`scripts/experiment-apparatus/` 下装置的进…」。推翻它的现象：若门禁把入库装置也收进 `scripts/mutation/tables/`（`grep -n 'singlefs-verification/src/bin' .claude/gate.d/code-source-discipline.sh` 能找到那一条），重叠就不存在。 |
| 3 | 第二版 `rules/code-discipline-by-review.md:4` | 够得着，没改错：现为「…同射程，只写上游没写的。」，「两处说同一件事以上游为准」已删，开头那半句留着的是说明（「`code-discipline-by-review.md` 里叫「上游」」是给本文件后面各处「上游」下的定义）。 |
| 4 | 第二版 `rules/session-wrapup.md:37`、`rules/evidence-discipline.md:55`、`rules/format-evolution.md:48`、`rules/evidence-discipline.md:98` | 够得着，四处都是原话，没改错。原件对第二版逐字：(a) `.claude/rules/session-wrapup.md:55`「**等改动停下再跑，或者用 `--staged`。**」→ 第二版 `:37`「对不上就判红；等改动停下再跑，或者用 `--staged`。」；(b) `.claude/rules/verify-before-claiming.md:48`「结局以调度它的那一方给出的最终判定为准，引的时候引那一行。」→ `evidence-discipline.md:55` 同句逐字；(c) `.claude/rules/format-evolution.md:62`「下标（`[147]`，即值 − 1）」→ `:48` 同；(d) `.claude/rules/evidence-discipline.md:91`「…每个都扫过；先问它是哪个参数的函数，再写进正文」→ `:98`「先问它是哪个参数的函数，再写进正文：说得出这条臂的代价与收益依赖装置里哪几个旋钮、每个都扫过」（句序前后对调、「代价 / 收益」写成「代价与收益」，意思没变）。(a) 里同时加了「分不清是谁的红、确要跑整轮门禁的，先照 `verification.md`「一个门禁：两种跑法」「提交之外」那一条弹窗问用户，同意了才跑」：这是用户定案第 18 条，不是放回的原话；`scripts/gate/gate-staged.sh:1-4`（文件头「跑 `gate.sh --staged`」）与它写的「经服务交」对得上。 |
| 5 | 第二版 `rules/fs-design.md:112` | 够得着，没改错。写成：怎么证明分得出差别、怎么绑到代码「每条 Never 有自己的去屏障对照组、绑到代码的锚点，`.claude/rules/verification-reference.md`「门禁管哪一半」lkmm 那一行与 `.claude/scripts/lkmm.sh` 文件头「会失败的检查」第 3、4 条」，「记在欠账表」已删（`grep -n '欠账表' …/rules/fs-design.md` 0 行）。落点现读：`verification-reference.md:157` 是「`litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符」那一行；`lkmm.sh:34`、`:40` 是第 3、4 条。 |
| 6 | 第二版 `contradictions.md:46` 的标签；规则文字不动 | 够得着：标签改成「改了判据（标签更正）」；规则文字 `rules/command-safety-by-review.md:18`「…报了之后调度的一方没有响应…报告的人或 agent 可以终止它」不放回「检出的工具」，与用户定案第 21 条一致。没改错。 |
| 7 | 第二版 `rules/writing-discipline-by-review.md:56`（判据句留项目一侧）；另 43 处换括号（第三节） | 够得着，没改错。`:56`「…有钩子判的几条在 `.claude/singlefs-ai-sop/rules/communication-discipline.md`；写给第三方读的产出（邮件、公告、对外文档）按它的读者写，不在射程里。」与上游 `:5` 一字不差；原件那一节的第二条「精确不让位于简短…」第二版 `:65` 还在。 |

没有「照改反而把对的改错」的一处。顺带核了两处第二版新写的、判决没点名的句子：
- `skills/crash-test/commit-stages.md:23`「命令都不带 `SINGLEFS_HEAVY_TESTS` 前缀（「跑之前定下来」一节）」：原件 `.claude/skills/crash-test/commit-stages.md:17` 写「命令都带「跑之前定下来」一节定的那个前缀」，那一节 `:9` 写的是「不带 `SINGLEFS_HEAVY_TESTS` 前缀」，原件自相矛盾；今天的服务把它列成退役变量（`task-server/task_server_lib.py:370`「退役的名字（RETIRED_VARIABLES：SINGLEFS_HEAVY_TESTS、…）：交给服务的命令不带它们」）。第二版改成「不带」是对的。同一类写法还剩一处没动：`skills/crash-test/SKILL.md:35`「谁在什么时候跑、带什么前缀」（原件 `.claude/skills/crash-test/SKILL.md:40` 同形，第一版、第二版都留着）；那里的「前缀」读成范围声明（写在 `task submit` 前面）也说得通，不算新问题，记一笔。
- `rules/command-safety-by-review.md:6` 到 `:10`（样本进程那一节，用户定案第 20 条恢复）：标题与第一句与原件 `.claude/rules/command-safety-by-review.md:4`、`:6` 除去原件的加粗外逐字相同；新加的「`setsid --wait` 起（不带 `-f`），同一层紧跟着 `名字=$!`」「`kill -TERM -- -"$名字"`」「组号只取自己用新会话起的那一个子进程的」对着钩子 `.claude/hooks/bash-command-detector.sh` 文件头 ⑥ 的放行行（第 185 行：「shell 里同一层先 `setsid [--wait] <命令> &`（不带 -f）、紧跟着 `名字=$!`，之后 `kill -<信号> -- -"$名字"`」）一致。规则写 `--wait`、钩子许 `--wait` 可带可不带，规则取了严的那一边，与共用约束 `agent-common.md` 同。

## 三、判决第 7 条：修补把指到上游的指针改了写法，逐处核

做法：第二版（`research/prompts/rules-compress-r2-merged/dot-claude/` 下全部 `.md`）里凡「`.claude/singlefs-ai-sop/rules/<文件>.md`」后紧跟「节名」或（节名）的，逐处在上游今天的标题行里找（`ptr.py`，输出 `r2-ptr.txt`；第一版同样扫一遍，`r1-ptr.txt`）。量的：第一版是 45 处「节名」写法（其中 2 处在总表 `contradictions.md:18`、`:20`，引的是写手原话，不算）；第二版 47 处（节名）写法。差：第一版 43 处换成了括号（`diff-from-r1.patch` 里 33 行是纯换括号，`pairs.py` 核的），另 2 处是第二版新写的指针（`rules/implementation-workflow.md:23`、`rules/test-script-discipline.md:4`），2 处不是指针（`contradictions.md:79` 引原话、`rules/code-discipline-by-review.md:4` 的括注）。逐处结果：45 处指针里 45 处的节名逐字等于上游某个标题行（上游标题现读：`grep -n "^#" .claude/singlefs-ai-sop/rules/<文件>.md`），没有一处指不到。

| 第二版文件:行 | 上游文件 | 节标题（括号里的） | 上游今天 |
|---|---|---|---|
| agent-common.md:20 | command-safety.md | 退不回去的操作，动手前先想一遍；`pkill -f` / `killall` 一律禁用 | 各段逐字命中 |
| agent-common.md:20 | writing-discipline.md | 文风要简单自然 | 逐字命中 |
| agent-common.md:42 | kb-discipline.md | 5. 编号只能做索引，不能做称呼 | 逐字命中 |
| skills/crash-test/SKILL.md:25 | show-me-test.md | 最终判据由项目定；门禁不许假装通过 | 各段逐字命中 |
| agents/clerk.md:16 | design-doc-discipline.md | 1. 正文只写当前现状，历史一律放到文末 | 逐字命中 |
| agents/experiment-runner.md:16 | command-safety.md | 测试镜像一律放临时目录 | 逐字命中 |
| agents/implementation-writer.md:16 | command-safety.md | 测试镜像一律放临时目录；子 agent 交回之前，删掉自己建的编译目录与仓副本 | 各段逐字命中 |
| agents/investigator.md:16 | command-safety.md | 子 agent 交回之前，删掉自己建的编译目录与仓副本 | 逐字命中 |
| agents/prior-art.md:16 | kb-discipline.md | 2. 每条带出处与状态 | 逐字命中 |
| agents/three-way-attack.md:17 | command-safety.md | 子 agent 交回之前，删掉自己建的编译目录与仓副本 | 逐字命中 |
| agents/tooling-writer.md:16 | sop-first.md | 每一条拒绝都必须给出下一步；加门禁或钩子之前，先找已有的；边界 | 各段逐字命中 |
| agents/tooling-writer.md:16 | command-safety.md | 并行不许把失败吃掉；子 shell 里的赋值传不回父进程；管道里的退出码不是你想要的那个；测试镜像一律放临时目录 | 各段逐字命中 |
| agents/tooling-writer.md:16 | show-me-test.md | 门禁不许假装通过 | 逐字命中 |
| agents/tooling-writer.md:16 | code-discipline.md | 注释、断言、常量、重复 | 逐字命中 |
| agents/tooling-writer.md:16 | rules-discipline.md | 1. 正文只写四样；3. 搬出来的东西直接删掉；4. 指向历史的链接要带劝阻句；5. 规则文件不留历史节；7. 正文不许用位置指代，也不许自称 | 各段逐字命中 |
| rules/changelog-format.md:6 | kb-discipline.md | 8. 正文只写现状，历史放文末的「历史版本」 | 逐字命中 |
| rules/code-discipline-by-review.md:87 | show-me-test.md | 最终判据由项目定 | 逐字命中 |
| rules/command-safety-by-review.md:29 | communication-discipline.md | 转述别处的做法，带出处或原文 | 逐字命中 |
| rules/evidence-discipline.md:32 | kb-discipline.md | 2. 每条带出处与状态 | 逐字命中 |
| rules/evidence-discipline.md:61 | kb-discipline.md | 2. 每条带出处与状态 | 逐字命中 |
| rules/evidence-discipline.md:81 | kb-discipline.md | 2. 每条带出处与状态 | 逐字命中 |
| rules/fs-design.md:87 | code-discipline.md | 类型：让非法状态写不出来 | 逐字命中 |
| rules/fs-design.md:113 | code-discipline.md | 分支：穷举写全，不写通配臂 | 逐字命中 |
| rules/implementation-workflow.md:23（新写） | show-me-test.md | 没有测试的 patch 一律不收 | 逐字命中 |
| rules/implementation-workflow.md:38 | show-me-test.md | 门禁不许假装通过 | 逐字命中 |
| rules/machine-first.md:13 | sop-first.md | 每一条拒绝都必须给出下一步 | 逐字命中 |
| rules/machine-first.md:95 | design-doc-discipline.md | 1. 正文只写当前现状，历史一律放到文末 | 逐字命中 |
| rules/machine-first.md:95 | kb-discipline.md | 8. 正文只写现状，历史放文末的「历史版本」 | 逐字命中 |
| rules/machine-first.md:106 | pushback-discipline.md | 警告写在哪：`.claude/warnings/<日期>.md` | 逐字命中 |
| rules/session-wrapup.md:15 | sop-first.md | 边界 | 逐字命中 |
| rules/session-wrapup.md:38 | kb-discipline.md | 8. 正文只写现状，历史放文末的「历史版本」 | 逐字命中 |
| rules/show-me-test-by-review.md:12 | sop-first.md | 推论 | 逐字命中 |
| rules/show-me-test-by-review.md:22 | code-discipline.md | 名字：光看名字，就说得出它是什么、做什么 | 逐字命中 |
| rules/show-me-test-by-review.md:52 | show-me-test.md | 最终判据由项目定 | 逐字命中 |
| rules/test-script-discipline.md:4（新写） | show-me-test.md | 门禁不许假装通过；每一个弄坏开关都必须会红 | 各段逐字命中 |
| rules/test-script-discipline.md:11 | command-safety.md | 测试镜像一律放临时目录 | 逐字命中 |
| rules/test-script-discipline.md:13 | code-discipline.md | 名字：光看名字，就说得出它是什么、做什么 | 逐字命中 |
| rules/test-script-discipline.md:30 | show-me-test.md | 门禁不许假装通过 | 逐字命中 |
| rules/test-script-discipline.md:31 | command-safety.md | 并行不许把失败吃掉 | 逐字命中 |
| rules/test-script-discipline.md:40 | show-me-test.md | 每一个弄坏开关都必须会红 | 逐字命中 |
| rules/test-script-discipline.md:53 | show-me-test.md | 门禁不许假装通过 | 逐字命中 |
| rules/writing-discipline-by-review.md:12 | rules-discipline.md | 5. 规则文件不留历史节 | 逐字命中 |
| rules/writing-discipline-by-review.md:22 | writing-discipline.md | 四类不许写 | 逐字命中 |
| rules/writing-discipline-by-review.md:26 | sop-first.md | 每一条拒绝都必须给出下一步 | 逐字命中 |
| rules/writing-discipline-by-review.md:65 | writing-discipline.md | 别矫枉过正 | 逐字命中 |

对这一组改动的复核：

- 够得着：判决第 7 条原来点的那一处（`射程与例外`）第二版改对了（第二节第 7 行）。另外 43 处换括号，是第二版的修稿方在总表 `contradictions.md:104`（合稿-12）写「上游正在自己压缩，节名会变（用户定案）」后做的；它出处是调度记录 `records/2026-11-16-三方豁免取消与存量补轮.md:86` 的「项目这一侧与上游重复的条款…项目侧删成指针，指针按上游此刻的节标题写，上游压完同步副本时再核一遍这些指针」。这一句说的是「按此刻的标题写、压完再核」，没说改成括号；换括号是修稿方自己的落法，不是判决第 7 条要求的（判决只写了 `射程与例外` 一处 0 命中）。
- 没改错：43 处换括号里 33 行是纯换括号（`pairs.py`：把第二版的「文件`（节）`」换回「文件`「节」`」，与第一版逐字相同），没有改动被指的节，也没加减判据；括号里的字全等于上游今天的标题。所以「改掉的那些在上游今天还指不指得到」答：45 处全指得到。
- 留着的三处小问题（都不是丢了判据，是同一类写法没收齐）：
  1. 同一类指针还剩 8 处停在「上游「节」」写法：第二版 `rules/code-discipline-by-review.md:37`、`:48`、`:49`、`:53`、`:59`、`:64`、`:69`、`:73`（例：`:49`「上游「分支：穷举写全，不写通配臂」」），上游今天的标题都在（`.claude/singlefs-ai-sop/rules/code-discipline.md:11`「名字：…」、`:80`「分支：…」、`:113`「类型：让非法状态写不出来」、`:139`「注释、断言、常量、重复」；`:48`、`:69` 写的「类型」是 `:113` 标题的前半截，上游以「类型」起头的标题只有这一个）。总表合稿-12 写「全树 43 处」，没有这 8 处，上游压缩时它们同样会漂。
  2. `rules/show-me-test-by-review.md:12`「照 `.claude/singlefs-ai-sop/rules/sop-first.md`（推论）与「边界」」：一句里一半换了括号、一半没换；括号里的「推论」读起来像注释，不像节名。
  3. `rules/show-me-test-by-review.md:52`「…照 `.claude/singlefs-ai-sop/rules/show-me-test.md`（最终判据由项目定）（这一轮跑过且通过）」：两个括号挨着，前一个是节名、后一个是原件 `.claude/rules/show-me-test-by-review.md:80` 的限定语，读得出来但两个括号性质不同。
- 第二版共用约束 `agent-common.md:17` 写「步骤里写「照 <规则>「<节>」」的，做那一步之前读那一节；只写「照 <规则>」没写节的…读那一份」：括号写法不在这两种形态里。读成「没写节」会多读整份上游文件，读成「写了节」读那一节，两种读法的结局都不是做错，只是读多读少；而定义里的这几处（`agents/*.md` 的 `:16` 或 `:17`，都是「开工先读：」行）与 `agent-common.md:20`（「每个定义都照守」那一条）属于 `:17` 前半说的「整份读」，按节读与整份读的差在那里不起作用；只剩 `agent-common.md:42`（写进仓里的注释引 D、E 编号）一处是平常的引用，读那一节或读整份都不会做错。记一笔，不算打中。
- 判决第 7 条「判据句留在项目一侧」：只有 `射程与例外` 一处真补了判据句。其余 43 处没补句子，被指的内容全靠上游；它们上游今天都有同名标题，所以没有此刻的损失，上游压缩之后才会显出来（总表合稿-12 写「在项目这一侧留判据句」，对不上它实际做的）。这条是对修稿方说明的订正，不是对判决的订正。

## 四、`dispositions.md` 里这一题写「不成立」「部分」「未复核」的行（第三节「常驻规则题」与相关行）

第二版 `research/prompts/rules-compress-r2-merged/dispositions.md` 里这一题落在不采纳一侧的行有：`:72`（不成立）、`:73`（部分成立）、`:74`（未复核）、`:75`（部分成立），另 `:63`（不放回，交用户）、`:117`、`:119`（部分，涉及 fs-design 与执行员例外）。逐行替被判不采纳的腿辩一次：

| 行 | 被不采纳的说法 | 辩方判定 | 证据 |
|---|---|---|---|
| `:72` 正推 格五 另列 3 | 射程多了「自证」一词 | 不采纳的理由站得住 | 第二版 `rules/command-safety-by-review.md:6`「装置、判法与复跑脚本起样本进程」与原件 `.claude/rules/command-safety-by-review.md:6` 同；节标题 `:4` 也恢复成原件的「装置起的样本进程，放进它自己的进程组」。 |
| `:73` 攻方 没算打中 | 总表 9 行「只改措辞」只核了常驻-11、12，标签可能不准 | 理由站不住（至少两行）：「别的 7 行没动」没有核 | 修稿方只重核了常驻-7、常驻-11、共用-10（腿点到的），改了标签；`research/prompts/rules-compress-r2-merged/revise-result.md:53` 自己写「别的 6 行没动」。我逐行对原件核了常驻-9、-10、-12、-14、-17、-18（总表 `contradictions.md:44`、`:45`、`:47`、`:49`、`:52`、`:53`）：-17、-18 确是只改措辞；-9、-10 是对着钩子文件头把原件不准的写法校正（钩子今天的判法没变，标签可商量）；-15 在共用约束与三方规则之间，不在这一组文件里，没核；**-12 与 -14 其实动了做法**：-12 原件 `.claude/rules/command-safety-by-review.md:30`「长活放后台，定期回来复检，不在前台干等」→ 第二版 `rules/command-safety-by-review.md:17`「主 agent 这边由看门狗复检、任务卡住由服务判，自己交的任务结束本轮等完成通知，不在前台干等、不轮询」，定期回来看没了、改成不轮询（对齐 `agent-common.md` 的「不写轮询」，向共用约束收，但规则句的做法变了）；-14 原件 `:41`「不归的，报给用户或调度的一方」→ 第二版 `rules/command-safety-by-review.md:31` 加「主 agent 自己就是调度的一方，照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 15 条」，而第 15 条（`dot-claude/main-agent.md:38`）是「没有会话认领、或对应的会话已经不在，自己做」，主 agent 一侧从「报给用户」变成「自己做」。两行标「只改措辞」与常驻-11 犯的是同一类标签错（判决第 6 条）。结论：该行理由不成立，应把 -12、-14 的标签改成「改了判据」，并让别的 5 行也核过再说「标签没错」；规则文字不用改（两处都是向更高一层的规定靠拢）。 |
| `:74` 攻方 没算打中 | 常驻-6「门禁是下限」上游 0 命中，处置依据是一次搜索 | 不采纳的理由站得住（现在是多次搜索） | `grep -rn '下限' .claude/singlefs-ai-sop/rules/*.md` 0 行；上游 `.claude/singlefs-ai-sop/rules/sop-first.md:7` 起「## 推论」四条里没有「门禁是下限」（原件 `.claude/rules/show-me-test-by-review.md:77`「**门禁是下限，不是上限**（`.claude/singlefs-ai-sop/rules/sop-first.md`）」指到的那份今天没有这一句；上游 `CHANGELOG.md:603` 写它曾在推论第 3 条）。第二版 `rules/show-me-test-by-review.md:51`「**门禁是下限，不是上限。**」留判据、删指针，对。 |
| `:75` 正推 节标题变动 21 个 | 按旧节名引的地方要跟着改 | 不采纳的理由站得住 | 对没了的 20 个标题（装置起的样本进程那一节第二版恢复，不算）逐个 `git grep -F "「<标题>"`，范围 `.` 去掉 `research`、`records`、`.claude/rules/*`、`.claude/skills/*`：除「判法」「落实」「长度要配得上改动的重量」这类到处是的普通词外全 0 行；「长度要配得上改动的重量」那一处是 `.claude/kb/milestone/03-third-txn.md:229` 的上游节名，不是被删的那一节。按旧文件名引的活文件（`.claude/gate.d/doc-text.sh:2914`、`.claude/hooks/ask-user-claim-guard.sh:42` 等）总表 `references.md:11`、`:12` 已登。 |
| `:63` 攻方 五-2 | 常驻-11 规则文字不放回，是否放回交用户 | 站得住，且已有定案 | 用户定案第 21 条（`records/2026-11-16-三方豁免取消与存量补轮.md:199`）：「照修稿，不放回「检出的工具」」。 |
| `:117`、`:119` | 欠账号没登、执行员「卡住」只写吃紧 | 不在这一组文件里，没复核 | `fs-design.md:112` 那半已删「记在欠账表」（第二节第 5 行）；执行员一条在 `agent-common.md`，归定义那一题。 |

另有一行没有写进 `dispositions.md` 的不采纳：`research/prompts/rules-compress-r2-merged/revise-result.md:23`「不成立的 11 条」里数了「常驻·攻 一条（技能 light / heavy 两档，总表已登）」，但 `dispositions.md` 常驻题那 18 行里没有这一行（全文 `grep -n 'light' dispositions.md` 0 行；`dispositions.md` 里写「不成立」的行总共 8 行，对不上「11」）。这一条站得住：`.claude/hooks/selftest-registry.tsv:8`「所有自证与样本档都盯，没有轻重之分」，第二版 `skills/gate/SKILL.md:12` 已写成「子 agent 不交」，总表 `contradictions.md:89`（按需-M14）登为「改了判据」。

## 五、核查方式的缺口：判决写「没有另跑 snapshot-tree.py、改成逐处现读」

判决 `rules-compress-residents-r1-main-verification.md:7` 写：腿的引文各自跑过 `cite-check.py` 全对上；主 agent 没跑 `snapshot-tree.py`（任务服务排着 91 条），改成对腿点名的行逐处用 grep 与 sed 现读，现读结果记在调度记录「主 agent 现查坐实的」一段。这样核，够的地方与缺的地方：

| 核什么 | 够不够 | 缺在哪 / 我补核的结果 |
|---|---|---|
| 引文在不在被引文件里（cite-check） | 够，但判决里写的是腿跑的、主 agent 没在一棵倒推的树上再跑一遍 | `snapshot-tree.py` 的作用是倒推开工那一刻的树、核 sha256。这一轮 35 份原件在腿跑完之后 sha256 全没变（本报告开头：`judged-sha256.txt` 在仓根核，非 OK 0 行；第一版合稿树按快照里的清单核，0 份不对）。所以直接在仓里跑 `cite-check.py`，等价于在倒推的树上跑：两份腿报告 2175 处引文全对上，没判 2 处裸行号（读了，对得上）。没覆盖的：腿引到的、不在 `judged-sha256.txt` 与合稿树清单里的文件（上游 `.claude/singlefs-ai-sop/rules/`、脚本与钩子文件头、`.claude/kb/`、`records/`）没有开工时的 sha256，无从证明「腿读到的就是今天读到的」；本报告读的是今天的，判决第 7 条也写「此刻」，所以这一缺口只影响「腿当时读到什么」，不影响今天成不成立。 |
| 判决第 1、2、3、7 条的现查 | 够 | 调度记录「主 agent 现查坐实的」`records/2026-11-16-三方豁免取消与存量补轮.md:177` 起：指针落空（lkmm、litmus、领卡、核账 0 命中）、`射程与例外` 上游 0 命中、`show-me-test-by-review.md:7` 写死 `crates/mutations.tsv`、`code-discipline-by-review.md:4` 原件 0 命中，四件都在；我复核都对（第一节）。 |
| 判决第 4 条的四处半句 | 记录里只现查了一处（「等改动停下」） | 「引的时候引那一行」「即值 − 1」「先问它是哪个参数的函数」三处没在「现查坐实」那一段里。我对着原件逐字核了（第二节第 4 行），都在、都是原话。 |
| 判决第 5 条（并行交错指对地方）、第 6 条（检出工具可终止） | 记录里没有现查记录 | 判决写的「攻方指出…主 agent 的处置错了」「标签更正」来自腿报告。我现读了原件 `implementation-workflow.md:77`、`command-safety-by-review.md:22`、`lkmm.sh:34`、`:40`，都对（第一节第 5、6 行）。 |
| 判决「其余…共同进了逐条去向表（129 条：成立 110、不成立 11、部分 8）」 | 数对不上，主 agent「读了其中不采纳的 14 行」也无从对照 | `dispositions.md` 全文写「不成立」的行 8 行，不是 11（`grep` 出的行：`:20`、`:21`、`:22`、`:51`、`:72`、`:91`、`:98`、`:99`）；「常驻·攻 技能 light / heavy 两档」那一条在 `revise-result.md:23` 数了、表里没有行。表是 94 行（把两条腿说同一件事的并成一行）。判决里「读了 14 行，理由是文件头或说明句」对这一题的行不适用：`dispositions.md` `:72` 的理由是用户定案第 20 条、`:73` 的理由是「别的 7 行没动」（站不住，第四节）、`:74`、`:75` 是一次搜索与 `git grep`，都不是「脚本文件头」「说明句」。 |
| 修稿方说「全树指针 330 处全指得到」 | 我重核了一遍，同意 | `res.py` 对第二版 `.md`（不含总表、引用清单、合稿说明）里 `` `路径`「节」 `` 的指针：283 处，节标题逐字或前缀命中，另 24 处只在正文（加粗条目名或表格行名，如 `agent-common.md`「子 agent 不交任务」），文件找不到 0、节找不到 0（输出 `res-out.txt`）。 |

## 六、旁记（不在四件事里，主 agent 取用）

- 第二版是用户定案第 23 到 26 条之前冻结的。第 24 条「带 `--full-gate` 或 `--whole-package-confirmed` 的命令由钩子弹的那一次就算确认，主 agent 不另问第二次」与第二版这一组里写的「先弹窗问用户，同意了才跑」是同一件事的两个说法，落点：第二版 `rules/session-wrapup.md:37`、`rules/verification.md:38`、`skills/crash-test/SKILL.md:12`（原写在第二版修稿报告 `revise-result.md:` 「④ 拿不准」第 1 条，修稿方也没核是不是重复）。第三轮要改的是这几处，攻方再看。
- 判决第 5 条对应的 `references.md`「登成脚本那一批要做的事」我没读。

## 要主 agent 跑的命令

没有。这一轮没写模型；上面的数都是只读命令的结果，主 agent 要复核可以在仓根直接跑：

- `sha256sum -c research/prompts/rules-compress-residents-r2-snapshot/judged-sha256.txt | grep -vc ': OK'`，输出 0。
- `grep -c '射程与例外' .claude/singlefs-ai-sop/rules/communication-discipline.md`，输出 0。
- `awk 'NR>=26 && NR<=92' research/prompts/rules-compress-r1-merged/dot-claude/rules/verification.md | grep -cE 'lkmm|litmus|领卡|核账|九格'`，输出 0。
- `grep -rn '下限' .claude/singlefs-ai-sop/rules/*.md | wc -l`，输出 0。
- `python3 -I ~/.local/share/singlefs/work/rules-compress-residents-r2-sonnet/ptr.py research/prompts/rules-compress-r2-merged .claude/singlefs-ai-sop/rules <输出文件>`：输出第一行 49，文件里 `P` 开头的行里 `exact` 之外只有 `contradictions.md:79` 与 `code-discipline-by-review.md:4` 两行（不是指针）。
- `python3 -I ~/.local/share/singlefs/work/rules-compress-residents-r2-sonnet/pairs.py research/prompts/rules-compress-r2-merged/diff-from-r1.patch`：第一行 `pure bracket swaps (lines): 33`。

## 什么现象会推翻这份报告

- 第一节第 1 条：第一版 `verification.md` 第 26 到 47 行里有 lkmm、litmus、领卡、核账中任何一个的行（`awk … | grep` 命中）。第 2 条：`scripts/mutation/tables/` 与 `crates/mutations.tsv` 之外还有第三张表也收新测试的证红，或门禁把入库装置的变异行也放进 `scripts/mutation/tables/`。第 3 条：原件里能找到同义的「以上游为准」。第 6 条：`agent-watch.py` 或某个钩子里有一处检出工具自己终止进程的代码。第 7 条：上游 `communication-discipline.md` 里有叫「射程与例外」的标题。
- 第二节：入库装置不在 `crates/singlefs-verification/src/bin/`，或 `experiment-runner.md:35` 那一句写错；那样「实验装置」写法没有重叠。
- 第三节：上游标题在这之后又被压缩改名，45 处里的某些就指不到了（这是总表合稿-12 说的风险，不是本报告漏核）。
- 第四节 `:73`：若 -12、-14 的原件句子在今天的规则里并不是「做法」而是描述（比如「定期回来复检」只是对看门狗的描述、与「不轮询」一致），那标签没错；我对这两行读成做法，把握中等。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没有要复跑的（没有模型）。
- 没读禁读清单里的文件：攻方腿的报告与模型目录、草稿目录，别的几题第二轮各腿的产出与草稿目录（调度记录里主 agent 写的一句话攻方交回数我读到过一行，没用）。
- 没核第二版里不在这一组文件里的改动（入口、验证、定义、总表其余行）；没核 `dispositions.md` 里不属于这一题的行。
- 没碰仓里文件，没建仓副本与编译目录；草稿目录 `~/.local/share/singlefs/work/rules-compress-residents-r2-sonnet/` 里只有三份脚本、几份输出，共约 50K，不留给别的趟用，交回前我不删（主 agent 看完要复核再删）。

脚本与输出的 sha256：
- `ptr.py` ec42f38a442164412cd9a68e3eacb44b13f268c26520167a4ecf580421f62dbe
- `pairs.py` 56e441f3ff9badd13d64dc0a6fe7f41222190f4bef3027cec3af5ba2cf00079f
- `res.py` 69e044b654b45c1c8364164ba64b21670677e94d3a1b5482cd80bd9f45b67b80
- `r1-ptr.txt` ffa0f379d08514da70632fa97fbf3b70584096facf1523e54d5ce4582bff52f1
- `r2-ptr.txt` 8650a419379a337956010e27bba8048fa2f4f0d81b445c6d3c7db77f582976a2
- `res-out.txt` 0b7111cd7a4156cf8c3b012fecbe7f4489e0b09143fc6f4525ab6d638cc3f69d
- `cc-r1.txt` efff99d02093e4c9a9e908bcbf0a47c4841c9df934b6598d7174ca11b3a455d8
