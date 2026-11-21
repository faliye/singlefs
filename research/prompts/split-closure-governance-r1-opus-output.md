# split-closure-governance-r1 攻方腿报告（治理文件轮第一轮）

日期：2026-11-20。被判：开工快照 `research/prompts/split-closure-governance-r1-snapshot/judged-sha256.txt` 点名的 43 份。攻击面：判据格 2「误伤与绕过」、判据格 5「排除表与用户定案」，加正文「## 已经量的」逐句能不能从调度记录 `records/2026-11-16-三方豁免取消与存量补轮.md` 第 711–782 行与它点到的日志核到。

## 复跑命令与模型文件

复跑（整条交任务服务，在仓根起；命令原样在文末「要主 agent 跑的命令」）：

    task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/split-closure-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-attack/frozen /home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-attack-probe-run /home/fy5090/code/singlefs/research/prompts/split-closure-governance-r1-opus-model

模型目录 `research/prompts/split-closure-governance-r1-opus-model/` 每个文件的 sha256（量的，`sha256sum` 现算）：

    f32df02be3c5526133f7d35a95e47794ca2e5e5ab13d3b50b1f320fdd5607e87  probe-stage-jobs.sh
    e82591b9444e9bcba8d65911442743d114695bbdd9625070acffa8b67cca170f  probe-family-growth.py
    5c907928a46e8d3735e8815e9327251362d64121ffe31aa0ca19f02428be6e2a  probe-closure-admission.sh
    6fc25289cedf7ea9cb76bfa913fc799890696844f05deb14e47ed941c0d61683  tools/rerun.sh
    e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh

冻结副本 `/home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-attack/frozen/` 是开工时从仓里拷的四份（不是仓副本，没有 `.git`、`crates`），sha256 与开工快照对得上的那份 `lib_lag_split_family.py` 相同：

    0173db21c3e1c07abe5cc011875337be82f7ec68ddac819564af386d0ac27822  .claude/gate.d/lib/stage-cells.sh
    473deb346842fc92886cfaebfbf6908fdfc41a593cb7a5aa43c8222fe4c84144  scripts/hosts/heavy-threads.sh
    872aac471a40685b66f247b75885d3e762696a4689d0e61cc8423672903cf60e  scripts/gate/lib_lag_split_family.py
    66aeed9f19a8ba5b56f60936906d69a2d9aeab50b360cb46de2418fd8c53137c  .claude/heavy-thread-default

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只改 `rerun_steps`（`diff` 只多出第 84–92 行）。三个探针都只打读数：
- `probe-stage-jobs.sh`：九格登记文件与 `SINGLEFS_THREAD_CAP` 组合下，`stage_cells_default_jobs` 与 `heavy_thread_default` 各交什么（每条样本 `setsid --wait … &` 自成一组、`wait` 收、跑完 `kill -0 -- -<组>` 核没有留下进程）。
- `probe-family-growth.py`：`keyed_family_growth` 与 `family_growth` 对同五批排除行各判几行涨。
- `probe-closure-admission.sh`：对仓根此刻的源码起一次 `lib_function_units.py closure . .claude/gate.d/code-tooling.sh --explain`，数 admission 一族的单元、点名四个单元在不在、经哪条 via 边进来；开头另 `sha256sum -c` 开工快照清单。

估时（推的）：前两个探针秒级；闭包一趟没量过，按记录里同一条 `closure` 命令能在服务上单跑出结局推几分钟到十几分钟，合计不到 40 分钟，不缩取样。内存由服务定、撞上限重排时外层先删运行目录。这一腿没跑过任何一步，下面标「量的」的数都来自记录、仓里文件与任务服务日志目录里已有的日志，模型读数全部「推的，待主 agent 跑」。

## 开工快照核对

开工时 `sha256sum -c` 43 份全 OK。写报告途中再核，两份对不上（量的）：

    .claude/gate.d/repository-copy-closure-exempt.tsv: FAILED
    .claude/memory-bounded-parallelism.tsv: FAILED

两份的修改时刻都晚于开工快照清单（`stat` 现查，同一日）。被判的是快照那一版：`repository-copy-closure-exempt.tsv` 快照那一版与工具员草稿 `~/.local/share/singlefs/work/exempt-tables-baseline-once/newtree/.claude/gate.d/repository-copy-closure-exempt.tsv` 逐字节相同（sha256 都是 `d71eafd5…8ed5d`，量的），下文引它第 44 行用那一份；此刻的仓里那一行已经不在。`memory-bounded-parallelism.tsv` 不在这条腿的格里，没看改了什么。

## 各格判定一览

