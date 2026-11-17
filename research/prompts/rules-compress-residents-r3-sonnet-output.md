# rules-compress-residents-r3 正推腿报告（其余项目规则与技能）

日期：2026-11-16。立场：正推，逐处核 `diff-from-r2.patch` 里落在这一组的改动。
合稿树简写：`R3` = `research/prompts/rules-compress-r3-merged/dot-claude`；原件指仓里 `.claude/` 下同路径的文件（`judged-sha256.txt` 与 `merged-tree-sha256.txt` 开工时我各核了一遍，`sha256sum -c` 全 OK、无不符）。

## 〇、先说结论

- 这一组在 `diff-from-r2.patch` 里的改动共 14 个文件：带文件名的上游指针 36 处（外加同句里跟在后面的 5 个节名）、其余 12 处。上游指针全部指得到、各只有一个标题；其余 12 处里 10 处对得上依据，2 处有问题（下面「五」）。
- 格一（第二轮修补对不对）：成立。没有哪一处与原件原话或脚本文件头不符而做法因此不同；定案第 24 条落的几处是那个意思；第 20 条落的那一句，多出来的 Python 半句要标出来（见「四」）。
- 格二（修补引出的新问题）：不成立（即打中了）两处——① 证红那一句里「入库装置」在这份规则里没有定义，读得出两种意思（换上之前必须改，一个词的事）；② 新补的「`stalled` 一行」指针指向 `task_server_lib.py` 文件头「错误类型」表，今天的文件头里没有这张表（换上之后可以再改，原件同样写错，不是压缩带来的）。
- 格三（能不能换上）：成立。这一组的改动照做的结果，与照原件做的结果不同的地方，全都在矛盾总表或定案里登了；没找到没登的不同。
- 一句话：这一组可以换上，换上之前必须改一处（「入库装置」那一个词），别的换上之后可以再改。

## 一、没碰的与读到的东西（照实说）

- 禁读清单里的文件我一份没打开：`rules-compress-residents-r3-opus-output.md`、`rules-compress-residents-r3-opus-model/`、攻方草稿目录、别题第三轮的产出与草稿目录。
- 有一次我对 `research/prompts/` 下按 `rules-compress-*-r*-opus-output.md` 通配做 grep，通配会圈进禁读的那份；权限闸当场拒了，一个字没读到。之后只对点名的文件做 grep。
- 读调度记录 `records/2026-11-16-三方豁免取消与存量补轮.md` 找第 23 到 26 条与 20 条时，顺带读到主 agent 写的一段「压缩 2h 攻方交回四条（腿的说法）」，其中有一句提到这一题攻方腿点了「入库装置」没有定义。我的「入库装置」这一条是在读到那一句之前，对着 `show-me-test-by-review.md` 第 7 行与 `experiment-runner.md` 第 30、37 行自己读出来的；这里按我自己读出的写，那一句只当事后旁证，不当依据。
- 没起模型、没写模型目录、没交任务；`rerun.sh` 与复跑封装不适用（没有模型）。我直接在本地跑了三个只读的 python 文本脚本（只读仓里的文件与合稿树，写的只有自己的草稿目录；`python3 -I` 起），命令与 sha256 在「八」。

## 二、方法

1. 把 `diff-from-r2.patch` 里落在这一组的加行逐处拆开（指针类用脚本抽，其余人读）。这一组的文件：规则 `changelog-format`、`code-discipline-by-review`、`command-safety-by-review`、`evidence-discipline`、`fs-design`、`machine-first`、`session-wrapup`、`show-me-test-by-review`、`test-script-discipline`、`writing-discipline-by-review`；技能 `crash-test/SKILL.md`、`crash-test/commit-stages.md`、`gate/SKILL.md`、`gate/staged-gate.md`。不在这一组的 `verification`、`implementation-workflow`、`three-way-inference`、`verification-reference`、`dispatch`、`three-way`、`merge-verify` 没判。
2. 上游指针：每一处读上游今天的文件（`.claude/singlefs-ai-sop/rules/`，此刻工作区版本），核节名逐字等于一个标题、且那份文件里只有这一个。
3. 脚本句子：读 `.claude/hooks/bash-command-detector.sh`、`.claude/hooks/run-scope-guard.sh`、`task-server/task_server_lib.py` 此刻的文件头与它们代码里的判法注释。
4. 定案：现读调度记录里那一条的原话，对改后那一句。

## 三、上游指针逐个核（36 处，加 5 个同句里的后续节名）（脚本 pointer_table.py 抽出，上游标题行用 grep 复核）

「上游标题行」是上游文件里 `#` 起头、去掉井号后与节名逐字相同的那一行的行号；「个数」是这样的标题行在那份文件里有几个。全部：个数 = 1。

