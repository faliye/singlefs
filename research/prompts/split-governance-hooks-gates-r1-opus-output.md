# 拆分换上的钩子与门禁：治理文件轮第一轮 攻方腿（split-governance-hooks-gates-r1，2026-11-20）

背景材料：`research/prompts/_split-governance-hooks-gates-r1-body.md`；只攻正文「要腿判的」第 1 到 4 问。草稿目录 `/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus`（下文写 `<草稿>`）。这条腿没写模型，探针与比对工具在 `<草稿>/tools/`，sha256 在本节末。

## 各格判定一览

| 问 | 判定 | 打中了什么（详见各节） |
|---|---|---|
| 1 挪动之外有没有行为变了 | 推不出（算打中） | ① 写闸、完工清理闸、派发闸读不到任何一份部分文件时 python 在引导里抛 FileNotFoundError、退 1，Claude Code 照常放行、检出记录一行不写；拆前读不到依赖库时写闸的设计是子 agent 的写按拒判。② `lib_shell_words.py` 拆成按裸名字 import 的真模块：载入一次就往 `sys.path` 最前塞钩子目录，`lib_shell_words_core` / `_embedded` 按裸名字进 `sys.modules`，同一进程里再从别的目录载入一份时用的是先载入那一份目录里的部分模块（拆前每次载入各是一份）。仓里今天没找到在一个进程里从两个目录载入它的调用方，影响是推的。③ 背景材料「拆前原文在 HEAD」不成立：16 道入口里 13 道 HEAD 那一版不是拆前那一版 |
| 2 判法只读入口原文的漏网 | 成立（打中） | 门禁 code-tooling 的 change-range-single-source 格只扫 `.claude/gate.d/*.sh`、`*.py` 与三道门禁（code-tooling、crash-point-replay、code-source-discipline）的库，doc 门禁五道拆出的 13 份库不扫；拆前 `doc-experiments.sh` 第 4687 行那条不带 `core.quotepath=false` 的 `git ls-tree --name-only` 在格的射程里，拆后同一行在 `lib/doc-experiments-preregistration-and-replay-cells.sh` 第 1434 行、出了射程。另有三处同形的只认部分族（research-script-selftest-coverage 只加了 code-tooling 的库；`lib_project_code_runs_selftest.py` 按 `bash-command-detector-*.py` 通配读检出钩子源码；`scripts/hosts/minimal-copy.py` 建的副本只拷三份内嵌 python 钩子的入口、不拷部分文件），今天没找到因此判错的格 |
| 3 自证换替身换没换在被测代码读的命名空间上 | 不成立（读到的范围内） | 逐处列在第 3 节；exec 进同一命名空间的七份照旧，`lib_shell_words` 唯一被换的模块变量 `SYSTEM_STARTUP_DIRECTORY` 留在读它的入口里 |
| 4 三份拆分族认法认出的族一不一样 | 成立（打中，两处用造的输入） | ① agent-write-scope 格按 `<主干>_*.py` 通配认族：`.claude/hooks/` 下一份叫 `lib_shell_words_<随便>.py`、手抄了 `shell_tokens` 而入口并不 import 它的文件被当成族里的、不报重定义；`source_family` 不认它（入口没点名）。② `source_family` 把同目录里被字符串整个点名的文件、被 import 的同目录共用库都算进族：要临时服务的 required 行，入口与它按命名拆出的部分里都没有核对，只要它点名了一份有核对的同目录脚本或 import 了一份有核对的共用库，就不报；`lib_lag_split_family` 的命名认法不收这两种 |

四句（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）在这一轮的对象上怎么答，写在各条打中的末尾：这一轮被判的不是「几条臂」而是「拆分只挪不改」这一句，「分不分辨臂」一律答「不涉臂：判的是拆前拆后同一份输入判得一不一样」。

## 复跑与文件

探针都还没跑（子 agent 不交任务），命令在末节「要主 agent 跑的命令」。下表的「预期」一律是推的。

| 文件（都在 `<草稿>/tools/`） | 做什么 | sha256 |
|---|---|---|
| `probe-write-guard-part-missing.sh` | 第 1 问 ①：四个写闸钩子目录（完好、缺 `lib_write_scope.py`、缺 `write-guard-decisions.py`、拆前单份入口同样没有部分文件）喂同一条 clerk 写 `crates/` 的 Write | 7c0f54ddce696f5e4506707ec1fc6b66bc6ad80c56828de8cdd67d00370cebc6 |
| `build_part_missing_arms.py` | 建上面四个钩子目录（已建好，在副本 `<草稿>/copy/.claude/hooks-probe-*`） | 365e459636cba2bef740bfcfac5aa7193eadb351478e0f5f8c2b96dd8bc08310 |
| `probe_shell_words_module_cache.py` | 第 1 问 ②：一个进程里先载入仓里的 `lib_shell_words.py`，再载入副本目录里 core 改过的一份；对照拆前单文件 | 18b468b3084d5a1a0d466a4a18e4fa18bc2a362e6df4f9cb2b39a10841428079 |
| `build_change_range_roots.py` | 第 2 问：建 change-range-single-source 的三份探针根（presplit、postsplit、renamed；已建好，在 `<草稿>/probe-change-range/`） | 3449ffabfd19e81263af1a66411c678946cadc508b66708d4c3b7ee79424aec9 |
| `probe_source_family_overreach.py` | 第 4 问 ②：样本树上调 `temporary_service_table_problems`、`source_family` 与 `lib_lag_split_family.family_members` | 1823a9f4cc51244db1c4e106c7dae325e1c5cde119130fa22014b8d00537f733 |
| `split_function_scopes.py`、`handoff_gaps.py`、`direct_scope_assignments.py` | 第 1 问：检出钩子 `selftest_in` 切成两个函数之后的作用域比对（只读源码，我直接跑过） | 9e0c056374268a8997ece2bf3dacec02076b8b0afc01f8844349fd4f5c1745cf、550ca5dddd093a16de655b2ed0e0063b88fcea00e0a8b163fee2fe932125aeaa、0d05d9094162cdad9a4713d47738d5a71ca09f92fcf93133d56d7ab1afeee620 |
| `inline_module_parts.py`、`top_level_statements.py`、`unresolved_globals.py`、`shell_functions_compare.py`、`library_top_level.py` | 第 1 问：拆前一份与拆后拼回去逐段比（只读源码，我直接跑过） | 2f72cccd0eadb97f51929d5d2bb3e8f0084dbfe0d0476a9c6ca0fdcc978fd17a、4e63c28142acac0894ac65157bb825d9829dde59884ef5d1327330914faa57b7、37427ea06092e33bfc8bcd770d82b141bf1ea9b176e7d3e67ca7425063467638、a7ff3cdc2a67a32eb7d2bfb766226b68b9777b22631c54b5a73228f7ee9a276d、847e1f4d24bd2be4d62675c6eb017753aefe3c0905b9bd74c8c1d22fae426ec4 |

