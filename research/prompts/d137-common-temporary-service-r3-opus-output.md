# d137-common-temporary-service-r3 攻方腿报告（A 格，2026-11-05）

被判：`.claude/agent-common.md` 第 64 行那一句与它点到的登记表文件头。没写模型目录（`research/prompts/d137-common-temporary-service-r3-opus-model/` 没建）；草稿脚本与交接副本在草稿目录 `/home/fy5090/.local/share/singlefs/work/d137-cts-r3-attack/`。

## 开工快照与此刻的 sha256

`sha256sum -c research/prompts/d137-common-temporary-service-r3-snapshot/files-sha256.txt`：10 份全部 OK（量的，开工时跑）。下面引到的文件此刻 sha256 前 8 位（量的，`sha256sum`）：

| 文件 | 前 8 位 | 在快照里 |
|---|---|---|
| `.claude/agent-common.md` | 9c5ba65e | 在，对上 |
| `.claude/hooks/selftest-temporary-service.tsv` | a3fcc2ca | 在，对上 |
| `.claude/hooks/lib_project_code_runs.py` | 4619ee9b | 在，对上 |
| `scripts/agents/dispatch-prompt.py` | b5f0023e | 在，对上 |
| `scripts/prune-server/prune_server_client_selftest.py` | a0b77c18 | 在，对上 |
| `task-server/task_server.py` | 85c94505 | 在，对上 |
| `task-server/task_server_lib.py` | 839ad871 | 在，对上 |
| `scripts/prune-server/prune_server_client.py` | e3df5128 | 不在 |
| `scripts/prune-server/prune-server-load-simulation.py` | 7ae53321 | 不在 |
| `.claude/hooks/selftest-registry.tsv` | 861a2e46 | 不在 |
| `.claude/gate.d/code-tooling.sh` | b4c6f008 | 不在（与第二轮快照那一份相同） |
| `.claude/hooks/lib_heavy_tests.py` | cd3fa91e | 不在（与第二轮快照那一份相同） |
| `prune-server/control` | 97c9f2a7 | 不在 |

草稿脚本 `feed_project_code_guard.py` sha256 前 16 位 65a991992ae0bf78。

## 各格判定一览

| 号 | 打的是什么 | 判 | 量的 / 推的 |
|---|---|---|---|
| A6 | 句子只给 `<解释器>` 的取法，整条写法里另一个占位 `<编好的 prune-server>` 没有取法；表头第 5 行（必带）与第 6 行（不带也行）给两个答案，拒绝说明、骨架、自证登记表、客户端文件头站第 5 行，门禁 code-tooling 站第 6 行；第二轮之后从「可选」改成了「必带」 | 打中 | 各处原文量的（grep）；改前改后量的（暂存区那一份对此刻） |
| A7 | 照第 5 行能填的值：直抄占位在 shell 里就退 1；能找到的路径只有主仓那份二进制，它比 12 份 prune-server 源码旧，现在没有哪条认可的路子会刷新它；受认可的编法编出的二进制结局时删、第二台的在非共享盘上 | 打中 | shell 退 1 量的（草稿目录里同形 echo）；12 份量的（find -newer）；钩子与服务的结局推的，命令在后 |
| A8 | 第二轮 A3 加的「仓副本里的同一份也算」：照第 49 行拷的副本不带 `prune-server/`，不带二进制的写法两台都退 2；带相对路径的同样两台都退 2 | 打中（推的） | 读源码；命令「副本缺源码」 |
| A9 | 前两轮交出表漏项：own-server 两行裸起 `--selftest` 不带二进制，服务不当会编的算，本机退 2、第二台能跑，与第 40 行相顶。属任务服务那一路 | 漏项（推的） | 读源码；命令「own-server 裸起」 |
| N | 前两轮报的数里推的、没量过的 7 个 | 逐个点名 | 见「推的数」一节 |

## A6　整条写法里有两个占位，句子只说了一个

句子 `.claude/agent-common.md:64`「照那张表文件头 required 一条给的整条写法套临时服务（`<解释器>`：`.py` 用 `python3`，别的用 `bash`」。它给了 `<解释器>` 的取法，没给 `<编好的 prune-server>` 的取法。

表头 required 一条（`.claude/hooks/selftest-temporary-service.tsv`，a3fcc2ca）里，两行给两个答案：

