# d137-common-temporary-service-r1 正推腿报告（格 F）

日期：2026-11-05。立场：从定义的读者（子 agent 写「要主 agent 跑的命令」一节）出发，照 `.claude/agent-common.md` 第 64 行新加的那一句写出命令，逐处读源码核它走不走得通。
引文都在被引文件里 `grep -nF` 过；行号是此刻的行号；sha256 取此刻前 8 位。读源码核的写「读源码核」，没读到、靠别处注释或类推的写「推的」。

## 〇、被判的那一句与此刻的文件

被判句（`.claude/agent-common.md`，sha 2e90e2f3，第 64 行）：

> 登记在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本，写进报告的自证命令要套成 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <解释器> <脚本> --selftest`

照这一句加第 65 行「命令原样（照「子 agent 不交任务」那一条拼成 submit 的整条…」，读者写出的整条是：

`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <解释器> <脚本> --selftest`

下面叫它「句中形」。句中没有 `[--server-binary <二进制>]` 那一段，所以句中形是「包装不带二进制、自己现编」的那一种。

快照比对（`research/prompts/d137-common-temporary-service-r1-snapshot/files-sha256.txt`，`sha256sum -c`）：

- 对得上：`.claude/agent-common.md`（2e90e2f3）、`.claude/hooks/selftest-temporary-service.tsv`（502ac94b）、`scripts/agents/dispatch-prompt.py`（ab7e73da）、`.claude/hooks/heavy-test-guard.sh`（63e03a97）、`.claude/hooks/lib_heavy_tests.py`（cd3fa91e）、`scripts/prune-server/prune_server_client_selftest.py`（4d35b619）。
- 对不上：`.claude/gate.d/code-tooling.sh`，快照 b53eaece，此刻 b4c6f008（`git diff --stat` 对 HEAD 是 91 增 295 删，别的会话在改）。本报告不依赖它的逻辑，只引了它里面 `SELFTEST_RUNNERS` 的两三行写法当旁证（第 625、956、960 行），那几行此刻的内容见下文；它改完以后那几行可能变。
- 另一个会话已加但还没注册进 `.claude/settings.json` 的 `.claude/hooks/project-code-guard.sh`（ed3bc47b）与 `.claude/hooks/lib_project_code_runs.py`（1e74cd7f）不在快照里，第八节单独说。`.claude/settings.json`（91f15598）第 85 行此刻注册的仍是 `heavy-test-guard.sh`。

## 一、26 行登记表与句中形的「解释器」

读源码核（`.claude/hooks/selftest-temporary-service.tsv`，502ac94b）：表里 24 行 required、2 行 own-server（`scripts/prune-server/prune_server_client_selftest.py`、`scripts/prune-server/prune-server-load-simulation.py`）。24 个 required 的脚本我逐个看了首行与扩展名：

- 扩展名 `.py` 的 10 份里 9 份首行是 `#!/usr/bin/env python3`，只有 `scripts/crash/lib_verdict_library.py` 首行是文档字符串（没有 shebang）；`.sh` 的 14 份首行都是 `#!/usr/bin/env bash`。
- 没有执行位的有 `lib_verdict_library.py`、`e165-pin-red-examples.py`、`verus-proof-check.py`、`crates-mutation-rows.py`、`test_audit_regressions.py` 五份（`[ -x ]` 现查），所以「带执行位直接起」不是通用写法，必须写解释器。
- 24 份源码里 `grep -c -- '--selftest'` 都大于 0，`grep -c selftest_service_problem` 都大于 0（`selftest_service_problem` 在 `scripts/prune-server/prune_server_client.py` 第 1093 行定义，sha 874ab975）。
- 句中的 `<解释器>` 句里没解释。钩子与派发骨架是同一条规则：`.claude/hooks/lib_heavy_tests.py`（cd3fa91e）第 3283 行 `temporary_service_command` 拼 `'python3' if key.endswith('.py') else 'bash'`，`scripts/agents/dispatch-prompt.py`（ab7e73da）第 314 行 `temporary_service_command` 同样。24 份都落在 `.py`/`.sh` 两种里，规则够用；但这条规则只在代码里，读者只读句子的话要自己猜（猜 `.sh` 用 `bash`、`.py` 用 `python3` 是对的；猜别的解释器我没核，推的）。

## 二、过 `heavy-test-guard.sh`（执行前拒）

读源码核（`.claude/hooks/heavy-test-guard.sh`，63e03a97；`.claude/hooks/lib_heavy_tests.py`，cd3fa91e）：

