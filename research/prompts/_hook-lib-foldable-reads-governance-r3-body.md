# 钩子共用库按名单读本仓文件：治理文件轮第三轮（最后一轮），攻第二轮判决的修补（hook-lib-foldable-reads-governance-r3，2026-11-21）

这一轮攻的是第二轮判决（`research/prompts/hook-lib-foldable-reads-governance-r2-main-verification.md`）「三、改法」四条已换上主工作区的修补；这是这一题的最后一轮，这一轮新冒出来的形态不再开第四轮，标零轮。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码，改的是项目代码闸的共用库，`crates/` 下一个字节没动。前两轮攻过而没打中的角度不重复：判定变化只在已知两处、派发起法那一支走不到、三张字面表与盘上对得上、加一道门禁阶段键会变、各开关红在点名的格。

## 问题

1. 每一处修补做的是不是第二轮判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 前两轮正文或判决里只抽查过、没看过的地方，与「实现今天的样子」每一条（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 595 行）：

- 改法 1：`.claude/hooks/lib_project_code_runs_launchers.py` 新识别法 `shell_source_lines`，照仓里现有写法认「变量赋成脚本自己所在目录、再 source "$变量/…"」几种（`X="$(cd "$(dirname "$0")" && pwd)"`、`$(cd "$X/../.." && pwd)`、脚本自己的路径再 `dirname`、`L="$X/lib"`、顶层 cd 之后的相对路径；库看得到入口设的变量）；认不出的 source 行先查登记表 `OWN_SCRIPT_UNRECOGNIZED_SOURCES`（10 条，`.claude/hooks/lib_project_code_runs_base.py`），没登记的让那一份拷贝照不同判（读正文）。库表 `OWN_SCRIPT_LIBRARY_FILES` 由 21 份补到 63 份。表那一格另加一条不经识别函数的核法（逐行数行首的 `source` / `.`）。
- 改法 2：本仓那一边挑不到库（表外的、表里点名而盘上没有的）一律判不同，表那一格不再用 `lexists` 筛；「先加 source 行、后建库」两步各一格样本；依赖 `.claude/gate.d/fixtures/crates-mutation-replay.sh/` 在不在的那一格改成把仓里的阶段源码拷进自证自己造的门禁目录里判。
- 改法 3：自证加「拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、它只赋一次、是字面串元组」一格（读自证自己的语法树，按循环体里的 `shutil.copy*` 认那一圈），开关 `part-probe-copy-loop-tuple-added`、`part-probe-copied-files-tuple-added`；表少一项配 `own-library-table-short`。
- 改法 4：`TASK_SERVER_LIBRARY_IMPORT` 那一判收窄：按模块名载入照拒；按文件载入要同时有载入调用（`spec_from_file_location`、`SourceFileLoader`、`run_path`、`exec(open(…` 这类）与文件名才拒；只读的 grep、sed -n、wc 里出现文件名放行。单独配开关 `task-server-library-file-loads-unjudged`。
- 文件头开关表 91 条。换上：干跑、真换各单独交、都退 0，四份与草稿 `~/.local/share/singlefs/work/hook-lib-r2-fix-b/new/` 逐字节相同；共用库自证、项目代码闸自证退 0；开关复跑「复跑了 91 条开关（1 份），自证红且点名的格与实际红的格逐格相同 91 条」；两条闭包（入口 `scripts/gate/selftest-process-guard.py::watch_judgement` 与 `.claude/hooks/lib_project_code_runs_selftest.py::selftest`）都是 refused 0 行、via 里没有目录、data-file 109 个、`.claude/gate.d/lib/*-cells.sh` 27 份与 `scripts/hosts/peer-host-lib.sh` 都在。
- 工具员留给主 agent 定、这一轮一并判的：「两条闭包应有的文件集合都在」这条正向断言放不进共用库自证（要起算键库），落点没定；`.claude/hooks/project_code_guard_selftest.py` 第 1081 行起那一格同样依赖 fixtures 目录在不在，没动。
- 改前原样：草稿 `~/.local/share/singlefs/work/hook-lib-r2-fix-b/base/`（第二轮修补之前）、`~/.local/share/singlefs/work/hook-lib-keys-cover-libraries/base/`（第一轮修补之前）；第二轮修补的工具员报告 `~/.local/share/singlefs/work/hook-lib-r2-fix-b/result.md`，识别法的原型与审计在同目录 `scratch/`。
- 第二轮核不动、这一轮接着看的：键那一半（库的存在性、fixtures 目录在不在进不进键）在攻方腿的副本上算不出键（`key` 退 4）；主工作区上 `closure` 算得出。

## 范围

这一轮被判的文件（4 份，逐个列在 `research/prompts/hook-lib-foldable-reads-governance-r3-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 改法 1 的识别法：仓里三张表 36 份脚本之外的写法（新脚本会用的 `readonly X=…`、`local`、`source <(…)`、`. "${X}/…"`、函数里 source、条件里 source）认不认，认不出时是不是落到「照不同判」而不是静默跳过；登记表 `OWN_SCRIPT_UNRECOGNIZED_SOURCES` 那 10 条各是不是真认不出、登记了之后那一条引的库还比不比、进不进键；识别法把不是库的东西认成库（误认）会不会让该判同族的判成不同、因而拒了该放的命令。
2. 改法 2：表外的库一律判不同，有没有一份现役脚本因此从「按名字判」掉成「读正文」而被项目代码闸拒（给出命令）；「先加 source 行、后建库」样本测的是不是判决说的那两步；fixtures 那一格换了做法之后还测不测得到原来那件事。
3. 改法 3、4：分辨格读自己的语法树，换一种等价写法（`for` 换 `while`、`map`、先赋给另一个名字）躲不躲得开；改法 4 收窄之后有没有一种真载入的写法漏过（`importlib.machinery`、`imp.load_source`、`__import__`、`open(...).read()` 再 `exec`、shell 里 `python3 task-server/task_server_lib.py` 直接起），有没有只读命令仍被拒。

正推腿（逐条回「成立」「不成立」「推不出」）：

4. 「实现今天的样子」每一条：今天的文件是不是这样写的（写出 文件:行 与那一句原文）。
5. 前两轮判决里写了「核不动」「推的」的地方逐条现查：主工作区上用 `closure` 能核的（库表 63 份是不是都在 data-file、10 条登记的 source 行引的库在不在）写成要主 agent 跑的命令；工具员留下没定的两处（闭包正向断言的落点、`project_code_guard_selftest.py` 那一格）各给一个判断与依据。

## 分工

攻方腿做第 1 到 3 问，正推腿做第 4、5 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
