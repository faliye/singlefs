# single-library-defs-r3 攻方腿（Opus）报告（2026-11-01）

攻击面：正文第三节 K2（站住的形态哪一格会错）与 K3（C948、C949 第三版）；不攻 K1；不重复前两轮攻过的角度（第一轮：开关没人设、E174/E178、崩溃三道只在本机、qemu.env、17 处旧句；第二轮：「判定」窄定义、中转库后门、调用方挑不了台、复用让第二台白跑、`--verify` 的 extra、「接完」没人宣布、旧库删后另起本地库、C948 第三列 18 行与样本、C949 服务不记台与 147 份）。
被判的四份文件开工时与 `research/prompts/single-library-defs-r3-snapshot/defs-sha256.txt` 逐份相同（本腿现算 sha256 核过）。派发提示点名的背景材料、附录、清单三份在 `research/prompts/` 下不在，只有正文 `_single-library-defs-r3-body.md`；本腿直接读被判文件与实现。
本腿开工之后别的会话改了 `.claude/kb/checks-owed.md`（C948、C949 从开工快照的第 605、606 行挪到第 604、605 行，本腿引到的句子仍逐字在）与 `task-server/task_server.py`、`scripts/hosts/multi-host-run.sh`、`crates/singlefs-checker-tier/src/crash_library.rs`（行号移了）；报告里的行号取交回前最后一次现查。
没写模型、没交任务。条数用 grep / python 在源文件上现数的标「量的」；从代码与注释推出的标「推的」。草稿目录 `~/.local/share/singlefs/work/single-library-defs-r3-attack/`（只有一份扫描脚本与它的输出，不是仓副本）。

## 各格判定一览

| 格 | 点 | 判定 | 一句话 |
|---|---|---|---|
| K2-1 | 宽定义拉进不该进的 | 打中 | 定义点名「用时」进库，同一份文件又许读按台的用时表与峰值表；占盘日志 target-occupancy.tsv 不按键存、只给派活用，却被列进「还没接完」要并进两台共用的库 |
| K2-1 | 宽定义罩到却没列的 | 打中 | `crates/singlefs-checker-tier/crash-full-run-backlog.tsv` 按行存「全绿 / 红：<签名>」与判法键、下一趟照它取件，按定义（窄的也算）是判定库内容，不在九样名单、不在 C948 前置 |
| K2-1 | （本腿加）许读三处「按台各一份」 | 打中 | 三处里峰值表与上次红的清单在多机配置登记的共享根 `~/.local/share/singlefs` 下，两台读写同一份 |
| K2-2 | 三件的次序 | 打中 | 条款不定次序；② 在 mutation-units 族过不了：timings/ 这类永远不红的行两条路各写过一次就成冲突，导入不写、`--verify` 的「字节不同」永不为 0 |
| K2-2 | ③「旧库目录删掉」 | 打中 | admission 的旧库目录 `singlefs-unit-verdicts/` 里住着 replay-row 与 flow-lines 两族（还没接完、导入不导）；③ 删目录与第 75 行「按导了的那几样删」相反 |
| K2-2 | 接完之后不带开关 | 打中 | task-server 只凭开关放第二台：接完的族照条款不带开关，就永远只派本机；第 77 行「开关不再有意义」不成立 |
| K2-2 | 过渡期混族的门禁 | 打中 | 「接完」登在登记表第三列，而第三列一行一份脚本；第 22、23 行各跨两族，一族接完另一族没接完时这一格写不出 |
| K2-3 | 全部现判的代价 | 打中 | 崩溃这一族的「全部现判」就是 `--full`，钩子按重型拒；新写一条崩溃用例要「写完」就得在提交之外跑一趟重型（全量两台 6–9 小时，推的） |
| K2-3 | 判不出的 | 打中 | 共享根上的旧库与状态两台读到同一份；第二台那一趟不读库，规则举的头一个例子（家目录下的旧判定库）在这一趟里根本不被读；判本机自己的门禁结局本该不同，被误判 |
| K2-3 | 缩成代表子集 | 部分（推的） | 随机或按类挑的代表子集会漏；缩成「本机那一趟现判过的项」对代码引入的依赖不漏，对共享根与 `--full` 那两类同样判不出 |
| K2-4 | 清单外的本机状态 | 打中 | `~/.config/singlefs/out-of-project-names` 与本机家目录列表改门禁 doc-text 的结局，第二台拿到的多机配置拷贝角色不同，都不在许读清单、不在只派一台的表；`.claude/heavy-thread-default` 进仓、两台相同，不算 |
| K2-4 | 欠账与「不许新增」 | 打中 | 新的只能一台跑的活没有合法去处：规则叫登表、表头不许新增条目、不登就会被派到第二台；表里此刻就有判定库服务化这一批登的两条 |
| K3 | C948 | 打中（立不成） | 负样本「只写产物缓存的不红」与宽定义相反；九样漏 backlog 表、`crash-amplification/` 下六份没人认领的旧库与两份改名的 layout-5（24G、14G）；「迁完删掉之后还在判红」按路径字面，改名就躲过 |
| K3 | C949 动态格 | 打中（立不成） | 第二台那一趟的凭据只在本机 task-server 的 sqlite 里（不在共享根，`clear-queue` 整表删），不绑被提交的那一版；`host=peer` 只在第二台的树还留着时打，`task clean` 之后就没了；这一格自己就读本机独有状态 |
| K3 | C949 静态格 | 打中 | 照新词新射程量出命中 142 份；只命中共享根 `~/.local/share/singlefs` 的 56 份、只命中 `~/.config/singlefs` 的 10 份、只命中许读三处的 3 份按规则合规，放行条款一样都不放，会被登进「只缩不涨」的滞后表而永远缩不掉 |