1. 钩子入口 `task_server_decision`（第 2412 行起）先 `decide`，再判四类：绕开服务、不经服务直接跑、子 agent 起客户端、④ 裸写要临时服务的自证（第 2448–2453 行）。
2. 句中形整条是 `task-server/task submit -- …`。`heavy_uses` 里 submit 这一支（第 1955 行起）把交给服务的那条另判一遍，但只收回它的检出、绕开写法、读过的脚本，以及里面「裸写的要临时服务的自证」那一类（第 1964 行 `direct_runs.extend(run for run in inner.direct_runs if run.allowed == TEMPORARY_SERVICE_BARE_LEVEL)`），然后 `continue`。里面那一条自己的重型/提交时检查分类（`uses`）没有并回去，所以不管包装带不带 `--server-binary`，外层都不会被「提交时才跑的检查被拒」或「重型测试被拒」拒掉。
3. 句中形（不带 `--server-binary`）在里面一层的分类：`classify_selftest`（`lib_heavy_tests.py` 第 2039 行）第 2046 行 `if wrapped is not None and wrapped[1]:` 只对带二进制的返回「不是自证」；不带的走第 2048–2051 行，交一条 `commit-check-selftest` 的 heavy_test，detail 里写「它要现编 prune-server（cargo build --release），不按里面那条原命令的档放行」。这一条落在 submit 里面，按第 2 点没有并回去，所以不拒；但它说明钩子自己把「不带二进制」看作要现编的那一种，并在说明里要求写成带 `--server-binary <二进制>` 再交（同一处括号里）。
4. ④ 裸写判（`temporary_service_bare_selftest`，`lib_heavy_tests.py` 第 3300 行）要求 `list(words[1:]) == ["--selftest"]`（第 3308 行判不等就交 None）。句中形里包装自己的词是 `prune_server_client_selftest.py --with-temporary-service -- …`，不满足，不拒；带 `--server-binary` 的则 `command_under_temporary_service` 剥开，`heavy-test-guard.sh` 第 2047 行用 `environment_under_temporary_service` 补上标记，里面那条也不拒。
5. 裸写形（`task-server/task submit -- bash scripts/crash/crash-amplification.sh --selftest`）：第 1986 行调 `temporary_service_bare_selftest`，命中表里 required 就记 `TEMPORARY_SERVICE_BARE_LEVEL`，被拒，说明里给整条写法（`lib_heavy_tests.py` 第 3289 行拼出的就是句中形，加 `task-server/task submit --` 前缀）。读源码核。
6. 结论（读源码核，没喂钩子）：句中形过 `heavy-test-guard.sh`；裸写形被拒，且拒绝说明给出的整条写法与句中形一致。喂钩子的实测在「要主 agent 跑的命令」第 1 条，没跑之前这一节的「过」是读源码得出的，不是实测。

## 三、过 `task-server/task submit` 的提交时拒

读源码核（`task-server/task_server.py`，e2201a51；`task-server/task_server_lib.py`，884e55a2；写到一半时它被别的会话改过，开工读时是 1f344354，改的是文件头预计用时那一段的说明文字，本节引的函数没动，下面行号是 884e55a2 的）。服务收到提交后依次判（`task_server.py` 第 4950–4962 行一带）：环境变量白名单 → `nested_problem`（第 4955 行）→ `target_directory_refusal` → `placement_refusal`（第 3223 行，依次 ① 共享根外路径、② 共享根、③ 判定库、④ 临时服务包装里会编或要卡的）。句中形逐项：

