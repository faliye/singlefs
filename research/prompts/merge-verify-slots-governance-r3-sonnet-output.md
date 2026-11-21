# 合入后验证槽位治理 第三轮 正推腿（three-way-forward）报告

日期：2026-11-21。判第 2、3 格；方案照 `crates/` 今天的实现来谈（本轮没改 `crates/`，被判的是脚本、登记表与共用约束）。被判文件的此刻哈希与开工快照 `research/prompts/merge-verify-slots-governance-r3-snapshot/judged-sha256.txt` 逐个相同（见「复跑与哈希」）。

## 复跑与哈希

模型（只读被判文件、不编、不起 cargo）：

- `research/prompts/merge-verify-slots-governance-r3-sonnet-model/progress_note_mutants.py`　sha256 `efe3cde136a6efbd9b422ab0fe3f5fdd6c7a51075e20087a20db02921d4417c1`
- `research/prompts/merge-verify-slots-governance-r3-sonnet-model/tools/rerun.sh`（模板拷来、只改 `rerun_steps`）　sha256 `ef39331a4f45ad1c16b2d4deeefeec84f246179fc539bcf2225d7fd1d436f71d`
- `research/prompts/merge-verify-slots-governance-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh`　sha256 `447ea3ff36f97f5a68bc03138014595f47938d2c1be8863e14dd58a32fa9a9af`

模型没跑过：本会话直接起它被项目代码闸拒（见「要主 agent 跑的命令」），只做了 `python3 -m py_compile` 与 `bash -n`，都过。下面凡是「模型替身上的读数」都标「待主 agent 跑」，现在写的红绿是读源码与读自证推的，按「推的」办。

## 第 2 格 · 一、`crash-library-totals.sh` 第 280、281 行与文件头、`task_server_lib.py:287` 是不是同一说法

判定：**成立**（同一说法：编译目录归树、不归任务，由服务按 `--cwd` 那一棵树给）；有一处口径窄一点、一处自证没钉住，都不构成反例。

依据（引文都现查过，行号取命中的那一行）：

- 失败句第一行：「✗ 没有编译目录 CARGO_TARGET_DIR：读库程序要编，编译目录归树、由服务给，不照仓根的 target 编」（`scripts/crash/crash-library-totals.sh:280`）
- 失败句第二行：「→ 怎么办：经 task-server/task submit -- bash scripts/crash/crash-library-totals.sh 交（服务按 --cwd 那一棵树给编译目录，编译目录归树、不归任务）」（`scripts/crash/crash-library-totals.sh:281`）
- 文件头：「# 编译目录：照调用方的 CARGO_TARGET_DIR（经 task-server 交时服务按 --cwd 那一棵树给，编译目录归树、不归任务）；没设退 2、一个 cargo 都不起（出路是经 task-server/task submit 交）。」（`scripts/crash/crash-library-totals.sh:35`）
- 服务一侧：「wait 末行不带 build=（编译目录归树、不归任务）。弄坏开关 clean-removes-build-slot 下 task clean 照删各次尝试用过的组里那一个。」（`task-server/task_server_lib.py:287`）；同段上一行「任务记下结局、task clean <号>、取消都不删组里的：clean 只删那一条任务自己的现场（下面「现场」）与分片父任务的测试二进制；取消的照旧删它自己的现场，编译目录放回组里。」（`task-server/task_server_lib.py:286`）；「树是任务 --cwd 解开符号链接后往上第一层有 .git 的那一层」（`task-server/task_server_lib.py:263`，「按『哪一棵树 × 哪一台』给」）。

对齐：

- 「归树」「不归任务」两个词在 280/281、文件头 35 与 287 逐字同；「按 --cwd 那一棵树给」在 281、35 与 263 同一件事（树的定义在 263）。
- 280 写「由服务给」，比文件头 35 的「照调用方的 CARGO_TARGET_DIR（经 task-server 交时服务…给）」窄：`task_server_lib.py:288` 写调用方可以自己给仓外的目录（「落在仓外的照它、不删（归调用方），不进组」）。280 这句只在「一个 CARGO_TARGET_DIR 都没设」的前提下说，出路 281 又点名经服务交，所以不矛盾；它没说「只有服务能给」。
- 280 的「不照仓根的 target 编」与服务一侧同义：`TASK_SERVER_BREAK=target-directory-unset` 的说明写「命令照 cargo 默认往起它的目录下的 target 编」（`task-server/task_server_lib.py:651`），提交警告写「绕过了服务给的编译目录，取仓根 target/ 下的二进制」（`task-server/README.md:172`）。
- 脚本的 `task-server-compiles: yes` 声明在文件头第 3 行（`scripts/crash/crash-library-totals.sh:3`），服务据它给编译目录，与 281 说的「服务…给编译目录」一致。