## 一、K2 第一条：宽定义「复用要读的一切」

被判条款：`.claude/rules/verification.md:72`：「结论（绿、红、拒算）与它的组成清单、判定向量、录入块、事实表、清单、覆盖、用时、流行、片结果都算」；同一节 `.claude/rules/verification.md:74`：「crates 变异的 target-occupancy.tsv 与 timings/」；另一节 `.claude/rules/verification.md:83`：「只影响排程的本机状态许读、按台各一份，就这几处」。

### 1.1 按定义进库却不该进的：排程用的用时与占盘 —— 打中

- 「用时」被点名进库。变异的用时表是派活次序用的，源码自己写另一台的快慢不代表本机：`scripts/mutation/crates-mutation-rows.py:127`：「import 不导：另一台的快慢不代表本机」。可导入脚本照导（`scripts/prune-server/prune-server-import.py:22`：「records/、baselines/、coverage/、compositions/、index/、timings/、units/」），开关开着时工作进程把它写进服务（`scripts/mutation/crates-mutation-rows.py:2066` 的 `timings_redirecting_writer(library, context["timings_directory"])`）。条款把它算判定库内容、并进两台共用的那一份，结局是第二台按本机的用时排活。
- 同一类东西条款在第 83 行许读、按台各一份：`run-durations.tsv`（用时表，键同峰值表）与 `memory-peaks.tsv`。定义只看「按键存下、下一趟要读」，分不出这两样与变异 timings/。照第 72 行它们也该进库，照第 83 行它们该留本机：同一份文件两条给相反的话。
- 第三份用时与峰值在 task-server 的库里：`task-server/control:248`：「history（用时与峰值记录）」。它在 `~/.local/state/singlefs/task-server/` 下，许读清单没有、定义罩得到。
- target-occupancy.tsv 不是按键存的：`scripts/mutation/crates-mutation-rows.py:449`：「追加进判定库目录下的 target-occupancy.tsv」，同一行写一行是「<时刻秒><TAB>普通|插桩<TAB><字节>」，下一行写派活前取最近 20 次最大的算磁盘余量。它是本机编译目录占盘的时序日志，用途与峰值表相同（排程），按第 72 行自己的「按单元键或行键存下」它不算，第 74 行却把它列进要并进服务的「还没接完的族」；并进两台共用库之后，第二台按本机那块盘的占盘涨幅算自己的磁盘余量（同一段注释记着第二台 1.8T 的盘写满过，推的：两台的涨幅不同）。
- 推翻条件：条款给「复用」划界（只算省掉现判的结论及其组成，排程用的用时、峰值、占盘不算），或第 83 行把变异 timings/ 与 target-occupancy.tsv 也列进许读的按台状态。

### 1.2 按定义该进库、却不在名单里：崩溃全量积压登记表 —— 打中

- `crates/singlefs-checker-tier/crash-full-run-backlog.tsv:8`：「进度（」，同一行接着写四种值（未跑、部分已判块、全绿、红带签名）；第 7 行是判法键；第 9 行写判法键对不上的行当未跑读、游标取今天判法键下不是全绿的头一行。读写由二进制做：`crates/singlefs-checker-tier/src/bin/crash_full_run_backlog.rs` 的 `write_the_backlog_file`（第 105 行起，`std::fs::write` 再 `rename`）。
- 这是一张按行键（项名加判法键）存绿红、给下一趟照它取件（跑哪件、跳哪件）的 TSV：窄定义也罩得住。它不在第 74 行的九样里，也不在 C948 前置里；C948 射程含 `crates/`，判法「写 TSV 存判定的调用不经 prune-server 客户端判红」，格一立这一份当天就红。
- 反过来它的位置是里程碑出口定的：`.claude/kb/milestone/04-fourth-txn.md:25`：「表放在 checker 档 crate 里」；模块文档同样写 `crates/singlefs-checker-tier/src/crash_full_run_backlog.rs:2`：「表是本包根上的 `crash-full-run-backlog.tsv`」。要么定义太宽（把进仓的登记表拉进 RocksDB，与里程碑出口相反），要么名单漏了一样。
- 同形的还有进仓的按行登记的滞后表（`.claude/gate.d/` 下 13 份 `*-lag.tsv`，量的）与按内容哈希认的豁免表 `.claude/crates-review-exempt`：都是按行键存下、门禁下一趟要读、决定红不红。C949 自己的存量办法就是再加一张滞后表（第五节）。定义没把进仓、人写的登记表与机器写的判定缓存分开。
- 推翻条件：条款写明进仓的登记表（积压表、滞后表、豁免表）不算判定库内容，或九样名单补上积压表并给出它进服务后里程碑出口怎么改。

### 1.3 （本腿加）许读的三处，两处不是「按台各一份」—— 打中

- 多机配置 `NFS_SHARED_ROOTS` 的值（量的，读仓根 `multi-host.env` 那一行，值换成说法）：`~/.local/share/singlefs` 整个与会话记录目录。峰值表与上次红的清单都在它下面：`scripts/hosts/memory_peaks_lib.py:330`：「worktree 与第二台的副本里跑的都读写本机那一份」。本机上这个目录是本地盘（`findmnt -T` 报 ext4），第二台照「两台挂在同一绝对路径上」挂过来（第二台那一侧本腿没连上核）。
- 所以第 83 行的「按台各一份」对三处里的两处不成立；只有 `~/.local/state/singlefs/run-durations.tsv` 不在共享根下。这一句是描述还是要求，第二轮攻方 3.3 问过；这里打中的是它作为描述与配置对不上。
- 同一个事实还碰到第 83 行举的头一个反例「本机家目录下的旧判定库」：崩溃库缺省目录 `~/.local/share/singlefs/crash-amplification/crash-state-verdicts`、广谱变异 `~/.local/share/singlefs/broad-mutation-units` 都在共享根下，不是一台独有（见 3.2）。
- 推翻条件：第二台那一侧没挂 `~/.local/share/singlefs`，或峰值表改成按台分文件。

