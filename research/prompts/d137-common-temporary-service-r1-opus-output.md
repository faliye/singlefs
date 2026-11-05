# d137-common-temporary-service-r1 攻方腿（A 格）报告

日期：2026-11-05。轮：d137-common-temporary-service-r1，第一轮，前几轮判决：无。攻击面：背景材料「三、格」A 格五面。
没写模型（这一格是文字与源码对读，没有要量的数），模型目录没建；要主 agent 跑的两条探针在末尾一节。
禁读清单（正推腿的报告与模型）没读。

## 被判文件此刻与快照

快照 `research/prompts/d137-common-temporary-service-r1-snapshot/files-sha256.txt` 的 7 份里，6 份此刻与快照相同；
**`.claude/gate.d/code-tooling.sh` 对不上**：快照 b53eaece，此刻 b4c6f008（另一个会话的改名已经改进来：读表改读 `lib_project_code_runs.py`，见 H3）。

本报告引到的文件此刻 sha256 前 8 位：
agent-common.md 2e90e2f3、selftest-temporary-service.tsv 502ac94b、dispatch-prompt.py ab7e73da、heavy-test-guard.sh 63e03a97、lib_heavy_tests.py cd3fa91e、
code-tooling.sh b4c6f008、prune_server_client_selftest.py 4d35b619、prune_server_client.py 874ab975、task_server.py e2201a51、task-server/README.md 2b34b54f、
task_server_lib.py 884e55a2（读的期间另一个会话改过两次，取交回前最后一次）、main-agent.md 3ee752df、lib_project_code_runs.py 1e74cd7f、project-code-guard.sh ed3bc47b、selftest-process-guard.py 75b15ad1、
kb-discipline.md dd25d234、e165-pin-red-examples.py d653b3af、crash-point-replay-shard-run.sh 994023f4、settings.json 91f15598、lib-governance-refs.py ceb73260。

## 各格判定一览

| 号 | 面 | 判定 | 核法 |
|---|---|---|---|
| H1 | 写法本身（`--server-binary` 漏了） | 打中：照这一句写出的命令过得了所有闸，但每次在任务里暗编一份 release 的 prune-server，编译目录落在被起那棵树的 `prune-server/target`（主工作区就落进主仓），服务看不见这次编译 | 读源码核了路径；暗编真的发生、落在哪里是推的，探针 P1、P2 待主 agent 跑 |
| H2 | 与骨架那一行重复登记 | 打中：两处已经分叉（骨架带 `--server-binary` 那半句，共用约束没带）；同一写法另有至少 6 处硬写，没有哪道门禁核共用约束里这一句与它们一致 | 读源码核了 |
| H3 | 钩子改名之后 | 部分打中：这一句本身点名的表名与包装写法在改名后的 `lib_project_code_runs.py` / `project-code-guard.sh` 里照旧成立；但它把读者指去的表头已经点着要改名的钩子与库，现在就对不上了 | 读源码核了 |
| H4 | 一份脚本有几种起法 | 打中（轻）：`crash-point-replay-shard-run.sh --selftest-real-sharded-run` 也要临时服务，这一句与钩子 ④ 都只认单独一个 `--selftest` | 读源码核了；后果是推的 |
| H5 | 「不带包装前缀」那一串 | 轻：措辞互顶、功能不顶。项目自己管这个叫「包装」，而同一条说「交给服务的命令不带包装前缀」；括号里的名单不含它，24 行 required 的单独 `--selftest` 也都不会被服务按「包装」退 3 | 读源码核了 |
| H6 | 同一份文件第 66 行的「单独 `--selftest`」 | 轻：replay.sh、crash-amplification.sh、broad-mutation.py 三行 required，照这一句写出的是包装形态，钩子把它判成「提交时才跑的检查」，不是第 66 行豁免的「登记为 light 的单独 `--selftest`」 | 读源码核了；被拒不被拒见 H1（submit 里不拒） |
| H7 | `<解释器>` 怎么取 | 没打中实害，只记一条写漏：24 行按「.py 用 python3、别的用 bash」都起得来；这一句没写这条对应（它写在钩子库和骨架里） | 逐行核了扩展名、`#!`、执行位 |
| — | own-server、表外的脚本 | 没打中：own-server 两行照这一句不套包装，裸写也不被拒；表外的自证不受这一句管 | 读源码核了 |

