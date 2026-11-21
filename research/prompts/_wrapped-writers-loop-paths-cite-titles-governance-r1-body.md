# 检出钩子循环变量、任务服务包着的写工具与退 10 两种成因、引文核对小节标题：治理文件轮第一轮（wrapped-writers-loop-paths-cite-titles-governance-r1，2026-11-21）

方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码，四处改的都是协作工具（检出钩子、统一任务服务、引文核对脚本）；`crates/` 下一个字节没动。

## 问题

1. 每一处改法做的是不是它要关的那一条说的事，修没修到位，有没有带进新的缺口（攻方腿）。
2. 「实现今天的样子」每一条今天的文件是不是这样写的，工具员报告里写「推的」的几处现查（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 559–574 行）。改前原样与工具员报告在各草稿目录：`base/` 是改前、`new/` 是新版、`result.md` 是报告。

- 改动 A，检出钩子 ⑦ 认 python 循环变量（草稿 `~/.local/share/singlefs/work/detector-loop-target-paths/`）：起因是 heredoc 里 `for f in (p,q): open(f,'w')` 就地改脚本草稿被放过。`.claude/hooks/bash_command_detector_literals_and_signals.py` 加 `name_binding_counts`、`literal_loop_candidates`、`referenced_loop_names`、`python_path_choices`，`python_literal_path` 多一个 `bound` 参数，`python_in_place_writes` 与 `python_service_writes` 按每种取值各交一份写；只认 `for 名字 in <元组、列表或集合字面量>:`、名字整段只被这一个 for 绑定、元素都算得出、调用在循环体里，组合多过 `MAXIMUM_LOOP_LANDINGS`（4096）的照算不出。⑦ ⑨ ⑫ ⑭ ⑯ ⑲ 共用这一段，各自的放行与拒绝判据没改；⑤ 不走这一段。文件头 ⑦「判不到的」改成现状。新开关 `python-loop-variable-unexpanded`，`bash_command_detector_scope_and_index.py` 的 `KNOWN_DETECTOR_BREAK_ITEMS` 加它；自证新加六格该拒、六格该放。换上后自证「自检通过（查了 1589 种）」，开关复跑「复跑了 173 条开关（1 份），自证红且点名的格与实际红的格逐格相同 173 条」。
- 改动 B，任务服务三处提醒认包着的写工作区工具（草稿 `~/.local/share/singlefs/work/task-server-wrapped-writer-reminders/`）：`task-server/task_server_scheduler_building.py` 加 `workspace_writer_command`（交认到的那一条命令的词与任务的工作目录），`wrapped_workspace_writer` 改交（词，工作目录）或 None，`script_body_writes_workspace` 改名 `script_body_workspace_writer`；`workspace_swap_tool` 改成 `workspace_writer_command(task) is not None`，`modifies_workspace` 判定不变。`task-server/task_server_scheduler_lifecycle.py` 的 `hook_swap_command`、退 7 找上一趟、`workspace_write_judged` 改取认到的那一条的词；工作目录取任务的 `--cwd`，`bash -c 'cd X && …'` 里的 `cd` 不跟（主 agent 读 diff 时看到的，没量）。三条新开关、三格新自证（`task-server/task_server_selftest_memory.py`）。换上后整份自证「task_server_lib.py 自证 447 格都对」，开关复跑「复跑了 11 条开关（1 份）…逐格相同 11 条」。
- 改动 C，任务服务换钩子退 10 分两种成因（草稿 `~/.local/share/singlefs/work/swap-exit-ten-two-causes/`）：起因是主 agent 的换上退 10、日志拒绝说明是「这个仓的换上正被另一趟 hook-swap.py 占着…换草稿目录没有用」，服务推的提醒却是「同一个 --scratch 正被另一趟占着」「或换一个 --scratch」（`scripts/gate/hook-swap.py` 文件头第 43、45、115 行，退 10 有两种成因）。`previous_swap_running_items` 多收 `attempts`，新加 `swap_busy_cause`（读最后一次尝试日志尾部认两种拒绝说明起头，都出现取最后一处，认不出写两种都可能）与 `swap_busy_item`；异常表 previous-swap-running 那一行出路改成共用的「等占着的那一趟结束再交」；两条新开关；自证 `previous-swap-running-reminded` 退 10 那一段改成七个日志样本（`task-server/task_server_selftest_notifying.py`）。主 agent 另改 `task-server/README` 三语第 174 行两句与 `task-server/task_server_constants.py:246` 行尾注释。换上验证点四格都对、整份自证「447 格都对」；开关复跑读写判定库，等别的会话删库重建之后再交，这一轮还没有读数。
- 改动 D，引文核对小节标题取全（草稿 `~/.local/share/singlefs/work/cite-check-title-lazy/`）：`scripts/agents/cite-check.py` 的 `LOCATION` 小节标题懒匹配 `{2,80}?` 在「引文在前、位置在后」那一形后面只剩可选反引号，只取两个字（主 agent 复现：`@分支的三条约束` 交「分支」）。加 `TITLE_END` 前瞻（反引号、「）)」、行尾，或位置在前写法里紧跟的「：」加引号），新开关 `title-lazy`，自证四格。换上验证与开关复跑「复跑了 9 条开关（1 份）…逐格相同 9 条」。
- 登记表 `.claude/gate.d/break-switch-replay-files.tsv`：第 76 行（检出钩子）改一百七十三条，第 77 行（任务服务）改五百四十一条开关、四百四十七格。
- 已知开着、这一轮不重复报的：任务服务改动 C 的开关复跑还没跑；开关复跑那一格跑检出钩子 173 条时服务报 31 线程平均只用 13.9 核，并行度不在这一轮的射程里。

## 范围

这一轮被判的文件（18 份，逐个列在 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 改动 A：有没有该拒的循环写法仍被放过（嵌套循环、同名另有绑定的、`for (a, b) in …`、`else` 段、`with` 与 `try` 里），有没有该放的被误拒（候选里有新建文件、目标不是脚本、循环体外的同名用法）；⑨ ⑫ ⑭ ⑯ ⑲ 跟着按候选判之后有没有一道的判据因此变宽或变窄而它的自证没格钉着；新开关是不是红在名字说的那件事。
2. 改动 B：`workspace_writer_command` 交出的词与工作目录在 `bash -c` 串、脚本正文两种形态下各取得对不对（`cd` 不跟那一条会不会让 apply-writer-patch.py 的结局文件那一支找错文件）；`modifies_workspace` 的判定是不是真没变；三格新自证与三条开关红没红在点名的事上。
3. 改动 C：两种拒绝说明的认法与 `scripts/gate/hook-swap.py` 现行原文对不对得上（两个都出现、只有一个、日志被截断、日志在第二台）；提醒文字里还有没有一处只对一种成因成立；`task-server/README` 三语改的两句与代码说的是不是一回事。
4. 改动 D：`TITLE_END` 有没有一种合法写法反而取短或取不到（标题里带括号、全角冒号、行尾有标点、位置在前写法后面隔着空白）；开关是不是红在点名的三格。

正推腿（逐条回「成立」「不成立」「推不出」）：

5. 「实现今天的样子」每一条：今天的文件是不是这样写的（写出 文件:行 与那一句原文）。
6. 四份工具员报告里写「推的」「没读」的地方，在只读能查到的范围内逐条现查。

## 分工

攻方腿做第 1 到 4 问，正推腿做第 5、6 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