## 二、K2 第二条：一族「接完」三件与登记表第三列

被判条款：`.claude/rules/verification.md:73`：「③ 旧库目录删掉」，同一行 ① 删本地路径不留开关、② 最后一次导入后立刻 `--verify` 只判缺与异；`.claude/rules/verification.md:75`：「按导了的那几样删」；`.claude/rules/verification.md:77`：「某一族接完之后这个开关对它不再有意义」。

### 2.1 ② 在 mutation-units 族过不了：永远不红的行成冲突 —— 打中

- 导入遇到同键不同：`scripts/prune-server/prune-server-import.py:27`：「别的不同（两份都不红、或服务里的是红）不写，记一条冲突」；红的认法只认 keys/ 与 records/（第 139–155 行 `is_red`），timings/、coverage/、index/、units/、baselines/、compositions/ 永远不红。冲突时服务那一份留着（第 284 行「冲突 … 条（服务里那一份留着）」），本地那一份也不动。`--verify` 照数「字节不同」：`scripts/prune-server/prune-server-import.py:14`：「后三个都是 0 退 0」。
- 历史：过渡期带开关的任务改服务里的 timings/<行>，不带开关的（提交时照技能跑的六道，第二轮 4.1 说过技能里没写开关）改本地的同一把键；两边各被现判过一次，值就不同（用时是秒数）。这之后每一次导入都记冲突、不写，`--verify` 的 different 永远大于 0，② 永远过不了，接完永远做不到；条款第 73 行把「只判缺与异」当成修好了第二轮 4.2，「异」这一半照旧卡住（量的：代码路径；推的：两条路都改过同一行的频度）。
- 推翻条件：导入对永远不红的种类照服务那一份为准、`--verify` 不数这几类的「异」，或条款写明迁移窗口内不带开关的任务一律不跑。

### 2.2 ③ 删目录会删掉还没接完的两族 —— 打中

- admission 一族的旧库目录是 `<库根>/singlefs-unit-verdicts/`（登记表 checker-tier-research-build-and-replay、checker-tier-crash-point-replay、code-tooling、harness-model-differential-and-scenarios 四行）。同一个目录里住着另外两族：`.claude/gate.d/verdict-library-scripts.tsv:21` 的「<库根的 git common-dir>/singlefs-unit-verdicts/replay-row/」，与各种类目录下的 flow-lines/（`scripts/prune-server/prune-server-import.py:21`：「（同一种类目录下 crash-flow-coverage.py 的 flow-lines/ 没换存法，不导）」）。两样都在第 74 行「还没接完」的名单里、导入不导。
- ③ 字面是「旧库目录删掉」：照做就删掉 replay-row 与 flow-lines 的全部记录（没进服务，丢了）；不照做 admission 就接不完。第 75 行说「按导了的那几样删」，与 ③ 对同一件事给两种范围。
- 同形：崩溃库的 layout-5 那一份（正文第二节：不迁）按第 75 行「按导了的那几样删」永远不删，与同一行开头的「不并存」相反（磁盘上 `crash-state-verdicts.layout-5-2026-11-01` 24G、`mutation-replay.layout-5-2026-11-01` 14G，量的，`du -sh`）。
- 推翻条件：③ 改成「导了的那几样删掉」并写明不导的去向，或 admission 接完排在 replay-row 与 flow-lines 之后。

### 2.3 三件不定次序 —— 打中（写法缺口）

- 第 73 行只说「三件都做到」，没说先后。「最后一次导进」只在 ① 生效之后才是最后一次；照常见的迁移次序先导、核、删旧库再改代码（②③①），③ 与 ① 之间不带开关的任务就重建旧库并写进去（第二轮 4.4 的形状），这批判定再也不会导。即使先做 ①，① 之前起的任务、仓副本与第二台树里的旧代码照旧写本地；条款没写要等它们结束。
- 导入脚本取库根靠被 ① 删的那一层：`scripts/prune-server/prune-server-import.py:8`：「库根照各家的取法」，同一行点名 `lib_verdict_library.py` 的 `library_root`（推的：① 若连 `library_root` 一起删，② 跑不起来）。
- 推翻条件：条款写明 ①→②→③ 的次序、① 生效前起的任务都结束才做 ②。

### 2.4 接完之后不带开关：task-server 只派本机 —— 打中

- 派不派第二台：`task-server/task_server.py:2113`：「self.peer_routes[key] = completed.returncode == 0 or (via_service and registered in VERDICT_SERVICE_RELEASED)」，`via_service` 就是命令带没带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`。问路那一步把第三列不以两种前缀起头的一律当带库、退 3：`scripts/hosts/multi-host-run.sh:858`：「NOT_LIBRARY_PREFIXES = ("不算：", "无判定库：")」。第三列改成「prune-server」照旧退 3。
- 接完之后条款说开关「不再有意义」，① 又要求不留开关；照条款交的不带开关的任务，task-server 判成带库、不走服务，只派本机，永远「第二台未跑」，与多机那一节直接相撞。主 agent 定义仍叫交任务带开关（`.claude/main-agent.md:12`：「交带判定库的任务带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`」），两份对接完之后带不带说法不一。
- 推翻条件：task-server 或问路那一步把第三列写「prune-server」的行当成走服务放行，或条款写明接完之后开关照带。

