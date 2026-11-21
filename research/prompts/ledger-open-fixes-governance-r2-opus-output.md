# ledger-open-fixes-governance-r2：攻方腿报告（第 1 到 5 问，2026-11-19）

约定：每问回「成立」「不成立」「推不出」。「成立」指问里设想的那个缺口在、算打中；推不出的也算打中。所有读数都是我按代码推的，探针没跑（子 agent 不交任务），命令在末尾「要主 agent 跑的命令」一节，每条写明看哪一行算打中。

## 探针（草稿目录，没有写模型目录）

这一轮的被判对象是治理文件，没有写崩溃状态模型，所以没有 `<模型目录>/tools/rerun.sh`；探针放在草稿目录 `/home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/probe/`，sha256：

| 文件 | sha256 | 问 |
|---|---|---|
| `taking_leftover_probe.py` | `b3eff1c56013d9a25ab83f6032a0e2d8c22b04d3ae2b804a16f8116a336def82` | 第 1 问 |
| `evidence_exact_rerun_probe.sh` | `c605f4ee814e2f7acb76a6c196809272843642139a2e4473f40a163c59f99605` | 第 2 问 |
| `own_layers_probe.py` | `e3e4b918d2bc4ff6cab6282828b5a49e8b170cfe49bbec0371f1086e9eb6b59a` | 第 3 问 |
| `pathspec_delete_probe.sh` | `af8c57cff97a6ce7d873415a89ebb94057b67a2fef1b91d7663976fbd3992948` | 第 4 问 |
| `pin_gate_probe.sh` | `06b4c055a37866cc7ac8fe0fe40fe6db8698db8e930c34e99dfb9f438d98fe3b` | 第 5 问 |

## 各格判定一览

| 问 | 格 | 判定 | 一句话 | 读数 |
|---|---|---|---|---|
| 1 | `.taking` 被 KILL 留下之后的长挡 | 成立（打中） | 没有谁清；按仓的那一份留下之后，每一次遇到被 KILL 的持有者留下的旧记录，全仓真换与 `--seal-outcome` 都退 10，记录放多久都一样；拒绝说明还写「300 秒没刷新就不再挡」，这句在这一格上是假的 | 推的，待主 agent 跑 |
| 1 | 不 KILL 也留下 `.taking` | 成立（打中，新的） | 第一轮判决第 4 条修了「link 回复丢了、重发报已有」，只修在放记录那一次；摘旧记录那一次 `link(path, path.taking)` 照旧报已有就当别人在摘，`finally` 不罩它，`.taking` 留下 | 推的，待主 agent 跑 |
| 1 | TERM 也留下 `.taking` | 成立（打中，新的） | 停任务与 `proc.py stop` 先发 TERM；占着草稿目录时 TERM 由 `release_then_die` 处理，放完锁按默认处理重发、进程当场死，摘旧记录的 `finally` 跑不到 | 推的，待主 agent 跑 |
| 1 | 两份锁反序互等 | 不成立 | 取锁只有「草稿目录→按仓」一种次序，`--seal-outcome` 只取按仓、`--dry-run` 只取草稿目录；取不到当场退 10，没有等 | 读代码 |
| 1 | `--dry-run` 不占按仓的锁 | 不成立 | 演练绿不带进真换：真换在锁里重读结局文件、重做基线核对，读到别的就当场拒（退 7、9、3），拒得对 | 读代码 |
| 2 | E162、E174、E184 今天判什么 | 成立（打中：背景里的说法错一半） | 三个实验的装置源码今天都不在改动范围里，这一格不判它们；E184 的装置在 `crates/singlefs-verification/src/bin/`，这一格的射程根本不认，「E184 以后改装置就红」不成立；装置在 `crates/*/src/bin/` 的实验有 19 个，都不判 | 读代码、`ls` 量的 |
| 2 | 改 exact 那一段的出路 | 不成立（出路走得通） | 重跑一条 exact 驱动、产物存进 `research/results/` 就绿 | 推的，待主 agent 跑 |
| 2 | 第一轮第 5 条的改法本身 | 成立（打中：改法碰不到打中的格） | 同一个出路对「改的是 timing 那一段」一样绿：只重跑 exact、甚至放一份任意以 `e<号>` 起头的文件，timing 产物不重跑也绿 | 推的，待主 agent 跑 |
| 2 | 成功时照打 git 警告让样本判错 | 不成立 | 样本档只核退出码与 `want=` 子串，多出的行既不让红样本变绿也不让绿样本变红 | 读代码 |
| 3 | 怎么认「自己写的」 | 成立（打中） | 认的是层（自己认领的、派发点名的草稿目录），不认是谁写的：承接的子 agent 被前一个写坏的行拦；嵌套派发时上层被下层声明过的交接副本拦 | 推的，待主 agent 跑 |
| 4 | 换名、挪走之后登记对不对得上 | 不成立 | 实验页与复跑表点名的是带日期的产物名与 `<输出文件>` 占位，换名正是钩子 ⑤ 给的出路；挪走则路径不变 | 读文件 |
| 4 | 删旧闸在边上判什么 | 成立（打中，新的） | 路径按 git 的 pathspec 解：输出文件名里有 `[` `]` `*` 时，一份没进 git 的输出被判成「在 git 里、没改过」删掉 | `git ls-files` 只读量了通配那一半；入口整条推的，待主 agent 跑 |
| 4 | `git` 不在 PATH、调用目录不在仓里 | 不成立 | 两种都落进「不在 git 里」那一支，拒、退 2、文件不动；经服务跑时缺 git 先被 run-condition 退 78 | 读代码 |
| 5 | 认起跑脚本、认装置、认登记 | 成立（打中） | 钉值行带行尾注释的不认（同一格认装置那一行却放过行尾注释）；头一句注释先点名上一次的登记就认错登记，那一份已归档就「不判」 | 推的，待主 agent 跑 |
| 5 | 登记已归档就不判 | 成立（打中，比问的更宽） | r2、r3 永远罩不到；r4 的登记能留在树里只是因为重判脚本的文档串碰巧写了它的路径；这一格的触发是「装置在改动里」，第一轮第 13 条那一种「只改钉值」整个罩不到 | 推的，待主 agent 跑 |
| 5 | 不是修订却放行的第十二节行 | 成立（打中） | 原地把登记时写死的那一行改成新值、词中间多一个空格、把已有的一行再抄一遍，都放行 | 推的，待主 agent 跑 |

