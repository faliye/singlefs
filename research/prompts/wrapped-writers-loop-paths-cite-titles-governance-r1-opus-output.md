# 攻方腿报告：检出钩子循环变量、任务服务包着的写工具、退 10 两种成因、引文核对小节标题（wrapped-writers-loop-paths-cite-titles-governance-r1，2026-11-21）

被判对象读主工作区此刻那一版（四个草稿目录的 `new/` 与主工作区逐份 `cmp` 相同；改动 B 的 `new/` 里 `task_server_lib.py`、`task_server_lib_constants.py`、`task_server_scheduler_lifecycle.py` 三份与主工作区不同，是改动 C 后来又改了这三份，攻的是主工作区那一版）。改前取各草稿目录的 `base/`。这条腿没写 `<模型目录>` 与 `rerun.sh`：要跑的只有一份改前改后对照的小探针（改动 A），在草稿目录，命令在「要主 agent 跑的命令」。

## 各格判定一览

| 问 | 判定 | 打中的东西（一句） | 证据性质 |
|---|---|---|---|
| 1 改动 A | 成立（打中） | ⑫ ⑯ ⑲ 三道跟着共用段收严、自证一格没钉；⑦ 自己「经只赋过一次的名字引到循环变量」那一支没格钉；推导式里的循环照放、文件头「判不到的」没列；带条件只写新文件的循环被误拒 | 代码读的（推的）；探针待主 agent 跑 |
| 2 改动 B | 成立（打中） | 一个 `-c` 串或脚本正文里有两个写工具时只取头一个：后面的换钩子退 7、9、10 不提醒，被杀也不报；打补丁的补丁目录写成 `$p`、`"$1"` 或 `cd` 之后的相对路径时，结局文件找不到被当成「还没开始换」，被杀不报（改前照报）；`modifies_workspace` 判定没变 | 代码读的（推的），没量 |
| 3 改动 C | 成立（打中） | 两种拒绝说明的认法与 `hook_swap_swap.py` 现行原文对得上；可「草稿目录被占的另可换一个 `--scratch`」在服务提醒与 README 三语里都只对「两趟里有一趟是 `--dry-run`」成立，两趟真换共用一个草稿目录时照它换了会再退 10；异常表那一行开头一句只对退 9 与仓被占成立 | 代码读的（推的），没量 |
| 4 改动 D | 成立（打中） | 标题带括号的小节写法（照文件头逐字、带反引号）改后整条认不出、不判也不列进没判：伪造的原文照样过；改前是判红 | 量的（主工作区 `cite-check.py` 现跑，输出见第 4 节） |

四句（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`「打中之后先问四句」）这一轮的对象不是带臂的实验，按改法的出入答：分辨臂一句改答「打中的是不是这一轮的改动带进来的」，各节逐条写。

## 第 1 问：改动 A（检出钩子认 python 循环变量）

判定：成立（打中三处，另有一处误拒）。下面「推的」都是读主工作区代码推的，探针（`tools/loop_probe.py`，改前改后两份钩子目录喂同一批 26 种写法）还没跑，待主 agent 跑。

### 1a. ⑫ ⑯ ⑲ 跟着收严，自证一格没钉（推的）

共用段改了之后，五道认 python 写的目标都按循环变量的每个元素各判一次。⑨ 与 ⑭ 各加了一格（`bash_command_detector_selftest_waits_and_scope.py` 第 520、594 行），⑫ ⑯ ⑲ 没有一格带 `for … in (…)` 的 python（`grep -n "for [a-z_]* in [(\[{]"` 扫三份自证文件，命中的循环格只在第 520、594、1880–1921、1931 行，都是 ⑦ ⑨ ⑭ 的）。三道各自在哪一行吃这一段：

- ⑫ 写范围与 ⑭ 服务源码共用：「for write in python_in_place_writes(step.code, every_write=not overwrite_only, service_writes=service_judged):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:1727`）。改前循环变量算不出、不交写，这一道照放；改后候选里有一个落在写范围外就拒。判据变窄（多拒），没格钉。
- ⑯ 往现行账本追加节标题：「refused += [f"{write.form} {relative}" for write in python_in_place_writes(step.code, every_write=True) if lands_on_ledger(write.path, base, ledger)]」（`.claude/hooks/bash_command_detector_selftest_entries_and_main.py:1760`）。同上，变窄，没格钉。
- ⑲ 往 records 追加：「for write in python_in_place_writes(step.code, every_write=True):」（`.claude/hooks/bash_command_detector_selftest_entries_and_main.py:1777`）。变窄，没格钉；探针 `records-append-loop` 一格喂的就是它（改前应 0 条、改后应 1 条，推的）。
- ⑭ 另认的删、挪走、`os.open`、`copytree` 几支（`python_service_writes`）也改成按 `choices` 逐个取值，只有 `open(…, 'w')` 一格（第 594 行）；那几支把 `choices` 接错不会有格红。

