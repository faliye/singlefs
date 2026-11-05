# single-library-defs-r2 攻方腿（Opus）报告（2026-11-01）

攻击面：正文第三节 K2（改后的形态哪一格会错）与 K3（C948、C949 新写法立得成立不成）；不攻 K1；不重复第一轮攻过的角度（开关没人设、E174/E178、崩溃三道只在本机、qemu.env、读本机运行状态、17 处旧句）。
被判文件开工时与 `research/prompts/single-library-defs-r2-snapshot/defs-sha256.txt`、`kb-sha256.txt` 逐份相同（本腿现算 sha256 核过）。
没写模型、没交任务；条数是本腿在源文件上用 grep / python 现数的（标「量的」），由代码与注释推出的标「推的」。
本腿开工之后 `task-server/task_server.py`、`task-server/task_server_lib.py` 被别的会话改过（同一次 grep 前后行号变了、`LOCAL_ONLY_DIRECTORIES` 由四个目录变成一个）；引这两份的行号取交回前最后一次现查。

## 各格判定一览

| 格 | 点 | 判定 | 一句话 |
|---|---|---|---|
| K2-1 | 「判定」定义罩不罩得住各家的东西 | 打中 | 定义按「结论」收窄，崩溃库二十来张表里多数（录入、事实表、清单、分片进度）、admission 的 items/ 与 flow-lines/、变异的 coverage/ timings/ index/ target-occupancy.tsv 都不算判定；而 replay-row、E182 片结果库按定义是判定，却既不走服务、也不在条款列的「还没接进的族」里 |
| K2-1 | （本腿加）开关开着照写旧库目录 | 打中 | admission 开关开着时 directory 照算，crash-flow-coverage 照它往旧库目录建 flow-lines/；变异照旧往旧库目录追加 target-occupancy.tsv；replay.sh 不经客户端 |
| K2-1 | （本腿加）撤回第一轮攻方 1.3 的「没打中」 | 撤回 | 第一轮说 replay.sh 的库开关开着就进服务；导入脚本与 replay.sh 都写明 replay-row 没换存法 |
| K2-2 | 每进程库「并回即删」谁判、没并回怎么红 | 打中 | 并不进去的每进程库照留，打的那一行全仓没有脚本读，不红 |
| K2-2 | 变异复跑的崩溃库 | 打中 | 放行第二台的变异门禁在家目录下每片建一份跨轮留着、按键复用、从不并回的崩溃判定库；开关开着时又把变异版的判定推进那一份 |
| K2-2 | 自证与压测的库怎么与真服务分开 | 打中 | 压测与开着开关跑的自证写的是真服务里的命名空间，服务没有删的操作，「格完即删」做不到；分法只看库目录在不在临时根下 |
| K2-2 | （本腿加）第二台片库并进的不是服务 | 打中 | E182 的第二台片库取回后并进本机 `.git` 下的文件库 |
| K2-3 | 「没派到第二台不算违例」会不会空转 | 打中 | 派哪台由服务按余量挑、调用方挑不了；没派到的只记 C949，提交不受阻；派到了也多半是复用本机判过的那一条，第二台什么都没判 |
| K2-3 | 「只影响排程的本机状态许读」边界谁判 | 打中 | 没有人判；条款点名的两样（峰值表、上次红的清单）就在 `~/.local/share/singlefs/` 下，C949 静态格照字面判红；变异的 timings/ 是排程用的却进了两台共用的那一份 |
| K2-3 | （本腿加）条款抄了服务表的现状 | 打中 | 条款写四个只在本机的目录，表今天只剩 `scripts/hosts`，且表头新写「不许新增条目」，与条款「登记进那几张表」相反 |
| K2-4 | 过渡期写旧库与「迁完之前只读不写」 | 打中 | 同一节两句直接冲突；`--verify` 要求两边逐条相同，服务一开用就多出来、旧库一被写就少，「迁完」只有冻结两边才达得到 |
| K2-4 | 「接完」谁宣布 | 打中 | 条款没写；唯一的机器名单是 task-server 的「已接」名单，名单里的门禁带着开关照写旧库目录；条款说的「各记欠账」在欠账表里一行都没有 |
| K3 | C948 | 打中（立不成） | 第三列今天 18 行全是本地路径，「过渡」与「中转库按文件头登记」都没有写法；静态判法只认 RocksDB / TSV / sqlite，漏掉「一目录存判定文件」这一类（replay-row、E182、flow-lines）；旧库删后正当代码会重建旧目录；两份样本分不出「只看注释」与「真认写法」 |
| K3 | C949 | 打中（立不成） | 服务按键只存值，没有台；复用不写、Rust 并库逐字节相同不写，服务看不到第二台判过；按值加台撞并库与导入的逐字节规矩；静态格照它自己的词与射程量出 147 份，射程不含 `crates/*/src`，漏掉崩溃库的家目录缺省 |

## 一、K2 第一条：「判定」的新定义罩不罩得住

被判条款：`.claude/rules/verification.md:72`：「判定指按单元键存下、给下一趟同键复用的结论（判绿、判红、拒算的记录，连同它的组成清单与判定向量）」；同一行：「还没接进服务的族（广谱变异的 TSV、E174 与 E178 装置自己的库）各记欠账」。

### 1.1 逐样核（罩住 / 没罩住）

