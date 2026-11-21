# 检出钩子循环变量、任务服务包着的写工具与退 10、引文核对小节标题、变异脚本分线程：治理文件轮第二轮，攻第一轮判决的修补（wrapped-writers-loop-paths-cite-titles-governance-r2，2026-11-21）

这一轮攻的是第一轮判决（`research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r1-main-verification.md`）「三、改法」四条已换上主工作区的修补，并派辩方腿复核这份判决；另带一处第一轮之后新做、还没被攻过的改动 E（变异脚本分线程）。方案按 `crates/` 今天的实现来谈：这一轮被判的文件里没有 `crates/` 下的代码，改的都是协作工具；这一题没碰 `crates/`。第一轮攻过而没打中的角度不重复：`modifies_workspace` 的判定没变、退 10 两种拒绝说明的认法与 `scripts/gate/hook-swap.py` 现行原文对得上、检出钩子新开关红在点名的格。

## 问题

1. 每一处修补做的是不是判决那一条说的事，修没修到位，有没有带进新的缺口；改动 E 做的是不是它要关的那件事（攻方腿）。
2. 第一轮判决够不够得着：判成坐实而采的依据撑不撑得住，改法选的做法是不是同样打中替代方案（辩方腿）。

## 实现今天的样子

主 agent 的观测（读的是换上之后主工作区的文件；读数取自主 agent 经任务服务交的那几趟，记录在 `records/2026-11-17-subagent账本清欠调度记录.md` 第 582–600 行）。改前原样与工具员报告在各草稿目录（`base/`、`new/`、`result.md`）。

- 改动 A 的修补（草稿 `~/.local/share/singlefs/work/detector-loop-target-paths-r1-fix/`）：检出钩子自证补 10 格（⑦ 判拒四格：经只赋过一次的名字引到循环变量、先判 exists 只写新文件、只写 .md 后缀、循环变量另在 `except … as` 里绑定；⑦ 判不到两格：推导式里的循环、lambda 经 map；stdin 一格钉拒绝说明里的出路；⑫ ⑯ ⑲ 各一格）；文件头 ⑦ 写明循环体里的条件不看、按每个候选判，拒绝说明加 Edit 或写临时文件再 `mv` 的出路；文件头照实写绑定次数数哪几种、「判不到的」补推导式与 lambda。不加新开关，新格挂进已有九条开关。读数：对照探针「26 种写法拒的条数与改后那一份逐格相同」；自证「自检通过（查了 1599 种）」；开关复跑 173 条逐格相同。
- 改动 B、C 的修补（草稿 `~/.local/share/singlefs/work/task-server-writers-exit-ten-r1-fix/`）：`task-server/task_server_scheduler_building.py` 加 `shell_text_runs`（只照平铺着跑的 cd、pushd 与只赋值的那一条算当前目录与变量，认不全的算不出），`workspace_writer_commands`、`wrapped_workspace_commands`、`script_body_workspace_commands` 交出认到的每一条写工具；`task-server/task_server_scheduler_lifecycle.py` 加 `writer_interruption`（补丁目录取代入之后的词，算不出的照「可能写了一半」报、不读结局文件）、`wrapped_hook_swap`（包着的 hook-swap 带 `--dry-run` 的也认，只读词只免它自己那一条），`workspace_write_judged` 逐条判；退 10 草稿目录被占那一支出路改成「这一种等占着的那一趟结束再交；占着的那一趟是 --dry-run 的，可以换一个 --scratch …」，异常表那一行分开写退 9 与退 10，README 三语第 174 行同改。新开关三条、新格三格。读数：整份自证「450 格都对」；三条新开关各只红点名那一格；20 条开关复跑逐格相同。
- 改动 B 的再修补（草稿 `~/.local/share/singlefs/work/task-server-read-outcome-foldable-b/`）：上一条把起 `--read-outcome` 的 `subprocess.run(read_back, …)` 挪进了 `writer_interruption`，算键库登记表 `scripts/gate/lib_function_units_registry.py:251` 点名的方法是 `SchedulerLifecycle.workspace_write_judged`，开关复跑开跑前 21 条全拒算；改成 `writer_interruption` 只拼命令、交回 `PatchOutcomeReadBack`，起法挪回 `workspace_write_judged`。
- 改动 D 的修补（草稿 `~/.local/share/singlefs/work/cite-check-title-parens-r1-fix/`）：`scripts/agents/cite-check.py` 小节标题里成对的括号照取（可再套一层），带反引号的取到反引号、不带的取到与开头那一层配对的右括号、「：」加引号或行尾；认不出结尾的列进「没判」并写「小节标题认不出结尾」；标题命中 0 次又带「，」「。」的单列。新开关三条。读数：自证过（小节标题带括号、带逗号与句号、括号没配上 9 种）；主 agent 复测攻方腿那一形交完整标题「不为省空间牺牲自包含（附）」；开关复跑 12 条逐格相同。
- 改动 E（新，草稿 `~/.local/share/singlefs/work/mutate-test-threads-divided/` 与 `~/.local/share/singlefs/work/mutate-header-named-cell/`）：起因是一条变异表任务开 7 个工作进程、每个测试二进制各 33 条线程、各四百多 %CPU，本机负载 191，NFS 回得慢、第二台读共享根超时。服务套的 `scripts/hosts/capped.sh` 把 `RUST_TEST_THREADS`、`CARGO_BUILD_JOBS`、`SINGLEFS_THREAD_CAP` 都设成登记值，`scripts/mutation/mutate.sh` 原样传给每个工作进程。改成：收到的 `RUST_TEST_THREADS`、`CARGO_BUILD_JOBS` 是正整数且小于线程上限的当用例显式要的、原样传，不小于上限的当服务设的、分成 max(1, 上限 ÷ 工作进程数)；`SINGLEFS_THREAD_CAP` 一律分。新格 test-threads-service、新开关三条；头注补了一条开关点名的格。读数：自证「mutate.sh 自检通过（查了 108 项…）」；开关复跑 51 条逐格相同。真任务上每个测试二进制的线程数没量过（那张表眼下没有待跑的行）。工具员报 `scripts/mutation/crates-mutation-rows.py` 同一毛病没改（它文件头明写「测试线程数：不压」，是门禁现行判法）。
- 已知开着、这一轮不重复报的：`crates-mutation-rows.py` 那一处等用户定；开关复跑那一格跑大表时核用不满；任务服务重启后卡在内核等 NFS 读委托那一件归别的会话。

