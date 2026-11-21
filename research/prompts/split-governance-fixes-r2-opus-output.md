# split-governance-fixes-r2 攻方腿报告（opus，2026-11-20）

正文 `research/prompts/_split-governance-fixes-r2-body.md` 第 1 到 4 问。被判文件读的是主工作区此刻那一版；我在草稿目录建的冻结副本 `/home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy`（`scripts/hosts/minimal-copy.py` 建，再把主工作区 `.claude/hooks/` 整份 rsync 进去：闭包没带上钩子的部分文件），对 `research/prompts/split-governance-fixes-r2-snapshot/judged-sha256.txt` 的 58 份逐个 `sha256sum -c` 都是 OK（命令与输出见「这条腿自己的限度」）。所有探针读数都还没跑：表里的「读数」一栏全是跑前写死的预期，标「推的，待主 agent 跑」。

## 复跑命令与文件

复跑（主 agent 交，见末节第 1 条）：`bash research/prompts/split-governance-fixes-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录，工作根下、以 -run 收尾> <模型目录>`；第 4 问的五条格命令在它之后（末节第 2 到 6 条）。

| 文件 | sha256 |
|---|---|
| tools/rerun.sh | 7d99dd092d2ce68bf886985083b9cc62176650988b4e1339f45d4be2d2ae6be7 |
| tools/rerun-from-a-clean-work-directory.sh | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |
| tools/probe_hook_part_states.py | 18e0fec96273e4548a51c04de61b5392d4ce5759943d0ac84a11bf4b041a429c |
| tools/probe_shell_words_paths.py | 1e14c58be832e24654cb1d8477a3f70a79693143878269335fa06bd99883d39c |
| tools/probe_split_family_recognition.py | 973db21a52e168c4364661b44118de7195fe64889d2f3a318745379a90557584 |
| tools/build_write_scope_samples.py | 0b202e214ceea702195e4ccc9a8737fa40eafa0e95e8051321e67ab8d01c9324 |

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`（`diff` 模板第 1–79、86–164 行与 rerun.sh 第 1–79、92–170 行无差）；`rerun-from-a-clean-work-directory.sh` 从 `git show HEAD:research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh` 原样取（sha256 与 c996-code-r2 那一份同值）。模板自带的 `--selftest` 用假仓跑 `rerun_steps`，换成这几份探针之后那一格会红，不拿它证什么。探针的内存与时间：推的，二十五次喂钩子加三份纯 python，估几分钟、几百 MiB 以内，没量过。

## 各格判定一览

| 问 | 判 | 一句话 | 依据的种类 |
|---|---|---|---|
| 1 读进并编过才 exec 还放行的状态 | 不成立（打中） | 编得过而不完整的部分文件（截成 0 字节、删掉末尾 `sys.exit(main())` 一行）过得了引导：带 main 的最后一份空着时写闸、派发闸、完工清理闸、项目代码闸、检出钩子都静默退 0，一条检出都不记；第一份空着时写闸、派发闸在 main 之前抛 NameError 退 1，正是修补要消掉的那一种 | 代码逐行（文:行在第 1 问一节）；探针 `probe_hook_part_states.py` 推的，待主 agent 跑 |
| 1 三份闸拒一切调用的出路 | 推不出（算打中） | 完工清理闸的引导在 Stop、SubagentStop 上照退 2，不看 `stop_hook_active`，正常判法里「续跑回来照放、免得卡死」那一条够不着；子 agent 写不了 `.claude/hooks/`、交回又被拦，出路「让主 agent 换回」它递不出去。项目代码闸与写闸同时读不到时主 agent 手里只剩只读工具，出路只在用户 | 代码逐行；探针那一格推的，待主 agent 跑；Claude Code 对 Stop 退 2 的处置是我印象里的，没核 |
| 2 按目录各存一份 | 成立（没打中） | 符号链接目录、`..`、相对路径载入同一个目录，`_LIBRARY_DIRECTORY` 都归到同一个 realpath；仓里唯一的 `isinstance(word, shell_words.ShellWord)` 那一处，词与类出自同一个目录。只有「符号链接后接 `..`」这一种写法会让入口与部分模块出自两个目录而不报错，仓里没有调用方这样写 | 代码逐行；探针 `probe_shell_words_paths.py` 推的，待主 agent 跑 |
| 3 loaded_family 多认少认 | 不成立（打中） | 载入那一半按正文正则认：注释、报错句、文档串里引号写死的同目录文件名与文档串里行首的 `import` 都算载入（多认）；`from . import`、`import_module("名")`、`source "$(dirname …)/名.sh"` 认不出（少认）；门禁阶段 lib/ 拆法在绝对路径下命名那一半就不认，`sibling_files` 也从不交 lib/ 下的候选 | 代码逐行；探针 `probe_split_family_recognition.py` 推的，待主 agent 跑 |
| 4 agent-write-scope 两族取法不同 | 不成立（打中） | 被判仓的族多认 → 手抄漏报（name-like-red 样本的入口多一行注释就判绿）；被判仓的族少认 → 被判仓自己的部分文件被报成手抄（误报）；这道脚本那一族少认时查的函数名跟着少，报得更少，与 `scripts/gate/lib_split_family_loaded.py:13` 说的「族小了报得更多」方向相反 | 代码逐行；五份样本上跑这一格推的，待主 agent 跑 |

撤回与收窄第一轮攻方腿自己给过的方案写在「撤回第一轮攻方腿的方案」一节：钩子与门禁那一题「三、改法」第 1 条只管读不到、编不过，没管编得过而不完整；第 4 条「交集把两种多认都排掉」说宽了。

## 跑前写死的判读表

### 第 1 问：`probe_hook_part_states.py`

每一臂一行 `PROBE_HOOK_PART hook=<钩子> arm=<臂> exit=<码> detections=<检出行数> stderr_first=<…>`。喂的输入：写闸是主 agent 用 Write 往 `research/` 下写带 U+2032 的 `.md`（撇号类角标，谁写都拒）；项目代码闸是主 agent 直接 `cargo build -p singlefs-core`；派发闸是派 `general-purpose` 而没有「通用已判」一行；检出钩子是 `rm -r <工作根>`（⑪）；完工清理闸是 `stop_hook_active` 为真的 Stop（会话号是假的、名下没有认领目录）。

| 钩子 | intact（对照） | main-part-empty | first-part-empty | main-part-missing（修补的对照） |
|---|---|---|---|---|
| write-guard | 2 | 0、stderr 空、detections=0 | 1、stderr 首行 Traceback，detections=0 | 2，点名 write-guard-selftest.py |
| project-code-guard | 2 | 0、stderr 空、detections=0 | 0、bash 那一层记 1 条检出（python 退 1 或 main 里接住） | 2，点名 project-code-guard-selftest.py |
| runner-dispatch-guard | 2 | 0、stderr 空、detections=0 | 1、Traceback，detections=0 | 2，点名 runner-dispatch-guard-selftest.py |
| completion-cleanup-guard | 0 | 0 | 1 | 2（`stop_hook_active` 为真照拦） |
| bash-command-detector | 2 | 0、stderr 空、detections=0 | 0、bash 那一层记 1 条检出 | 0、记 1 条检出（引导退 70） |

main-part-empty 那一臂另打 `PROBE_HOOK_PART_SELFTEST`：预期五条都 `exit=0 output_lines=0`（自证一格不跑、也报退 0）。

判法：
- 坐实「编得过而不完整的部分文件让闸静默放行」：写闸、项目代码闸、派发闸三条里任一条 intact=2、main-part-missing=2 而 main-part-empty=0 且 detections=0。
- 坐实「main 之前抛错退 1 照常放行」：写闸或派发闸 first-part-empty=1 且 detections=0。
- 坐实「完工清理闸引导不看 stop_hook_active」：completion-cleanup-guard intact=0 而 main-part-missing=2。
- 作废：某钩子 intact 不是表里的值（输入在副本里判不出拒），那一条钩子的整行不判；`PROBE_HOOK_PART_SETUP` 行出现（部分文件名单对不上）同样作废那一条。
- 推翻：main-part-empty 在三条闸上都不是 0（例如退 2 或记了检出），第 1 问前半改判「成立」。

### 第 2 问：`probe_shell_words_paths.py`

| form | 预期 entry_mark | 预期 core_mark | 预期 same_core_as_absolute | 预期 shellword_same_class |
|---|---|---|---|---|
| absolute | A | A | True | True |
| relative | A | A | True | True |
| directory-symlink | A | A | True | True |
| dot-dot | A | A | True | True |
| symlink-then-dot-dot | A | W | False | False |
| other-directory | W | W | False | False |

判法：前四行与表相同，第 2 问前半「同目录不同写法是同一份」成立；任一行 same_core_as_absolute=False 就是打中。symlink-then-dot-dot 那一行若真是「入口 A、core W」，记一条「入口与部分模块出自两个目录而 `_end_part_imports` 不报」，但仓里没有调用方这样拼路径，不算打中真调用方。

### 第 3 问：`probe_split_family_recognition.py`

| case | 预期 |
|---|---|
| gate-stage-naming | in_family_relative=True in_family_absolute=False |
| gate-stage-loaded | relative_with_lib_candidate=['stagex-cells.sh'] absolute_with_lib_candidate=[] absolute_with_sibling_files=[] |
| quoted-name-in-comment | family=['lib_demo_shadow.py']（多认） |
| quoted-name-in-message | family=['lib_demo_shadow.py']（多认） |
| import-line-in-docstring | family=['lib_demo_shadow.py']（多认） |
| from-import-control | family=['lib_demo_part.py'] |
| relative-from-import | family=[]（少认） |
| import-module-by-name | family=[]（少认） |
| from-dotted-import | family=[]（正则只认 `from <名> import`；这种写法对平铺文件跑不起来，只记不判） |
| shell-source-dirname | family=[]（少认） |
| shell-source-variable-control | family=['demo-steps.sh'] |
| task-server-entry | selftest_members 大于 0（`task_server_lib.py` 函数里的 `import task_server_selftest` 把自证各份带进族） |
| hooks-lib_shell_words.py | family=['lib_shell_words_core.py', 'lib_shell_words_embedded.py'] |
| hooks-lib_project_code_runs.py | family=['lib_project_code_runs_launchers.py', 'lib_project_code_runs_selftest.py'] |

判法：三条多认里任一条 family 带 `lib_demo_shadow.py`、或三条少认里任一条是 []，第 3 问打中；gate-stage-naming 的 absolute 为 False 坐实「绝对路径下 lib/ 拆法不认」。推翻：多认三条全是 []、少认三条全带上了部分文件。

### 第 4 问：五份样本上的 agent-write-scope 格

| 样本 | 预期 |
|---|---|
| name-like-red-unchanged（对照） | exit=1，点名 `.claude/hooks/lib_shell_words_shadow.py:4  def commands_at_command_position（共用模块里定义在 lib_shell_words.py）` |
| name-like-red-quoted-comment | exit=0，成功行（漏报） |
| name-like-red-docstring-import | exit=0，成功行（漏报） |
| green-core-by-import-module | exit=1，点名 `.claude/hooks/lib_shell_words_core.py:1  def brace_expand（共用模块里定义在 lib_shell_words_core.py）`（误报：被判仓自己的部分文件） |
| green-core-by-from-import（对照） | exit=0，成功行 |

判法：对照两份与表不同，五份整组作废；对照对上而 quoted-comment 或 docstring-import 退 0，坐实漏报；对照对上而 import-module 退 1 点名 `lib_shell_words_core.py`，坐实误报。推翻：两份多认样本照红、import-module 样本判绿。

## 第 1 问：引导「先读进并编过、全编成了才 exec」

### 1a 编得过而不完整的部分文件：不成立（打中）

五条钩子的引导都是同一个形状：逐份 `open` 加 `compile`，只接 `except (OSError, SyntaxError, ValueError) as hook_part_error:`（`.claude/hooks/write-guard.sh:368`），接住才退 2；接不住的、编得过的，一律走到 `for hook_part_code in hook_part_codes:`（`.claude/hooks/write-guard.sh:374`、`.claude/hooks/project-code-guard.sh:697`、`.claude/hooks/runner-dispatch-guard.sh:1035`、`.claude/hooks/completion-cleanup-guard.sh:435`、`.claude/hooks/bash-command-detector.sh:1431`）里裸 `exec`，外面没有 try。判的是「读得到、编得过」，不是「这一份完整」。

带入口流程的是每族最后一份，而且是它自己在模块顶层起 main：`sys.exit(main())` 在 `.claude/hooks/runner-dispatch-guard-selftest.py:1924`、`.claude/hooks/completion-cleanup-guard-selftest.py:1344`、`.claude/hooks/project-code-guard-selftest.py:1543`、`.claude/hooks/bash-command-detector-selftest-entries-and-main.py:1710`；写闸是 `.claude/hooks/write-guard-selftest.py:1358` 起的顶层流程，收在 `sys.exit(code)`（`.claude/hooks/write-guard-selftest.py:1387`）。别的几份顶层没有 `sys.exit`、`main()`、`if __name__`（`grep -cE '^(sys\.exit|main\(\)|if __name__)'` 对 decisions、prompt-checks、locks-and-verdicts、waits-and-scope 各份都是 0）。由此三种状态：

| 状态 | 怎么到这一步（用户或 agent 可做的） | 引导 | 结果 |
|---|---|---|---|
| 带 main 的最后一份截成 0 字节，或删掉末尾那一行 `sys.exit(main())` | 主 agent 用 Edit 改 `*-selftest.py` 末尾时 old_string 带上了最后一行、new_string 漏写；Write 整份写了空内容；写一半撞满盘 | 编得过（空文件、少一行都编得过） | 前几份只定义函数，没人读 stdin、没人判：五条钩子都退 0、stderr 空、检出不记（推的，待主 agent 跑 main-part-empty 一列） |
| 第一份截成 0 字节，或少了一段被后面几份顶层引到的定义 | 同上 | 编得过 | 写闸、派发闸、完工清理闸在 main 之前抛 NameError、Traceback 退 1：Claude Code 当非阻断错误照常放行，这三条没有 bash 那一层的检出记录；项目代码闸、检出钩子由 bash 那一层兜成放行加一条检出（推的，待主 agent 跑 first-part-empty 一列） |
| 部分文件换成指向别的编得过的文件的符号链接 | 手工 `ln -sf` 或把草稿目录里的一份链进来 | `open` 跟着链接读，编得过 | 跑的是链接那头的版本；引导不核部分文件是不是本目录里的普通文件（没做探针，推的） |

写闸自己挡不住前两种：第六道只做 `ast.parse(content, filename=relative)`（`.claude/hooks/write-guard-decisions.py:685`），空内容与少一行都解析得过；第十一道只拦 `tooling-writer`，主 agent 改主工作区的钩子部分文件照放。`--selftest` 也看不出：最后一份空着时 `bash <钩子> --selftest` 同样什么都不跑、退 0（预期 `PROBE_HOOK_PART_SELFTEST … exit=0 output_lines=0`，推的，待主 agent 跑）。

拆分带进来的一点：拆前入口流程在 `.sh` 的 heredoc 里，拆后住在名叫 `*-selftest.py` 的那一份末尾（`.claude/hooks/write-guard.sh:347` 写的是「write-guard-selftest.py：自证 selftest、弄坏开关只在自证里认的拒绝与读 hook 输入的入口流程」）。改「自证」那一份的人不一定想到最后一行是整道闸的入口（推的）。

第一种状态在写闸、派发闸、完工清理闸、项目代码闸上的后果，与修补要防的「读不到时照常放行」同类而更静：读不到时还有退 1 与 Traceback、修补后有退 2 与点名，这一种连 stderr 都没有。第二种状态正是 `.claude/hooks/write-guard.sh:363` 那行注释说要消掉的「python 在 main 之前抛错退 1 而被照常放行」。

### 1b 三份闸拒一切调用时的出路：推不出（算打中）

逐条看拒绝说明给的出路、给谁：

- 写闸（挂 `"matcher": "Write|Edit|MultiEdit"`，`.claude/settings.json:104`）：说明是「把这一份恢复成编得过的版本（git 里取，或让主 agent 换回）再写」（`.claude/hooks/write-guard.sh:372`）。主 agent 走 Bash 走得通，但不是 `git checkout -- <部分文件>`：那条带路径撤没暂存的改动，检出钩子 ⑱ 拒（部分文件此刻只在索引里，`git cat-file -e HEAD:.claude/hooks/write-guard-decisions.py` 退 128，「exists on disk, but not in 'HEAD'」）；走得通的是 `git show :<路径> > <同目录临时文件>` 再 `mv` 换上（⑦ 的出路）。说明里没写这一条，照「git 里取」最先想到的那一条会被 ⑱ 拒一次（推的）。
- 项目代码闸（挂 Bash）：说明是「用 Edit、Write 把这一份恢复成编得过的版本」（`.claude/hooks/project-code-guard.sh:695`）。编不过的那一种，Read 出来用 Edit 修得回；读不到（删了、截空了）的那一种，Bash 全拒、`git show` 起不来，要写回去得先在盘上找到一份逐字节相同的拷贝（别的草稿目录里的副本）再整份 Write，没有就只能凭记忆重写上千行。能马上起作用的反而是 Write 一份只有 `import sys` 的空壳：引导编得过、整道闸退 0，这正是 1a 的第一种状态（推的）。
- 两道同时读不到：主 agent 剩 Read、Glob、Grep（只读）与 NotebookEdit（`.claude/settings.json` 里只有 root-entry-guard 与 code-length-guard 挂它，能不能写 `.py` 没核）；派出去的子 agent 挂同一套钩子。出路只在用户在 Claude Code 之外恢复，两份说明都指向对方那一道，没写这一种（推的）。
- 完工清理闸：引导读不到部分文件时 `sys.exit(2)`（`.claude/hooks/completion-cleanup-guard.sh:434`），这时 stdin 还没读，Stop、SubagentStop、SubagentHandback 一律退 2。正常判法在 `.claude/hooks/completion-cleanup-guard-decisions.py:1666` 认 `stop_hook_active`，文件头写的是「拦过一次、续跑回来（stop_hook_active 为真）还是同一张单（一个都没删）：放行，把单子作 systemMessage 交给用户处置（免得卡死）」（`.claude/hooks/completion-cleanup-guard.sh:120`）；引导这一支够不着它。子 agent 写不了 `.claude/hooks/`，交回（SubagentHandback）被拦、收工（SubagentStop）被拦，说明里的「让主 agent 换回」它递不出去，只能空转到主 agent 自己发现。主 agent 的 Stop 被拦回之后能用 Bash 恢复，走得通（探针 completion-cleanup-guard main-part-missing 一格预期退 2 而 intact 退 0，推的，待主 agent 跑；Claude Code 对 Stop、SubagentStop 退 2 是「拦下、把 stderr 交回模型接着做」，是我印象里的，没核）。

这一问的四句（`.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：
- 分不分辨臂：分辨修补前后两臂吗？不分。1a 的两种状态修补前后结局一样（修补只改了读不到、编不过那一支），它打中的是修补的射程，不是「修补有没有用」；修补对 main-part-missing 一列照样有用（预期退 2）。
- 被判的系统当时看不看得到判别它的东西：看得到。引导 exec 完全部之后，命名空间里有没有 `main`、有没有每份末尾的标记，是这一刻就能核的。
- 满足的是判据字面的哪一个分句：正文第 1 问「编得过而运行到一半缺名字」「读得到而 exec 时抛错」两个分句，与「三份闸…出路走不走得通」那一分句。
- 跑前条款给的改法在打中的格上还中不中：第一轮改法第 1 条（读不到、编不过按拒判）在 main-part-empty、first-part-empty 两格上不起作用；它只修 main-part-missing 那一格。