## H1　照这一句写出的命令每次暗编一份 prune-server，编译目录服务看不见

**照这一句写出的具体命令**（表 `.claude/hooks/selftest-temporary-service.tsv` 第 21 行 `scripts/crash/crash-point-replay.sh required`；24 行 required 的每一行都一样）：

    task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-point-replay.sh --selftest

句子原文：`.claude/agent-common.md:64`：「写进报告的自证命令要套成 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <解释器> <脚本> --selftest`」。模板里没有给 `--server-binary` 留位置。

一步步走（每一步指到许可它的那一句）：

1. 过重型闸：交给服务的那一条，闸只收 ④（裸写的要临时服务的自证），别的一律不收。`.claude/hooks/heavy-test-guard.sh:1964`：「direct_runs.extend(run for run in inner.direct_runs if run.allowed == TEMPORARY_SERVICE_BARE_LEVEL)」；文件头同一层意思 `.claude/hooks/heavy-test-guard.sh:328`：「submit 里那一条只收这一种」。没给 `--server-binary` 的包装不剥（`command_under_temporary_service` 只剥给了的），里面的 `--selftest` 不重判，不记 ④。放行。
2. 过服务的提交时拒：服务判「会不会编」只看命令位置上的 cargo（`task_server.py` 的 `compiles`，4416 行起），外层是 `python3 …`、包装剥出来的原命令是 `bash … --selftest`，都没有 cargo；单独 `--selftest` 不算要卡。收下，`compiles` 为假。
3. 服务因此不给编译目录：`task-server/task_server.py:4250`：「if (task.get("compiles") and not any(task["environment"].get(name) for name in lib.TARGET_DIRECTORY_VARIABLES)」，给 `CARGO_TARGET_DIR` 的是 `task-server/task_server.py:4257`：「environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")」。这一条不进这个分支：没有 `build=`，没有编译目录占盘的记账。
4. 包装自己现编：没给 `--server-binary`，环境里也没有 `PRUNE_SERVER_BINARY`（不在服务带进任务的白名单 `CARRIED_VARIABLES` 里），`resolve_binary` 落到 `scripts/prune-server/prune_server_client_selftest.py:1054`：「return build_server_binary()」，编法 `scripts/prune-server/prune_server_client_selftest.py:143`：「completed = subprocess.run(["cargo", "build", "--release", "--offline", "--manifest-path", manifest, "--bin", "prune-server",」，清单 `scripts/prune-server/prune_server_client_selftest.py:142`：「manifest = os.path.join(REPOSITORY_ROOT, "prune-server", "Cargo.toml")」，`REPOSITORY_ROOT` 是包装所在那棵树（`scripts/prune-server/prune_server_client.py:296`：「REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))」）。从仓根交就编进主仓的 `prune-server/target/`（`.gitignore` 第 3 行忽略它，此刻在，里面有 11-04 编的 `release/prune-server`）。
5. 主工作区里编的 cargo 本来由重型闸拒，这一次拒不到：`.claude/hooks/heavy-test-guard.sh:337`：「入口脚本自己起的 cargo 不经这一道（不读脚本正文）」。

**和哪几句相顶**：
- `.claude/main-agent.md:33`：「编译目录由服务给、不往 `<仓根>/target` 编」；`.claude/hooks/heavy-test-guard.sh:333`：「主工作区的 target/ 删了又长」（用户定案「所有脚本自行编译，完成后自行删除 不再预留生成物」）。`prune-server/target` 不是 `<仓根>/target` 字面那一个，但它就是这条定案要防的那种「主工作区里的编译目录又长出来」。
- 钩子自己把这一形态判成「提交时才跑的检查」，理由正是现编：`.claude/hooks/lib_heavy_tests.py:2050`：「它要现编 prune-server（cargo build --release），不按里面那条原命令的档放行」。盯自证进程的判法也不盯它：`scripts/gate/selftest-process-guard.py:34`：「没给 --server-binary 的包装要现编 prune-server（cargo build --release），不盯。」
- 别处给的出路都是带 `--server-binary`：钩子拒绝说明 `.claude/hooks/lib_heavy_tests.py:3320`：「有编好的 prune-server 二进制时，在」…加 `--server-binary`；客户端 `scripts/prune-server/prune_server_client.py:1109`：「（带 --server-binary 就不编）」；骨架 `scripts/agents/dispatch-prompt.py:330`：「（有编好的 prune-server 二进制时在 --with-temporary-service 后面加 --server-binary <二进制>）」；脚本自己的文件头 `scripts/experiment-apparatus/e165-pin-red-examples.py:113`：「python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <prune-server 二进制> -- python3 scripts/experiment-apparatus/e165-pin-red-examples.py --selftest」。共用约束这一句是这几处里唯一一处连「可以带」都没写的。

**推的后果（没跑，待 P1、P2）**：
- 每条这样的自证都先跑一遍 `cargo build --release`；主仓里 prune-server 源码没变时 cargo 判新、很快，变了就在主仓里重编。
- 服务两台都可能派到，仓在共享根上：第二台上跑的那一次编进同一个 `prune-server/target`，两台 RocksDB 预编库的取值按台不同（`PEER_HOST_SPECIFIC_VARIABLES`），进 librocksdb-sys 的指纹，两台轮流来就轮流重编（同一个机理 `.cargo/config.toml:5`：「两处不一样，同一个 target 目录里 RocksDB 的 C++ 就会被反复重编。」）。
- 仓副本里交（`--cwd <副本>`，副本照规矩 `--exclude target` 拷的）每次都从零编一份 release，内存按没有记录的 4G 起，撞了再放大。

**四句**：
1. 分不分辨臂：这一轮的「臂」是这一句的去留与写法——照原样留 / 补上 `[--server-binary <二进制>]` / 删掉只留骨架那一行。H1 分辨得出「照原样留」，分辨不出后两种：骨架那一行默认形态同样不带 `--server-binary`（`scripts/agents/dispatch-prompt.py:295`：「TEMPORARY_SERVICE_WRAPPER = "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --"」，带不带只是括号里一句提示）。
2. 被判的系统看不看得到：看得到。钩子 `temporary_service_wrapped` 交回「给没给 `--server-binary`」，服务的 `unwrapped_temporary_service` 也剥得开；两边都认得出这条命令会现编，只是都放行。
3. 满足的分句：「写进报告的自证命令要套成 `python3 … --with-temporary-service -- <解释器> <脚本> --selftest`」整句的模板本身，没有可选的 `--server-binary` 位置。
4. 改法在这几格上还中不中：只在句子里补「有编好的二进制时加 `--server-binary <二进制>`」碰不到打中的格：子 agent 手里没有可信的二进制路径（主仓的 `prune-server/target/release/prune-server` 可能比副本里改过的源码旧），照样会交不带的那一种。能修这一格的在句子之外：包装在服务底下不现编（没给二进制就退出、说明写去哪取），或服务把不带 `--server-binary` 的包装当「会编」、给它编译目录。都是推的，没实现、没跑。

## H2　与派发骨架那一行是第二处登记位，而且已经分叉

`.claude/singlefs-ai-sop/rules/kb-discipline.md:69`：「同一个事实只许有**一处**权威记录，别处一律链过去。」（第 4 条「矛盾比空白更糟」）

同一个写法此刻硬写在这些地方（逐处 `grep -nF` 核过）：

| 处 | 原文 | 带不带 `--server-binary` |
|---|---|---|
| 共用约束 | `.claude/agent-common.md:64`：「写进报告的自证命令要套成」 | 不带，也没说可以带 |
| 骨架 | `scripts/agents/dispatch-prompt.py:329`：「这些脚本的自证整条拼成」，下一行括号里写可以加 | 提了一句可以加 |
| 钩子库拒绝说明 | `.claude/hooks/lib_heavy_tests.py:3289` 拼整条，3320 行补「有编好的 prune-server 二进制时，在」…加 | 提了一句可以加 |
| 改名后的钩子库 | `.claude/hooks/lib_project_code_runs.py:1819` 同一段拼法 | 同上 |
| 表头 | `.claude/hooks/selftest-temporary-service.tsv` 第 5 行写成 `[--server-binary <二进制>]` | 写成可选 |
| 门禁出路 | `.claude/gate.d/code-tooling.sh:625`（runner 表的出路） | 不带（门禁自己跑时在 927 行起另插） |
| 服务 README | `task-server/README.md:103`：「剥出的原命令里有会编的 cargo、或要卡的」那一段写整条 | 写成可选 |
| 各脚本文件头 | 例 `scripts/experiment-apparatus/e165-pin-red-examples.py:113` | 带 |

骨架与共用约束两处对「`--server-binary` 能不能写」已经说法不同：照骨架写的带了，照共用约束写的不带；两种都过闸，但一种暗编、一种不编（H1）。
没有哪道门禁核共用约束里这一句与上面几处一致：门禁 doc-registries 的 governance-refs 格扫共用约束（`.claude/gate.d/lib-governance-refs.py:4`：「治理文档：CLAUDE.md、.claude/main-agent.md、.claude/agent-common.md」），但它只核路径在不在；`--with-temporary-service` 是参数名，不是路径，改了它没人报。

**反方向也核了，不是纯重复**：骨架那一行只给四种类型（`scripts/agents/dispatch-prompt.py:287`：「SELFTEST_RUNNER_TYPES = ("tooling-writer", "implementation-writer", "investigator", "experiment-runner")」）。三方三条腿、sweep、mutation-triage、experiment-designer 这些类型的派发提示里没有那一行，可它们同样会往「要主 agent 跑的命令」里写自证（三方腿的定义就要求写复跑命令）。对这几类，共用约束这一句是唯一一处。所以删掉它不行，要做的是让它不另写一份写法，而是指到唯一的权威处。

**照它会错的具体命令**：同一个三方攻方腿要验 `scripts/mutation/crates-mutation-rows.py`（表第 35 行），照共用约束写出 `task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/mutation/crates-mutation-rows.py --selftest`；同一批里实现员照骨架写出的带 `--server-binary`。主 agent 看到同一个脚本的两种写法，判不出哪种对。

四句：分辨臂——分辨「照原样留」与「改成指到表头第 5 行」；系统看得到——这是文字，没有系统在判它，所以分叉一直留着；分句——「写进报告的自证命令要套成 `…`」把写法抄了一份，而不是指过去；改法——改成「写法照 `.claude/hooks/selftest-temporary-service.tsv` 文件头 required 那一行」能消掉分叉，但 H1 那一格照中（表头的默认形态同样不带 `--server-binary`）。

## H3　钩子改名之后：句子本身站得住，句子指过去的表头已经开始烂

另一个会话的改名已经部分进了工作区：`.claude/hooks/project-code-guard.sh` 与 `.claude/hooks/lib_project_code_runs.py` 都在（暂存区新加），`.claude/settings.json:85` 挂的仍是 `heavy-test-guard.sh`。

- **句子本身不受影响（核过）**：它只点名表路径和包装写法。改名后的库读的是同一张表（`.claude/hooks/lib_project_code_runs.py:402`：「TEMPORARY_SERVICE_TABLE_NAME = "selftest-temporary-service.tsv"」，同目录），④ 原样搬了过去（`.claude/hooks/project-code-guard.sh:208`：「④ 要临时 prune-server 的自证写成裸的单独一个 --selftest」）。用户定「不再分轻重快慢」，删掉的是轻 / 重的档，required / own-server 这一判不在档里，留着。
- **句子把读者指去的表头已经对不上**：表头第 2 行 `.claude/hooks/selftest-temporary-service.tsv:2`：「判法在 .claude/hooks/lib_heavy_tests.py 的 temporary_service_table_at」，第 6 行 `.claude/hooks/selftest-temporary-service.tsv:6`：「裸写的 --selftest 在执行前被 heavy-test-guard.sh 拒」。而 `code-tooling.sh` 在快照之后已经改了口（`.claude/gate.d/code-tooling.sh:1045`：「读法是 .claude/hooks/lib_project_code_runs.py 的 read_temporary_service_table」）。同一时刻两份文件对「判法在哪」说法不同：这就是第 4 条那种矛盾，现在就有。
- **谁来同步**：表头不在 governance-refs 扫的治理文档里（那一格扫 `.md`）；`lib_heavy_tests.py` 删掉之后，表头那两行要靠改名的那个会话自己记得改。共用约束这一句如果照 H2 的改法改成「写法照表头」，表头烂了它跟着烂。

四句：分辨臂——分辨「句子指到表头」与「句子指到钩子库的 `temporary_service_command`」两种改法：后一种跟着改名走（库名会变，但 governance-refs 会报路径不在），前一种不会；系统看得到——看得到的是路径，看不到「判法在哪个库」这种说法；分句——表头第 2、6 行；改法——让表头只写写法、不写判法在哪个文件，能碰到这一格。

## H4　一份脚本有两种要临时服务的起法，句子只管一种

`scripts/crash/crash-point-replay-shard-run.sh`（表第 20 行，required）除 `--selftest` 外还有 `--selftest-real-sharded-run`：`scripts/crash/crash-point-replay-shard-run.sh:18`：「只跑真跑那一格 real-sharded-run」，`scripts/crash/crash-point-replay-shard-run.sh:19`：「2 不带别的参数、不在临时 prune-server 底下起同 --selftest。」——裸起同样退 2。

- 句子的模板只有 `<脚本> --selftest`；钩子 ④ 只认参数正好是单独一个 `--selftest`（`.claude/hooks/lib_heavy_tests.py:3308`：「or list(words[1:]) != ["--selftest"]:」）。
- 照句子字面，`--selftest-real-sharded-run` 不在它管的范围，读者照「子 agent 不交任务」那一条的通用写法写成 `task-server/task submit -- bash scripts/crash/crash-point-replay-shard-run.sh --selftest-real-sharded-run`：钩子 ④ 不拦，服务收下，脚本退 2、什么都不判，白交一趟（推的，没跑）。
- 这一种起 cargo、跑 checker 档用例，本身是重型，子 agent 写它的机会少；所以记为轻。
- 别的几个带 `--selftest-` 前缀的参数核过，不是命令行用法：e165 的 `--selftest-library`、`--selftest-forced-judge` 只在自证内部传给 main；verus-proof-check.py 的 `--selftest-mir` 是自证里的假 verus 桩。

四句：分辨臂——不分辨这一轮几种改法（三种都只写 `--selftest`），病根在表按脚本登记、不按起法登记，是共用前提；系统看得到——钩子看得到参数，但表里没有「哪几种起法要临时服务」这一栏；分句——模板里的 `<脚本> --selftest`；改法——这一句碰不到，要在表里加一栏或给这一行写理由。

## H5　「交给服务的命令不带包装前缀」与这一句：措辞顶，功能不顶

同一条前面：`.claude/agent-common.md:64`：「交给服务的命令不带包装前缀」，括号里点名九样，后面「写了提交时当场拒」。
项目自己管 `prune_server_client_selftest.py --with-temporary-service` 叫「包装」：表头 `.claude/hooks/selftest-temporary-service.tsv:4`：「必须写成包装写法」；钩子拒绝说明 `.claude/hooks/heavy-test-guard.sh:2231`：「要在包装 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 底下起」；服务在某些情形下也真把它当包装退回，`task-server/README.md:103`：「与出路（去掉包装重交）」。

核了功能：
- 服务那一侧按名字拒的前缀表与派发脚本表都不含它（`task_server_lib.py` 的 `REFUSED_PREFIXES`、`NESTED_DISPATCH_SCRIPTS`，另一个会话正在改这份文件，行号这几分钟里挪过，不写行号）。
- 服务按「包装里会编或要卡」退 3 那一判（`task_server.py` 的 `temporary_service_heavy_refusal`）：剥出来的是 `<解释器> <脚本> --selftest`，没有字面的 cargo；单独 `--selftest` 不算要卡（`task-server/task_server.py:1535` 那一行把参数正好是 `["--selftest"]` 的排除）。24 行 required 照这一句写，一行都不会退 3。

所以照字面两句会被读成「一边说不带包装，一边要套包装」，括号里的名单把它排除在外了，但是要读者自己推出来。记为轻。

## H6　同一份文件第 66 行「登记为 light 的单独 `--selftest` 不算」

第 66 行把全部实验复跑、崩溃放量、广谱变异列为重型，各带一个豁免：`.claude/agent-common.md:66`：「与登记为 light 的单独 `--selftest` 不算」。三份入口 `replay.sh`、`crash-amplification.sh`、`broad-mutation.py` 都在表里是 required（第 28、18、34 行）。
照第 64 行这一句，它们的自证要写成包装形态；包装形态没给 `--server-binary` 时，钩子判它是「提交时才跑的检查」，不是「登记为 light 的单独 `--selftest`」：`.claude/hooks/lib_heavy_tests.py:2049`：「return True, heavy_test("commit-check-selftest"」。
后果：submit 里这一判不收（H1 第 1 步），所以不会被拒；但读者对着第 66 行问「包着的这一条算不算重型」，字面答不出来（包装形态既不是「单独 `--selftest`」，第 66 行也没提包装）。记为轻；改法是在第 64 行这一句里补一句「包在临时服务里的单独 `--selftest` 照第 66 行的豁免算」，或者在第 66 行那三处豁免里补上。

## H7　`<解释器>` 怎么取

逐行核了表里 24 行 required（扩展名、首行、执行位）：

- `.sh` 的 14 行首行都是 `#!/usr/bin/env bash`；`.py` 的 10 行里 9 行首行是 `#!/usr/bin/env python3`，`scripts/crash/lib_verdict_library.py` 没有 `#!`（首行是文档串）。
- 没有执行位的 5 份：`lib_verdict_library.py`、`e165-pin-red-examples.py`、`verus-proof-check.py`、`crates-mutation-rows.py`、`mutation/tests/test_audit_regressions.py`。包装用 `subprocess.run(command)` 起（不经 shell），写成 `-- scripts/…/x.py --selftest`（带执行位直接起的写法）对这 5 份起不来；但句子要求写 `<解释器>`，照句子写不会撞上。
- 按「.py 用 python3、别的用 bash」拼，24 行都起得来，没有一行拼错。这条对应写在钩子库（`.claude/hooks/lib_heavy_tests.py:3286`：「bare = f"{'python3' if key.endswith('.py') else 'bash'} {key} --selftest"」）与骨架里，句子没写。读者写成 `sh scripts/…/x.sh` 会在 bash 写法上出错（推的，没跑）。不算打中，记一条写漏。