## 范围

这一轮被判的文件（17 份，逐个列在 `research/prompts/wrapped-writers-loop-paths-cite-titles-governance-r2-snapshot/judged-sha256.txt`，都读主工作区此刻那一版）。第一轮判决与它引的主 agent 复跑读数在辩方腿的射程里。

## 要腿判的

攻方腿（每问回「成立」「不成立」「推不出」，推不出的算打中；打中的给能照喂的输入或 文件:行）：

1. 改动 A 的修补：补的 10 格是不是各钉住判决点名的那一件（在改动 A 之前的代码上该红的红不红靠哪条开关证、那条开关是不是真让这一格红）；文件头写的「条件不看、按每个候选判」「判不到的」与代码逐句对不对得上。
2. 改动 B、C 的修补：`shell_text_runs` 算当前目录与变量有没有算错的形态（`cd -`、`cd` 带选项、子 shell 里的 cd、`&&` 与 `;` 混写、赋值带命令替换、`export`、`pushd`/`popd` 不配对），算错时是落到「算不出、照报」还是算出一个错路径、读错结局文件；逐条判之后同一条任务报几项提醒、会不会把正常结束的写工具也报；`wrapped_hook_swap` 认 `--dry-run` 之后退 7、9、10 的三种提醒文字对干跑成不成立；起法挪回 `workspace_write_judged` 之后行为与挪之前是不是逐条相同。
3. 改动 D 的修补：成对括号的取法有没有取过头的合法写法（标题后面紧跟另一个括号说明、位置在前写法里括号套引号、中英文括号混配）；「认不出结尾列进没判」会不会把此前判得了的引文降成没判。
4. 改动 E：「不小于登记值当服务设的」这条判据在哪种调用下判错（用例显式要整份登记值的、登记值是 1 或 2 的、工作进程数大于登记值的、`MUTATE_CARGO_JOBS` 与 `CARGO_BUILD_JOBS` 同给的）；`SINGLEFS_THREAD_CAP` 一律分之后，工作进程数那一项自己也读它（文件头「并行」），会不会分了两遍或让工作进程数跟着变；分过之后一条任务各工作进程的线程合计是不是不超过登记值；新格与三条开关红在名字说的那件事。

辩方腿（逐条回「成立」「不成立」「推不出」）：

5. 第一轮判决「二、判决」四条：主 agent 引的读数分不分辨得出（对照探针、现读的几行、`citations_in` 的两次调用），判成误拒、回退的几处判得对不对。
6. 第一轮改法 1–4 选的做法与替代方案各在哪种情形下判错：改法 1 选「条件不看、文件头写明」而不认条件；改法 2 选「算不出照报」而不拒收这类命令；改法 3 选改提醒文字而不改 hook-swap 的退出码；改法 4 选「认不出结尾列进没判」而不判红。

## 分工

攻方腿做第 1 到 4 问，辩方腿做第 5、6 问。两条腿都读「范围」里的文件；只有一条腿读到的范围在报告里写明。
