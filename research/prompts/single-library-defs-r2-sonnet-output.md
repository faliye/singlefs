# single-library-defs-r2 辩方腿报告（K1：复核第一轮判决够不够得着、改法落没落）

范围：只做正文第三节 K1。没读禁读清单里的文件（`research/prompts/single-library-defs-r2-opus-output.md`、`research/prompts/single-library-defs-r2-opus-model/`、`records/2026-11-01-判定库服务化.md`、`research/prompts/single-library-defs-r2-main-verification.md`）。不改仓里任何文件，没往统一任务服务交任务，没写模型。

## 开工核的

- 八份被判文件此刻的 sha256 与 `research/prompts/single-library-defs-r2-snapshot/defs-sha256.txt` 逐行相同（`sha256sum -c` 八行 OK，含 `.claude/kb/checks-owed.md`），所以下面的行号既是此刻文件的行号，也是快照的行号。
- 每句引文写进来之前在被引文件里 `grep -nF` 过一次（草稿目录 `/home/fy5090/.local/share/singlefs/work/single-library-defs-r2-defense/` 里 `locate.py`、`locate2.py` 逐条打行号，命中 1 次才写成引文）。
- 判决里写的行号在判决那一刻的树上算；`verification.md` 的两节重写多出 2 行，所以判决的 `:142` 此刻是 `:144`（见第三节）。

## 一、第二节每一格：第四节那一处改法落没落

「落了 / 没落 / 落偏」按改后的原句判；辩方判定列用定义里的三档（站得住 / 够不着 / 同样打中别的臂）。