弄坏开关 `python-loop-variable-unexpanded` 打开后红的六格（`.claude/hooks/bash-command-detector.sh` 第 733–735 行）里同样没有 ⑫ ⑯ ⑲ 的；开关名字说的「循环变量不按元素逐个代入」在这三道上发生了也不红。

### 1b. ⑦ 自己：「经只赋过一次的名字引到循环变量」那一支没格钉（推的）

文件头 ⑦ 写「循环体里拿它（或经只赋过一次的名字引到它）当目标的」（`.claude/hooks/bash-command-detector.sh:223`）。走这一支的是两行：「return python_literal_path(assignments[node.id], assignments, depth + 1, bound)」（`.claude/hooks/bash_command_detector_literals_and_signals.py:377`）与「nested = [item.id] if item.id in loops else referenced_loop_names(assignments.get(item.id), assignments, loops, depth + 1)」（`.claude/hooks/bash_command_detector_literals_and_signals.py:396`）。新加的四格该拒里，目标都是循环变量本身或 `os.path.join('scripts', name)`，没有一格是「循环体里 `t = os.path.join(…, name)`，再 `open(t, 'w')`」。把第 377 行的 `bound` 去掉、或第 396 行不往赋值里找，四格照绿。探针 `body-assignment-references-loop-variable` 喂的就是这一形。

### 1c. 仍被 ⑦ 放过的循环写法（推的，探针待跑）

文件头「判不到的」（`.claude/hooks/bash-command-detector.sh` 第 226–228 行）列了：元素算不出或带星号、名字另有绑定、目标不是单个名字、可迭代对象不是字面量（range、推导式、变量里的列表）、循环体之外（含 else 段）、组合多过 `MAXIMUM_LOOP_LANDINGS`。下面这几形照放：

| 写法（探针格名） | 照放的原因 | 文件头列了没 |
|---|---|---|
| `[open(f, 'w').write('x') for f in ('notes.md', '<仓>/scripts/running.sh')]`（list-comprehension-loop） | 推导式的 `for` 子句不是 `ast.For`，`literal_loop_candidates` 不看 | 没列：文件头说的「推导式」是可迭代对象是推导式，不是循环本身写成推导式 |
| `list(map(lambda f: open(f, 'w').write('x'), (…,)))`（map-lambda） | lambda 的参数算一次绑定、也不是 for | 没列 |
| `for f in {'<仓>/scripts/running.sh': 1}:`（dict-literal） | 只认元组、列表、集合 | 没明列（字典也是字面量） |
| `names = (…); for f in names:`（iterable-in-single-assigned-name） | 可迭代对象是名字 | 列了（「变量里的列表」） |
| `for a, b in …`、`enumerate(…)`、else 段、外层循环变量拼内层元素、66×66 组合 | 同文件头 | 列了 |
| `'<仓>/scripts/' + n`、f 串拼目标 | 加号与 f 串不在认得的路径形态里（改前就这样） | 不是这一轮的事 |

打中的是推导式那一形：与起因（heredoc 里逐个改两份脚本）同一件事，写法只差把 `for` 写进方括号，现在照放，文件头又没写它判不到，读文件头的人会以为认了。

### 1d. 该放的被误拒（推的）

