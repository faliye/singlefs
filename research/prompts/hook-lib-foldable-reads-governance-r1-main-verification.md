# hook-lib-foldable-reads-governance-r1：判决（2026-11-21）

正文 `research/prompts/_hook-lib-foldable-reads-governance-r1-body.md`，开工快照 `research/prompts/hook-lib-foldable-reads-governance-r1-snapshot/`。这是这一题的第一轮（治理文件轮）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/hook-lib-foldable-reads-governance-r1-opus-output.md` | 第 1 问不成立（除工具员写的两处外没有第三处判定不同，两处都只收严）；第 2 问打中（改前整目录数据单元连带罩住的 `.claude/gate.d/lib/stage-cells.sh`、四份 `crash-point-replay-*.sh` 库、`scripts/hosts/heavy-threads.sh`、`.claude/hooks/bash_command_detector_scope_and_index.py` 改后掉出键，判定照样随它们变；库那一半该拒算或收进键；正文「48 个 data-file 都是点名的」与三张表加元组的 44 个对不上）；第 3 问打中（新六格里第 1、2 格在改前代码上照绿，换一个「先查表、查中照旧拼路径读」的变体六格与新开关全绿；第 3 格读目录列举、不进键） |
| 正推（Sonnet） | `research/prompts/hook-lib-foldable-reads-governance-r1-sonnet-output.md` | 第 4 问多数成立；换前三行整目录读、data-file 7、473 种、1492 种、83 条推不出（主 agent 的读数没复核）；「48 个都是点名的单个文件」按字面不成立（点名 44 个）；第 5 问派发起法那一支走不到、库那一半「不收也不拒算」静态走通、仍是推的；开工之后共用库三份被别的会话改过 |

核查：

- 开工快照：快照里 `.claude/hooks/lib_project_code_runs.py`、`_base.py`、`_selftest.py` 在派腿之后被别的会话改过（去掉 `gpu-verification-server/control serves` 那一项与它的一格自证，与这一轮的改法无关），`.claude/gate.d/break-switch-replay-files.tsv` 之后另有改动。倒推：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-main-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py hook-lib-foldable-reads-governance-r1 --out /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-main-rerun/hook-lib-foldable-reads-governance-r1-tree --repository /home/fy5090/code/singlefs`，type=command-failed exit=1，「清单点名 5 份，倒推出 1 份、sha256 对上 1 份；倒推不出 4 份」。共用库四份改用换上草稿 `~/.local/share/singlefs/work/hook-lib-foldable-reads/new/`：`sha256sum -c` 对快照清单四份都 OK；登记表那一份倒推不出，腿只引它第 70 行「八十三条开关」，主 agent 现读那一行，开工后的改动没碰那一句。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/hook-lib-foldable-reads-governance-r1-opus-output.md research/prompts/hook-lib-foldable-reads-governance-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads/new --root /home/fy5090/code/singlefs --background research/prompts/_hook-lib-foldable-reads-governance-r1-body.md`，末行「核了 70 处引文、73 处裸行号，4 处对不上，78 处没判」。对不上的 4 处都是引文在、行号差 1 或 2（selftest 第 1057、1053 行，原文在第 1059、1055 行；base 第 933 行，原文在第 934 行）：腿取的是别的会话改过之后的行号，攻方腿报告里写明了这一偏移；不算引错。
- 主 agent 复跑（攻方腿的两份副本 `copy-base`、`copy-new` 与两条探针拷进 `~/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-main-rerun/`，探针 sha256 与腿报告相同，在副本里交）：
  - 判定对照：`task-server/task submit --cwd <复跑目录> -- python3 <复跑目录>/probes/probe_function_differential.py drive <复跑目录>`。第一次 type=command-failed exit=1，是探针自己的错：共用库的部分模块载入后不留在 `sys.modules`（`.claude/hooks/lib_project_code_runs.py:338`–`:355`），探针 `sys.modules["lib_project_code_runs_launchers"]` 取不到。主 agent 在副本里把两条探针取模块那一处改成从函数的 `__globals__` 取（判定对照）或取入口的再导出（键漏），不动判法。重交 type=succeeded exit=0，末行「SUMMARY inputs=447 differing=28 unexpected=0 predicted-absent=0 reach-true=3」：28 处不同全落在工具员写的两处（启动器名单外的交 False、阶段不在 `GATE_STAGE_FILES` 里的拷贝读正文）。
  - 键漏：`task-server/task submit --cwd <复跑目录> -- python3 <复跑目录>/probes/probe_key_leak.py <复跑目录>`，type=command-failed exit=2。KEY 段四个基线都「baseline key not computed」：副本里那一版算键库对这两个入口拒算（`external-program-unkeyed` herd7、cargo 与 `data-path-unfolded`），算不出键，这一段核不动。FLIP 段三行与预言相同：「stage copy with its libraries vs own .claude/gate.d/lib/stage-cells.sh before=True after=False」「capped.sh copy with heavy-threads.sh vs own scripts/hosts/heavy-threads.sh before=False after=True」「doc-text.sh copy with preflight.sh vs own .claude/scripts/preflight.sh before=True after=False」；FLIP-SELFTEST 改检出钩子字面集合、加一道阶段各让共用库自证红一格。MUTANT 段变体下共用库自证「exit=0 failures=0」，闭包「whole-directory-via=0 refused=1」（`lib_project_code_runs_base.py:496`）。
  - 换上之后主工作区的键：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-sonnet -- python3 /home/fy5090/code/singlefs/scripts/gate/lib_function_units.py closure /home/fy5090/code/singlefs scripts/gate/selftest-process-guard.py::watch_judgement --judging --explain`，type=succeeded exit=0：整目录 via 0 行、refused 0 行、data-file 48 个；48 个里没有 `stage-cells.sh`、`heavy-threads.sh`、`preflight.sh`、`bash_command_detector_scope_and_index.py`，也没有门禁目录的列举。48 个里 44 个是三张表与 `PART_PROBE_COPIED_FILES` 点名的，另 4 个（两张自证登记表、`scripts/kb/replace-once.py`、`task-server/task`）换前就在 7 个里。

