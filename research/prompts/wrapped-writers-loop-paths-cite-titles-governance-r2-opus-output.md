# 攻方腿报告：治理文件轮第二轮，攻第一轮判决的四处修补与改动 E（wrapped-writers-loop-paths-cite-titles-governance-r2，2026-11-21）

被判对象读主工作区此刻那一版（`research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-snapshot/judged-sha256.txt` 列的 17 份；开工时 `sha256sum -c` 全部 OK，交回前再核有 6 份对不上：`task-server/README.md`、`task_server_lib_constants.py`、`task_server_lib.py`、`task_server.py`、`task_server_selftest_notifying.py`、`task_server_selftest.py`，`git diff` 看是别的改动在加 GPU 服务健康那一块，不碰这一轮判的几处；本报告引的 README 第 174 行与异常表第 129 行那两句交回前现查仍在，引的行号都是交回前现取的）。改前原样取各草稿目录的 `base/`。这条腿没写 `<模型目录>` 与 `rerun.sh`：被判的是协作工具，不是崩溃状态模型；要跑的只有一份改动 D 的改前改后对照探针（只调 `scripts/agents/cite-check.py` 自己的 `citations_in` 与 `judge`）与一份交主 agent 定的任务服务样本格，命令在「要主 agent 跑的命令」。

