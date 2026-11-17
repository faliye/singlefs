# m3-close-b20-code-r2 判决：C971 与 C973 代码轮第二轮（2026-11-09）

正文 `research/prompts/_m3-close-b20-code-r2-body.md`；腿报告：辩方 `research/prompts/m3-close-b20-code-r2-sonnet-output.md`（sha256 e851ea2024ae…），攻方 `research/prompts/m3-close-b20-code-r2-opus-output.md`（sha256 865c8fa46cfa…）。开工快照目录：`research/prompts/m3-close-b20-code-r2-snapshot/`。上一轮判决 `research/prompts/m3-close-b20-code-r1-main-verification.md`。

## 这一轮被判过的文件

- crates/singlefs-core/src/recovery.rs：别的池的槽的计数
- crates/singlefs-core/src/mount.rs：乙入判不出多一支别的池的槽、拒因注释
- crates/singlefs-core/src/instance_table_sweep.rs：C973 第 0 代根判据第二份样本

## 一、核查

- 快照倒推（任务 10853）：`task-server/task submit --cwd ~/.local/share/singlefs/work/m3-close-b20-code-r2-judge -- python3 <仓根>/scripts/agents/snapshot-tree.py m3-close-b20-code-r2 --out ~/.local/share/singlefs/work/m3-close-b20-code-r2-judge/r2-tree --repository <仓根>`，「快照 m3-close-b20-code-r2：清单点名 5 份，倒推出 5 份，sha256 对上 5 份」。
- 引文核对：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b20-code-r2-sonnet-output.md research/prompts/m3-close-b20-code-r2-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b20-code-r2-judge/r2-tree --background research/prompts/_m3-close-b20-code-r2-background.md`，「核了 50 处引文、87 处裸行号，对上 21 处，没判 116 处（其中裸行号没有引文可比 49 处）」；没判的是快照外的文件与裸行号，下面用到的逐处现读或由用例量。
- 攻方模型与用例复跑：副本 `attack-rerun-2`。闭式核 `name=genesis_closed_form cells_compared=42354`、`test result: ok. 1 passed`；模型 `ring_pass.out`：`mode=misread` 13 档都没有 `hidden=`（S = 4..=16、三槽同错），`mode=stale` 13 档都有；8 条攻方用例（任务 10563，复跑模板撞内存上限后点名测试目标重交）7 过 1 红，红的是 A3 `the_judge_and_the_product_disagree_on_a_slot_with_an_unknown_format_version_and_wrong_node_bytes`，读数「判官判这一拒判据复算为假；得到 Legal(NewerPublishWitnessedBySystemConfiguration)」。
- 辩方模型复跑：副本 `defense-rerun`，`violations=0`，两个弄坏开关 `never`、`take-short` 各 306969、70434；等价变异 `exclusive` 报 4524，都落在 oracle p4 那一栏（它覆盖可达域之外的组合），p1、p2 都是 0，等价不被推翻。读数 `S=16 l1_longest_false_stop_in_publishes=47 l2_unflagged_newest=2 three_s=48`。
- 变异：任务 10312 判这批 90 行，抓到 88、没红 2（`crates/mutations.tsv:1232`、`:2680`）；调查员探针任务 10641（工作区）与 10642（HEAD）量出两行在 HEAD 上同样不红，取样点不敏感，不是这批带来的；同一探针量出 `crates/mutations.tsv:626` 同样不红。第 7 格「带 --item 的命令与项」就此有结局。

## 二、逐格判决

| 格 | 判 | 依据 |
|---|---|---|
| r1 三处打中与改法 | **站得住**（辩方） | 别池的槽修到了打中那一格；重读粒度改条款不改代码站得住，但 r1「用户原句没说只重读那一槽」要收窄：只对登记第 46 行成立，第 45 行说了「那一槽」，第 46 行「仍是」偏槽级，整个读阶段重读来自 D23 第 397 行与 F1 乙的先例；多拒一侧的域没量，是推的 |
| 取号、F2 退回、逐盘核不读新字段 | **成立**（攻方第三节 grep 原样，辩方同） | 读 `slots_of_another_pool_whose_checksum_verifies` 的只有 `crates/singlefs-core/src/mount.rs:9247` |
| A1 判官把合法的拒判成不合法 | **打中，量的**（用例绿，断言就是这个结局） | 判官 `choose_system_configuration` 不筛 fsid；形态是「判别子观测不到」：`judge_writable_mount_refusal` 手里没有本池 fsid。盘面手造，崩溃走不到（`crates/singlefs-core/src/make_filesystem.rs:694`）。修判官，产品不动 |
| A2 不该有的拒判官判合法 | **打中，量的** | 同一处；今天产品在这一形不拒，是判别力缺口。同一个改法 |
| A3 判官与实现查一槽的次序不同 | **作废** | 攻方写明「某一条红就是我推错了那一格」，用例红，判官给 Legal |
| B1 别池世代号大、每块盘都有：可写拒、只读与恢复 NoValidRoot、离线救得回 | **打中，量的** | 用例 `newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only` 绿，手造。用户定「条款写明只剩离线」 |
| B2 丢写之后一块盘两槽都自证过，乙不判，倒挂放行 | **打中，量的**（对照盘面拒） | 用例 `a_device_holding_two_stale_self_verified_slots_lets_the_inverted_first_txg_through` 与 `the_same_faults_are_refused_when_device_zero_is_not_behind` 绿。分辨臂：乙直拒与甲不中（推的）。纯崩溃走不到（推的，读代码，要落后两次轮换）。实现照条款，不是实现错。用户定「写进射程不罩」 |
| B3 别池又布局位不认的槽 | **打中（线索），量的** | 用例 `a_slot_of_another_pool_with_unknown_incompat_bits_lets_the_inverted_first_txg_through` 绿。登记第 47 行两句相撞；这一槽要有别的版本布局的池，第一版没有。登记欠账，条款定优先次序时一起 |
| C973 判据第二份样本 | **没打中（单故障，错位读），两份样本都没打中** | 攻方证明加闭式核加模型 `misread`；辩方独立模型 `violations=0`。`mode=stale` 的 `hidden` 非 0 不在 C973 射程（读回这一槽上一次的内容），记欠账 |
| C973 射程句 | **打中（轻，两条腿）** | 不要说谎的设备，一次崩溃就走得到；「最多停一圈」要加「跳号不重复」，模型 `sweep_goes_through S=4` 跳 `[1]` 从 13 起、`[1, 13]` 从 25 起、`[1, 13, 25]` 从 37 起放行（`ring_pass.out`）；单次白停 3S−1 次（辩方 `l1=47`，S=16），不是 3S。改射程句，代码不动 |
| C973 丢写加错位读（辩方零轮） | **交欠账，零轮** | `l2_unflagged_newest=2`（模型），后果是可能删现行实例那一行（推的）；要两处故障叠加、T 恰好 3S、实例在环里只有这一条根，超出单故障承诺。写进 C973 射程并登记 |
| 随机历史新步与理想模型 | 没打中 | 模型那一步答拒是常量（`crates/singlefs-verification/src/model.rs:8052`），前提用被测函数算（`crates/singlefs-verification/src/history.rs:7926`），坏了会悄悄跳过，记欠账；只有全零一种读坏进随机历史 |
| 检查单八格 | 成立 | 两份报告；第 7 格由任务 10312 定结局；第 2 格 `RootRingSlot` 裸整数照 r1 记下一批，辩方旁注同 |

## 三、C971 之后全量红 20 条（任务 10298）的判定

调查报告 `~/.local/share/singlefs/work/m3-close-b20-c971-reds/result.md`。二分（任务 10381–10391，副本里乙恒假）全绿，20 条都由乙带出，没有第三类原因。换造法（任务 10392–10397）全绿。B7（10398）第 8、20 两态被拒，只读都做成。B8（10399）不带读故障那一遍就拒。B9（10400）、B10（10401）拒因如调查报告所算。C 组（10402）写一轮与写两轮 `second_mount`、`offline_way_out`、`mount_after_the_way_out` 三行相同。

## 四、用户定案（2026-11-09 弹窗）

| 问 | 定 |
|---|---|
| 取号之后两块盘最新一槽持续坏 | 「拒，留两轮」：判拒、只读可读，改 D22 第 185 行与用例；取号写两轮不动，开欠账判去留；变异行 4837 改用写数断言证红 |
| E205 | 「退场」：删装置、只供测试的开关、开关测试目标、`replay.sh` 两行与点到它们的变异行；实验页标退役，产物与代价数留在 git 历史 |
| 丢写倒挂（B2） | 「写进射程不罩」：归 C6（块层语义假设写错） 那一类丢写，立用例钉现状，代码不动 |
| 别池世代号高（B1） | 「条款写明只剩离线」：D22 写明，立用例，代码不动 |

## 五、被攻过零轮的

判官加 fsid 的改法；B1、B2、C973 射程句的条款写法；取号这一形的 D22 新写法。第三轮这一题不再开：C971 判法本身已经过 r1、r2 两轮，多数打穿的三处都已处置，下一次代码轮攻这一批修补。

## 六、去向

| 事 | 去向 |
|---|---|
| 判官加本池 fsid（A1、A2）、攻方用例入库 | 下一批 `implementation-writer` |
| 15 条换造法、#15、#16、#17，变异行 1232、626、2680 换点新用例，4837 改写数断言 | 下一批 `implementation-writer` |
| E205 退场（代码、测试、变异行） | 下一批 `implementation-writer`；`replay.sh` 与 `scripts/experiment-apparatus/device-code-key.py` 第 34–35 行例子归主 agent 或工具员 |
| D22 已定项 8、C971、C973 射程句、D23 第 424 行、E205 页标退役、新欠账（两轮去留、B3 优先次序、`mode=stale`、丢写加错位读、模型前提用被测函数算） | 写回批 `kb-scribe` |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 8 | 支撑 | 2026-11-09 改了：取号这一形判拒、别池世代号高只剩离线、丢写倒挂不罩写进射程 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-11-09 不受影响：第 3 条不变，第 424 行的现状句归写回批 |
| D16（发布语义） 已定项 6 | 不影响 | 2026-11-09 不受影响：判据写法在欠账表 |