| 合稿文件:行（`R3/rules/…`） | 上游文件:标题行 | 节名 | 对不对 |
|---|---|---|---|
| changelog-format.md 第 6 行 | kb-discipline.md 第 81 行 | 8. 正文只写现状，历史放文末的「历史版本」 | 对 |
| code-discipline-by-review.md 第 37 行 | code-discipline.md 第 11 行 | 名字：光看名字，就说得出它是什么、做什么（同行「类型」是同一份里的另一节，下行 113） | 对 |
| code-discipline-by-review.md 第 48 行、53、69 | code-discipline.md 第 113 行 | 类型：让非法状态写不出来 | 对 |
| code-discipline-by-review.md 第 49 行、59 | code-discipline.md 第 80 行 | 分支：穷举写全，不写通配臂 | 对 |
| code-discipline-by-review.md 第 53 行、64、73 | code-discipline.md 第 139 行 | 注释、断言、常量、重复 | 对 |
| code-discipline-by-review.md 第 87 行 | show-me-test.md 第 19 行 | 最终判据由项目定 | 对 |
| command-safety-by-review.md 第 29 行 | communication-discipline.md 第 15 行 | 转述别处的做法，带出处或原文 | 对 |
| evidence-discipline.md 第 32 行、61、81 | kb-discipline.md 第 32 行 | 2. 每条带出处与状态 | 对 |
| fs-design.md 第 87 行 | code-discipline.md 第 113 行 | 类型：让非法状态写不出来 | 对 |
| fs-design.md 第 113 行 | code-discipline.md 第 80 行 | 分支：穷举写全，不写通配臂 | 对 |
| machine-first.md 第 13 行 | sop-first.md 第 18 行 | 每一条拒绝都必须给出下一步 | 对 |
| machine-first.md 第 95 行 | design-doc-discipline.md 第 7 行；kb-discipline.md 第 81 行 | 1. 正文只写当前现状，历史一律放到文末；8. 正文只写现状… | 对 |
| machine-first.md 第 106 行 | pushback-discipline.md 第 6 行 | 警告写在哪：`.claude/warnings/<日期>.md`（标题行就带这段反引号） | 对 |
| session-wrapup.md 第 15 行 | sop-first.md 第 40 行 | 边界 | 对 |
| session-wrapup.md 第 38 行 | kb-discipline.md 第 81 行 | 8. 正文只写现状，历史放文末的「历史版本」 | 对 |
| show-me-test-by-review.md 第 12 行 | sop-first.md 第 7 行；同行后半的「边界」= sop-first.md 第 40 行 | 推论；边界 | 对 |
| show-me-test-by-review.md 第 22 行 | code-discipline.md 第 11 行 | 名字：光看名字，就说得出它是什么、做什么 | 对 |
| show-me-test-by-review.md 第 52 行 | show-me-test.md 第 19 行 | 最终判据由项目定 | 对 |
| test-script-discipline.md 第 11 行 | command-safety.md 第 86 行 | 测试镜像一律放临时目录 | 对 |
| test-script-discipline.md 第 13 行 | code-discipline.md 第 11 行 | 名字：光看名字，就说得出它是什么、做什么 | 对 |
| test-script-discipline.md 第 30 行、53 | show-me-test.md 第 27 行 | 门禁不许假装通过 | 对 |
| test-script-discipline.md 第 31 行 | command-safety.md 第 57 行 | 并行不许把失败吃掉 | 对 |
| test-script-discipline.md 第 40 行 | show-me-test.md 第 54 行 | 每一个弄坏开关都必须会红 | 对 |
| writing-discipline-by-review.md 第 12 行 | rules-discipline.md 第 40 行 | 5. 规则文件不留历史节 | 对 |
| writing-discipline-by-review.md 第 22 行 | writing-discipline.md 第 10 行 | 四类不许写 | 对 |
| writing-discipline-by-review.md 第 26 行 | sop-first.md 第 18 行 | 每一条拒绝都必须给出下一步 | 对 |
| writing-discipline-by-review.md 第 65 行 | writing-discipline.md 第 41 行 | 别矫枉过正 | 对 |
| skills/crash-test/SKILL.md 第 25 行 | show-me-test.md 第 19 行；同句后半的「门禁不许假装通过」= show-me-test.md 第 27 行 | 最终判据由项目定；门禁不许假装通过 | 对 |
| test-script-discipline.md 第 4 行（整份指针，没有节名） | show-me-test.md 整份；括号里的两节 :27、:54 | 门禁不许假装通过；每一个弄坏开关都必须会红 | 对 |

- 复核命令与输出（原样）：`grep -n '^#.*边界' .claude/singlefs-ai-sop/rules/*.md` 只出 `sop-first.md:40:## 边界`；`grep -n '^#.*推论' .claude/singlefs-ai-sop/rules/sop-first.md` 只出 `7:## 推论`；`grep -n '^#.*警告写在哪' .claude/singlefs-ai-sop/rules/*.md` 只出 `pushback-discipline.md:6:## 警告写在哪：`.claude/warnings/<日期>.md``。
- 嵌套引号：`kb-discipline.md` 第 81 行标题本身带「历史版本」，合稿写成「…「历史版本」」两层闭合，读得清（标题逐字取上游）。门禁 governance-refs 对嵌套只核到第一个闭引号之前（`.claude/gate.d/lib-governance-refs.py` 里写了这条），所以门禁看不出标题是不是唯一；我这里是自己按标题逐字比的。
- 这一组里还有 4 处指向别的合稿文件的节名我顺手核了（`提交之外`、`一个门禁：两种跑法`、`新增的测试必须先证明它会红`、`测试脚本自己也要被测试证明有效`）：都指得到、各一个；`提交之外` 是 `R3/rules/verification.md` 第 38 行的加粗条目名，不是标题，全文只有这一条。
- 上游今天的文件是仓里 `.claude/singlefs-ai-sop/rules/` 的那份；记录里说「上游压完同步副本时再统一核」，这一点我没法替上游核，写在「没做什么」。