| 格 | 判 | 一句话 | 依据（详见各节） |
|---|---|---|---|
| 2 `admission_run` 那一行算不算第三种排除 | 成立（打中，字面） | 登记理由是「与 NOT_JUDGING_FILES 同一个角色」，admission.py 不在文件级排除的两份里，也不是三格自证的边级排除，也不在算键库文件头列的不拒算类别里；同一种起 admission.py 的边别处登在会进键的表 | `.claude/rules/verification.md:41`、`scripts/gate/lib_function_units_registry.py:65`、`:409`、`:411` |
| 2 `admission_run` 让哪些单元出键、改 admission.py 会不会错复用 | 不成立（没打中） | 开工前一版的 code-tooling.sh 闭包输出里 admission 一族照在，连只经 `admission_run` 起的 library-files 那一支都在：这一行在效果上不让任何单元出键 | 日志里那一份闭包输出（量的，开工前一版）；复跑待主 agent |
| 2 `admission_run` 的 `command` 参数 | 推不出（潜在绕过） | 登记按整句 `command or [...]` 认，调用方往 `command=` 交什么都不拒算、不连；今天只有样本造卡住一处交 | `code-tooling-shared-libraries.sh:735`、`:894` |
| 2 `TripLauncher.run` 那一行 | 不成立 | 落在文件头「表行与样本给的命令」那一类，被起的那份由每条判定的请求进键 | `code-tooling-break-switch-replay-cells.sh:505`、`:507` |
| 2 `TAIL_READ_SCOPES` 新加那一行 | 不成立 | 读的是自证临时目录里钩子刚写的检出记录，登空元组合「单元键的组成与排除」末两条 | `lib_function_units_registry.py:514`、`verification-reference.md:193` |
| 5 五行 `STAGE_CELLS_JOBS` | 成立（打中） | 是真跑法（门禁格并行的缺省路数），不经登记位点名的取法：不认 `SINGLEFS_THREAD_CAP`、登记写坏或不在时悄悄退成 1；主 agent 对 doc-text 同一写法退回过 | `.claude/heavy-thread-default:2`、`:5`、`stage-cells.sh:307`、记录 `:756`、`:764` |
| 5 快照那一版 repository-copy-closure 第 44 行 `SHARED_INPUTS` | 成立（打中） | 开工快照那棵树里锚串一处都不命中（被锚的写法开工前已改），门禁自己判这张表「有 1 处不对」 | `code-tooling-unit-verdict-and-entry-cells.sh:747`、开工后一趟门禁日志 |
| 5 crates 四行 | 不成立（不是真该改的），但不在用户点名的三类里 | 主 agent 另收的「合规写法、判法没认出」一类 | 记录 `:759` |
| 5 hardcoded-quota 四行、repository-copy-closure 其余八行 | 不成立 | 逐行在提交基准里找到同一写法，都是自证假值、开关坏命令文字、单份拷贝或只读摘要范围 | 本报告「判据格 5」逐行表 |
| 5 doc-text 那一处还在不在排除表 | 成立（不在） | 三张表里 `doc-text` 零命中 | `grep` 现查 |
| 5 旁支：「之后照旧只缩」拆分之后守不守得住 | 成立（打中，推的待复跑） | `keyed_family_growth` 按（族入口, 锚串）集合比、不比行数：同族新文件抄已有锚串登几行都不算涨 | `lib_lag_split_family.py:178`、`heavy-thread-exempt:18` |
| 已经量的：「号见调度记录」 | 不成立 | 第 711–782 行没有任务号也没有日志路径（任务号本就不许写），日志从记录点不到 | `grep` 现查 |
| 已经量的：其余各句 | 多数核得到；四句核不全或与快照对不上 | 见「已经量的」逐句表 | 记录各行与日志行 |

## 判据格 2：误伤与绕过

### `admission_run` 那一行

登记行：「"admission_run", "command or ['python3', admission, *arguments]"」（`scripts/gate/lib_function_units_registry.py:63`），理由「管这一条复用不复用、不管被判的脚本判对判错，与 NOT_JUDGING_FILES 同一个角色，由 admission.py 自己的 --selftest 判；」（`scripts/gate/lib_function_units_registry.py:65`）。主 agent 收它的那一句：「主 agent 收这两行、交这一批的三方攻」（`records/2026-11-16-三方豁免取消与存量补轮.md:773`）。

**第三种排除：成立（字面）。** 规则只认两种：「两种排除不在这句全称里：算键的库自己两份」（`.claude/rules/verification.md:41`），另一种是「一条只打提示、不改判定结局的子进程边按实测排」（`.claude/rules/verification.md:41`）。逐个对：
- 文件级：「按角色排，管怎么存、怎么算键，不管判对错」（`.claude/rules/verification-reference.md:190`）点名的是 lib_verdict_library.py 与 lib_function_units.py（及它的拆分族，登记在 `NOT_JUDGING_FILES`）；admission.py 不在里面。这一行拿「同一个角色」把文件级排除的理由用到第三份文件上，却不登进 `NOT_JUDGING_FILES`。
- 边级：「每一条登记的边配三格自证」（`.claude/rules/verification-reference.md:191`），登记位是 `EXCLUDED_PROGRAM_EDGES`；这一行登在 `UNKEYED_PROGRAM_WORDS`，没有三格。而且这条边改判定结局：共用段起的 plan-failure 退非 0 就让那一格判红（`code-tooling-shared-libraries.sh` 共用段注释「失灵与服务不可用交 plan-failure 判」），不是「只打提示」。
- 算键库自己给 `UNKEYED_PROGRAM_WORDS` 列的类别是「而它不是这一条判定要收的仓里代码的那几处，逐处登记：编出来的装置」（`scripts/gate/lib_function_units.py:232`）起的一串，没有「判定库记账与算键」这一类；起仓里的代码、折不出的，文件头给的登记位是另一张表：「而被起的是仓里的代码（自证起这份文件自己的别的子命令」（`scripts/gate/lib_function_units.py:251`），「不拒算，登记的那几份照程序入口整份连上（参数不知道，不剪）」（`scripts/gate/lib_function_units.py:252`）。
- 同一种边别处就是这样登的：「('scripts/gate/refused-test-registry.py', 'run_command', 'command'): ('scripts/gate/admission.py',),」（`scripts/gate/lib_function_units_registry.py:409`），钩子的「designer_admission_verdict」（`scripts/gate/lib_function_units_registry.py:411`）同样连到 admission.py。同一份 admission.py，这两处进键，`admission_run` 这一处按角色不进。

先例（不是这一批加的，照实写）：`device-code-key.py` 的 `call_unit_graph` 那一行与 `EXCLUDED_CALL_EDGES` 的 `built_unit_graph` 那一行，理由同样写「与 NOT_JUDGING_FILES 同一个角色」；字面上的第三种排除在这一批之前就有，这一批多了一处、射程是 admission.py 整份。