## 读到的范围

主工作区此刻的：`scripts/gate/hook-swap.py`（文件头、第 2368–2996 行）、`scripts/gate/changed-paths.sh`（第 1–248 行）、`.claude/gate.d/doc-experiments.sh`（对 HEAD 的 diff 全部、evidence-in-repo 一格的 python 段、pinned-apparatus-revision 整格）、`.claude/gate.d/doc-process-records.sh`（`load_change_range` 一段）、`.claude/hooks/completion-cleanup-guard.sh`（对 HEAD 的 diff、`subagent_roots`、`decide` 开头、自证的 `Scenario`）、四份虚机入口的删旧闸那一段与参数检查、`e185-r2/r3/r4` 起跑脚本的钉值与登记点名、`research/prompts/e185-r4-prereg.md` 第十二节、`scripts/experiment-apparatus/replay.sh` 里 E162、E174、E184、E201、E210 的行、`scripts/kb/archive-past-rounds.py` 文件头与 `still_an_input`、两份样本（`evidence-in-repo-mixed-replay-red`、`pinned-apparatus-revision-red`）与 `batch-scope-git-warning-red`。

没读：`.claude/gate.d/split-proposal-ledgers.tsv`、`stage-owners.tsv`、`scripts/kb/stale-candidates-record-carriers.tsv`、`break-switch-replay-files.tsv`、`selftest-registry.tsv` 的那几行、`scripts/experiment-apparatus/README.md`、`.claude/main-agent.md` 与两份规则、两份定义的改动（这些归第 6 到 9 问，分给辩方腿）。

## 第 1 问：换上工具的按仓占用与硬链接摘旧记录

### 1.1 `.taking` 被 KILL 留下：成立（打中）

谁来清：没有。代码里删 `.taking` 的只有摘的那一趟自己的 `finally`：「os.unlink(taking)」（`scripts/gate/hook-swap.py:2822`）。文件头写的出路是人工：「别的趟正在摘（.taking 在）时这一趟当占着，退 10；摘的那一趟被 KILL 会留下 .taking，拒绝说明点名它，确认没有在跑的 hook-swap.py 就删它。」（`scripts/gate/hook-swap.py:86`）。

能照喂的历史（按仓的那一份，探针 `taking-left-by-kill-repository-lock` 一臂）：

1. 一趟真换被 KILL（内存上限、断电），`.git/singlefs-hook-swap.lock` 留下，写它的进程没了。
2. 下一趟真换判它不挡，进 `remove_stale_record`：「os.link(path, taking)」（`scripts/gate/hook-swap.py:2806`）做成之后、删记录之前被 KILL。`.taking` 与旧记录都留在盘上。
3. 之后每一趟真换与 `--seal-outcome`：判旧记录不挡，再去 link `.taking`，报已有，「return RECORD_BEING_REMOVED」（`scripts/gate/hook-swap.py:2810`），当占着，退 10。记录的修改时间放多久都一样：持有者读成「这一台、进程没了」，根本不看年龄，挡人的是 `.taking`。
4. 拒绝说明里 holder 是 None，`on_this_machine` 为假，照打「秒没刷新就不再挡；确认它死了也可以直接删」（`scripts/gate/hook-swap.py:2750`，草稿目录那一份是第 2724 行）。前半句在这一格上是假的：不会自己过期。
5. 照后半句删掉记录，这一趟换得上；`.taking` 还在。下一次再有一趟真换被 KILL 留下记录，第 3 步重来。删记录这条出路只解一次。

可达性：要两件事（一趟被 KILL 留下记录，下一趟在摘的几毫秒里又被 KILL）。窗口窄，但下面两格不用第二次 KILL。

### 1.2 不 KILL 也留下 `.taking`：成立（打中，第一轮没看到的）

第一轮判决第 4 条修的是放记录那一次 link 回复丢了的情形，工具员只修在 `link_scratch_record` 里。摘旧记录那一次 `os.link(path, taking)` 走的是同一种 NFS link：服务端做成、回复丢了、重发报已有。代码在 `try` 外面接 `FileExistsError` 直接 `return RECORD_BEING_REMOVED`（第 2809–2810 行），`finally: os.unlink(taking)` 只罩第 2811 行起的那一段，`.taking` 留下，后果同 1.1 第 3 到 5 步。探针 `taking-link-reply-lost` 一臂造这一种（换掉被测模块的 `os.link`：真做成再抛 `FileExistsError`）。

### 1.3 TERM 也留下 `.taking`：成立（打中，第一轮没看到的）

真换先占草稿目录，`start_holding` 把 TERM 的处理换成 `release_then_die`；摘按仓那一份时收到 TERM（任务服务停任务、`proc.py stop` 先发的就是它），处理函数放掉草稿目录的记录，然后「signal.raise_signal(signal_number)」（`scripts/gate/hook-swap.py:2888`）按默认处理当场死，`remove_stale_record` 的 `finally` 跑不到。探针 `taking-left-by-term-with-scratch-held` 一臂：子进程占着草稿目录、停在摘按仓旧记录的当中，父进程对它的进程组发 TERM，看退出状态 -15、草稿目录记录放了、`.taking` 留下。