## 第 2 问：`lib_shell_words.py` 按目录各存一份：成立（没打中）

- 键是 `_LIBRARY_DIRECTORY = os.path.realpath(os.path.dirname(os.path.abspath(__file__)))`（`.claude/hooks/lib_shell_words.py:299`）：`abspath` 先按字面去掉 `..`、补全相对路径，`realpath` 再解目录上的符号链接。目录符号链接、`A/inner/..`、chdir 之后的相对路径，三种写法都归到同一个键，第二次载入从 `sys.modules` 的目录键取同一份 core 与 embedded，`ShellWord` 是同一个类（探针前四行预期 `same_core_as_absolute=True shellword_same_class=True`，推的，待主 agent 跑）。
- 唯一的边：`W/link/../lib_shell_words.py`，`link` 指向别的目录里的子目录。内核先解 `link` 再上一层，真执行的是那一头的入口；`abspath` 按字面去掉 `link/..`，键是 `W`，部分模块从 `W` 载入；核对 `if os.path.realpath(os.path.dirname(os.path.abspath(module.__file__))) != _LIBRARY_DIRECTORY]`（`.claude/hooks/lib_shell_words.py:336`）拿部分模块的目录比的也是这个字面键，所以入口与部分模块出自两个目录而不报（探针 symlink-then-dot-dot 一行预期 entry_mark=A core_mark=W，推的）。仓里载入它的地方都是 `os.path.join(<目录>, "lib_shell_words.py")`，目录取自 `os.path.dirname(os.path.abspath(__file__))`（`.claude/hooks/lib_project_code_runs.py:329` 那一族）或钩子入口 `$(cd "$(dirname "$0")" && pwd)`，都不会拼出「链接后接 `..`」；我没找到这样的调用方，不算打中。
- `isinstance` 判错的调用方：全仓（`.claude/hooks`、`scripts`、`.claude/gate.d`、`task-server` 下，样本档除外）对 `shell_words.<类>` 做 `isinstance` 的只有 `.claude/hooks/lib_project_code_runs_launchers.py:1095` 一处，词来自同一个模块自己的 `shell_words`（`.claude/hooks/lib_project_code_runs.py:329`）。检出钩子另载入一份 `lib_shell_words`（`.claude/hooks/bash-command-detector-scope-and-index.py:15`），把切出来的词交给 `lib_project_code_runs.command_under_two_logs`，两份都取 `sys.argv[1]` 那个目录，键相同、类相同。对 `shell_words.<名>` 做 `is` 比较的没有（`grep -rn ' is shell_words\.'` 零命中，`==` 只有 `.claude/hooks/project-code-guard-decisions.py:934` 比一个字符串常量）。
- 推翻它的观测：探针前四行里任一行 `same_core_as_absolute=False`；或找到一处从两个目录各载入一次、把一边切出的词交给另一边的调用方。