1. 环境变量：句中形不带环境赋值；范围声明不是必须（第四点）。过。
2. `nested_dispatch`（`task_server_lib.py` 第 2013 行）：命令词是 `python3`，不在 `REFUSED_PREFIXES`（第 1161 行，只有 `nice`、`timeout`、`capped.sh`、`two-logs.sh`、`gpu-quiesce-run.sh`）；`NESTED_DISPATCH_SCRIPTS`（第 1159 行）是 `run-with-memory-cap.sh`、`host-dispatch.py`、`multi-host-run.sh`，也没有它；`RESOURCE_ARGUMENTS`（第 1162 行）里是 `--memory` 一族，`--with-temporary-service`、`--server-binary` 都不在。过。结论：句中形的包装层不是被拒的「包装前缀」。
3. ① 共享根外路径：`find_outside_shared_path`（`task_server.py` 第 3263 行）只看绝对路径的词与工作目录；句中形全是相对路径，工作目录取提交时的目录（仓根在共享根下）。过。给了 `--server-binary <二进制>` 的话，二进制路径要落在共享根下，否则这一条拒（读源码核：它判整个词是绝对路径的情形）。
4. ③ 判定库（`verdict_library_refusal`，第 3296 行）：先 `unwrapped_temporary_service`（`task_server_lib.py` 第 1322 行）剥包装，再按 `.claude/gate.d/verdict-library-scripts.tsv`（92284a8b）的第三列判。我把 24 份 required 脚本逐个对了这张表：登记在表里的 14 份，第三列要么以 `prune-server` 起头（`crash-amplification.sh`、`crash-point-replay.sh`、`device-code-key.py`、`replay.sh`、`verus-proof-check.py`、`broad-mutation.py`、`crates-mutation-rows.py`、`mutate.sh`），要么以「不算：」「无判定库：」起头（`crash-amplification-item.sh`、`-lib.sh`、`-two-hosts.sh`、`crash-point-replay-shard-run.sh`、`mutation-shard-run.sh`、`merge-and-verify.sh`，后者第 43 行是「无判定库：待立」），都是 `registry_family_of` 跳过或放行的（`REGISTRY_NO_LIBRARY_PREFIXES` 在第 3320 行用到）；其余 10 份不在表里。没有一份被拒。过。
5. ④ `temporary_service_heavy_refusal`（第 3243 行）：剥出原命令后看 `compiles` 与 `gpu_need`。`compiles`（第 4416 行）靠 `cargo_invocations`（`.claude/hooks/lib_workspace_cargo_lock.py`，300a843a，第 83 行起）找命令位置上的 cargo，原命令是 `bash X --selftest` / `python3 X --selftest`，没有 cargo，不编。`gpu_need`（第 4400 行）经 `recognize_gpu`：`script_uses_gpu`（第 1531 行）和 `declared_gpu_run`（第 1548 行）开头都有 `command.words[1:] == ["--selftest"]` 的放行，`python_entry_uses_gpu` 只认 `device-code-key.py run --feature …gpu…`。我没有实际调这几个函数（要起项目代码，列进第 2 条命令的旁证里），是按它们的条件读出的「不要卡」：推的程度——条件我读全了，但 `commands_at_command_position` 对 `bash X` / `python3 X` 把命令词落在解释器还是脚本上我没核；两种落法下 `--selftest` 的判断我读下来都是不要卡，所以结论不依赖它。
6. 小结：句中形在提交时五项都不拒（③ 读源码逐份核过，④ 读源码、推的一处如上）。实测看第 2、3 条命令。

## 四、范围声明与 `run-scope-guard.sh`

读源码核（`.claude/hooks/run-scope-guard.sh`，0aeb251f，第 60 行）：不要声明的形态里有「上面几类之外的命令：钩子与研究脚本的 --selftest、stage-selftest.sh 喂样本、python 起的脚本、仓里的工具脚本等」。句中形外层是 `python3` 起的脚本，属这一类；带 `--server-binary` 的被 `command_under_launcher` 剥开，里面 `crash-point-replay.sh`、`crash-amplification*.sh`、`merge-and-verify.sh`、`replay.sh` 的 `--selftest` 在该文件头同一段「不要声明的形态」列了。所以读者在句中形前面不用也不该加范围声明。句子里没有提，不冲突；第 64 行讲范围声明写在 submit 前面，是对要声明的命令说的。读源码核；喂钩子的实测在第 1 条命令。

## 五、`prune_server_client_selftest.py` 自己的参数解析，并真在临时服务底下起自证

读源码核（`scripts/prune-server/prune_server_client_selftest.py`，4d35b619）：

