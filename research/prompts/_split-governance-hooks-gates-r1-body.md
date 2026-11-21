# crates 代码拆分换上的钩子与门禁：治理文件轮第一轮（split-governance-hooks-gates-r1，2026-11-20）

这一轮攻的是按 2000 行拆分之后换上主工作区的钩子与门禁：`.claude/hooks/` 两组八份、`.claude/gate.d/` 三道加 doc 门禁五道拆成入口加部分文件，以及拆分换上之后为它修的四件（两件在这一题、两件在 `split-governance-scripts-services-r1`）。拆分期间用户定「不需要走三方 先拆分代码 全部拆完再说」（`records/2026-11-19-crates代码拆分.md` 第 261 行），这一轮补上。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码；与 `crates/` 有交点的是门禁 crash-point-replay、code-source-discipline 两道拆出的格库读 `crates/` 下的测试与变异表、门禁 code-tooling 的格按 `crates/*/Cargo.toml` 判包。

## 问题

1. 拆分声称只挪不改：入口留原路径，部分文件放同层（钩子是 `<入口主干>-<名>.py` 或 `<库主干>_<名>.py`，三份内嵌 python 的钩子入口只剩引导、按钩子目录把部分文件依次 exec 进同一个命名空间；门禁的格库放 `.claude/gate.d/lib/`）。挪动之外，有没有一处行为变了（攻方腿）。
2. 拆分带出的同一类缺陷有没有漏网的：判法只读入口原文而读不到挪进部分文件的代码，自证把替身换在错的模块或命名空间上，自证或门禁拷库、拷钩子目录时漏了部分文件，exec 进同一命名空间的部分文件互相覆盖名字或依赖次序（攻方腿）。
3. 拆分之后换上的四件修补（两件在这一题：项目代码闸两份自证改认拆分族、agent-write-scope 格改认两份共用模块的拆分族），做的是不是它说的事（正推腿）。

## 实现今天的样子

主 agent 的观测（读的是此刻主工作区；拆前原文在 HEAD，`git show HEAD:<路径>`）：

- 钩子两组换上的经过在 `records/2026-11-19-crates代码拆分.md` 第 294、302 行：先放部分文件，再逐份经 `scripts/gate/hook-swap.py` 先 `--dry-run` 再真换。三份内嵌 python 的入口（`bash-command-detector.sh`、`completion-cleanup-guard.sh`、`write-guard.sh`）只剩引导；`selftest_in`（拆前 3013 行单函数）切成两个函数、局部名经字典交接；检出钩子先红在交接（`value` 是有条件才绑定的局部名），主 agent 改成有条件取、换上后自证绿。
- 门禁三道（`code-tooling.sh`、`crash-point-replay.sh`、`code-source-discipline.sh`）拆成入口加 `.claude/gate.d/lib/` 下 16 份库（第 295 行），doc 门禁五道入口加 13 份库（第 301 行），五条 `--list` 与五道样本档换上时绿。
- 拆分之后修的两件（这一题；记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 501–513 行）：
  - 项目代码闸：`.claude/hooks/lib_project_code_runs_launchers.py` 加 `source_family`（入口加它 source、import、按文件名载入的同目录文件，四层），要临时服务的登记行「源码里有核对」认整族；新开关 `temporary-service-mark-entry-only`、`temporary-service-family-by-directory`。换上后 `lib_project_code_runs.py --selftest`、`project-code-guard.sh --selftest` 都绿，开关复跑 76 条、151 条逐格相同。
  - agent-write-scope 格：`.claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh` 取函数名时取 `lib_shell_words.py`、`lib_project_code_runs.py` 的整族（入口加同层 `<主干>_*.py`），新开关 `AGENT_WRITE_SCOPE_BREAK=family-parts-unread`、新样本 `agent-write-scope-parts-red`。换上后主工作区单跑这一格绿，整档样本里这一格 10 份都判得对；它的 11 条开关复跑开跑前算键全部拒算，没跑。
- 已知开着、各有会话在修、这一轮不重复报的：门禁 code-tooling 的 break-switch-replay、break-switch-headers、break-switch-outside-selftest 三格把部分文件当独立脚本判（mainagent(B)）；算键闭包跨不过拆出的文件（门禁三道拆后闭包少收单元、stage-break-switch-replay 格开跑前全部拒算、样本档里 function-units-two-implementations-agree 与 mutation-list-items 两族红，mainagent(C)）。这一轮有三份拆分族认法（`source_family`、agent-write-scope 格里的 `<主干>_*.py`、mainagent(C) 在扩的 `scripts/gate/lib_lag_split_family.py`），会不会分叉在这一轮里攻。

## 范围

这一轮被判的文件（127 份，逐个列在 `research/prompts/split-governance-hooks-gates-r1-snapshot/judged-sha256.txt`；都读主工作区此刻那一版，拆前那一版 `git show HEAD:<入口路径>`，部分文件在 HEAD 里没有）：`.claude/hooks/` 下拆分两组的入口与部分文件、`.claude/gate.d/` 下拆过的八道门禁入口与 `.claude/gate.d/lib/` 下拆出的格库、拆分与这一轮修补带进来的样本。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 拆前原文与「入口加部分文件」拼起来之后，挪动之外有没有行为变了：判法的分支、默认值、退出码、拒绝说明、`--list` 与格名表、弄坏开关的认法。给得出一条命令或一份输入，拆前拆后判得不一样就是打中。三份 exec 进同一命名空间的钩子另核：部分文件之间有没有同名覆盖、次序依赖、`__file__` 与 `__name__` 指错。
2. 「判法只读入口原文」这一类，在这一题的文件里除了「实现今天的样子」列的已知几处，还有没有：按文件名读源码文本、`sed`/`grep` 自己的文件头、拷库清单与临时钩子目录的拷贝表。
3. 自证换替身、打补丁的地方（`globals()`、模块属性、`patch.object`、环境变量）在拆分后是不是换在被测代码真正读的那个命名空间上。
4. 三份拆分族认法：同一个入口，三份认出的族一不一样；给得出一份入口使一份认法多认或少认，而那一处判法因此判错的，就是打中。

正推腿（逐条回「成立」「不成立」「推不出」）：

5. 两件修补（项目代码闸改认拆分族、agent-write-scope 格改认拆分族）逐条核今天的代码做的是不是「实现今天的样子」那一条说的事，写出 文件:行 与那一句原文。
6. 两件修补新加的自证格、样本与弄坏开关：每一样红的地方是不是它名字说的那件事（给一份样本或一格自证，说清它红在哪一行判法上）。
7. 拆分换上的八道门禁：每道入口的格名表（`# gate-cell:`）与 `--list`、`stage_cell` 登记逐格相同，格的函数都住在入口或它 source 的库里、没有一格的函数找不到。

## 分工

攻方腿做第 1 到 4 问，正推腿做第 5 到 7 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
