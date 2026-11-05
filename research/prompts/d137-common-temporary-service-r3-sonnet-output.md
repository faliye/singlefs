# d137-common-temporary-service-r3 辩方腿报告（R 格，第三轮最终轮）

腿：`three-way-defense`（Sonnet）。背景材料 `research/prompts/_d137-common-temporary-service-r3-body.md` 第三节「R」。本腿没写模型；只写了一份喂钩子的草稿脚本，复跑包装与喂钩子的命令都在末段「要主 agent 跑的命令」，由主 agent 交。禁读清单里的文件（同一轮攻方腿的报告）没读。

## 复跑与 sha256

| 文件 | sha256 | 用途 |
|---|---|---|
| `/home/fy5090/.local/share/singlefs/work/d137-cts-r3-defense/judge_r3.py` | 80297d7fe6a17a0b471553995ac5acd99634b78d7384ae01de640627ffe82fb5 | 把表头给的整条写法三种形态喂给 settings 里登记的两道钩子；只读，不起被判命令 |

复跑命令：见「要主 agent 跑的命令」C1。

## 开工快照

`sha256sum -c research/prompts/d137-common-temporary-service-r3-snapshot/files-sha256.txt`（从仓根跑）：10 份全部 OK，没有对不上的（量的）。第一次我在快照目录里跑，找不到相对路径，是我的工作目录错了，不是内容对不上。

此刻 sha256 前 8 位（量的，`sha256sum`）：

| 文件 | 前 8 位 | 在快照里 |
|---|---|---|
| `.claude/agent-common.md` | 9c5ba65e | 是 |
| `.claude/hooks/selftest-temporary-service.tsv` | a3fcc2ca | 是 |
| `.claude/hooks/lib_project_code_runs.py` | 4619ee9b | 是 |
| `.claude/hooks/project-code-guard.sh` | 39dc39a0 | 是 |
| `scripts/agents/dispatch-prompt.py` | b5f0023e | 是 |
| `scripts/prune-server/prune_server_client_selftest.py` | a0b77c18 | 是 |
| `task-server/task_server.py` | 975c20a1 | 是 |
| `task-server/task_server_lib.py` | 93079807 | 是 |
| 第一轮判决 | e153406b | 是 |
| 第二轮判决 | 768f8ca4 | 是 |
| `scripts/prune-server/prune_server_client.py` | e3df5128 | 否 |
| `scripts/crash/crash-point-replay-shard-run.sh` | da062681 | 否（工作区有未提交改动，另一个会话在改） |
| `.claude/gate.d/code-tooling.sh` | b4c6f008 | 否 |
| `.claude/hooks/selftest-registry.tsv` | 861a2e46 | 否 |

## 三条结论一览

1. 第二轮判决逐条「采」：10 条里 7 条站得住（R2、R3、R4、A2、A3、A4、A5）；A1 与 R1（H1 改判采）的后半句变了（根已落地，按源码两台都给编译目录，但落地后一条任务都没跑过）；R5（H4、缺口 3 换理由）里 H4 的理由变了（那条「孤例」的对象今天在源码里一处都没有）。没有一条站不住。
2. 辩护：H4 维持不采，缺口 3 维持不采，A5 维持记账（这一轮不采）。三条各有推翻条件，A5 的那条要等 C1 的读数。
3. 重数：24 行 required、2 行 own-server、14 行 `.sh`、10 行 `.py`、`crates/` 零命中今天都能由一条命令重数且数对；引文总数（r1 的 66 与 9，r2 的 53 与 47、33 与 9）重数得出，但「对上 N 处」只有给快照树当根才复得出，对主工作区今天已对不上（r1 对不上 35 处，r2 对不上 41 处）；行号漂了的、数已变的、数不出来的在第三段逐个点名。

## 另外撞见的三件事（不在三条任务里，交主 agent）

- 登记表文件头、拒绝说明、派发骨架给的「整条写法」带 `--server-binary`，门禁 runner 表（`.claude/gate.d/code-tooling.sh`）里 24 份 required 全部用不带的写法（量的，见第三段 3.9）。这是攻方腿的格，我只贴事实，标「零轮」。
- `task-server/task_server.py:134` 的注释把「6778 退 2」写成已发生的事，量的是退 0（见 1.1）。
- 拆分提案一百九十六节 ② 的 H4 点名的 `--selftest-real-sharded-run` 今天在 `scripts/`、`.claude/`、`task-server/` 里一处都没有（见 2.1）。

## 一、第二轮判决逐条「采」的依据今天还在不在

每条一行判定：站得住 / 站不住 / 变了。证据后面带文件:行与 sha256 前 8 位。读数都是今天读的；任务记录用 `task-server/task status <号> --json`、日志用 `task-server/task log <号> --all`（只读）。

### 1.1　R：第一轮 H1「采一半」改判「采」（6834、6835、6838 撑的）：变了

