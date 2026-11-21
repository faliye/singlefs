# 治理文件轮第一轮正推腿报告（split-governance-hooks-gates-r1，第 5 到 7 问）

日期：2026-11-20。读的是主工作区此刻那一版；拆前原文用 `git show HEAD:<入口路径>` 读（草稿目录里的 `head-*.sh` 是它们的拷贝）。方案按 `crates/` 今天的实现来谈：这三问判的文件里没有 `crates/` 下的代码，与 `crates/` 的交点只在两件修补读的拆分族（`.claude/hooks/` 下）与格库读 `crates/*/Cargo.toml`，这一腿没有碰。

## 读到的范围与一处与开工快照不一致

- 读了 `research/prompts/split-governance-hooks-gates-r1-snapshot/judged-sha256.txt` 列的 127 份里与第 5 到 7 问有关的：`.claude/hooks/lib_project_code_runs.py`、`lib_project_code_runs_launchers.py`、`lib_project_code_runs_selftest.py`、`lib_shell_words*.py`、`project-code-guard.sh`、`project-code-guard-selftest.py`（只读相关段）、`.claude/gate.d/` 八道入口、`lib/` 下 16 份格库里与格函数登记有关的段、`lib/code-tooling-rules-and-agents-cells.sh`、`fixtures/code-tooling.sh/agent-write-scope-*`。另读了不在快照名单里的 `lib/stage-cells.sh`（`--list` 的实现）、`scripts/gate/gate-structure-check.py` 文件头、`fixtures/code-tooling.sh/agent-write-scope-green/expect`。只有这一腿读到的范围就是这些；别的文件没读。
- 命令：`sha256sum -c research/prompts/split-governance-hooks-gates-r1-snapshot/judged-sha256.txt 2>&1 | grep -v ': OK'` 的原样输出：

```
.claude/gate.d/break-switch-replay-files.tsv: FAILED
sha256sum: WARNING: 1 computed checksum did NOT match
```

  127 份里 126 份与快照一致，这一份在快照之后又改过。`git diff -- .claude/gate.d/break-switch-replay-files.tsv`（工作区对暂存区）改到三行：`scripts/gate/apply-writer-patch.py` 那一行（不属两件修补）、`.claude/hooks/lib_project_code_runs.py` 与 `.claude/hooks/project-code-guard.sh` 两行（两件修补之一的条数注释）。第 5 问引用的两行以此刻工作区为准。

## 怎么核的，什么没能核

- 只读核：读代码与文件头、`grep -nF`、`git show HEAD:`，加草稿目录里八份只读的复刻脚本（`/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-sonnet/` 下，脚本里的正则抄自被判文件，抄的是读到的原文；它们是核对口径的复刻，不是被判代码本身的运行）。脚本与 sha256 在「复刻脚本」一节。
- 不能直接跑：两份自证（`lib_project_code_runs.py --selftest`、`project-code-guard.sh --selftest`）、开关复跑、门禁 code-tooling 的 `agent-write-scope`、`gate-structure` 两格、样本档、八道门禁的 `--list`。这些归主 agent，命令在「要主 agent 跑的命令」一节；对应的格在「判定一览」里写「待主 agent 跑」。

## 第 5 问：两件修补做的是不是「实现今天的样子」那一条说的事

### 5a 项目代码闸改认拆分族：成立

正文那一条（`research/prompts/_split-governance-hooks-gates-r1-body.md:18`）原文：「项目代码闸：`.claude/hooks/lib_project_code_runs_launchers.py` 加 `source_family`（入口加它 source、import、按文件名载入的同目录文件，四层），要临时服务的登记行「源码里有核对」认整族；新开关 `temporary-service-mark-entry-only`、`temporary-service-family-by-directory`。换上后 `lib_project_code_runs.py --selftest`、`project-code-guard.sh --selftest` 都绿，开关复跑 76 条、151 条逐格相同。」逐句核：

