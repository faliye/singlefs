# single-library-defs-r2：「判定库只有一份」「所有测试多机可用」第二轮正文（2026-11-01）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、题面

第一轮判决 `research/prompts/single-library-defs-r1-main-verification.md` 判两条核心策略写进定义的第一版打穿，照打中的改回（判决第四节 11 条改法）。这一轮判改后的形态：

- `.claude/rules/verification.md`「判定库只有一份：全仓统一写进 prune-server 的那份 RocksDB」「所有测试都要多机可用，不许依赖某一台」两节（整节重写）。
- `.claude/agent-common.md`「写」一节「不新建判定库」「所有测试都要多机可用，不许依赖某一台」两条。
- `.claude/main-agent.md`「禁止」一节「判定库只有一份，不许各自建库」「所有测试都要多机可用，不许依赖某一台」两条。
- 随判决改掉的旧句：`verification.md` 第 39、57 行，`.claude/rules/implementation-workflow.md` 的「各自的判定库」，`.claude/rules/test-script-discipline.md`「判定库放哪」那一句，`.claude/skills/gate/staged-gate.md`、`.claude/skills/crash-test/commit-stages.md` 的「查自己的判定库」，`main-agent.md` 的「库拷贝」与「各自的判定库」，`agent-common.md` 的「库拷贝」。
- 欠账 C948、C949 在 `.claude/kb/checks-owed.md` 重写的「怎么拦」。

用户在第一轮之后定的三条，改后的文字按它们写：「不加任何参数 删掉布局选择 只能用 rocksdb 除非 rocksdb在功能和设计上不可用」「不得原地打开。 迁移完成后 数据无误，删掉原来的数据库。 测试这里所有的只要功能满足都要走 rocksdb」「把库合并到一起 用rustdb」。

## 二、实现今天的样子（主 agent 的观测，2026-11-01）

方案按 `crates/`、`scripts/`、`prune-server/` 今天的实现来谈（主工作区现状）：

- 服务 `prune-server/`：本机常驻一份，状态目录 `~/.local/state/singlefs/prune-server`，库在它下面的 `library`；布局改成只有一种（全部表在默认列族、键原样、开库选项照 `crates/singlefs-checker-tier/src/verdict_store.rs`）与迁移旧库的子命令正在写（调度记录 `records/2026-11-01-判定库服务化.md` 批次 9），还没编过。
- 客户端：Python 三家经 `scripts/prune-server/prune_server_client.py`（开关 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`，文件头「键空间」），自证全绿；Rust 崩溃库经 `prune_server::reliable_client`（补丁已打进 `verdict_store.rs`、`crash_library.rs`，`PRUNE_SERVER_KEY_NAMESPACE` 已改成空、键原样），还没编过。
- 旧库：崩溃判定 RocksDB 在 `crash_library_directory`（24 GB）；三家文件库在主仓 `.git/` 下的 `singlefs-*` 目录；verdict-library 一家 7200 条已导进服务并 `--verify` 整份相同。
- task-server：`task-server/task_server.py` 的 `LOCAL_ONLY_DIRECTORIES`、`LOCAL_ONLY_WRITERS`、`LOCAL_RESOURCE_ENTRIES`、`PEER_ONLY_ENTRIES` 四张表与 `VERDICT_SERVICE_RELEASED` 名单（带开关的 Python 三家放行派第二台，崩溃三道不放）。

开工快照：`research/prompts/single-library-defs-r2-snapshot/defs-sha256.txt`（八份被判与随判决改过的文件）、`defs-vs-head.diff`。

## 三、格

### K1　第一轮判决够不够得着、改法落没落（辩方）

- 判决第二节逐格：腿打中的那一条，判决采了之后第四节那一处改法有没有真落在那一格上（对着 `defs-vs-head.diff`）；判决翻掉攻方两条（「本机服务开的不是家目录那一份」「现在时写的是目标态」）靠的是用户后续定案，翻得对不对。
- 判决「留着的 4 处」（`verification.md:40`、`:42`、`:142`，`commit-stages.md:45`）按新节「还没接进服务的族接进之前照旧」留着，这个理由够不够得着那四处。
- 判决里没采、没写去向的腿的发现有没有（逐条对两份腿报告）。

### K2　改后的形态哪一格会错（攻方）

- 「判定」的新定义（按单元键存、给下一趟同键复用的结论，连同组成清单与判定向量）罩不罩得住今天各家真写的东西（admission 的 items/、compositions/、flow-lines/，crates 变异的 coverage/、timings/、target-occupancy.tsv，崩溃库的录入块与事实表）；哪些按定义不算判定却照旧散在各处。
- 「中转库不算另开」会不会成后门：每进程库、片库、第二台一趟的片库「并回即删」谁来判、没并回时怎么红；「自证与压测给临时服务起的库只在那一格里活」怎么与真服务的库分开。
- 「服务没把它派到第二台的……不算写完也不算违例」会不会让多机那一条空转：第二台长期用不了时，是不是所有新测试都停在「本次未跑」；「只影响排程的本机状态许读」的边界谁判。
- 过渡期开关：「没带的接完前算过渡、接完后算违例」——「接完」谁宣布、怎么判；过渡期里本地文件库仍被写，与「旧库迁完之前只读不写」是不是矛盾。

### K3　C948、C949 的新写法立得成立不成（攻方）

- C948 按带库脚本登记表第三列判：登记表 `.claude/gate.d/verdict-library-scripts.tsv` 第三列今天写的是各族目录，改成「prune-server」要谁改、什么时候改；「中转库与自证库按文件头登记放行」的登记形态有没有定；两份判别力样本能不能真让格转红。
- C949 动态格以 `host=peer` 为据：服务那一侧「哪几台判过」一列今天没有，加这一列要改哪里；静态格的滞后表怎么建、`~/`、`$HOME` 的 55 份存量怎么分合规与违例。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K1 | 复核第一轮判决：够不够得着、改法落没落、翻掉的两条翻得对不对、有没有漏去向 |
| 云端攻方（Opus） | K2、K3 | 攻改后的形态与欠账新写法；不重复第一轮攻过的角度（开关没人设、E174/E178、崩溃三道只在本机、qemu.env、读本机运行状态、17 处旧句） |

## 五、被判的改动

`research/prompts/single-library-defs-r2-snapshot/defs-vs-head.diff`（八份文件对 HEAD 的 diff）里属于这一题的段：`verification.md` 两节与第 39、57 行，`agent-common.md`「写」两条与「库拷贝」那一条，`main-agent.md`「禁止」两条、第 33 行与第 100 行，`implementation-workflow.md`、`test-script-discipline.md`、`staged-gate.md`、`commit-stages.md` 各一处；`checks-owed.md` 的 C948、C949 两行。同一份 diff 里别的改动是别的会话的，不判。