## 开工时对不上的两件（`.claude/rules/machine-first.md@对不上就当场说`）

1. **拆前原文不在 HEAD。** 背景材料写「拆前原文在 HEAD，`git show HEAD:<路径>`」。我拿各次拆分草稿里留的基线逐份比 sha256（量的，命令是 `git show HEAD:<路径> | sha256sum` 对基线文件 `sha256sum`）：`lib_shell_words.py`、`doc-decisions.sh`、`doc-registries.sh` 三份 HEAD 与拆前相同，其余 13 道（`bash-command-detector.sh`、`completion-cleanup-guard.sh`、`write-guard.sh`、`project-code-guard.sh`、`runner-dispatch-guard.sh`、`lib_project_code_runs.py`、`lib_run_scope.py`、`code-tooling.sh`、`crash-point-replay.sh`、`code-source-discipline.sh`、`doc-experiments.sh`、`doc-process-records.sh`、`doc-text.sh`）HEAD 那一版都不是拆前那一版：拆分之前工作区里已有大量没提交的改动，拿 HEAD 比会把别的会话的改动算成拆分改的。检出钩子 HEAD 的 python 段 6758 行、拆后拼回去 9215 行，差的 2457 行几乎全是拆分之前别处的改动（量的）。我用的拆前基线：
   - 钩子第一组：`/home/fy5090/.local/share/singlefs/work/crates-split-hooks-first/base/.claude/hooks/`（`orig/` 里抽出的 python 段与之逐字节相同，量的）；
   - 钩子第二组：`/home/fy5090/.local/share/singlefs/work/crates-split-hooks-second/base/`；
   - 门禁三道：草稿 `crates-split-gates-first` 只留了 sha256 清单、没留原文，我把它的 `tooling.patch` 在 `<草稿>/recon/gates-base/` 里对 `new/` 反着打回去，三份入口的 sha256 与 `originals.sha256` 逐个对上（量的）；
   - doc 门禁五道：`/home/fy5090/.local/share/singlefs/work/crates-split-doc-gates/originals/.claude/gate.d/`。
2. **被判文件在这一轮里还在被改。** `sha256sum -c research/prompts/split-governance-hooks-gates-r1-snapshot/judged-sha256.txt` 现跑：127 份里 14 份对不上快照（`break-switch-replay-files.tsv`、`code-source-discipline.sh`、`code-tooling.sh`、`doc-process-records.sh`、`lib/code-source-discipline-mutation-and-reading-cells.sh`、`lib/code-source-discipline-unsafe-and-registry-cells.sh`、`lib/code-tooling-fixtures-and-layout-cells.sh`、`lib/doc-process-records-rounds-and-ledger-cells.sh`、`bash-command-detector-scope-and-index.py`、`bash-command-detector-selftest-entries-and-main.py`、`bash-command-detector-selftest-waits-and-scope.py`、`bash-command-detector.sh`、`write-guard-decisions.py`、`write-guard.sh`），读的过程中还看到 `lib/code-source-discipline-mutation-and-reading-cells.sh` 与 `write-guard.sh` 又变了一次。我读到的那一版整份拷在 `<草稿>/judged-at-reading/`，逐份 sha256 与「同快照 / 与快照不同」在 `<草稿>/judged-at-reading.sha256`。下文引的行号都在交回前对着主工作区现查过一遍（cite-check）；主 agent 判决时以开工快照倒推的树为准，对不上的行先按那份拷贝找。

## 第 1 问：拆前原文与入口加部分文件拼起来之后，挪动之外的行为变化

### 1.0 怎么比的、比了哪些（量的，工具在 `<草稿>/tools/`，结果在 `<草稿>/recon/`）