1. 「加 `source_family`」：成立。`.claude/hooks/lib_project_code_runs_launchers.py:299`：「def source_family(path):」；点名同目录文件的 `source_family_names` 在 `.claude/hooks/lib_project_code_runs_launchers.py:284`：「def source_family_names(path):」。
2. 「入口加它 source、import、按文件名载入的同目录文件」：成立。`:284` 起的 `source_family_names`：shell 的 `source` / `.` 后写死的 `<名>.sh`（`SOURCE_FAMILY_SHELL_SOURCE`，`:281`），python 的 `import <模块>`、`from <模块> import` 与引号里整个写死的 `<名>.py` / `<名>.sh`（复用 `PYTHON_SIBLING_REFERENCE`，定义在 `:926`）。「按文件名载入」对上的真实写法是 `lib_project_code_runs.py:1545`：`PROJECT_CODE_RUNS_PART_NAMES = ("lib_project_code_runs_launchers.py", "lib_project_code_runs_selftest.py",)`，引号里的整名 `.py` 被第三个捕获组认出。
3. 「四层」：成立但「四层」指入口之后再往下跟四层。`.claude/hooks/lib_project_code_runs_launchers.py:927`：「SOURCED_LIBRARY_DEPTH = 4」；`.claude/hooks/lib_project_code_runs_launchers.py:310`：「for _level in range(SOURCED_LIBRARY_DEPTH):」，每层对 `pending` 里的每个文件取 `source_family_names`、同目录里在盘上的才收；入口算第 0 层，族最深到入口之后第四层。文档串 `.claude/hooks/lib_project_code_runs_launchers.py:300`：「族里的文件再点名的也算（SOURCED_LIBRARY_DEPTH 层为止）」，与代码一致。
4. 「别的目录的库不在族里」：成立。`:301` 文档串写「别的目录的库（同目录之外的 import、`../` 的 source）不在族里」；代码对 `source` 的前缀目录段含 `..` 的丢掉（`:293`，`names = [... if ".." not in (match.group(1) or "").split("/")]`），其余只按文件名在入口所在目录里 `os.path.isfile` 取。
5. 「登记行『源码里有核对』认整族」：成立。`.claude/hooks/lib_project_code_runs_launchers.py:351`：「elif judgement == TEMPORARY_SERVICE_REQUIRED and not marked and not family_has_temporary_service_mark(os.path.join(root, key)):」；`family_has_temporary_service_mark` 在 `.claude/hooks/lib_project_code_runs_launchers.py:325`：「return any(source_has_temporary_service_mark(member) for member in source_family(path))」；`source_has_temporary_service_mark` 逐份读（`:438`）。只在 `required` 判定下用整族；漏登那一条仍只读入口原文（`temporary_service_table_problems` 文档串 `:332` 写明理由：族里的文件可能是别的脚本的共用库）。
6. 「两个新开关」：成立，各在两处。`.claude/hooks/lib_project_code_runs_launchers.py:304`：「if break_is_set("temporary-service-mark-entry-only"):」交回只有入口；`.claude/hooks/lib_project_code_runs_launchers.py:307`：「if break_is_set("temporary-service-family-by-directory"):」把同目录每份文件都算族里的。开关表在 `.claude/hooks/lib_project_code_runs.py:281`、`:283`，`project-code-guard.sh:587` 一带引用。
7. 「开关复跑 76 条、151 条」里的两个数：成立。复刻 `switch_entries` 的读法（`.claude/gate.d/lib/code-tooling-shared-libraries.sh:230` 的 `def switch_entries(text):` 与它上面的 `SWITCH_ENTRY`）数文件头，原样输出：

```
.claude/hooks/lib_project_code_runs.py 76 {'LIB_PROJECT_CODE_RUNS_BREAK': 76}
.claude/hooks/project-code-guard.sh 151 {'PROJECT_CODE_GUARD_BREAK': 45, 'WORKSPACE_CARGO_LOCK_BREAK': 6, 'LIB_SHELL_WORDS_BREAK': 69, 'LIB_PROJECT_CODE_RUNS_BREAK': 31}
```

   登记表对得上：`.claude/gate.d/break-switch-replay-files.tsv:70` 写「七十六条开关」，`:75` 写「一百五十一条开关」。
8. 「换上后两份自证都绿」「开关复跑逐格相同」：这一腿复核不了（要起自证与开关轮），待主 agent 跑。

### 5b agent-write-scope 格改认拆分族：成立

正文那一条（`research/prompts/_split-governance-hooks-gates-r1-body.md:19`）原文：「agent-write-scope 格：`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh` 取函数名时取 `lib_shell_words.py`、`lib_project_code_runs.py` 的整族（入口加同层 `<主干>_*.py`），新开关 `AGENT_WRITE_SCOPE_BREAK=family-parts-unread`、新样本 `agent-write-scope-parts-red`。换上后主工作区单跑这一格绿，整档样本里这一格 10 份都判得对；它的 11 条开关复跑开跑前算键全部拒算，没跑。」逐句核：