## 第 3 问：`lib_split_family_loaded.loaded_family`：不成立（打中）

### 3a 命名那一半在绝对路径下

`script_recipes` 只在 `if directory == GATE_STAGE_DIRECTORY:`（`scripts/gate/lib_lag_split_family.py:97`）时加门禁阶段 lib/ 拆法，`GATE_STAGE_DIRECTORY = ".claude/gate.d"`（`scripts/gate/lib_lag_split_family.py:63`）是仓内相对路径。入口给绝对路径时 directory 是 `/…/.claude/gate.d`，这一种拆法不加；Python 拆法与钩子拆法按入口自己的目录接前缀，绝对路径照认。`scripts/gate/lib_split_family_loaded.py:21` 写的是「入口与候选用同一种写法：都是绝对路径，或都是相对同一个根的路径」，`:7` 又把「门禁阶段 lib/ 下的 <名>-<部分>.sh|py」列进命名族：对门禁阶段，绝对路径那一种用法这一半不成立（探针 gate-stage-naming 预期 `in_family_relative=True in_family_absolute=False`，推的，待主 agent 跑）。

另一层：`sibling_files` 只列入口同目录（`scripts/gate/lib_split_family_loaded.py:104`），三处调用方都拿它当候选（`.claude/hooks/lib_project_code_runs_launchers.py:310`、`scripts/hosts/queue-health.py:172`、`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:700` 与 `:703`），lib/ 下的文件从来进不了候选；载入那一半也认不出门禁阶段最常见的写法（3b 表里最后一行，`.claude/gate.d/crash-point-replay.sh:894` 起四行的四份库都这样 source）。三层叠起来：任何门禁阶段经这三处调用方认族，族里都只有入口自己。

