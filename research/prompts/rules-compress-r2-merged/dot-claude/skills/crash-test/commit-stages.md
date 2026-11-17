# 提交时的六道重阶段：主 agent 照这一份跑

子 agent 不交这几道（`.claude/agent-common.md`「子 agent 不交任务」）。主 agent 跑的是 `.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 3 步与 `.claude/rules/verification.md`「一个门禁：两种跑法」里提交时跑的六道门禁（文件是 `.claude/gate.d/<名>.sh`）：`crash-point-replay`、qemu-device-streams、lkmm、crates-mutation-replay、formal-verification、broad-mutation（照第 2b 步），加崩溃放量全部分项。
开工先读：`.claude/skills/crash-test/SKILL.md`「判读纪律」；`.claude/rules/verification.md`「崩溃一致性只能靠崩溃点重放验证」「文件系统特有的反推缺口」；`.claude/rules/test-discipline.md`「阴性结果要能和「代码没跑到」分开」。

## 跑之前定下来

- 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
- 这一次是提交时跑还是用户要求时跑（只在这两种时候跑；交给服务的命令不带 `SINGLEFS_HEAVY_TESTS` 前缀）。
- 崩溃放量全量：每次提交都跑全部分项（不带 `--item`），与六道同列提交时的门禁；崩溃枚举用例与崩溃放量分项合起来判过的崩溃状态每次提交不少于 100 亿，不满就补跑到满；在主工作区跑，写法在第 2 步；开发时只跑几个分项的加 `--item <分项>`。
- 草稿目录。

## 做什么

1. 另有别的 `gate.sh` 正在跑 `crash-point-replay`、或有别的 `.claude/gate.d/crash-point-replay.sh` 在跑时，不起门禁 `crash-point-replay`，等它结束。qemu-device-streams、lkmm、crates-mutation-replay、formal-verification 在主工作区跑、判工作区那一份，每道直接起，不先问要不要跑，成功句报「现判 N、复用 M」（有拒算的报拒算 R）。开跑前核工作区等于暂存区：
   - 输入路径：qemu-device-streams、crates-mutation-replay、formal-verification 取各自阶段脚本文件头列的输入路径（不取 `.claude/gate.d/stage-inputs.tsv` 第二列：那里登记的是函数单元闭包的入口），lkmm 取 `litmus crates .claude/scripts/lkmm.sh .claude/scripts/fetch-deps.sh`，四道都另加自己的阶段脚本 `.claude/gate.d/<文件>`。跑 `git diff --quiet -- <这些路径>`，要退 0。
   - 未跟踪文件：跑 `git ls-files --others --exclude-standard -- <路径>`，要没有输出；qemu-device-streams、crates-mutation-replay 查 `crates`（qemu-device-streams 按名字读的 `research/results/` 产物这里不查，由整轮门禁里 research-build-and-replay 的 experiment-replay 格兜）；formal-verification 查 `crates` 与 `research/results`（别的会话在 `research/results/` 里有没暂存的改动或没跟踪的产物，这一道不跑，停下，照同一条找那个会话协商）；lkmm 查 `crates litmus .lkmm-static-only`。别处的未跟踪文件不挡；lkmm 读的 `.claude/singlefs-ai-sop/scripts/lib.sh` 不在这一步里。
   - 两样有一样不过就不跑那一道，停下，记「工作区与暂存区不同，判的不是要提交的那一批」与两条命令的原样输出，照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 9 条找改那几条路径的会话协商，等它们暂存、提交或撤掉再跑。
   - broad-mutation 照第 2b 步。
   1b. 六道与崩溃放量不指定范围的那一趟一律写成 `task-server/task submit --full-gate -- <命令>`：只有它们与 `scripts/gate/gate-staged.sh` 认这个标识；带了它不受四小时硬上限（四小时上限只管开发时指定范围的跑法）；不指定范围而没带它的，服务当场拒，整包确认标识 `--whole-package-confirmed` 顶替不了它。这一条弹窗要用户确认、对话不自己决定，提交时共 7 次（六道与崩溃放量各 1 次；`.claude/skills/gate/staged-gate.md` 的 `gate-staged.sh` 另算 1 次，一次提交合起来 8 次）。崩溃放量点名分项（带 `--item`）的经 `task-server/task submit -- <命令>` 交。交了的每一条都再用 run_in_background 起 `task-server/task wait <号>`。命令写法（不套包装、不写上限线程与挑台参数）、撞内存上限到两台最大时（`memory-cap-at-maximum`）怎么办，照 `.claude/agent-common.md`「子 agent 不交任务」；分片与按需领卡由服务照各自文件头做（`task-server/task_server_lib.py` 文件头「分片」与「GPU」）。结局读 `task wait` 的退出码与末行 `TASK_SERVER_RESULT`（分片的那一件有一步没成，末行另带 `shard=<片号>/<片数>`，计划、合并那一步带 `step=plan`、`step=merge`），跑完 `task-server/task clean <号>`。
2. 只在提交时（或用户要求时）跑这六道，与 `.claude/main-agent.md`「暂存之后、提交之前跑门禁」那一行点名的相同，不从阶段归属表取；不跑别的门禁、`gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/project-code-guard.sh` 拒）。
   - 次序：qemu-device-streams、lkmm、formal-verification 与 `crash-point-replay` 同时交（各自后台 `task wait`，照第 1b 步），crates-mutation-replay 等 `crash-point-replay` 跑完再起，broad-mutation 照第 2b 步。
   - 命令都不带 `SINGLEFS_HEAVY_TESTS` 前缀（「跑之前定下来」一节）、都带四段范围声明（写法照 `.claude/hooks/run-scope-guard.sh` 文件头，为什么一段填这一次的理由）：`SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=门禁 <文件> 的全部格; 范围=整轮; 为什么=<为什么这么跑、这个范围为什么合适>' task-server/task submit --full-gate -- bash .claude/gate.d/<文件>`；每道记耗时与退出码。
   - `crash-point-replay` 只有这一种跑法：不带 `--check` 与 `--item` 就是九格全跑，没有档（门禁文件头「只有一条路」）。服务把它拆成计划、各片、合并，各片与合并各判什么、要卡的三处怎么按需领卡，照 `.claude/rules/verification-reference.md`「门禁 crash-point-replay 怎么跑」与门禁文件头「分片」「显卡与清场」；领卡被拒的那一格判红。
   - 崩溃放量全量在主工作区另起：`SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=崩溃放量的全部分项（不带 --item）; 范围=整轮; 为什么=<为什么这么跑、这个范围为什么合适>' task-server/task submit --full-gate -- bash scripts/crash/crash-amplification.sh`（不带 `--crash-expand-up-to`；开发时点名分项的加 `--item`、去掉 `--full-gate`、声明改成 `种类=items; 跑什么=崩溃放量点名的分项 <分项>; 范围=<N> 项`）。开跑前照第 1 步那两条命令核 `crates scripts/crash` 的工作区等于暂存区。服务照入口文件头按项分片（一项一片）、录完一批才领卡（入口文件头「按需领卡」）；一趟以小时计，后台 `task wait` 等。退 78 判「环境」（运行条件不满足、某一项的驱动退 78、按需领卡被拒，照它的「→」），退 1 判红，退 0 才判绿。要复原清场停下的服务，手动跑多机配置 `QUIESCE_START_COMMAND` 那一条私有复原脚本，再跑 `QUIESCE_STARTED_CHECK_COMMAND` 回读到它退 0。原样抄每一行 `CRASH_AMPLIFICATION_ITEM` 与末行总计、成功句；在当前里程碑记录里记一行「上一次全量：<日期> 崩溃放量 <结论>」。
   - 每道的退出码取 `task wait` 的退出码与末行 `TASK_SERVER_RESULT`，整份输出 `task-server/task log <号> --all > <草稿目录>/<阶段>.log`，结束后读输出。别的会话同时在跑 cargo 时编译卡在「Blocking waiting for file lock」的，照实记等了多久，不算这个阶段的耗时。
   2b. broad-mutation 等 crates-mutation-replay 跑完再起，在主工作区按次序做两步。判定在 prune-server 的 broad-mutation 族里按每条变异的单元键逐条沿用，不按提交、不按 HEAD 选条，不包暂存树、不换版本；每条变异 CPU 那一趟判定在 `verdicts/` 表，GPU 那一趟在 `gpu-verdicts/` 表（驱动文件头「判定库」「GPU 那一趟」）；库目录 `<数据根>/broad-mutation-units/` 只放这一趟的运行目录、日志与锁。
   ① 门禁起的驱动判的是主工作区（`--source <被判的根>`），要它等于暂存区：取 `.claude/gate.d/stage-inputs.tsv` 里 crates-mutation-replay 那一行 `copy-range=` 的路径（驱动的导出范围），加 `scripts/mutation/broad-mutation.py scripts/gate/lib_function_units.py .claude/gate.d/broad-mutation.sh`，跑 `git diff --quiet -- <这些路径>`（要退 0）与 `git ls-files --others --exclude-standard -- crates .cargo`（要没有输出）；有一样不过就不起 ②，停下，记「工作区与暂存区不同，判的不是要提交的那一批」与两条命令的原样输出，照第 1 步那一句找改那几条路径的会话协商。
   ② `SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=门禁 broad-mutation.sh 的全部格; 范围=整轮; 为什么=<为什么这么跑、这个范围为什么合适>' task-server/task submit --full-gate -- bash .claude/gate.d/broad-mutation.sh`（弹窗确认算在第 1b 步的 7 次里），整份输出 `task-server/task log <号> --all` 写进 `<草稿目录>/broad-mutation.log`。服务把它拆成计划、两片、合并，每步起这一道一次、这一道照同一步起驱动 `scripts/mutation/broad-mutation.py --source <被判的根>`（门禁文件头「判什么」，驱动文件头「分片」「按需领卡」）；合并那一步驱动核账：CPU 那一趟还有要现判的条就退 1、逐条点名，GPU 那一趟还没结论的条逐条打 `BROAD_MUTATION_GPU_NOT_RUN reason=not-judged-in-shards`；退 0 再读驱动 `--summary` 判 gate-verdict 格（只读判定库）。一趟以小时计（新库第一趟每条都现判），后台 `task wait` 等。
   ② 退非 0 时原样抄这一道与驱动的「✗」「→」与退出码（驱动退出码的含义在驱动文件头「退出码」一段），不换参数重跑：
   - 分片里没成的那一步照第 1b 步抄末行的 `shard=` 或 `step=`。
   - 退 1 里有一种是 GPU 那一趟的复用哨兵判红（「✗」那一行带「广谱变异 GPU 趟」，项名 `gpu:<序号>`），原样抄「✗」「→」。
   - 驱动退 4（库目录的锁被别的一趟占着）：抄 `ps -o pid,args -u "$(id -u)"` 里那一趟驱动的进程行，不等、不抢。
   - 退 2 是用法错这一类，照它的「→」改命令；退 5、6 照抄；退 75 是判定库服务不可用（整道记「prune-server 不可用」，不是红、也不算通过）。
   - 被停（退 143，在途那一条不落行）原样记下，重交同一条 ②（沿用得了的条不重跑）。
   - gate-verdict 格的判定原样抄，`BROAD_MUTATION_GATE` 那一行另抄 `gpu_needed`、`gpu_reused`、`gpu_not_run`、`gpu_without_conclusion` 四个数。
   - 它列的 NO_CONCLUSION 里理由写 `gpu-not-run` 的是 GPU 那一趟本次未跑的条（没卡、按需领卡被拒、各片没判到）：重交同一条 ② 补上。理由写 `gpu-driver-error` 的是 GPU 趟驱动错（不是这条变异的结局），来历三种：编译步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类）、测试步退 0 而一个测试都没跑到、编译步或测试步退了认不出的退出码；查判定库 `gpu-verdicts/` 表里那条最后一行的备注列与这一趟的日志，修好环境再重交同一条 ②（驱动错的条下一趟重试）。
   - 它列的 UNLISTED（有单元没记清单，开跑之后工作区又改了）照 ① 重做再重交 ②；红在驱动 `--summary` 退 1、报判定库里有认不出的结局的，照驱动打的「→」办。
   - ② 开跑之后、收尾之前工作区或暂存区又改了的，① 重做。
   2d. 门禁 `crash-point-replay` 跑完，它的 product-gpu-real-card-tests 格报了「本次未跑（这不是通过）：产品 GPU 包对拍目标 <目标>（<源文件>）声明 N 张、拿到 M 张，提交时单交」的（这道门禁一回只领一张卡，要两张、三张卡的对拍目标在格里不起），逐个点名单交：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=产品 GPU 包 singlefs-background-gpu 的对拍目标 <目标>; 范围=1 个测试目标; 为什么=门禁 product-gpu-real-card-tests 格里拿到的卡少于这个目标文件头声明的张数，提交时单交' task-server/task submit -- cargo test --release -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --test <目标>`，照第 1b 步后台 `task wait` 等。命令里不写台、不写卡：服务照测试文件头的「// task-server-gpu-cards: <张数>」挑凑得出这么多张空卡的一台、整组分卡清场。每个目标的结局绿（`task wait` 退 0、末行 `TASK_SERVER_RESULT` 的 `exit=0`，日志里没有 `NOT_RUN_NO_GPU_ADAPTER`）才算那一格过；红照第 3 步原样抄「✗」与 panic 段；日志里有 `NOT_RUN_NO_GPU_ADAPTER` 的记「本次未跑」；服务判 `never-fits`（哪一台都凑不出这么多张空卡）或别的服务结局的照第 3 步记「没跑成」、原样抄末行与出路。那一格在产出表与监控行里按「本次未跑」记，单交的目标全绿之后改记绿，单交的命令、任务号与结局抄在那一格下面。