四句（1.1 到 1.3 一起）：

- 分不分辨臂：这一格不是在两个改法之间判，问的是硬链接摘法有没有可达的长挡。改名摘法（第一轮判决第 3 条的字面）没有 `.taking`，这一格不中；它有缝一（第一轮探针 `gap-three-contenders-on-stale-record`），所以这一格分得开两种摘法，各有一种坏法。
- 系统当时看不看得到判别它的东西：看得到一部分。`.taking` 与记录是同一个 inode，分不出是别人正在摘还是被杀留下的；`.taking` 的 ctime 是 link 那一刻，正在摘的不超过毫秒，留下的会越来越旧。
- 满足的是哪一个分句：第 1 问「`.taking` 被 KILL 留下之后谁来清、清之前所有趟都退 10」的字面；1.2、1.3 是同一个后果、不同的起因。
- 改法在打中的几格上还中不中：判决第 3 条的字面（改名摘走、核内容、放回）碰不到 `.taking`，但会把缝一带回来；见下面「改法表」。

### 1.4 两份锁的取放次序：不成立

取锁的地方只有三处：真换「held, busy_lines = hold_scratch(scratch, mode)」（`scripts/gate/hook-swap.py:2962`）之后「held, busy_lines = hold_repository(outcome_file, scratch, mode)」（`scripts/gate/hook-swap.py:2977`）；`--seal-outcome` 只取按仓的「held, busy_lines = hold_repository(path, None, mode)」（`scripts/gate/hook-swap.py:2609`）；`--dry-run` 只取草稿目录。没有一处先取按仓再取草稿目录。`acquire_record` 取不到当场交回拒绝说明，不等；所以「占着一份等另一份」本身不存在。试过的形状：两趟真换各占一个草稿目录同时取按仓（后一趟退 10）；真换与 `--seal-outcome` 交叉（`--seal-outcome` 不碰草稿目录）；非 git 根上按仓那一份落在草稿目录里（两份锁在同一个目录、次序不变）。

### 1.5 `--dry-run` 不占按仓的锁：不成立

`--dry-run` 也读结局文件「unfinished = pending_refusal(root, outcome_file, mode)」（`scripts/gate/hook-swap.py:2988`），不占按仓的锁时照样走这一行，读到的是那一刻的；真换在按仓的锁里重读一遍、重做第 1 步与第 4 步的基线核对。演练绿之后别人换上了，真换按此刻的内容拒（规格命中不到退 2、基线变了退 3、上一趟没完退 7 或 9），每一种拒都是此刻该拒的，不是误判；演练绿也没有被谁当成真换的凭据带过去。另一台起的真换正在跑时，演练读到开始行会退 7 并给「退回、补完」的出路，这是改动之前就有的，拒绝说明头一句已写「先看起它的那一趟还在不在跑」。

## 第 2 问：evidence-in-repo 与 `_lib_changed_paths_run_git`

### 2.1 E162、E174、E184 今天在真仓上判什么：成立（打中：背景里的说法错一半）

这一格的对象是改动范围里配得上两种形状的路径：「(re.compile(r"^scripts/experiment-apparatus/e([0-9]+)_[^/]*\.rs$"), "装置源码"),」（`.claude/gate.d/doc-experiments.sh:934`）与 `scripts/mutation/tables/e<号>_*.tsv`。

- 今天（`git status --short -- scripts/experiment-apparatus/ scripts/mutation/tables/` 量的）这三个实验的 `.rs` 与变异表都不在改动范围里，这一格对它们不判。改动范围里的是 `e184-segment-one-long-run.sh`，`.sh` 不是这一格的形状。
- E162：装置在 `scripts/experiment-apparatus/e162_*.rs`，复跑表两条 exact、三条 timing；`research/results/` 下今天一份 `e162` 起头的产物都没有（`ls` 量的）。哪一天改了装置，红行是「一份 E162 的产物都没有」，改动前后一样，第一轮的修补没改变它。
- E174：装置在 `scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs`，复跑表四条 exact、两条 timing；四条 exact 点名的产物都不在盘（在盘的是两段 timing 产物与 `simulated-second` 那一份），所以改动前也不走「按复跑表判」。装置的修改日期晚于在盘最新的产物（`stat` 量的），装置一进改动范围就红，改动前后一样。
- E184：装置是 `crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts/main.rs`，不配这一格的任何一种形状，改它这一格永远没有对象。「副作用是 E162、E174、E184 以后改装置而产物不比装置新就红」（`research/prompts/_ledger-open-fixes-governance-r2-body.md:21`）对 E184 不成立，对 E162、E174 是改动之前就有的。
- 射程更宽的一句（从未被看过的地方）：`crates/*/src/bin/` 下以 `e<号>_` 起头的装置有 23 份、属于 19 个实验（`ls crates/*/src/bin/ | grep -E '^e[0-9]+_'` 量的：142 156 158 161 165 168 170 171 173 175 180 181 182 183 184 185 197 198 202），改它们这一格都不判；同一轮新立的 pinned-apparatus-revision 格却认 `crates/*/src/bin/` 下的装置。两格对「装置在哪」的认法不一，没有登记为什么。

### 2.2 改 exact 那一段而产物不比装置新：出路走得通（不成立）

红行的出路是「→ 怎么办：跑一次把产物存进 {results_dir}/，并在 scripts/experiment-apparatus/replay.sh 把这个实验的登记行指到它。」（`.claude/gate.d/doc-experiments.sh:1143`）。按出路做完就绿的路径（以 E174 为例，推的）：只改 judge 那一段；照复跑表 `driver_e174_checks` 那一行起一次（不上卡），产物存成 `research/results/e174-lease-drop-wait-memory-return-checks-<今天>.out`，把那一行的产物名改成它。这一格取「这个实验在盘最新的那一份产物」比修改时刻，新存的这一份比装置新，绿。探针 `evidence_exact_rerun_probe.sh` 的 `exact-only-rerun` 样本照样本 `evidence-in-repo-mixed-replay-red` 起手、只多放一份更新的 exact 产物，预期退 0。

