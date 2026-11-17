# rules-compress-residents-r3 攻方腿报告

日期：2026-11-16。轮名 `rules-compress-residents-r3`（第三轮，最后一轮）。被判对象：第三版合稿树 `research/prompts/rules-compress-r3-merged/` 里这一组（快照清单 `research/prompts/rules-compress-residents-r3-snapshot/judged-sha256.txt` 的 35 份）的规则与技能。开工时 `sha256sum -c` 两份快照清单全部 OK（原件 35 份、合稿树 56 份，量的）。没写模型，没有要复跑的数。

## 各格判定一览

| 格 | 判定 | 打中 |
|---|---|---|
| 一 第二轮修补对不对 | 打中 1 处（证红那一句新写的「入库装置」）；用户定案第 24 条在三份技能里的落法对，另有 1 处落宽（见格二 2） | 一-1 |
| 二 修补引出的新问题 | 打中 3 处：一-1 的二义同时算这一格；staged-gate 跑全量工作区那一句提交时要不要弹窗读得出两种；样本进程 `setsid` 的写法住两处、两种写法（矛盾总表登了） | 二-1、二-2、二-3 |
| 三 能不能换上 | 打中 1 处：被并掉的 8 份文件的改法清单漏了 5 处活文件里的旧名（不带 `.md` 的写法），换上后这 5 处点名一份不在的规则；规则正文本身没找到没登的做法不同 | 三-1 |

一句结论：这一组可以换上，但证红那一句的「入库装置」要在换上之前改成写明路径的说法（一处措辞）；其余三处可以换上之后再改。推翻条件：主 agent 现查发现规则目录与技能里另有一处给「入库装置」下了定义（指到 `crates/singlefs-verification/src/bin/`），一-1 就降为轻。

## 格一 第二轮修补对不对

### 一-1（也算格二）证红那一句的「入库装置」读得出两种意思，其中一种与前半句相抵 —— 换上之前必须改

- 第三版：`research/prompts/rules-compress-r3-merged/dot-claude/rules/show-me-test-by-review.md:7`：「`scripts/experiment-apparatus/` 下装置的进 `scripts/mutation/tables/` 下它那一张表，入库装置的变异行进 `crates/mutations.tsv`」
- 原件：`.claude/rules/show-me-test-by-review.md:12`：「有变异 harness 的仓要再进一步：把」（原件只说进「存进仓里的变异清单」，不分表；分表是第二轮去向表那一行补的，`research/prompts/rules-compress-r3-merged/dispositions-r2.md:66`：「不加限定读成含入库装置」）。
- 「入库装置」只在实验执行员定义里有定义：`.claude/agents/experiment-runner.md:30`：「入库装置那一支（`crates/singlefs-verification/src/bin/`）」。规则目录与技能里没有定义它；第三版规则目录里另一处「入库的装置」是另一个意思——仓里那一份正式装置（对着腿的副本说），`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:36`：「要在入库的装置上重做一次才能引」；测试脚本规则里「入库」是「进仓」：`research/prompts/rules-compress-r3-merged/dot-claude/rules/test-script-discipline.md:40`：「变异进入库的变异清单」。我在第三版 `dot-claude/rules/`、`dot-claude/skills/` 全部 `.md` 里 grep「入库」，规则与技能里这三处之外没有给「入库装置」下定义的句子（量的，grep 输出见草稿目录 `ptr-out.txt` 同一趟的命令，没另存）。
- 为什么算打中：show-me-test-by-review 是常驻规则（`CLAUDE-merged.md` 的 `@` 列表里有它），主 agent 与实现员读得到；读的人照三方规则那一处的用法把「入库装置」读成「仓里的装置」时，`scripts/experiment-apparatus/` 下的装置也是仓里的装置，同一句前半要它进 `scripts/mutation/tables/`、后半要它进 `crates/mutations.tsv`，两半相抵。第二轮攻的正是「『实验装置』不加限定」这一种二义，修补换了一个同样不加限定的词。
- 照第三版做会做成什么（推的）：给 `scripts/experiment-apparatus/` 下新装置写变异的人可能把变异行追加进 `crates/mutations.tsv`；那张表的表头只说 crates 的变异（`crates/mutations.tsv:1`：「# crates 的变异表：每行六段」），门禁 crates-mutation-replay 照它的 `cargo test` 参数在仓根工作区跑，指到 `scripts/experiment-apparatus` 的行会不会编得过、记成什么，我没跑，推不出。
- 四句：不涉及臂；读规则的人当时看得到的只有这一句与三方规则那一句，看不到实验员定义里的定义（规则不点名那份定义），判不出是哪一种；满足格二「新写的句子读不读得出两种意思」、格一「放回的话……做法因此不同」两个分句；修稿提的改法（照实验员定义第 3 条的写法）只照搬了词，没照搬定义，在这一格上还中。
- 我提的改法（只在我读的文字上推过、被攻过零轮）：把「入库装置的变异行」改成「`crates/singlefs-verification/src/bin/` 下入库装置的变异行」，或干脆删掉这半句（这类装置在 `crates/` 下，前面「`crates/` 的测试进 `crates/mutations.tsv`」已经罩住，推的）。

