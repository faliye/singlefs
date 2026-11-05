# d137-common-temporary-service-r2 辩方腿（格 R）报告（2026-11-05）

立场：复核 `research/prompts/d137-common-temporary-service-r1-main-verification.md` 的每一条判，替被判不采的一方（正推缺口 3、H4、H6）辩护，查「照表头那一条」是否同样打中别的写法。
依据只认判决「核查怎么做的」点名的任务 6777–6782（`task-server/task log <号> --all` 与 `task-server/task status <号> --json`，我拷在草稿目录 `~/.local/share/singlefs/work/d137-common-temporary-service-r2-sonnet/` 的 `log-<号>.txt`、`status-<号>.json`）与源码。没写模型，没有复跑命令要给 sha256；要主 agent 跑的命令在最后一节。
禁读清单里的文件（r2 攻方腿的报告与模型目录）一份没读。

## 一、被判文件此刻的 sha256 前 8 位，与快照对比

快照：`research/prompts/d137-common-temporary-service-r2-snapshot/files-sha256.txt` 的 8 份。此刻逐份 `sha256sum`：

| 文件 | 快照前 8 位 | 此刻前 8 位 | 对上？ |
|---|---|---|---|
| `.claude/agent-common.md` | d7b985eb | d7b985eb | 对上 |
| `.claude/hooks/selftest-temporary-service.tsv` | 502ac94b | 502ac94b | 对上 |
| `scripts/agents/dispatch-prompt.py` | ab7e73da | ab7e73da | 对上 |
| `.claude/hooks/heavy-test-guard.sh` | 63e03a97 | 63e03a97 | 对上 |
| `.claude/hooks/lib_heavy_tests.py` | cd3fa91e | cd3fa91e | 对上 |
| `.claude/gate.d/code-tooling.sh` | b4c6f008 | b4c6f008 | 对上 |
| `scripts/prune-server/prune_server_client_selftest.py` | 4d35b619 | a0b77c18（开工时 96bc4b76，中途读到 28eaf674、24fd2c35，是另一个会话正在改） | 对不上，且我读的过程中还在变 |
| `research/prompts/d137-common-temporary-service-r1-main-verification.md` | e153406b | e153406b | 对上 |

这份文件未进 git（`git status` 的 `AM`），HEAD 里没有基底，快照那一刻的样子倒推不出。第一轮判决引它的行号（143、1054–1056、1105）因此要按此刻的样子重对：此刻 `build_server_binary` 仍在第 143 行起，`resolve_binary` 在第 1123 行起，行号以我最后一次现查为准，下面引它的地方都同时写函数名，行号会随那个会话继续改而移动。

本报告里别的文件此刻 sha256 前 8 位（引到才列）：`.claude/hooks/selftest-registry.tsv` f266752a、`.claude/hooks/lib_project_code_runs.py` 1e74cd7f、`.claude/hooks/project-code-guard.sh` ed3bc47b、`.claude/hooks/run-scope-guard.sh` 0aeb251f、`scripts/prune-server/prune_server_client.py` 78ef4060、`scripts/crash/crash-point-replay-shard-run.sh` 994023f4、`task-server/task_server.py` e2201a51、`task-server/task_server_lib.py` 07914d6e、`scripts/hosts/multi-host-run.sh` f4efc54d、`records/2026-11-05-删轻型并入重型.md` c0374def、`records/2026-10-29-subagent拆分提案.md` 03e90bbc。

## 二、先说最要紧的三件（后面逐条展开）

1. H1 的前提已经被别的会话改掉：此刻 `prune_server_client_selftest.py` 的 `resolve_binary` 在没有 `--server-binary`、没有 `PRUNE_SERVER_BINARY`、环境里也没有 `CARGO_TARGET_DIR` 时不再现编，直接退 2、出路点名 `--server-binary`。判决「采一半」里「句子里写可以带 `--server-binary` 修不了根」那半句的前提（根在别处会修、句子不用管）变了：根的修法恰好是把二进制路径推回给调用方，写命令的人（句子读者）必须写 `--server-binary`，否则派到本机就退 2。表头与句子里的 `[--server-binary <二进制>]` 方括号因此不再是「可选」。
2. 判决用来坐实「本机那一侧不给编译目录」的证据列（任务 6778 的「编译目录 无」）本身分不出：任务 6780 的同一栏也是「无」，而它的进程里真有 `CARGO_TARGET_DIR`。结论本身对，靠的是 `task_server.py` 的 `compiles` 分支与 `status --json` 里 6778 的 `"compiles": false`，不是那一栏。
3. 判决里 H5 的「24 行 required 照写一行都不被服务退 3，6777」、H7 的「按……拼全都起得来（6777）」，6777 并没有测这两件：它只把命令喂给三道钩子，不提交、不起命令（脚本自己写着「只读」）。服务提交时的拒绝与「真起得来」由 6778、6779、6780 三个真提交的样本撑，没有一个是 `.sh` 的 required 脚本。