没钉住的一处（推的）：自证里「没有 CARGO_TARGET_DIR」那一格只核退 2、没起 cargo、标准错误含子串 `CARGO_TARGET_DIR`：「if ! { [[ "$exit_code" == 2 ]] && calls_empty && stderr_has 'CARGO_TARGET_DIR'; }; then」（`scripts/crash/crash-library-totals.sh:477`）。280、281 两句的措辞没有自证钉着，改成与服务相反的说法（例：把 281 末尾改成「编译目录归任务」）这一格照绿。这不是现在的错，是这句话以后会漂的位置。

什么现象会推翻「同一说法」：服务一侧把编译目录改回按任务给（`task_server_lib.py:287` 的 `wait` 末行重新带 `build=`），或 `task_server_lib.py:263` 的树不再按 `--cwd`；那时 280/281 与文件头 35 要一起改。

## 第 2 格 · 二、`selftest_scratch_mentions` 新格分不分得出取头与取尾，有没有别的改坏照样绿

判定两句分开答：

- 分得出取头与取尾：**成立**。
- 没有别的改坏照样绿：**不成立**——有十一处改坏、`progress_note` 的几支，现在的自证与单元用例都杀不掉（推的；模型替身上的读数待主 agent 跑）。

### 分得出取头与取尾（成立）

- 新格造的进度文件是 `PROGRESS_TAIL_LINES + 2` 行：「total_lines = PROGRESS_TAIL_LINES + 2」（`scripts/agents/agent_watch_alerts.py:1323`），常量值 3：「PROGRESS_TAIL_LINES = 3」（`scripts/agents/agent_watch_alerts.py:64`）。五行里头三行与末三行只有「进度行03」共有。
- 断言三件一起判：「if f"末 {PROGRESS_TAIL_LINES} 行" not in note or not all(line in note for line in kept) or any(line in note for line in dropped):」（`scripts/agents/agent_watch_alerts.py:1330`）。取头得「进度行01、02、03」：`kept` 里的 04、05 不在、`dropped` 里的 01、02 在，两样都判红。
- 弄坏开关的取法在这一行：「lines = filled_lines[:PROGRESS_TAIL_LINES] if BROKEN_DETECTION == "progresstailhead" else filled_lines[-PROGRESS_TAIL_LINES:]」（`scripts/agents/agent_watch_alerts.py:184`）。
- 旧格分不出，作者已写明：「# 上面与 selftest_session_watch_in 造的进度文件都恰好 PROGRESS_TAIL_LINES 行，头几行与末几行是同一批，判不出取错了哪头。」（`scripts/agents/agent_watch_alerts.py:1319`）。旧格的进度文件是三行：「第一格跑完\n第二格跑完\n第三格在跑，预计 40 分钟\n」（`scripts/agents/agent_watch_alerts.py:1341`），断言只看最后一行在不在：「if inquiry is None or "第三格在跑，预计 40 分钟" not in inquiry[1]:」（`scripts/agents/agent_watch_alerts.py:1364`）。三行文件里头与尾是同一批，取头照绿，所以新格才有判别力。
- 开关表点名了这一格、没点别的：「AGENT_WATCH_BREAK=progresstailhead  progress_note() 取 progress.md 的头几行、不取末几行」（`scripts/agents/agent-watch.py:409`）。除新格外别的格用的进度文件都是三行或更短，取头与取尾同批，所以不会连带红别的格（推的）。
- 另外三种取错（取四行、取三行里漏末行、首行加末两行）也红：「末 N 行」的 N 不是 3 或 `kept` 缺行；模型里有这三条（`tail-takes-one-more-line`、`tail-drops-the-last-line`、`first-line-plus-last-two`），待主 agent 跑。

### 照样绿的改坏（不成立那一句的依据）

`progress_note` 取文件的那几行（`scripts/agents/agent_watch_alerts.py:183` 起）与拼句（`scripts/agents/agent_watch_alerts.py:188`）。下表每条都是「现有断言一条都没点到那一处」；「现有」指：新格（1330）、旧格（1364）、「没有草稿目录一行取不到」那一格（`scripts/agents/agent_watch_alerts.py:1315` 起三行）、单元用例 `test_progress_deleted_before_read_is_reported`（`scripts/agents/tests/test_agents.py:291`）。