今天打不到判错：`source_family` 的入口是要临时服务的自证登记表里的行，表里没有 `.claude/gate.d/` 下的行（`grep -c '^\.claude/gate\.d/' .claude/hooks/selftest-temporary-service.tsv` 输出 0）；另两处的入口是 `task-server/task_server.py` 与两份共用模块。登记表哪天加了门禁阶段、记号又挪进 lib/，`temporary_service_table_problems` 那一支（`.claude/hooks/lib_project_code_runs_launchers.py:349`）会报「登记了 required，源码里却没有」（误报，推的）。

### 3b 载入那一半的几种写法

正则在 `scripts/gate/lib_split_family_loaded.py:48`、`:50`、`:51`，按整份文本扫，不分代码、注释与字符串：

| 写法 | 认不认 | 例（探针 case） | 后果方向 |
|---|---|---|---|
| 注释里引号写死同目录文件名 `# 'lib_demo_shadow.py' …` | 认成载入（`QUOTED_SCRIPT_NAME` 不看上下文） | quoted-name-in-comment | 多认 |
| 报错句、提示句的字符串里引号写死文件名 | 认成载入 | quoted-name-in-message | 多认 |
| 文档串或 heredoc 里缩进着的一行 `import <名>` | 认成载入（`PYTHON_IMPORT_LINE` 是 `^[ \t]*import`） | import-line-in-docstring | 多认 |
| 函数体里的 `import task_server_selftest`（`task-server/task_server_lib.py:1920`，只在 `--selftest` 那一支执行） | 认成载入 | task-server-entry | 自证各份进了 queue-health 读「源码写法」的族 |
| `from . import <名>` | 不认（文件头 `:19` 自己写了） | relative-from-import | 少认 |
| `importlib.import_module("<名>")`（不带 `.py`） | 不认 | import-module-by-name | 少认 |
| `source "$(dirname "${BASH_SOURCE[0]}")/<名>.sh"` | 不认：目录段里有空格与引号，`([^\s"';&|)]*/)?` 配不上，`QUOTED_SCRIPT_NAME` 又要引号紧贴文件名；文件头 `:18` 写的是「目录段是变量或没有」都认 | shell-source-dirname | 少认 |