1. 入口 `main`（第 1185 行起），第 1190 行 `if arguments[:1] == ["--with-temporary-service"]:` 进 `with_temporary_service`（第 1136 行）。头一个参数必须是 `--with-temporary-service`（句中形满足）。
2. `with_temporary_service`：`--` 必须在、且后面要有命令（第 1137 行判）；`--` 之前只许 `--server-binary <路径>`，别的退 2（第 1143 行 `unknown`）。句中形 `--` 前面什么都没有，满足；`--` 后面的 `<解释器> <脚本> --selftest` 整段当命令。
3. 起服务：`temporary_service`（第 1100 行起）经 `resolve_binary`（第 1046 行）取二进制：`--server-binary` → 环境变量 `PRUNE_SERVER_BINARY` → `build_server_binary`（第 140 行）。句中形没有前两样，除非环境里恰好有 `PRUNE_SERVER_BINARY`（见下）就走第三样：第 143 行 `subprocess.run(["cargo", "build", "--release", "--offline", …])`，清单是 `<脚本所在仓根>/prune-server/Cargo.toml`（`REPOSITORY_ROOT` 在 `scripts/prune-server/prune_server_client.py` 第 296 行由脚本自己的位置算）。
4. 临时服务的环境：第 1119–1120 行把 `SOCKET_VARIABLE`、`TCP_ADDRESS_VARIABLE`、`TEMPORARY_SERVICE_VARIABLE`（值是 socket 路径）、`BINARY_VARIABLE` 设进本进程环境，然后第 1153 行 `subprocess.run(command, cwd=os.getcwd())` 起原命令，环境继承。`selftest_service_problem`（`prune_server_client.py` 第 1093 行）看的正是 `TEMPORARY_SERVICE_VARIABLE` 非空、等于 `SOCKET_VARIABLE`、`TCP_ADDRESS_VARIABLE` 非空、socket 文件在；这三样包装都设了，required 脚本开跑先核就过。所以句中形「真在临时服务底下起自证」读源码是成立的。
5. 结局怎么看：包装退原命令的退出码（第 1156 行），并打 `PRUNE_SERVER_TEMPORARY_SERVICE exit=<码> server_alive_at_end=<0|1>`（第 1155 行）。包装自己的用法错/起不来服务退 `EXIT_USAGE`（2），而 required 脚本裸跑也退 2——任务的 `exit=2` 单看分不出是哪一种，要看日志里有没有第 1155 行那一行（没有这一行就是包装没把原命令起起来，有 `exit=2` 才是原命令自己退 2）。句子和第 65 行都没有这条「跑完看哪几行」，读者只会写 `TASK_SERVER_RESULT` 的 `exit=`。这是句子缺的一块，不是错。
6. `PRUNE_SERVER_BINARY` 不在服务带进任务的环境白名单里（`task_server_lib.py` 第 1119–1124 行的 `CARRIED_VARIABLES` 没有它），所以主 agent 在自己的 shell 里 export 它不会带进任务；要给二进制只能写 `--server-binary <路径>`（路径要在共享根下，第三节第 3 点）。

不带二进制的句中形有一个后果（读源码核到的是机制，后果是推的）：服务只在 `task["compiles"]` 为真时才给任务自己的编译目录并设 `CARGO_TARGET_DIR`（`task_server.py` 第 4250–4257 行），`compiles` 只看提交的命令里命令位置上有没有 cargo（第 4416 行）；句中形的外层命令是 `python3 …`，里面的 `cargo build` 是包装脚本自己起的，服务看不见，因此不给编译目录（也不设 `CARGO_TARGET_DIR`），cargo 会按 `prune-server/Cargo.toml` 的默认 target 落在仓里的 `prune-server/target`（此刻 `ls` 有 `prune-server/target/release/prune-server`，目录被 git 忽略）。同一个问题在 `task_server.py` 第 3243 行 `temporary_service_heavy_refusal` 的说明里写过一回（任务 4229 套 cargo test 编进了仓根 target）；那一条只拦「原命令」里的 cargo，不拦包装自己的现编。钩子自己的说明（`lib_heavy_tests.py` 第 2048–2052 行）与派发骨架（`scripts/agents/dispatch-prompt.py` 第 330 行）都把 `--server-binary <二进制>` 作为有编好二进制时的写法，句子里没有。

## 六、与第 64、65 行前后句的比对

读源码核（`.claude/agent-common.md`，2e90e2f3）。同一条（第 64 行）里与这一句相邻或相关的几句：