| 第二节的格 | 判决说要改的那一处 | 落没落 | 改后的原句（文件:行） | 辩方判定 |
|---|---|---|---|---|
| K1.1 六处对用户原话：「判定」没定义 | 第四节 1 | 落了 | 「判定指按单元键存下、给下一趟同键复用的结论（判绿、判红、拒算的记录，连同它的组成清单与判定向量）」（`.claude/rules/verification.md:72`） | 站得住 |
| 同格：开关没写 | 第四节 1 | 落了（规则句），示例命令没带 | 「交任务时带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`」（`.claude/rules/verification.md:75`）；主 agent 一侧「交带判定库的任务带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`」（`.claude/main-agent.md:12`）；共用约束只指路：「判定的定义、中转库与过渡期开关都在那一节」（`.claude/agent-common.md:39`） | 站得住；`.claude/main-agent.md:100`、`.claude/skills/gate/staged-gate.md:16`、`.claude/skills/crash-test/commit-stages.md:16` 里的 `task-server/task submit -- …` 示例仍不带开关，判决没要求改，不算没落 |
| 同格：旧库一项主 agent 一处缺 | 第四节 3 | 落了（指路式） | 「判定的定义、中转库、旧库迁完即删、过渡期开关都在那一节」（`.claude/main-agent.md:12`）；子 agent 一侧写了规则本身：「并回即删的中转库与自证临时服务的库不算；旧库迁完之前只读不写」（`.claude/agent-common.md:39`） | 站得住；主 agent 一处没写「只读不写」本身，只指路 |
| 同格：replay-row 与 flow-lines 没接 | 第四节 8（「规则不点名，归……各记欠账」） | 没落 | 「还没接进服务的族（广谱变异的 TSV、E174 与 E178 装置自己的库）各记欠账，接进之前照旧，不算这一条的例外」（`.claude/rules/verification.md:72`）：括号里没有 replay-row 与 flow-lines；C948 的前置只写「先把 Python 三家与崩溃判定库全接进 prune-server」（`.claude/kb/checks-owed.md:605`） | 够不着：判决自己写的去向是「规则不点名，归「还没接进服务的族各记欠账」」（`research/prompts/single-library-defs-r1-main-verification.md:40`），可仓里没有一行欠账点这两样，见第四节 4.1 |
| K1.1 处 5「写完」 | 第四节 6 | 落了 | 「把在第二台跑一趟的命令写进报告「要主 agent 跑的命令」一节，由主 agent 交服务，`task-server/task wait` 末行 `host=peer` 且结局与本机相同才算写完；只在本机绿过的不算」（`.claude/agent-common.md:40`） | 站得住：子 agent 现在只写命令，不派发 |
| K1.1 处 4「一律不登那张表」 | 第四节 7 | 落了；其中「列全四张表」落偏 | 「新写的测试一律不登那几张表；表里已登的目录（`task-server/`、`scripts/hosts/`、`.claude/hooks/`、`scripts/agents/`）下的自证算只能在一台上跑的活」（`.claude/rules/verification.md:82`）；表名「`LOCAL_ONLY_DIRECTORIES`、`LOCAL_ONLY_WRITERS`、`LOCAL_RESOURCE_ENTRIES`、`PEER_ONLY_ENTRIES`」同在 `:82` | 站得住（一律不登的冲突清掉了）；「列全」不成立：`task-server/task_server.py:245` 还有 `LOCAL_ONLY_WRITER_DIRECTORIES`，`:226` 还有 `LOCAL_ONLY_ARGUMENTS`；开工时（我第一次读）还有 `LOCAL_ONLY_CARGO_BINS`（登着读本机崩溃判定库的二进制 `crash_library_reader`），写这份报告时已被别的会话删掉，`task-server/` 不在开工快照里，是活文件，行号会动。句子是「那几张表（……）」的括号式，读起来是穷尽的 |
| K1.2 三处宽窄（判定库） | 第四节 1–3 对齐 | 落了 | 管谁的列举两处逐字相同：「轻型、重型、崩溃几道、形式化、lkmm、QEMU、崩溃点重放快档几格、模型对拍、变异表、广谱变异、实验装置的判定全部写进它」（`.claude/rules/verification.md:72`；`.claude/main-agent.md:12` 同串，后者写「全部写进 prune-server 自己的那份 RocksDB」） | 站得住 |
| K1.2 三处宽窄（多机） | 第四节 1–3 对齐 | 落了，余差很小 | 规则「每条测试、门禁、入口脚本、实验装置在两台里任何一台都要跑得起、判得对」（`.claude/rules/verification.md:81`）；共用约束「写测试、自证、门禁、入口脚本、实验装置时不读会改变判定结局的本机独有状态」（`.claude/agent-common.md:40`）；主 agent 「审 diff 时见到测试、自证、门禁、入口脚本、装置读会改变判定结局的本机独有状态、写死主机名与地址、只在本机才跑得起的，一律打回」（`.claude/main-agent.md:13`） | 站得住；余差：主 agent 一处写「装置」不写「实验装置」，且只有它多一类「只在本机才跑得起的」；「只在本机绿过的不算」只剩共用约束有，规则一处的写完句改成「结局与本机相同」（`.claude/rules/verification.md:83`），等价 |
| K2.1 判定库冲突点：开关没人设 | 第四节 1 | 落了 | 同上（`.claude/rules/verification.md:75`）；还写了没带的结局：「没带的走旧文件库，接完之前算过渡，接完之后算违例」（`.claude/rules/verification.md:75`） | 站得住 |
| K2.1：E174、E178、广谱变异没罩住 | 第四节 1（各记欠账，欠账写成 C948） | 括号落了，「各记欠账」没落 | `.claude/rules/verification.md:72` 括号点了三样；C948 一行（`.claude/kb/checks-owed.md:605`）讲的是「怎么拦」，前置只管 Python 三家与崩溃库；`.claude/kb/checks-owed.md` 里没有另一行记这三样接进服务（全文件现搜：flow-lines、E174、E178 0 行；replay-row 只命中第 484 行 C777，是别的题；广谱变异命中第 317、365、549、550、552 行，都是别的题；prune-server 只命中第 605、606 行） | 够不着：「各记欠账」落在一个并不记接入的欠账上 |
| K2.1：中转库、每进程库、崩溃接入自己先建每进程库 | 第四节 1 | 落了 | 「中转库不算另开：每进程库、分片的片库、第二台一趟的片库这类只在并回服务那一份之前存在、并回即删的库；自证与压测给临时服务起的库只在那一格里活、格完即删」（`.claude/rules/verification.md:74`） | 站得住（谁判「并回即删」是第二轮攻方的格，这里不判） |
| K2.1：本机服务开的不是家目录那一份 | 第四节 1、翻成「服务自己那一份就是唯一的库」 | 落了 | 「全仓只有一份判定库：prune-server 自己的那份 RocksDB」一句在 `.claude/rules/verification.md:72`；「旧库不原地打开、不并存」`.claude/rules/verification.md:73` | 站得住，细节见第二节 |
| K2.1：复用三问自证要空库 | 第四节 1（「自证与压测给临时服务起的库」） | 落偏 | `.claude/rules/verification.md:74` 只放行「给临时服务起的库」；Rust 一侧的一次性库不是这一种：「一次性的库（[`DisposableLibraryDirectory`]，流水线单测从空库起跑用）除外」（`crates/singlefs-checker-tier/src/crash_library.rs:12`），两份测试还写「各写一份自己的库（家目录下流水线单元测试那一类里以用例标签命名、起跑先清空、跑完留着）」（`crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs:1514`、`crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs:3951`）——它们跑完留着、不并回，既不是中转库也不是临时服务的库 | 同样打中别的臂：攻方腿原话「Rust 一次性库与服务自己的自证照旧是另开的 RocksDB」（`research/prompts/single-library-defs-r1-opus-output.md:98`），判决只接了服务自证那一半 |
| K2.1：replay.sh「没打中」 | 判决写「一次抽样，不支撑「没打中」」 | 判决的保留站得住 | `scripts/prune-server/prune-server-import.py:20`「replay-row（replay.sh 自己的那一份，没换存法）」；攻方腿 1.3 说它「开关开着就进服务」，与此相反 | 站得住：这一格不能记「没打中」，replay-row 开关开着也写文件 |
| K2.2：崩溃三道只在本机 | 第四节 6、7 | 落了（通则） | 「服务没把它派到第二台（第二台用不了、配置写的是单机、入口登在只在本机的表里）的，报告里写明「第二台那一趟本次未跑」，不算写完也不算违例，记进欠账 C949」（`.claude/rules/verification.md:83`）；崩溃三道其实靠放行名单留在本机：「崩溃判定库那几道（crash-point-replay、crash-amplification、checker-tier-crash-point-replay）不在这里，照旧只在本机」（`task-server/task_server.py:211`）（活文件，开工时在第 203 行），括号三种原因里没有这一种，靠开头「服务没把它派到第二台」兜住 | 站得住（通则罩得住，括号不全） |
| K2.2：「先在第二台跑过」没派法 | 第四节 6 | 落了 | 「由主 agent 经服务交一趟、被派到第二台跑过（`task-server/task wait` 末行 `host=peer`）且结局与本机相同才算写完」（`.claude/rules/verification.md:83`） | 站得住 |
| K2.2：第二台掉线、配置写单机 | 第四节 6 | 落了 | 同 `.claude/rules/verification.md:83` 引文；C949 同写：「服务没派到第二台（第二台用不了、配置写单机、入口在只在本机的表里）的报本次未跑、点名入口」（`.claude/kb/checks-owed.md:606`） | 站得住 |
| K2.2：只在本机的四个目录、真块设备（PEER_ONLY） | 第四节 7 | 落了 | 目录那一半见 `.claude/rules/verification.md:82`；真盘一半：「只能在一台上跑的活（装服务、改本机工作区、读本机私有库导入、量某一台硬件的装置）不算测试」（`.claude/rules/verification.md:82`）加表名里的 `PEER_ONLY_ENTRIES` | 站得住（表名不全见上） |
| K2.2：QEMU 的 qemu.env | 判决：「不改规则，记进 C949 的前置（QEMU 门禁接服务时一并把 qemu.env 并进多机配置）」（`research/prompts/single-library-defs-r1-main-verification.md:22`） | 没落 | C949 前置只有「先把 task-server 派第二台放开到全部测试（判定库经 prune-server 之后），再立格」（`.claude/kb/checks-owed.md:606`）；中英日三份 `checks-owed` 的 C949 一行都不含 `qemu.env`；`defs-vs-head.diff` 里 `qemu.env` 出现 0 次；规则、共用约束、主 agent 定义里也没有 | 够不着：判决写了去向，仓里没有 |
| K2.2：读本机运行状态 | 第四节 2 | 落了 | 「不读会改变判定结局的本机独有状态（本机家目录下的旧判定库、只在本机的设备），只影响排程的本机状态（内存峰值表、上次红的清单）按台各一份、许读」（`.claude/rules/verification.md:81`） | 站得住；边界谁判是第二轮攻方的格 |
| K2.2：两节互相矛盾 | 第四节 9 | 落了 | `.claude/rules/verification.md:81` 把「本机家目录下的库」改成「本机家目录下的旧判定库」，「只有一份」一节里家目录路径只剩作迁移源（`.claude/rules/verification.md:73`）。读那一半没补：「要存判定经 prune-server 写进那一份库」（`.claude/rules/verification.md:81`）仍只说存，正推腿 1.2 的「而那一句只说存、没说读」（`research/prompts/single-library-defs-r1-sonnet-output.md:55`）未接 | 站得住（矛盾清了；读没写） |
| K2.3 现有规则 17 处冲突：「采 13 处当场改；4 处留着」 | 第四节 5、10 | 数对不上：第四节 5 列了 9 处改动，加两处靠别的改法顺带解掉，留着 4 处，另有 2 处（`.claude/rules/verification.md:37`、`.claude/gate.d/verdict-library-scripts.tsv:25`）既没改也没算进留着 | 见下面 17 处逐条表 | 够不着：「13 处」比实际改的多 2 |
| K3 C948 | 第四节 11 | 落了（三语） | 「门禁一格，按带判定库脚本登记表 `.claude/gate.d/verdict-library-scripts.tsv` 判：第三列「判定库放哪」不是 prune-server 的行判红（接进之前登记为过渡的行报本次未跑）；射程连 `scripts/experiment-apparatus/*.rs` 与 `crates/`」；「中转库（并回即删）与自证临时服务的库按文件头登记放行」；「迁完删掉之后还在就判红」；「判别力两份样本」（均 `.claude/kb/checks-owed.md:605`）；en、ja 两份同在第 605 行有这一行 | 站得住 |
| K3 C949 | 第四节 11（连同 qemu.env 前置） | 落了，qemu.env 没落 | 「动态一格：新写或改过的测试与自证每条由主 agent 经服务交过一趟、`task wait` 末行 `host=peer` 且结局与本机相同才绿」；「台记在服务那一侧按键存的「哪几台判过」一列（要先给判定格式加这一列）」；静态格列了 `~/`、`$HOME`、`expanduser`，存量「登进只缩不涨的滞后表」（均 `.claude/kb/checks-owed.md:606`） | 站得住，除 qemu.env |

