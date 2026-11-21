# 正推腿报告（Sonnet）：四处协作工具改动第三轮，第 5、6 问（wrapped-writers-loop-paths-cite-titles-governance-r3，2026-11-21）

分给这条腿的是第 5 问（「实现今天的样子」每一条回「成立」「不成立」「推不出」）与第 6 问（第二轮判决里「推的」「核不动」的逐条现查）。没碰攻方腿的第 1 到 4 问；没读禁读清单里的文件（另一条腿这一轮的报告）。

## 读到的范围与证据标记

- 被判 14 份读主工作区此刻那一版：`sha256sum -c research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-snapshot/judged-sha256.txt` 14 行都是 OK（量的，我跑的）。两条腿都读这 14 份，这条腿另读了 `.claude/hooks/` 下检出钩子的 python 部分、`scripts/gate/hook-swap.py` 与 `scripts/gate/hook_swap_swap.py`（核换钩子结局文件的语义）、`scripts/mutation/mutate.sh` 的代码段。
- 改前原样：任务服务取草稿 `~/.local/share/singlefs/work/task-server-r2-fix/base/task-server/`（它的 `task_server_scheduler_building.py`、`task_server_scheduler_lifecycle.py` 的 sha256 与第二轮开工快照 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-snapshot/judged-sha256.txt` 里同名两行逐字相同，量的）；引文核对取 `~/.local/share/singlefs/work/cite-check-r2-fix/base/`（与 `cite-check-title-parens-r1-fix/new/` 的 `cite-check.py` 逐字节相同，`cmp` 量的）；检出钩子的两个旧状态取 `~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base/hooks/` 与 `probe-new/hooks/`。
- 证据标记：「量」是我直接跑出来的只读命令（`grep`、`sha256sum`、`cmp`、`python3 scripts/agents/cite-check.py`，以及只用标准库读文本的 `python3 -I`）；「读」是读代码推的；「待主 agent 跑」写进末尾「要主 agent 跑的命令」。没有一条结论靠转述。
- 引文约定：改前草稿里的行写绝对路径；仓内的行写相对仓根的路径。引的行都是写报告时现 `grep -nF` 取的。

## 第 5 问：「实现今天的样子」逐条

正文那一节共十一条陈述（任务服务 5 条、引文核对 1 条、检出钩子 1 条、变异脚本 1 条、被判范围与别的会话那一条 2 条、读数 1 条），下面按它们的次序写，每条一个小标题。

### 任务服务·切词与算当前目录（正文「shell_text_pieces 把花括号组记进分隔…三形都落到算不出」）：成立（读）

- 花括号组记进分隔：「elif braces and token in SHELL_GROUP_BRACES and not current:」（`task-server/task_server_scheduler_building.py:134`），命令位置上单独一个的 `{`、`}` 追加进分隔、不当词。
- 层数把它数进去：「depth += sum(token.count("(") + token.count("{") - token.count(")") - token.count("}") for token in joint)」（`task-server/task_server_scheduler_building.py:210`）。
- heredoc 正文的行标上不算平铺：「line_marks = ([PIECE_HEREDOC_BODY] if body else []) + ([PIECE_UNLEXED] if unlexed else [])」（`task-server/task_server_scheduler_building.py:110`）；平铺的前置分隔只许行首与分号：「SHELL_STRAIGHT_BEFORE = (PIECE_LINE_START, ";")」（`task-server/task_server_scheduler_building.py:52`），`<<` 不在里面，正文里的 `cd` 不算平铺。
- 串里自己带 env 的 -C / --chdir 的那一条目录算不出：「moved_itself = chdir_read and shell_directory_option(words[:lib.command_word_index(words, environment)])」（`task-server/task_server_scheduler_building.py:234`）与「runs.append(ShellRun(list(words), expanded, None if moved_itself else current))」（`task-server/task_server_scheduler_building.py:235`）。
- 「三形都落到算不出、照可能写了一半报」逐形走今天的码（推导同第 6 问里对改前的写法，这里只写落点）：
  - 花括号组接管道（`{ cd nested; true; } | cat` 换行 `python3 scripts/gate/apply-writer-patch.py patch-half`）：`{` 记进 `cd nested` 那条的分隔、`}` 记进 `cat` 那条的分隔，层数在 `cd nested` 处是 1，它不平铺，整段 `directory_known` 为假，打补丁那条的当前目录是 None；`writer_interruption` 里「if (patch is None or root_value is None」（`task-server/task_server_scheduler_lifecycle.py:1851`）的第三支 `directory is None` 且补丁目录是相对的成立，交带「从这条命令里算不出（变量、位置参数、命令替换或 cd 到算不出的目录），没读结局文件」（`task-server/task_server_scheduler_lifecycle.py:1854`）的那一项，照可能写了一半报。
  - heredoc 正文里的 `cd`（`cat > notes.txt <<'EOF'`、`cd nested`、定界词、打补丁）：切得开的行里 `<<` 之后的词记成定界词，`cd nested` 那一行的分隔带 `<<` 记号、不平铺，同上落到 None、算不出。
  - 串里的 `env -C nested python3 …`：`command_word_index` 剥掉 env 与它的 `-C nested` 之后，`words[:index]` 里有 `-C`，这一条自己的当前目录是 None，同上。
- 旁见（不是这一条陈述的错，推的）：`{ … ; }` 不接管道也不接 `&` 的平常写法（`bash -c '{ cd nested && python3 scripts/gate/apply-writer-patch.py patch-nested-half; }'`）改前算出 `nested`（组跑在当前 shell，算得对），今天因花括号层把整段当「算不出」，一条已经打全的补丁被杀也照「可能写了一半」报（多报，方向偏严）。正文与 `shell_text_runs` 文档串都写了这是花括号层的取舍，没写这一形；攻方腿第 1 问问的就是这类多报。
- 什么现象会推翻：对今天的 `shell_text_runs` 喂这三串，打补丁那一条的 `directory` 不是 None。

### 任务服务·`hook_swap_read_back` 与 `workspace_write_judged`（正文 `hook_swap_read_back` 那一条）：成立（读），行为变化一处也成立

- 新方法在：「def hook_swap_read_back(self, writer, tool, after, after_expanded, named_paths):」（`task-server/task_server_scheduler_lifecycle.py:1869`）；带这一条自己的 `--root` 与 `--scratch`（`for option in ("--root", "--scratch"):` 一段，`task-server/task_server_scheduler_lifecycle.py:1878-1887`），在这一条起时的当前目录里起：「directory = writer.directory or writer.cwd」（`task-server/task_server_scheduler_lifecycle.py:1884`）。
- 换完了、已退回不报：「if verdict in (0, 1):」（`task-server/task_server_scheduler_lifecycle.py:1803`）；退码含义对着 `scripts/gate/hook-swap.py` 文件头：「--read-outcome 只看不写：退 0 换完了（有完成，每份文件现在都等于回读）」（`scripts/gate/hook-swap.py:70`），1 已退回。
- 没换完而归另一条：「if (verdict == HOOK_SWAP_UNFINISHED_VERDICT and one.owner_marks and not owner_ignored」（`task-server/task_server_scheduler_lifecycle.py:1806`）加「and any(mark in text for mark in task_marks) and not any(mark in text for mark in one.owner_marks)):」（`task-server/task_server_scheduler_lifecycle.py:1807`）：输出里点名了这条任务里别条的反向规格路径、又没点名自己的，才不报。
- `subprocess.run(read_back, …)` 留在 `workspace_write_judged`：「completed = subprocess.run(read_back, cwd=one.directory, capture_output=True, text=True, env=self.environment, stdin=subprocess.DEVNULL,」（`task-server/task_server_scheduler_lifecycle.py:1797`）落在 `workspace_write_judged`（定义行 1773，到 1818 行结束，下一个方法 `writer_interruption` 从 1820 行起）；算键库登记 `scripts/gate/lib_function_units_registry.py:256` 的键是（任务服务生命周期文件、`SchedulerLifecycle.workspace_write_judged`、`read_back`），对得上。
- 「被杀在写开始行之前的换钩子（结局文件里还是上一趟的完成）不报了」成立，而且是该报的对面：开始行、说明与每份文件的行是一次原子写——「write_outcome_atomically(path, outcome_text([start]) + "".join(f"{line}\n" for line in OUTCOME_COMMENTS) + outcome_text(files))」（`scripts/gate/hook_swap_swap.py:256`），原子写的定义：「同目录临时文件写完、刷盘，再改名换上，再刷目录：读的人看到旧的全文或新的全文，没有半截。」（`scripts/gate/hook_swap_swap.py:221`）；文件头：「所以换上之前先把这一趟要换哪几份、各自换前与换后的 sha256、反向规格在哪写进结局文件（同目录临时文件写完、刷盘再改名）」（`scripts/gate/hook-swap.py:50`）。换第一份文件发生在这次原子写之后，所以「开始行写了一半」「换了第一份而结局文件还是旧的」两种在 hook-swap.py 的写法下都不会出现：被杀在这次改名之前，主工作区一个字节没动；之后结局文件已有开始行与文件行，`--read-outcome` 退 3，今天的码照报。
- 什么现象会推翻：一趟被杀的真换，主工作区已有一份文件换成换后的内容、而结局文件还是上一趟的「完成」；即 `hook_swap_swap.py` 里换第一份文件的调用排在第 256 行那次写之前。次序读码核过：「write_outcome_start(outcome_file, root, reverse_spec_path, relatives, base_bytes, planned, mode)」（`scripts/gate/hook_swap_swap.py:1066`）排在换文件的循环里「replace_file_contents_by_rename(path, planned[relative], check_before_rename=unchanged_since_draft)」（`scripts/gate/hook_swap_swap.py:1075`）之前；只有弄坏开关 outcome-start-omitted 下开始行拖到第一次追加（`scripts/gate/hook_swap_swap.py:1064`），那是装置证红用的，不是现行写法。

### 任务服务·退 10 草稿目录被占的出路、异常表那一行、README 三语第 174 行（正文退 10 出路那一条）：成立（读）

- 服务提醒：「这一种等占着的那一趟结束再交；两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是），可以换一个 --scratch」（`task-server/task_server_scheduler_lifecycle.py:1737`），方法文档串第 1693 行起同句。
- 异常表那一行：「退 10 的等占着的那一趟结束再交；占着的是哪一种、能不能换草稿目录（只有草稿目录被占、两趟里又有一趟是干跑的——这一条自己是，或占着的那一趟是——才能换：干跑不占这个仓，真换先占这个仓）」（`task-server/task_server_lib_constants.py:129`）。这一行没写 `--scratch` 字样（工具员报告写明是自证 `swap_busy_problems` 要求这一行不点名 `--scratch`），条件写在「能不能换草稿目录」后面，与正文「同改」对得上。
- README 中文：「退 10 而是草稿目录被占的，两趟里有一趟是 `--dry-run` 的（这一条自己是，或占着的那一趟是）可以换一个 `--scratch`（干跑不占这个仓，真换先占这个仓：两趟真换换了草稿目录照样被仓占着；仓被占的换草稿目录没有用）」（`task-server/README.md:174`）。
- README 英文：「when one of the two swaps is a `--dry-run` (this command itself, or the swap holding the directory)」（`task-server/README.en.md:174`）；日文：「二つの入れ替えのどちらかが `--dry-run`（このコマンド自身か、占有している入れ替え）なら別の `--scratch` を指定してもよい」（`task-server/README.ja.md:174`）。三语条件一致、都带「干跑不占仓、两趟真换换草稿目录无用」的括号。
- 三处说的是不是一回事：提醒（真任务里照日志认成因后才给这一句）、异常表、README 三语四处同一个条件「两趟里至少一趟是干跑」，成立。四种组合各成不成立是攻方腿第 3 问，我只核到文字一致，没有判组合；组合的依据在 `scripts/gate/hook-swap.py` 文件头「按仓的占用」一节：「--dry-run、--read-outcome、--feed-only、--check-callers 不占」那一行（`scripts/gate/hook-swap.py:111`），读它能推出「干跑不占仓」那一半。
- 什么现象会推翻：三语里任一份第 174 行的条件与服务那一句不同；我读了四处全文，没找到不同。

### 任务服务·新格与五条新开关、`KNOWN_BREAK_ITEMS`（正文新格与新开关那一条）：成立（读、量）

- 两格在、已登记：「def cell_wrapped_patch_directory_unresolvable(world):」（`task-server/task_server_selftest_memory.py:1375`）、「def cell_hook_swap_outcome_judged(world):」（`task-server/task_server_selftest_memory.py:1406`）；登记：「"wrapped-patch-directory-unresolvable": cell_wrapped_patch_directory_unresolvable,」（`task-server/task_server_selftest.py:550`）、「"hook-swap-outcome-judged": cell_hook_swap_outcome_judged,」（`task-server/task_server_selftest.py:551`）。
- 五条开关名在已知名单里：「"brace-group-not-layered", "heredoc-body-as-commands", "string-env-chdir-unread", "swap-outcome-unread", "swap-outcome-owner-ignored")」（`task-server/task_server_lib_constants.py:700`），名单从 `task-server/task_server_lib_constants.py:582` 的「KNOWN_BREAK_ITEMS = (」起；五条在 `task_server_lib.py` 文件头开关表里各有一行（`grep -nF` 量过，行首「TASK_SERVER_BREAK=<名>」各命中 1 次）。
- 「新开关五条」这个数：按开关表行首「TASK_SERVER_BREAK=」数，今天文件里 553 条（`grep -c '^  TASK_SERVER_BREAK=' task-server/task_server_lib.py` 量的 553），与 `.claude/gate.d/break-switch-replay-files.tsv` 第 77 行「现数五百五十三条开关」一致；登记「自证四百五十五格」对 `task_server_selftest.py` 的 `CELLS` 字面表（`ast` 数键，量的 455）一致。
- 两格是不是红在名字说的那件事上：读格里的样本——花括号组接管道、heredoc 正文里的 cd、串里的 env -C 三串各自被杀，期望依据写「算不出」（`task-server/task_server_selftest_memory.py:1389-1395`）；换钩子那格期望的点名与不点名各写成字面片段（`task-server/task_server_selftest_memory.py:1422-1431`）。在改前的服务码上红与否，我从第 6 问里对改前的读码推得「三串都红」，没跑；开关点名格逐格相同的复跑结局是主 agent 经服务交的（正文只给了「复跑了 10 条开关（1 份）…逐格相同 10 条」这一个读数，我没读到那条任务的日志）。
- 什么现象会推翻：开关表里少一条或名单里没有；上面两条 `grep`、`ast` 数不对得上就是推翻，实际对得上。

### 读数：18 格、455 格、10 条开关、服务重启、真任务上的一次提醒（正文读数那一条）：455 与 553 成立（量）；其余推不出

- 「整份自证 455 格都对」里的 455：量到字面表 455（上一条），格数成立；「都对」是主 agent 经服务交的那一趟的结局，我没有那条任务的日志（子 agent 只读主 agent 给了号的 `task log`），记「推不出」，它的出处是 `records/2026-11-17-subagent账本清欠调度记录.md` 第 611 行（「重启任务服务：stop 回读 inactive，start 之后任务服务在跑、NFS 共享过」（`records/2026-11-17-subagent账本清欠调度记录.md:611`）与同一行的「整份自证」「复跑了 10 条开关」读数；这一行的位置对：第 610、611 行是第二轮修补两条）。
- 「换上验证 18 格都对」：同一行 611 有，18 格的名单对 `result.md` 的点格自证命令（逐个数过是 18 个格名）成立；结局推不出。
- 真任务上一次退 10 的提醒文字：服务里两句都在——「换钩子的命令退 10：这个仓的换上正被另一趟 hook-swap.py 占着」（`task-server/task_server_scheduler_lifecycle.py:1743`，这是 `why` 里的一句）与「这一种只有等那一趟结束再交，换草稿目录没有用」（`task-server/task_server_scheduler_lifecycle.py:1745`，`next` 里的一句），两句文字与正文引的对得上；记录 611 里引的那一句少了「hook-swap.py」几个字（「服务推的提醒写」之后的引号里，写成「这个仓的换上正被另一趟占着…换草稿目录没有用」），正文引得比记录准、与代码一致。实际推给主 agent 的那一条消息，我读不到，推不出。
- 「任务服务已重启」：记录 611 写了，我没有读服务进程的起始时刻，推不出。
- 什么现象会推翻：`task-server/task status` 之外主 agent 手里的日志与上面不符；我这边没有能推翻它的观测。

### 工具员写明没做的三条（正文读数那一条之下）：成立（读）

- 两条换钩子用同一个 `--scratch` 时两条都点名：两条的 `owner_marks` 相同，输出点名了它们共有的反向规格，「not any(mark in text for mark in one.owner_marks)」那一半对两条都为假，两条都不跳过（`task-server/task_server_scheduler_lifecycle.py:1807`），偏严。
- 算术里的 `<<` 当 heredoc、之后的行都算不出：「算术里的 <<（$((1 << 2))）同样当 heredoc 认，之后的行当正文（偏向算不出，shell_text_runs）。」（`task-server/task_server_scheduler_building.py:91`）。
- 脚本正文的路径不跟 cd：「script = os.path.realpath(os.path.join(cwd, words[index]))」（`task-server/task_server_scheduler_building.py:1516`），`cwd` 是任务的工作目录：「cwd: str             # 任务的工作目录：认它是不是写工具、把词折成相对仓根的路径都照它（modifies_workspace 的判法不跟 cd）」（`task-server/task_server_scheduler_building.py:72`）。
- 什么现象会推翻：三处里有任一处在代码里已经另有处理；读了，没有。

### 引文核对（正文「scripts/agents/cite-check.py 带反引号的位置…」）：成立（读、量）；换上验证与开关复跑的结局推不出

- 带反引号、标题带「，」「。」而文件里没有这样整行标题的判红：带反引号时 `skipped_for_comma` 为假：「skipped_for_comma = not ticked or BROKEN == "ticked-title-comma-skipped"」（`scripts/agents/cite-check.py:340`），单列没判那一支要它为真（`scripts/agents/cite-check.py:341`），所以落到「if len(heads) != 1 and BROKEN != "title-any-section":」（`scripts/agents/cite-check.py:343`）判红。不带反引号的才列进没判。量的：我写了两份一行的探针（草稿目录 `forged-ticked-comma.md` 与 `forged-plain-comma.md`，伪造原文加「根本没有这一节，随便写的」这个带逗号的假标题，一份带反引号、一份不带），直接跑此刻主工作区那份 `python3 scripts/agents/cite-check.py`：带反引号那份退 1、末行「核了 1 处引文、0 处裸行号，1 处对不上，0 处没判」，不带反引号那份退 0、列进没判、理由「小节标题认不出结尾（取到「根本没有这一节，随便写的」，里面带「，」或「。」…）」。带反引号那份的末行，前半「核了 1 处引文、0 处裸行号，1 处对不上，0 处没判」与记录第 610 行主 agent 复测读到的逐字相同（我这边末行后面多一段「其中裸行号没有引文可比 0 处」的括号）。
- 逐层去掉末尾括号：「if not heads and not ticked and BROKEN != "paren-note-not-peeled":」（`scripts/agents/cite-check.py:331`）起的循环：整串命中 0 次、不带反引号才进；`without_trailing_group`（`scripts/agents/cite-check.py:312`）每次去一层末尾成对括号；去掉后命中恰好一处就取它，几处同名的不取（`if len(found) == 1 or BROKEN == "paren-note-peeled-any":` 之后 `break`，`scripts/agents/cite-check.py:336-338`）。
- 兜底只收小节写法，理由三种：兜底两条正则的路径之后都是 `@` 加标题，不收 `:行号`（`scripts/agents/cite-check.py:129` 与 `scripts/agents/cite-check.py:130` 两行）；理由三种：「"title-end-unpaired": "括号没配上",」（`scripts/agents/cite-check.py:136`）、「"title-end-too-long": "标题超出八十个字",」（`scripts/agents/cite-check.py:137`）与第 138 行的 `title-end-other`（标题的结尾认不出，括号里写反引号没配上或标题里带引号），`unknown_end`（`scripts/agents/cite-check.py:150`）按取到的一串分。
- 新开关五条、总数 17：文件头 `CITE_CHECK_BREAK=` 行首计数 17（`grep -cE '^  CITE_CHECK_BREAK=' scripts/agents/cite-check.py` 量的 17）；与改前草稿 `cite-check-r2-fix/base/` 比，多出来的恰是 `loose-takes-prose`、`paren-note-not-peeled`、`paren-note-peeled-any`、`ticked-title-comma-skipped`、`unknown-end-reason-generic` 五条（两份文件头名单 `diff`，量的）。
- 开关复跑「第一趟 `title-parens-unpaired` 多点名一格、头注删了那一处」：此刻 `title-parens-unpaired` 那一段（`scripts/agents/cite-check.py:68-75`）不再列「标题带括号的四种写法交同一个完整标题」，那一格只留在 `title-lazy` 名下（`scripts/agents/cite-check.py:64`），与记录第 610 行的描述一致；复跑结局推不出（没有日志）。
- 工具员写明没做的「不带反引号、位置紧挨引号的散文仍走严格那一支」：`cite-check-r2-fix/result.md` 末段写了；码上：严格那一支的标题只认到「引号之前的可选空白与冒号」或括号、行尾（`scripts/agents/cite-check.py:120`），散文「`路径@一节` 里说「…」」若不带反引号，标题取成「一节 里说」判红或没判。成立。
- 什么现象会推翻：此刻主工作区 `cite-check.py` 对第一份探针不退 1；我量到的是退 1。

### 检出钩子第 224 行（正文检出钩子那一条）：成立（量）

- 今天那一行：「同名的这几种在时也照按候选判）、每个元素照前面几种都算得出的，」（`.claude/hooks/bash-command-detector.sh:224`）。对草稿 `r2-small-wording-fixes/base/` 逐行 `diff`：只第 224 行一处不同，改前是「在也照按候选判」，改后「在时也」；mutate.sh 同法也只有 178 行起一处。量的。
- 什么现象会推翻：`diff` 里还有别的行；没有。

### 变异脚本文件头「并行」那一段（正文变异脚本那一条）：成立（量、读）

- 今天的两行：「这一路一条任务里各工作进程的测试线程合计不超过线程上限；用例显式要的原样传给每个工作进程，」（`scripts/mutation/mutate.sh:178`）与「合计是显式值 × 工作进程数，可以超过线程上限（显式给 8、4 个工作进程合计 32）。」（`scripts/mutation/mutate.sh:179`）。正文引的那句是这两行的前半，少了第 179 行括号里的举例；不影响句意。
- 这句话对码成立（读）：显式值走「(($1 < THREAD_CAP_FOR_SPLIT))」（`scripts/mutation/mutate.sh:2134`）判成显式，设了就原样传：「每条 cargo test 照调用方设的 RUST_TEST_THREADS=$RUST_TEST_THREADS 跑（不覆盖）」（`scripts/mutation/mutate.sh:2200`），没往 `TEST_THREADS_ENVIRONMENT` 里加分过的 `RUST_TEST_THREADS`（`SINGLEFS_THREAD_CAP` 那一路另照工作进程数分，`scripts/mutation/mutate.sh:2216-2219`，不影响这一句），所以每个工作进程各拿显式值，合计 = 显式值 × 工作进程数；显式 8、线程上限 31 时 8 小于 31 判成显式，4 个工作进程合计 32，超过 31。这是第二轮判决里「核不动、照推的办」的第 4 问显式值那一句，此刻读码核过一遍（工作进程数取到 4 要表里有足够多的行，我没核工作进程数的取法，它是另一条已判过的路径，这里只核显式值原样传这一半）。
- 什么现象会推翻：显式小于线程上限的值在 `settle_test_threads_and_cargo_jobs`（`scripts/mutation/mutate.sh:2194` 起）里被除以工作进程数；读到的是不除。

### 末句：算键库别的会话把超折叠上限改成拒算（正文最后一条）：成立（量）

- 新拒算种类在：「FOLD_LIMIT_FORM = "fold-limit-exceeded"」（`scripts/gate/lib_function_units_base.py:57`）。「与这一轮被判的文件无关」：14 份被判文件里没有 `scripts/gate/` 下的文件（`judged-sha256.txt` 的 14 行，量的）。
- 旁见（不是这一条的错）：算键库登记 `scripts/gate/lib_function_units_registry.py:257` 起那一行说明还只写「起任务工作目录那棵树里的 apply-writer-patch.py --read-outcome」与假脚本 `FAKE_APPLY_WRITER_PATCH`，今天同一处 `subprocess.run(read_back, …)` 也起 hook-swap.py 的 `--read-outcome`（工具员报告「交主 agent 的」一节写了这句说明要补）；键没变、照认，只是那行说明不全。

### 被判范围 14 份（正文「范围」）：成立（量）

- `sha256sum -c research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-snapshot/judged-sha256.txt`：14 行，都是「OK」，没有对不上的。

## 第 6 问：第二轮判决里「推的」「核不动」的逐条现查

第二轮判决「核不动、照『推的』办的」三条（`research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-main-verification.md` 第 19 行）：攻方第 2 问三种算错当前目录的写法与多报；攻方第 4 问显式值合计超上限；辩方腿七条补充探针没跑。第二条在第 5 问里「变异脚本」那一小节读码核过了（成立，显式 8、4 个工作进程合计 32）；下面写第一条的两半与第三条。

### 6.1 三形算错当前目录，在改前的码上是不是真算错：真算错（读），改前三形都算出一个错目录、读错结局文件、漏报

改前取 `~/.local/share/singlefs/work/task-server-r2-fix/base/task-server/`。下面三串都是 `bash -c`、任务工作目录 `<仓根>`（假设 `<仓根>/nested` 在，`<仓根>/patch-half` 没打完），被杀（退 137）。改前切词的两个起点：词两头剥括号与花括号：「WORD_EDGE_CHARACTERS = "[](){}」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:34`），「word = token.strip(WORD_EDGE_CHARACTERS)」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:111`）；按行切、不认 heredoc：「for line in text.replace("\\\n", " ").splitlines():」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:84`）。