## 没打中的形状

- **own-server 两行**（表第 39、40 行）：照句子不套包装，裸写的 `--selftest` 不被 ④ 拒（④ 只认 required，`temporary_service_bare_selftest` 里 own-server 只在弄坏开关下才算），自证自己起服务。没打中。附带一条：`prune_server_client_selftest.py --selftest` 裸起也走 `resolve_binary`，同样暗编（H1 的同一个机理，不归这一句）。
- **表外的脚本**：句子条件是「登在表里是 required」，表外的不受它管；`.claude/gate.d/` 下的阶段（code-tooling.sh 源码里有 `selftest_service_problem`）按表头第 12 行不在对一遍的范围，它们的 `--selftest` 自己在里面套包装。没找到表外而裸起退 2 的自证（扫了自证登记表里不在这张表、源码里提到临时服务的 12 份，都是门禁阶段、钩子与工具本身）。
- **与「范围声明」「白名单环境变量」的前后句**：单独 `--selftest`（包着的、剥开后的）都不要范围声明（run-scope-guard.sh 文件头「不要声明的形态」）；包装要的标记由包装自己设，不经白名单。没顶。
- **与「写法照下一条」**：这一句给的是不带 `task-server/task submit --` 的内层，第 65 行要求写成 submit 的整条；两句接起来就是骨架那一种，没顶（字面上「套成 `python3 …`」容易被读成整条就写到这里为止，算不上打中）。
- **跨机器**：包装在共享根上的同一绝对路径，`--cwd` 在共享根下，交到第二台照样起；共享根外的路径只可能出在 `--server-binary` 给了 `/tmp` 下的二进制时（服务退 3），句子不带 `--server-binary`，碰不到。
- **服务的提交时拒**：24 行逐行照句子拼，按服务的 `compiles` 与 `gpu_need` 读法推，没有一行退 3（单独 `--selftest` 排除在要卡之外，剥出的原命令没有字面的 cargo）。是读源码推的，没交服务。