- 第 4、5 行：`.claude/hooks/selftest-temporary-service.tsv:4`「必须写成包装写法」，紧接 `.claude/hooks/selftest-temporary-service.tsv:5`「--with-temporary-service --server-binary <编好的 prune-server> -- <解释器> <脚本> --selftest」。这是整条写法，`--server-binary` 不在方括号里。
- 第 6 行：`.claude/hooks/selftest-temporary-service.tsv:6`「两台都给编译目录，包装现编一遍 prune-server」，即不带也行。
- 第 7 行：`.claude/hooks/selftest-temporary-service.tsv:7`「拒绝说明给整条写法（带 --server-binary」。

别处给整条写法的地方，逐处现查（量的，`grep -nF`）：

| 读者从哪拿 | 给的写法 | 出处 |
|---|---|---|
| 拒绝说明（`project-code-guard.sh` 拒裸写时） | 带 `--server-binary <编好的 prune-server>`，并叫换成路径 | `.claude/hooks/lib_project_code_runs.py:1788`「换成编好的 prune-server 二进制的路径」；占位常量 `.claude/hooks/lib_project_code_runs.py:434`「TEMPORARY_SERVICE_BINARY_PLACEHOLDER = "<编好的 prune-server>"」；文件头 `.claude/hooks/lib_project_code_runs.py:78`「出路写给 --server-binary」 |
| 派发骨架（四种类型） | 带 `--server-binary <编好的 prune-server>`，点名的脚本拼成「整条」 | `scripts/agents/dispatch-prompt.py:295`「TEMPORARY_SERVICE_WRAPPER = "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <编好的 prune-server> --"」；括注 `scripts/agents/dispatch-prompt.py:330`「--server-binary 后面是编好的 prune-server 二进制，没给它又没有任务服务给的编译目录时包装不现编、退 2」；文件头 `scripts/agents/dispatch-prompt.py:203`「这一行不是占位，原样留在提示里」 |
| 自证登记表 | 带 `--server-binary <二进制>`；23 行理由写「起法 … --server-binary <prune-server 二进制> -- …」 | `.claude/hooks/selftest-registry.tsv:10`「--with-temporary-service --server-binary <二进制> -- <原命令>」（861a2e46）；23 行是 `grep -o` 数的 |
| 客户端文件头 | 出路带 `--server-binary` | `scripts/prune-server/prune_server_client.py:112`「出路是带 --server-binary <编好的 prune-server>」（e3df5128） |
| 门禁 code-tooling 的出路与 runner 表 | 不带 | `.claude/gate.d/code-tooling.sh:625`「--with-temporary-service -- <解释器> <脚本> --selftest」（b4c6f008） |
| 旧钩子库（已不在 settings 里注册） | 不带，另说「省去现编」 | `.claude/hooks/lib_heavy_tests.py:3320`「省去现编」（cd3fa91e） |

同一个占位写成三种字样：`<编好的 prune-server>`、`<prune-server 二进制>`、`<二进制>`。

**第二轮之后往「必带」那边改了（量的）。** 暂存区里的登记表（sha256 502ac94b，即第二轮快照那一份）第 5 行写的是 `[--server-binary <二进制>]`（`git show :.claude/hooks/selftest-temporary-service.tsv` 第 5 行）；暂存区里的骨架（ab7e73da）第 295 行 `TEMPORARY_SERVICE_WRAPPER` 不带 `--server-binary`。此刻两处都改成了必带的占位。A1 的根落地之后，不带二进制在主工作区两台都能起（服务侧看 A7 与「推的数」N1），而三处文字同时往「必须填一个路径」那边改了，方向与根的修法相反。

**读者会写出哪一种**：只读句子、照「整条写法」抄的，拿到第 5 行，要么原样带占位（A7 甲），要么找一个路径填（A7 乙、丙）；读到第 6 行的会去掉它（A7 丁）。拒绝说明与骨架的读者只看得到「填路径」，括注还说没给它可能退 2，会去找路径。

四句：
1. 分辨臂：分。臂是「带路径」与「不带」：主工作区里不带两台都给编译目录，带的那一臂取不到又新又两台都读得到的值（A7）。
2. 系统看不看得到：写报告的子 agent 看不到会派到哪台、看不到主仓那份二进制新不新；三处文字也没让它去看。
3. 判据分句：`.claude/agent-common.md:40`「派到两台里任何一台都要跑得起、判得对」的「判得对」（A7 丙）；`.claude/agent-common.md:65`「命令原样」（A7 甲：原样交不出去）。
4. 第二轮改法在这几格中不中：第二轮的改法表写「表头随之写清带不带二进制」（`research/prompts/d137-common-temporary-service-r2-main-verification.md:39`「表头随之写清带不带二进制」）。表头写了两样、没写清，所以这一格照中；根的那一半（服务给编译目录）只修 A7 丁，修不了甲、乙、丙。