- 站得住的一半（量的）：没有二进制、没有编译目录时包装退 2。`scripts/prune-server/prune_server_client_selftest.py:1131`（a0b77c18）：`    if not os.environ.get(CARGO_TARGET_DIRECTORY_VARIABLE) and not break_is("build-without-target-directory"):`，它的下一行（:1132）起返回「不现编」的说明。6838 记录退 2、command-failed，日志里的原话：`  ✗ 临时 prune-server 起不来：没给 --server-binary 与 PRUNE_SERVER_BINARY，环境里也没有任务服务给的编译目录（CARGO_TARGET_DIR）：不现编，现编会把二进制编进主仓的 prune-server/target`。
- 站得住的另一半（量的）：不带二进制在第二台能起。6834、6835 记录都是 host=peer、退 0，日志都有 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1`，`build_peak` 都是 231849731 字节（两条同一条命令、同一台，算同一台上抽两次，不是两台各一次）。
- 变了的一半：判决写「不带二进制的写法按派到哪台分红绿，与 `.claude/agent-common.md:40`「所有任务都要多机可用」相顶」。这半句的前提是服务不把这一层认成会编的。今天源码：`task-server/task_server_lib.py:1390`（93079807）`def temporary_service_builds_server(command, task_environment=None, environment=None):`；`task-server/task_server.py:4713`（975c20a1）`        if lib.temporary_service_builds_server(task["command"], task.get("environment"), self.environment):` 在 `compiles`（:4709）里；`:4795` `        task["compiles"] = self.compiles(task)` 提交时记下；`:4543` `        if (task.get("compiles") and not any(task["environment"].get(name) for name in lib.TARGET_DIRECTORY_VARIABLES)` 起尝试时 compiles 为真就给编译目录，本机与第二台走同一段。按源码，两台都给，不再分红绿。
- 落地后零观测（量的）：证据任务 6777、6778、6779、6780、6781、6834、6835、6836、6837、6838 共 10 条，`task status <号> --json` 的 `.task.compiles` 全是 false，都是落地前提交的。所以「本机也给编译目录、退 0」今天是源码读出来的，不是跑出来的。读数等 C2。
- 对不上的一处注释（量的）：`task-server/task_server.py:134` 写「任务 6778 与 6834、6835 同一条命令：派本机的 compiles 为 false、不给编译目录、包装退 2」。6778 的命令里层是 `python3 scripts/crash/lib_verdict_library.py --selftest`，6834、6835 的里层是 `printenv PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET`，不是同一条；6778 记录 host=local、退 0（它跑的是改 resolve_binary 之前的包装，编进了主仓，`prune-server/target/release/prune-server` 的修改时间落在 6778 的起止之间，仍是「相容」不是证明）。「本机退 2」没有任何一条任务实测过，是第二轮判决里标了「推的」的那一句。注释该改成「推的」或删。
- 推翻：C2 里落地后提交的不带二进制的包装，`.task.compiles` 不是 true，或派到本机仍退 2。

### 1.2　R：第一轮拿「编译目录 无」一栏证本机，分不出机器：站得住

- 量的：6778（host=local）与 6780（host=peer）的 `task status` 都写「编译目录 无」；`.task.compiles` 都是 false；能分出两台的是 `.attempts[0].build_peak`：6778 为 null，6780 为 231632808。6780 日志（`task log --all` 输出的第 7 行）写编译目录在第二台这一趟的本地目录下，同一输出的第 11、12 行打出 `PRUNE_SERVER_BINARY` 与 `CARGO_TARGET_DIR` 都在其中。
- 补一条（推的）：落地后本机若真给了编译目录，`build_peak` 该是数字；所以 C2 的判法比判决当时的「那一栏」更准。
- 推翻：找到一条本机跑的、给了编译目录而 `build_peak` 仍为 null 的任务记录，这条鉴别量就失效，改回看日志里的路径。

### 1.3　R：H5、H7 的证据改引真起的 6778–6780 与 6837：站得住，但证据薄、有一个数变了

- 量的：6778（`.py`，host=local）、6779（`.py`，带 `--server-binary`，host=peer）、6780（printenv）、6837（`.sh`，带 `--server-binary`，host=peer）今天读仍是退 0；6779 日志里有 `crates-mutation-rows.py 自证 191 格都对`；6778 日志里有 `lib_verdict_library 自证 17 格都判对`。
- 数变了（量的）：判决写 6837「crash-point-replay-shard-run 自证通过：15 格绿」。这个数是当时那一份脚本的。`scripts/crash/crash-point-replay-shard-run.sh`（da062681，不在快照里）的修改时间比 6837 的 `started_at` 晚 2533 秒（`stat -c %Y` 减 `.attempts[0].started_at`），工作区对暂存区（`git diff --stat`）有 16 增 265 删，自证格数组 `:161`–`:162` 现在数得 12 项（`selftest_cells=(...)` 两行，`awk` 取出再数）。结论（`.sh` 能经包装起）没变，「15」不能再当现状引。
- 薄在哪（量的）：真起过的 `.py` 2 份（共 10）、`.sh` 1 份（共 14）；其余 21 份只在 6777 里喂过钩子；那一趟喂的三道里有 `heavy-test-guard.sh`，今天 settings 登记的是 `project-code-guard.sh` 与 `run-scope-guard.sh`（`.claude/settings.json:85`、`:89`），钩子本身也改过。C1 用今天的两道补喂。
- 推翻：C1 里任何一份 required 按「`.py` 用 python3、别的用 bash」拼出的包装写法被钩子拒；或 C3 里包装起 `.sh`/`.py` 退非 0 且原因在包装写法。

### 1.4　R：H3 旧钩子名在表头第 2、6、10 行：站得住

- 量的：`grep -n -E 'lib_heavy_tests|heavy-test-guard' .claude/hooks/selftest-temporary-service.tsv`（a3fcc2ca）零命中；表头现在第 2 行与第 12 行写 `lib_project_code_runs.py`，第 7 行写 `project-code-guard.sh`。
- 同一批旧名还在 `.claude/agent-common.md`（9c5ba65e）：`lib_heavy_tests.py` 在第 64、96 行，`heavy-test-guard.sh` 在第 66（两处）、96 行（`grep -n -o -E` 量的）；盘上 `.claude/hooks/heavy-test-guard.sh`、`.claude/hooks/lib_heavy_tests.py` 两个旧文件还在，settings 登记的是新名。不在这一句里，也不在表头里，归改钩子名的那一路，这里只点名。
- 推翻：表头再出现旧钩子名，或 `.claude/settings.json` 的钩子表指回旧名。

### 1.5　R：H4、缺口 3「不采」站得住、理由换：H4 的理由变了，缺口 3 站得住

- 逐条见第二段 2.1、2.2。一句话：H4 判决写的理由是「24 行里只有 shard-run 有第二种自证用法，孤例」，今天连这一例都没有了；缺口 3 的理由（读法在共用约束第 65 行、统一「看 ✗ 行」会错判）今天的日志还能复现。

### 1.6　A1：「写不写 `--server-binary` 都有一台会错」：变了（后半句仍站得住）

- 前半（不写会错）变了：同 1.1，源码里已不分台，落地后零观测。
- 后半（写了也没有可写的路径）站得住（量的）：占位仍是 `.claude/hooks/lib_project_code_runs.py:434` `TEMPORARY_SERVICE_BINARY_PLACEHOLDER = "<编好的 prune-server>"`（4619ee9b），拒绝说明 `:1788` 只说「换成编好的 prune-server 二进制的路径」，表头第 5 行、`scripts/agents/dispatch-prompt.py:295`（b5f0023e）同一个占位；四处都没写这个二进制从哪来。判决里真带二进制的 6779、6836、6837 用的是主仓 `prune-server/target/release/prune-server` 的绝对或相对路径，是主 agent 临场挑的。
- 表头已按根改过（量的）：`.claude/hooks/selftest-temporary-service.tsv:6` 写不带 `--server-binary`、命令前的赋值与任务环境里也没有 `PRUNE_SERVER_BINARY` 时服务把这条命令认成会编的、两台都给编译目录，与 `task-server/task_server_lib.py:244`（93079807）对得上；:6 写「认法在 task-server/task_server.py」，函数本体在 `task_server_lib.py:1390`，`task_server.py:4709` 的 `compiles` 调它，说法近似、不算错。own-server 一句（:9）说二进制与编译目录都没有时退 2、不现编，`scripts/prune-server/prune-server-load-simulation.py:191` `    binary, problem = harness.resolve_binary(arguments)` 走的就是同一个函数，对得上。
- 推翻：C2 在本机退 2（前半翻回去）；或表头/拒绝说明/骨架任何一处写出了二进制的来源（后半不再成立）。

### 1.7　A2：`--` 后面那条的包装服务不拦、句子写「照样不带」：站得住

- 量的：6836 记录 host=peer、退 0，命令是 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary /home/fy5090/code/singlefs/prune-server/target/release/prune-server -- timeout 1800 printenv PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET`，里层的 `timeout` 被收下。
- 源码对得上（读的，推的：只有 `timeout` 实测过）：`task-server/task_server_lib.py:2083` `        for word in argv[:index]:` 只看命令词之前的词，`:2088` 判命令词本身是不是 `REFUSED_PREFIXES`（:1192）里的名字，`:2091` `        for word in rest:` 只拦 `RESOURCE_ARGUMENTS`（:1193）里的资源参数。包装自己的名字不在 `REFUSED_PREFIXES` 里，所以句子写「这一层本身不算那一串里的包装」对；里层的 `nice`、`capped.sh` 之类服务看不见，句子靠读者自己守，没有哪一道检查替它把关（零轮，归记账）。
- 推翻：把里层写成 `nice` 或 `capped.sh` 在提交时被服务拒（那就是服务也拦了里层，句子里「照样不带」成了双保险而非唯一防线）。

### 1.8　A3：仓副本里改过的 required 脚本照这一句读成不归表：站得住（表头行号从第 9 行挪到第 11 行）

- 量的：`.claude/hooks/lib_project_code_runs.py:732` `def selftest_registered_file(word, directory, key):` 要求命令词解到的真实路径正是登记表所在仓里 `<仓根>/<键>`；所以仓副本里的同一份，钩子不认作表里那一行（`:1768` `def temporary_service_bare_selftest(words, directory, environment=None):` 经它判）。表头 `:11` 是这条规矩的文字（原第 9 行）。
- 句子「（仓副本里的同一份也算）」对得上，因为它管的是脚本自己的行为，不是钩子的认法：`scripts/prune-server/prune_server_client.py:1148`（e3df5128）`def selftest_service_problem(environment=None):` 只看环境里的标记，不看脚本在哪棵树，副本里的同一份裸跑一样退 2。
- 要注的一处（读的）：副本里的裸写不会走 `temporary_service_bare_selftest` 的拒绝说明，说明里不会有整条包装写法；它由「仓副本里的 `--selftest` 只在三条同时满足时放行」那一条拒（`.claude/agent-common.md:96`）。读者只靠这一句写副本里的命令，不靠拒绝说明。
- 推翻：副本里同一份 required 脚本裸跑不退 2（脚本改成不查标记），或钩子改成认副本路径。

### 1.9　A4：表头写法没有检查绑住、三处已对不上：站得住（三处今天已改，绑检查仍没有）

- 三处过时的说法今天都改了（量的）：表头 :9 写「二进制都没给、也没有任务服务给的 CARGO_TARGET_DIR 时退 2、不现编」；拒绝说明 `lib_project_code_runs.py:1788` 写「包装没给 --server-binary、环境里也没有任务服务给的 CARGO_TARGET_DIR 时不现编，退 2」；`grep -n -F '省去现编'` 在这八份文件加 `.claude/hooks/selftest-registry.tsv`（861a2e46）里零命中；「要现编」还在 `task-server/task_server_lib.py:243`、`task-server/task_server.py:134`、`.claude/hooks/selftest-registry.tsv:11`，三处写的都是有编译目录才现编、没有就不现编的条件，与源码相合。
- 绑检查仍没有（量的）：`lib_project_code_runs.py:1691` `        if not line.strip() or line.lstrip().startswith("#"):` 读表时跳过 `#` 行，表头文字不进任何判定；一百九十七节 ① 的欠仍开着。
- 推翻：表头第 5 行的写法与 `scripts/prune-server/prune_server_client_selftest.py` 的用法逐字对上的检查落地；或表头第 5、6 行又和代码对不上。

### 1.10　A5：`dispatch-prompt.py` 占位写裸的 `<脚本> --selftest`：站得住（仍开着）

- 量的：`scripts/agents/dispatch-prompt.py:282`（b5f0023e）仍是「自证、样本档与弄坏开关拼成 `task-server/task submit -- <脚本> --selftest` 这样的整条」；同一份骨架 :317 `temporary_service_command`、:326–:329 另给整条包装写法。取舍见 2.3。

### 1.11　R：第二轮复跑的其余几条读数

- 「`timeout 1800` 写在包装外，提交时当场拒」：站得住，但没有任务号可查，只能读源码或重提交一次看拒绝。源码（读的）：`task-server/task:250` 在 `lib.nested_dispatch(command)` 有说明时 `fail(f"命令自己套了一层：{nested}", …, REFUSED_EXIT)`，退出码 `task-server/task:54` `REFUSED_EXIT = 3`；`task_server_lib.py:1192` 的 `REFUSED_PREFIXES` 含 `timeout`。重提交见 C6。
- 「6836 里层 timeout 收下、6837 起 `.sh` 退 0」：记录今天读仍是退 0，见 1.3、1.7。

## 二、替「不采」「记账」的一方辩护：H4、正推缺口 3、A5

### 2.1　H4：维持不采，理由要换

- 判决：第一轮「不采，记账」；第二轮采理由「24 行里只有 `crash-point-replay-shard-run.sh` 有第二种自证用法，孤例」。
- 今天（量的）：`grep -rn -F -e 'selftest-real-sharded-run' scripts .claude/hooks .claude/gate.d task-server` 零命中；全仓（排除 `research/results/`）只剩 `records/` 下 3 份与 `research/prompts/` 下 3 份文字里提到它。24 份 required 的源码里，`grep -o -E -e '--selftest[a-z-]*'` 取出的只有 `--selftest`，三处例外是 `scripts/experiment-apparatus/e165-pin-red-examples.py` 的 `--selftest-forced-judge`、`--selftest-library`（:1883、:1885，自证里传给内层 main 的参数）与 `scripts/gate/verus-proof-check.py` 的 `--selftest-mir`（:4166，假 verus 的入口），都不是命令行上单独起、会调 `selftest_service_problem` 的自证入口（读的，推的：两个 `selftest_service_problem` 调用点 e165 :1506、verus :4946 没逐个追到它们属于哪个入口）。
- 这个入口曾经有过（量的）：6837 日志里 shard-run 的成功句还写着「真跑那一格另由 --selftest-real-sharded-run 跑」；脚本在 6837 之后被改（1.3），自证格数组里没有它，脚本 :14 的注释转述为两台真分判不在自证里、由门禁判。H4 点名的东西今天不存在，所以「不采」站得住，理由从「孤例」换成「对象已没有」。
- 交主 agent（转述）：`records/2026-10-29-subagent拆分提案.md:1026` 一百九十六节 ② 与「欠 ②」点名的是这个不存在的参数，该改写成「再有 `--selftest*` 入口调 `selftest_service_problem` 时并进」，或销账。该脚本是另一个会话在改的未提交改动，回来则 H4 回来。
- 推翻：任一 required 脚本重新有从命令行单独起、会调 `selftest_service_problem` 的 `--selftest*` 入口。

### 2.2　正推缺口 3（没写跑完看哪一行）：维持不采

- 量的：`.claude/agent-common.md:65`（9c5ba65e）仍写「跑完看哪几行判结果（`TASK_SERVER_RESULT` 那一行的 `type=` 与 `exit=`，`task-server/task log <号>` 里的成功句、`✗` 行、计数行各是什么算过）」（`grep -n -o -F` 命中）。
- 统一「看 ✗ 行」会错判（量的）：6778 的日志里有一行 `✗ 判定库没处放…（退 2）`，任务退 0、自证全绿——那是自证里故意走的失败路径格。
- 成功句各脚本不同（量的，3 份）：6837 `crash-point-replay-shard-run 自证通过：N 格绿`、6778 `lib_verdict_library 自证 17 格都判对`、6779 `crates-mutation-rows.py 自证 191 格都对`。其余 21 份没量。写一个统一的「看哪一行」进这一句会写错；该句给写报告的子 agent 看，读结局的是主 agent，:65 已给判法。
- 推翻：24 份 required 的成功句统一成同一个格式，或出现退 0 而自证其实没跑的假绿（那时句子里要加「看成功句」）。

### 2.3　A5：维持记账，这一轮不采

- 量的：`scripts/agents/dispatch-prompt.py:282` 仍是裸写的占位（1.10）。
- 为什么裸写不会静默出错（读的，C1 的 B、S 两类案例出读数前是推的）：`.claude/hooks/project-code-guard.sh:1370` 把交给服务那条里裸写的要临时服务的自证也收进来，`:1392` 用 `temporary_service_bare_selftest` 判，`:1807` 拒并给整条包装写法；注释 `:209` 写「经不经 submit、谁起都拒」。6777 里 24 条裸写都被拒（量的：54 条案例只有 1 条对不上，且是 run-scope-guard 的预期写错，见第一轮判决）。后果是主 agent 多交一趟被拒、报告里改写，不是错判。
- 同一份骨架另有「要临时服务的自证」一行，`dispatch-prompt.py:326`–`:329` 给整条包装写法；占位只在派发时没点名脚本的情况下是子 agent 看到的唯一写法，那时还有 `agent-common.md:64` 的句子兜底。
- 不现在采的理由：改动落在 `dispatch-prompt.py` 的占位文字上，它的自证（`:836`）按整句比对，要连自证一起改，是工具员的活，一百九十七节 ② 已登；占位对表外的 `--selftest` 本来是对的，不能一刀换成「指到要临时服务那一行」；这一轮的范围是共用约束里的那一句。
- 推翻（转为该采）：C1 里 B 或 S 案例不是退 2（裸写没被拦住）；或有实例：主 agent 照占位提交了 required 脚本的裸写并白跑一趟。

## 三、前两轮判决里的任务号、行号、计数：能不能由一条命令重数

每条：命令、今天的现数、跟判决对不对得上。「量的」是今天跑出来的；跑不出来的点名。

| 编号 | 判决里的说法 | 重数命令 | 今天的现数 | 对不对得上 |
|---|---|---|---|---|
| 3.1 | 24 行 required、2 行 own-server、14 行 `.sh`、10 行 `.py`（第一轮 H7、第二轮、背景材料第二节） | `awk -F'\t' '$2=="required"' .claude/hooks/selftest-temporary-service.tsv \| wc -l`；`$2=="own-server"`；`&& $1 ~ /\.sh$/`；`&& $1 ~ /\.py$/` | 24、2、14、10（量的，表共 26 行有效行） | 对上 |
| 3.2 | `grep -rn selftest-temporary-service crates/` 零命中 | 同一条命令 `\| wc -l` | 0（量的） | 对上 |
| 3.3 | 第一轮：66 处引文，对上 38、没判 37；再对主工作区「对上 66、没判 9」 | `python3 scripts/agents/cite-check.py <r1 两份腿报告> --root <第一轮倒推树> --background <r1 正文>`（直接跑的文本工具）；去掉 `--root` 即对主工作区 | 用倒推树（仍在盘上 `~/.local/share/singlefs/work/d137-common-temporary-service-r1-main/d137-common-temporary-service-r1-tree`）：「核了 66 处引文、9 处裸行号，对上 38 处，没判 37 处」；对主工作区：「核了 66 处引文、9 处裸行号，35 处对不上，9 处没判」（量的） | 第一句逐字对上；第二句的「对上 66」今天复不出（引到的文件被改过），要靠倒推树 |
| 3.4 | 第二轮：辩方腿 53 处引文、47 处裸行号，攻方腿 33 处、9 处，「对上」全数 | 同上，一份腿报告一条，`--background` 给 r2 正文 | 辩方腿 `r2-sonnet-output.md`：53 处引文、47 处裸行号、27 处对不上；攻方腿 `r2-opus-output.md`：33 处、9 处、14 处对不上（量的，对主工作区） | 总数对上；「对上 53、对上 33」今天对主工作区复不出；第二轮倒推树不在盘上（`ls ~/.local/share/singlefs/work/` 没有 r2 的 main 目录），要 C5 重建 |
| 3.5 | 任务 6777 `cases=54 mismatches=1`；6778–6781 退 0；6782 倒推「7 份、6 份、1 份」；6834、6835 退 0；6836、6837 退 0；6838 退 2 | `task-server/task status <号>`，`task-server/task log <号> --all`（只读） | 6777 日志末有 `SENTENCE_JUDGE cases=54 mismatches=1`；6778–6781 退 0；6782 退 1 且日志有「清单点名 7 份，倒推出 6 份、sha256 对上 6 份；倒推不出 1 份、对不上 0 份」；6834–6837 退 0；6838 退 2（量的） | 对上；6782 的退 1 是它故意报的「有一份倒推不出」 |
| 3.6 | 6777 的 54 条案例 | 重跑要重交，输入脚本 `judge_sentence.py` 还在盘上（`.../d137-common-temporary-service-r1-sonnet/`），但它喂的三道里有今天已不在 settings 里的 `heavy-test-guard.sh` | 数不出今天同口径的数 | 不能同数；C1 用今天的两道钩子重做，案例数 77（判 75、观测 2） |
| 3.7 | 6837「15 格绿」 | `awk 'NR>=161{print} /\)$/ && NR>=161{exit}' scripts/crash/crash-point-replay-shard-run.sh \| ...` 数自证格数组（读源码）；真数要 C3 | 数组 12 项（量的，读源码）；脚本在 6837 之后改过（1.3） | 对不上，不是 15 了；真跑出来的句子等 C3 |
| 3.8 | 行号：第一轮判决引 `prune_server_client_selftest.py` 第 1054–1056 行（`resolve_binary`）、第 1105 行起、第 143 行起；`dispatch-prompt.py:287`；`agent-common.md:40`；`code-tooling.sh:625`、`:1045`；一百九十六节 ① 同引 1054–1056 | `grep -n -E 'def (resolve_binary\|build_server_binary)' scripts/prune-server/prune_server_client_selftest.py`；`awk 'NR==287'`；`awk 'NR==40'` | `build_server_binary` 仍是 :143；`resolve_binary` 今天是 :1123–:1134，调用点含 :1141、:1185（量的，自证里另有 :1037、:1040）；`dispatch-prompt.py:287` 仍是 `SELFTEST_RUNNER_TYPES`；`agent-common.md:40` 仍是「所有任务都要多机可用」那一条；`code-tooling.sh:1045` 仍写读法在 `lib_project_code_runs.py`，`:625` 今天是 temporary-service-runners 格的出路文字、不再写它 | 143、287、40、1045 对上；1054–1056、1105、625 漂了，按名字找 |
| 3.9 | （新数，不是判决里的）门禁 runner 表里 required 的写法 | 对表里每个 required 键 `grep -q -F -e "with-temporary-service -- bash <键> --selftest" -e "with-temporary-service -- python3 <键> --selftest" .claude/gate.d/code-tooling.sh`；`grep -c -F -e '--with-temporary-service --server-binary' .claude/gate.d/code-tooling.sh` | 24 份 required 全部以不带 `--server-binary` 的写法登在 `code-tooling.sh:956`–`:1017` 一带；带 `--server-binary` 的只有 1 行（:1412，量的，b4c6f008） | 不是判决的数；这是表头第 5 行与 runner 表不一致的事实，零轮，归攻方腿 |

数不出来的，逐个点名：

- 6781（第一轮 P2）：喂的是旧名 `heavy-test-guard.sh`，输入 `p2a-hook-input.json`、`p2b-hook-input.json` 所在的第一轮攻方腿工作目录今天是空的（`ls` 量的 0 项），重做不了；它撑的结论（P2 与预期相同）今天的钩子上要重新喂，C1 覆盖了写法类，不覆盖它当时那条具体输入。
- 第二轮「本机会照 6838 退 2」：标着「推的」，没有任何任务实测，落地后源码已不再支持它（1.1）。
- 第一轮「主仓二进制的修改时间落在 6778 那一趟里」：区间能量（`stat -c %Y prune-server/target/release/prune-server` 落在 6778 的 `started_at` 与 `ended_at` 之间，量的），是谁编的数不出来，判决写「相容，推的」，对。
- 第二轮「`timeout` 写在包装外提交时当场拒」：被拒的提交不进库、没有任务号，数不出来，只能读源码或 C6 重提交。

## 四、要主 agent 跑的命令

我没往统一任务服务交任务，下面每条都是 `task-server/task submit -- <命令>` 的整条，由主 agent 交。没有一条是重型（不含 checker 档、三道重型带 `--item`、全量 `cargo test`）；C2、C3 的包装里会在服务给的编译目录里现编一份 prune-server（离线、`cargo build --release`），编译目录归服务，交的主 agent 用完 `task-server/task clean <号>`。读数未出之前，下面「预期」都是推的。

### C1　喂今天的两道钩子（判 1.3、1.10、2.3，补 3.6）

- 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d137-cts-r3-defense -- python3 /home/fy5090/.local/share/singlefs/work/d137-cts-r3-defense/judge_r3.py --repo /home/fy5090/code/singlefs --out /home/fy5090/.local/share/singlefs/work/d137-cts-r3-defense/judge-out`
- 在哪起：上面命令的 `--cwd`（草稿目录）。
- 看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；日志末行 `R3_HOOK_FEED cases=77 judged=75 mismatches=0 observed=2`；有 `MISMATCH` 行就逐条看（标签 `W-bin`、`W-nobin` 是包装写法，`B` 是裸写，`S-subagent` 是子 agent 身份的裸写，`O` 是 own-server 的裸写）。两条 `OBSERVE` 行（`X-placeholder-literal` 占位原样、`X-inner-timeout` 包装里层套 `timeout`）只记退出码，不判。预期（推的）：`W-bin`、`W-nobin`、`O` 两道钩子都是 0；`B` 与 `S-subagent` 里 `project-code-guard.sh` 是 2、`run-scope-guard.sh` 是 0。
- 脚本把每次喂钩子放进自己的进程组，超时与结束都对整组发信号（`start_new_session=True`，`os.killpg`）；跑完 `ps` 看不到遗留进程即可。

### C2　落地后不带二进制的包装，派到本机能不能退 0（判 1.1、1.6，A1 的根）

- 命令（同一条交 3 次）：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- printenv PRUNE_SERVER_BINARY CARGO_TARGET_DIR`
- 在哪起：仓根。
- 看哪几行：每个号 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task status <号> --json` 的 `.task.compiles` 应为 true（落地前提交的 10 条全是 false）、`.attempts[0].host`（local 或 peer）、`.attempts[0].build_peak`（本机有编译目录时该是数字，不是 null）；`task-server/task log <号> --all` 里 `PRUNE_SERVER_BINARY` 与 `CARGO_TARGET_DIR` 两个路径都在服务给的编译目录下，以及 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1`。派到哪台由服务定，我不写 `--peer` 之类；3 次都派到第二台，则本机一侧如实写「落地后仍没实测」，再多交几条到出现 host=local 为止，或主 agent 判够。
- 判法：`.task.compiles` 为 false，或 host=local 而退 2，则 1.1 的「变了」翻回去（根没落到跑着的服务，先看服务是否重启过）。

### C3　包装起 `.sh`、`.py` 的 required 自证，重数格数（判 1.3、3.7）

- 命令 C3a：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-point-replay-shard-run.sh --selftest`
- 命令 C3b：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/lib_verdict_library.py --selftest`
- 在哪起：仓根。
- 看哪几行：`exit=0`；C3a 日志里 `crash-point-replay-shard-run 自证通过：N 格绿` 的 N（预期 12，推的：来自自证格数组 12 项；脚本还在被别的会话改，读到别的数以读到的为准）；C3b 日志里 `lib_verdict_library 自证 N 格都判对` 的 N（6778 当时是 17，今天的数待读）；`PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1`。

### C4　任务服务自己的自证里认包装会编的那一格（判 1.1 的源码侧）

- 命令 C4a：`task-server/task submit -- python3 task-server/task_server_lib.py --selftest`
- 命令 C4b：`task-server/task submit -- env TASK_SERVER_BREAK=temporary-service-build-unrecognized python3 task-server/task_server_lib.py --selftest`
- 在哪起：仓根。
- 看哪几行：C4a `exit=0`、没有 `✗` 行；C4b 预期（推的，来自 `task-server/task_server_lib.py:829`–`:830` 的开关表）`exit=1`，且日志里有 `✗ task_server_lib.py 自证：temporary-service-wrapper-compiles：`。C4b 不红，则这一格没绑住 `temporary_service_builds_server`，1.1 的源码侧结论要降成「读的」。

### C5　重建第二轮倒推树，复数「对上 53」「对上 33」（判 3.4）

- 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d137-cts-r3-defense -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py d137-common-temporary-service-r2 --out /home/fy5090/.local/share/singlefs/work/d137-cts-r3-defense/d137-common-temporary-service-r2-tree --repository /home/fy5090/code/singlefs`
- 在哪起：上面命令的 `--cwd`。
- 看哪几行：日志里「清单点名 8 份，倒推出 … 份、sha256 对上 … 份；倒推不出 … 份、对不上 … 份」（第二轮判决写 `prune_server_client_selftest.py` 对不上快照，预期倒推不出这一份，推的）。成功后直接跑（文本工具）：`python3 scripts/agents/cite-check.py research/prompts/d137-common-temporary-service-r2-sonnet-output.md research/prompts/d137-common-temporary-service-r2-opus-output.md --root /home/fy5090/.local/share/singlefs/work/d137-cts-r3-defense/d137-common-temporary-service-r2-tree --background research/prompts/_d137-common-temporary-service-r2-body.md`，对照「53 处引文、47 处裸行号，对上 53」与「33 处引文、9 处裸行号，对上 33」（两份分开跑，各自 `--root`）。读完把那棵树删掉（它是仓副本类目录）。

### C6　外层套 `timeout` 的提交当场被拒（判 1.11）

- 命令：`task-server/task submit -- timeout 1800 printenv PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET`
- 在哪起：仓根。
- 看哪几行：提交被拒、没有任务号；客户端退 3 时 stderr 有「命令自己套了一层：timeout …：限时由服务定，命令里不套它（去掉这一层）」；若是钩子先拒，则读钩子的拒绝说明，这条只是为了见一次「外层被拒」，不要为它改写法绕过去。

## 五、撞见的、不在三条任务里的事（零轮，交主 agent 分路）

- Z1　「整条写法」有两种口径同时在仓里：登记表文件头第 5 行（`.claude/hooks/selftest-temporary-service.tsv:5`）、拒绝说明（`.claude/hooks/lib_project_code_runs.py:1788`）、钩子注释（`.claude/hooks/project-code-guard.sh:210`）、骨架（`scripts/agents/dispatch-prompt.py:295`）带 `--server-binary <编好的 prune-server>`；门禁 runner 表 24 份 required 全部不带（3.9），`.claude/gate.d/code-tooling.sh:625` 的出路文字也写不带的形态。表头第 6 行同时说不带也行。读者照句子「照那张表文件头 required 一条给的整条写法」会选哪一种，是攻方腿的格，我只贴事实。
- Z2　服务认「包装会编」只认命令词就是包装脚本的写法：`task-server/task_server_lib.py:1373` `def temporary_service_split(words):` 经 `command_word_index` 剥 env、nice、timeout、setsid、`bash`/`python3` 起脚本，`bash -c '…包装…'` 停在 `-c`，认不出就是 `compiles` 为 false。把包装写进 `bash -c` 的提交，本机仍可能没有编译目录而退 2（读的，推的，没实测）。
- Z3　`.claude/agent-common.md` 第 64、66、96 行还写旧钩子名与旧库名（1.4）。
- Z4　`task-server/task_server.py:134` 的注释把「本机退 2」写成 6778 的事实（1.1）。
- Z5　`records/2026-10-29-subagent拆分提案.md:1026` 的 H4 点名的参数今天不存在（2.1）。
- Z6　6834、6835 是同一条命令、同一台（第二台）、`build_peak` 字节数相同，算同一台上抽了两次；「两台」的分辨靠 6778（本机，改包装之前）与第二台那几条，落地后的对照一条都没有（C2）。

## 六、什么现象会推翻本报告的结论

- 结论 1（逐条判）：C2 落地后本机仍退 2 或 `.task.compiles` 为 false，推翻 1.1、1.6 的「变了」；C1 里任一 required 的包装写法被钩子拒，推翻 1.3 的「站得住」。
- 结论 2（辩护）：见 2.1、2.2、2.3 各自的「推翻」；其中 A5 的转折点是 C1 里 `B`、`S-subagent` 案例是不是退 2。
- 结论 3（重数）：命令都在表里，数对不上就是推翻；3.3、3.4 的「对上」要靠倒推树，不靠主工作区。

## 没做什么

- 没往统一任务服务交任务（`submit`、`wait`、`cancel`、`clean` 一条都没起），C1–C6 全由主 agent 交；`judge_r3.py` 只做了语法检查（`python3 -m py_compile`），没跑；判决要引的数没复跑，C2、C3 的读数写「推的，待主 agent 跑」。
- 没判攻方腿的格（表头第 5、6 行是不是同一个答案、交出表有没有漏）；Z1–Z6 只贴事实。
- 没读禁读清单里的文件（同一轮攻方腿的报告）；没改仓里任何文件，只写了这份报告、`judge_r3.py` 和草稿目录里的任务记录副本（交回前删掉）。
- 没逐个追 24 份 required 脚本各自的成功句（只量了 3 份）；没追 e165 与 verus 的 `selftest_service_problem` 调用点属于哪个入口（2.1 标了推的）。
- 没碰另一个会话正在改的 `scripts/crash/crash-point-replay-shard-run.sh`；它的数（12 项）会随那个会话继续变。
