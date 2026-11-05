# single-library-defs-r1 攻方腿（Opus）报告（2026-11-01）

攻击面：正文第三节 K2（条款与现有做法的冲突点、边角、没罩住的家）与 K3（欠账 C948、C949 能不能立成会红的检查）；不攻 K1。
被判的是 `.claude/rules/verification.md` 第 70–82 行两节、`.claude/agent-common.md` 第 39、40 行、`.claude/main-agent.md` 第 12、13 行（三份文件的 sha256 开工时与 `research/prompts/single-library-defs-r1-snapshot/defs-sha256.txt` 逐份相同，本腿现算核过）。
没写模型、没交任务；文中的条数都是本腿在源文件上用 grep / python 现数的（标「量的」），由路线、代码推出的标「推的」。

## 各格判定一览

| 格 | 点 | 判定 | 一句话 |
|---|---|---|---|
| K2-1 | 轻型三条在副本里跑写哪一份库 | 打中 | 定义与规则没有一处叫人设开关 `SINGLEFS_VERDICT_LIBRARY_SERVICE`，照 verification.md 第 50 行交的轻型三条按客户端缺省写旧文件库，正撞「只读不写」 |
| K2-1 | 实验装置 E174、E178（连带 E162） | 打中 | 「判定」没定义：轮库、项库是续跑缓存还是判定库，条款没说；登记表还写着「装置各自的库」；E162 装置开 RocksDB 是被测对象本身 |
| K2-1 | replay.sh | 没打中（库归属） | 它的库是 unit-verdicts 族，键空间里有；只连带开关问题与只在本机的问题（K2-2） |
| K2-1 | 广谱变异 TSV | 打中 | 条款点名广谱变异写进那一份、按「各族的键前缀」，而键空间只有三族、没有广谱变异；带密钥校验列这一层服务里没有对应 |
| K2-1 | 崩溃放量双机分片拷库快照再并回 | 打中 | 第 1 片在产物目录里另开一份 RocksDB、红了不并回；crates 变异门禁每片各一份崩溃库；照条款字面都算「另开一份 RocksDB」 |
| K2-1 | （本腿加）每进程一份库 | 打中 | `crash_library.rs` 开关开着时照样每个进程在库旁边建自己一份再并库 |
| K2-1 | （本腿加）今天的服务开的不是那一份 | 打中 | 在跑的 prune-server 开的是状态目录下的库，不是 `crash_library_directory`；今天照条款开开关，判定进的正是「另一份 RocksDB」 |
| K2-1 | （本腿加）自证的一次性库 | 打中 | verification.md 第 61 行要每份带库脚本自证复用三问，要空库；条款没给自证留口子 |
| K2-2 | 崩溃那几道只在本机 | 打中 | `task_server.py` 第 203 行写明崩溃判定库那几道照旧只在本机；它们不在条款点名的那两张表里，条款「没有例外」罩不住 |
| K2-2 | 「先在第二台跑过一趟」靠什么 | 打中 | 提交不选机器、调用方不许写挑台参数；除登记表外没有一条路能把一条任务钉到第二台 |
| K2-2 | 第二台掉线 / 配置是单机 | 打中 | 第二台用不了服务就本机跑；条款没写那时怎么算写完；配置写单机时按条款自己的「没写就是没有」永远写不完 |
| K2-2 | 只在本机的目录里的自证 | 打中 | `.claude/hooks`、`scripts/hosts`、`task-server`、`scripts/agents` 整目录只在本机，那里新写的自证永远到不了第二台；表里还登着一道门禁 |
| K2-2 | 真块设备 | 打中 | E185 的真盘臂登在「只在第二台跑」的表里，条款点名的两张表不含这一张 |
| K2-2 | QEMU | 打中（一半推的） | 路径来自另一份本地配置 `qemu.env`，第二台那棵树不带被忽略的文件；QEMU 门禁能派第二台、前提没齐判红而不是本次未跑 |
| K2-2 | GPU | 没打中 | 卡按台写在多机配置里（`local:`、`peer:`），清场归服务 |
| K2-2 | 读本机运行状态（memory-peaks、last-runs） | 打中 | crates 变异门禁读本机峰值表定每格内存；条款不分「影响判定」与「只影响排程」 |
| K2-2 | crates 变异双机分片 | 多机这一面没打中；判定库这一面打中 | 分片归服务、能派第二台；每片一份崩溃库撞「只有一份」 |
| K2-2 | （本腿加）两节互相矛盾 | 打中 | 「只有一份」点名的是家目录缺省路径下的库，「多机可用」不许读本机家目录下的库、不许补缺省 |
| K2-3 | 现有规则、定义与登记表里的冲突 | 打中 | 本腿 grep 范围内列出 17 处（文件与行在第三节），一处都没随这次改 |
| K3 | C948 能不能立成会红的检查 | 打中（现写法立不成） | 「写判定的调用」认不出；样本写法没钉，判别力那一句可真可假；「HEAD 之后」对 `.git` 下没跟踪的目录没有意思 |
| K3 | C949 能不能立成会红的检查 | 打中（现写法立不成） | 键里不带台、提交不能挑台、没库的测试无处记；静态格的规则写「家目录绝对路径」，判别力样本写的是 `~/`，规则抓不到样本 |

## 一、K2 第一条：「判定库只有一份」与现有做法的冲突点

被判条款：`.claude/rules/verification.md:72`：「全仓只有一份判定库：本机那份崩溃判定 RocksDB」；`.claude/rules/verification.md:73`：「另开一个目录存判定文件、另写一张 TSV 或 sqlite、另开一份 RocksDB 都算」，同一行「三家原来的文件库是旧物，只读不写」；`.claude/rules/verification.md:74`：「轻型与重型是同一套脚本、同一份库，只差范围」；`.claude/agent-common.md:39`：「不许另开目录、TSV、sqlite 或另一份 RocksDB 存判定」；`.claude/main-agent.md:12`：「实验装置的判定全部写进 prune-server 原地打开的那份 RocksDB」。

### 1.1 轻型三条在副本里跑写哪一份库 —— 打中