打翻条件：仓里有一处写明 `<编好的 prune-server>` 从哪取，而且取到的值两台读得到、与此刻源码同一版；或者喂钩子（命令「喂钩子」）的 `literal-placeholder` 一行退 2 且拒绝说明给出可填的值。

## A7　照第 5 行填 `<编好的 prune-server>`：四种写法各落到哪

主工作区里起（`--cwd` 仓根）的四种写法；副本里起的在 A8。「量的」都是这一轮现跑或现读的，「推的」给了命令。

| 写法 | 钩子（主 agent 交） | 提交时 | 本机 | 第二台 |
|---|---|---|---|---|
| 甲：原样带占位 `--server-binary <编好的 prune-server>`（骨架给的「整条」、拒绝说明给的整条，抄下就是这样） | 推的，命令「喂钩子」的 `literal-placeholder` | 到不了：bash 把 `<编好的` 当输入重定向，命令不执行（量的，见下） | 不跑 | 不跑 |
| 乙：填相对路径 `prune-server/target/release/prune-server` | 推的，`binary-main-relative` | 收（给了二进制，`compiles` 假） | 同丙 | 同丙 |
| 丙：填主仓绝对路径 `/home/fy5090/code/singlefs/prune-server/target/release/prune-server` | 推的，`binary-main-absolute` | 收；路径在共享根下（量的，见下） | 起得来，用的是旧二进制（推的，命令「旧二进制自证」） | 第一轮 6779 退 0（第一轮判决，量的）；用的是同一份旧二进制 |
| 丁：不带（表头第 6 行） | 推的，`binary-omitted` | 收；`compiles` 真（量的，任务 7030） | 推的：服务给编译目录、现编（命令「不带二进制落本机」） | 7030 派到第二台、此刻在跑；第二轮 6834、6835 退 0（改前） |

**甲在 shell 里就退 1（量的）。** 在草稿目录里起同形的一条（只把 `task-server/task submit -- python3 …` 换成 `echo x`）：

```
$ echo x --server-binary <编好的 prune-server> -- bash scripts/x.sh --selftest; echo "rc=$?"
/bin/bash: line 1: 编好的: No such file or directory
rc=1
```

起前起后 `ls -A` 一样，没有多出名叫 `--` 的文件。骨架给点名脚本拼的就是这一形（`scripts/agents/dispatch-prompt.py:314` 的 `temporary_service_command` 把 295 行的常量原样接上），而 203 行说这一行「原样留在提示里」；派发闸只认 `<填 …>` 这一种占位（`.claude/hooks/runner-dispatch-guard.sh:15`「⑬ 骨架的占位没填完：提示里还有 `<填 …>`」），`<编好的 prune-server>` 不在它判的范围里。

**丙、乙用的那份二进制是旧的（量的）。** `find prune-server/src prune-server/Cargo.toml prune-server/Cargo.lock -newer prune-server/target/release/prune-server -type f | wc -l` 打出 12；逐个是 `bin/prune_server.rs`、`break_switches.rs`、`client.rs`、`committer.rs`、`lib.rs`、`protocol.rs`、`reliable_client.rs`、`selftest_fault_cells.rs`、`selftest_library_layout_cells.rs`、`selftest.rs`、`server.rs`、`storage.rs`（都在 `prune-server/src/` 下）。二进制修改日期 2026-11-05。