### 用户定案第 24 条落到三份技能：落对的几处

逐句对了 `records/2026-11-16-三方豁免取消与存量补轮.md:212`：「命令不带这两个标记、钩子不弹的，主 agent 自己弹窗问一次」与钩子文件头 `.claude/hooks/run-scope-guard.sh:34`：「钩子不放行也不拒：标准输出打 permissionDecision 为 ask 的 JSON、退 0」：

- `research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/SKILL.md:12`：「提交之外要跑照 `.claude/rules/verification.md`」——只指过去，不另写；验证包整包测试要带 `--whole-package-confirmed`（服务拒没带的，`task-server/task_server.py:9324`：「整包测试没带确认标识」），钩子弹的那一次就是确认，与定案一致。
- `research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/SKILL.md:10`：「提交之外要跑的确认照 `.claude/rules/verification.md`」——同上。
- `research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:21`：「或用户要求时，确认照 `.claude/rules/verification.md`」——六道与崩溃放量不带 `--full-gate` 交服务当场拒（`task-server/task_server.py:9312`：「没带 --full-gate：全量门禁都带它交」），所以用户要求的那一趟也带标记、钩子弹的那一次就是确认，与定案一致。
- `research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:17`：「钩子弹窗要用户确认，提交时 1 次」——带 `--full-gate` 的那一条对。跑全量工作区那一条见二-2。

### 格一里核过、没打中的几处修补

- 上游指针：第三版 `dot-claude/rules/` 与 `dot-claude/skills/` 全部 `.md` 里「反引号 `.claude/singlefs-ai-sop/rules/<文件>.md` 加「节」」的写法，用自己的脚本逐处取第一个节名，对上游副本今天的标题行逐字比：指不到 0 处、标题重复 0 处、括号写法残留 0 处（量的，脚本 `~/.local/share/singlefs/work/rules-compress-residents-r3-opus/tools/ptr.py`，输出 `ptr-out.txt`）。逐个读了每一处指到的那一节的内容，没有指错节的：`code-discipline.md` 的「名字」「类型」「分支」「注释、断言、常量、重复」四节；`kb-discipline.md`「2. 每条带出处与状态」「8. 正文只写现状，历史放文末的「历史版本」」（`.claude/history-carriers` 的判法在那一节第 83、84 行，与原件 changelog-format 说的「登记的必须以历史版本收尾」是一回事）；`sop-first.md`「推论」「边界」「每一条拒绝都必须给出下一步」；`show-me-test.md`「最终判据由项目定」「门禁不许假装通过」「每一个弄坏开关都必须会红」；`design-doc-discipline.md`「1. …」；`pushback-discipline.md`「警告写在哪：…」；`writing-discipline.md`「四类不许写」「别矫枉过正」；`rules-discipline.md`「5. 规则文件不留历史节」；`communication-discipline.md`「转述别处的做法，带出处或原文」；`command-safety.md`「测试镜像一律放临时目录」「并行不许把失败吃掉」。
- 「任务卡住由服务判」那半句：`task-server/task_server_lib.py:2141` 的 `stalled` 一行写「服务停了它」，与第三版括注一致。
- 样本进程 `setsid [--wait] <命令> &`：与钩子文件头 `.claude/hooks/bash-command-detector.sh:185`：「shell 里同一层先 `setsid [--wait] <命令> &`（不带 -f）、紧跟着」逐字一致；Python 里 `os.killpg` 在循环里放行与 `:182` 只拒循环里的 `os.kill` 一致。
- 测试脚本规则「show-me-test 整份」：原件写的是整份上游文件，第三版两节只当括注，对。
- session-wrapup「确认一次，同意了才跑 `gate.sh --staged`」：`gate-staged.sh` 带 `--full-gate`，钩子弹的那一次就是确认，与定案 24 一致。