1. 前一句「三道重型带 `--item`、clippy / build、harness 测试、证红、`--selftest`、样本档、弄坏开关……子 agent 一律写进报告「要主 agent 跑的命令」一节（写法照下一条）」：被判句紧接在后，讲其中 `--selftest` 里 required 那一部分的具体写法。不顶。
2. 被判句本身同时用了「套成」，而同一行后面写「交给服务的命令不带包装前缀（`capped.sh`、`two-logs.sh`、`gpu-quiesce-run.sh`、`nice`、`timeout`、`run-with-memory-cap.sh`、`host-dispatch.py`、`multi-host-run.sh`、`detached-run.sh`）」。字面上：被判句要求的 `prune_server_client_selftest.py --with-temporary-service --` 是一层包在命令外面的东西，而不带包装前缀那句没有把它列为例外。源码对得上被判句：第三节第 2 点读到 `nested_dispatch` 不拒它，钩子也不拒它（第二节）。不一致的只在文字：读者照「包装前缀」字面读，可能认为套了就违规；没有顶死，因为那一句点名了九样、不含这一层，是「点名」而不是「一切包装」。建议被判句里补半句「这一层不算下面的包装前缀」，或把那一串点名之后加「（临时服务的这一层除外）」。这一点攻方腿 A 的第一面会再看。
3. 旁注：那一串里的 `detached-run.sh` 在 `task-server/` 下的两份源码里 `grep -nF` 零命中（提交时拒的包装表 `REFUSED_PREFIXES` 只有五样，见第三节），它在别处是否被拒我没追（不属本格）。
4. 同一行的范围声明「`SINGLEFS_RUN_SCOPE='…' task-server/task submit -- <原命令>`」：对 `--selftest` 不要声明（第四节），被判句不涉及，不冲突。
5. 同一行「交给服务的命令不写 `SINGLEFS_HEAVY_TESTS=…` 前缀」「白名单外的写成 `-- env <名>=<值> <命令…>`」：与被判句无关；唯一的牵连是要给二进制只能用 `--server-binary`（第五节第 6 点）。
6. 第 65 行「命令原样（照「子 agent 不交任务」那一条拼成 submit 的整条……）」「跑完看哪几行（`TASK_SERVER_RESULT` 那一行的 `type=` 与 `exit=`，……成功句、`✗` 行、计数行各是什么算过）」：照被判句拼整条没问题；但「跑完看哪几行」对包装底下的自证要多看 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1` 那一行（第五节第 5 点），两处都没提。
7. 术语：被判句和 tsv、钩子文件头都用「原命令」，在包装文档里（`lib_heavy_tests.py` 第 51–54 行）它指「包装 `--` 后面那条」，在第 64 行 submit 模板里它指「提交的那条」。读被判句时这两个意思会叠在一起（提交的那条＝包装＋被包的那条）。不影响拼出的命令，只让「原命令」这个词在同一份文件里有两个指向。

## 七、与 `.claude/main-agent.md`、`.claude/agents/` 各定义的比对

读源码核（`.claude/main-agent.md`，3ee752df；`.claude/agents/tooling-writer.md`，6a2f1611；其余定义 `grep -nE '(python3|bash) [^ ]+ --selftest'` 零命中）：

- `.claude/main-agent.md` 第 19 行：「……`--selftest`、样本档、弄坏开关、命名检查、喂钩子、实验与模型复跑，与重型一样，它写进报告「要主 agent 跑的命令」一节交回；主 agent 照原样经服务交……」。没有给任何一条具体的 `--selftest` 命令写法，也没有说要临时服务。与被判句不冲突，也没有重复：它让主 agent「照原样交」，被判句保证子 agent 写进去的原样就是能交的。
- `.claude/agents/tooling-writer.md` 第 29 行「自证……写成 `task-server/task submit -- <命令>` 的整条」和第 35 行「`--selftest` 与样本档照钩子拒绝说明里写的起法」：第 35 行指向钩子的拒绝说明，那段说明（`lib_heavy_tests.py` 第 3319 行一带）拼出的整条和被判句一致（第二节第 5 点），而且多一句「有编好的二进制时加 `--server-binary`」。不冲突。
- 其余定义（`investigator.md`、`experiment-runner.md`、`implementation-writer.md`、`clerk.md`、三方四份、`mutation-triage.md`、`kb-scribe.md` 等）只写「照共用约束「不做」一节「子 agent 不交任务」」，没有任何一条 `<解释器> <脚本> --selftest` 的字面写法，所以不存在与被判句相反的示例。
- 结论：定义和 main-agent 里没有任何一处写了裸的要临时服务的自证；它们都把写法交给第 64 行，被判句是这一族命令写法的单一落点之一（另外几处见第八节末的清单）。读者（investigator 之类）要能读到被判句，前提是他们真的读 `.claude/agent-common.md` 第 64 行，各定义都有「开工前先读共用约束」，所以读得到。
- 没有定义点名 `.claude/hooks/selftest-temporary-service.tsv`；只有共用约束和派发骨架（`scripts/agents/dispatch-prompt.py` 第 294 行 `TEMPORARY_SERVICE_TABLE`）点名它。子 agent 靠被判句知道有这张表。

## 八、钩子改名、不分轻重之后这一句还成不成立

读源码核（`.claude/settings.json`，91f15598；`.claude/hooks/project-code-guard.sh`，ed3bc47b；`.claude/hooks/lib_project_code_runs.py`，1e74cd7f）：

1. 此刻 `.claude/settings.json` 第 85 行注册的仍是 `heavy-test-guard.sh`；`project-code-guard.sh` 已在 `.claude/hooks/` 里（git 状态 `A`，未注册）。
2. 新钩子把同一套判法带过去了：文件头第 208–215 行是同一段「④ 要临时 prune-server 的自证写成裸的单独一个 --selftest」，读 `.claude/hooks/selftest-temporary-service.tsv`，判函数在 `lib_project_code_runs.py` 第 1830 行 `temporary_service_bare_selftest`；submit 那一支第 1361 行 `direct_runs.extend(run for run in inner.direct_runs if run.allowed == TEMPORARY_SERVICE_BARE_LEVEL)` 与旧钩子第 1964 行是同一句。包装剥法 `command_under_temporary_service` 也在（`lib_project_code_runs.py` 第 1736 行）。
3. 被判句里点名的是表 `.claude/hooks/selftest-temporary-service.tsv` 与包装脚本 `scripts/prune-server/prune_server_client_selftest.py`，没有点名钩子的文件名或判轻重的词，所以改名与不分轻重不会让被判句的字面过期。读源码核。
4. 不分轻重后的变化只在钩子内部：旧钩子给不带二进制的包装一条 `commit-check-selftest`（第二节第 3 点）。这一条在 submit 里面本来就不并回外层，所以对「句中形过不过」没有影响；新钩子里这一类是否还在、叫什么，我没逐行核（`lib_project_code_runs.py` 里 `wrapped[1]` 那一句在第 1736–1739 行，同样只对带二进制的剥开）。推的：新钩子注册后句中形仍过，要喂钩子才知（第 1 条命令已把 `project-code-guard.sh` 列进去）。
5. 表头（`selftest-temporary-service.tsv` 第 14 行）写「派发提示骨架（scripts/agents/dispatch-prompt.py）按 required 行给子 agent 拼整条命令」，骨架里也有同样的句中形（`dispatch-prompt.py` 第 295 行定义 `TEMPORARY_SERVICE_WRAPPER`，第 314–317 行拼整条）。所以这一族命令的写法此刻有六处写着同一串包装：表头第 5 行、`lib_heavy_tests.py` 第 3289 行、`dispatch-prompt.py` 第 295 行、`heavy-test-guard.sh` 文件头第 326 行、`code-tooling.sh` 第 625 行（此刻的 sha b4c6f008，快照对不上那一份）、被判句。其中有 `[--server-binary <二进制>]` 的是表头、钩子说明、骨架、客户端文件头；被判句和 `code-tooling.sh` 的 SELFTEST_RUNNERS 写法（第 956、960、962 行）没有。这是 A 格的「第二处登记位」问题，F 只把事实摆出来。

## 九、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| F1 过 `heavy-test-guard.sh` 的执行前拒 | 一致（读源码核，待喂钩子实测） | 句中形整条在 submit 里，里面的分类不并回外层、④ 要求参数正好是 `--selftest`，不拒；裸写形被拒，拒绝说明给的整条与句中形相同（第二节） |
| F2 过 `task submit` 的提交时拒 | 一致（读源码核；`gpu_need` 一处推的） | `nested_dispatch` 不认这层为包装前缀，路径全相对，24 份脚本逐个对过判定库登记表无一被拒，原命令不编不要卡（第三节） |
| F3 过 `prune_server_client_selftest.py` 自己的参数解析 | 一致（读源码核） | `--with-temporary-service` 开头、`--` 后有命令即可；`--` 前只许 `--server-binary`（第五节第 1–2 点） |
| F4 真在临时服务底下起自证 | 一致（读源码核，待真跑） | 包装设三样环境变量再起原命令，`selftest_service_problem` 正好核这三样（第五节第 4 点） |
| F5 句中形不带 `--server-binary` | 句子缺一块 | 包装自己现编 prune-server，服务看不见这次编译：不给编译目录，cargo 落仓里 `prune-server/target`（机制读源码核、后果推的）；钩子说明、表头、骨架、客户端文件头都写了可选的 `--server-binary`（第五节末、第八节第 5 点） |
| F6 「解释器」怎么取 | 句子缺一块（不致错） | 规则（`.py` 用 python3，别的用 bash）只在钩子和骨架的代码里，24 份都落在这两类（第一节） |
| F7 跑完看哪几行 | 句子缺一块 | 包装退 2 与原命令退 2 分不开，要看 `PRUNE_SERVER_TEMPORARY_SERVICE exit=… server_alive_at_end=…` 那一行（第五节第 5 点） |
| F8 与同一条前后句 | 文字上有一处疑似顶（不是真顶） | 被判句要求「套」一层，同一行「不带包装前缀」点名九样不含这一层；源码两边都放行，建议补半句例外（第六节第 2 点） |
| F9 与 `.claude/main-agent.md`、`.claude/agents/` 各定义 | 一致 | 没有一处写了裸写法，都把写法交给第 64 行（第七节） |
| F10 钩子改名、不分轻重之后 | 一致（读源码核；新钩子未注册，待喂钩子） | 被判句没点名钩子文件名；新钩子带着同一套 ④ 与同一张表（第八节） |
| 快照比对 | `code-tooling.sh` 与快照不一致 | 快照 b53eaece，此刻 b4c6f008；本报告不依赖其逻辑，只引了 3 行写法作旁证 |

## 十、结论

站得住：照被判句写出的整条（`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <解释器> <脚本> --selftest`）读源码能走通：钩子不拒、提交时不拒、包装自己的参数解析收、包装设的环境正好满足 required 脚本开跑的核对。裸写会被钩子拒，拒绝说明给的与被判句是同一条。这个结论是读源码得出的，三条真跑（下节）没有跑之前不算实测。

站不住或不够的地方（都不是「命令跑不通」，是「读者只读这一句会少什么」）：

1. 少了可选的 `[--server-binary <二进制>]`。表头、钩子拒绝说明、派发骨架、客户端文件头四处都写了它，被判句和 `code-tooling.sh` 的 runner 表没有写。不带二进制时包装会自己 `cargo build --release`，服务没有给它编译目录（第五节末）。能不能带、带哪一份二进制两台都读得到（`prune-server/target/release/prune-server` 此刻在仓里，但它是本机编的，第二台读同一绝对路径的是同一份共享目录，是不是两台都新鲜我没核），这是下一步要定的事，F 不替它定。
2. 没说 `<解释器>` 怎么取（`.py` → `python3`，别的 → `bash`）。
3. 没说跑完看哪几行（多看 `PRUNE_SERVER_TEMPORARY_SERVICE exit=… server_alive_at_end=…`）。
4. 「套成」与同一行「不带包装前缀」文字上挨着，没写例外。

建议的改法（只动被判句，保持「只写怎么做」；`[...]` 是可选）：

> 登记在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本，写进报告的自证命令要套成 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <二进制>] -- <解释器> <脚本> --selftest`（`.py` 用 `python3`，别的用 `bash`；这一层不算下面的包装前缀；跑完另看日志里 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1` 那一行）

要不要把 `[--server-binary …]` 写进句子、写了之后二进制从哪来，取决于第一点里那个我没核的两台问题，所以我只给改法、不判这一格的取舍。

什么现象会推翻上面各条：
- F1、F2：喂钩子（第 1 条命令）有任何一行 `MISMATCH`，或真提交（第 2、3 条）被退 3；
- F4：第 2 条的日志里没有 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1`，或脚本报「没有临时 prune-server」；
- F5 的后果：第 2 条跑前后 `du -sm prune-server/target` 不涨（二进制是最新的，cargo 不用编，我这个推断在这一次测不出来，要把 `prune-server/src` 碰一下才测得出，F 不碰）；`task status <号>` 的编译目录一栏有值（说明服务给了目录，我读错了）；
- F10：新钩子注册后第 1 条命令里 `project-code-guard.sh` 的任一行 `MISMATCH`。

