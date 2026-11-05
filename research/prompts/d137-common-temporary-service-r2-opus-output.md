# d137-common-temporary-service-r2 攻方腿（A 格）报告

日期：2026-11-05。轮：d137-common-temporary-service-r2，第二轮；前几轮判决 `research/prompts/d137-common-temporary-service-r1-main-verification.md`。攻击面：背景材料「三、格」A 格（只攻改过之后的那一句与它点到的表头，及第一轮没看过的地方）。
没写模型（文字与源码对读，没有要量的数），模型目录没建、没有 rerun.sh；要主 agent 跑的探针在末尾一节。禁读清单（同轮辩方腿的报告与模型）没读。第一轮攻方 H1–H7、正推缺口 1–4 不原样再提；碰到同根的地方写明「同 H× 的根」，只报新的那一半。

## 被判文件此刻与快照

快照 `research/prompts/d137-common-temporary-service-r2-snapshot/files-sha256.txt` 的 8 份里 7 份此刻与快照相同（sha256 前 8 位：agent-common.md d7b985eb、selftest-temporary-service.tsv 502ac94b、dispatch-prompt.py ab7e73da、heavy-test-guard.sh 63e03a97、lib_heavy_tests.py cd3fa91e、code-tooling.sh b4c6f008、第一轮判决 e153406b）。
**对不上的一份：`scripts/prune-server/prune_server_client_selftest.py`**：快照 4d35b619（= 暂存区那一份），此刻工作区 a0b77c18。我读的这段时间里它被另一个会话连改了几次（先后见到 70a4722c、a0b77c18），未暂存的改动是 H1 的根那一路：没给二进制、环境里又没有 `CARGO_TARGET_DIR` 时不再现编（见 A1）。`scripts/prune-server/prune_server_client.py` 也在改（`EXIT_USAGE = 2` 那一行在两次读之间从 264 挪到 265）。引这两份的行号是交回前最后一次 `grep -nF` 取的，主 agent 核时可能又挪了。

本报告另引的文件此刻 sha256 前 8 位：lib_project_code_runs.py 1e74cd7f、task_server.py e2201a51、task_server_lib.py 07914d6e、prune_server_client.py e3df5128、multi-host-run.sh f4efc54d、three-way-attack.md 003ee15e、three-way-inference.md 52d317ca、lib-governance-refs.py ceb73260。

被判句 `.claude/agent-common.md:64`：「登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本，写进报告的自证命令照那张表文件头 required 一条给的整条写法套临时服务」。它点到的表头那一条：`.claude/hooks/selftest-temporary-service.tsv:5`：「python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service [--server-binary <二进制>] -- <解释器> <脚本> --selftest；」

## 各格判定一览

| 号 | 面 | 判定 | 核法 |
|---|---|---|---|
| A1 | 表头的可选段 `[--server-binary <二进制>]`：写不写、写什么 | 打中：两种写法都有一台机器上错。不写：照此刻工作区的包装，派到本机退 2、什么都不判，派到第二台照编照跑（同一条命令按派到哪台分红绿）；写：表头、句子、拒绝说明都没给出可写的路径，仓里自己的门禁也从不用这两种写法（自己编一份进临时目录） | 读源码推的（包装 a0b77c18 的 `resolve_binary`、服务给编译目录的条件、multi-host-run.sh 第二台设 `CARGO_TARGET_DIR`）；探针 P1 待主 agent 跑 |
| A2 | 「这一层不算……那一串里的包装」的例外 | 打中：例外只说外层这一层不算，没说层里面那条还守不守那一串；服务的按名拒也只看外层，`timeout` 写在 `--` 外面退 3、写在 `--` 里面收下。同一个意图两种写法、一拒一收 | 读源码推的（`nested_dispatch` 只看命令词之前与命令词本身）；探针 P3 待主 agent 跑 |
| A3 | 「登在表里是 required 的脚本」在副本里怎么认 | 打中：表头第 9 行说键认的是本仓那一份；三方攻方腿（定义许它在 rsync 的副本里改代码）改了一份 required 脚本，照表头的认法它「不登在表里」、写裸的；照文件名认它要套包装。裸的那种钩子 ④ 在副本里不拦（真实路径不是主仓那一份），交上去退 2、什么都不判 | 读源码推的 |
| A4 | 表头是没人读、没人核的注释，却成了句子唯一指去的写法 | 打中：读这张表的三处代码都跳过 `#` 行；表头此刻已与包装对不上——第 7 行说 own-server 的 `prune_server_client_selftest.py --selftest`「裸跑不退 2」，此刻工作区那一份在本机没给二进制时退 2；钩子拒绝说明还写「省去现编」 | 读源码核了（注释不被读：逐份看了读表的函数）；退 2 是推的，P1 同一个机理 |
| A5 | 派发骨架「自证在哪跑：」占位里的提示 | 轻：同一份骨架给四种类型的那一行占位提示写 `task-server/task submit -- <脚本> --selftest`（裸的、没有解释器），与「要临时服务的自证」那一行、与被判句给的形态在同一份提示里并存 | 读源码核了；主 agent 填占位时会不会照抄是推的 |
| — | 表改名、表行增删、`<解释器>` 的取法、骨架之外的类型、范围声明 | 没打中（逐面见「没打中的形状」） | 读源码核了 |