## 格二 修补引出的新问题

一-1 同时是这一格的打中，不重写。

### 二-2 staged-gate 跑全量工作区那一句：提交时要不要另弹窗，读得出两种 —— 换上之后可以再改

- 第三版：`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:18`：「这一条不带 `--full-gate`、钩子不弹窗，主 agent 先照 `.claude/rules/verification.md`」（后接「提交之外」那一条「自己弹窗问用户一次」）。这一句挂在 `research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:16`：「2. 只在提交时（或用户要求时）跑：」下面，提交时也走得到这一条：`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:7`：「没暂存就不跑，或者明定」（后接「跑全量工作区」）。
- 原件：`.claude/skills/gate/staged-gate.md:16`：「跑全量工作区的跑 … 不带参数（前缀照带）」——提交时跑全量工作区不弹窗。
- 定案：`records/2026-11-16-三方豁免取消与存量补轮.md:212`：「提交之外跑不指定范围的门禁」——只管提交之外。第三版验证规则另写提交时的确认就是带 `--full-gate` 的那一次钩子弹窗：`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`：「提交时那一趟的确认也是带 `--full-gate` 的那一次钩子弹窗」；而 `gate.sh` 不在服务认 `--full-gate` 的名单里（`task-server/task_server_lib.py:2264`：「"scripts/crash/crash-amplification.sh", "scripts/gate/gate-staged.sh")」），提交时跑全量工作区的那一趟带不了 `--full-gate`。
- 两种读法：甲、新句子不带条件，提交时跑全量工作区也要主 agent 先弹窗一次（比原件严、比定案 24 宽）；乙、句子指到「提交之外」那一条，只在提交之外弹（与原件、定案一致）。照甲做，提交时多一次弹窗；照乙做，提交时这一趟按验证规则第 38 行又找不到「带 `--full-gate` 的那一次钩子弹窗」，确认落空。
- 四句：不涉及臂；读的人看得到两句、判不出哪一句管提交时；满足格二「新写的句子读不读得出两种意思」；修稿的改法（把这一句补上）在这一格上还中。
- 我提的改法（推的、被攻过零轮）：句首加「提交之外跑的」，或写明「提交时跑全量工作区不另弹」。

### 二-3 样本进程的 `setsid` 写法住两处、两种写法 —— 换上之后可以再改（矛盾总表登了）

- 第三版规则：`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:8`：「shell 里用 `setsid [--wait] <命令> &` 起（不带 `-f`）」（第三版这一处由第二版的 `setsid --wait` 改来）。
- 第三版共用约束：`research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:67`：「shell 用 `setsid --wait` 起」，同一句又写「写法照」规则那一节。
- 总表：`research/prompts/rules-compress-r3-merged/contradictions.md:49`：「共用约束保留原件的 `--wait`」——登了，但同一条判据仍住两处、两处写法不同，指过去的那一处与被指的那一处说的不一样。
- 照第三版做：子 agent 照共用约束写 `setsid --wait`，主 agent 照规则可以不带 `--wait`；钩子两种都放行（`.claude/hooks/bash-command-detector.sh:185`），所以做法上不出事，只是判据住两处。轻。

### 格二里核过、没打中的