3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。formal-verification 一格里的「✗」只数那一格自己打的：从「── <格名>：」那一行到下一个「──」之间的「✗」行；不数「── 汇总：」一段里格共用库 `.claude/gate.d/lib/stage-cells.sh` 打的「✗ formal-verification.sh：N 格里 M 格红（…）」与它下面的「→」。服务判出的结局（`task wait` 末行 `type=` 不是 `succeeded`、`command-failed`、`not-run` 的）那一道这一趟没判出结果：表里记「没跑成」，不记成红在证明、表或实现上，原样抄末行与服务给的出路，照它改命令重新交，不绕开服务重跑。
4. 计数行原样抄：崩溃状态数、恢复结果、与 E142（新池新建文件的干跑）产物或闭式比对的那几行；门禁 `crash-point-replay` 合并那一步逐条登记的崩溃枚举用例各抄一行（「✓ <用例> 判绿」连同它记下的行，全部块复用的那条另有一行「· <用例> 复用」，「✗」连同紧跟的「→」；登记行第三列带 `separate-run=` 的用例没被点名时不跑、报本次未跑，照登记行写的理由另行点名交），末尾「✓ 崩溃状态块：现判 N、复用 M、拒算 R、本次未跑 0」与「✓ 登记的 N 条崩溃枚举用例这一趟都判绿」两行原样抄；各片只看 `cargo test` 的退出码，用例判在合并那一步的核账（门禁文件头「分片」merge：每条用例起一趟 `0/1` 的核账，核账那一趟现判了块的判红）。输出里读不到判定行的阶段记「作废」，不记通过。
5. 缺 herd7、缺 KVM、虚机起不来这一类判「环境」，以阶段自己的报错为准，另贴 `command -v herd7`、`ls -l /dev/kvm`、`command -v qemu-system-x86_64` 的原样输出作旁证；这三条命令不单独下判断。
6. 每个阶段开跑与结束时各记一次指纹，开跑那次与起阶段写在同一条 Bash 命令里：`git rev-parse HEAD`、`git diff HEAD -- crates litmus Cargo.toml Cargo.lock scripts | sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus | xargs -0 -r sha256sum | sha256sum`（未跟踪文件按内容）；跑 formal-verification 的那一趟，两条命令的路径再加 `.claude/gate.d/verus-pins.tsv .claude/gate.d/e165-pinned-red-examples.tsv .claude/gate.d/lib/stage-cells.sh scripts/mutation/tables/e165_replay_bounded_enumeration_custom.tsv`（`formal-verification.sh` 文件头列的输入里这里还没有的几条）；跑 broad-mutation 的那一趟两条命令的路径再加 `.cargo`。前后或阶段之间不同，就写明「这几个绿对应的不是同一份源码」、列出哪几个阶段之间变了。