三件都不推翻判决对被判句的改法（H2、H5、H7 照采没错），推翻的是 H1 那一行的去向写法与几处证据的归属。

## 三、证据原件（整行抄，供下面各节引）

任务 6778（派到本机，`task-server/task status 6778`）：

> 任务 6778：finished、结局 succeeded、退 0、会话 21407d62-caa5-46d9-a562-0b933dbd280d、不要卡、命令 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/lib_verdict_library.py --selftest
> 第 1 次：本机、上限 822M、卡 无、线程 31、状态 ended、退 0（succeeded）、用时 139.715 秒、内存峰值 795 MiB、平均在用核数 1.08、显存峰值 无、编译目录 无、单元 singlefs-task-b0f6b921-6778-1.service、日志 /home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-6778.attempt-1.log

6778 的 `task-server/task log 6778 --all` 整份 4 行里的两行：

> PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1
> ✓ lib_verdict_library 自证 17 格都判对；弄坏开关 7 个，每个都红在点名那一格

6778 的 `status --json`（拷在 `status-6778.json`）里读到的字段，量的：`"compiles": false`，`"build_directory": ""`，任务环境的键只有 LANG、PATH、ROCKSDB_LIB_DIR、ROCKSDB_STATIC、SNAPPY_LIB_DIR、SNAPPY_STATIC（没有 `CARGO_TARGET_DIR`）。开跑与结束的时刻差 139.8 秒。