`grep -rhoF 'source "$(dirname "${BASH_SOURCE[0]}")/'` 在 `scripts`、`.claude/gate.d`、`.claude/hooks` 的 `.sh` 里数出 114 处（89 份文件），多数 source 的是 `preflight.sh` 与共用库，拆出的部分文件这样 source 的有 `.claude/gate.d/crash-point-replay.sh:894` 到 `:897` 四行。

交集只挡得住「名字像、文本里一个字都没提」的那一种（第一轮第 4 条打中的）与「提到了、名字不像」的那一种（第 5 条打中的）；「名字像、只在注释或字符串里提到」照样进族。

### 3c 三处调用方各自因此判错的入口

- agent-write-scope 格：被判仓入口多一行注释 `# 'lib_shell_words_shadow.py' 是别的钩子留下的影子，不归这一份。`，影子里的手抄不报（第 4 问样本 name-like-red-quoted-comment，预期退 0，推的，待主 agent 跑）。
- `source_family`（要临时服务的自证登记核对）：登记成 required 的 `scripts/<目录>/foo.sh` 把 `selftest_service_problem` 调用挪进 `foo-steps.sh`，入口用 `source "$(dirname "${BASH_SOURCE[0]}")/foo-steps.sh"` 载入：族里只有入口，`.claude/hooks/lib_project_code_runs_launchers.py:349` 那一支报「登记写错了？」（误报；探针 shell-source-dirname 预期 family=[]，推的）。反方向：入口没有记号、注释里引号写着一份留在目录里的旧部分文件名而那份旧文件有记号，核对放过（漏报，推的，没做探针）。
- queue-health 的 attempt-forms-match-sources：族里带着 `task_server_selftest*.py`（探针 task-server-entry 预期 selftest_members 大于 0，推的）。今天两种写法只在 `task_server_scheduler_starting.py` 里（`grep -ln` 只命中这一份），所以今天判得对；自证哪天为了拼期望值写下同一个 f-string，真代码改了写法这一格也照绿（漏报，推的，没做探针）。