取样范围：表里 26 行全部；同一条第 64 行前后句全部；第 65、66 行；`.claude/main-agent.md`、`.claude/agents/*.md`、`.claude/rules/*.md` 里提到这张表或包装的地方（只有共用约束一处）。

## 这条腿自己的限度

- 一条命令都没交服务，也没喂钩子；H1 的「暗编真的发生、编进哪里」、H4 的「白交一趟退 2」都是读源码推的，判决要引就要主 agent 跑 P1、P2。
- `task-server/task_server_lib.py` 与 `.claude/gate.d/code-tooling.sh` 在我读的这段时间里被另一个会话改着（前者 `REFUSED_PREFIXES` 的行号在两次读之间挪了 7 行），引这两份的地方核的是此刻的样子。
- 没写模型，没有 rerun.sh。
- 只攻 A 格；F 格（正推）不判。

## 打中的逐条清单与改法建议

| 号 | 打中 | 改法建议 | 状态 |
|---|---|---|---|
| H1 | 照句子写出的形态每次在任务里暗编 prune-server，编译目录落进被起那棵树的 `prune-server/target`，服务不给编译目录、不记占盘 | 句子之外修：包装在服务底下没给二进制时不现编（退出、说明去哪取），或服务把不带 `--server-binary` 的包装当「会编」给编译目录；句子里只补「可以加 `--server-binary`」碰不到这一格 | 推的，只在读源码上推过、被攻过零轮，待 P1、P2 |
| H2 | 与骨架那一行是两处登记、已分叉；另有 6 处硬写，没有门禁核一致 | 句子不再抄写法，改成指到唯一一处（表头第 5 行，或钩子库 `temporary_service_command`），保留「登在表里 required 的自证要包在临时服务里」这一层意思（四类之外的定义只有它） | 推的，被攻过零轮 |
| H3 | 句子指去的表头第 2、6 行点着要改名的库与钩子，`code-tooling.sh` 已经改口，现在就矛盾 | 表头只写写法，不写判法在哪个文件；或交改名的那个会话在同一批里改表头 | 核过（文字比对） |
| H4 | `crash-point-replay-shard-run.sh --selftest-real-sharded-run` 也要临时服务，句子与 ④ 只认 `--selftest` | 表里给要临时服务的起法单列（或这一行的理由写明两种起法），句子写「表里登记的起法」而不是固定 `--selftest` | 推的，被攻过零轮 |
| H5 | 同一条「不带包装前缀」与「套成包装」措辞互顶 | 「不带包装前缀」那一串后面补「临时服务的包装不算，见下一句」 | 推的，被攻过零轮 |
| H6 | 第 66 行豁免「登记为 light 的单独 `--selftest`」不认包装形态 | 第 64 行这一句补「包在临时服务里的单独 `--selftest` 照第 66 行的豁免算」 | 推的，被攻过零轮 |
| H7 | （写漏，不算打中）没写 `<解释器>` 怎么取 | 补「.py 用 python3、别的用 bash」，或照 H2 指到钩子库那一处 | 核过（逐行） |