### 2.3 第一轮第 5 条的改法本身：成立（打中：改法碰不到打中的格）

第一轮打中的格是「同一个实验既有 exact 行又有 timing 行，改的是只出 timing 产物的那一段，按复跑表放过」。修补之后，同一段改动照 2.2 的路径只重跑那条便宜的 exact 驱动就绿：取的是「在盘最新的一份产物」，「product_form = re.compile(r"^e([0-9]+)(?![0-9])")」（`.claude/gate.d/doc-experiments.sh:937`）连文件种类与名字都不看。timing 产物没重跑、timing 那一段改了什么都没人比。探针里两个样本：`exact-only-rerun`（只多一份 exact 产物）、`unrelated-e915-file`（只多一份 `e915-notes-<日期>.txt`），预期都退 0；对照 `control-as-fixture` 预期退 1。

四句：

- 分不分辨臂：分。改动前（按实验号放过）与改动后（比在盘最新的一份）在 timing 那一格上都放过；改动后多出的只是「改 exact 那一段也要存一份新产物」的代价。
- 系统看不看得到判别它的东西：看不到。装置源码与复跑表的行没有机读的对应（这一格自己的注释也这样写），改的是哪一段，这一格分不出；它能看到的是每一行点名的产物名。
- 满足的是哪一个分句：`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 表里「改法碰不到打中的格」那一行。
- 改法还中不中：第一轮判决的改法（「同一个实验有 timing 行而装置改了，照旧比修改时刻」）在打中的那一格上照样中。能碰到它的是按行判：有 timing 行的实验，复跑表里每一条 timing 行点名的产物都要在盘、而且不比装置旧（exact 行照旧归逐字节复跑）；代价是改 exact 那一段也要重跑 timing（E174 要上卡），这是要主 agent 定的取舍，见「撤回第一轮攻方腿的方案」一节。

### 2.4 成功时照打 git 警告：不成立

调用方里会把成功时的警告打进格输出的：`doc-process-records.sh` 的 `load_change_range`、`doc-decisions.sh` 第 1716、1738 行、`doc-registries.sh` 第 4180、4864 行、`checker-independence-and-sync.sh` 第 1027 行、`doc-text.sh` 第 3162 行（这几处都在标准错误上，进格的输出）。样本档只比退出码与 `want=` 子串：「mapfile -t want_list < <(sed -n 's/^want=//p' "${CASE_SOURCES[$case_index]}/expect")」（`.claude/singlefs-ai-sop/scripts/stage-selftest.sh:317`），没有「不许出现」的写法（`grep -rhoE '^[a-z_-]+=' .claude/gate.d/fixtures/*/*/expect | sort | uniq -c` 量的：只有 `exit=`、`want=`、`not-green-by-design=` 三种键）。多出的行不会让一份样本判错。`want=` 写了 git 警告原文的只有 `batch-scope-git-warning-red` 一份（`want=diff.renameLimit`），那是有意的。

旁注（不算打中）：「#     git 成了而往标准错误写了东西（例：重命名探测超上限的警告）的，原样交到标准错误，与直接起 git 时一样（gate_added_lines 同样）。」（`scripts/gate/changed-paths.sh:21`）说的是共用库本身；`doc-experiments.sh` 把成功时的警告接进 `CHANGE_RANGE_ERROR_FILE`，只在取不到时打，成功的格输出里看不到。与 `doc-process-records.sh`、`doc-text.sh` 的做法不一，不影响结局。

## 第 3 问：完工清理闸「只判自己写的那几层」

### 3.1 怎么认「自己写的」：成立（打中，后果轻）

认法：「own_problems = problems if broken("handback-parent-declaration-judged") else declarations([*(root[0] for root in roots), *layers], role)[1]」（`.claude/hooks/completion-cleanup-guard.sh:1940`）。`roots` 来自 `subagent_roots`：这个子 agent 自己的会话记录里 claim 过的、工作根下一层、归属是本会话的目录，加派发提示「草稿目录：」点名的那一个（归属是本会话或没有归属）；每一个都配上它所在的工作根下一层，「roots.append((path, top, set()))」（`.claude/hooks/completion-cleanup-guard.sh:736`）。判的是「哪一层」，不是「谁写的那一行」：归属文件只记会话，不记 agent。

认错的输入（探针 `own_layers_probe.py`，用钩子自证的 `Scenario` 造现场、调 `decide`）：

1. 承接的子 agent 派到前一个用过的同一个草稿目录（或主 agent 在派发前往那个目录的 `.deliverables` 写过一行），那里有一行别人写坏的；承接的这一个什么都没写、一个副本都没留，交回被拒、点名那一行（`successor-blocked-by-line-it-did-not-write`）。出路在它手里：那个目录在它的写范围里，改对那一行再交回；后果是一次多拒，不是长挡。
2. 嵌套派发：共用约束写「子目录另派给别的子 agent 之后，父目录的持锁者不写那个子目录」（`.claude/agent-common.md:35`）。下层子 agent 在它那一层的 `.deliverables` 把交接副本声明了，自己交回放行（`nested-lower-handback-control`）；上层子 agent 交回时，下层那一份 `.deliverables` 在它的认领目录底下更深一层、不读，下层的交接副本照列、拦下上层（`nested-upper-blocked-by-lower-handover-copy`）。上层的出路：在自己那一层替下层写一行交接声明，或写「交主 agent 删」一行；前一条要写的正是共用约束说父目录持锁者不写的那个子目录里的东西（写的是父目录的 `.deliverables`，不碰子目录，严格说不违例）。
3. 同上，下层那一份 `.deliverables` 写坏一行：下层交回拦（对）、上层交回不读也不拦（`nested-lower-bad-line-seen-by-upper` 预期放行、不点名那一行）。这一种不是认错，是第一轮第 9 条「写在副本那一层的不读」的延伸，照文件头是设计内。

主 agent 与子 agent 写同一份 `.deliverables`：子 agent 交回只按层判，那一份在它的层里就判、谁写的都一样（同第 1 种）；主 agent 结束一轮（Stop）写坏的行不拦，只在单子里带「!」行。

四句：

- 分不分辨臂：这一问不在臂之间判；它问的是「自己写的」这句话有没有被实现成真。没有：实现的是「自己的层」。
- 系统看不看得到判别它的东西：看不到。`.deliverables` 里一行不带作者，钩子手里只有会话号与子 agent 号；要按作者判，得让写的人在行里或旁边留下自己的号。
- 满足的是哪一个分句：第一轮判决「改法」一节「只判交回的这一个 agent 自己写的那几层」里的「自己写的」；文件头改写成了「这个子 agent 自己写的那几层（它认领的目录、派发提示点名的草稿目录…）」，括号里的定义与括号外的话不是一回事。
- 改法还中不中：建议不改判法，改说法：文件头与判决里写成「这个子 agent 认领的与派发提示点名的那几层，不论那一行是谁写的」。第 2 种另写进嵌套派发的出路句。只推的，没实现。

## 第 4 问：删旧闸

### 4.1 复跑同名、内容会变的入库产物，换名或挪走之后登记对不对得上：不成立

四份入口写的输出路径来自参数，实验页给的起法写的是占位，例：E53 实验页第 58 行的起法 `env VM_DISKS=4 bash scripts/experiment-apparatus/e53-vm-bench-run.sh <输出文件>`；复跑表里虚机入口没有行（`grep -nE '^E(07|7|53|56|159)\|' scripts/experiment-apparatus/replay.sh` 只命中 E159 的 `anchors` 一行，那一行起的是装置本身，不经虚机入口）。所以：

- 换名：实验页引的是当时那一份的带日期名字，新跑的一份换名正是钩子 ⑤ 与本项目「按日期另存新文件名」的做法，旧引用照旧指到旧的那一份，对得上。
- 挪走：输出路径不变，新的一份落在原路径上；挪走的那一份若是已进 git 的旧产物，挪走本身是一次没登记的路径搬迁，归门禁 doc-experiments 的 cited-paths 一类格判，不是删旧闸的事。
- 反向那一支（不是问里的，记一笔）：已进 git、没改过的同名产物，删旧闸照删、新跑的整份写在同一个路径上。这一份是冻结证据，实验页引的那几行内容会变；判决第 11 条有意这样留（「进了 git 的照删」），找回靠 git，引文对不上要由引产物行的那几格判。没打中，只写出来。

### 4.2 删旧闸在边上判什么：成立（打中，第一轮没看到的）

删旧闸的判法：「if ! git -C "$output_directory" ls-files --error-unmatch -- "$output_name" > /dev/null 2>&1 \」（`scripts/experiment-apparatus/e07-vm-bench-run.sh:402`）与下一行 `git diff --quiet HEAD -- "$output_name"`，e53、e159、e56 同一段（各在第 425、422、396 行）。`--` 之后的词是 git 的 pathspec，通配照通配认。在仓里用只读命令量过通配那一半：

    cd research/results && git ls-files --error-unmatch -- 'e174-lease-drop-wait-memory-return-segment-[ot]*-2026-10-08.out'; echo rc=$?
    e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out
    e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out
    rc=0
    git diff --quiet HEAD -- 'e174-lease-drop-wait-memory-return-segment-[ot]*-2026-10-08.out'; echo rc=$?
    rc=0
    git ls-files --error-unmatch -- ':(literal)e174-lease-drop-wait-memory-return-segment-[ot]*-2026-10-08.out'; echo rc=$?
    error: pathspec ':(literal)e174-lease-drop-wait-memory-return-segment-[ot]*-2026-10-08.out' did not match any file(s) known to git
    Did you forget to 'git add'?
    rc=1

能照喂的输入：输出目录里已提交、没改过的 `e07-vm-bench-r2.out`；这一趟的输出文件名是 `e07-vm-bench-r[2].out`，它已存在而没进 git。两条 git 都按 pathspec 命中 `r2` 那一份、都退 0，删旧闸判「在 git 里、没改过」，`rm -f` 删掉的是没进 git 的 `r[2]` 那一份——这道闸要防的正是这一种。探针 `pathspec_delete_probe.sh` 对四份入口各喂一次 `r3`（对照：拒、退 2、文件留着）与 `r[2]`（预期：删了、走到假 cargo 那一步）。

四句：

- 分不分辨臂：分。改动前（一律删）删它，改动后照删；`:(literal)` 前缀或 `GIT_LITERAL_PATHSPECS=1` 的写法不删。
- 系统看不看得到判别它的东西：看得到，文件名就在手里，是把它交给 git 的方式错了。
- 满足的是哪一个分句：第 4 问「这一闸判什么」，以及入口自己文件头那一句「输出文件已存在而在 git 里没有……的不删、退 2」。
- 改法还中不中：判决第 11 条的字面（核进没进 git）在这一格上照样中，因为「核」用的就是这两条 git；加 `:(literal)` 之后不中。名字带 `[`、`]`、`*`、`?` 的输出不常见，后果是删掉一份找不回的产物。

### 4.3 `git` 不在 PATH、调用目录不在仓里：不成立

两种情形下两条 git 都退非 0（找不到命令、不是仓），落进「不在 git 里」那一支：拒、退 2、文件不动，说明写「不在 git 里」——缺 git 时说法不准，但方向是安全的。经任务服务跑时，文件头 `run-condition: command … git` 先判，缺 git 退 78，到不了这一段。试过的别的形状：输出目录在别的仓里（按那个仓判，删了在那个仓的历史里找得回）、文件已暂存未提交（`diff HEAD` 非 0，拒）、输出路径是指到仓外的符号链接（`rm -f` 只删链接本身）。

## 第 5 问：装置钉值的门禁格（零轮）

### 5.1 认起跑脚本、认装置、认登记：成立（打中）

- 认起跑脚本：「PIN = re.compile(r"^registered_apparatus_sha256=[0-9a-f]{64}[ \t]*$", re.M)」（`.claude/gate.d/doc-experiments.sh:4397`）。钉值行带行尾注释（`registered_apparatus_sha256=<64 位>  # 登记第十二节 R2-0`）、加引号、写成 `readonly …`、十六进制大写，整份起跑脚本不进对象、不报「认不出」。同一格认装置那一行明着放过行尾注释：「SOURCE_ASSIGNMENT = re.compile(r"^apparatus_source=([^\s#]+)[ \t]*(?:#.*)?$", re.M)」（`.claude/gate.d/doc-experiments.sh:4398`）。探针样本 `pin-with-trailing-comment`：装置改了、登记没动，预期退 0、成功句只数到另一份起跑脚本。
- 认装置：`apparatus_source=` 取整份脚本里头一处（`SOURCE_ASSIGNMENT.search`），不分是不是在自证函数、heredoc 里；写成带引号或带变量的，`os.path.isfile` 判不在树里、报红（报出来了，不算漏）。今天三份 e185 起跑脚本没有这种写法（`grep -n '^apparatus_source=' scripts/experiment-apparatus/e185-r*-segment-run.sh` 量的：r3、r4 各一处，r2 没有、走 `--bin`）。
- 认登记：注释行里头一处 `research/prompts/<…prereg…>.md`（`registry_of`：「if line.lstrip().startswith("#"):」（`.claude/gate.d/doc-experiments.sh:4437`））。重跑的起跑脚本头一句写「沿用 `research/prompts/e<号>-r1-prereg.md` 的几何」再写自己的登记，就认成 r1；r1 已归档时落进「登记已归档、不在树里不判」，自己那份第十二节没动也绿。探针样本 `registry-named-second`，预期退 0。

