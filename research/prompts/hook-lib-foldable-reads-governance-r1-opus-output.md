# 攻方腿报告：钩子共用库按名单读本仓文件（hook-lib-foldable-reads-governance-r1，Opus）

日期：2026-11-21。攻击面：正文「要腿判的」第 1 到 3 问。读的范围：开工快照点名的五份（主工作区此刻那一版，与草稿 `new/` 逐字节相同，`cmp` 核过）、草稿 `base/` 的四份、工具员报告与 `change.diff`、算键库 `scripts/gate/lib_function_units*.py`（只读）、项目代码闸 `.claude/hooks/project_code_guard_decisions.py` 的调用处、被按名字判的阶段与脚本的 source 行。没读禁读清单里的另一条腿的报告。

## 探针（都在草稿目录，没跑，命令在「要主 agent 跑的命令」）

草稿目录 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus/`：

| 文件 | sha256 | 做什么 |
|---|---|---|
| `probes/probe_function_differential.py` | `3d434782e78543de7fea1577f68aae25045f0b466a0ab5ad28abf48202e99a74` | 第 1 问：改前改后两份共用库对同一批输入逐个调 `judged_by_name`、`existing_copy_elsewhere`、`dispatched_under_memory_cap`、`command_under_dispatch_launcher`，打每个不同的输入 |
| `probes/probe_key_leak.py` | `45944996ec7e2a6e4b2bf59c54e5cc298bd3853480f008bf87877bff03db27eb` | 第 2、3 问：两臂逐个改一份文件重算键；改后那一臂同一输入改前后各判一次；临时变体（撤掉这一改的实质、名字留着）跑共用库自证与 closure |
| `copy-new/` | 仓副本（`minimal-copy.py` 建，217M） | 改后那一臂：主工作区此刻 |
| `copy-base/` | 仓副本（同上，217M） | 改前那一臂：同一棵树，四份 `lib_project_code_runs*.py` 换成工具员草稿 `base/` 的；`diff -rq copy-base copy-new -x .git` 只报这四份 |

两份副本在草稿目录 `.deliverables` 写成「交接」。副本上量出的数只算线索，判决要引的由主 agent 照下面的命令复跑。

## 各格判定一览

| 问 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 1 判定变化 | 不成立 | 项目代码闸里调到 `same_content_as_own` 的三处（`project_code_guard_decisions.py` 第 938、1051、1083 行）逐个对过，除工具员写的两处没有第三处判定不同的输入；两处都往 False 走（读正文、不算经内存包装），只收严；派发那一支闸里走不到 | 读码推的；差分探针待主 agent 跑 |
| 2 键漏输入 | 成立（打中） | 改前整目录数据单元（递归）罩着的 `.claude/gate.d/lib/stage-cells.sh` 与四份 `crash-point-replay-*.sh`、`scripts/hosts/heavy-threads.sh`、`.claude/hooks/bash_command_detector_scope_and_index.py`，改后掉出键，判定照样随它们变；`.claude/scripts/preflight.sh` 改前改后都不在键里；库那一半照 fs-design 第 3 条该拒算（或改成收进键），不该静默 | 读码推的；键探针待主 agent 跑 |
| 3 新六格与开关 | 成立（打中） | 第 1、2 格在改前代码上一样绿；六格加新开关都不红在这一改的目的（读法折得出）上：查表之后照旧拿 `location` 拼路径读、拷文件改回元组相加的变体上六格照绿（推的）；第 3 格的输入是目录列举、不进键，漂移那一刻红不出来 | 读码推的；变体探针待主 agent 跑 |

## 第 1 问：判定变化

### 三处调用点逐个对

`own_script_digest` 取代的只有一处（「if digest is None or digest != own_script_digest(location):」（`.claude/hooks/lib_project_code_runs_launchers.py:1370`）），改前那一句是 `digest != file_sha256(own)`，`own` 是同一函数里「own = os.path.join(HOOK_REPOSITORY, location)」（`.claude/hooks/lib_project_code_runs_launchers.py:1368`）。所以判定不同只能出在 `location` 取值上：`file_sha256(HOOK_REPOSITORY/location)` 与 `own_script_digest(location)` 不同的那些 `location`。`same_content_as_own` 在全仓 `.py`、`.sh`（不含 `fixtures/` 与 `research/`）只有三个调用者，都在 launchers 里：

| 调用者 | `location` 怎么来 | 改前读 | 改后读 | 不同的输入 |
|---|---|---|---|---|
| `judged_by_name` 阶段那一支 | 「or same_content_as_own(resolved, os.path.join(GATE_STAGE_DIRECTORY, stage)))」（`.claude/hooks/lib_project_code_runs_launchers.py:1330`），`stage` 取「GATE_STAGE_PATH = re.compile(r"(?:^|/)\.claude/gate\.d/([^/]+\.sh)$")」（`.claude/hooks/lib_project_code_runs_base.py:334`）那一组，不含 `/` | `HOOK_REPOSITORY/.claude/gate.d/<stage>` | 同一份路径，前提是 `stage` 在 `GATE_STAGE_FILES` 里（「if os.path.join(GATE_STAGE_DIRECTORY, stage) == location:」（`.claude/hooks/lib_project_code_runs_base.py:514`）） | 只有「盘上本仓有这一道、表里没有」（工具员写的漂移那一处） |
| `existing_copy_elsewhere`（闸第 938 行剥包装、`judged_by_name` 按名字那一支都走它） | 「location = KNOWN_SCRIPT_LOCATIONS.get(os.path.basename(word))」（`.claude/hooks/lib_project_code_runs_launchers.py:1354`） | `HOOK_REPOSITORY/<KNOWN 的值>` | KNOWN 那一圈先比、比中就读同一份（「return file_sha256(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[name]))」（`.claude/hooks/lib_project_code_runs_base.py:511`）） | 无：`location` 恒是 KNOWN 的一个值 |
| `dispatched_under_memory_cap` | 「location = os.path.join(DISPATCH_LAUNCHER_DIRECTORY, os.path.basename(words[0]))」（`.claude/hooks/lib_project_code_runs_launchers.py:485`） | `HOOK_REPOSITORY/scripts/hosts/<basename>` | 只有 `multi-host-run.sh`（「if os.path.join(DISPATCH_LAUNCHER_DIRECTORY, launcher) == location:」（`.claude/hooks/lib_project_code_runs_base.py:517`））与 KNOWN 里落在 `scripts/hosts` 的 `capped.sh`、`run-with-memory-cap.sh` 挑得到 | basename 是 `scripts/hosts` 下别的文件（工具员写的派发那一处） |

`HOOK_REPOSITORY` 没有被改绑的地方：`grep -rnE 'HOOK_REPOSITORY\s*=|\.HOOK_REPOSITORY\s*=|setattr\([^)]*HOOK_REPOSITORY'`（全仓 `.py`、`.sh`，去掉 `research/`）只命中定义那一行 `.claude/hooks/lib_project_code_runs_base.py:159`，所以改前读的 launchers 模块里的 `HOOK_REPOSITORY` 与改后读的 base 模块里的是同一个值。弄坏开关 `own-script-digest-stages-unlisted` 只在设了变量时生效，不算平常的判定变化。

### 「现有调用方走不到」那一支：成立

闸里只有一处调 `dispatched_under_memory_cap`：「inner_capped = inner_capped or project_code_runs.dispatched_under_memory_cap(command.words, command.directory)」（`.claude/hooks/project_code_guard_decisions.py:1051`），它在「launched = project_code_runs.command_under_dispatch_launcher(command.words, command.directory)」（`.claude/hooks/project_code_guard_decisions.py:1049`）交回非 None 之后才走，两次用的是同一组词与目录；`command_under_dispatch_launcher` 先过「if name not in DISPATCH_LAUNCHER_OPTIONS or not located_in(words[0], directory, DISPATCH_LAUNCHER_DIRECTORY):」（`.claude/hooks/lib_project_code_runs_launchers.py:234`），而 `DISPATCH_LAUNCHER_OPTIONS` 只有一个键（「MULTI_HOST_RUN_SCRIPT: (frozenset({"--peer"}), {"--memory-cap": 1, "--label": 1}),」（`.claude/hooks/lib_project_code_runs_base.py:196`））。走到第 1051 行的 basename 恒是 `multi-host-run.sh`，改前改后读同一份。自证里的三次调用（selftest 第 799、801、874 行）命令词的 basename 也都是 `multi-host-run.sh`。工具员引的「`lib_project_code_runs_launchers.py:233`」是改前文件的行号（改后 import 多一行，同一句在第 234 行），不是调用处，是 `command_under_dispatch_launcher` 里那一道名字检查。

### 两处变化是不是只收严：是（推的）

- 漂移：`judged_by_name` 改前 True、改后 False。闸第 1083 行「if project_code_runs.judged_by_name(command.words[0], command.directory):」（`.claude/hooks/project_code_guard_decisions.py:1083`）为真就 `continue`、不读正文；为假接着往下读正文，只多出 use 与 notice，前面已记下的直接起不撤。我没找到读正文让一条原来拒的命令变成放行的路，没跑过。
- 派发：改后 False 让 `inner_capped` 不因派发器变真，里面那条按没经内存包装判，是严的一边；而且这一支闸里走不到。
- 附带（不是放宽，是可用性）：漂移期间，仓副本、合入后验证的快照里起那一道新阶段的拷贝都要读正文；正文里有闸认不出的写法时，那条提交被拒，直到有人把名字补进 `GATE_STAGE_FILES`。补表的提示靠第 3 格，而第 3 格在漂移那一刻红不出来（第 3 问）。

差分探针把上面两类之外的「不同」逐条打成 `DIFF UNEXPECTED`，把预言要不同而没不同的打成 `PREDICTED-DIFF-ABSENT`；输入是 17 道阶段与 18 份按名字判的脚本各取别处逐字节相同（绝对、相对两种写法）、内容不同、本仓那一份、盘上没有四种，`scripts/hosts` 顶层每个文件取逐字节相同、内容不同、本仓三种，外加阶段连库逐字节相同与库陈旧两棵树、漂移一格。

## 第 2 问：键漏输入

### 三张字面表与盘上：对得上（量的，只读命令）

- `GATE_STAGE_FILES` 与 `.claude/gate.d` 顶层不以 `lib-` 起头的 `*.sh`：`ls .claude/gate.d/*.sh | xargs -n1 basename | grep -v '^lib-'` 与用 `ast.literal_eval` 读出的元组比，打出 `17 17 True`。
- `KNOWN_SCRIPT_LOCATIONS` 的 18 个值加 `scripts/hosts/multi-host-run.sh`：逐个 `[ -f ]`，打出 `19 ok`、没有 `MISSING`。
- `DISPATCH_LAUNCHER_SCRIPTS` 与 `DISPATCH_LAUNCHER_OPTIONS` 的键都只有 `multi-host-run.sh`（「DISPATCH_LAUNCHER_SCRIPTS = (MULTI_HOST_RUN_SCRIPT,)」（`.claude/hooks/lib_project_code_runs_base.py:199`））。

### 打中：改前被整目录数据单元罩着、改后掉出键的文件

机制：改前那三行 via 收的是 data-directory，指纹是「数据目录的指纹：目录下（递归，什么目录都不跳过）」（`scripts/gate/lib_function_units_base.py:350`），`.claude/gate.d`、`scripts/hosts`、`.claude/hooks` 底下每一份文件（含子目录）都在键里。改后这三份数据单元没了，只剩点名的单个文件。闭包里还有几处读，读的是本仓这三个目录里的文件、算键库一直「不收也不拒算」，改前靠整目录单元碰巧罩着，改后罩不住了：

1. 库那一半。「if os.path.lexists(copy_library) and file_sha256(copy_library) != file_sha256(os.path.normpath(os.path.join(os.path.dirname(own), relative))):」（`.claude/hooks/lib_project_code_runs_launchers.py:1374`）读本仓那一份的库。我照 launchers 第 958-995 行手抄了 `script_sourced_libraries` 的正则与递归（草稿 `scratch/sourced_libs_reimpl.py`，不 import 本仓代码；转述，真函数的结果由探针现算），对 18 份按名字判的脚本、17 道阶段与 `multi-host-run.sh` 列出本仓那一边的库（输出 `scratch/sourced_libs.txt`，61 行）。落在改前那三个目录里的：
   - `.claude/gate.d/lib/stage-cells.sh`：6 道阶段 source 它（broad-mutation、crash-point-replay、crates-mutation-replay、formal-verification、lkmm、qemu-device-streams），写法认得出，例：「source "$(dirname "${BASH_SOURCE[0]}")/lib/stage-cells.sh"」（`.claude/gate.d/crash-point-replay.sh:868`）。
   - `.claude/gate.d/lib/crash-point-replay-selection-and-plan.sh`、`crash-point-replay-package-pass-verdicts.sh`、`crash-point-replay-crash-case-cells.sh`、`crash-point-replay-feature-card-and-audit-cells.sh`：`crash-point-replay.sh` source。
   - `scripts/hosts/heavy-threads.sh`：「source "$(dirname "${BASH_SOURCE[0]}")/heavy-threads.sh"」（`scripts/hosts/capped.sh:47`），`capped.sh` 是按名字判的一份。
2. 自证读检出钩子部分文件：「for detector_source_path in [os.path.join(hook_directory_of_library, "bash-command-detector.sh")] + sorted(glob.glob(os.path.join(hook_directory_of_library, "bash_command_detector_*.py"))):」（`.claude/hooks/lib_project_code_runs_selftest.py:825`）。glob 挑出的那几份算键库不收（「读目录（os.listdir / os.walk / glob）同样还不收」（`scripts/gate/lib_function_units.py:174`）），那一格比的集合就定义在「CLAUDE_EXECUTABLE_PATH_WORDS = {"$CLAUDE_CODE_EXECPATH", "${CLAUDE_CODE_EXECPATH}"}」（`.claude/hooks/bash_command_detector_scope_and_index.py:991`）。工具员报告把这一处列成「附带看到、没改」，没说改前它被 `.claude/hooks` 整目录单元罩着。
3. 新加的第 3 格读门禁目录列举：「listed_stages = sorted(name for name in os.listdir(gate_stage_root)」（`.claude/hooks/lib_project_code_runs_selftest.py:880`），同一条「同样还不收」。

能照喂的输入（第 1 类，推的，键探针与翻面探针逐条量）：

- 树 T 是一份仓副本，里面 `.claude/gate.d/crash-point-replay.sh` 与 `.claude/gate.d/lib/` 下那五份和主仓逐字节相同。`judged_by_name("T/.claude/gate.d/crash-point-replay.sh", "T")` 为 True（第 1374 行库也相同）。
- 改主仓 `.claude/gate.d/lib/stage-cells.sh` 一行（探针追加一行 `:`）。同一输入变成 False（库对不上、读正文）。
- `scripts/gate/selftest-process-guard.py::watch_judgement --judging` 的键：改前那一版随它变（`.claude/gate.d` 整目录单元），改后不变。判定变了、键没变，复用的旧判定照用。
- `scripts/hosts/heavy-threads.sh` 对 `existing_copy_elsewhere("T/scripts/hosts/capped.sh", "T")` 同理（False 变 True）；`bash_command_detector_scope_and_index.py` 第 991 行集合里多一个词，共用库自证「认得出：起 claude 的命令词与 bash-command-detector.sh 的 CLAUDE_EXECUTABLE_PATH_WORDS 同一组」那一格红，自证入口与 watch_judgement 的键都不变。

改前改后都不在键里的（这一改没碰，但它是「现在会漏进键的那一份」）：`.claude/scripts/preflight.sh`。17 道阶段与 `multi-host-run.sh` 都经 `$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh` 这一类写法 source 它（`scratch/sourced_libs.txt` 里 30 行），例如 `.claude/gate.d/doc-text.sh:1092`；它在 `.claude/scripts/`，改前那三个整目录单元也罩不到。另有 `.claude/singlefs-ai-sop/scripts/lib.sh`、`.claude/singlefs-ai-sop/scripts/preflight.sh`（`stage-selftest.sh` 的库）与 `scripts/mutation/` 下 11 份（8 份 `broad_mutation_*.py`、`crates-mutation-rows.py`、`lib_broad_mutation_selftest_fixtures.py`、`mutation-prune.py`，都是 `broad-mutation.py` 的库）同样一直不在键里。

### 库那一半「不收也不拒算」该不该拒算：该拒算（或收进键）

- 条款：「单元切不开、调用图收不全而拒算键的照跑，不静默当成没变」（`.claude/rules/fs-design.md:73`，「分支的三条约束」第 3 条末句）。
- 算键库今天把这一处归进二：「出处推不出、前缀也推不出的（对象属性、读进来的值、循环变量、调用的返回值）：照调用方交进来的认，不收也不拒算」（`scripts/gate/lib_function_units.py:157`）。它站得住的前提是调用方已经把被读的东西折进了这一条判定的输入；这里不是：读的是 `HOOK_REPOSITORY` 下本仓那一份库，watch_judgement 的请求只带被看着的命令，不带本仓 `.claude/gate.d/lib/stage-cells.sh` 的内容。前缀其实看得出是本仓（`own` 由 `HOOK_REPOSITORY` 拼出），算键库沿 `own` 往回找时停在 `HOOK_REPOSITORY` 交 None，才落到「推不出」。
- 这一改之前，`.claude/gate.d`、`scripts/hosts` 下的库靠整目录单元碰巧进键；工具员写「所以库文件此刻不在键里，改前改后都一样」（`/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads/result.md:15`），对这六份库不成立：改前在键里，改后不在。
- 打中之后的四句：分辨臂，分辨（改前那一版罩住这六份与检出钩子部分文件、改后罩不住；`preflight.sh` 两版都漏，不分辨，病根是两版共用的「二」那条归类）。被判的系统看得到判别它的东西：看得到，算键库收闭包时读得到这一处读法与它的 `HOOK_REPOSITORY` 前缀，能判成拒算。满足的分句：正文第 2 问「该进键而没进的、该拒算而没拒算的」两句都中，fs-design 第 3 条「不静默当成没变」那一句。改法在打中的格上还中不中：工具员「没做什么」里的「给每份脚本写一张字面库表」修第 1 类（表写全的话），修不到第 2、3 类；「拒算」修第 1 类，代价是闭包里有这一处的判定一律重跑；两样都碰不到第 2 类（glob）与第 3 类（列举），那两处要各自改读法。
- 我提的改法（只在读码上推的、没量过、被攻过零轮）：第 2 类改成只读定义 `CLAUDE_EXECUTABLE_PATH_WORDS` 的那一份字面路径，不 glob；第 1 类在 `own_script_digest` 旁边给每个位置配一张本仓库文件的字面表，`same_content_as_own` 按表读本仓那一边，表与 `script_sourced_libraries(本仓那一份)` 逐个相同由门禁 code-tooling 的一格核（放在门禁格里，不放进 `selftest`，免得核表那一处的读法又回到 `selftest` 的闭包）；做不到之前，算键库把「根里的目录 + 出处推不出」这一类读改判拒算（`scripts/gate/` 下，不在这一轮写范围，交主 agent 转给管算键库的会话）。

### 对不上的一处（主 agent 的观测，我没看到原输出）

正文写换上之后「data-file 48 个，都是三张表与 `PART_PROBE_COPIED_FILES` 点名的单个文件」，而三张表与那张元组合起来是 18 + 17 + 1 + 8 = 44 个互不相同的路径（KNOWN 的值没有落在 `.claude/gate.d/` 顶层、`.claude/hooks/` 或 `scripts/hosts/multi-host-run.sh` 上的）。多出的 4 个不是这几张表点名的（推测是改前就有的 7 个里留下的，例如按字面路径读的 `.claude/hooks/bash-command-detector.sh`）。键探针第三段把 copy-new 上这一条 closure 的每个 data-file 逐行打成 `CLOSURE-DATA-FILE`，主 agent 照它对。

## 第 3 问：新六格与新开关

「撤掉实质的变体」是键探针第三段临时套在 copy-new 上的那一版：`own_script_digest` 改成先拿三张表凑一个集合，`location` 在集合里就照旧 `file_sha256(os.path.join(HOOK_REPOSITORY, location))`，开关那一支照留；自证拷文件那一处「for copied_name in PART_PROBE_COPIED_FILES:」（`.claude/hooks/lib_project_code_runs_selftest.py:1057`）改回改前的元组相加，`PART_PROBE_COPIED_FILES` 照留。这一版每个名字都在、判定与改后逐条相同，只是读法又是「折得出的目录 + 折不出的一截」，算键库应当又收回整目录（或拒算）。

| 格 | 名字说的 | 改前代码上 | 变体上（推的，待探针） | 开关下 |
|---|---|---|---|---|
| 「同族：与本仓那一份逐字节相同的门禁阶段拷贝照按名字判」（`.claude/hooks/lib_project_code_runs_selftest.py:870`） | 逐字节相同的阶段拷贝照按名字判 | 绿：改前比的是同一份 `file_sha256(own)`；拷贝树里只有 `doc-text.sh`，第 1374 行的库一份都不在、整段跳过 | 绿 | `family-copy-content-unchecked`、`own-script-digest-stages-unlisted` 下红，红在名字说的那件事 |
| 派发起法逐字节相同那一格（`.claude/hooks/lib_project_code_runs_selftest.py:874`） | 逐字节相同的 multi-host-run.sh 拷贝算经内存包装 | 绿：同上，`../../.claude/scripts/preflight.sh` 在拷贝树里不在 | 绿 | `family-copy-content-unchecked` 下红 |
| 「同族：GATE_STAGE_FILES 与门禁目录顶层的阶段逐个相同」（`.claude/hooks/lib_project_code_runs_selftest.py:882`） | 表与目录逐个相同 | 跑不起来（名字不在），不是绿 | 绿 | 不挂开关。输入是目录列举、不进键：加一道阶段，自证入口与 watch_judgement 的键都不变（键探针 `new-stage-file` 两行），复用的绿照用，这一格要等别的改动让键变了才重跑；工具员报告第 110 行自己写了这一条 |
| 「按位置都挑得到」那一格（`.claude/hooks/lib_project_code_runs_selftest.py:886`） | 每个位置都挑得到 | 跑不起来 | 绿 | `own-script-digest-stages-unlisted` 下红、列出 17 道，红在名字说的那件事 |
| 「同族：字面表之外的位置不读、交 None（scripts/hosts/two-logs.sh 在盘上也一样）」（`.claude/hooks/lib_project_code_runs_selftest.py:887`） | 表外不读、交 None | 跑不起来；把 `own_script_digest` 写成改前那样只拼路径读，它红（`ls -la scripts/hosts/two-logs.sh` 在盘上，7459 字节） | 绿：变体先查表 | 不挂开关。名字里的「不读」它看不见，只看返回值：先读再按表丢掉结果的写法照绿 |
| 「引导：拷进临时目录的文件里有引导那一份与每一份部分文件」（`.claude/hooks/lib_project_code_runs_selftest.py:1053`） | 元组里有引导与每份部分文件 | 跑不起来 | 绿：元组照留，拷文件那一处改回元组相加 | 不挂开关 |
| 开关 `own-script-digest-stages-unlisted` | 门禁阶段不从表里挑 | 改前没有 | 变体照留这一支，照红第 1、4 格 | 红在名字说的那件事 |

判定：成立（打中）。第 1、2 格在改前代码上一样绿，是行为不变的护栏，分不出有无这一改；第 3 到 6 格在改前代码上跑不起来，只因为名字不在，撤掉这一改的实质而名字留着的变体上照绿。这一改的目的（判法的键只连点名的几份、不连整目录）没有一格与开关管，只有主 agent 换上之后交的那一条 closure 看过一次；以后谁把读法改回拼路径，自证、开关复跑都不红。

打中之后的四句：分辨臂，分辨（有这一改与撤掉实质的变体，在六格上逐格相同，在 closure 上不同）。被判的系统看得到判别它的东西：自证看不到，键折不折得出只有算键库看得到，所以这一项该放进门禁格（code-tooling 那一道已有起算键库的格），不放进 `selftest`。满足的分句：正文第 3 问「有没有一格在改前的代码上也绿（分不出有无这一改）」，第 1、2 格照字面中，第 3 到 6 格按变体中。改法在打中的格上还中不中：工具员没给改法；我提的（被攻过零轮、没量过）是门禁 code-tooling 加一格，对 `scripts/gate/selftest-process-guard.py::watch_judgement` 与 `.claude/hooks/lib_project_code_runs_selftest.py::selftest` 跑 `lib_function_units.py closure … --judging --explain`，断言没有 `.claude/gate.d`、`.claude/hooks`、`scripts/hosts` 三个 data-directory、没有 refused；它修「变体上全绿」那几格，修不到第 3 格的列举不进键（那一格的输入要换成算键库收得到的读法，或者把漂移检查挪进不复用的门禁格）。

## 要主 agent 跑的命令

两条按次序交，第二条等第一条出了结局再交（第二条要第一条建的 `foreign/`，又会临时改 `copy-new` 里第一条读的文件）。都不编译、不起 cargo，不设崩溃域。

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus/probes/probe_function_differential.py drive /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus`（即 `--cwd`）。
   - 看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里两行 `ARM … exit=0`，末行 `SUMMARY inputs=<N> differing=<M> unexpected=0 predicted-absent=0 reach-true=<K>`；`DIFF` 行只有 `drift`、`dispatch-unlisted` 两类；`REACH` 行只有 `scripts/hosts/multi-host-run.sh` 那几条；`LIBS` 行里 `libs-identical` 为 True、`libs-stale` 为 False。`exit=1` 时 `DIFF UNEXPECTED` 与 `PREDICTED-DIFF-ABSENT` 行就是第 1 问翻面的证据。估时推的：两个子进程各载一次共用库、几百次函数调用，一分钟以内。
2. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus/probes/probe_key_leak.py /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus`
   - 在哪起：同上。
   - 看：`exit=0` 是预言逐条都中；`exit=1` 看 `MISMATCH` 行（预言错在哪一格）；`exit=2` 看 `PROBE-BROKEN` 行（装置坏）。`task-server/task log <号> --all` 里：`KEY` 行的 `observed`（第 2 问：copy-new 一臂 `stage-library-*`、`hosts-library-heavy-threads`、`detector-literal-set`、`new-stage-file` 为 unchanged、copy-base 一臂为 changed，`control-stage-in-table` 两臂都 changed，`preflight-library` 两臂都 unchanged）；`FLIP` 三行（判定前后不同）；`FLIP-SELFTEST` 两行（`exit=1`、`named-cell-lines` 大于 0）；`CLOSURE` 一行（copy-new 不改时 `whole-directory-via=0`、`refused=0`）与 `CLOSURE-DATA-FILE` 每行（对正文的 48 个）；`MUTANT-SELFTEST`（`exit=0` 即六格在变体上全绿，第 3 问）与 `MUTANT-CLOSURE`（`whole-directory-via` 或 `refused` 大于 0）。估时推的：26 次算键、2 次 closure、4 次共用库自证，每次算键的时长没量过；进度逐行追加在 `probes/progress.md`。装置改的每份文件都在 finally 里写回原字节并核 sha256。

## 没打中的形状（第 1 问）

试过、没有判定不同的：`stage` 里带 `/` 或 `..`（`GATE_STAGE_PATH` 那一组 `[^/]+\.sh` 挡住）；阶段名与 KNOWN 的名字撞（`lkmm.sh`、`crash-point-replay.sh`：位置不同，KNOWN 那一圈比的是整条位置，挑不到对方那一份）；KNOWN 里两个键指同一个值（没有，18 个值互不相同）；`HOOK_REPOSITORY` 被改绑（全仓没有）；闸第 938 行剥包装（词恒是 KNOWN 里的 `capped.sh` 或 `run-with-memory-cap.sh`）；自证里 `FAMILY_OWN_REPOSITORY` 换成样本工作区的几段（`same_content_as_own` 两版都只认 `HOOK_REPOSITORY`）；自证拷文件的名单（改前元组相加与改后 `PART_PROBE_COPIED_FILES` 是同样 8 个名字、同一次序）。取样范围是读码，覆盖了 `same_content_as_own` 的全部调用者；差分探针在副本上逐个输入核。

## 这条腿自己的限度

- 三问的结论都是读码推的。装置写好了、只过了 `python3 -m py_compile`，一条都没跑（子 agent 不交任务）；探针的预言写死在装置里（`PREDICTED`、`expected_difference`），跑出来对不上的逐条打 `MISMATCH`、`DIFF UNEXPECTED`，结果反过来我照它改结论：键探针里 copy-new 那一臂若 `stage-library-stage-cells` 为 changed，第 2 问第 1 类撤回（说明这几份库另有路进键）；`MUTANT-SELFTEST` 若退非 0，第 3 问撤回点名红的那一格。
- 库文件清单是手抄 `script_sourced_libraries` 的正则与递归算的（转述）。真函数在探针里现算；手抄与真函数若不一样，以探针的 `FLIP`、`LIBS` 行为准。
- 只算了两个入口的键（`watch_judgement` 与 `lib_project_code_runs_selftest.py::selftest`）。别的入口（项目代码闸自证这类）的闭包里有没有同样的读，没核。
- 主 agent 观测的 closure 原输出（48 个 data-file、改前的 7 个）我没看到，只读了背景材料的转述。
- 「读正文只多出 use 与 notice、不会把拒的变成放的」是读 `scan_uses` 与 `scan_decision` 推的，没有逐条证。
- 没建模型目录、没写 `rerun.sh`：派发提示把要跑的写成在草稿目录里交的命令，探针不碰崩溃状态，不走崩溃枚举。
- 两份副本是 `minimal-copy.py` 照闭包拷的，不是整仓；副本里不在的文件在两臂里一样缺，闭包与键只在两臂之间比、不与主工作区比。
- 开工之后主工作区里被判的五份有三份又被改过（不是这一改的 diff）：`lib_project_code_runs_base.py`、`lib_project_code_runs.py`、`lib_project_code_runs_selftest.py` 此刻的 sha256 与 `judged-sha256.txt` 不同，`diff` 草稿 `new/` 与此刻那一版，只差主 agent 名单里去掉 `gpu-verification-server/control serves` 与它的一格自证（base 第 850 行一带、入口第 208 行、selftest 第 951 行一带）。两份副本是开工那一刻的版本（`copy-new` 三份的 sha256 与 `judged-sha256.txt` 逐个相同）。报告里的行号照 `cite-check.py` 现判的此刻那一版写：selftest 第 951 行之后的两处（`PART_PROBE_COPIED_FILES` 那一格与拷文件那一行）在开工快照里各往后 2 行（1055、1059）。`.claude/gate.d/crash-point-replay.sh` 与 `.claude/hooks/bash_command_detector_scope_and_index.py` 也在我读过之后被改过，行号同样取此刻的。
- `scripts/agents/cite-check.py` 的 `@标题` 写法（`LOCATION` 正则里标题那一段是 `{2,80}?` 懒匹配、后面只跟一个可有可无的反引号）只取得到标题的头两个字，`.claude/rules/fs-design.md@分支的三条约束` 被判成标题「分支」命中 0 次；我改用行号引。这是算引文脚本的缺陷，不在这一轮写范围，交主 agent。

## 没做什么

- 不判正推腿那两问；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」）。
- 没改被判的五份文件，没改算键库；我提的三条改法（检出钩子只读那一份字面路径、本仓库文件的字面表加门禁格核、门禁格断言闭包里没有整目录与拒算）都被攻过零轮、没量过。
- 没删 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus/copy-new`：交主 agent 跑两条探针（217M，`.deliverables` 写成交接）。
- 没删 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r1-opus/copy-base`：交主 agent 跑两条探针（217M，`.deliverables` 写成交接）。
- 没建编译目录。草稿目录另有 `probes/`（两份探针，跑完再多出两份差分输出与 `progress.md`）与 `scratch/`（手抄的库清单脚本、它的输出、两份 `minimal-copy.py` 的输出）。