1. 「取整族（入口加同层 `<主干>_*.py`）」：成立。`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:680`：「part_paths = sorted(glob.glob(os.path.join(glob.escape(os.path.dirname(module_path)), glob.escape(module_stem) + "_*.py")))」；`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:681`：「family_names = {os.path.basename(path) for path in [module_path, *part_paths]}」。部分文件的顶层函数进 `function_home`（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:694`：「function_home.setdefault(node.name, os.path.basename(family_path))」），入口自己的进 `entry_function_names`（`:696`）；`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:699`：「function_names = list(function_home)」，查的名字、`NOT_SHARED_JUDGMENT` 的过期登记核对（`:709`）都从整族取。
2. 「族」的范围与真实载入方式对得上：成立。这一格的 `glob` 在 `.claude/hooks/` 里取到 `lib_shell_words_core.py`、`lib_shell_words_embedded.py` 与 `lib_project_code_runs_launchers.py`、`lib_project_code_runs_selftest.py`（`ls .claude/hooks | grep -E '^lib_(shell_words|project_code_runs)'` 列出六份：两份入口加四份部分文件）；真实载入：`lib_project_code_runs.py:1545` 的 `PROJECT_CODE_RUNS_PART_NAMES = ("lib_project_code_runs_launchers.py", "lib_project_code_runs_selftest.py",)` 与 `lib_shell_words.py` 里 `from lib_shell_words_core import (`、`from lib_shell_words_embedded import (`，两边点名的文件与 `glob` 取到的相同；没有 `<主干>_*.py` 形状而被载入的部分文件，也没有形状对而没被载入的。
3. 一族之外才查手抄：成立。`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:718`：「if directory == hooks_directory and file_name in family_names:」跳过的是 `.claude/hooks/` 顶层的一族文件（拆前只跳入口本身）；`:726` 的报告行带「共用模块里定义在 <文件名>」，部分文件里的函数被手抄时指得到住在哪一份。
4. 「新开关 `family-parts-unread`」：成立。`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:676`：「family_parts_unread = os.environ.get("AGENT_WRITE_SCOPE_BREAK") == "family-parts-unread"」；`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:711`：「checked_names = [name for name in function_names if name not in not_judgment and (not family_parts_unread or name in entry_function_names)]」，开关下只查入口顶层函数；过期登记仍照整族判（`:376` 的说明「NOT_SHARED_JUDGMENT 的登记照一族判」与 `:699` 一致）。开关表在 `.claude/gate.d/code-tooling.sh:932`：「#   AGENT_WRITE_SCOPE_BREAK=family-parts-unread  agent-write-scope 查手抄的函数名只取共用模块入口文件里的顶层函数、不取同层部分文件里的」，点名的样本在 `.claude/gate.d/code-tooling.sh:933`：「#     agent-write-scope-red（手抄的 shell_tokens、judged_by_name 都住在部分文件里）、agent-write-scope-parts-red」。
5. 「另查出换上前就红」那一句（记录 `records/2026-11-17-subagent账本清欠调度记录.md:510`）与代码对得上：登记 `NOT_SHARED_JUDGMENT`（`:657`）里的 `selftest` 此刻住在 `lib_project_code_runs_selftest.py:341`（`def selftest():`），`load_sibling_module` 住在入口 `lib_project_code_runs.py:312`；只读入口的老读法下 `selftest` 不在入口顶层函数里，过期登记核对会判红；整族读法下两个都在。复刻脚本 `family_defs.py` 的原样输出（读法与 `:680` 到 `:726` 同口径，只读）：

```
lib_shell_words.py parts ['lib_shell_words_core.py', 'lib_shell_words_embedded.py'] functions 139 checked 139 entry-only 23 scanned 72 stale [] hits []
  selftest home: None  load_sibling_module home: None
lib_project_code_runs.py parts ['lib_project_code_runs_launchers.py', 'lib_project_code_runs_selftest.py'] functions 116 checked 114 entry-only 56 scanned 72 stale [] hits []
  selftest home: lib_project_code_runs_selftest.py  load_sibling_module home: lib_project_code_runs.py
```

   此刻整族读法下没有手抄命中、没有过期登记；单跑这一格该打出「139 个函数（lib_shell_words.py 加 2 个部分文件）」「114 个函数（lib_project_code_runs.py 加 2 个部分文件）」「另外 72 个文件」（推的：复刻读法得出，没起这一格；对账命令在「要主 agent 跑的命令」一节）。
6. 「新样本 `agent-write-scope-parts-red`」：成立。目录在盘；`diff -rq agent-write-scope-green agent-write-scope-parts-red` 原样输出只有两行：`Only in agent-write-scope-parts-red/.claude/hooks: copied-part-functions.sh` 与 `Files agent-write-scope-green/expect and agent-write-scope-parts-red/expect differ`，即「green 那一份多一个 hook」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:365` 的说明）成立。
7. 「整档样本里这一格 10 份」：数目成立：`ls .claude/gate.d/fixtures/code-tooling.sh | grep agent-write-scope` 列出 find-timestamp-red、green、multiedit-red、parts-red、placement-red、readers-red、red、run-scope-red、session-number-red、timer-red 共 10 份；「都判得对」与「单跑这一格绿」复核不了，待主 agent 跑。
8. 「11 条开关开跑前算键全部拒算，没跑」：条数成立，复跑状态不属这一腿。复刻 `switch_entries` 对 `.claude/gate.d/code-tooling.sh` 数「点名的项里出现 agent-write-scope」的开关条目：`11 [('STAGE_CELLS_BREAK', 'red-swallowed'), ('AGENT_WRITE_SCOPE_BREAK', 'any-model'), …, ('AGENT_WRITE_SCOPE_BREAK', 'family-parts-unread')]`，即十条 `AGENT_WRITE_SCOPE_BREAK` 加 `STAGE_CELLS_BREAK=red-swallowed` 一条；这一句说的 11 与它同口径。

## 第 5 问之外顺带读到的、与「做的是不是说的那件事」有关的几处

- 文件头与记录对不上一处：`.claude/hooks/project-code-guard.sh:587` 对 `temporary-service-mark-entry-only` 写「钩子这一侧红的格（推的，没复跑；只在 lib_project_code_runs.py 的自检里）」，`:590` 对 `temporary-service-family-by-directory` 同样；而记录 `records/2026-11-17-subagent账本清欠调度记录.md:512` 与登记表 `.claude/gate.d/break-switch-replay-files.tsv:75` 写这一份 151 条开关整份复跑逐格相同。两处说的是同一批开关是否复跑过，写法冲突（以主 agent 复跑为准，复跑后头注的「没复跑」要改）。
- 「漏登只读入口」是有意的，不是漏改：复刻 `unlisted_family.py` 对 `selftest-registry.tsv` 里不在表里的 162 行（非门禁阶段）算「入口没有记号、族里有」，得到 8 行（原样输出的前两行：`family-only-mark-not-in-table scripts/gate/checker-direction-overlap.py ['scripts/gate/admission.py']`、`family-only-mark-not-in-table scripts/crash/crash-library-totals.sh ['scripts/crash/crash-amplification-lib.sh']`），记号都来自别的脚本也在 source / import 的共用库；整族算进漏登那一条会把这 8 份判成漏登，`lib_project_code_runs_launchers.py:332` 的文档串给的就是这个理由，成立。
- `required` 四行转绿的出处：复刻 `family_replica.py` 对 `selftest-temporary-service.tsv` 的 50 行 `required` 逐行算，原样汇总：`46 entry-only-marked`（输出里写作 entry-marked）、`4 family-only`，四行是 `scripts/crash/crash-point-replay.sh`、`scripts/experiment-apparatus/replay.sh`、`scripts/gate/verus-proof-check.py`、`scripts/mutation/broad-mutation.py`，与记录 `:512` 点名的四行相同；`family_paths.py` 对这四行各给出记号所在的文件都是入口直接点名的一份（`crash-amplification-lib.sh`、`replay-selftest.sh`、`lib_reuse_checks.py`、`broad_mutation_selftest_b.py`），没有靠第二层才转绿的。

## 第 6 问：两件修补新加的自证格、样本与弄坏开关，红的地方是不是名字说的那件事

方法：不起自证，读每一格的输入与期望，再逐条代入两个开关的新读法（`source_family` 的三个分支，`.claude/hooks/lib_project_code_runs_launchers.py:299` 起）推哪几格会红，与文件头点名的那几格逐格对。推出来的红格不是实测，实测归主 agent（命令在「要主 agent 跑的命令」一节）。

### 6a 项目代码闸的拆分族格（`.claude/hooks/lib_project_code_runs_selftest.py`）：成立

样本布局（`:119` 一带，逐份写进临时目录的 `scripts/gate/`）：python 入口 `sample-split-marked.py` 只写 `from sample_split_marked_part import value`、记号在同目录 `sample_split_marked_part.py`；`sample-split-unmarked.py` 与它的部分文件都没有记号；shell 两对同形（`source "$sample_directory/sample-split-marked-part.sh"`）；`sample-elsewhere-import.py` import 的 `sample_elsewhere_part` 放在 `scripts/crash/`（`:133`：`("scripts/crash/sample_elsewhere_part.py", marked)):`），不是同目录。

| 格（`:` 行号是文件里的行） | 判的是哪一件事 | `entry-only` 下 | `family-by-directory` 下 |
|---|---|---|---|
| `:183` 样本表写坏的、两边对不上的几行逐条报出来 | `temporary_service_table_problems` 报出的键集合恰是 10 个（不含 split-marked 两份） | 红：split-marked 两份读成「没有核对」被多报 | 红：split-unmarked 两份与 elsewhere-import 被整目录的族里别的有记号文件盖住、少报 |
| `:190` 拆分族里记号挪进同目录部分文件的 required 行不报（期望 `(False, False)`） | 记号在 import / source 的同目录部分文件里的 required 行不被报 | 红（族里只有入口） | 绿（整目录的族一定含部分文件） |
| `:192` 拆分族里哪一份都没有记号的 required 行照旧报源码里没有核对（期望 `(True, True, True)`） | 族里没有记号的、记号在别的目录模块里的，仍被报 | 绿（只有入口时照样报） | 红（同目录里别的有记号文件算进族，三个都不报） |
| `:195`、`:197` 拆分族：python / shell 入口连同它 import / source 的同目录模块 | `source_family` 交回的列表恰是 入口 + 点名的部分文件 | 红（只交入口） | 红（交回整个 `scripts/gate/`） |
| `:199` 拆分族：别的目录里的模块不在族里（期望 `["scripts/gate/sample-elsewhere-import.py"]`） | 别的目录的模块不进族 | 绿（只有入口） | 红（整目录的族多出同目录的文件） |
| `:209` 仓里的表与自证登记表、脚本源码对得上（期望 `[]`） | 仓里 `selftest-temporary-service.tsv` 的 `required` 行都对得上 | 红：四行记号在部分文件的 required 行被报（四行见 5b 末尾「`required` 四行转绿的出处」） | 绿（只会少报，不多报） |

与文件头点名逐格对：`.claude/hooks/lib_project_code_runs.py:283` 起 `temporary-service-mark-entry-only` 点名的是「样本表写坏的…」「拆分族里记号挪进同目录部分文件的 required 行不报」「拆分族：python 入口…」「拆分族：shell 入口…」「仓里的表」五格，与上表 `entry-only` 一列的红格相同；`:281` 起 `temporary-service-family-by-directory` 点名的是「样本表写坏的…」「拆分族里哪一份都没有记号的 required 行照旧报源码里没有核对」「拆分族：python 入口…」「拆分族：shell 入口…」「拆分族：别的目录里的模块不在族里」五格，与 `family-by-directory` 一列相同；两处的格名在 `lib_project_code_runs_selftest.py`、`lib_project_code_runs.py` 头注、`project-code-guard.sh:587` 起的两条里各用 `grep -cF` 数到 1 次或 2 次，没有哪一格名在文件里找不到（`grep -cF` 数得：六个格名在 selftest 里各 1 次，在 `lib_project_code_runs.py` 头注里 1 或 2 次，在 `project-code-guard.sh` 头注里 1 或 2 次）。红的地方在名字说的那件事上：每一格的红都落在 `source_family` 的某个分支（只交入口 / 整目录 / 按点名取）上，没有红在别的判法上。

没有格钉住的几处（推的，没起自证，要推翻只要找到钉住它们的格）：
- 第二层以下（部分文件再点名的部分文件）与 `SOURCED_LIBRARY_DEPTH = 4`（`:927`）的截断：样本里每个入口只点名一层（`:119` 到 `:133` 的 16 个样本文件），把 `:310` 的 `range(SOURCED_LIBRARY_DEPTH)` 改成 `range(1)` 这一类改动，表中那些格都不会红。仓里 `required` 四行也都是入口直接点名的（5b 末尾 `family_paths.py` 的输出）。
- `source_family_names` 的 shell 读法只看文件名、不核前缀目录是不是入口所在目录（只丢含 `..` 的）：`source "$REPO/scripts/lib/x.sh"` 在入口同目录也有个 `x.sh` 时，同名那一份会被当族员。样本里没有 `../` 的 source、也没有这种同名撞库的样本。

### 6b agent-write-scope 的开关与样本：成立

- 开关 `family-parts-unread`：`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:711` 开关下只查入口顶层函数名。代入 `agent-write-scope-red`：它手抄的 `shell_tokens` 住在 `.claude/hooks/lib_shell_words_core.py:495`：「def shell_tokens(text):」、`judged_by_name` 住在 `.claude/hooks/lib_project_code_runs_launchers.py:1282`：「def judged_by_name(word, directory):」，都不是入口顶层函数，开关下漏报；该样本 `expect` 里的 `want=共用切词模块 .claude/hooks/lib_shell_words.py 里的函数，在 .claude/hooks/ 下别的文件里又定义了一份`、`want=.claude/hooks/copied-shell-words.sh:4  def shell_tokens`、`want=.claude/hooks/copied-project-code.sh:4  def judged_by_name` 等行找不到，红在第 ⑬ 条（`:358` 起的判据）。
- 样本 `agent-write-scope-parts-red`：`.claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/copied-part-functions.sh:4`：「def brace_expand(word, mask):」、`.claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/copied-part-functions.sh:6`：「def classify(words, directory):」；真实模块里 `.claude/hooks/lib_shell_words_core.py:710`：「def brace_expand(word, mask):」，`.claude/hooks/lib_project_code_runs_launchers.py:1202`：「def classify(words, directory, environment=None, expand_patterns=True):」；`expect` 的 `:3`、`:5` 两行：`want=.claude/hooks/copied-part-functions.sh:4  def brace_expand（共用模块里定义在 lib_shell_words_core.py）`、`want=.claude/hooks/copied-part-functions.sh:6  def classify（共用模块里定义在 lib_project_code_runs_launchers.py）`，要求的就是 `:726` 的「共用模块里定义在 <文件名>」那一段。只读入口的读法（`:711` 开关下就是这个读法）下，两个函数都不在入口顶层函数名里，这份样本判绿而 `expect` 第 1 行要 `exit=1`，所以样本红在「函数住在部分文件里」这一件事上、没有别的红因：它与 `agent-write-scope-green` 只差这一个 hook（`diff -rq` 输出见 5b 第 6 条）。
- 开关下 `agent-write-scope-green` 等不受影响（它没有手抄的函数），与 `.claude/gate.d/code-tooling.sh:933` 只点名 `agent-write-scope-red`、`agent-write-scope-parts-red` 一致。

## 第 7 问：八道门禁的格名表、`--list`、`stage_cell` 登记逐格相同，格的函数都找得到：成立（`--list` 的实际输出待主 agent 跑）

### 做法

`--list` 的来历写在 `.claude/gate.d/lib/stage-cells.sh`：`:20` 原文「# 文件头的格名表每格一行 `# gate-cell: <格名> <判什么的一句>`，与 stage_cell 登记的逐格相同、次序相同」，`:24`「#   --list                     逐行打「格名<制表符>判什么」到 stdout，退 0，不跑任何格」，`:210` 起 `if ((list_requested)); then` 照 `STAGE_CELLS_NAMES` 逐格打名字与标题。所以七道经这份库的入口，`--list` 的输出就是 `stage_cell` 登记的名字与标题；门禁 crash-point-replay 不经这份库登记，它的格名表在入口里写成常量（`.claude/gate.d/crash-point-replay.sh:853` 原文：「# 格名表：与文件头九行 `# gate-cell:` 逐字相同、次序相同」），`--list` 走 `crash_point_replay_print_cells`（`crash-point-replay.sh:913` 的 `    --list) crash_point_replay_list_cells=1 ;;`、`:989` 的 `  crash_point_replay_print_cells`，函数在 `.claude/gate.d/lib/crash-point-replay-selection-and-plan.sh:7`：「crash_point_replay_print_cells() {」）。

复刻脚本（`cells_check.py`、`desc_check2.py`，抄 `# gate-cell:` 与 `^stage_cell` 行的读法，续行 `\` 并行、双引号内按 bash 的 `\$` `\"` `\\` 还原）对七道经共用库的入口逐格比 名字、标题、次序，原样输出：

```
code-tooling cells 44 name+title differences: [] args per row: [4]
code-source-discipline cells 17 name+title differences: [] args per row: [4]
doc-decisions cells 21 name+title differences: [] args per row: [4]
doc-experiments cells 16 name+title differences: [] args per row: [4]
doc-process-records cells 14 name+title differences: [] args per row: [4]
doc-registries cells 30 name+title differences: [] args per row: [4]
doc-text cells 17 name+title differences: [] args per row: [4]
```

`cells_check.py` 另给出名字次序逐格相等（七道都 `names-equal-in-order=True`）、每个 `stage_cell` 登记的函数在入口或它 source 的库里都找到（七道 `registered function not found anywhere in entry+libs: []`）。

### 逐道结果

| 入口 | `# gate-cell:` 行数 | `stage_cell` 行数 | 名字、标题、次序 | source 的格库（共用库除外） | 登记的函数住在 |
|---|---|---|---|---|---|
| `code-tooling.sh` | 44（`:23` 起） | 44（`:1083` 到 `:1169`） | 逐格相同 | 9 份：`:1222` 到 `:1230` 的 `source "$STAGE_DIRECTORY/lib/code-tooling-*.sh"` | selftest-cells 4、unit-verdict-and-entry-cells 7、rules-and-agents-cells 7、fixtures-and-layout-cells 10、source-scan-cells 7、stage-break-switch-replay-cells 2、break-switch-replay-cells 3、copy-closure-and-selftest-shape-cells 4，共 44 |
| `code-source-discipline.sh` | 17（`:17` 起） | 17（`:376` 到 `:408`） | 逐格相同 | 3 份（`:433` 到 `:435`） | naming-and-constants 6、mutation-and-reading 3、unsafe-and-registry 8，共 17 |
| `doc-decisions.sh` | 21（`:14` 起） | 21（`:452` 到 `:492`） | 逐格相同 | 3 份（`:517` 到 `:519`） | 21 个包装函数都在 `lib/doc-decisions-evidence-and-owed-cells.sh`（如 `.claude/gate.d/lib/doc-decisions-evidence-and-owed-cells.sh:958`：「judge_kb_shape()」的 `judge_kb_shape()` 一行，函数体调 `grid_kb_shape`），判法函数 `grid_*` 分在三份库里 |
| `doc-experiments.sh` | 16（`:17` 起） | 16（`:532` 到 `:562`） | 逐格相同 | 3 份（`:638` 到 `:640`） | index-and-evidence 4、verdict-and-citation 7、preregistration-and-replay 5，共 16 |
| `doc-process-records.sh` | 14（`:13` 起） | 14（`:856` 到 `:882`） | 逐格相同 | 2 份（`:1097`、`:1098`）；另 `:931` 的 `source "$LIB_CHANGED_PATHS"`（共用的改动范围库） | batch-and-review 5、rounds-and-ledger 9，共 14 |
| `doc-registries.sh` | 30（`:18` 起） | 30（`:502` 到 `:560`） | 逐格相同 | 3 份（`:595` 到 `:597`） | 30 个都在 `lib/doc-registries-reference-and-invariant-cells.sh` |
| `doc-text.sh` | 17（`:15` 起） | 17（`:1410` 到 `:1442`） | 逐格相同 | 2 份（`:1468`、`:1469`） | notation-and-translation 13、content-and-history 4，共 17 |
| `crash-point-replay.sh` | 9（`:11` 到 `:19`） | 无 `stage_cell`，格名表是入口常量 | 九格名字、标题、次序与 `crash_point_replay_print_cells` 打出的相同（见下） | 4 份（`:894` 到 `:897`） | 格不是函数：`crash-case-replay`、`feature-gated-tests`、`product-gpu-real-card-tests` 在入口与 `lib/crash-point-replay-feature-card-and-audit-cells.sh` 里按 `crash_point_replay_runs_cell "$格名"` 判跑不跑（库里 `:749`、`:765`、`:916`，入口 `:1260`、`:1726`），其余六格经 `crash_point_replay_run_audit_cells`（`lib/…feature-card-and-audit-cells.sh` 的 `run_crash_audit_cell`，`:797`）跑 |

「登记的函数住在」一列的计数来自 `cells_check.py` 输出的 `cell functions live in`（与 `stage_cell` 第二个词对 `^name() {` 的定义查找）；没有同名函数在入口与库、或库与库之间重复定义（脚本对重复打 `DUP`，输出里没有）。

### crash-point-replay 的九格

`desc_check.py` 原样输出：

```
crash-point-replay print_cells order == header order: True
  title text equal per cell: [True, True, True, True, True, True, True, True, True]
  NAME constants: crash-case-replay feature-gated-tests product-gpu-real-card-tests crash-case-provenance ['crash-flow-coverage', 'crash-ignored-nodes', 'invariant-zero-evaluations', 'crash-case-must-red', 'crash-knobs']
```

打印次序见 `.claude/gate.d/lib/crash-point-replay-selection-and-plan.sh:9-12`（`crash-case-replay`、`product-gpu-real-card-tests`、`CRASH_AUDIT_CELL_NAMES` 的第 3、4 项、`crash-case-provenance`、`feature-gated-tests`、第 0 到 2 项），与文件头 `:11` 到 `:19` 的次序逐个相同。

### 与拆前相比

- 拆前原文的格名表（`git show HEAD:.claude/gate.d/<入口>` 在草稿目录里的拷贝，`names-<入口>.diff`）与此刻逐格相同，只多四格，都不是拆分带出来的：`code-tooling.sh` 多 `crate-roles`、`git-hooks-installed`，`doc-experiments.sh` 多 `pinned-apparatus-revision`，`doc-text.sh` 多 `task-and-session-numbers`；crash-point-replay、code-source-discipline、doc-decisions、doc-process-records、doc-registries 与拆前一个名字都不差。
- 一处与拆分无关的旧账：`.claude/gate.d/code-tooling.sh:4` 写「工具层自检（四十五格」、`:22` 写「不给就四十五格按下面的次序全跑」，而此刻 `# gate-cell:` 行数是 44；拆前原文写四十三格、`# gate-cell:` 行是 42，差一同样存在。`.claude/gate.d/doc-registries.sh:4` 写「三十一格」，`# gate-cell:` 是 30，拆前同样。叙述里的格数比登记多一，不是拆分改出来的，也不影响 `--list` 与 `stage_cell` 逐格相同（推的：读 `scripts/gate/gate-structure-check.py` 文件头，它判 table、preflight、list、unknown 四样，没有一样数叙述里的格数）。

## 要主 agent 跑的命令

每条一项；「在哪个目录起」写在命令后。结局看各自任务末行 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`。

1. 起：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 agent-write-scope、gate-structure 两格; 范围=2 格; 为什么=核拆分族认法改完后 agent-write-scope 在主工作区判绿、八道门禁格名表与 --list 逐格相同' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check agent-write-scope,gate-structure`
   在仓根起。看：`exit=0`；`task-server/task log <号>` 里两格各一行 `✓`，agent-write-scope 的成功句应含「共用切词模块的 139 个函数（lib_shell_words.py 加 2 个部分文件）」「共用项目代码判定模块的 114 个函数（lib_project_code_runs.py 加 2 个部分文件）」「另外 72 个文件」（这三个数是本报告第 5b 第 5 条复刻出来的推的数，对不上就是复刻口径与这一格不同）；gate-structure 没有 `✗` 行。
2. 起：`task-server/task submit -- python3 .claude/hooks/lib_project_code_runs.py --selftest`
   在仓根起。看：`exit=0`，日志里「要临时服务：拆分族…」六格都不在 `✗` 行里。
3. 起：`task-server/task submit -- python3 .claude/hooks/project-code-guard-selftest.py`
   在仓根起（自证入口是 `.claude/hooks/project-code-guard.sh --selftest` 还是这份文件，以 `.claude/hooks/project-code-guard.sh` 文件头为准，这一腿没核）。看：`exit=0`，里面「lib_project_code_runs.py 的自检」那一格无 `✗`。
4. 起：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格里 lib_project_code_runs.py 一份; 范围=1 项; 为什么=核两条新开关点名的格与实际红的格逐格相同' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/lib_project_code_runs.py`
   在仓根起。看：成功句「复跑了 76 条开关（1 份），自证红且点名的格与实际红的格逐格相同 76 条」；`temporary-service-mark-entry-only`、`temporary-service-family-by-directory` 两条各红 5 格（第 6a 节的表）。`--item` 的写法是这一腿照 `.claude/agent-common.md` 与记录里的 `--check break-switch-replay --item scripts/agents/agent-watch.py` 推的，不对就用 `--list-items` 看。
5. 起：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格里 project-code-guard.sh 一份; 范围=1 项; 为什么=核 151 条开关含两条新开关点名的格与实际红的格逐格相同' task-server/task submit -- bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item .claude/hooks/project-code-guard.sh`
   在仓根起。看：成功句里 `151 条`；并核 `.claude/hooks/project-code-guard.sh:587`、`:590` 的「推的，没复跑」两句要不要改。
6. 起：`task-server/task submit -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item code-tooling.sh`
   在仓根起（样本档整份，约 259 份样本）。看：`exit=0`；输出里没有 `✗ code-tooling.sh agent-write-scope-` 开头的行（10 份：find-timestamp-red、green、multiedit-red、parts-red、placement-red、readers-red、red、run-scope-red、session-number-red、timer-red）。已知 function-units-two-implementations-agree、mutation-list-items 两族红归别的会话，不在这一腿射程。
7. 开关下证红（agent-write-scope，样本档在 `AGENT_WRITE_SCOPE_BREAK=family-parts-unread` 下该红 `agent-write-scope-red` 与 `agent-write-scope-parts-red` 两份、别的不红）：起：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 agent-write-scope 格对 agent-write-scope-parts-red 样本; 范围=1 格; 为什么=证 family-parts-unread 开关下 parts-red 样本判绿、即样本红在函数住进部分文件这一件事上' task-server/task submit -- env AGENT_WRITE_SCOPE_BREAK=family-parts-unread bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item code-tooling.sh`
   在仓根起。这一条把弄坏开关交给整份样本档，只有样本根上才认开关（`.claude/gate.d/lib/stage-cells.sh` 文件头「弄坏开关只在样本根上认」），所以只会在样本里生效；看：失败行恰是 `agent-write-scope-red`、`agent-write-scope-parts-red` 两份（其余 agent-write-scope-* 不在失败行里）。如果服务认为要包一层临时服务写法，照 `.claude/hooks/project-code-guard.sh` 的拒绝说明改写。
8. 复刻脚本复跑（草稿目录里，只读仓）：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-sonnet -- python3 -I cells_check.py /home/fy5090/code/singlefs`；同样把 `cells_check.py` 换成 `desc_check.py`、`desc_check2.py`、`family_replica.py`、`family_paths.py`、`family_defs.py`、`unlisted_family.py`、`aws_entries.py`（后者参数是 `.claude/gate.d/code-tooling.sh` 的绝对路径）各一条；`count_switches.py` 的参数是 `.claude/hooks/lib_project_code_runs.py` 与 `.claude/hooks/project-code-guard.sh`、在仓根起。看：各条的输出与本报告贴的相同。

## 复刻脚本（草稿目录 `/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-sonnet/`，只读仓；不是模型，没有 `rerun.sh`）

| 文件 | sha256 前 16 位 | 用途 |
|---|---|---|
| `family_replica.py` | 7ca10bf569ccf3a1 | 照 `source_family` 的读法对 `selftest-temporary-service.tsv` 的 required 行算记号在入口还是族里 |
| `family_paths.py` | 3b03bd2d0385538d | 对四个转绿的入口给出记号第一次出现的文件与路径 |
| `family_defs.py` | 8083f41713972098 | 照 agent-write-scope 格 `:680` 到 `:726` 的读法算两族的函数名、过期登记、手抄命中 |
| `count_switches.py` | 54b0656e159f999e | 照 `switch_entries` 数文件头开关条数 |
| `aws_entries.py` | 46776c1a46ae542a | 数 code-tooling.sh 头注里点名 agent-write-scope 的开关条目 |
| `unlisted_family.py` | 35218da9da21b64f | 算「漏登只读入口」的取舍影响了几行 |
| `cells_check.py` | 7c63d4368100b15a | 八道门禁的 `# gate-cell:`、`stage_cell`、source 的库与函数定义比对 |
| `desc_check.py` | 925aee802b2cf1b9 | crash-point-replay 的 `print_cells` 与文件头比对（另含 shlex 版的七道比对，对三个带 `\$`、`\"` 的标题误报，已由 `desc_check2.py` 替代，引用以 `desc_check2.py` 为准） |
| `desc_check2.py` | a85ad1b9531a32a2 | 七道经共用库的入口，名字与标题逐格比 |

`family_replica.py` 输出汇总（`cut -f1 family_replica.out | sort | uniq -c`）：`46 entry-marked`、`4 family-only`；探针用时都在 1 秒内、没有重进程（读文件加 ast.parse，内存按几十 MB 估，未量）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 5a 项目代码闸改认拆分族 | 成立 | `source_family`（`lib_project_code_runs_launchers.py:299`）按入口加 source / import / 按名载入的同目录文件、四层取族，`:351` 的 required 行判认整族，漏登仍只读入口；两份自证与开关复跑绿待主 agent 跑 |
| 5b agent-write-scope 格改认拆分族 | 成立 | `:680` 取 `<主干>_*.py`，`:711` 的开关只查入口名，条目数 11 与 10 份样本都对得上；单跑与整档样本绿待主 agent 跑 |
| 5 的附带发现 | 冲突（文档） | `project-code-guard.sh:587`、`:590` 写「没复跑」，记录与登记表写 151 条整份复跑过；复刻出 76 与 151 两个条数与登记相同 |
| 6a 临时服务拆分族的样本与开关 | 成立 | 六格对两个开关各红在 `source_family` 的一个分支上，红格与文件头点名逐格相同（推的，未起自证）；第二层与深度截断、前缀目录同名撞库没有格钉住 |
| 6b family-parts-unread 与 parts-red 样本 | 成立 | 手抄的 `brace_expand`、`classify` 住在部分文件里，开关下漏报，样本只比 green 多一个 hook，红因唯一；实测待主 agent 跑 |
| 7 八道门禁格名表、`--list`、`stage_cell` | 成立 | 七道名字、标题、次序逐格相同、所有登记函数找得到；crash-point-replay 九格名字标题次序与 `print_cells` 相同；`--list` 实际输出待主 agent 跑（命令 1） |
| 7 的附带发现 | 旧账 | `code-tooling.sh:4`、`doc-registries.sh:4` 叙述里的格数比登记多一，拆前同样；多出的四格不是拆分带来的 |

## 每条结论什么现象会推翻它

- 5a：命令 2 或命令 4 的结局不是 `exit=0` / 76 条逐格相同；或 `selftest-temporary-service.tsv` 的某一行 required 在主工作区被报「源码里却没有」（复刻说四行靠族转绿，若报了就是复刻口径与 `source_family` 不一样）；或找得到一份入口，用 `SOURCE_FAMILY_SHELL_SOURCE` 与 `source_family_names` 认出的族和「入口 source / import / 按名载入」的字面意思不同。
- 5b：命令 1 的 agent-write-scope 红，或成功句的三个数与 139 / 114 / 72 不同；或 `.claude/hooks/` 里出现一份形状是 `<主干>_*.py` 而没被入口载入的文件（它会被当族员）、或形状不是而被载入的部分文件（它会被当手抄）。
- 6：命令 4 里新开关点名的格与实际红的格不同；或在 `family-parts-unread` 下 `agent-write-scope-parts-red` 仍判红（那表示它红在别的地方）。
- 7：命令 1 里 gate-structure 红；或任一入口 `--list` 的输出与 `# gate-cell:` 不逐格相同；复刻脚本漏掉的写法（例如 `stage_cell` 参数里用了我的复刻没处理的引号形态）也会推翻第 7 条里「逐格相同」那一半。

## 没做什么

- 没有起任何自证、样本档、开关复跑、门禁格或 `--list`，也没有往统一任务服务交任务：上面这些的结论里「待主 agent 跑」的都是推的；判决要引的数没复跑，命令在「要主 agent 跑的命令」一节。
- 不判别的腿的格（第 1 到 4 问：行为有没有变、判法只读入口原文那一类、替身换错命名空间、三份拆分族认法会不会分叉）没写，也不替主 agent 采纳或出判决。第 4 问的「三份认法分叉」只在 6a 末尾顺带写了 `source_family_names` 的一个读法差异（只按文件名、只丢含 `..` 的前缀），没有拿 `scripts/gate/lib_lag_split_family.py` 比。
- 没读 `.claude/hooks/` 里另外两组拆分钩子（`bash-command-detector`、`completion-cleanup-guard`、`write-guard`、`runner-dispatch-guard`、`lib_run_scope*`）的拆分内容，第 7 问也只比了八道门禁入口，没比 `lib/` 里格库内部的 `grid_*` 这一层辅助函数（`callee_check.py` 试过，读法太糙、误报多，结果不引）。
- 没建仓副本、编译目录与模型；草稿目录 3.5M，都是读文件的脚本与拷贝，交回前不删（`.writer-lock`、`.owner` 在里面；脚本是「要主 agent 跑的命令」第 8 条的输入）。