| 拆分族 | 比法 | 拆前拆后差在哪（挪动、部分文件头三行说明注释之外） |
|---|---|---|
| 检出钩子 `bash-command-detector.sh` 加 5 份 | 拆前入口 heredoc 里的 python 段，对 5 份部分文件按 `DETECTOR_PART_NAMES` 次序接起来，`diff` | ① `SHELL_WORDS_PART_NAMES` 新常量；② 三处拷库清单加部分文件名；③ 读自己源码（弄坏开关对账）改成入口加 5 份接起来；④ `selftest_in` 切成两个函数、经 `shared` 字典交接。bash 那一层除入口文件头加的说明行外逐字相同 |
| 写闸 `write-guard.sh` 加 2 份、完工清理闸 `completion-cleanup-guard.sh` 加 2 份 | 同上 | 只有部分文件头的说明注释 |
| 派发闸 `runner-dispatch-guard.sh` 加 3 份 | 同上 | `LINE_START`、`ZERO_ROUND_READING_LINE`、`ZERO_ROUND_TASK_NUMBER`、`BATCH_LINE_AGENTS` 四行从原位置挪到引导最前（别的脚本按行读它们）；部分文件里没有再给这四个名字赋值（grep 量的） |
| 项目代码闸 `project-code-guard.sh` 加 2 份 | 同上 | 两处拷库清单加部分文件名；那一格的注释改写 |
| `lib_project_code_runs.py` 加 2 份、`lib_run_scope.py` 加 1 份（exec 进模块命名空间） | 把入口里 exec 那一段与紧跟的「按名再绑一次」元组换回部分文件正文，`diff` | `lib_run_scope` 只有说明注释；`lib_project_code_runs` 另有拆分之后的修补（`source_family` 一族与自证格，背景材料第 5 问那一件，不归这条腿） |
| `lib_shell_words.py` 加 `_core`、`_embedded`（真模块，按名字 import） | 顶层语句按 `ast.dump` 逐条比；每份模块里函数读的全局名有没有绑定（symtable） | 顶层语句集合相同、没有同名的两份定义；三份里读而没绑定的全局名都是 0 个（检查器先拿一份带未绑定名的样本证过会红）。差在载入方式，见 1.2 |
| 门禁三道加 16 份库 | 顶层函数逐个比正文；函数外的代码行按次序比 | 7 把 `judge:` 令牌改成库的路径；`crash_point_replay_plan_tokens`、`crash_point_replay_verdict_kind` 里的 `${BASH_SOURCE[0]}` 改成 `$0`；change-range-single-source、research-script-selftest-coverage 两格的扫描范围加了库（见第 2 问）；几句出路文字里的文件名换成库名 |
| doc 门禁五道加 13 份库 | 同上 | 只有出路文字里的文件名换成库名，与末尾那一段（`--write` 分支、`stage_cells_run`）挪到 source 之后 |

另核的几件，都没出问题（量的或逐行读的）：
- 检出钩子 `selftest_in` 切成两个函数：后一半与它里面的嵌套函数按全局名读、而前一半绑成局部名的名字 0 个（`handoff_gaps.py`，对拆分刚换上那一版与此刻那一版各跑一次，此刻前一半多了 2 个局部名也照样是 0）；前一半里定义的闭包捕获的 7 个名字在后一半只在交接那一行重绑、值相同（`direct_scope_assignments.py`）。
- exec 进同一命名空间的七份：部分文件次序与拆前原文次序相同（拼回去与原文逐段相同，上表），所以同名覆盖与次序依赖与拆前一样；`__file__` 在引导里仍是 `/dev/fd/3`、`__name__` 仍是 `__main__`；部分文件里没有 `__future__`、`inspect`、`co_filename` 一类读自己文件名的用法（grep 量的）；`globals()["git_hook_findings"] = …`、`global writer_lock`、`global birth_times, decide, background_work` 都落在同一个字典上。
- 门禁库里 source 时就执行的顶层语句只有常量与 heredoc 赋值、`declare -A …=()` 与一处 `grep_error_file="$work_directory/grep-errors"`；`work_directory` 在 `doc-registries.sh` 里 source 那几行之前就赋了值，入口在 source 之后也不再写那几个 `declare` 过的名字（grep 量的）。
- `BASH_SOURCE[0]` 改 `$0`：两者只在阶段被 `source` 进来时不同；`scripts/`、`.claude/`、`task-server/`、`crates/` 下没有一处 `source` / `.` 这道阶段（grep 量的，0 处），今天等价。

### 1.1 打中：三道闸读不到部分文件时整闸放行、一行检出都不记

写闸的入口在引导里按名字 `open` 部分文件再 exec：「        exec(compile(hook_part_handle.read(), hook_part_path, "exec"))」（`.claude/hooks/write-guard.sh:357`），heredoc 收尾的 `PY` 之后整份入口再没有一行，python 的退出码原样交出去；派发闸「        exec(compile(hook_part_handle.read(), hook_part_path, "exec"))」（`.claude/hooks/runner-dispatch-guard.sh:1018`）、完工清理闸「        exec(compile(hook_part_handle.read(), hook_part_path, "exec"))」（`.claude/hooks/completion-cleanup-guard.sh:421`）同样，`PY` 之后都没有别的行（`awk '/^PY$/{f=1} f'` 量的，三份都只打出 `PY` 一行）。哪一份部分文件不在、改了名、读不了，`open` 在 python 的 main 之前抛 FileNotFoundError，退 1。检出钩子入口自己写着这时的后果：「钩子 bash 这一层起不来时也记一条检出（一百三十七节 ⑤）：python3 起不来、/dev/fd/3 读不到、python 在 main() 之前崩了，这条命令照常执行（钩子退出码不是 0、2 时 Claude Code 照常放行），」（`.claude/hooks/bash-command-detector.sh:1355`）。检出钩子与项目代码闸在 bash 那一层兜住了（`hook_record_bash_layer_failure` 记一条、退 0）；写闸、派发闸、完工清理闸没有这一层。

拆前这三份的 python 全在入口的 heredoc 里，「读不到部分文件」这种输入不存在；它们读不到别的依赖时各有写明的做法，写闸是「# 读不到它时子 agent 的写按拒判（decide_scope），拒绝信息点名那份库」（`.claude/hooks/write-guard-decisions.py:21`），完工清理闸是自己出错照「放行但必记检出」：「    except Exception as error:  # noqa: BLE001 —— 自己出错不拦，免得一个缺陷挡住所有收工」（`.claude/hooks/completion-cleanup-guard-decisions.py:1730`），紧跟着 `report_unjudged`。拆分加进来的这一种依赖，三份都既不拒、也不记。

能照喂的输入（探针 `probe-write-guard-part-missing.sh`，四个钩子目录已建在副本 `<草稿>/copy/.claude/` 下）：clerk 用 Write 写副本 `crates/probe-part-missing.rs`（clerk 的写范围里没有 `crates/`）。预期（推的，待主 agent 跑）：