## 四、其余 12 处逐处核对（改前 / 改后 / 依据 / 对不对）

改前 = 第二版（`diff-from-r2.patch` 的 `-` 行，引补丁行号）；改后 = 第三版合稿树；依据 = 仓里原件、脚本文件头或定案原话。

### 4.1 code-discipline-by-review.md 第 4 行（文件头自称）

- 改前：「与 `.claude/singlefs-ai-sop/rules/code-discipline.md`（`code-discipline-by-review.md` 里叫」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:599`）
- 改后：「与 `.claude/singlefs-ai-sop/rules/code-discipline.md` 同射程，只写上游没写的。」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/code-discipline-by-review.md:4`）
- 依据：原件开头三行没有这一句，也没有「上游」二字（`grep -c 上游 .claude/rules/code-discipline-by-review.md` 输出 0）；括注是第二版自己加的自称，删掉不改做法。
- 对不对：对。注意「只写上游没写的」这半句在第二版与第三版都有、原件没有，是第一遍压缩带进来的范围说明，不是这一轮补丁新加的；它不规定做法，列进「六」。

### 4.2 command-safety-by-review.md 第 8 行（样本进程用 `setsid` 起）

- 改前：「shell 里用 `setsid --wait` 起（不带 `-f`），同一层紧跟着 `名字=$!`」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:669`）
- 改后：「shell 里用 `setsid [--wait] <命令> &` 起（不带 `-f`），同一层紧跟着 `名字=$!`」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:8`）
- 依据 1，原件：「shell 里用 `setsid` 起，Python 的 `subprocess` 带 `start_new_session=True`」（`.claude/rules/command-safety-by-review.md:8`），没写 `--wait`。
- 依据 2，钩子文件头 ⑥ 放行行：「shell 里同一层先 `setsid [--wait] <命令> &`（不带 -f）、紧跟着 `名字=$!`」（`.claude/hooks/bash-command-detector.sh:185`）；代码里的认法注释：「同一层里先用命令位置上的 setsid（不带 -f / --fork）放后台起一条」（`.claude/hooks/bash-command-detector.sh:4342`）。
- 对不对：对。改后与文件头逐字同一回事（`[--wait]` 是可带可不带）；第二版把 `--wait` 写成必带，比钩子认的窄，这一处改回。多出来的半句：无（`<命令> &` 与 `名字=$!` 都是文件头的字）。少掉的：文件头还认 Python 的 `process_group=0` 与 `os.getpgid(<子进程>.pid)`，规则没列，但同一句末尾写「以钩子文件头 ⑥ 为准」，不算丢。矛盾总表已登（常驻-10，照脚本改）。

### 4.3 command-safety-by-review.md 第 9 行（对整组发信号，循环怎么算）

- 改前：「不对别的进程组发信号，不在循环里逐个发，不先挑进程再发」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:670`）
- 改后：「不对别的进程组发信号，不先挑进程再发，不在循环里逐个发 `kill`（shell 里的循环一律拒；Python 里每一轮 `os.killpg` 停自己用新会话起的那一个子进程的组，钩子放行）」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:9`）
- 依据 1，定案第 20 条原话：「命令行上与脚本里都只许对自己用新会话起的那一个子进程的进程组发信号，别的组、循环逐个发、先挑再发一律拒」（`records/2026-11-16-三方豁免取消与存量补轮.md:194`），同一条开头是「维持钩子现在的判法」。
- 依据 2，钩子判法：shell 一侧循环里的 kill 一律记一条：「if in_loop:」（`.claude/hooks/bash-command-detector.sh:4008`），文件头 ⑥ 拒的写法里有「在 for / while / until 循环里逐个发」（`.claude/hooks/bash-command-detector.sh:170`）；Python 一侧：「在循环里调也放行（发的是那一个子进程的组）」（`.claude/hooks/bash-command-detector.sh:4114`），判法里写成「if loop_depth and not own_group:」（`.claude/hooks/bash-command-detector.sh:4333`），即循环里调、目标又不是自己起的组才记。
- 主 agent 的处置也在调度记录里：「的射程以钩子为准（用户定案第 20 条）：钩子放行的」（`records/2026-11-16-三方豁免取消与存量补轮.md:226`）。
- 对不对：对，但要把多出来的半句标出来。「shell 里的循环一律拒」与定案第 20 条同向、与钩子 ⑥ 同向。「Python 里每一轮 `os.killpg` 停自己用新会话起的那一个子进程的组，钩子放行」这半句：定案第 20 条的字面没有它，字面是「循环逐个发……一律拒」；它的依据是同条里的「维持钩子现在的判法」加钩子代码里的那段注释（第 4114 行），钩子文件头 ⑥ 本身没写 Python 循环放行。按用户说的「维持钩子现在的判法」读，这半句成立；用户若是要连 Python 循环里的 `os.killpg` 也拒，那是钩子要改的事，这里原话判不了（修稿报告 ④.1 同样拿不准）。归「换上之后可以再改」，等用户一句话。

### 4.4 command-safety-by-review.md 第 17 行（任务卡住由服务判）