### 2.5 过渡期一道门禁跨几族 —— 打中

- 「接完」登在第三列，第三列一行一份脚本。`.claude/gate.d/verdict-library-scripts.tsv:22`（门禁 checker-tier-research-build-and-replay）同时有 admission 那几格与 experiment-replay 格（replay-row）；`.claude/gate.d/verdict-library-scripts.tsv:23`（`scripts/crash/crash-point-replay.sh`）同时有放量部分（崩溃库）与快档（admission）。admission 接完、另一族没接完时，这两行第三列写「prune-server」就把没接完的那一半也宣布成接完，不写就让 C948「第三列不是 prune-server 判红」那一格一直认不出 admission 已接完。
- 一条任务只有一个开关值：同一趟里接完的那族不认开关、没接完的那族照开关走，不带开关时一趟里一半写服务、一半写旧库；task-server 照 2.4 只派本机。code-tooling 只有 research-script-selftests 一格带库（登记表第 27 行，admission），本身不跨族；它起的各份自证跨族，属第二轮 2.3 的形状，不重复。
- 推翻条件：第三列改成按族登（一行一族，或一格写几族各自的状态）。

## 三、K2 第三条：「第二台那一趟全部现判、结局逐项相同」

被判条款：`.claude/rules/verification.md:85`：「这一趟不读判定库、全部现判」；C949 写出了做法：`.claude/kb/checks-owed.md:605`：「崩溃点重放入口带 `--full`、两道门禁设 `SINGLEFS_GATE_FULL=1`、cargo test 不经库」。

### 3.1 代价：崩溃这一族的「全部现判」是重型 —— 打中

- 钩子把崩溃入口 `--quick` 加 `--full` 判成重型：`.claude/hooks/lib_heavy_tests.py:6130`：「重型：崩溃点重放入口 --quick 又带 --full」；checker 档包的测试本身也是重型。重型只在提交时或用户要求时跑：`.claude/agent-common.md:64`：「只在提交代码时跑一次、或用户要求时跑」。
- 所以新写或改过的崩溃用例、checker 档测试，要照第 85 行「写完」，就得在开发途中跑一趟重型；不跑就只能一直挂「第二台未跑」到提交。另两道（形式化、变异）的 `SINGLEFS_GATE_FULL=1` 写在 `--quick` 上钩子不看（`.claude/hooks/lib_heavy_tests.py:733`：「罩不住的：环境变量改轻型的范围（SINGLEFS_GATE_FULL」），那两道照轻型交得出去，而且这是钩子自己列的漏洞，不是规定。三道里同一个判据一道是重型、两道靠漏洞算轻型。
- 一趟多久（全是推的或别处量的）：崩溃全量两台一趟，`records/2026-10-13-代码欠账批.md:183`：「它推的 6–9 小时」（别的会话估的）。本机用时表里带复用的轻型最近一趟（量的，`~/.local/state/singlefs/run-durations.tsv`，每把键只留最后一次）：`crash-point-replay.sh --quick` 905 秒、`checker-tier-formal-verification.sh --quick` 1803 秒、`checker-tier-crates-mutation-replay.sh --quick --item …` 最长 3764 秒；不读库的那一趟只会更长，长多少要看复用的份额，本腿没量。每改一次测试就要再来一趟。
- 推翻条件：钩子许崩溃入口带 `--item` 的 `--full` 算轻型，或条款写明第二台那一趟只现判点名的项、且那种跑法照轻型交。

### 3.2 全部现判也判不出的 —— 打中

1. 共享根上的东西：`~/.local/share/singlefs` 整个是共享根（1.3），崩溃库与广谱变异的旧库、峰值表、上次红的清单、`qemu/`、`formal-tools/`、`rocksdb-prebuilt/` 都在它下面（量的，`ls`）。读它们的测试两台读到同一份，结局相同，判据放行；换一台没挂共享根的机器才红。
2. 不读库的那一趟碰不到「依赖库」：第 83 行举的头一个反例是「本机家目录下的旧判定库」。依赖它的只有复用那条路（读库、跳过），第二台那一趟按第 85 行偏偏不读库，这条依赖在这一趟里根本不出现。这是「判别子观测不到」：要分的两种情形（依赖本机库 / 不依赖），第二台那一趟能看到的输出逐条相同。
3. 两台各装了一份的本机状态：rocksdb 预编库每台各编一份（`scripts/hosts/rocksdb-prebuilt.py:4`：「librocksdb-sys 的 C++（librocksdb.a 与 libsnappy.a）每台主机只编一次」），环境取自 `~/.config/singlefs/rocksdb-prebuilt.env` 这类本机文件；两台都装了就相同，判据只证明「这两台」，不证明「不依赖一台」。
4. 两台都缺资源而都报本次未跑的：例 doc-text 的 private-values 在没有多机配置时退 77（`.claude/gate.d/doc-text.sh:118`：「没有 multi-host.env（别人的克隆、第二台）」）。两台都 77 算不算「现判的结局相同」，条款没写。
5. 反方向的误判：判的就是这一台的门禁，结局本来该不同。doc-text 的 out-of-project-names 读本机家目录的条目名（`.claude/gate.d/doc-text.sh:147`：「名字从哪来（门禁跑时现读，名字不进仓）：本机家目录（$HOME）下一层的条目名」），harness-test-environment 的 machine-clean 判本机的 loop、dm、挂载与临时目录残留，`scripts/hosts/gpu-card-health.py` 按角色定查哪几张卡（起活那一台查两台的、角色是第二台只查第二台的，文件头「哪一台查哪几张」）。两台结局不同不说明它们错了，判据却判它「依赖一台」；条款里给这类的去处只有只派一台的表，而表不许新增（4.2）。
- 推翻条件：第二台那一趟另加一趟读库的对照，或规则把「判本台」的门禁另列一类、判据写「两台各自判对」而不是「结局相同」。