| 臂 | 预期退出码 | 预期检出行数 |
|---|---|---|
| `hooks-probe-intact`（此刻的 `.claude/hooks` 整份） | 2 | 1 |
| `hooks-probe-scope-library-missing`（缺 `lib_write_scope.py`，拆前就有的依赖） | 2（拒绝说明点名那份库） | 1 |
| `hooks-probe-part-missing`（缺 `write-guard-decisions.py`） | 1，stderr 是 python 的 FileNotFoundError | 0 |
| `hooks-probe-presplit-part-missing`（拆前单份入口，同样没有 `write-guard-*.py`） | 2 | 1 |

最后一臂用的是拆前那一版入口配此刻的共用库，库的接口这期间若改过会让它另外出错，那样这一臂作废，只看前三臂。

够得着的操作序列：`tooling-writer` 与主 agent 的写范围含 `.claude/hooks/`；改部分文件的名字、或按 hook-swap「先放部分文件、再换入口」反着做（先删旧部分文件再换入口），中间那一段写闸对所有人的 Write / Edit 全放行、派发闸对所有派发全放行，检出记录里看不到。拆前同样的改名在一份文件里一次做完，没有这一段。

四句：不涉臂；闸当时看得到「部分文件读不到」（`open` 抛的就是它），判据（「放行但必记检出」与写闸读不到依赖按拒）字面就管这一格；满足的是「挪动之外行为变了」里「退出码、拒绝说明」两项。

自提的改法（只在我的读法上推过、没量过、被攻过零轮）：三份入口在 `PY` 之后照检出钩子补 bash 那一层（`hook_status=$?`，不是 0、2 的记一条检出），写闸那一层对子 agent 再按「读不到判法按拒」退 2。修的是上表第三臂；对第四臂（拆前单份）不起作用、也不需要。

### 1.2 推不出（算打中）：`lib_shell_words` 拆成按裸名字 import 的模块，载入一次就改了进程的两处全局状态