- 改前：「任务卡住由服务判，自己交的任务结束本轮等完成通知」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:680`）
- 改后：「任务卡住由服务判（它停任务，不属于」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/command-safety-by-review.md:17`）；同一行后半：「表的 `stalled` 一行）」（同文件同行）。整句读完：「它停任务，不属于「检出和处置分开」那一条说的检出；判法在 `task-server/task_server_lib.py` 文件头「错误类型」表的 `stalled` 一行」。
- 依据 1，原件：「检出卡住与空转的工具（hook、看门狗脚本）只记录、只报告，不拦命令」（`.claude/rules/command-safety-by-review.md:21`）——「检出」的对象列的是 hook 与看门狗脚本，服务不在其中；共用约束原件：「表的 `stalled` 一行），经」（`.claude/agent-common.md:81`）。
- 依据 2，服务停任务这件事：异常表 stalled 行写「服务停了它｜task-server/task log {task} 核最后一行停在哪一步」（`task-server/task_server_lib.py:2141`）。
- 对不对：做法对（服务判卡住并停任务，与「检出的工具只报不停」不冲突；定案第 21 条说的是「看门狗、钩子」）。「它停任务，不属于……检出」是改后多出来的半句，没有定案原话，依据是上面两条，不改做法，只是消歧。指针不对：见「五」第 2 条。

### 4.5 session-wrapup.md 第 37 行（分不清是谁的红，要不要问）

- 改前：「那一条弹窗问用户，同意了才跑 `gate.sh --staged`」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:836`）
- 改后：「那一条确认一次，同意了才跑 `gate.sh --staged`」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/session-wrapup.md:37`）
- 依据 1，原件没有这个确认：「分不清的时候跑 `bash .claude/scripts/gate.sh --staged`」（`.claude/rules/session-wrapup.md:54`）；原件验证规则：「提交之外要跑先弹窗问用户」（`.claude/rules/verification.md:33`）。
- 依据 2，定案第 18 条：「用户要求的那一趟也弹窗确认一次再跑」（`records/2026-11-16-三方豁免取消与存量补轮.md:170`）；第 24 条：「带 `--full-gate` 或 `--whole-package-confirmed` 的命令由钩子弹的那一次就算确认，主 agent 不另问第二次；命令不带这两个标记、钩子不弹的，主 agent 自己弹窗问一次」（`records/2026-11-16-三方豁免取消与存量补轮.md:212`）。
- 落点：第三版的权威句：「提交之外要跑（包括用户要求的那一趟），要用户确认一次、同意了才跑」与「命令不带这两个标记、钩子不弹的，主 agent 自己弹窗问用户一次」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`），与定案第 24 条逐段同一回事。
- 对不对：对。整句读完：这一句指过去，没有自己再写一遍规则；它交的 `gate-staged.sh` 带 `--full-gate`（`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:17`），钩子弹的那一次就是它要的确认。多出来的半句「确要跑整轮门禁的」：原件只写「分不清的时候跑」，这里加了前提；依据是上面第 18、24 条与原件 verification.md:33，不改做法。矛盾总表已登（常驻-8）。

### 4.6 show-me-test-by-review.md 第 7 行（变异清单落哪、没有变异表怎么办）

- 改前：「`crates/` 的测试进 `crates/mutations.tsv`，实验装置的进 `scripts/mutation/tables/` 下它那一张表）」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:850`）
- 改后：「`crates/` 的测试进 `crates/mutations.tsv`，`scripts/experiment-apparatus/` 下装置的进 `scripts/mutation/tables/` 下它那一张表，入库装置的变异行进 `crates/mutations.tsv`」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/show-me-test-by-review.md:7`）；后半：「没有变异表收的测试（钩子、门禁与 `scripts/` 下非装置脚本的自证）用弄坏开关证红」与「commit message 照样写明」（同文件同行）。
- 依据 1，原件只说「变异清单」没写路径：「有变异 harness 的仓要再进一步」（`.claude/rules/show-me-test-by-review.md:12`）、「commit message 里的叙述是没有 harness 时的退路，不是首选」（`.claude/rules/show-me-test-by-review.md:13`）；定案第 4 条：「变异行之外 commit message 也写改坏了哪一行、哪条断言红」（`records/2026-11-16-三方豁免取消与存量补轮.md:121`）——这两句里，原件第 13 行那句被第 4 条替掉，改后「变异行之外，commit message 里也写明……」落的是第 4 条的意思。
- 依据 2，两张表各装什么：变异表在实验装置目录下跑：「在 `scripts/experiment-apparatus/` 下跑」（`scripts/mutation/tables/README.md:3`）；入库装置：「入库装置那一支（`crates/singlefs-verification/src/bin/`）」（`.claude/agents/experiment-runner.md:30`）、「入库装置那一支不写 research 的变异表、不跑 `mutate.sh`」（`.claude/agents/experiment-runner.md:35`）。
- 依据 3，没有变异表收的测试用弄坏开关：原件测试脚本规则：「每一处会判红的分支，配一个弄坏开关或一条变异」（`.claude/rules/test-script-discipline.md:43`）；原件对每个测试的 commit message 要求：「在 commit message 里写明你是怎么确认它会红的」（`.claude/rules/show-me-test-by-review.md:9`），所以「commit message 照样写明」对所有测试成立。
- 整句读完后的结论：做法对得上依据。两处读法问题：
  1. 「入库装置」这个词在这份规则里没有定义，全句里它与「`scripts/experiment-apparatus/` 下装置」并列，而 `scripts/experiment-apparatus/` 下的装置也是「入库」（进了仓）的；「入库」在这个仓里还有「判定入库」「按条入库」的用法。它的定义只在执行员定义里（`.claude/agents/experiment-runner.md:30`），规则读者看不到。第二种读法（把进了仓的装置都算入库装置）会把 `scripts/experiment-apparatus/` 下装置的变异行写进 `crates/mutations.tsv`，与句子前半相反。这是第三版新写的句子带出的二义，格二打中。改法：把「入库装置的变异行」改成「`crates/singlefs-verification/src/bin/` 下装置的变异行」，一个词的事。归「换上之前必须改」。
  2. 括号里「钩子、门禁与 `scripts/` 下非装置脚本的自证」是举例还是穷举没写；`task-server/`、`prune-server/` 的自证没点名，但它们落在前面的「没有变异表收的测试」这个总括下，读得通。归「换上之后可以再改」，不算打中。
- 矛盾总表已登：合稿-5、常驻-20、用户定-4。

### 4.7 test-script-discipline.md 第 4 行（管整份上游 show-me-test）

- 改前：「`.claude/singlefs-ai-sop/rules/show-me-test.md`（门禁不许假装通过；每一个弄坏开关都必须会红）」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:886`）
- 改后：「`.claude/singlefs-ai-sop/rules/show-me-test.md` 整份（含」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/test-script-discipline.md:4`），括号里两节名见「三」。
- 依据：原件：「它们同样受 `.claude/singlefs-ai-sop/rules/show-me-test.md`（新测试先证明会红）」（`.claude/rules/test-script-discipline.md:7`）——原件点名的是整份文件，括号只是一句描述；第三版整份指针 + 两节名，意思同；原件那句描述「新测试先证明会红」在上游 `show-me-test.md` 里不是任何一个标题（上游标题 :8、:12、:19、:27、:45、:54），第三版把它换成项目那一节的指针「新增的测试必须先证明它会红」写在同一句前面，指得到。
- 对不对：对。矛盾总表已登（按需-M4、常驻-22）。

### 4.8 crash-test/commit-stages.md 第 21 行（六道门禁，用户要求时）

- 改前：「只在提交时（或用户要求时）跑这六道，与」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:1107`）；原件同样没有确认：「只在提交时（或用户要求时）跑这六道」（`.claude/skills/crash-test/commit-stages.md:17`）。
- 改后：「只在提交时（或用户要求时，确认照 `.claude/rules/verification.md`」（`research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:21`），后接「提交之外」那一条。
- 依据：定案第 18、24 条（原话同 4.5）；钩子对六道的要求：不指定范围的六道不带放宽标识就在执行前拒：「提交时照 commit-stages.md 带 --full-gate（会弹窗要用户确认），平时改成点名项或格」（`.claude/hooks/run-scope-guard.sh:45`），所以用户要求时跑六道也是带 `--full-gate` 交、钩子弹的那一次就是确认，与第三版的权威句同一回事。
- 对不对：对。是改后比原件多出来的半句（原件没写确认），依据是第 18、24 条，已登（入口-1 同组）。