探针与样本（草稿目录 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-b/`，sha256 是这一次写完的）：

| 文件 | sha256 | 是什么 |
|---|---|---|
| `cite/forms.md` | `44b938fa912e1e0334338f871d3197b514773ba4db6e4c43398c0151142b418f` | 改动 D 的 16 行写法（第 3–18 行，D1–D16） |
| `cite/forms.out` | `c4afcb9174efb696e627e85d0119b177ac9064fcc10fbc8accf704199c2cdd18` | 主工作区 cite-check 跑 forms.md 的原样输出（这条腿直接跑的，名单上的文本工具） |
| `tools/cite_compare.py` | `b69f98d02714ad185c74e2e69c58af667fa69aa2ca8597c28885bf7f0e4dfe7b` | 改前（`cite-check-title-parens-r1-fix/base/`）与改后两份 cite-check 逐处比结局 |
| `tools/run-cite-compare.sh` | `f6cbbb20dd3e8c687223245d3f43be68ec9b6f5c0a3edfbeaa80a2461730a809` | 起上面那份三次：forms.md、第一轮攻方腿的 cite-forms.md、仓里 292 份腿报告 |
| `tools/reports.txt` | `ec33b4ba43055a83a2e65cbe2d930e62cadf8af13d1d86d308ed885ccc21b39f` | 那 292 份报告的绝对路径（不含禁读那一份与本报告） |
| `task-server-cells.py` | `ff09479f7337722d65492ff0da5b2887a8eefaf6e89aeea4c66c21884ca2a892` | 改动 B 三形算错目录的样本格（照 `cell_wrapped_patch_directory_resolved` 写），交主 agent 定 |

## 各格判定一览

| 问 | 判定 | 打中的东西（一句） | 证据性质 |
|---|---|---|---|
| 1 改动 A 的修补 | 不成立（没打中） | 10 格各钉住判决点名的那一件，8 格在开关 `python-loop-variable-unexpanded` 下红（即改前判法），stdin 一格在 base 上就红；文件头「条件不看」「判不到的」与代码逐句对得上。只有一处数目：工具员报告与正文写「九条开关」，文件头点名新格的是 8 条 | 读代码（推的）；8 条是 awk 数的（量的） |
| 2 改动 B、C 的修补 | 成立（打中） | 正文列的几形（`cd -`、带选项、子 shell、`&&` 与 `;`、命令替换、`export`、`pushd`/`popd`）都落到「算不出、照报」或算对；但另有三形算出一个错的当前目录、读错结局文件而漏报：花括号组接在管道或 `&` 上、heredoc 正文里的一行 `cd`、`-c` 串里的 `env -C`；另：干跑那一趟退 10（草稿目录被占）的出路漏了「这一条自己是 --dry-run 就能换草稿目录」；几个写工具里已经正常跑完的换钩子照样被点名（多报）；起法挪回逐条相同 | 读代码（推的）；样本格待主 agent 定 |
| 3 改动 D 的修补 | 成立（打中） | 带反引号、标题带「，」或「。」而写错（或伪造）的标题，改前判红、改后降成没判，交回闸放行——伪造的原文照过；标题后面紧跟一段括号说明（不带反引号）取过头、判红；兜底那一支把「位置之后隔几个字才接引号」的散文也收进没判，理由写成「括号没配上」 | 改后那一版量的（forms.out，与 292 份报告现跑）；改前那一版推的，对照探针待主 agent 跑 |
| 4 改动 E | 成立（轻） | 判据对服务那一路判得对，工作进程数受登记值压着，不会分两遍；但文件头「一条任务里各工作进程的测试线程合计不超过线程上限」对用例显式要的那一支不成立：显式给 8、开 4 个工作进程合计 32，超过 31；上卡的表串行跑测试二进制也照工作进程数分（少给，不是超卖）。新格与三条新开关红在名字说的那件事上 | 读代码（推的） |

四句（`.claude/rules/evidence-discipline.md` 第 125 行那一节），这一轮的对象不是带臂的实验，照第一轮攻方腿的读法答：「分不分辨臂」改答「打中的是不是这一轮的修补带进来的」，各节末尾逐条写。

## 第 1 问：改动 A 的修补（检出钩子 python 循环变量）

判定：不成立。读的是主工作区那四份（与 `detector-loop-target-paths-r1-fix/new/` 逐字节相同，`cmp` 核过），改前取同一草稿的 `base/`（= 第一轮改动 A 换上之后的样子）。

### 1a. 10 格逐格：钉的是哪一件、哪条开关让它红（推的）

改动 A 之前的判法由开关 `python-loop-variable-unexpanded` 重现：「if not (isinstance(node, ast.For) and isinstance(node.target, ast.Name) and isinstance(node.iter, (ast.Tuple, ast.List, ast.Set))」（`.claude/hooks/bash_command_detector_literals_and_signals.py:361`）之前那一句开关一开就交空表，循环变量照算不出。

| 格（简称） | 判决点名的那一件 | 候选次序：已有脚本 / 写范围外 / 账本 / records 在第几个 | 开关下红不红 |
|---|---|---|---|
| ⑦ `target = os.path.join('scripts', name)` 再 `open(target)` | 经只赋过一次的名字引到循环变量 | 第二个（`running.sh`） | 红：`name` 算不出，`target` 跟着算不出 |
| ⑦ 先判 `os.path.exists` 只写新文件 | 条件不看、按候选判（误拒那一形写进文件头） | 第一个 | 红 |
| ⑦ 只写 `.md` 后缀 | 同上 | 第二个 | 红 |
| ⑦ 循环变量另在 `except … as` 里绑定 | 绑定次数不数 `except … as` | 第二个 | 红 |
| ⑦ 推导式里的循环、lambda 经 map（期望放） | 文件头「判不到的」补这两形 | — | 不红（本来就放），只钉住「判不到」这一现状 |
| stdin 先判 exists 只写新文件 | 拒绝说明给出这一形的出路 | — | base 上就红（base 的拒绝说明没有那三段字），开关下退 0 也红 |
| ⑫ clerk 写范围外 | ⑫ 跟着收严要有格 | 第二个（`.claude/kb/a.md`；`records/a.md` 在 clerk 写范围里，写范围表 `.claude/hooks/agent-write-scope.tsv:53`） | 红 |
| ⑯ 现行账本 | ⑯ 要有格 | 第二个（`records/ledger.md`，样本登记表里 current 那一行） | 红 |
| ⑲ records 追加 | ⑲ 要有格 | 第二个（`records/run.md`） | 红 |

要拒的那个候选除了 exists 一格都不在头一个，所以「只判头一个候选」那种回退也会让这几格红，不只是整段关掉才红。

### 1b. 文件头与代码逐句

- 「名字整段只被这一个 for 绑定（绑定次数数赋值、for 与推导式的目标、with … as、函数与 lambda 的参数、import、增量赋值、海象与 del」（`.claude/hooks/bash-command-detector.sh:223`）：代码数的是 Store / Del 上下文的 Name、`ast.arg` 与 import 别名，「if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):」（`.claude/hooks/bash_command_detector_literals_and_signals.py:329`）。`except … as`、`def`、`class`、match 绑的名字在语法树里是字符串，不进这三支，与第 224 行说的不数一致。带注解的赋值（`f: str = …`）也是 Store 的 Name，归在「赋值」里，对得上。
- 「循环体里的条件（if、continue、break、try）不看、按每个候选判」（`.claude/hooks/bash-command-detector.sh:228`）：`LoopCandidates.body_nodes` 收的是循环体每条语句 `ast.walk` 出的全部节点，`if`、`try` 里的调用都在里面，`else` 段不在，与文件头一致。
- 「判不到的」第 229–234 行：推导式、lambda 两形与 `literal_loop_candidates` 只认 `ast.For` 一致；`async for` 是 `ast.AsyncFor`，同样不认、文件头没写，形状要包在 `async def` 里才写得出来，不算漏列。
- 拒绝说明那一句加在 ⑦ 每一次拒绝上（不分是不是循环引出的），`sed -i` 改脚本被拒也会看到「python 的 `for` 逐个取字面元素时…」那一句；这是多打一句说明，不是判错。

### 1c. 唯一对不上的数目（量的）

工具员报告写「不加新开关。下面九条开关的点名里补了新格」（`/home/fy5090/.local/share/singlefs/work/detector-loop-target-paths-r1-fix/result.md:51`），正文第 14 行同样写「新格挂进已有九条开关」。照文件头开关表数（`awk` 按开关块认新格名，命令在草稿目录可照跑）只有 8 条：`disable-check`、`allow-script-in-place-write`、`python-loop-variable-unexpanded`、`allow-bash-scope-write`、`heredoc-body-judged-as-commands`、`allow-ledger-append`、`allow-record-append`、`record-append-python-unseen`；工具员那张表本身也是 8 行。主 agent 的开关复跑「173 条逐格相同」照的是文件头，不受这个数影响。

四句：没有打中；判别的东西（整段 python 的语法树）钩子当时看得到；字面上对的是第 1 问「各钉住判决点名的那一件」「逐句对得上」两个分句；没有要修的格。

## 第 2 问：改动 B、C 的修补（任务服务包着的写工具与退 10）

判定：成立。全部是读主工作区代码推的：任务服务的判定函数这条腿载不进来（项目代码闸拒），复现写成样本格 `task-server-cells.py` 交主 agent 定。

### 2a. 正文点名的几形：都落到「算不出」或算对（推的）

逐形照 `shell_text_runs`（`task-server/task_server_scheduler_building.py` 第 160 行起）走一遍：

| 形状 | 走到哪一支 | 结局 |
|---|---|---|
| `cd -` | 目标是 `-`，`target` 交 None | 之后算不出，照报 |
| `cd -P x`、`cd -- x` | 选项跳过 | 算对 |
| `cd -Pe x`（合写的选项） | 剩两个词，`len(targets) == 1` 不成立 | 算不出，照报 |
| `(cd x && …)`、`$(cd x; pwd)` | `(` 记进分隔，层数 1，不是平铺 | 整段算不出，照报 |
| `cd x && a; b`、`cd x \|\| exit 1; b` | 后面接 `&&`、`\|\|` 的 cd 算平铺 | 算对（判的那一刻 x 在就当 cd 成了） |
| `false && cd x; b`、`cd x \|\| cd y; b` | 有一处 cd 不是平铺 | 整段算不出，照报 |
| `p=$(mktemp -d)`、`` p=`pwd` ``、`p=~/x`、`p=$HOME/x` | 代入后留着 `$`、反引号或 `~` 起头 | 算不出，照报 |
| `export P=/w/patch` | 声明词剥掉照赋值 | 算对 |
| 同名赋两次、`for p in …`、`read p` | 名字出现不止一次，或没有赋值 | 算不出，照报 |
| `pushd x` 不配 `popd`；`popd` | pushd 照 cd 算；popd 之后算不出 | 算对 / 照报 |

列出的这些形状都没有「算出一个错路径」的。

### 2b. 算出一个错目录、读错结局文件的三形（打中，推的）

三形里 `shell_text_runs` 都给出一个「算得出」的目录，而 bash 真跑时不在那里；结局文件拼在错目录上（「outcome_file = os.path.join(directory or "", patch, PATCH_WRITER_OUTCOME_NAME)」（`task-server/task_server_scheduler_lifecycle.py:1830`）），那里没有结局文件就走「return None   # 结局文件在动手换之前写：不在就是还没开始换，主工作区没动」（`task-server/task_server_scheduler_lifecycle.py:1832`）：被杀不报。错目录下另有一份同名、打全了的补丁目录时，读的是那一份、`--read-outcome` 退 0，同样不报。三形都能照喂（任务 `--cwd` 仓根，被杀退 137）：

1. **花括号组接在管道或 `&` 上**：`bash -c '{ cd nested; true; } | cat` 换行 `python3 scripts/gate/apply-writer-patch.py patch-half'`。bash 让接在管道上的整组跑在子 shell 里，组里的 `cd` 留不到组外，打补丁在仓根跑。切词时花括号被当成词两头的边缘字符剥掉（「WORD_EDGE_CHARACTERS = "[](){},'\"`"」（`task-server/task_server_scheduler_building.py:34`）；「word = token.strip(WORD_EDGE_CHARACTERS)」（`task-server/task_server_scheduler_building.py:111`）），`{` 剥成空词丢掉，组里的 `cd nested` 前面只剩行首、后面是 `;`，算平铺，`current` 变成 `<仓根>/nested`（「current = moved if moved is not None and os.path.isdir(moved) else None」（`task-server/task_server_scheduler_building.py:204`）），之后每一条都当在 nested 里跑。单行、多行写法一样；`{ …; } &` 同理。
2. **heredoc 正文里的一行 `cd`**：`bash -c 'cat > notes.txt <<EOF` 换行 `cd nested` 换行 `EOF` 换行 `python3 scripts/gate/apply-writer-patch.py patch-half'`。`shell_text_pieces` 把正文每一行当命令切（`<<` 只把紧跟的定界词当重定向目标去掉），`cd nested` 算平铺，打补丁那一条被当成在 nested 里。
3. **`-c` 串里的 `env -C`**：`bash -c 'env -C nested python3 scripts/gate/apply-writer-patch.py patch-nested-half'`。外层命令起 shell 之前的 `env -C` 有判（「start = given_cwd if literal or not shell_directory_option(command[:index]) else None」（`task-server/task_server_scheduler_building.py:1441`）），串里那一条自己的 `env -C` / `--chdir` 不看，当前目录照旧是仓根，读的是 `<仓根>/patch-nested-half/apply-outcome.tsv`（不在 → 不报），真打的是 `nested/` 下那一份。

改前（第一轮改动 B 之前）这三形写工具落成 `bash`、不读结局文件、照报；第一轮改动 B 之后补丁目录只认字面、`cwd` 不跟，同样会找错。所以这三形是第一轮修补（「照这一条命令里能认出的 `cd` 与赋值算」）没覆盖到的，属于第一轮判决甲那一类回退的残留，不是这一轮新开的口子。判的那一刻服务手里有整条命令与整个串，花括号、heredoc 定界、串里的 `env -C` 都看得到。

样本格 `cell_wrapped_patch_directory_misresolved`（`task-server-cells.py`，三形一格，期望出 workspace-write-interrupted、依据写判没打完或算不出）：按读代码推，此刻三形都不出，这一格红。推翻条件：主 agent 把它放进 `task_server_selftest_memory.py`、登记进 CELLS 之后单跑是绿的。

### 2c. 认写工具与脚本正文不跟 cd（见到的，不是第 2 问点名的那一件）

`WorkspaceWriter.cwd` 的注释写明认写工具、折相对路径都照任务的工作目录、不跟 cd；脚本正文的路径同样照它拼：「script = os.path.realpath(os.path.join(cwd, words[index]))」（`task-server/task_server_scheduler_building.py:1477`）。于是 `bash -c 'cd <草稿目录> && bash ./apply-chain.sh'`（`--cwd` 仓根）去仓根找 `apply-chain.sh`，找不到就整条不认写、`modifies_workspace` 也是假的，被杀不报。这与第一轮判决「`modifies_workspace` 判定没变」一致（是有意不跟），这里只记一笔：串里 cd 之后起的脚本正文，第一轮修补的「脚本正文从起它的那一条的当前目录起」只对正文里的命令成立，对找脚本本身不成立。

### 2d. 逐条判之后：一条任务报几项、正常结束的写工具报不报（推的）

- 报几项：不管认到几条写工具，`workspace_write_judged` 并成一项 workspace-write-interrupted（依据、写工具、要核的路径把判出可能写了一半的那几条接起来）；换钩子退 7、9、10 各一项。与正文说的一致。
- 正常结束的报不报：打补丁那一种读结局文件，打全了的跳过，不报，对。换钩子与暂存（`hook-swap.py`、`stage-mine.py`）不读结局，一律「要核的是这几处」：「return InterruptedWriter(tool, "", f"要核的是{('这几处：' + '、'.join(shown)) if shown else '它改的文件'}", True)」（`task-server/task_server_scheduler_lifecycle.py:1816`）。于是 `bash -c 'python3 scripts/gate/hook-swap.py --spec a.json --scratch s1 && python3 scripts/gate/hook-swap.py --spec b.json --scratch s2'` 在第二趟被杀，第一趟已正常换完也照样点名 a.json、s1；反过来在第一趟被杀，第二趟根本没起也照样点名 b.json、s2。这是多报（叫人多核几份没动过的文件），方向是偏严。第一轮改动之后只取头一个，那时是漏报第二趟；现在漏报没了、多出这一种多报。服务手里有换钩子的 `--read-outcome`（hook-swap.py 自己的结局文件）可以分辨，没用。
- 没跑到的那一支也算：`… || python3 scripts/gate/apply-writer-patch.py --rollback-pending` 这类兜底，没跑也会出「--pending-status 看有没有没换完的」那一句。同属多报。

### 2e. `wrapped_hook_swap` 认 `--dry-run` 之后三种提醒对干跑成不成立（部分不成立，推的）

hook-swap 干跑只占草稿目录、不占仓：「if dry_run:」（`scripts/gate/hook_swap_swap.py:941`）下一行直接「return swap_holding_repository(spec, hook, scratch, root, mode, feeds, dry_run, after_draft, after_swap, verify, after_file_swapped, outcome_file)」（`scripts/gate/hook_swap_swap.py:942`），真换才走第 943 行的 `hold_repository`。所以：

- 退 7、退 9：干跑同样先读结局文件，「一个字节没写」「这一趟没动任何文件」对干跑成立；「拒绝再换」对干跑是拒绝再干跑，字面偏了一点，意思不错。
- 退 10：干跑只会是草稿目录被占那一种。服务给的出路是这一句：「这一种等占着的那一趟结束再交；占着的那一趟是 --dry-run 的，可以换一个 --scratch（要并行的几条各给一个草稿目录；两趟真换换了草稿目录照样被这个仓的换上挡住）；」（`task-server/task_server_scheduler_lifecycle.py:1729`）。被挡的这一条自己是干跑时，不管占着的那一趟是不是干跑，换一个 `--scratch` 都能过（它不占仓）；这一句把能换的条件只写成「占着的那一趟是 --dry-run」，干跑被真换占着草稿目录时，读的人照它只会等。README 中文第 174 行同一句（转述：退 10 而是草稿目录被占的，占着的那一趟是 `--dry-run` 的可以换一个 `--scratch`）同样漏了这一形，英、日两份照它译。
- 判的那一刻服务看得到这一条自己带不带 `--dry-run`（命令在手上），看不到对面那一趟是不是。所以能补的是「这一条是 --dry-run 的同样可以换」，不需要读对面。

这一处是第一轮改法 3 收窄时丢掉的半句：第一轮攻方腿 3b 说的是「只在两趟里有一趟是 --dry-run 时有用」，改法只留了「占着的那一趟」那一半。

### 2f. 起法挪回之后逐条相同（没打中，读代码对的）

拿 `task-server-read-outcome-foldable-b/lifecycle.diff` 逐支对：改前 `writer_interruption` 自己起 `--read-outcome`，退 0、1 交 None，别的交带依据的 `InterruptedWriter`；改后交 `PatchOutcomeReadBack`，`workspace_write_judged` 起同一条 `read_back`、同一个 `cwd`（`directory or cwd`）、同一个超时，退 0、1 `continue`，别的交 `unread._replace(basis=…)`，依据串逐字同（`outcome_file` 是同一个值）；`apply-outcome-ignored` 下改前交依据为空的那一条，改后交 `unread`（依据同为空）。`writer_interruption` 只有第 1776 行一处调用（`grep` 全 task-server 目录），没有别的调用方会收到新的那种返回值。算键库登记行在 `scripts/gate/lib_function_units_registry.py:256`，点的就是 `workspace_write_judged` 与 `read_back`。

四句：2b 三形是第一轮修补没覆盖到的（不是这一轮新开的），2d 多报是这一轮「逐条判」带进来的，2e 是第一轮改法 3 收窄过头；服务判的那一刻三处要分辨的东西都在手上（整个串、这一条的 `--dry-run`）；满足的是第 2 问「算出一个错路径、读错结局文件」「会不会把正常结束的写工具也报」「对干跑成不成立」三个分句。

## 第 3 问：改动 D 的修补（引文核对小节标题里的括号）

判定：成立。改后那一版是量的：主工作区 `scripts/agents/cite-check.py`（与 `cite-check-title-parens-r1-fix/new/` 逐字节相同）直接跑在 `cite/forms.md` 上，退 1，原样输出在 `cite/forms.out`（不贴进本报告：那几行里的位置与引号会被本报告自己的核对当成引文再判一遍）。改前那一版（草稿 `base/`）这条腿起不了（项目代码闸拒；它在草稿里也载不到 preflight），读数是照改前正则推的，对照探针 `tools/run-cite-compare.sh` 待主 agent 跑。下面「第 N 行」都指 `cite/forms.md` 的行。

### 3a. 降成没判：带反引号、带「，」或「。」的写错的标题（打中）

`forms.md` 第 3 行（引文在前、带反引号）与第 5 行（位置在前、带反引号）：原文是伪造的，标题是真标题「对不上就当场说」后面多写了「，三样都说清」；第 6 行：标题是真标题截掉后半（「…整行抄，不许摘句」，少了破折号后那一段）；第 14 行：真标题后多一个句号。`forms.out` 里这四处都是「没判：…小节标题认不出结尾（…里面带「，」或「。」，文件里又没有这样的整行标题——多半是标题后面接了说明）」，不在对不上里。

- 走的是哪一句：`judge_section` 里「if not heads and re.search("[，。]", title) and BROKEN != "title-comma-judged":」（`scripts/agents/cite-check.py:249`）。它不分这处位置带不带反引号。带反引号的标题取到反引号为止（`TITLE_TICKED` 与 `TITLE_TICKED_END`，`scripts/agents/cite-check.py:89` 与第 91 行；转述），结尾是写出来的，「多半是标题后面接了说明」这个理由对它不成立。
- 改前（推的）：改前标题字符类只排除反引号、「」」与括号，「，」照取，带反引号时取到反引号，标题命中 0 次，判红（「标题…命中 0 次（要恰好 1 次）」）。
- 后果：交回闸只拒对不上的（`.claude/hooks/handback-guard.sh` 第 26 行，转述：有对不上的拒），没判放行。腿在带反引号的小节写法里写一个带逗号的错标题、再配一句伪造的原文，改前会被拦，改后交得出去。对照：同一伪造原文配不带逗号的错标题（第 4 行）照判红，配真标题（第 7 行）照判红——逗号就是那道口子。
- 现在发生得多不多（量的）：主工作区 cite-check 跑仓里 292 份腿报告（`tools/reports.txt`），「带「，」或「。」」那一种没判 0 处；这一形眼下没出现过，是开着的口子，不是已发生的漏判。

### 3b. 取过头：不带反引号的标题后面紧跟一段括号说明（打中）

第 8 行（引文在前）与第 9 行（位置在前）：标题是「放宽的（人脑带宽的产物）」，后面紧跟「（表里第一行）」，原文在那一节里是真的。改后标题取成「放宽的（人脑带宽的产物）（表里第一行）」，命中 0 次，判红。文件头第 14 行写的就是成对的括号照取、取到与开头那一层配对的右括号，这一形照字面确实该取进去，可读的人写的是「标题 + 说明」。同一形改用逗号接说明（第 13 行）落到没判，改用括号接说明落到判红，两种接法结局不同。改前（推的）：括号不进标题字符类，`（` 处既不能往下也找不到结尾，整条认不出、不判不列。所以这一形是从「看不见」变成「判红」（偏严，原文是真的也红），出路是给位置加反引号。

另两形（正文点名的）：

- 中英文括号混配：`TITLE_GROUP`（`scripts/agents/cite-check.py:85`）左右括号各认全角与半角，不要求配同一种；标题写成「放宽的(人脑带宽的产物）」这类混配时取成一个括号组，标题不逐字、命中 0 次、判红。判得对（标题本来就不逐字），没有取过头。
- 位置在前写法里括号套引号：仓里真有这种标题，`.claude/kb/prior-art.ja.md:794`、`.claude/kb/verification-build.md:273`（标题括号里带「」，后者还带反引号）。标题字符类排除「」」，带不带反引号都取不出，落到兜底那一支、列进没判。改前同样认不出（改前整条丢掉），改后至少列出来；这两节此刻引不了，文件头没写「标题里带「」的引不了」。

### 3c. 兜底那一支收进散文、理由写错（打中，量的）

兜底正则 `LOOSE_LOCATION_THEN_QUOTE`（`scripts/agents/cite-check.py:100`，转述：标题取 2 到 200 个不是换行与引号的字，懒匹配到下一个引号之前）的标题能越过收尾的反引号一路取到下一个引号。主工作区 cite-check 跑那 292 份报告，没判里写「小节标题认不出结尾」的 67 处（`grep -c`），抽看的前 30 处几乎都是「带反引号的位置、反引号收尾、隔几个字（里、的、说、表里）再接一句引号」的散文：标题本身完好，结尾也写出来了，没判的理由却一律写「括号没配上，或标题超出八十个字」（第 217 行那一句）。第 18 行 D16（两处带反引号的位置并排、后接一句引号）也是这样：头一处列进没判，取到的「标题」跨过了第二处位置。这些改前不认、不列，改后列进没判——不是把判得了的降下来，是没判多出一批、理由不对，会让读交回的人以为是括号写错。

### 3d. 判得了的没有被降成没判（除 3a 那一形，推的，待对照探针核）

严格那两条正则改动只有两处：带反引号的标题字符类放宽到「除反引号、」」、换行以外都取」，结尾只认反引号；不带反引号的标题多认成对括号。兜底那一支只收严格两条没认的位置（`located` 里没有的路径起点）。逐形推：改前严格认得、改后严格认不得的，只有「带反引号、标题里有括号或 `）` 而改前在那里收尾」这种原本就判错的写法；所以判得了而被降成没判的只有 3a（judge_section 那一句）。对照探针（`tools/run-cite-compare.sh`）逐处比改前改后结局，打出不同的每一处；推翻条件：292 份报告那一份输出里有 3a 那一形之外的「改前 bad 或 ok、改后 skip」。

四句：3a、3b、3c 都是这一轮修补带进来的（judge_section 的逗号那一句、成对括号的取法、兜底正则）；脚本判的那一刻整行都在手上，反引号写没写、标题后面跟的是什么都看得到；满足的是第 3 问「取过头」与「把此前判得了的引文降成没判」两个分句。

## 第 4 问：改动 E（变异脚本分线程）

判定：成立（轻）。判据对服务那一路判得对，只有文件头一句合计的说法比代码宽。全部读主工作区 `scripts/mutation/mutate.sh`（与 `mutate-header-named-cell/new/` 相同，`diff` 核过）推的，没量。服务每次尝试一律套 `capped.sh <登记值>`，登记值此刻是 31（`.claude/heavy-thread-default` 第一条非注释行），`RUST_TEST_THREADS`、`CARGO_BUILD_JOBS`、`SINGLEFS_THREAD_CAP` 三样同值；命令里写的 `env RUST_TEST_THREADS=<n>` 在 capped.sh 里面起，盖过服务设的那一个，`SINGLEFS_THREAD_CAP` 服务不许调用方写。

### 4a. 正文点名的四种调用（推的）

| 调用（环境变量 / 工作进程数 N） | 判成哪一种 | 每条 cargo test 收到 | 判得对不对 |
|---|---|---|---|
| 服务设的三样都 31，N = 7 | 不小于上限，服务设的 | RUST_TEST_THREADS 4、SINGLEFS_THREAD_CAP 4、CARGO_BUILD_JOBS 4，合计 28 | 对 |
| 用例显式要 `env RUST_TEST_THREADS=31`（整份），N = 4 | 等于上限，当服务设的分 | 7 | 照文件头第 173 行是有意的（要整份的设 `MUTATE_JOBS=1`），不算判错 |
| 登记值 1，显式 `RUST_TEST_THREADS=1` | 不小于上限，当服务设的分 | max(1, 1 ÷ N) = 1，且 N 受上限压到 1 | 值相同，只是 stderr 那一行把它写成「当服务设的分」 |
| 登记值 2，显式 1，N ≤ 2 | 小于上限，原样 | 1，合计 ≤ 2 | 对 |
| 工作进程数大于登记值 | 到不了：工作进程数头一项就是登记值（「bound_names+=("每台默认重活线程数"); bound_counts+=("$HOST_HEAVY_THREADS"); …」（`scripts/mutation/mutate.sh:2145`）），N ≤ 上限，max(1, 上限 ÷ N) × N ≤ 上限 | — | 对；只在弄坏开关 `workersbyprocessor` 下 N 可以超过上限 |
| `MUTATE_CARGO_JOBS` 与 `CARGO_BUILD_JOBS` 同给 | 先用 `MUTATE_CARGO_JOBS`，不分（「CARGO_JOBS_PER_WORKER="$REQUESTED_CARGO_JOBS"; CARGO_JOBS_SOURCE="MUTATE_CARGO_JOBS"」（`scripts/mutation/mutate.sh:2165`）） | 给的数原样，`MUTATE_CARGO_JOBS=31`、N = 7 就是 217 个 rustc 并行 | 文件头写明「先用 MUTATE_CARGO_JOBS」，是显式覆盖；第 176 行「免得 N 个 cargo 各按整份开 rustc」对这一支不保证 |

不是正整数的原样传（「[[ "$1" =~ ^[1-9][0-9]*$ && -n "$THREAD_CAP_FOR_SPLIT" ]] || return 0」（`scripts/mutation/mutate.sh:2132`））：写成 `031`、`+31` 这类 libtest 照样解析成 31 的写法会绕过「不小于上限」那一判、每个工作进程各拿 31。服务自己设的是规整的十进制，只有调用方故意这样写才到得了，记在「没打中的形状」。

### 4b. 合计超过登记值的情形（打中，文件头说宽）

文件头第 177–178 行把三样连成一句：用例显式要的原样传；没设的与服务设的取 max(1, 上限 ÷ N)，「RUST_TEST_THREADS=max(1, 线程上限 ÷ 工作进程数)，一条任务里各工作进程的测试线程合计不超过线程上限。」（`scripts/mutation/mutate.sh:178`）。显式那一支不分：任务命令写 `env RUST_TEST_THREADS=8 bash scripts/mutation/mutate.sh <bin> <源> <四条以上的表>`，8 < 31 判成显式、原样传，开 4 个工作进程，合计 32 > 31；开 7 个是 56。只要显式给的 e 落在 (上限 ÷ N, 上限) 之间就超。这不是这一改带进的（改前一切非空值都原样传，超得更多），是文件头那一句合计只对分过的那两支成立、读起来像对整条规则成立。`CARGO_BUILD_JOBS` 显式那一支同理。

### 4c. 分两遍、工作进程数跟着变：不会（推的）

- `SINGLEFS_THREAD_CAP` 分过的值只进 `TEST_THREADS_ENVIRONMENT`，交给 `run_cargo_test` 与 `run_test_executable` 起的子进程，不 export；本进程的 `HOST_HEAVY_THREADS` 与 `THREAD_CAP_FOR_SPLIT` 在分之前各取一次，收小工作进程数之后重算（`worker_count_and_head_line "$JUDGING_COUNT"` 再 `settle_test_threads_and_cargo_jobs`）用的还是这两个、与开跑时收到的 `CALLER_CARGO_BUILD_JOBS`，不读已 export 的、分过的 `CARGO_BUILD_JOBS`。所以本进程里不会分两遍，工作进程数也不跟着变。
- 外层已经把 `SINGLEFS_THREAD_CAP` 压小（开关复跑那一格每一趟只另设它）而 `RUST_TEST_THREADS` 还是服务的 31：31 ≥ 压小后的上限 → 照压小后的上限再分，是对的那一次分，不是第二遍。
- 测试二进制自己读 `SINGLEFS_THREAD_CAP` 起线程（`default_heavy_threads` 那一路）的，拿到的是分过的值；它与 libtest 的并行相乘（每个工作进程至多 (上限 ÷ N)² 条），那是测试自己起线程的事，改前改后都在，这一改没让它变坏。

### 4d. 上卡的表：串行跑的测试二进制也照 N 分（少给，推的）

上卡的表在工作进程收齐之后由父进程串行跑测试二进制（「串行跑基线（红了退 2）、各条（表序，判法同 judge_test_output）、各份副本还原之后那一次（退出码写进 restore-<号>.exit）；不领卡。」（`scripts/mutation/mutate.sh:3056`）），同一时刻只有一个二进制在跑，可它收到的仍是按 N 分过的 `RUST_TEST_THREADS` 与 `SINGLEFS_THREAD_CAP`（文件头「这两样只设在自己起的那几次 cargo test 与上卡的表起的测试二进制的环境里、不 export。」（`scripts/mutation/mutate.sh:181`））。这是少给（跑得慢），不是超卖；`RUST_TEST_THREADS` 那一半改前就这样，`SINGLEFS_THREAD_CAP` 那一半是这一改新加的。

### 4e. 新格与三条开关（推的，与工具员读法相合）

- `registeredtestthreadskept`：服务设的整份当显式原样传 → test-threads-service 那一格收到 10 不是 2，红；别的格 `RUST_TEST_THREADS` 不是空就是 7（小于 10），不受影响。
- `threadcapundivided`：不往环境里加分过的 `SINGLEFS_THREAD_CAP`，cargo 收到 run_case 给的 10 → service 与 caller 两格的 thread-cap 检查红；test-threads、floor 两格不查这一样，不红。与文件头点名一致。
- `registeredcargojobskept`：服务设的 `CARGO_BUILD_JOBS` 整份原样 → service 格的 build-jobs 检查红；别的格不查 `CARGO_BUILD_JOBS`（`grep` 自证里查编译并行度的只有第 1556 行那一处），不红。
- 新格把三样都设成 10 = 上限，正好落在边界上：把判据的 `<` 改成 `<=` 也会让它红。它没钉「显式给的是上限减一」那一侧（caller 格用 7），`<` 改成 `< 上限 - 1` 这类偏移不红；这是边界只钉了一侧，记一笔，不算打中。

四句：4b 不是这一改带进的（改前更宽），4d 的 `SINGLEFS_THREAD_CAP` 一半是这一改带进的少给；脚本判的那一刻收到的值、上限、工作进程数都在手上，显式与服务设的只能靠大小分（这一点文件头写明了）；满足的是第 4 问「合计超登记值的情形」分句。

## 第 5 节：改法（只在我的读法上推过、被攻过零轮）

全部没实现、没跑，格里一律「推的」；主 agent 要用照判决的零轮改法规矩先做 A/B。

| 打中的格 | 改法 | 修哪一格 | 量过没 |
|---|---|---|---|
| 2b 花括号组接管道或 `&` | `shell_text_pieces` 不把 `{`、`}` 当边缘字符剥掉，`shell_text_runs` 把 `{ … }` 当一层：组后面接 `\|` 或 `&` 的，组里的 cd 让整段算不出；或简单些：整段里出现 `{` 词就当前目录算不出 | 2b 第 1 形 | 推的，待主 agent 跑 |
| 2b heredoc 正文里的 cd | 切词时认出 `<<定界词`，到定界词那一行之前的正文不当命令；或整段有 `<<` 就当前目录算不出 | 2b 第 2 形 | 推的 |
| 2b 串里的 `env -C` | 串里每一条也照 `shell_directory_option` 判，那一条有 `-C` / `--chdir` 的当前目录算不出 | 2b 第 3 形 | 推的 |
| 2b 三形一起的兜底 | 结局文件按算出的目录找不到时不交 None，改成也试任务的工作目录；两处都没有才信「还没开始换」 | 只修「错目录下没有结局文件」那一半；错目录下恰好有一份打全了的同名补丁目录时不起作用 | 推的 |
| 2d 正常结束的换钩子也点名 | 换钩子那一种照它自己的 `--read-outcome` 判，结局写着完成的不点名 | 2d 第一条；`--rollback-pending` 兜底没跑的那一形不起作用 | 推的 |
| 2e 干跑退 10 的出路 | 草稿目录被占那一项：这一条带 `--dry-run` 的写「这一条是干跑、不占仓，换一个 --scratch 就能过」；README 三语第 174 行同改 | 2e | 推的 |
| 3a 带反引号的错标题带逗号降成没判 | `judge_section` 的逗号那一句只对不带反引号的位置用（`citations_in` 把带不带反引号交下去）；带反引号的照命中 0 次判红 | 3a；不带反引号、标题真带逗号又写错的那一形仍是没判 | 推的，待对照探针 |
| 3b 括号说明取过头 | 不改取法（照文件头），自证补一格「不带反引号、标题后紧跟括号说明 → 判红」钉住现状，文件头写明「标题后面要接说明的，位置加反引号」 | 只让文件头照现状 | 推的 |
| 3c 兜底收进散文、理由写错 | 兜底那一支对带反引号的位置只取到收尾的反引号；收尾反引号之后不紧跟引号的不收；理由按实际（括号没配上 / 超长）分开写 | 3c | 推的 |
| 4b 合计说宽 | 文件头第 178 行那一句挪到「没设的与服务设的」那半句里，另写「用例显式要的不分，N 个工作进程合计是 N × 给的数」 | 只让文件头照现状 | 推的 |
| 4d 上卡串行少给 | `run_test_executable` 起的那几次用不分的上限（同一时刻只有一个二进制） | 4d | 推的 |

## 没打中的形状

- 改动 A：10 格逐格对过候选次序与开关（1a）；`except … as`、`def`、`class`、match、带注解的赋值、嵌套函数里的同名绑定都对过绑定计数；`async for` 不认、文件头没写（形状要包在 `async def` 里，不算漏列）。没找到文件头与代码对不上的句子。
- 改动 B：正文点名的 `cd -`、合写选项、子 shell、`$(…)`、`&&` 与 `;` 混写、`||` 后接 cd、命令替换与反引号赋值、`~`、`$HOME`、`export`、`pushd`/`popd` 不配对、同名两次赋值、`for`/`read` 变量、`cd ""`（空词被丢、剩下光 `cd`）、`cd "$1"`、`bash -c "cd x"` 套在串里（引号里的 cd 不当 cd，对）都落到算不出或算对（2a）；`cd -P` 进符号链接再 `..` 的物理路径与 normpath 不同，要 `-P` 加链接加 `..` 三样凑齐，太刻意，没列为打中。退出码属于串里哪一条：`apply-writer-patch.py` 只退 0 与 2，换钩子之后用 `&&` 接别的命令时 7、9、10 只可能来自换钩子；用 `;` 接一条自己会退 7、9、10 的命令才会认错，没找到这样的现役命令。起法挪回（2f）逐支相同。
- 改动 C：退 7、退 9 两种提醒对干跑成立（2e）；异常表那一行开头一句退 9、退 10 已分开写。
- 改动 D：成对括号一层套一层（`.claude/kb/vm-harness.md` 第 143 行那种两层）取全、判绿（forms.md 第 12 行）；半角外括号配全角标题括号（第 11 行）取全；带反引号、同一括号里两处位置（第 17 行）头一处照判；带反引号、标题括号没配上（第 16 行）照判红。
- 改动 E：登记值 1、2，工作进程数大于登记值，`MUTATE_CARGO_JOBS` 与 `CARGO_BUILD_JOBS` 同给，外层压小 `SINGLEFS_THREAD_CAP`，收小工作进程数后重算（4a、4c）都没判错；`031`、`+31` 这类写法能绕过「不小于上限」，要调用方故意写，记在这里不算打中；新格边界只钉了 `<` 改 `<=` 那一侧（4e）。

## 这条腿自己的限度

- 改动 A、B、C、E 的判定全是读代码推的；任务服务的判定函数载不进来，改动 B 的三形只写成了样本格，没跑。
- 改动 D 改后那一版是量的（这条腿直接跑主工作区 cite-check，名单上的文本工具）；改前那一版起不了，「改前判红」是照改前正则推的，对照探针待主 agent 跑。
- 292 份报告是现在仓里的腿报告，多数指向旧树，对不上的那一大批（11257 处）与这一轮无关；我只拿它数「认不出结尾」那一种没判。
- 背景材料里的读数（1599 种、173 条、450 格、21 条、12 条、108 项、51 条）没复核，照「推的」引。
- 只攻分给我的第 1 到 4 问，辩方腿的第 5、6 问没碰；禁读的另一条腿报告没读，对照探针的报告清单也排掉了它。

## 要主 agent 跑的命令

1. 改动 D 改前改后对照探针（只载入两份 `cite-check.py` 调 `citations_in` 与 `judge`，不起服务、不写仓、不读写判定库；只写草稿目录 `cite/` 下三份 `.compare.out`；禁读那一份报告不在清单里）：

   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-b -- bash /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-b/tools/run-cite-compare.sh /home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-b`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-b`（`.deliverables` 里 `tools` 写成交接）。
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里三行以 `done` 起头的计数行（报告份数、改前改后认出几处、结局不同几处）；逐处的不同在 `cite/forms.compare.out`、`cite/r1-forms.compare.out`、`cite/reports.compare.out`，每行写改前、改后各判成什么。按读代码推的应当是（推的，待跑）：`forms.compare.out` 里 forms.md 第 3、5、6、14 行改前 `bad`、改后 `skip`（3a）；第 8、9 行改前没认出、改后 `bad`（3b）；`reports.compare.out` 里不出现「改前 `bad` 或 `ok`、改后 `skip`」而原因不是 3a 那一形的行（3d）。哪一行与这里不同，就是这条腿推错了那一格，照实际读数改判。

2. 改动 B 三形的样本格（2b）没写进自证：要不要把 `/home/fy5090/.local/share/singlefs/work/wrapped-writers-loop-paths-cite-titles-governance-r2-opus-b/task-server-cells.py` 里的 `cell_wrapped_patch_directory_misresolved` 放进 `task-server/task_server_selftest_memory.py`、在 `task_server_selftest.py` 的 CELLS 里登记，交主 agent 定（写范围不含任务服务的副本，这条腿也载不进任务服务的库）。放进副本之后单跑的写法照 `task-server/task_server_selftest.py` 第 612 行那一句（这一份在我开工之后被别的改动动过，行号是交回前现取的）（`TASK_SERVER_SELFTEST_CELLS=<格名>` 跑 `task_server_lib.py --selftest`）；按读代码推，此刻这一格红。

## 没做什么

- 没往任务服务交任务；对照探针、样本格、任何自证与开关复跑一条没跑。改动 A、B、C、E 的结论都是读代码推的；改动 D 只量了改后那一版。
- 没写模型目录与 `rerun.sh`：被判的不是崩溃状态模型，没有崩溃枚举要跑。
- 没碰判定库，没写会读写判定库的命令。
- 不判辩方腿那几格，不替主 agent 采纳；第 5 节的改法都是零轮。
- 没建仓副本与编译目录。草稿目录（`du -sh` 4.0M）里留着 `cite/`（探针串、主工作区 cite-check 的两份原样输出，其中 `reports-new.out` 是跑 292 份报告的那一份）、`tools/`、`task-server-cells.py`、`a.diff`，都在 `.deliverables` 里；主 agent 跑完对照探针、判决引完数之后整目录删。
