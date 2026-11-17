# rules-compress-residents-r2 攻方腿报告

日期：2026-11-16。立场：假设「第二版合稿树里这一组规则与技能相对原件一条没丢、没有二义、没有悄悄改做法，第一轮的修补都改对了」是错的，找反例。只读第一轮两份腿报告与判决，禁读清单里的一份没读。

没写模型、没有要主 agent 复跑的模型数。草稿目录 `~/.local/share/singlefs/work/rules-compress-residents-r2-opus/` 里只有一份只读核对脚本 `paren-pointer-check.py`（sha256 `3fbf36dc132ad4c9ae1fb3126f7c758ac87f12377eff7d9c0d49a4b52bfc0a8b`），只读文件、不起项目代码。开工核过：`judged-sha256.txt` 35 份、`merged-tree-sha256.txt` 全部对上（`sha256sum -c` 没有一行不是 OK，量的）。

合稿树简写「第二版」= `research/prompts/rules-compress-r2-merged/dot-claude/`，引文照写全路径。行号都是 `grep -nF` 现取的。

## 各格判定一览

| 格 | 判定 | 打中几处 | 最重的一处 |
|---|---|---|---|
| 一 修补对不对 | 打中 | 2 | 一-1：43 处指上游「节」的指针全改成括号写法，只有 1 处是判决点名要改的，其余今天都指得到，改完它们出了门禁 governance-refs ③ 的射程 |
| 二 修补引出的新问题 | 打中 | 3 | 二-2：用户定案第 18 条只落到 verification.md 与它的几处指针，门禁技能、提交技能三处仍写「提交时（或用户要求时）跑」，没指过去 |
| 三 第一轮没看到的 | 18 份逐句对过；只找到 1 处小的（原件带来的残句） | 1 | 三-1：staged-gate.md 第 17 行的残句 `=user-request` |
| 四 不采纳的那几条 | 不归攻方（正文分工表），没攻 | — | — |

打中的都只抽了一次样，照 `.claude/rules/three-way-inference.md@多轮：一次打穿不算数，三轮里多数打穿才算` 先挂起，由主 agent 现查。

## 格一：修补本身

### 一-1 「上游节名指针改成指到文件」做过了头：43 处里只有 1 处该改，改完全部躲开了会红的那道检查

- 判决要改的只是指不到的那一类：`research/prompts/rules-compress-residents-r1-main-verification.md:22`：「上游副本此刻 grep 那个节名 0 命中，改成指到文件并把判据句留在项目一侧」。第一轮打中的指不到的只有「射程与例外」一处（第二版已改成本地一句，见 `diff-from-r1.patch` 第 1012 行，改得对）。
- 修稿把全树 43 处都改了，登记在矛盾总表：`research/prompts/rules-compress-r2-merged/contradictions.md:104`：「开工先读行里的节名改成括号里的要点，不再是会被核的节标题」。量的：patch 里删掉的 `上游文件「节」` 写法 43 处、加上的括号写法 47 处；第二版全树此刻 `上游文件「节」` 0 处、括号写法 46 处（命令见「要主 agent 跑的命令」第 1 条）。
- 这 46 处括号里的节名，今天在上游全都指得到：`paren-pointer-check.py` 逐段核括号里的字与上游那份文件的标题行是否逐字相等，输出 `paren-pointers segments=59 heading-exact=58 not-heading=1`，唯一的 1 段是 `code-discipline-by-review.md:4` 那句「里叫上游」的说明，不是指针（量的）。所以除「射程与例外」外没有一处是判决要改的。
- 这样改丢掉的是检查：门禁 doc-registries 的 governance-refs 格 ③ 只认反引号文件名后紧跟「」的写法，`.claude/gate.d/lib-governance-refs.py:24`：「在那份文件的全文里一处都找不到」，匹配式在 `.claude/gate.d/lib-governance-refs.py:178`：「FILE_WITH_SECTIONS = re.compile」，判的循环从 `.claude/gate.d/lib-governance-refs.py:664`：「for match in FILE_WITH_SECTIONS.finditer(line):」起；上游副本虽被 `.gitignore` 挡着，这一支用 `os.path.isfile` 判、照判（读代码推的，没跑门禁）。括号写法匹配不上，这 46 处从此没有任何检查盯。
- 与主 agent 记下的做法相反：`records/2026-11-16-三方豁免取消与存量补轮.md:86`：「指针按上游此刻的节标题写，上游压完同步副本时再核一遍这些指针」。「再核一遍」靠的正是 ③；矛盾总表那一行把依据写成「用户定案」，而定案原话只说上游在自己压缩，没说改写法。
- 照第二版做会怎样：上游压完、同步副本之后节名变了，46 处括号里的旧节名不红、没人发现；读的人照括号里的字去上游找，找不到就要整份读。判据句多数留在项目一侧（逐处看过 `rules/` 下这一组的 30 处），所以做法今天不变，坏的是将来那一步。
- 另有一处混写：`research/prompts/rules-compress-r2-merged/dot-claude/rules/show-me-test-by-review.md:12`：「形态与立的时机照」之后是 `sop-first.md`（推论）接着一个 「」 括起的「边界」：前一个改成括号、后一个没改，后一个也因为前面隔着「（推论）与」而匹配不上 ③，读起来分不清它是节名还是一个词。
- 改法（只在我读过的文字上推的、被攻过零轮）：今天指得到的 45 处改回 `文件「节」`，只有「射程与例外」那一处照第二版留本地句；矛盾总表合稿-12 的射程缩成那一处。
- 四句：不涉及臂；系统（门禁 ③）当时看得到写法，判别它的就是写法本身；满足格一「与文件头不符」不成立、满足的是「放回的话…做法因此不同」那一句里的做法——检查做法；没有跑前条款给的改法。