| 改坏 | 为什么现有断言杀不掉（依据） | 模型里的名字 |
|---|---|---|
| 末几行次序反过来 | 新格逐行用 `line in note` 判成员，不判先后；旧格只含最后一行 | `tail-in-reverse-order` |
| 去掉空行过滤（`if line.strip()` 删掉） | 所有造的文件都没有空行，行 183 的过滤这一支走不到「有空行」 | `blank-lines-kept` |
| 不去首尾空白 | 造的行都没有首尾空白；断言是子串成员 | `lines-not-stripped` |
| 不截断 120 字（`line[:120]` 改成 `line` 或 `line[:20]`） | 最长的造的行是旧格那行 14 个字；没有断言造超过 120 字的行 | `no-truncation`、`truncation-at-twenty` |
| 空文件的「（空）」改成空串 | 没有一格造空进度文件 | `empty-text-blank` |
| 「末 {len(lines)} 行」改成「末 {PROGRESS_TAIL_LINES} 行」 | 造的文件都至少 3 行，行数不足 3 行时的字样没人判 | `count-is-the-constant` |
| 目录有、`progress.md` 没有那一支的文案（`scripts/agents/agent_watch_alerts.py:176`）改掉 | 全仓 `scripts/agents/` 下只有行 176 自己出现「草稿目录下没有」（`grep -nF` 命中 1 行） | `directory-without-file-text-dropped` |
| 「改于 N 前」的时长改成 0 或把 `now - modified` 的符号翻过来 | 「改于」全仓只出现在行 187，没有断言读它 | `age-always-zero`、`age-sign-flipped` |
| `errors="replace"` 改成 `strict`（遇到非 UTF-8 字节抛 `UnicodeDecodeError`，不是 `OSError`，不会被行 185 接住） | 造的文件都是合法 UTF-8 | `invalid-utf8-raises` |
| 行与行之间的分隔「 ｜ 」改掉 | 「｜」全仓只出现在行 188，没有断言读它 | `join-without-separator` |
| 读文件抛 `OSError` 那一支的文案（行 186）改掉 | 作者已写明自证造不出这个窗口（「# progress_note 读文件的另两支这里不判」，`scripts/agents/agent_watch_alerts.py:1320`；「# 读不到修改时刻时交 0.0 的改坏，输出与原来逐字相同，没有测试杀得掉。」，`scripts/agents/agent_watch_alerts.py:1322`）；单元用例只判读不到修改时刻那一支 | `os-error-branch-text-dropped` |

次序反转与空行过滤两条最值得补：看门狗告警里附的就是「末几行」，次序错了读的人读到的是倒着的进展。

能杀掉的对照：读不到修改时刻那一支的文案被改掉会被单元用例杀（`scripts/agents/tests/test_agents.py:291` 断言含「已消失或不可读」），模型里 `unreadable-mtime-text-dropped` 应当被杀；`progressnote` 开关（整句不附）会被旧格与单元用例杀。

反例输入（要补一格才抓得到，写给主 agent 采不采）：进度文件内容 `甲行\n\n` + 130 个「乙」+ `\n丙行  \n\n丁行\n\n\n`，期望附句里有「末 3 行：」后跟 120 个「乙」 ｜ 丙行 ｜ 丁行（次序、空行、空白、截断、分隔五样一次判）；再加一份只有一行的文件期望「末 1 行：」、一份空文件期望「末 0 行：（空）」。模型的 `proposed` 一组就是这几格，待主 agent 跑出「提议那一组」能否把上面十一条全杀。

什么现象会推翻这一结论：模型跑出表里某一条 `real_red` 不是 `none`（现有断言其实能杀它，我读漏了一处）；或 `agent-watch.py --selftest` 在改坏了的副本上红了点名的格（模型替身与真函数在那一处对不上）。

## 第 2 格 · 三、`.claude/agent-common.md:37` 新句与服务两处是不是同一件事，合不合规则文件铁律

判定两句分开答：

- 与 `task-server/task_server_lib.py:286`、`task-server/README.md:224` 是不是同一件事：**成立**（同一件事；口径只差一处，不改它的结论）。
- 合不合 `.claude/singlefs-ai-sop/rules/rules-discipline.md`：**成立**（照它「1. 正文只写四样」「2. 判据留下，论证搬走」「3. 搬出来的东西直接删掉」「7. 正文不许用位置指代，也不许自称」逐条对过，没有违例）。

### 与服务两处对

- 新句：「经服务跑的编译目录由服务按树给、归树不归任务：任务记下结局、`task-server/task clean <号>`、取消都不删它，树（含仓副本）删了服务才跟着删，谁都不自己 `rm`；`task clean <号>` 只清那一条任务自己的现场。」（`.claude/agent-common.md:37`，整句抄在「用完就删」那一条里）
- 服务文件头：「任务记下结局、task clean <号>、取消都不删组里的：clean 只删那一条任务自己的现场（下面「现场」）与分片父任务的测试二进制；取消的照旧删它自己的现场，编译目录放回组里。」（`task-server/task_server_lib.py:286`）；「树删了编译目录跟着删——组里那一行的树此刻不是目录了、那一个空着的，服务删它」（`task-server/task_server_lib.py:283`）
- 服务 README：「任务记下结局、`task clean`、取消都不删组里的编译目录。」（`task-server/README.md:224`）；「树是任务 `--cwd` 解开符号链接后往上第一层有 `.git` 的那一层（仓副本是 git 工作树，各是一棵）」（`task-server/README.md:221`）

对得上的四件事：结局、`task clean`、取消三样都不删编译目录（三处同）；删树才删它（283 与 224 同）；编译目录按树给、归树不归任务（286 注、287 同）；仓副本是一棵树（221 与 `scripts/hosts/minimal-copy.py:28`「副本是 git 工作树」同）。