候选全取、不看循环体里的条件：`for f in ('<仓>/scripts/running.sh', '<仓>/scripts/new-step.sh'):` 下面 `if os.path.exists(f): continue` 再 `open(f, 'w')`（只写不存在的新文件，探针 exists-guard-writes-only-new-files），与 `if f.endswith('.md'):` 才写（suffix-guard-writes-only-markdown），改后都拒，改前都放。只赋过一次的名字一路早就不看条件（`p = '<脚本>'; if False: open(p, 'w')` 照拒），这一形是同一类在循环上新出的实例，代价是照拒绝说明改成临时文件再 `mv`。按「打中的是不是这一轮带进来的」答：是，但属于已接受的那一类放大。

### 1e. 新开关红在名字说的事上（推的）

`literal_loop_candidates` 开头开关一打开交空表，`python_path_choices` 走「没有循环变量」那一支，循环变量照算不出，⑦ 三格该拒、拒绝点名一格、⑨ ⑭ 各一格变放行，正是文件头列的六格；六格该放的在开关下照放、不红。主 agent 的开关复跑「逐格相同 173 条」（背景材料，推的）与这一读法相合。缺口只是 1a 说的 ⑫ ⑯ ⑲ 没格。

四句：打中的都是这一轮带进来的（1a、1b 是新共用段没钉，1c 推导式是新文件头没写全）；被判的系统（钩子）判那一刻看得到整段 python 的语法树，推导式与条件都看得到；满足的是第 1 问「仍被放过」「被误拒」「自证没格钉着」三个分句；修法见第 5 节。

## 第 2 问：改动 B（任务服务三处提醒认包着的写工具）

判定：成立（打中两形）。`modifies_workspace` 判定没变（2c）。全是读代码推的，任务服务的判定函数子 agent 载不进来（项目代码闸拒载 `task_server_lib`），没量；复现要主 agent 照 2d 加自证格。

### 2a. 一条命令里有两个写工具，只认头一个（推的）

`wrapped_workspace_writer` 在 `-c` 串里认到头一条就交：「return inner, given_cwd」（`task-server/task_server_scheduler_building.py:1291`），脚本正文同样「return found, given_cwd」（`task-server/task_server_scheduler_building.py:1299`），文件头写明「先 ① 后 ②，各自照先后取头一条认到的。」（`task-server/task_server_scheduler_building.py:1281`）。三处提醒都只拿这一条：

- 换钩子认不认：「return HOOK_SWAP_ENTRY in self.repository_paths(words, os.path.realpath(cwd))」（`task-server/task_server_scheduler_lifecycle.py:1649`）。
- 被杀时写工具与路径：「command, cwd = list(found[0]), found[1]」（`task-server/task_server_scheduler_lifecycle.py:1738`）。

能照喂的任务命令（`--cwd` 仓根；`<W>` 是工作根下一个草稿目录的绝对路径）：

```
bash -c 'python3 scripts/gate/apply-writer-patch.py <W>/patch && python3 scripts/gate/hook-swap.py --spec <W>/swap.json --scratch <W>/swap-scratch'
```

- 打补丁打全了、换钩子退 7（或 9、10），`bash` 照退同一个码：头一条认到的是 `apply-writer-patch.py`，`writer_is_hook_swap` 交 False，命令那一层也没有 `hook-swap.py`，workspace-write-blocked 与 previous-swap-running 都不出（漏报）。改前同样不出，不算回退，但这正是改动 B 要认的「包着的换钩子」。
- 换钩子换到一半被杀（退 137 或撞上限）：`workspace_write_judged` 取的是打补丁那一条，补丁目录的结局文件写着完成，`--read-outcome` 退 0，走「if verdict in (0, 1):」（`task-server/task_server_scheduler_lifecycle.py:1770`）交 None，workspace-write-interrupted 不出。改前写工具取命令那一层的 `bash`、不读结局文件，照报「可能写了一半」。这是回退：换钩子停在半截而服务一声不响。
- 退 7 找上一趟时，库里这样一条被杀的任务同样点不出（同一个 `writer_is_hook_swap`）。

### 2b. 补丁目录不是字面路径，结局文件找不到被当成「还没开始换」（推的）

被杀时打补丁那一支：「outcome_file = os.path.join(cwd, named[0], PATCH_WRITER_OUTCOME_NAME)」（`task-server/task_server_scheduler_lifecycle.py:1757`），文件不在就「return None   # 结局文件在动手换之前写：不在就是还没开始换，主工作区没动」（`task-server/task_server_scheduler_lifecycle.py:1759`）。`named[0]` 是切出来的词原样，`$p`、`$1` 不展开，`cwd` 是任务的 `--cwd`、串里的 `cd` 不跟。三形都让这一句为真而主工作区其实可能停在半截：

