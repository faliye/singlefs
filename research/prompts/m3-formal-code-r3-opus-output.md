# m3-formal-code-r3 攻方腿（Opus）报告（2026-10-03）

<!-- doc-lint:not-numbers K1 K2 K3 K4 L1 L2 L3 L4 L5 L6 H1 G1 G2 U1 U2 KF KF3 KF9 KFW KG KA KJ1T SN0 SN1 SB RN RI RNSB T1 TP TV4 O1 O2 O3 S5 A1 A2 A3 B1 B2 B3 -->

被判对象：冻结副本 `/tmp/claude-1000/m3-formal-code-r3/tree/`（开工时 `sha256sum -c --quiet research/prompts/m3-formal-code-r3-snapshot/crates-sha256.txt` 无输出）。所有改动与编译都在草稿目录的拷贝 `/tmp/claude-1000/m3-formal-code-r3-attack/` 上做，冻结副本没改。副本上的数只是副本上的数。

## 复跑

```
bash research/prompts/m3-formal-code-r3-opus-model/tools/rerun.sh /tmp/claude-1000/m3-formal-code-r3/tree <新草稿目录> [spans k1 segs k4t1 inject]
```

脚本把冻结副本 rsync 进草稿目录、加一个原型包 `crates/r3probe`（依赖 core、harness、checker、checker 档库、RefFS），经 `run-with-memory-cap.sh 2G` 与 `capped.sh 6` 编 release，五件各跑一遍，输出与 `outputs/` 同名文件逐字节比（去掉包装行与 `elapsed_seconds` 行）。不跑任何测试二进制，也不跑 checker 档装置。整遍本机挂钟 5 分 21 秒（负载 7 上下）。我在 `/tmp/claude-1000/m3-formal-code-r3-attack/rerun1/` 用这个脚本从头跑了一遍，`outputs/` 就是那一遍的产物。

模型目录 `research/prompts/m3-formal-code-r3-opus-model/` 每个文件的 sha256（`SHA256SUMS` 原样）：