口径差一处（不改结论）：服务把 `task clean` 删的东西写成「现场」加「分片父任务的测试二进制」两样；新句只写「现场」。代码里 `clean` 对分片父任务确实多删一份测试二进制：「if task.get("sharded_entry"):   # 分片的父任务：计划目录下预先编好的测试二进制一并删（文件头 ④）」（`task-server/task_server_scheduler_starting.py:752`）。这一句的要点是「编译目录不被删」，多删的那一份不是编译目录，所以不影响新句；子 agent 又不起 `task clean`，没有读错的后果。要对齐就写成「只清那一条任务自己的现场（分片的父任务另有计划目录下的测试二进制）」，不对齐也不错。

没有被新句盖住的东西：调用方自己给了仓外编译目录的（归调用方、不进组：`task-server/task_server_lib.py:288`），新句开头就限定「经服务跑的编译目录由服务按树给」，没把它们算进「谁都不自己 `rm`」；同一条前半句「删不掉的编译目录、工作树与仓副本…交主 agent 删」管的就是这类自己建的。

### 合规则文件铁律

- 「1. 正文只写四样」：新句是「做什么、不做什么」加「射程」（不自己 `rm`，哪些编译目录算）。「任务记下结局、clean、取消都不删它」是服务的行为、不是对子 agent 的指令，但它回答「删掉这句，读的人少知道什么」：原来这一条写的是「交任务的主 agent 用完 `task-server/task clean <号>` 删」（取自 `git diff HEAD` 里被删的旧句），不写这半句读的人会照旧去 `clean` 并以为删掉了。按「2. 判据留下，论证搬走」的判据留得住；没有「为什么这么定」的论证句。
- 「3. 搬出来的东西直接删掉」：第 37 行对「原为、此前、原先、起初、改成、换成」与「本节、本条、本表、本文档、本章节」逐个 `grep -noE` 零命中；旧句整句被换掉，没留「原先由主 agent clean」的痕迹。
- 「7. 正文不许用位置指代，也不许自称」：新句里没有「本节、本条」这类指文档结构的词；指别处用的是小节名（「用完就删」所在条目自己的上下文）、文件路径，没有「上面」「下面」。
- 这份文件自己的开头也只收「怎么做」：「共用约束与各份定义只写怎么做：步骤、约束、判据、交付。」（`.claude/agent-common.md:4`）。新句没有实测、日期、经过。
- 门禁判字面那一半：阶段「规则纪律（项目本地）」把这份文件交给 rules-lint：「    ".claude/agents/*.md" ".claude/agent-common.md" ".claude/main-agent.md" ".claude/skills/*/SKILL.md"」（`.claude/singlefs-ai-sop/scripts/gate.sh:505`），`.claude/rules-lint-exclude` 不存在、没有排除。这一句我没单跑那一阶段（要起脚本，待主 agent 跑，命令在「要主 agent 跑的命令」）；上面逐条对是读出来的，「第 2 条的分界」本来就归 review。

什么现象会推翻：服务把 `task clean` 改成会删编译目录（`clean-removes-build-slot` 开关打开时的行为变成默认），或把「树删了才删」改成别的触发；这时 `.claude/agent-common.md` 的 37 行、`task-server/task_server_lib.py` 的 283、286 行与 `task-server/README.md` 的 224 行要同改。另：同一说法的技能与钩子几处归别的会话在改，我只现查了 `.claude/skills/dispatch/SKILL.md:18` 已是新说法，别的没逐处查。

## 第 3 格 · 三处登记行改完之后锚串命中、理由与代码或现场对得上

判定：三处都 **成立**；一处有口径差、一处有一句没核到，各写在下面。命中数都是我这一次 `grep -c` / `ls` / `awk` 现数的（量的）；命中不等于「门禁绿」，门禁两格我没跑（要起脚本，命令在「要主 agent 跑的命令」）。

### 一、`.claude/memory-bounded-parallelism.tsv:10`

行（入口是 `.claude/gate.d/code-tooling.sh`，理由列整段抄）：「门禁拆成入口加 lib/ 下的库，入口罩着整族：research-script-selftests 格按每台默认重活线程数分批并行，两条真仓现算单元图的大自证同批会叠到超过每台内存上限，大自证登记在 SELFTEST_RUNNERS_MEMORY_BOUND、各占一批、同一时刻至多一条；break-switch-replay、selftest-cells-alone、stage-break-switch-replay 三格各只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory、stage-replay-jobs-by-memory 下按内存上限与每趟内存算同时跑几条（⌊份额 ÷ 每趟内存⌋），平常不按内存算」（`.claude/memory-bounded-parallelism.tsv:10`）

锚串命中（`grep -rl` / `grep -rn` 现数）：