### 一-2 变异清单只补了实验装置一张表，脚本、钩子、门禁的自证仍没有落点，原件「有 harness 才要」的条件丢了

- 原件把变异清单写成有条件的：`.claude/rules/show-me-test-by-review.md:12`：「有变异 harness 的仓要再进一步」，下一行给出没有 harness 时怎么办：`.claude/rules/show-me-test-by-review.md:13`：「commit message 里的叙述是没有 harness 时的退路，不是首选」。
- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/show-me-test-by-review.md:7`：「两样都要：把…做成存进仓里的变异清单交给门禁反复复跑（`crates/` 的测试进 `crates/mutations.tsv`，实验装置的进 `scripts/mutation/tables/` 下它那一张表）」。括号只列两类；`scripts/mutation/tables/` 是一个实验一张：`scripts/mutation/README.md:3`：「变异表在 `tables/`（一个实验一张，`e<编号>_<简称>.tsv`）」，去掉 `e<数>` 开头的只剩 `README.md`（`ls | grep -v '^e[0-9]'` 量的）。
- 第一轮这一条的打中（攻方二-1）点名过「实验装置的测试、脚本与钩子的自证也在内」，修稿只补了实验装置。脚本、钩子、门禁的自证仍被这一节管：第二版测试脚本规则写 `research/prompts/rules-compress-r2-merged/dot-claude/rules/test-script-discipline.md:40`：「变异进入库的变异清单」，后面点名的就是 show-me-test-by-review 这一节。
- 两种读法：甲，「两样都要」管所有新测试，钩子与门禁脚本的自证也要一行入库变异，可仓里没有收它们的表，做不到；乙，括号是射程，只有 `crates/` 与实验装置要变异行，其余只剩 commit message。原件按「有没有 harness」分开说清了，第二版把条件删了、射程又只写了两类。
- 照第二版做会怎样：写门禁或钩子自证的工具员，照甲要找一张不存在的表，照乙不写变异行；两种做法不同。
- 改法（零轮）：在括号后补一句「没有变异表收的测试（钩子、门禁与 `scripts/` 下非装置脚本的自证）照弄坏开关证红，commit message 写明」，或把原件「有变异 harness 的」放回句首。
- 四句：不涉及臂；读的人看得到两种读法、分不出；满足格一「放回的话与原件不同而做法因此不同」；没有跑前条款的改法。

## 格二：修补引出的新问题

### 二-1 「新测试先证明会红」的判据修完住进了两处，两处都写全

- 原件实现流程那一行只写一句：`.claude/rules/implementation-workflow.md:9`：「会不会红写在 commit message 里」，判据全文在 show-me-test-by-review。
- 第二版把同一句判据（两样都要、两张表各收哪一类）在两处各写一遍：`research/prompts/rules-compress-r2-merged/dot-claude/rules/implementation-workflow.md:9`：「实验装置的进 `scripts/mutation/tables/` 下它那一张表）与 commit message，两样都要」；`research/prompts/rules-compress-r2-merged/dot-claude/rules/show-me-test-by-review.md:7` 同一句（一-1 引过）。修稿自己的去向表也写落两处：`research/prompts/rules-compress-r2-merged/dispositions.md:61`：「被写死成」那一行的落点列点名这两份文件。
- 违的是压缩口径第 2 条：`records/2026-11-16-三方豁免取消与存量补轮.md:56`：「一件事只在一处写全」。实现流程那一行管的只是 `crates/` 下的改动，却抄进了实验装置那张表。
- 照第二版做会怎样：一-2 那条要改的时候（补钩子、门禁的落点），两处要同改；改一处漏一处，两处就说不同的话。改法（零轮）：实现流程那一格只留「两样都要」与指针，指到 show-me-test-by-review 那一节。

### 二-2 用户定案第 18 条（提交之外先弹窗）在这一组的门禁、提交技能里没落：三处仍写「用户要求时」照跑，不指过去

- 定案：`records/2026-11-16-三方豁免取消与存量补轮.md:170`：「提交之外跑不指定范围的门禁」，同一行要求「用户要求的那一趟也弹窗确认一次再跑」「三份统一」。
- 修稿说这句只住一处、别处指过去：`research/prompts/rules-compress-r2-merged/dispositions.md:18`：「句子只住 `dot-claude/rules/verification.md:38`」，点名指过去的五处里有 crash-test 技能，没有门禁技能、staged-gate 与 commit-stages。
- 这三份照旧把「用户要求时」当一种跑的时机、不提先弹窗：
  - `research/prompts/rules-compress-r2-merged/dot-claude/skills/gate/SKILL.md:10`：「与用户要求时，由主 agent 照」staged-gate 交，句末是「其余时候不跑」。
  - `research/prompts/rules-compress-r2-merged/dot-claude/skills/gate/staged-gate.md:16`：「2. 只在提交时（或用户要求时）跑：」。
  - `research/prompts/rules-compress-r2-merged/dot-claude/skills/crash-test/commit-stages.md:21`：「2. 只在提交时（或用户要求时）跑这六道」，同文件 `research/prompts/rules-compress-r2-merged/dot-claude/skills/crash-test/commit-stages.md:9`：「这一次是提交时跑还是用户要求时跑」。
- 而权威那一句只把提交时的确认算成 `--full-gate` 那次弹窗：`research/prompts/rules-compress-r2-merged/dot-claude/rules/verification.md:38`：「提交时那一趟的确认就是带 `--full-gate` 的那一次弹窗」。
- 两种读法：甲（照门禁技能）用户要求了就照 staged-gate、commit-stages 直接交，确认只有服务对 `--full-gate` 的那一次；乙（照 verification.md）用户要求的也先弹窗问一次、同意了再交，交的时候 `--full-gate` 再问一次。正是第 18 条要收的「用户要求的照跑」那一种分歧，只是换到了技能里。
- 照第二版做会怎样：主 agent 在提交之外被用户叫跑整轮，读门禁技能就走甲；读 verification.md 就走乙。实际后果有 `--full-gate` 的服务弹窗兜着，用户总会被问到一次，所以是二义、不是越权跑（推的，没核服务对提交之外的 `--full-gate` 收不收：`task-server/task_server.py` 的 `full_gate_refusal` 只读了注释，没读判法）。
- 改法（零轮）：三处的「（或用户要求时）」后面接「，用户要求的照 `.claude/rules/verification.md`「提交之外」先弹窗」，或删掉「或用户要求时」只留指针。

### 二-3 测试脚本规则补回上游 show-me-test 时写成两节，读得出「只受这两节管」

- 原件管辖清单点的是整份上游文件：`.claude/rules/test-script-discipline.md:7`：「（新测试先证明会红）」那个括号是节名写错了（按需-M4），点名的是整份 `show-me-test.md`。
- 第二版：`research/prompts/rules-compress-r2-merged/dot-claude/rules/test-script-discipline.md:4`：「（门禁不许假装通过；每一个弄坏开关都必须会红）」，括号里两节。
- 两种读法：甲，括号是举例，整份上游 show-me-test 照管；乙，只有这两节管测试脚本。上游另有一节正管测试装置：`.claude/singlefs-ai-sop/rules/show-me-test.md:45`：「## 分段计时在被测进程里计」，照乙它就出了这份规则的射程。
- 照第二版做会怎样：写装置计时的人照乙不受「分段计时在被测进程里计」管；那一节有门禁（上游 relay-timing-lint）兜，所以做法上多半不出错（推的）。程度轻。改法（零轮）：括号去掉，或写成「（整份；常用的是……）」。

## 格三：第一轮只抽了、只机械核了的 18 份

### 三-1 staged-gate 第 17 行留着已退役前缀的残句（原件带来；第二版在 commit-stages 修了同类一处、这一处没修）

- `research/prompts/rules-compress-r2-merged/dot-claude/skills/gate/staged-gate.md:17`：「（用户要求时 `=user-request`）」。同一份文件开头说不带那个前缀：`research/prompts/rules-compress-r2-merged/dot-claude/skills/gate/staged-gate.md:8`：「交给服务的命令不带 `SINGLEFS_HEAVY_TESTS` 前缀」；那个前缀已退役：`.claude/kb/decisions-history.md:6627`：「这个前缀已退役，用户要求时由主 agent 经 `task-server/task submit` 交，命令不带它」。
- `=user-request` 在仓里（`research/` 之外）只剩 kb 里记当时跑法的几行（决策变更史、实验 168 与 171 的页）与这一处（`grep -rn 'user-request'` 量的，原件 `.claude/skills/gate/staged-gate.md` 第 16 行同样有）。第二版把 commit-stages 的「带…前缀」改成了「不带 `SINGLEFS_HEAVY_TESTS` 前缀」（patch 第 1031 行），staged-gate 这一处残句没动。
- 照第二版做会怎样：读到这一句的人要么当它是要带的标记（照旧前缀补成 `SINGLEFS_HEAVY_TESTS=user-request`，与第 8 行矛盾），要么不知道它指什么。不是压缩新带来的；按格三「二义的」报，程度轻。改法：删掉这个括号。
- staged-gate 不在正文点名的 19 份里，是对 gate 技能逐句时顺带读到的；第一轮攻方说它逐行对过，这一处是原件就有的，所以那一轮没报不算漏。

### 逐句对过、没找到丢失或二义的

逐份把原件每一句在第二版里找了落点（含指到上游与别的规则的那几句去那一处核了在不在）：

| 原件 | 第二版 | 结果 |
|---|---|---|
| code-discipline-by-review（182 行） | 同名（104 行） | 七张审过的表逐行对过：写法删成「写法在上游某节」的那些行，去上游 `code-discipline.md` 核了 getter / setter、`Default`、`..Default::default()`、builder、`try_from`、单字母与缩写、布尔、测试名、`BlockAddress`、「未知」做成显式成员、注释只写为什么、`TODO`、`SAFETY`、边界上验一次，都有（`grep -n` 量的）；「函数要短」处置从「留下」改「改写」已登按需-M1，写法不变 |
| path-moves（73 行） | 同名（58 行） | 第 2 步细则下沉到 `.claude/gate.d/lib-lag-baseline.py` 文件头「搬迁登记」：生效条件、不许串、搬迁实据、新行必须生效、`HEAD~1`、`${变量:-`、占位文件、E171 钉值倒换、单文件最后一段不许变，关键词在那份文件里都命中（`grep -c` 量的，只数了命中，没逐句对那份文件头） |
| timer-only-remind、directory-layout、implementation-first、long-run-split、changelog-format | 同名 | 逐句都在；changelog-format 删掉的「登记的必须以历史版本收尾、没登记的不许有」在上游 `kb-discipline.md` 第 8 节逐字有；directory-layout 删掉的「不经 `~` 取」由 patch 补进了 verification.md 被指到的那一节 |
| mutation-sampling（93 行） | 同名（80 行） | crates-mutation-replay 判每一条的长句拆成 1 到 7 步，次序与每一支的结局逐支对上；「这条只在有变异 harness 的仓里生效」一句删了，本仓有 harness，做法不变 |
| test-discipline（126 行） | 同名（91 行） | 「自证每一格单跑与整份跑判定相同、失败句不冒充别的格」一节挪进 test-script-discipline「用例原子化」第 6、8 行与失败句那一条，都在 |
| test-script-discipline（97 行） | 同名（90 行） | 除二-3 外逐句都在；「红在点名的那一项」「复跑的一方逐个打开」交给上游「每一个弄坏开关都必须会红」，上游那一节逐字有 |
| machine-first 原有各节（78 行） | 同名前 95 行 | 前提、收回表、四步流程、判据、放宽与留下两张表都在；「审完的结果在上游 code-discipline、唯一一处」改指 code-discipline-by-review（按需-M6 已登） |
| remind、halt、decide、queue-health、trilingual、crash-test、gate 技能 | 同名 | remind 把「不是定时的等待」交给规则那一节，少了「或结束本轮等完成通知」这一种（共用约束「不做」一节另有，不算丢）；crash-test 删掉的「跑了且通过才换到由项目本地阶段覆盖」「退 77 记本次未跑、不算覆盖」在上游 show-me-test「最终判据由项目定」「门禁不许假装通过」逐字有；gate 技能 light / heavy 两档没了已登按需-M14，与 `.claude/hooks/selftest-registry.tsv` 文件头「没有轻重之分」一致 |

## 用户定案在这一组落没落对

| 定案 | 第二版落点 | 判定 |
|---|---|---|
| 第 3 条 并行度缺省取登记值 | `research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:40`：「不按 `nproc` 取」；全树 `grep -rn 'nproc'` 只剩这一处与实验设计员定义里两处「核数」（量的） | 落对 |
| 第 4 条 证红两样都要 | show-me-test-by-review 第 7 行 | 两样都在；射程见一-2，住两处见二-1 |
| 第 20 条 只许对自己起的组发 | `research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:9`：「组号只取自己用新会话起的那一个子进程的」，并以钩子 ⑥ 为准；钩子文件头放行的正是这一种：`.claude/hooks/bash-command-detector.sh:184`：「对自己用新会话起的那一个子进程的进程组发信号（上游」 | 这一组落对。第 20 条原话「命令行上与脚本里都只许…」，旁组（定义那一题）有两处读得窄：`research/prompts/rules-compress-r2-merged/dot-claude/agent-common.md:67`：「命令行里停进程照旧一次点名一个」，而同文件第 80 行 ⑥ 放行对自己起的组发；`research/prompts/rules-compress-r2-merged/dot-claude/agents/tooling-writer.md:33`：「自证里只对自己起的进程号发信号」，自证脚本里能不能对自己起的组发读出两种，交定义那一题 |
| 第 21 条 检出工具只报不停 | `research/prompts/rules-compress-r2-merged/dot-claude/rules/command-safety-by-review.md:18`：「不拦命令、不停进程与子 agent」，终止归报告的人或 agent | 落对；只是矛盾总表那一行还写着要交用户定：`research/prompts/rules-compress-r2-merged/contradictions.md:46` 常驻-11 末列「是否放回交用户定」，没改成引第 21 条（材料，不是规则，不算打中） |

## 我认为最脆弱的三处

1. **一-1 括号写法的上游指针**：46 处今天全对，明天上游一改名就全部悄悄落空，而它们是唯一会被自动核的那一类。病根是把「指不到的改」推广成「全部改」；判决第 7 条的射程（只改 0 命中的）没被照字面落。
2. **二-2 提交之外先弹窗**：同一件事此刻在 verification.md 写「先弹窗」，三份技能写「用户要求时跑」；第 18 条定案就是为收这种分歧立的，技能这一侧没收。
3. **一-2 变异清单的射程**：第一轮已打中、修了一半；钩子与门禁自证的证红落点仍没写，而 test-script-discipline 正指到这一句。

## 没打中的形状与范围

- 格一：patch 里落在这一组的每一处改动都对过原件或脚本文件头：evidence-discipline 放回的「引的时候引那一行」「先问它是哪个参数的函数，再写进正文」、format-evolution 放回的「即值 − 1」，三句与原件逐字相同（`grep -nF` 量的）；fs-design 两处新指针（verification-reference「各道的判定单位」里有「lkmm 每份 litmus」，「门禁管哪一半」有 lkmm 那一行，`.claude/scripts/lkmm.sh` 文件头「会失败的检查」第 3、4 条是去屏障对照组与锚点）都指得到；commit-stages 改指 verification-reference「门禁 crash-point-replay 怎么跑」，那一节有九格、核账与「要上卡的三处」；session-wrapup 放回「等改动停下再跑」并加先弹窗，与第 18 条一致；code-discipline 删「两处说同一件事以上游为准」，原件确实没有；command-safety 第一节照第 20 条恢复，「装置、判法与复跑脚本」与原件同（没有第一版多出的「自证」）。
- 格二：查了修补后的判据是不是住两处（二-1 一处；样本进程那条共用约束与规则两处说，原件就是两处、共用约束写「写法照」规则，不算新）、指针指不指得到（第二版里这一组的 `文件「节」` 指针我只核了修补新加的那几处，其余靠修稿报的「指不到 0」，没复核）、新句子有没有两种读法（二-2、二-3）。
- 格三：上表 18 份逐句对过（path-moves 下沉进脚本文件头的那一半只数了关键词命中）；staged-gate 只读了第 8、16、17 行。

## 这条腿自己的限度

- 每个打中只抽了一次样；三条改法都是我只读文字提的、被攻过零轮。
- 「逐句对过」是人眼并排读，换了次序、拆成几条的地方可能漏看限定词；没写比对脚本。
- 一-1 里门禁 ③ 认不认括号写法是读 `lib-governance-refs.py` 的匹配式推的，没跑那一格；判据句是不是都留在项目一侧，只看了 `rules/` 与 crash-test 技能里的 30 处，别的组没看。
- 二-2 只读了 `task_server.py` 里 `full_gate` 那一行注释，没读判法，「提交之外带 `--full-gate` 服务收不收」是推的。
- 没攻格四（分工表不归攻方），没读禁读清单里的任何文件，没读修稿写手的对照表与 merge-result、revise-result 的正文。

## 什么现象会推翻这份报告

- 一-1：门禁 ③ 其实也认括号写法（`lib-governance-refs.py` 别处另有一条匹配），或上游同步时另有一道检查核项目侧括号里的节名。
- 一-2：钩子、门禁与非装置脚本的自证另有一张入库变异表，或「两样都要」被别处明写成只管 `crates/` 与装置。
- 二-1：实现流程那一行的变异句被判成只是指针的展开、两处由门禁核同步。
- 二-2：用户或主 agent 说明第 18 条的「先弹窗」就是 `--full-gate` 那一次服务弹窗（那就只有一种读法，三份技能不用改）。
- 三-1：`=user-request` 在某个脚本里仍被认（`grep -rn` 只排了 `research/`，没排编译目录与被忽略的文件）。

## 要主 agent 跑的命令

没有要交任务服务的命令（没写模型、没起项目代码）。判决要引上面标「量的」的几个数时，主 agent 可在仓根直接重跑这几条只读命令：

1. `grep -rhoE 'singlefs-ai-sop/rules/[a-z-]+\.md`「' research/prompts/rules-compress-r2-merged/dot-claude research/prompts/rules-compress-r2-merged/CLAUDE-merged.md | wc -l`，要输出 0；把式子末尾的「换成全角左括号再跑一次，要输出 46。
2. `python3 ~/.local/share/singlefs/work/rules-compress-residents-r2-opus/paren-pointer-check.py . research/prompts/rules-compress-r2-merged`（草稿目录里的只读脚本，不是仓里的代码），末行要是 `paren-pointers segments=59 heading-exact=58 not-heading=1`。
3. `ls scripts/mutation/tables/ | grep -v '^e[0-9]'`，要只剩 `README.md`。
4. `grep -rn 'user-request' --include='*.md' --include='*.sh' --include='*.py' . | grep -v '^./research/'`，看 staged-gate 那一处与 kb 几行。

## 没做什么

- 不判辩方那几格，不替主 agent 采纳；没往统一任务服务交任务；没有副本上的数。
- 没建仓副本、编译目录；草稿目录里只有一份核对脚本与两份 cite-check 输出，不用清。