### 17 处逐条（对着开工快照与此刻的文件；17 处的编号取攻方腿第三节表，`research/prompts/single-library-defs-r1-opus-output.md`）

| # | 位置 | 此刻的原句（片段） | 状态 |
|---|---|---|---|
| 1 | `.claude/rules/verification.md:37` | 「共用一份崩溃判定库 `${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/crash-state-verdicts`」 | 没改，也不在「留着 4 处」里：无去向 |
| 2 | `.claude/rules/verification.md:39` | 「各道的键与它在同一份判定库里的键前缀写在它的文件头」 | 改了；但「键前缀写在文件头」字面不成立：`.claude/gate.d/*.sh` 里「键前缀」0 次，六道重型门禁各自的文件头也是 0 次，文件头写的是库目录名（例 `singlefs-qemu-verdicts/`，登记表第三列同），键空间的定义只在 `scripts/prune-server/prune_server_client.py` 文件头「键空间」；这一句新引入了一处读者按字面找不到的指路（落偏，小） |
| 3 | `.claude/rules/verification.md:40` | 「这些的判定库放在库根那棵仓的 git common-dir 下（广谱变异的库目录除外）」 | 留着（见第三节） |
| 4 | `.claude/rules/verification.md:42` | 「崩溃判定库的键与库目录都不读仓根：库目录由家目录定」 | 留着（见第三节） |
| 5 | `.claude/rules/verification.md:57` | 「它们各按自己的键前缀把判定写进同一份判定库」 | 改了，落了 |
| 6 | `.claude/rules/verification.md:61` | 「自证里各有一格 `reuse-three-questions`」 | 没直接改；靠第 74 行「自证与压测给临时服务起的库」顺带，Rust 单测一次性库没罩住（上表 K2.1 自证格） |
| 7 | 判决的 `:142`，此刻 `.claude/rules/verification.md:144` | 「判定库 `verdicts.tsv` 每一行带用库目录与仓之外的密钥算的校验列」 | 留着；行号漂了 2 行（第三节） |
| 8 | `.claude/rules/implementation-workflow.md:44` | 「它们各按自己的键前缀把判定写进同一份判定库」 | 改了，落了 |
| 9 | `.claude/skills/gate/staged-gate.md:16` | 「六道各自按每条判定的单元键查同一份判定库里自己那一族的判定」 | 改了，落了 |
| 10 | `.claude/skills/crash-test/commit-stages.md:16` | 「每道自己按每条判定的单元键查同一份判定库里它那一族的判定」 | 改了，落了 |
| 11 | `.claude/skills/crash-test/commit-stages.md:45` | 「广谱变异驱动写的库目录（`${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation-units}`）是驱动本身的行为」 | 留着（见第三节） |
| 12 | `.claude/main-agent.md:33` | 「中转库（崩溃放量、变异跑完留下的每进程库与片库，并回服务那一份之后就该删）」 | 改了，落了 |
| 13 | `.claude/main-agent.md:100` | 「判了的各按自己的键前缀写进同一份判定库」 | 改了，落了 |
| 14 | `.claude/agent-common.md:50` | 「副本、编译目录与运行目录里的中转库（并回服务那一份之后就该删的每进程库与片库）用完就删」 | 改了，落了 |
| 15 | `.claude/agent-common.md:62` | 攻方腿引的是「调用方不写、不判」撞「先在第二台跑过一趟」 | 靠重写写完句顺带解掉（子 agent 只写命令，`.claude/agent-common.md:40`），算落了 |
| 16 | `.claude/rules/test-script-discipline.md:66` | 「判定库只有一份、放在 prune-server 那一侧（`.claude/rules/verification.md`「判定库只有一份」），不随这一趟的临时目录」 | 改了，落了 |
| 17 | `.claude/gate.d/verdict-library-scripts.tsv:25` | 「装置各自的库（E174、E178）按这一份交的代码键存」 | 没改，也不在「留着 4 处」里：无去向（按内容属 E174/E178「接进之前照旧」） |