### 5.2 登记已归档就不判：成立（打中，比问的更宽）

- r2、r3 永远罩不到：「if not os.path.isfile(registry):」（`.claude/gate.d/doc-experiments.sh:4502`）之后只记「不判」。登记归档之后不会再回到树里（证据纪律：冻结的证据下一次提交删掉、要查去 git 历史），所以这两份起跑脚本从此对这一格是无对象。
- r4 也只是碰巧还在：归档脚本不删「还被代码当输入」的，判法只看非注释行，「只看非注释行：代码注释里写」（`scripts/kb/archive-past-rounds.py:236`，那一句接着说注释里写的路径是记录引用、不保）。起跑脚本点名登记的那一句是注释（这一格自己就规定从注释里认登记），保不住它；今天保住 `e185-r4-prereg.md` 的，看起来只有重判脚本的文档串「E185 第 4 次重跑重判段（research/prompts/e185-r4-prereg.md 第五节」（`scripts/experiment-apparatus/e185_r4_baseline_rejudge.py:4`，文档串不以 `#` 起头，被当成非注释行）。这是推的，主 agent 跑 `archive-past-rounds.py --check` 看它列不列 r4 那一份可以核。哪一天那份重判脚本改写或删掉，r4 也归档，这一格对全仓三份钉值的起跑脚本都只剩「不判」。
- 触发条件本身罩不住第一轮第 13 条：判据只在「装置源码在改动范围里」时起作用，「if source not in changed:」（`.claude/gate.d/doc-experiments.sh:4499`）之后记「装置没变不判」，文件头也写明「管不到：……起跑脚本的钉值有没有改成新值」（`.claude/gate.d/doc-experiments.sh:4377`）。第一轮第 13 条打中的恰是「装置在早先的提交里就改了，这一次只把起跑脚本的钉值改成现算值」，判决原话「照起跑脚本自己的出路，改钉值要连登记一起改，不该单改脚本」（`research/prompts/ledger-open-fixes-governance-r1-main-verification.md:40`）。今天的仓就停在这一格的起点：装置现算 `d312b9aa71d0…`（`sha256sum crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs` 量的），三份起跑脚本钉的是各自登记的值，r4 第十二节没有 `d312b9aa` 那一行（`grep -n 'd312b9aa' research/prompts/e185-r4-prereg.md` 零命中）；这一次只把 r4 的「registered_apparatus_sha256=9e813cbd36900f1b0c7001e6de0f98de01080bbf7ac6aab39d8e7c9854bd97f8」（`scripts/experiment-apparatus/e185-r4-segment-run.sh:94`）改成 `d312b9aa…`，这一格绿、开跑时那一道「if [[ "$apparatus_sha256" != "$registered_apparatus_sha256" ]] && ! e185_r4_break_is sha-unchecked; then」（`scripts/experiment-apparatus/e185-r4-segment-run.sh:485`）也过，停机 K-S6 又变成照跑。探针样本 `repin-only` 造这一种，预期退 0。