- 新写的句子里有没有「又住两处」：证红的分表只住 show-me-test-by-review 一处（实现流程规则那一格只留「两样都要」与指针）；用户定案 24 的句子只住验证规则第 38 行，三份技能只指过去（逐句读过，见格一）；「任务卡住由服务判」只住 command-safety-by-review 一处。
- 循环逐个发的射程：规则、共用约束、工具员第 5 步三处都写「shell 循环一律拒、Python 每一轮 `os.killpg` 停自己起的组放行」，三处同一个意思、都指钩子文件头 ⑥；这是修稿报告第 ④ 节第 1 条挂着要用户定的，我不攻该不该。
- 指针唯一：上游指针 0 处重复（见格一）；项目规则之间的「文件「节」」指针，第三版规则与技能里 0 处指不到（只取第一个节名比，`ptr-out.txt` 里 22 处是加粗条目名或正文里的条目名，逐个看过都在被指的文件里）。

## 格三 能不能换上

### 三-1 被并掉的 8 份文件：改法清单漏了 5 处不带 `.md` 的旧名 —— 换上之后可以再改（换上那一步顺手补最省）

- 清单的口径：`research/prompts/rules-compress-r3-merged/references.md:3`：「合稿树 `merged/` 换上之后，合稿树之外的活文件里按旧名引被并掉、被改名的文件与节」。原件路径规则要求旧名四种写法各搜一次：`.claude/rules/path-moves.md:12`：「光文件名、不带 `.md` 的名字，四种各搜一次」。
- 我按 8 个文件名不带 `.md` 的写法 `git grep -n --untracked`（排除 `research/`、`records/`）搜到 98 行（量的，输出在草稿目录 `refs8.txt`），去掉合稿树里整份替换的文件（`CLAUDE.md`、`.claude/rules/`、`.claude/agents/`、`.claude/agent-common.md`）之后，清单第一节里找不到的 5 处：
  - `scripts/experiment-apparatus/e68_inline_threshold.rs:3`：「被引用条款逐字贴在这里（verify-before-claiming）」
  - `scripts/experiment-apparatus/e89_interval_frontier.rs:183`：「verify-before-claiming：引用决策去推导之前先贴定义」
  - `.claude/kb/experiments/91-ring-accounting-and-admission-accounting-under-settled-format.en.md:16`（标题行 `### Verbatim Cited Clauses (verify-before-claiming)`）
  - `.claude/kb/experiments/91-已定格式下的环账与准入账.md` 第 17 行与 `.claude/kb/experiments/91-確定フォーマット下でのリング会計と受入会計.ja.md` 第 17 行（同一个标题的中文、日文版）
- 为什么算打中：清单是换上那一步「全仓引用处」的依据；照它改完，这 5 处仍点名一份已经 `git rm` 的规则。这几处不带反引号与 `.md`，门禁 doc-registries 的 governance-refs 格认不出（推的：它判的是反引号里的路径），不会红。
- 照第三版做会做成什么：换上之后两份装置注释与三份 kb 实验页的标题指到不在的规则名，读的人找不到「它定了没」那一节现在住在 `.claude/rules/evidence-discipline.md`。规则本身的做法不变。
- 我提的改法（推的、被攻过零轮）：清单第一节补这 5 行，改成 `evidence-discipline.md`「它定了没，和它到底怎么定义的，是两个问题」。

### 格三里核过、没打中的（规则正文的做法）