任务 6780（派到第二台，命令 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- printenv PRUNE_SERVER_BINARY CARGO_TARGET_DIR`）：

> 第 1 次：第二台、上限 4096M、卡 无、线程 31、状态 ended、退 0（succeeded）、用时 17.311 秒、内存峰值 1.11 GiB、平均在用核数 没量到、显存峰值 无、编译目录 无、单元 singlefs-task-b0f6b921-6780-1.service、日志 /home/fy5090/.local/state/singlefs/task-server/logs/task-b0f6b921-6780.attempt-1.log

6780 的 `task log` 里三行：

> 第二台的目录/builds/task-b0f6b921-6780-1/target/release/prune-server
> 第二台的目录/builds/task-b0f6b921-6780-1/target
> MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=231632808

（前两行是 `printenv` 打出的 `PRUNE_SERVER_BINARY` 与 `CARGO_TARGET_DIR`，路径里的第二台本地目录已被日志换成「第二台的目录」；第三行是编译占盘峰值。）6780 的 `status --json`：`"compiles": false`、`"build_directory": ""`、任务环境的键与 6778 相同、没有 `CARGO_TARGET_DIR`。同一栏「编译目录 无」，进程里却有编译目录。

任务 6779（派到第二台，带 `--server-binary`）的 `task log` 两行：

> PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1
> MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=0

任务 6777（只喂钩子）的 `task log` 两行：

> MISMATCH case=S scripts/crash/lib_verdict_library.py hook=run-scope-guard.sh want=2 got=0 
> SENTENCE_JUDGE cases=54 mismatches=1

## 四、逐条复核：判决里「采」和「采一半」的几条

### H1 / 正推缺口 1（判「采一半」，根交 prune-server 那一路）

辩方判定：「本机那一侧成立」站得住，但证据要换；「句子里写可以带 `--server-binary` 修不了根」站不住，因为根已经被修成了要求调用方带它。

1. 回落链源码在。`scripts/prune-server/prune_server_client_selftest.py`（此刻 sha a0b77c18）：「completed = subprocess.run(["cargo", "build", "--release", "--offline",」（`scripts/prune-server/prune_server_client_selftest.py:146`）在 `build_server_binary`（第 143 行起）里，没有 `--target-dir`。
2. 「本机不给编译目录」的证据列分不出。6778 的「编译目录 无」与 6780 的「编译目录 无」逐字相同（见第三节），而 6780 的进程里有 `CARGO_TARGET_DIR`（`printenv` 打出了 `builds/…/target`）。`status --json` 的 `"compiles"` 在两个任务里都是 false。所以判决「6778 那一栏「无」」撑不起结论。
3. 真正撑得起的是服务源码：「build_directory, made_here = "", True」（`task-server/task_server.py:4249`，sha e2201a51）接着第 4250 行起只有任务的 `compiles` 为真才设 `CARGO_TARGET_DIR`，第 4257 行「environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")」。文件头也写了同一件事：「调用方没给编译目录的，本机这一次尝试设 CARGO_TARGET_DIR=<TASK_SERVER_BUILD_DIRECTORY>/<尝试标签>/target」（`task-server/task_server_lib.py:290`，sha 07914d6e）。命令位置上没有会编的 cargo 的任务（包装命令就是）`compiles` 为 false，本机不设。第二台是 `multi-host-run.sh` 无条件给：「build_text="CARGO_TARGET_DIR=$(printf '%q' "$peer_scratch/target") "」（`scripts/hosts/multi-host-run.sh:770`，sha f4efc54d），6777、6781、6782 这些不编任何东西的任务日志里也有「编译目录在第二台这一趟的本地目录」那一句。所以「只在本机那一侧成立」成立，依据是 `compiles` 分支，量的（读源码加 `status --json`）。
4. 「主仓二进制被这一趟重编」仍是「相容，推的」，判决这样写对。我现查：`prune-server/target/release/prune-server` 的修改时刻在 6778 开跑之后约 7 秒、结束之前约 133 秒（6778 `status --json` 的 `started_at`、`ended_at` 与 `stat` 的差，量的）；6778 的 `task log` 整份 4 行，没有 cargo 的输出行。同一个仓里别的会话也可能在这个窗口编同一个目标，分不出，所以只能写相容。
5. 第二台那一侧的对照是量的：不带 `--server-binary` 的 6780 里 `MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=231632808`（现编了一份，落在第二台本地的编译目录、跑完删），带了的 6779 里 `MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=0`。
6. 根已被改：「if not os.environ.get(CARGO_TARGET_DIRECTORY_VARIABLE) and not break_is("build-without-target-directory"):」（`scripts/prune-server/prune_server_client_selftest.py:1131`）下面两行写着「现编会把二进制编进主仓的 prune-server/target」，退出的是 `return None, (…)`，调用方 `with_temporary_service` 在 `server is None` 时 `return EXIT_USAGE`（`scripts/prune-server/prune_server_client_selftest.py:1232`；`EXIT_USAGE = 2` 在 `scripts/prune-server/prune_server_client.py:265`，sha 78ef4060），另配了一格 `cell_binary_not_built_without_target_directory`（`scripts/prune-server/prune_server_client_selftest.py:1029`）。这正是判决「去向」一栏、拆分提案一百九十六节「欠」里第一种修法：「或包装在没有服务给的编译目录时拒绝现编，出路写带 --server-binary；配自证格」（`records/2026-10-29-subagent拆分提案.md:1025`，sha 03e90bbc）。`records/2026-11-05-删轻型并入重型.md:170` 记着这一条已转给在跑的判定库清理组，写明「没有服务给的 `CARGO_TARGET_DIR` 时不现编、退 2、出路带 `--server-binary`」（sha c0374def）。
7. 后果（推的，读源码；确认命令在第八节）：包装不带 `--server-binary` 的整条命令，派到第二台能跑（现编，约十几秒）、派到本机退 2。派到哪台由服务定，写命令的人不判（`.claude/agent-common.md:64` 的「哪台机器……由服务定」一段，sha d7b985eb），所以「`[--server-binary <二进制>]` 可写可不写」变成按机器随机红的写法，撞上 `.claude/agent-common.md:40` 的「所有任务都要多机可用」。骨架自动拼出的整条命令更直接受害：`scripts/agents/dispatch-prompt.py:295`（sha ab7e73da）的 `TEMPORARY_SERVICE_WRAPPER` 不含 `--server-binary`，`temporary_service_command`（同文件第 314 行起）按它拼，骨架里「有编好的 prune-server 二进制时在 --with-temporary-service 后面加 --server-binary <二进制>」（`scripts/agents/dispatch-prompt.py:330`）把它当可有可无。
8. 判决那半句「子 agent 给不出可信的二进制路径」仍然对（没有哪处告诉子 agent 二进制在哪），但推出「所以不写进句子」不成立：根的修法已经假定调用方有路径可给。主 agent 在 6779 里用过的路径是共享根下的主仓构建产物，两台读同一份；它的新旧由谁保证，没有一处写。这是 H1 该重新判的地方：采，而不是「采一半」。什么现象会推翻我这一条：第八节命令 1 在本机一侧退 0，或服务把 `compiles` 改成对包装命令为真。

### H2（采）

站得住。「骨架只给四种类型」：「SELFTEST_RUNNER_TYPES = ("tooling-writer", "implementation-writer", "investigator", "experiment-runner")」（`scripts/agents/dispatch-prompt.py:287`），三方腿、回扫员（`sweep`）不在里面；这三类的定义 `tools` 只有 Read 与 Bash（`.claude/agents/three-way-forward.md:4` 等），回报里写的复跑命令只能靠共用约束那一句。
但「改成指到表头，不再自己抄一份」只去掉了共用约束里的那一份。同一个写法此刻在下面六处各有一份（含表头），彼此不一致：

| 位置 | 有无 `--server-binary` |
|---|---|
| 表头 `.claude/hooks/selftest-temporary-service.tsv:5`（sha 502ac94b） | 方括号可选 |
| `.claude/hooks/selftest-registry.tsv:9`（sha f266752a） | 必写，且同表第 10 行写「没给 --server-binary 的包装要现编 prune-server，算提交时才跑的检查」 |
| `scripts/agents/dispatch-prompt.py:295` | 没有 |
| `.claude/gate.d/code-tooling.sh:625`（sha b4c6f008） | 没有 |
| `.claude/hooks/lib_project_code_runs.py:396`（sha 1e74cd7f）常量注释 | 方括号可选 |
| 同文件 `temporary_service_command`（第 1813 行起） | 没有 |

表头的方括号写法没有检查在盯：表的两道读法只读行，不读头。`temporary_service_table_problems`（`.claude/hooks/lib_project_code_runs.py:1782`）逐行核文件在不在、有没有登记、源码里有没有记号；`code-tooling.sh` 里读表的那段（`.claude/gate.d/code-tooling.sh:1063` 起）只调 `read_temporary_service_table` 打 `rows` 与 `problems`。所以「表头错了谁来发现」的答案是：没有人。表头第 6 行此刻就已经写旧了（见 H3），这是现成的例子。
子 agent 读得到表头：所有项目子 agent 的 `tools` 都含 Read。

### H3（采，交改名那一路）

站得住，范围少数了一处。判决写「文件头第 2、6 行」，实际此刻旧名出现在三行：「判法在 .claude/hooks/lib_heavy_tests.py 的 temporary_service_table_at」（`.claude/hooks/selftest-temporary-service.tsv:2`）、「裸写的 --selftest 在执行前被 heavy-test-guard.sh 拒」（同文件第 6 行）、「对一遍（lib_heavy_tests.py 的 temporary_service_table_problems」（同文件第 10 行）。新读法在 `.claude/gate.d/code-tooling.sh:1045`：「读法是 .claude/hooks/lib_project_code_runs.py 的 read_temporary_service_table」（sha b4c6f008）。旧库 `lib_heavy_tests.py`（sha cd3fa91e）与旧闸 `heavy-test-guard.sh`（sha 63e03a97）此刻都还在盘上，所以这三行此刻不是指到不存在的东西，是指到即将被替换的那一份。改名那一路改头时这三行都要换。

### H5（采）

站得住，证据归属要改。结论「功能上不顶」：服务拒的是写死的前缀名，「REFUSED_PREFIXES = {"nice": "优先级", "timeout": "限时", "capped.sh": "线程", "two-logs.sh": "日志", "gpu-quiesce-run.sh": "清场与挑卡"}」（`task-server/task_server_lib.py:1140`），嵌套派发的脚本名在第 1138 行「NESTED_DISPATCH_SCRIPTS = ("run-with-memory-cap.sh", "host-dispatch.py", "multi-host-run.sh")」，包装脚本在两处名单里都没有。真提交被服务收下的样本是 6778、6779、6780（三个都退 0，是量的）。判决写的「24 行 required 照写一行都不被服务退 3，6777」不对：6777 日志里只有喂钩子的「ok case=W …」行，没有提交。

### H7 / 正推缺口 2（采）与 6777 的那一条对不上

规则本身站得住：表里 24 行 required，14 行 `.sh`、10 行 `.py`（`awk -F'\t' '$2=="required"' …/selftest-temporary-service.tsv` 数的，量的）。「6777 都起得来」要改口：6777 的 54 条是 24×2（写包装、写裸）加 2 行 own-server 加 3 条带二进制加 1 条子 agent，全是把字符串喂给三道钩子看退几，不起命令。真提交真起命令的只有 6778（`.py`）、6779（`.py`，带二进制）、6780（`printenv`）。14 行 `.sh` 里没有一行真经包装起过。`bash <脚本> --selftest` 经包装应当起得来是推的：包装用 `subprocess.run(command, cwd=os.getcwd())`（`scripts/prune-server/prune_server_client_selftest.py:1233`，不改环境，继承 `os.environ`），临时服务的标记在 `os.environ.update` 里设（同文件第 1199 行起）。什么现象会推翻：任一 `.sh` 的 required 自证经包装退 2 且日志写「没有临时服务」。

### 6777 里那条不符

判决说 `run-scope-guard.sh` 退 0 是「腿的预期写错」。读 `.claude/hooks/run-scope-guard.sh`（sha 0aeb251f）：「# 不要声明的形态（照常放行、不记）：」（第 54 行）下面第 60 行写「上面几类之外的命令：钩子与研究脚本的 --selftest」，这一道对这类命令不判，连子 agent 经 submit 那一条也在判之前就放了；子 agent 交 submit 的实际拒绝由另外两道钩子做（6777 日志里同一条 S 用例两道各退 2：`ok case=S scripts/crash/lib_verdict_library.py hook=heavy-test-guard.sh want=2 got=2` 与 `… hook=project-code-guard.sh want=2 got=2`）。这个解释读文件头得出，我没有起那一道钩子（起钩子是自证，写进第八节），所以是读出来的，判决「腿预期写错、不是被判句的毛病」站得住。

## 五、替被判「不采」的三条辩护

### 正推缺口 3（没写跑完看哪一行）——判「不采」：站得住，换理由

判决理由：「这一句给写报告的子 agent 看；读结局的是主 agent，读法在登记表文件头与包装输出里」（`research/prompts/d137-common-temporary-service-r1-main-verification.md:24`，sha e153406b）。
辩方先替正推说最好的话：读法并不在登记表文件头。表头 1–14 行讲的是登记写法、判定、两边对一遍，没有一句说跑完看哪一行判过（现查，读了全文）。包装输出里有一行「PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1」（6778、6779、6780 日志各有，量的），但它不改退出码：「return completed.returncode」（`scripts/prune-server/prune_server_client_selftest.py:1236`），`server_alive_at_end=0` 而内层退 0 在代码里是通过的。所以「读法在表头」那半句站不住。
不采仍然站得住，靠的是别的条款：同一条共用约束已经要求每条命令写读法，「跑完看哪几行判结果」（`.claude/agent-common.md:65`，sha d7b985eb）。包装命令不另是一类，成功句是内层脚本自己的（6778 的「✓ lib_verdict_library 自证 17 格都判对；弄坏开关 7 个，每个都红在点名那一格」）。反而有一条证据说明读法只能由写命令的人按命令逐条写、不能由这一句统一给：6778 退 0 的日志整份 4 行里第 2 行是

> ✗ 判定库没处放：/tmp/singlefs-run-with-memory-cap-1738135-AVbHV1/lib-verdict-library-selftest-vzeq7wmb/copy-tree 不是 git 工作树（取不到 git common-dir）（退 2）

（那是自证里一格有意造的负例，第 4 行的 ✓ 才是成功句）。一条「看 ✗ 行」的统一读法会把这趟成功的跑判成败。
什么现象会推翻不采：主 agent 收到的写包装命令的报告里，因为没写读法而误判过一次（比如把 `server_alive_at_end=0` 当过）。此刻没有这样的观测。残留的一点是真的：`server_alive_at_end=0` 而 exit 0 该不该算过，没有一处写，归包装那一路。

### H4（`--selftest-real-sharded-run` 裸起也退 2）——判「不归这一句，记账」：站得住，但「不归」的理由要换

事实核对：「2 不带别的参数、不在临时 prune-server 底下起同 --selftest」（`scripts/crash/crash-point-replay-shard-run.sh:19`，sha 994023f4），真跑那一格也一样：「在临时服务底下起 --selftest-real-sharded-run（带 --server-binary 就不编）」（同文件第 617 行）。这一格的包装写法跟句子给的完全相同，只是把末尾的 `--selftest` 换成它。登记表那一行写明它不进表：「真跑那一格是单独参数 --selftest-real-sharded-run」「带了 --selftest 之外参数的不进这张表，重型测试闸照起 checker 档 cargo 判重型」（`.claude/hooks/selftest-registry.tsv:118`，sha f266752a）。拆分提案已登记：「裸起也退 2，登记表与执行前拒都只认单独一个 --selftest，照通用写法交会白跑一趟」（`records/2026-10-29-subagent拆分提案.md:1024`，sha 03e90bbc）。
辩方最好的理由（该采）：句子说「required 的脚本」的「自证命令」，`--selftest-real-sharded-run` 名字里就是自证，读者可能把它也套这一句，而这一句没拦。
为什么仍然不采：表头给的整条写法末尾是固定的 `<脚本> --selftest`（`.claude/hooks/selftest-temporary-service.tsv:5`），读者照写不会写出 `--selftest-real-sharded-run`；要写它的人需要的是脚本自己的出路那一句（第 617 行，已经写对了）。真白跑的情形是主 agent 或别人裸起它，这归「执行前拒只认单独一个 --selftest」，在 `lib_project_code_runs.py` 的 `temporary_service_bare_selftest`（第 1830 行起）的射程，不是这一句的射程。
「同样打中别的写法」我数了：24 行 required 里，带第二个自证参数名的脚本我逐份 grep `--selftest-[a-z]`：`scripts/crash/crash-point-replay-shard-run.sh` 的 `--selftest-real-sharded-run` 是对外的第二种用法；`scripts/experiment-apparatus/e165-pin-red-examples.py` 的 `--selftest-library`、`--selftest-forced-judge` 是脚本内部喂给自己 main 的参数（该文件第 1896–1901 行），`scripts/gate/verus-proof-check.py` 的 `--selftest-mir` 是喂假 verus 的参数（该文件第 4166 行）；后两份不是对外用法。所以 H4 是一个孤例，不采加记账站得住。
什么现象会推翻：另有一份 required 脚本被发现带对外的第二种自证用法；或某次有人照这一句写出 `--selftest-real-sharded-run` 的包装命令被拒。

### H6（第 66 行豁免「登记为 light 的单独 `--selftest`」管不到包装形态）——判「不采，交改名那一路」：站得住

辩方先替 H6 说：此刻 `.claude/agent-common.md:66`（sha d7b985eb）还写着「登记为 light 的单独 --selftest 不算」，同一份文件第 96 行还写着「里登记为 light 且副本里这一行与主仓逐字相同」；旧闸对包装的分档是反着的：包装不带二进制算重型，带了才按内层的登记判。`.claude/hooks/selftest-registry.tsv:10` 写「没给 --server-binary 的包装要现编 prune-server，算提交时才跑的检查」（sha f266752a），所以 `[--server-binary <二进制>]` 这个方括号在旧闸下会翻档。
不采站得住的依据三条，都现查过：
1. 用户的话：「以后都是一个门禁」（`records/2026-11-05-删轻型并入重型.md:108`，sha c0374def）与「没有轻重了」（同行）；改后形态那张表第 117 行「自证登记的 light、heavy、samples-light、samples-heavy 档 | 不分档」。
2. 新闸已经没有这套分档（量的，`grep -c classify_selftest`）：`.claude/hooks/lib_project_code_runs.py`（sha 1e74cd7f）0 处、`.claude/hooks/project-code-guard.sh`（sha ed3bc47b）0 处；旧库 `lib_heavy_tests.py`（sha cd3fa91e）31 处、旧闸 `heavy-test-guard.sh`（sha 63e03a97）3 处。新库里作为独立单词出现 light 的只有一行，是一个弄坏开关的说明（`.claude/hooks/lib_project_code_runs.py:143`）；新闸里作为独立单词出现 light 的 69 行，我抽读了前 12 行，都是它自证里名叫 `light.sh`、`startup-light.sh` 的样本文件名，不是分档判法，其余没逐行读。
3. 交接已经写明：「并进规则与定义组」（`records/2026-11-05-删轻型并入重型.md:170`），并且写明以改过的这一句为「唯一写法」。
这一条的代价提醒（推的）：因为这一句将成为唯一写法，H1 里方括号的含义必须在它落成「唯一」之前定下。H6 本身不用采。什么现象会推翻不采：改名那一路交回的第 66 行与第 96 行仍留着 light 一词，或新闸里出现按 light/heavy 分档的判法。

## 六、「照表头那一条」之后同样打中别的写法吗

打中的（不是句子自己的字，是它把写法交出去之后的后果）：

1. 一处写法交给表头，另有五处各抄一份，表头那一份没有检查盯（见 H2 的表）。表头第 5 行、`.claude/hooks/selftest-registry.tsv:9`、`scripts/agents/dispatch-prompt.py:295`、`.claude/gate.d/code-tooling.sh:625`、`.claude/hooks/lib_project_code_runs.py:396` 对 `--server-binary` 的态度不一致：必写 1 处、可选 2 处、不写 3 处（把 `temporary_service_command` 算上，见 H2 的表）。H1 之后要改的是它们一起，不是只改句子。
2. 打在骨架上：`scripts/agents/dispatch-prompt.py:330` 的「有编好的 prune-server 二进制时在 --with-temporary-service 后面加 --server-binary <二进制>」把二进制当可选，而 `temporary_service_command` 拼出的点名脚本整条命令（同文件第 314 行起）不含它。H1 的退 2 落成之后，骨架给的现成命令在本机一侧会红。
3. 没打中的（站得住的）：句子只写表的路径，不点名钩子与库，改名那一路换钩子时句子不用动；表在 `.claude/hooks/` 下，所有项目子 agent 的 `tools` 含 Read，读得到；「`<解释器>`：`.py` 用 `python3`，别的用 `bash`」与 24 行的后缀对得上。

## 七、没做什么与没核到的

- 没起任何命令、没交任务、没起钩子；任务 6777–6782 只读 `task log`、`task status` 与 `task status --json`。判决里 6782（倒推第一轮快照）我只读了 `task log`；第二轮快照里 `code-tooling.sh` 与此刻 sha 对上（b4c6f008），第一轮那种倒推不出的情形这一轮没有。
- 判决「核查怎么做的」里 cite-check 报的 66 处引文我没重跑，那份在第一轮的树上。
- 「主仓二进制被 6778 重编」没有分出来，只是相容。
- 不替主 agent 撤销或维持判决，只交复核结果。

## 八、要主 agent 跑的命令

两条，都不是重型，都不写文件。都在仓根起。跑之前记下 `scripts/prune-server/prune_server_client_selftest.py` 此刻的 sha256（另一个会话还在改它，我读过 96bc4b76、28eaf674、24fd2c35、a0b77c18 四个样子），因为读数取决于它。

1. 确认 H1 的新行为（本机一侧退 2、第二台一侧退 0）。
   命令：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- printenv PRUNE_SERVER_BINARY CARGO_TARGET_DIR`
   在哪起：仓根 `/home/fy5090/code/singlefs`。
   看哪几行：`task-server/task status <号>` 的机器栏。派到本机的，预期 `TASK_SERVER_RESULT` 的 `exit=2`，日志里有「✗ 临时 prune-server 起不来：」那一行；派到第二台的，预期 `exit=0`，日志末尾有「PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1」。哪一台由服务定，只落到一台的就再交一次（或看下一条）。本机一侧也退 0 就推翻我第四节 H1 的第 7 条。
   同一条的确定版（两台都预期退 2，不依赖派到哪台）：`task-server/task submit -- env -u CARGO_TARGET_DIR -u PRUNE_SERVER_BINARY python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- true`，服务拒 `env -u` 的话用上一条；预期 `exit=2`，日志里有「环境里也没有任务服务给的编译目录」。