## 监控行的三栏怎么数

监控行「这一趟 ✓ N ✗ M 本次未跑 K」的 N、M、K 数第 3、4 步抄下来的判定行，不另写 grep：

- 单位是一条判定：阶段汇总里的一格（「<格名>：绿 / 红（退出 N）/ 本次未跑」那一行）、门禁 `crash-point-replay` 合并那一步逐条用例行里的一条用例、阶段自己逐项打的一项（一档、一份 litmus、一条变异行）；阶段没有逐项行的，整道阶段数一条，按它的退出码进栏。
- ✓：判定是绿的。`crash-point-replay` 逐条用例行里每条用例的「✓ <用例> 判绿」数一条（全部块复用的那条另打的「· <用例> 复用」不另数）。
- ✗：判定是红的。一条判定的「✗」行连同紧跟的「→」数一条；格共用库打的汇总「✗ <阶段>：N 格里 M 格红」不数。
- 本次未跑：退出码 77 的格、项与阶段，报「本次未跑」的用例，加上「作废」（第 4 步）与「没跑成」（第 3 步）；只进这一栏，不进 ✓，也不进 ✗。
- ✗ 逐条点名；K 后面逐条点名是哪一条、属于 77、作废、没跑成里的哪一种。
- 三栏相加等于这一趟数到的判定条数；对不上就回头重数，对上之前不抄这一行。