- 被并掉的 8 份逐条对到第三版落点（整份读原件、读落点那一节）：
  - `verify-before-claiming.md` 54 行 → `evidence-discipline.md` 第 22 到 55 行：表 6 行、「不用现查的」、「落实」三条、「它定了没」、「核了窄的」三条与截断那一条、「检查还没出结局」四句都在；少的是「门禁只在提交时跑」这个理由与「不跟着已经过了的那几件一起写成过了」这半句重复的说法，做法不变。
  - `artifact-line-pointer.md` 13 行 → `evidence-discipline.md` 第 141 到 145 行：指针写法、前缀起头、前缀登记表、判法、接门禁前记欠账，全在。
  - `exhaustive-interleaving.md` 26 行 → `fs-design.md` 第 103 到 114 行：分工表、口径、撑起什么、收回条件都在；「用哪个工具……写在 verification.md」改指 lkmm 与 `.claude/scripts/lkmm.sh` 文件头「会失败的检查」第 3、4 条，那两条就是对照组与绑到代码（现读 `.claude/scripts/lkmm.sh:34-43`）。
  - `service-stop-readback.md` 10 行 → `command-safety-by-review.md` 第 46 到 50 行：四条全在。
  - `communication-discipline-by-review.md` 17 行 → `writing-discipline-by-review.md` 第 54 到 65 行：五条与射程两条都在。
  - `engineering-philosophy.md`、`design-doc-discipline-by-review.md`、`pushback-discipline-by-review.md` → `machine-first.md`：两条轴表、AI 友好 ≠ 不可读、四项不放宽、四件事、分界线、说服两件事、长度指针、对不上就当场说三格、做完了才发现、分清核过与印象、对方坚持，全在；指针改动（M6 到 M9）矛盾总表登了。
- 关键字句对照：35 份原件里带「不」「只」「一律」「除外」「先」「才」的句子共切出的每一句，用自己的脚本按三字片段在第三版全树里找最像的一句，相似度低于 0.55 的 208 句逐句人工看（量的，脚本 `tools/sentmatch.py`，输出 `sent-out.txt`）。看下来要么第三版换了说法、意思在（多数），要么挪进了上游文件或脚本文件头（例：路径规则的生效条件、`${变量:-旧前缀/…}` 那一条在 `.claude/gate.d/lib-lag-baseline.py:53-98`「搬迁登记」里逐条都有），要么矛盾总表或用户定案登了（例：`函数要短` 留下改改写是按需-M1，`--selftest` 不再分 light / heavy 是按需-M14，长活定期复检改不轮询是常驻-12）。没找到另一处没登的做法不同。
- 原件里带反引号的词在第三版全树里一个都找不到的 42 个（量的，`tools/tokens.py`，`tokens-out.txt`），逐个看：示例代码与例子（code-discipline-by-review 的 `stripe_index`、`block::Address` 这类，用户定案第 8 条删代码示例；同样的例子在上游 `code-discipline.md` 第 24 行）、被并掉文件的旧名、已删的 `=user-request`（常驻-21）、挪进文件头的写法、remind 技能「只许这一个」挪进规则「不用」一节。没有判据丢失。

## 没打中的形状（查过的范围）

- 第二轮修补：`diff-from-r2.patch` 落在这一组的 14 份（changelog-format、code-discipline-by-review、command-safety-by-review、evidence-discipline、fs-design、machine-first、session-wrapup、show-me-test-by-review、test-script-discipline、writing-discipline-by-review、crash-test 的 SKILL 与 commit-stages、gate 的 SKILL 与 staged-gate）每一个 hunk 逐行读过，对了原件与它引的脚本、钩子、服务文件头（run-scope-guard 第 7、33 到 47 行，bash-command-detector ⑥ 第 165 到 199 行，task_server_lib 的 `stalled` 行与 `COMMIT_GATE_ENTRIES`、`UNSCOPED_RUN_ENTRIES`，task_server 的 `full_gate_refusal` 与 `whole_package_refusal`，lkmm.sh「会失败的检查」，lib-lag-baseline「搬迁登记」，selftest-registry 表头）。
- 定案 23 到 26：23、25、26 落在别的组的文件（实现流程、入口、三方规则），这一组只看了它们没在本组文件里留与之相抵的句子；24 见格一、格二。
- 上游指针：规则目录与技能里全部「`.claude/singlefs-ai-sop/rules/<文件>.md`「节」」逐处对上游副本今天的标题，指不到 0、重复 0（量的）。
- 八份被并掉的文件逐条对到落点，见格三；它们在合稿树里的旧名 0 处（量的：`grep -rn` 八个名字在 `dot-claude/` 与 `CLAUDE-merged.md` 里无输出）。
- 35 份原件带关键字的句子：相似度阈值 0.55 以下的 208 句人工看过；0.55 以上的没逐句人工看（按片段相似算过，没细读）。
- 试过、没成立的形状：staged-gate 删掉了原件那半句里分片阶段读不到时的条件（门禁 crash-point-replay 文件头「分片」一段第 190 行写 gate.sh 整轮里本来就没有分片阶段、照计划单片合并三步自己走，删了不改做法）；staged-gate 删掉「六道各自按单元键查同一份判定库」（说明句，做法在「照各自的复用判定走」里）；timer-only-remind 把 `until ! kill -0` 列成照旧的等待而前台又被 ② 拒（原件同形，常驻-9 登了）；样本进程「留下的停掉」与「不先挑进程再发」（原件就有这句，用户定案 20 以钩子为准，只能再对自己的组发）；`gate.sh` 全量工作区服务收不收（服务不拦它：不在 `UNSCOPED_RUN_ENTRIES`，所以只靠主 agent 自己弹窗，与二-2 同一处）。