### 5.3 不是修订却放行的第十二节行：成立（打中）

判据是「added = nonblank(section) - nonblank(twelfth_section(previous) or [])」（`.claude/gate.d/doc-experiments.sh:4523`），文件头写「基准那一版第十二节里没有的非空行（逐行去首尾空白比，同一行写了几遍按次数算）」（`.claude/gate.d/doc-experiments.sh:4375`）。放行而不是新加修订的三种写法（探针各一个样本，预期都退 0）：

1. `inplace-rewrite`：把登记时写死装置 sha256 的那一行原地改成新值。比「没写修订」更坏：登记的原值被改掉，修订历史没了，而这一格判它「新加了修订」。
2. `internal-whitespace`：那一行词中间多一个空格（只改首尾空白的不放行，`strip` 抹掉了）。
3. `duplicated-line`：把第十二节已有的一行再抄一遍（「按次数算」让第二份算新加的）。

另：只改了空行、或只改首尾空白，不放行（判对的）；改第十二节标题那一行不算节里的行，也不放行。

四句（5.1 到 5.3）：

- 分不分辨臂：这一格是零轮新立的判据，问的是它判得对不对；5.2 第三条分得开「按装置变了触发」与「按钉值变了触发」两种写法，前一种不中第一轮第 13 条那一格，后一种中。
- 系统看不看得到判别它的东西：看得到。钉值行在不在改动里、新钉值在不在登记第十二节里、基准那一版第十二节的行有没有少，都在手里。
- 满足的是哪一个分句：第 5 问三个子句逐个；5.2 第三条另满足第 1 问「新立的门禁格判得对不对」。
- 改法还中不中：见下表。