## 要主 agent 跑的命令

1. 喂钩子（轻；起 162 次钩子进程，检出记录写到 `--out` 下，不进工作根）。草稿脚本 `/home/fy5090/.local/share/singlefs/work/d137-common-temporary-service-r1-sonnet/judge_sentence.py`（`python3 -m py_compile` 过；每个钩子进程放进自己的进程组，超时与结束都对整组发 SIGKILL）。
   命令：`task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 /home/fy5090/.local/share/singlefs/work/d137-common-temporary-service-r1-sonnet/judge_sentence.py --repo /home/fy5090/code/singlefs --out /home/fy5090/.local/share/singlefs/work/d137-common-temporary-service-r1-sonnet/judge-out`
   在哪个目录起：仓根 `/home/fy5090/code/singlefs`。看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`，日志末行 `SENTENCE_JUDGE cases=54 mismatches=0`；有 `MISMATCH` 行的逐行读（`case=` 是 W 句中形、B 裸写形、O 自己起服务的两份、WB 带二进制、S 子 agent 提交）。`hook=project-code-guard.sh` 那几行出错是第八节的信息，不是被判句的错。
2. 真起一份（不带二进制，句中形原样；轻，包装会现编 prune-server）。提交前先 `du -sm prune-server/target`，提交后再量一次。
   命令：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/lib_verdict_library.py --selftest`
   在哪个目录起：仓根。看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号>` 里有 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1`；`task-server/task status <号>` 看编译目录一栏是否为空（F5）。
