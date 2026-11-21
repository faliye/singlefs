# 攻方腿报告：钩子共用库按名单读本仓文件，第二轮（hook-lib-foldable-reads-governance-r2，Opus）

日期：2026-11-21。攻击面：正文「要腿判的」第 1 到 3 问（第一轮改法 1 字面库表、改法 2 glob 列阶段、改法 3 新格与开关）。读的范围：被判的四份（`judged-sha256.txt` 四行与主工作区此刻逐个相同，`sha256sum` 核过）、工具员草稿 `hook-lib-keys-cover-libraries/` 的 `change.diff` 与 `result.md`、算键库 `scripts/gate/lib_function_units*.py` 的文件头与读法认法（只读）、三张表里 36 份脚本的 source 行、`scripts/gate/selftest-process-guard.py` 的 `watch_judgement` 与判定库那一段。没读禁读清单里的另一条腿的报告。第一轮攻过而没打中的三个角度（判定变化只在两处、派发起法走不到、三张字面表对得上）不重复。

## 复跑命令与文件

没建模型目录：这一轮的装置是草稿目录里的一条探针，不碰崩溃状态、不起 cargo、不读写判定库（派发提示点名的写法：`task-server/task submit --cwd <草稿目录> -- <命令>`）。四段命令在文末「要主 agent 跑的命令」。