## 写范围

- 草稿目录。阶段自己用的临时目录、编译产物（crates-mutation-replay 的 `GATE_MUTATION_TARGET_DIR` 等）与各道经客户端写进 prune-server 那一份库的判定是阶段本身的行为；广谱变异驱动的库目录（`--home`，缺省是数据根下的 `broad-mutation-units`，数据根照 `.claude/hooks/lib_work_root.py` 的 `shared_data_path` 取）是驱动本身的行为；这几道不改仓里任何文件。

## 产出

- 一张表：阶段 / 退出码（0、非 0、77）/ 耗时 / 原样的 ✓，或 ✗ 与 → / 计数行 / 日志路径；末尾「判读边界」两条。每个阶段的整份输出都写进 `<草稿目录>/<阶段>.log`（前台跑也写），crates-mutation-replay 那一份是变异分诊员要的输入。

## 判读边界

- 红了不在这里修：照 `.claude/skills/gate/staged-gate.md` 第 3 步判归属，修照 `.claude/main-agent.md`「测试或门禁结果与预期不符」那一行派 `investigator`。
- 全绿只说明这几道的证据要求满足；崩溃点重放覆盖到哪几条流、跑了哪几条崩溃枚举用例、哪些没进来，看 `.claude/gate.d/crash-point-replay.sh` 文件头与 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行，不由你外推。