### 3.3 能不能缩成「第二台现判一个代表子集」—— 部分（推的）

- 随机或按类挑的代表子集会漏：依赖一台是逐项的（一条用例读了一份本机文件），没被挑中的那一项照漏。
- 缩成「本机那一趟现判过的项」（单元键变了的那些）不漏：单元键由判定依赖的函数单元闭包组成（`.claude/rules/verification.md:51` 那一句列的组成），代码改动引入的本机读法一定落在闭包里，闭包变了的项本机那一趟一定现判；键没变的项上一次已经过了这一关（规则立之前的存量除外）。对上面 1–3 那三类它与「全部现判」一样判不出；闭包外的输入（`NOT_JUDGING_FILES`、`EXCLUDED_PROGRAM_EDGES`、多机配置）引入的依赖，两种都漏。代价从「整道全部」降到「本机现判的那几项」。没在任何装置上量过。
- 推翻条件：找到一项依赖一台、单元键没变、而这一次改动让它的结局变了的例子。

## 四、K2 第四条：许读清单之外的本机状态；只派一台的表

被判条款：`.claude/rules/verification.md:83` 的「不读会改变判定结局的本机独有状态」与同一行「环境相关的一切只由多机配置 `multi-host.env` 定」；`.claude/rules/verification.md:84`：「登记进 `task-server/task_server.py` 文件头点名的那几张只派一台的表」。

### 4.1 清单外改结局的本机状态 —— 打中

- `~/.config/singlefs/out-of-project-names`：门禁 doc-text 的 out-of-project-names 格读它（`.claude/gate.d/doc-text.sh:148`：「第二台的同类条目名在私有名单 ~/.config/singlefs/out-of-project-names」），连同本机家目录与仓上一级目录的条目名（3.2 第 5 条）。同一份新加的行，本机判红、第二台判绿或本次未查。私有值只许放本机是共用约束定的（`.claude/agent-common.md:54`：「真实值只在仓根被 git 忽略的 `multi-host.env` 与 `~/.config/singlefs/` 里」），判它的门禁只能读本机；doc-text 不在只派一台的表里（`LOCAL_RESOURCE_ENTRIES` 只登了 doc-registries.sh），门禁目录派得到第二台。两条核心约束在这一格上互相顶着。
- 多机配置本身：第二台那棵树拿到的是拷贝、角色不同（`task-server/task_server.py:222`：「双机那一套在第二台那棵树里拿到的是多机配置的拷贝、角色是第二台」）。读角色、读点名的卡、读双机开关的测试在两台天然结局不同，`scripts/crash/crash-point-replay-shard-configuration-check.sh` 就是因此登进只派一台的表的。「环境只由多机配置定」与「两台结局相同」对多机配置自己不能同时成立。（doc-text 第 118 行又写第二台没有多机配置，与 task-server 第 221 行说法不一，哪一句对本腿没连第二台核。）
- 不从多机配置来的环境：`~/.config/singlefs/rocksdb-prebuilt.env`、`run-with-memory-cap.env`（量的，`ls ~/.config/singlefs/`）。rocksdb 那一份的 `entry` 判这台没装就退 1、入口一个 cargo 都不起（`rocksdb-prebuilt.py` 文件头 `entry` 那一段），没装的那台门禁全红；条款的「环境相关的一切只由多机配置定」在今天的实现里不成立。
- task-server 的状态目录：`~/.local/state/singlefs/task-server/`（不在共享根），只给排程与 C949 动态格用（第五节）。它的 sqlite 里另有一份用时与峰值（1.1），许读清单没列。
- `.claude/heavy-thread-default`：进仓（`git ls-files` 认得），第二台那棵树里同一份，文件头写双机时每台各取这一个数；不是本机独有状态，不算打中。
- 推翻条件：许读清单与只派一台的表补上这几样并写明「判本台」一类的判法，或这些门禁改成照参数读一份给定的私有名单、两台各自判。

### 4.2 「每条都是欠账」与「不许新增」—— 打中

- 规则第 84 行叫只能一台跑的活登表；表头：`task-server/task_server.py:221`：「不许新增条目）」。一件新的只能一台跑的活（新的量某一台硬件的装置、新的装本机服务的工具）三条路都不通：登表违表头，不登就被派到第二台假红或装错台，又不许算测试。两句能同时成立的唯一读法是「从此不许再有只能一台跑的活」，规则没这样写。
- 表此刻就有这一批登的：`task-server/task_server.py:255`：「"prune-server/control": "装与起停本机的用户级 systemd 单元」，上面一行注释写「管判定库服务化的会话点名的三条」。`task_server.py` 不在 HEAD 里（`git diff HEAD --stat` 报整份新增），表头与这两条谁先谁后从 git 看不出（推的：表头没拦住新增）。
- 表里还登着门禁与测试：`.claude/gate.d/doc-registries.sh`、`scripts/crash/crash-point-replay-shard-configuration-check.sh`（`LOCAL_RESOURCE_ENTRIES`）。规则说登表的「不算测试」，可这两样就是门禁与带自证的测试入口，分类与「所有测试都要多机可用，没有例外」相撞。
- 推翻条件：规则写明新的只能一台跑的活怎么办（例：弹窗交用户、登表时同一次改动带去掉它的路的期限），并把表里的门禁另记成测试的欠账。

