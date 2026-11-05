# d137-common-temporary-service-r3 判决（第三轮，最终轮，2026-11-05）

被判：`research/prompts/_d137-common-temporary-service-r3-body.md` 第一节——两轮修过之后 `.claude/agent-common.md` 第 64 行那一句与它点到的登记表文件头，连同前两轮判决（`research/prompts/d137-common-temporary-service-r1-main-verification.md`、`research/prompts/d137-common-temporary-service-r2-main-verification.md`）。腿：辩方腿 `research/prompts/d137-common-temporary-service-r3-sonnet-output.md`（R 格），攻方腿 `research/prompts/d137-common-temporary-service-r3-opus-output.md`（A 格）。最终轮：这一轮之后不再开轮。

## 核查怎么做的

- 开工快照倒推：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d137-cts-r3-verify -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py d137-common-temporary-service-r3 --out /home/fy5090/.local/share/singlefs/work/d137-cts-r3-verify/d137-common-temporary-service-r3-tree --repository /home/fy5090/code/singlefs`（任务 7113）：「清单点名 10 份，倒推出 7 份、sha256 对上 7 份；倒推不出 3 份、对不上 0 份」。倒推不出的三份：
  - `.claude/hooks/lib_project_code_runs.py`：腿跑着时改钩子那一路照用户弹窗所定改了三行（第 1066 行子命令集合加 `pause`、`resume`，第 3437–3438 行两条样本），改动清单在 `research/prompts/d137-common-temporary-service-r3-snapshot/foreign-edit-lib_project_code_runs.diff`，改前整份 sha256 等于快照（主 agent 现核）。腿引这份的行（434、732、1691、1768、1786–1788）都不落在那三行上。
  - `task-server/task_server.py`、`task-server/task_server_lib.py`：开轮之后做统一任务服务的会话又改过（此刻前 8 位 45b9064f、839ad871），基底不在 diff 里。腿引的承重几行主 agent 照此刻工作区按名字逐处人工核：`temporary_service_split` 在 `task_server_lib.py:1373`，`temporary_service_builds_server` 在 `:1390`，`task_server.py:4713` 的 `compiles` 调它；`task_server.py:134` 那一句注释原样在（「任务 6778 与 6834、6835 同一条命令：派本机的 compiles 为 false、不给编译目录、包装退 2」）。
- 引文：`python3 scripts/agents/cite-check.py <腿报告> --root /home/fy5090/.local/share/singlefs/work/d137-cts-r3-verify/d137-common-temporary-service-r3-tree --background research/prompts/_d137-common-temporary-service-r3-body.md`，两份分开跑：辩方腿「核了 1 处引文、44 处裸行号，对上 1 处，没判 44 处」，攻方腿「核了 44 处引文、7 处裸行号，对上 28 处，没判 23 处」（没判的引到快照外的文件与倒推不出的三份）。以主工作区为根再跑同一条：辩方腿「对上 1 处，没判 44 处（其中裸行号没有引文可比 40 处）」，攻方腿「核了 44 处引文、7 处裸行号，对上 44 处，没判 7 处」。
- 腿交回的命令，主 agent 经服务交（这一轮 17 条派到哪台由服务定，全部落在第二台，本机一条都没落到）：
  - 喂今天的钩子：攻方 7062「FEED_SUMMARY shapes=7 refused=2」，退 2 的只有两种裸写，`literal-placeholder`（照抄 `<编好的 prune-server>` 占位）、不带二进制、带主仓绝对或相对路径、own-server 裸写都退 0；辩方 7107「R3_HOOK_FEED cases=77 judged=75 mismatches=0 observed=2」，观测的两条（占位原样、包装里层套 `timeout`）两道钩子都退 0。
  - 不带二进制的包装：7063、7064、7065（里层 `true`）、7068（里层客户端自证）都是 `"compiles": true`、退 0、日志「PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1」，编译目录峰值 231849731 字节，现编连起跑用时 34.3–36.8 秒（7063–7065）。之前交的 7030 同样 `"compiles": true`、退 0（「crates-mutation-rows.py 自证 195 格都对」）。
  - 主仓那份二进制起客户端自证（7067）退 1：「PRUNE_SERVER_CLIENT_SELFTEST cells=24 passed=23 failed=1 failed_cells=delete-prefix-by-namespace」，原话「服务拒了操作码 9 的帧（状态 2）：认不得的操作码 9」；同一份自证经不带二进制的包装现编起（7068）：「PRUNE_SERVER_CLIENT_SELFTEST cells=24 passed=24 failed=0」「PRUNE_SERVER_CLIENT_BREAK_SWEEP switches=25 matched=25」。
  - 副本里起：攻方交接副本（只拷 `scripts/prune-server/`）7066 退 1，原因是缺 `.claude/scripts/project_preflight.py`，不是腿推的那一条，作废；主 agent 照 `.claude/agent-common.md:49` 的拷法另拷一份（带 `scripts/`、`.claude/scripts`、`.claude/singlefs-ai-sop`，不带 `prune-server/`）重交 7191：`"compiles": true`、退 2，「✗ 临时 prune-server 起不来：cargo build 退 101：error: manifest path `…/prune-server/Cargo.toml` does not exist」。那份副本 3.8G，读完当场删。
  - own-server 裸起 `prune_server_client_selftest.py --selftest`（7069、7070、7071）：都是 `"compiles": false`，都落第二台、退 0（第二台照例给编译目录）。
  - 24 份 required 照包装写法（不带二进制）逐条交（7072–7095）：24 条全收、提交时一条没拒，收下即取消。
  - 包装外套 `timeout 1800`：当场拒、退 3，「✗ 命令自己套了一层：timeout …：限时由服务定，命令里不套它（去掉这一层）」。
  - 服务自证：7110 退 0；`TASK_SERVER_BREAK=temporary-service-build-unrecognized` 下 7111 退 1，红的唯一一格是「temporary-service-wrapper-compiles：…派本机、不带 --server-binary：compiles 应当是 True…实际 compiles False…；派第二台、不带 --server-binary：…实际 compiles False」。
  - 包装起 `.py`、`.sh` 的 required 自证：`lib_verdict_library.py`（7109）退 0、「lib_verdict_library 自证 17 格都判对；弄坏开关 7 个，每个都红在点名那一格」；`crash-point-replay-shard-run.sh`（7108）里层退 1，「✗ 开关轮：打开 CRASH_POINT_REPLAY_SHARD_RUN_BREAK=unknown-option-accepted 该红的是「unknown-option-refused」，实际红的是「」」：那一格第 238 行起子进程时 `env -u` 去掉了开关，是那份脚本暂存区里新加的一段，不是包装写法的事，已交改它的那一路（认了）。
  - 第二轮倒推树重建（7112）：「清单点名 8 份，倒推出 4 份、sha256 对上 4 份；倒推不出 4 份」——第二轮判决里「对上 53」「对上 33」两个数今天复不出（快照之后有提交改过那几份）。

## 逐条判

| 条 | 腿 | 判 | 主 agent 现查 |
|---|---|---|---|
| R 1.1：第二轮 H1「采」的前提（服务不认这一层会编）变了 | 辩方 | 采 | 两台都给编译目录：源码（`task_server_lib.py:1390`、`task_server.py:4713`）与服务自证格 `temporary-service-wrapper-compiles`（7110 绿、7111 只红这一格，本机与第二台两支都在格里）；真机只在第二台量过（7063–7065、7068、7030），本机 0 条，本机那一侧是源码与自证格撑、真机没量 |
| R 1.1 附：`task_server.py:134` 注释把「6778 本机退 2」写成发生过的事 | 辩方 | 采，交做统一任务服务的会话 | 6778 是本机、退 0、跑的是改包装之前的旧版；「本机退 2」是第二轮判决里标了推的那一句 |
| R 1.2、1.3：鉴别量与证据改引 | 辩方 | 采 | `build_peak` 分得出台（7063 等为数字）；6837「15 格绿」是那时的脚本，今天自证格数组 12 项，不再当现状引 |
| R 1.4：表头旧钩子名已换 | 辩方 | 采 | 表头零命中；`.claude/agent-common.md` 第 64、66、96 行还有 `lib_heavy_tests.py` / `heavy-test-guard.sh`（`grep -n` 量的），不在被判句里，交改钩子名的那一路 |
| R 1.6–1.9：A1 后半、A2、A3、A4 | 辩方 | 采 | A1 后半（写了也没有可写的路径）并进 A6、A7；A2 照旧（7107 里层套 `timeout` 两道钩子放行）；A3 的认法照旧，但副本里起不带二进制的写法退 2（A8）；A4 的检查仍开着 |
| R 2.1：H4 维持不采，理由换成「对象已没有」 | 辩方 | 采 | `grep -rn -F -e 'selftest-real-sharded-run' scripts .claude/hooks .claude/gate.d task-server` 零命中；拆分提案一百九十六节 ② 销账 |
| R 2.2：正推缺口 3 维持不采 | 辩方 | 采 | `.claude/agent-common.md:65` 仍给读法；成功句各脚本不同（6778、6779、6837、7109 四种） |
| R 2.3：A5 维持记账 | 辩方 | 采 | 裸写两道里 `project-code-guard.sh` 都拒（7062 两种裸写退 2，7107 B、S 两类 mismatches=0），后果是多交一趟，不是错判；一百九十七节 ② 照旧开着 |
| R 3：前两轮的数能不能重数 | 辩方 | 采 | 24、2、14、10 与 `crates/` 零命中能由一条命令数对；第一轮「对上 38」由仍在盘上的倒推树复得出；第二轮「对上 53」「对上 33」今天复不出（7112 倒推不出 4 份） |
| A6：句子只给 `<解释器>` 的取法，表头第 5 行（必带 `--server-binary <编好的 prune-server>`）与第 6 行（不带也行）两个答案；拒绝说明、骨架、自证登记表、客户端文件头站第 5 行，门禁 code-tooling 的 runner 表站第 6 行 | 攻方 | 采 | 原文逐处在（`lib_project_code_runs.py:1788`、`dispatch-prompt.py:295`、`selftest-temporary-service.tsv:5`、`:6`、`code-tooling.sh` runner 表 24 份全不带）；第 5 行那种照抄与照填都出错（A7），第 6 行那种 24 条全收、两台给编译目录（本机未量）、现编的新二进制判全对（7068）。定：整条写法统一成不带 `--server-binary` 的一形，`--server-binary` 只在整条之外留一句「自己编了、两台读得到的二进制可以加」；被判句不动，它指的表头改成一形 |
| A7 甲：照抄占位 | 攻方 | 采 | 两道钩子都放行（7062 `literal-placeholder` 退 0、7107 观测退 0），落到 shell 里 `<编好的` 当输入重定向、命令不执行（腿在草稿目录量的 rc=1）；拒绝说明给的就是这一形 |
| A7 丙：照填主仓那份二进制 | 攻方 | 采 | 那份比 12 份 prune-server 源码旧（腿 `find -newer` 量的），自证红一格（7067 `delete-prefix-by-namespace`，认不得操作码 9）：起得来、判不对 |
| A8：照第 49 行拷的副本不带 `prune-server/`，不带二进制的写法退 2 | 攻方 | 采 | 7191 退 2、`Cargo.toml` 不在；第二轮 A3 加「仓副本里的同一份也算」时没连拷法一起改 |
| A9：前两轮交出表漏了 own-server 两行：裸起服务不当会编，本机推的退 2 | 攻方 | 采，本机一侧推的 | 7069–7071 都是 `"compiles": false`，都落第二台、退 0；本机没落到。源码（`task_server_lib.py:1373` 的认法只认包装）撑「本机不给编译目录」；交做统一任务服务的会话 |
| N1、N5、N6：本机给编译目录、退 0 | 攻方 | 采，仍是推的 | 落地后 17 条全落第二台；本机只由源码与服务自证格撑 |
| N3：24 行照写都不被服务退 3 | 攻方 | 采，现已量 | 第一轮引的 6777 只喂钩子；这一轮 7072–7095 24 条全收 |
| N4：24 行照拼全都起得来 | 攻方 | 采，量了 3 份 | 起过 `crates-mutation-rows.py`（7030）、`lib_verdict_library.py`（7109）、`crash-point-replay-shard-run.sh`（7108，包装起来了，里层自己的开关轮红）；其余 21 份没起过，不写「全都起得来」 |
| N7：现编一次的代价 | 攻方 | 采，现已量 | 编译目录峰值 231849731 字节；里层 `true` 连现编 34.3–36.8 秒（第二台） |
| N2：主仓二进制谁编的 | 攻方 | 采 | 仍推不出；这一轮引的是它比源码旧（A7 丙） |

## 改法（零轮）

被判句不改。

| 改法 | 零轮 | 去向 |
|---|---|---|
| 表头第 5、6 行，拒绝说明（`lib_project_code_runs.py` 的 `temporary_service_command` 与括注），骨架常量与括注（`dispatch-prompt.py:295`、`:330`），自证登记表第 10 行与理由，客户端文件头（`prune_server_client.py:112`）：整条写法统一成不带 `--server-binary` 的一形（A6、A7） | 零轮 | 交改钩子名与表头的那一路（这几份它在改）；被判句随表头，不另改 |
| 第 49 行副本拷法：要在副本里起登记表里自证的，连 `prune-server/`（不带 `target`）一起拷（A8） | 零轮 | 改共用约束，登进拆分提案，下一批定义改动时并进、照规矩走三方或用户豁免 |
| 服务把 own-server 两份不带二进制的单独 `--selftest` 也认成会编的（A9） | 零轮 | 交做统一任务服务的会话 |
| `task_server.py:134` 注释改成推的或删 | — | 交做统一任务服务的会话 |
| `.claude/agent-common.md` 第 64、66、96 行旧钩子名 | — | 交改钩子名的那一路 |
| 拆分提案一百九十六节 ② | — | 销账：对象已没有 |
| `crash-point-replay-shard-run.sh` 第 238 行开关传不到 | — | 改它的那一路已认 |

本机那一侧（不带二进制的包装、own-server 裸起）落地后真机 0 条，下一次有任务派到本机时由交的一方顺带读 `"compiles"` 与结局，读到不合本判决的照「所有任务都要多机可用」当场修。

## 回看决策

不涉及决策：这一轮判的是共用约束里的一句与配套的登记表、钩子、骨架、包装、任务服务的认法，没有动 `.claude/kb/decisions/` 里的任何分项。
