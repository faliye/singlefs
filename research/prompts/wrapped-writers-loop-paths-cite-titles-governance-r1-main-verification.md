# wrapped-writers-loop-paths-cite-titles-governance-r1：判决（2026-11-21）

正文 `research/prompts/_wrapped-writers-loop-paths-cite-titles-governance-r1-body.md`，开工快照 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-snapshot/`。这是这一题的第一轮（治理文件轮）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-opus-output.md` | 四问都成立（打中）：A ⑫ ⑯ ⑲ 跟着收严而自证没格、⑦「经只赋过一次的名字引到循环变量」那一支没格、推导式与 lambda 的循环照放而文件头没列、带条件只写新文件的循环被误拒；B 一个 `-c` 串里有两个写工具只取头一个、补丁目录写成 `$p` 或 `cd` 之后的相对路径时结局文件找不到当成「还没开始换」被杀不报（改前照报）；C 「草稿目录被占的可换 --scratch」只在占着的那一趟是 `--dry-run` 时成立、异常表那一行开头一句只对退 9 与仓被占成立；D 标题带括号的小节写法改后整条认不出、不判也不列进没判 |
| 正推（Sonnet） | `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet-output.md` | 第 5 问多数成立；A「名字整段只被这一个 for 绑定」说宽（计数不数 `except … as`、`def`、`class`、match）；B 包着的那一条带五个只读词之一的都不算、命令那一层只排除 `--feed-only`，包着的 `hook-swap.py … --dry-run` 退 10 不出提醒，没格钉；B `cd` 不跟；C 正文把「447 格都对」写在 C 那一条上而记录第 574 行只写退 0（447 的引号读数在第 565 行）；D 标题后紧跟「，」「。」又没右括号的写法会取一整串（探针串） |

核查：