四句：分辨臂——分辨「只按载入、只按命名、交集」三种认法吗？3b 前三行在「只按载入」与「交集」两臂上一样多认，在「只按命名」臂上也多认，三臂一起中，病根在「载入」这个判别子是按文本扫出来的；判别子观测得到——是，`ast` 分得清代码与字符串、注释；满足的分句是正文第 3 问「注释与字符串里的 `import`、引号里写死的文件名恰好是别的脚本」与「调用方给绝对路径时这一半还认不认得」；第一轮第 4 条改法在这几格上不起作用。

## 第 4 问：agent-write-scope 格两族取法不同：不成立（打中）

格的取法：查的函数名取 `HOOKS_DIRECTORY="$(cd "$STAGE_DIRECTORY/../hooks" && pwd)"`（`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh:387`）那一族，`part_paths = sorted(family_library.loaded_family(module_path`（`:700`）；跳过的文件名取被判仓 `.claude/hooks/` 的族（`:703`），只在顶层按文件名跳（`if directory == hooks_directory and file_name in family_names:`，`:740`）。两族各自经 `loaded_family`，第 3 问的多认、少认分别落在两边，方向不同：

| 哪一族认错 | 怎么错 | 后果 | 布局（样本） |
|---|---|---|---|
| 被判仓的族多认 | 入口注释或字符串里引号写了影子文件名、文档串里有一行 `import <影子>` | 影子进跳过名单，里面的手抄不报：漏报 | name-like-red 加一行注释（name-like-red-quoted-comment）；加一个字符串常量（name-like-red-docstring-import）；预期都退 0，推的，待主 agent 跑 |
| 被判仓的族少认 | 入口用 `importlib.import_module("lib_shell_words_core")` 载入自己的部分文件 | 被判仓自己的部分文件被逐行扫，里面与这道脚本那一族同名的 def 报成手抄：误报 | green 加一份 core 与这一行（green-core-by-import-module），预期退 1 点名 `lib_shell_words_core.py:1`；换成 `from … import` 的对照预期退 0，推的 |
| 这道脚本那一族少认 | 共用模块哪天用认不出的写法载入一份新拆出的部分文件 | 那一份的函数名不进 `checked_names`（`:733`），别处手抄它们不报：漏报；成功行里「加 N 个部分文件」的 N 少一个，没有哪一格核它 | 没做样本：要改这道脚本旁边的 `.claude/hooks/`，样本仓碰不到（`:381` 有意这样设计）。已有的 parts-red 样本手抄的是 `brace_expand` 与 `classify`，只盖住住着这两个函数的 core 与 launchers，新拆的那一份没人盖（推的） |

`scripts/gate/lib_split_family_loaded.py:13` 写「读不了的文件当没有载入任何东西（族只会变小，各调用方都按族小了报得更多的方向用它）」。这句对读不了的文件成立（这一格对族里每一份 `ast.parse`，读不了就红），对认不出的载入写法不成立：这道脚本那一族因此变小时这一格报得更少，不是更多。

正文点的「真仓里部分文件刚加还没换上」：这一格在哪棵树里起，两族就取自哪棵树（`$STAGE_DIRECTORY/../hooks` 与当前目录的 `.claude/hooks` 是同一个目录），真仓自己判自己时两族相同，取法不同不起作用，错只来自 3b 的认法。两族真不同的是拿一棵树的门禁判另一棵树（第一轮主 agent 复跑就是 `bash .claude/gate.d/code-tooling.sh --check agent-write-scope <副本>` 这种起法）：副本里新加进共用族的函数不查（漏报），副本里从共用族挪出、只给一道钩子用的函数照查（误报）；这是 `:381` 为了「样本仓里放坏的模块碰不到它」付的代价，只在跨树判时出现，推的，没做样本。

四句：分辨臂——取法不同这件事本身不分辨「两族同源」与「两族不同」两臂（真仓里两族相同，错照出），打中的是共用前提（第 3 问的认法），判时把它记在第 3 问、这里只记两种方向；判别子观测得到——是；满足的分句是正文第 4 问「有没有一种布局让手抄漏报或误报」；第一轮第 4 条改法在 quoted-comment、docstring-import、import-module 三格上不起作用。

## 撤回第一轮攻方腿的方案