合计：改了且落了 8（5、8、9、10、12、13、14、16），改了但落偏 1（2），顺带解 2（6 落偏、15 落了），留着 4（3、4、7、11），没改没去向 2（1、17）。判决写的「13 处当场改」对不上：9 + 2 + 2 = 13 里把第 1、17 两处也算了进去，而它们此刻没动。

## 二、翻掉攻方两条：翻得对不对（第 2 条）

判决的依据：用户后续定案「不得原地打开。 迁移完成后 数据无误，删掉原来的数据库。 测试这里所有的只要功能满足都要走 rocksdb」「把库合并到一起 用rustdb」（`research/prompts/_single-library-defs-r2-body.md:15` 抄的原话），判决据此把「prune-server 原地打开」整句作废（`research/prompts/single-library-defs-r1-main-verification.md:29`）。

### 2.1 「本机服务开的不是家目录那一份」

结论：翻得对。改后的文字不再有这条说的毛病。

- 毛病原形是规则把「那一份」定成家目录下的崩溃判定库、写成由服务原地打开，而在跑的服务开的是状态目录下另一份。改后：「全仓只有一份判定库：prune-server 自己的那份 RocksDB」（`.claude/rules/verification.md:72`），「旧库不原地打开、不并存」（`.claude/rules/verification.md:73`），迁完「旧库由主 agent 删掉」（`.claude/rules/verification.md:73`）。家目录那份在规则里只剩迁移源的身份。
- 前提跟着变了：用户定案让服务自己的库成为唯一的库，不是服务去开家目录那份，所以攻方说的「两份 RocksDB 同时有判定」只剩迁移途中一种状态，由第 73 行与第 75 行管。
- 旁证：服务这一侧的说明已经同向：「服务不原地打开别的库，只开自己那一份（状态目录下的 `library`）」（`prune-server/README.md:25`）；「原地打开」一词全仓 `.claude/` 下（`grep -rnF` 现搜）只剩第 73 行那句否定，判定库一节里已没有「独占」。
- 余留的字面毛病：第 73 行写「不并存」，第 75 行写「没带的走旧文件库，接完之前算过渡」，过渡期里两份库本来就并存。「不并存」写重了；这是第二轮攻方 K2 第四条（过渡期开关）的格，这里只记。