- 客户端只有开关开着才走服务：`scripts/prune-server/prune_server_client.py:14`：「不设或设成空串：三家照旧读写本地文件，这份客户端一个字节都不碰。」
- 规则、定义、技能里没有一处叫人设这个开关（量的：`grep -rn 'SINGLEFS_VERDICT_LIBRARY_SERVICE' .claude/agent-common.md .claude/main-agent.md .claude/rules/*.md .claude/skills -r` 零行；它只出现在 `scripts/` 下五份脚本与 `task-server/task_server_lib.py` 里，`task-server/task_server_lib.py:633`：「VERDICT_LIBRARY_SERVICE_VARIABLE = "SINGLEFS_VERDICT_LIBRARY_SERVICE"」）。
- 轻型那一行给的命令也不带它：`.claude/rules/verification.md:50`：「实现员、调查员把在自己副本里跑的那几条」交主 agent，命令是三条 `--quick` 原样。
- 照条款执行会出错的场景：实现员在 git worktree 副本里改了 `scripts/mutation/` 下的一份文件，报告里写 `SINGLEFS_RUN_SCOPE='种类=quick; …' task-server/task submit --cwd <副本> -- bash .claude/gate.d/checker-tier-crates-mutation-replay.sh --quick <副本>`（照 `.claude/rules/verification.md:50`），主 agent 照交。环境里没有开关 ⇒ `lib_mutation_units` 走本地 ⇒ 写进 `<库根的 git common-dir>/singlefs-crates-mutation-library/`（worktree 的 common-dir 就是主仓 `.git`）。这正是「三家原来的文件库」被写，撞 `.claude/rules/verification.md:73`。rsync 拷的副本没有 `.git`，`library_root` 取不到 common-dir（`scripts/crash/lib_verdict_library.py` 第 324–326 行，取不到交 None），那一趟不进任何一份库，又撞「两档判出来的判定都写进同一份库」。
- 现状核对（量的）：主仓 `.git` 下 `find <目录> -newer .git/HEAD -type f | head -1000 | wc -l`，`singlefs-crates-mutation-library`、`singlefs-formal-unit-verdicts`、`singlefs-mutate-library`、`singlefs-unit-verdicts` 各到 1000（截在 1000），即旧文件库今天还在被写。
- 推翻条件：找到一处规则、定义或 task-server 里给交轻型、门禁的命令自动带上开关的地方。

### 1.2 实验装置 E174、E178 各自的库 —— 打中

- 带库脚本的登记表现在还写着：`.claude/gate.d/verdict-library-scripts.tsv:25`：「装置各自的库（E174、E178）按这一份交的代码键存」。
- 两份装置的缺省目录：`scripts/experiment-apparatus/e174_lease_drop_wait_memory_return.rs:20`：「默认 ~/.local/share/singlefs/experiments/e174/rounds」；`scripts/experiment-apparatus/e178_leaf_and_record_death_counts.rs:20`：「项库在环境变量 E178_SEGMENT_FOUR_LIBRARY（默认 ~/.local/share/singlefs/experiments/e178/segment-four）」。
- 条款写了没有：没有。条款不定义「判定」。E174 的轮库存的是做完的一轮的读数（续跑用），E178 的项库存的是每个工作项算出的产物行，两样都不是「绿 / 红」的结论；崩溃判定库自己也存录入段的流、事实表与判表（`crash_library.rs` 第 14 行的文件头）。按「存判定」字面读，E174、E178 可以说不归它管；按「按键存、跨趟复用的东西」读，二者都归它管。两种读法下审 diff 的人判得相反。
- 连带：带库脚本登记表的现扫射程只到 `scripts/experiment-apparatus/replay.sh` 与 `device-code-key.py`，`.claude/gate.d/verdict-library-scripts.tsv:11`：「# scan: .claude/gate.d/*.sh scripts/crash/*.sh」，装置 `.rs` 不在射程里；装置里用的词是「轮库」「项库」，不是「判定库」，这张表现扫也扫不到。
- 再连带 E162：本腿量的 `DB::open` 一族调用点在 6 份文件里，其中 3 份是 E162 的装置（`scripts/experiment-apparatus/e162_crash_verdict_block_store.rs`、`e162_verdict_store_network.rs`、`e162_verdict_store_power_cut.rs`），量的就是判定库存储本身，开自己的 RocksDB 是实验对象，照条款字面同样算「另开一份 RocksDB」。
- 照条款执行会出错的场景：实验设计员给新实验写续跑项库，审的人按 `.claude/main-agent.md:12` 打回；设计员改走 prune-server，可装置的 Cargo 工作区 `scripts/experiment-apparatus/Cargo.toml` 的依赖只有 `aes-gcm`、`chacha20poly1305`，要接 `prune_server::reliable_client` 得先给装置加跨工作区依赖，条款没说可不可以、谁来做。
- 推翻条件：规则里补出「判定」的定义且明说续跑缓存与被测存储归不归它。

### 1.3 replay.sh —— 库归属没打中

- 它的库是 unit-verdicts 族：判定库 `<库根的 git common-dir>/singlefs-unit-verdicts/replay-row/`（`scripts/experiment-apparatus/replay.sh` 第 59 行），`scripts/prune-server/prune_server_client.py:20`：「族名：verdict-library」那一段列了 unit-verdicts，开关开着就进服务。
- 只连带两样：1.1 的开关没人设；它只在本机跑（见 2.1）。

### 1.4 广谱变异的 TSV —— 打中

- 条款把广谱变异列进「全部写进它，各族各自的键前缀（Python 一侧 `scripts/prune-server/prune_server_client.py` 文件头「键空间」）」（`.claude/rules/verification.md:72`）。
- 键空间只有三族：`scripts/prune-server/prune_server_client.py:20`：「族名：verdict-library」，接下去两行是 unit-verdicts、mutation-units，没有广谱变异；广谱变异的库是 `scripts/mutation/broad-mutation.py:49`：「默认 ${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation-units}」，不经这份客户端。
- 广谱变异的判定行带防手改的校验列：`.claude/rules/verification.md:142`：「每一行带用库目录与仓之外的密钥算的校验列」，`scripts/mutation/broad-mutation.py:102`：「密钥放库目录之外、仓之外」。服务那一侧不认证（`prune_server_client.py` 文件头第 11 行「TCP 上不认证」）。
- 照条款执行会出错的场景：主 agent 照 `.claude/main-agent.md:12` 派人把广谱变异改进服务，派发提示照它写「经 prune-server 客户端写进那一份库」。写的人找不到族前缀，只能自己起一族（条款说前缀在客户端文件头，那里没有）；校验列搬不过去就丢了门禁 gate-verdict 格逐行核密钥那一层，搬过去就要服务外再存一把密钥。条款没写这两样怎么办。
- 推翻条件：键空间里已有或条款里写明广谱变异的族名，并写明校验列的去向。

### 1.5 崩溃放量双机分片拷库快照再并回 —— 打中