- 钩子与门禁那一题「三、改法」第 1 条（`research/prompts/split-governance-hooks-gates-r1-main-verification.md:41`）写的是「读不到、编不过任何一份部分文件时，照拆前缺依赖的处置按拒判」，并说「不让 python 在 main 之前抛错退 1」。我收窄它：照它做完，读不到、编不过这一支是堵上了，「编得过而不完整」（第 1 问 1a 前两行）照样静默放行或在 main 之前抛错退 1；它给的自证格「删掉一份部分文件，钩子退 2」盖不住这一种。这一条的「不让 python 在 main 之前抛错退 1」那半句，照写法实现不了，应撤回或换成下面零轮改法甲。
- 同一题第 4 条（`:44`）写「只按名字的多认（第 4 条）与只按载入的多认（第 5 条）都排掉」。这一句说宽了：交集排掉的是那两种具体布局；载入那一半按文本扫，「名字像、只在注释或字符串里提到」照样进族（第 3 问 3b、第 4 问前两行）。撤回「都排掉」的说法，改成「排掉第 4、5 条那两种布局；载入按文本认，注释、字符串、文档串里写到的同样算载入」。
- 同一题第 2 条（共用切词模块按带入口路径的唯一名字载入）不撤：第 2 问没打中。

## 改法（只在我的模型上推过、被攻过零轮，一格都没实现、没跑）

| 名 | 改什么 | 修哪几格 | 不修哪几格 | 量过还是推的 |
|---|---|---|---|---|
| 零轮改法甲 | 五条钩子的引导：每份部分文件末尾写一行登记（例 `HOOK_PARTS_LOADED.append("<文件名>")`），引导 exec 完全部之后核登记与 `*_PART_NAMES` 逐个相同、命名空间里有 `main`，由引导自己 `sys.exit(main())`（部分文件不再自己起 main）；exec 包进 try，接 `Exception`，点名正在 exec 的那一份，照各自读不到时的处置（三份闸与项目代码闸退 2，检出钩子退 70）；自证加两格：最后一份截空、第一份截空 | 第 1 问 main-part-empty、first-part-empty 两列；删末行 `sys.exit(main())` 那一种 | 符号链接指到别处的编得过的文件；整份换成另一份完整而错的版本 | 推的，待主 agent 跑 |
| 零轮改法乙 | 完工清理闸引导的拒判那一支先读 stdin：事件不是 PreToolUse 且 `stop_hook_active` 为真时照 `.claude/hooks/completion-cleanup-guard-decisions.py:1666` 那一支放行、说明作 systemMessage；三份闸的说明里给可照抄的 Bash 恢复写法（`git show :<路径>` 写同目录临时文件再 `mv`），项目代码闸的说明补「删了或截空、盘上又没有拷贝时请用户在 Claude Code 之外恢复」 | 第 1 问 1b 的完工清理闸卡死、说明里缺的那一条 | 两道同时读不到时主 agent 自己恢复不了（只能交用户） | 推的，待主 agent 跑 |
| 零轮改法丙 | `loaded_file_names` 对 `.py` 用 `ast` 认 Import、ImportFrom 节点与顶层赋给 `*_PART_NAMES` 的字符串元组，不扫注释、文档串与别的字符串；对 `.sh` 认 `source`/`.` 后面那个词的最后一个路径段（`$(dirname …)/` 前缀照认，带 `..` 的照旧不算）；门禁阶段 lib/ 拆法按路径里以 `/.claude/gate.d` 收尾的目录认，`sibling_files` 对门禁阶段另列 `lib/` | 第 3 问 quoted-name-in-comment、quoted-name-in-message、import-line-in-docstring、shell-source-dirname、gate-stage-naming 五格；第 4 问 quoted-comment、docstring-import 两份样本 | import-module-by-name（按名字现拼的载入照旧认不出）；第 4 问 import-module 样本的误报；`.sh` 入口 heredoc 里的 python 仍要按文本认 | 推的，待主 agent 跑 |
| 零轮改法丁 | agent-write-scope 格把这道脚本那一族的部分文件个数与一张写死的期望（两份共用模块各几份）比，对不上判红 | 第 4 问表第三行（这道脚本那一族少认时静默少查） | 被判仓那一族的认错 | 推的，没做样本 |

## 没打中的形状与取样范围

- 第 1 问，部分文件是目录（`IsADirectoryError`）、权限 000（`PermissionError`）、悬空符号链接（`FileNotFoundError`）、非 UTF-8（`UnicodeDecodeError` 是 `ValueError`）、含空字节（`compile` 抛 `SyntaxError` 或 `ValueError`）：都落进 `.claude/hooks/write-guard.sh:368` 那一组，按拒判；只按代码读，探针只放了「删掉」一种。部分文件换成 FIFO 时 `open` 阻塞到钩子超时，太牵强，不算。
- 第 1 问，两份共用库（`lib_project_code_runs.py`、`lib_run_scope.py`）的部分文件截空：库照常载入、调用方判的时候抛 AttributeError，照「判的时候抛了」那一支放行并记检出（项目代码闸是 `.claude/hooks/project-code-guard-selftest.py:1526`），与修补给它们定的处置同向（放行加检出），不算打中；没做探针。
- 第 2 问，五种写法与「同目录载入两次」「两个目录各一次」，全仓找 `isinstance(…shell_words.…)` 与 `is shell_words.` 两种用法（范围 `.claude/hooks`、`scripts`、`.claude/gate.d`、`task-server`，样本档除外）。
- 第 3 问，`from <名> import`、`*_PART_NAMES` 元组里写死的文件名、`source "$<变量>/<名>.sh"`：照认，是对照格。
- 第 4 问，真仓自己判自己：两族同源，取法不同不起作用。

## 这条腿自己的限度