## 这条腿自己的限度

- 一-1 里我写「grep 输出见草稿目录 `ptr-out.txt` 同一趟的命令，没另存」不准：那一次 grep「入库」的输出只在我这一轮的命令输出里，草稿目录里没有存；`ptr-out.txt` 是上游与项目指针那一趟的输出。要复核请主 agent 现跑 `grep -rn '入库' research/prompts/rules-compress-r3-merged/dot-claude/rules research/prompts/rules-compress-r3-merged/dot-claude/skills`。
- 相似度脚本只是筛子：阈值以上的句子我没逐句人工看，一句里换了一个限定词（「只」「先」）而其余字都相同的，这个脚本筛不出来。
- 指针脚本只核每处指针的第一个节名，连写的第二、三个节名（「…」「…」）没逐个机核，只在格一点名的那些处人工看过。
- 二-1、三-1 的「照第三版做会做成什么」里，crates-mutation-replay 对指到 `scripts/experiment-apparatus` 的行怎么判、governance-refs 认不认不带反引号的旧名，都是推的，没跑。
- 别的组的文件（入口、共用约束、定义、验证规则、三方规则、实现流程规则、三份新技能）只在这一组的句子指到它们时读了那一处，没整份判。
- 这是一次抽样；照三方规则，一条腿的「没打中」不单独支撑结论。

## 要主 agent 跑的命令

没有。这一轮没写模型、没拷仓副本、没编译；跑过的只有只读命令、`cite-check.py` 与我放在草稿目录里只读文本的四个小脚本（`tools/ptr.py`、`tools/tokens.py`、`tools/sentmatch.py`、`tools/disp.py`，用 `python3` 直接跑，只读仓与合稿树）。主 agent 要复核数，在仓根直接跑（都是只读文本）：

- 在 `research/prompts/rules-compress-r3-merged/` 里起：`python3 /home/fy5090/.local/share/singlefs/work/rules-compress-residents-r3-opus/tools/ptr.py $(cd dot-claude && ls rules/*.md skills/*/*.md | sed 's#^#dot-claude/#')`；输出里 `NOFILE` 是指到 kb 等合稿树外文件的（脚本只在合稿树里找 `.claude/` 下的文件），`TEXTONLY` 是加粗条目名，看有没有 `MISSING`、`HDUP`。
- `git grep -n --untracked -E 'verify-before-claiming|artifact-line-pointer|exhaustive-interleaving|service-stop-readback|communication-discipline-by-review|engineering-philosophy|design-doc-discipline-by-review|pushback-discipline-by-review' -- . ':!research' ':!records'`：看三-1 那 5 行在不在。

## 没做什么

- 没判正推那一格的落点表；没替主 agent 采纳；没改任何文件（报告与草稿目录之外）。
- 没交任务、没跑门禁与自证；没建仓副本与编译目录，草稿目录 `~/.local/share/singlefs/work/rules-compress-residents-r3-opus/` 里只有四个只读小脚本与它们的输出文本（交回前 `du -sh` 记在交回正文）。
- 没读禁读清单里的文件（这一轮正推腿的报告、模型与草稿目录，别的四题第三轮各腿的产出）。