拆前 `lib_shell_words.py` 是一份文件，调用方按路径 `spec_from_file_location` 载入，每载入一次就是独立的一份，不碰 `sys.path`（拆前那一份里 `sys.path` 只出现在一句注释里，grep 量的）。拆后入口这样载入部分：「if _LIBRARY_DIRECTORY not in sys.path:」（`.claude/hooks/lib_shell_words.py:289`）、「    sys.path.insert(0, _LIBRARY_DIRECTORY)」（`.claude/hooks/lib_shell_words.py:290`）、「from lib_shell_words_core import (」（`.claude/hooks/lib_shell_words.py:291`）。由此两件拆前没有的事：

1. 任何载入它的进程，`sys.path` 最前多一项钩子目录，之后这个进程里裸名字的 `import` 先到 `.claude/hooks/` 找。今天钩子目录里没有与 `scripts/`、`task-server/` 等处同名的模块（逐个按文件名找过，0 个），所以今天没撞上。
2. `lib_shell_words_core`、`lib_shell_words_embedded` 按裸名字进 `sys.modules`：同一进程里先从目录 A 载入过，再按路径载入目录 B 的 `lib_shell_words.py`，B 的入口 `from lib_shell_words_core import` 拿到的是 A 的部分模块。连带「GIT_CONFIG_QUERY_CACHE = {}」（`.claude/hooks/lib_shell_words_embedded.py:94`）从「每载入一份一个缓存」变成整个进程一个。

探针 `probe_shell_words_module_cache.py`：仓里那一份先载入，再载入副本目录里 core 的 `shell_tokens` 改成交 `["副本"]` 的一份；对照臂是拆前单文件同样改过的一份。预期（推的，待主 agent 跑）：`sys_path_added` 是仓里的 `.claude/hooks`；`arm=split-copy` 的 `core_file` 指回仓里、`shell_tokens('a b')` 不是 `['副本']`；`arm=presplit-copy` 交 `['副本']`。

为什么只算推不出：判法会不会因此判错，要有一个进程从两个目录载入它。我在钩子、`scripts/gate/hook_swap*.py`、`scripts/gate/selftest-process-guard.py` 里找过，载入都只来自同一个钩子目录，从临时钩子目录起的都另起子进程，所以今天没找到判错的格。会撞上的形状：哪个自证或换上工具在一个进程里先载入主工作区那一份、再载入待换上的新一份来核，新一份的 core 改动就被旧的盖掉，核的是旧代码。四句：不涉臂；载入方看不到用的是哪一份（`__file__` 指 B、部分模块却是 A 的）；属于「挪动之外行为变了」。

改法（零轮、推的）：入口按自己目录用 `spec_from_file_location` 载入两份部分、换一个带目录摘要的模块名，或照另外几份钩子改成 exec 进入口自己的命名空间；不往 `sys.path` 塞东西。

### 1.3 其余几处不是挪动、今天等价（推的，依据在 1.0 表）

- 7 把 `judge:<文件>::<函数>` 令牌改成库的路径：判定键的字面变了，复用的判定全部作废一次；拆分报告（`/home/fy5090/.local/share/singlefs/work/crates-split-gates-first/result.md`）自己写了这一条。
- `crash-point-replay` 两个函数 `${BASH_SOURCE[0]}` 改 `$0`：「  crash_point_replay_test_functions_request "$(basename "$0")" "$1"」（`.claude/gate.d/lib/crash-point-replay-selection-and-plan.sh:80`）；阶段从不被 `source`，等价。哪天有脚本 `source` 这道阶段（比如为复用它的函数），阶段文件名与判法根会取成那份脚本的。

## 第 2 问：「判法只读入口原文」这一类，已知几处之外的漏网

### 2.1 打中：change-range-single-source 格读不到 doc 门禁五道拆出的库

格的扫描范围是「scanned = sorted(path for pattern in ('.claude/gate.d/*.sh', '.claude/gate.d/*.py', '.claude/gate.d/lib/code-tooling-*.sh',」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:1185`）续「'.claude/gate.d/lib/crash-point-replay-*.sh', '.claude/gate.d/lib/code-source-discipline-*.sh')」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:1186`）。门禁三道拆分时把这三族的库加了进去（拆前那一行只有前两项，量的：拆前基线与拆分产物逐函数比出来的唯一差别），doc 门禁五道后拆，`lib/doc-*.sh` 13 份没进来；格的文件头还写着「# 判据（射程 .claude/gate.d/ 顶层的 *.sh 与 *.py；这一道文件里只除掉这一格自己：从这一格的标题行到下一格的标题行按空行判），任一条不成立判红：」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:979`）。

doc 门禁 13 份库里有 61 行出现 `--name-only`、`--name-status`、`ls-files`、`gate_changed_paths`、`gate_added_lines`（`cat .claude/gate.d/lib/doc-*.sh | grep -cE` 量的，含注释与出路文字），拆前都在 `.claude/gate.d/doc-*.sh` 顶层、在射程里。其中一行不带 `core.quotepath=false`、也不经同文件里带它的变量或包装函数：拆前「    listed = subprocess.run(["git", "ls-tree", "--name-only", base, "--", path], capture_output=True, text=True)」在 `doc-experiments.sh` 第 4687 行（拆前基线里 grep 量的），拆后同一行在「    listed = subprocess.run(["git", "ls-tree", "--name-only", base, "--", path], capture_output=True, text=True)」（`.claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh:1434`）。照格的判法（`judge` 里 `LIST_PATTERN` 命中、`uses_quoting` 不成立、不以引号开头）拆前该点名它，拆后读不到它。

探针（三份根已建在 `<草稿>/probe-change-range/`，都从样本 `change-range-single-source-green` 拷来再加 doc-experiments 的文件）：`presplit` 加拆前那份 `doc-experiments.sh`；`postsplit` 加此刻的入口与三份库；`renamed` 同 postsplit，只把三份库改名成 `lib/code-tooling-doc-experiments-*.sh`。预期（推的，待主 agent 跑）：presplit 红、点名拆前那份入口 `.claude/gate.d/doc-experiments.sh` 的第 4687 行；postsplit 不点名任何 `lib/doc-experiments-*` 的行；renamed 点名改名之后的 `lib/code-tooling-doc-experiments-preregistration-and-replay-cells.sh` 第 1434 行。renamed 那一臂证明差别只在文件名落不落进那串通配。

四句：不涉臂；格看得到这一行（文件就在 `.claude/gate.d/lib/` 下），是射程没写到；满足的是第 2 问「判法只读入口原文」那一类；改法（零轮）：射程写成 `.claude/gate.d/lib/*.sh`（库都只给门禁用），或按 `lib_lag_split_family` 的 shell 拆法从每道入口推出它的库。只补 `doc-*` 一族修得了这一格，下一道拆分照样漏。

`ls-tree --name-only` 这一行要不要真改，归 doc-experiments 那一格的主人判；这里只说格判不到它。

### 2.2 同一形状、今天没找到判错的三处（线索，交主 agent 定要不要一并改）

- research-script-selftest-coverage 格只加了 code-tooling 一族：「for stage in sorted(glob.glob(os.path.join(".claude/gate.d", stage_pattern)) + glob.glob(".claude/gate.d/lib/code-tooling-*.sh")):」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:1253`）。crash-point-replay、code-source-discipline、doc 五道的库里今天没有 `<脚本> --selftest` 这种写法（grep 量的，0 行），所以没有脚本被误判成「没有门禁在跑」；哪份库以后起了某个脚本的自证，这一格看不见、照报它没人跑（误红）。
- 第四份拆分族认法：`lib_project_code_runs` 的自证读检出钩子源码时按通配取部分：「        for detector_source_path in [os.path.join(hook_directory_of_library, "bash-command-detector.sh")] + sorted(glob.glob(os.path.join(hook_directory_of_library, "bash-command-detector-*.py"))):」（`.claude/hooks/lib_project_code_runs_selftest.py:748`），再 `re.search` 取头一处 `CLAUDE_EXECUTABLE_PATH_WORDS = {…}`。检出钩子运行时按 `DETECTOR_PART_NAMES` 显式五份载入，这里按通配：同目录一份不在名单里的 `bash-command-detector-<排在 scope-and-index 之前的名字>.py`（比如留下的旧部分文件）里有同名一行，这一格比的就是它。
- 仓副本缺部分文件：我照共用约束用 `python3 scripts/hosts/minimal-copy.py <仓根> <草稿>/copy --with-test-inputs --with-gate-tables` 建了副本，它拷了 31 份钩子文件，含 `bash-command-detector.sh`、`completion-cleanup-guard.sh`、`project-code-guard.sh`、`runner-dispatch-guard.sh`、`write-guard.sh` 五份入口，它们的部分文件一份都没拷（计划行在 `<草稿>/minimal-copy.txt`，`grep -c` 部分文件名为 0，量的）；`lib_project_code_runs`、`lib_shell_words`、`lib_run_scope` 的部分都拷了。闭包认 source、import 与按路径起，认不出 heredoc 里一张名字表 exec 进来的文件。在这种副本里交这五道闸的 `--selftest` 会退 1（读不到部分文件），是吵的红、不是漏判；`scripts/hosts/lib_repository_copy.py` 不在这一轮的被判文件里，只报不判。

### 2.3 查过、没漏的（逐处读的）

- 写闸自证读派发闸源码里的三条正则、`scripts/agents/dispatch-prompt.py` 读 `BATCH_LINE_AGENTS`、`scripts/agents/verdict-skeleton.py` 读三条正则：四行留在派发闸入口的引导里，读得到。
- 各道钩子自证里的拷库清单（检出钩子三处、项目代码闸两处）都列全了此刻的部分文件；只是 `lib_shell_words` 的两份名字在检出钩子里另抄了一份：「SHELL_WORDS_PART_NAMES = ("lib_shell_words_core.py", "lib_shell_words_embedded.py")   # lib_shell_words.py 按自己的目录导入的同目录部分：自证拷库到临时钩子目录时一起拷」（`.claude/hooks/bash-command-detector-scope-and-index.py:11`），项目代码闸自证里又写死了一遍，与 `lib_shell_words.py` 自己的 import 是三处同步，今天一致。
- stage-location-before-cd 格明写 lib/ 不判；拆前 `crash-point-replay.sh` 里用 `dirname "$0"` 的那几个函数定义在 `cd "$ROOT"` 那一行之前（基线 grep 量的：cd 在第 1188 行，函数在第 927–945 行），拆前这一格同样不点名它们，没有变。
- break-switches-sample-only、stage-owners、fixture-claims 三格只读入口（判的是入口调没调 `stage_cells_run`、`--list` 与文件头），入口里都还在。
- 门禁 code-tooling 的 break-switch-replay 一族三格、算键闭包两件，背景材料列为已知、各有会话在修，没重复追。

## 第 3 问：自证换替身、打补丁换没换在被测代码读的命名空间上

不成立（读到的范围：下表七份 exec 族与 `lib_shell_words` 一族的全部自证；`grep` 找换替身的写法 `globals()[`、`global `、`<模块>.<名字> =`、`setattr(`、`patch.object`、`importlib.reload`，逐处看被读的那一边）。

| 换在哪 | 被测代码在哪读 | 同不同一个命名空间 |
|---|---|---|
| 检出钩子自证 `globals()["git_hook_findings"] = …` | 同一份 exec 进来的 `__main__` 全局 | 同 |
| 写闸自证 `global writer_lock` | `write-guard-decisions.py` 的函数读模块全局 `writer_lock` | 同（exec 进同一个字典） |
| 完工清理闸自证 `global birth_times, decide, background_work` | decisions 那一份 | 同 |
| `lib_project_code_runs_selftest.py` 的 `global SELFTEST_REGISTRY_PATH, …`、`lib_run_scope_selftest.py` 的 `global submit_flag_asks_in_text` | 入口与另一份部分（都 exec 进模块自己的命名空间） | 同 |
| 派发闸自证 `snapshot_library.open_three_way_rounds = …` | `lib_snapshot_conflicts`（没拆） | 同 |
| 检出钩子与项目代码闸自证 `shell_words.SYSTEM_STARTUP_DIRECTORY = …` | `startup_file_words` 在 `lib_shell_words.py` 入口里读它 | 同；入口文件头写明这一类变量留在读它的那一份里 |
| 弄坏开关的环境变量 | 每次调用时现读 `os.environ`（`words_break_is_set` 等），没有载入时缓存的 | 不涉命名空间 |

`lib_shell_words` 一族另核：`lib_shell_words_core.py`、`_embedded.py` 里没有 `global` 语句，模块变量没有在运行时被重绑的（只有 `GIT_CONFIG_QUERY_CACHE` 原地改字典，入口再导出的是同一个对象）。会翻掉这一条的：哪处自证换 `shell_words.<名字>` 而读它的函数在 `_core` / `_embedded` 里——今天 0 处（上面那几种写法全仓 grep，只有 `SYSTEM_STARTUP_DIRECTORY` 六处）。

## 第 4 问：三份拆分族认法，同一个入口认出的族一不一样

三份认法（引的是此刻的代码）：
- `source_family`（项目代码闸）：入口与族里每一份里 shell 的 `source` / `.`、python 的 `import`、`from … import` 与引号里整个是 `<名>.py` / `<名>.sh` 的字符串，同目录、在盘上的，最多 4 层：「PYTHON_SIBLING_REFERENCE = re.compile(r"^\s*(?:import\s+([A-Za-z_][A-Za-z0-9_]*)|from\s+([A-Za-z_][A-Za-z0-9_]*)\s+import)|[\"']([\w.-]+\.(?:py|sh))[\"']", re.MULTILINE)」（`.claude/hooks/lib_project_code_runs_launchers.py:926`）；「    交回完整路径、不重复、入口在最前。别的目录的库（同目录之外的 import、`../` 的 source）不在族里。」（`.claude/hooks/lib_project_code_runs_launchers.py:301`）。
- agent-write-scope 格：入口加同层 `<主干>_*.py` 通配：「    part_paths = sorted(glob.glob(os.path.join(glob.escape(os.path.dirname(module_path)), glob.escape(module_stem) + "_*.py")))」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:680`），只管 `lib_shell_words.py`、`lib_project_code_runs.py` 两个入口。
- `lib_lag_split_family`：按命名（转述 `scripts/gate/lib_lag_split_family.py:11`：Python 拆法认入口同目录下的 `<模块名>_<部分>.py`，模块名是入口文件名去掉后缀、连字符换成下划线），另有钩子拆法与 `.claude/gate.d/lib/` 的 shell 拆法。

在被判的入口上认出的族（`source_family` 一列是我照它的两条正则在主工作区逐层模拟的，推的；另两列按命名算，量的）：

| 入口 | `source_family` | agent-write-scope | `lib_lag_split_family` |
|---|---|---|---|
| `lib_shell_words.py` | 入口加 `_core`、`_embedded` | 同 | 同 |
| `lib_project_code_runs.py` | 入口加两份部分，再往下带进 `lib_shell_words` 一族等（import 与引号里的名字） | 入口加两份部分 | 入口加两份部分 |
| `lib_run_scope.py` | 14 份：带进 `lib_run_scope_target_sources.py`、`lib_project_code_runs` 一族、`run-scope-guard.sh`、检出钩子入口与五份部分 | 不管 | 入口加 `lib_run_scope_selftest.py`、`lib_run_scope_target_sources.py`（后者是独立的库，不是拆出来的） |
| `bash-command-detector.sh` | 25 份：五份部分之外，经自证里拷库清单的引号名字带进 `lib_*` 十来份、项目代码闸入口与部分、`run-scope-guard.sh` | 不管 | 五份部分 |
| `write-guard.sh` | 14 份：两份部分之外带进派发闸入口与三份部分、`handback-guard.sh` 等 | 不管 | 两份部分 |
| `.claude/gate.d/<门禁>.sh` | 只有入口（库在 `lib/`，「别的目录」） | 不管 | 入口加 `lib/<门禁>-*.sh` |

三份在两个真拆的模块上一致，其余全不一致。不一致落到判错的两处：

### 4.1 打中：agent-write-scope 格按通配多认，名字落进族里的手抄函数不报

格判的是「一族里每个文件顶层定义的每个函数名，.claude/hooks/ 下一族之外的文件里不许再 def 一份」，族的定义是「#      入口把部分文件的名字带出来、调用方只从入口导入）合起来叫一族；一族里每个文件顶层定义的每个函数名，.claude/hooks/ 下一族之外的文件里不许再 def 一份；名字从这一族现读，」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:359`），扫的时候族里的文件整份跳过：「            if directory == hooks_directory and file_name in family_names:」（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:718`）。族由通配认，入口 import 没 import 它都算族里。

能照喂的输入：`.claude/hooks/lib_shell_words_shadow.py` 写一份 `def shell_tokens(text):`，入口 `lib_shell_words.py` 不 import 它（别的钩子可以直接 import 它，绕开入口那一份）。`source_family` 不认它（入口没点名），格却把它算进族、跳过不扫，手抄的 `shell_tokens` 不报；同一份内容换个名字（`shadow_shell_words.py`）就报。两份已放进副本 `<草稿>/copy/.claude/hooks/`。预期（推的，待主 agent 跑）：格的输出点名 `.claude/hooks/shadow_shell_words.py` 第 2 行的 `def shell_tokens`（共用模块里定义在 `lib_shell_words_core.py`），不点名 `lib_shell_words_shadow.py`。格判红是因为对照那一份，看的是点名的行。

四句：不涉臂；格看得到入口的 import（它本就 `ast.parse` 每份族里的文件），只是不用；满足格自己定义里「入口把部分文件的名字带出来」那一分句的反面；改法（零轮、推的）：族改成入口里 `from <模块> import` 点名的同目录模块（与 `source_family` 的 python 一半同义），或直接调 `source_family`。

### 4.2 打中：`source_family` 多认，要临时服务的 required 行在入口与真拆出的部分都没有核对时不报

核对那一句：「        elif judgement == TEMPORARY_SERVICE_REQUIRED and not marked and not family_has_temporary_service_mark(os.path.join(root, key)):」（`.claude/hooks/lib_project_code_runs_launchers.py:351`）。族里收引号里整个是文件名的字符串与同目录的任何 `import`：一份入口只要在字符串里写了同目录另一份有核对的脚本的名字（出路文字、拷贝清单、`HELPER = "x.py"`），或 import 了同目录一份有核对的共用库（像 `scripts/gate/lib_reuse_checks.py`，四份 verus 脚本都 import 它，里面有记号），它自己拆出的部分里没有核对也算「源码里有」。

探针 `probe_source_family_overreach.py` 在临时样本树上直接调此刻的 `temporary_service_table_problems`、`source_family` 与 `lib_lag_split_family.family_members`。预期（推的，待主 agent 跑）：

| 样本入口（都登 required，入口与按命名拆出的部分里都没有核对） | `source_family` 预期成员 | `lag_family` 预期成员 | 预期 `reported_unmarked` |
|---|---|---|---|
| `sample-quotes-sibling.py`（`HELPER = "sample-own-checker.py"`） | 入口、`sample-own-checker.py` | 入口、`sample_quotes_sibling_part.py` | False（该报而没报） |
| `sample-imports-shared.py`（`import sample_shared_checks`） | 入口、`sample_shared_checks.py` | 只有入口 | False（该报而没报） |
| `sample-plain-unmarked.py`（对照） | 只有入口 | 只有入口 | True |

今天的 required 行里入口自己没有记号的 4 行（`scripts/crash/crash-point-replay.sh`、`scripts/experiment-apparatus/replay.sh`、`scripts/gate/verus-proof-check.py`、`scripts/mutation/broad-mutation.py`，awk 加 grep 量的），按命名都能在真拆出的部分里找到记号（`crash-point-replay-selftest.sh`、`replay-selftest.sh`、`verus_proof_check_selftest.py`、`broad_mutation_selftest_b.py`），所以今天没有放过的行，打中靠造的输入。四句：不涉臂；判法看不出被点名的文件是不是它拆出来的；满足第 4 问「一份认法多认，那一处判法因此判错」；改法（零轮）：required 这一边改用 `lib_lag_split_family` 的命名认法，或 `source_family` 只收入口里 exec / source / import 的那几种、不收任意引号里的文件名，二者都要再攻。

### 4.3 不一致、今天没判错的

- `.claude/gate.d/<门禁>.sh`：`source_family` 不收 `lib/`，命名认法收。`source_family` 今天只有 required 核对一个调用方，临时服务表里没有门禁阶段的行，没撞上；哪天登一道门禁、记号挪进了它的 `lib/`，就误报「源码里没有核对」。
- `lib_run_scope_target_sources.py` 在命名认法里归 `lib_run_scope.py` 的族。滞后表里今天没有钩子的行，没撞上；它若在 `comment-history-lag.tsv` 有了 `lib_run_scope.py` 那一行，`target_sources` 里新加的历史注释会被那一行的额度吸收，而不是当族外判红。

## 没打中的形状

- exec 进同一命名空间的七份（检出钩子、写闸、完工清理闸、派发闸、项目代码闸、`lib_project_code_runs`、`lib_run_scope`）：拼回去与拆前逐段相同，同名覆盖、次序依赖、`__file__`、`__name__`、`__future__`、编译单元拆开之后读自己行号与文件名的用法，都没找到差别（1.0）。检出钩子两半交接：漏交接 0 个、闭包看不到后一半重绑 0 处，对拆分刚换上那一版与此刻那一版各量一次。
- 门禁八道：函数正文逐个比，挪动之外只有 1.0 表里那几处；库在 source 时执行的顶层语句没有依赖入口里 source 之后才赋的变量；`declare -A` 在顶层 source、照旧是全局。
- 换替身（第 3 问）：见第 3 节的表，0 处换错命名空间。
- 拆分族认法在两个真拆的 python 模块（`lib_shell_words`、`lib_project_code_runs`）上三份一致。
- 取样范围：被判 127 份里，16 道拆过的入口与它们的 48 份部分文件、库（钩子 19 份、门禁库 29 份）全部机械比过（拆前基线见开工那一节）；读判法正文的是第 2 问那几格与第 4 问三份认法的实现。

## 这条腿自己的限度

- 没读的：样本目录下的 `expect`、`.gate-cells` 与样本源码（checker-independence-and-sync、heavy-thread-defaults、agent-write-scope-parts-red、unit-key-gpu-switch-not-in-key 那几份），`checker-independence-and-sync.sh`、`model-differential-and-scenarios.sh`、`qemu-device-streams.sh` 三道没拆的门禁，`break-switch-replay-files.tsv`、`selftest-registry.tsv` 两张表，`run-scope-guard.sh` 只读了它与 `lib_run_scope.py` 的接法，`lib_run_scope_target_sources.py` 只看了它被谁 import。这几份只有这条腿没读到的，拿「没打中」当依据之前要另派一条同立场腿。
- 门禁三道的拆前原文是我从 `tooling.patch` 反打出来的，入口三份 sha256 对得上，库是新建文件、反打时删掉，没有可比的拆前版本（本来也没有）。
- 拆后的比对对的是我读那一刻的文件；14 份在这一轮里被改过（开工那一节），我在交回前对此刻的文件重跑了检出钩子交接那一项，别的比对没重跑。
- `source_family` 一列是照它的正则在 python 里重抄一遍模拟的，不是调它本身；4.2 的探针调的才是它本身。
- 全部「预期」都是推的，一条探针都没跑；会推翻 1.1 的观测：第三臂退 2 或记了检出；推翻 2.1：presplit 臂不点名拆前 `doc-experiments.sh` 的第 4687 行（那样拆前这一格本来就判不到它，射程漏的只是今天没有违例的行）；推翻 4.1：输出点名了 `lib_shell_words_shadow.py`；推翻 4.2：前两行 `reported_unmarked=True`；推翻 1.2：`arm=split-copy` 交 `['副本']`。

## 要主 agent 跑的命令

`<草稿>` 写全：`/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus`。探针都只跑几秒到一两分钟、内存在几百 MB 以内（推的，没量）。

1. 写闸读不到部分文件（1.1）。
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus -- bash /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/tools/probe-write-guard-part-missing.sh`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus`
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里四行 `PROBE arm=… exit=… detections=…`，对 1.1 的表。
2. `lib_shell_words` 跨目录载入（1.2）。
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus -- python3 /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/tools/probe_shell_words_module_cache.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/crates-split-hooks-first/base/.claude/hooks/lib_shell_words.py /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/probe-shell-words-cache`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus`
   - 看哪几行：`exit=0`；日志里四行 `PROBE`，对 1.2 的预期。
3. change-range-single-source 拆前那一臂（2.1）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 change-range-single-source 格，判探针根 presplit; 范围=1 格; 为什么=核拆前 doc-experiments.sh 里不带 quotepath 的 git 调用在这一格的射程里' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-tooling.sh --check change-range-single-source /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/probe-change-range/presplit`
   - 在哪起：仓根
   - 看哪几行：日志里「行列路径的 git 调用没带 -c core.quotepath=false」下有没有拆前入口 `.claude/gate.d/doc-experiments.sh` 的第 4687 行；`exit=1` 是预期。
4. change-range-single-source 拆后那一臂（2.1）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 change-range-single-source 格，判探针根 postsplit; 范围=1 格; 为什么=核拆后 doc 门禁的库出了这一格的射程、同一行不再被点名' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-tooling.sh --check change-range-single-source /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/probe-change-range/postsplit`
   - 在哪起：仓根
   - 看哪几行：日志里没有任何 `lib/doc-experiments-` 的行即预期；入口自己的行另看。
5. change-range-single-source 改名对照臂（2.1）。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 change-range-single-source 格，判探针根 renamed; 范围=1 格; 为什么=对照：库名落进格的通配就照判，差别只在射程' task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-tooling.sh --check change-range-single-source /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/probe-change-range/renamed`
   - 在哪起：仓根
   - 看哪几行：有改名之后的 `lib/code-tooling-doc-experiments-preregistration-and-replay-cells.sh` 第 1434 行；`exit=1` 是预期。
6. agent-write-scope 多认（4.1），在副本里跑副本那一份门禁。
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=副本里门禁 code-tooling.sh 的 agent-write-scope 格; 范围=1 格; 为什么=核按通配认族时名字落进族里的手抄函数不报、换个名字就报' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/copy -- bash .claude/gate.d/code-tooling.sh --check agent-write-scope /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/copy`
   - 在哪起：副本 `/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/copy`（`.deliverables` 写成交接）
   - 看哪几行：「又定义了一份」下点名 `shadow_shell_words.py` 第 2 行而不点名 `lib_shell_words_shadow.py`；格别的红行（副本里没跟踪的文件这类）不看。
7. `source_family` 多认（4.2）。
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus -- python3 /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/tools/probe_source_family_overreach.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus/probe-source-family`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/split-governance-hooks-gates-r1-opus`
   - 看哪几行：`exit=0`；三行 `PROBE entry=…`，对 4.2 的表。

## 没做什么

- 不判正推那几格（第 5 到 7 问），不替主 agent 采纳；改法都只在我的读法上推过、被攻过零轮。
- 没往统一任务服务交任务，七条探针都没跑；判决要引的数由主 agent 复跑。
- 副本 `<草稿>/copy`（`du -sh` 193M）留给第 6 条与第 1 条（第 1 条的四个钩子目录在它里面），在 `<草稿>/.deliverables` 写成交接；`<草稿>/recon/`、`<草稿>/judged-at-reading/` 是比对材料与读到那一刻的被判文件，不是仓副本。