## 二、判决

1. 攻方第 1 问：不成立，判对照探针坐实（447 个输入、28 处不同全在已知两处、unexpected=0）。两处都只收严。
2. 攻方第 2 问：坐实而采。FLIP 段量到：改本仓那一份 `stage-cells.sh`、`heavy-threads.sh`、`preflight.sh` 各让判定翻面，改检出钩子 `bash_command_detector_scope_and_index.py` 的字面集合、加一道门禁阶段各让共用库自证红；换上之后主工作区闭包里这几样一个都不在键里，也没有拒算行。所以依赖这几份库的判定会在这几份改了之后照旧复用。`preflight.sh` 改前也不在键里（攻方腿读码推的），是老的漏口；其余几样改前由整目录数据单元连带罩住（改前罩没罩住是攻方腿读码推的，KEY 段核不动），这一改把它们掉出了键。
3. 攻方第 3 问：坐实而采。MUTANT 段变体下自证全绿；第 1、2 格是绿样本，改前代码上同样绿；第 3 格读目录列举，阶段漂移那一刻键不变、红不出来。
4. 正推第 4、5 问：正文「换上之后 data-file 48 个，都是三张表与 `PART_PROBE_COPIED_FILES` 点名的单个文件」写宽了，改正为 44 个点名的加 4 个换前就在的；其余成立，推不出的几个读数主 agent 复跑之外的照腿判。派发起法那一支走不到，坐实（判定对照 reach-true=3 都在 `command_under_dispatch_launcher` 认出之后）。

## 三、改法

1. 键里收全判定真读到的本仓库文件：`same_content_as_own` 逐份比的库（门禁阶段 source 的 `.claude/gate.d/lib/*.sh`、`crash-point-replay-*.sh` 那几份、`scripts/hosts/capped.sh` source 的 `heavy-threads.sh`、`.claude/scripts/preflight.sh`）写成算键库折得出的字面表，或认不出的那一处让算键库拒算；不许不收也不拒算。验收：主工作区同一条闭包的 data-file 里有这几份，改其中任一份键变。零轮。
2. 共用库自证读检出钩子 `bash_command_detector_scope_and_index.py` 那一处改成只读那一份字面路径；「GATE_STAGE_FILES 与门禁目录顶层的阶段逐个相同」那一格读的目录列举进键（或拒算），加一道阶段键要变。零轮。
3. 自证补分得出这一改的格：变体「先查表、查中照旧拼路径读」与「拷文件改回元组相加」下至少一格红（例：断言闭包里没有整目录读、没有拒算，或断言 `own_script_digest` 只读字面路径），配弄坏开关。零轮。
4. 改法 1–3 是治理文件，不进产品代码，不做 A/B；改完照规矩开第二轮。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/gate.d/break-switch-replay-files.tsv：判了第 70 行「八十三条开关」与开关复跑读数一致，成立
- .claude/hooks/lib_project_code_runs.py：判了文件头开关表与再导出，成立；新开关分不出变体（坐实，改法 3）
- .claude/hooks/lib_project_code_runs_base.py：判了 `own_script_digest` 与三张字面表，判定只在两处收严（成立）；库那一半与阶段列举不进键（坐实，改法 1、2）
- .claude/hooks/lib_project_code_runs_launchers.py：判了比内容那一处改调 `own_script_digest`，判定变化只在两处、都收严，成立；库那一半原样，掉出键（坐实，改法 1）
- .claude/hooks/lib_project_code_runs_selftest.py：判了 `PART_PROBE_COPIED_FILES` 与新六格；检出钩子那一处 glob 读掉出键、六格分不出变体（坐实，改法 2、3）

## 回看决策

不涉及决策：这一轮判的是项目代码闸共用库与开关复跑登记表的改动，没有拿依据去推翻或确立 `.claude/kb/decisions.md` 里任何一条。