```
ba1eab13615de45450b50a27de07653cc913a7fca936c0ac6da1b9387256ca0d  ./outputs/inject-1.log
f76fdf62f1dac5bb6230431e04d7c7c0ff03840cc18b0c75f36c10cb2bdd1098  ./outputs/inject-2.log
ba1eab13615de45450b50a27de07653cc913a7fca936c0ac6da1b9387256ca0d  ./outputs/inject-3.log
82589c3e04d354ab72f5a8cbe2c79ac6555b32d0b4b874ac2b6ee706831ed1e4  ./outputs/inject-4.log
82589c3e04d354ab72f5a8cbe2c79ac6555b32d0b4b874ac2b6ee706831ed1e4  ./outputs/inject-5.log
1fd01798c86e8d50d0ea857c7fd1bfc061ff7f5b75020c30b4a7a7c3a0bfc96a  ./outputs/inject-6.log
aa5bce9e7297f67899072be402c6818fe4079a557c5ef95a0b3acec6d3350c90  ./outputs/k1-broad-3.log
c46217b617e343165bc05fc6d5f16353a21d9411039960c01c08a41c0f3ba96a  ./outputs/k1-broad-8.log
f70903aad35bcbf0bcf89b4382a408162aa8514f0abc33314c17c937f14428af  ./outputs/k4t1.log
5cdfd9bfc6823c708bf4f882d551b3aef33519724985fb2d30e4c2bc592b8870  ./outputs/segs-rbf-0.log
c82d818cebbdf4eb04f677a61e144d983c6b3203de15a82eed3b5ac436d04d71  ./outputs/segs-rbf-1.log
0fbe6b284d97db207cbb3748a8905eed1144910e97c77e8657f05432b87af79d  ./outputs/segs-rbf-2.log
71badfb6385acf4bb10076e75be617ef1c6c85d7c2f4f11e59afdfdf68c8233a  ./outputs/segs-rbf-3.log
1c524505ba4864af4c193d00811dd2b472a4edead46154577e78ded63b886db8  ./outputs/spans-broad-256-16.log
a709cc719f4e1c1f7ea53b7b4920dc7e0a38d22d75b0e57ef207734c072ad419  ./outputs/spans-broad-4g-8.log
89b325ba23c6fa5d9dc7b44563e4dbcddbc9aa7ef90ef55e1da5d4e232d30e73  ./outputs/spans-rbf-256-24.log
2434a625c4992a65bbd3abc1c83d98f6ffa473b24c898debd03cf05485d854e3  ./outputs/spans-rbf-4g-20.log
f7917665ca3587775f76dc8fd17e90d71b6576564d41893fc2e0fa08f14adb45  ./outputs/spans-reuse-4g-20.log
748a3bd36cae30f6d148bd2132233d5e09be8c0c41b5c62cd3f6017eee593e8e  ./prototypes/r3probe/Cargo.toml
a58e9dd5f162fbbcaa9e4c638ab1cfcd98956986f7aa931df4d67f980a83d251  ./prototypes/r3probe/src/k4t1.rs
09539bd9866bedbe372d60034be1db7e778b4d0a6c0d3db753c499f1f08dd5d6  ./prototypes/r3probe/src/main.rs
870ff50f38384a6b4bb4d754879940cbc53ced50eaa08a3e6383aa62213c0231  ./tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 | 依据在哪 |
|---|---|---|---|
| K1-a 翻译层崩溃时刻按切段屏障归属，在每种步上只有一种读法 | 没打中 | 今天五个取样点 120 段历史、8 113 个候选段里没有一段跨步：每段的写全由关上它的那一步发出，按屏障归属（se）与按第一个被扣下的写归属（fw）在今天的历史上逐段相同 | 第一节 1 |
| K1-b RefFS ∧ O1 在合法历史上 RefFS 先误红 | 没打中 | 崩溃注入调用点原样（它自己的抽样）80 段历史、534 个崩溃状态、1 602 个二次崩溃状态，RefFS 与 O1 全部「两者都绿」，RefFS 一次都没弃权 | 第一节 2 |
| K1-c 该判的弃权（m 表查不到、回退没报目标） | 没打中 | 同一批里弃权 0 次；回退一步只写一条根、是该步最后一条根（rbf 四段历史的段表） | 第一节 2、3 |
| K1-d 层 0 枚举域逐态判 RefFS ∧ O1 | 跑不动，交主 agent | 任何带内容发布的历史都有一段 24 个单元写（2^24 ≈ 1 678 万态），3 步历史 10 段里 9 段 1 678 万到 6 711 万态、8 步一段 5.9 亿态；能跑的那一段 45 态全「两者都绿」 | 第一节 4 |
| K4-a E165 r4 生成器的 T1 时机与今天的可写挂载对不上 | **打中（量过，实现一侧）** | 照登记字面的 T1（这次挂载里对那个落点的第一次读读不出），实现第一次读系统配置槽是在择系统配置那一步、不在见证读里；SN0、SN1、SB、RNSB 的 T1 在实现上都不走重读，生成器给的是「走第二遍」，E165 页「走第二遍重读各 28 466」与登记的 T1 生效自证都是生成器自己的读法 | 第二节 |
| K4-b E165 r4 的 KG 拿掉的两道保护是不是条款里那两道 | 没打中（读码） | KG 关掉的是 N-配置连同取号那一刻、与写行 txg 看记录，与登记 5.3 第 6 条一致；取号、计数器、写行 txg、系统配置轮换的取法与 `mount.rs`、`transaction.rs` 逐条对得上 | 第二节 5 |
| K4-c E171 装置某一列静默不判 | **打中（读码，没跑装置）** | 取号或暖机返回错误时，O1-对拍不问模型（记 not_modelled），之后写落在那两次调用里的态 O1-崩溃判官记「不适用」，S5 只在基线判：这一行进分母、O1 读成「没红」，登记读法 5 末句许诺的「模型说该成功 ⇒ O1-对拍红」在这两次调用上落空 | 第三节 |
| K2 二次崩溃 F 的界按内容认 | 没攻到底 | 只读码；调用点样本里二次崩溃 1 602 态没有新发现；按内容认的界在「抬到的 F 小于盘上已生效的 F」时找不到位置会 panic，这一形今天的历史走不走得到没造出来（推的） | 第四节 |
| K3 `walk.rs` 新 fold 与 `choose_root` 同值 | 没打中（读码） | 两边都按区域主序、槽次序排，键相同都取前一个 | 第四节 |

## 一、K1　RefFS 接调用点（重点）

### 1. 崩溃时刻按切段屏障归属：今天的历史上段不跨步

找的形状：一段写里混着前一步的写、由后一步发的屏障关上。这样的段上，崩溃注入摆的真子集里会有「前一步的根 FUA 没落、后一步的写落了」这种不可达的态（切段规则自己写着「这样不漏可达的崩溃状态，多出不可达的」（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/segments.rs:101`）），RefFS 把它算在后一步里（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:678` 「.find(|(_, stream_length)| *stream_length > closing_stream_index)」），前一步的发布已返回、抬了 lo，恢复读回前一版就判红；O1 判官不按步归属，拿持久的最新根比，照样判绿。只要有一段跨步，RefFS 就会在合法实现上先误红。

做法（`r3probe spans`）：照崩溃注入的接法跑历史（`execute_history_with`、同一个观察者登记每一步跑完时的流长、同一个 `translate`），候选段取与崩溃注入同一个范围（段里每个写都在 mkfs 之后、最后一个跑完的步之前），逐段比「段里每个写落在哪一步」与 `crash_moment_in_segment` 交回的关段那一步。

| 取样点（比重、盘宽、步数、种子） | 产物原行 |
|---|---|
| BROAD、4 GiB、8 步、0–39 | 「name=spans histories=40 completed=40 candidate_segments=1343 spanning=0 spanning_with_an_earlier_root_write=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/spans-broad-4g-8.log:1`） |
| ROLLBACK_AFTER_RAISING_THE_FLOOR、4 GiB、20 步、0–19 | 「name=spans histories=20 completed=20 candidate_segments=1520 spanning=0 spanning_with_an_earlier_root_write=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/spans-rbf-4g-20.log:1`） |
| REUSE_AFTER_RAISING_THE_FLOOR、4 GiB、20 步、0–19 | 「name=spans histories=20 completed=20 candidate_segments=2090 spanning=0 spanning_with_an_earlier_root_write=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/spans-reuse-4g-20.log:1`） |
| BROAD、单元区 256 槽、16 步、0–19 | 「name=spans histories=20 completed=20 candidate_segments=1232 spanning=0 spanning_with_an_earlier_root_write=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/spans-broad-256-16.log:1`） |
| ROLLBACK_AFTER_RAISING_THE_FLOOR、单元区 256 槽、24 步、0–19 | 「name=spans histories=20 completed=20 candidate_segments=1928 spanning=0 spanning_with_an_earlier_root_write=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/spans-rbf-256-24.log:1`） |