现在没有哪条认可的路子会刷新主仓这一份（读源码，推的）：
- 包装不再往主仓编：`scripts/prune-server/prune_server_client_selftest.py:1133`「现编会把二进制编进主仓的 prune-server/target」，没有编译目录就退 2。
- 写明怎么编二进制的只有 `prune-server/control:136`「task-server/task submit --cwd <仓根>/prune-server -- cargo build --release --offline --bin prune-server」：经服务编，落在服务给的编译目录里。本机那一份结局时删（`task-server/task_server_lib.py:300`「任务记下结局就删它各次尝试的编译目录」，只有设了 `SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1` 才留，同文件第 302 行）；第二台那一份在第二台本地盘上、跑完删（同文件第 304 行）。我量了多机配置：`BUILD_ROOT_BY_HOST` 两项都不在 `NFS_SHARED_ROOTS` 任何一段下，本机服务的编译根在共享根下，仓与工作根都在共享根下（量的，读 `multi-host.env` 做前缀比较，值没抄）。所以经认可的路子编出的二进制，要么交完就删，要么派到第二台、本机读不到；写进 `--server-binary` 的绝对路径落在共享根外的，提交时拒（`task-server/task_server.py` 文件头 ②「提交时拒」①）。
- 结论：两台都读得到、又和此刻源码同一版的 `<编好的 prune-server>`，现在没有一处给得出。

**丁在服务那一侧（量的一半）。** `task-server/task status 7030 --json` 读出 `"compiles": true`，任务环境的键只有 LANG、PATH 与四个按台取值的库变量；命令是不带二进制的包装套 `crates-mutation-rows.py --selftest`；第 1 次派到第二台、此刻在跑（status 原样里有第二台的路径，没抄）。认法：`task-server/task_server.py:4713`「if lib.temporary_service_builds_server(task["command"], task.get("environment"), self.environment):」；派到本机给不给编译目录这一句只有源码，本机那一侧落地之后还没有一条量过。

四句：
1. 分辨臂：分。甲、乙、丙这一臂（带）在「两台都起得来」上过（丙）或根本交不出去（甲），在「判得对」上没有保证；丁这一臂在主工作区两台都给编译目录。
2. 看不看得到：服务只看 `--server-binary` 给没给（`task-server/task_server_lib.py:1377`），不看二进制新旧；子 agent 也没有哪一句叫它比新旧。
3. 判据分句：甲是 `.claude/agent-common.md:65`「命令原样」交不出去；丙是 `.claude/agent-common.md:40`「派到两台里任何一台都要跑得起、判得对」的「判得对」（旧二进制上判的，推的，看「旧二进制自证」）。
4. 改法中不中：第二轮交出的根（服务把不带二进制的包装认成会编的）在丁上起作用，对甲、乙、丙不起作用（它们给了二进制，`compiles` 照旧假）。

打翻条件：命令「旧二进制自证」退 0、开关轮全对上（那就只剩「将来会旧」的风险，丙降为推的）；命令「喂钩子」里 `literal-placeholder` 退 2、拒绝说明给出一个可填的值（甲就在钩子那一层被挡下、并给了出路）。

## A8　仓副本里起：照第 49 行拷的副本不带 prune-server 源码

第二轮 A3 往句子里加了「（仓副本里的同一份也算）」。副本怎么拷，共用约束第 49 行写的是 `.claude/agent-common.md:49`「副本只拷要编的」，点名 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，要门禁、重型、自证的再加 `.claude/gate.d`、`scripts/`、`.claude/scripts`、`.claude/singlefs-ai-sop`，没有 `prune-server/`。

- 丁（不带）：包装照自己所在的树编：仓根取法 `scripts/prune-server/prune_server_client.py:316`「REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))」，清单 `scripts/prune-server/prune_server_client_selftest.py:145`「manifest = os.path.join(REPOSITORY_ROOT, "prune-server", "Cargo.toml")」。副本里没有这份清单，`cargo build` 失败，包装退 2（`EXIT_USAGE`，`scripts/prune-server/prune_server_client.py:265`「EXIT_USAGE = 2」）。服务照样给编译目录（只看名字与 `--with-temporary-service`），两台都退 2。推的；命令「副本缺源码」。
- 乙（相对路径）：`scripts/prune-server/prune_server_client_selftest.py:1128`「return os.path.abspath(arguments[index + 1]), ""」照副本解；副本拷法排除 `target`，文件不在，`scripts/prune-server/prune_server_client_selftest.py:1187`「yield None, problem or f"{binary} 不可执行"」，退 2，两台一样。推的。
- 丙（主仓绝对路径）：起得来，旧二进制的问题同 A7。

用 `rsync -a --exclude target --exclude .git` 整份拷的副本（三方腿定义「写范围」一节的拷法）带着 `prune-server/` 源码，丁能编。两份拷法给出两个结局，句子没说副本要带 `prune-server/`。

