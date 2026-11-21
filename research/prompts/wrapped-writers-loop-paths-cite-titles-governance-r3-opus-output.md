# 攻方腿报告：治理文件轮第三轮（最后一轮），攻第二轮判决五条修补里的四处（wrapped-writers-loop-paths-cite-titles-governance-r3，2026-11-21）

被判对象读主工作区此刻那一版（`research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r3-snapshot/judged-sha256.txt` 列的 14 份，开工时 `sha256sum -c` 全部 OK）。改前原样取工具员草稿 `task-server-r2-fix/base/`、`cite-check-r2-fix/base/`。另读了被判文件之外的 `scripts/gate/hook-swap.py`、`scripts/gate/hook_swap_swap.py`、`scripts/kb/lib_atomic_replace.py`、`task-server/task_server_lib_configuration.py`（第 2 问要的写结局文件次序与 `command_word_index`）。

这条腿没写 `<模型目录>` 与 `rerun.sh`：被判的是协作工具，不是崩溃状态模型，没有崩溃枚举要跑。要跑的是两份只调被判文件里函数的探针，命令在「要主 agent 跑的命令」。任务服务的函数这条腿没载（项目代码闸），第 1 问的读数全是读码推的、外加 bash 真跑出的「当前目录在哪」；第 4 问改后那一版是这条腿直接跑主工作区 `scripts/agents/cite-check.py` 量的，改前那一版待主 agent 跑对照。