草稿目录 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus/`：

| 文件 | sha256 | 做什么 |
|---|---|---|
| `probes/probe_r2.py` | `88b8682522f52afcfa2d40308de2b2240820b4c51242c0aee614fba390eb6a2f` | 四段：family（同族判定两臂）、keys（键与数据行，加自证红格）、switches（五条开关红的格对开关表）、mutants（八个变体下的自证与闭包） |
| `baselibs/lib_project_code_runs.py` | `61bd1bd31a62fd7907ac347f03e679ded34d620bb461df278262669900411071` | 第一轮修补之前那一版（拷自 `hook-lib-keys-cover-libraries/base/`），family 段修补前那一臂换上 |
| `baselibs/lib_project_code_runs_base.py` | `e7b0f29cfe6ff539ef7046be0f0b72f43f0cb037c8e47f527b6b32d59b283a85` | 同上 |
| `baselibs/lib_project_code_runs_launchers.py` | `0ed6842f76185b8374bc7eeb5094f0f17f15cdb985d843d2c5ee6a8a8630a67a` | 同上 |
| `baselibs/lib_project_code_runs_selftest.py` | `7bc78370226fdd5f3ce89cc9baaebd398c1fcc93825e7736b0cb6c32d2c3ff88` | 同上 |
| `scratch/source_lines_audit.py` | `3cded48ba5f13a2559bcf64511c70a878857219adef0c0abb0b3480426491944` | 我写的只读核对：手抄 `script_sourced_libraries` 的两条正则，对 36 份脚本与认出的库逐条标 source 行认没认出（不 import 本仓代码，直接跑过） |
| `scratch/sourced_libs_reimpl.py` | `640f6f6516a278ce1a98159d1740e802dee6fd9e2f9161b4e4441aba6fafe2de` | 第一轮攻方腿手抄的递归（原样拷来，上一份 import 它） |
| `scratch/audit.txt` | `df1d6b917a2787feb6f7bcb7a4b5e9a13f3ad28bd405b6acae3de939741b4c3c` | 上一份的输出：178 行 `SOURCE`、`LIB` 每份 `.claude/gate.d/lib/` 库一行 |
| `copy/` | 仓副本（`minimal-copy.py` 建，188M，`.deliverables` 写成交接） | 四段都在它上面改、跑、写回 |

探针改副本里的文件之前先存一份进 `probe-originals/`，finally 里写回并核 sha256；四段改同一份副本，要一段一段交。副本上量出的数只算线索。

## 各格判定一览

| 问 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| 1 字面库表与「从正文现算」那一格 | 成立（打中） | 比得到的 21 份都进键（推的）；可是 `script_sourced_libraries` 认不出最常见的「变量存脚本目录再 source」写法，36 份脚本里 70 条库的 source 行没被认出，25 份 `.claude/gate.d/lib/*-cells.sh`、`peer-host-lib.sh`、`run-build-directory.sh` 等从来不比：别处拷贝改了这几份库照判同族、不读正文，违背 `judged_by_name` 文档串；新格和被判的代码共用同一个识别函数，看不见这一类。另一处：那一格靠 `os.path.lexists` 认「盘上有的」，存在性不进键：先加 source 行、后建库的两步里，第二步之后那一格该红而键不变，复用旧绿；同一状态下闸把逐字节相同的拷贝判成不同（比修补前严） | source 行是量的（`scratch/audit.txt`，直接跑的只读脚本）；判定与键待 family、keys 两段 |
| 2 glob 列阶段 | 「加一道阶段键会不会变」不成立；「非顶层、非 .sh 的文件」成立（打中） | 加一道阶段：按算键库 `pattern_listed_files` 的认法键会变（推的），核法是 keys 段 `new-stage-file`（另有对照 `new-top-non-shell-file`）。`.claude/gate.d/` 下非顶层的东西有两样影响自证判定而不进键：`fixtures/crates-mutation-replay.sh/` 这个目录在不在（`os.path.isdir`），决定「要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档…」那一格；登记表点名的 `lib/rejection-blocks.py`、`lib/broad-mutation-samples.py` 正文里有没有临时服务记号，决定「要临时服务：仓里的表…对得上」那一格（后一样进不进键是推的，keys 段量） | 读码推的；keys 段待跑 |
| 3 新格与开关 | 成立（打中） | 新格与两条新开关红在名字说的事上（推的，switches 段核）；打中的是没开关的那一格：「OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本按自己所在目录引的库…逐个相同」不挂任何开关、没有一次证明它会红，名字说的是「每份脚本按自己所在目录引的库」，实际核的只是表与识别函数同步，第 1 问那 70 条它照绿；`own-script-digest-location-joined` 只在自证里换掉调用，`own_library_digest` 同形的「查表命中拼路径读」变体没有开关（mutants 段对两种真代码变体各量一次）。「拷文件改回元组相加」在自证里分得出：读自己源码的语法树，核拷文件那一圈迭代的是 `PART_PROBE_COPIED_FILES` 这个名字、它只赋一次、是字面串元组；探针里写了这一格，并对两种「元组相加」变体（改循环、改常量）各量一次 | mutants、switches 两段待跑 |

## 第 1 问：字面库表与「从正文现算」那一格

### 比得到的那 21 份：都在键里（推的）

`same_content_as_own` 读本仓那一边只经两处：脚本本身「if digest is None or digest != own_script_digest(location):」（`.claude/hooks/lib_project_code_runs_launchers.py:1370`），库「if os.path.lexists(copy_library) and file_sha256(copy_library) != own_library_digest(os.path.normpath(os.path.join(os.path.dirname(location), relative))):」（`.claude/hooks/lib_project_code_runs_launchers.py:1374`）。两个函数都按相等从同模块字面元组里挑、拿表里的字面拼路径读，算键库折得出（工具员推的，主 agent 换上之后那两条 closure 的 data-file 里有 `stage-cells.sh`、`heavy-threads.sh`、`preflight.sh`，是正文转述的读数，我没看到原输出）。多层 source 与 `../` 链：递归交回的是相对入口所在目录的路径、`os.path.normpath` 过，`../scripts/preflight.sh` 落到 `.claude/scripts/preflight.sh`，表里有；被比的库没有一份因为这两种写法掉出键。keys 段 `table-library-stage-cells-changed` 一格是对照（预言两个入口的键都变）。

### 打中一：「变量存脚本目录再 source」的库一份都不比

识别函数只认两种写法：`source "$(dirname "${BASH_SOURCE[0]}")/…"` 一类，与 `source "$变量/…"` 而那个变量的赋值是「SCRIPT_DIRECTORY_VARIABLE = re.compile(r'^\s*(?:local\s+|readonly\s+|declare\s+(?:-\w+\s+)?)?([A-Za-z_][A-Za-z0-9_]*)=\S*(?:BASH_SOURCE|dirname\s+"?\$0)', re.MULTILINE)」（`.claude/hooks/lib_project_code_runs_launchers.py:958`）认得出的。`=\S*` 之后紧跟 `BASH_SOURCE` 或 `dirname "$0`，中间不许有空白；仓里通行的 `"$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"` 在 `cd` 后面就有空格，认不出。我把这条正则原样抄进 `python3 -I -c` 对四种赋值逐个试（量的，原样输出）：

```
'STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"' []
'STAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"' []
'X="$(dirname "${BASH_SOURCE[0]}")"' []
'X=$(dirname "$0")' ['X']
```

认不出的变量起头的 source 行整行跳过，库不进 `script_sourced_libraries`，`same_content_as_own` 只在「for relative in script_sourced_libraries(path):」（`.claude/hooks/lib_project_code_runs_launchers.py:1372`）这一圈里比，所以这些库从来不比，别处那边改了照判同族。函数文档串说的是「盘上有这份文件、又不是本仓那一份（路径尾部相同的拷贝、经符号链接起的、内容连同它 source / import 的库与本仓那一份不逐字节相同的）为假、照读正文；」（`.claude/hooks/lib_project_code_runs_launchers.py:1319`）。

覆盖面（量的，`scratch/source_lines_audit.py` 直接跑在主工作区此刻那一版上，`scratch/unrecognized-library-sources.txt` 70 行）：36 份脚本与认出的库里，以 `.sh` 收尾的库 source 行有 70 条认不出。例：

- 门禁阶段：「STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"」（`.claude/gate.d/doc-text.sh:1099`）之后「source "$STAGE_DIR/lib/stage-cells.sh"」（`.claude/gate.d/doc-text.sh:1101`）与「source "$STAGE_DIR/lib/doc-text-content-and-history-cells.sh"」（`.claude/gate.d/doc-text.sh:1474`）。`.claude/gate.d/lib/` 下 25 份 `*-cells.sh` 与 `code-tooling-shared-libraries.sh` 全靠这种写法 source、一份都不在表里（`scratch/audit.txt` 的 `LIB … NOT-in-own-table sourced-by=.claude/gate.d/…` 25 行）；`lib/stage-cells.sh` 有 11 道阶段这样 source（认得出的只有 6 道），第一轮修补收进键的那一份，对这 11 道照样不比。
- 派发起法：「source "$multi_host_script_directory/peer-host-lib.sh"」（`scripts/hosts/multi-host-run.sh:411`），变量在第 409 行照同一写法赋。`--peer` 在第二台跑的就是这份库里的函数，它不比，`dispatched_under_memory_cap` 对改过它的拷贝照算经内存包装。
- 内存包装：「SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"」（`scripts/hosts/run-with-memory-cap.sh:393`）之后第 394 行 source `heavy-threads.sh`。同一份库，`capped.sh` 用认得出的写法 source（第一轮已量过它翻面），`run-with-memory-cap.sh` 这一边不比。
- 其余：`run-build-directory.sh`（gate-staged.sh、三份崩溃放量、crash-point-replay.sh、replay.sh）、`crash-amplification-lib.sh`、`replay.sh` 的八份库、`mutation-shard-run.sh` 与 `build-patched-qemu.sh` 的 `../hosts/heavy-threads.sh`、`changed-paths.sh`。

能照喂的输入（family 段，每一条都是「入口与认出的库、另列的库都从副本原样拷进一棵树，再给点名的那份末尾加一行注释」，两臂预言相同）：

| 名字 | 调 | 照文档串应当是 | 预言 |
|---|---|---|---|
| `doc-text-stale-stage-cells` | `judged_by_name(".claude/gate.d/doc-text.sh", <树>)` | False | True |
| `doc-text-stale-content-and-history-cells` | 同上 | False | True |
| `code-tooling-stale-selftest-cells` | `judged_by_name(".claude/gate.d/code-tooling.sh", <树>)` | False | True |
| `multi-host-run-stale-peer-host-lib` | `dispatched_under_memory_cap([<树>/scripts/hosts/multi-host-run.sh, "--peer", "--", "cargo", "test"], <树>)` | False | True |
| `run-with-memory-cap-stale-heavy-threads` | `existing_copy_elsewhere(<树>/scripts/hosts/run-with-memory-cap.sh, <树>)` | True | False |
| 对照 `doc-text-stale-preflight`、`multi-host-run-stale-preflight`、`capped-stale-heavy-threads`、`crash-point-replay-stale-stage-cells` | 认得出的库改过 | 照文档串 | 照文档串 |

键那一边：本仓这几份库判定里不读，所以不是键漏，是判定漏比（keys 段 `unrecognized-doc-text-library-changed` 预言两个入口的键都不变，判定也不变，对得上）。

打中之后的四句：分不分辨臂：不分辨，修补前后两臂一样漏（family 段两臂预言相同），病根在两臂共用的识别函数；第一轮修补的表是照它现算的，修不到这一类。系统当时看不看得到判别它的东西：看得到，正文就在手里，同一份共用库里另有拆分族的认法认得出 `lib/<阶段>-<部分>.sh`（「门禁阶段入口的部分文件在 .claude/gate.d/lib/ 下、不在入口的同目录里，由调用方给候选。」（`.claude/hooks/lib_project_code_runs_launchers.py:334`）），只用在临时服务记号上。满足的分句：第 1 问「多层 source、`../` 链、变量拼的 source 路径」里的「变量拼的」，与 `judged_by_name` 文档串「连同它 source / import 的库」。改法在打中的格上还中不中：第一轮改法 1（字面库表加一格从正文现算）不中，表与那一格都照同一个识别函数算；「让算键库拒算」也不中（本仓那一边根本不读，没有可拒的读）。

我提的改法（只在读码上推的、没量过、被攻过零轮）：识别变量那条正则认 `"$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"`、`"$(dirname "${BASH_SOURCE[0]}")"` 这几种赋值；`"$X/../.."` 这类再往上的变量不当脚本目录认，source 它的那一行照认不出办。表随之补齐（推的：至少 25 份门禁库加 `peer-host-lib.sh` 等）。另加一格不依赖识别函数：对表里每份脚本列出正文里每一条 `source` / `.` 行，参数里以 `.sh` 收尾、能在仓里找到的，逐条要求要么在识别函数的结果里，要么登进一张点名的「认不出也不比」表，两边都不在就红。

### 打中二：先加 source 行、后建库，第二步之后那一格该红而键不变

那一格只收「盘上有的」：「if os.path.lexists(os.path.join(HOOK_REPOSITORY, sourced_library)):」（`.claude/hooks/lib_project_code_runs_selftest.py:961`）。算键库认的读只有「读文件的写法：open / io.open / codecs.open（模式里没有 w a x +）、<路径>.read_text() / .read_bytes()、Path(…).open()、shutil.copy / copy2 / copyfile 的源。」（`scripts/gate/lib_function_units.py:129`），`lexists` 不算读，那几份不存在的路径也不是哪一处读的 data-file。

历史（keys 段与 family 段 two-step 照它造）：

1. 往 `crash-point-replay.sh` 末尾加一行 `source "$(dirname "${BASH_SOURCE[0]}")/lib/probe-new-library.sh"`，库还没建。`crash-point-replay.sh` 在键里，两个入口的键都变（预言 `changed`）；自证跑一次，那一格两边都没有这份库，绿，记库。
2. 再建 `lib/probe-new-library.sh`，`OWN_SCRIPT_LIBRARY_FILES` 不补。新库不在表里，`own_library_digest` 不读它，那一格的 `lexists` 不进键：两个入口的键与第 1 步相同（预言 `same-as:source-line-added`），复用第 1 步的绿；现判的话那一格红（预言多红一格，前缀「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本按自己所在目录引的库」）。
3. 同一状态下，别处一棵与本仓逐字节相同的拷贝（连新库）：此刻那一臂 `judged_by_name` 交 False（表外的库照不同认），修补前那一臂交 True（预言，family 段 `crash-point-replay-new-library-identical`）。这就是正文问的「此前该判同族的判成不同」：拷贝照读正文，正文里有闸认不出的写法时那一条被拒（拒不拒是推的，没逐条证）。

`lib/lib/stage-cells.sh` 是同一形的现成例：`stage-cells.sh` 文件头第 15 行的注释被正则认成一条 source，路径盘上没有。建一份空文件，自证那一格该红、键不变（keys 段 `lib-lib-stage-cells-created`）。

四句：分辨臂：分辨（修补前那一臂比的是本仓那一份，第 3 步交 True；此刻交 False）；系统看得到：那一格自己就在 `lexists`，只是这一步不进键；满足的分句：第 1 问「那一格本身读 36 份脚本正文不收也不拒算，脚本改了加一行 source 时，那一格的判定会不会照旧复用」，加一行 source 那一步键会变，后建库那一步才复用；改法：第一轮改法 1 碰不到（它造出了这一格）。我提的改法（零轮）：那一格不用 `lexists` 筛，识别函数认出的每一条都要么在表里、要么登进「认出了而盘上没有」那张点名表，盘上出现了就必须从那张表挪进库表；这样「盘上有没有」不再改变那一格的判定，也就不用进键。

### 表外的库照不同认：此刻盘上没有判反的（推的）

识别函数认得出、本仓盘上有的库都在表里（主 agent 换上之后共用库自证退 0）。认得出而本仓盘上没有的（`lib/lib/stage-cells.sh`），修补前比的是读不到的那一份（`file_sha256` 交 None），此刻是表外交 None，两臂都 False（family 段 `crash-point-replay-copy-only-lib-lib`）。所以只在上面第 2 步那种「盘上有、表里没有」的状态下两臂才分开。Python 同目录 import 走的是另一条正则，判在不在用的是拷贝那一边的 `os.path.isfile`，表里 11 份 `scripts/mutation/` 库照它现算；存在性同样不进键，形与打中二相同，没单独造。

## 第 2 问：glob 列阶段

### 加一道阶段键会不会变：会变（推的），核法

读法是「for listed_stage_path in sorted(glob.glob(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "*.sh"))):」（`.claude/hooks/lib_project_code_runs_selftest.py:924`），每一道再拿 `os.path.basename` 拼回门禁目录读 sha256。算键库对这一形的认法是「→ 目录里名字合模式的那几份普通文件逐份收成 data-file，不收整个目录（读的是 <目录>/<文件名>，文件名合模式；在哪个目录里列的、列了几层都不改这一点）。」（`scripts/gate/lib_function_units.py:150`），列目录是收闭包那一刻现列的，所以多一份 `*.sh` 就多一个 data-file，键变。

核法（只用 key / closure，在我自己的副本里；keys 段照它跑）：

1. 副本里 `python3 scripts/gate/lib_function_units.py key <副本> .claude/hooks/lib_project_code_runs_selftest.py::selftest --judging`，记下 `key` 行与全部 `data` 行（拒算退 4 时改跑 `closure` 取 `data` 行）。`scripts/gate/selftest-process-guard.py::watch_judgement` 同样记一次。
2. 副本里建 `.claude/gate.d/probe-stage.sh`（两行，不进 `GATE_STAGE_FILES`）。
3. 同两条命令再跑；比 `key` 与 `data` 行。预言：两个入口都 `changed`，多出的数据行是 `data:.claude/gate.d/probe-stage.sh`（`watch_judgement` 也变，是推的：正文转述两条闭包都是 69 个 data-file，我推它连着 `selftest()`）；自证多红两格（glob 那一格与「按位置都挑得到」那一格）。
4. 对照：建 `.claude/gate.d/probe-notes.txt`，预言键不变（不合 `*.sh`）。

结果反过来（`new-stage-file` 键不变）就是打中：说明 `pattern_listed_files` 没认出这一圈（三参数的 `os.path.join` 或拼回去那一步），第 2 问「加一道阶段键会不会变」那一半改判成立。

### 非顶层、非 `.sh`：两样影响自证判定而不进键（打中）

1. `fixtures/<阶段>/` 这个目录在不在。自证「要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法」（`.claude/hooks/lib_project_code_runs_selftest.py:347`）拿主仓门禁目录喂 `temporary_service_bare_stage_samples`，它经 `stage_selftest_fed_stages` 只留「if stage.endswith(".sh") and not stage.startswith(".") and os.path.isdir(os.path.join(gate_directory, "fixtures", stage))」（`.claude/hooks/lib_project_code_runs_base.py:735`）的阶段。`isdir` 不是读，门禁目录又是从命令词解出来的（`stage_selftest_directory`），不进键。把 `.claude/gate.d/fixtures/crates-mutation-replay.sh/` 挪走（或改名），那一格从 `(True, True)` 变成 `(False, False)`、红，键不变（keys 段 `crates-mutation-replay-fixtures-moved-away`，预言键 `unchanged`、多红这一格）。改前整目录数据单元罩着门禁目录的递归列表（第一轮攻方腿读码推的），这一处是那一改掉出来的。
2. 登记表点名的门禁库正文。「要临时服务：仓里的表（…）与自证登记表、脚本源码对得上」那一格调 `temporary_service_table_problems`，它对登记表每一行「with open(os.path.join(root, key), encoding="utf-8", errors="replace") as handle:」（`.claude/hooks/lib_project_code_runs_launchers.py:375`），登记表里有「.claude/gate.d/lib/rejection-blocks.py	selftest	# 没有自证入口，直接执行只读进定义、什么都不跑」（`.claude/hooks/selftest-registry.tsv:228`）与 `lib/broad-mutation-samples.py`。往其中一份加一行带临时服务记号的注释，那一格该红（登记了、源码里有记号、要临时服务的表里没有这一行）。这一处读的路径是「根 + 读登记表得来的一截」，算键库归哪一类我推不出（主 agent 量到 refused 0，所以不是拒算；收没收要看它对 `root` 参数的追法），keys 段 `registered-gate-library-gets-service-mark` 量：键不变就是漏。同一组文件还有存在性：「if not os.path.isfile(os.path.join(root, key)) and not break_is_set("selftest-registry-lenient"):」（`.claude/hooks/lib_project_code_runs_base.py:481`）判「仓里的登记表…都在」那一格，删掉 `lib/rejection-blocks.py` 那一格红；它不是哪一处读的 data-file 时，键同样不变（没单独造）。

`watch_judgement` 那一边：门禁样本档那一支本来就不按键复用（「return None, "stage-selftest.sh 喂的样本档：喂进来的样本不在单元闭包里，键罩不住它"」（`scripts/gate/selftest-process-guard.py:803`）），所以样本档目录那一样只漏在共用库自证的复用上，不漏在被盯命令的复用上。

四句：分辨臂：样本档目录那一样分辨（改前整目录单元罩着，改后不罩）；登记表点名的门禁库那一样推不出。系统看得到：看得到，读法就在自证里。满足的分句：第 2 问「`.claude/gate.d/` 下非顶层、非 `.sh` 的文件有没有影响判定却不进键」。改法：第一轮改法 2（glob 列阶段逐份读）只管顶层 `*.sh`，碰不到这两样；我提的（零轮）：那一格的样本档目录改成从 `GATE_STAGE_FILES` 与字面 `fixtures/<阶段>/expect` 一类文件读（读得到就进键，不靠 `isdir`），或者那几格拿自证自己造的门禁目录喂、不喂主仓那一份。

## 第 3 问：新格与开关

### 逐格

| 格（自证里的名字） | 名字说的 | 修补前代码上 | 点名它的开关（文件头） | 我的判法 |
|---|---|---|---|---|
| 「同族：连同 source 的库都与本仓那一份逐字节相同的门禁阶段拷贝照按名字判（crash-point-replay.sh 与它 lib/ 下的库）」（`.claude/hooks/lib_project_code_runs_selftest.py:914`） | 行为：连库相同照按名字判 | 绿（比的是本仓那一份） | `family-copy-content-unchecked`、`own-script-digest-stages-unlisted`、`own-library-digest-unlisted` | 红在名字说的事上；它是行为护栏，`own_library_digest` 改成「查表命中拼路径读」照绿（mutants 段 `own-library-digest-location-joined` 预言只红 EqualLocation 那一格） |
| 「同族：source 的库与本仓那一份不同的门禁阶段拷贝不按名字判（读正文）」（`.claude/hooks/lib_project_code_runs_selftest.py:916`） | 行为：库不同读正文 | 绿 | `same-family-by-path-tail` | 红在名字说的事上 |
| 「同族：门禁目录顶层列出来的阶段与 GATE_STAGE_FILES 逐个相同、每一道从表里挑出的 sha256 与盘上那一份相同」（`.claude/hooks/lib_project_code_runs_selftest.py:930`） | 表与盘上逐个相同 | 能跑、绿 | `own-script-digest-stages-unlisted` | 「逐个相同」红在名字说的事上（新阶段那一刻键也变，第 2 问）；「sha256 与盘上那一份相同」比的是同一路径读两次，只在挑不到时红，对「拼路径读」的变体照绿 |
| 「同族：本仓那一份照表里的字面路径读，不拿传进来的位置拼（与表里那一项相等、字面指到别处的位置照样读到本仓那一份）」（`.claude/hooks/lib_project_code_runs_selftest.py:951`） | 读法：照表里的字面读 | 脚本那一半能跑、绿（第一轮修补就是这样读的）；库那一半没有函数 | `own-script-digest-location-joined`、`own-library-digest-unlisted`、`own-script-digest-stages-unlisted` | 红在名字说的事上；`own-script-digest-location-joined` 只把自证里那一次调用换成「def location_joined_digest(location):」（`.claude/hooks/lib_project_code_runs_selftest.py:164`），证的是这一格对一种写法判得出，不是真代码；真代码的两种变体由 mutants 段 `own-script-digest-location-joined`、`own-library-digest-location-joined` 各量一次（预言都只红这一格） |
| 「同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本按自己所在目录引的库（盘上有的）逐个相同」（`.claude/hooks/lib_project_code_runs_selftest.py:963`） | 表收全了每份脚本按自己所在目录引的库 | 没有这张表 | 无 | 打中：没有开关证过它会红（mutants 段 `library-table-drops-preflight` 补一次，预言红它与「连同 source 的库都…逐字节相同」那一格）；名字说「每份脚本按自己所在目录引的库」，实际核的是表与识别函数同步，第 1 问 70 条认不出的它照绿；还有打中二那一处存在性不进键 |
| 「认得出：起 claude 的命令词与检出钩子 bash_command_detector_scope_and_index.py 的 CLAUDE_EXECUTABLE_PATH_WORDS 同一组」 | 两边同一组 | 能跑、绿 | 无（第一轮也没有） | 只读那一份字面路径，常量挪去别的部分文件时交 None、红；没开关，不在这一轮新加的范围 |

两条新开关点名的格与实际红的格逐格对：switches 段对五条开关各跑一次自证，与基线比多红的格，和文件头这一条下面「…」里的格名比，打 `red-but-unnamed` 与 `named-but-not-red`（预言两样都空）。这一段不经门禁 code-tooling，不读写判定库。

### 「拷文件改回元组相加」：自证里分得出（推的，mutants 段量）

运行时分不出：改前「元组相加」与改后 `PART_PROBE_COPIED_FILES` 是同样 8 个名字、同一次序（第一轮攻方腿与工具员都核过）。能分的是源码：自证按「for copied_name in PART_PROBE_COPIED_FILES:」（`.claude/hooks/lib_project_code_runs_selftest.py:1134`）所在的那份文件，读自己（`os.path.abspath(__file__)`，工具员推的折得出）、`ast.parse`，核两件：

- 每一处 `for copied_name in …` 的迭代对象是名字 `PART_PROBE_COPIED_FILES`（改循环那一种变体：迭代对象成了 `(…) + PROJECT_CODE_RUNS_PART_NAMES`，红）；
- `PART_PROBE_COPIED_FILES` 在模块顶层只赋一次、值是全由字面串组成的元组（改常量那一种变体：循环照写名字，常量写成 `(…, PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME) + PROJECT_CODE_RUNS_PART_NAMES`，只查循环的格照绿、算键库照样折不出，这一条才红）。

探针里这一格的原文在 `probes/probe_r2.py` 的 `DISCRIMINATOR_CODE`，格名「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代，它在模块里只赋一次、是字面串的元组（算键库折得出拷的是哪几份）」。mutants 段五行：两种变体单独套（预言自证不多红，说明现有自证分不出）、只加这一格（预言不多红、闭包拒算行数与基线相同）、这一格加两种变体各一次（预言只多红这一格）。

打中之后的四句：分辨臂：分辨（有这一格与没有，在两种变体上判法不同）；系统看得到：看得到，判别它的是自证自己的源码；满足的分句：第 3 问「有没有一种办法在共用库自证里分得出（不碰门禁 code-tooling）」答「有」；改法：工具员报告写的「只能靠闭包看」不成立，至少这一半不用。这一格的限度：按循环变量名 `copied_name` 找循环，换个变量名就躲开，稳的写法是找循环体里 `shutil.copy` 源路径用到的那个循环；同一套做法也能钉 `own_script_digest`、`own_library_digest` 只迭代本模块的字面元组（零轮，没写）。

## 要主 agent 跑的命令

四条改同一份副本，一条出了结局再交下一条（次序不限）。都不编译、不起 cargo、不设崩溃域、不读写判定库（自证在副本里直接起，不经 `selftest-process-guard.py`；算键只用 `lib_function_units.py key` / `closure`）。探针取共用库函数用的是入口模块再导出的名字（`lib_project_code_runs.judged_by_name` 这类），不经 `sys.modules`。每一步的耗时逐行追加在 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus/progress.md`。

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus/probes/probe_r2.py /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus family`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus`（即 `--cwd`）。
   - 看：末行 `TASK_SERVER_RESULT` 的 `exit=0` 是预言逐条都中，`exit=1` 看 `task-server/task log <号> --all` 里以 `MISMATCH` 收尾的 `FAMILY` 行，`exit=2` 看 `PROBE-BROKEN`。四行 `FAMILY-ARM … exit=0|1`；`FAMILY-RECOGNIZED` 每行是真函数认出的库（第 1 问打中一的证据：`doc-text-*` 那几行里没有 `lib/doc-text-*-cells.sh` 与 `lib/stage-cells.sh`）；`FAMILY` 行的 `got=` 与 `spec=` 不同的就是判定漏比（预言五行）；`two-step` 两行里 `crash-point-replay-new-library-identical` 此刻那一臂 `got=False`、修补前那一臂 `got=True`（打中二）。估时推的：两臂各载一次共用库，几十次函数调用，几十秒。
2. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus/probes/probe_r2.py /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus keys`
   - 在哪起：同上。
   - 看：`exit=0|1|2` 同上。`KEY-BASELINE` 两行（`refused=` 为 0 才有 `key=`；不为 0 时比的是数据行）；`KEY` 行逐个场景 `got=changed|unchanged|same-as:…`；`KEY-SCENARIO` 行的 `added=` / `removed=` 是多出、少了的数据行；`SELFTEST-SCENARIO` 行的 `extra=` 是比基线多红的格。第 2 问前半看 `new-stage-file`（预言两个入口 `changed`、`added=data:.claude/gate.d/probe-stage.sh`）与 `new-top-non-shell-file`（预言 `unchanged`）；后半看 `crates-mutation-replay-fixtures-moved-away` 与 `registered-gate-library-gets-service-mark`（预言键 `unchanged`、自证多红一格：这两行预言中就是键漏）；第 1 问打中二看 `source-line-added-then-library-created`（预言 `same-as:source-line-added`、多红表那一格）与 `lib-lib-stage-cells-created`。估时推的：17 次算键、8 次共用库自证，每次算键的时长没量过（第一轮主 agent 那两条 closure 的时长我没看到）。
3. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus/probes/probe_r2.py /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus switches`
   - 在哪起：同上。
   - 看：五行 `SWITCH`，`red-but-unnamed=-` 与 `named-but-not-red=-` 是那一条开关红的格与文件头点名的逐格相同；`SWITCH-RED` 逐格列。估时推的：6 次共用库自证。
4. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus -- python3 /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus/probes/probe_r2.py /home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus mutants`
   - 在哪起：同上。
   - 看：`MUTANT-BASELINE-CLOSURE` 一行；每个变体一行 `MUTANT-SELFTEST`（`extra=` 比基线多红的格）、一行 `MUTANT-CLOSURE`（`refused=`、表里的库还剩几份在数据行里、多出少了的数据行）。第 3 问看：`copy-loop-tuple-addition`、`copied-constant-tuple-addition` 预言 `extra=-`（现有自证分不出）；`discriminator-cell-only` 预言 `extra=-` 且 `refused=` 与基线相同；两条 `discriminator-cell-with-…` 预言 `extra=` 只有「引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代…」；`own-script-digest-location-joined`、`own-library-digest-location-joined` 预言只红 EqualLocation 那一格；`library-table-drops-preflight` 预言红表那一格与「连同 source 的库都…逐字节相同」那一格。估时推的：9 次共用库自证、9 次算键。

结果反过来我照它改结论：family 段打中一那五行若 `got=` 等于 `spec=`，打中一撤回（说明另有路比到这几份库）；keys 段 `source-line-added-then-library-created` 若 `differs`，打中二撤回；`crates-mutation-replay-fixtures-moved-away` 若 `changed`，第 2 问后半那一样撤回；mutants 段 `discriminator-cell-with-…` 两行若不红，第 3 问「分得出」撤回。

## 没打中的形状

- 多层 source 与 `../` 链：递归与 `normpath` 对得上，被比的库没有一份因此掉出键（读码；真函数认出的库待 family 段的 `FAMILY-RECOGNIZED` 行）。认得出的 45 行库 source 全是参数里直接写 `$(dirname …)` 的那一种（`grep '^SOURCE' scratch/audit.txt | grep -v UNRECOGNIZED` 那 45 行），没有一行经「认成脚本目录、其实是往上两层」的变量，所以此刻没有解错路径的。
- 「表外的库照不同认」在此刻盘上：两臂判定相同（读码推的，family 段 `plain` 那一段核）；只在「盘上有、表里没有」的状态下分开（打中二）。
- EqualLocation 那一格对「查表命中拼路径读」：子类重载的反射比较先走、拼出的是 `<草稿>/equal-location-elsewhere/<位置>` 的绝对路径、读不到交 None，判得出（读码；mutants 段量真代码）。
- 检出钩子那一格：只读一份字面路径，常量挪走交 None、红，没找到绕过的写法。
- 新阶段文件做成符号链接：`pattern_listed_files` 照 `os.path.isfile` 跟链接认，照收。

## 这条腿自己的限度

- 判定与键的结论都是读码推的，探针只过了 `python3 -m py_compile`，一段都没跑（子 agent 不交任务）。source 行的计数是量的，量的是我手抄的正则（转述，`scratch/unrecognized-library-sources.txt` sha256 `e15cbbde55f1d1bba3780a9ea5d4634e58eef9ff56b8b494fbf0b7c6597a34fe`），真函数的结果以 family 段的 `FAMILY-RECOGNIZED` 为准。
- 副本是 `minimal-copy.py` 按闭包加声明拷的，不是整仓；副本里共用库自证的基线就有红的格（登记表点名的文件有的不在副本里），探针只比「比基线多红、少红」的格，不看退出码。
- 算键只算了两个入口（`lib_project_code_runs_selftest.py::selftest` 与 `watch_judgement`），没算被盯的那条命令自己的入口（`.claude/hooks/lib_project_code_runs.py` 整份）；`watch_judgement` 连着 `selftest()` 是从正文转述的两个 69 推的。
- 「判成不同之后读正文、拒不拒」没逐条证。
- 递归深度 `SOURCED_LIBRARY_DEPTH = 4` 有没有在哪份脚本上截断，没核。
- 第 2 问第二样（登记表点名的门禁库）算键库归哪一类，我推不出，等 keys 段那一行。

## 没做什么

- 不判辩方那两问；不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没往统一任务服务交任务，判决要引的数没复跑（命令在「要主 agent 跑的命令」）。
- 没改被判的四份，没改算键库；我提的改法（识别变量的正则放宽加一格独立的 source 行核对、库表那一格不用 `lexists` 筛、样本档那几格不靠 `isdir`、拷文件那一圈的语法树格）都只在读码上推、被攻过零轮。
- 没建模型目录与 `rerun.sh`：探针不碰崩溃状态，派发提示把要跑的写成在草稿目录里交的命令。
- 没删 `/home/fy5090/.local/share/singlefs/work/hook-lib-foldable-reads-governance-r2-opus/copy`：主 agent 要在里面跑四段探针（188M，`.deliverables` 写成交接）。