- 分片驱动的第 1 片另开一份库：`scripts/crash/crash-amplification-two-hosts.sh:24`：「库写进它的产物目录，不钉，由服务照记录挑机器」；红了不并回：`scripts/crash/crash-amplification-two-hosts.sh:38`：「第 1 片的随它的产物目录留在服务那边、不并进长期库」。库自己的文件头也这样写：`crates/singlefs-checker-tier/src/crash_library.rs:10`：「第二台那一份落在树副本里、取回再导入」。
- 现有规则许这件事：`.claude/rules/verification.md:42`：「另交的库目录（环境变量」`SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`）。
- crates 变异门禁同形：`.claude/gate.d/checker-tier-crates-mutation-replay.sh:81`：「崩溃判定库每片各一份」；`.claude/gate.d/checker-tier-crates-mutation-replay.sh:79`：「分片驱动派到第二台的那一片给它设那一趟的树里一份」。
- 条款写了没有：没有。条款没分「一份长期库」与「中转用、并回即删的暂存库」，字面上「另开一份 RocksDB 都算」。
- 照条款执行会出错的场景：主 agent 照 `.claude/main-agent.md:12` 审一份改 `crash-amplification-two-hosts.sh` 的 diff（哪怕只改日志行），diff 上下文里就有「第 1 片的库写产物目录」，审的人照字面只能打回；或者照字面「修」：把每片各一份改回共用一份，就回到 `checker-tier-crates-mutation-replay.sh` 第 82 行写的几个工作进程互等开库锁（`DEFAULT_LOCK_WAIT_SECONDS`，1800 秒）。
- 矛不矛盾：和「只有一份长期库」不矛盾（并回之后只剩一份），和条款字面矛盾。是条款写宽了，不是做法错。
- 推翻条件：条款里补出「中转库、每进程库在并回长期库之前存在」算不算。

### 1.6 （本腿加）每进程一份库，开关开着也照建 —— 打中

- `crates/singlefs-checker-tier/src/crash_library.rs:21`：「建这个进程自己的一份库垫在快照上」（本机一份库时）；开关开着时同样：`crates/singlefs-checker-tier/src/crash_library.rs:1388`：「再在库目录旁边建这个进程自己那一份、垫在服务上」。
- 也就是条款要求的「经 `prune_server::reliable_client` 写进那一份」这条路本身，在客户端这一侧每个进程先开一份自己的 RocksDB。照字面，批次 7 正在写的那份实现就过不了 `.claude/main-agent.md:12` 的审。
- 每进程那一份是 RocksDB（量的，读源码）：`VerdictStore::create_over_the_prune_server`（`crates/singlefs-checker-tier/src/verdict_store.rs` 第 5916 行起）调 `Self::create_empty`，后者在第 3169 行 `rocksdb::DB::open(&library_options(LibraryCreation::CreateEmpty), directory)`。
- 推翻条件：条款里写明「并回即删的每进程库」不算另开一份。

### 1.7 （本腿加）今天的 prune-server 开的不是条款点名的那一份 —— 打中

- 服务缺省库在状态目录：`prune-server/src/configuration.rs:8`：「状态目录：`~/.local/state/singlefs/prune-server`」，库是它下面的 `library`。
- 本机在跑的单元（`systemctl --user status singlefs-prune-server`，本腿现查）：命令行带 `--state-directory`、不带 `--library` 与 `--library-layout`；起来时打的 `PRUNE_SERVER_LISTENING` 行里 `library=` 指到状态目录下的 `library`（那一行的本机主机名与有线地址换掉了，不抄）。即今天服务开的是缺省的多列族库，不是 `crash_library_directory`。
- 条款写的是现在时：`.claude/rules/verification.md:72`：「全仓只有一份判定库：本机那份崩溃判定 RocksDB」，「由 prune-server 原地打开、独占」。今天按条款「经 prune-server 的客户端写进那一份」开开关，Python 三家的判定进的是状态目录下那份库；崩溃判定库照旧在家目录。两份 RocksDB 同时有判定，正是条款禁的样子。
- 服务的说明还许绕过服务直接开库：`prune-server/README.md:28`：「开关不开、要直接开那份库的任务」先停服务、跑完再起。条款「独占」与它冲突，条款没改它。
- 推翻条件：条款加一句「批次 7 换上 crash-rocksdb 布局之前不生效」，或服务已换上那个布局（本腿查时没有）。

### 1.8 （本腿加）自证与单测的一次性库 —— 打中

- 现有规则要求每份带库脚本有复用三问自证：`.claude/rules/verification.md:61`：「自证里各有一格」`reuse-three-questions`，三问要「同一份输入第二趟现判 0」，得从空库起。
- 客户端自己的自证：`scripts/prune-server/prune_server_client.py:55`：「每格在临时目录起一份真 prune-server」，每格一份库；崩溃库的单测用一次性库：`crates/singlefs-checker-tier/src/crash_library.rs:12`：「流水线单测从空库起跑用」。
- 条款写了没有：没有。`.claude/rules/verification.md:73` 的「不许任何测试……新建自己的判定库」把自证、单测也罩进去。Python 三家开关开着时临时库进同一份库的「@命名空间」，算得上「一份」；Rust 一次性库与服务自己的自证照旧是另开的 RocksDB。
- 照条款执行会出错的场景：工具员给新的带库门禁写复用三问自证，照 `.claude/agent-common.md:39` 不许另开目录、照 `.claude/rules/verification.md:61` 又必须从空库起；两条只能违一条。
- 推翻条件：条款里写明测试替身、一次性库、命名空间分区算不算「自己的库」。

## 二、K2 第二条：「所有测试多机可用」的边角

被判条款：`.claude/rules/verification.md:79`：「不读本机独有的路径与状态（本机家目录下的库」，同一行「环境相关的一切只由多机配置」`multi-host.env` 定（没写就是没有，不补缺省）；`.claude/rules/verification.md:80`：「不算测试，登记进 task-server 只在本机的表」，同一行「测试一律不登那张表」；`.claude/rules/verification.md:81`：「新写的测试与自证先在第二台跑过一趟才算写完；只在本机绿过的不算。」；`.claude/agent-common.md:40`：「才算写完，报告里写明在哪台跑过」；`.claude/main-agent.md:13`：「新写的测试没在第二台跑过一趟不算写完」。

### 2.1 崩溃那几道今天只在本机 —— 打中