### 4.9 crash-test/SKILL.md 第 12 行（验证包，提交之外）

- 改前：「那一条先弹窗。」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:1119`）；原件：「提交之外要跑先弹窗问用户」（`.claude/skills/crash-test/SKILL.md:14`），同一行前面写「不指定范围的主 agent 只在提交时或用户要求时交」（同文件同行）。
- 改后：「提交之外要跑照 `.claude/rules/verification.md`」与「那一条确认一次。」（都在 `research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/SKILL.md:12`）。
- 依据：定案第 24 条，原话见 4.5。
- 对不对：对。「先弹窗」改成「确认一次」是照第 24 条：带标记的由钩子弹、不带的主 agent 问一次；这一行指过去，不再写第二遍。

### 4.10 gate/SKILL.md 第 10 行

- 改前：「其余时候不跑。」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:1173`）；原件：「用户要求时同样交，其余时候不跑」（`.claude/skills/gate/SKILL.md:12`）。
- 改后：「其余时候不跑；提交之外要跑的确认照 `.claude/rules/verification.md`」（`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/SKILL.md:10`），后接「一个门禁：两种跑法」「提交之外」那一条。
- 依据：定案第 18、24 条；原件验证规则「提交之外要跑先弹窗问用户」（`.claude/rules/verification.md:33`）。
- 对不对：对。整句读完：前半「只在提交时与用户要求时」、后半「提交之外要跑的确认照……」，「提交之外」含用户要求的那一趟（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`「包括用户要求的那一趟」），两半不冲突。

### 4.11 gate/staged-gate.md 第 17 行（暂存过的整轮门禁）

- 改前：「（用户要求时 `=user-request`）。这一条弹窗要用户确认，提交时 1 次」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:1185`）。
- 改后：「这一条带 `--full-gate`，钩子弹窗要用户确认，提交时 1 次」（`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:17`），后接「提交之外的确认照 `.claude/rules/verification.md`……钩子弹的那一次就是」。
- 依据 1，删掉 `=user-request`：原件同文件第 8 行写：「交给服务的命令不带 `SINGLEFS_HEAVY_TESTS` 前缀」（`.claude/skills/gate/staged-gate.md:8`），原件第 16 行才有残句：「用户要求时 `=user-request`」（`.claude/skills/gate/staged-gate.md:16`）；退役记录：「这个前缀已退役，用户要求时由主 agent 经 `task-server/task submit` 交，命令不带它」（`.claude/kb/decisions-history.md:6627`）。仓里别处（`.claude/hooks`、`task-server/`、规则、技能）没有任何地方再认 `=user-request`（`grep -rn user-request` 在这些目录零命中；原件 staged-gate 第 16 行以外只剩 kb 历史）。
- 依据 2，钩子弹窗：「主 agent 的命令里 task-server/task submit 在 `--` 之前带这两个标识之一的，钩子不放行也不拒：标准输出打 permissionDecision 为 ask 的 JSON、退 0」（`.claude/hooks/run-scope-guard.sh:34`）；服务认 `gate-staged.sh` 带全量标识：「命令不是提交时的六道门禁、scripts/crash/crash-amplification.sh 或 scripts/gate/gate-staged.sh」（`task-server/task_server_lib.py:2092`）。
- 对不对：对。删掉残句不改做法（前缀已退役，原件第 8 行自己也这么说）；「提交时 1 次」与原件「提交时共 1 次」同数。矛盾总表已登（常驻-21）。