1. `-c` 串里经变量给补丁目录（`--cwd` 仓根）：`bash -c 'for p in <W>/patch-a <W>/patch-b; do python3 scripts/gate/apply-writer-patch.py "$p"; done'`。切出的那一条是 `do python3 scripts/gate/apply-writer-patch.py $p`（① 不剥 `do`，可词里有写工具照认），`named[0]` 是 `$p`，找的是 `<仓根>/$p/apply-outcome.tsv`。
2. 脚本正文经位置参数给：`bash <W>/apply-chain.sh <W>/patch`，正文一行 `python3 scripts/gate/apply-writer-patch.py "$1"`。`named[0]` 是 `$1`。新自证格 `wrapped-writer-interrupted-judged` 自己就期望脚本正文那一条点名「$1、swap-scratch」，说明 `$1` 原样进 `named`；换钩子那一支只拿它点名，不出事，打补丁这一支拿它找文件就出事。
3. `cd` 之后写相对补丁目录（`--cwd` 仓根）：`bash -c 'cd <W> && python3 /home/fy5090/code/singlefs/scripts/gate/apply-writer-patch.py patch'`。找的是 `<仓根>/patch/apply-outcome.tsv`。`--cwd` 下另有一份旧的、打全了的同名补丁目录时，读的是那一份、判打全了，同样不报。

三形改前都照报（写工具写成 `bash`，不读结局文件）。按第 2 问字面：`cd` 不跟那一条确实让结局文件那一支找错文件，后果是漏报而不是多报。

### 2c. `modifies_workspace` 没变（推的）

`workspace_swap_tool` 改成「return self.workspace_writer_command(task) is not None」（`task-server/task_server_scheduler_building.py:1251`）。命令那一层判出写的交整条、判出只读交 None：「return (list(task["command"]), cwd) if verdict else None」（`task-server/task_server_scheduler_building.py:1260`），与改前的 `return verdict` 真假一致；判不出的走 `wrapped_workspace_writer`，改前交 True 的每一处改后交一个元组，改前交 False 的每一处改后交 None，两个弄坏开关的早退也一一对上；`script_body_workspace_writer` 交的 `spoken` 只在 `position < len(spoken)` 时交，不会是空表。逐分支对过，真假一致；能推翻它的是：哪一条命令改后 `workspace_swap_tool` 与改前不同（主 agent 的 447 格自证里撞上限重排那几格都绿，背景材料，推的）。

### 2d. 三格新自证与三条开关

三格（`task_server_selftest_memory.py` 的 `cell_wrapped_hook_swap_reminded`、`cell_wrapped_previous_swap_named`、`cell_wrapped_writer_interrupted_judged`）每一条命令都只有一个写工具、补丁目录都是字面相对仓根；三条开关各自只改取词那一处，读代码看红在名字说的事上（主 agent 复跑「逐格相同 11 条」，背景材料，推的）。2a、2b 两形没有格：要证红，加两格（推的，待主 agent 跑）：同一个 `-c` 串里先打补丁（`patch-full`）后换钩子、退 137，应出 workspace-write-interrupted 且写工具写 `hook-swap.py`；`-c` 串里 `apply-writer-patch.py "$p"`、退 137，应出 workspace-write-interrupted（不交 None）。

四句：2a 被杀那一支与 2b 是这一轮带进来的回退（改前照报）；服务判那一刻看得到整条命令与 `-c` 串，看得到词里有 `$`、有 `cd`、有第二个写工具，判别的东西在手上；满足的是第 2 问「取错词或工作目录，让三处提醒报错或漏报」的漏报分句。

## 第 3 问：改动 C（换钩子退 10 分两种成因）

判定：成立（打中在出路文字，认法本身没打中）。读代码推的，没量。

### 3a. 认法对得上现行原文（没打中）

