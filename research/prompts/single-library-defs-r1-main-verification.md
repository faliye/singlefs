# single-library-defs-r1 判决（2026-11-01）

<!-- doc-lint:not-numbers K1 K2 K3 -->

## 一、核了什么

- 两条腿都交齐：正推 `research/prompts/single-library-defs-r1-sonnet-output.md`（Sonnet，K1）、攻方 `research/prompts/single-library-defs-r1-opus-output.md`（Opus，K2、K3）。没派辩方（第一轮）。
- 开工快照：`research/prompts/single-library-defs-r1-snapshot/defs-sha256.txt`、`defs-vs-head.diff`、`kb-sha256.txt`。腿交齐之后主 agent 照用户后续定案与两腿的改法改了三份被判文件（下面第四节），所以判决前先倒推开工那一版：`git show HEAD:<三份>` 打上 `defs-vs-head.diff`，三份 sha256 与 `defs-sha256.txt` 逐行相同（任务外本机直接核）；kb 那三份经 `scripts/agents/snapshot-tree.py`（任务 2537）倒推，3 份 sha256 对上。
- `scripts/agents/cite-check.py` 两份腿报告对着倒推出的那棵树（任务 2542）：核了 114 处引文、55 处裸行号，对上 59 处，没判 110 处（裸行号没有引文可比 42 处；其余是指到 `scripts/`、`task-server/`、`prune-server/` 下文件的引文，不在快照清单里）。没判的里主 agent 抽核了 4 处（正推 12、13、14 行引 `scripts/prune-server/prune_server_client.py:14`、`prune-server-import.py:20-21`、`task-server/task_server_lib.py:5`），引文与现文逐字相同。腿自己交回前的 cite-check：正推对上 28、攻方对上 85，都没有红行。
- 攻方「要主 agent 跑的命令」：`multi-host-run.sh --verdict-library-route -- bash scripts/crash/crash-point-replay.sh --quick`（任务 2538）交了、还在排，结局另记进调度记录；它只坐实 2.1 那一条，判决不等它（2.1 已由 task-server 监控会话的名单核实：崩溃三道不在 VERDICT_SERVICE_RELEASED 里）。

## 二、各格判定

| 格 | 腿 | 腿的判定 | 主 agent 判 | 去向 |
|---|---|---|---|---|
| K1.1 六处对用户原话 | 正推 | 做的是，有缺口（「判定」没定义、开关没写、replay-row 与 flow-lines 没接、旧库一项主 agent 一处缺） | 采 | 第四节 1、2、4、8 |
| K1.1 处 5「写完」 | 正推 | 不是：要子 agent 做它不许做的派发 | 采 | 第四节 6 |
| K1.1 处 4「一律不登那张表」 | 正推 | 与 LOCAL_ONLY_DIRECTORIES 现状冲突 | 采 | 第四节 7 |
| K1.2 三处宽窄 | 正推 | 判定库一致（主 agent 一处窄）、多机宽窄交错 | 采 | 第四节 1–3 对齐 |
| K1.3 只写怎么做 | 正推 | 守；「是旧物」「还没判」两处边缘 | 采 | 第四节 2 删「是旧物」 |
| K2.1 判定库冲突点 | 攻方 | 打中：开关没人设、E174/E178 与广谱变异没罩住、中转库与每进程库字面违规、崩溃接入自己先建每进程库、本机服务开的不是家目录那一份、复用三问自证要空库；replay.sh 没打中（一次抽样，不支撑「没打中」） | 采；「本机服务开的不是家目录那一份」照用户后续定案（不原地打开、迁移后删旧库）翻成：服务自己那一份就是唯一的库 | 第四节 1、3、5 |
| K2.2 多机边角 | 攻方 | 打中：崩溃三道只在本机、「先在第二台跑过」没派法、第二台掉线没写、只在本机的四个目录、真块设备（PEER_ONLY）、QEMU 的 qemu.env、读本机运行状态、两节互相矛盾；GPU 没打中（一次抽样） | 采；qemu.env 那一条不改规则，记进 C949 的前置（QEMU 门禁接服务时一并把 qemu.env 并进多机配置） | 第四节 6、7、9 |
| K2.3 现有规则 17 处冲突 | 攻方 | 打中 | 采 13 处当场改；4 处（verification.md:40、:42、:142，commit-stages.md:45 说各家库此刻在哪）按新节「还没接进服务的族接进之前照旧」留着，接进时随那一家改 | 第四节 10 |
| K3 C948 | 攻方 | 立不成：调用认不出、旧库在 .git 下不被跟踪、样本不清 | 采 | 第四节 11 |
| K3 C949 | 攻方 | 立不成：键里没有台、提交挑不了台、静态规则抓不到样本 | 采 | 第四节 11 |