## 要主 agent 跑的命令

都不是重型。我一条都没交。

**P1　照这一句的形态，包装在任务里是不是现编、编进哪里（核 H1 第 3、4 步）**

    task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- printenv PRUNE_SERVER_BINARY CARGO_TARGET_DIR

- 在哪起：仓根 `/home/fy5090/code/singlefs`。不要范围声明（python 起的脚本，run-scope-guard.sh 文件头「不要声明的形态」）。
- 为什么里层是 `printenv`：包装先 `resolve_binary` 再起临时服务、再起里层命令（`scripts/prune-server/prune_server_client_selftest.py:1105` 起），起服务之后把用的二进制写进 `PRUNE_SERVER_BINARY` 交给里层；没给 `--server-binary`、服务又不带这个变量进任务，这个值只能来自 `build_server_binary` 读 cargo 的 JSON。换成哪一份真自证，编不编都一样，`printenv` 只是不跑别的。
- 怎么判：`task-server/task log <号>` 里
  ① 有一行 `<仓根>/prune-server/target/release/prune-server`（或别的 `…/prune-server/target/…` 路径）：说明包装在任务里调了 cargo，编进被起那棵树的 `prune-server/target`，H1 第 4 步坐实；
  ② 没有 `CARGO_TARGET_DIR` 的值（`printenv` 因此退 1，末行 `PRUNE_SERVER_TEMPORARY_SERVICE exit=1 server_alive_at_end=1`，`TASK_SERVER_RESULT` 的 `type=command-failed exit=1`）：说明服务没给编译目录，H1 第 3 步坐实；
  ③ `task wait` 末行没有 `build=`。
  反过来：①没有那一行、换成「✗ 临时 prune-server 起不来：cargo build 退 …」，说明现编失败（同样说明它试了编）；②打出了 `CARGO_TARGET_DIR` 的值、或末行有 `build=`，H1 第 3 步打翻，改判。