2. 补上 `.sh` 的 required 自证从没经包装真起过的缺口（第四节 H7）。
   命令：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary prune-server/target/release/prune-server -- bash scripts/crash/crash-point-replay-shard-run.sh --selftest`
   在哪起：仓根。这一份在 `.claude/hooks/selftest-registry.tsv:118` 登记 light（假驱动、不编、不跑、不连第二台）；带二进制路径是为了不依赖派到哪台。
   看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；日志末尾「PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1」；这份脚本自己的成功句含「格绿」。退 2 且日志写「自证没跑」就推翻「`.sh` 经包装起得来」。

## 九、结论逐条

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| H1 / 正推缺口 1（采一半） | 站不住：前提已被别的会话改掉，证据列还分不出机器；「本机那一侧」这个事实判断本身站得住 | 第四节 H1 的 1–8：`task_server.py:4249`、`task_server_lib.py:290`、`multi-host-run.sh:770`、`prune_server_client_selftest.py:1131`；6778 与 6780 两条「编译目录 无」逐字相同 |
| H2（采） | 站得住；「不再自己抄一份」只去掉共用约束里的一份，表头无检查盯，同一写法另有五处各抄一份且不一致 | `dispatch-prompt.py:287`、`:295`、`:330`；`selftest-registry.tsv:9`；`code-tooling.sh:625`；`lib_project_code_runs.py:396`、`:1782` |
| H3（采，交改名那一路） | 站得住；范围少数一处，旧名在表头第 2、6、10 行 | `selftest-temporary-service.tsv:2`、`:6`、`:10`；`code-tooling.sh:1045` |
| H4（不采，记账） | 站得住；「不归这一句」的理由改成「句子给的末尾固定是 `--selftest`、孤例、出路在脚本自己的提示」；记账已在提案 | `crash-point-replay-shard-run.sh:19`、`:617`；`selftest-registry.tsv:118`；提案 `:1024`；24 行里只有它有对外的第二种自证用法 |
| H5（采） | 站得住；结论对，6777 不是证据（只喂钩子），服务不拒前缀靠 6778–6780 与 `REFUSED_PREFIXES` | `task_server_lib.py:1138`、`:1140`；任务 6778、6779、6780 退 0 |
| H6（不采，交改名那一路） | 站得住；用户已定不分轻重、新闸没有分档判法；提醒：旧闸下方括号会翻档 | `records/2026-11-05-删轻型并入重型.md:108`、`:117`、`:170`；`classify_selftest` 命中数 0 对 31；`selftest-registry.tsv:10` |
| H7 / 正推缺口 2（采） | 站得住；规则对（24 行 14 个 `.sh`、10 个 `.py`），「6777 起得来」要改成「钩子放行」，`.sh` 没有真起过 | `selftest-temporary-service.tsv` 数的；`prune_server_client_selftest.py:1233`；任务 6778–6780 |
| 正推缺口 3（不采） | 站得住；换理由：读法不在表头，在 `.claude/agent-common.md:65`，且 6778 退 0 的日志里有一行 ✗ 说明统一读法会错判 | `agent-common.md:65`；任务 6778 日志第 2 行；`prune_server_client_selftest.py:1236` |
| 正推缺口 4（并进 H5） | 站得住 | 同 H5 |
| 6777 里 run-scope-guard 那条不符（腿预期写错） | 站得住；读文件头得出，没有起钩子 | `run-scope-guard.sh:54`、`:60`；6777 日志里两道钩子对同一用例各退 2 |
| 改法「照表头那一条」（零轮） | 同样打中别的写法：骨架 `dispatch-prompt.py:330` 与自动拼的整条命令；表头方括号在 H1 落成后的含义没定；表头无检查盯 | 第六节 |

## 十、没做什么

- 没替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，判决要引的数没复跑，命令在第八节。
- 没起钩子、没跑任何自证。判决里 6777 的钩子读数我没复跑，读到的只有 `task log 6777`。
- 没写模型，没有模型目录，草稿目录里只有日志与 `status --json` 的拷贝和一个读 `status --json` 的几行脚本，我没起过任何长驻进程。
- 禁读清单里的文件（r2 攻方腿的报告与模型目录）没读。

## 十一、补引文（上面只写了位置的行，在这里逐字对一遍；位置都带完整路径）

- 「照那张表文件头 required 一条给的整条写法套临时服务」（`.claude/agent-common.md:64`）
- 「跑完看哪几行判结果」（`.claude/agent-common.md:65`）
- 「所有任务都要多机可用，不许绑定某一台」（`.claude/agent-common.md:40`）
- 「--with-temporary-service [--server-binary <二进制>] -- <解释器> <脚本> --selftest」（`.claude/hooks/selftest-temporary-service.tsv:5`）
- 「判法在 .claude/hooks/lib_heavy_tests.py 的 temporary_service_table_at」（`.claude/hooks/selftest-temporary-service.tsv:2`）
- 「裸写的 --selftest 在执行前被 heavy-test-guard.sh 拒」（`.claude/hooks/selftest-temporary-service.tsv:6`）
- 「对一遍（lib_heavy_tests.py 的 temporary_service_table_problems」（`.claude/hooks/selftest-temporary-service.tsv:10`）
- 「--with-temporary-service --server-binary <二进制> -- <原命令>」（`.claude/hooks/selftest-registry.tsv:9`）
- 「没给 --server-binary 的包装要现编 prune-server，算提交时才跑的检查」（`.claude/hooks/selftest-registry.tsv:10`）
- 「真跑那一格是单独参数 --selftest-real-sharded-run」（`.claude/hooks/selftest-registry.tsv:118`）
- 「TEMPORARY_SERVICE_WRAPPER = "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --"」（`scripts/agents/dispatch-prompt.py:295`）
- 「有编好的 prune-server 二进制时在 --with-temporary-service 后面加 --server-binary <二进制>」（`scripts/agents/dispatch-prompt.py:330`）
- 「--with-temporary-service -- <解释器> <脚本> --selftest」（`.claude/gate.d/code-tooling.sh:625`）
- 「读法是 .claude/hooks/lib_project_code_runs.py 的 read_temporary_service_table」（`.claude/gate.d/code-tooling.sh:1045`）
- 「--with-temporary-service [--server-binary <二进制>] -- <原命令…>」（`.claude/hooks/lib_project_code_runs.py:396`）
- 「没给 --server-binary（包装要现编 prune-server）与别的交 None」（`.claude/hooks/lib_project_code_runs.py:1737`）
- 「def temporary_service_table_problems(path, registry_path, root):」（`.claude/hooks/lib_project_code_runs.py:1782`）
- 「def temporary_service_command(key):」（`.claude/hooks/lib_project_code_runs.py:1813`）
- 「if not os.environ.get(CARGO_TARGET_DIRECTORY_VARIABLE) and not break_is("build-without-target-directory"):」（`scripts/prune-server/prune_server_client_selftest.py:1131`）
- 「现编会把二进制编进主仓的 prune-server/target」（`scripts/prune-server/prune_server_client_selftest.py:1133`）
- 「completed = subprocess.run(command, cwd=os.getcwd())」（`scripts/prune-server/prune_server_client_selftest.py:1233`）
- 「return completed.returncode」（`scripts/prune-server/prune_server_client_selftest.py:1236`）
- 「build_text="CARGO_TARGET_DIR=$(printf '%q' "$peer_scratch/target") "」（`scripts/hosts/multi-host-run.sh:770`）
- 「NESTED_DISPATCH_SCRIPTS = ("run-with-memory-cap.sh", "host-dispatch.py", "multi-host-run.sh")」（`task-server/task_server_lib.py:1138`）
- 「2 不带别的参数、不在临时 prune-server 底下起同 --selftest」（`scripts/crash/crash-point-replay-shard-run.sh:19`）
- 「在临时服务底下起 --selftest-real-sharded-run（带 --server-binary 就不编）」（`scripts/crash/crash-point-replay-shard-run.sh:617`）
- 「# 不要声明的形态（照常放行、不记）：」（`.claude/hooks/run-scope-guard.sh:54`）
- 「上面几类之外的命令：钩子与研究脚本的 --selftest」（`.claude/hooks/run-scope-guard.sh:60`）
- 「以后都是一个门禁」（`records/2026-11-05-删轻型并入重型.md:108`）
- 「自证登记的 light、heavy、samples-light、samples-heavy 档 | 不分档」（`records/2026-11-05-删轻型并入重型.md:117`）
- 「并进规则与定义组」（`records/2026-11-05-删轻型并入重型.md:170`）
- 「裸起也退 2，登记表与执行前拒都只认单独一个 --selftest，照通用写法交会白跑一趟」（`records/2026-10-29-subagent拆分提案.md:1024`）
- 「或包装在没有服务给的编译目录时拒绝现编，出路写带 --server-binary；配自证格」（`records/2026-10-29-subagent拆分提案.md:1025`）
- 「tools: Read, Bash」（`.claude/agents/three-way-forward.md:4`）