- `task-server/task_server.py:203`：「不在这里，照旧只在本机」（崩溃判定库那几道：crash-point-replay、crash-amplification、checker-tier-crash-point-replay）。
- 它们只在本机靠的不是条款点名的那两张表（`LOCAL_ONLY_DIRECTORIES`、`LOCAL_RESOURCE_ENTRIES`），是第三条路：`peer_eligible` 经 `multi-host-run.sh --verdict-library-route` 问「带不带判定库」，带而不在 `VERDICT_SERVICE_RELEASED` 里的不派第二台（`task-server/task_server.py` 第 1958–1971 行）。
- 本腿量的（python 读 `.claude/gate.d/verdict-library-scripts.tsv` 第一列与 `VERDICT_SERVICE_RELEASED`）：登记表 32 行，放行 12 份；第三列写着真有库、又不在放行名单里的有 7 份：`scripts/mutation/broad-mutation.py`、`.claude/gate.d/checker-tier-broad-mutation.sh`、`scripts/experiment-apparatus/replay.sh`、`scripts/crash/crash-point-replay.sh`、`scripts/crash/crash-amplification.sh`、`scripts/experiment-apparatus/device-code-key.py`、`.claude/gate.d/checker-tier-crash-point-replay.sh`。这 7 份今天派不到第二台（推的：按代码读的路线，本腿没起 `--verdict-library-route`，命令在「要主 agent 跑的命令」）。
- 条款罩不罩得住：罩不住。这 7 份全是测试、门禁或入口，按 `.claude/rules/verification.md:80`「测试一律不登那张表」不能登表，按「没有例外」又必须两台都跑得起；条款没有过渡句。崩溃点重放是崩溃一致性唯一的验法（`.claude/rules/verification.md@崩溃一致性只能靠崩溃点重放验证`），它正好在这 7 份里。
- 照条款执行会出错的场景：实现员加一条崩溃枚举用例，照 `.claude/agent-common.md:40` 要先在第二台跑一趟；主 agent 交 `task-server/task submit -- bash scripts/crash/crash-point-replay.sh --quick --item <新项>`，服务按路线只在本机起。这条用例照条款永远「没写完」。
- 推翻条件：放行名单里已有崩溃那几道，或条款写明「接进 prune-server 之前照旧只在本机、不算违例」。

### 2.2 「新写的先在第二台跑过一趟」靠什么派 —— 打中

- 客户端：`task-server/task:2`：「提交时不选机器、不给内存、不给 GPU 参数」。
- 共用约束同时禁调用方挑台：`.claude/agent-common.md:62`：「`--peer`、`--local-only`」写了提交时当场拒，同一行「调用方不写、不判」。
- 服务里唯一把一条任务钉到第二台的是登记表 `PEER_ONLY_ENTRIES`（`task-server/task_server.py:217`：「的真盘臂要量第二台那块盘」那一行），登进去的入口就成了「只在第二台跑」，又撞条款的「不依赖某一台」。
- 照条款执行会出错的场景：工具员写了一份新脚本的 `--selftest`，主 agent 交 `task-server/task submit -- python3 scripts/…/新脚本.py --selftest`；服务挑了本机（本机空、第二台忙都会这样），`task wait` 末行没有 `host=peer`。照 `.claude/agent-common.md:40` 这份自证没写完，主 agent 能做的只有重交、碰运气；条款与共用约束都没给第三条路。
- 推翻条件：task-server 有不经登记表就能要求「这一条在第二台跑」的办法（本腿读了 `task-server/task` 文件头与 `peer_eligible`、`candidate_hosts` 附近，没找到）。

### 2.3 第二台掉线、配置写单机 —— 打中

- 第二台用不了时服务只在本机排：`task-server/task_server.py:1931`：「if not snapshot.peer_available:」后面交回 False。条款没写这时「写完」怎么算：等、交用户、记欠账都没写。
- 配置可以合法地写成单机：模板 `multi-host.env.example` 里开双机的键 `ENABLE_ACROSS_MACHINES` 模板值写 0，紧挨着的注释写「崩溃放量两台分份都要它是 1」（这份模板本轮里有别的会话在改，本腿两次 grep 行号不同，不写行号）；用户对配置的口径：`records/2026-10-31-task-server巡检.md:79`：「配置文件没写多机 没有定义的内容，绝对不能自动补齐」。条款自己也写「没写就是没有」（`.claude/rules/verification.md:79`）。配置写单机时，按条款自己的口径「第二台跑过一趟」就永远做不到。
- 推翻条件：条款或共用约束里有第二台用不了时的出路。

### 2.4 只在本机的目录里的自证 —— 打中

- `task-server/task_server.py:212`：「LOCAL_ONLY_DIRECTORIES = ("task-server", "scripts/hosts", ".claude/hooks", "scripts/agents")」，同一行理由「.claude/hooks：喂钩子的样本带本机路径，第二台判出来的不可信」。这四个目录下的脚本连同它们的 `--selftest` 整个只在本机。
- 条款同时说「测试一律不登那张表」（`.claude/rules/verification.md:80`）与「新写的测试与自证先在第二台跑过一趟」（`.claude/rules/verification.md:81`）。钩子、多机脚本、任务服务、agent 工具的自证都是自证，表里却整目录登着。
- 表里还登着一道门禁：`task-server/task_server.py:232`：「prior-art 钉值那一格读仓外的参照源码树 fs-refs，第二台没有」（`.claude/gate.d/doc-registries.sh`）。条款第一条把门禁列在「每条测试、门禁、入口脚本、实验装置」里，这一格照条款不许只在本机，照表只在本机。
- 照条款执行会出错的场景：工具员给 `.claude/hooks/` 加一道钩子和它的 `--selftest`，照 `.claude/agent-common.md:40` 要第二台跑一趟；服务按目录永远不派。工具员永远交不了「写完」。
- 推翻条件：条款写明这四个目录的自证算「只能在一台上跑的活」，或把它们从表里拿出来。

### 2.5 真块设备 —— 打中

- `task-server/task_server.py:217`：「的真盘臂要量第二台那块盘」（E185 的分段驱动，登在 `PEER_ONLY_ENTRIES`）。这是一份实验装置依赖某一台（第二台）的盘。
- 条款点名的两张表是 `LOCAL_ONLY_DIRECTORIES`、`LOCAL_RESOURCE_ENTRIES`，不含「只在第二台」那张；条款的「只能在一台上跑的」举例也只有本机那一侧。量具体某块盘的装置天生只能在那块盘所在的那台跑，条款没给它位置。
- 推翻条件：条款写明「量某台硬件的装置」归哪一类、登哪张表。

### 2.6 QEMU —— 打中（第二台有没有补好的 QEMU 没核）