- 开工快照：`sha256sum -c research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-snapshot/judged-sha256.txt` 18 份都 OK，主工作区此刻就是开工那一刻的树，没另起 snapshot-tree。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-opus-output.md research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet-output.md --background research/prompts/_wrapped-writers-loop-paths-cite-titles-governance-r1-body.md`，末行「核了 95 处引文、70 处裸行号，对上 80 处，没判 85 处」，没有对不上的；没判的多是指向仓外工具员 `result.md` 的引用。
- 主 agent 复跑（攻方腿的两份钩子目录与探针拷进 `~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/`，包装脚本里的路径换成复跑目录，`probe-new/hooks` 里检出钩子各份与主工作区相同）：
  - 改动 A 对照：`task-server/task submit --cwd <复跑目录> -- bash <复跑目录>/tools/run-loop-probe.sh`，type=succeeded exit=0，「done base 26」「done new 26」。改前 0、改后 1 的：nested-loops-join、try-in-body、with-in-body、body-assignment-references-loop-variable、exists-guard-writes-only-new-files、suffix-guard-writes-only-markdown、copy-into-directory-loop-source、set-literal、loop-inside-function、path-division-write-text、records-append-loop、one-line-python-c；改前改后都是 0 的：inner-elements-use-outer-variable、tuple-target、enumerate、else-branch、list-comprehension-loop、map-lambda、iterable-in-single-assigned-name、temporary-then-replace、dict-literal、same-name-in-later-comprehension、string-concatenation、combinations-over-limit-target-uses-both。
  - 改动 D：主 agent 在本机直接调 `citations_in`（改前取草稿 `cite-check-title-lazy/base/`），「「伪造的原文」（`.claude/rules/fs-design.md@不为省空间牺牲自包含（附）`）」改前认成标题「不为」（标题命中 0 次，判红）、改后 `[]`（整条认不出、不判不列）；不带括号的长标题改后取全。
  - 改动 C 的两条新开关与改动 B 的三条新开关（正推腿给的五条点格自证，在复跑目录里交）：三条 B 开关下十一格旧格「11 格都对」、四条旧开关下三格 B 新格「3 格都对」、两条 C 开关各只红 `previous-swap-running-reminded` 那一格；整格开关复跑读写判定库，判定库在被别的会话删了重建，还没跑。
  - 改动 B 与 C 两处打中主 agent 现读坐实：`task-server/task_server_scheduler_lifecycle.py:1757`–`:1759` 结局文件按 `os.path.join(cwd, named[0], …)` 拼、不在就交 None（`named[0]` 是 `$p` 或 `cd` 之后的相对路径时落到这一支）；`:1697`–`:1698` 退 10 草稿目录被占那一支出路「或换一个 --scratch」；`task-server/task_server_lib_constants.py:129` 异常表那一行出路「等上一趟写完成再把同一条命令原样交一遍」对退 10 不分。

## 二、判决

1. 攻方第 1 问（改动 A）：坐实而采。对照探针量到 12 种写法改前放、改后拒，其中 ⑲（records-append-loop）、带条件只写新文件两形（exists-guard、suffix-guard）、经只赋过一次的名字引到循环变量一形（body-assignment-references-loop-variable）在自证里没有格钉着；带条件的两形是误拒，误拒的出路（Edit 或写临时文件再 `mv`）在 ⑦ 的拒绝说明里有，判它为「要么认条件、要么文件头写明条件不看、按每个候选判」；推导式与 lambda 写法改前改后都放（list-comprehension-loop、map-lambda），文件头「判不到的」没列这两形。正推腿另报的绑定计数说宽，方向是多不认（偏严）、没构造出漏放的写法，采为文件头写法的修正。
2. 攻方第 2 问（改动 B）：坐实而采两处。甲：补丁目录写成变量或 `cd` 之后的相对路径时，改后结局文件找不到、交 None，被杀不报；改前 writer 落成 `bash`、不进结局文件那一支、照报，这是这一改带进的回退。乙：一个 `-c` 串或脚本正文里有两个写工具时只取头一个（`wrapped_workspace_writer` 认到第一条就交）。`modifies_workspace` 判定没变，成立。正推腿另报的「包着的那一条带只读词就整条不算」坐实为同一类（包着的 `hook-swap.py … --dry-run` 不报），并进改法。
3. 攻方第 3 问（改动 C）：坐实而采。两种拒绝说明的认法成立；出路「或换一个 --scratch」只在占着那一趟是 `--dry-run` 时有用（两趟真换共用一个草稿目录，换了草稿目录照样被仓占着退 10，`scripts/gate/hook-swap.py` 文件头第 115 行），服务提醒与 README 三语第 174 行都要改；异常表那一行出路开头一句退 9、退 10 不分。正推腿另报正文把 447 写在 C 那一条上：正文那一句写宽了（447 是 B 换上之后的读数，C 换上之后整份自证同样是 447，记录第 574 行只写退 0），这里改正。
4. 攻方第 4 问（改动 D）：坐实而采：标题带括号的小节写法改后整条认不出，伪造的原文不判也不列进「没判」，比改前（取短、判红）更松。正推腿另报的「，」「。」那一形是探针串，仓里没有出现过，判它同一个根子（`TITLE_END` 没认的结尾就整条不配或配过头），并进改法。

## 三、改法

1. 改动 A：⑫ ⑯ ⑲ 各补一格按候选判的自证（候选里有一个落在写范围外 / 现行账本 / records 下就拒），⑦ 补「经只赋过一次的名字引到循环变量」一格；带条件的循环在文件头写明「条件不看、按每个候选判」，拒绝说明里写出这一形的出路；文件头「判不到的」补推导式与 lambda 两形，并改正绑定计数那一句（`except … as`、`def`、`class`、match 不计）。零轮。
2. 改动 B：`workspace_writer_command` 改成交认到的全部写工具（不只头一个），三处提醒逐个判；补丁目录的路径照这一条命令里能认出的 `cd` 与赋值算，算不出的（变量、`"$1"`）照「可能写了一半」报，不当成还没开始换；包着的那一条带只读词时只免那一条，同一个串里别的写工具照判。每处补红样本格。零轮。
3. 改动 C：退 10 草稿目录被占那一支出路改成「等占着的那一趟结束再交；占着的那一趟是 --dry-run 的，可以换一个 --scratch」，README 三语第 174 行同改；异常表那一行出路开头一句分开写退 9 与退 10。零轮。
4. 改动 D：小节标题的结尾改成认出标题里的成对括号（取到与开头那一层配对的右括号之后），认不出结尾的不整条丢掉、列进「没判」并写明「小节标题认不出结尾」；自证补标题带括号、后接「，」「。」两形。零轮。
5. 改法 1–4 都是治理文件，不进产品代码，不做 A/B；改完照规矩开第二轮。改动 C 的整格开关复跑等判定库重建之后交。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/hooks/bash-command-detector.sh：判了 ⑦「判不到的」那一句与开关表；漏列推导式与 lambda、绑定计数说宽（坐实，改法 1）
- .claude/hooks/bash_command_detector_literals_and_signals.py：判了按候选逐个判的共用段，⑫ ⑯ ⑲ 跟着收严、带条件循环误拒（坐实，改法 1）
- .claude/hooks/bash_command_detector_scope_and_index.py：判了新开关名登进已知名单，成立
- .claude/hooks/bash_command_detector_selftest_waits_and_scope.py：判了六格该拒六格该放；⑫ ⑯ ⑲ 与经名字引到循环变量那一支没格（坐实，改法 1）
- task-server/task_server_lib.py：判了文件头开关表与两处说明，成立；退 10 说明同改法 3
- task-server/task_server_lib_constants.py：判了新开关名登进已知名单成立；异常表第 129 行出路开头一句退 9 退 10 不分（坐实，改法 3）
- task-server/task_server_scheduler_building.py：判了 `workspace_writer_command` 与 `modifies_workspace` 判定不变（成立）；只取头一个写工具、只读词免整条（坐实，改法 2）
- task-server/task_server_scheduler_lifecycle.py：判了三处提醒改取包着那一条的词；结局文件路径不跟 `cd` 与变量时被杀不报、退 10 出路「换 --scratch」说宽（坐实，改法 2、3）
- task-server/task_server_selftest_memory.py：判了三格新格；没有两个写工具、变量补丁目录的红样本（坐实，改法 2）
- task-server/task_server_selftest.py：判了格名登记与 447 格现数，成立
- task-server/task_server_selftest_notifying.py：判了退 10 七个日志样本与两种拒绝说明现行原文对得上，成立
- task-server/task_server.py：判了文件头 previous-swap-running 两种成因的现状句，成立；出路同改法 3
- task-server/task_server_constants.py：判了 `SWAP_SCRATCH_BUSY_EXIT_CODE` 行尾注释写两种成因，成立
- task-server/README.md：判了第 174 行两句；「换 --scratch」只对占着那一趟是 --dry-run 时成立（坐实，改法 3）
- task-server/README.en.md：判了第 174 行同一句，同改法 3
- task-server/README.ja.md：判了第 174 行同一句，同改法 3
- scripts/agents/cite-check.py：判了 `TITLE_END`；标题带括号整条认不出不判不列（坐实，改法 4）
- .claude/gate.d/break-switch-replay-files.tsv：判了第 76、77 行条数与复跑读数一致，成立

## 回看决策

不涉及决策：这一轮判的是检出钩子、统一任务服务与引文核对脚本的改动，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