| 锚串 | 命中 | 落在哪 |
|---|---|---|
| 入口 `.claude/gate.d/code-tooling.sh` | 文件在（`ls` 命中 1）；表里这个入口登记 1 行（`awk -F'\t'` 数各行第 1 列，每个入口一行） | |
| `SELFTEST_RUNNERS_MEMORY_BOUND` | 门禁库里 1 份文件：`.claude/gate.d/lib/code-tooling-selftest-cells.sh`（行 22、210、211、574 等多行）；另有样本目录 `fixtures/code-tooling.sh/runner-memory-bound-*` 3 份 | 定义在 211 |
| `replay-jobs-by-memory` | 3 份：`code-tooling.sh`、`lib/code-tooling-break-switch-replay-cells.sh`、`lib/code-tooling-stage-break-switch-replay-cells.sh` | 判法在 779 |
| `cells-alone-jobs-by-memory` | 3 份：`code-tooling.sh`、`lib/code-tooling-copy-closure-and-selftest-shape-cells.sh`、`lib/code-tooling-stage-break-switch-replay-cells.sh` | 判法在 660 |
| `stage-replay-jobs-by-memory` | 2 份：`code-tooling.sh`、`lib/code-tooling-stage-break-switch-replay-cells.sh` | 判法在 1037 |
| 格名 `research-script-selftests`、`break-switch-replay`、`selftest-cells-alone`、`stage-break-switch-replay` | 各 1 行 `# gate-cell:` 声明，都在 `code-tooling.sh`（行 23、54、64、52） | |

（`replay-jobs-by-memory` 是 `stage-replay-jobs-by-memory` 的子串，所以 `grep -F` 它会把后者也数进去；上表按文件名与整词分开看。）

理由与代码对得上：