### 4.12 gate/staged-gate.md 第 18 行（跑全量工作区的）

- 改前：「`task-server/task submit -- bash .claude/scripts/gate.sh`，不带参数。」（`research/prompts/rules-compress-r3-merged/diff-from-r2.patch:1186`）；原件：「不带参数（前缀照带）」（`.claude/skills/gate/staged-gate.md:16`）。
- 改后：「这一条不带 `--full-gate`、钩子不弹窗，主 agent 先照」（`research/prompts/rules-compress-r3-merged/dot-claude/skills/gate/staged-gate.md:18`），后接 `verification.md`「提交之外」那一条自己弹窗问一次。
- 依据：定案第 24 条后半「命令不带这两个标记、钩子不弹的，主 agent 自己弹窗问一次」（`records/2026-11-16-三方豁免取消与存量补轮.md:212`）；钩子：整轮 `gate.sh` 不在六道的「提交时才许」名单里：「gate-staged.sh、gate.sh、replay.sh 不带实验号的那一形」（`.claude/hooks/run-scope-guard.sh:47`），也不带放宽标识；服务拒 `gate.sh` 带 `--full-gate`（`task-server/task_server_lib.py:2092`，命令不是那几个入口）。
- 对不对：对。「不带 `--full-gate`」不是可选项而是只能这样，写得准。

## 五、要改的逐条（标换上之前 / 换上之后）

1. 换上之前必须改：`show-me-test-by-review.md:7` 的「入库装置」没有定义（4.6 第 1 点）。改法一个词：把「入库装置的变异行进 `crates/mutations.tsv`」写成「`crates/singlefs-verification/src/bin/` 下装置的变异行进 `crates/mutations.tsv`」。理由：这是照第三版写新装置的变异行时会碰上的一句，两种读法落点相反；而原件在这一处只写「变异清单」，没给路径，所以这是第三版新写的句子带进来的二义；改动只有一处、一个词，不牵连别的行。
2. 换上之后可以再改：`command-safety-by-review.md:17` 指向「`task-server/task_server_lib.py` 文件头「错误类型」表的 `stalled` 一行」，今天指不到。
   - 现状（原样输出）：`grep -n '^错误类型' task-server/task_server_lib.py | wc -l` 得 0；`git show HEAD:task-server/task_server_lib.py | grep -n '^错误类型'` 得 `90:错误类型（封闭的表；每次尝试记一种，……`；工作区与暂存区一致（`git diff --stat task-server/task_server_lib.py` 无输出）。即 HEAD 的文件头有「错误类型」表、此刻的版本没有，`stalled` 那一行现在在异常表里：「stalled｜打回｜宽限过后日志与 CPU 都满 STALL_SECONDS（10 分钟）不涨」（`task-server/task_server_lib.py:2141`），异常表自己写明它接了错误类型表的活：「错误类型表里除 succeeded 之外的每一种结局」（`task-server/task_server_lib.py:2072`）。
   - 不是压缩带来的：这个指针原件就这么写：「表的 `stalled` 一行），经」（`.claude/agent-common.md:81`），`.claude/rules/long-run-split.md:34` 的「文件头「错误类型」表 needs-attention 一行」同样指不到（合稿同文件 `research/prompts/rules-compress-r3-merged/dot-claude/rules/long-run-split.md:26` 原样带着）。所以照第三版做与照原件做没有不同，只是第三版在 `command-safety-by-review.md:17` 又添了一处；改法是把「文件头「错误类型」表」换成「异常表」，一次扫一遍这三处与 `dispatch` 技能里同样的写法。
   - 为什么不挡：脚本一侧此刻的文件头已经变了，主 agent 在记录里写了「换上时以文件头的段落为准」；文件头本身又在动，不如换上之后一起扫。
3. 换上之后可以再改：`command-safety-by-review.md:9` 里「Python 里每一轮 `os.killpg` 停自己……钩子放行」依据在钩子代码注释（第 4114 行），不在文件头 ⑥ 的字面里，也不在定案第 20 条的字面里（4.3）。等用户一句话（修稿报告 ④.1 已问）。
4. 换上之后可以再改（可不改）：`show-me-test-by-review.md:7` 括号里的「钩子、门禁与 `scripts/` 下非装置脚本」没写是举例还是穷举（4.6 第 2 点）。

## 六、改后比依据多出来的、少掉的半句（这一组，只列补丁带的与第三版新写的）

多出来：