`SWAP_BUSY_SCRATCH_LINE` 与 `SWAP_BUSY_REPOSITORY_LINE`（`task-server/task_server_scheduler_lifecycle.py` 第 34 行起）对 `scripts/gate/hook_swap_swap.py` 的 `scratch_busy_lines`（第 512 行起）与 `repository_busy_lines`（第 557 行起）逐字对过：两句起头、全角括号都对得上；`holder` 读不全、正在摘旧记录（`removing`）、另一台持有时，起头那一句不变，只是 `who` 与多出的一行不同。退 10 只有这两处出（`EXIT_SCRATCH_BUSY` 的三处 `return`：第 398 行 `--seal-outcome` 走仓那一种、第 930 与 945 行）。

- 两个都出现：取最后一处。同一个日志里出两种，只在一条命令里接连起了两趟换钩子（不带 `set -e` 的 `;` 串）时，退出码跟最后那一趟，取最后一处与退出码对得上。
- 只出现一个、日志被截断：`log_tail` 取末 65536 字节（`task_server_constants.py` 的 `LOG_TAIL_BYTES`），拒绝说明在退出前最后几行；截断只会让起头丢掉、落到「两种都可能」，不会认错。
- 日志在第二台：`log_tail` 读不到交空串，落到「两种都可能」，不会认错。
- 没钉住的：自证样本是手抄的现行原文（`swap_busy_log_lines`，`task-server/task_server_selftest_notifying.py` 第 1303 行起），不调 `hook_swap_swap.py` 的那两个函数；那边改了起头一句，服务认不出而自证照绿。这是漂移没人看，不是今天认错（推的）。

### 3b. 只对一种情形成立的句子（打中）

「换一个 `--scratch`」只在两趟里有一趟是 `--dry-run` 时有用。真换先占草稿目录、再占仓：「held, busy_lines = hold_scratch(scratch, mode)」（`scripts/gate/hook_swap_swap.py:928`），之后「held, busy_lines = hold_repository(outcome_file, scratch, mode)」（`scripts/gate/hook_swap_swap.py:943`）。两趟真换给了同一个 `--scratch`，后一趟退 10、日志是「✗ 草稿目录」那一种；照提醒换一个草稿目录重交，它占到新草稿目录、再占仓被拒，又退 10（仓被占那一种）。hook-swap 自己的出路写了这一层：「真换在同一个仓里一次只许一趟，另一趟真换在跑时换草稿目录也会被拒」（`scripts/gate/hook_swap_swap.py:514`），服务与 README 把它丢了：

- 服务提醒（草稿目录被占那一项）：「这一种等那一趟结束再交，或换一个 --scratch（要并行的几条各给一个草稿目录）」（`task-server/task_server_scheduler_lifecycle.py:1698`）。
- README 中文：「退 10 而是草稿目录被占的另可换一个 `--scratch`（仓被占的换草稿目录没有用）」（`task-server/README.md:174`）。
- README 英文：「for exit 10 caused by a held scratch directory, to give a different `--scratch`」（`task-server/README.en.md:174`）。
- README 日文：「10 で作業ディレクトリが占有されている場合は別の `--scratch` を指定してもよい」（`task-server/README.ja.md:174`）。

服务判那一刻看得到这一条自己带不带 `--dry-run`（命令在手上）；看不到占着的那一趟是不是 `--dry-run`（拒绝说明里没写）。所以这一句可以照「这一条是 `--dry-run` 的才叫换」收窄，或照抄 hook-swap 那一句限定。

另一句：异常表 previous-swap-running 那一行的出路开头「等上一趟写完成再把同一条命令原样交一遍」（`task-server/task_server_lib_constants.py:129`），接着「等那个进程退出、python3 scripts/gate/hook-swap.py --read-outcome 读得到它写了完成再交」（`task-server/task_server_lib_constants.py:129`），没写「退 9 的」。退 10 的那一项出路也以这一整行起头（自证 `swap_busy_problems` 要求 `item["next"]` 以它起头）。占着草稿目录的是一趟 `--dry-run` 时，它不写结局文件，「读得到它写了完成」这个信号永远等不来；这一句只对退 9 与仓被占成立。同一行后半「退 10 的等占着的那一趟结束再交」是对的，两句并排，前一句管不管退 10 要读的人自己猜。

### 3c. README 改的两句与代码说的是一回事（没打中）

