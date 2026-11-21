# 正推腿报告：四处协作工具改动（wrapped-writers-loop-paths-cite-titles-governance-r1）

日期 2026-11-21。腿：three-way-forward（Sonnet）。分工：第 5 问（「实现今天的样子」逐条）、第 6 问（工具员报告里写「推的」「没读」的几处）。

## 读到的范围与前提

- 被判 18 份都读主工作区此刻那一版：`sha256sum -c research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-snapshot/judged-sha256.txt` 我跑了，`grep -c ': OK$'` 数得 18（量的），`wc -l` 该清单 18 行。
- 四个草稿目录（`~/.local/share/singlefs/work/` 下 `detector-loop-target-paths`、`task-server-wrapped-writer-reminders`、`swap-exit-ten-two-causes`、`cite-check-title-lazy`）：`new/` 与主工作区逐份 `cmp`：检出钩子四份、换钩子退 10 五份、引文核对一份相同；任务服务三处提醒那一份里 `task_server_lib.py`、`task_server_lib_constants.py`、`task_server_scheduler_lifecycle.py` 三份与主工作区不同，是退 10 那一批接着改了它们（量的：三份 `cmp` 打 DIFF，退 10 草稿 `new/` 里同名三份与主工作区相同）。`new/` 里的 `.pyc` 与主工作区不同，不算。
- 四份工具员报告里只有三份在：`detector-loop-target-paths/result.md`、`task-server-wrapped-writer-reminders/result.md`、`swap-exit-ten-two-causes/result.md`。`cite-check-title-lazy/` 里没有 `result.md`，`find` 不到任何名字带 result 的文件（量的）。引文核对那一处的说法只剩背景材料正文与调度记录可对，下面第 6 问写成「没有报告可查」。
- 我没起任何项目脚本、没交任务。要起脚本才核得了的格，写在末尾「要主 agent 跑的命令」，那几格写「待主 agent 跑」。判定库不碰。
- 我自己写的探针只有一份：`~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet/title_probe.py`，把 `scripts/agents/cite-check.py` 第 68–72 行的正则字符串抄进去用 `re` 单独试，不载入项目代码。

## 第 5 问：「实现今天的样子」逐条

### 改动 A（检出钩子 ⑦ 认 python 循环变量）

**A-1 新函数与 `bound` 参数：成立。**
- 「def name_binding_counts(tree):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:323`）
- 「def literal_loop_candidates(tree, assignments):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:351`）
- 「def python_literal_path(node, assignments, depth=0, bound=None):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:367`）
- 「def referenced_loop_names(node, assignments, loops, depth=0):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:388`）
- 「def python_path_choices(call, expressions, assignments, loops):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:400`）
- 「def python_service_writes(node, name, owner, receiver, choices):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:437`）
- `python_in_place_writes` 在 `.claude/hooks/bash_command_detector_literals_and_signals.py:462`，里面 `choices` 闭包交每个调用的取值（:479-480）。`python_service_writes` 只有这一处调用（`grep -rnE` 在 `.claude`、`scripts`、`task-server`、`prune-server` 下只命中 :501 一处调用）。
- 推翻它：上面任一 `def` 行不在、或 `python_service_writes` 在别处还有第二个调用点。

**A-2 认法条件：成立三条，第四条说宽了。**
- 可迭代对象只认元组、列表、集合：「isinstance(node.iter, (ast.Tuple, ast.List, ast.Set))」（`.claude/hooks/bash_command_detector_literals_and_signals.py:359`）。成立。
- 名字只绑定一次且元素非空：「and bindings.get(node.target.id) == 1 and node.iter.elts):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:360`）。
- 元素都算得出：:362 把带星号的元素算 None、其余交 `python_literal_path`，:363 `if None not in paths:` 才登记（任何一个元素算不出，整个循环不认）。成立。
- 调用在循环体里：「frozenset(id(item) for statement in node.body for item in ast.walk(statement))」（`.claude/hooks/bash_command_detector_literals_and_signals.py:364`），取值处「id(call) in loops[name].body_nodes」（`.claude/hooks/bash_command_detector_literals_and_signals.py:406`）；`node.orelse` 不在里面，所以 else 段与循环之后不代入。成立。
- 「名字整段只被这一个 for 绑定」：**不成立（说宽了）。** 计数函数 `name_binding_counts`（:323-336）只数 `ast.Name` 的 Store/Del、`ast.arg`、import；`except E as f`、`def f`、`class f`、match 的 `as` 不进计数。我用标准库 `ast` 解析 `try: pass / except E as f: pass / def g / class h` 核过：`ast.Name` 只出现 `E`（Load），`ExceptHandler.name` 是字符串 `'f'`（量的，`python3 -I` 现跑的 `ast` 一段，不是项目代码）。所以循环变量同名的 `except … as f` 之类不会让循环作废。方向：多出来的候选只会让钩子多拒，我没构造出由此漏放的例子，留给攻方腿。这一条在 `.claude/hooks/bash-command-detector.sh:226-229` 的「判不到」清单里写成「名字另有绑定的（赋值、别的 for、函数参数这些）」，没写这四种，清单本身没说错、只是不全。
- 推翻第四条的「说宽了」：这四种绑法放进 `name_binding_counts` 后与它的文档串一致，或证明文档串没写它们（文档串 :324 写的是「赋值、for 与推导式的目标、with … as、函数参数、import、增量赋值、海象与 del」，确实没写这四种，所以这是条文与实现的缝，不是实现违背自己的文档串）。

**A-3 组合上限 4096：成立。**
- 「MAXIMUM_LOOP_LANDINGS = 4096」（`.claude/hooks/bash_command_detector_scope_and_index.py:825`）；「if math.prod(len(loops[name].paths) for name in names) > MAXIMUM_LOOP_LANDINGS:」（`.claude/hooks/bash_command_detector_literals_and_signals.py:409`），其后一行 :410 交一组全 None（照算不出）。「多过」是严格大于，恰好 4096 种照代入。

**A-4 ⑦ ⑨ ⑫ ⑭ ⑯ ⑲ 共用这一段：成立。** 调用点：
- ⑦：「for write in python_in_place_writes(step.code):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:529`，`script_in_place_verdict` 里）。
- ⑨：「for write in python_in_place_writes(step.code, every_write=True):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:1606`，`compile_first_verdict` 里，def 在 :1587）。
- ⑫ 与 ⑭：「for write in python_in_place_writes(step.code, every_write=not overwrite_only, service_writes=service_judged):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:1727`，同一个函数 `bash_scope_verdict`（:1634）；`service_judged` 为真是 ⑭，见 :1647）。
- ⑯：`ledger_append_refusal`（`.claude/hooks/bash_command_detector_selftest_entries_and_main.py:1747`）里 :1760 调；⑲：`record_append_refusal`（`.claude/hooks/bash_command_detector_selftest_entries_and_main.py:1767`）里 :1777 调。
- 注：⑯ ⑲ 的实现在名叫 `selftest_entries_and_main` 的那一份里，不在 `literals_and_signals`；这一份没在 18 份被判清单里改，A 的 `new/` 也没改它（`git diff --no-index --stat base new` 只有四份：.sh、literals、scope_and_index、selftest_waits_and_scope）。