四句：
1. 分辨臂：分，而且与 A7 方向相反：在副本里丁两台都退 2，丙起得来。没有哪一臂在主工作区与副本里都又起得来又用新二进制。
2. 看不看得到：服务看不到副本里有没有 `prune-server/`。
3. 判据分句：两台都退 2，不是「多机可用」那一句（两台一样）；是句子 A3 那半句「仓副本里的同一份也算」许出的命令跑不起来。
4. 改法中不中：根的修法（给编译目录）在这一格不起作用：有编译目录也没有源码。

打翻条件：命令「副本缺源码」退 0，或报的不是 `cargo build` 失败（那就是我读错了仓根取法）。

## A9　交出表漏项：own-server 两行裸起，本机退 2、第二台能跑（任务服务那一路）

登记表 own-server 两行（`.claude/hooks/selftest-temporary-service.tsv` 第 41、42 行：`prune_server_client_selftest.py`、`prune-server-load-simulation.py`）不归句子管（句子只管 required），子 agent 照「子 agent 不交任务」那一条的通例写裸的 `task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest`。

- 二进制的取法：`--selftest` 走 `scripts/prune-server/prune_server_client_selftest.py:1267`「return selftest(arguments[1:])」，取二进制是 `scripts/prune-server/prune_server_client_selftest.py:1141`「binary, problem = resolve_binary(arguments)」；压测那一份是 `scripts/prune-server/prune-server-load-simulation.py:191`「binary, problem = harness.resolve_binary(arguments)」。没给二进制、没有 `CARGO_TARGET_DIR` 时退 2，表头自己也写了：`.claude/hooks/selftest-temporary-service.tsv:9`「二进制都没给、也没有任务服务给的 CARGO_TARGET_DIR 时退 2、不现编」。
- 服务认会编只认包装：`task-server/task_server_lib.py:1377`「if index >= len(words) - 1 or os.path.basename(words[index]) != TEMPORARY_SERVICE_SCRIPT or words[index + 1] != TEMPORARY_SERVICE_FLAG:」，第二个词是 `--selftest` 不是 `--with-temporary-service`，`temporary_service_builds_server` 交假；命令位置上也没有 cargo，`compiles` 假。
- 所以：派到本机不给编译目录，退 2；派到第二台由 `multi-host-run.sh` 照例给编译目录，现编、能跑。与第二轮 A1 改前的形状相同，与 `.claude/agent-common.md:40`「派到两台里任何一台都要跑得起、判得对」相顶。
- 门禁自己是包着起的：`.claude/gate.d/code-tooling.sh:1016`「python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest」，第 1017 行压测那一份同样包着；表头第 10 行说 runner 表包着它「只为把二进制交下去」。门禁之外没有一处叫人照这样写。

第二轮交出表（`research/prompts/d137-common-temporary-service-r2-main-verification.md` 第 38–41 行）交给任务服务那一路的只有「A1 的根：服务把这一层认成会编、本机也给编译目录」，管的是包装；own-server 两行不在里面。第一轮交出表（同目录 r1 判决第 35–38 行）也没有。属任务服务那一路（让服务把这两份不带二进制的 `--selftest` 也认成会编的），或共用约束这一句（把 own-server 两行也写成包着起）。

四句：
1. 分辨臂：这一格不分「带 / 不带」那两臂，它打的是句子的射程（只管 required）。
2. 看不看得到：服务看得到命令，认法里没有这两份。
3. 判据分句：第 40 行「派到两台里任何一台都要跑得起」。
4. 改法中不中：第二轮的根只认包装，这一格照中。

打翻条件：命令「own-server 裸起」读出 `"compiles": true`，或派到本机退 0。

## 交出表还查了哪几处、没漏的

逐行对了第一轮判决「改法」表四行、第二轮判决「改法」表四行与两份判决「逐条判」里写了去向的每一行：