## 三、与用户后续定案的关系

两腿开工后用户又定了三条，判决按它们改，不按开工时的文字：「不加任何参数 删掉布局选择 只能用 rocksdb」「不得原地打开。 迁移完成后 数据无误，删掉原来的数据库。 测试这里所有的只要功能满足都要走 rocksdb」「把库合并到一起 用rustdb」。所以「prune-server 原地打开的那份 RocksDB」整句作废，改成「prune-server 自己的那份 RocksDB，旧库迁进去、核过、删掉」；攻方 1.7「现在时写的是目标态」与「本机服务开的不是家目录那一份」两条因此不再成立。

## 四、改了什么

1. `.claude/rules/verification.md`「判定库只有一份」整节重写：加「判定」的定义；库是 prune-server 自己那一份、单布局；旧库迁移后删、迁完之前只读；中转库与自证临时服务的库不算另开；交带库任务带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`，没带的接完前算过渡、接完后算违例；还没接进服务的族（广谱变异 TSV、E174、E178）各记欠账；欠账写成 C948。
2. 同文件「所有测试都要多机可用」整节重写：「不读本机状态」收窄到会改变判定结局的；只影响排程的按台各一份许读；只能在一台上跑的活列全四张表（LOCAL_ONLY_DIRECTORIES、LOCAL_ONLY_WRITERS、LOCAL_RESOURCE_ENTRIES、PEER_ONLY_ENTRIES）与已登目录下的自证；「写完」改成主 agent 经服务交、`task wait` 末行 `host=peer` 且结局相同；服务没派到第二台的写「第二台那一趟本次未跑」、记 C949。
3. `.claude/agent-common.md`「写」两条、`.claude/main-agent.md`「禁止」两条照规则对齐（列举补全、「写完」改成写进「要主 agent 跑的命令」、带开关）。
4. 删「是旧物」；C948 的指路补上。
5. 旧句改掉：`verification.md:39`「各道的键与判定库写在它的文件头」、`:57`「各自的判定库」、`implementation-workflow.md:44`、`staged-gate.md:16`、`commit-stages.md:16`、`main-agent.md:100` 的「各自的判定库」→「同一份判定库里自己那一族」；`main-agent.md:33`、`agent-common.md:50` 的「库拷贝」→「中转库（并回服务那一份之后就该删）」；`test-script-discipline.md:66`「判定库放哪照 command-safety 跨轮缓存那一句」→ 指到「判定库只有一份」。
6. 「写完」一句照 K1.1 处 5 与 K2.2 改（第 2 条）。
7. 「一律不登那张表」改成「新写的测试一律不登；已登目录下的自证算只能在一台上跑的活」。
8. replay-row 与 flow-lines：规则不点名，归「还没接进服务的族各记欠账」；接进归批次 6 的后续（调度记录）。
9. 「两节互相矛盾」：「只有一份」不再点名家目录路径，矛盾消失。
10. 留着的 4 处（见第二节 K2.3）。
11. `.claude/kb/checks-owed.md` C948、C949 的「怎么拦」照 K3 重写（三语）：C948 按带库脚本登记表第三列判、射程到装置与 crates、中转库与自证库按文件头放行、旧库删后还在判红、判别力两份样本；C949 动态格以 `host=peer` 为据、没派到的报本次未跑、台记在服务那一侧、静态格规则与样本对齐并给滞后表。

## 五、这一轮的结论

两条核心策略写进三处的文字：第一轮**打穿**（K1 有缺口、K2 三条打中、K3 两条立不成），已照打中的改回；改后的形态被攻过零轮。照 `.claude/rules/three-way-inference.md`「多轮」，再攻两轮、三轮里多数打穿才算打穿，改后的文字这一轮不算过。下一轮的攻击面：改后的「判定」定义、中转库这一条会不会被当成后门、「服务没派到第二台不算违例」会不会让多机那一条空转、C948/C949 新写法能不能真立；派一条辩方腿复核这一轮判决。

## 六、没核动的

- 攻方 2.1 的命令（2538）结局还没出；2.6 QEMU 在第二台的情况是攻方推的，没起 QEMU。
- 两台的实际行为（带开关的真门禁一趟）在调度记录里另核（2516/2517），不在这一轮。
- `rules-lint.sh`、`doc-lint.sh` 对改后文字的判定归提交前整轮门禁。

## 回看决策

不涉及决策：这一轮判的是治理文件（规则、共用约束、主 agent 定义）的两条新条款与欠账的写法，不碰 `.claude/kb/decisions/`。

## 这一轮被判过的文件

- `.claude/rules/verification.md`
- `.claude/agent-common.md`
- `.claude/main-agent.md`