## 改法表（都只在我的推导上、被攻过零轮；没有一格量过）

| 改法 | 修哪一格 | 量过还是推的 |
|---|---|---|
| `remove_stale_record` 报已有时看 `.taking` 的 ctime：比 `SCRATCH_LOCK_STALE_SECONDS` 旧的当留下的、删掉重判；新的照旧当别人在摘 | 1.1、1.2、1.3 | 推的，待主 agent 跑：要先在探针里做成会红的格（这一版下探针三臂 `taking_left=True` 之后照旧退 10） |
| 拒绝说明在 `.taking` 在时只写「删 `.taking`」，不写「300 秒没刷新就不再挡」 | 1.1 第 4 步说法 | 推的 |
| 删旧闸的两条 git 写 `:(literal)` 前缀（或设 `GIT_LITERAL_PATHSPECS=1`） | 4.2 | `git ls-files` 那一半量过（上面原样输出，在主工作区只读跑的）；入口整条推的，待主 agent 跑 |
| 钉值格多一个触发：起跑脚本的钉值行在改动里，新钉值要在登记（在树里的）第十二节新加的行里出现；登记已归档的判红（重跑要新登记） | 5.2 第三条、5.2 头两条 | 推的 |
| 判据收严：基准那一版第十二节的非空行一行不少（只许追加），新加的行里要有装置此刻的 sha256 | 5.3 三种写法 | 推的；与这一格文件头「管不到：新加的那一行有没有写对 sha256」相反，要主 agent 定 |
| 钉值行的正则放过行尾注释、认不出的钉值写法报「认不出」而不是不进对象；认登记取与脚本同名轮次的那一份（或改成要求行首 `registry=<路径>`） | 5.1 | 推的 |
| evidence-in-repo 对有 timing 行的实验按行判：每条 timing 行点名的产物在盘且不比装置旧 | 2.3 | 推的；代价是改 exact 那一段也要重跑 timing |
| 完工清理闸文件头与判决把「自己写的」改成「自己认领与派发点名的那几层，不论谁写的」 | 3.1 | 推的，只改说法 |

## 撤回第一轮攻方腿的方案

照「攻击腿撤回自己先前给的方案，比它打中新东西更有价值。」（`.claude/rules/three-way-inference.md:45`）：

1. **撤回「以后怎么拦」给的那一格的触发条件。** 第一轮交用户的是「装置改了而登记第十二节没补修订行时，提交就红的门禁格立不立」（`research/prompts/ledger-open-fixes-governance-r1-main-verification.md:74`）。照这个字面做出来的格（5.2 第三条）拦不住第一轮第 13 条那一种只改钉值的重钉，登记一归档就只剩「不判」。该拦的事件是「钉值变了」，不是「装置变了」：装置变了而钉值没变，开跑时那一道停机就拦住；钉值变了而登记没跟上，才是绕过停机的那一步。建议交用户时把题目改成「钉值改了而新钉值不在登记第十二节新加的行里，提交就红」，原来那一格要不要留另说。
2. **第一轮第 5 条的改法该撤或改成按行。** 2.3：照判决字面修完之后，timing 那一段改了照样能绿（只多一次便宜的 exact 重跑），修补换来的只是「改 exact 那一段也要存一份新产物」的代价。两条出路交主 agent：撤回到按实验号放（承认这一格只判形式，timing 段归复跑与评审），或照改法表按行判 timing 产物。我倾向前者，理由是按行判在 E174 这类上卡的实验上代价大、收益只是一道修改时刻的形式检查；没量，推的。
3. **第一轮第 3 条（改名摘走）不撤，硬链接也不撤，补 `.taking` 的清法。** 改名摘法有缝一（两个持有者同时在），硬链接摘法有 1.1 到 1.3 的长挡；前者坏的是互斥，后者坏的是可用性，按仓那一份护的是结局文件，可用性坏了比互斥坏了好收拾。所以工具员偏离判决字面那一处偏得对，只是缺了 `.taking` 的清法。

## 没打中的形状