结论：今天每一步都以「每块有写的盘都被放行」收尾（段表里每一步最后一段是两盘的系统配置槽写，由这一步自己的屏障关上，例 `research/prompts/m3-formal-code-r3-opus-model/outputs/segs-rbf-0.log:50` 「SEG 53 closing=op6:RollBackWhileMounted writes=2 kinds={"SystemConfigurationSlot:d0": 1, "SystemConfigurationSlot:d1": 1}」），所以按切段屏障归属（H1-收严的 se）与崩溃注入给 O1 用的按段里第一个被扣下的写落在哪一步归属（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/crash_injection.rs:945` 「.find(|(_, operations_so_far)| *operations_so_far > stream_index)」）在今天的历史上逐段落在同一步。翻译层这一格只有一种读法，也就没有误红的入口。

由此推的一句（推的，没在坏实现上量）：se 归属只在「一步返回时还留着没放行的写」的实现上与 fw 分开，也就是 r6 判决里那种「回退在持久之前就返回」的坏实现；它的判别力全在坏实现一侧，合法历史上不添也不减。

什么会推翻：换一种取样点（例：单元区 384 槽、带冷启动比重更高的历史）里 `spanning` 不为 0。

### 2. 调用点原样跑：RefFS ∧ O1 两截全「两者都绿」

`r3probe inject` 直接调 `singlefs_checker_tier::crash_injection::inject_crashes_into_history`（崩溃注入那一截与二次崩溃那一截的 RefFS 都在里面：`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/crash_injection.rs:1112` 「reference_filesystem_history.crash_moment_in_segment(crash_point.segment_index),」），摆法用它自己的 `CrashPointDraw::Sampled`、每段历史 6 个，六件并行（每件 1 线程）。这一项的崩溃状态是被判对象自己的抽样，不是层 0 的枚举域（层 0 跑不动，见 4）。

| 件 | 比重、步数、种子 | 产物原行 |
|---|---|---|
| 1 | BROAD、8、0–19 | 「name=inject histories=20 crash_points=118 second_crash_points=354 new_findings=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-1.log:3`） |
| 2 | BROAD、8、20–39 | 「name=inject histories=20 crash_points=120 second_crash_points=360 new_findings=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-2.log:3`） |
| 3 | BROAD、8、40–59 | 「name=inject histories=20 crash_points=118 second_crash_points=354 new_findings=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-3.log:3`） |
| 4 | ROLLBACK_AFTER_RAISING_THE_FLOOR、20、0–9 | 「name=inject histories=10 crash_points=59 second_crash_points=177 new_findings=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-4.log:3`） |
| 5 | ROLLBACK_AFTER_RAISING_THE_FLOOR、20、10–19 | 「name=inject histories=10 crash_points=59 second_crash_points=177 new_findings=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-5.log:3`） |
| 6 | REUSE_AFTER_RAISING_THE_FLOOR、20、0–9 | 「name=inject histories=10 crash_points=60 second_crash_points=180 new_findings=2」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-6.log:5`） |

每件的票只有「CRASH_IMAGE 两者都绿」「SECOND_CRASH 两者都绿」两行，没有 ABSTAIN 行：合计 534 个崩溃状态、1 602 个二次崩溃状态，RefFS 一次没红、一次没弃权。第 6 件的两条新发现不是 RefFS 的（I-3.1、I-7.9，判红在可写挂载加一次发布那一截），见第五节。

二次崩溃那一截 RefFS 比 O1 严：RefFS 在第一次崩溃判绿之后以读出的内容重新起头、挂载不进操作序列，允许集合只剩那一份（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:887` 「let crash = restarted.crash_after_every_operation_returned();」），O1 允许模型提交过的每一版加挂载写的根（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/crash_injection.rs:2335` 「let mut versions_allowed = committed_versions.clone();」）。合法实现在挂载途中再崩也读回第一次崩溃读出的那一份，1 602 态里没有一态让严的一边先红。

### 3. m 表与「该判的弃权」

- 找的形状：一步写了几条根、它自己那条不是最后一条（发布之后再推一条），m 表就会把它自己的根映到上一版（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:266` 「NewVersionWrittenBy::TheLastRootOfTheStep if is_the_last_root_of_the_step => {」），之后回退到那条根时 RefFS 取错内容、在合法历史上判红。今天没有：rbf 四段历史的段表里回退一步只有一个根槽写（`research/prompts/m3-formal-code-r3-opus-model/outputs/segs-rbf-0.log:49` 「SEG 52 closing=op6:RollBackWhileMounted writes=1 kinds={"RootRecordFua:d0": 1}」），发布一步的根槽写之后只剩系统配置轮换。
- 回退目标根查不到 m（弃权）：目标从盘上根环取，盘上的根都是这条录制流写的（mkfs 那几条按偏移也归进根槽写），2 里 534 态弃权 0 次。
- 抛弃最新根的那次挂载：今天这一步恒被拒（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/history.rs:394` 「C554 乙之后读阶段重读一次仍读不出、取号之前拒可写」），翻译层按「在任何写之前拒」交空事件，RefFS 不重新起头、盘上也没变，前后一致；翻译层文件头说的「以读出的内容重新起头」那一支（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:15`）今天的历史走不到，没被调用点考过。这是覆盖缺口，不是误红。

### 4. 层 0 枚举域：跑不动，照规矩没跑

照定义第 3b 条，崩溃状态要取层 0 的整段枚举（每段整写子集、原地覆写三态）。`r3probe k1` 用 `enumerate_layer0_selecting_versions_observing_each_state` 逐态判 RefFS ∧ O1，每段历史先按 `layer0_state_count_with_torn_in_place_overwrites` 算全量、超过 20 万态的整段不跑：

- 任何带内容发布的步都有一段 24 个单元写（`research/prompts/m3-formal-code-r3-opus-model/outputs/segs-rbf-0.log:9` 「SEG 12 closing=start(AfterFirstFile) writes=24 kinds={"UnitWrite:d0": 12, "UnitWrite:d1": 12}」），一段就是 2^24 − 1 个态。
- BROAD 8 步种子 0：「SKIP seed=0 states=587595905」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k1-broad-8.log:1`）。
- BROAD 3 步种子 0–9：9 段 1 677 万到 6 711 万态不跑，只有种子 6 一段 45 态（没有内容发布）跑了：「JOINT 两者都绿 count=45」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k1-broad-3.log:15`）。

没缩崩溃状态去凑：这一格交主 agent。要在层 0 域上判 RefFS ∧ O1，得是 checker 档的整段枚举（重型）或崩溃放量那一路，我这条腿不跑。

## 二、K4　E165 r4 生成器的 T1 时机与今天的可写挂载对不上（打中）

### 1. 被判对象里许可它的那几句

登记把 T1 写成两句：「T1 = 这次挂载里对那个落点的第一次读读不出、之后每一次读都读得出」（`research/prompts/e165-r4-prereg.md:32`），括号里接着「读阶段第一遍读不出，重读那一遍、取号那一刻的读与写侧的读都读得出」（`research/prompts/e165-r4-prereg.md:32`）。前一句说的是「这次挂载里第一次读」，后一句说的是「读阶段第一遍」。根槽的第一次读就在读阶段第一遍里，两句是同一件事；系统配置槽不是。

生成器照后一句写：故障菜单的注释同前一句（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:12442` 「时机 T1 = 这次挂载里对那个落点的第一次读读不出、之后每一次读都读得出」），字段却是「读阶段第一遍的读视图里读不出的落点（T1 与 TP 相同）」（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:12447`），闸的第一遍拿它判（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:13123` 「if newer_publish_is_witnessed(world, entry.first_read_unreadable, landed) {」），判法里系统配置槽少一槽就判真（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:12952` 「if readable_slot_count < FAULT_SYSTEM_SLOT_COUNT_OF_THE_POOL {」）——SN0、SN1、SB、RNSB 的 T1 因此一律走第二遍。

今天的实现：可写挂载第一次读系统配置槽是在择系统配置那一步（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/mount.rs:4631` 「let system_configuration = choose_system_configuration(&*devices)?;」），读阶段的见证读是之后另一次直接读盘（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/mount.rs:4558` 「let witness = newer_publish_witness(devices, system_configuration, &effective_root, &records);」）。照前一句的 T1，那一槽在见证读时已经读得出，「有一槽读不出」那一支（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/mount.rs:406` 「| WitnessedCounterComparison::SomeSystemConfigurationSlotUnreadOrUnverified => true,」）走不到。

### 2. 量过（实现一侧，`r3probe k4t1`）

池：两块内存稀疏盘（4 GiB），新池新建文件（mkfs、取号实例 1、暖机、第一个文件）之后不再写，相当于 E165 r4 里「挂载 1 写完、挂载 2 开头加故障」那一格的起点。挂载 2 用 `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`，两种时机各跑一遍：甲 = 每个落点只坏这次挂载里的第一次读（登记前一句）；乙 = 一直坏到重读之前、钩子里撤（登记后一句，也是生成器的读法）。

| 落点 | 甲（第一次读坏） | 乙（坏到重读之前） |
|---|---|---|
| 无 | 「name=k4t1 site=none timing=OnlyTheFirstRead outcome=mounted:OnTheFirstRead hook_calls=0 ranges=[] total_reads=393686」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:1`） | — |
| SN0 | 「site=SN0 timing=OnlyTheFirstRead outcome=mounted:OnTheFirstRead hook_calls=0 ranges=[d0@4096:reads=19:failed=1:first_read_seq=Some(3)]」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:2`） | 「site=SN0 timing=UntilTheReread outcome=mounted:OnTheOneReread hook_calls=1」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:3`） |
| SN1 | 「site=SN1 timing=OnlyTheFirstRead outcome=mounted:OnTheFirstRead hook_calls=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:4`） | 「site=SN1 timing=UntilTheReread outcome=mounted:OnTheOneReread hook_calls=1」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:5`） |
| SB | 「site=SB timing=OnlyTheFirstRead outcome=mounted:OnTheFirstRead hook_calls=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:6`） | 「site=SB timing=UntilTheReread outcome=mounted:OnTheOneReread hook_calls=1」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:7`） |
| RN | 「site=RN timing=OnlyTheFirstRead outcome=mounted:OnTheFirstRead hook_calls=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:8`） | 「site=RN timing=UntilTheReread outcome=mounted:OnTheFirstRead hook_calls=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:9`） |
| RNSB | 「site=RNSB timing=OnlyTheFirstRead outcome=mounted:OnTheFirstRead hook_calls=0」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:10`） | 「site=RNSB timing=UntilTheReread outcome=mounted:OnTheOneReread hook_calls=1」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:11`） |

读序：一次挂载一共约 39 万次读，盘 0 那一槽系统配置是第 3 次读、盘 1 是第 4 次（「first_read_seq=Some(3)」「first_read_seq=Some(4)」，同两行），一次挂载里那一槽被读 19 次；根槽第一次读在第 10 次。甲时机下系统配置槽的那一次坏读被择系统配置吃掉，见证读看到四槽都在、不走重读；乙时机下走重读。RN 两种时机都不重读（这一池的最新根由记录补得出），与生成器 RN-T1 同向。

生成器在 KF 的 M = 2 格里把 SN0-T1 记成「走第二遍」28 466 次（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:14692` 「KF SN0-T1 28776 310 28466」），E165 页写成结论「SN0、SN1、SB 三项的 T1 走第二遍重读各 28 466」（`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:748`），登记的作废条款拿它当「T1 生效」的判据：「各是 28 466（不是 0）；不如此走 TV4」（`research/prompts/e165-r4-prereg.md:881`），产物里的自证也是这一条：「E7RESULT name=self_proof item=t1_takes_the_second_read_in_the_gate_on_domain passed=true」（`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:646`）。照登记前一句的 T1 让今天的实现来走，这四项的「走第二遍」都是 0：自证判的是生成器自己的读法，不是实现。

### 3. 打中之后的四句

1. **分不分辨臂**：不分辨 KF 与 KG（Q1 的两条臂）。它打的是「读故障时机 T1 对 TP」那个旋钮的读数、M = 2 故障表的「走第二遍」一列、TV4 的前提与那条自证。对 Q1（承诺内 KJ1、KJ1T 违例）推的是不动：SN、SB 只碰系统配置槽，写行 txg、计数器、父链都从根与记录的读视图算，生成器在两种时机下写出的历史逐项相同；RNSB-T1 在实现上等于 RN-T1（落到哪一版、父是谁随 RN-T1 走），生成器给的是「无故障」那一支，两支在 M = 2 的枚举里本来都在，历史集合不变、只是标签换了位（推的，没在装置上量）。
2. **系统当时看不看得到**：看得到。实现在择系统配置那一步就碰上了这次坏读（第 3、4 次读），后面的见证读是另一次读盘。生成器没有「择系统配置」这一步的模型，第一遍读视图把系统配置槽的 T1 一直带到见证读。
3. **满足的是哪一句**：正文 K4 第一句「（闸、取号、系统配置轮换与 `crates/singlefs-core/src/mount.rs`、`transaction.rs` 对不对得上）」的「闸」那一格；登记一侧是「臂的定义写了两句、前一句推不出后一句」：T1 的定义句（第一次读）推不出括号里的「读阶段第一遍读不出」。
4. **改法在打中的格上还中不中**：

| 改法 | 修哪一格 | 打中的四项（SN0、SN1、SB、RNSB 的 T1）上 | 标 |
|---|---|---|---|
| 甲：生成器第一遍读视图只带根槽的 T1，系统配置槽的 T1 不进见证（照实现的读序） | 「走第二遍」列与 TV4 的前提 | SN0、SN1、SB 的 T1 走第二遍变 0，RNSB-T1 与 RN-T1 同表；TV4 那一条要改写，不然自己触发 | 推的（没改生成器） |
| 乙：登记把 T1 改写成「从这次挂载第一次读那个落点起一直读不出，重读之前恢复」，E165 页那一句注明是这个时机 | 定义句与括号对齐 | 生成器不动，28 466 照旧成立；实现在这个时机下确实走重读（乙时机那四行量过） | 乙时机下的实现读数量过；改写本身零轮 |
| 丙：两种时机都枚举（第一次读坏 / 坏到重读之前） | 两种读法各有一列 | 四项各多一行 | 推的 |

三个改法都只在我的模型上量过实现一侧的读数，被攻过零轮。

什么会推翻：登记或 D23 已定项 14 另有一句把「这次挂载里的读」限定在读阶段之内（我只读了登记第 32 行与生成器；D23 那一项原文没读）；或者实现的择系统配置在另一条路径上不读那一槽（`k4t1` 只走了 `mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread` 这一个入口）。

### 4. 附带：RN、RI 一类 T1 与 TP 读法

根槽的 T1 两句同义：实现第一次读根槽在读阶段第一遍里（第 10 次读），与生成器一致。乙时机的 RN（一直坏到重读之前，但这一池没走重读）实际是 TP：「failed=20」（`research/prompts/m3-formal-code-r3-opus-model/outputs/k4t1.log:9`）——见证没判真时整次挂载都读不出那条根，挂载照常往下走，与生成器 RN-TP 的「最新根由记录补得出时不拒」同向。

### 5. K4-b：KG 拿掉的两道保护、取号与轮换（没打中）

- 生成器写的两道保护：「条款的两道保护开着：可写挂载判 N-配置（实现的取甲）与取号那一刻、写行 txg 看记录（登记 5.3 第 6 条）。KG 关掉。」（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:12179`）。实现对应：写行 txg 取根与记录的大者（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/mount.rs:1165` 「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」），生成器同取（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:13201` 「highest_root_txg.max(highest_record_txg) + 1」）；N-配置与取号那一刻见第 1 小节。两道是登记那两道。
- 系统配置轮换：实现按这块盘读得出的槽世代号最大值加一、写在「世代号 mod 2」号槽（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/transaction.rs:414` 「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」），生成器同（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:12394` 起那一段）。
- 取号：实现取盘上最大实例代号加一（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/transaction.rs:695` 「取号会取到的号 = max(每块盘两槽里全部自证过的槽的实例代号, 根环里全部根记录的实例代号) + 1」），生成器取「之前取过号的挂载数加一」。今天的历史里取号写不摆崩溃点、被拒的挂载不取号，两者逐次相等（推的）。
- 新实例第一条记录的计数器：实现取环里自证过的记录计数器最大值加一（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/mount.rs:4679` 「let next_counter = records」），生成器同取。
- 取号之前逐盘核「这块盘落没落后于这一版」（落后又缺单元或缺根就拒可写）生成器没有；今天的崩溃点里根落了之前记录都落了、记录落了之前单元都落了，落后的盘不缺东西，这一核放行（推的，生成器不建单元，没法在装置上量）。

## 三、K4　E171 装置：取号、暖机出错时 O1 一列静默不判（打中，读码）

### 1. 被判对象里许可它的那几句

- 两条流都从取号、暖机起（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:987` 「if !self.step_acquire_and_warm_up() || !self.step_publish_first_file() {」）。这两次调用不对拍：「/// 取号、暖机两次调用；两次都返回之后模型推一步。」（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:616`），理想模型那一侧也只有一个无条件往前走的入口（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/model.rs:778` 「pub fn acquire_and_warm_up_in_the_make_filesystem_process(&mut self) {」），没有 `answer_*`。
- 取号或暖机返回错误：`call` 记 `err:<成员>`、停流、让模型停在这次调用（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:442` 「if outcome != CallOutcome::Succeeded {」），然后 `step_acquire_and_warm_up` 直接交回 false（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:626` 「let Ok(instance) = self.driver.call("acquire", || acquire_instance(&mut pool)) else {」、`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:636` 「.call("warm_up", || warm_up(&mut pool, &genesis.root, instance))」），不调 `judge`：这次调用的 O1-对拍一栏留在 `not_modelled`。
- O1-对拍的红只数 `Disagrees`（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:4728` 「red_calls += usize::from(matches!(call.model, ModelVerdictOfCall::Disagrees { .. }));」）。
- 写落在这次调用（含）之后的态，O1-崩溃判官一律记不适用（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:1885` 「!touched_call.is_some_and(|call| model_stopped_at_call.is_some_and(|stopped| call >= stopped))」）；不适用算不红。
- 「不适用只许来自 unmount 与对拍判出不同之后」那条停机 S5 只在基线判（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:5049` 「if run_kind == RunKind::Baseline」），坏实现那一趟（`row`）不判。

登记一侧：一行在 O1 上红要「X 在这一行两条流的批 1 里至少判红一处」（`research/prompts/e171-preregistration.md:39`）；读法 5 末句许诺「一次调用返回错误而模型说该成功，不算自崩，是 O1-对拍的红」（`research/prompts/e171-preregistration.md:41`）；读法 6 写「除这两种来源之外出现不适用，走停机 S5」（`research/prompts/e171-preregistration.md:42`）。

### 2. 造出来的一行

坏实现：取号那一刻的逐盘核把「少于两槽」写成「不多于两槽」（`crates/singlefs-core/src/transaction.rs` 第 661 行那个比较，冻结副本里原文是 `< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE`），不带故障的盘上每次取号都拒、一个字节不写。照上面几句逐步走：

1. 两条流的第 0 次调用 `acquire` 都是 `err:DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness`，流停，模型停在第 0 次调用。
2. O1-对拍：第 0 次调用 `not_modelled`，`o1_comparison_red_calls=0`。
3. 批 1 的态里没有一个写落在取号之后（取号在任何写之前拒）；不触达任何调用的态 O1-崩溃判官照判，读回 mkfs 的第 0 代根，模型提交过的版本里有它，判绿（推的）。
4. O2 判 mkfs 之后的镜像，绿；O3 在同一批态上判「在」（推的，执行器没跑）。
5. 这一行算触达（读法 4：调用结局与基线不同），进 Q1 的分母，三列都不红：Q1 里多出一行「三条全放行」。

同一条路还罩着暖机报错（第 1 次调用），以及任何让取号或暖机交回 `Err` 的坏实现。117 行里有没有落到这条路的，我没量（E171 装置是 checker 档 `src/bin/` 装置，我不跑）；主 agent 跑整表时一行的 E7RESULT 里有 `name=call stream=… call=0 label=acquire` 或 `call=1 label=warm_up` 且 `status=err:` 的，就是这一形。

### 3. 打中之后的四句

1. **分不分辨臂**：分辨。这一行该归「O1 抓到」（读法 5 末句）还是「三条全放行」，直接改 Q1 的答案与 Q3「只靠一条」的计数。
2. **系统当时看不看得到**：看得到。装置手里有这次调用的结局 `err`，模型在不带故障的流上对取号、暖机只有一种走法（往前走），「模型说该成功」不用另造答案。
3. **满足的是哪一句**：正文 K4 末句「E171 装置逐行打变异时基线、阳性对照、关一条开关会不会让某一列静默不判（判成「没红」）」里的「逐行打变异」那一格，O1 一列；登记读法 5 末句在这两次调用上没落地，读法 6 的停机 S5 在 `row` 一趟不判。
4. **改法**：

| 改法 | 修哪一格 | 造出来的那一行上 | 标 |
|---|---|---|---|
| 甲：取号、暖机交回 `Err` 时记 O1-对拍「对不上」（模型在不带故障的流上只许成功） | O1 一列的红 | O1 红，这一行归「O1 抓到」 | 推的 |
| 乙：读法 5 的不进分母那几类加一类「O1 没有答的调用出错」，照数报 | Q1 分母 | 这一行出分母，不再冒充三条全放行 | 推的 |
| 丙：S5 在 `row` 一趟也判，不适用来自别处就停机交主 agent | 静默变停机 | 这一行停机，不进表 | 推的 |

三个改法都只在读码上推过、被攻过零轮。

什么会推翻：E171 的整表汇总脚本另外把「调用 `status=err` 而 `o1_comparison=not_modelled`」判成 O1 红（我没找到这样的脚本：`ls research/scripts | grep -i e171` 无输出）；或 117 行里没有一行让取号、暖机出错（那样这一格是装置的缺口，不改这一轮 Q1 的数）。

## 四、K2、K3（不是重点，只读码加调用点样本）

**K2 二次崩溃 F 的界按内容认**：界取挂载写表里第一个「F 等于第一串抬 F 第一次发布的根里的 F」的系统配置槽写（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/mount_transparency.rs:296` 「.is_some_and(|written| written.quantities.rollback_floor == raised_floor)」），找不到就 panic（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/mount_transparency.rs:298`）。实现写抬 F 的系统配置时取抬到的 F 与盘上已生效的 F 的大者（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/transaction.rs:388` 「raised_floor.max(pool_effective_floor)」）。两种形状会让按内容认出错：① 盘上已生效的 F 高于这一串抬到的（第一次崩溃落在历史里一次抬 F 的系统配置写之后、根之前）——系统配置里写的是大者、不等于根里的 F，界找不到，装置 panic（不是误红，是判官自崩）；② 挂载里更早的系统配置写（取号、写行、暖机末尾的轮换）带的已生效 F 恰好等于这一串抬到的——界认早了，暖机那几态 F 不判（漏判）。两种都要「挂载推的抬 F 目标不高于盘上已生效的 F」，今天准入算目标的式子会不会给出这样的数我没查，也没造出来（推的）。调用点样本里 1 602 个二次崩溃态没有新发现、没有 panic（第一节 2）。

**K3 `walk.rs` 新 fold 与 `choose_root` 同值**：checker 的根按 `root_slot_positions` 排（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker/src/image.rs:852` 「for region in 0..geometry.regions {」，区域主序、区域内槽号升序），实现的读根环次序同为区域主序（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-core/src/recovery.rs:1202` 「.flat_map(|region| {」）；新 fold 只在键严格更大时换，键相同留前一个，与核同一择法。没打中；R 不等于 3 的池上两边的区域数来源不同（checker 取系统配置里的 R、实现取常量 3），那是池级字段越界那一族，前两轮已记欠账，不重攻。

## 五、攻击面之外的观测（不归这一轮的格，交主 agent 定去向）

调用点样本第 6 件（REUSE_AFTER_RAISING_THE_FLOOR、20 步、4 GiB、每段 6 个崩溃状态）在「可写挂载加一次发布」那一截有两条新发现，都不是 RefFS 的：种子 0「I-3.1」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-6.log:1` 「FINDING seed=0 stage=AfterTheWritableMountAndOnePublish」），种子 1「I-7.9」（`research/prompts/m3-formal-code-r3-opus-model/outputs/inject-6.log:2` 「FINDING seed=1 stage=AfterTheWritableMountAndOnePublish」）。这个比重今天没有崩溃注入的取样点用它（`grep -rn REUSE_AFTER_RAISING_THE_FLOOR crates --include=*.rs` 只命中 harness 的随机历史、故障注入与坏盘输入那几处）。是不是已知红、是不是副本才有，我没查；复跑命令是 `r3probe inject reuse 4g 20 0 10 6`（`tools/rerun.sh` 的 inject 一件）。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 段跨步（前一步的根 FUA 与后一步的写同段，RefFS 抬 lo 误红） | 五个取样点、120 段历史、8 113 个候选段 | 0 段跨步 |
| 回退一步写几条根、自己那条不是最后一条（m 表映错版本） | rbf 四段历史的段表 | 回退一步只有一个根槽写 |
| 回退目标根查不到 m（弃权） | 调用点样本 80 段历史、534 态 | 弃权 0 |
| 二次崩溃 RefFS 只许一份、比 O1 严（合法实现先红） | 1 602 个二次崩溃态 | 全「两者都绿」 |
| 抛弃最新根的那次挂载的重新起头 | 读码 | 今天恒被拒，那一支走不到（覆盖缺口） |
| 恢复失败时 RefFS 判红而不弃权 | 读码（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-harness/src/reference_filesystem_translation.rs:815`） | O1 同样判红，两票不分歧 |
| 层 0 枚举域逐态 | 3 步 10 段、8 步 1 段 | 只跑得动 1 段 45 态，全「两者都绿」 |
| E165 r4 写行 txg、计数器、取号、轮换、逐盘核 | 读码对实现 | 对得上（取号与逐盘核那两条是推的） |
| E171 关一条开关、阳性对照 | 读码 | 关 O1 记 off、关 O2 置 None，S5 在带开关的基线上不判（`/tmp/claude-1000/m3-formal-code-r3/tree/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:5048` 「S5 只判不带对照、不关 oracle 的基线」）；这几样都照登记读法 9 记成 off，不冒充没红 |

## 这条腿自己的限度

- K1 的「没打中」是一次抽样：调用点样本是被判对象自己的抽样（每段历史 6 个崩溃状态），不是层 0 枚举域；层 0 域上一段带内容发布的历史就超过 10⁶ 态，照定义第 3b 条没跑，交主 agent。
- 段跨步只扫了五个取样点（BROAD、ROLLBACK_AFTER_RAISING_THE_FLOOR、REUSE_AFTER_RAISING_THE_FLOOR；4 GiB 与单元区 256 槽），没扫单元区 384 槽、240 槽与带故障的历史。
- 由用户决定的动作：段跨步与调用点样本都是生成器按种子定的操作序列，我没有把「回退之后先做什么」单独放开扫；K1 两项都没打中，没有要判「同一段历史上不中」的臂。
- K4-a 只在一个入口（`mount_writable_with_test_only_switches_and_a_hook_before_the_one_reread`）、一种池（新池新建文件之后）上量了实现的读序；生成器一侧的数（28 466）是抄的钉值，没跑 E165 装置（checker 档装置，我不跑）。对 Q1 不动是推的。
- K4-c 全是读码，E171 装置没跑；117 行里有没有落到这一形的行没量。
- 原型跑的状态合计：层 0 域 45 态，调用点样本 534 + 1 602 态，挂载 11 次；远低于 10⁷。

## 要主 agent 跑的命令

1. 复跑这条腿的原型（不重型，约 5 分钟，本机负载 7 上下）：工作目录仓根，`bash research/prompts/m3-formal-code-r3-opus-model/tools/rerun.sh /tmp/claude-1000/m3-formal-code-r3/tree <新草稿目录>`；脚本内部每条经 `run-with-memory-cap.sh`（编 2G、跑 2G 与 1G）与 `capped.sh 6`（inject 六件各 1 线程）。看末尾 `same <文件名>` 有 18 行、没有 `DIFF`。
2. K4-c 在真装置上坐实（重型，E171 是 checker 档 `src/bin/` 装置）：在冻结副本的一份拷贝上把 `crates/singlefs-core/src/transaction.rs` 第 661 行的 `< SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE` 改成 `<= SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE`，跑 `e171_three_oracle_collusion row --output-directory <新目录>`（带 `SINGLEFS_HEAVY_TESTS=…` 前缀、经内存包装）。照我的读法应看到：两条流 `name=call … call=0 label=acquire … status=err:DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness o1_comparison=not_modelled`，`name=verdict` 行 `o1_comparison_red_calls=0 o1_crash_judge_red_states=0`，退出码 0。看到 O1 红就推翻这一格。
3. E171 整表（14-2）跑完之后：`grep -E 'name=call .*label=(acquire|warm_up) .*status=err' <每一行的 E7RESULT 输出>`，有命中的行按第三节第 4 条的改法之一处置，Q1 的分母与「三条全放行」计数重算。

## 没做什么

- 没跑任何测试二进制（harness 档也没跑），没跑 checker 档装置（E165、E170、E171）、层 0 整段枚举、崩溃放量、变异表。
- 没判 K2 准入算的抬 F 目标会不会不高于已生效的 F；K3 只读了 `walk.rs` 那一处与根的次序，13 种手放槽照派发提示不重攻；E170 新模式与旧模式输出没比。
- 没读禁读清单里的 `research/prompts/m3-formal-code-r3-sonnet-output.md`；没读背景材料的 kb 附录全文，只按需 grep 了 E165 页、两份登记。
- 第五节两条新发现没查是不是已知红。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-r3-attack/` 里的仓副本与编译目录交回之前删掉，只留日志与 `SHA256SUMS`（模型目录里有同一份）。