- QEMU 路径来自另一份本地配置：`scripts/devices/vm-bench.sh:268`：「路径来自主工作树根的本地配置 qemu.env」；补好的 QEMU 装在 `scripts/devices/build-patched-qemu.sh:19`：「SINGLEFS_QEMU_HOME         私有目录，默认 $HOME/.local/share/singlefs/qemu」。两样都不是 `multi-host.env`，后一样是条款点名不许读的 `~/.local/share/singlefs/` 下。
- 第二台那棵树不带被忽略的文件：`scripts/hosts/multi-host-run.sh:35`：「不带 .git、target、本地配置 multi-host.env 与别的被忽略的文件」（`multi-host.env` 另外照带，`qemu.env` 不带）。
- QEMU 门禁在放行名单里（开关开着就能派第二台），前提没齐判红：`.claude/gate.d/checker-tier-qemu-device-streams.sh:142`：「没齐判红。」。派到第二台、那边树里没有 `qemu.env` 时，按代码推是判红而不是「本次未跑」（推的，没在第二台起过）。
- 条款罩不罩得住：条款要求「两台里任何一台都要跑得起」，又只认 `multi-host.env`；QEMU 这一道今天两样都不满足，条款没写它怎么过渡、第二台谁来装。
- 推翻条件：第二台的树里有 `qemu.env` 或 QEMU 门禁不派第二台（本腿没连第二台，未核）。

### 2.7 GPU —— 没打中

- 卡按台写在多机配置里（模板 `multi-host.env.example` 里「显卡的先后与额度」那一段：`GPU_CARDS_BY_PRIORITY` 的 `local:`、`peer:` 两类项，开双机时两台各点名至少一张；行号在动，不写），清场与分卡归服务（`.claude/rules/verification.md` 第 63 行起「凡用 GPU 先清场」那一段）。没找到 GPU 测试必须读本机独有东西的地方。
- 取样范围：只读了多机配置模板与 verification.md 的 GPU 段，没读 GPU 判器的取卡代码，没看第二台的卡。

### 2.8 读本机运行状态（memory-peaks.tsv、last-runs）—— 打中

- crates 变异门禁读本机峰值表定每格内存：`scripts/mutation/crates-mutation-rows.py:2529`：「table = environment.get(PEAK_TABLE_VARIABLE, "").strip() or memory_peaks_lib.default_peak_table()」；峰值表按台各一份：`scripts/hosts/run-with-memory-cap.sh:63`：「第二台那一趟记在第二台自己的那一份」。这道门禁在放行名单里，开关开着能派第二台，派过去读的是第二台那一份。
- 它只影响每格内存与并行度、不影响判定结局（推的：按 `crates-mutation-rows.py` 文件头第 440–441 行与第 2526 行起那一段读的）。条款不分「影响判定」与「只影响排程」：`.claude/rules/verification.md:79` 字面上点名「本机 `~/.local/share/singlefs/` 下的运行状态」不许读，又说「不补缺省」，`default_peak_table()` 正是缺省。
- 上次红的清单：`.claude/rules/verification.md:60`：「取自 `~/.local/share/singlefs/last-runs/`」，每道门禁与入口跑完写在跑它的那一台；分片驱动在第二台那一片不写（`crash-amplification-two-hosts.sh` 文件头「上次红的清单」一段）。派到第二台跑的门禁写在第二台，本机范围声明闸读本机那份，是旧的。条款没提它。
- 照条款执行会出错的场景：审 diff 的人照 `.claude/main-agent.md:13`「读本机独有路径或状态……一律打回」，碰到任何改 `crates-mutation-rows.py` 峰值那一段的 diff 只能打回；可这一段是现行的排程做法（`crates-mutation-rows.py` 第 722–727 行的注释：每格峰值先取实测，查不到按缺省），打回它条款没给替代。
- 推翻条件：条款写明只管影响判定结局的读。

### 2.9 crates 变异双机分片 —— 多机这一面没打中，判定库这一面打中

- 多机这一面：分片归服务（`.claude/gate.d/checker-tier-crates-mutation-replay.sh` 第 108–110 行：这一道不自己分两台，另经 task-server 交 `scripts/mutation/mutation-shard-run.sh`），门禁在放行名单里，能派第二台。没找到它依赖某一台的地方（峰值表那一样见 2.8）。
- 判定库这一面：见 1.5，每片一份崩溃库、第二台那一片在树里一份，撞「只有一份」字面。

### 2.10 （本腿加）两节互相矛盾 —— 打中

- 「只有一份」点名的那一份：`.claude/rules/verification.md:72` 说是「`crash_library_directory` 指的目录」；它由家目录定、带缺省：`.claude/rules/verification.md:37` 的「`${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}`」「/crash-state-verdicts」，`.claude/rules/verification.md:42` 也写「库目录由家目录定」。
- 「多机可用」不许读的：`.claude/rules/verification.md:79`：「不读本机独有的路径与状态（本机家目录下的库」，又「不补缺省」。
- 开关开着时 Rust 一侧照样用这个家目录路径当「名字、并库锁与每进程库的落点」（`crash_library.rs` 第 1389–1390 行），在第二台就在第二台的家目录下建目录与锁文件。两节字面合起来：同一个路径既是唯一那一份的定义，又是不许读的本机家目录下的库。
- 照条款执行会出错的场景：审一份给崩溃库加读家目录缺省的 diff，按第一节它在描述那一份库，按第二节必须打回。
- 推翻条件：条款把「那一份」的定义改成「prune-server 那一侧的库，客户端不读路径」。

## 三、K2 第三条：现有规则、定义、技能、登记表里与「核心策略，没有例外」冲突而没改的

grep 范围（本腿只核这个范围，不外推到 kb 与 records）：`.claude/rules/`、`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/skills/`，词「判定库」「库拷贝」「库快照」「项库」「轮库」；另加带库脚本登记表 `.claude/gate.d/verdict-library-scripts.tsv`。命中后逐处读、只留与新条款字面冲突的。量的：下表 17 处。