- 副作用：仓根 `prune-server/target` 里 prune-server 源码没变时 cargo 判新、不重编；变了会在主仓里编一次（这正是 H1 说的事）。

**P2　喂重型闸两条：照这一句写的整条放不放、`--selftest-real-sharded-run` 拦不拦（核 H1 第 1 步与 H4）**

    task-server/task submit -- bash -c 'bash .claude/hooks/heavy-test-guard.sh < /home/fy5090/.local/share/singlefs/work/d137-common-temporary-service-r1-opus/p2a-hook-input.json; echo p2a_rc=$?; bash .claude/hooks/heavy-test-guard.sh < /home/fy5090/.local/share/singlefs/work/d137-common-temporary-service-r1-opus/p2b-hook-input.json; echo p2b_rc=$?'

- 在哪起：仓根 `/home/fy5090/code/singlefs`。两份输入是我写在草稿目录里的 hook JSON（不带 `agent_type`，按主 agent 判），sha256 前 16 位 p2a c006d779f93e39c9、p2b 2401dd1c1c76e1dc；p2a 的命令是 H1 那一条整条，p2b 是 `task-server/task submit -- bash scripts/crash/crash-point-replay-shard-run.sh --selftest-real-sharded-run`。
- 怎么判：log 里 `p2a_rc=0`（放行，H1 第 1 步坐实）；`p2b_rc=0`（④ 不拦 `--selftest-real-sharded-run`，H4 坐实）。`p2a_rc=2` 打翻 H1 第 1 步；`p2b_rc=2` 而 stderr 是「要临时 prune-server 的自证写成了裸的 --selftest 被拒」的，H4 打翻。
- 两份输入用完由主 agent 删，或在这条腿的草稿目录收尾时一起删。
- 改名之后（settings 换挂 `project-code-guard.sh`）要再核一次，把命令里的钩子名换成新的那一份。

**引文核对（文本工具，我已直接跑）**：`python3 scripts/agents/cite-check.py research/prompts/d137-common-temporary-service-r1-opus-output.md`，末行见交回。

## 没做什么

不判正推那一格；不替主 agent 采纳；没写模型；没往统一任务服务交任务，判决要引的两件事（H1 的暗编、H4 的不拦）没坐实，命令在上一节。