**让哪些单元出键、改 admission.py 会不会错复用：不成立（没打中）。** 试的形状：共用段只经 `admission_run` 起 admission.py 的 `library-files` 子命令——「admission_run(self.admission, ["library-files", "fetch", library, directory, *relatives]」（`.claude/gate.d/lib/code-tooling-shared-libraries.sh:1030`）、「admission_run(self.verdicts.admission, ["library-files", "store"」（`.claude/gate.d/lib/code-tooling-shared-libraries.sh:1141`）；shell 那一格直接起的只有 plan-failure、verdict-record、verdict-plan 三个子命令（例「python3 "$STAGE_REPOSITORY/scripts/gate/admission.py" plan-failure」，`.claude/gate.d/lib/code-tooling-selftest-cells.sh:453`），算键库照字面子命令只连分派表那一项（「分派表：本模块只赋过一次的字典字面量[已知的串] 只连那一项的值」，`scripts/gate/lib_function_units.py:111`；分派表里「"library-files": command_library_files,」，`scripts/gate/admission.py:1078`）。假如 library-files 那一支只经 `admission_run` 才连得到，改 `library_files_fetch` 不变 code-tooling.sh 的键，而样本 `break-switch-replay-key-changed-red`、`stage-break-switch-replay-key-changed-red` 的 expect 要「组成清单有」那一句（靠 library-files fetch 搬回组成清单），以 code-tooling.sh 为阶段的那几条开关复跑判定就会照旧键复用。

反证（量的，开工前一版）：任务服务日志目录 `~/.local/state/singlefs/task-server/logs/` 里同一日有一份 code-tooling.sh 的闭包输出（`kept-comments` 那一行是 `.claude/gate.d/code-tooling.sh:1-1491`；调度记录没点这份日志，是我按内容 grep 到的，日志文件名不写），admission 一族按文件数的单元行：admission.py 414、admission_base.py 180、admission_commands.py 196、admission_crash_case.py 110、admission_unit_key.py 198；点名的几个单元都在，原样四行：

    def	command_verdict_plan	scripts/gate/admission_commands.py:724-750	f21419c7d500a9e790e1a898e13328f984b7de01221198786f230ae3fe55c905
    def	library_files_fetch	scripts/gate/admission_unit_key.py:1598-1619	ad88cb521f2c5d61ed6253e4f647f417e5287ffa4e5404adb4556fddea8d5010
    def	command_library_files	scripts/gate/admission_unit_key.py:1640-1674	ccb80274e60e1d3cf6dfeb4dbfe312321e0481e2b3ad959a088119256c4edf46
    import	library_files_fetch	scripts/gate/admission.py:840-854	f14be4d5ab8246dd0e7438883cbc956d18a3dd25334e2f55c932db4cc15fb5ad

所以 admission.py 经别的边（推的：模块级 import 那一句与 shell 格直接起的几条）整族在 code-tooling.sh 的闭包里，`admission_run` 这一行在效果上不让任何单元出键，只是不让那 7 处拒算。工具员当时写的理由「不登、照折的话 admission.py 连同它 import 的算键库进 code-tooling.sh 每条判定的键」（`/home/fy5090/.local/share/singlefs/work/function-units-library-text-folds/result.md:66`）与这份输出对不上：不登它也已经在键里。这份闭包是开工前一版的，开工快照那一刻没量；`probe-closure-admission.sh` 对此刻的仓根重算一次、打 via 边（待主 agent 跑）。推翻这一条的观测：重算的闭包里 `def library_files_fetch` 或 `def command_library_files` 不在。

**`command` 参数：推不出（潜在绕过，推的）。** 起法是「child = subprocess.Popen(command or ["python3", admission, *arguments]」（`.claude/gate.d/lib/code-tooling-shared-libraries.sh:735`），登记按整句认，`command` 交什么都不拒算、不连；今天交它的只有样本造卡住那一处「changes, **stall)」（`.claude/gate.d/lib/code-tooling-shared-libraries.sh:894`），理由里也写了「command 只在样本档造卡住时给」（`scripts/gate/lib_function_units_registry.py:66`）。核登记的那一格只核函数与词还在，不核调用方交的是什么；以后有人经 `command=` 交一份仓里的脚本，那份脚本静默出键。今天没有这样的调用方，算不上打中。

### `TripLauncher.run` 那一行：不成立

登记在「(".claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh", "run", "command"):」（`scripts/gate/lib_function_units_registry.py:263`），经它起的只有 break-switch-replay 与 selftest-cells-alone 两格（`grep` 现查 `TripLauncher(` 只有这两处）。它落在文件头「表行与样本给的命令」那一类；被起的那份是每条判定的入口：「tokens.append("program-argument:--selftest")」（`.claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh:505`），要套临时服务的另带「"line:temporary-service=wrapped"」（`.claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh:507`）。改了被复跑的自证，那一条判定的入口单元就变，键变、现判。没找到误伤或错复用的形状。

### `TAIL_READ_SCOPES` 新加那一行：不成立

「(".claude/singlefs-ai-sop/scripts/claude-hooks/discarded-output-count-check.sh", "run_selftest", "os.path.join(scratch_directory, 'records.tsv')"): ((),」（`scripts/gate/lib_function_units_registry.py:514`）：读的是自证临时目录里钩子刚写下的检出记录，不登的话按尾巴把 `research/prompts/` 下一份 records.tsv 收进键；登空元组照「读的是调用方交进来的输入的登空元组」（`.claude/rules/verification-reference.md:193`）同一道理（这里是自证自己造的输入），不是排除代码单元。别的会话加的 `unkeyed_toolchain_and_configuration_lines` 那一行只在弄坏开关下读被判那棵树根上的几份，同理不成立。

旁支（不在这一格的问句里，照实交）：同一张表 Q 份登的 `package_key` 那一行，理由写「name 取 ROOT_INPUTS（Cargo.toml、Cargo.lock、rust-toolchain」（`scripts/gate/lib_function_units_registry.py:597`），而此刻「ROOT_INPUTS = ("Cargo.toml",)」（`scripts/gate/crate-cross-references-check.py:258`）：理由文字落后于代码，登记本身（空元组）不受影响。

## 判据格 5：排除表与用户定案

用户定案（主 agent 的转述，记录里只有选项名「按基准登一次表」是原话）：「只对这一批放开一次「只缩不涨」，之后照旧只缩；真该改的用法照改代码」（`records/2026-11-16-三方豁免取消与存量补轮.md:757`），同一行点名的类是「自证造的假值、钉住一份脚本的单文件拷贝、`${X:-1}` 这类不并行的写法」。正文第 6 条把第 757 行叫「原话的出处」，那一行是主 agent 记的定案、不是用户原话逐字，照实写一句。

逐行先核「提交基准里就有」：扫描格三登的 25 行里有 3 行（`crates_mutation_rows_base.py`、`crates_mutation_rows_selftest_fixtures.py` 两份的变异自证夹具）开工快照之前已从 heavy 表删掉，快照那一版三张表比 HEAD 新加 22 行、改了锚串 1 行；锚串在 `git grep -nF <锚串> HEAD`（五行 `STAGE_CELLS_JOBS` 用 `git show HEAD:<入口> | grep -nF`）里逐条都找得到同一写法（量的，在仓根逐条跑过，命中原样从略，每条至少 1 处）。下面只判「确实不是跑法」。

### 打中：五行 `STAGE_CELLS_JOBS`（`.claude/heavy-thread-exempt` 第 63–67 行）

登记理由：「取不出交 1 即不并行），STAGE_CELLS_JOBS 只是调用方压过它的开关，这一格认不出这个包装函数」（`.claude/heavy-thread-exempt:62`），例「.claude/gate.d/doc-decisions.sh:STAGE_CELLS_JOBS="${STAGE_CELLS_JOBS:-$(stage_cells_default_jobs)}"」（`.claude/heavy-thread-exempt:64`）。

为什么是「真该改的跑法」而不是「认不出包装」：
1. 它就是跑法：五道门禁入口的格同时开几路，缺省取它；不属于点名三类里任何一类（不是假值、不是单份拷贝、也不是 `${X:-1}` 那种不并行——缺省是 31 路）。
2. 登记位只认两种取法：「取法：shell 经 scripts/hosts/heavy-threads.sh 的 heavy_thread_default」（`.claude/heavy-thread-default:2`），且「SINGLEFS_THREAD_CAP 设了照它，脚本的参数给了照参数。」（`.claude/heavy-thread-default:4`）、「写坏或删掉这份文件，上面几个取法一律拒绝，不退回按核数算。」（`.claude/heavy-thread-default:5`）。`stage_cells_default_jobs` 自己读登记文件「registered="$(sed -n」（`.claude/gate.d/lib/stage-cells.sh:306`），读不出就「[[ "$registered" =~ ^[1-9][0-9]*$ ]] || registered=1」（`.claude/gate.d/lib/stage-cells.sh:307`）：不认 `SINGLEFS_THREAD_CAP`、写坏与不在都悄悄退成 1 路、`2>/dev/null` 吞掉原因。heavy-thread-defaults 格判的正是「不经这里自己定并行度默认值的写法」，它红在这五行上是判对了，不是判法认不出。
3. 主 agent 在同一轮对同一种写法两次退回：退回扫描格二的改法时写「不 source 时 `heavy_thread_default` 没定义、又有 `2>/dev/null`，各道门禁的格并行会悄悄退成串行」（`records/2026-11-16-三方豁免取消与存量补轮.md:756`）；doc-text 那一处改成取登记值时写明「也不用取不出就交 1 的 `stage_cells_default_jobs`」（`records/2026-11-16-三方豁免取消与存量补轮.md:764`）。同一个函数，doc-text 那一处判成要改，五道入口却登成「不是跑法」。

具体输入（每一步许可它的那一句在上面 2 条里）：
- 任务服务照 `capped.sh <N>` 设 `SINGLEFS_THREAD_CAP=5`、登记 31：`heavy_thread_default` 交 5，`stage_cells_default_jobs` 交 31，格并行超出服务给的线程数。
- 登记文件第一条非注释行写坏（`abc  # 理由`）或文件不在：`heavy_thread_default` 退 2 拒，`stage_cells_default_jobs` 退 0 交 1，五道门禁照常跑完、只是悄悄串行。
- 登记第一条非注释行写坏、后面另有一行 `7  # 理由`：前者拒，后者交 7（`sed` 取的是第一条以数字起头的行，不是第一条非注释行）。

推的各格读数（待 `probe-stage-jobs.sh` 复跑；每格「stage_cells_default_jobs 交什么／退几」对「heavy_thread_default 交什么／退几」）：

| 格（探针里的名字） | 登记 | cap | stage 侧 | heavy 侧 |
|---|---|---|---|---|
| registered-value | `31  # 样本理由` | 不设 | 31／0 | 31／0 |
| cap-five-over-registered | 同上 | 5 | 31／0 | 5／0 |
| cap-not-a-number | 同上 | abc | 31／0 | 空／2 |
| registration-missing | 不建 | 不设 | 1／0 | 空／2 |
| registration-not-a-number | `abc  # 样本理由` | 不设 | 1／0 | 空／2 |
| registration-zero | `0  # 样本理由` | 不设 | 1／0 | 空／2 |
| registration-without-reason | `7` | 不设 | 7／0 | 空／2 |
| registration-without-space-before-hash | `7# 样本理由` | 不设 | 7／0 | 空／2 |
| malformed-line-before-a-number | `abc  # 写坏的一行` 换行 `7  # 样本理由` | 不设 | 7／0 | 空／2 |

推翻条件：复跑 `name=stage_jobs_summary` 的 `disagree=0`（九格两边逐格同值同码），这一条撤回。

### 打中：快照那一版 repository-copy-closure-exempt.tsv 第 44 行 `SHARED_INPUTS`

快照那一版第 44 行是 `.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh:SHARED_INPUTS = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain"`（照工具员草稿那一份逐字节相同的拷贝转述，那份在仓外）。开工快照之前那份库已经改成「SHARED_INPUTS = ("Cargo.toml", ".claude/gate.d/stage-inputs.tsv",」（`.claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh:747`；修改时刻早于快照清单，`stat` 现查），被锚的写法不在了。开工快照之后同一日一趟 `code-tooling.sh --check change-range-single-source,heavy-thread-defaults,hardcoded-quota,memory-bounded-parallelism,repository-copy-closure` 的日志（按内容 grep 到，五格，结局 change-range-single-source、heavy-thread-defaults、hardcoded-quota、repository-copy-closure 红，memory-bounded-parallelism 绿）原样：

    ✗ 排除表 .claude/gate.d/repository-copy-closure-exempt.tsv 有 1 处不对：
       第 44 行的锚串「SHARED_INPUTS = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain"」在 .claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh 里一处被判行都不命中（那一处已经改成经共用库拷，或锚串抄错了）

同一份日志里 repository-copy-closure 没登记的写法只剩 `lib_function_units_selftest_rust_cells.py:1026`、`:1027` 两处，不再有 `SHARED_INPUTS`：这一行登进来时罩的是提交基准里的写法，到开工快照那一刻已经什么都不罩，只让门禁多红一处。这是「排除表与用户定案」格里「新登的行逐行站不站得住」的打中，不是「借放开登跑法」那一种。开工快照之后这一行已被删（此刻的表 49 行、没有它），所以对此刻的仓不再成立。

### 不成立的几行（逐行）

| 表与行 | 写法 | 这一行站不站得住 |
|---|---|---|
| heavy 第 68 行 `stage-cells.sh` `${STAGE_CELLS_JOBS:-1}` | 没设就一格接一格跑 | 点名三类里的 `${X:-1}` 那一类，成立登表 |
| heavy 第 71 行 crates `temporary_servers` | 把调用方给的 `heavy_threads` 分一半给临时服务 | 是跑法、但取的是登记值，不是真该改；不在点名三类里，属主 agent 另收的「是合规写法、判法没认出，照收」（`records/2026-11-16-三方豁免取消与存量补轮.md:759`）一类，交主 agent 判定案罩不罩这一类 |
| heavy 第 72–74 行三份测试的 `CPU_THREADS_ENVIRONMENT_VARIABLE` | 变量名常量 | 不是跑法，成立 |
| quota 第 16–18 行 e185 三份 `E185_MEMORY_CAP=3G` | 自证「调用方已给了照留」那一格的假值 | 自证假值，成立 |
| quota 第 19 行 `merge-and-verify-steps-lib.sh` `36G` | 只在 `MERGE_AND_VERIFY_BREAK=rerun-command-wrapped` 那一支拼成文字打印（`grep` 现查那一支的 `if`），不起 | 开关坏命令文字，成立 |
| repo 第 45–47 行 `e171-row-shard-selftest.sh` 两处 `cp` 与假驱动的 `SYNCED_ENTRIES` | 自证小树里钉单份脚本、假驱动常量 | 成立 |
| repo 第 48 行 `replay.sh` `cp -- "research/results/$name"` | 把一份入库产物取进这一行的临时目录 | 是复跑真走的路，但不是建仓副本，成立 |
| repo 第 49 行 `admission_selftest_cells.py` 拷 `lib_work_root.py`、第 50 行 `merge-and-verify-selftest-lib.sh` 留底 | 自证单份拷贝 | 成立 |
| repo 第 51 行 `merge-and-verify.sh` `WORKSPACE_ROOT_ENTRIES` | 指纹起点、只读不拷 | 成立 |
| repo 第 13 行 e171 `row_driver.py` 锚串补 `, *CONFIGURATION_CHECK_ENTRIES` | 冻结装置；基准那一版锚串本就与文件对不上 | 改锚串，不是新登，成立 |

doc-text 那一处：三张表里 `grep -n 'doc-text'` 零命中（退出码 1，量的），`cell_date_once` 改成 source `scripts/hosts/heavy-threads.sh` 取 `heavy_thread_default`、取不出判红，它本就没登进表：「已经不该在排除表里」成立。

### 旁支打中：「之后照旧只缩」在拆分族上守不住（推的，待复跑）

三张表比基准经 `keyed_family_growth`：「基准里那个入口下有同一段的不算涨；族外、段对不上的算涨。」（`scripts/gate/lib_lag_split_family.py:178`）。它按（族入口, 锚串）的集合比，不比行数；同一份库里给滞后表用的 `family_growth` 是「归到同一行基准的几个路径，计数之和不超过基准那一行就不算新加」（`scripts/gate/lib_lag_split_family.py:23`）。

具体历史：基准里 admission.py 有锚串 `"nproc"`、`os.cpu_count(), len(os.sched_getaffinity(0))` 两行；拆分之后新建同族文件 `scripts/gate/admission_thread_probe.py`，里面真拿核数定并行度，再往 heavy 表照抄两条锚串各登一行。`keyed_family_growth` 判 0 行涨，heavy-thread-defaults 被排除表罩住、也不红，表从 2 行涨到 4 行而门禁绿。用户定案「之后照旧只缩」在这一条路上不成立。

快照那一版表里已经有这种行（不是这一次放开登的，是拆分族改表那一件带进来的，`git show HEAD:.claude/heavy-thread-exempt` 现查基准各一行）：「scripts/gate/admission_base.py:"nproc"」（`.claude/heavy-thread-exempt:17`）与「scripts/gate/admission_selftest_cells.py:"nproc"」（`.claude/heavy-thread-exempt:18`）对基准一行；`crates_mutation_rows_memory.py` 与 `crates_mutation_rows_pool.py` 两行 `os.cpu_count() or 4`（第 40、41 行）对基准 `crates-mutation-rows.py` 一行。这几行罩的都是自证或「要改」的老写法，内容本身没问题；问题是这条路对任何新文件都开着，这三格的样本没有一份喂「同族新文件抄已有锚串」。

推的读数（待 `probe-family-growth.py` 复跑，keyed／counted 两列是两个函数各交几行涨）：split-moves-rows 0／0；same-anchor-in-two-members 0／1；new-member-copies-anchors 0／2；outside-family-copies-anchor 1／1；new-member-new-anchor 1／1。推翻条件：`new-member-copies-anchors` 那一行 `keyed_grown` 不是 0。

这一条碰的是拆分族认法（判据格 1、4 归正推腿），这里只从格 5「之后照旧只缩」的角度交；与正推腿重了的，由主 agent 并。

## 「已经量的」逐句核对

记录指第 711–782 行；日志是我在任务服务日志目录按内容 grep 到的同一日的日志（记录里没有一处任务号或日志路径，见末行），只抄行的字。

| 正文那一句 | 记录 | 日志 | 判 |
|---|---|---|---|
| Z、P4 与变异键整包都换上之后算键库自证 exit=1、787 格只红一格，是 `crate-cross-references-check.py:919` 的尾巴读法 | 「算键库自证 exit=1、787 格只红一格」（`records/2026-11-16-三方豁免取消与存量补轮.md:781`） | `✗ lib_function_units.py 自证没过：1 格判错（共 787 格，并行 31 个进程，621.8 秒）`，红的那一格点名 `scripts/gate/crate-cross-references-check.py:919 unkeyed_toolchain_and_configuration_lines` | 核得到。开工快照之后同一日另有一趟同一自证同样「1 格判错（共 787 格…）」，红的换成 `scripts/experiment-apparatus/e171_three_oracle_collusion/test_o3_columns.py:117`（`crate-cross-references-check.py` 的修改时刻早于快照清单）：快照那一刻这句已不是现状，在哪棵树上跑的我核不到 |
| P4 换上前那一趟 exit=0「788 格都对…弄坏开关 344 个各红在点名的格」 | 「算键库自证 exit=0「788 格都对…弄坏开关 344 个各红在点名的格」」（`records/2026-11-16-三方豁免取消与存量补轮.md:777`） | `✓ lib_function_units.py 自证通过：788 格都对（…弄坏开关 344 个各红在点名的格；并行 31 个进程；606.0 秒）` | 核得到 |
| X、Y、Z 新加的开关各单跑 exit=1、红在点名的格 | X：「三条新旧开关各 exit=1、红在点名的格（python-module-object-exec-refused 下比报告预计多红一格」（`records/2026-11-16-三方豁免取消与存量补轮.md:771`）；Y：「六条新开关单跑各红在点名的格（python-module-object-refusal-lines-at-exec 下另红登记表那一格，开关表没点名）」（`records/2026-11-16-三方豁免取消与存量补轮.md:774`）；Z：只重跑两条旧开关「shell-heredoc-captures-unfolded、python-module-object-refusal-lines-at-exec 两条开关各只红点名的格」（`records/2026-11-16-三方豁免取消与存量补轮.md:777`） | 没逐条找 | 核不全：X、Y 那几趟各多红一格（之后改开关表补点名，补完没见单跑记录）；Z 的补丁没有新开关（`grep '^+.*FUNCTION_UNITS_BREAK='` 在 Z 补丁里只命中改说明的两条旧开关）；Y、Z 的单跑记录没写 exit。「各红在点名的格」能由 788 格那一趟的开关轮整体支撑，不由「各单跑」支撑 |
| lib_verdict_library 等四份与 verus-proof-check 自证 exit=0，verus「196 格都判对；弄坏开关 122 个，每个都让至少一格判错」 | 四份：「entries、cross-run-caches、lib_verdict_library、crates-split-pure-move、rust-name-collisions、checker-direction-overlap exit=0」（`records/2026-11-16-三方豁免取消与存量补轮.md:781`）；verus：「verus-proof-check exit=0「196 格都判对」」（`records/2026-11-16-三方豁免取消与存量补轮.md:782`）；带「每个都让至少一格判错」的整句只在第 734、750 行（更早的两版） | 同一日最后一趟 verus 自证日志原样 `✓ verus-proof-check 自证 196 格都判对；弄坏开关 122 个，每个都让至少一格判错` | 核得到（引号里那半句靠日志，不靠记录） |
| admission 自证在整包换上前一直 exit=1、43 格「读不了测试服务代码登记表」 | 「十来格「读不了测试服务代码登记表」（`records/2026-11-16-三方豁免取消与存量补轮.md:758`），之后「admission 自证 exit=1、43 格判错」（`records/2026-11-16-三方豁免取消与存量补轮.md:771`）、「admission 自证照旧 43 格」（`records/2026-11-16-三方豁免取消与存量补轮.md:774`） | 同一日几趟 `✗ admission.py 自证没过：43 格判错（共 417 格）`，中间夹着几趟 `1 格判错（共 421 格）` | 大体核得到；「一直 43 格」不准，最早那一趟记录写的是「十来格」 |
| 整包换上后的那一趟归别的会话在交 | 第 778–782 行没有一句提 admission 自证 | 同一日更晚有两趟 `✓ admission.py 自证通过：421 格都对（…）` | 记录里核不到；日志显示已经跑出绿的了（在哪棵树上、谁交的核不到），正文这句是旧的 |
| `key . code-tooling.sh` exit=4、只 5 行 external-program-unkeyed；`closure` exit=0、0 行拒算；`key . doc-decisions.sh` exit=0 | 「code-tooling 闭包 exit=0、0 行拒算，`key` exit=4 只 5 行 external-program-unkeyed」与「doc-decisions `key` exit=0」（`records/2026-11-16-三方豁免取消与存量补轮.md:777`） | 两份 `key` 日志各 5 行 `refused	external-program-unkeyed`（cargo、herd7、两种 qemu-system、busybox） | 核得到，但第 777 行是 P4 换上之前；第 778–782 行 P4 换上之后没再记这三条，开工快照那一刻推不出 |
| stage-break-switch-replay 两趟「现判 2、复用 0、拒算 0」「现判 13、复用 0、拒算 0」，每条「点了没错：无」，多判错 10 份与 `mutation-list-items-reuse-three-questions` | 两条开关那一趟在第 774 行（Y 换上后、Z 与 P4 之前）；13 条那一趟「开跑前「现判 13、复用 0、拒算 0」」（`records/2026-11-16-三方豁免取消与存量补轮.md:782`） | 两条那一趟 `· 按条复用：2 条开关（1 道阶段），现判 2、复用 0、拒算 0`、`✗ 门禁阶段的弄坏开关复跑：2 处不对（复跑了 2 条、1 道阶段；本次未跑 0 项）`；13 条那一趟 `· 按条复用：13 条开关（1 道阶段），现判 13、复用 0、拒算 0`、`✗ 门禁阶段的弄坏开关复跑：13 处不对（复跑了 13 条、1 道阶段；本次未跑 0 项）`，「点了没错：无」13 处 | 核得到；两条开关那一趟是 Y 那一版的算键库，不是快照那一版 |
| doc-text 样本档 exit=0；开关下只红点名那一个用例 | 「type=command-failed exit=1，判错的只有点名那一个用例「date-once-thread-default-red 期望退出 1，实测 0」」与「不开关那一趟 type=succeeded exit=0」（`records/2026-11-16-三方豁免取消与存量补轮.md:768`） | `✓ 点名的 1 道阶段判得都对（115 个用例）`；开关那一趟 `✗ 1 个用例判错（判对 114 个，0 个阶段没有用例）`、`✗ doc-text date-once-thread-default-red 期望退出 1，实测 0` | 核得到 |
| 三张表「排除表涨了」各 13、4、9 行，与新登的逐行对上 | 「三处「排除表涨了：这几行基准 4319aa2b035b 那一版里没有」各 13、4、9 行，与新登记的逐行对上」（`records/2026-11-16-三方豁免取消与存量补轮.md:761`）；登的是「照用户定案登 25 行」（`records/2026-11-16-三方豁免取消与存量补轮.md:759`）；后来另一处报「第 63–68、71–73 行」（`records/2026-11-16-三方豁免取消与存量补轮.md:781`） | 开工快照之后那一趟：heavy 列第 63–68、71–74 行共 10 行，quota 第 16–19 行 4 行，repo 第 13、44–51 行 9 行，另报 repo 第 44 行锚串一处不命中 | 对记录核得到；对开工快照对不上：heavy 是 10 不是 13（3 行开工前已删）；repo 的 9 行里第 13 行是改锚串、不是新登（13+4+9=26，登的是 25）；新登的第 44 行在快照里已成死行 |
| 标题「号见调度记录」 | 第 711–782 行按任务号、日志文件名、`log=` 与 `.log` 四种写法 `grep -nE` 零命中（量的，命令在本节下方） | — | 不成立：任务号按规矩不进记录，日志从记录点不到；正文应写成「日志位置不在记录里」或另给命令与结局行 |

末行的命令与输出（量的；先把第 711–782 行取进草稿目录再数，`grep -c` 退 1 是零命中、不是出错）：

    awk 'NR>=711 && NR<=782' 'records/2026-11-16-三方豁免取消与存量补轮.md' > <草稿目录>/record-711-782.txt   → awk_rc=0，72 行
    grep -cE '<任务号、日志文件名、log=、.log 四种写法>' <草稿目录>/record-711-782.txt                  → 0，grep_rc=1

## 打中之后的四句

| 打中 | 分不分辨臂 | 系统当时看不看得到 | 满足判据字面哪一分句 | 改法在打中的格上还中不中 |
|---|---|---|---|---|
| 五行 `STAGE_CELLS_JOBS` | 分辨：「登表」那一臂九格里 8 格两边不同（推的），「照改代码」那一臂改成经 `heavy_thread_default` 后九格同值（推的） | 看得到：heavy-thread-defaults 在这五行上判红；主 agent 在记录第 764 行点名过这个函数的退回写法 | 格 5「有没有借这次放开把真该改的跑法也登进去」；定案「真该改的用法照改代码」 | 改法「留表」八格照中；改法 S（下表）八格都不中（推的） |
| 快照里 repo 第 44 行 | 分辨：留行门禁红、删行不红 | 看得到：门禁自己报「一处被判行都不命中」 | 格 5「这一次新登的行，逐行是不是「提交基准里就有、确实不是跑法的写法」」——快照里它罩的写法已不在 | 删行即修；开工之后已删（此刻表里没有，量的） |
| `keyed_family_growth` 不比行数 | 分辨：集合比判 0 行涨，计数比判 2 行涨（推的） | 看得到：判法手里有此刻每行的路径与锚串，计数现成 | 定案「之后照旧只缩」 | 改法 K 修「新文件抄锚串」那一格；不修「行数没超但新文件里换了写法」那一种 |
| `admission_run` 字面第三种排除 | 字面分辨、效果不分辨：admission 一族已经经别的边在闭包里（量的，开工前一版） | 看得到起的是 admission.py；只是 `admission` 那个参数从载入方内联正文来，折不到 | `.claude/rules/verification.md:41` 只认两种排除 | 改法 A 修字面那一格；修不了 `command` 那条潜在绕过（登的仍是整句） |

## 改法（都只在我的推理上过、没在模型上量过，被攻过零轮）

| 改法 | 做什么 | 修哪一格 | 量过／推的 |
|---|---|---|---|
| S | `stage_cells_default_jobs` 改成 source 仓根的 `scripts/hosts/heavy-threads.sh`（从 `.claude/gate.d/lib/` 往上三级，不是扫描格二写错的两级）再取 `heavy_thread_default`，取不出整道判红、不吞标准错误，照 `cell_date_once` 的写法；删 heavy 表第 63–67 行 | 五行 `STAGE_CELLS_JOBS` 的八格不同 | 推的，待主 agent 跑（改它会让五道门禁的键全变、整道重判一次，推的） |
| 删行 | 删快照那一版 repo 表第 44 行 | 第 44 行死行 | 推的；开工之后已有人删了（量的） |
| K | `keyed_family_growth` 照 `family_growth` 的样子按（族入口, 锚串）比计数之和，超了判涨 | new-member-copies-anchors | 推的，待主 agent 跑；此刻表里 admission 两行 `"nproc"`、crates 变异两行 `os.cpu_count() or 4` 会被判涨，要么合并、要么同一次交用户 |
| A | `admission_run` 那一行从 `UNKEYED_PROGRAM_WORDS` 挪到 `LAUNCHED_REPOSITORY_PROGRAMS`，被起的写 `scripts/gate/admission.py`，与第 409、411 行同一种登法 | 字面第三种排除 | 推的；键在效果上不变（admission 一族已在闭包里，量的开工前一版），复跑闭包可证 |

## 没打中的形状与取样范围

- `admission_run` 让 library-files 那一支出键、改 admission.py 错复用：试了 `library_files_fetch`、`command_library_files` 两个只经 `admission_run` 起的子命令的单元，开工前一版的闭包输出里都在（量的），没打中。
- `admission_run` 让别的格错复用：共用段 `UNIT_VERDICT_ROUNDS_LIBRARY` 只有 code-tooling.sh 一道载入（`grep -rln code-tooling-shared-libraries` 在 `.claude/gate.d`、`scripts`、`.claude/hooks` 只命中 code-tooling.sh 与算键登记表），门禁格本身不按键复用（`stage-cells.sh` 里只有「上次红的清单」），所以只看了以 code-tooling.sh 为闭包的那几种判定。
- `TripLauncher.run`：break-switch-replay 的请求行逐项看了（入口、`program-argument:--selftest`、临时服务包装）；selftest-cells-alone 的请求行没读代码，只读了登记理由里的描述。
- `TAIL_READ_SCOPES` 两行：看了读的表达式与登空元组的条款，没找到收进真仓文件或漏收的形状。
- quota 四行与 repo 其余八行：逐行在 HEAD 找到同一写法、读了上下文；`36G` 那一行核了只在开关那一支拼成文字。
- 历史与崩溃状态：这两格判的是登记表与门禁格，不涉及池、写流与崩溃枚举，第 3b 步的崩溃状态取样不适用；用户动作没有写死在装置里的问题。

## 这条腿自己的限度

- 一步都没跑：模型三个探针的读数全部「推的，待主 agent 跑」；表里标「量的」的只有仓里文件、记录、`git` 与任务服务日志目录里已有日志的原样行。
- 日志是我按内容在 `~/.local/state/singlefs/task-server/logs/` 里 grep 的（调度记录点不到日志）：几份日志在哪棵树上、谁交的，日志本身不写，我核不到；文件名带任务号，按规矩没写进报告。日志里的私有值（主机地址）没抄。
- admission 一族在闭包里的证据是开工前一版的闭包输出；`probe-closure-admission.sh` 对仓根此刻的源码重算，算的时候仓又可能变了（它开头另核快照清单，打对上几份）。
- 写报告途中开工快照里两份文件被改（见「开工快照核对」），判的是快照那一版；repo 表第 44 行那一条对此刻的仓已不成立。
- 第 5 格旁支碰到了拆分族认法（正推腿的格 1、4），只从「之后照旧只缩」的角度写，没判它别的形状。

## 要主 agent 跑的命令

1. `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/split-closure-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-attack/frozen /home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-attack-probe-run /home/fy5090/code/singlefs/research/prompts/split-closure-governance-r1-opus-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`（新 shell）。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0` 算跑完（退非 0 的件数为 0）；日志里 `name=stage_jobs_summary cases=9 disagree=<n> leftover_groups=0`（`disagree` 不是 0 就坐实五行 `STAGE_CELLS_JOBS` 那一条，逐格对上面的推的表）；五行 `name=family_growth_case`（`new-member-copies-anchors` 的 `keyed_grown=0` 坐实旁支那一条）；`name=closure_snapshot_check`（`failed=` 报开工快照之后又改了几份）、`name=closure_run exit=0 … refused=0` 与四行 `name=closure_unit … present=yes`（坐实「admission 一族在键里、`admission_run` 那一行效果上不排任何单元」；`present=no` 推翻它）；末行 `name=rerun_done probes=3`。出现 `name=rerun_probes skipped=` 是冻结副本给错了。
   - 探针的内存与时间：没量；闭包那一步是大头，推的几 GiB、十几分钟以内；撞上限由服务重排，外层先删运行目录。

## 没做什么

- 不判正推腿的格 1、3、4；不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在上一节）。
- 没建仓副本与编译目录；冻结副本只是从仓里拷的四份文件。
- 没读禁读清单里的三处。

## 清理

草稿目录 `/home/fy5090/.local/share/singlefs/work/split-closure-governance-r1-attack` 里只有文本：`frozen/`（复跑要用的四份，留给主 agent）、`exempt-diff.txt`、`record-711-782.txt`、`rerun-steps-block.sh`、`frozen-sha256.txt`；没有仓副本、编译目录与库拷贝，不用删。复跑的运行目录 `split-closure-governance-r1-attack-probe-run` 由主 agent 跑完后删。