「服务照最后一次尝试日志里的拒绝说明分这两种，认不出的写两种都可能」与 `swap_busy_cause`、`swap_busy_item` 第三支一致；「仓被占的换草稿目录没有用」与仓那一支的出路一致。三语之间的意思对得上（逐句对过中、英、日第 174 行那两段）。不一致的只有 3b 那一处，三语一起丢了限定。

四句：3b 是这一轮新写的句子（改前退 10 只有一种出路）；服务看得到这一条带不带 `--dry-run`，看不到对面那一趟；满足的是第 3 问「提醒文字里还有没有一处只对一种成因成立」分句（严格说是只对其中一个子情形成立）。

## 第 4 问：改动 D（引文核对小节标题取全）

判定：成立（打中：标题带括号的整条认不出）。量的：主工作区 `scripts/agents/cite-check.py`（与 `cite-check-title-lazy/new/` 逐字节相同）直接跑在草稿目录的 `cite-forms.md` 上（这份工具许子 agent 直接跑，`.claude/hooks/lib_project_code_runs_base.py` 第 820 行）。

探针文件 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-opus/cite-forms.md` 十行，每行一种写法（T1 到 T10，原文在文件里，第 3 行起）。跑法 `cd /home/fy5090/code/singlefs && python3 scripts/agents/cite-check.py <草稿目录>/cite-forms.md`，退 1，原样输出整份存在同目录的 `cite-forms.out`（不贴进本报告：贴进来会被本报告自己的核对当成引文再判一遍），倒数第二行原样：

```
  ✗ 核了 11 处引文、0 处裸行号，7 处对不上，0 处没判（其中裸行号没有引文可比 0 处；逐处列在上面）