## 五、K3：C948、C949 第三版

### 5.1 C948 —— 打中，立不成

欠账原文：`.claude/kb/checks-owed.md:604`：「一份只写产物缓存的（不红）」；同一行「迁完删掉之后还在就判红」；前置同一行列九样、键前缀、`CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 非 0 判红、判法另认「一个目录存判定文件」与「旧库删后目录被重建」。

1. 负样本与它要守的规则相反：宽定义下，按键存下、下一趟复用要读的产物缓存就是判定库内容（`.claude/rules/verification.md:72`），应当转红；样本却定它不红。照样本实现的判法放过的正是宽定义新罩进来的那一类。（两份样本分不出两种实现是第二轮 5.1 第 7 条，那一条判决「采」而这一行样本没改，属 K1，不重复。）
2. 新加的两种判法没有样本：「目录存判定文件」与「旧目录被重建」各一份正样本、一份负样本都没有，判别力为零。
3. 九样漏的（量的）：积压登记表 `crates/singlefs-checker-tier/crash-full-run-backlog.tsv`（1.2，射程含 `crates/`，格一立就红）；`~/.local/share/singlefs/crash-amplification/` 下六份旧库 `amplification-unit-tests-library`（40M）、`crash-state-enumeration-library`（331M）、`facts-pairing-library`（3.3M）、`given-states-library`（11M，另有一份 `.polluted-by-gpu-fault-tests-2026-10-11`）、`judge-tables-pairing-library`（168M）、`library`（272M），`crates/`、`scripts/`、`.claude/gate.d/` 里没有一处代码点名它们（`grep -rln` 零行），没人写、没人删、没人认；两份改名的 layout-5（24G、14G）。
4. 「迁完删掉之后还在就判红」按路径字面：同一行点名的旧库只有 `.git` 下的 `singlefs-*` 与 `crash-state-verdicts`。删兼容那一批正是把旧库改名成 `crash-state-verdicts.layout-5-2026-11-01` 留着的，改个名就躲过这条判法，也躲过第 75 行「按导了的那几样删」（不导的不删），与「不并存」相反。
5. 服务模式下崩溃库照旧在旧库旁边写：`crates/singlefs-checker-tier/src/crash_library.rs:1562`：「配置里的库目录只当名字、并库锁与每进程库的落点」。此刻 `crash-amplification/` 下就有 `crash-state-verdicts.libraries-of-processes/` 与 `crash-state-verdicts.merge-lock`（量的，`ls -a`）。删掉旧库之后每一趟走服务的照建上层目录、每进程库与锁；「旧目录被重建」要分得出旧库本身与这两样兄弟，「目录存判定文件」要放过 `.libraries-of-processes/`，欠账两样都没写。
6. 前置写「全接进」，规则定义的是「接完」三件：只接进（开关开着走服务）不做 ③，旧库目录还在，第 4 条那一格当场红；前置该写「接完」。
- 没打中的一处：`CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 那一行两种开库都打（同一份文件第 1529、1578 行），今天没有入口读它（`scripts/`、`.claude/gate.d/` 里 grep 零行），前置要做的「数它、非 0 判红」做得出。
- 推翻条件：负样本改成「只写进仓产物、不复用」，两种新判法各补一对样本，前置补积压表与六份旧库并写明改名的旧库怎么判。

### 5.2 C949 动态格 —— 打中，立不成

欠账原文：`.claude/kb/checks-owed.md:605`：「新写或改过的测试与自证每条由主 agent 经服务交过一趟」，同一行写 `task wait` 末行 `host=peer` 且结局相同才绿、不靠服务记台。

1. 凭据住哪：task-server 的库在 `${XDG_STATE_HOME:-~/.local/state}/singlefs/task-server`（`task-server/task_server_lib.py:18`），不在共享根，第二台读不到。门禁的一格要判「每条新测试有一趟 host=peer」，只能读这份本机库：这一格自己就在读「会改变判定结局的本机独有状态」，被派到第二台跑就没有凭据。
2. 凭据会整表没：`task-server/control:17`：「删 tasks 与 attempts 的全部行」；此刻状态目录里有三份名字带 before-clear 的库备份（量的，`ls`），清过不止一次。
3. `host=peer` 留不住：`task-server/task:11`：「最后一次在第二台跑、那棵树留着的再接 host=peer」（第 212 行 `if kept_peer_tree(last):` 才拼这一段）。共用约束要求取完就 `task clean` 删那棵树，删了之后再问 `task wait` 末行就没有 `host=peer`。
4. 不绑被提交的那一版：任务记录按号、命令、工作目录存，没有树或内容的指纹（`task_server.py` 里 grep 不到树哈希一类的字段，推的）。测试先在第二台过一趟、之后又改，旧那一趟照算。单元键那一套的「键相同才复用」在这里没有对应物。
5. 「逐项相同」要两趟每一项的结局：第二台写的文件留在第二台的树、`task clean` 删；剩下的只有本机 `logs/` 下的尝试日志（2438 份，量的），逐项比要解析日志，欠账没定格式。
6. 谁交：主 agent 交，第二台那一趟由服务挑台（第二轮已攻挑台）；这一格只能事后认，认的凭据就是上面 1–5。结论：动态格是 review 的步骤，不是门禁的一格；要立成格，得先有一份两台都读得到、按内容指纹记「第二台现判过、结局逐项」的记录。
- 推翻条件：task-server 把每一趟的树指纹、`host`、逐项结局写进共享根下（或 prune-server 里）一份不随 `clear-queue` 与 `clean` 消失的记录。