| # | 位置 | 半句 | 依据有没有 |
|---|---|---|---|
| 1 | command-safety-by-review.md:9 | 「Python 里每一轮 `os.killpg` 停自己用新会话起的那一个子进程的组，钩子放行」 | 钩子代码注释有（:4114、:4333），文件头 ⑥ 与定案第 20 条字面没有；见 4.3 |
| 2 | command-safety-by-review.md:17 | 「它停任务，不属于「检出和处置分开」那一条说的检出」 | 无定案；依据原件第 21 行检出对象列的是 hook 与看门狗脚本，异常表 stalled 行「服务停了它」；只消歧 |
| 3 | command-safety-by-review.md:17 | 「`stalled` 一行」指针 | 原件共用约束同写法，今天指不到；见「五」第 2 条 |
| 4 | show-me-test-by-review.md:7 | 「入库装置的变异行进 `crates/mutations.tsv`」 | 依据在 `.claude/agents/experiment-runner.md:30、35`，规则里没有这个词的定义；见「五」第 1 条 |
| 5 | show-me-test-by-review.md:7 | 「没有变异表收的测试……用弄坏开关证红」 | 原件 `.claude/rules/test-script-discipline.md:43`「配一个弄坏开关或一条变异」+ 上游 `show-me-test.md` 「每一个弄坏开关都必须会红」那一节；不改做法 |
| 6 | session-wrapup.md:37 | 「确要跑整轮门禁的」前提 | 定案第 18、24 条 + 原件 `verification.md:33`；已登（常驻-8） |
| 7 | commit-stages.md:21、crash-test SKILL.md:12、gate SKILL.md:10 | 「提交之外」的确认指针 | 定案第 18、24 条；各指到同一处，不另写 |
| 8 | staged-gate.md:17、:18 | 「钩子弹窗」「钩子不弹窗」 | 钩子文件头 `run-scope-guard.sh:34`、`:47`；服务 `task_server_lib.py:2092` |
| 9 | code-discipline-by-review.md:4 | 「只写上游没写的」（第二版就有，不是这一轮补丁新加） | 原件没有；范围说明，不规定做法 |

少掉：

| # | 位置 | 少掉的 | 判断 |
|---|---|---|---|
| 1 | staged-gate.md:17 | 「（用户要求时 `=user-request`）」 | 前缀已退役（`decisions-history.md:6627`），对 |
| 2 | show-me-test-by-review.md | 原件「commit message 里的叙述是没有 harness 时的退路，不是首选」（`.claude/rules/show-me-test-by-review.md:13`） | 定案第 4 条替掉，不是这一轮补丁，已登（用户定-4） |
| 3 | command-safety-by-review.md:8 | 文件头认的 Python `process_group=0`、`os.getpgid(<子进程>.pid)` | 规则末尾写「以钩子文件头 ⑥ 为准」，不算丢 |
| 4 | test-script-discipline.md:4 | 原件括号「新测试先证明会红」的描述 | 上游没有这一节名，项目节名已写在同句前面，不算丢 |

这一组没有落点的定案：第 23、25、26 条不落在这一组的文件里（23、26 落 `implementation-workflow.md` 与 `three-way-inference.md`，25 落入口与共用约束）；第 24 条落在 `session-wrapup.md:37`、`crash-test/SKILL.md:12`、`gate/SKILL.md:10`、`staged-gate.md:17、:18`、`commit-stages.md:21`，整句读完都对（4.5、4.8 到 4.12）；第 20 条落在 `command-safety-by-review.md:8、:9`（4.2、4.3）；第 21 条的意思没被这一轮补丁动。

## 七、三格的结论与证据

### 格一 第二轮修补对不对：成立

- 放回的：这一组里只有 `setsid` 那一句是「改回」性质（4.2）：与钩子文件头 ⑥ 同一回事；定案第 24 条的几处（4.5、4.8 到 4.12）与那一条的两半同一回事；第 20 条的 `setsid` 与「别的组、先挑再发」两句对得上，循环那一句见 4.3。
- 没有一处「与原件或文件头不符而做法因此不同」：`stalled` 那句的做法（服务判卡住并停任务）与异常表 stalled 行一致，错的只是指针的名字（格二）。
- 什么现象会推翻：用户说第 20 条要连 Python 循环里的 `os.killpg` 也拒——那 4.3 的半句就是落偏；或钩子 ⑥ 文件头改写了放行行。

### 格二 修补引出的新问题：不成立（打中两处）

- 同一条判据又住了两处：没有新住。「提交之外要确认」的句子只在 `verification.md:38`（权威句）全写，别处都是指针加半句事实（钩子弹或不弹）；半句事实与那一句同向。循环逐个发的写法在规则、共用约束、工具员定义三处（`R3/agent-common.md:67、:80`、`R3/agents/tooling-writer.md:33`，不在这一组）措辞不同但读出来同向，我没发现做法不同。
- 指针指不指得到、唯一：上游 36 处加 5 个后续节名全部指得到、唯一（「三」）；这一组指向合稿别处的 4 处也指得到；`stalled` 那一处指不到（「五」第 2 条）。
- 新写的句子读出两种意思：打中一处（「入库装置」，「五」第 1 条）。
- 什么现象会推翻：在 `show-me-test-by-review.md` 或规则目录里找到「入库装置」的定义；或 `task_server_lib.py` 文件头里找到「错误类型」表。我找了前者（`grep -rn 入库装置 R3/rules R3/CLAUDE-merged.md` 只出 `three-way-inference.md:36` 与 `show-me-test-by-review.md:7`，都不是定义）和后者（0 行以「错误类型」起头）。