- 第 1 问：两趟真换各占一个草稿目录交叉取按仓（后一趟退 10，对）；真换在 `--verify` 里起的命令再起 hook-swap（自证用临时仓，git-dir 不同）；linked worktree 当仓根（git-dir 是 `.git/worktrees/<名>`，各一份）；`--dry-run` 读到正在追加的结局文件半截（没有完成行就按没换完、进程在就退 9）；非 git 根上两份锁都落在草稿目录里（同一个仓根换两个草稿目录就各一份，这是改动之前就写明的「下一趟要用同一个 --scratch」，不算新缝；`minimal-copy.py` 建的副本都是 git 工作树）。
- 第 2 问：E201、E210 今天在改动范围里，有没有被这一格判错，没逐个推（命令见下面第 3 条，跑了就有）。
- 第 3 问：两个子 agent 同时派到同一个草稿目录（派发闸的写者锁先拦）；符号链接的认领目录（`subagent_roots` 跳过）。
- 第 4 问：输出目录被 `.gitignore` 忽略（`ls-files` 退非 0，拒）；仓里还没有 HEAD（`diff HEAD` 退非 0，拒）。
- 第 5 问：围栏里的「## 」标题（判法跳过围栏，对）；第十二节里只加了 `###` 小标题（算新加的非空行，放行；与 5.3 同一类，没单列）。

## 这条腿自己的限度

- 一个探针都没跑，表里每一个预期读数都是推的；四份探针都没被攻过。`taking_leftover_probe.py` 的子进程等标记文件最多 60 秒，机器很忙时那一臂会报 `child_reached_taking=False`，那一次作废。
- 第 1 问 1.2 的「回复丢了」是换掉 `os.link` 造的，真 NFS 上丢回复的频率没量。
- 第 5 问 5.2「r4 登记碰巧被文档串保住」是读 `still_an_input` 推的，没跑归档脚本。
- 没有建仓副本、没有编译目录；没读辩方腿那几问的文件（见「读到的范围」）。

## 要主 agent 跑的命令

1. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack -- python3 /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/probe/taking_leftover_probe.py /home/fy5090/code/singlefs/scripts/gate/hook-swap.py /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/taking-run`
   - 目录：草稿目录。看 `TASK_SERVER_RESULT` 的 `exit=0` 与日志：`arm=stale-record-without-taking-control held=True`；`arm=taking-left-by-kill-repository-lock` 里 `child_reached_taking=True taking_left=True next_swap_exit=10 seal_exit=10 refusal_promises_expiry=True after_record_removed_exit=0 next_killed_holder_then_swap_exit=10 after_taking_removed_exit=0` 算 1.1 打中；`arm=taking-left-by-term-with-scratch-held` 里 `child_exit=-15 scratch_record_released=True taking_left=True` 算 1.3 打中；`arm=taking-link-reply-lost` 里 `first_held=False taking_left=True plain_retry_held=False` 算 1.2 打中；末行 `probe-done arms=4`。
2. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/probe/evidence_exact_rerun_probe.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/evidence-run`
   - 目录：草稿目录。`sample=control-as-fixture exit=1`（与样本档同一个红）、`sample=exact-only-rerun exit=0`、`sample=unrelated-e915-file exit=0` 算 2.3 打中；末行 `probe-done samples=3`。
3. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-experiments.sh 点名的格 evidence-in-repo，在主工作区上; 范围=1 格; 为什么=第二轮攻方腿第 2 问：看 E162、E174、E184 今天在真仓上进不进这一格的判定' task-server/task submit -- bash .claude/gate.d/doc-experiments.sh --check evidence-in-repo`
   - 目录：仓根。看格输出里有没有 `E162`、`E174`、`E184`：一处都没有，2.1 第一条成立。
4. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack -- python3 /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/probe/own_layers_probe.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/own-layers-run`
   - 目录：草稿目录。`arm=successor-blocked-by-line-it-did-not-write action=block names_line=True`、`arm=nested-lower-handback-control action=pass`、`arm=nested-upper-blocked-by-lower-handover-copy action=block names_copy=True` 算 3.1 打中；`arm=nested-lower-bad-line-blocks-lower action=block names_line=True`、`arm=nested-lower-bad-line-seen-by-upper action=pass names_line=False` 是设计内的对照；末行 `probe-done arms=5`。
5. `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/probe/pathspec_delete_probe.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/pathspec-run`
   - 目录：仓根。每份入口两行：`arm=<入口>:e07-vm-bench-r3.out exit=2 untracked_output_kept=true refused_by_delete_gate=true reached_build=false` 是对照；`arm=<入口>:e07-vm-bench-r[2].out exit=2 untracked_output_kept=false refused_by_delete_gate=false reached_build=true` 算 4.2 打中；`committed_file_untouched=true`；末行 `probe-done arms=8`。
6. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/probe/pin_gate_probe.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r2-attack/pin-run`
   - 目录：草稿目录。六行 `sample=<名> exit=0`（`inplace-rewrite`、`internal-whitespace`、`duplicated-line`、`repin-only`、`pin-with-trailing-comment`、`registry-named-second`）各算 5.3、5.2、5.1 的那一格打中；`repin-only` 的成功句里有「装置没变不判 1 份」，`pin-with-trailing-comment` 的成功句写「起跑脚本 1 份」，`registry-named-second` 的成功句写「登记已归档、不在树里不判 1 份」；末行 `probe-done samples=6`。
7. `task-server/task submit -- python3 scripts/kb/archive-past-rounds.py --check`
   - 目录：仓根。看列出的待归档文件里有没有 `research/prompts/e185-r4-prereg.md`：没有，5.2 第二条「被别处保住」那一半成立；有，r4 下一次提交就归档，这一格对三份起跑脚本全部「不判」来得更早。

## 没做什么

- 不判辩方腿的第 6 到 9 问；不替主 agent 采纳；没往任务服务交任务，判决要引的数都没复跑（命令在上一节）。
- 没写模型目录、没有 `rerun.sh`（被判对象是治理文件，没有崩溃状态模型）。
- 草稿目录里留着：探针五份、两份 diff（`doc-experiments.diff`、`cleanup-guard.diff`，读代码用的）；没有仓副本与编译目录。