- 花括号组接管道（`{ cd nested; true; } | cat` 换行 `python3 scripts/gate/apply-writer-patch.py patch-half`）：
  - 切出：`{`、`}` 剥成空词丢掉，得到四条：`cd nested`（分隔只有行首）、`true`（分隔 `;`）、`cat`（分隔 `;`、`|`）、打补丁（分隔行首）。
  - 平铺判：「straight.append(depth == 0 and all(token in SHELL_STRAIGHT_BEFORE for token in joint)」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:178`）对 `cd nested` 为真（深度 0、分隔只有行首、后面接的是 `;`），`cat` 为假（前面是 `|`）。整段「directory_known = not opaque and all(」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:189`）为真。
  - 算出：`cd nested` 之后「current = moved if moved is not None and os.path.isdir(moved) else None」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:204`）是 `<仓根>/nested`；打补丁那条记下的目录「runs.append(ShellRun(list(words), expanded, current))」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:197`）就是 `<仓根>/nested`。
  - bash 真跑：接在 `|` 左边的整组跑在子 shell 里，`cd` 留不到组外，打补丁在 `<仓根>`，补丁目录是 `<仓根>/patch-half`。
- heredoc 正文里的 `cd`（`cat > notes.txt <<'EOF'`、`cd nested`、`EOF`、打补丁 `patch-half` 四行）：
  - 切出：第一行的 `<<` 与它的目标 `EOF` 当重定向去掉，只剩 `cat`；`cd nested` 与 `EOF` 各是一条（分隔都只有行首）；打补丁一条。没有任何记号说明前两条是正文。
  - 平铺判对 `cd nested` 为真，算出的当前目录与上一串同，打补丁那条是 `<仓根>/nested`。
  - bash 真跑：`cd nested` 是喂给 `cat` 的文字，打补丁在 `<仓根>`。
- 串里的 `env -C nested python3 scripts/gate/apply-writer-patch.py patch-nested-half`：
  - 改前只看串外那几个词有没有 `-C`：「start = given_cwd if literal or not shell_directory_option(command[:index]) else None」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:1441`），`command[:index]` 是 `bash`、`-c`，没有 `-C`，起点是任务工作目录；串里没有 `cd`，「current = os.path.normpath(directory) if directory is not None and directory_known else None」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:193`）就是 `<仓根>`，打补丁那条的目录是 `<仓根>`。
  - bash 真跑：`env -C nested` 把那个进程的目录换成 `<仓根>/nested`，补丁目录是 `<仓根>/nested/patch-nested-half`。
- 三形落到读结局文件时（改前 `writer_interruption`）：「outcome_file = os.path.join(directory or "", patch, PATCH_WRITER_OUTCOME_NAME)」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_lifecycle.py:1830`）读的是 `<仓根>/nested/patch-half/…`、`<仓根>/nested/patch-half/…`、`<仓根>/patch-nested-half/…`，都不是真补丁目录；不在就「return None   # 结局文件在动手换之前写：不在就是还没开始换，主工作区没动」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_lifecycle.py:1832`）：被杀不报，真没打完的补丁漏报。在错目录下恰好有一份同名补丁目录而它打全了时读到的是「退 0」，同样不报。
- 前提与限度：前两形要 `<仓根>/nested` 在判的那一刻存在（改前 `os.path.isdir(moved)`，不在就当前目录算成 None、落到「算不出」照报）；这是服务在任务结局之后读文件系统，不是 bash 当时的目录。三形也要 `modifies_workspace` 为真（包着的写工具改前就认，未变）。
- 旁证（工具员自己跑过的，不是我跑的）：草稿探针在内存里载入 `shell_text_*` 几段源码，三条新开关各让一形退回改前的算法，输出里打补丁那一条的目录依次是 `nested`（「['patch-half'] ['patch-half'] nested」，`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/probe/out-brace-group-not-layered.txt:5`）、`nested`（同一行，`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/probe/out-heredoc-body-as-commands.txt:10`）、仓根（「['patch-nested-half'] ['patch-nested-half'] .」，`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/probe/out-string-env-chdir-unread.txt:11`）；进度记录写了直接对改前文件跑的读数：「base 与 new 跑过（base 三形算出 nested、nested、仓根，new 三形都是 None」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/progress.md:4`）。我的读码与它一致；直接对改前文件起探针的命令写在末尾「要主 agent 跑的命令」第一条。
- 判定：改前三形都真算错——花括号组接管道 → `nested`，heredoc 正文里的 `cd` → `nested`，串里的 `env -C` → 仓根；后果都是读错结局文件、被杀的半截补丁漏报。第二轮判决「采、标推的」这一条，读码核到「成立」，直接读数待主 agent 跑。
- 什么现象会推翻：对改前文件起探针，打补丁那条的目录不是 `nested`、`nested`、`.`；或 `<仓根>/nested` 在时 `writer_interruption` 对前两串仍交出「算不出」那一项。

### 6.2 正常结束的换钩子被点名，在改前是不是真会发生：真会发生（读）

- 判到这一步的条件：命令被信号杀：「interrupted = outcome in WORKSPACE_WRITER_INTERRUPTED_OUTCOMES or (outcome == "command-failed" and exit_code in SIGNAL_EXIT_CODES)」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_lifecycle.py:1754`）、且「if interrupted and self.modifies_workspace(record):」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_lifecycle.py:1758`）。
- 认到几条：包着的取每一条写工具：「return [writer for writer, writes in self.wrapped_workspace_commands(task) if writes]」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py:1408`）；逐条判：「for writer in found or [self.command_workspace_writer(record)]:」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_lifecycle.py:1775`）。
- 换钩子那一条不读结局：「if os.path.relpath(os.path.normpath(os.path.join(os.path.realpath(cwd), tool)), root) != WRITER_PATCH_ENTRY:」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_lifecycle.py:1815`）之后直接「return InterruptedWriter(tool, "", f"要核的是{('这几处：' + '、'.join(shown)) if shown else '它改的文件'}", True)」（`/home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_lifecycle.py:1816`），`shown` 是它后面的规格与草稿目录路径。
- 走一遍：`bash -c 'python3 scripts/gate/hook-swap.py --spec a.json --scratch s1 && python3 scripts/gate/hook-swap.py --spec b.json --scratch s2'` 在第二条换钩子里被杀（退 137）。两条都认成写工具（写判真）；两条都走「不是打补丁」那一支、不读结局文件、不分换完没换完；两条都进 `judged`，出一项 workspace-write-interrupted，出路里两条都点名「这几处：a.json、s1」「这几处：b.json、s2」。第一条已经正常换完（退 0，结局文件里是完成），仍被点名。
- 反过来、在串里前面那条换钩子里被杀，后面那条根本没起，也照样点名后面那条的路径。单条命令（命令那一层的换钩子）被杀是真半截，点名是对的；这里说的只是包着的、已经正常结束的或没起的那几条。
- 工具员的自证格 `hook-swap-outcome-judged` 就是照这一形写的（`task-server/task_server_selftest_memory.py:1402-1403` 的 `TWO_SWAPS_CHAIN`），工具员报告「期望」一栏写了两条都点名（改前红，读码推的，没跑）。
- 判定：成立。第二轮判决「采、标推的」这一条读码核到「改前真会发生」。
- 什么现象会推翻：在改前的服务码上起这样一串被杀的任务，出的 workspace-write-interrupted 的出路里没有已经换完的那条换钩子的 a.json、s1。

### 6.3 辩方腿七条补充探针（第二轮辩方报告「要主 agent 跑的命令」一节的 1 到 7）：能只读核的都核了

七条按它们是什么命名，不用序号。第二轮辩方报告在 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-output.md`，命令在它末尾。检出钩子有三个状态：改前（动改动 A 之前，草稿 `probe-base/hooks`，`bash_command_detector_literals_and_signals.py` 的 sha256 前缀 `5c233be0`）、第一轮修补之后（`probe-new/hooks`，前缀 `e297a24c`）、此刻主工作区；后两者的这一份文件逐行 `diff` 只有两处文档串不同（量的，两个 hunk 都是 `"""` 里的字），判法码相同。

- 钩子探针·改前钩子目录（六种写法）：读核掉。
  - 读：改前 `single_assignments` 只收 `名字 = 值` 的赋值（「if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):」（`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base/hooks/bash_command_detector_literals_and_signals.py:335`）），`for` 的循环变量不在里面，所以循环里的 `open(f, 'w')` 路径算不出；整棵语法树逐个调用判：「for node in ast.walk(tree):」（`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base/hooks/bash_command_detector_literals_and_signals.py:424`）；`ast.If` 在这份文件里 0 处（`grep -c 'ast\.If'` 量的 0，此刻主工作区那份同为 0）。
  - 读数：`single-name-under-false-guard` 1、`single-name-exists-guard-existing-file` 1（单名字、`if` 不读，目标是已有 `running.sh`）；`loop-exists-guard-existing-and-new` 0、`loop-rebound-by-except-as` 0、`loop-rebound-by-def` 0、`loop-all-new-files-no-guard` 0（循环变量改前算不出）。与辩方腿写的推的读数逐项相同。
  - 同类的已量到的读数：`loop-exists-guard-existing-and-new` 的代码与第一轮探针 `exists-guard-writes-only-new-files` 逐字相同，主 agent 第一轮量到改前 0：「case exists-guard-writes-only-new-files 0」（`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base.out:12`）；单名字写在大循环里改前量到 1：「case fixed-target-inside-big-loops 1」（`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-base.out:22`）。
  - 结论：辩方腿说的「条件不看」在改前的单名字一路上早就有（先例成立），改法 1 是把 python 循环拉到同一读法。
  - 什么现象会推翻：真起这份探针，头两行读 0。
- 钩子探针·第一轮状态钩子目录（同六种写法）：读核掉。
  - 读：此刻（与第一轮状态同码）`name_binding_counts` 只数 Name 的 Store 与 Del、`ast.arg`、import：「if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:329`）；`except … as` 与 `def` 绑的名字在语法树里是字符串，不进这三支，循环变量的计数仍是 1，`literal_loop_candidates` 照认：「and bindings.get(node.target.id) == 1 and node.iter.elts):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:362`），两个元素逐个代入，已有的 `running.sh` 落到。
  - 读数：前五行都是 1（`loop-rebound-by-except-as` 与 `loop-rebound-by-def` 是 1，就是第一轮正推腿说的「绑定计数说宽」的实证）、`loop-all-new-files-no-guard` 是 0（两个文件都不存在）。`loop-exists-guard-existing-and-new` 已量到 1：「case exists-guard-writes-only-new-files 1」（`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r1-main-rerun/probe-new.out:12`）。六种从改前到第一轮状态只有 0 变 1，没有 1 变 0，说宽的方向是多拒、不是漏放，判决与辩方腿都成立。
  - 什么现象会推翻：真起探针，`loop-rebound-by-except-as` 或 `loop-rebound-by-def` 读 0。
- 三个状态的标题取法（`title_states.py` 那份）：读核掉，并有辩方腿自己量到的对照。
  - 读：改前与第一轮状态各自的 `LOCATION`、`TITLE_END` 正则行在草稿 `cc-base-run`、`cc-r1-run` 里（`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/cc-r1-run/scripts/agents/cite-check.py:68`、`:69`），与辩方腿抄进 `frag-base.py`、`frag-r1.py` 的行一致。改前标题字符类懒匹配、两个字就能收尾（反引号可选）；第一轮状态要求结尾是反引号、右括号、行尾或紧跟引号，括号不进字符类。逐串走：带括号的真标题，改前取「放宽」、第一轮状态取不出；不带括号的长标题，改前取「不为」、第一轮状态取全；主 agent 第一轮复测用的「不为省空间牺牲自包含（附）」，改前取「不为」、第一轮状态取不出；位置在前、标题带括号的，两个状态都取不出。与辩方腿量到的表逐行相同。
  - 此刻整份的读数我直接跑了（下一条）。
  - 什么现象会推翻：用同样的正则行对 `probe1.md` 起 `title_states.py`，表里有一行与上面不同。
- 改前整份核对（`cc-base-run`）：读核掉。改前 `judge_section` 没有任何括号、逗号的宽容，标题命中不是恰好一行就判红：「if len(heads) != 1 and BROKEN != "title-any-section":」（`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-sonnet-b/cc-base-run/scripts/agents/cite-check.py:203`）。仓里 `.claude/rules/machine-first.md` 的标题是「放宽的（人脑带宽的产物）」（第 76 行）、`.claude/rules/fs-design.md` 的是「不为省空间牺牲自包含」（第 33 行），都没有标题恰好是「放宽」或「不为」的行（`grep -nE '^#+ *放宽'`、`'^#+ *不为'` 量的，各只命中那一行完整标题）。所以改前认出的六处（探针串里的六条带引文的位置，位置在前那一条改前认不出）全部判红，其中两处真原文（`probe1.md` 里引「设计要统一、概念要少」与「默认答案是不省」的那两行）也红：「核了 6 处引文、6 处对不上」。辩方腿说的「改前对真原文也红、改前从没能验过这一类标题」读码成立。什么现象会推翻：起改前整份核对，真原文那两行判绿。
- 第一轮状态整份核对（`cc-r1-run`）：读核掉。这份文件没有兜底、也没有认不出结尾的单列（`grep -c 'LOOSE\|认不出结尾'` 量的 0）；认出的只有两串（不带括号的长标题），其中真原文那串判绿、伪造那串判红，别的五串认不出、既不判也不列：「核了 2 处引文、1 处对不上」。「引文在前的括号标题，伪造的原文照过」成立。什么现象会推翻：起第一轮状态整份核对，括号标题那三行有判红的。
- 此刻整份核对：量的（我直接跑了此刻主工作区的 `python3 scripts/agents/cite-check.py`）。`probe1.md`：退 1，「核了 7 处引文、0 处裸行号，5 处对不上，0 处没判」，红的是第 4、5、7、8、9 行，与辩方腿第二轮量到的逐行相同（真原文那两行没红）。`probe2.md`：退 1，「核了 3 处引文、0 处裸行号，3 处对不上，0 处没判」，三行都是「命中 0 次」；辩方腿第二轮量到的是「1 处对不上、2 处没判」，差别正是判决改法 4 的新行为（带反引号的带逗号标题、真标题加说明、括号没配上，今天都判红）。输出落在 `~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet/probe1-now.txt` 与 `probe2-now.txt`。
  - 顺带记一笔（攻方腿第 4 问要看的，我只记事实）：`probe2.md` 第 4 行是真标题「放宽的（人脑带宽的产物）」后面接「，补充说明」、带反引号、原文是真的，今天判红（标题「放宽的（人脑带宽的产物），补充说明」命中 0 次）；这是改法 4 的设计取舍（反引号把标题头尾定死了），改前是没判。
- 整批报告里「小节标题认不出结尾」的没判行：量的，我对仓里 294 份腿报告（排除禁读的这一轮另一条腿的报告与本报告，名单落在草稿 `reports.txt`）直接跑了此刻的 `cite-check.py`，用时 22.7 秒（`time` 量的，没量内存）：末行「核了 15031 处引文、10649 处裸行号，11293 处对不上，8248 处没判」；输出里「小节标题认不出结尾」0 处、「括号没配上」「超出八十」「结尾认不出」都是 0 处（`grep -c` 量的）。第二轮攻方腿在第一轮修补状态上量到同一类 67 处，对它那份输出（`~/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-b/cite/reports-new.out`）按「报告:行」对键：这 67 行里 46 行今天的输出里没有任何一行、21 行今天以别的理由出现（行号超出文件、原文不在那一行等，与小节标题无关）。
  - 46 行里有多少是「改判绿了」、多少是「兜底不收散文而整条不判也不列」，只看输出分不出：判绿的不打印。这一点写进末尾「改前改后逐处对照」那条命令（改前是第二轮修补之前的那份），由主 agent 跑。
  - 辩方腿写的两个推翻条件，今天：「有伪造的原文」（那样判红更好）——这批输出里没有「认不出结尾」行，无从判；「一条都没有真正认不出」（那样这一支是死代码）——在这 294 份报告上今天确是 0 次触发，但这一支有 20 格自证与 5 条开关各自红在名字说的那件事上，不是死代码，只是这批报告里没有真正的「标题结尾认不出」的写法；第二轮攻方腿 3c 说的 67 处是散文，与这个读数相合（推的）。

### 6.4 第二轮判决「核不动」里的另一条：攻方第 4 问显式值合计超上限

已在第 5 问「变异脚本文件头」一节读码核过：显式值走 `explicitly_requested_threads`、原样传、合计 = 显式值 × 工作进程数。成立；文件头现在写对了。工具员当时没把它做成样本格的那一项（攻方腿 3d、4d 的少给）不在这一轮的改动里，没碰。

## 要主 agent 跑的命令

草稿目录 `W` = `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet`。两条都只读（载入或读文本、不写仓、不起服务、不碰判定库），只往 `W` 里写。我没量它们的内存与时间：第一条只载入两个源文件里几段函数、读几行文本，推的是秒级、几十兆；第二条是我直接跑同一批 294 份报告、一份 `cite-check.py` 的 22.7 秒（量的），这里要载入两份脚本各判一遍，推的是一分钟上下、不到一千兆。

1. 改前三形直接读数（对着草稿 `base/` 的 `task_server_scheduler_building.py` 起工具员写的探针，只取 `shell_text_*` 那几段源码在内存里跑）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet -- python3 -I /home/fy5090/.local/share/singlefs/work/task-server-r2-fix/probe/runs_probe.py /home/fy5090/.local/share/singlefs/work/task-server-r2-fix/base/task-server/task_server_scheduler_building.py /home/fy5090/.local/share/singlefs/work/task-server-r2-fix/probe/wrapper_words_source.txt /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet/probe-root`
   - 在哪起：`W`。
   - 看：`TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；日志里每个串一段，花括号组接管道、heredoc 正文里的 cd、串里的 env -C 这三段各自最后一行（打补丁那一条）的目录列：我读码推的是 `nested`、`nested`、`.`（仓根）。对不上就是 6.1 那一条要改判。后四段是对照串（`cd nested && …` 应当 `nested`、赋值再代入应当 `.`），不判。
2. 引文核对第二轮修补的改前改后逐处对照（回答「兜底不收散文之后，此前列进没判的真小节写法有没有掉出去不判也不列」）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet -- bash /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet/tools/run-cite-compare-r2fix.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/cite-check-r2-fix/base/scripts/agents/cite-check.py /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-sonnet`
   - 在哪起：`W`。`tools/cite_compare.py` 是第二轮攻方腿的对照脚本原样拷来（sha256 `b69f98d02714ad185c74e2e69c58af667fa69aa2ca8597c28885bf7f0e4dfe7b`，与它报告里登记的相同），`tools/run-cite-compare-r2fix.sh` 是我写的包装（sha256 `ed519352d4618c955650a25882a2f4e31addd9e8f33ea25fef720b8cb89ee88f`），`reports.txt` 是 294 份报告的相对路径（sha256 `bc7a66adcef34c3df80aa40792a27d0c4598b815d6af67c3734dbbdab34cb62f`）。改前那一份是草稿 `cite-check-r2-fix/base/` 的，与第二轮修补之前主工作区那份逐字节相同（`cmp` 量的）；要先在那个草稿目录还在时跑。
   - 看：`exit=0`；日志末行「done 报告 294 份 改前认出 N 处 改后认出 M 处 结局不同 K 处」；`W/reports-r2fix.compare.out` 里写「改前」里有「小节标题认不出结尾」的行：「改后 没认出」的就是掉出去既不判也不列的，「改后」是 `ok` 或 `bad` 的是被判了；我读输出推的是这类行里没有真小节写法掉出去（46 行缺席里应当是散文），这一条要它读出来才算。
3. 可选，我读码已核掉、不是必须：辩方腿第二轮报告「要主 agent 跑的命令」一节里钩子探针两个状态、三状态标题取法、改前与第一轮状态整份核对那五条原样交，读数与我上面写的不同才有信息。此刻整份核对与整批报告核对那两条我已直接跑了。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 5 任务服务·切词与算当前目录 | 成立（读） | 花括号组记进分隔并当一层、heredoc 正文标上 `<<`、串里 `env -C` 那条目录 None，三形今天都落到「算不出」；旁见：不接管道的 `{ …; }` 也被降成算不出（多报，偏严） |
| 5 任务服务·`hook_swap_read_back` 与 `workspace_write_judged` | 成立（读） | 起法、退 0 与 1 不报、归另一条的不报、`subprocess.run(read_back, …)` 留在原方法，都在 |
| 5 被杀在写开始行之前不报了 | 成立（读） | 开始行与文件行是一次原子写，且排在第一份文件换上之前；这一支该不报 |
| 5 退 10 出路、异常表、README 三语 | 成立（读） | 四处同一个条件「两趟里至少一趟是干跑」，文字一致；四种组合各成不成立归攻方腿 |
| 5 新格与五条新开关、`KNOWN_BREAK_ITEMS` | 成立（读、量） | 两格与五条名字都在、已登记；红在名字说的那件事上的复跑结局推不出 |
| 5 读数：455 格、553 条、18 格名单 | 成立（量） | `CELLS` 455、开关表行首 553、点格命令 18 个名字都在 `CELLS` 里 |
| 5 读数：各条自证与开关复跑的结局、服务重启、真任务上那一次提醒 | 推不出 | 没有那几条任务的日志与服务进程信息；提醒的两句文字在码里都有，记录 611 引得缺「hook-swap.py」几个字 |
| 5 工具员没做的三条 | 成立（读） | 同 scratch 两条都点名、算术 `<<`、脚本正文路径不跟 cd，码里都是这样，偏严 |
| 5 引文核对 | 成立（读、量） | 带反引号带逗号判红（我跑了两份探针：带反引号退 1、不带退 0）、逐层去括号同名不取、兜底三种理由、开关 17 条里新五条；换上验证与复跑结局推不出 |
| 5 检出钩子第 224 行 | 成立（量） | 对 base 逐行 `diff` 只这一处，「在时」 |
| 5 变异脚本文件头 | 成立（量、读） | 对 base 逐行 `diff` 只这一处；显式 8、4 个工作进程合计 32 对码成立 |
| 5 末句：fold-limit-exceeded | 成立（量） | `lib_function_units_base.py:57` 在，被判 14 份里没有 `scripts/gate/` 下的文件；旁见：算键库登记那行说明只写了 apply-writer-patch.py |
| 5 被判范围 14 份 | 成立（量） | `sha256sum -c` 14 行全 OK |
| 6 三形算错当前目录在改前 | 成立（读） | 改前依次算出 `nested`、`nested`、仓根，读错结局文件、半截补丁漏报；直接读数待主 agent 跑（「改前三形直接读数」那条命令） |
| 6 正常结束的换钩子被点名在改前 | 成立（读） | 改前换钩子那一支不读结局，`&&` 串里已换完的那条也被点名 |
| 6 钩子探针·改前与第一轮状态 | 读核掉 | 改前：单名字两种 1、循环四种 0；第一轮状态：前五种 1、全新文件 0；「条件不看」先例成立、「绑定计数说宽」只多拒 |
| 6 三状态标题取法、改前整份、第一轮状态整份 | 读核掉 | 与辩方腿量到的表逐行相同；改前对真原文也红（6 处认出、6 处红），第一轮状态认出 2 处、红 1 处 |
| 6 此刻整份核对 | 量 | `probe1.md` 7 处引文 5 处红、与辩方腿读数同；`probe2.md` 3 处红，是改法 4 的新行为 |
| 6 整批报告里「认不出结尾」的没判行 | 量（今天 0 处）；掉出去的分布待主 agent 跑 | 294 份报告今天 0 处，第二轮量到的 67 处里 46 行今天无输出，判绿还是不判不列要「改前改后逐处对照」那条命令分 |
| 6 攻方第 4 问显式值合计 | 成立（读） | 同第 5 问变异脚本那一格 |

## 没做什么

- 不替主 agent 采纳或出判决；没往统一任务服务交任务，两条要引的读数（改前三形直接读数、引文核对改前改后逐处对照）都没复跑，命令在上一节「要主 agent 跑的命令」。
- 没起任何项目代码：任务服务的函数、检出钩子的判定函数、`runs_probe.py`、`cite_compare.py` 都没起；对改前文件的结论全是读码推的。我直接跑过的只有 `grep`、`sha256sum`、`cmp`、`diff`、只用标准库读文本的 `python3 -I`，和 `python3 scripts/agents/cite-check.py` 对几份我自己写的探针文件与 294 份腿报告。
- 没写模型、没建 `rerun.sh`：这一轮判的是协作工具，没有状态模型；包装脚本 `tools/run-cite-compare-r2fix.sh` 是给主 agent 跑的一条对照的包装，不是模型。
- 攻方腿的第 1 到 4 问没碰：「平常写法被连带降成算不出」只在旁见里记了不接管道的花括号组一形，没系统找；退 10 四种组合没判；引文核对逐层去括号撞上另一节、同名标题两处、「标题后面跟行号」的误红，没单独构造。
- 没读禁读清单里的文件；没改任何仓内文件，只写了本报告与草稿目录。草稿目录 `W` 现 4.0M（`du -sh`，量的），没有仓副本与编译目录，里面的 `tools/` 与 `reports.txt` 是「改前改后逐处对照」那条命令要用的，跑完判决引完数由主 agent 整目录删。