```

`cite-forms.out` 里列的 7 处是文件第 3 行（T1）、第 5 行（T3，两处）、第 6 行（T4）、第 10 行（T8，两处）、第 12 行（T10）；第 4 行（T2）与第 7、8、9、11 行（T5、T6、T7、T9）不在里面。

读法：

- 文件第 4 行（T2，标题带全角括号、带反引号，原文是伪造的）一处都没列：不在「对不上」里，也不在「没判」里，十行数到 11 处引文、少的就是它。原样（这一行在本报告里同样认不出，所以照抄不会让本报告的核对红）：`「这一句是伪造的，节里没有」（`.claude/rules/machine-first.md@放宽的（人脑带宽的产物）`）`。`.claude/rules/machine-first.md` 第 76 行的标题就是「放宽的（人脑带宽的产物）」，照文件头「标题不带井号、逐字」与「带反引号的取到反引号为止」（转述 `scripts/agents/cite-check.py` 第 14 行）是合法写法。原因：标题字符类排除了「（」，`TITLE_END` 的几种结尾里只有右括号，取到「放宽的」之后既不能往下也找不到结尾，`@` 那一支整条失败（`scripts/agents/cite-check.py:68`「TITLE_END = "" if BROKEN == "title-lazy" else」）。改前懒匹配取两个字「放宽」、标题命中 0 次、判红（推的：照改前正则推，改前那一版子 agent 不能直接跑）。改后这一形从「判红」变成「看不见」：交回闸跑的是同一个脚本，三方腿在这种标题下写伪造的原文照过。仓里 `.claude/rules/`、`.claude/singlefs-ai-sop/rules/` 下标题带括号的有好几处（`grep -n "^#.*[（(]"` 列到 `.claude/rules/machine-first.md` 第 76 行、`.claude/rules/changelog-format.md` 第 11 行与几份规则的一级标题）。
- T3、T8（标题带「」）：改后标题取到「之前、把标题里的「…」当成原文另记一处，两处都红；改前懒匹配取两个字，同样红。改前改后都引不了，不是这一轮带进来的，可它说明文件头「带反引号的取到反引号为止」对带「」的标题也不成立。
- T4、T10（不带反引号、标题后面跟着别的字）：改后标题吞进「 那一节」「，第二段」判红；改前取两个字「多轮」同样判红。文件头写了不带反引号取到括号或行尾，照现状。
- T5、T6、T7、T9（位置在前隔空白不带反引号、位置在前带反引号、行尾有句号、括号里前面有「见 」）都对上、不列，取全了。

开关：`title-lazy` 下 `TITLE_END` 是空串，正则回到改前，引文在前的两种写法交两个字（带反引号的、不带反引号的两格红），三种写法交同一标题那一格红；位置在前那一格在改前正则下也取得全（懒匹配要接上后面的「：「」才停），不红。文件头点名的正是这三格，读法相合；主 agent 的开关复跑「逐格相同 9 条」（背景材料，推的）。T2 这一形没有格钉。

四句：T2 是这一轮带进来的（改前红、改后看不见）；被判的脚本那一刻看得到整行，括号就在那里；满足的是第 4 问「合法写法下取不到」分句。

## 第 5 节：改法（只在我的读法上推过、被攻过零轮）

全部没实现、没跑，格里一律「推的」；主 agent 要用照「腿提的、判决里标「零轮」的改法」先做 A/B。

| 打中的格 | 改法 | 修哪一格 | 量过没 |
|---|---|---|---|
| 1a ⑫ ⑯ ⑲ 没格 | 三道各加一格：clerk 在 python heredoc 里 `for f in ('<草稿>/a.md', '.claude/kb/x.md'): open(f, 'w')` 拒；往现行账本写节标题的循环拒；`for f in ('<草稿>/a.md', 'records/x.md'): open(f, 'a')` 拒；三格挂进 `python-loop-variable-unexpanded` 的红格清单 | 只钉 1a，不改判据 | 推的，待主 agent 跑 |
| 1b 经名字引到循环变量 | ⑦ 加一格：循环体里 `t = os.path.join('scripts', name)` 再 `open(t, 'w')` 拒 | 只钉 1b | 推的，待主 agent 跑 |
| 1c 推导式 | 二选一：`literal_loop_candidates` 另认 `ast.comprehension`（目标是单个名字、`iter` 是字面量、调用在 `elt` 里）；或文件头「判不到的」写上「推导式与 lambda 里的循环」 | 前者修 1c，后者只让文件头照现状 | 推的 |
| 1d 带条件只写新文件被误拒 | 不改（与只赋过一次的名字同一类放大），文件头 ⑦ 写一句「不看循环体里的条件，候选里有已有脚本就拒」 | 只让文件头照现状 | 推的 |
| 2a 两个写工具只认头一个 | `wrapped_workspace_writer` 交认到的每一条（列表）；`writer_is_hook_swap` 判任一条是换钩子；`workspace_write_judged` 对每一条各判一次、任一条没打全就报 | 修 2a 两支 | 推的 |
| 2b 补丁目录不是字面路径 | `named[0]` 带 `$`、命令替换或串里前面有 `cd` 时不交 None，照改前报「可能写了一半」、点名原词；只在补丁目录是字面且工作目录算得出时才信「不在就是还没开始」 | 修 2b 三形；不修 2a | 推的 |
| 3b 换 `--scratch` 的限定 | 服务那一句与 README 三语加 hook-swap 那一句限定（真换在同一个仓里一次只许一趟），或照这一条带不带 `--dry-run` 分写；异常表那一行开头那一句前面加「退 9 的」 | 修 3b 两句 | 推的 |
| 4 括号标题看不见 | 带反引号的位置里标题字符类只排除反引号与换行（取到反引号为止，与文件头那一句一致）；不带反引号的照现状；另加一格「带括号的标题、带反引号、原文伪造 → 判红」 | 修 4；T3、T8 带「」的标题在带反引号时一并能引 | 推的 |

## 没打中的形状

- 改动 A：嵌套循环（两层都字面、目标用两个循环变量拼）、`with` 与 `try` 嵌在循环体里、函数体里的循环、集合字面量、`Path(…) / 循环变量` 的 `write_text`、`shutil.copy(循环变量, 已有目录)`、`python3 -c` 一行写法，读代码都按每个元素判、拒得对；目标不是脚本、候选里有新文件、循环体外的同名用法、临时文件再 `os.replace` 的写法都放；同名另被推导式绑定的照放（文件头列了）。组合上限：`python_path_choices` 超过 `MAXIMUM_LOOP_LANDINGS` 交全 None（第 409 行），⑦ 照放，与文件头一致；⑫ 对同样超限的 shell 循环是「算不出照拒」（第 1745 行），python 那一支超限是不判，两边不对称，可 python 算不出的目标在 ⑫ 一向不判，不是这一轮带进来的。⑤ 照文件头不走这一段（`results_overwrite_verdict` 遇 `PythonCode` 直接跳过）。没想出 `def`、`class`、`except … as` 与同名循环变量重绑定之外让代入值错的写法；这几种绑定 `name_binding_counts` 不数，可要让它们撞上一个写脚本的循环变量，写法已经不像真用的。
- 改动 B：`--feed-only` 在 `-c` 串里与命令那一层两处都排除；单写工具的 `-c` 串、脚本正文两形取的词对；`git -C <工作目录> status` 那一句提示用的是任务的工作目录，改前改后一样。
- 改动 C：认法、截断、第二台、两种都出现（3a）。
- 改动 D：T5、T6、T7、T9 四形取全；全角冒号在标题中间（`多轮：一次打穿…`）取全，没有被「：「」结尾提前截断。

## 这条腿自己的限度

- 改动 A 的判定全是读代码推的，探针没跑（子 agent 不喂钩子）；探针只调 ⑦ 与 ⑲ 两道的判定函数，⑫ ⑯ 要写范围表与登记表，没进探针，靠读第 1727、1760 行。
- 改动 B、C 的判定函数子 agent 载不进来，一格没量；2a、2b 的结论推翻条件：主 agent 照第 2 节 2d 加的两格在主工作区那一版上绿（即服务照报）。
- 改动 D 只量了改后那一版；改前那一版的读数是照改前正则推的。
- 只攻了分给我的第 1 到 4 问，正推腿的第 5、6 问没碰；禁读的另一条腿报告没读。
- 背景材料里主 agent 的复跑读数（1589 种、173 条、447 格、11 条、9 条）没复核，照「推的」引。

## 要主 agent 跑的命令

1. 改动 A 的改前改后对照探针（只调 ⑦ 与 ⑲ 的判定函数，不起钩子入口、不读写判定库、不起任务服务；只写草稿目录下的 `probe-work/`、`probe-base.out`、`probe-new.out`，`probe-work/` 跑完删掉）：

   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-opus -- bash /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-opus/tools/run-loop-probe.sh`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-opus`（`.deliverables` 里 `probe-base`、`probe-new`、`tools` 写成交接）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 末尾两行 `done	base	26`、`done	new	26`（两份各跑完 26 种）；表里每一行「写法、改前拒的条数、改后拒的条数」。按读代码推的应当是（推的，待跑）：改前 0、改后 1 的是 nested-loops-join、try-in-body、with-in-body、body-assignment-references-loop-variable（1b）、exists-guard-writes-only-new-files 与 suffix-guard-writes-only-markdown（1d 误拒）、copy-into-directory-loop-source、set-literal、loop-inside-function、path-division-write-text、records-append-loop（1a 的 ⑲）、one-line-python-c；改前改后都是 1 的是 fixed-target-inside-big-loops、records-append-single-name（对照）；其余十一种改前改后都是 0，其中 list-comprehension-loop 与 map-lambda 是 1c 的打中。哪一行与这里不同，就是这条腿推错了那一格，照实际读数改判。

2. 改动 B 的两格证红（2d）要先在副本里写成 `task_server_selftest_memory.py` 的新格才能跑，这条腿没写（写范围不含任务服务的副本，也载不进 `task_server_lib`）；要不要写交主 agent 定。

## 没做什么

- 没往任务服务交任务；探针、自证、开关复跑一条没跑。改动 A 的十三格「改后 1」读数、改动 B 与 C 的全部结论都是读代码推的。
- 没写模型目录与 `rerun.sh`：要跑的只是一份对照探针，不是崩溃状态模型。
- 没碰判定库，也没写会读写判定库的命令（门禁的开关复跑、按条复用的格）。
- 不判正推腿那几格，不替主 agent 采纳。
- 草稿目录里留着：`probe-base/`、`probe-new/`（各约 5 MB，`.claude/hooks` 的两份拷贝，不是仓副本，没有 `.git` 与编译目录）、`tools/`、`cite-forms.md`、`cite-forms.out`、`a.diff`、`b.diff`、`c.diff`；主 agent 跑完探针、判决引完数之后整目录删。没建仓副本与编译目录。