## A1　表头的可选段：不写在本机退 2，写了没有可写的路径

**照这一句写出来的两种命令**（表 `.claude/hooks/selftest-temporary-service.tsv:21`：「scripts/crash/crash-point-replay.sh required」；24 行 required 每一行都一样，行数是 `grep -cP '\trequired\t'` 数的）：

    （不写可选段）task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-point-replay.sh --selftest
    （写可选段）  task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <？> -- bash scripts/crash/crash-point-replay.sh --selftest

被判句只说「照……整条写法」，表头把 `--server-binary` 写在方括号里；句子、表头都没说什么时候写、写什么。第一轮 H1 报的是「句子里没有这个位置」，判「采一半」、根交 prune-server 那一路；这里报的是那一路的修补进了工作区之后，可选段两种读法各自在哪儿错——第一轮看的那一份包装不带这条分支，第一轮没看到过。

**不写的那一种，一步步走**：

1. 钩子、服务都收：钩子 submit 里只收 ④，包装没给二进制时不剥（`.claude/hooks/lib_heavy_tests.py:3209` 起的 `command_under_temporary_service` 只剥给了的）；服务判会不会编只看命令位置上的 cargo，`task-server/task_server.py:4420`：「return any(run.subcommand in self.cargo_lock.BUILDING_SUBCOMMANDS for run in runs)」，外层是 `python3 …`，`compiles` 为假。
2. 派到本机：服务只给会编的任务编译目录，`task-server/task_server.py:4250`：「if (task.get("compiles") and not any(task["environment"].get(name) for name in lib.TARGET_DIRECTORY_VARIABLES)」，给的那一句 `task-server/task_server.py:4257`：「environment["CARGO_TARGET_DIR"] = os.path.join(build_directory, "target")」。这一条不进分支，环境里没有 `CARGO_TARGET_DIR`；`PRUNE_SERVER_BINARY` 也不在服务带进任务的名单里（`task-server/task_server_lib.py` 的 `CARRIED_VARIABLES`，第 1106 行起，逐项看了没有它）。
3. 此刻工作区的包装在这时不编、直接交「没有」：`scripts/prune-server/prune_server_client_selftest.py:1131`：「if not os.environ.get(CARGO_TARGET_DIRECTORY_VARIABLE) and not break_is("build-without-target-directory"):」，理由 `scripts/prune-server/prune_server_client_selftest.py:1133`：「现编会把二进制编进主仓的 prune-server/target」。`with_temporary_service` 照它打 `scripts/prune-server/prune_server_client_selftest.py:1230`：「print(f"  ✗ 临时 prune-server 起不来：{problem}", file=sys.stderr)」并交 `EXIT_USAGE`（`scripts/prune-server/prune_server_client.py:265`：「EXIT_USAGE = 2」）。里面那条自证一步没起，排了一趟队白交。
4. 派到第二台：multi-host-run.sh 给第二台那条命令设编译目录，`scripts/hosts/multi-host-run.sh:50`：「第二台那条命令：env -C <这一刻的工作目录> 起（同一个绝对路径），环境里 CARGO_TARGET_DIR=<这一趟的本地目录>/target」（第一轮任务 6780 在第二台打出了它，见第一轮判决「核查怎么做的」）。包装走到 `scripts/prune-server/prune_server_client_selftest.py:1134`：「return build_server_binary()」，编一份、起服务、跑自证。

同一条命令，派到本机红（退 2）、派到第二台绿：`.claude/agent-common.md:40`：「派到某一台就红的是缺陷，当场修」。哪台由服务挑，写报告的子 agent 与交它的主 agent 都定不了。

**写的那一种**：`<二进制>` 填什么，被判句、表头第 5 行都没说。能找到的说法：

- 钩子拒绝说明 `.claude/hooks/lib_heavy_tests.py:3320`：「（有编好的 prune-server 二进制时，在 {TEMPORARY_SERVICE_WRAPPER_MODE} 后面加 {TEMPORARY_SERVICE_BINARY_OPTION} <二进制>，省去现编）」——改名后的 `.claude/hooks/lib_project_code_runs.py:1850` 同一句。「省去现编」说的是不写就现编，和第 3 步此刻的包装对不上。
- 骨架 `scripts/agents/dispatch-prompt.py:330`：「（有编好的 prune-server 二进制时在 --with-temporary-service 后面加 --server-binary <二进制>）」——同样只说「有的时候」，没说在哪。
- 仓里自己起这些自证的门禁两种都不用：表写成不带二进制的包装（`.claude/gate.d/code-tooling.sh:923`：「表里写成包装写法「$SELFTEST_TEMPORARY_SERVICE_PREFIX <原命令>」，不写二进制」），跑时另取一份，`.claude/gate.d/code-tooling.sh:926`：「PRUNE_SERVER_BINARY 设了用它，没设照 prune-server-selftest 格的编法」编进这一格的临时目录、跑完删。也就是说，仓里唯一一处真跑这些自证的地方，二进制是同一趟里现编的，不是谁预先写好的路径。
- 子 agent 能写出的只有主仓的 `prune-server/target/release/prune-server`（此刻在，修改日期 2026-11-05，`ls -la` 看的）。第 3 步那句理由正是不要往那里编；第一轮判决 H1 一行也写了「子 agent 给不出可信的二进制路径」。写了它，服务收（第一轮 6779 带这个路径在第二台退 0），但起的是谁什么时候编的那一份，写的人不知道。

**四句**：
1. 分不分辨臂：这一格的臂是「句子照现在指到表头、表头可选段照现在写」/「表头把 `--server-binary` 改成必写并写明取法」/「服务给不带二进制的包装编译目录（两台都给）」。A1 分辨得出第一种与后两种：第一种在本机红；第二种还要回答「取法」那一问，答不出就仍是第一轮 H1 第四句那一格；第三种两台都绿。
2. 被判的系统看不看得到判别它的东西：看得到。钩子 `temporary_service_wrapped` 交回给没给 `--server-binary`，服务 `unwrapped_temporary_service` 剥得开，包装自己在第 3 步就知道没有编译目录；只是没有一处在提交那一刻拒，拒在排完队、起了包装之后。
3. 满足的分句：被判句「照那张表文件头 required 一条给的整条写法」加表头第 5 行的 `[--server-binary <二进制>]`——可选段两个取值，句子不给取哪个的条件。
4. 跑前条款给的改法在这几格上还中不中：第一轮采的改法（被判句指到表头）在这一格照中：它把写法交给表头，表头的可选段正是病处。只在表头补「有二进制时写」碰不到（第一轮 H1 第四句）；碰得到的在句子之外：包装与服务对「没给二进制」商定一种两台相同的结局（都编、或都在提交时拒），表头再照那一种写死。都是推的。

**什么现象推翻 A1**：P1 派到本机而 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0`（说明本机任务环境里另有 `CARGO_TARGET_DIR` 或包装又改回现编）；或另一个会话把 `resolve_binary` 那一条分支撤回。后一种只推翻 A1 的「本机退 2」那一半，「写什么路径」那一半不受影响。

## A2　例外只免了外层这一层，层里面那条守不守「不带包装前缀」没说，服务也不看

被判句的例外：`.claude/agent-common.md:64`：「这一层不算「交给服务的命令不带包装前缀」那一串里的包装」。第一轮 H5 报的是两处同叫「包装」字面互顶，判「采」，改法就是这一句例外。这里攻的是改法本身：它说外层这一层不算，没说 `--` 后面那条原命令还算不算「交给服务的命令」。读者可以读成两样：

- 读法甲：层里面那条仍是交给服务的命令，`timeout`、`nice` 这类照样不许写——限时要写只能写在外层，而外层写了服务拒。
- 读法乙：例外免的是「这一层」，层里面是包装自己起的子进程，不算交给服务的那一条，限时、优先级写在 `--` 后面不归那一串管。

**照两种读法写出的具体命令**（表 `.claude/hooks/selftest-temporary-service.tsv:22`：「scripts/crash/lib_verdict_library.py required」；想给这份自证限一个时长的读者）：

    （读法甲，写外层）task-server/task submit -- timeout 1800 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/lib_verdict_library.py --selftest
    （读法乙，写里层）task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- timeout 1800 python3 scripts/crash/lib_verdict_library.py --selftest

**服务对两条的判法**（读源码推的）：按名拒的表 `task-server/task_server_lib.py:1140`：「REFUSED_PREFIXES = {"nice": "优先级", "timeout": "限时", "capped.sh": "线程", "two-logs.sh": "日志", "gpu-quiesce-run.sh": "清场与挑卡"}」。`nested_dispatch` 先用 `command_word_index` 剥到命令词（`task-server/task_server_lib.py:1940` 起：剥掉 env、nice、timeout、setsid、bash / sh / python3 起脚本、capped.sh 之后的位置），再只看命令词之前与命令词本身：`task-server/task_server_lib.py:2000`：「for word in argv[:index]:」，`task-server/task_server_lib.py:2005`：「if name in REFUSED_PREFIXES and rest not in WRAPPER_OWN_ARGUMENTS:」。

- 外层那条：`timeout` 落在 `argv[:index]` 里，提交时拒（退 3）。
- 里层那条：命令词是 `prune_server_client_selftest.py`，`timeout` 在 `rest` 里；`rest` 只按 `RESOURCE_ARGUMENTS`（`--memory`、`--threads`、`--peer` 这类）逐词看，不按 `REFUSED_PREFIXES` 看。`inner_nested` 只钻 `-c` 那一段。服务对包装的剥法 `unwrapped_temporary_service` 只用在判编不编、要不要卡、带不带判定库三处（`task-server/task_server_lib.py:235`：「编译目录、分卡与清场只照外层那一条给。」），不用在按名拒上。收下。
- 钩子：`.claude/agent-common.md:96`：「经 submit 交给服务的那一条不判内存包装（套了包装的由服务当场拒）」，钩子把按名拒交给了服务，服务又不看包装里面。同一个推法对 `nice`、`capped.sh`、`run-with-memory-cap.sh <上限>` 写在 `--` 后面的同样成立（`run-with-memory-cap.sh` 那一判也只看命令词 `name`），都是推的。

同一个意图，读法甲被拒、读法乙被收；被收的那条正是「交给服务的命令不带包装前缀」要拦的（限时由服务定）。

**四句**：
1. 分不分辨臂：臂是「例外照现在写」/「例外后面补『`--` 后面那条照样不带那一串前缀』」/「服务按名拒也照剥开的原命令判」。A2 分辨第一种与后两种。
2. 系统看不看得到：看得到。服务手里有 `unwrapped_temporary_service`，剥出的就是里层那条；只是 `nested_problem` 不调它。
3. 满足的分句：「这一层不算……那一串里的包装」——只界定了外层，对层内沉默。
4. 改法在这一格上还中不中：第一轮 H5 采的改法就是这句例外，它在这一格照中（它正是打开读法乙的那句话）。只改句子（补一句层内照守）能消掉两种读法，但服务照收里层那条，靠读者守；服务的 `nested_problem` 对剥开的原命令再调一次 `nested_dispatch` 才碰得到收不收那一半。推的，没实现、没跑。

**什么现象推翻 A2**：P3 里层那条提交时退 3，说明服务另有一处按名拒照剥开的命令判（我没找到）。

这一节不是在造绕过的写法：两条命令都只为核「服务收不收」，P3 里把自证换成了 `printenv`。

## A3　「登在表里是 required 的脚本」：副本里改过的那一份算不算

被判句的条件是「登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本」。它指去的表头对「登在表里」有一条认法：`.claude/hooks/selftest-temporary-service.tsv:9`：「键认的是本仓里那份文件，同 selftest-registry.tsv：命令词解析后的真实路径要正是 <仓根>/<键>」。钩子照这条认：`.claude/hooks/lib_heavy_tests.py:2034`：「if os.path.realpath(resolved) == os.path.realpath(os.path.join(SELFTEST_REGISTRY_ROOT, key)):」，不是主仓那一份的，只有工作根底下、主仓登记成 light、副本里脚本与主仓逐字节相同的才认（`selftest_work_copy_problem`，第 1941 行起）。

第一轮没看过的读者：骨架不给那一行的类型里，三方攻方腿的定义许它改代码试（`.claude/agents/three-way-attack.md:51`：「要改代码试的，把仓拷到草稿目录（`rsync -a --exclude target --exclude .git`），只在副本上改」），并且把副本上的编译与跑写进报告；共用约束第 96 行也写「改过的脚本的自证不在副本里起……写进报告「要主 agent 跑的命令」一节交主 agent；主 agent 在集成副本（补丁打上之后的树）里经服务交」。两处都落在副本里，而副本里的脚本照表头第 9 行「不登在表里」。

**照两种读法写出的具体命令**（腿在副本 `<副本>` 里改了 `scripts/crash/lib_verdict_library.py`，表第 22 行 required）：

    （照表头第 9 行：不登在表里，不套）task-server/task submit --cwd <副本> -- python3 scripts/crash/lib_verdict_library.py --selftest
    （照文件名：登在表里，套）       task-server/task submit --cwd <副本> -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/crash/lib_verdict_library.py --selftest

- 不套的那条：脚本开跑先核临时服务，没有就退 2、什么都不判（表头第 3、4 行）。钩子 ④ 在这里不拦：副本里的那一份真实路径不是主仓的，也不是「与主仓逐字节相同」的那一种（改过了），`temporary_service_bare_selftest` 在 `selftest_registered_file` 那一步交 None。主 agent 照报告交上去，排一趟、退 2、什么都没判。在主仓里写错会被 ④ 当场拦、拒绝说明给整条写法，副本里没有这道兜底，被判句是唯一提醒的地方，而它的条件句照表头读恰好不罩副本。
- 套的那条：在副本里同样碰上 A1（不写二进制的话派到本机退 2）。另外副本照 rsync 拷全了才有 `scripts/prune-server/` 与 `prune-server/`；照共用约束第 49 行「副本只拷要编的」拷的副本没有这两处，包装起不来（`python3` 打不开文件，推的）。

**四句**：
1. 分不分辨臂：臂是「条件照现在写」/「条件写明『按相对仓根的路径认，副本与集成副本里的同一路径也算』」。分辨得出。第一轮几种改法都没碰条件句，这一格对它们不分辨。
2. 系统看不看得到：钩子看得到命令词的真实路径与副本根，但它的认法就是表头第 9 行那一条，副本里改过的一律不认，于是不拦。
3. 满足的分句：「登在 `.claude/hooks/selftest-temporary-service.tsv` 里是 required 的脚本」与表头第 9 行「键认的是本仓里那份文件」连读。
4. 改法在这一格上还中不中：第一轮采的改法（写法交表头）照中：表头连「认哪一份」也一起给了，给的正是不罩副本的那一种。

## A4　表头是没人读、没人核的注释，此刻已经与包装对不上

第一轮判 H2「采」，理由是句子不再自己抄一份写法、改成指到表头那一条。这一节问：表头改了谁跟、包装改了谁去改表头。

**表头没有代码读**（逐份看了读这张表的函数）：钩子库 `.claude/hooks/lib_heavy_tests.py:3224`：「if not line.strip() or line.lstrip().startswith("#"):」（跳过注释行），改名后的库 `.claude/hooks/lib_project_code_runs.py:1856` 同一句，骨架 `scripts/agents/dispatch-prompt.py:310`：「rows = [line.split(None, 2) for line in lines if line.strip() and not line.lstrip().startswith("#")]」；门禁 code-tooling 经钩子库读（第 1045 行写读法在 `lib_project_code_runs.py`）。三处都只读数据行。门禁 doc-registries 的 governance-refs 格扫共用约束只核反引号里的路径在不在（`.claude/gate.d/lib-governance-refs.py:14`：「② 反引号里的仓内路径」），表文件改名它会报，表头里那一行写法改成什么它不管。所以被判句指去的整条写法，是一段没有任何检查绑住的注释；它和钩子 `temporary_service_command`、骨架 `TEMPORARY_SERVICE_WRAPPER`、包装的真实行为之间，只靠改的人记得。

**此刻就有的三处对不上**（包装是此刻工作区 a0b77c18 那一份，另一个会话还在改）：

| 表头或拒绝说明写的 | 包装此刻做的 | 照被判句会写出的命令 |
|---|---|---|
| `.claude/hooks/selftest-temporary-service.tsv:7`：「但 --selftest 自己用 --server-binary 或 PRUNE_SERVER_BINARY 起真 prune-server，裸跑不退 2」 | `selftest()` 第 1141 行调 `resolve_binary`，没给二进制、没有 `CARGO_TARGET_DIR` 时打 `scripts/prune-server/prune_server_client_selftest.py:1143`：「print(f"  ✗ 自证起不了服务：{problem or binary + ' 不可执行'}")」并交 `EXIT_USAGE`（= 2）。派到本机就是裸跑退 2 | 表第 39 行 own-server，被判句只管 required，读者照「子 agent 不交任务」那一条的通用写法写成 `task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest`：派到本机退 2（推的，同 A1 的机理） |
| 表头第 5 行把 `--server-binary` 写成可选，不说不写会怎样 | 不写：本机退 2、第二台现编（A1） | A1 那两条 |
| 表头第 6 行指去的拒绝说明 `.claude/hooks/lib_heavy_tests.py:3320` 写「省去现编」；判档那一句 `.claude/hooks/lib_heavy_tests.py:2050`：「它要现编 prune-server（cargo build --release），不按里面那条原命令的档放行；」；盯自证进程的 `scripts/gate/selftest-process-guard.py:711` 同样说它要现编 | 本机不现编 | 读者照拒绝说明以为不写只是慢一点，写不写都行 |

第一轮判决里第一种对不上还不存在（那时包装照编），第一轮的腿看不到；它是 H1 的根那一路的修补与第一轮修补（指到表头）碰在一起之后才有的。表头第 2、6 行点名旧钩子名那一处是第一轮 H3，已交改名那一路，这里不再算。

**四句**：
1. 分不分辨臂：臂是「句子指到表头注释」/「句子指到钩子库 `temporary_service_command`（拒绝说明里那一条，代码、带自证）」/「句子照旧抄一份」。A4 分辨第一种与第二种：第二种的拼法有钩子自证盯着（`temporary_service_cells` 核拒绝说明里有包装写法），注释没有；第三种同样没人核，是第一轮 H2 那一格。
2. 系统看不看得到：看不到。没有一个检查读注释。
3. 满足的分句：被判句「照那张表文件头 required 一条给的整条写法」——把权威交给了一段注释。
4. 改法在这一格上还中不中：第一轮采的改法就是这一格的来由。只改句子碰不到；碰得到的是给表头那一行一个会红的核对（例：钩子自证里比表头第 5 行与 `temporary_service_command` 拼出的形态），或句子改指钩子拒绝说明。推的，被攻过零轮。

## A5　骨架「自证在哪跑：」占位里的提示写的是裸形态（轻）

同一份骨架给四种类型（`scripts/agents/dispatch-prompt.py:287`：「SELFTEST_RUNNER_TYPES = ("tooling-writer", "implementation-writer", "investigator", "experiment-runner")」）两行：「要临时服务的自证」那一行（第 471 行起加进去，形态与被判句一致），和「自证在哪跑：」占位。后者里的提示 `scripts/agents/dispatch-prompt.py:282`：「自证、样本档与弄坏开关拼成 `task-server/task submit -- <脚本> --selftest` 这样的整条」——裸的、连解释器也没有。共用约束第 109 行说「派发提示的「自证在哪跑：」一行写的是这些命令在哪棵树里跑（submit 的 `--cwd`），子 agent 照它填命令里的路径」，子 agent 会照这一行拼命令。

照它写出的具体命令（表第 22 行）：`task-server/task submit -- scripts/crash/lib_verdict_library.py --selftest`——这份文件没有执行位、没有 `#!`（第一轮 H7 逐行核过），起不来；即使写成 `python3 …`，也是被判句要套包装的那一种裸写。
记为轻：提示在 `<填 …>` 占位里，主 agent 填的时候可能整个换掉（推的）；同一份提示里另一行给了对的形态。第一轮只比了「要临时服务的自证」那一行与被判句（H2），没看这一行。

四句：分辨臂——不分辨被判句的几种改法（病在骨架）；系统看得到——骨架自证只核这句提示在不在（第 835 行），不核它与另一行一致；分句——第 282 行那句提示；改法——被判句怎么改都碰不到，要改骨架那句提示（写成「`<解释器> <脚本> --selftest`，表里 required 的照『要临时服务的自证』那一行」）。

## 没打中的形状

- **表改名**：被判句里表路径写在反引号里，governance-refs 格的 ② 扫共用约束的反引号路径，表改名而句子没跟会红。没打中。
- **表行增删**：被判句按「登在表里是 required」泛指，不点名哪几行；行与源码记号由 `temporary_service_table_problems` 两边对（两份钩子自证各有一格）。增一行 required，句子照罩；删一行，句子照不罩。没打中。
- **`<解释器>` 的取法**：此刻 required 24 行，`.sh` 14 行、`.py` 10 行（`cut -f1 | grep -c` 数的），「`.py` 用 `python3`，别的用 `bash`」都拼得出（第一轮 6777 都起得来）。没有别的后缀的行；以后加 `.pl` 这类才会错，此刻没有。没打中。
- **骨架之外的类型**：`.claude/agents/*.md` 里 grep `--selftest`、`temporary-service`、`自证`，只有 tooling-writer 第 29 行（写成 `task-server/task submit -- <命令>` 的整条，没给形态，不顶）与 experiment-runner 第 41 行（复跑登记，不涉自证写法）。三方腿、sweep、clerk 这些类型照被判句写不出与它们定义相顶的命令；它们会撞上的是 A1、A3。
- **「整条」含不含 `task-server/task submit --`**：表头第 5 行不含，第 6 行指去的拒绝说明含；被判句所在那一条后面第 65 行要求写成 submit 的整条。接起来只有一种，没打中。
- **`<二进制>` 占位留着不填**：`<…>` 在 shell 里是重定向，留着不填会把 `--` 当输出文件名。但 `<解释器>`、`<脚本>` 同样是占位，全仓都这么写，不归这一句。没打中。
- **弄坏开关的证红**（`env X_BREAK=… `写在包装外还是 `--` 后面）：两种写法包装都把环境传给里层；钩子剥 `env` 之后认包装；服务按 `-- env 名=值` 收。两种都对，没打中（推的，没跑）。
- **三方腿 `--cwd` 给草稿目录**（`.claude/rules/three-way-inference.md:107` 要写文件的 `--cwd` 给腿自己的草稿目录）：表头写法是相对仓根的路径，`--cwd` 换到草稿目录就打不开包装。这是一切相对路径命令的通病，不归这一句。
- **第 109 行按 light / heavy 标档**：包装形态标哪一档，与第一轮 H6 同根（用户已定不再分轻重快慢，交改名那一路），不再算。

取样范围：表 26 个数据行与 14 行表头全部；被判句所在第 64 行前后句与第 40、49、65、96、109 行；读这张表的三处代码（钩子库两份、骨架）与门禁 code-tooling 的读法与出路；服务提交时拒的四判（`nested_problem`、`verdict_library_refusal`、`temporary_service_heavy_refusal`、`compiles`）与给编译目录的条件；包装此刻工作区那一份的 `resolve_binary`、`selftest`、`with_temporary_service`；`.claude/agents/*.md` 里提到自证的地方。

## 这条腿自己的限度

- 一条命令都没交服务，也没喂钩子；A1 的「本机退 2、第二台照编」、A2 的「里层收下」、A3 的「副本里 ④ 不拦」、A4 第一行的「own-server 裸跑在本机退 2」都是读源码推的，判决要引就要主 agent 跑 P1、P3。
- `scripts/prune-server/prune_server_client_selftest.py` 在我读的期间被另一个会话改了几次，A1、A4 依赖的是它此刻工作区那一份（a0b77c18）里没暂存的一条分支；那一路若撤回这条分支，A1 的「本机退 2」与 A4 第一行一起失效，A1 的「写什么路径」、A2、A3、A4 的「注释没人核」、A5 不受影响。
- 服务挑哪台我定不了，A1 的两台各自的结局要 P1 两台各落一次才算都核过。
- 只攻 A 格；R 格（辩方）不判。A4 与 R 格背景里的例子「表头错了谁来发现」挨着：我只报此刻能指出的那几处对不上，不复核第一轮判决。

## 打中的逐条清单与改法建议

改法都只在读源码上推过、被攻过零轮。

| 号 | 打中 | 改法建议 | 状态 |
|---|---|---|---|
| A1 | 表头第 5 行可选段 `[--server-binary <二进制>]`：不写，此刻工作区的包装派到本机退 2、第二台现编；写，没有一处给出可写的路径 | 先由包装与服务商定「没给二进制」两台相同的结局（服务给这类包装编译目录，或提交时就拒并写明去哪取二进制），表头第 5 行照那一种写死、不留方括号；被判句不动 | 推的，待 P1；被攻过零轮 |
| A2 | 例外「这一层不算……那一串里的包装」对层内沉默；服务按名拒只看外层，`timeout` 写在 `--` 后面收下 | 句子补「`--` 后面那条照样不带那一串前缀」；服务 `nested_problem` 对 `unwrapped_temporary_service` 剥出的原命令再判一次 `nested_dispatch` | 推的，待 P3；被攻过零轮 |
| A3 | 「登在表里是 required 的脚本」照表头第 9 行读不罩副本里改过的那一份；那种裸写钩子 ④ 不拦，交上去退 2 | 句子条件写明「按相对仓根的路径认，副本与集成副本里的同一路径也算」 | 推的；被攻过零轮 |
| A4 | 被判句把写法交给一段没有代码读、没有检查绑住的注释；此刻表头第 7 行、拒绝说明的「省去现编」已与包装对不上 | 句子改指钩子拒绝说明（`temporary_service_command`，有钩子自证盯着），或给表头第 5 行加一格会红的核对（与 `temporary_service_command` 拼出的形态比）；表头第 7 行与几处「要现编」的说法随 A1 的商定一起改 | 注释没人读是核过的；对不上的后果推的 |
| A5 | 骨架「自证在哪跑：」占位提示写裸形态、没有解释器 | 提示改成「`<解释器> <脚本> --selftest`，表里 required 的照『要临时服务的自证』那一行」 | 轻；推的 |

## 要主 agent 跑的命令

都不是重型，都在仓根 `/home/fy5090/code/singlefs` 起；都是 python 起的脚本，不要范围声明（run-scope-guard.sh 文件头「不要声明的形态」，第一轮同形的 P1 照此交过）。等另一个会话改完 `prune_server_client_selftest.py` 再交，判决里记下交的那一刻它的 sha256。

**P1　表头不写可选段时，两台各自的结局（核 A1、A4 第一行的机理）**

    task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- printenv PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET

- 怎么判：先 `task-server/task status <号>` 看机器一栏。
  - 本机：A1 预期 `task-server/task log <号>` 里有「✗ 临时 prune-server 起不来：没给 --server-binary 与 PRUNE_SERVER_BINARY，环境里也没有任务服务给的编译目录（CARGO_TARGET_DIR）：不现编」，末行 `TASK_SERVER_RESULT` 是 `type=command-failed exit=2`，没有 `PRUNE_SERVER_TEMPORARY_SERVICE` 行。打翻：出现 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0`。
  - 第二台：A1 预期打出一条 socket 路径、`PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1`，`exit=0`（日志里可能有 cargo 编译的行）。打翻：与本机同样退 2。
- 一次只落一台；两台各落一次才算 A1 两半都核过。落在同一台就再交一次。

**P3　同一个限时写在包装外与 `--` 后面，服务收不收（核 A2）**

    task-server/task submit -- timeout 1800 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary /home/fy5090/code/singlefs/prune-server/target/release/prune-server -- printenv PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET
    task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary /home/fy5090/code/singlefs/prune-server/target/release/prune-server -- timeout 1800 printenv PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET

- 带 `--server-binary` 是为了避开 A1（不带的话派到本机先退 2，看不到服务收不收）；路径是主仓那一份，在共享根下（第一轮 6779 用过）。
- 怎么判：外层那条 `task submit` 当场退 3，说明里有「timeout …：限时由服务定，命令里不套它」（钩子先拒也算外层被拒，记下是谁拒的）；里层那条打出任务号、收下，`task wait` 末行 `exit=0`，log 里有 socket 路径与 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0`。打翻 A2：里层那条也当场退 3。
- 里层那条只起 `printenv`，跑完即止。

**引文核对（文本工具，我已直接跑）**：`python3 scripts/agents/cite-check.py research/prompts/d137-common-temporary-service-r2-opus-output.md`，末行见交回。

## 没做什么

不判辩方那一格；不替主 agent 采纳；没写模型、没有 rerun.sh；没往统一任务服务交任务，也没喂钩子，A1–A4 里推的那几步待 P1、P3；没读禁读清单里的同轮辩方腿报告与模型。草稿目录里没建仓副本、编译目录或别的文件（只有认领时留下的 `.owner` 与写者锁）。