| # | 位置与原文 | 撞的是哪一条 |
|---|---|---|
| 1 | `.claude/rules/verification.md:37`：「/crash-state-verdicts」（路径前半是 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}`） | 多机：本机家目录下的库、补缺省 |
| 2 | `.claude/rules/verification.md:39`：「各道的键与判定库写在它的文件头」 | 只有一份：各道各一份库的说法 |
| 3 | `.claude/rules/verification.md:40`：「这些的判定库放在库根那棵仓的 git common-dir 下（广谱变异的库目录除外）」 | 只有一份：库在 git common-dir；旧文件库照写 |
| 4 | `.claude/rules/verification.md:42`：「另交的库目录（环境变量」 | 只有一份：分片驱动另交一份库 |
| 5 | `.claude/rules/verification.md:57`：「它们把判定写进各自的判定库」 | 只有一份 |
| 6 | `.claude/rules/verification.md:61`：「自证里各有一格」（复用三问要空库） | 只有一份：自证的一次性库（1.8） |
| 7 | `.claude/rules/verification.md:142`：「每一行带用库目录与仓之外的密钥算的校验列」（判定库 `verdicts.tsv`） | 只有一份：TSV 判定库；多机：密钥在本机 `~/.config/singlefs` |
| 8 | `.claude/rules/implementation-workflow.md:44`：「它们把判定写进各自的判定库」 | 只有一份 |
| 9 | `.claude/skills/gate/staged-gate.md:16`：「六道各自按每条判定的单元键查自己的判定库」 | 只有一份 |
| 10 | `.claude/skills/crash-test/commit-stages.md:16`：「每道自己按每条判定的单元键查它的判定库」 | 只有一份 |
| 11 | `.claude/skills/crash-test/commit-stages.md:45`：「广谱变异驱动写的库目录」（同一行还写门禁写进 git common-dir 的判定库是阶段本身的行为） | 只有一份 |
| 12 | `.claude/main-agent.md:33`：「运行目录里的库拷贝（崩溃放量、变异跑完留下的）」 | 只有一份：库拷贝当常态 |
| 13 | `.claude/main-agent.md:100`：「判了的写进各自的判定库」 | 只有一份 |
| 14 | `.claude/agent-common.md:50`：「运行目录里的库拷贝用完就删」 | 只有一份：库拷贝当常态 |
| 15 | `.claude/agent-common.md:62`：「调用方不写、不判」（同一行禁 `--peer`） | 多机：「先在第二台跑过一趟」没有派法（2.2） |
| 16 | `.claude/rules/test-script-discipline.md:66`：「判定库放哪照」`command-safety.md`「测试镜像一律放临时目录」里跨轮缓存那一句 | 只有一份：库的位置另有一处规定 |
| 17 | `.claude/gate.d/verdict-library-scripts.tsv:25`：「装置各自的库（E174、E178）按这一份交的代码键存」；同一张表第三列「判定库放哪」各行写的都是各族自己的目录 | 只有一份 |

代码与服务登记表里（不是规则，但审的人照新条款会碰上）：

- `task-server/task_server.py:212`：「LOCAL_ONLY_DIRECTORIES = ("task-server", "scripts/hosts", ".claude/hooks", "scripts/agents")」与 `task-server/task_server.py:232` 那一道门禁（2.4）。
- `task-server/task_server.py:217`：「的真盘臂要量第二台那块盘」（2.5）。
- `task-server/task_server.py:203`：「不在这里，照旧只在本机」（2.1）。
- `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_segment_one_revision_11_multi_host.sh:13`：`REPO=` 写死本机仓的绝对路径（量的：同目录 7 份 `.sh` 带 `/home/<用户>/` 字面路径，其中 6 份以 `REPO=/home/` 起行；路径带本机用户名，不抄）。撞「不写死主机名与地址」，也撞私有信息不进仓。
- 没核的：`.claude/kb/`（`verification-build.md`、实验页）与 `records/` 里描述各家库位置的现状句，本腿没扫。

## 四、K3：C948、C949 能不能立成会红的检查

### 4.1 C948（判定另存别处的库）—— 现写法立不成

欠账原文的拦法：`.claude/kb/checks-owed.md:605`：「三家旧文件库目录在 HEAD 之后有新写判红」，判别力：`.claude/kb/checks-owed.md:605`：「样本里放一份往 TSV 写判定的脚本，那一格转红」。

第一格「扫写判定的调用、不经客户端写的判红」缺三样：

1. 「写判定的调用」在语法上认不出。Python 三家与广谱变异写判定用的是普通文件写（原子换名、追加 TSV），和写日志、写产物、写上次红的清单同形；Rust 一侧本腿数到 `rocksdb::DB::open` 一族调用点落在 6 份文件（量的：`grep -rn 'DB::open\|…'` 在 `crates`、`prune-server/src`、`scripts` 下）：`crates/singlefs-checker-tier/src/verdict_store.rs`（每进程库、一次性库都从这里开，合规的）、`prune-server/src/storage.rs` 与 `prune-server/src/selftest_library_layout_cells.rs`（服务自己）、E162 的 3 份装置（实验对象，见 1.2）。「`VerdictStore` 之外开 RocksDB 判红」在今天的树上就红 6 份里的 5 份，其中没有一份是条款想拦的。sqlite 那一类同理：用 sqlite 的是 `task-server/` 与两道钩子（量的，`grep -rln 'import sqlite3\|sqlite3.connect'`），存的是任务记录，不是判定。
2. 能做成的只有「按登记判」：带库脚本登记表 `.claude/gate.d/verdict-library-scripts.tsv` 已有第三列「判定库放哪」，可以立成「第三列不是 prune-server 的判红」。但它的现扫射程（`.claude/gate.d/verdict-library-scripts.tsv:11`：「# scan: .claude/gate.d/*.sh scripts/crash/*.sh」那一行）不含 `scripts/experiment-apparatus/*.rs` 与 `crates/`，而且只认注释里写了「判定库」「复用」的脚本：E174 的「轮库」、E178 的「项库」扫不到。
3. 判别力那一句没钉样本长什么样。样本脚本注释里写「判定库」，按登记表那一套会红；不写，照样往 TSV 写判定，就不红。欠账没写样本的注释与写法，这一句可真可假。要钉，得写两份样本：一份注释里有「判定库」、一份没有，后一份也要红；做到后一份就回到第 1 条「认不出」。

第二格「三家旧文件库目录在 HEAD 之后有新写判红」缺三样：

1. 旧库在 git common-dir 下、不被跟踪，git 没有「HEAD 之后写过」这回事；只能比文件改动时刻与 HEAD 提交时刻，或开格时落一份清单再比。欠账没选。
2. 「三家旧文件库目录」是哪几个目录没列。主仓 `.git` 下 `singlefs-*` 有 16 项（量的，`ls -d .git/singlefs-*`），里面混着判定库（`singlefs-formal-unit-verdicts`、`singlefs-unit-verdicts`、`singlefs-crates-mutation-library`、`singlefs-mutate-library`、`singlefs-lkmm-verdicts`、`singlefs-qemu-verdicts`）、旧名的库（`singlefs-formal-verdicts`、`singlefs-mutation-rows`）、缓存（`singlefs-judging-units-cache`、`singlefs-rust-lex-cache`、`singlefs-unit-graph-product-view`）、日志（`singlefs-crash-point-replay-logs`、`singlefs-mutation-shard-logs`）与锁。哪些算「旧文件库」要一张表。广谱变异的库目录、E174 / E178 的库、崩溃库旁边的每进程目录、第二台树里的库都不在 `.git` 下，这一格看不见。
3. 今天立就是红：量的，`find .git/<目录> -newer .git/HEAD -type f | head -1000 | wc -l` 在 4 个判定库目录各到 1000。这与欠账自己写的「先接进 prune-server 再立格」相符，但前提里漏了一样：开关得成缺省（今天开关没人设，见 1.1）。另：门禁在 `gate.sh --staged` 的临时树里跑，`library_root` 取的是被判那棵树的 common-dir，要看主仓 `.git` 得另指。

### 4.2 C949（测试只在一台机器上跑得起）—— 现写法立不成

欠账原文的拦法：`.claude/kb/checks-owed.md:606`：「判定库里按单元键记两台各一条」；静态一格：`.claude/kb/checks-owed.md:606`：「家目录绝对路径判红」；判别力：`.claude/kb/checks-owed.md:606`：「样本里放一份读」`~/.local/share/singlefs/` 的自证，那一格转红。

动态一格缺四样：

1. 派不到第二台：提交不选机器、调用方不许挑台（2.2）。格要的「每条在第二台跑过一趟」没有办法保证发生。
2. 判定里没有台：单元键不含台，`.claude/rules/verification.md:51`：「路径、名字、行号、资源上限、限时、提交与树不进键」；服务里的键也不含库根，`scripts/prune-server/prune_server_client.py:28`：「库根本身（哪棵仓的 git common-dir）不进键」，两台写同一把键。「两台各一条」要么把台加进键（第二台判过的不再给本机复用，复用的意义就去了一半），要么在值里另记台（键只有一份值，后写的盖前写的）。欠账没选，两条路都要改判定格式。
3. 没库的测试无处记：登记表第三列写「无判定库：待立」的几行（`scripts/gate/merge-and-verify.sh` 的 harness 档、`.claude/gate.d/fixtures`、`scripts/gate/light-selftest-guard.py`），以及整个 harness 档、钩子与脚本的 `--selftest`，没有按条判定可记。
4. 第二台掉线或配置写单机时这一格怎么报（红、本次未跑）没写；照第 2.3 节，这时永远到不了绿。

静态一格的判别力那一句写错了：

- 规则写「家目录绝对路径」，样本写的是「读 `~/.local/share/singlefs/` 的自证」。`~/` 不是绝对路径；按规则字面实现（匹配 `/home/<用户>/` 一类），样本不红，判别力那一句是假的。
- 把规则放宽到 `~/`、`$HOME`、`expanduser` 一类，样本会红，可今天的树上就红一大片：量的，`scripts`、`.claude/gate.d`、`crates` 下 `.py`、`.sh`、`.rs` 里写着 `.local/share/singlefs`、`.local/state/singlefs`、`.config/singlefs` 的 55 份（列表落在 `/home/fy5090/.local/share/singlefs/work/single-library-defs-r1-attack/home-path-files.txt`），其中有 `crates/singlefs-checker-tier/src/crash_library.rs`（崩溃库的缺省路径，正是「只有一份」点名的那一份）。
- 按字面 `/home/<用户>/` 匹配，量的 15 份，里面混着注释与自证里的假路径（例 `scripts/crash/crash-point-replay.sh` 第 2185、2189 行的 `$work/home/runs`），真写死的是 E171 那 6 份 `REPO=`。
- 主机名与第二台路径不能写进仓（`.claude/agent-common.md@写` 里私有主机信息那一条），这一格的匹配词只能运行时从 `multi-host.env` 读；做得到（门禁 doc-text 的 private-values 格同形），欠账没写。

缺的东西合起来：一份「判定」与「运行状态」的定义（哪些读写算违例）、task-server 一个「这一条要在第二台跑」的提交办法、判定记录里的台字段与它和复用的关系、第二台用不了时的结局、两份钉死的样本（各有一个反面样本说明不该红的不红）。

## 五、交主 agent 的：每条打中对应的改法

都是本腿按读到的条文推的改法，没实现、没跑，被攻过零轮。「修哪一格」一栏全是「推的」。

| 打中 | 改哪一处 | 改成什么（要点） | 修哪一格 |
|---|---|---|---|
| 1.1 开关没人设 | `.claude/rules/verification.md` 「判定库只有一份」一节，连同第 50 行轻型那一行 | 写明交带库的门禁、入口、轻型三条时带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`（或由 task-server 缺省带上），没带的算违例；副本里跑的不再靠 `SINGLEFS_VERDICT_LIBRARY_ROOT` | 1.1；连带 4.1 第二格的前提 | 
| 1.2、1.8 「判定」没定义 | 同一节第二条 | 补一句定义：判定 = 按单元键存、给复用读的结论；续跑缓存、产物缓存归不归它写明；被测对象本身是判定存储的实验装置（E162）、自证与单测的一次性库、prune-server 自己的自证写明不算或怎么算 | 1.2、1.8 |
| 1.4 广谱变异没有族 | 同一节第一条与 `scripts/prune-server/prune_server_client.py` 文件头「键空间」 | 要么条款里先把广谱变异移出「全部写进它」的名单、记欠账，要么键空间先定族名，并写明校验列的去向 | 1.4 |
| 1.5、1.6 中转库与每进程库 | 同一节第二条 | 写明「在并回长期库之前存在、并完即删的中转库（每进程库、分片的片库、第二台片库）不算另开一份」，并给出它必须并回或删的时限 | 1.5、1.6 |
| 1.7 现在时写的是目标态 | 同一节第一条 | 改成「目标：…；批次 7 换上 crash-rocksdb 布局之前，状态目录下那份与家目录下那份并存」，或等换上之后再生效；`prune-server/README.md` 第 28 行直接开库那条路写明还许不许 | 1.7 |
| 2.1 崩溃那几道只在本机 | 「所有测试都要多机可用」一节 | 加一条过渡：放行名单（`task-server/task_server.py` 的 `VERDICT_SERVICE_RELEASED`）外的带库入口照旧只在本机，列出、记欠账（可并进 C949），并写明崩溃那几道接进服务之后才算受这条管 | 2.1 |
| 2.2 派不到第二台 | 同一节第三条与 `.claude/agent-common.md` 第 40 行 | 二选一：给 task-server 加一个「这一条要在第二台跑」的提交办法并在共用约束里写明，与第 62 行禁挑台的那句划清；或把「先在第二台跑过一趟」改成「被服务派到第二台跑过一趟、日志里有 `host=peer`」并写明没派到时怎么办 | 2.2、4.2 动态格第 1 条 |
| 2.3 第二台掉线、配置单机 | 同一节第三条 | 写明第二台用不了或配置写单机时的结局（例：报「第二台那一趟本次未跑」、记进欠账，不算写完也不算违例） | 2.3、4.2 动态格第 4 条 |
| 2.4 只在本机的目录 | 同一节第二条 | 写明 `.claude/hooks`、`scripts/hosts`、`task-server`、`scripts/agents` 下的自证归「只能在一台上跑的活」还是要搬出表；`doc-registries.sh` 那一格同样写明 | 2.4 |
| 2.5 真块设备 | 同一节第二条 | 把「只在第二台跑的入口」（`PEER_ONLY_ENTRIES`）与「量某台硬件的装置」写进例外类，与本机那张表同等登记 | 2.5 |
| 2.6 QEMU | 同一节第一条 | 写明 `qemu.env` 与补好的 QEMU 的位置要并进 `multi-host.env`，或 QEMU 门禁先不放行第二台、前提没齐在第二台报本次未跑 | 2.6 |
| 2.8 读本机运行状态 | 同一节第一条 | 把「不读本机运行状态」收窄到「不读会改变判定结局的本机状态」；排程用的峰值表、上次红的清单写明许读、按台各一份 | 2.8 |
| 2.10 两节互相矛盾 | 「判定库只有一份」第一条 | 「那一份」改按服务定义：客户端不读库路径；家目录路径只留给服务一侧与中转库落点 | 2.10 |
| 第三节 17 处 | 各处原文 | 逐处改成「各族在那一份库里的键前缀」或删「各自」二字；库拷贝两处写明是中转库；第 37、40、42、142 行的库位置改写成过渡说明 | 第三节 |
| 4.1 C948 | `.claude/kb/checks-owed.md:605` | 拦法改成「按带库脚本登记表第三列判 + 射程扩到 `scripts/experiment-apparatus/*.rs` 与 `crates/`」；列出「旧文件库目录」的表与「新写」的判法；判别力写两份样本（注释里有「判定库」的与没有的） | 4.1 |
| 4.2 C949 | `.claude/kb/checks-owed.md:606` | 静态格的规则与样本对齐：要么规则写 `~/`、`$HOME`、`expanduser` 并给出今天 55 份怎么分合规与违例，要么样本改成写死 `/home/<用户>/`；动态格写明台记在哪、怎么派到第二台、没库的测试怎么判、第二台用不了时怎么报 | 4.2 |

## 没打中的形状

- replay.sh 的库归属（1.3）：它在 unit-verdicts 族里，开关开着进同一份库。试的形状：找它另开目录、TSV 的地方；读了它文件头判定库那一段（第 52–68 行）与自证那一段。
- GPU（2.7）：试的形状：GPU 测试读本机独有的卡号、驱动路径；只读了多机配置模板的 GPU 段与 verification.md 的清场段，没读取卡代码。
- crates 变异双机分片的多机那一面（2.9）：试的形状：分片驱动自己连第二台、写死台；读了门禁文件头第 76–110 行。
- 这三格各只抽了一次，按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不支撑「没打中」。

## 这条腿自己的限度

- 没连第二台：QEMU 在第二台有没有、第二台树里有没有 `qemu.env`、第二台家目录下有没有库，都是按代码推的。
- 2.1 的 7 份「派不到第二台」是读 `peer_eligible` 与登记表推的，没起 `--verdict-library-route`；命令在下一节。
- 没扫 `.claude/kb/` 与 `records/` 里的现状句；第三节的 17 处只在规则、定义、技能与带库脚本登记表里。
- 禁读清单：本腿一次全仓 grep 开关名时，输出里带出了禁读清单上 `records/2026-11-01-判定库服务化.md` 的一行（第 118 行，讲开关的写法）。之后的 grep 都排除了 `records/` 下那份文件；报告里没有用到那一行的内容（开关的写法取自 `scripts/prune-server/prune_server_client.py` 第 13–17 行与 `task-server/task_server.py` 第 202 行）。
- `multi-host.env.example` 本轮里有别的会话在改（两次 grep 行号不同），引它的地方没写行号。
- 私有值：`systemctl --user status` 的输出里有本机主机名与有线地址，报告里没抄。

## 要主 agent 跑的命令

1. 把 2.1 的「推的」变成量的（不是重型，不跑项目测试，只问路线）：
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/scripts/hosts/multi-host-run.sh --verdict-library-route -- bash scripts/crash/crash-point-replay.sh --quick`
   - 在哪起：仓根。
   - 看哪几行：`TASK_SERVER_RESULT` 那一行 `exit=3` 且 `task-server/task log <号>` 里一行「scripts/crash/crash-point-replay.sh<TAB>…」算 2.1 坐实；`exit=0`（不带判定库）推翻 2.1 里这一份。对另外 6 份（`broad-mutation.py`、`checker-tier-broad-mutation.sh`、`replay.sh`、`crash-amplification.sh`、`device-code-key.py`、`checker-tier-crash-point-replay.sh`）照同一条换 `--` 后面的命令各交一次。
   - 注：这份脚本在 `scripts/hosts/` 下，服务只在本机起它，这正是本腿要核的本机路线，不影响读数。

没写模型，没有复跑脚本。

## 没做什么

- 不判 K1（正推腿那几格）；不替主 agent 采纳。
- 没往统一任务服务交任何任务；上面那条命令交主 agent。
- 没连第二台，没跑任何项目代码；条数全是在源文件上 grep / python 现数的。
- 草稿目录里只落了一份 grep 列表 `/home/fy5090/.local/share/singlefs/work/single-library-defs-r1-attack/home-path-files.txt`（55 行，4.2 引的那一份），没建仓副本、编译目录与库拷贝。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/single-library-defs-r1-opus-output.md --background research/prompts/_single-library-defs-r1-background.md` 末行（本腿交回前现跑）：「✓ 核了 85 处引文、21 处裸行号，对上 85 处，没判 21 处（其中裸行号没有引文可比 21 处；逐处列在上面）」。21 处裸行号是同一份文件里已带引文的那几行的复指。