**A-5 「各自的放行与拒绝判据没改」：成立。**
- 我把草稿 `base/` 与 `new/` 的 `bash_command_detector_literals_and_signals.py` 做了 `git diff --no-index -U0`（量的）：改动只有 `name_binding_counts` 拆出、`literal_loop_candidates` 等四个新函数、`python_literal_path` 多 `bound` 参数、`python_service_writes` 与 `python_in_place_writes` 里「算一个路径」改成「对每种取值各算一个」。`mode_writes`、`opening_mode`、放行条件（`flags is not None and os_open_flags_write(flags)` 等）一字没动，只是 `path is not None` 的判断挪进了列表推导的 `if path is not None`。
- 没有循环变量时 `python_path_choices` 交回只有一组、照 `python_literal_path` 算（`.claude/hooks/bash_command_detector_literals_and_signals.py:407-408`），与改前逐个相同。`single_assignments` 改成调 `name_binding_counts`：base 的 :321-336 与现在的 :323-345 逐句对过，计数规则相同。
- 推翻它：任一调用点（A-4 列的五处）的放行/拒绝分支行与草稿 `base/` 不同，或有循环变量的路径之外的输入在新旧版交回不同的写。

**A-6 ⑤ 不走这一段：成立。** `results_overwrite_verdict` 里「if isinstance(step, PythonCode):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:2087`）下一行 `continue`（:2088）；整个函数没有调 `python_in_place_writes`。

**A-7 文件头 ⑦「判不到的」改成现状：成立。**
- 「判不到的：变量里拼出来的命令、eval、`bash x.sh` 起的脚本文件里的写法」（`.claude/hooks/bash-command-detector.sh:226`）后接 :227-229 写循环变量判不到的几样，收在「`os.open` 与 `os.write`、经 subprocess 起的写法」（`.claude/hooks/bash-command-detector.sh:229`）；`rsync --inplace` 那一句不在 ⑦ 的「判不到」里了。
- 报告说删它是因为 ⑦ 已认：「⑦ 另认 `rsync --inplace 源 已有文件`」（`.claude/hooks/bash-command-detector.sh:1361`）。成立。

**A-8 新开关与登记：成立。**
- 开关读取：「if detector_break_is_set("python-loop-variable-unexpanded"):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:355`），其后 :356 `return {}`（循环变量全部不认）。
- 名单：`.claude/hooks/bash_command_detector_scope_and_index.py:261` 末项 `"python-loop-variable-unexpanded"`（`KNOWN_DETECTOR_BREAK_ITEMS` 在 :229 起）。名单不登这一项时自证里「里的项都认得」那一格红，读它的代码：「unknown_break_items = [item for item in os.environ.get(DETECTOR_BREAK_VARIABLE, "").split(",") if item and item not in KNOWN_DETECTOR_BREAK_I」（截断，见 `.claude/hooks/bash_command_detector_selftest_entries_and_main.py:252`）与 :253 的格名。工具员报告说「不进，那一格在开关打开时也红」读得出来，成立。
- 文件头开关表项在 `.claude/hooks/bash-command-detector.sh:733-735`，末尾「（共 6 格）」（:735）。

**A-9 自证新加六格该拒、六格该放：成立（数量与形态，没起自证）。**
- 该拒六格（`.claude/hooks/bash_command_detector_selftest_waits_and_scope.py`）：⑨ 一格 :519-520；⑭ 一格 :593-594；⑦ 三格 :1878-1883（元组变量、列表字面量、嵌在 for 里的 `with open(os.path.join(…, 循环变量), 'w')`）；⑦ 拒绝点名候选一格 :1930-1932。我数到的该拒共 1+1+3+1 = 6。
- 该放六格 :1915-1921（候选都不是脚本、候选是新文件名与非脚本、元素算不出、循环变量另有赋值、可迭代对象是推导式、循环体之外用）。数 6。
- 这六格的 want 与开关表点名的六格（`.claude/hooks/bash-command-detector.sh:733-735`）逐名相同；开关打开后 `literal_loop_candidates` 交空，循环变量名既不在 `assignments`、也不在 `loops`，`python_literal_path` 交 None，该拒六格全变放行（读码，:355-356、:374-377、:407-408），该放六格本来就放。
- 没有格钉着的地方：⑫ ⑯ ⑲ 的自证里没有 python 循环的格，见第 6 问 A-d。⑭ 的 `python_service_writes` 里 holding、directory-copy 几种在循环里的取值，也没有格（⑭ 只钉了 `open(…, 'w')` 一格，:593-594）。
- 推翻它：自证输出里这十二格任一与 want 不符。

**A-10 换上后读数：记录原样可对，自证与复跑本身待主 agent 的下一次。**
- 「自检通过（查了 1589 种）」在 `records/2026-11-17-subagent账本清欠调度记录.md:562`；「复跑了 173 条开关（1 份）」在 `records/2026-11-17-subagent账本清欠调度记录.md:571`。都是主 agent 自己交的任务读数，我没法复核任务结局（没有任务号、也不许起任务）。
- 登记表：`.claude/gate.d/break-switch-replay-files.tsv:76` 写「一百七十三条开关」，文件头里行首五种 `*_BREAK=` 写法我数得 136（`BASH_COMMAND_DETECTOR_BREAK`）+2+8+25+2 = 173（量的，`grep -oE '^#   [A-Z_]+_BREAK='` 按名分组 `uniq -c` 的输出：136、2、8、25、2）。与 173 相符。
- 「+12 格」那条推算：1589 − 记录里更早的「查了 1577 种」（`records/2026-11-17-subagent账本清欠调度记录.md:543`）= 12，与「该拒六加该放六」相符；但两次之间没有别的检出钩子自证读数，不能说换上前一刻的数就是 1577，写作「相符、前提未核」。

### 改动 B（任务服务三处提醒认包着的写工作区工具）

**B-1 `workspace_writer_command` 与改名：成立。**
- 「def workspace_writer_command(self, task):」（`task-server/task_server_scheduler_building.py:1253`）；「return self.workspace_writer_command(task) is not None」（`task-server/task_server_scheduler_building.py:1251`，在 `workspace_swap_tool` 里，def 在 :1245）。
- 命令那一层判出写的交整条命令与任务工作目录：「return (list(task["command"]), cwd) if verdict else None」（`task-server/task_server_scheduler_building.py:1260`），`cwd` 是 :1257 的 `task.get("cwd") or self.peer_root`。
- 包着的交（词，工作目录）：「return inner, given_cwd」（`task-server/task_server_scheduler_building.py:1291`，`bash -c` 串），「return found, given_cwd」（`task-server/task_server_scheduler_building.py:1299`，脚本正文）；`given_cwd` 是 :1283 的任务工作目录。
- 改名：「def script_body_workspace_writer(self, words, cwd):」（`task-server/task_server_scheduler_building.py:1302`）交「return spoken」（`task-server/task_server_scheduler_building.py:1326`）；旧名 `script_body_writes_workspace` 全仓（排除 `research/`、`.git`、`target`）`grep -rnF` 零命中（量的）。
- 推翻它：旧名在仓里还有引用，或 `workspace_swap_tool` 不是 `is not None`。

**B-2 `modifies_workspace` 判定不变：成立（读 diff，没起脚本）。**
- `git diff --no-index -U0` 草稿 `base/` 与 `new/` 的 `task_server_scheduler_building.py`：`modifies_workspace`（:1219-1233）没有出现在任何一个 hunk 里，改的是 `workspace_swap_tool` 的收尾与 `wrapped_workspace_writer` 的返回值。
- 逐路对：改前命令那一层 `verdict` 为 True/False 就原样交 `verdict`，现在 True 交非空元组、False 交 None，`is not None` 与改前的真值一一对应；`verdict is None` 往下交 `wrapped_workspace_writer`，改前交布尔、现在交元组或 None：认到交 `(inner, given_cwd)` 或 `(found, given_cwd)`（都是非 None 的元组，`found` 本身是非空词表，:1325 保证 `spoken` 非空），没认到交 None。`True` ↔ 非 None，`False` ↔ None，逐路相同。
- 推翻它：构造一条命令使改前 `modifies_workspace` 与现在不同；我没构造出来。

**B-3 三处改取认到的那一条的词：成立。**
- `hook_swap_command`：「if "--feed-only" in (record.get("command") or []):」（`task-server/task_server_scheduler_lifecycle.py:1632`）先挡，再「return not lib.broken("wrapped-hook-swap-unread", self.environment) and self.writer_is_hook_swap(record)」（`task-server/task_server_scheduler_lifecycle.py:1636`）；`writer_is_hook_swap`（:1643-1649）取 `workspace_writer_command` 的词与工作目录。退 7 在 :1610、退 9 与 10 在 :1659 都调它。
- 退 7 找上一趟：「and (self.hook_swap_on_command_level(other)」（`task-server/task_server_scheduler_lifecycle.py:1616`），下一行 :1617 是 `wrapped-previous-swap-unnamed` 开关与 `writer_is_hook_swap(other)`。`hook_swap_on_command_level`（:1638-1641）与改前那段折法相同（我对过 diff：改前 `os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(other.get("cwd") or root), word)), root)`，现在走 `repository_paths`，`root` 与 `cwd` 取法没变）。
- `workspace_write_judged`：「found = None if lib.broken("wrapped-writer-words-unused", self.environment) else self.workspace_writer_command(record)」（`task-server/task_server_scheduler_lifecycle.py:1736`），随后 :1737-1738 把 `command, cwd` 换成认到的那一条。认不到的只有 `merge-and-verify-counted-as-writer` 下的合入后验证：`modifies_workspace` 在该开关下提前交 True（:1231-1232）而 `workspace_writer_command` 交 None，与报告「认不到（只有弄坏开关…）」相符。

**B-4 「`bash -c 'cd X && …'` 里的 `cd` 不跟」：成立（读码），后果「找错结局文件」推得出、没量过。**
- `shell_text_commands`（`task-server/task_server_scheduler_building.py:36-65`）把 `&&`、`;` 切成几条命令，没有任何 `cd` 的处理；认到的那一条交回的工作目录一律是任务自己的（:1291、:1299 的 `given_cwd`）。
- 后果：`apply-writer-patch.py` 那一支的结局文件按 `cwd` 拼：「outcome_file = os.path.join(cwd, named[0], PATCH_WRITER_OUTCOME_NAME)」（`task-server/task_server_scheduler_lifecycle.py:1757`）；不在就「return None   # 结局文件在动手换之前写：不在就是还没开始换，主工作区没动」（`task-server/task_server_scheduler_lifecycle.py:1759`）。补丁目录写成相对路径、又在 `bash -c` 里先 `cd` 到别处时，`outcome_file` 拼在任务的工作目录下找不到，落到 :1759 的「没开始换」，被杀的打补丁不报。方向是漏报。这条在改前也是这样（`cwd` 取法在 base 里也是任务的），不是这一改带进来的，但三格新自证里的 `bash -c` 样本都不带 `cd`（`task-server/task_server_selftest_memory.py:1247-1275` 那一格），没有格钉着。
- 推翻它：喂一条 `bash -c 'cd <另一目录> && python3 scripts/gate/apply-writer-patch.py <相对补丁目录>'` 被杀（退 137）的任务，结局文件在 `<另一目录>/<补丁目录>/` 下写着没打完，而 `workspace-write-interrupted` 照样报了。

**B-5 报告结论 1 说「包着的那一条带 `--feed-only` 的照旧不算」：不成立（说窄了），同时是一处命令层与包着层的不一致。**
- 包着那一层判写不写走 `writer_command_verdict`：「if (any(argument in WORKSPACE_READ_ONLY_ARGUMENTS for argument in words)」（`task-server/task_server_scheduler_building.py:1267`）交 False；`WORKSPACE_READ_ONLY_ARGUMENTS` 是「WORKSPACE_READ_ONLY_ARGUMENTS = ("--selftest", "--list-items", "--dry-run", "--read-outcome", "--pending-status")」（`task-server/task_server_constants.py:249`）五个词，不止 `--feed-only`。所以包着的 `hook-swap.py … --dry-run` 同样认不到。
- 命令那一层不是这样：`hook_swap_command` 只排除 `--feed-only`（:1632），`hook_swap_on_command_level` 只看路径（:1641），所以 `python3 scripts/gate/hook-swap.py … --dry-run` 退 10 照报，`bash -c 'python3 scripts/gate/hook-swap.py … --dry-run'` 退 10 不报。而 hook-swap.py 自己说 `--dry-run` 同样占草稿目录、退 10：「10 草稿目录正被另一趟 hook-swap.py 占着（--dry-run 同样；见」（`scripts/gate/hook-swap.py:162`）。
- 三格新自证里包着的只喂样本都用 `--feed-only`（`task-server/task_server_selftest_memory.py:1196-1224` 那一格），没有包着的 `--dry-run` 样本。
- 推翻它：喂 `bash -c 'python3 scripts/gate/hook-swap.py --spec s.json --scratch d --dry-run'` 退 10，`previous-swap-running` 照出。读码得到相反结论的话，是我漏看了 `hook_swap_command` 之外的另一个入口。这条属 B 的第 2 问边缘，交攻方腿与主 agent 判是不是缺口。

**B-6 三条新开关与三格登记：成立。**
- 开关在 `task-server/task_server_lib.py:1613`（`wrapped-hook-swap-unread`）、:1616（`wrapped-previous-swap-unnamed`）、:1619（`wrapped-writer-words-unused`），名单 `KNOWN_BREAK_ITEMS` 里三项都在（ast 取值核过：541 项、无重复，三项 `in` 为 True）。读开关处：`task-server/task_server_scheduler_lifecycle.py:1636`、:1617、:1736。
- 格登记：`task-server/task_server_selftest.py:538-540`；格函数在 `task-server/task_server_selftest_memory.py:1196`、:1225、:1247。
- 开关不进名单时设上退 2 认不出：「unknown = [item for item in items if item not in lib.KNOWN_BREAK_ITEMS]」（`task-server/task_server_selftest.py:660`）。报告说的「不加，设上新开关自证退 2」读得出来，成立。
- 读数：整份自证「447 格都对」与 11 条开关复跑在 `records/2026-11-17-subagent账本清欠调度记录.md:565`；`CELLS` 现数 447（量的：`python3 -I` 用 `ast` 数 `task_server_selftest.py` 里 `CELLS` 字典的键，447 个、无重复）。登记表 `.claude/gate.d/break-switch-replay-files.tsv:77` 写「五百四十一条开关」「四百四十七格」，`KNOWN_BREAK_ITEMS` 与文件头 `^  TASK_SERVER_BREAK=` 都数得 541（量的）。

### 改动 C（换钩子退 10 分两种成因）

**C-1 两种拒绝说明的认法：成立，与 hook-swap 现行原文对得上。**
- 认法：「SWAP_BUSY_SCRATCH_LINE = re.compile(r"✗ 草稿目录 .+ 正被另一趟 hook-swap\.py 占着（")」（`task-server/task_server_scheduler_lifecycle.py:34`）与「SWAP_BUSY_REPOSITORY_LINE = re.compile(r"✗ 这个仓的换上正被另一趟 hook-swap\.py 占着（")」（`task-server/task_server_scheduler_lifecycle.py:35`）。
- 原文：「✗ 草稿目录 {scratch} 正被另一趟 hook-swap.py 占着（{who}）」（`scripts/gate/hook_swap_swap.py:512`）与「✗ 这个仓的换上正被另一趟 hook-swap.py 占着（{who}）」（`scripts/gate/hook_swap_swap.py:557`）。两条正则都能匹配这两行起头（`who` 在括号里，正则只吃到「（」）。
- 退 10 的出处只有三处 `return EXIT_SCRATCH_BUSY, busy_lines`（`scripts/gate/hook_swap_swap.py:398`、:930、:945），`busy_lines` 都来自 `scratch_busy_lines` 或 `repository_busy_lines`（:883、:917），没有第三种说明。成立。
- 两个都出现取最后一处：循环里后出现的覆盖前面的（`task-server/task_server_scheduler_lifecycle.py:1680-1684`）；日志读尾部：「LOG_TAIL_BYTES = 65536」（`task-server/task_server_constants.py:157`）。
- 推翻它：hook-swap 另有一条退 10 的路径，说明里既没有「✗ 草稿目录」也没有「✗ 这个仓的换上」起头。

**C-2 `swap_busy_cause` / `swap_busy_item` 的形态：成立。**
- 「def swap_busy_cause(self, attempts):」（`task-server/task_server_scheduler_lifecycle.py:1671`）：先看 `swap-busy-cause-unread`，再取最大号尝试的日志逐行认（:1678-1684），都没认到交 None（:1687）；`swap_busy_item`（:1689）三支：草稿目录被占（:1693-1698，出路写「换一个 --scratch」）、这个仓的换上被占（:1699-1704，出路「换草稿目录没有用」、整条无 `--scratch`）、认不出（:1705-1709，依据写「两种都可能」、出路叫照日志里的拒绝说明与它的「→ 怎么办」办，无 `--scratch`）。
- `previous_swap_running_items` 多收 `attempts`：「items += self.previous_swap_running_items(record, attempts)」（`task-server/task_server_scheduler_lifecycle.py:1539`）；退 10 走「return [self.swap_busy_item(record, self.swap_busy_cause(attempts))]」（`task-server/task_server_scheduler_lifecycle.py:1664`），退 9 的句子一字没动（:1665-1669）。

**C-3 异常表那一行出路改成共用的一句：成立。** `task-server/task_server_lib_constants.py:129` 出路栏里写「退 10 的等占着的那一趟结束再交；占着的是哪一种、能不能换草稿目录，服务照这条任务日志里拒绝说明的行首认出来写在这一句后面」，出路栏里没有 `--scratch`（条件栏里的「同一个 --scratch」是条件，不是出路）。我对全仓非 `research/` 的 `.md`、`.py`、`.sh`、`.tsv` 里「换一个 --scratch」逐条列过（量的：`grep -rnF` 在仓里 `.py`、`.md`、`.sh`、`.tsv` 上命中 18 行，排除 `research/` 后 14 行）：服务这一侧能被读到的出现处是 `task-server/task_server_scheduler_lifecycle.py:1655`（文档串，按成因分）、:1698（草稿目录被占那一支）、`task-server/task_server.py:280`（按成因分）与自证；没有一处在不分成因时写它。

**C-4 README 三语与 `task_server_constants.py:246`：成立，三语与代码说的是一回事。**
- 中文：`task-server/README.md:174` 里「退 10（同一个 `--scratch` 正被另一趟占着，或这个仓的真换正被另一趟占着，没建树、没写文件；服务照最后一次尝试日志里的拒绝说明分这两种」与「退 10 而是草稿目录被占的另可换一个 `--scratch`（仓被占的换草稿目录没有用）」。
- 英文 `task-server/README.en.md:174` 同一行：「when the repository is held, a different scratch directory does not help」，且「when it recognizes neither, says either may apply」；日文 `task-server/README.ja.md:174` 有「リポジトリが占有されている場合は作業ディレクトリを変えても効かない」与「どちらとも認められないときは両方ありうる」。三份与 `swap_busy_item` 三支一一对得上。
- `task-server/task_server_constants.py:246`：「SWAP_SCRATCH_BUSY_EXIT_CODE = 10」那一行行尾注释写「同一个 --scratch 正被另一趟 hook-swap.py 占着，或这个仓的真换正被另一趟占着（两种照日志里的拒绝说明分，scheduler_lifecycle 的 swap_busy_cause）」，对。
- `task-server/task_server.py:278-283` 文件头那一段同样按成因写，含两条新开关（:282-283）。

**C-5 自证七个样本：成立。** `swap_busy_log_lines`（`task-server/task_server_selftest_notifying.py:1303`）出两种拒绝说明的现行原文（与上面 C-1 的 `hook_swap_swap.py:512`、:557 起头相同）；`swap_busy_problems`（:1319）里 `for what, task_id, name, lines, attempted, wanted in (` 之后七行：草稿目录被占、这个仓的换上被占、两种都出现草稿目录在后、两种都出现仓的换上在后、日志里没有这两种、日志文件不在、没有尝试（量的：七个元组）。旧断言「出路点名 --scratch」换成「advice 里不含 --scratch」（:1326）。

**C-6 读数：「447 格都对」写在了 C 一条上，记录只给了 B 那一条。** 正文写「换上验证点四格都对、整份自证「447 格都对」」。记录里：`records/2026-11-17-subagent账本清欠调度记录.md:574` 写「真换验证「task_server_lib.py 自证 4 格都对」、整份自证照跑，都退 0」，没有格数；「447 格都对」在 `records/2026-11-17-subagent账本清欠调度记录.md:565`（B 换上之后）。`CELLS` 现数 447（量的，见 B-6），所以这个数此刻为真，但「C 之后整份自证报 447」这句读数在记录里推不出，只能写「退 0；CELLS 现数 447」。主 agent 若手里有那一趟日志，对那一行即可。

### 改动 D（引文核对小节标题取全）

**D-1 病根：成立（独立复现）。** base 的 `LOCATION` 标题是 `{2,80}?` 懒匹配、没有结尾：我把 `scripts/agents/cite-check.py` 第 68–72 行的正则字符串抄进 `title_probe.py`，旧版（`TITLE_END=""`）对「三条都要守」（`kb/rule.md@分支的三条约束`）这一形交的标题是 `分支`，不带反引号的括号写法同样交 `分支`；位置在前写法（`kb/rule.md@分支的三条约束`：「…」）交完整标题（探针输出，量的）。与正文「`@分支的三条约束` 交「分支」」相符。

**D-2 `TITLE_END` 的定义：成立，代码比条文宽一点。**
- 「TITLE_END = "" if BROKEN == "title-lazy" else r"(?=`|[）)]|$|\s*[：:]?\s*「)"」（`scripts/agents/cite-check.py:68`）：反引号、全角或半角右括号、行尾、或（空白）可选的冒号（全角或半角）与可选空白后紧跟的「「」。条文写「位置在前写法里紧跟的「：」加引号」，代码里冒号是可选的、半角冒号也算、冒号与引号之间可有空白。`LOCATION` 引用它：（`scripts/agents/cite-check.py:69`）。
- 其余改动：文件头第 14 行补一句标题取到哪里（`scripts/agents/cite-check.py:14`）。

**D-3 开关 `title-lazy` 与自证：成立（读码与探针，没起自证）。**
- 开关：「CITE_CHECK_BREAK=title-lazy」（`scripts/agents/cite-check.py:50`）；`TITLE_END` 在开关下为空串（:68）；文件头登记三格：带反引号的长标题、不带反引号的长标题、三种写法交同一个完整标题（:51-53）；`selftest` 里新加三条用例与一段 `section_titles` 核对（草稿 `base`、`new` 的 diff：`@@ -339,0 +345,3 @@` 与 `@@ -353,0 +362,7 @@`），共四格。
- 红在点名的三格：「位置在前的长标题」一格开关下仍绿，因为位置在前的写法本来由后面的「「」把懒匹配撑到完整标题（探针对旧版：位置在前交完整标题，见 D-1）；其余两格与 `section_titles` 一段交 `分支`，而 `judge_section` 要求标题逐字命中恰好一行：「heads = [(index, len(line) - len(line.lstrip("#")))」（`scripts/agents/cite-check.py:206`）下面 :207 的 `== title`，`分支` 命中 0 次，判红。与文件头三格一一对应。
- 登记数：`records/2026-11-17-subagent账本清欠调度记录.md:569` 写「复跑了 9 条开关（1 份）…逐格相同 9 条」。读数原样，复核待主 agent。

**D-4 「标题里带括号、全角冒号、行尾标点、位置在前后面隔着空白」几种合法写法取得对不对：推不出「都对」，有两处我探针里看到了不同（给攻方腿）。**
- 标题里带全角括号：新旧两版都取不到（`含括号（说明）的标题`，字符集排除了（）()，探针交空）；新版不比旧版差，旧版交 `含括`。
- 标题后面紧跟「，」「。」又没有右括号时，新版取到那一整串（`标题后跟句号。`、`标题后跟逗号，补充说明`），而旧版取 `标题`（两个字，只是巧）。如果写引文的人把标题后面接说明写在同一对括号里（`（kb/x.md@标题，补充说明）`），新版会判成标题不存在。
- 位置在前、后面隔着空白与冒号都可（探针：`@分支的三条约束 :  「…」` 与 `… 「…」` 都交完整标题）。
- 引文在前不带反引号、后面是 `这一节` 一类（`（见 kb/rule.md@分支的三条约束 这一节）`）：新版交 `分支的三条约束 这一节`，不交完整标题。
- 这几条都是探针里的字符串，不是仓里真出现过的引文；是否构成缺口交主 agent 判。

## tsv 登记表两行

**T-1 `.claude/gate.d/break-switch-replay-files.tsv:76` 写「一百七十三条开关」：成立**（173 的数法见 A-10）。**T-2 :77 写「五百四十一条开关」「四百四十七格」：成立**（541、447 的数法见 B-6）。两行都是理由文字，格式是 `路径<TAB>数<TAB>#…`；第 77 行里「此前…」那串历史文字属于登记表自己的说明，不在本问射程。

## 第 6 问：工具员报告里写「推的」「没读」的几处，逐处现查

每处写：查了什么、结论（坐实 / 推翻 / 核不动）、依据。「主 agent 读数」指 `records/2026-11-17-subagent账本清欠调度记录.md` 里主 agent 自己交的任务的结局，我没法复核任务本身。

### 检出钩子那份（`detector-loop-target-paths/result.md`）

**6-A1 「期望的结局（都没跑，推的）」自证退 0、种数多 12（`result.md:52-54`）：坐实退 0，+12 相符前提未核。** 主 agent 读数：自检通过（查了 1589 种），`records/…:562`。+12 的底数见 A-10。

**6-A2 开关复跑「新开关红在点名的六格；五条已有开关逐格相同」（`result.md:55`）：坐实。** 主 agent 读数「复跑了 173 条开关（1 份）…逐格相同 173 条」`records/…:571`，173 条包含这六条里的全部（登记表 `.claude/gate.d/break-switch-replay-files.tsv:76` 与我数的 136+2+8+25+2 = 173 相符）。我另外对了报告的补点名与 `.sh` 的 diff（草稿 `base` 与 `new` 的 `bash-command-detector.sh`，`git diff --no-index -U0`）：disable-check 487→492（⑨ 一格加 ⑦ 四格，没有 ⑭ 那一格，即报告说的「⑭ 的 python 格不在 disable-check 下红」）、allow-script-in-place-write 47→51（⑦ 四格）、heredoc-body-judged-as-commands 35→39（⑦ 三个 heredoc 格加点名候选一格）、allow-compile-first-bypass 21→22（⑨ 一格）、allow-service-source-write 79→80（⑭ 一格）。五处增量与报告各格的形态一一相符。

**6-A3 「换上的喂法前两条 exit 2 是新判法、后四条 neutral」（`result.md:56`）：坐实（读文件）。** `feeds.json`：前两条 `"exit": 2` 没有 `neutral`，后四条带 `"neutral": true`；前两条分别是 `for f in (p,q):` 与 `for f in ['…loop-notes.md', '…lib_shell_words.py']:`，与报告所述相同；sha256 `629ab2a0…` 与报告所写相同（我现算，量的）。

**6-A4 规格「4 份文件 44 处（literals 26、.sh 11、selftest 6、scope_and_index 1）」（`result.md:64`）：坐实。** 我用 `python3 -I` 读 `swap-spec.json`（JSON 列表，每项 `file`/`old`/`new`），按 `file` 计数：44 项，26、11、6、1。四个文件的 sha256（`swap-spec.json` `13f5e14b…`、`reverse-spec.json` `26ba2f33…`、`feeds-reverse.json` `09380ef4…`）与报告相同（现算）。

**6-A5 「主工作区那两份 `git diff --stat` 是别的会话没暂存的改动」（`result.md:73-81`）：坐实。** 拿暂存区版本 `git show :<路径>` 与草稿 `base/` 做 `diff`：`bash-command-detector.sh` 三行、`bash_command_detector_selftest_waits_and_scope.py` 一行，差别都是去掉「正推腿乙 Z8；」那几个字（量的），与报告的「4 插 4 删」相符。

**6-A6 「⑫ ⑯ ⑲ 的自证里没有 python 循环的格」（`result.md:99`）：坐实，限在两类文件。** 在 `.claude/hooks/` 下 `bash_command_detector_selftest_*.py` 与 `lib_*selftest*.py` 里 `grep -nE 'for [a-z_]+ in [(\[{]'` 并要求含 python 或 `\n`：除 `lib_project_code_runs_selftest.py:197`（写样本表的生成式，不是喂钩子的命令）外，命中都是 A-9 列的新格；另一条 `grep -nE '\\nfor [a-z_]+ in|python3 -c "for '` 命中 14 行（量的）：A-9 的新格 9 行（:1881、:1883、:1915、:1917-1921、:1931）加五行不相干的（`bash_command_detector_selftest_waits_and_scope.py:1502`、:1559、:1571、:1685 是 shell 的 for、发信号的 for 与写脚本文件的字符串，`lib_project_code_runs_selftest.py:639` 是 shell 脚本字符串），没有一行是 ⑫ ⑯ ⑲ 的 python 循环写。限定词：只查了这两类文件、`for` 名字只含小写字母与下划线的写法；没有查 `.claude/gate.d/fixtures/` 里喂钩子的样本档。

**6-A7 `bash_command_detector_scope_and_index.py` 不登新名则自证多红一格、四处对账缺一处（`result.md:14`）：坐实（读码）。** 见 A-8（`.claude/hooks/bash_command_detector_selftest_entries_and_main.py:252-253` 与 `.claude/hooks/bash_command_detector_scope_and_index.py:263`「弄坏开关的名字在四处写」）。

**6-A8 「新函数的行为是读代码推的，靠第 3、4 条坐实」（`result.md:105`）：第 3、4 条的读数已有（见 6-A1、6-A2）；读码这一半我另读了一遍（A-2 至 A-9），除 A-2 里「名字整段只被这一个 for 绑定」说宽了，没有发现行为与说法不符。**

### 任务服务三处提醒那份（`task-server-wrapped-writer-reminders/result.md`）

**6-B1 八条已有开关补点名的新格「读源码推的，没跑过」（`result.md:71-82`）：坐实。** 主 agent 读数：开关复跑「复跑了 11 条开关（1 份）…逐格相同 11 条」`records/…:565`。我另核了文件头开关表里这八条的点名：对 `task-server/task_server_lib.py` 里每条开关行与其后四行取「自证：wrapped-…」，`feed-only-counted-as-writer` 点名 wrapped-hook-swap-reminded、wrapped-previous-swap-named（与已有的 wrapped-writer-read-only-retried），`workspace-write-interrupted-unreported` 与 `apply-outcome-ignored` 点名 wrapped-writer-interrupted-judged，`workspace-write-blocked-unreported` 点名前两格，`previous-swap-running-unreported` 与 `swap-scratch-busy-unreported` 点名 wrapped-hook-swap-reminded，`shell-string-writers-unread` 与 `script-body-writers-unread` 点名三格加各自已有的一格（量的：awk 取到的格名）；与报告表格一致。

**6-B2 「推的、不会红新格的」四条（`read-only-arguments-counted-as-writer`、`merge-and-verify-counted-as-writer`、`workspace-writers-retried`、`run-hard-limit-writer-unreminded`；`result.md:84`）：读码未推翻，没复跑，待主 agent 跑。** 读到的依据：①新格样本没有 `merge-and-verify` 入口，该开关只在 `MERGE_AND_VERIFY_ENTRY in relative` 时起作用（`task-server/task_server_scheduler_building.py:1231`）；②`workspace-writers-retried` 只在 `task-server/task_server_scheduler_notifying.py:1466` 的重排那一支起作用，新格不走；③`run-hard-limit-writer-unreminded` 只在结局是硬上限时起作用（`task-server/task_server_scheduler_lifecycle.py:1722`），新格样本是 `command-failed`；④`read-only-arguments-counted-as-writer` 下 `feed-chain.sh` 里 `stage-mine.py --selftest` 变成「认到的写」，但它不是 hook-swap.py，`writer_is_hook_swap` 交 False，三格里没有靠它出提醒的样本（`task-server/task_server_selftest_memory.py:1014-1017` 是 `feed-chain.sh` 正文）。文件头开关表里这四条也没点名新格（上面 6-B1 那段 awk 的输出里它们没有 `wrapped-hook-swap-reminded` 等三格）。命令见末节第 1 条。

**6-B3 「已有格在三条新开关下不红：它们的样本都在命令那一层（推的）」（`result.md:84`）：读码未推翻，没复跑，待主 agent 跑。** 三条新开关各只在 `task-server/task_server_scheduler_lifecycle.py:1636`、:1617、:1736 改变「包着的」那一支；命令那一层的路径没有这三个开关。已有格里 `wrapped-writer-*-not-retried`、`wrapped-writer-read-only-retried`（`task-server/task_server_selftest_memory.py:1032-1088`）的样本是 `bash -c` 与脚本正文，但它们读的是「重排与否」（`modifies_workspace`），不走 `hook_swap_command` 等三处。命令见末节第 2 条。

**6-B4 「447 格」（`result.md:102`）：坐实。** 主 agent 读数「447 格都对」`records/…:565`；`CELLS` 现数 447（B-6）。

**6-B5 「`task_server.py` 文件头 277–280 行没说包着的也认；不算写错」（`result.md:114`）：成立，仍然如此。** 现在的 `task-server/task_server.py:276` 起 workspace-write-blocked 一项写「换钩子的命令（scripts/gate/hook-swap.py，不带 --feed-only）」，:270-284 范围里没有「bash -c」「包着」「包在」字样（`grep -nE 'bash -c|包着|包在'` 零命中，量的）；`task-server/task_server_lib.py:236-240` 与 `hook_swap_command` 文档串（`task-server/task_server_scheduler_lifecycle.py:1628-1631`）写了包着的也认。缺的是 `task_server.py` 文件头那一句，不是说错。

### 换钩子退 10 那份（`swap-exit-ten-two-causes/result.md`）

**6-C1 「这些格与开关我都没跑，没验证过」「关着开关的期望结局都是推的」（`result.md:47-54`）：一半坐实，一半待主 agent 跑。**
- 「两条开关关着时七个样本应绿」：坐实。`records/…:574` 写真换验证「4 格都对」、整份自证照跑都退 0（该行没抄格数，见 C-6），格函数里七个样本就在 `previous-swap-running-reminded` 里（C-5）。
- 「`swap-busy-cause-unread` 下仓样本得到草稿目录那一支，应红」「`swap-busy-unknown-guessed` 下三个认不出的样本得到草稿目录那一支，应红」：读码相符：前者 `swap_busy_cause` 在开关下直接交 `SWAP_BUSY_SCRATCH`（`task-server/task_server_scheduler_lifecycle.py:1675-1676`），仓样本断言依据含「这个仓的换上正被」、出路不含 --scratch（`task-server/task_server_selftest_notifying.py:1355`）落空；后者 `task-server/task_server_scheduler_lifecycle.py:1685-1686`。开关复跑读写判定库，此刻不交；改用直接设开关点格的两条命令（末节第 3、4 条），不碰判定库。待主 agent 跑。
- 报告写的「开关复跑」命令（`result.md:92-98`）里的四条 `--item` 都是读写判定库的那一种，这一轮我不重写它们；等判定库重建完再由主 agent 交。

**6-C2 「`wrapped-hook-swap-reminded` 只看出了哪一种提醒与依据里的「退 10」……应不受影响」（`result.md:52`）：读码坐实，待主 agent 跑一次。** 该格对依据只断言一条：`f"退 {code}" not in item["basis"]`（`task-server/task_server_selftest_memory.py:1213`，格函数在 :1196-1224），而 `swap_busy_item` 三支的依据句都以「换钩子的命令退 10：」起头（`task-server/task_server_scheduler_lifecycle.py:1696`、:1702、:1707）；该格的退 10 样本没有尝试（`finished_attention_kinds` 里 `attempts_of` 交空），`swap_busy_cause` 交 None，落认不出那一支，依据仍写「退 10」。命令见末节第 5 条。

**6-C3 `README.ja.md` 「我没查」（`result.md:58`）：现查：已有对应句。** `task-server/README.ja.md:174` 有「同じ `--scratch` が別の入れ替えに占有されているか、このリポジトリの本番の入れ替えが別の入れ替えに占有されており」与「どちらとも認められないときは両方ありうる」与「リポジトリが占有されている場合は作業ディレクトリを変えても効かない」（grep 取得）；中文、英文同一行相同（C-4）。这是主 agent 改过之后的现状，不是该报告写作时的样子。

**6-C4 登记表现数「五百三十九条」要改 541（`result.md:60`）：坐实。** 现 `.claude/gate.d/break-switch-replay-files.tsv:77` 写五百四十一条，`KNOWN_BREAK_ITEMS` 541（B-6，量的）。

**6-C5 `task_server_constants.py:246` 的行尾注释只写草稿目录被占（`result.md:59`）：该报告的说法在写作时成立，现已改。** 现状见 C-4。

**6-C6 「对着 hook-swap.py 现行原文把两条正则单独试了一遍」（`result.md:105`）：坐实（我另试一遍）。** 我把 `task_server_scheduler_lifecycle.py:34-35` 的两条正则抄进 `busy_regex_probe.py`，对 `hook_swap_swap.py:512` 与 :557 的行首代入三种 `who`（读得到记录、读不全、摘记录被接手）与三种草稿路径（含空格与全角括号）试了 9 组：每组「草稿目录行只被草稿目录那条认、仓行只被仓那条认」（输出九行均为 `True False False True`，量的）。

**6-C7 「`judged` 里 `attempts_of` 的桩」（`result.md:106` 推翻条件）：桩会被读到。** `attention_of_record` 里「attempts = [attempt for attempt, valid in self.store.attempts_of(task_id) if valid]」（`task-server/task_server_scheduler_lifecycle.py:1470`），样本 `[({"number": 1, "state": "ended", "log": path}, True)]`（`task-server/task_server_selftest_notifying.py` 里 `judged`）元组形态对得上。

### 引文核对那份

**6-D1 没有报告可查。** `cite-check-title-lazy/` 里没有 `result.md`（`find … -name 'result*'` 无输出，量的）；该目录 `.owner` 写的用途里含「报告」二字（`cat .owner`），报告应该是在交回消息里。正文改动 D 的说法我只能对代码与记录：D-1 至 D-4 与 `records/…:567`（主 agent 复现）、:569（换上与 9 条开关复跑）。

## 要主 agent 跑的命令

都不碰判定库。五条都是点格的 `--selftest`（假调度器上的格，不起服务），在我的草稿目录起；脚本路径写绝对路径，因为 `--cwd` 不是仓根。若被范围声明闸拒，照拒绝说明补 `SINGLEFS_RUN_SCOPE`，不换写法。

1. 6-B2：四条「推的、不会红新格」的开关一起设上，三格新格应全绿。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet -- env TASK_SERVER_BREAK=read-only-arguments-counted-as-writer,merge-and-verify-counted-as-writer,workspace-writers-retried,run-hard-limit-writer-unreminded TASK_SERVER_SELFTEST_CELLS=wrapped-hook-swap-reminded,wrapped-previous-swap-named,wrapped-writer-interrupted-judged python3 /home/fy5090/code/singlefs/task-server/task_server_lib.py --selftest`
   - 看：末行 `TASK_SERVER_RESULT` 的 `exit=0`、日志里没有 `✗` 行。有 `✗ task_server_lib.py 自证：<格名>：` 的，说明「不会红」被推翻，拆成四条各设一个开关再交，看哪一条。
2. 6-B3：三条新开关一起设上，已有的十一格应全绿。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet -- env TASK_SERVER_BREAK=wrapped-hook-swap-unread,wrapped-previous-swap-unnamed,wrapped-writer-words-unused TASK_SERVER_SELFTEST_CELLS=workspace-write-blocked-reminded,previous-swap-running-reminded,workspace-write-interrupted-reported,workspace-write-complete-not-reported,wrapped-writer-shell-string-not-retried,wrapped-writer-script-body-not-retried,wrapped-writer-read-only-retried,workspace-writer-not-retried,writer-dry-run-retried,run-hard-limit-writer-stopped,workspace-writer-cap-has-headroom python3 /home/fy5090/code/singlefs/task-server/task_server_lib.py --selftest`
   - 看：`exit=0`、无 `✗` 行。有 `✗` 的，格名就是「已有格被新开关打红」的反例。
3. 6-C1：`swap-busy-cause-unread` 设上，只跑退 10 那一格，应红在它。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet -- env TASK_SERVER_BREAK=swap-busy-cause-unread TASK_SERVER_SELFTEST_CELLS=previous-swap-running-reminded python3 /home/fy5090/code/singlefs/task-server/task_server_lib.py --selftest`
   - 看：`exit=` 非 0，日志里恰有一行 `✗ task_server_lib.py 自证：previous-swap-running-reminded：`，没有别的格名。绿了说明样本没走到新逻辑（该报告的推翻条件）。
4. 6-C1：`swap-busy-unknown-guessed` 设上，同一格应红。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet -- env TASK_SERVER_BREAK=swap-busy-unknown-guessed TASK_SERVER_SELFTEST_CELLS=previous-swap-running-reminded python3 /home/fy5090/code/singlefs/task-server/task_server_lib.py --selftest`
   - 看：同第 3 条。
5. 6-C2：两条新开关一起设上，只跑 `wrapped-hook-swap-reminded`，应绿。
   - `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet -- env TASK_SERVER_BREAK=swap-busy-cause-unread,swap-busy-unknown-guessed TASK_SERVER_SELFTEST_CELLS=wrapped-hook-swap-reminded python3 /home/fy5090/code/singlefs/task-server/task_server_lib.py --selftest`
   - 看：`exit=0`、无 `✗` 行。

没写的命令：B-4（`cd` 不跟）与 B-5（包着的 `--dry-run` 不认）要新写样本才喂得出，没有现成的格；它们是读码结论，写进判定一览，交攻方腿喂或主 agent 另派。
我自己的两份探针（不是模型目录、没有 `tools/rerun.sh`）都在 `~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-sonnet/`：`title_probe.py`、`busy_regex_probe.py`，只用 `re`、不载入项目代码，复跑 `python3 -I <路径>`；sha256 见末节。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A-1 新函数与 bound 参数 | 成立 | 六个 def 行都在，`python_service_writes` 只一处调用 |
| A-2 认法条件 | 成立三条、说宽一条 | 「名字整段只被这一个 for 绑定」：`except … as`、`def`、`class`、match 绑定不进计数（literals:323-336），方向偏严，没构造出漏放 |
| A-3 组合上限 4096 | 成立 | 严格大于 4096 才算不出（literals:409，scope_and_index:825） |
| A-4 六道共用这一段 | 成立 | 五个调用点逐个列出；⑯ ⑲ 的实现在 selftest_entries_and_main 里 |
| A-5 各道判据没改 | 成立 | 草稿 base 与 new 的 diff 只动取值，放行分支一字未动 |
| A-6 ⑤ 不走这一段 | 成立 | results_overwrite_verdict 遇 PythonCode 直接 continue |
| A-7 文件头 ⑦ 判不到改现状 | 成立 | rsync 那句删得对（⑦ 已认，sh:1361） |
| A-8 新开关与名单 | 成立 | 名单不登会让「项都认得」一格多红，读码相符 |
| A-9 自证六拒六放 | 成立（没起自证） | 数得 6 加 6，与开关表点名逐名相同；⑫ ⑯ ⑲ 与 ⑭ 的删除、挪走、整目录拷在循环里没有格 |
| A-10 读数 1589、173 | 记录可对，复核待主 agent | 173 我数得 136+2+8+25+2；+12 的底数未核 |
| B-1 workspace_writer_command 与改名 | 成立 | 交（词，工作目录）或 None，旧名零命中 |
| B-2 modifies_workspace 不变 | 成立（读 diff） | 该函数不在任何 hunk；True/False/None 与改前逐路对应 |
| B-3 三处取认到的那一条 | 成立 | 1632、1636、1616-1617、1736 |
| B-4 cd 不跟 | 成立（读码），后果推的 | 打补丁结局文件按任务工作目录拼，目录里先 cd 会落到「没开始换」不报；改前同样，没有格钉着 |
| B-5 「只有 --feed-only 不算」 | 不成立（说窄了） | 包着层五个只读词都不算，命令层只排除 --feed-only，包着的 `--dry-run` 退 10 不报；无格；交攻方腿与主 agent 判 |
| B-6 三开关三格 | 成立 | 名单、格登记、读开关处、541、447 都核过 |
| C-1 两种拒绝说明的认法 | 成立 | 与 hook_swap_swap.py:512、557 现行原文对得上，退 10 只有三处出口 |
| C-2 cause 与 item 形态 | 成立 | 三支各自出路 |
| C-3 异常表出路共用 | 成立 | 出路栏无 --scratch；服务侧无不分成因写它的地方 |
| C-4 README 三语与常量注释 | 成立 | 三语 174 行与代码三支一一对应 |
| C-5 七个样本 | 成立 | 七个元组、旧断言换成不含 --scratch |
| C-6 「447 格」读数 | 推不出（数此刻为真） | 记录 574 行只写退 0，447 的引号读数在 565 行 |
| D-1 病根 | 成立 | 独立探针复现旧版交 `分支` |
| D-2 TITLE_END | 成立，代码比条文宽 | 冒号可选、半角也认、可隔空白 |
| D-3 开关与自证 | 成立（没起自证） | 三格红、位置在前一格仍绿，与文件头一致 |
| D-4 各种合法写法 | 推不出「都对」 | 标题后跟「，」「。」无右括号时取一整串；`（见 … 这一节）` 取到「这一节」；不带反引号的括号后跟说明会判不存在 |
| T-1、T-2 登记表两行 | 成立 | 173 与 541、447 都能数出 |
| 6-A1 至 6-A8 | 坐实 | 主 agent 读数（562、571）加我的 diff、计数、文件核对 |
| 6-B1、6-B4、6-B5 | 坐实 | 表、447、文件头缺一句都核到 |
| 6-B2、6-B3 | 读码未推翻，待主 agent 跑 | 末节第 1、2 条 |
| 6-C1、6-C2 | 一半坐实，一半待主 agent 跑 | 末节第 3、4、5 条；开关复跑的四条 `--item` 读写判定库，此刻不交 |
| 6-C3 至 6-C7 | 坐实 | README 日文、登记表、常量注释、正则探针、桩 |
| 6-D1 | 没有报告可查 | 引文核对的草稿目录里没有 result.md |

## 没做什么

- 没跑自证、门禁、开关复跑、任何项目脚本；没交任务、没读写判定库。读数里「主 agent 读数」那几条我没法复核任务结局。
- 没判攻方腿的格（四处改法有没有被攻穿）；B-4、B-5、D-4 只是读码或探针里看到的现象，是不是缺口、要不要改交主 agent。
- 没建仓副本、编译目录、模型目录；草稿目录只有两份探针与这份报告的草稿（`du -sh` 24K，量的）：`title_probe.py`（sha256 `0a611eca94ee538803ec8e34ca6c900af596a81a145db8301c5e01185517e19b`）、`busy_regex_probe.py`（sha256 `e1e4b0709b2690e2f18f57319ef83c37bc0820e5593fde0e5b1234e388e871ae`）。
- 没读攻方腿这一轮的报告；没读别的腿这一轮的提示。
- 「名字整段只被这一个 for 绑定」之外，A 的自证与开关我没找到与说法不符的地方，但这只是读码，不等于证明没有；推翻 A-9 的现象：自证输出里那十二格任一与 want 不符。