- 一个探针都没跑，表里的预期全是推的；子 agent 不交任务，读数等主 agent 交。
- 冻结副本：`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy --with-test-inputs --with-gate-tables --with .claude/scripts=… --with .claude/singlefs-ai-sop=… --with scripts=…` 退 0，计划里 `.claude/hooks/` 只列了入口与几份库、没有各钩子的部分文件；我再 `rsync -a --exclude __pycache__ .claude/hooks/ <副本>/.claude/hooks/`（退 0），之后在副本里 `sha256sum -c research/prompts/split-governance-fixes-r2-snapshot/judged-sha256.txt` 数出 58 行 OK、非 OK 零行。闭包漏了钩子部分文件这一点与正文说的「算键闭包跨不过拆出的文件」是不是同一件，我没核。
- 探针的根用符号链接接冻结副本的其余条目，只有 `.claude/hooks` 是真拷贝；钩子里按真实路径判的地方（`realpath` 之后落进冻结副本）会看到冻结副本的路径。intact 一列是对照：对照对不上表里的值，那一条钩子整行作废。
- Claude Code 对钩子退 1、退 2 的处置（PreToolUse 退 1 照常执行、Stop 与 SubagentStop 退 2 拦下接着做），是我印象里的，这一轮没核；NotebookEdit 能不能写 `.py` 没核。
- 读了 `/home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/hooks-gates/tools/` 下第一轮主 agent 复跑用的两份写闸探针（只看怎么建臂），没读禁读清单里的五份报告。
- 第 4 问只造了被判仓那一侧的样本；这道脚本那一侧要改冻结副本的 `.claude/hooks/`，没做。

## 要主 agent 跑的命令

第 1 条先跑；第 2 到 6 条读第 1 条建在运行目录里的样本，第 1 条退 0 之后再交，五条之间互不依赖。

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus -- bash /home/fy5090/code/singlefs/research/prompts/split-governance-fixes-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus-rerun-run /home/fy5090/code/singlefs/research/prompts/split-governance-fixes-r2-opus-model`
   - 目录：`/home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus`（冻结副本 `copy` 在 `.deliverables` 写成交接）。
   - 看：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；`task-server/task log <号> --all` 里 `PROBE_HOOK_PART ` 20 行、`PROBE_HOOK_PART_SELFTEST` 5 行、`PROBE_SPLIT_FAMILY` 14 行（`PROBE_SHELL_WORDS` 6 行，载入失败的那一格打 `load_error=`，也算一行）、`SAMPLE` 5 行、`PROBE_HOOK_PART_SETUP` 0 行、末尾 `name=rerun_roots`；逐行对「跑前写死的判读表」。退非 0 是哪一份探针在哪一步出错，看日志最后的 Traceback。
2. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 agent-write-scope 格; 范围=1 格; 为什么=split-governance-fixes-r2 攻方腿第 4 问对照：原样的 name-like-red 样本上这一格该红并点名影子文件那一行' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus -- bash /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy/.claude/gate.d/code-tooling.sh --check agent-write-scope /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus-rerun-run/samples/name-like-red-unchanged`
   - 目录：同上。看：`TASK_SERVER_RESULT` 的 `exit=1`，日志里有 `.claude/hooks/lib_shell_words_shadow.py:4  def commands_at_command_position（共用模块里定义在 lib_shell_words.py）`。
3. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 agent-write-scope 格; 范围=1 格; 为什么=split-governance-fixes-r2 攻方腿第 4 问：被判仓入口注释里引号写了影子文件名时这一格还红不红' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus -- bash /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy/.claude/gate.d/code-tooling.sh --check agent-write-scope /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus-rerun-run/samples/name-like-red-quoted-comment`
   - 目录：同上。看：`exit=` 与日志里有没有 `lib_shell_words_shadow.py:4` 那一行；预期 `exit=0` 加 `✓ 写范围闸` 起头的成功行（漏报）。
4. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 agent-write-scope 格; 范围=1 格; 为什么=split-governance-fixes-r2 攻方腿第 4 问：被判仓入口的字符串常量里有一行 import 影子时这一格还红不红' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus -- bash /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy/.claude/gate.d/code-tooling.sh --check agent-write-scope /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus-rerun-run/samples/name-like-red-docstring-import`
   - 目录：同上。看：同第 3 条；预期 `exit=0`（漏报）。
5. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 agent-write-scope 格; 范围=1 格; 为什么=split-governance-fixes-r2 攻方腿第 4 问：被判仓入口按 import_module 载入自己的部分文件时这一格会不会把它报成手抄' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus -- bash /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy/.claude/gate.d/code-tooling.sh --check agent-write-scope /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus-rerun-run/samples/green-core-by-import-module`
   - 目录：同上。看：预期 `exit=1`，日志里有 `.claude/hooks/lib_shell_words_core.py:1  def brace_expand（共用模块里定义在 lib_shell_words_core.py）`（误报）。
6. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 agent-write-scope 格; 范围=1 格; 为什么=split-governance-fixes-r2 攻方腿第 4 问对照：被判仓入口按 from import 载入自己的部分文件时这一格判绿' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus -- bash /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy/.claude/gate.d/code-tooling.sh --check agent-write-scope /home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus-rerun-run/samples/green-core-by-from-import`
   - 目录：同上。看：预期 `exit=0` 加成功行。

用完之后：运行目录 `/home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus-rerun-run` 与冻结副本 `/home/fy5090/.local/share/singlefs/work/split-governance-fixes-r2-opus/copy` 由主 agent 删。

## 没做什么

- 不判辩方那两问（第 5、6 问），不替主 agent 采纳；四份改法一份都没实现。
- 没往统一任务服务交任务，判决要引的数一条都没复跑（命令在上一节）；副本上的数将来也只是副本上的。
- 没做「这道脚本那一族少认」的样本、没做 FIFO 与符号链接指到别处的部分文件那两种状态的探针、没核 Claude Code 对钩子退出码的处置与 NotebookEdit 的写法。