- 「research-script-selftests 格按每台默认重活线程数分批并行」：「N = 每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default：SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default 的登记值；」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:20`）。
- 「大自证…各占一批、同一时刻至多一条」：「SELFTEST_RUNNERS_MEMORY_BOUND 里的大自证（真仓现算整张单元图的这类，一条十几 GiB；这一道整格登记在 .claude/memory-bounded-parallelism.tsv 里准许按内存定并发）各占一批：」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:22`）。表里登记两条：「SELFTEST_RUNNERS_MEMORY_BOUND=("python3 scripts/gate/lib_function_units.py --selftest" "python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/verus-proof-check.py --selftest")」（`.claude/gate.d/lib/code-tooling-selftest-cells.sh:211`），与「两条」对得上（量的：数组两项）。「真仓现算单元图」这半句对第二条（`verus-proof-check.py --selftest`）我没核到它在自证里现算整张单元图：该脚本只读登记表、核经 `scripts/gate/lib_function_units.py` 切单元（`scripts/gate/verus-proof-check.py` 文件头），自证夹具里也引 `lib_function_units`，但「整张真仓单元图」没读到，标**推的**，出处是库文件头 22 行那句自己的说法。
- 三格「平常不按内存算」：break-switch-replay：「# 并行度（这一格文件头「并行度」）：并行条数取登记的每台默认重活线程数，内存上限只写进开跑那一行。」（`.claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh:776`），按内存的那一支在开关下：「jobs = parallel_library.jobs_for(threads, memory_limit, fallback_trip_bytes) if by_memory else threads」（`.claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh:780`）；selftest-cells-alone 同形：「jobs = parallel_library.jobs_for(default_threads, memory_limit, parallel_library.TRIP_MEMORY_FALLBACK_BYTES) if by_memory else default_threads」（`.claude/gate.d/lib/code-tooling-copy-closure-and-selftest-shape-cells.sh:660`）；stage-break-switch-replay：平常「memory_text = f"{limits_text}；由服务定、撞了服务照乘 1.5 放大重排，不按它收并行"」（`.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh:1032`），开关下才收：「if broken == "stage-replay-jobs-by-memory" and budget is not None and known_peaks:」（`.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh:1037`）。
- 口径差（不改结论）：理由对三格都写「⌊份额 ÷ 每趟内存⌋」。前两格是这个式子（`.claude/gate.d/code-tooling.sh:925`「⌊上限的份额 ÷ 每趟内存⌋、不超过线程数，parallel_library.jobs_for」、`.claude/gate.d/code-tooling.sh:974` 同）；stage-break-switch-replay 的除数是样本根写的「那一道阶段每条开关开满时的峰值」、被除数是「这一台能给的与这一条拿到的上限里小的、扣掉本进程用量」：「max_running = max(1, min(max_running, budget // max(known_peaks)))」（`.claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh:1038`），是 `⌊份额 ÷ 峰值⌋`，不是「每趟内存」那个常量。同形（整除、取份额）、不同的除数与份额口径；登记表只要求「理由写明为什么不能照每台默认重活线程数同时开」，所以不致红，但「每趟内存」对第三格是近似说法。

什么现象会推翻：门禁 code-tooling 的 `memory-bounded-parallelism` 格单跑红（它先判入口在不在、再判入口罩着的文件里有没有按内存定并发的写法）；或 `SELFTEST_RUNNERS_MEMORY_BOUND` 与三条开关里任何一个在代码里改了名（上表命中数变 0）。

### 二、`.claude/preflight-exclude` e203 四行（第 164 至 167 行）

四行的理由列相同（`sed -n 164,167p … | sort -u | wc -l` 得 1，量的）：「被 scripts/experiment-apparatus/e203_admission_scan_cost_counters.py import 的模块，没有自己的入口，不单独调；入口自己的准入与运行条件归入口那一份，preflight-lint 照判入口」（`.claude/preflight-exclude:164`，另三行 165、166、167 只有路径不同）。

锚串命中（量的）：

| 锚串 | 命中 |
|---|---|
| 四个模块路径（`_applying_edits_and_copying_the_tree`、`_insertion_edits`、`_row_table_and_module_template`、`_selftest`）各在排除表里出现 | 各 1 行（`grep -cF "<路径>  #"`）；文件都在（`ls` 命中 5 份：入口加四个模块） |
| 入口 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 在排除表里 | 0 行（`grep -n e203 .claude/preflight-exclude` 只命中 164 至 167 四行）：入口没被排除，这与理由「preflight-lint 照判入口」一致 |
| 入口 `import` 这四个模块 | 各 1 行：「from e203_admission_scan_cost_counters_row_table_and_module_template import *  # noqa: E402,F401,F403」（`scripts/experiment-apparatus/e203_admission_scan_cost_counters.py:39`），另三个在 40、41、42 |
| 四个模块里有自己的入口（`__main__`、`def main`、`argparse`、`sys.argv`） | 各 0 行（`grep -nE` 零命中） |
| 谁单独起这四个模块（`python3 <模块>.py`），在 `research`、`.claude`、`scripts`、`crates`、`task-server` 下 | 0 份文件（`grep -rlE`），只有它们互相 `import` 与入口 `import` |

理由与现场对得上：

- 「被…import 的模块，没有自己的入口，不单独调」：成立，见上表后三行；规则文件铁律也这样分类：「不判的三类：被 source 或 import 的库、样本、只 exec 共享脚本的包装」（`.claude/singlefs-ai-sop/rules/preflight-discipline.md:16`）。
- 「preflight-lint 照判入口」：成立。判的范围是登记目录那一层：「scripts/experiment-apparatus  # 实验二进制（E<号>_ 开头的计数模型与装置）」（`.claude/preflight-dirs:2`），后缀含 `.py`：「SCRIPT_SUFFIXES = ('.sh', '.py', '.rs')」（`.claude/singlefs-ai-sop/scripts/preflight-lint.py:71`）；入口不在排除表，所以被判。入口文件头没有 `admission:`、`run-condition:` 两行（`grep -n -i '^# *\(admission\|run-condition\)'` 零命中），lint 对缺声明点名：「if section not in declared_sections and not problems:」（`.claude/singlefs-ai-sop/scripts/preflight-lint.py:401`）。
- 要分清的一点：这句理由**没说**入口合规。第二轮判决记着入口现在就红：「e203 只剩入口 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 三行」（`research/prompts/merge-verify-slots-governance-r2-main-verification.md:35`，主 agent 的观测，我没复跑 lint，待主 agent 跑）。换上的新理由比旧理由对：旧的写「准入与运行条件由入口判」，而入口「自己既没写 `admission:`、也不调 `preflight`」（同一份判决:52），新的只说「归入口那一份、lint 照判入口」，不假装入口已经写了。入口那三行是另一笔存量，不属于这四行排除，也没被这四行盖住。

什么现象会推翻：四个模块里任何一个长出 `__main__`（它就成了入口，排除表要删那一行）；或有命令开始单独起某个模块；或入口被加进排除表（那「照判入口」就不真了）；或 lint 单跑给出「排除的…不在判的范围里」点名这四行（`.claude/singlefs-ai-sop/scripts/preflight-lint.py:464`）。

### 三、`scripts/experiment-apparatus/README.md:7`

行（整行抄）：「- 分成多份模块的装置各占一层子目录，`main.rs` 是目标根（`Cargo.toml` 的 `path` 指它），例 `e139_tightened_floor/`、`e203_admission_scan_cost/`，全部用 `ls scripts/experiment-apparatus/*/main.rs` 列。按职责分的子模块是根的直接孩子（`<名>.rs`），单测在 `<名>/tests.rs`，几份测试共用的辅助在 `test_support_for_<…>.rs`；根里每个子模块 `mod <名>;` 加 `use self::<名>::*;`，子模块 `use super::*;`。装置读写自己源文件的地方（`preflight` 的源文件路径、`include_str!`、`include!`）都按 `main.rs` 所在的这一层写。`e203_admission_scan_cost_counters.py` 是入口，同目录 `e203_admission_scan_cost_counters_<部分>.py` 四份是它拆出的部分，入口开头把它们的名字原样再导出，副本的输入键与变异检查读这五份合起来的全文。」（`scripts/experiment-apparatus/README.md:7`）

锚串命中（量的，`ls`、`grep -cF`、`awk` 现数）：

| 锚串 | 命中 |
|---|---|
| `ls scripts/experiment-apparatus/*/main.rs` | 31 份 |
| 这 31 份各自被 `scripts/experiment-apparatus/Cargo.toml` 的 `path = "<目录>/main.rs"` 指到 | 31 份各恰好 1 行（逐份 `grep -cF`，没有 0 或 2） |
| 例 `e139_tightened_floor/`、`e203_admission_scan_cost/` | 两个目录都在、各有 `main.rs`，`Cargo.toml` 里 `path = "e139_tightened_floor/main.rs"`（131、132 行附近）与 `path = "e203_admission_scan_cost/main.rs"`（787 行）都在 |
| 入口加四个部分「五份」 | 5 份：入口 1、`e203_admission_scan_cost_counters_<部分>.py` 4（`ls` 命中 5） |
| 「入口开头把它们的名字原样再导出」 | 入口文件头：「这个文件是入口：放变异检查、弄坏开关的判断与 `main`；其余在同目录 `e203_admission_scan_cost_counters_<部分>.py` 四份（行表与读数模块文字、」（`scripts/experiment-apparatus/e203_admission_scan_cost_counters.py:4`）与 39 至 42 行四条 `import *` |
| 「副本的输入键与变异检查读这五份合起来的全文」 | 「# 这个脚本的全部文件：入口与拆出的各部分（都在同一个目录）。副本的输入键与变异检查按这张表读本脚本全文。」（`scripts/experiment-apparatus/e203_admission_scan_cost_counters_row_table_and_module_template.py:1`）；表本身五项，入口读它：「script_sources = {name: (directory_of_the_script / name).read_text(encoding="utf-8") for name in SCRIPT_FILE_NAMES}」（`scripts/experiment-apparatus/e203_admission_scan_cost_counters.py:54`） |

与现场的一处要读清：`scripts/experiment-apparatus/` 下共有 48 个子目录（`ls -d */` 数，量的），其中 17 个没有 `main.rs`（打印清单：`e170_*` 六个、`e171_*` 两个、`e176_crates_oracle`、`e183_*`、`e197_*`、`e200_*`、`e201_*`、`heavy_threads_default`、`long-run-lib`，外加 `__pycache__`、`target`）。我抽看了其中七个，里面是 Python、shell 或单份 `.rs`，不是「分成多份模块、以 `main.rs` 为目标根」的 Rust 装置；这一行写的是「分成多份模块的装置各占一层子目录」加「全部用 `ls …/*/main.rs` 列」，范围就是有 `main.rs` 的那 31 个，没把 48 个都说成这一类，所以对得上。我没逐个核另外十个子目录里有没有「分成多份模块的 Rust 装置」而没写 `main.rs` 的；`Cargo.toml` 里 `path = "…/main.rs"` 共 31 行，与 `ls */main.rs` 的 31 份逐个相同（两份排序后 `diff` 无输出，量的），所以 `ls` 列出的恰是进了 `Cargo.toml` 的那些。

什么现象会推翻：`ls scripts/experiment-apparatus/*/main.rs` 的条数与 `Cargo.toml` 里指向 `*/main.rs` 的 `path` 条数不一致（现在都是 31）；或出现一个目录式 Rust 装置的目标根不叫 `main.rs`；或 e203 入口的 `SCRIPT_FILE_NAMES` 不再是五项。

## 要主 agent 跑的命令

我没往统一任务服务交任务（项目代码闸拒了我直接起模型）。下面整条由主 agent 交；每条写明了在哪棵树里起、跑完看哪几行。

1. 模型（看门狗新格取头取尾、别的改坏的红绿）。先建空目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-sonnet-cwd`（它没有 `.git`，服务按它本身认树），再交：

```
task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-sonnet-cwd -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r3-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-sonnet-model-run /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r3-sonnet-model
```

   - 第一个 `/home/fy5090/code/singlefs` 是仓根、第二个是冻结副本（模型只读 `scripts/agents/agent_watch_alerts.py`，它此刻的哈希与开工快照一致：`b42822f0f084b4f479be0a65d5d57e651446666220a0721703e67d12b54049c4`，所以直接给仓根；如果这个文件在我之后又被别的会话改了，换成开工快照还原出的那份）。运行目录以 `-run` 收尾，包装先删它再跑。
   - 看哪几行：`MODEL_BASELINE tail_lines=3 real_red=none proposed_red=none`（不是就是替身与真函数对不上，下面读数作废）；`MODEL_MUTANT` 共 20 行（18 个改坏加 `switch-progresstailhead`、`switch-progressnote`）；`MODEL_SUMMARY` 一行；末行是 `grep -c` 数出的 20；`task wait` 末行退 0。
   - 我按读源码预测的读数（推的，跑出来对不上就是我读错了）：`killed-by-real` 7 条（`tail-takes-one-more-line`、`tail-drops-the-last-line`、`first-line-plus-last-two`、`truncation-at-three`、`unreadable-mtime-text-dropped`、`switch-progresstailhead`、`switch-progressnote`），`only-proposed-kills` 13 条（其余），`survived-everything` 0 条；`switch-progresstailhead` 的 `real_red` 只有 `tail-cell-more-lines-than-limit` 一格。
   - 内存与时间：没量。模型只读一个文件、起 20 趟只在内存里 `exec` 一个函数加写几个小文件，预计秒级、几十 MiB；这是推的，以服务给的结局行为准。
   - 我没跑 `rerun.sh --selftest`：模板自带的自证要求 `rerun_steps` 拷整棵冻结副本（约 2 GiB），这一腿换成了只读一个文件，所以那一格拷树的断言对它不适用。

2. 登记表一格（第 3 格第一处）。在仓根里起：

```
task-server/task submit --cwd /home/fy5090/code/singlefs -- bash .claude/gate.d/code-tooling.sh --check memory-bounded-parallelism
```

   看汇总行是 `✓` 且报了查了多少项；报 77 或 0 项不算过。

3. 准入与运行条件（第 3 格第二处）。在仓根里起：

```
task-server/task submit --cwd /home/fy5090/code/singlefs -- python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py /home/fy5090/code/singlefs
```

   看：e203 只剩入口 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py` 的几行（第二轮判决记着三行）；四个模块名一行都不出现；没有「排除的…不在判的范围里」点名这四行。

4. 共用约束字面那一半（第 2 格第三项）。在仓根里起（`RULES_LINT_FILES` 若要相对仓根的路径，把绝对路径换成 `.claude/agent-common.md`）：

```
task-server/task submit --cwd /home/fy5090/code/singlefs -- env RULES_LINT_DIR=/home/fy5090/code/singlefs/.claude/rules RULES_LINT_FILES=/home/fy5090/code/singlefs/.claude/agent-common.md bash .claude/singlefs-ai-sop/scripts/rules-lint.sh /home/fy5090/code/singlefs
```

   看：没有点名 `.claude/agent-common.md:37` 的行。这条的调用写法（`RULES_LINT_DIR`、`RULES_LINT_FILES` 怎么给）我是按它文件头读来的，没跑过，写错了会退 2；那就改用门禁阶段「规则纪律（项目本地）」整段（`gate.sh` 里 `run_rules_lint` 调的那一段）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 第 2 格 一：`crash-library-totals.sh` 280、281 与文件头、`task-server/task_server_lib.py:287` | 成立 | 同一说法「编译目录归树、不归任务、按 `--cwd` 那棵树由服务给」；280 比文件头窄（没提调用方可给仓外目录）不矛盾；自证只钉子串 `CARGO_TARGET_DIR`，两句的措辞没人钉 |
| 第 2 格 二（a）：新格分不分得出取头与取尾 | 成立 | 五行文件、末三行 vs 头三行只共用一行，断言判行数字样、`kept` 全在、`dropped` 全不在；取头照红 |
| 第 2 格 二（b）：有没有别的改坏照样绿 | 不成立（有照样绿的） | 十一处：次序反转、空行过滤、首尾空白、120 字截断、空文件「（空）」、`len(lines)`、目录有文件无的文案、「改于」时长、`errors="replace"`、分隔「 ｜ 」、读文件抛 `OSError` 那一支；模型待主 agent 跑 |
| 第 2 格 三（a）：`.claude/agent-common.md:37` 与 `task-server/task_server_lib.py:286`、`task-server/README.md:224` 同不同 | 成立 | 同一件事；服务把 `clean` 多删的分片测试二进制写了，新句只写「现场」，不影响「不删编译目录」这个要点 |
| 第 2 格 三（b）：合不合 `rules-discipline.md` | 成立（读出来的） | 无历史痕迹、无位置指代、判据留下；字面那一半的 lint 我没跑，命令在上面第 4 条 |
| 第 3 格 一：`.claude/memory-bounded-parallelism.tsv:10` | 成立 | 入口与六个锚串都命中（`SELFTEST_RUNNERS_MEMORY_BOUND` 与三条开关、四个格名）；一处口径差（第三格除数是阶段峰值不是「每趟内存」）、一句「真仓现算」对第二条自证只能标推的 |
| 第 3 格 二：`preflight-exclude` e203 四行 | 成立 | 四个模块各 1 行、入口 import 各 1 行、模块里入口特征 0 行、没人单独起；新理由不再谎称入口合规，入口那三行是另一笔存量 |
| 第 3 格 三：`scripts/experiment-apparatus/README.md:7` | 成立 | 31 份 `main.rs` 与 `Cargo.toml` 的 31 条 `path` 逐个相同；两个例子、五份 e203 文件、再导出与全文键都对上 |

## 没做什么

- 不判别的腿的格（第 1、4 格归攻方腿），不替主 agent 采纳或出判决；没往统一任务服务交任务，判决要引的数一个都没复跑，命令在「要主 agent 跑的命令」；模型没跑过，里面的红绿预测与「十一处照样绿」都按「推的」办。
- 没起门禁的 `memory-bounded-parallelism` 格、`preflight-lint`、`rules-lint`：写的命中数是用 `grep`、`ls`、`awk` 对文件现数的，不是格的结局。
- 没核 `scripts/gate/verus-proof-check.py --selftest` 是否「真仓现算整张单元图」；没逐个核 `scripts/experiment-apparatus/` 下另外十个没有 `main.rs` 的子目录；没逐处核「同一说法的技能与钩子几处」，只看了 `.claude/skills/dispatch/SKILL.md:18`。
- 没读禁读清单里的另一条腿的报告与模型。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-sonnet/` 里没建副本、没编译目录（交回前它是空的）。
