# d137-common-temporary-service-r3：共用约束里「要临时服务的自证怎么写命令」那一句——第三轮（最终轮，2026-11-05）

## 一、题面

前两轮判决：`research/prompts/d137-common-temporary-service-r1-main-verification.md`、`research/prompts/d137-common-temporary-service-r2-main-verification.md`。照第二轮判决，`.claude/agent-common.md` 第 64 行那一句现在是：

> 登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本（仓副本里的同一份也算），写进报告的自证命令照那张表文件头 required 一条给的整条写法套临时服务（`<解释器>`：`.py` 用 `python3`，别的用 `bash`；这一层本身不算「交给服务的命令不带包装前缀」那一串里的包装，`--` 后面那条照样不带）

第二轮交出去的几样，开这一轮时已由别的会话落地：A1 的根（统一任务服务把不带 `--server-binary` 的包装认成会编的、两台都给编译目录，`task-server/task_server_lib.py` 的 `temporary_service_builds_server`，`task-server/task_server.py` 的 `compiles` 调它）；登记表文件头改写（旧钩子名换成 `project-code-guard.sh`、`lib_project_code_runs.py`；own-server 那一句、拒绝说明、带不带二进制的说法改过）。A4（绑住表头写法与包装用法的检查）、A5（`scripts/agents/dispatch-prompt.py` 骨架占位）登在拆分提案一百九十七节，这一轮不判它们本身。

这一轮照 `.claude/rules/three-way-inference.md`「多轮」第三轮分工：「已定」的形态撑不撑得住、未定项清单完不完整、前两轮报出的数能不能核。第三轮为最终轮：只攻前两轮站住的形态，不重复前两轮攻过的角度（第一轮 H1–H7、正推缺口 1–4，第二轮 R 格五条、A1–A5 不再原样提）；这一轮新冒出来的零轮形态不再为它开一轮，判决里标「零轮」交出去。

被判的文件（快照 `research/prompts/d137-common-temporary-service-r3-snapshot/files-sha256.txt`）：`.claude/agent-common.md`、`.claude/hooks/selftest-temporary-service.tsv`、`.claude/hooks/lib_project_code_runs.py`、`.claude/hooks/project-code-guard.sh`、`scripts/agents/dispatch-prompt.py`、`scripts/prune-server/prune_server_client_selftest.py`、`task-server/task_server.py`、`task-server/task_server_lib.py`、前两轮判决。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

- 这一批不改 `crates/` 的源码；`grep -rn selftest-temporary-service crates/` 零命中。
- 登记表 24 行 required、2 行 own-server（`grep -c` 数）。
- 登记表文件头第 5 行整条写法：`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <编好的 prune-server> -- <解释器> <脚本> --selftest`；第 6 行写不带 `--server-binary` 时任务服务把这一条认成会编的、两台都给编译目录，包装现编；不经任务服务起、没有 `CARGO_TARGET_DIR` 时退 2。
- `.claude/hooks/lib_project_code_runs.py` 的 `temporary_service_bare_selftest` 拒绝说明给的整条写法带 `--server-binary` 占位，并写「换成编好的 prune-server 二进制的路径」。
- 改表头的那一路会话的说法（转述，当线索）：子 agent 照写第 5 行那种（带 `--server-binary`），第 6 行是退路（两台都给编译目录、每次现编一遍）。它说骨架那一行写「有编好的 prune-server 二进制时在 --with-temporary-service 后面加 --server-binary」，主 agent 现查不是：`scripts/agents/dispatch-prompt.py:295` 的 `TEMPORARY_SERVICE_WRAPPER` 是「python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <编好的 prune-server> --」，骨架那一行括注「--server-binary 后面是编好的 prune-server 二进制，没给它又没有任务服务给的编译目录时包装不现编、退 2」。
- 统一任务服务在 `temporary_service_builds_server` 落地之后重启过；这一轮还没有一条不带二进制的包装任务在落地后的服务上派到本机复跑过（第二轮 6838 在落地之前）。

## 三、格

### R　前两轮站住的形态撑不撑得住（辩方腿）
复核第二轮判决逐条「采」的依据今天还在不在（A1 的根落地之后，6834、6835、6838 那几条读数撑的结论变没变；A2、A3 写进句子的两处说法与 `prune_server_client_selftest.py`、`lib_project_code_runs.py`、`task_server_lib.py` 的源码对不对得上）；替前两轮「不采」「记账」的一方辩护（H4、正推缺口 3、A5 该不该在这一轮采）。核前两轮判决里引的任务号、行号、计数（24 行、14 行 `.sh` 10 行 `.py`、53 处引文这类）能不能由一条命令重新数出来，数不出来的点名。

### A　已定形态与未定清单（攻方腿）
只攻改过两轮之后的那一句与它点到的登记表文件头：表头第 5 行（必带 `--server-binary`）与第 6 行（不带也能跑）给子 agent 的是不是同一个答案，照句子「照那张表文件头 required 一条给的整条写法」读者会写出哪一种、写出的命令在两台、在拒绝说明、在派发骨架里各落到什么结局；前两轮判决的交出表（未定、交别的会话、记账的几行）有没有漏掉一样，漏的点名属于哪一路；前两轮报的数里哪一个是推的、没量过（例：第二轮「本机会照 6838 退 2」写着推的）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿 | R | 复核前两轮判决站不站得住，替不采的一方辩护，核报出的数 |
| 攻方腿 | A | 只攻两轮修过的形态与交出表的完整性 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引定义、规则、脚本、钩子写主工作区的路径与行号，同时写那份文件此刻的 sha256 前 8 位；快照里的 sha256 与此刻对不上的，写明是哪一份。
- 子 agent 不往统一任务服务交任务：要跑的命令（只读的 `task-server/task log` 除外）拼成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交。
- 不造新的绕过钩子的写法。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