什么现象会推翻它：规则或登记表里还有一处把家目录下的崩溃库写成「那一份」（本腿在七份规则与定义文件（不含 `checks-owed.md`）里搜「原地打开」「独占」「crash_library_directory」只命中第 73 行，没搜 `.claude/kb/` 与 `records/`）。

### 2.2 「现在时写的是目标态」

结论：原句翻得对，同类写法没清干净，判决说「不再成立」说早了一半。

- 原句（把还没有的东西写成现在时）确实没了：新的规则句是命令式，带了过渡闸（`.claude/rules/verification.md:72` 的「接进之前照旧」、`:75` 的「接完之前算过渡」）。
- 还留着三处读起来是现状、此刻并不成立的句子：
  1. 「布局只有一种：全部表在默认列族、键原样、开库选项照 `crates/singlefs-checker-tier/src/verdict_store.rs`」（`.claude/rules/verification.md:72`）。主 agent 的观测（转述）：布局改成单种与迁移子命令在写、还没编过（`research/prompts/_single-library-defs-r2-body.md:21`）。仓里 `prune-server/src/` 已有对应的自证格与迁移模块，但服务此刻开的是哪种布局，本腿没起任何东西，记「不核」。
  2. 「迁完之前旧库只读不写」（`.claude/rules/verification.md:73`）与同节第 75 行允许没带开关的走旧文件库写，且 `.claude/rules/verification.md:37`、`:42` 仍写崩溃放量照家目录库目录运行：同一节里前后两句，一句说不写，一句说过渡期照写。
  3. 「全仓只有一份判定库」（`.claude/rules/verification.md:72`）而 replay-row 与 flow-lines 开关开着也写文件（`scripts/prune-server/prune-server-import.py:20`「replay-row（replay.sh 自己的那一份，没换存法）」、`:21`「flow-lines/ 没换存法，不导」），括号里没点（见第一节「没落」一行）。
- 辩护：规则本来就是写目标的文件，命令式加过渡闸的写法站得住；不站得住的只是第 73 行那一句「只读不写」没挂过渡条件。这一条和攻方第二轮的格重合，只给方向：第 73 行那句加「（过渡行除外）」类的限定，或把「只读不写」改成「不新增写入旧库的路径」。

什么现象会推翻它：主 agent 起一趟 `prune-server` 看当前布局已是单默认列族，且崩溃库、Python 三家在没带开关时确实已经不写旧库——那三处就成了现状，我这一节的「没清干净」撤回。

## 三、判决「留着的 4 处」：理由够不够得着（第 3 条）

判决的理由原话：「4 处（verification.md:40、:42、:142，commit-stages.md:45 说各家库此刻在哪）按新节「还没接进服务的族接进之前照旧」留着，接进时随那一家改」（`research/prompts/single-library-defs-r1-main-verification.md:23`）。新节里那句的原文是「还没接进服务的族（广谱变异的 TSV、E174 与 E178 装置自己的库）各记欠账，接进之前照旧，不算这一条的例外」（`.claude/rules/verification.md:72`），括号只点了三样。逐处读原句判：