草稿目录 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus/`（`du -sh` 4.0M），文件与 sha256：

| 文件 | sha256 | 是什么 |
|---|---|---|
| `tools/runs_probe_r3.py` | `4cd2fbc318c95f919244942daae087580ddbebb412a55a748aaf6c6171d2cc05` | 第 1 问 13 段文字喂 `shell_text_runs`（照工具员 `probe/runs_probe.py` 的取法，只取那几段源码在内存里跑） |
| `tools/run-runs-probe.sh` | `1c5a754e7a2c3b756da4d329bf382f059f8e0ab191675e5b2533133470982a5e` | 改后（主工作区）、改前（`task-server-r2-fix/base/`）各跑一遍，写 `probe/runs-new.out`、`probe/runs-base.out` |
| `tools/cite_compare.py` | `b69f98d02714ad185c74e2e69c58af667fa69aa2ca8597c28885bf7f0e4dfe7b` | 第二轮攻方腿那一份原样拷来（sha256 与它报告里写的相同） |
| `tools/run-cite-compare-r3.sh` | `b6d76533b7b5387160dbba08415fd1e86698f107ec92ce9faa5de206170dff67` | 第 4 问改前（`cite-check-r2-fix/base/`）改后逐处比：`cite/forms-r3.md` 一次、仓里腿报告一次 |
| `tools/reports.txt` | `30fc9536dfb59b72c97d3ce6267b78e9b57b9d278cec9a561b0ff823999542b6` | `research/prompts/*-output.md` 295 份（不含禁读那一份与本报告） |
| `cite/forms-r3.md` | `fee4ec999cf3b9ff7fcc95f3c9f63e1dbe104ece398cc7ad05a0480c6b04fc7c` | 第 4 问 12 形（第 3–14 行，编号 Q4-03 到 Q4-14 即行号） |
| `cite/forms-r3.out` | `1a0a4bceb7b949ccad94fa65c50ec61b342f51177be733451ad24cf13d342aff` | 主工作区 cite-check 跑 forms-r3.md 的原样输出（退 1） |
| `cite/reports-new.out` | `02edf8c72735d70120afba07235fe84e4375e8db6732079fd78f674da0805d8d` | 主工作区 cite-check 跑那 295 份的原样输出 |

## 各格判定一览

| 问 | 判定 | 打中的东西（一句） | 证据性质 |
|---|---|---|---|
| 1 任务服务算当前目录 | 成立（打中） | 连带降成算不出（多报）的平常写法五类：heredoc 正文末行是空行或注释行、`{ …; }` 不接管道（组里的 cd 留在这一层）、`}` 之后的下一条、heredoc 正文里单独一行的 `{` / `}`（JSON、python 字典）、接管道的组里 cd 之后组里自己的写工具；仍算出错目录的四形：`env -Cnested` 合写（串里与命令那一层都是）、行首反引号命令替换里的 cd、`<< -EOF`（空格后的 `-` 是定界词的一部分）、判的那一刻目录在而跑的那一刻不在 | 读码推的；bash 真跑的当前目录是量的；探针待主 agent 跑 |
| 2 `hook_swap_read_back` | 成立（轻，偏严） | 认是不是同一趟只靠退 3 时输出里反向规格的路径前缀（`--scratch` 推出），不看进程号与时刻；退 0、1、4 不认归属：更早一趟完成之后文件被改过（退 4），这一条被杀在写开始行之前也照报（多报）。「写开始行之前不报」没有漏掉写了一半的被跟踪文件：开始行连同每份的换前换后是改名换上一次写全的，在第一份文件换上之前；之前写进主工作区的只有预演的同目录临时文件 | 读码推的；结局文件此刻读出退 0 是现读结局文件、现算 sha256 量的 |
| 3 退 10 出路的新句 | 不成立（没打中） | 四种组合逐个对得上：干跑 × 干跑、干跑 × 真换、真换 × 干跑可换，真换 × 真换不可换；提醒、异常表、README 三语说的是一回事，与 hook-swap 自己的拒绝说明不冲突。只有一处字面：「真换先占这个仓」按字面读成先占仓、后占草稿目录，与代码次序相反 | 读码与读文的 |
| 4 引文核对 | 成立（打中） | 掉出去不判也不列：标题里本身带反引号的真小节（kb 与规则 86 份里 17 个标题，规则里 7 个，其中有 `.claude/agent-common.md` 第 21 行点名要照守的那一节）照逐字写进带反引号的小节写法，配真原文、配伪造原文都既不判也不列；位置与引文之间隔「，」的小节写法同样不判不列，而同形的行号写法照列进裸行号。误红：不带反引号、位置后接散文再接引文的判红（工具员写明没做的那一形）。去括号取错节：编造的「标题（说明）」去括号后落在父一节，原文在父一节里就判对上（宽，不是放过伪造原文） | 改后量的（forms-r3.out）；改前推的，对照待主 agent 跑 |

四句（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 那一节）照第二轮攻方腿的读法答：这一轮的对象不是带臂的实验，「分不分辨臂」改答「打中的是不是这一轮的修补带进来的」，各节末尾逐条写。

## 第 1 问：任务服务算当前目录

判定：成立（打中）。全部读主工作区 `task-server/task_server_scheduler_building.py` 推的；「bash 真跑时那一条在哪」一列是这条腿在草稿目录里拿 `pwd` 代替打补丁、用 bash 真跑出来的（量的），行首反引号那一形被项目代码闸拒了没跑，照 bash 语义推。逐形的探针在 `tools/runs_probe_r3.py`（13 段，样本名与下表同），改前改后两份读数待主 agent 跑。

### 1a. 三形改成算不出时连带的平常写法（多报，推的）

改动落在两处：`{`、`}` 记进分隔，「elif braces and token in SHELL_GROUP_BRACES and not current:」（`task-server/task_server_scheduler_building.py:134`）；heredoc 正文的行在分隔里多记一个记号，「line_marks = ([PIECE_HEREDOC_BODY] if body else []) + ([PIECE_UNLEXED] if unlexed else [])」（`task-server/task_server_scheduler_building.py:110`）。分隔只在这一行切出了命令时才清空（「joint = []」（`task-server/task_server_scheduler_building.py:148`）），切不出命令的行（空行、注释行、单独一个 `{` 或 `}` 的行）记下的分隔并进下一条命令的分隔。平铺的判法要分隔里每个记号都是行首或分号，「straight.append(depth == 0 and all(token in SHELL_STRAIGHT_BEFORE for token in joint)」（`task-server/task_server_scheduler_building.py:214`），整段有一处 cd 不平铺就整段算不出（「or (straight[index] and words[0] in SHELL_DIRECTORY_WORDS and sum(word in SHELL_DIRECTORY_WORDS for word in words) == 1)」（`task-server/task_server_scheduler_building.py:227`））。层数照分隔里的括号与花括号数，「depth += sum(token.count("(") + token.count("{") - token.count(")") - token.count("}") for token in joint)」（`task-server/task_server_scheduler_building.py:210`），数成负的整段不透明（「opaque = opaque or depth < 0」（`task-server/task_server_scheduler_building.py:218`）），末尾不平同样不透明。

下表五类八形改前都算得对（改前花括号当词两头的边缘字符剥掉、heredoc 正文照命令切，正文里没有 cd 时之后的 cd 照平铺算），改后都落到算不出；补丁目录是相对的那一条被杀时照「补丁目录从这条命令里算不出」报，打全了的也报。`--cwd` 是假仓根，`nested` 是它下面的目录，`<打补丁>` 是 `python3 scripts/gate/apply-writer-patch.py`：

| 样本名 | `bash -c` 的串（换行写成 ⏎） | bash 真跑时打补丁在哪 | 改前推出 | 改后推出 | 为什么 |
|---|---|---|---|---|---|
| heredoc-blank-last-line | `cat > notes.txt <<'EOF'` ⏎ `hello` ⏎（空行）⏎ `EOF` ⏎ `cd nested && <打补丁> patch-nested-half` | nested（量的） | nested | 算不出 | 空行切不出命令，它记的「行首、正文记号」两样留在分隔里；定界词那一行 `continue` 不记；于是 `cd nested` 的分隔带着正文记号，不平铺，整段算不出 |
| heredoc-comment-last-line | 同上，正文末行写 `# end` | nested（推的：同上一形，注释行 shlex 不出词） | nested | 算不出 | 同上：`#` 起的行 shlex 交空 |
| brace-group-plain | `{ cd nested; }` ⏎ `<打补丁> patch-nested-half` | nested（量的） | nested | 算不出 | 不接管道的花括号组跑在这一层，cd 留得住；改后组里的 cd 一律在层里 |
| line-after-brace-group | `{ echo a; } > log` ⏎ `cd nested` ⏎ `<打补丁> patch-nested-half` | nested（量的） | nested | 算不出 | `}` 之后这一行切不出命令，`}` 留在分隔里，下一行的 `cd nested` 前面有 `}`，不平铺 |
| python-heredoc-dict | `python3 - <<'PY'` ⏎ `settings = {` ⏎ `    'a': 1,` ⏎ `}` ⏎ `PY` ⏎ `cd nested && <打补丁> patch-nested-half` | nested（推的） | nested | 算不出 | 正文里单独一行的 `}` 记进分隔、层数减一，`{` 在 `settings =` 之后不在命令位置、不加；层数成负，整段不透明 |
| json-heredoc-two-objects | `cat > spec.json <<'EOF'` 写两个对象的数组（`{`、`},`、`{`、`}` 各占一行）⏎ `EOF` ⏎ `cd nested && <打补丁> patch-nested-half` | nested（推的） | nested | 算不出 | 单独一行的 `{` 两次加一，`},` 是一个词不算，末行 `}` 减一；层数停在 1，cd 在层里、末尾不平 |
| brace-group-to-tee-writer-inside | `{ cd nested && <打补丁> patch-nested-half; } 2>&1 \| tee log` | nested（量的） | nested | 算不出 | 组接管道跑在子 shell 里，组里的 cd 对组外不算数，对组里自己的打补丁算数；改后组里 cd 一律让整段算不出 |
| brace-group-to-tee-assignment-inside | `{ p=patch-half; <打补丁> "$p"; } 2>&1 \| tee log` | 仓根，补丁目录 patch-half（推的） | 代入得出 patch-half | 代入不了 | 层里的赋值不记，`$p` 算不出 |

另两种同类、不单列：heredoc 写 JSON 或 python 之后用变量给 `hook-swap.py` 的 `--scratch`，变量跟着整段不透明算不出，换完了的换钩子照点名的路径报（`hook_swap_read_back` 那一支「值代入不了…交 named_paths」）；heredoc 正文里有一行以 `cd` 起头的（喂给 `cat` 写一份脚本），之后的绝对路径 cd 改前算对、改后算不出。

这些都是偏严（多报一项「可能写了一半」），不是漏报；平常到什么程度：`grep` 记录、提示、技能与定义里 `bash -c` 之后接这两样的，只命中前两轮的探针串与讨论；工作根下各草稿 `tools/*.sh` 与顶层 `*.sh` 里提到这两样的脚本有 73 份（`grep -rlE … | wc -l`），没逐份看是不是在命令位置上起、有没有这几形（推的），八形里 `{ …; } 2>&1 | tee log` 与 heredoc 写 JSON 规格最像会真写出来的。

### 1b. 仍算出错目录的形态（漏报，推的）

| 样本名 | 串 | bash 真跑时 | 改后推出 | 读的结局文件 | 缘由 |
|---|---|---|---|---|---|
| env-attached-chdir | `env -Cnested <打补丁> patch-nested-half` | nested（量的：`bash -c 'env -Cnested pwd'` 打出 nested） | 仓根 | `<仓根>/patch-nested-half/apply-outcome.tsv`：不在就不报，在一份打全了的旧的也不报 | 认 env 换目录只认整词，「return any(word in ("-C", "--chdir") or word.startswith("--chdir=") for word in words)」（`task-server/task_server_scheduler_building.py:79`）；GNU env 的 `-C` 可以合写，`-Cnested` 不中 |
| env-grouped-chdir | `env -vC nested <打补丁> patch-nested-half` | nested（量的：`env -vC nested pwd` 打出 nested） | 仓根 | 同上 | `-vC` 不是整词 `-C`；`command_word_index` 照「while index < len(argv) and ("=" in argv[index] or argv[index].startswith("-")):」（`task-server/task_server_lib_configuration.py:1146`）只跳一个词，命令词落到 `nested` 上，写工具照样认得出 |
| （命令那一层） | 任务命令 `env -Cnested bash -c '<打补丁> patch-nested-half'` | nested（量的：`env -Cnested bash -c 'pwd'` 打出 nested） | 仓根 | 同上 | 串从哪起照「start = given_cwd if literal or not shell_directory_option(command[:index]) else None」（`task-server/task_server_scheduler_building.py:1480`），同一个判法 |
| backtick-cd-at-command-position | 行首 `` `cd nested; true` `` ⏎ `<打补丁> patch-half` | 仓根（推的：反引号里是命令替换、在子 shell 里跑） | nested | `nested/patch-half/…` 不在，不报 | 反引号是词两头剥掉的边缘字符（「word = token.strip(WORD_EDGE_CHARACTERS)」（`task-server/task_server_scheduler_building.py:143`）），`` `cd `` 剥成 `cd`，行首、后接分号，算平铺 |
| heredoc-space-dash-delimiter | `cat > notes.txt << -EOF` ⏎ `EOF` ⏎ `cd nested` ⏎ `-EOF` ⏎ `<打补丁> patch-half` | 仓根（量的：pwd 打仓根，notes.txt 里是 `EOF` 与 `cd nested` 两行） | nested | `nested/patch-half/…` 不在，不报 | shlex 把 `<< -EOF` 与 `<<-EOF` 都切成 `<<`、`-EOF`，服务都当 `<<-` 带定界词 `EOF`；bash 对前一种的定界词是 `-EOF` |

另一形没放进探针：判的那一刻目录在、跑的那一刻不在（`cd out; <打补丁> patch` 而 `out` 是这条命令后面才建的），服务照判的那一刻 `os.path.isdir` 算成 out，bash 的 cd 失败、照旧在仓根打。env 合写那三行是第二轮判决改法 2 第三形（`env -C`）自己的近亲，改法只认了整词（命令那一层那一行改前也一样）；反引号、`<< -EOF` 与判的那一刻目录在三形改前就有。判的那一刻服务手里有整个串，`-Cnested`、反引号、`<< -EOF` 都看得到。

### 1c. 正文点名的四种：都对

子 shell `( cd x; … )` 与 `$( … )`：`(` 记进分隔、层数加一，整段算不出。函数里的 cd：`SHELL_FUNCTION_DEFINITION` 命中，整段不透明。`&&` 链上前一条失败：`false && cd x` 里的 cd 分隔是 `&&`、不平铺，整段算不出；`cd x || exit 1` 的 cd 平铺照算，cd 失败时后面的不跑，算对。`time cd x`、`nice cd x`、`command cd x`、`builtin cd x`、`! cd x`：词里有 cd 而头一个词不是 cd，整段算不出。

四句：1a 五类八形是这一轮修补带进的连带（改前算对、改后算不出），1b 的 env 合写是修补没覆盖到的同类、别的几形改前就有；服务判的那一刻整个串都在手上；满足第 1 问「平常写法被连带降成算不出」「还有没有算出错目录的形态」两个分句；第二轮判决改法 2 给的「照算不出报」在 1a 的格上正是多报的来源，在 1b 的格上不起作用（那几形没被认成算不出）。

## 第 2 问：`hook_swap_read_back`

判定：成立（轻，偏严方向）。读 `task-server/task_server_scheduler_lifecycle.py` 与 hook-swap 两份推的；结局文件此刻的状态是这条腿直接读 git-dir 下那份、对着主工作区现算 sha256 量的。

### 2a. 被杀的这一条与结局文件里那一趟靠什么认

只靠一样：退 3 时输出里点名的反向规格路径，开头是不是这一条 `--scratch` 下的 `hook-swap/reverse-`，「marks = tuple(dict.fromkeys(os.path.join(form, *HOOK_SWAP_REVERSE_SPEC_PARTS) for form in (os.path.abspath(scratch), os.path.realpath(scratch))))」（`task-server/task_server_scheduler_lifecycle.py:1891`），而且只在同一条命令里另有一条换钩子的标记出现时才免，「and any(mark in text for mark in task_marks) and not any(mark in text for mark in one.owner_marks)):」（`task-server/task_server_scheduler_lifecycle.py:1807`）。结局文件开始行记着的进程号、开始秒数、进程启动刻都不读。退 0、1 不认归属，直接不报：「if verdict in (0, 1):」（`task-server/task_server_scheduler_lifecycle.py:1803`）、「continue   # 打全了、换完了、已退回：主工作区不在半截上」（`task-server/task_server_scheduler_lifecycle.py:1804`）。退 2、4 一律报。

### 2b. 认错时的方向

- 多报（退 4，归更早的一趟）：更早一趟（别的任务、别的会话的）完成之后，它换过的某份文件又被直接改了（Edit、合入），`--read-outcome` 读出「完成或退回之后文件又被改了」，「return 4, [f"✗ 完成过，可是这几份现在的 sha256 不等于回读的换后（之后被改了，或被退回了）：{path}",」（`scripts/gate/hook_swap_swap.py:375`）。此时被杀在写开始行之前的这一条（草稿里验、喂法、调用方名字核对那几步都在开始行之前；这几步多久没量，推的）一个字节没写，照样出一项 workspace-write-interrupted，依据写的是那一趟的文件，出路点名这一条的规格与草稿目录。此刻不在这个状态（量的）：git-dir 下的结局文件是 `task-server-known-break-items` 那一次换上，只一份 `task-server/task_server_lib_constants.py`，回读的 sha256 `4732003b…` 与那份文件此刻的 sha256 相同，读出来是退 0；主 agent 下一次直接改这份文件之后就是退 4。
- 多报（退 3，归别的任务）：别的任务的一趟换到一半被杀、之后有人照拒绝说明手工退回（几份文件全回到换前，按内容判就不再拦，结局文件不删），这一条被杀在写开始行之前，读出退 3、输出里只有那一趟的反向规格，这一条命令里没有别的换钩子标记，照报。
- 多报（同一条命令两条换钩子用同一个 `--scratch`）：两条的标记相同，都不免。工具员写明了。
- 漏报：没找到。写开始行之前主工作区里被跟踪的文件一份没动：次序是预演「rehearse(os.path.join(root, relative), planned[relative])」（`scripts/gate/hook_swap_swap.py:1049`），再写开始行与每份的文件行「write_outcome_start(outcome_file, root, reverse_spec_path, relatives, base_bytes, planned, mode)」（`scripts/gate/hook_swap_swap.py:1066`），之后才逐份「replace_file_contents_by_rename(path, planned[relative], check_before_rename=unchanged_since_draft)」（`scripts/gate/hook_swap_swap.py:1075`）。开始行不会写一半：整份写同目录临时文件、刷盘、「os.replace(temporary, path)」（`scripts/gate/hook_swap_swap.py:229`）。「换了第一份还没记」也不会：文件行与开始行同一次写全（文件头第 50 行，转述：换上之前先把要换哪几份、换前换后的 sha256 写进结局文件）。开始行之前写进主工作区的只有预演建的同目录临时文件（「prefix='.' + os.path.basename(target_path) + '.', suffix=TEMPORARY_FILE_SUFFIX, dir=directory)」（`scripts/kb/lib_atomic_replace.py:226`），预演完就删）；被 KILL 在预演那几步之间会留下一份以点起头的临时文件，读出退 0、不报。它不是写了一半的被跟踪文件，记在这里。
- 第二台：结局文件在共享根下的 git-dir，两台同一路径；读法在服务那一台起，现算的是共享的主工作区，两台读出相同（推的）。

### 2c. 反向规格在另一条的 `--scratch` 下那一支

逐种对过（推的）：前一条换完、后一条换到一半 → 只报后一条；前一条换到一半、后一条没起 → 只报前一条；反向规格在别的草稿目录下 → 两条都报；两条同一个草稿目录 → 两条都报。相对的 `--scratch` 照这一条起时的当前目录拼（「directory = writer.directory or writer.cwd」（`task-server/task_server_scheduler_lifecycle.py:1884`））：当前目录算错时（第 1 问 1b 那几形）标记跟着错，同一条命令里两条都不免，方向是多报；相对的 `--root` 在错目录里起读法，多半读不到结局文件、退 2、照报。没找到让这一支漏报的写法。

### 2d. 两格新格与五条开关

- `wrapped-patch-directory-unresolvable`（`task-server/task_server_selftest_memory.py:1375`）：三形各一条，判据「出一项、依据里有算不出」。brace-group-not-layered、heredoc-body-as-commands、string-env-chdir-unread 各把一形退回改前的算法，推出的错目录下没有那份结局文件、交 None、不出这一项，红；每条只让自己那一形红，红在名字说的那件事。没钉的一侧：1a 那几形（不接管道的组、heredoc 末行空行）没有格，改成「只在接管道或 `&` 时当一层」「末行空行不留记号」这类收窄不会让任何格变绿或变红，连带的多报不受格保护也不受格拦。
- `hook-swap-outcome-judged`（`task-server/task_server_selftest_memory.py:1406`）：五个样本。swap-outcome-unread 下五个样本一律点名，退 3 的前两个与退 0 的后两个红；swap-outcome-owner-ignored 下退 3 的两条都点名，前两个红；都红在名字说的那件事。没钉的：退 1（已退回）与退 4 没有样本（把 `(0, 1)` 改成 `(0,)` 不红）；没有 `cd` 之后写相对 `--scratch` 的样本（把标记照任务工作目录拼、不照这一条起时的当前目录拼，五个样本照绿）；没有符号链接下的草稿目录（只留 `abspath` 一种写法照绿）；`--scratch` 的值代入不了时交 named_paths 那一支没有样本。
- 主 agent 的开关复跑读数「逐格相同 10 条」照正文转述，这条腿没复跑。

四句：2b 三种多报里退 4、退 3 归别的任务两种是这一轮改法带进的「读结局」本身的边界（改前一律报，所以对改前不是回退，对「换完了的不报」这一判据是说宽了：结局文件里记的不是这一趟时，它的退 3、退 4 照算到这一条头上）；读出的那几行里带着开始行记的进程号（退 0、3 的那一行写「进程 N」），服务手里另有这条任务尝试的起止时刻，拿它们分不分辨得了要另核（推的）；满足第 2 问「认错时是多报还是漏报」与「两格新格与五条新开关红在名字说的那件事」两个分句；「写开始行之前不报」那一分句没打中。

## 第 3 问：退 10 出路的新句

判定：不成立（没打中）。读码与读文的。

hook-swap 的占法：先占草稿目录（`scripts/gate/hook_swap_swap.py:928`，转述：调 hold_scratch，被占着就在建树、写任何文件之前拒绝；那一行注释里带引号，不整行抄）；干跑到此为止、不占仓（`scripts/gate/hook_swap_swap.py:941` 起的两行，转述：带 --dry-run 的直接进下一步）；真换再占仓，「held, busy_lines = hold_repository(outcome_file, scratch, mode)」（`scripts/gate/hook_swap_swap.py:943`）。服务的新句：「f"这一种等占着的那一趟结束再交；两趟里有一趟是 --dry-run 的（这一条自己是，或占着的那一趟是），可以换一个 --scratch"」（`task-server/task_server_scheduler_lifecycle.py:1737`）。

| 这一条 × 占着草稿目录的那一趟 | 换一个 `--scratch` 之后 | 新句说 | 成不成立 |
|---|---|---|---|
| 干跑 × 干跑 | 新草稿目录占得到，干跑不占仓，过 | 可以换 | 成立 |
| 干跑 × 真换 | 同上，过（那一趟占着仓也不碍干跑） | 可以换 | 成立 |
| 真换 × 干跑 | 新草稿目录占得到；干跑不占仓，仓空着就过，仓被第三趟真换占着就退 10「这个仓的换上」，那一种另有自己的出路 | 可以换 | 成立 |
| 真换 × 真换 | 新草稿目录占得到，仓被那一趟占着，退 10「这个仓的换上」 | 括号里写「两趟真换换了草稿目录照样被这个仓的换上挡住」 | 成立；那一趟占了草稿目录、还没占到仓的那一瞬，这一条换了草稿目录会先占到仓，反过来挡住那一趟（两趟本来就互斥，换的人照样要等） |

三处说的是一回事：异常表那一行（`task-server/task_server_lib_constants.py:129`，转述：退 10 的等占着的那一趟结束再交；只有草稿目录被占、两趟里又有一趟是干跑的——这一条自己是，或占着的那一趟是——才能换，干跑不占这个仓、真换先占这个仓）写的是必要条件，提醒写的是充分条件，合起来是当且仅当，与第二轮判决采的辩方写法相同；README 中文第 174 行（`task-server/README.md:174`，转述：退 10 而是草稿目录被占的，两趟里有一趟是 `--dry-run` 的可以换一个 `--scratch`，仓被占的换草稿目录没有用）、英文与日文第 174 行（`task-server/README.en.md:174`、`task-server/README.ja.md:174`）逐句照中文译，条件与括号里的理由都在。hook-swap 自己的拒绝说明「"→ 怎么办：等那一趟结束再交，或换一个 --scratch（--dry-run 要并行的几条各给一个草稿目录；真换在同一个仓里一次只许一趟，另一趟真换在跑时换草稿目录也会被拒）"]」（`scripts/gate/hook_swap_swap.py:514`）不写条件、括号里补真换的那一道，与新句不冲突。

只有一处字面：服务的几处都写「真换先占这个仓」（`task-server/task_server_scheduler_lifecycle.py:1738`、`:1694`，`task-server/task_server.py:288`，`task-server/task_server_lib_constants.py:129`，`task-server/task_server_selftest_notifying.py:1448`，README 三语第 174 行英文作「a real swap holds it first」、日文作「本番の入れ替えは先にリポジトリを占有する」）。按字面读是「先占仓、再占草稿目录」，而代码是先占草稿目录、再占仓，hook-swap 文件头也这样写：「所以真换占完草稿目录之后、读结局文件之前，在结局文件所在的目录放一份占用记录 singlefs-hook-swap.lock（git-dir 下；仓根不是 git 工作树的根时是 <草稿目录>/hook-swap/），」（`scripts/gate/hook-swap.py:113`）。工具员的意思是「动手之前先占住仓」（它报告引的就是第 113 行），结论不受影响，只是字面会被读反。

四句：没有打中；服务判的那一刻看得到这一条自己带不带 `--dry-run`，看不到占着的那一趟是不是（新句把判断交给读的人，对）；对的是第 3 问「四种组合各成不成立」「三处说的是不是一回事」两个分句；字面那一处的改法写在第 5 节。

## 第 4 问：引文核对

判定：成立（打中）。改后那一版是量的：这条腿直接跑主工作区 `scripts/agents/cite-check.py`（名单上的文本工具）喂 `cite/forms-r3.md`，退 1，原样输出 `cite/forms-r3.out`（6 行，不贴进本报告：探针串里的位置与引号会被本报告自己的核对当成引文再判一遍）。改前那一版（`cite-check-r2-fix/base/`，第二轮修补换上之前）这条腿起不了，读数照改前正则推的，对照 `tools/run-cite-compare-r3.sh` 待主 agent 跑。下面 Q4-NN 就是 forms-r3.md 的第 NN 行。

### 4a. 掉出去不判也不列（打中）

`forms-r3.out` 末行「核了 6 处引文、1 处裸行号」：12 形里认出的是 Q4-03、04、05、12、13、14 六处带引文的与 Q4-11 一处裸行号，Q4-06 到 Q4-10 五处一处都没认出，既不在对不上里，也不在没判里。

- Q4-06 到 Q4-09：标题里本身带反引号的真小节，照逐字写进带反引号的小节写法。Q4-06 是 `.claude/singlefs-ai-sop/rules/command-safety.md` 第 67 行那一节、配真原文；Q4-07 同一节、引文在前、原文是伪造的；Q4-08 是 `.claude/kb/verification-build.md` 第 273 行（标题里另带「」）；Q4-09 是 `.claude/rules/format-evolution.md` 第 46 行。严格那两条认不出：带反引号的标题从 `@` 后紧跟的反引号起就取不到两个字（`scripts/agents/cite-check.py:117`，转述：带反引号的标题取除反引号、右引号、换行以外的字，到反引号为止）。兜底那一支这一轮改成不越过反引号（`scripts/agents/cite-check.py:128`，转述：兜底标题的字不含反引号、换行与引号；弄坏开关 loose-takes-prose 下才越过），同样从 `@` 起就断，整条丢掉。改前（推的）兜底的字是「除换行与引号都取」（`cite-check-r2-fix/base/scripts/agents/cite-check.py` 第 99、100 行，转述），越过反引号取到引号之前，这四形都列进没判。
- 这类标题有多少（量的，草稿里一段 python 扫 `.claude/rules/`、`.claude/kb/`、`.claude/singlefs-ai-sop/rules/` 与 `.claude/*.md` 共 86 份的标题行）：带反引号的标题 17 个，规则文件里 7 个（`.claude/rules/format-evolution.md:46`、`.claude/rules/implementation-first.md:1`、`.claude/rules/session-wrapup.md:12`、`.claude/singlefs-ai-sop/rules/command-safety.md:48`、`:67`、`:97`、`.claude/singlefs-ai-sop/rules/pushback-discipline.md:6`）。其中「## `pkill -f` / `killall` 一律禁用」（`.claude/singlefs-ai-sop/rules/command-safety.md:48`）正是 `.claude/agent-common.md` 第 21 行点名每个定义都照守的那一节（转述：跑命令照 command-safety.md 的「退不回去的操作，动手前先想一遍」与这一节），共用约束第 125 行又叫引规则写成小节写法。腿照这两句写，引文就不被核；配的是伪造原文（Q4-07 那一形）也交得出去，而且没判里看不到。仓里腿报告里眼下没有这样写的（`grep -rnE` 找「路径加 @ 紧跟反引号」在 `research/prompts/*-output.md` 里 0 处，不含禁读那一份与本报告，量的），是开着的口子，不是已发生的漏判。
- Q4-10：位置之后隔一个「，」再接引文（原文伪造）。严格那两条只许位置与引号之间隔空白与冒号，兜底同样；不越过反引号之后整条丢掉。同形的行号写法 Q4-11 照列进「没判（裸行号，没有引文可比）」。两种位置写法隔同一个逗号，一种被列出、一种消失。改前（推的）兜底取到「标题加反引号加逗号」、列进没判。
- 这一轮之后兜底那一支在 295 份报告上一处不收（量的：`cite/reports-new.out` 里「小节标题认不出结尾」0 处、「多半是标题后面接了说明」0 处；第二轮攻方腿量的改前是 67 处、抽看几乎都是散文）。对照探针的 `cite/reports-r3.compare.out` 会列出这 67 处逐处的去向；里面要是有「改前没判、改后没认出」而不是散文的，就是这一节的又一处，待主 agent 跑。

### 4b. 误红

- Q4-12：不带反引号、位置后接一段散文再接引文（原文是真的），标题取成「对不上就当场说 里写」、命中 0 次、判红，交回闸挡。工具员报告「没做什么」写明了这一形。散文写法被当成引文判红，按文件头第 21 行（转述：位置与引文之间隔着别的话的不是引文写法、不收不判不列）它应当不收；带反引号的同一句不收，不带的判红，两边不一致。
- Q4-04：带反引号、标题后面在反引号里接 `:45`。判红「命中 0 次」。不带逗号的同形 Q4-05 同样判红，改前带逗号的这一形是没判（推的，改前那一句对带不带反引号不分），这一轮让两者一致。`路径@标题:行号` 不是文件头认的写法，标题确实不逐字，记在这里、不算误红。
- 标题真带「，」、整行逐字、带反引号的 Q4-03 判对上；同一个标题在别的层级（`#` 个数不同）照认，命中只数整行逐字的标题行、不看井号个数（「return [(index, len(line) - len(line.lstrip("#"))) for index, line in enumerate(lines)」（`scripts/agents/cite-check.py:308`））。同名标题两处的照判红「命中 2 次」，改前就是这样；那样的节在这种写法下引不了。
- 旁见（改前就有，不是这一轮的）：认标题与定节尾都把以 `#` 起头的行当标题，「if lines[index].startswith("#") and (len(lines[index]) - len(lines[index].lstrip("#"))) <= level:」（`scripts/agents/cite-check.py:348`），代码块里的 `## …` 或 `# 注释` 也算（`.claude/kb/pending-rules.md:18` 是代码块里的一行 `##`），一节里代码块之后的原文判成不在那一节。

### 4c. 逐层去括号

去括号只在不带反引号、整串命中 0 次时做，「if not heads and not ticked and BROKEN != "paren-note-not-peeled":   # 弄坏开关：括号说明不去掉」（`scripts/agents/cite-check.py:331`），去掉后要恰好一处，「if len(found) == 1 or BROKEN == "paren-note-peeled-any":   # 弄坏开关：同名的几处也取头一处」（`scripts/agents/cite-check.py:336`）。

- 括号是标题的一部分：标题「X（Y）」在文件里，整串命中，不去括号，取对。写成半角括号或括号里写错一个字时整串 0 次、去括号后取「X」，文件里没有「X」就照整串判红，有「X」就取到「X」那一节，原文不在那里就判红；说明里写的是「X」那一节，读的人会困惑，不会放过。
- 撞上另一节：Q4-14 写的标题「判据（细则）」文件里没有，去掉括号落在「判据」一节，原文就在那一节，判对上（量的，forms-r3.out 里这一行不在对不上里）。更一般地，写错的子节标题「父（错字）」去括号后落在父一节，父一节包着它的子节（节尾是同级或更高一级的下一个标题），原文在子节里就判对上。这是按第二轮判决改法 4 的设计放宽的：标题不逐字也过，原文照核，不放过伪造原文。规则与 kb 86 份里没有「X」与「X（…）」同在一份文件里的一对（量的，同一段扫描「peel pairs 0」）。

四句：4a 是这一轮改法 4「兜底不收散文」带进的（改前列进没判，推的，待对照）；脚本判的那一刻整行都在手上，`@` 之后紧跟反引号、位置与引号之间隔逗号都看得到；满足第 4 问「此前列进没判的真小节写法有没有掉出去不判也不列」分句；4b 的 Q4-12 满足「合法写法被误红」分句（工具员已写明）；4c 没打中。第二轮判决改法 4 对 4a 那几格不起作用：它管的是「兜底只收位置形态确是小节写法的」，Q4-06 到 Q4-10 的位置形态就是小节写法，被丢是因为「不越过反引号」把标题里的反引号也当成了散文的边界。

## 第 5 节：改法（只在我的读法上推过、被攻过零轮）

全部没实现、没跑，一律「推的」；这一题是最后一轮，照判决的零轮规矩交主 agent 定。

| 打中的格 | 改法 | 修哪一格 | 量过没 |
|---|---|---|---|
| 1a heredoc 正文末行是空行或注释行 | 读到定界词那一行时把分隔里属于正文的记号清掉（那一行之前积下的行首与正文记号不并进下一条） | heredoc-blank-last-line、heredoc-comment-last-line | 推的，待主 agent 跑 |
| 1a heredoc 正文里单独一行的 `{` / `}` | 正文的行不把 `{`、`}` 记进分隔（花括号那一支加「不在正文里」） | python-heredoc-dict、json-heredoc-two-objects | 推的 |
| 1a `}` 留在下一条的分隔里 | 层数回到 0 的 `}` 之后，下一条照分号看平铺（只在组没接管道与 `&` 时） | line-after-brace-group | 推的 |
| 1a 不接管道的组、接管道的组里自己的写工具 | 组后面紧跟 `\|` 或 `&` 的才当子 shell；组里的 cd 对组里后面的命令另算一个当前目录 | brace-group-plain、brace-group-to-tee-writer-inside；assignment-inside 要同样另算变量 | 推的；改动比上面几条大，不做就只是多报 |
| 1b env 合写 | 换目录的选项认 `-C` 起头的合写与带 `C` 的短选项组（`-vC`），串里与命令那一层两处同改 | env-attached-chdir、env-grouped-chdir、命令那一层那一形 | 推的 |
| 1b 行首反引号 | 词里剥掉反引号之前带反引号的命令，整段当前目录算不出 | backtick-cd-at-command-position | 推的 |
| 1b `<< -EOF` | `<<` 后紧跟的词以 `-` 起头时，分不出 `<<-` 还是定界词带 `-`，之后整段算不出 | heredoc-space-dash-delimiter | 推的 |
| 2b 退 3、退 4 归别的一趟 | 读出 3 或 4 时比输出里开始行的「进程 N」与开始秒数和这条任务这次尝试的起止：开始秒数早于这次尝试开跑的不算这一条，照「换完了或还没动手」不报 | 2b 前两种多报；同一个 `--scratch` 那一种不修 | 推的 |
| 2d 格没钉的一侧 | `hook-swap-outcome-judged` 补退 1 一个样本、`cd` 之后写相对 `--scratch` 一个样本 | 2d 列的两处 | 推的 |
| 3 字面 | 「真换先占这个仓」改成「真换动手之前另占这个仓」，三语同改 | 3 那一处 | 推的 |
| 4a 带反引号标题、隔逗号 | 兜底那一支对 `@` 后紧跟反引号的位置照收，标题取到「反引号、可选冒号、引号」之前（允许里面有成对的反引号），列进没判；位置与引号之间隔「，」的同样列进没判 | Q4-06 到 Q4-10 | 推的，待对照探针 |
| 4b 不带反引号的散文 | 不带反引号、取到的标题里有空白而整串命中 0 次的，列进没判、不判红 | Q4-12；带反引号的同一句照不收 | 推的 |

## 没打中的形状

- 第 1 问：正文点名的子 shell、`$( … )`、函数里的 cd、`&&` 之后的 cd、`false && cd`、`cd x || exit 1`、`time` / `nice` / `command` / `builtin` / `!` 前缀（1c）都落到算不出或算对。另试过：`cd` 不带参数（去家目录，算不出）、`cd a b`（算不出）、`pushd -n`（算不出）、`<(cd x; ls)`（`<(` 切成一个词、`)` 让层数成负，整段不透明）、here-string `<<<`（不当 heredoc，对）、`$((1 << 2))`（当 heredoc，偏严，工具员写明了）、`<<'EOF'`、`<<\EOF`、`<<-` 两种写法、一行两个 heredoc、heredoc 起头那一行后面接 `; cd x`（cd 在这一层，算对）、`{ … }` 紧接 `>log`、`true;}`（shlex 切成 `true`、`;`、`}`，量的）、`! { …; }` 与 `time { …; }`（`{` 不在命令位置、被剥掉，整条里有 cd 不在头，算不出）。行尾 `\\`（转义的反斜杠）接换行会被当续行接上、吃掉下一行开头的 `cd`，算出错目录；这一形要行尾恰好写转义反斜杠，太刻意，只记在这里。
- 第 2 问：两条换钩子先后、各一个草稿目录的四种（2c）；`--scratch` 带末尾斜杠（两边 `abspath` 都去掉）；两个草稿目录名互为前缀（标记后面带 `/hook-swap/reverse-`，不会互相包含）；`--seal-outcome` 被杀（追加的行写一半时读成完成、而几份文件已都是换后，不是半截）；结局文件在草稿目录下（仓根不是 git 工作树）的两条各读各的；第二台。
- 第 3 问：四种组合与三处文字（第 3 问那一节）。
- 第 4 问：Q4-03、Q4-13 判对上（量的）；同名两处判红；标题在别的层级照认。去括号放过伪造原文的写法没造出来：原文照节核。

## 这条腿自己的限度

- 第 1 问与第 2 问全是读码推的：任务服务的函数这条腿没载，探针写好了没跑；bash 真跑的那一列只量了 `pwd` 落在哪，没起打补丁与服务。
- 第 4 问改前那一版没跑，「改前列进没判」是照改前正则推的。
- 第 2 问只读了 hook-swap 写结局文件、读结局文件与占用那几段，没通读 `scripts/gate/hook_swap_swap.py` 的回滚与 `--seal-outcome` 全部分支。
- 背景材料里的读数（18 格、455 格、10 条、17 条）没复核，照「推的」引。
- 只攻第 1 到 4 问；正推腿的第 5、6 问没碰；禁读的另一条腿报告没读，对照探针的报告清单也排掉了它。

## 要主 agent 跑的命令

两条都只读主工作区与工具员草稿里的源码、在内存里调被判文件的函数，不起服务、不写仓、不读写判定库；只写本草稿目录下的 `probe/` 与 `cite/`。

1. 第 1 问的 `shell_text_runs` 探针（改后、改前各一遍）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus/tools/run-runs-probe.sh /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus`（`.deliverables` 里 `tools` 写成交接）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；`task-server/task log <号>` 里以 `new` 与 `base` 起头的各 13 行（标签、样本名、`patch=` 代入后的补丁词、`directory=` 相对假仓根的当前目录或 None、`real=` bash 真跑时那一条在哪）与两行「done 13 段」。按读码推的（待跑）：`new` 下 1a 八形 `directory=None`（brace-group-to-tee-assignment-inside 是 `patch=None`、`directory=.`），`base` 下这八形 `directory=nested`（assignment-inside 是 `patch=patch-half`、`directory=.`）；1b 五形 `new` 与 `base` 都是 `directory` 与 `real` 不同（env 两形 `directory=.`、`real=nested`，反引号与 `<< -EOF` 两形 `directory=nested`、`real=.`）；控制组 heredoc-text-last-line-control 两边都是 `nested`；末尾 `shell_directory_option(['env', '-Cnested', 'bash'])=False`。哪一行与这里不同，就是这条腿推错了那一格，照读数改判。

2. 第 4 问改前改后对照（`tools/cite_compare.py`，第二轮攻方腿那一份）：
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus/tools/run-cite-compare-r3.sh /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r3-opus`
   - 在哪起：同上。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；日志两行以 `done` 起头的计数行（报告份数、改前改后认出几处、结局不同几处）；逐处在 `cite/forms-r3.compare.out` 与 `cite/reports-r3.compare.out`。按读码推的（待跑）：`forms-r3.compare.out` 里第 6 到 10 行改前 `skip`、改后「没认出」（4a）；第 4 行改前 `skip`、改后 `bad`；第 13、14 行改前 `bad`、改后 `ok`（去括号）。`reports-r3.compare.out` 里「改前 skip、改后没认出」的每一行逐个看：是散文的照第二轮判决采的设计，是小节写法的（位置与引号紧挨、只隔反引号、冒号或逗号）就是 4a 的又一处。

## 没做什么

- 没往任务服务交任务；两份探针一条没跑。第 1、2 问的结论都是读码推的，第 4 问只量了改后那一版。
- 没写模型目录与 `rerun.sh`：被判的不是崩溃状态模型，没有崩溃枚举要跑。
- 没建仓副本与编译目录；没碰判定库，没写会读写判定库的命令。
- 不判正推腿那几格，不替主 agent 采纳；第 5 节的改法都是零轮。
- 草稿目录（`du -sh` 4.0M）里留着 `tools/`（两份探针、两份起法、报告清单）、`cite/`（探针串与主工作区 cite-check 的两份原样输出）、`probe/root/`（空的假仓根，探针要用）、`heading-scan.txt`（标题扫描的原样输出），都在 `.deliverables` 里；主 agent 跑完探针、判决引完数之后整目录删。