### 格三 能不能换上：成立

- 这一组的 12 处与指针改动，照第三版做与照原件做不同的地方：循环/`setsid` 写法（用户定-20、常驻-10、主 agent 处置-11）、卡住由服务判（常驻-12）、变异清单路径与弄坏开关（合稿-5、常驻-20、用户定-4）、提交之外的确认（常驻-8、入口-1、用户定-18、-24）、删残句（常驻-21）、整份上游（常驻-22、按需-M4）；每一处在矛盾总表里都有行（`research/prompts/rules-compress-r3-merged/contradictions.md`）。指针改动都不改做法。没找到没登的不同。
- 什么现象会推翻：在这 12 处里找到一句话，照它做与照原件做结果不同、而总表与定案都没登。我对每一处做的对照：改后的「不」「只」「先」「才」「一律」各句（4.3 的「一律拒」、4.5 的「同意了才跑」、4.12 的「先」、4.10 的「其余时候不跑」）逐句对过原件，同向。

## 八、要主 agent 跑的命令

无必需。三个只读文本脚本我已直接在本机跑过（读仓里文件与合稿树，输出到标准输出；不写仓、不起进程；`python3 -I`）；主 agent 要复核可以交下面三条（草稿目录 `~/.local/share/singlefs/work/rules-compress-residents-r3-sonnet/`）：

- `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/rules-compress-residents-r3-sonnet/pointer_table.py`：输出 36 行（指针逐处：合稿行、上游标题行、个数；同一行上的重复指针只报首次出现的行号），个数一列全是 1。sha256 `402faf1d24f49b675a285562520321fb85644b9727fc7ba365e0082e6f26aebc`。
- `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/rules-compress-residents-r3-sonnet/check_pointers.py`：每处 exact=1（它也扫补丁里不在这一组的文件，另有 contradictions.md 里一行说明文字被它当成指针、exact=0，不是真指针）。sha256 `16da677a50ced51d09848a978d27229e9a317759abc6efeee97922d0e29a620c`。
- `task-server/task submit -- python3 -I /home/fy5090/.local/share/singlefs/work/rules-compress-residents-r3-sonnet/check_internal.py`：这一组指向合稿别处的节名，末行 `total 117 bad 7`，其中 6 行是指向 `main-agent.md` 派发表行名（不是标题，是表里行名，我另用 grep 在 `R3/main-agent.md` 第 96、98、100 行核到）与 1 行 `format-evolution.md` 指向的 kb 文件不在合稿树里，都不是真缺。sha256 `e9a7a9b7451ed92f5ec7f9c56178073371b81fb684678050efb45be6f8af6632`。

## 九、判定一览

| 格 / 项 | 判定 | 一句话 |
|---|---|---|
| 指针 36 处（上游今天的标题、各一个） | 成立 | 全部指得到、唯一，节名逐字等于标题 |
| 4.1 code-discipline-by-review.md:4 | 成立 | 删自称，不改做法 |
| 4.2 command-safety-by-review.md:8 | 成立 | `setsid [--wait] <命令> &` 与钩子 ⑥ 同一回事 |
| 4.3 command-safety-by-review.md:9 | 成立（一个半句标出） | Python 循环放行半句依据在钩子代码注释，等用户一句话 |
| 4.4 command-safety-by-review.md:17 | 做法成立，指针不成立 | 「错误类型」表今天的文件头里没有；原件同写法，换上之后可以再改 |
| 4.5 session-wrapup.md:37 | 成立 | 第 18、24 条同一回事 |
| 4.6 show-me-test-by-review.md:7 | 做法成立，读法不成立 | 「入库装置」无定义，换上之前必须改一个词 |
| 4.7 test-script-discipline.md:4 | 成立 | 整份上游，与原件同 |
| 4.8 到 4.10、4.12 | 成立 | 第 24 条的指针与半句事实对得上钩子与服务 |
| 4.11 staged-gate.md:17 | 成立 | 删退役前缀残句，钩子弹窗属实 |
| 格一 | 成立 | 与原件、文件头、定案第 24 条同一回事；第 20 条一个半句待用户 |
| 格二 | 不成立（打中 2 处） | 「入库装置」二义（换上之前改）、`stalled` 指针（换上之后改） |
| 格三 | 成立 | 这一组的不同全在总表与定案里，没找到没登的 |
| 这一组能不能换上 | 能 | 换上之前改一处（`show-me-test-by-review.md:7` 的「入库装置」），其余换上之后可以再改 |

## 没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决；没往统一任务服务交任务，没起模型、装置，判决要引的数没复跑（命令在「八」）。
- 上游副本与上游本体是否一致没核（记录里写「上游压完同步副本时再统一核」）；这一组之外的文件（入口、共用约束、定义、`verification`、`implementation-workflow`、`three-way-inference`、`verification-reference`、`dispatch`、`three-way`、`merge-verify`）没判。
- 没逐句重读这一组 24 份文件的全文去对原件（那是攻方腿的格三）；我读的是补丁改到的行与它们紧邻的句子，以及我顺手 grep 的几处（`用户要求`、`弹窗`、`确认一次`、`错误类型`、`入库装置`）。