| 查的 | 结论 |
|---|---|
| 钩子那一路：拒绝说明的整条写法 | 不算漏，但第二轮交出的「表头随之写清」没点名拒绝说明与骨架，这两处随后改成了必带的占位（A6），与登记表第 6 行不一致。这一处归 A6，不另算一条漏项 |
| 钩子那一路：旧钩子库 `.claude/hooks/lib_heavy_tests.py:3320` 还留着「省去现编」与不带二进制的整条 | 没漏：第一轮 H3、H6 已交改钩子名的那一路；`.claude/settings.json` 里只注册 `project-code-guard.sh`（量的，grep），这段文字此刻不出现在拒绝说明里 |
| 骨架那一路：第 282 行附近「自证在哪跑」的占位 | 没漏：第二轮 A5 已登账本 |
| 骨架那一路：`TEMPORARY_SERVICE_WRAPPER` 与第 330 行括注 | 归 A6、A7 甲（这一轮新冒出来的形态，第二轮之后才改成必带） |
| 账本那一路：绑住表头写法与包装用法的检查 | 没漏：第二轮 A4 已登。补一句：这一轮量出的写法有七处、三种占位字样（A6 表），A4 的检查要绑的不止表头与包装两处 |
| 任务服务那一路：own-server 两行 | 漏，A9 |
| 共用约束本身：第 49 行副本拷法与 A3 那半句 | 漏，A8（第二轮采 A3 时没连拷法一起改） |
| prune-server 那一路：认可的编法编出的二进制两台读不到、结局即删，主仓那份没人刷新 | 漏，A7 丙（第二轮把「子 agent 给不出路径」交给了根，根落地后路径仍给不出，表头却改成必带） |

## 推的数：前两轮报的数里没量过的

| 号 | 原话（出处） | 为什么是推的 | 能量它的命令 |
|---|---|---|---|
| N1 | `research/prompts/d137-common-temporary-service-r2-main-verification.md:11`「（推的：这一轮复跑都没落到本机）」 | 判决自己标了推的。根落地之后这句不再是现状，但换上来的「本机给编译目录、现编、退 0」同样没有一条落到本机量过；7030 的 `compiles` 真是量的，它派到了第二台 | 「不带二进制落本机」 |
| N2 | `research/prompts/d137-common-temporary-service-r1-main-verification.md:17`「主仓里那份二进制在这一趟里被重编（相容，推的）」 | 判决自己标了推的。这一轮量到 12 份源码比它新：它是不是 6778 编的仍推不出，只知道那之后没再编 | 不用再跑；A7 丙引的是这一轮的 `find -newer` |
| N3 | `research/prompts/d137-common-temporary-service-r1-main-verification.md:21`「24 行 required 照写一行都不被服务退 3，6777」 | 6777 是喂三道钩子，服务的提交时拒不在钩子里，24 行没有一条经 6777 交到服务。经服务交过、判决里写明内层的只有 6779（`crates-mutation-rows.py`）与 6837（`crash-point-replay-shard-run.sh`）；6834–6836、6838 的内层判决没写 | 「二十四行收不收」 |
| N4 | `research/prompts/d137-common-temporary-service-r1-main-verification.md:23`「拼全都起得来（6777）」 | 6777 不起脚本。第二轮 R 改引真起的 6778–6780 与 6837，只撑得起「.py、.sh 各起过一份」；其余 22 行起不起得来没量过。行数本身 24、14、10 这一轮重数过（量的：`grep -cP '\trequired\t'` 打 24，按后缀数打 14 与 10，own-server 2） | 「二十四行收不收」让它们跑完 |
| N5 | `.claude/hooks/selftest-temporary-service.tsv:6`「两台都给编译目录，包装现编一遍 prune-server」 | 本机那一侧只有源码，没有落地后的读数 | 「不带二进制落本机」 |
| N6 | `research/prompts/d137-common-temporary-service-r2-main-verification.md:24`「服务照会编的给编译目录，两台就都能不带二进制起」 | 本机那一侧推的；副本里不成立（A8，推的） | 「不带二进制落本机」「副本缺源码」 |
| N7 | 背景材料第二节转述的「每次现编一遍」 | 两份判决都没给现编一次要多久、占多少盘 | 「不带二进制落本机」的用时与编译目录占盘 |

## 我提的改法（只在我读源码推过、被攻过零轮）

