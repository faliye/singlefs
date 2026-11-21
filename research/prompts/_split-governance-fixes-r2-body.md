# crates 代码拆分换上的治理文件：治理文件轮第二轮，攻第一轮两份判决的修补（split-governance-fixes-r2，2026-11-20）

这一轮攻的是两份第一轮判决（`research/prompts/split-governance-hooks-gates-r1-main-verification.md`、`research/prompts/split-governance-scripts-services-r1-main-verification.md`）「三、改法」里已换上主工作区的修补，并派辩方腿复核这两份判决。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码；与 `crates/` 有交点的是新共用库 `scripts/gate/lib_split_family_loaded.py` 调的 `scripts/gate/lib_lag_split_family.py` 认 `.rs` 拆法的那一半（这一轮的调用方只认 `.py`、`.sh`）。第一轮攻过而没打中的角度不重复：自证替身换在被测命名空间上（钩子与门禁那一题第 3 问）、task-server 五个混入类、子进程与 `__file__`、两个 Rust 服务的 `control` 读法与 `#[path]`、task-server 三个入口再导出的名字。

## 问题

1. 每一处修补做的是不是判决那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 两份第一轮判决够不够得着：判成坐实的几条，依据（主 agent 复跑的读数）撑不撑得住；判成推翻、不采、没打中的几条，判得对不对（辩方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 514–517 行）：

- 四份钩子引导（`write-guard.sh`、`runner-dispatch-guard.sh`、`completion-cleanup-guard.sh`、`bash-command-detector.sh`）改成先读进并编过每份部分文件、全编成了才 exec；三份闸读不到或编不过退 2 并点名缺的那一份，检出钩子照它文件头「判不了的（放行但必记检出）」处置（引导退 70，bash 一层放行并记检出）。各加一条只在自证里认的 `part-unreadable-passes`。换上后五份自证绿，开关复跑「复跑了 420 条开关（4 份）…逐格相同 420 条」。
- `lib_shell_words.py` 的部分模块按真实目录各存一份（`sys.modules["<名>@<目录>"]`），载入后核 `__file__` 在本目录；同目录载入两次是同一份（`lib_project_code_runs_launchers.py` 里有 `isinstance(word, shell_words.ShellWord)`）。
- 另三处同写法的引导（`project-code-guard.sh`、`lib_run_scope.py`、`lib_project_code_runs.py`）改成先读进并编过：闸退 2 点名；两份共用库抛 `PartFileUnreadable`（`ImportError` 的子类），`lib_run_scope.py` 当入口起时退 70、`run-scope-guard.sh` 照「判法没跑完」放行并记检出。换上后范围声明闸自证红在开关 `judge-crash-unrecorded` 漏点名两格，主 agent 照实测补进文件头与开关表后三份自证都绿，开关复跑两份 230 条逐格相同。
- 认拆分族只剩一份：`scripts/gate/lib_split_family_loaded.py` 的 `loaded_family(入口, 候选)` 取命名族（`lib_lag_split_family.in_family`）与「入口或族里别的文件真载入」的交集；`.claude/hooks/lib_project_code_runs_launchers.py` 的 `source_family`、`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh` 的 agent-write-scope 格、`scripts/hosts/queue-health.py` 的 `entry_with_split_family` 改调它。agent-write-scope 格的读法变了一处：跳过的文件名取被判仓自己 `.claude/hooks/` 里入口连同它载入的部分，查的函数名仍取这一道脚本旁边那一族。新样本 `agent-write-scope-name-like-red`、`agent-write-scope-loaded-shared-red`。换上后整档样本里 agent-write-scope 十二份都判对，queue-health 自证「44 格全过；弄坏开关 33 个逐个打开都红在点名的格」，开关复跑三份 267 条逐格相同；agent-write-scope 格的开关复跑开跑前算键全拒算，没跑。
- queue-health 开关 `unread-cgroup-as-zero` 的点名补了第三项「✗ 任务 501 第 2 次 · 本机：现算与 cgroup 对不上」。
- `project-code-guard.sh` 两条开关的头注从「推的，没复跑」改成现状（第一轮判决第 6 条），这一批三条新开关的头注改成实测。
- 第一轮判决里交给别的会话的几条（change-range-single-source 扫描范围归 mainagent(C)，已换上；变异判法两处归 mainagent(B)；`admission_unit_key.py` 那一处等 mainagent(C) 算键库第二轮）不在这一轮的射程里。判决第 4 条（verus 计数替身）推翻、不改。
- 已知开着、各有会话在修、这一轮不重复报的：break-switch 三格把部分文件当独立脚本判（mainagent(B)）；算键闭包跨不过拆出的文件（mainagent(C)）。

## 范围

这一轮被判的文件（58 份，逐个列在 `research/prompts/split-governance-fixes-r2-snapshot/judged-sha256.txt`；都读主工作区此刻那一版，要看修补改了哪几行，对着第一轮快照 `research/prompts/split-governance-hooks-gates-r1-snapshot/judged-sha256.txt` 与各修补草稿目录里的 `base/` 比）。两份第一轮判决与它们引的主 agent 复跑读数也在辩方腿的射程里。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 四份钩子与另三处引导：「先读进并编过、全编成了才 exec」有没有一种部分文件的状态（读得到而 exec 时抛错、编得过而运行到一半缺名字、部分文件是符号链接或目录、权限变了）让闸照常放行；三份闸在部分文件读不到时拒一切调用，出路走不走得通（写闸挂 Write、Edit，项目代码闸挂 Bash，二者同时读不到时主 agent 还剩什么路）。
2. `lib_shell_words.py` 按目录各存一份：同目录用不同写法的路径（符号链接、`..`、相对路径）载入两次，是不是同一份；不同目录而 `ShellWord` 类不同时，`isinstance` 判错的调用方有没有。
3. `lib_split_family_loaded.loaded_family`：命名族那一半调的是 `lib_lag_split_family.in_family`，它认门禁阶段 `lib/` 拆法时按仓内相对路径（`.claude/gate.d`）比；调用方给绝对路径时这一半还认不认得；载入那一半的几种写法（`import a.b`、`from . import`、注释与字符串里的 `import`、引号里写死的文件名恰好是别的脚本）会不会多认或少认，给一份入口使三处调用方里任一处因此判错。
4. agent-write-scope 格「跳过的文件名取被判仓自己的族、查的函数名取脚本旁边那一族」：两族不同时（样本仓、真仓里部分文件刚加还没换上）有没有一种布局让手抄漏报或误报。

辩方腿（逐条回「成立」「不成立」「推不出」）：

5. 两份第一轮判决「二、判决」每一条：判成坐实而采的，主 agent 引的复跑读数是不是真的分辨得出（拆前拆后两臂、有无修补两臂），有没有一条的读数在别的原因下也会这样；判成推翻、不采、没打中的（钩子与门禁那一题第 7、8 条，脚本与服务那一题第 4、6、7、8 条），判得对不对，替被判出局的一方辩护。
6. 第一轮改法第 4 条（认族并成一份）的定义从「入口真载入」改成「命名与载入的交集」，这一改是不是同样打中所有替代方案：只按载入、只按命名、按 `__all__` 或再导出名单认，各在哪一种布局下判错。

## 分工

攻方腿做第 1 到 4 问，辩方腿做第 5、6 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
