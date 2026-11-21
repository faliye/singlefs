# hook-lib-foldable-reads-governance-r2：判决（2026-11-21）

正文 `research/prompts/_hook-lib-foldable-reads-governance-r2-body.md`，开工快照 `research/prompts/hook-lib-foldable-reads-governance-r2-snapshot/`。这是这一题的第二轮（治理文件轮），攻第一轮判决 `research/prompts/hook-lib-foldable-reads-governance-r1-main-verification.md` 的修补，辩方腿复核那份判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/hook-lib-foldable-reads-governance-r2-opus-output.md` | 第 1 问打中：`script_sourced_libraries` 认不出 `X="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"` 再 `source "$X/…"` 的写法，36 份脚本里 70 条库的 source 行没认出，这些库从来不比、也不进键；新格与被判代码用同一个识别函数，看不见；库的存在性（`lexists`）不进键，先加 source 行后建库那一形此刻把逐字节相同的拷贝判成不同。第 2 问：加一道阶段键会变（不成立）；`.claude/gate.d/fixtures/crates-mutation-replay.sh/` 在不在决定自证一格、不进键（打中，推的）。第 3 问打中：表那一格不挂开关；`own-script-digest-location-joined` 只换自证里那一次调用；「拷文件改回元组相加」在自证里分得出（读自己源码的语法树核那一圈迭代的名字） |
| 辩方（Sonnet） | `research/prompts/hook-lib-foldable-reads-governance-r2-sonnet-output.md` | 第一轮判决四条站得住；「改前由整目录连带罩住」一句够不着：算键库此刻默认是拒算，改前是「没有键、照跑」，那一改把它换成「有键、缺这几份、会复用」；真受影响的是 `selftest-registry.tsv` 两条自证。替代方案：拒算让登记表每行每次现判；收整个目录会把 `.claude/gate.d` 九千多份样本档收进键；断言「没有整目录读、没有拒算」对不收也不拒算的读全绿，要配正向断言 |

核查：

- 开工快照：`sha256sum -c research/prompts/hook-lib-foldable-reads-governance-r2-snapshot/judged-sha256.txt` 4 份都 OK，主工作区此刻就是开工那一刻的树，没另起 snapshot-tree。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/hook-lib-foldable-reads-governance-r2-opus-output.md research/prompts/hook-lib-foldable-reads-governance-r2-sonnet-output.md --background research/prompts/_hook-lib-foldable-reads-governance-r2-body.md`，末行「核了 59 处引文、7 处裸行号，2 处对不上，7 处没判」。对不上的 2 处都在辩方腿：`scripts/gate/admission.py:56` 的原文在第 35 行、`scripts/gate/lib_function_units_script_closure.py:548` 的原文在第 547 行，引文在、行号偏，两份都不在这一轮的快照里（别的会话在改算键库）；不算引错。
- 主 agent 复跑（攻方腿的副本、探针与第一轮修补之前的四份库拷进 `~/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-main-rerun/`，探针 sha256 与腿报告相同，副本里四份共用库与主工作区相同）。命令都是 `task-server/task submit --cwd <复跑目录> -- python3 <复跑目录>/probes/probe_r2.py <复跑目录> <段>`：
  - family 段：type=succeeded exit=0，「SUMMARY family mismatches=0」。此刻代码与第一轮修补之前都判同族（应判不同）的：doc-text 过期 `stage-cells.sh`、doc-text 过期 `doc-text-content-and-history-cells.sh`、code-tooling 过期 selftest-cells、multi-host-run 过期 `peer-host-lib.sh`；「two-step current crash-point-replay-new-library-identical spec=True got=False」（修补之前那一臂 got=True）。
  - keys 段：type=command-failed exit=1，两个基线都「key=None … exit=4」，副本里算不出键，这一段的「unchanged」都是 None 对 None，核不动。主 agent 改在主工作区闭包上现查：`.claude/gate.d/lib/*-cells.sh` 27 份里 24 份与 `scripts/hosts/peer-host-lib.sh` 不在 `selftest-process-guard.py::watch_judgement` 闭包的 data-file 里（在的 3 份是 crash-point-replay 那几份）。
  - switches 段：type=succeeded exit=0，「SUMMARY switches mismatches=0」，各开关红的格与点名的格相同。
  - mutants 段：type=command-failed exit=1（末尾复原副本时缺一份备份文件，各行读数已打出）：「copy-loop-tuple-addition」「copied-constant-tuple-addition」两个变体下自证 exit=0、没有格红；攻方腿写的分辨格在变体下红（「discriminator-cell-with-copy-loop-tuple-addition exit=1」）、没有变体时绿（「discriminator-cell-only exit=0」）；表里拿掉 `preflight.sh` 一项时表那一格与连同库原样拷那一格红。
  - 辩方腿的闭包成员核对（`closure_membership_check.py`，同样拷进复跑目录交）：type=succeeded exit=0，「EXPECTED total=65」，两个入口都是 refused=0、whole-directory-via=0、missing=0、extra=4，多的 4 个是第一轮判决写的那 4 个。