### 5.3 C949 静态格存量怎么分 —— 打中

- 照欠账的词（`~/`、`$HOME`、`${HOME`、`var_os("HOME")`、`expanduser`、`/home/<用户>/`；主机名与第二台路径是私有值，本腿没扫）与射程（git 跟踪的 `scripts/`、`.claude/gate.d/`、`crates/*/src/`、`crates/*/tests/`）现扫（量的，脚本与逐份分类在草稿目录 `scan/c949_scan.py`、`scan/c949-hits.tsv`）：

| 量 | 份数 |
|---|---|
| 射程里的文本文件 | 7110 |
| 命中 | 142（其中门禁样本 `.claude/gate.d/fixtures/` 下 52） |
| 只命中共享根 `~/.local/share/singlefs` 下的路径（工作根 19、其余 37） | 56 |
| 只命中 `~/.claude/` 下的路径（2 份是共享根 `~/.claude/projects`，1 份是样本的 expect） | 3 |
| 只命中 `~/.config/singlefs` | 10 |
| 只命中许读三处（峰值表、用时表、上次红的清单） | 3 |
| 只命中旧库路径（崩溃库、广谱变异库） | 5 |
| 只命中 `~/.cargo` / `~/.rustup` | 2 |
| 只命中 task-server / prune-server 状态 | 1 |
| 只命中认不出路径的 / 几类混着的 | 23 / 39 |

- 分法按行粗分（一行里先认哪个路径），没逐份读。照规则合规的至少 69 份（56 + 10 + 3，会话记录那 2 份另算）：共享根下的路径照共用约束原样带到第二台，许读三处是规则许的，`~/.config/singlefs` 是私有值唯一许放的地方（4.1）。欠账的放行只有「登记在 task-server 只在本机的表里的入口与多机配置模板」（`.claude/kb/checks-owed.md:605`：「登记在 task-server 只在本机的表里的入口与多机配置模板放行」），这 69 份一份都放不了，只能登进「只缩不涨」的滞后表，而它们永远不必改，表永远缩不到零，滞后表里欠账与合规混在一起。
- 同一张滞后表按 1.2 也是一张按行存「红但放过」的 TSV。
- 推翻条件：静态格的放行补上共享根（照 `NFS_SHARED_ROOTS` 现读）、许读三处与 `~/.config/singlefs`，滞后表只登剩下的。

## 六、打中之后的四句

这一题没有并列的臂，「分不分辨臂」改问「打中的是被判条款本身，还是各条款共用的前提」。

| 打中 | 打中的是哪 | 被判的系统当时看不看得到 | 满足的是字面哪一句 | 前两轮的改法在这一格上还中不中 |
|---|---|---|---|---|
| 1.1 用时、占盘 | 第 72 行定义与第 83 行许读清单互相顶 | 看得到：两样都在条款点名的文件里 | 第 72 行「用时」、第 74 行「target-occupancy.tsv」对第 83 行「按台各一份」 | 第二轮把定义放宽成「复用要读的一切」，这一格是放宽造出来的 |
| 1.2 积压表 | 第 72、74 行与 C948 前置 | 看得到：进仓的 TSV | 第 72 行「结论（绿、红、拒算）」 | 第二轮补名单没补到它 |
| 1.3 共享根 | 第 83 行的描述 | 看得到：`NFS_SHARED_ROOTS` | 第 83 行「按台各一份」 | 第二轮列全路径时没对配置 |
| 2.1 冲突卡 ② | 第 73 行 ② | 看得到：导入与核对的末行 | 第 73 行 ② 的「字节不同」 | 第二轮去掉 extra，不碰 different |
| 2.2 删目录 | 第 73 行 ③ 对第 75 行 | 看得到：登记表第三列 | 第 73 行「旧库目录删掉」 | 不中 |
| 2.4 不带开关 | 第 73 行 ①、第 77 行与 task-server | 看得到：`task-server/task_server.py:2113` | 第 77 行「不再有意义」 | 第二轮「接完」三件造出来的 |
| 3.1 重型 | 第 85 行与重型的时机 | 看得到：钩子 | 第 85 行「全部现判」 | 第二轮加「全部现判」造出来的 |
| 3.2 判不出 | 第 85 行的判别力 | 看不到：第二台那一趟不读库，要分的两种情形输出相同（判别子观测不到） | 第 83 行「本机家目录下的旧判定库」 | 同上 |
| 4.1 清单外状态 | 第 83 行对共用约束私有值那一条 | 看得到 | 第 83 行「不读会改变判定结局的本机独有状态」 | 不中 |
| 4.2 不许新增 | 第 84 行对表头 | 看得到 | 第 84 行「登记进」 | 第二轮 3.4 攻过、改法只改成「指到文件头」，冲突没去 |
| 5.1–5.3 | C948、C949 第三版 | 看得到 | 见各条 | 第二轮改法各自落了，新冒出的这几格不中 |

## 七、交主 agent 的：每条打中对应的改法

都只在本腿读代码推过、被攻过零轮（第三轮之后不再开轮，照规则写进交用户表、标「零轮」）。