| 处 | 原句 | 理由够不够得着 | 依据与缺口 |
|---|---|---|---|
| `.claude/rules/verification.md:40` | 「这些的判定库放在库根那棵仓的 git common-dir 下（广谱变异的库目录除外）」 | 部分：括号里的广谱变异够得着；这句的主体不是广谱变异 | 主体管的是 qemu、lkmm、形式化、crates 变异、模型对拍、research-unit-tests 等格，它们属 Python 三家（`scripts/prune-server/prune_server_client.py:14`「不设或设成空串：三家照旧读写本地文件」），已经接了开关，不在「还没接进服务的族」那个括号里。真正罩住它的是另一句：「没带的走旧文件库，接完之前算过渡，接完之后算违例」（`.claude/rules/verification.md:75`）。所以留着是对的，理由引错了条款；这一句本身没挂任何过渡标记，只看第 40 行的读者读到的是规范性位置 |
| `.claude/rules/verification.md:42` | 「崩溃判定库的键与库目录都不读仓根：库目录由家目录定」，后半「分片驱动……另交的库目录（环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY`）只收绝对路径」 | 够不着前半，够得着后半 | 前半说的是崩溃库放量一侧的家目录库，这一族不在第 72 行括号里；第 73 行把它定成「旧库」：「迁完之前旧库只读不写」，而第 42 行（与 `:37`）把它当放量照写的目标。后半「分片驱动另交的库目录」是片库，由「中转库不算另开」（`.claude/rules/verification.md:74`）罩住。这一处要么把崩溃库（Rust 一侧，迁入前）补进第 72 行括号，要么第 42 行前半挂「迁入服务前」 |
| 判决的 `:142`，此刻 `.claude/rules/verification.md:144` | 「判定库 `verdicts.tsv` 每一行带用库目录与仓之外的密钥算的校验列」 | 够得着 | 这是广谱变异的 TSV（第 72 行括号里的第一样），行里写的校验列在服务那一侧没有对应（攻方腿：「校验列搬不过去就丢了门禁 gate-verdict 格逐行核密钥那一层」，`research/prompts/single-library-defs-r1-opus-output.md:66`）。缺口两个：① 行号：判决写 `:142`，此刻 `:142` 是崩溃点重放那一行，广谱变异的在 `:144`；② 「接进时随那一家改」没有去向，校验列的去向没有记在任何一行（见第一节「各记欠账」那一行） |
| `.claude/skills/crash-test/commit-stages.md:45` | 「广谱变异驱动写的库目录（`${SINGLEFS_BROAD_MUTATION_HOME:-~/.local/share/singlefs/broad-mutation-units}`）是驱动本身的行为」 | 够得着 | 广谱变异在括号里。同一行还有「`checker-tier-crash-point-replay` 写进 git common-dir 的判定库是阶段本身的行为」（`.claude/skills/crash-test/commit-stages.md:45`），这半句是 Python 三家，归第 75 行的过渡闸，和 `:40` 同形 |

结论：4 处里 `commit-stages.md:45` 与 `verification.md:144` 站得住；`verification.md:40` 留得对、引的条款不对；`verification.md:42` 前半够不着。

同样打中别的臂：同一个理由对 `.claude/rules/verification.md:37`（放量照家目录库）和 `.claude/gate.d/verdict-library-scripts.tsv:25`（「装置各自的库（E174、E178）按这一份交的代码键存」）一样够得着或一样够不着，这两处判决既没改也没算进「留着」（第一节 17 处表第 1、17 行），登记表第三列里崩溃两行写的也是家目录路径（`.claude/gate.d/verdict-library-scripts.tsv` 里 crash-point-replay.sh 与 crash-amplification.sh 两行）。

什么现象会推翻它：把 `:40`、`:42` 补上过渡标记之后，这两处的旧位置描述与「判定库只有一份」不再被读成冲突，则「理由引错」只是引文问题；如果主 agent 认为「还没接进服务的族」本来就指所有接入进度不到位的家（含 Python 三家的没带开关状态与崩溃库），则 `:40` 与 `:42` 的理由就够得着，我这一节的「引错」撤回，但那样第 72 行括号要改成不限于三样。

## 四、两份腿报告里判决没采、也没写去向的发现（第 4 条）

逐条对了正推腿（`research/prompts/single-library-defs-r1-sonnet-output.md`）与攻方腿（`research/prompts/single-library-defs-r1-opus-output.md`）的每个发现。判决采了且改法落了的不列（见第一节）。没采也没写去向的，或写了去向而仓里没有的：

### 4.1 判决写了去向、仓里此刻没有的

| 发现 | 判决的去向 | 仓里此刻 |
|---|---|---|
| replay-row 与 flow-lines 没接进服务 | 「规则不点名，归「还没接进服务的族各记欠账」；接进归批次 6 的后续（调度记录）」（`research/prompts/single-library-defs-r1-main-verification.md:40`） | 规则括号与 C948 都没点这两样（第一节）；调度记录在我的禁读清单里，那一头有没有记，本腿没读，不核 |
| QEMU 的 qemu.env | 「记进 C949 的前置」（`research/prompts/single-library-defs-r1-main-verification.md:22`） | 没有（第一节 K2.2 qemu.env 一行） |
| 广谱变异 TSV 的校验列去向 | 「接进时随那一家改」 | 没有记在任何一行；攻方腿要求写明去向（`research/prompts/single-library-defs-r1-opus-output.md:66`） |

### 4.2 判决没采、也没写去向的

| # | 发现（出处） | 此刻的文字 | 辩方判定 |
|---|---|---|---|
| 1 | 「核心策略，没有例外」与同一条里的例外子句字面冲突（正推腿：「该处头部同时写着「核心策略，没有例外」，同一条里有例外子句，字面自相冲突。」，`research/prompts/single-library-defs-r1-sonnet-output.md:66`） | 判决第四节没有一项改它，而且复制到了新写的几处：`.claude/main-agent.md:13` 「（核心策略，没有例外；」+「只能在一台上跑的活登进 task-server 的那几张表（那一节点名的），不当测试」；`.claude/agent-common.md:40` 「（核心策略，没有例外；」+「只能在一台上跑的活照那一节登表，不当测试」；`.claude/agent-common.md:39` 与 `.claude/main-agent.md:12` 「（核心策略，没有例外）」+「并回即删的中转库与自证临时服务的库不算」 | 漏了；四处都有。同样打中别的臂：不管走哪一种改法，只要保留这个括注就中，去掉括注或改成「射程以那一节为准」才清 |
| 2 | 共用约束「旧库、旧镜像、旧产物不迁移」与「三家原来的文件库只读不写」会不会互相矛盾（正推腿「没核动的」，`research/prompts/single-library-defs-r1-sonnet-output.md:158`，交主 agent 判） | 此刻 `.claude/agent-common.md:52` 仍写「旧库、旧镜像、旧产物不迁移」，并禁「迁移脚本与补旧行的子命令」；而新节要求「经服务的迁移命令整份拷进服务的库」「经 `scripts/prune-server/prune-server-import.py` 整份导进去」（`.claude/rules/verification.md:73`），`prune-server/README.md:23` 有「## 迁移旧库进服务」一节 | 漏了，而且这次是真冲突的字面：子 agent 写迁移命令按第 52 行是越界。该条的射程（里程碑 17 收尾前的文件系统格式）是否包括判定库，要主 agent 判；至少该在第 52 行或新节里写明一边 |
| 3 | 表名补全（正推腿：「条文还把表写成两张：「LOCAL_ONLY_DIRECTORIES、LOCAL_RESOURCE_ENTRIES」」，`research/prompts/single-library-defs-r1-sonnet-output.md:53`） | 判决采了「列全四张表」，实际常量不止四个：`task-server/task_server.py:226`、`:245`（活文件，开工时是 `:229`、`:242`） | 采了一半（第一节「处 4」一行） |
| 4 | 「三家」未定义（正推腿：「「三家」在六处里都没定义」，`research/prompts/single-library-defs-r1-sonnet-output.md:33`）；新族的键前缀由谁定（正推腿：「新族要登记键前缀由谁定也没写」，`research/prompts/single-library-defs-r1-sonnet-output.md:32`） | `.claude/rules/verification.md:73` 仍写「三家的文件库」，全文件只此一处，没有定义；键前缀归属没写 | 漏了，小；第 73 行把「三家」换成「Python 三家（verdict-library、unit-verdicts、mutation-units）」即可 |
| 5 | 读也要走服务（正推腿：「而那一句只说存、没说读」，`research/prompts/single-library-defs-r1-sonnet-output.md:55`） | `.claude/rules/verification.md:81` 仍只写「要存判定经 prune-server 写进那一份库」 | 漏了，小 |
| 6 | E162 一类「被测对象本身是判定存储」的装置，条款字面同样算「另开一份 RocksDB」（攻方腿：「再连带 E162：本腿量的 `DB::open` 一族调用点在 6 份文件里」，`research/prompts/single-library-defs-r1-opus-output.md:52`） | 新「判定」定义（`.claude/rules/verification.md:72`）是「按单元键存下、给下一趟同键复用的结论」，E162 装置按定义不算，但第 74 行「另开一份 RocksDB 都算」的字面没挂「判定」二字之外的限定 | 漏了去向；按定义能读出「不算」，但没写，审 diff 的人仍会打回 |
| 7 | 装置要接 `reliable_client` 得先加跨工作区依赖（攻方腿：「可装置的 Cargo 工作区 `scripts/experiment-apparatus/Cargo.toml` 的依赖只有 `aes-gcm`、`chacha20poly1305`」，`research/prompts/single-library-defs-r1-opus-output.md:53`） | 没有；E174、E178 接入没有欠账行 | 漏了 |
| 8 | 没库的测试无处记（攻方腿 `research/prompts/single-library-defs-r1-opus-output.md:235`） | C949 动态格以 `task wait` 末行为据（`.claude/kb/checks-owed.md:606`），台记在「按键存的」一列，没键的测试只靠 `task wait`，未明说 | 部分解；一句话写明「无判定库的测试只看 `task wait` 末行」即可 |
| 9 | 静态格匹配词（主机名、第二台路径）只能运行时从多机配置读，不能写进仓（攻方腿 `research/prompts/single-library-defs-r1-opus-output.md:243`） | C949 静态格列了「写死的主机名、第二台路径」判红，没写匹配词从哪来（`.claude/kb/checks-owed.md:606`） | 漏了，小 |
| 10 | `gate.sh --staged` 临时树里 `library_root` 取的是被判那棵树的 common-dir（攻方腿 `research/prompts/single-library-defs-r1-opus-output.md:225`） | C948 的「旧文件库（登记表点名的 `.git` 下 `singlefs-*` 目录……）」没写看哪棵树的 `.git`（`.claude/kb/checks-owed.md:605`） | 漏了，小 |
| 11 | E171 的 6 份 `REPO=` 写死本机仓绝对路径（攻方腿 `research/prompts/single-library-defs-r1-opus-output.md:206`） | 没有单独去向；C949 静态格的「`/home/<用户>/` 绝对路径」加滞后表可以兜住（`.claude/kb/checks-owed.md:606`） | 隐含解；判决没说 |

没有发现被判决漏掉的第三类：两份腿报告里标「没打中」的几格（GPU、crates 变异的多机一面、replay.sh 的库归属）判决都按「一次抽样不算」处理了（`research/prompts/single-library-defs-r1-main-verification.md:21`、`:22`），没有拿它们去支撑结论；其中 replay.sh 那一格本腿用 `scripts/prune-server/prune-server-import.py:20` 现查，反例成立（第一节 K2.1 replay.sh 一行）。

什么现象会推翻第四节：4.2 里任何一行在 `.claude/` 此刻的文件里已经有等价的处置（本腿对每行都 `grep -nF` 过此刻的原句，没搜 `.claude/kb/` 的叙述性文字与 `records/`）。

## 五、交主 agent 的

判决该补的去向，按重要性：

1. 判决第二节 K2.3 一行的「13 处当场改；4 处留着」改成对得上的数：改了 9、顺带解 2、留着 4，另有 `.claude/rules/verification.md:37` 与 `.claude/gate.d/verdict-library-scripts.tsv:25` 两处要么并进「留着」（则是 6 处），要么改；留着的 `:142` 改写成 `:144`。
2. 留着的几处各加一句过渡标记，别只靠第 72 行括号：`.claude/rules/verification.md:40`、`:42`、`:37`（以及 `.claude/skills/crash-test/commit-stages.md:45` 的后半）后面各补「（接进服务之前的位置，见「判定库只有一份」）」一类半句；并把崩溃判定库（Rust 一侧）补进第 72 行括号，或把第 42 行前半改成挂「迁入服务前」。
3. 第 72 行括号补 replay-row 与 flow-lines（`scripts/prune-server/prune-server-import.py:20`、`:21` 已写明没换存法），并在 `.claude/kb/checks-owed.md` 补一行接入欠账（广谱变异 TSV 含校验列去向、E174 与 E178 含跨工作区依赖、replay-row、flow-lines）；C948 的前置现在只写 Python 三家与崩溃库。
4. qemu.env：判决说记进 C949 前置，仓里没有；补进 `.claude/kb/checks-owed.md:606` 的前置列（三语）。
5. 「核心策略，没有例外」括注与例外子句并存（`.claude/main-agent.md:12`、`:13`，`.claude/agent-common.md:39`、`:40`）：改成「射程以那一节为准」，或去掉「没有例外」。
6. `.claude/agent-common.md:52` 的「旧库、旧镜像、旧产物不迁移」与迁移命令：判一边，至少写「判定库的迁移不在此列」。
7. 第 82 行「那几张表」：补 `LOCAL_ONLY_WRITER_DIRECTORIES`、`LOCAL_ONLY_ARGUMENTS`（`LOCAL_ONLY_CARGO_BINS` 此刻已不在），或改成「`task-server/task_server.py` 里 `LOCAL_ONLY_*` 与 `LOCAL_RESOURCE_ENTRIES`、`PEER_ONLY_ENTRIES` 几张登记表」（不要写「列全」）。
8. 第 74 行「自证与压测给临时服务起的库」只罩服务自证；Rust 单测一次性库（`crates/singlefs-checker-tier/src/crash_library.rs:12` 的 `DisposableLibraryDirectory` 与两份测试的「跑完留着」）要么加一句「单测的一次性库（不并回、不留作判定）不算」，要么给它们定删的时限。
9. 此刻文字里该再改的字句（我的发现，不是判决漏的）：
   - `.claude/rules/verification.md:39`「各道的键与它在同一份判定库里的键前缀写在它的文件头」：文件头写的是库目录名，改成「各道的库目录名（对应键空间里的种类）写在它的文件头」。
   - `.claude/rules/verification.md:73`「不并存」与同节第 75 行过渡期并存冲突，改成「不并存写入」之类，或去掉；「迁完之前旧库只读不写」加过渡行限定。
   - `.claude/rules/verification.md:73`「三家的文件库」换成「Python 三家（verdict-library、unit-verdicts、mutation-units）的文件库」；`:81` 「要存判定经 prune-server 写进那一份库」补「读也经它」。

## 要主 agent 跑的命令

没写模型，没有复跑脚本，判决要引的数没有一个是我量的。判决第六节把 `rules-lint.sh`、`doc-lint.sh` 对改后文字的判定归提交前整轮门禁；下面这一条是正推腿上一轮留下的、本腿同样没跑的，不重型，主 agent 想提前看时交：

1. `task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/rules-lint.sh`（仓根起；本腿没读它的文件头，参数与输出判法没核；看 `.claude/rules/verification.md` 70–84 行、`.claude/main-agent.md` 12、13 行、`.claude/agent-common.md` 39、40 行有没有报告）。

## 没核动的

- 此刻在跑的 prune-server 开的是单默认列族还是多列族，本腿没起任何东西，没查 `systemctl --user`，记「不核」（第二节 2.2 第 1 点）。
- 调度记录 `records/2026-11-01-判定库服务化.md` 在禁读清单里，replay-row、flow-lines、qemu.env 是否另有去向写在那头，本腿没读，只能说「规则与 kb 里没有」。
- `.claude/kb/` 其余文件与 `records/` 里描述各家库位置的现状句没扫；17 处沿用攻方腿的 grep 范围加一遍 `.claude/`、`scripts/`、`crates/`、`prune-server/`、`task-server/` 对「各自的判定库」「自己的判定库」「库拷贝」的现搜，现搜命中的 `crates/`、`scripts/`、`task-server/` 文件里都是代码注释或测试描述，不属规则文字。
- 本腿判的是「判决够不够得着、改法落没落」，没有重判 K2、K3 的新攻击面（过渡期谁宣布「接完」、中转库谁判并回、`host=peer` 台列怎么加），那是攻方腿的格。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务、判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 没改仓里任何文件；没读禁读清单；没写模型；没跑 `rules-lint.sh` 与 `doc-lint.sh`。