| 改法 | A6 | A7 甲 | A7 丙 | A8 | A9 |
|---|---|---|---|---|---|
| 改 A6：登记表第 5 行、拒绝说明（`lib_project_code_runs.py` 的 `temporary_service_command` 与 1788 行括注）、骨架常量（295 行）与括注（330 行）、自证登记表第 10 行与 23 行理由、客户端文件头第 112 行，整条写法都换成不带 `--server-binary` 的一形（同 `code-tooling.sh:625`）；`--server-binary` 只在整条之外留一句「自己编了、两台读得到的二进制可以加」；句子不动 | 修（推的） | 修（推的：整条里不再有占位） | 修（推的：没人被叫去填旧路径） | 不修 | 不修 |
| 改 A8：第 49 行副本拷法加「要在副本里起登记表里的自证的，连 `prune-server/` 一起拷（不带 `target`）」 | — | — | — | 修丁（推的） | — |
| 改 A9 甲：服务把 own-server 两份不带二进制的单独 `--selftest` 也认成会编的（任务服务那一路） | — | — | — | — | 修（推的） |
| 改 A9 乙：句子射程扩到 own-server 两行，写成门禁那样包着起（`code-tooling.sh:1016`） | — | — | — | — | 修（推的，靠写报告的人照做） |

每一格都是「推的，待主 agent 跑」：没实现、没跑。

## 要主 agent 跑的命令

都在仓根起（「副本缺源码」除外）；都不是重型；交完用 run_in_background 起 `task-server/task wait <号>`，再读 `task-server/task status <号>`（看「第 1 次：本机 / 第二台」与用时）、`task-server/task status <号> --json`（看 `"compiles"`）、`task-server/task log <号>`。

1. 「喂钩子」（起 7 次 `project-code-guard.sh`，几秒到几分钟）：
   ```
   task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d137-cts-r3-attack -- python3 -I /home/fy5090/.local/share/singlefs/work/d137-cts-r3-attack/feed_project_code_guard.py /home/fy5090/code/singlefs
   ```
   看：末行 `FEED_SUMMARY shapes=7 refused=<n>`；各行 `FEED shape=<名> rc=`。我推的是：`binary-omitted`、`binary-main-absolute`、`binary-main-relative`、`own-server-bare-submitted` 退 0；`required-bare-direct` 退 2，拒绝说明里带 `<编好的 prune-server>`；`literal-placeholder` 与 `required-bare-submitted` 我推不出，照读。`literal-placeholder` 退 0 就是钩子放行、落到 shell 里退 1（A7 甲）。
2. 「不带二进制落本机」（同一条交两次，放在同一回里交；两条都派到第二台的，再交一次，直到有一条落到本机；落到本机那一条读完，其余的可以 `cancel`）：
   ```
   task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- true
   ```
   看：落本机那一条 `"compiles": true`、`TASK_SERVER_RESULT` 的 `exit=0`、日志末行 `PRUNE_SERVER_TEMPORARY_SERVICE exit=0 server_alive_at_end=1`；用时与编译目录占盘就是 N7 的数。本机那一条退 2、日志有「不现编」，N1、N5、N6 的本机那一侧就翻了。
3. 「副本缺源码」（`--cwd` 是草稿目录里的交接副本，只拷了 `scripts/prune-server/` 下三份 `.py` 与 README，与主仓 sha256 相同）：
   ```
   task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/d137-cts-r3-attack/copy-without-prune-server -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- true
   ```
   看：我推的是 `exit=2`，日志里「✗ 临时 prune-server 起不来：cargo build 退 …」，后面是清单不在。退 0 或报别的错，A8 翻。
4. 「旧二进制自证」与「新二进制对照」（两条各起一遍客户端自证与开关轮）：
   ```
   task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest --server-binary /home/fy5090/code/singlefs/prune-server/target/release/prune-server
   task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest
   ```
   看：两条各自的 `PRUNE_SERVER_CLIENT_SELFTEST cells=… passed=… failed=… failed_cells=…` 与 `PRUNE_SERVER_CLIENT_BREAK_SWEEP switches=… matched=…`。旧的那条有格红、新的全绿，A7 丙的「判得对」坐实；两条一样，A7 丙降为「将来会旧」的推的。
5. 「own-server 裸起」：
   ```
   task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --selftest
   ```
   看：`"compiles"`（推的是 false）；落本机的 `exit=2`、日志「✗ 自证起不了服务：没给 --server-binary 与 PRUNE_SERVER_BINARY」；落第二台的能编能跑。要两台都看到就照命令 2 那样多交。`"compiles": true` 或本机退 0，A9 翻。
6. 「二十四行收不收」（量大：24 条自证，各现编一遍 prune-server）：`/home/fy5090/.local/share/singlefs/work/d137-cts-r3-attack/c6-commands.txt` 里 24 行，每行一条，形如
   ```
   task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/crash/crash-amplification-item.sh --selftest
   ```
   （`.py` 用 `python3`，别的用 `bash`，照登记表 required 行的次序）。看：每条打出任务号是收、`task submit` 退 3 是拒（N3）；要量 N4 就让它们跑完，逐条看 `exit=` 与日志末行 `PRUNE_SERVER_TEMPORARY_SERVICE exit=`。只量 N3 的，收下后可 `cancel`。