| 打中 | 改法 | 量过 / 推的 |
|---|---|---|
| 1.1 | 第 72 行给「复用」划界：省掉现判的结论及其组成算，排程用的用时、峰值、占盘不算；第 74 行删 target-occupancy.tsv 与变异 timings/，挪进第 83 行的许读清单（或改成按台分的文件） | 推的 |
| 1.2 | 第 72 行写明进仓的登记表（积压表、滞后表、豁免表）不算判定库内容；或九样补上积压表并改里程碑四出口 10 的「表放在 checker 档 crate 里」 | 推的 |
| 1.3 | 第 83 行删「按台各一份」，改写三处各在哪（两处在共享根、两台同一份） | 推的 |
| 2.1 | 导入对永远不红的种类照服务为准；`--verify` 只比 keys/、records/，或数「异」时扣掉服务那一份更新的 | 推的，待主 agent 定后由实现员改 `prune-server-import.py` 并加自证格 |
| 2.2 | ③ 改成「导了的那几样删」，与第 75 行同一种说法；admission 的接完排在 replay-row、flow-lines 之后；layout-5 两份写明去向（不迁就删，或登记留到哪天） | 推的 |
| 2.3 | 第 73 行写次序 ①→②→③，① 生效前起的任务（含仓副本、第二台的树）都结束才做 ② | 推的 |
| 2.4 | task-server 的问路把第三列写「prune-server」的行当成走服务放行；或第 77 行改成接完之后照带开关、主 agent 定义同步 | 推的 |
| 2.5 | 第三列改成按族登，或「接完」另起一张按族的表 | 推的 |
| 3.1 | 第 85 行写第二台那一趟只现判本机那一趟现判过的项（`--item` 点名），并让钩子把这种 `--full --item` 照轻型放；否则写明新写的崩溃用例「写完」推到提交那一趟 | 推的 |
| 3.2 | 第 85 行加一趟读库的对照，或「判本台」的门禁另列一类、判据写「两台各自判对」；共享根上的依赖另记欠账（判据换台也判不出） | 推的 |
| 4.1 | 许读清单补 `~/.config/singlefs/` 的私有名单与多机配置的第二台拷贝，写明只许私有值类门禁读；`rocksdb-prebuilt.env` 这类环境并进多机配置或在第 83 行写明例外 | 推的 |
| 4.2 | 第 84 行写新的只能一台跑的活怎么办（弹窗交用户、登表时带去掉它的期限），表里的门禁另记成测试的欠账 | 推的 |
| 5.1 | 负样本改成「只写进仓产物、不复用」；两种新判法各补一对样本；前置补积压表、六份旧库与改名旧库的判法；「接进」改「接完」 | 推的 |
| 5.2 | 动态格先降成 review 步骤；要立格先让 task-server 把树指纹、`host`、逐项结局记进两台都读得到、不随清队列与 `clean` 消失的地方 | 推的 |
| 5.3 | 静态格放行补共享根（现读 `NFS_SHARED_ROOTS`）、许读三处、`~/.config/singlefs`，滞后表只登剩下的约 73 份 | 推的（份数是粗分） |

## 没打中的形状

- 随机种子让两趟结局不同：`crates/*/tests`、`crates/*/src` 里取熵的随机数（`thread_rng`、`from_entropy`、`OsRng`、`rand::random`）0 份（量的）；`scripts/` 两层里用 python `random` 的 2 份没逐份看。
- 挂钟断言让两台结局不同：`.claude/gate.d/wallclock-assert-lag.tsv` 0 行（量的）。
- `.claude/heavy-thread-default` 算本机状态：进仓、两台同一份（4.1）。
- 上次红的清单改结局：只给 `run-scope-guard` 的拒绝说明读（`scripts/gate/lib_reuse_checks.py` 文件头「三」），不进任何一道的判定。
- code-tooling 一道跨族：带库的只有 research-script-selftests 一格。
- `CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 立不起来：两种开库都打（5.1）。
- 取样：只看了被判四份文件、登记表、导入脚本、task-server 两份、崩溃库、变异与 doc-text、harness-test-environment 的文件头与相关函数；没读 prune-server 的 Rust 源码、admission 的写库路径、E174/E178/E182 装置。一次抽样，「没打中」不支撑结论。

## 这条腿自己的限度

- 没连第二台：共享根在第二台那一侧挂没挂、第二台有没有多机配置的拷贝（doc-text 第 118 行与 task-server 第 221 行说法不一）都没核。
- 用时全部是推的或别的会话估的；用时表每把键只留最后一次，不读库的那一趟没有读数。
- 5.3 的分类按行首认路径的粗分，主机名与第二台路径没扫。
- 2.1 的冲突是从代码路径推的，没造一段「两条路各写一次 timings/ 再导入」的历史跑一遍。

## 要主 agent 跑的命令

无。本腿没写模型、没改装置；判决要引的数（2.1 的冲突）要坐实的话，由主 agent 另派实现员在导入脚本的自证里加一格「timings/ 两边各写一次之后导入、`--verify`」再交服务。

## 没做什么

- 不判 K1；不替主 agent 采纳；没往统一任务服务交任务。
- 没核第二台；没读禁读清单里的三份文件。
- 草稿目录 `~/.local/share/singlefs/work/single-library-defs-r3-attack/` 只有 `scan/` 下的扫描脚本与两份输出（不是仓副本、不是编译目录、不是库拷贝），留着给主 agent 复核 5.3。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/single-library-defs-r3-opus-output.md --background research/prompts/_single-library-defs-r3-body.md` 末行（交回前最后一次，对着此刻的工作区）：

```
  ✓ 核了 40 处引文、9 处裸行号，对上 40 处，没判 9 处（其中裸行号没有引文可比 9 处；逐处列在上面）
```