3. 真起一份（带二进制）。
   命令：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary /home/fy5090/code/singlefs/prune-server/target/release/prune-server -- python3 scripts/mutation/crates-mutation-rows.py --selftest`
   在哪个目录起：仓根。看哪几行：同第 2 条；这一份是任务 2731 退 2 的那个脚本，期望不再退 2。二进制路径此刻在仓里存在（`ls` 核过），文件是不是新的我没核。

等这三条结果才能往下做的：F1、F2、F4、F10 的「待实测」转实测；F5 后果的核对。

## 没做什么

- 不判别的腿（A 格）的面：只在第六、八节把事实摆出来，没有替攻方判「两句互相否定」「第二处登记位」是不是缺陷。
- 没往统一任务服务交任务，也没起 `run_in_background`；喂钩子、真起自证、判决要引的数都在上面三条命令里，没复跑。
- 没写模型目录（这一格没有模型，只有一份草稿脚本）。
- 没调 `task_server.py` 的 `compiles`、`gpu_need`、`placement_refusal` 等函数，也没喂 `heavy-test-guard.sh`、`run-scope-guard.sh`、`project-code-guard.sh`：这些都要起项目代码，第三、二、八节的「过」是读源码得出的。
- 没碰禁读清单里攻方腿的报告与模型目录。
- `.claude/gate.d/code-tooling.sh` 与快照不一致，没追是谁改的。

## 附：逐字引文（供 `cite-check.py` 核；每处是此刻的行号，sha 前 8 位写在括号后）

- 「写进报告的自证命令要套成」（`.claude/agent-common.md:64`，2e90e2f3）
- 「交给服务的命令不带包装前缀」（`.claude/agent-common.md:64`，2e90e2f3）
- 「命令原样（照「子 agent 不交任务」那一条拼成 submit 的整条」（`.claude/agent-common.md:65`，2e90e2f3）
- 「if wrapped is not None and wrapped[1]:」（`.claude/hooks/lib_heavy_tests.py:2046`，cd3fa91e）
- 「它要现编 prune-server（cargo build --release）」（`.claude/hooks/lib_heavy_tests.py:2050`，cd3fa91e）
- 「return f"python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}」（`.claude/hooks/lib_heavy_tests.py:3289`，cd3fa91e）
- 「list(words[1:]) != ["--selftest"]」（`.claude/hooks/lib_heavy_tests.py:3308`，cd3fa91e）
- 「run.allowed == TEMPORARY_SERVICE_BARE_LEVEL」（`.claude/hooks/heavy-test-guard.sh:1964`，63e03a97）
- 「environment_under_temporary_service」（`.claude/hooks/heavy-test-guard.sh:2047`，63e03a97）
- 「run.allowed == TEMPORARY_SERVICE_BARE_LEVEL」（`.claude/hooks/project-code-guard.sh:1361`，ed3bc47b）
- 「heavy-test-guard.sh」（`.claude/settings.json:85`，91f15598）
- 「钩子与研究脚本的 --selftest」（`.claude/hooks/run-scope-guard.sh:60`，0aeb251f）
- 「按 required 行给子 agent 拼整条命令」（`.claude/hooks/selftest-temporary-service.tsv:14`，502ac94b）
- 「--server-binary <二进制>」（`scripts/agents/dispatch-prompt.py:330`，ab7e73da）
- 「REFUSED_PREFIXES = {"nice"」（`task-server/task_server_lib.py:1161`，884e55a2）
- 「RESOURCE_ARGUMENTS = ("--memory"」（`task-server/task_server_lib.py:1162`，884e55a2）
- 「if (task.get("compiles") and not any」（`task-server/task_server.py:4250`，e2201a51）
- 「environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")」（`task-server/task_server.py:4257`，e2201a51）
- 「def temporary_service_heavy_refusal」（`task-server/task_server.py:3243`，e2201a51）
- 「subprocess.run(["cargo", "build", "--release", "--offline"」（`scripts/prune-server/prune_server_client_selftest.py:143`，4d35b619）
- 「subprocess.run(command, cwd=os.getcwd())」（`scripts/prune-server/prune_server_client_selftest.py:1153`，4d35b619）
- 「PRUNE_SERVER_TEMPORARY_SERVICE exit=」（`scripts/prune-server/prune_server_client_selftest.py:1155`，4d35b619）
- 「if arguments[:1] == ["--with-temporary-service"]:」（`scripts/prune-server/prune_server_client_selftest.py:1190`，4d35b619）
- 「if marked != environment.get(SOCKET_VARIABLE, "") or not environment.get(TCP_ADDRESS_VARIABLE, ""):」（`scripts/prune-server/prune_server_client.py:1102`，874ab975）
- 「它写进报告「要主 agent 跑的命令」一节交回」（`.claude/main-agent.md:19`，3ee752df）
- 「照钩子拒绝说明里写的起法」（`.claude/agents/tooling-writer.md:35`，6a2f1611）
- 「无判定库：待立」（`.claude/gate.d/verdict-library-scripts.tsv:43`，92284a8b）