## 没打中的形状

- `--server-binary=<路径>` 等号写法：服务认不出这一层（`task-server/task_server_lib.py:1385`「if position >= len(words) - 1 or words[position] != "--":」），包装自己当认不出的参数退 2（`scripts/prune-server/prune_server_client_selftest.py:1223` 起的 `unknown` 判）。七处写法没有一处写成等号，读者照哪一处都写不出它，不算打中。
- `--server-binary` 写在 `--with-temporary-service` 前面：包装第一个参数不认、退 2；同样没有来源会这样写。
- `PRUNE_SERVER_BINARY=` 写在包装前：服务照赋值认成给了二进制（`temporary_service_builds_server` 看包装前的词），值的来源问题同 A7 丙，不另算。
- 解释器取法：24 行按后缀拼，14 行 `bash`、10 行 `python3`，与表里的后缀一一对得上（量的，按后缀数）；没打中。
- 包装里的 24 条原命令会不会被服务的 ④ 拒：「要卡」那一判对单独一个 `--selftest` 不算（`task-server/task_server.py:1627`、`:1640` 两处都排除 `["--selftest"]`）；「会编」那一判只看命令位置上的 cargo，24 条原命令命令位置上没有 cargo（推的，读源码；命令 6 量）。
- 第二轮攻过的 `timeout` 套在包装外、`--` 后面那条带包装（A2）、仓副本键（A3）的键认法本身，没再攻。

取样范围：登记表 42 行（16 行表头、24 行 required、2 行 own-server）；句子所在第 64 行与第 40、49、65 行；写整条写法的七处（表头、拒绝说明、骨架、自证登记表、客户端文件头、门禁 code-tooling、旧钩子库）；服务的 `compiles`、`temporary_service_split`、`temporary_service_builds_server`、提交时拒 ①–④；包装的 `resolve_binary`、`build_server_binary`、`with_temporary_service`、`temporary_service`、`main`；`prune-server/control` 的编法；多机配置里共享根与编译根的包含关系；主仓二进制与源码的新旧；两份判决的「逐条判」与「改法」表全部行。

## 这条腿自己的限度

- 一个项目代码都没起：钩子、服务、包装的结局除 A7 甲（shell 层，现跑）与 7030 的 `compiles`（现读 status）外都是读源码推的，命令在上一节。
- 7030 不是我交的，我只读了它的 `status` 与 `status --json`，没读它的日志。status 原样里有第二台的路径，没抄进报告。
- A8 用的交接副本只拷了 `scripts/prune-server/`，不是照第 49 行整份拷；包装的仓根只由自己所在的位置定，所以结局应当相同，但这一点本身是推的。
- 「判得对」那一半（A7 丙）要靠命令 4 才知道旧二进制此刻会不会让哪一格判错；在它跑出来之前，只能说「没有保证」。
- 这是一份样本；按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」，上一节「没打中」的几条不能单独当「没问题」用。

## 没做什么

- 不判 R 格（辩方腿的）；没读禁读清单里的辩方报告。
- 没往统一任务服务交任务，没起 `run_in_background`；判决要引的数都没复跑，命令在「要主 agent 跑的命令」一节。
- 没写模型目录（`research/prompts/d137-common-temporary-service-r3-opus-model/` 没建），所以没有 `tools/rerun.sh`；草稿脚本 `/home/fy5090/.local/share/singlefs/work/d137-cts-r3-attack/feed_project_code_guard.py` 只做喂钩子一件事，`python3 -m py_compile` 过。
- 没改任何仓内文件，改法都在「我提的改法」一节，交主 agent。
- 草稿目录里留着：交接副本 `/home/fy5090/.local/share/singlefs/work/d137-cts-r3-attack/copy-without-prune-server`（212K，只有 `scripts/prune-server/` 下三份 `.py` 与 README，不是带 `crates/` 的仓副本；`.deliverables` 写成「交接」，命令 3 在里面跑，主 agent 用完删）、`feed_project_code_guard.py`、`c6-commands.txt`、`status-7030.json`、`status-all.txt`、`cite1.txt`。没建编译目录。