| 家 | 东西 | 按定义 | 今天存哪、走不走服务 | 依据 |
|---|---|---|---|---|
| admission | keys/ | 罩住（判绿判红的记录） | 开关开着进服务 | `scripts/gate/admission.py:1441`：「directory 照旧算（只当这份库的名字与命名空间），本地目录不建不读不写」 |
| admission | compositions/ | 罩住（组成清单） | 同上，导入脚本导 | `scripts/prune-server/prune-server-import.py:20`：「底下除 replay-row（replay.sh 自己的那一份，没换存法）之外每个种类的 keys/、items/、compositions/」 |
| admission | items/ | 没罩住：按名字的 sha256 存、不供复用 | 进服务（被当成判定库的一部分导） | `scripts/gate/admission.py:52`：「items/<名字的 sha256>.json 是这个名字最近一次判绿的那一份，只用来在键对不上时比组成清单、报出变了的单元。」 |
| admission | flow-lines/ | 两可：按组成键存、整份复用时取回，但存的是覆盖行与计数行，不是绿红 | 不经客户端，开关开着也写旧库目录；导入不导 | `scripts/crash/crash-flow-coverage.py:32`：「这一趟整份现判（一条复用都没有）的，打出来的覆盖行与计数行照组成键存进判定库」；`scripts/prune-server/prune-server-import.py:21`：「（同一种类目录下 crash-flow-coverage.py 的 flow-lines/ 没换存法，不导）」 |
| admission 族目录 | replay-row/ | 罩住（行键、记 drift / broken 撤绿） | 不经客户端，开关开着也写旧库；导入不导 | `scripts/experiment-apparatus/replay.sh:3446`：「REPLAY_ROW_LIBRARY="$replay_common_directory/singlefs-unit-verdicts/replay-row"」；导入那一行同上第 20 行 |
| admission 族目录 | e182-single-device/ | 罩住（按复用键存的片结果，续跑复用） | 装置自己写文件，不经客户端，登记表没有这一行 | `scripts/experiment-apparatus/e182-single-device-long-run.sh:6`：「片结果按复用键原子存进片结果库」 |
| crates 变异 | records/、baselines/ | 罩住 | 进服务 | `scripts/mutation/lib_mutation_units.py:65`：「（服务没有删的操作），交空的计数」（同一段讲走服务） |
| crates 变异 | coverage/ | 没罩住：覆盖记录，定哪条测试去判，不是结论 | 进服务 | `scripts/mutation/lib_mutation_units.py:56`：「coverage/<号>.json.gz   覆盖记录：一条测试执行过的单元（crates 变异表用）」 |
| crates 变异 | timings/ | 没罩住：源码自己写「不是判定」 | 进服务（两台共用一份） | `scripts/mutation/lib_mutation_units.py:60`：「timings/<六段的 sha256>  上一次现跑的用时（派活次序用，不是判定）」 |
| crates 变异 | index/ | 没罩住：只说现判的原因 | 进服务 | `scripts/mutation/lib_mutation_units.py:59`：「这一行上一次的键与几样组成的号：只用来说清」 |
| crates 变异 | target-occupancy.tsv | 没罩住：磁盘预算的读数 | 开关开着也追加进旧库目录；导入不导 | `scripts/mutation/crates-mutation-rows.py:2715`：「descriptor = os.open(target_occupancy_file(library_directory), os.O_WRONLY」（同一行带 `O_CREAT`）；`scripts/prune-server/prune-server-import.py:23`：「库目录顶层的文件（target-occupancy.tsv）不导」 |
| 崩溃库 | 结论块、结论违例、判定向量 | 罩住 | 同一份 RocksDB（Rust 接服务还没编过） | `crates/singlefs-checker-tier/src/verdict_store.rs:1346`：「/// 结论块：键是块键 + 结论种类 + 版本，值是每个状态一个判定向量编号。」 |
| 崩溃库 | 录入块（流程、清单、观察者基底） | 没罩住：按录入键存、命中跳过，是复用的但不是结论 | 同上 | `crates/singlefs-checker-tier/src/verdict_store.rs:1361`：「/// 录入段一趟的清单：键是录入键」 |
| 崩溃库 | 事实表、核对表、GPU 判表 | 没罩住：中间产物 | 同上 | `crates/singlefs-checker-tier/src/verdict_store.rs:1336`：「/// 三段流第 ① 段的事实表（`crash_facts::FlowFacts`）」 |
| 崩溃库 | 分片进度 | 没罩住：自己写明只是加速 | 同上 | `crates/singlefs-checker-tier/src/verdict_store.rs:1368`：「只是加速，真相以块为准」 |

### 1.2 打中在哪

- 定义括号里只列「判绿、判红、拒算的记录」与组成清单、判定向量。照这个读，上表「没罩住」的十来样都不归条款管：条款第三条只禁「新建自己的判定库」，存非判定的东西另开 RocksDB 或目录不算违例。崩溃库二十来张表里多数是录入与中间产物（`verdict_store.rs` 第 1324 行起那张表的逐张注释），用户定的「CPU 产出的数据一律进库」管的正是它们，定义却把它们排在外面。
- 反过来，按定义是判定、条款却当它已接进或干脆没提的有两家（量的：`ls`、`du`）：
  - replay-row：`.git/singlefs-unit-verdicts/replay-row` 277M，最近一次写在 2026-10-29；登记表第三列写的就是它（`.claude/gate.d/verdict-library-scripts.tsv` 中 `scripts/experiment-apparatus/replay.sh` 那一行）。第一轮判决把它与 flow-lines 归进「还没接进服务的族各记欠账」（`research/prompts/single-library-defs-r1-main-verification.md:40`：「replay-row 与 flow-lines：规则不点名」），可条款的括号只列广谱变异、E174、E178。
  - E182 片结果库：`.git/singlefs-unit-verdicts/e182-single-device` 1078 个 `.shard`、4.4M；登记表 43 行里没有 E182 的行（量的：`grep -n e182 .claude/gate.d/verdict-library-scripts.tsv` 零行），装置路径也不在登记表的现扫射程里（`.claude/gate.d/verdict-library-scripts.tsv:11`：「# scan: .claude/gate.d/*.sh scripts/crash/*.sh」那一行只点名 `replay.sh` 与 `device-code-key.py` 两份装置脚本）。
- 条款说的「各记欠账」没有落：`grep -n 'prune-server' .claude/kb/checks-owed.md` 只命中 C948、C949 两行，没有广谱变异、E174、E178 各自接进服务的欠账行（量的）。
- 条款第二条要「三家的文件库经 `scripts/prune-server/prune-server-import.py` 整份导进去」（`.claude/rules/verification.md:73`：「三家的文件库经 `scripts/prune-server/prune-server-import.py` 整份导进去」），可导入脚本明写不导 replay-row、flow-lines、target-occupancy.tsv（上表第 20、21、23 行），`--verify` 只比导了的前缀，照样整份相同；主 agent 照条款「整份相同之后删旧库」删 `.git/singlefs-unit-verdicts`，277M 的 replay-row 判定连同 E182 片结果一起没了。条款没写「旧库」按目录还是按导了的那几样删。
- 照条款执行会出错的场景：主 agent 照 `.claude/main-agent.md:12`：「交带判定库的任务带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`」交 checker-tier-research-build-and-replay（在 task-server 的已接名单里）。experiment-replay 格起 replay.sh，replay-row 照旧写 `<库根>/.git/singlefs-unit-verdicts/replay-row`：开关开着，旧库照写，撞 `.claude/rules/verification.md:73`：「迁完之前旧库只读不写」。派到第二台时（这道门禁在放行名单里，第二台那棵树带 git 历史）写进那棵树的 `.git`，随树删，判定没并回任何一处（推的：按 `replay.sh` 第 3438–3446 行的库根取法与 `task-server/task_server_lib.py:163`：「第二台那棵树带本机的 git 历史与暂存区」推，没在第二台起过）。
- 推翻条件：定义改成罩住「按键存、给下一趟复用的一切」，或条款明写非判定的东西放哪；replay-row、E182 进了服务或进了条款的欠账名单与欠账表。

### 1.3 （本腿加）开关开着照写旧库目录 —— 打中

- admission 开关开着时 `directory` 照算（上表第一行引文），`scripts/crash/crash-flow-coverage.py:292`：「return os.path.join(directory, FLOW_LINES_DIRECTORY) if directory else None」，存时 `scripts/crash/crash-flow-coverage.py:317`：「os.makedirs(directory, exist_ok=True)」。开着开关跑 checker-tier-crash-point-replay 快档，flow-lines 照建进 `.git/singlefs-unit-verdicts/<种类>/flow-lines/`（推的：按代码路径读，本腿没跑；核它的命令在「要主 agent 跑的命令」）。
- 变异门禁同形：`scripts/mutation/crates-mutation-rows.py:449`：「追加进判定库目录下的 target-occupancy.tsv」，`os.open(…, O_CREAT)` 不看开关（量的：`.git/singlefs-crates-mutation-library/target-occupancy.tsv` 710 行，今天还在写）。
- 结果：条款第 75 行（`.claude/rules/verification.md:75`：「没带的走旧文件库，接完之前算过渡，接完之后算违例。」）反过来读就是带开关不碰旧库，实际带着开关旧库目录照被写、删了还会被重建（连到 K3 C948 第 6 条）。
- 推翻条件：crash-flow-coverage 与 target-occupancy 换了存处，或开关开着时 `directory` 交 None。

### 1.4 撤回第一轮攻方 1.3 的「没打中」

第一轮攻方 `research/prompts/single-library-defs-r1-opus-output.md:58`：「它的库是 unit-verdicts 族」，同一行说开关开着就进服务。这是错的：导入脚本第 20 行写明 replay-row 没换存法，`replay.sh` 全文没有一处 `prune`（量的：`grep -n prune scripts/experiment-apparatus/replay.sh` 零行），库读写是 bash 的 `mkdir`、`mv`、`cat`（`scripts/experiment-apparatus/replay.sh:3467`：「mkdir -p -- "$REPLAY_ROW_LIBRARY/keys" "$REPLAY_ROW_LIBRARY/items"」）。第一轮判决据此把它挂到「各记欠账」，而欠账没记（1.2）。

## 二、K2 第二条：「中转库不算另开」会不会成后门

被判条款：`.claude/rules/verification.md:74`：「中转库不算另开：每进程库、分片的片库、第二台一趟的片库这类只在并回服务那一份之前存在、并回即删的库；自证与压测给临时服务起的库只在那一格里活、格完即删。」；`.claude/agent-common.md:39`：「并回即删的中转库与自证临时服务的库不算；旧库迁完之前只读不写」。

### 2.1 每进程库并不进去就一直留着，没人判 —— 打中

- 崩溃库自己写明并不进去的照留：`crates/singlefs-checker-tier/src/crash_library.rs:27`：「开库收养完打一行 [`LIBRARIES_LEFT_UNMERGED_LINE_PREFIX`]（几份并不进去留着，」，`crates/singlefs-checker-tier/src/crash_library.rs:29`：「并不进去打一行 [`MERGE_REFUSED_LINE_PREFIX`] 或」。并不进去的条件是「两份都不红而判定不同」，也就是两边判得不一样，正是最该看的那一种。
- 谁读这几行：量的，`grep -rln 'LIBRARIES_LEFT_UNMERGED\|MERGE_REFUSED\|ADOPTION_REFUSED'` 在仓里（不含 `research/`、`target`）只命中 `crash_library.rs` 自己、`src/bin/crash_library_reader.rs`（手动 `adopt` 子命令，有并不进去的退 1）与三份 records；`scripts/` 与 `.claude/gate.d/` 下零处。没有一道门禁或入口把「留着没并」当红。
- 照条款执行会出错的场景：两条用例在同一块上判出不同的非红结论（判器改了、GPU 与 CPU 不一致），后开的进程并库被拒，它那份库留在 `<库目录>.libraries-of-processes/` 下；之后每次开库收养再被拒、再留着。条款说这类库「并回即删」才不算另开，它永远不并回，按条款它就是另开的一份 RocksDB；可没有任何东西变红，审 diff 也看不到（它不是代码改动，是运行时留下的）。C948 写的「中转库……按文件头登记放行」（`.claude/kb/checks-owed.md:605`：「中转库（并回即删）与自证临时服务的库按文件头登记放行」）按文件头放行，放行的是写法，不看这一份到底并没并回。
- 推翻条件：某道门禁或入口数 `CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 行并在非 0 时判红或本次未跑。

### 2.2 变异复跑的崩溃库：跨轮留着、从不并回 —— 打中

- `scripts/mutation/crates-mutation-rows.py:511`：「走判定库的 checker 档用例在变异版上判出的结论不写进真跑的长期库」；`scripts/mutation/crates-mutation-rows.py:513`：「没设是 ~/.local/share/singlefs/crash-amplification）底下的 mutation-replay/，跨轮留着、照用例自己的键复用」。门禁文件头同一段还写每个工作进程在它底下再分 `shard-<片号>` 一份（`.claude/gate.d/checker-tier-crates-mutation-replay.sh` 第 76–84 行）。
- 按条款：这是按块键存、跨轮复用的判定（罩住），落在家目录下、一片一份 RocksDB、设计上就不并回。不是「并回即删」的中转库，不是临时服务的库，也不在第 72 行「还没接进服务的族」的括号里。条款字面判它违例，而它所在的门禁正在 task-server 的已接名单里（`VERDICT_SERVICE_RELEASED` 含 `.claude/gate.d/checker-tier-crates-mutation-replay.sh`，本腿现读）。第一轮 1.5、2.9 攻的是「每片一份、第二台一片在树里」（中转那一面），没攻这几份跨轮留着、不并回的长期库。
- 开关开着更糟：Rust 那一侧换了存处之后，`crates/singlefs-checker-tier/src/verdict_store.rs:2403`：「每个进程照旧写自己那一份本机库，读先看自己那一份、没有再问服务」，下一行写收尾时推进服务；开关只看环境变量（`crash_library.rs` 的 `shared_library_backing_of_this_process`），不看库目录。`mutation_crash_library_environment` 只换家目录、清 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`，不清开关（量的：`grep -n VERDICT_LIBRARY_SERVICE scripts/mutation/crates-mutation-rows.py` 只命中第 1751 行一处文档）。照 `.claude/main-agent.md:12` 带开关交这道门禁，变异版判出的结论就推进那一份唯一的库，正是门禁文件头要防的（推的：Rust 接服务的补丁还没编过，按源码注释推）。
- 两头都中：不带开关违「只有一份」，带开关违门禁自己的「变异版结论不进长期库」；条款没写变异复跑这种「故意不进长期库」的判定放哪。
- 推翻条件：条款把变异复跑的崩溃库写成例外或欠账，或 `mutation_crash_library_environment` 清掉开关并改成并回即删。

### 2.3 自证与压测写的是真服务，服务删不了 —— 打中

- 协议没有删：`scripts/prune-server/prune_server_client.py:9`：「用到的操作码：成批点读 1、成批写 2（带客户端身份与请求号）、按前缀扫 3、统计 4、报到 6（序号 0、服务不回）、宣布停服务 8（只认 unix socket，只给自证）」；`scripts/mutation/lib_mutation_units.py:65`：「（服务没有删的操作），交空的计数」。
- 压测对着的就是真服务：`scripts/prune-server/prune-server-load-simulation.py:11`：「服务没有删的操作，这一趟写的留在服务里」。条款写的是「自证与压测给临时服务起的库只在那一格里活、格完即删」，实现是「对着真服务、写下的永远留着」。
- 开着开关跑的自证同样：命名空间只按库目录在不在临时根下分（`scripts/prune-server/prune_server_client.py:26`：「命名空间：库的本地目录落在临时目录（TEMPORARY_ROOTS 与 tempfile.gettempdir()）底下的」，`scripts/prune-server/prune_server_client.py:115`：「TEMPORARY_ROOTS = ("/tmp", "/var/tmp", "/dev/shm")」）。三家的自证没有一处摘开关（量的：开关名在 `lib_verdict_library.py`、`admission.py`、`lib_mutation_units.py` 里各只命中文件头文档），code-tooling 在已接名单里、要带开关交，它的 research-script-selftests 格起全部研究脚本的自证，开关随环境传下去；每份自证的临时库在真服务里各占一个「@<16 位>」命名空间，删不掉，一趟一批（推的：按客户端与 code-tooling 读的，本腿没跑）。
- 怎么与真服务分开：服务那一侧分不开，只看键里有没有「@」；库目录不在临时根下的自证（例：放在工作根 `~/.local/share/singlefs/work/` 下的副本里）不带命名空间，写进两台共用的那一份（推的，本腿没找到今天这样做的自证）。条款与 C948 都没写「自证写进真服务」这第三种。
- 推翻条件：服务有删的操作且自证与压测格完删掉，或三家自证起来先摘开关、只对临时服务。

### 2.4 （本腿加）第二台片库并进的不是服务 —— 打中

`scripts/experiment-apparatus/e182-single-device-long-run.sh:13`：「取回之后在本机 --import-library <取回的目录> 把片文件并进本机库」，本机库就是 `.git/singlefs-unit-verdicts/e182-single-device`（第 149 行缺省）。条款给「第二台一趟的片库」放行的前提是「并回服务那一份」；这里并回的是另一份文件库。条款只写了前提，没写并回别处算什么、谁看。

### 2.5 （本腿加）第二台家目录下的崩溃库 —— 推的，没核

- checker 档的 `cargo test` 派得到第二台：路线判法不认 PATH 上的程序（`scripts/hosts/multi-host-run.sh:747`：「PATH 上的程序（命令词不带 /、不是 bash / python 起的）不算」），`cargo test -p singlefs-checker-tier …` 判成不带判定库；`task-server/task_server.py:2682` 那行注释（「任务 2175 的 5 条测试 panic」，同一行接着点到 `crates/singlefs-checker-tier/`）记着任务 2175 在第二台跑过 checker 档的测试。
- 崩溃库目录缺省取 `$HOME`（`crash_library.rs` 的 `crash_library_home`）。不带开关派到第二台，就在第二台家目录下建、写一份长期的崩溃判定库；带开关时每进程库建在第二台家目录旁边，进程被杀时留在那里，只有第二台下一个开库的进程收养（崩溃那几道只在本机，第二台未必再有人开）。两种都不是条款说的中转库，也不随树删。
- 没核：本腿不连第二台，第二台家目录下有没有这份库没看。核法写在「要主 agent 跑的命令」之外（要连第二台，交主 agent 定）。

## 三、K2 第三条：「没派到第二台不算违例」会不会让多机那一条空转

被判条款：`.claude/rules/verification.md:83` 的「不算写完也不算违例，记进欠账 C949」，同一行「被派到第二台跑过（`task-server/task wait` 末行 `host=peer`）且结局与本机相同才算写完」；`.claude/main-agent.md:13`：「服务没派到第二台的记进欠账 C949」。

### 3.1 派不派第二台由服务挑，没派到没有后果 —— 打中

- 服务挑台：`task-server/task_server_lib.py:145`：「放得下的不止一台时挑放下之后剩余占那台总上限比例大的，一样大取本机」；`task-server/task_server_lib.py:167`：「提交时不收挑机器的参数，没有」（同一行接着写没有点名要第二台的写法）。本机空、第二台忙时，新测试被派在本机，是常态而不是例外。
- 条款括号里列三种原因（第二台用不了、配置写单机、入口登在只在本机的表里），主句是「服务没把它派到第二台」，按余量挑了本机也落进主句。落进去之后只「记进欠账 C949」：条款、共用约束、主 agent 定义都没有一句「没写完不许提交」或「隔多久重交」（量的：`grep -n` 没写完、不许提交、不算写完 三个词在 `verification.md`、`agent-common.md`、`main-agent.md` 里命中 3 行：本条自己、主 agent 定义第 100 行讲广谱变异、共用约束第 48 行讲写者锁）。一条读了本机独有状态的新测试，只要一直没被派到第二台，就一直「本次未跑」、一直不违例。
- 还有几条结构性的「永远在本机」：命令带 `--write` 的（`task-server/task_server.py:226`：「LOCAL_ONLY_ARGUMENTS = ("--write",)」）、登记表里有库而不在已接名单里的（今天崩溃那几道、`replay.sh`、`broad-mutation.py`、`device-code-key.py`）、放在 `scripts/hosts/` 下的。最后一种连「登表」都不用：目录整个登着，新文件自动罩进去，而条款第 82 行把那里的自证直接算成「只能在一台上跑的活」。
- 第二台长期用不了时：所有新测试都停在「本次未跑」，条款没有上限，C949 只涨不判。
- 推翻条件：条款或主 agent 定义写明「第二台那一趟本次未跑」的测试不许提交、或要在多久之内补跑；或 task-server 有办法让一条任务必须在第二台跑一趟。

### 3.2 派到了也多半什么都没判：复用本机那一条 —— 打中

- 两台读写同一份库、键里没有台：`scripts/prune-server/prune_server_client.py:27`：「别处的（主仓与仓副本的 git common-dir、第二台那一趟的树）不带，两台读写同一份。」；单元键不含台（`.claude/rules/verification.md:51` 的「路径、名字、行号、资源上限、限时、提交与树不进键」）。
- 所以「新写的测试先在本机交一趟、再被派到第二台一趟」的第二趟，键与第一趟相同、库里有绿，第二台照复用，成功句报「现判 0、复用 N」，`host=peer`、结局相同：照条款第 83 行就算写完了，第二台一条都没判。对带判定库的门禁与入口（登记表 18 行带库的全是）这是缺省走法，不是边角。读本机独有状态的那条用例恰好在复用里被跳过。
- 条款没写第二台那一趟要不读库（`--full` / `SINGLEFS_GATE_FULL=1`），也没写「结局相同」比的是现判还是复用。第一轮 4.2 第 2 条攻的是 C949 怎么记台；这里打中的是规则第 83 行「写完」的判据本身。
- 推翻条件：条款写明第二台那一趟不读库、全部现判，或「结局相同」只认现判的项。

### 3.3 「只影响排程的本机状态许读」：边界没人判 —— 打中

- 条款点名许读的两样都在 `~/.local/share/singlefs/` 下：峰值表（`scripts/hosts/memory_peaks_lib.py:331` 取 `~/.local/share/singlefs/memory-peaks.tsv`）、上次红的清单（`scripts/gate/lib_reuse_checks.py:123`：「DEFAULT_LAST_RUNS_DIRECTORY = "~/.local/share/singlefs/last-runs"」）。C949 的静态格判 `~/`、`$HOME`、`expanduser`，判别力样本是「读 `~/.local/share/singlefs/` 的自证」转红（`.claude/kb/checks-owed.md:606`：「样本里放一份读 `~/.local/share/singlefs/` 的自证（转红）」）。照字面实现，`lib_reuse_checks.py` 读上次红的清单那一处判红；词法格分不出「影响排程」与「影响判定」，许读的那一类躲不开它。
- 反过来，排程用的东西不一定「按台各一份」：变异的 timings/ 源码自己写「派活次序用，不是判定」，开关开着时进了两台共用的那一份（1.1 表）。条款「按台各一份」是描述还是要求，没说。
- 谁判：条款没点名；`.claude/main-agent.md:13` 只说审 diff 见到「读会改变判定结局的本机独有状态」打回，判断交给审的人，没有清单、没有格。
- 推翻条件：条款给出许读的本机状态的登记处（文件或路径表），C949 静态格照它放行。

### 3.4 （本腿加）条款抄了服务表的现状，今天已经对不上 —— 打中

- `.claude/rules/verification.md:82` 点名「表里已登的目录（`task-server/`、`scripts/hosts/`、`.claude/hooks/`、`scripts/agents/`）」；本腿交回前现读：`task-server/task_server.py:223`：「LOCAL_ONLY_DIRECTORIES = ("scripts/hosts",)」。开工快照之后别的会话改了表，条款里四个目录有三个已不在表里。
- 同一处表头新写：`task-server/task_server.py:220`：「# 只在本机的那几张名单（用户：」，同一行接着写每一条都是欠账、不许新增条目。条款第 82 行却叫只能在一台上跑的活「登记进 task-server 的那几张表」：一个叫登、一个不许登。
- 推翻条件：条款改成只指到表、不抄目录名，且与表头的「不许新增」对齐。

## 四、K2 第四条：过渡期开关

被判条款：`.claude/rules/verification.md:75`：「没带的走旧文件库，接完之前算过渡，接完之后算违例。」；`.claude/rules/verification.md:73`：「各家迁完、`--verify` 整份回读逐条相同之后，旧库由主 agent 删掉。迁完之前旧库只读不写。」

### 4.1 同一节两句直接冲突 —— 打中

- 第 75 行：过渡期没带开关的走旧文件库（读写）；第 73 行：迁完之前旧库只读不写。迁完在接完之前或同时（先迁、核、删，才谈得上接完），所以过渡期整段都在「迁完之前」：过渡期许写、迁完之前不许写，两句对同一段时间给出相反的话。
- 提交那一条路照规则走 `.claude/skills/crash-test/commit-stages.md` 与 `.claude/skills/gate/staged-gate.md`（`.claude/rules/verification.md:57`），两份技能都没写开关（量的：`grep -rn SINGLEFS_VERDICT_LIBRARY_SERVICE .claude/skills` 零行），照技能交的六道重型门禁就是「没带的走旧文件库」：每次提交都写旧库。这一条与第一轮「开关没人设」相邻，本腿只拿它说明冲突在提交路径上必然发生。
- 推翻条件：第 73 行改成「迁完之前旧库只由没带开关的过渡任务写」或第 75 行改成过渡期没带开关的只读不写。

### 4.2 「迁完」的判据与用服务互斥 —— 打中

- `--verify` 三个数都要 0：`scripts/prune-server/prune-server-import.py:14`：「后三个都是 0 退 0，否则退 1 并列出前几条。导完跑它一遍才算导对。」，其中 extra 是服务有、本地没有（第 251–252 行）。
- 开关一开，那一族的新判定只进服务、本地不建不写 ⇒ 下一次 `--verify` 的 extra 大于 0；过渡期没带开关的任务写本地 ⇒ missing 大于 0。只要那一族在两条路上任何一条被用过，`--verify` 就再也过不了，除非重导一遍再在没人用的空当里核。条款写「各家迁完、`--verify` 整份回读逐条相同之后」删旧库，没写要停哪些任务、停多久、谁停。
- 今天的数（摘自正文第二节主 agent 的观测，本腿没复核）：verdict-library 一家 7200 条导进并 `--verify` 整份相同；此后开着开关跑过的形式化、lkmm、QEMU 门禁都会让它再核不过（推的）。
- 推翻条件：`--verify` 改成只核「本地有的服务都有且相同」（不数 extra），或条款写明迁移窗口内停掉那一族的全部任务。

### 4.3 「接完」没人宣布，现有的「已接」名单与实际对不上 —— 打中

- 条款与两份定义都没写谁宣布接完、宣布在哪（量的：`grep -n 接完` 在 `verification.md`、`agent-common.md`、`main-agent.md` 里只命中第 75 行这一处）。
- 仓里唯一一份机器可读的「已接」名单在 task-server：`task-server/task_server.py:211`：「# 崩溃判定库那几道（crash-point-replay、crash-amplification、checker-tier-crash-point-replay）不在这里，照旧只在本机」，上一行写它是「判定库读写已接 prune-server 的登记路径」。它管的是派不派第二台，不管存哪；名单里的 checker-tier-research-build-and-replay 带着开关照写 replay-row（1.2），checker-tier-crates-mutation-replay 带着开关照写 target-occupancy.tsv（1.3）、开着家目录下的变异复跑崩溃库（2.2）。拿它当「接完」的宣布，接完之后带开关也照写旧库；不拿它，就没有宣布。
- 「接完之后没带的算违例」没有任何东西认：task-server 只拿开关挑台，钩子与技能里没有一处查开关（量的：`grep -rn SINGLEFS_VERDICT_LIBRARY_SERVICE .claude/hooks .claude/skills` 零行）。
- 推翻条件：条款点名一处「哪一族接完」的登记（例：登记表第三列或单独一张表）并有一格按它判没带开关的任务。

### 4.4 （本腿加）旧库删掉之后，不带开关就悄悄另起一份 —— 打中

- 客户端缺省走本地：`scripts/prune-server/prune_server_client.py:14` 的「不设或设成空串：三家照旧读写本地文件，这份客户端一个字节都不碰。」Rust 那一侧同样（`crash_library.rs` 的 `prune_server_is_switched_on_by`：不设或空不开）。
- 旧库按第 73 行删掉之后，任何一趟没带开关的门禁照常起：库目录不在就建，空库全部现判、写回本地（`lib_verdict_library`、admission、`lib_mutation_units` 都是「取不到就建」的写法，例 `scripts/experiment-apparatus/replay.sh:3467` 的 `mkdir -p`）。结局是绿的，只是慢；什么都不红，「只有一份」悄悄变成两份。C948 的「迁完删掉之后还在就判红」要等格立起来才认得出，今天没有。
- 推翻条件：接完的族不带开关时退出（不建本地库），或 C948 那一格已立。

## 五、K3：C948、C949 新写法立得成立不成

### 5.1 C948（判定另存别处的库）—— 打中，立不成

欠账原文：`.claude/kb/checks-owed.md:605`：「接进之前登记为过渡的行报本次未跑」；同一行「开 RocksDB、写 TSV / sqlite 存判定的调用不经 prune-server 客户端」判红；同一行「中转库（并回即删）与自证临时服务的库按文件头登记放行」；同一行「迁完删掉之后还在就判红」；判别力同一行「往 TSV 写判定的脚本（转红），一份只写产物缓存的（不红）」；前置同一行「先把 Python 三家与崩溃判定库全接进 prune-server」。

1. 第三列今天写什么（量的，python 读 `.claude/gate.d/verdict-library-scripts.tsv`）：带库的 18 行全是本地位置（`<库根的 git common-dir>/…`、`${SINGLEFS_BROAD_MUTATION_HOME:-…}/verdicts.tsv`、`${SINGLEFS_CRASH_AMPLIFICATION_HOME:-…}/crash-state-verdicts`、「装置各自的库」），写「prune-server」的 0 行；另有「不算：」11 行、「无判定库：」3 行（照「第三列不是 prune-server 判红」的字面，这 14 行也红，欠账没写它们怎么办）。`scripts/crash/crash-point-replay.sh` 一行的第三列同时写放量与快档两份库，快档那份（admission）接了、放量那份（崩溃库）没接时，一格写不出「prune-server」也写不出「过渡」。
2. 谁改、什么时候改：欠账没写。第三列不只是给人看的位置，`reuse-selftests` 拿它认判法脚本（`scripts/gate/lib_reuse_checks.py:661`：「named = leaked_library if "audit-leaked-library" in BREAK else row_library」，下一行从第三列里抽 `scripts/…` 路径当另认的脚本名）；整格改成「prune-server」会丢掉这一层（推的：哪几行今天真靠它认，本腿没逐行跑）。
3. 「过渡」没有写法：`scripts/gate/lib_reuse_checks.py:42`：「第三列三种写法：」，三种是库在哪、「无判定库：」、「不算：」，没有第四种；加一种要先改这份判法与它的自证。
4. 「中转库与自证库按文件头登记」没有定：全仓代码里没有一份文件头写「中转库」（量的：`grep -rln 中转库 scripts crates prune-server task-server .claude/gate.d .claude/hooks` 零行），也没有约定的标记行；而且按文件头放行只放写法，不看那一份到底并没并回（2.1）。
5. 静态判法漏一整类：欠账只认「开 RocksDB、写 TSV / sqlite」，规则第 74 行列的第一类「另开一个目录存判定文件」不在里面。今天活着的三处都是这一类：replay-row（`keys/<行键>/verdict` 一个文件一样）、E182（`.shard` 文件）、flow-lines（`.lines` 文件）。射程也够不着：E182 的写者是 `scripts/experiment-apparatus/e182-single-device-long-run.sh`，`crash-flow-coverage.py` 在 `scripts/crash/` 下而现扫射程只扫那里的 `*.sh`（第 11 行）；欠账把射程扩到 `scripts/experiment-apparatus/*.rs` 与 `crates/`，这两份仍在外面。
6. 「旧库删掉之后还在就判红」会撞正当代码：开关开着时 crash-flow-coverage 照建 `<种类>/flow-lines/`（`crash-flow-coverage.py:317`）、变异照 `O_CREAT` 建 `target-occupancy.tsv`、replay.sh 照 `mkdir -p` 建 replay-row（1.2、1.3）。照欠账立格，删完旧库的第一趟提交就红在这三处；欠账的前置只写「Python 三家与崩溃判定库全接进」，没列这三处要先搬。
7. 两份样本分不出判法：正样本同时具备「注释写判定库」与「往 TSV 写判定」两样，负样本两样都没有。一个只看注释词（照登记表现扫的 `^#.*(判定库|复用)` 那一套）、根本不认写法的实现，正样本红、负样本绿，两份都过；欠账却声称要认「开 RocksDB、写 TSV / sqlite 存判定的调用」。真正漏的形状（注释只写「片结果库」「复用」、写的是文件目录，E182 那样）没有样本。第一轮攻方 4.1 第 3 条要的是「注释里没有判定库、照样写判定的也要红」那一份，新写法把它换成了「只写产物缓存的不红」，判别力那一半掉了。
- 推翻条件：欠账补出第四种写法与中转库的标记行、把「目录存判定文件」列进判法与样本、前置列出 flow-lines、target-occupancy、replay-row 三处，并加一份「注释没有判定库、照样写判定」的正样本。

### 5.2 C949（测试只在一台机器上跑得起）—— 打中，立不成

欠账原文：`.claude/kb/checks-owed.md:606`：「判定库里的判定键不带台，台记在服务那一侧按键存的」，同一行「（要先给判定格式加这一列）」；静态格射程同一行「`scripts/`、`.claude/gate.d/`、`crates/*/tests/`、`scripts/experiment-apparatus/` 里写死的主机名」，存量同一行「存量登进只缩不涨的滞后表」。

动态格（「哪几台判过」一列）：

1. 服务今天按键只存值：`prune-server/src/storage.rs` 文件头写库里只有数据行与两张记账表，记账表一张是 `prune-server/src/storage.rs:9` 的「已生效的请求号：键 `F0`」（身份加请求号 → 写了几条），一张是认领；没有按数据键记台的地方。客户端身份里带主机名（`scripts/prune-server/prune_server_client.py:389`：「self.identity = f"python:{socket.gethostname()}:{os.getpid()}:{os.urandom(4).hex()}"」），但只进去重那张表，键是身份不是数据键。
2. 服务看不到「第二台判过」：第二台复用一条是读，不发写帧（3.2）；第二台现判出与库里逐字节相同的结论，Rust 一侧并库照规矩跳过（`crates/singlefs-checker-tier/src/verdict_store.rs:2404`：「照并库的规矩（[`VerdictStore::merge_the_rows_of_the_library_of_this_process`]：逐字节相同跳过、有红整份换、两份不红而不同拒）推进服务。」），同样不发。服务那一侧加一列也记不到这两种，得在协议里加一种「这一台判过这一键」的帧，两家客户端都改。
3. 欠账写「给判定格式加这一列」：加在判定的值里，第二台写同一键时值里的台不同 ⇒ 两份都不红而字节不同 ⇒ Rust 并库拒（上面那一行的「两份不红而不同拒」）、并不进去的每进程库照留（2.1）；Python 一侧导入把它当冲突（`scripts/prune-server/prune-server-import.py:27`：「别的不同（两份都不红、或服务里的是红）不写，记一条冲突」）。加在值里与并库、导入的逐字节规矩冲突；加在服务的记账里又不是「判定格式」，欠账没选。
4. 动态格的绿要「结局与本机相同」，可复用让第二台那一趟天然相同（3.2），这一格不改判据照样空转。

静态格：

5. 照欠账自己的词（`~/`、`$HOME`、`${HOME`、`expanduser`、`/home/<用户>/`）与射程（git 跟踪的 `scripts/`、`.claude/gate.d/`、`crates/*/tests/`）现数（量的，python 扫，列表落在草稿目录 `c949-static-scan.txt`）：射程 6942 份文件，命中 147 份；其中门禁样本（`.claude/gate.d/fixtures/`）63 份，命中只在注释类行（以 `#`、`//`、`*`、`-`、`|`、三引号起头）的 23 份（粗分，按行首判）。正文说的「55 份」是第一轮用另一组词（`.local/share/singlefs` 一类）、另一组射程数的，与 C949 现写的词与射程不是一回事。
6. 放行能放的（task-server 只在本机的几张表今天点到的）：`scripts/hosts/` 下 13 份、`scripts/kb/` 下 6 份、`LOCAL_RESOURCE_ENTRIES` 3 份、`LOCAL_ONLY_WRITERS` 1 份，合 23 份；滞后表要登约 124 份（推的：147 − 23，没逐份分合规与违例）。服务表头今天写着每一条都是欠账、不许新增（3.4），表一缩，放行的就变成滞后表里的，「只缩不涨」那一头会被表的收缩顶着涨。
7. 射程漏掉影响判定最大的那一处：崩溃库的库目录缺省取 `$HOME`，住在 `crates/singlefs-checker-tier/src/crash_library.rs`（`crash_library_home`），用的是 `std::env::var_os("HOME")`；射程只有 `crates/*/tests/`，词里也没有 `var_os("HOME")` 这种写法。射程里的 `scripts/mutation/crates-mutation-rows.py`（变异复跑崩溃库那一处）倒是能扫到。
8. 判别力样本与规则冲突：规则许读的上次红的清单就在 `~/.local/share/singlefs/last-runs`（3.3），样本却把「读 `~/.local/share/singlefs/` 的自证」定成转红；照样本实现，`scripts/gate/lib_reuse_checks.py` 第 123 行判红。
- 推翻条件：服务里已有按数据键记台、且复用与逐字节跳过也记得到的路；静态格的射程含 `crates/*/src` 并有许读路径的登记表，样本与它对齐。

## 六、打中之后的四句

这一轮判的是一份条款的写法，没有并列的几条臂；「分不分辨臂」一列答的是：打中的是改后这一版独有，还是第一版与改后这一版一起中。

| 打中 | 分辨 | 被判的一方当时看不看得到判别它的东西 | 满足的是哪个分句 | 已写的改法（C948、C949 新写法）还中不中 |
|---|---|---|---|---|
| 1.2 定义罩不住 / replay-row、E182 不在名单 | 改后这一版独有（第一版没定义） | 审 diff 的人看得到代码；条款里没有「非判定的东西」怎么办 | 第 72 行定义与括号名单 | 还中：C948 按登记表判，E182 不在登记表、replay-row 照旧写本地 |
| 1.3 开关开着照写旧库目录 | 两版一起中 | 看得到（代码写着），没人看 | 第 75 行「没带的走旧文件库」的反面 | 还中：C948 第六条会红在正当代码上 |
| 2.1 每进程库并不进去照留 | 改后这一版独有（中转库这一句是新加的） | 运行时才有，审 diff 看不到 | 第 74 行「并回即删」 | 还中：C948 按文件头放行 |
| 2.2 变异复跑崩溃库 | 两版一起中 | 看得到（门禁文件头写着） | 第 74 行「另开一份 RocksDB」、第 72 行名单 | 还中：C948 的射程含 `scripts/mutation/*.py`，但它按第三列判，这一行第三列写的是变异库，崩溃那份没登 |
| 2.3 自证、压测写真服务 | 改后这一版独有 | 看得到（压测文件头写着） | 第 74 行「给临时服务起的库……格完即删」 | 还中：C948 放行「自证临时服务的库」，这里不是临时服务 |
| 3.1、3.2 空转 | 改后这一版独有（「不算违例」与 `host=peer` 是新写的） | 主 agent 看得到 `host=` 与「现判 / 复用」，条款不让它看后者 | 第 83 行「写完」与「不算违例」两句 | 还中：C949 动态格同一判据 |
| 3.3 边界 | 改后这一版独有 | 审的人看得到路径，看不出它影不影响判定 | 第 81 行「只影响排程的……许读」 | 还中：C949 静态格是词法的 |
| 3.4 条款抄表 | 改后这一版独有 | 看得到 | 第 82 行目录名单 | C948、C949 不管 |
| 4.1、4.2 过渡与迁完 | 改后这一版独有 | 看得到 | 第 73 行与第 75 行 | 不管 |
| 4.3、4.4 接完与悄悄另起 | 改后这一版独有 | 运行时 | 第 75 行「接完之后算违例」 | C948 第六条立了才认 |

## 七、交主 agent 的：每条打中对应的改法

都是本腿按读到的条文与代码推的改法，没实现、没跑，只在本腿的读法上成立、被攻过零轮；「修哪一格」一栏全是「推的」。

| 打中 | 改哪一处 | 改成什么（要点） | 修哪一格 |
|---|---|---|---|
| 1.2 定义罩不住 | `verification.md:72` | 定义二选一：按「按键存、给下一趟复用的一切」定（录入块、事实表、覆盖、用时一并进那一份）；或保留窄定义，另写一句非判定的东西放哪、许不许另开。括号名单补 replay-row、E182 片结果库、变异复跑崩溃库，并在 `checks-owed.md` 各记一行 | 1.2、2.2 |
| 1.2 导入不导的几样 | `verification.md:73` | 「整份导进」改成点名导入脚本不导的几样（replay-row、flow-lines、target-occupancy.tsv）各自的去向；删旧库按导了的那几样删，不按整个目录删 | 1.2 |
| 1.3 开关开着照写旧库目录 | crash-flow-coverage、crates-mutation-rows 的存处 | flow-lines 经客户端进 unit-verdicts 族；target-occupancy 搬到按台的运行状态目录；或开关开着时这两处不取 `directory` | 1.3、C948 第 6 条 |
| 2.1 每进程库并不进去照留 | 崩溃入口与 checker-tier-crash-point-replay | 数 `CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 行，非 0 判红或本次未跑；条款写明并不进去的中转库算违例、由开它的那一趟报 | 2.1 |
| 2.2 变异复跑崩溃库 | `verification.md:74` 与 `crates-mutation-rows.py` 的 `mutation_crash_library_environment` | 条款写明变异版判定放哪（例外或欠账）；函数清掉开关、片库改成一趟一份用完即删 | 2.2 |
| 2.3 自证、压测写真服务 | `verification.md:74`；三家自证与 load-simulation | 要么自证与压测一律对临时服务、起来先摘开关；要么条款改说「写进真服务的命名空间、永不删」并给上限 | 2.3 |
| 2.4 E182 片库并进文件库 | E182 驱动 | 片结果经客户端进服务，`--import-library` 并进服务；或进条款欠账名单 | 2.4、1.2 |
| 2.5 第二台家目录崩溃库 | 路线判法或 task-server | `cargo test -p singlefs-checker-tier` 判成带判定库（不派第二台），或派第二台时强制带开关 | 2.5 |
| 3.1 没派到没有后果 | `verification.md:83`、`.claude/main-agent.md:13` | 写明「第二台那一趟本次未跑」的测试不算写完就不许提交，或给补跑时限；要让一条任务必须在第二台跑一趟，得改「提交不挑台」那条（要问用户） | 3.1 |
| 3.2 第二台复用本机那一条 | `verification.md:83` | 第二台那一趟不读库、全部现判（`SINGLEFS_GATE_FULL=1` 一类），「结局相同」只比现判的项 | 3.2、C949 动态格第 4 条 |
| 3.3 边界没人判 | `verification.md:81` | 列一张许读的本机状态路径表（峰值表、上次红的清单……），C949 静态格照它放行；timings/ 写明按台还是共用 | 3.3、C949 静态格第 8 条 |
| 3.4 条款抄表 | `verification.md:82` | 只指到 `task_server.py` 那几张表，不抄目录名；「登记进那几张表」与表头「不许新增」二选一对齐 | 3.4 |
| 4.1 过渡与只读不写冲突 | `verification.md:73`、`:75` | 改成「迁完之前旧库只由没带开关的过渡任务写」，或过渡期没带开关的只读；技能 commit-stages、staged-gate 写明带不带开关 | 4.1 |
| 4.2 迁完判据与用服务互斥 | `prune-server-import.py --verify` 或条款 | `--verify` 只判 missing 与 different 为 0（服务里多出来的不算错）；或条款写明迁移窗口停哪些任务 | 4.2 |
| 4.3 接完没人宣布 | `verification.md:75` | 点名一处「哪一族接完」的登记，与 task-server 已接名单分开；加一格判接完的族没带开关的任务 | 4.3 |
| 4.4 删后悄悄另起 | 三家库类与 Rust 开库 | 接完的族不带开关时退出、不建本地库 | 4.4 |
| C948 七条 | `.claude/kb/checks-owed.md:605` | 补第三列第四种写法与它的判法；定中转库与自证库的标记行，并加「运行时没并回」的检查；判法与样本加「目录存判定文件」一类；射程补 E182 驱动、`scripts/crash/*.py`；前置列出 flow-lines、target-occupancy、replay-row；加一份「注释没有判定库、照样写判定」的正样本 | 5.1 |
| C949 八条 | `.claude/kb/checks-owed.md:606` | 动态格：协议加「这一台判过这一键」的帧（复用与逐字节跳过也记），或改判据为第二台全部现判；静态格：射程含 `crates/*/src`、词含 `var_os("HOME")`，许读路径表放行，存量照现写的词与射程重数（本腿数出 147 份） | 5.2 |

## 没打中的形状

- 「判定」定义与 admission keys/、compositions/、变异 records/、baselines/、崩溃结论块：罩得住，没打中（1.1 表）。
- 中转库里的「分片的片库并回之后删」：本腿读到的崩溃双机分片第 0 片写长期库、第 1 片导入后随产物目录由 `task clean` 删（推的，没跟到删那一步）；红了不并回那一面第一轮 1.5 攻过，本腿没重复。
- 「不写死主机名与地址」：没找新的反例（第一轮已列 E171 的 `REPO=`）。
- 这几格各只抽一次，照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不支撑「没打中」。

## 这条腿自己的限度

- 没连第二台：2.5、1.2 里「第二台那棵树的 `.git`」、3.1 里派台的实际分布都是按代码推的。
- 没跑任何项目代码：1.3、2.2、2.3 的行为按源码与注释推；Rust 接服务的补丁还没编过（正文第二节）。
- `task_server.py`、`task_server_lib.py` 本腿开工后被别的会话改过，3.4 引的是交回前那一刻的样子；它们再改，3.4 的行号与内容会跟着动。
- C949 静态格的数是本腿的正则（`~/|\$HOME|\$\{HOME|expanduser|/home/<小写名>/`）在 git 跟踪的文件上数的；「只在注释里」是按行首字符粗分的。
- 禁读清单上的三份没读；调度记录批次号只取自正文第二节。

## 要主 agent 跑的命令

1. 核 1.3「开关开着时 admission 照算 `directory`、flow-lines 落进旧库目录」（不是重型，只载入模块、算一个路径，不连服务、不写文件）：
   - 命令：`SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server task-server/task submit -- python3 -c "import importlib.util; s=importlib.util.spec_from_file_location('admission_probe','/home/fy5090/code/singlefs/scripts/gate/admission.py'); m=importlib.util.module_from_spec(s); s.loader.exec_module(m); v=m.VerdictLibrary('/home/fy5090/code/singlefs','checker-tier-crash-point-replay.sh:quick-tier-test-functions'); print('FLOW_LINES_PROBE directory_set=%s store_set=%s' % (v.directory is not None, v.store is not None))"`
   - 在哪起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 那一行 `exit=0`；`task-server/task log <号>` 里 `FLOW_LINES_PROBE directory_set=True store_set=True` 算 1.3 前半坐实（开关开着、库走服务，而 crash-flow-coverage 取的 `directory` 不是 None）；`directory_set=False` 推翻。范围声明闸若要声明，照它的拒绝说明补。
2. 没写模型，没有复跑脚本。2.5 要看第二台家目录下有没有崩溃库，得连第二台，本腿不写命令，交主 agent 定。

## 没做什么

- 不判 K1（辩方那一格）；不替主 agent 采纳。
- 没往统一任务服务交任何任务；上面那条命令交主 agent。
- 没连第二台，没跑任何项目代码；条数是在源文件上 grep / python 现数的。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/single-library-defs-r2-attack/` 里只落了两份清单：`c949-static-scan.txt`（5.2 第 5–6 条的逐份命中）与 `heavy-selftests.txt`（自证登记表里登 heavy 一类的 5 行，本腿用来排除 3.3 的一条潜在反例：登记表 18 行带库的自证全登 light，`reuse-selftests` 读家目录下「上一次输出」的那条路今天没有一行走到，所以没写进打中）；没建仓副本、编译目录与库拷贝。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/single-library-defs-r2-opus-output.md --background research/prompts/_single-library-defs-r2-background.md` 末行（本腿交回前现跑）：「✓ 核了 67 处引文、25 处裸行号，对上 67 处，没判 25 处（其中裸行号没有引文可比 25 处；逐处列在上面）」。25 处裸行号多是交主 agent 表里与段内对已引过那一行的复指。