- 核不动的：攻方第 2 问「fixtures 目录在不在不进键」与第 1 问「存在性不进键」的键那一半（keys 段算不出键）；判定那一半（two-step）量到了。

## 二、判决

1. 攻方第 1 问：坐实而采。用 `$X` 引的库不比、不进键，family 段量到四种过期库照判同族，主工作区闭包现查 25 份不在键里；这个缺口第一轮修补之前就在（family 段 base 那一臂相同），但第一轮改法 1 的验收写的是「表里点名三张已知表里每份脚本 source 的每一份库」，没达成，表那一格用同一个识别函数所以照绿。先加 source 行后建库那一形判定回退（此刻判不同、修补之前判同族）坐实；存在性不进键那一半核不动，改法里一并处理。
2. 攻方第 2 问：加一道阶段键会变，不成立这一半照腿判（主 agent 没量键，闭包里 17 道阶段逐份在）；fixtures 目录那一半核不动，照「推的」并进改法。
3. 攻方第 3 问：坐实而采。两个元组相加变体下自证全绿（mutants 段）；腿写的分辨格量到在变体下红、平时绿，采它。表那一格在表里少一项时会红（mutants 段），但文件头没有开关点名它，补一条。
4. 辩方第 4、5 问：第一轮判决第 2 条里「其余几样改前由整目录数据单元连带罩住」改正为：改前算键库对这几处是拒算（没有键、照跑），第一次换上把它变成有键而缺输入；结论（要收全）不变。三个替代方案各有判错的情形，第一轮选的做法照留；「断言闭包」要配正向断言，采进改法 3。
5. 另一处同文件的误拒并进这一轮改法（看门狗检出里三条只读 grep 被拒）：`.claude/hooks/lib_project_code_runs_base.py:905` `TASK_SERVER_LIBRARY_IMPORT` 末一支把任何带引号的 `task_server_lib.py` 文件名当成载入。

## 三、改法

1. `script_sourced_libraries` 认「变量赋成脚本自己所在目录、再 `source "$变量/…"`」这一形（照 36 份脚本里现有的写法逐种认），认不出的 source 行让那一份脚本照「读正文」判（不按名字判同族），不静默跳过；库表 `OWN_SCRIPT_LIBRARY_FILES` 补全，表那一格另加一条不经识别函数的核法（例：逐份脚本数 `source`、`.` 开头的行，认出的条数加登记的例外等于总数）。验收：family 段那四种过期库都判不同；主工作区闭包里 `.claude/gate.d/lib/*-cells.sh` 与 `peer-host-lib.sh` 都在。零轮。
2. 库在不在盘上：表里点名而盘上没有的照「不同」判并让表那一格红，不拿 `lexists` 筛掉；先加 source 行后建库那一形两步各自的判定写成样本格。fixtures 目录在不在决定的那一格，改成不依赖那一处的存在性，或把它读成算键库收得进的形态。零轮。
3. 自证加攻方腿那一格（读自己源码的语法树，核拷文件那一圈只照 `PART_PROBE_COPIED_FILES` 迭代、它只赋一次、是字面串元组）与表少一项的开关；正向断言：自证里核两条闭包应有的文件集合都在（期望集合照三张表、库表与 `PART_PROBE_COPIED_FILES` 现算）这一条放不进共用库自证的（要起算键库），写进报告交主 agent 定落点。零轮。
4. `TASK_SERVER_LIBRARY_IMPORT` 末一支收窄到真载入的写法（`spec_from_file_location`、`SourceFileLoader`、`runpy.run_path`、`exec(open(…` 这类调用里带那个文件名），`grep`、`sed -n`、`wc` 这类只读命令里出现文件名不拒；自证补只读 grep 放行与真载入照拒两样本。零轮。
5. 改法 1–4 都是治理文件，不进产品代码，不做 A/B；改完开第三轮（最后一轮）。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/hooks/lib_project_code_runs.py：判了文件头开关表；表那一格没有开关点名（坐实，改法 3）
- .claude/hooks/lib_project_code_runs_base.py：判了 `OWN_SCRIPT_LIBRARY_FILES` 与 `own_library_digest`，表里 21 份都进键成立；用变量引的库不在表里（坐实，改法 1），`TASK_SERVER_LIBRARY_IMPORT` 误拒只读 grep（改法 4）
- .claude/hooks/lib_project_code_runs_launchers.py：判了 `script_sourced_libraries` 的识别与 `same_content_as_own`；变量引的 source 行认不出、存在性筛掉（坐实，改法 1、2）
- .claude/hooks/lib_project_code_runs_selftest.py：判了新四格与两条新开关红在点名的格（成立）；元组相加变体分不出、fixtures 目录存在性（坐实与推的，改法 2、3）

## 回看决策

不涉及决策：这一轮判的是项目代码闸共用库的修补，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
