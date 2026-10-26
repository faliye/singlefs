# E184 跑前登记：常规层与大负载层的代价计数

写于 2026-10-18，装置写之前。

<!-- doc-lint:not-numbers J1 J2 J3 J4 G1 G2 G3 G4 K3 K4 Q1 Q4 T1 T2 T3 T4 T5 T6 T7 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 B1 B2 B3 B4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 V1 V2 V3 V4 V5 V6 V7 V8 V9 V10 V11 V12 E161 E164 E180 C409 C577 C635 -->

判据、门槛、作废与停机条款在这里写死；跑出数之后要改，照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」那三步办。

**这一份回答问题单 `research/prompts/m3-close-c409-cost-questions.md` 的 J1–J4 四行**（12 行，sha256 `e09c8dd62bcad6e225f72daec4906bd920a8ff630e604a2469dbb39d24dca059`），它们对应岔路单 `research/prompts/m3-close-c409-r1-forks.md`（12 行，sha256 `518b75d48fc8d1046ef9831297e157efef885739edc4b30dc92bd3e3e08f86c9`）的 G1–G3；G4 这一份不罩（问题单没有对它的一行）。

**装置写在哪（写死）：入库装置。** 执行员新建两份 bin：`crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs`（全部子命令与内联单测）与 `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts_write_dump.rs`（只录写流、写转储，只引 `singlefs-harness`、`singlefs-core`、`singlefs-format` 与经 `#[path]` 引的测试公共模块，不引 checker 档包自己的模块，好让它在第二段的旧提交副本里也编得过）。依据：J1、J3 问的是 `crates/` 这份代码的性质——今天的写路径录出什么写流、崩溃放量与崩溃枚举用例的块键算法（`crates/singlefs-checker-tier/src/crash_identity.rs`、`crash_amplification.rs` 的 `plan_crash_points` 与 `PlannedCrashPoint::block_key`）给这些写流算出哪些块键；另写一份独立模型答不了「crates 的键对不上几块」。块键一律调 crates 的函数算，不另写一份。

**这条实验要读 `crates/` 的这些常量——执行员逐个写成装置里的本地常量（值抄自下列登记位），再加一条与 `crates/` 那一份回比的断言（停机 T1），不许直接引：**

| 本地常量 | 值 | 登记位 | 回比 `crates/` 的哪一个 |
|---|---|---|---|
| 每块状态数 | 65536 | `crates/singlefs-checker-tier/src/crash_amplification.rs:175`（D13 已定项 16 没写块长，块长只在实现里） | `crash_amplification::STATES_PER_BLOCK` |
| 随机历史种子基 | 7463871032432355113 | `crates/singlefs-harness/src/history.rs:123`（「种子基永久固定」） | `history::SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE` |
| 闭式能算的最长段 | 63 | D13 已定项 16「整域装不进 u64」；`crates/singlefs-harness/src/memory_pool.rs:663` 注释 | `memory_pool::closed_form_state_count` 在 64 写的段上 panic（单测核） |

步数档、种子数、盘宽档、比重组、覆盖写链的单元数、补丁位、提交清单一律写死在第五节，不从 `crates/` 算。

**起跑前提（写死，第十一节停机 T0）**：执行员开工第一步核两样，任一不成立就不开工、交回：① 第五节 5.6 分段表第一行（装置交主 agent 后台编）已照做，主工作区里 `e184_regular_layer_cost_counts*.rs` 两份是 `compile-then-swap.py` 换进来的那一版；② 第十二节「修订」里没有未经主 agent 认的改动。

## 一、问题

英文名：regular_layer_cost_counts

主 agent 给的问题（派发提示逐字）：「C409 常规层与大负载层的代价问题单 J1–J4 跑前登记」；「J1 的计数探针放入库装置（`crates/singlefs-checker-tier/src/bin/` 下新文件，读主工作区现状的 `crates/`），录制候选负载、切段用与崩溃点重放相同的切段规则；「与冒烟层登记流共用的块键数」用 `crates/singlefs-checker-tier/src/crash_identity.rs` 的块键算法算，不另写一份」；「J2 冷吞吐是重型（起崩溃放量一个小项与一个中项、新建空库目录），登记写成要主 agent 跑的命令段，单列一段，标明重型、要先弹窗问用户；J3 的「改一处写路径字节」只在仓副本里做，补丁不回主工作区」；「登记的分段第一步写「装置写完先交主 agent 经 `scripts/hosts/compile-then-swap.py` 后台编」，单测与产物在编过之后由续派的执行员跑」。

问题单原文（`cat research/prompts/m3-close-c409-cost-questions.md`，逐字）：

# C409 常规层、大负载层：要先量代价才能交用户定的几条（问题单，2026-10-18）

<!-- doc-lint:not-numbers G1 G2 G3 G4 J1 J2 J3 J4 -->

形态照 `.claude/rules/three-way-inference.md`「交岔路时写岔路单」：只写问题与候选定义，不写倾向、不写已有的数。岔路单 `research/prompts/m3-close-c409-r1-forks.md`，判决 `research/prompts/m3-close-c409-r1-main-verification.md`；调度在 `records/2026-10-18-里程碑三收尾与近期欠账.md`。方案按 `crates/` 今天的实现来谈（主工作区现状，含里程碑四会话没提交的改动）。

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| J1 G1 各候选负载有多大、和冒烟层重叠多少 | G1 甲（按全量态数与最长段登记）、乙（按负载来源分：固定种子随机历史整段全量展开）、丙（不分，并进冒烟层）各自划进常规层的负载 | 候选负载的态数区间与冒烟层登记流的态数区间重不重叠；候选负载与冒烟层登记流共用的块键占比 | 入库装置在每个候选的负载上录一遍：写数、段数、最长段、两态闭式、甲二下界、与冒烟层登记流共用的块键数；随机历史至少三档步数、每档至少 20 个种子 | 开着 |
| J2 G2 冷吞吐与库增量 | G2 甲、乙、丙、丁 | 今天的树上每秒判多少态、库每态多少字节，乘 J1 的态数后，提交那一趟的挂钟落在哪一档 | 今天的树上在新建的空库目录上冷跑一个已登记的小项与一个中项，读驱动的态数、秒数、录入与判段各自用时与库字节（重型，主 agent 起） | 开着 |
| J3 G2 块键作废面 | G2 甲、乙、丙 | 一次动写路径的提交让一条负载的块作废多少；最近若干次提交里块键对不上的比例 | 副本上改一处写路径的盘上字节（如一个记录头的保留字节），数同一负载前后两份写流第一处不同的写落在第几段、作废块数；对最近至少 10 次碰 `crates/` 的提交各算一遍键对不上的块数 | 开着 |
| J4 G3 两种落法的固定开销 | G3 甲（门禁加一格）、乙（另立一道阶段） | 没有一块要现判时，加一格与多一道阶段各自让提交那一趟多花多少挂钟 | 两种落法各在样本仓上起一次、负载全复用，量挂钟 | 开着 |

岔路单原文（`cat research/prompts/m3-close-c409-r1-forks.md`，逐字）：

# 崩溃点重放的常规层、大负载层怎么立：开着的岔路（2026-10-18）

<!-- doc-lint:not-numbers G1 G2 G3 G4 K3 K4 Q1 Q4 -->

判决 `research/prompts/m3-close-c409-r1-main-verification.md`；调度在 `records/2026-10-18-里程碑三收尾与近期欠账.md`。只写问题与候选定义，不写倾向与已有的数。

| 岔路 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| G1 常规层的负载与冒烟层怎么分开（D13 已定项 9 层表：冒烟层「固定种子、固定脚本……不按写请求数限规模」，常规层「写请求数上千」） | 甲：每条负载登记它的全量态数与最长段写数，层表的规模分句写成这两个数的区间，「写请求数上千」只当描述；乙：按负载来源分，冒烟层是手写固定脚本与登记的崩溃枚举用例，常规层是固定种子的随机历史整段全量展开；丙：不分，常规层并进冒烟层，三层改两层（冒烟层全量 / 大负载层） | 候选划进常规层的负载，与冒烟层已登记的流在块键上的交集占比；两层的态数区间重叠与否 | 计数探针在入库装置上对每个候选的负载录一遍（写数、段数、最长段、两态闭式、甲二下界），并数出与冒烟层登记流共用的块键数 | 开着 |
| G2 常规层提交时那一档（层表「每次提交」×「精确全量」） | 甲：照字面，每次提交按块键增量判全量；乙：提交时只展开甲二（块级复用），全量归用户要求时；丙：提交时只查库，判器单元或写路径变了的提交由主 agent 起常规层全量；丁：节点登记表静态标 ignore（开着 / ignore / 等实现），用一个先声明的挂钟预算约束一次提交能开几个节点，不按挂钟截断枚举 | 一次动写路径的提交与一次只动判器的提交上，各候选在提交那一趟要现判的块数与挂钟；最近若干次提交里块键作废的比例 | 今天的树上量出冷吞吐（态每秒）与库每态字节；在副本上改一处写路径字节，数作废块数；对最近若干次提交各算一遍键对不上的块数 | 开着 |
| G3 落在哪 | 甲：并进崩溃点重放入口与门禁 `checker-tier-crash-point-replay`，加一格，常规层登记行在提交时照 G2 选定的档真判（不是只查库）；乙：另立一道门禁阶段，阶段里只转调同一个入口，文件头写 `gate-similar` 说明；丙：只并进入口，门禁不加格 | 用户口径里的「立阶段」是新文件还是加格；加一格与多一道阶段各自的固定开销 | G2 定了之后；两种落法的固定开销各量一次 | 开着 |
| G4 大负载层的「多设备档位交叉」与「N≥5 轮」 | 设备档：甲：照实登记「多设备：本次未跑（只有两块盘）」，挂到三块盘以上那条线，盘宽与 io_min 另起名字当几何档；乙：盘宽 × io_min 当多设备档位。轮：子：轮 = 不同种子或不同内容的负载份数；丑：轮 = 提交协议结构等价类数（D17 已定项 2）；寅：轮 = 不经判定库复用的独立重放（不同二进制、真设备） | 选定读法下同一负载跑 5 轮，第 2–5 轮现判的块数是否为 0 | 「轮」的读法定下；读法子要 G1 的计数探针给出每份负载的态数 | 开着 |

### 1.1 读法写死

- **流**：一条负载录出来的写流，从 mkfs 做完的那一刻起：基镜像 = 空盘叠上 mkfs 的全部写（连同清零写），流里的写表与段只取 mkfs 之后的操作。随机历史照 `crates/singlefs-harness/src/history.rs:3533` 的 `operations_written_by_make_filesystem` 切开（同 `HistoryRun` 那一行注释写的「崩溃点重放那一路同样从 mkfs_operation_count 之后起枚举」）；冒烟层与覆盖写链的流照它们各自的建池函数（建池之后的池就是基镜像）。
- **写数**：流的写表条数（`writes.len()`，D13 已定项 4「枚举的单位是一次整写」）。「上千」读作写数 ≥ 1000。
- **段**：`crates/singlefs-harness/src/memory_pool.rs:579` 的 `writes_and_segments_with_stream_indexes_and_entries` 切出来的段（与崩溃点重放同一条规则，D13 已定项 4）；随机历史只切 mkfs 之后那一截操作。**最长段**：段写数的最大值。
- **两态闭式**：`crates/singlefs-harness/src/memory_pool.rs:659` 的 `closed_form_state_count`（1 + Σ(2^|段| − 1)）；有一段 ≥ 64 写时记「装不进 u64」，不报数。
- **全量态数**：`crates/singlefs-checker-tier/src/crash.rs:2418` 的 `crash_state_count_with_torn_in_place_overwrites` 配 `full_expansion`（`crash.rs:2459`），原地覆写取第三态；装不进 u64（函数 panic）记「装不进 u64」。
- **甲二态数**：同一个函数配 `quick_tier_expansion`（`crash.rs:2450`）。问题单写的「甲二下界」读作这个精确数：它不小于「原地写只按两态算」的下界，这一份不另算下界。
- **节点与块**：节点按崩溃枚举用例那一路切（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:765` 的 `crash_points_of_the_steps`：写下标区间交叠的几段并成一步，一步一个崩溃点）；计划由 `crash_amplification.rs:505` 的 `plan_crash_points` 排，展开取 `full_expansion`；块 = 每个点 `PlannedCrashPoint::blocks`（`crash_amplification.rs:319`），块键 = `PlannedCrashPoint::block_key`（`crash_amplification.rs:359`）。`crash_points_of_the_steps` 是私有函数，装置照抄（逐字，停机 T2 钉住），不改它的可见性。
- **冒烟层比较集 R4**：四条已登记的崩溃枚举用例流在装置里重录：`crash-case:new-pool-file-creation-stream`、`crash-case:overwrite-fixed-script-stream`（到 E 再正常卸载那一支）、`crash-case:multi-record-publish-stream`、`crash-case:snapshot-creation-stream`。建法：能经 `#[path]` 引测试公共模块的直接调（`crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs:475` 的 `prepare_the_snapshot_crash_flow`、`crates/singlefs-harness/tests/common/mod.rs:747` 的 `build_pool`），测试文件里私有的 `prepare` 逐字照抄（E161 装置照抄固定脚本那一例），抄的每一段由停机 T2 钉住；版本表照各测试文件交给 `enumerate_crash_states_*` 的那一份。选这四条的理由：块键含起步镜像，两条流只在同一基镜像上逐字节相同的前缀里共用块；这四条罩住标准前缀（取号、暖机、第一个文件）与它之后的三种走法（覆盖写、多记录顺序写、建快照）。其余 25 条登记行不重录，它们与候选共用的块记「没量」，由附带量 J1-库（第六节）当旁证。
- **冒烟层态数区间 S**：`.claude/gate.d/stage-inputs.tsv` 里 `crash-case:` 开头、测试文件里钉了段形或两态闭式常量的那几条流的两态闭式（执行员 grep 测试文件取常量、照 1 + Σ(2^|段| − 1) 现算核一遍，取不到的那一条记「没钉」），加上 R4 四条的现录值；区间取最小值与最大值。
- **共用块键数**：一条候选流的块键集合与 R4 四条块键并集的交集大小；**共用占比** = 共用块数 ÷ 候选流的块数；另报按态数加权的占比（共用块里的态数 ÷ 候选流的态数）与复用键（`ReuseKey`，不含计划哈希）的共用占比（块键共用的上界）。
- **冷吞吐**：两种都报、都用：`R_total` = 现判态数 ÷ 那一项的挂钟（录入加判）；`R_judge` = 现判态数 ÷ 判段用时（驱动在进程里打的 `judge_seconds`）。库增量：新建空库目录跑完后的字节数（`du -sb`）÷ 现判态数，与 ÷ 录入块数。
- **提交类别**：写路径提交（流的写字节变了）、只动判器的提交（判法单元键变了、写流不变）、两样都不动的提交。
- **挂钟档**：≤ 2 小时、2–24 小时、> 24 小时。2 小时取自 `scripts/hosts/detached-run.sh` 的 `--wall-clock within-2h / over-2h` 分界，24 小时是一天；两个界都不是从哪条臂的定义推出来的。

### 1.2 问题单各行在这一份里由谁罩

| 问题单行 | 岔路行 | 这一份的量（第六节） | 段（5.6） |
|---|---|---|---|
| J1 | G1 | J1-甲（每条流的写数、段数、最长段、三种态数、块数）、J1-乙（区间重叠）、J1-丙（共用块键）、J1-库（附带） | 第一段 |
| J2 | G2 | J2-甲（冷吞吐与库增量）、J2-乙（各候选提交那一趟的挂钟与档） | 第三段（重型，主 agent 跑）；J2-乙 在第三段末尾合算 |
| J3 | G2 | J3-甲（合成补丁的作废面）、J3-乙（最近提交的键对不上比例） | 第二段 |
| J4 | G3 | J4-甲（两种落法的固定开销） | 第四段（G2 由用户定了之后才跑） |

## 二、被测条款与它引的定义

`scripts/kb/quote-kb.py` 整段抄（出口 `~/.local/share/singlefs/work/m3-close-w4-c409-cost-design/quoted-clauses.md`，脚本回读逐字节一致），再原样追加在下面。抄哪几节：D13（验证路线） 已定项 9（层表三行，被测）、已定项 11（ignore 节点什么时候开、T1 不写进 kb，被测）、已定项 17（判定按函数单元键复用，被测：块键与判法单元键的定义）；它们引的定义：已定项 4（崩溃状态与切段，「写请求」「整写」的口径）、已定项 15（快档与全量、提交时跑什么）、已定项 16（按段登记、长段的态数）、已定项 10（已定项 9 射程点名的精确剪枝归属）；欠账表 C409 那一行。D17（实现分层与第三方管道） 已定项 2 只与 G4 有关，这一份不罩 G4，不抄。T1 是已定项 11 的符号（常规层 挂钟预算）：这一份只报挂钟档，不给 T1 定值，也不往 kb 写挂钟数。

**出处 `.claude/kb/decisions/13-验证路线.md:240-273`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 冒烟层 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档 |
| 常规层 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 大负载层 | 常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。**
**里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑。**
「全量」指两样：checker-tier-crash-point-replay `--full`（门禁 checker-tier-crash-point-replay，已定项 15 的全量）与崩溃放量全量（`scripts/crash/crash-amplification.sh` 不带参数）。
「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。
「上一次全量」由主 agent 写进当前里程碑记录一行：日期、两样各自的结论。
**全量档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个全量档 bug 的挂钟 ≈ `log2(N) × 一趟全量的挂钟`，一个不可复现的全量档失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：只有冒烟层（门禁 checker-tier-crash-point-replay）；冒烟层分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-crash-point-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求时跑；常规层、大负载层在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束，常规层、大负载层 的负载与挂钟都还没有，没有可量的量。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；崩溃点重放规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。
- 用户定案（2026-10-01）：全量档全绿的时点是里程碑收尾与动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑，原话在变更史；交用户那一格是三方判决 `research/prompts/test-script-align-r1-main-verification.md` 第二节 t1-3。
- 三方判决 `research/prompts/test-script-align-r2-main-verification.md` 第二节 U3（「U3 t1-3 写回照字面做不了」那一行）：「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句。

**欠**：C409（常规层、大负载层在门禁里没有阶段）；C635（上一次全量没有机器可读的标记）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:304-338`（整段抄，未转述）**

```markdown
#### 已定项 11：ignore 的节点什么时候开：否决按欠账轮转的账本，留下三条

**定案**：

**「按欠账轮转的账本」调度：否决。** 格子 = `(布局, 子阶段桶)`、按欠账降序取队首、声称任一格子最多 `K_max = ⌈N×M/C⌉` 次提交必被覆盖的那个调度被否决：它保证不了的恰恰是它要防的那一类——慢的、罕见的分支（实测验算与另外四条否决理由在变更史）。

**留下的三条**：

1. **方向对**：要一个**确定性的、能被门禁判红的覆盖记录**，而不是概率论证——
   概率保证在长期维护里等于没有保证。

<!-- doc-lint:registry name-col=2 -->

| 符号 | 简称 | 含义 |
|---|---|---|
| T1 | 常规层 挂钟预算 | 常规层 本地挂钟预算，是输入不是输出 |

2. **`T1`（常规层 挂钟预算） 是输入，不是输出**（`.claude/rules/fs-design.md`：门禁运行时长本身是设计约束）。
   正确顺序是「先声明预算 → 用它约束一次提交能开几个 ignore 节点」，不是「量出耗时 → 推出能开多少」。
   **这一条不需要任何测量，现在就该定。**
3. **判别力必须与覆盖分开检查**：每个 ignore 节点被打开的那次提交里，额外对一份「已知坏的镜像」跑它自己的 checker，
   不报错即判红——这是 D13（验证路线）已有的阳性对照纪律的直接复用。见 C13（checker 判定失效）。

门禁里没有「常规层 崩溃点重放」这个阶段，量不出 `T1`（常规层 挂钟预算） / `C` / 每格耗时。
⚠️ **因此 `C`、`K_max`、`T1`（常规层 挂钟预算） 这三个数一个都不许现在写进 kb。**

**射程**：只管 ignore 的节点什么时候开；冒烟层全量枚举不需要 `T1`（常规层 挂钟预算）。第 2 条的「现在就该定」与末句「一个都不许现在写进 kb」说反话，记在 [verification-build.md](../verification-build.md)「待定案」第 6 问，没判。节点登记表带状态字段（开着 / ignore / 等实现），另加三条——每个 ignore 行点名一个会变的对象、那个对象变绿而节点仍 ignore 时判红、覆盖报告列 ignore 的状态数与占比：这三条被攻过零轮（`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节），不写成定案，等第二、三轮再攻。

**依据**：
- 无实验：否决账本靠的是一次调度规则的模拟验算（200 轮、取后半程稳态最大值、严格前缀贪心，三行数在变更史），没有登记成实验；留下的三条是门禁设计原则，没有可量的量。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。
- 用户定案：「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 11 推翻，「逐格判决」一节给节点登记表带状态字段的形态与三条待攻的补充条件。

**欠**：C8（门禁范围判不出来）；C13（checker 判定失效）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:424-444`（整段抄，未转述）**

```markdown
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 重型门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、快档与模型对拍的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

**射程**：管重型门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与什么时候请用户跑全量（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）的键，不论这一块在 CPU 还是 GPU 上判的：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（快档的 GPU 与 CPU 对拍用例），它的单元只进块属性当出处，不进键、不分种类；显卡驱动、卡型、几张卡怎么分活同样留作块的属性或跑的参数，不进键。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 用户定：提交与重跑无关，重跑只跟代码变化走」一节、「2026-10-15 批3」那几段。
- 用户定案（2026-10-17）：判定不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备；原话在变更史与 `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 叫停全量那几段。
- 主 agent 定案（2026-10-15）：仓根不进键与库、库目录只收绝对路径、cfg 成分只取闭包真引到的原子，出处同一份记录「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案第 1、2 条。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 盘上的数：`research/prompts/pruning-survey-2026-10-15/ledger.md`「一之二、盘上量到的」一节——只差名字的块各存各判 4100 块与 9679 块；整库一把的判法摘要一换累计多判 3 935 242 934 态，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同：键带名字、按文件或整版取指纹，同一份判定会重判。
- 没有腿况记录：未经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。

**欠**：C8（门禁范围判不出来）、C489（singlefs-core 的广谱变异对今天的代码没有结论）、C528（两条门禁路判的改动窗口不同）、C564（另外 6 条崩溃点重放全量没人跑）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C630（提交时形式化那一道第 1 步停下的频率没量）、C635（上一次全量没有机器可读的标记）、C645（崩溃放量全量没有人能派）、C668（随机历史脚手架的引入者白名单能被别名写法绕过）、C777（五个实验键在 stage-inputs.tsv 没有登记行）、C836（staged-green 挂的树不一定是被判的树）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:134-149`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。崩溃点重放的扇区取 512 字节；两条崩溃点重放流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条崩溃点重放流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；崩溃点重放的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，崩溃点重放小流里记录核对器报出来）在 `m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `e142-r19-runner3-report.md` 结论段，登记 `e142-r19-prereg.md` 的 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:395-407`（整段抄，未转述）**

```markdown
#### 已定项 15：验证代码分两档——harness 档随时跑，checker 档默认只在提交时跑

**定案**：`crates/` 下的验证代码按「什么时候跑」分两档，各住一个包，与池级 checker 一共三个名字：**harness 档**是 `crates/singlefs-harness`，单元测试与集成测试加上它们用的脚手架库（录制器、内存池、理想模型、随机历史、故障注入设备、场景），改了代码随时跑；包内再分轻重：debug 下单条跑到 60 秒及以上的是重，标 `#[ignore]`，单条用时看平常跑的时候量到的、不另外单独量，要跑随时跑，一律经内存包装；一条用例一个场景，测试文件按测什么起名、不按里程碑起名。**checker 档**是 `crates/singlefs-checker-tier`，崩溃态枚举引擎与记录核对器、崩溃枚举用例经判定库的两段与它的计数、发现日志与工具链身份、一份长期判定库（块键只由内容定，几类测试各自要哪些块只当索引）与两段协议、崩溃注入与坏盘输入两场战役、真设备一侧的设备日志比对、实验与真设备装置二进制，以及它们的全部用例，连同 QEMU（门禁 checker-tier-qemu-device-streams）、herd7（checker-tier-lkmm）、crates 变异整表（checker-tier-crates-mutation-replay）、全部实验复跑（checker-tier-research-build-and-replay 的 experiment-replay 格）：默认只在提交时跑，能单独跑（命令带 `SINGLEFS_HEAVY_TESTS=user-request`）。checker 档里的崩溃点重放（门禁 checker-tier-crash-point-replay）分两档，checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-research-build-and-replay 不分：**快档**是两趟：`cargo test --release -p singlefs-checker-tier --lib --tests` 不带 `--ignored`（`--tests` 带上全部 `test = true` 的目标，装置二进制的内联单测也在里面），再一趟带 `tests/` 里 `cfg` 点到的特性（`--features <特性> --lib --test <挂特性的测试文件>…`，特性里有 `gpu` 的这一趟经 `scripts/crash/gpu-quiesce-run.sh` 起）（门禁 checker-tier-crash-point-replay 在整轮门禁与提交时都跑这两趟，第二趟是它的 feature-gated-tests 格），**全量**是登记在 `.claude/gate.d/stage-inputs.tsv` 键为 `crash-case:` 的用例逐条 `--include-ignored --exact`（checker-tier-crash-point-replay `--full`），提交时不默认跑，由用户要求时跑，跑法按已定项 9 分层；崩溃放量判器整套上了 GPU，罩全部池级 checker 不变量（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`）。**池级 checker** 是 `crates/singlefs-checker`，判决器库，被两档调用。依赖只许一个方向：checker 档依赖 harness 档与池级 checker，harness 档不依赖 checker 档（门禁 checker-independence-and-sync 的 checker-implementation-disjoint 格第 ④ 条判，dev-dependencies 也算）；两档都要的一小段代码宁可各留一份。崩溃枚举用例一律写在 `crates/singlefs-checker-tier/tests/` 里（`scripts/crash/crash-case-check.py` 判），重型测试闸按包判：跑到 `singlefs-checker-tier` 的测试就是 checker 档。

**射程**：管 `crates/` 下代码与测试住哪个包、什么时候跑、闸按什么认、三个包怎么称呼（`.claude/rules/verification.md`「定义与名字」）；不管每条不变量判什么（[invariants/README.md](../../../crates/singlefs-checker/invariants/README.md)）、不管枚举域怎么定（已定项 4）、不管池级 checker 与实现共享什么（已定项 5：池级 checker 仍只依赖 `singlefs-format`）。上游 SOP 不再写这个项目的验证手段：崩溃点重放、模型对拍、checker 即规范、文件系统特有的反推缺口四节与 `gate.sh` 未实现清单里的项目键都收进本仓（`.claude/rules/verification.md`、`.claude/gate-not-implemented.tsv`），上游那一批已同步进副本、未发版。拆分那一轮的记录在 `records/2026-09-27-验证两档拆分.md`。

**依据**：
- 无实验：什么时候跑、住哪个包是流程政策，没有可量的量；两档各自的判别力由各自的用例与变异表证。
- 用户定案（2026-09-27），原话在变更史；三方 `verification-split-r1`（判决 `research/prompts/verification-split-r1-main-verification.md`）攻的是拆出 `singlefs-checker-tier` 之前的那一版，拆包之后的改动被攻过零轮。
- 用户定案（2026-10-01）：全量不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:408-423`（整段抄，未转述）**

```markdown
#### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数

**定案**：
- 分片：崩溃放量里按段登记的段（`.claude/gate.d/crash-point-replay-segments.tsv`，一行一段五列：项名、流名、段号、段写数、段写表摘要），整域态数装不进 u64 时按固定大小的片依次判；片大小由环境变量 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 定，没设取 2^32 态；片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。整域装得进 u64 的登记段不分片，整域一次判。代码落点：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- 位宽：登记段的总态数、片起点、片尾与进度一律用 1280 位（10 个 u128）定宽无符号整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`，肢数是同一文件的 `STATE_COUNT_LIMBS`），算术全部 checked，不饱和、不截断；驱动行 `registered_segment_progress=` 的已判态数与总态数都写精确十进制，不写幂的形态。
- 登记表怎么交给引擎：引擎只经环境变量 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 读按段登记表（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`，在 `fn registered_segments_of_the_flow_from_the_environment` 里），没设就当一段都没登，没有别的退路。144 个单元的文件顺序写成 145 个、extent 下段长成两层的那条流（引擎流名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）的单元写段（322 次单元写）与记录段（290 次 journal 记录原地写）两段都登。

**射程**：管崩溃放量引擎对按段登记的段怎么分片、态数用什么整数记、登记表从哪里读；不管哪几段该登（由用户点头，写在登记表文件头）、不管登记段的域本身怎么展开（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10）、不管每个崩溃状态怎么定义（已定项 4）。没登记的段的态数、与整条流合起来的态数仍要装得进 u64，装不下在写库之前拒跑；登记段整域越过 1280 位同样在写库之前拒跑。片大小只许 1 到 2^32：两个 GPU 内核把片内序号当 u32 加到片起点上（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；轻型（`scripts/crash/crash-point-replay.sh --quick`）调用方没设片大小时设成 `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`。展开上限盖不住又没登记的段，放量入口 `scripts/crash/crash-amplification.sh` 判红；崩溃点重放入口、放量入口与分项驱动开跑前核这份表再送给引擎。

**依据**：
- E161（崩溃放量的去重与分段耗时）「第 2 次重跑：取样域按 11 段的流重定（2026-10-05）」一节：段的全量态数与闭式、装置逐个数、crates 计划逐段相等，态数随段写数按幂长，撑的只是「长段整域装不进 u64」这个前提；片大小、位宽取值与登记表的读法是工程参数，用户定，它不撑。
- 用户定案，原话在变更史：分片与每片 2^32 态（`records/2026-10-11-崩溃放量两段与脚本审计.md` 第 96 行）、位宽 1280 位（同一份记录第 98、99 行）、两段都登（同一份记录第 92 行）。
- 分片与位宽是对判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10（按段登记）的补充，没经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:274-303`（整段抄，未转述）**

```markdown
#### 已定项 10：真正必须精确覆盖的 N 不是「布局套数」，是「提交协议结构的等价类数」

**定案**：常规层 精确覆盖先缩问题，再谈剪枝。崩溃点的定义域是**事务层状态机**，而 `.claude/rules/fs-design.md` 有两句顶层规则：
「一个事务层，所有结构共用」与「唯一不许多样化的是事务层」。所以布局能改的只有两类：

| 差异 | 改变的是 | 判定 |
|---|---|---|
| 节点大小、条带宽度 | 一步写多少字节、某步重复几次 | **参数**，不新增步骤种类 |
| 旋转 vs SSD | seek 代价、IO 调度 | 落在设备层，**不落在事务层协议里** |
| **zoned（ZNS）** | **能不能原地覆盖根指针** | **结构性**：只能追加、根位置漂移，且多出 zone finish/reset |

**落地手段是一个封闭枚举 + 穷尽 match**：`crates/singlefs-core/src/transaction.rs` 的 `CommitStep`，步骤种类的登记位是 [layout/01-first-txn.md](../layout/01-first-txn.md) 八的段序列登记表。
任何布局要引入新的结构性步骤**必须改这个共享定义 ⇒ 编译器强制所有引用处补分支 ⇒
这个 diff 门禁 grep 得到**，可以自动打标「本次引入了新的结构等价类，需为它新开一轮独立大负载层 重放」，
**不靠人记得申报**（按 diff 算门禁范围欠在 C8（门禁范围判不出来））。再加 newtype 隔离（`RotationalPtr` / `ZonedPtr`），
把 D12（目标介质） 已定项 2 里「不许共用」那一桶（记账事实的判定路径）变成编译期约束。
⚠️ **它有第二个职责（D17（实现分层与第三方管道） 加的）：它是第三方管道的接入点。**
协议归本工程所有，一条管道要引入新的结构性提交步骤必须往本仓发这个 diff。

**射程**：旋转与 SSD 在提交协议层面同构，由 D12（目标介质） 那一轮三方论证承担；zoned 那一行只说明它至少多出一个等价类。等价关系按录制流的段序列同构、有几个，归 D17（实现分层与第三方管道） 已定项 2。按种类串认节点够不够，第一轮判决打中（挂起，待第二、三轮攻）：`research/prompts/m3-prune-gpu-r1-main-verification.md`「逐格判决」一节。
⚠️ **它钉住的只是「有哪些步骤」，钉不住「每一步的失败原子性宽度」**——
持久内存与对象存储改变的是每一步的失败原子性宽度，枚举接不住（缺口与补法见 D17（实现分层与第三方管道））。
`RotationalPtr` / `ZonedPtr` 这层 newtype 隔离 `crates/` 里没有（`grep -rn "RotationalPtr\|ZonedPtr" crates` 零命中）。

**依据**：
- 无实验：N 是什么由 `.claude/rules/fs-design.md` 那两句顶层规则推出，没有可量的量；等价关系怎么判、有几个的依据在 D17（实现分层与第三方管道） 已定项 2。
- 三方论证：常规层 抽样那一轮（「三方论证 + 实测验算」），材料没有留存，经过在变更史。

**欠**：C8（门禁范围判不出来）。

```

**出处 `.claude/kb/checks-owed.md:294-294`（整段抄，未转述）**

```markdown
| C409 | 常规层、大负载层在门禁里没有阶段 | **D13（验证路线） 已定项 9 把崩溃点重放分三层，并定「提交前 = 快速档全绿（必要条件）」与「里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑」。** 现查 `.claude/gate.d/` 只有 门禁 checker-tier-crash-point-replay 冒烟层（固定种子的最小复现负载、不抽样全量），常规层（上千写请求、按崩溃点所在子阶段分桶抽样、每次提交）与大负载层（常规层负载加大规模负载加多设备档位交叉、N≥5 轮、用户要求时或发布前）一个阶段都没有；立账之前 D13（验证路线） 已定项 9 的欠块逐字写着「层 1、层 2 在门禁里没有阶段，没有 C 编号」，立账后欠块已列 C409（常规层、大负载层在门禁里没有阶段）⇒ 「里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑」这条异步准入判据没有执行路径，而门禁汇总里看不出它缺席 | 常规层、大负载层各接一个门禁阶段；某次运行无对象时按 `.claude/singlefs-ai-sop/rules/show-me-test.md` 写退出码 77 明说未跑、不记通过。常规层那个阶段的成功行要报「这次抽了多少个崩溃点、分在哪几个桶」，并把没抽到的桶逐个列名。判别力自证：把常规层的抽样数设成 0，阶段必须判红而不是报绿 | 常规层、大负载层的负载（上千写请求、多设备档位交叉）还没有；冒烟层已有（门禁 checker-tier-crash-point-replay）；用户 2026-10-18 弹窗定「立」（常规层、大负载层在门禁里立阶段），负载形态先开三方定 | 2026-09-19 第二批 D13（验证路线） 瘦身时子 agent 指出，主 agent 现查 D13（验证路线） 已定项 9 的三层表、射程与欠块，并逐个看 `.claude/gate.d/` 下的阶段名坐实；引号里的句子照 D13（验证路线） 已定项 9 现文：`research/prompts/test-script-align-r3-main-verification.md` 第二节「V2 两笔矛盾」一行 |
```

## 三、实现今天的样子

主工作区现状（含里程碑四会话没提交的改动），2026-10-18 现读：

- 块键：`crates/singlefs-checker-tier/src/verdict_store.rs:355` `VerdictBlockKey { node_input_digest, enumeration_plan_hash, block_start }`；`crates/singlefs-checker-tier/src/crash_amplification.rs:359` `block_key`、`:406` `block_key_under`（复用键 + 计划哈希 + 点内块起点）。复用键 `crates/singlefs-checker-tier/src/crash_identity.rs:95` `StepAttributes`（`:185` `step_write_table_with_barriers` 带屏障、FUA、盘、偏移、内容摘要、段号的写表摘要；`:223` `SectorImage` 起步镜像的扇区终值摘要）。计划哈希含段展开名、判器另读的输入（`crash_amplification.rs:921` `JudgeInputsOfTheFlow`、`:1061` `digest_for_a_point`：基镜像盘、写表前缀摘要、往下游找的根槽写、够得着的版本表）与态数。判法单元键另走 `crash_identity.rs:619` `judging_units_key`，不进块键。
- 计划：`crash_amplification.rs:505` `plan_crash_points`（`:593` 起每个点各算一次 `SectorImage::digest`）；`:234` `CrashPointSpan`（`ignored` 字段）、`:247` `CrashFlow`、`:267` `PlannedCrashPoint`、`:319` `blocks`、`:175` `STATES_PER_BLOCK = 1 << 16`；`:12173` `period_of_write`（点名用，不进键）。
- 崩溃枚举用例那一路的节点切法：`crates/singlefs-checker-tier/src/crash_state_enumeration.rs:765` `crash_points_of_the_steps`（私有）；录入段打 `CRASH_POINT_REPLAY_FLOW_RECORDED flow= blocks= states=`（`:877`）；只查块键那一趟 `:1061` `blocks_queried_without_judging`（`SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1`，`:925`），提交时门禁快档逐条用它（`.claude/gate.d/checker-tier-crash-point-replay.sh:21`）。
- 态数：`crates/singlefs-harness/src/memory_pool.rs:659` `closed_form_state_count`（64 写以上的段 panic）；`crates/singlefs-checker-tier/src/crash.rs:2418` `crash_state_count_with_torn_in_place_overwrites`、`:2450` `quick_tier_expansion`、`:2459` `full_expansion`、`:1349` `crash_state_ranges_by_segment`。
- 切段：`crates/singlefs-harness/src/memory_pool.rs:549` `writes_and_segments`、`:579` `writes_and_segments_with_stream_indexes_and_entries`（「切段只有这一份实现」）。
- 随机历史：`crates/singlefs-harness/src/history.rs:1854` `generate_history`（比重 `GenerationWeights::BROAD`，`:920`）、`:1860` `generate_history_with_weights`、`:1411` `BROAD_WITH_COMPACTION_WHILE_MOUNTED`、`:7521` `execute_history_with`、`:128` `HistoryDeviceWidth`（四档）、`:3525` `HistoryRun`（`:3533` `operations_written_by_make_filesystem`、`:3538` `committed_versions_at_the_end`）、`:123` 种子基。随机历史录流、切段的现成用法在 `crates/singlefs-checker-tier/src/crash_injection.rs:1488`–`:1575`（崩溃注入：同一条切段规则、基线取 mkfs 之后）。随机历史生成器到 `CrashFlow` 的转换 crates 里没有：`grep -rn "CrashFlow" crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-harness/src` 零命中（正推腿报告 1.5 节同一条命令，这一轮主 agent 没复跑，执行员开工复跑一次记进产物）。
- 冒烟层流的建法：`crates/singlefs-harness/tests/common/mod.rs:747` `build_pool`、`:56` `parameters`；`crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs:475` `prepare_the_snapshot_crash_flow`；`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:44` `content_needing`、`:419` `overwrite_step`；测试文件私有的 `prepare`：`crash_enumeration_new_pool_file_creation_stream.rs:794`、`crash_enumeration_fixed_script_stream.rs:255`、`crash_enumeration_multi_record_publish_stream.rs:140`（都在 `crates/singlefs-checker-tier/tests/`）。bin 经 `#[path]` 引测试公共模块的先例：`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs:23`。
- 库：`crates/singlefs-checker-tier/src/verdict_store.rs:2077` `open_existing_for_reading_only`、`:3283` `recorded_blocks`；`crates/singlefs-checker-tier/src/crash_library.rs:593` `open_the_library_for_reading_only`、`:120` `toolchain_of_this_process`；读库程序 `crates/singlefs-checker-tier/src/bin/crash_library_reader.rs`（`keys` 子命令报录入块数与键对不上的块数）。
- 写路径上两处第一版恒 0 的字节（第二段的补丁位）：journal 记录头 MAC 16 字节，写在 `crates/singlefs-core/src/journal.rs:258`（`writer.skip(16); // MAC：第一版留位全 0`），读者在 `:352` 跳过不判；整理意图 value 末尾的补零，写在 `crates/singlefs-core/src/logged_operations.rs:245`（`writer.skip(COMPACTION_INTENT_PADDING_BYTES);`），`:29` 注释写「读者遇到非零一律拒收」。
- 崩溃放量入口：`scripts/crash/crash-amplification.sh`（`--item`、`--library <库目录>`、`--label`、`--threads`，`--list-items` 现列 20 项，含 `snapshot-creation`、`multi-record-publish`）；每项打 `CRASH_AMPLIFICATION_ITEM … states= … seconds=`，GPU 判段打 `CRASH_AMPLIFICATION_GPU_TOTAL … compile_seconds= judge_seconds= states_per_second= seconds_storing=`（`crash_amplification.rs:3969`，进程里计时）；录入段没有单独的秒数行（`grep -n 'CRASH_AMPLIFICATION_RECORDED' crash_amplification.rs` 那一行 `:12936` 不带秒数）。
- 层：`.claude/gate.d/crash-point-replay-items.tsv` 四列、没有层这一列；`.claude/gate.d/stage-inputs.tsv` 现有 `crash-case:` 29 行（第 46–74 行）。

## 四、跑之前已经存在的数

写判据之前我读到的数，照实列（来源、数、对判据的影响）。门槛（第六节）没有照其中任何一个数调；凡是下面的数已经暗示了某一格的答案，那一格在第六节另报原始区间，读者可以不认我的门槛自己判。

- 正推腿报告 `research/prompts/m3-close-c409-r1-sonnet-output.md`：
  - 1.2 节十一条固定写表常量（段数 / 写数 / 最长段 / 两态闭式）：lifecycle_closure 139 / 824 / 26 / 288751838；unlink_after_the_fixed_script 78 / 503 / 28 / 2013790337；compaction_relocating_a_block_shared_with_a_snapshot 48 / 272 / 26 / 123338833；compaction_relocation 40 / 238 / 30 / 1377370253；truncate_under_a_snapshot 35 / 223 / 30 / 2438004868；truncate_into_a_unit_then_extend 31 / 204 / 30 / 1695547518；snapshot_destruction_resumed_after_a_crash 39 / 204 / 26 / 84869186；truncate_to_zero 31 / 196 / 30 / 1628700798；overwrite_and_unlink_under_a_snapshot 27 / 153 / 26 / 167837744；unlink_under_a_snapshot 23 / 126 / 28 / 352387114；snapshot_creation 15 / 62 / 24 / 16842782。≥ 10 写的段 105 条、取值 16–30 八种；一次带单元的普通发布 2^n + 11 态；发布写条数 n + 5。
  - 1.3 节上千写的推算：n = 16、20、24、28、30 时 48、40、35、31、29 次发布，合计 3146256、41943480、587202945、8321499477、31138513215 态；第九节一万写：n = 16 → 477 次 31265919 态，n = 28 → 304 次 81604381968 态，n = 30 → 286 次 307090164810 态。
  - 1.8 节旧数：`CRASH_AMPLIFICATION_TOTAL items=7 failed=0 states=2563727926 red=0 seconds=27288`、「到 E 那一项平均每秒约 9.9 万态」；录入约占整趟 78%（推的）、「录入 508 秒、判段 663 秒」；E161 段头 4096 态 `state_elapsed_ns=30213684776`（约 7.4 毫秒每态）；轻型整趟 `seconds=1856`；库 `du -sh` 7.0G；2026-09-15 以来 196 次提交、碰 crates 的 35 次；旧数「每条大流约 480–520 秒」录入。
  - 其余：io_min 8192 那条流 33 态；`multi-record-publish` 2.68×10^8 态（268435552）；`snapshot-creation` 1.68×10^7 态；`StepKind` 7 个成员、`CommitStep` 5 个；登记行 29 条、合表 28 项四列、放量 20 项；给定状态那条三段 32 种子 × 150 步。
- 攻方腿报告 `research/prompts/m3-close-c409-r1-opus-output.md` 与模型产物：
  - 随机历史（BROAD、两块 4 GiB 盘、种子基起连续 20 个种子，mkfs 之后）五档（最小/中位/最大）：20 步写 36/444/634、最长段 12/28/30、log2 两态闭式 12.01/29.01/30.94、log2 甲二下界 5.09/7.21/7.74；50 步 382/1040/1464、26/28/30、27.67/30.38/32.43、7.47/8.54/9.27；100 步 1008/2262/2752、26/30/36、28.60/31.09/36.22、8.69/10.16/11.73；200 步 2912/4328/5509、28/32/36、29.36/33.44/36.81、10.10/11.99/15.22；400 步 4817/8580/10707、28/34/40、31.46/35.06/40.11、11.32/13.97/17.27；`segments_ge64=0`、`panicked=0`、7 个种子两遍写流摘要相同。
  - 种类串去重：50 步 3397 段 22 种串 2^35.17 → 2^31.77；100 步 7058 段 30 种 2^37.96 → 2^36.53；200 步 14534 段 35 种 2^39.03 → 2^37.47。
  - 冒烟层对照：固定脚本到卸载 1678377204（≈ 2^30.6）、truncate_under_a_snapshot 2438004868（≈ 2^31.2）、并行线一那条流 12230590578（≈ 2^33.5）；崩溃注入快档 24 种子 × 24 步、每段 4 个取样点；顺序复跑约 8 分钟（五件并行 187 秒、三件并行 106 秒）。
- 判决 `research/prompts/m3-close-c409-r1-main-verification.md`：cite-check 92 处引文、93 处裸行号；攻方模型复跑 `rerun_compare result=identical`。
- 记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`：E180 执行员冷编时本机负载约 340、前台 240 秒编不完；`crash-case:` 29 行、合表 28 项、放量 20 项（与上面同）。别的数（盘点行数、stash 份数、变异行数、自证格数）与这一份的量无关。
- 测试文件常量（第三节现读）：新池新建文件流 `FULL_STATES_WITH_TWO_STATES_PER_WRITE = 16_777_240`、`FULL_STATES = 16_777_260`、`QUICK_TIER_STATES = 46`（`crash_enumeration_new_pool_file_creation_stream.rs:762`、`:766`、`:770`）；固定脚本到卸载 `1_678_377_089`、`1_678_377_204`、快档 `278`（`crash_enumeration_fixed_script_stream.rs:99`、`:104`、`:119`）；建快照流段形 `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]`（`crash_enumeration_snapshot_creation_stream.rs:48`），文件头写枚举域 16842807、两个单元段占 16842750、两态闭式 16842782；多记录流文件头（`crash_enumeration_multi_record_publish_stream.rs:22`）写「两段同样占 1342177278；C577 之前轮换并进单元段，12230590578 与 5435818083」。
- 我自己算的（第十三节命令）：建快照流两态闭式 16842782；它前 11 段 `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]` 写数 41、两态闭式 16777240；65537 态切 2 块、65536 态切 1 块；log2(16842782) = 24.0056。
- `--emit-hosts` 的输出（私有主机值，不抄）：双机、开 GPU。

对判据的影响：
1. 攻方模型的数已经说明随机历史的两态闭式（2^12–2^40）与冒烟层登记流（2^24–2^33.5）大面积重叠，J1-乙 很可能判「重叠」。门槛（重叠占比 ≥ 0.5 记重叠、≤ 0.1 记分开）是写判据时取的整数分界，不是从这几个数凑的；我接受反过来的结果：R 族在上千写那一截落在区间外的占比 ≥ 0.9，就记「分开」，甲那一读成立。
2. D13 已定项 9 射程里的 12230590578 是多记录流在 C577 之前的数（测试文件头自己这样写）。锚 B1（第七节）拿它比装置重录的多记录流，预计不符（C577（系统配置没见证到的最新根，乙罩不到） 改了段形）、触发「条款这一格可能错」（败 F6），不作废。
3. 正推腿的旧吞吐（约 10^5 态每秒）只当量级参考：J2 的挂钟档由这一次冷跑的 `R_total`、`R_judge` 现算，不用旧数。
4. 碰 crates 的提交 35/196 是上界：J3-乙 照第五节钉住的提交清单现算，不用这个比例。

## 五、臂、阳性对照、真实基线

### 5.1 臂（每条臂是一族负载；岔路 G1 的三个候选各自划进常规层的就是这几族）

- **臂 R（固定种子随机历史整段全量展开）**——G1 乙的负载；G1 甲、丙要收随机历史时收的也是它。怎么做：种子 `种子基 + i`（i = 0..19，20 个，种子基见文件头常量表），`generate_history_with_weights(HistorySeed(种子), 步数, &比重)`，在两块 `盘宽` 的内存盘上 `execute_history_with` 跑、录制流开内容保留（同崩溃注入，`crash_injection.rs:1488` 起），流照 1.1 节切开。主取样：比重 `BROAD`、盘宽 `FourGibibytes`、步数四档 50、100、200、400。版本表取 `HistoryRun::committed_versions_at_the_end`：内容是 `Some` 的进 `versions`（`PublishedVersion { instance, checkpoint_txg, content }`），`None` 的进 `versions_without_a_file`。历史没跑完（`ending` 不是 `Completed`）的照录到停下那一步、单列一行 `ending=`，不换种子补数。做完会怎样（这条臂的主张，由 J1-丙 检验）：随机历史与冒烟层登记流来源不同，块键几乎不共用，所以它在冒烟层之外加了真的新块。推得出：块键含起步镜像，基镜像或前缀写流一处不同，之后每个点的复用键都不同（`crash_identity.rs:95`、`:223`）。
- **臂 F（覆盖写链：固定脚本加长到上千写）**——G1 甲、丙在固定脚本一侧会收的负载，照正推腿报告「覆盖写链」那一条候选写成支持它的人认的样子。怎么做：`build_pool` 起标准前缀（取号、暖机、第一个文件 A，与新池新建文件流同一份写法），之后对第一个文件逐次覆盖写，第 k 次（k 从 1 起）内容取 `content_needing(u, k)`（`common_crash_points/mod.rs:44`），经 `overwrite_step`（`:419`，录崩溃点的那一路）发布，直到写数 ≥ 1000 为止，覆盖写次数 K 由装置数出来、写进产物；u 取 1、4、16 三档（每次覆盖写的数据单元数）。版本表照录崩溃点那一路的记录器交回的那一份。做完会怎样：上千写的固定脚本负载，与冒烟层登记流只共用标准前缀那几块。推得出：同上，A 之后的写字节与登记流不同（内容种子与次数不同）。
- **真实基线（冒烟层）**：R4 四条重录流（1.1 节）与区间 S。它们是今天提交时快档只查库、全量由用户要求时跑的那一层。

结果反过来我接受：臂 R 的共用占比 ≥ 0.5，就记「随机历史与冒烟层共用多数块」，乙的「按来源分」名不副实；臂 F 的共用占比 ≤ 0.05，就记「覆盖写链在冒烟层之外加了真的新块」，丙的「并进冒烟层」不是只换名字。

### 5.2 阳性对照（每一条臂都跑：对照那段循环遍历 R 的每个步数档种子 0、F 的每个 u、R4 的每一条，不许只拿第一条跑）

| 对照 | 怎么造 | 必须得到 | 证明什么 |
|---|---|---|---|
| 自比 | 这条流对一个只含它自己的比较集算共用 | 共用占比恰好 1（共用块数 = 它的块数） | 共用计数数得到共用 |
| 基镜像改一扇区 | 复制这条流，在基镜像里挑一个流里没有任何写碰到、基镜像里也没写过的扇区（盘 0 上号最大的那一个，装置核它没被写过），写成 512 个 0xFF | 与原流的共用块数恰好 0 | 共用计数分得出起步镜像的差别 |
| 同流两种展开 | 同一条流各按 `full_expansion` 与 `quick_tier_expansion` 排计划 | 复用键共用占比恰好 1；块键共用块数恰好等于两份计划里不管任何段的点（末节点在内）的块数之和 | 块键分得出计划哈希的差别，复用键分不出 |
| 名字不进键 | 同一条流换项名与设点文件再排计划 | 块键逐个相同（共用占比 1） | D13 已定项 17「名字、路径不进键」在装置这一路上成立 |
| 转储往返 | 同一条流直接算块键，与写转储、读回再算的块键比 | 逐个相同 | 第二段旧提交那一路（转储再算）与直接算是同一个数 |

J2、J3、J4 的阳性对照写在 5.4、5.5 各自那一段里。

### 5.3 装置（两份 bin，执行员在草稿副本里写；`scripts/hosts/compile-then-swap.py` 换进主工作区）

`e184_regular_layer_cost_counts_write_dump`（只引 harness、core、format 与 `#[path]` 引的 `crates/singlefs-harness/tests/common/mod.rs`；第二段要在旧提交的副本里编）：
- `dump-random-histories --steps <N> --weights <BROAD|BROAD_WITH_COMPACTION_WHILE_MOUNTED> --width <FourGibibytes|UnitAreaOf240Slots> --seeds 20 --out <目录>`：每段历史一份转储（mkfs 那几步的写与清零写、mkfs 之后的写表与段、版本表、`ending`），文件名带种子，写到临时名再改名；末行 `name=dump_done histories=<n>`，打每份转储的 sha256。

`e184_regular_layer_cost_counts`（全部子命令）：
- `selfcheck`：停机 T1、T2、T3 与第七节锚点逐条核，打 `name=stop_check item=<名> result=<pass|fail> …`；有一条 fail 退 3。
- `record-fixed-chains --out <目录>`：臂 F 三档与 R4 四条，各写一份转储（与上面同一个格式）。
- `keys --dump <目录> --out <目录>`：逐份转储重建 `CrashFlow`（节点照抄来的 `crash_points_of_the_steps` 切）、`plan_crash_points`、逐块 `block_key`，交回每条流一行 `name=flow …`（字段见第六节 J1-甲）与一份块键文件（每块一行：复用键、计划哈希、块起点、这一块的态数，十六进制），末行 `name=keys_done flows=<n>`。
- `share --candidates <目录> --against <目录> --out <文件>`：J1-丙 与 5.2 的对照，按第六节写法打行。
- `library --keys <目录> --library <绝对路径>`（附带）：只读开长期库（`open_existing_for_reading_only`），数每条候选流的块键在录入表里的个数。
- `compare --before <目录> --after <目录> --out <文件>`：J3，逐条流报第一处不同的写（写表下标、所在段号、所在点号）、作废块数（after 里不在 before 里的块）、作废态数、两边块数。
- `compose --j1 <文件> --j2 <文件> --j3 <文件>`：J2-乙 的纯算术合算（第六节）。
- 并行：流与流之间互相独立，按流切片并行，线程数取 `E184_THREADS`，没设取 `SINGLEFS_THREAD_CAP`、再没设取 `.claude/heavy-thread-default` 的登记值；合并按流的次序；单线程与多线程在 R 的 50 步档上逐字对拍（单测）。每完成一条流往 stdout 打一行进度（流名、已完成条数、用时，进程里计时）。
- 退出码：0 正常；2 命令行不对；3 停机条款没过（产物照打、交主 agent）；4 阳性对照没过（作废，第十一节）。

### 5.4 J2：冷吞吐与库增量（重型：崩溃放量真跑。主 agent 跑，起之前先弹窗问用户）

两项各起一次，各用一个新建的空库目录（两项不共库：两项共标准前缀，共库就不冷）。项按态数取一小一中：小项 `snapshot-creation`（测试文件头写枚举域 16842807 态），中项 `overwrite-and-unlink-under-a-snapshot`（正推腿表里两态闭式 167837744，约小项的 10 倍；不取 `multi-record-publish`：它的枚举域在 10^9 以上，冷跑以小时计、超出这一段的预算）。命令（工作目录仓根；线程照 `.claude/heavy-thread-default` 的登记值，经 `capped.sh default` 起；内存上限照入口自己的 `--memory-cap` 默认，入口在分项驱动里逐项套包装）：

```
D="$HOME/.local/share/singlefs/work/e184-j2-cold"
mkdir -p "$D" && test -z "$(ls -A "$D")"
SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃放量入口点名 snapshot-creation; 范围=1 项; 为什么=E184 第三段 J2：新建空库目录上冷跑一个已登记的小项，量今天的树上的冷吞吐与库每态字节' \
  bash scripts/hosts/capped.sh default bash scripts/crash/crash-amplification.sh --item snapshot-creation --library "$D/library-small" --label e184-cold-small \
  > "$D/small.log" 2>&1; echo "exit=$?" >> "$D/small.log"
SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃放量入口点名 overwrite-and-unlink-under-a-snapshot; 范围=1 项; 为什么=E184 第三段 J2：新建空库目录上冷跑一个已登记的中项，与小项两点出斜率' \
  bash scripts/hosts/capped.sh default bash scripts/crash/crash-amplification.sh --item overwrite-and-unlink-under-a-snapshot --library "$D/library-medium" --label e184-cold-medium \
  > "$D/medium.log" 2>&1; echo "exit=$?" >> "$D/medium.log"
du -sb "$D/library-small" "$D/library-medium" > "$D/library-bytes.txt"
```

（`--library` 要的是绝对路径、开跑前不存在或为空；入口照多机配置清场、双机、上 GPU，这一趟的台数、卡与清场结果原样留在两份日志里。）跑完再各读一次录入块数：`bash scripts/hosts/run-with-memory-cap.sh 8G <编好的 crash_library_reader> keys --library "$D/library-small"`（中项同），读 `recorded_blocks=`。

读哪几行：每项的 `CRASH_AMPLIFICATION_ITEM item=… exit= states= reused= … seconds=`（`states=` 是现判态数，冷库上 `reused` 应为 0）；分项日志里的 `CRASH_AMPLIFICATION_GPU_TOTAL … compile_seconds= judge_seconds= states_per_second= seconds_storing=`（被测进程里计的时）。录入段用时 = 项的 `seconds` − `compile_seconds` − `judge_seconds` − `seconds_storing`（推的，四个数都是进程或入口自己计的，不拿转打循环里行到达的时刻）；包含自检：`seconds` ≥ 后三者之和，包不住的那一项不进统计（`.claude/singlefs-ai-sop/rules/test-discipline.md`「分段计时在被测进程里计」）。日志与 `library-bytes.txt` 由第三段的执行员抄进产物（5.6）。

J2 阳性对照：中项的 `states` 比小项多，`seconds` 也必须比小项多（两点分得出规模）；小项的 `states` 必须等于锚 A7（第七节）。真实基线：两项都是今天已登记的冒烟层流本身。

### 5.5 J3 与 J4 的副本与原型

J3-甲（合成补丁，只在仓副本里改，补丁不回主工作区）：草稿目录里建三份副本（只拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，拷的时候不保留修改时刻），各一个编译目录：
- `null`：不改一个字节（阳性对照：与第一段主工作区那一份比，作废块数必须恰好 0）。
- `journal-mac`：`crates/singlefs-core/src/journal.rs` 里 `writer.skip(16); // MAC：第一版留位全 0`（第 258 行，恰好命中一次）换成 `writer.put_u8(1); writer.skip(15); // E184 补丁：MAC 第一字节写 1`。每次发布的 journal 记录都带它，读者在 `:352` 跳过不判：作废面最大的那一头。
- `compaction-padding`：`crates/singlefs-core/src/logged_operations.rs` 里 `writer.skip(COMPACTION_INTENT_PADDING_BYTES);`（第 245 行，恰好命中一次）换成 `writer.put_u8(1); writer.skip(COMPACTION_INTENT_PADDING_BYTES - 1);`。只有整理意图写它：`BROAD` 比重的随机历史与覆盖写链都不整理，是方向相反的取样点；`BROAD_WITH_COMPACTION_WHILE_MOUNTED` 比重的历史整理时写它（读者见非零拒收，那一段历史之后的写流跟着变，照样数作废块）。
每份副本编两份 bin，跑：R 的 100 步档 20 个种子（`BROAD`、`FourGibibytes`）、R 的 100 步档 20 个种子（`BROAD_WITH_COMPACTION_WHILE_MOUNTED`、`FourGibibytes`）、F 三档；`keys` 一律用主工作区那一份编好的 bin 算（块键算法不随副本变），`compare` 与第一段的同一组转储比。

J3-乙（真实提交）：钉住的提交清单（`git log --format='%H' -n 13 -- crates/`，2026-10-18 取）：
`df5866f6706a3378563c9b0aac1403992ba06ac5`、`7b5711fd4b39cb9584bda0b8f8c3c819357fcd5b`、`d6dd6da42834476cbb6298504204438f05edeecf`、`73866746b31e6696f7a71e837163f5d0c47f32ba`、`f8a9ddaab885073cc311b708042cbbf94cd828a9`、`40412d3045a7d50bb7356572fcc40d4bd66e1957`、`c34be3b40bd210391341dd98d526c071ddb04949`、`6c1509edd7c333e9bda87da8834a2aa4fc04ea70`、`c3e9d9edbe45a84fb75870b4988ff169bf0b5090`、`5d6374691099e22aaffb8072be884040415e32ec`、`13ca0a19a69c6220b0ffc6477c424d5a012b2279`、`deee47f4234b8a26212e7716c1920804ef001414`、`0c7fdb82b4a759436d85043c412dcec944c684af`。
每个提交：`git archive <提交> crates Cargo.toml Cargo.lock .cargo | tar -x -C <副本>`（只读仓，不建 git 工作树），放进转储 bin 的源文件，编、跑 `dump-random-histories --steps 100 --weights BROAD --width FourGibibytes --seeds 20`；再用主工作区那一份 `keys` 算块键。相邻两个提交（按时间次序）各比一次，共 12 对；主工作区（含没提交的改动）对 `df5866f67` 再比一次，共 13 对。编不过、或旧提交里缺转储要的接口（`operations_written_by_make_filesystem`、`committed_versions_at_the_end` 这类）的提交记「取不出」，牵连的两对都记「取不出」，不记 0。判器单元键变没变：两边都编得出 `judging_units_key(JudgementOfTheCrashImages)` 的那几对另报一列，别的写「取不出」。阳性对照：同一个提交对它自己（取清单里第一个编得过的）作废 0。
只比随机历史一族：覆盖写链要的录崩溃点接口在旧提交里改过多次，放进转储 bin 会让旧提交大面积编不过；覆盖写链的作废面由 J3-甲 罩。

J4（两种落法的固定开销；G2 由用户定了之后才跑，第四段）：仓副本里造两份原型，都只转调同一个入口、同一条登记行（`crash-case:snapshot-creation-stream`，快档那一路只查库）：甲 = 在 `.claude/gate.d/checker-tier-crash-point-replay.sh` 里加一格；乙 = 新建一道阶段文件，文件头照已有阶段写全、`gate-similar` 说明转调同一入口。基线 = 不加格、不加阶段。三种各跑 5 轮、交错次序，外层挂钟（命令前后各取一次单调时钟）与门禁自己报的这一道 / 这一格秒数各记一个，外层必须包住里层。已知开销对照：再造一份「乙 + 阶段里先睡 30 秒」，测出的增量减去乙的增量必须落在 30 ± 5 秒（证明这套计时分得出 30 秒）。原型由主 agent 定派谁写（只在副本里，不碰主工作区的门禁文件）；门禁阶段在副本里跑也是 checker 档那一道，归主 agent 起。

### 5.6 分段、实现量与每段的挂钟

实现量：负载两族（R：主取样 4 档 × 20 段，敏感性 2 档 × 20 段；F：3 档）加基线 R4 四条；几何格见第八节；没有崩溃支线、没有坏镜像；要另写的子系统只有随机历史与覆盖写链到 `CrashFlow` 的转换（crates 里没有，第三节）、转储格式与共用 / 作废的计数。一个执行员一次做不完，按岔路排序分四段。每段交回后主 agent 对着岔路单判续不续，一行开着的都不剩就停，后面的段不跑。

| 步 | 段 | 谁 | 做什么 | 补岔路哪一行 | 产物 |
|---|---|---|---|---|---|
| 1 | 第一段 | 执行员 | **装置写完先交主 agent 经 `scripts/hosts/compile-then-swap.py` 后台编**：草稿副本里写两份 bin（内联单测、文件头、`crates/mutations.tsv` 变异行草稿），自己不编、不跑，交回时写明两条 `compile-then-swap.py` 命令 | — | 草稿目录里的两份源文件 |
| 2 | 第一段 | 主 agent | 后台起 `bash scripts/hosts/capped.sh default python3 scripts/hosts/compile-then-swap.py <草稿副本> crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs --scratch <草稿目录> --release`（`…_write_dump.rs` 同）；编过之后跑两份 bin 的内联单测（checker 档包的测试，重型：带 `SINGLEFS_HEAVY_TESTS=user-request` 与 target 范围声明） | — | 主工作区两份 bin；单测日志 |
| 3 | 第一段 | 续派的执行员 | `selfcheck`（停机 T1–T3、锚点）→ 5.2 阳性对照（全部臂）→ R 主取样与敏感性的 `dump-random-histories`、`record-fixed-chains`、`keys`、`share`；最后附带的 `library` | G1：J1 够判条件全部量到就够判 | `research/results/e184-regular-layer-cost-counts-segment-one-<日期>.out` |
| 4 | 第二段 | 主 agent | 后台编 J3-甲 三份副本、J3-乙 13 个提交的副本（每份副本各一个编译目录；编译输出里核改过的包有 `Compiling` 行） | — | 草稿目录里的编译日志 |
| 5 | 第二段 | 续派的执行员 | 各副本跑转储、主工作区 bin 跑 `keys` 与 `compare` | G2：J3 两项 | `research/results/e184-regular-layer-cost-counts-segment-two-<日期>.out` |
| 6 | 第三段（**重型，先弹窗问用户**） | 主 agent | 5.4 两条崩溃放量命令与读库 | — | `~/.local/share/singlefs/work/e184-j2-cold/` 下两份日志与 `library-bytes.txt` |
| 7 | 第三段 | 续派的执行员 | 把第 6 步的日志行原样抄进产物，跑 `compose` 合算 J2-乙 | G2：J2 两项（连同第二段就够判） | `research/results/e184-regular-layer-cost-counts-segment-three-<日期>.out` |
| 8 | 第四段（G2 由用户定了之后才跑） | 主 agent 定派谁写原型；门禁那一道由主 agent 起 | 5.5 的 J4 | G3：J4 | `research/results/e184-regular-layer-cost-counts-segment-four-<日期>.out` |

第二段与第三段互不依赖，第一段交回后可以同时派；第四段等 G2。G3 若由用户对「立阶段」的读法先定掉，第四段标「够判后未跑」。

每段挂钟（推的，都没量）：
- 第一段：编译 10–40 分钟（checker 档包要现编 `librocksdb-sys`；E180 执行员在本机负载约 340 时前台 240 秒编不完）；单测几分钟；产物里转储 120 段历史约 10–20 分钟（攻方腿的只数闭式模型顺序跑 100 段约 8 分钟），`keys` 每个点算一次整张父镜像的摘要（`crash_amplification.rs:593` 起），400 步的历史上千个点，单条流几十秒到几十分钟没量，合计 0.5–6 小时。**按超过 2 小时备好分片与续跑**；执行员先跑 50 步档 20 段，量出每条流的平均用时写进进度，按它报整段的估计。
- 第二段：16 份副本编译（一份冷编 10–40 分钟，其余在同一个编译目录里增量编、每份 5–15 分钟）加转储与 `keys`，合计 2–6 小时：**超过 2 小时**。
- 第三段：小项 1.7×10^7 态、中项约 1.7×10^8 态；按正推腿的旧量级 10^5 态每秒（今天没量）判段约 3 分钟与 30 分钟，加录入、清场与 GPU 编译，合计 0.5–2 小时；小项跑完超过 30 分钟就停下、先不起中项，交主 agent 定。
- 第四段：四种 × 5 轮，每轮一次只查一条登记行的门禁，单轮 2–10 分钟，合计 40 分钟到 3.5 小时；超过 2 小时照下面三行办。

分片：第一段一条流一片（R 每个种子每个步数档一片、F 每档一片、R4 每条一片）；第二段一份副本或一个提交一片，转储与 `keys` 同一片里做；第四段一轮一片。片与片之间不共享状态。
续跑：同一条命令重起只补没做完的片：每片的转储目录与块键文件写到临时名、写完改名，末行 `name=dump_done` / `name=keys_done` 在的片跳过；`compare`、`share` 只读做完的片，缺片的那一行写「取不出」，不写 0。
多机：台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量。
第二台拿到的片照同一条命令跑，产物取回本机按片的次序合并；第二段的编译各台各编各的副本。第三段的台数与卡照崩溃放量入口自己的多机配置。

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定，不合成一个真假。门槛先与臂的定义并排看过：臂 R、F 的定义只说负载怎么造，没有一句推得出它与冒烟层的重叠占比、共用占比或挂钟落在哪一档。

| 量 | 怎么算 | 门槛与判定 | 岔路行 | 翻面取值 |
|---|---|---|---|---|
| J1-甲 每条流的规模 | `keys` 每条流一行：`name=flow family= tier= seed= ending= writes= segments= longest_segment= two_state= log2_two_state= full= quick_tier= blocks_full= blocks_quick= points= keys_seconds=`（`two_state`、`full`、`quick_tier` 装不进 u64 写 `beyond_u64`；`keys_seconds` 是这条流在进程里算计划与块键的用时）；每族每档另报一行 `name=family_summary`：上面各量的最小 / 中位 / 最大，与写数 ≥ 1000 的流数 | 不设门槛，供 J1-乙、J2-乙 用；问题单 J1 要的写数、段数、最长段、两态闭式、甲二（精确数）都在这一行 | G1 甲 | 某族有流 `two_state=beyond_u64`：甲「登记全量态数」对那条流只能按已定项 16 分片登记，记一行 |
| J1-乙 态数区间重叠 | 冒烟层区间 S = [S_min, S_max]（1.1 节）；每族只取写数 ≥ 1000 的流：落在 S 里的占比 `inside`、高于 S_max 的占比 `above`；最长段同样算一遍（冒烟层最长段区间对比） | `inside` ≥ 0.5 记「重叠」，≤ 0.1 记「分开」，其间记「部分」；另报原始区间 | G1 甲 | R、F 两族都「重叠」且 `above` 都 < 0.5：甲按数分层分不开（翻离甲）；有一族 `above` ≥ 0.5：甲可以把常规层写成「高于冒烟层上界」那一截 |
| J1-丙 与冒烟层共用块键 | 每条流对 R4 块键并集的共用占比（按块）；另报按态数加权的占比与复用键共用占比（上界）；每族报中位与最大 | 族中位 ≥ 0.5 记「换名」；族最大 ≤ 0.05 记「独立」；其余记「部分」 | G1 乙、丙 | R「换名」：乙按来源分名不副实（翻向丙）；R「独立」：乙加的是真的新块。F「换名」：覆盖写链只是冒烟层换名（丙）；F「独立」：丙并进冒烟层不只是换名，冒烟层要多判这些块 |
| J1-库 长期库里已录的同键块 | `library`：每条流的块键在长期库录入表里的个数 ÷ 它的块数 | 附带，够判后不跑；只在 J1-丙 有一族判「部分」时跑，用来看 R4 之外的登记流有没有共用块 | （G1 旁证） | 某族的库占比比 J1-丙 高 0.1 以上：R4 没罩住的登记流与它共用块，J1-丙 那一族改记「R4 罩不全」 |
| J2-甲 冷吞吐与库增量 | 5.4：两项各 `R_total`、`R_judge`、录入段用时（推的）、库字节 ÷ 现判态数、库字节 ÷ 录入块数；两点拟合 秒 = 固定 + 态数 ÷ 斜率（按 `seconds` 与 `judge_seconds` 各拟一条） | 不设门槛，供 J2-乙 用 | G2 | — |
| J2-乙 提交那一趟的挂钟落在哪一档 | `compose`：常规层名义负载 N = R 的 100 步档 20 段 ∪ F 三档。每个 G2 候选 × 提交类别算一格：甲 = N 的 `keys_seconds` 之和 + 作废态数 ÷ `R_judge`（写路径提交的作废占比取 J3-乙 的中位，另用 J3-甲 `journal-mac` 的占比算一格上界；只动判器的提交作废占比取 1）；乙 = 同一式子，态数换成甲二态数；丙 = N 的 `keys_seconds` 之和（只查库），另列一格「主 agent 起的全量」= N 的全量态数 ÷ `R_total`；丁 = 2 小时与 24 小时两个预算下按点的态数升序开、开得下的点数占比与态数占比 | 每格落 ≤ 2 小时 / 2–24 小时 / > 24 小时 哪一档 | G2 | 甲的写路径提交落 > 24 小时：甲不能照字面每次提交跑（翻离甲）；乙的写路径提交落 ≤ 2 小时：乙在提交时跑得完；丁在 2 小时预算下开得下的态数占比 < 0.1：丁在提交时等于几乎全 ignore |
| J3-甲 合成补丁的作废面 | `compare`：每条流第一处不同的写（写表下标、段号、点号；基镜像就不同写 `base`）、作废块占比、作废态占比；每个补丁位 × 每族报中位与最大 | 不设门槛（机制量）；供 J2-乙 当上界 | G2 | — |
| J3-乙 最近提交的键对不上比例 | 13 对提交（5.5）各一行：作废块占比（20 段历史的块合起来算）、判器单元键变没变（取不出照写）；汇总：算得出的对数、作废占比 > 0 的对数、峰值、中位、主工作区对 `df5866f67` 那一对的值 | 中位 ≥ 0.5 记「写路径提交大多作废过半」；≤ 0.1 记「块级复用省下大半」；其间「部分」；算得出的对数 < 10 记「还差」 | G2 | 「作废过半」：块级增量在动写路径的提交上省不下多少（翻离甲，偏丙、丁）；「省下大半」：甲每次提交只判少数块 |
| J4-甲 两种落法的固定开销 | 5.5：基线、甲、乙各 5 轮；增量 = 变体 − 基线（外层挂钟），D = 乙的增量 − 甲的增量；5 轮各报一次 | 5 轮的 |D| 都 ≤ 60 秒记「固定开销分不出落法」；5 轮的 D 都 > 60 秒记「另立多花 D」；其余记「不稳定」 | G3 | 「分不出」：G3 只按用户对「立阶段」的读法定；「另立多花 D」：把 D 交用户当乙的代价 |

## 七、钉绝对值的断言

每条互比（J1-丙 的共用、J3 的作废）旁边都有钉死的数：三族一起算错时，下面这些格会红。

### 7.1 出自被测条款本身的（不符时两边都查；都核过仍不符，记「条款这一格可能错」，败 F6）

| 锚 | 条款句 | 断言 |
|---|---|---|
| B1 | D13 已定项 9 射程「登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态」 | R4 里多记录流的全量态数 = 12230590578（第四节已写明预计不符：测试文件头说这是 C577 之前的数） |
| B2 | D13 已定项 4「一次写取两态……原地覆写……多取第三态」 | 每条流：两态闭式 ≤ 全量态数；流里没有可撕的原地覆写时两者相等 |
| B3 | D13 已定项 17「一律不进键……路径（仓根也算）、名字、种子串」 | 5.2「名字不进键」：换项名与设点文件，块键逐个相同 |
| B4 | D13 已定项 16「整域态数装不进 u64 时按固定大小的片依次判」 | 一段 64 写的合成流上 `closed_form_state_count` panic（单测），装置把这种流记 `beyond_u64` 不报数 |

### 7.2 独立算出、用命令核过的（第十三节命令；不符先查装置，装置对了再查 crates，两边都查完仍不符按停机 T3 办）

| 锚 | 断言 | 出处 |
|---|---|---|
| A1 | 建快照流两态闭式 = 16842782 | `python3` 按段形 `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]` 现算（第十三节命令三） |
| A2 | 新池新建文件流两态闭式 = 16777240（建快照流前 11 段 `[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`，写数 41） | 同上命令；与测试常量 `crash_enumeration_new_pool_file_creation_stream.rs:762` 相等 |
| A3 | 一个点 65537 态切 2 块，65536 态切 1 块 | 同上命令；单测拿合成流核 `PlannedCrashPoint::blocks` |
| A4 | 本地每块状态数 65536 = `STATES_PER_BLOCK`；本地种子基 = `SEED_BASE_DRAWN_FOR_THIS_TEST_CYCLE`；R 第一段历史的种子 = 种子基 | 文件头常量表 |
| A5 | R4 重录值等于测试钉的常量：新池新建文件流 两态 16777240、全量 16777260、甲二 46；固定脚本到卸载 两态 1678377089、全量 1678377204、甲二 278；建快照流 段形同 A1、两态 16842782、全量 16842807 | 测试文件 `:762`、`:766`、`:770`；`crash_enumeration_fixed_script_stream.rs:99`、`:104`、`:119`；建快照流文件头与 `:48` |
| A6 | 多记录流的段形与写数等于它测试文件里钉的常量（执行员 grep 出原值写进修订，找不到写「没钉」） | `crash_enumeration_multi_record_publish_stream.rs` |
| A7 | J2 小项 `states=` = 16842807 = 装置重录建快照流的全量态数 | 建快照流测试文件头；第一段产物 |
| A8 | J2 中项 `states=` = 它测试文件钉的全量常量（执行员 grep；没钉写「取不出」，不作废） | `crash_enumeration_overwrite_and_unlink_under_a_snapshot_stream.rs` |
| A9 | `null` 副本对主工作区：每条流作废块数 = 0 | 5.5 |
| A10 | `journal-mac` 副本对主工作区、在 F 每档上：第一处不同的写 = 这条流写表里第一条种类是 journal 记录的写（装置另按写的种类逐条扫一遍得出）；基镜像里 mkfs 就写了 journal 记录、两边基镜像摘要不同时，第一处不同 = `base` | 5.5 |
| A11 | `compaction-padding` 副本在 `BROAD` 比重的 R 与 F 上：每条流作废块数 = 0（`BROAD` 比重表 `history.rs:920` 起没有 `CompactWhileMounted`；装置另数这些历史里整理操作的次数 = 0 当独立路） | 5.5 |
| A12 | 5.2 五个对照的数（自比 1、改一扇区 0、两种展开的复用键 1 与块键等于不管段的点的块数之和、名字不进键 1、转储往返逐个相同） | 5.2 |

## 八、轨迹与几何敏感性

### 8.1 轨迹（被条款当谓词输入的量）

- J3-乙 的作废占比是 G2 的谓词输入：报 13 对里算得出的对数、作废占比 > 0 的对数（为正轮数）、峰值、中位与期末值（主工作区对 `df5866f67` 那一对），不只报中位；正文不许写「每次」「从来」这类说轨迹的词，除非 13 对都算得出且都满足。
- J2-乙 丁那一格是预算对开得下的节点的谓词：报 2 小时、24 小时两个预算各自开得下的点数与态数，连同按点态数升序的累计曲线（每 10% 点数一个刻度）。
- J1-甲 按步数档报：每档最小 / 中位 / 最大，四档连起来看态数随步数怎么长；不只报 400 步那一档。

### 8.2 几何敏感性（`.claude/rules/mutation-sampling.md` 第六类：至少一个方向相反的取样点）

| 判决 | 主取样 | 方向相反的取样点 | 记法 |
|---|---|---|---|
| J1-乙、J1-丙（R） | `BROAD`、`FourGibibytes`、100 步 | 盘宽 `UnitAreaOf240Slots`（单元区小、回收与落点拒绝走得到）100 步 20 段；比重 `BROAD_WITH_COMPACTION_WHILE_MOUNTED` 100 步 20 段（带整理，写流里多整理那几种发布） | 两个取样点上照同一门槛重判；判定与主取样不同就记「不稳定」，相同记两点 |
| J1-乙、J1-丙（F） | u = 4 | u = 1 与 u = 16（每次覆盖写的单元段一短一长） | 同上 |
| J1-乙（步数） | 写数 ≥ 1000 的全部流 | 只取 50 步档（多数不到上千写）重算 `inside` | 同上 |
| J3-甲 | `journal-mac` | `compaction-padding` | 两个补丁位各报，不合并 |
| J2-甲 | 小项 | 中项 | 两点各报，拟合另报 |
| J3-乙 | 作废占比按块 | 按态数 | 两种各判一次门槛 |

判别力自证（单测）：把 J1-乙、J1-丙、J3-乙 的门槛挪到两个取样点的值中间，判定必须由一边换到另一边。

## 九、变异

执行员照 `crates/mutations.tsv` 第 1 行表头的六段格式，把下面每一条写成一行追加在表末（原文在装置源码里恰好命中一次；替换文编得过；「必须红的测试名」点名单测）。每条写明它在哪个取样点上改变输出；没有敏感取样点的不登记，先补取样点。

| 变异 | 改什么 | 在哪个取样点上改变输出（那条单测红） |
|---|---|---|
| V1 | 共用判定只比复用键、不比计划哈希 | 5.2「同流两种展开」：块键共用块数从「不管段的点的块数之和」变成全部块 |
| V2 | 共用占比的分母取比较集的块数 | 单测：候选 3 块、比较集 5 块且含这 3 块，占比应为 1，变异后 0.6 |
| V3 | 装置本地复算两态闭式时丢掉开头的 `1 +` | 锚 A1、A2（16842782、16777240 各少 1） |
| V4 | 最长段取第一段的写数 | 建快照流（第一段 2 写、最长段 24 写），锚 A5 的段形那一格 |
| V5 | R 的流从录制流第 0 步起切（不扣 mkfs 那几步） | 单测：一段 50 步历史，写数应等于整条写表减去 mkfs 那几步的写（mkfs 的写另按 `operations_written_by_make_filesystem` 数） |
| V6 | 第 i 段历史的种子取种子基 + i + 1 | 锚 A4「R 第一段历史的种子 = 种子基」 |
| V7 | 作废块数取 before 里不在 after 里的块 | 单测：before 4 块、after 6 块、共 3 块，作废应为 3，变异后 1 |
| V8 | 照抄来的 `crash_points_of_the_steps` 里 `.skip(1)` 改成 `.skip(0)` | 停机 T2 的逐字比对；单测：手造三段流，点的写区间应为 [0, a)、[a, b)、[b, 末) |
| V9 | `inside` 用开区间判 | 单测：一条流的两态闭式恰好等于 S_min，应算在里面 |
| V10 | 甲那一格的判段用 `R_total` 代 `R_judge` | 单测：合成 `R_total` = 1000、`R_judge` = 4000 态每秒，甲的格数值差 4 倍 |
| V11 | 挂钟档 2 小时的界写成 720 秒 | 单测：1 小时的格应落 ≤ 2 小时档 |
| V12 | J3-乙 的为正对数按 ≥ 0 数 | 单测：三对里一对作废 0，为正对数应为 2 |
| V13 | 转储里的基镜像漏掉 mkfs 的清零写 | 5.2「转储往返」：读回的基镜像摘要与直接算的不同 |

等价变异的处置照 `.claude/rules/mutation-sampling.md`：没红先换取样点，确认在所有输入上同值才写成等价性单测留档。

## 十、失败条款

这几条是正当结果（真实基线或被测条款的格），如实记下，不作废。

- **F1（甲分不了层）**：R、F 两族在写数 ≥ 1000 的流上 J1-乙 都判「重叠」，且两族 `above` 都 < 0.5。触发它的观测：两族上千写的流里，两态闭式落在冒烟层区间 S 里的各占一半以上。
- **F2（乙名不副实）**：R 族 J1-丙 判「换名」。触发它的观测：随机历史的块一半以上（按块的族中位）在 R4 的块键并集里。
- **F3（覆盖写链只是换名）**：F 族 J1-丙 判「换名」。触发它的观测：覆盖写链的块一半以上在 R4 的块键并集里。
- **F4（甲不能照字面每次提交跑）**：J2-乙 甲的写路径提交那一格落 > 24 小时。触发它的观测：N 的作废态数 ÷ 今天量到的 `R_judge` 加上 `keys_seconds` 之和超过 86400 秒。
- **F5（块级复用罩不住写路径提交）**：J3-乙 判「写路径提交大多作废过半」。触发它的观测：算得出的提交对里，一半以上的作废块占比 ≥ 0.5。
- **F6（条款这一格可能错）**：锚 B1–B4 有一条在装置与 crates 两边都核过之后仍不符；或 R 族写数 ≥ 1000 的流里两态闭式 ≤ S_max 的占一半以上（「写请求数上千」没有把负载选得比冒烟层大）。触发它的观测：B1 的多记录流全量态数 ≠ 12230590578；或上面那个占比 ≥ 0.5。
- **F7（固定开销分不出落法）**：J4-甲 判「分不出」。触发它的观测：5 轮里每一轮的 |D| ≤ 60 秒。

## 十一、作废条款与停机条款

### 作废（装置或测量有问题，那一格或整段不出结果）

- **X1**：5.2 任一阳性对照在任一条臂上没过（装置退 4）。那一段的 J1 产物整段作废，修装置后重跑。
- **X2**：R 的确定性：每档种子 0 与种子 1 各录两遍，写表摘要不同。R 那一族作废。
- **X3**：J2 任一项 `reused` ≠ 0（库不冷）、包含自检没过（`seconds` 包不住后三者之和）、中项的 `seconds` 不大于小项（5.4 阳性对照）、或小项 `states` ≠ 16842807。那一项（自检、单调那两种是整个 J2）作废，换新的空库目录重跑。
- **X4**：J3 的 `null` 副本有一条流作废块数 ≠ 0（锚 A9）。整个第二段作废：副本编出来的写路径与主工作区不是同一份。
- **X5**：J4 的已知开销对照（多睡 30 秒）测出的增量差不在 30 ± 5 秒。第四段作废。
- **X6**：读不到不等于读到 0：任何一行该有的计数行没打出来、或字段读不出，那一格写「取不出」、那一轮作废，不当 0 算（`.claude/singlefs-ai-sop/rules/test-discipline.md`「读不到 ≠ 读到 0」）。
- **X7**：内存包装退 245 或 250–254：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。

### 停机（两边都查，既不作废也不当结果；`.claude/rules/implementation-first.md` 第 4 条）

- **T0**：起跑前提（文件头）不成立。
- **T1**：文件头常量表的本地常量与 crates 的同名常量不等。
- **T2**：照抄的每一段（`crash_points_of_the_steps`；三个测试文件私有的 `prepare` 与它们调到的私有函数；别的照抄）在原文件里逐字找不到。做法：装置文件头列出每一段的出处（文件与起止行）与允许的替换（只许 `crate::` 换成 `singlefs_checker_tier::`、可见性词、函数名前缀），装置用 `include_str!` 把原文件编进来，`selfcheck` 把照抄的那一段做完允许的替换后在原文里逐字查一次，查不到就停。crates 那边改了切法或建流，装置照新原文重抄，写进修订。
- **T3**：R4 重录的写数、段形、两态闭式、全量、甲二与测试钉的常量（锚 A5、A6）不等。
- **T4**：同一条流上 crates 的两条路给的全量态数不等：`plan_crash_points` 的末节点序号尾（`points.last().ordinals.end`）与 `crash_state_count_with_torn_in_place_overwrites` 配 `full_expansion`。
- **T5**：J2 小项 `states=`（崩溃放量驱动那一路）与装置重录建快照流的全量态数（第一段）不等（锚 A7）。
- **T6**：开工复跑 `grep -rn "CrashFlow" crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-harness/src` 不再是零命中：crates 里已经有随机历史到 `CrashFlow` 的转换，装置要改用它，停下交主 agent 定。
- **T7**：随机历史的写流在切段时撞上 `writes_and_segments_with_stream_indexes_and_entries` 的卸载记号断言（C557（卸载记号只由卸载入口打没有检查））或别的 panic：R 那一族停，原样交回（种子、步数、panic 消息）。

### 够判停机

- G1 够判：J1-甲、J1-乙、J1-丙 在 R、F 两族的主取样与 8.2 的敏感性取样点上都有判定（「不稳定」也算有判定）。
- G2 够判：J2-乙 的表每一格都有档，J3-乙 算得出的对数 ≥ 10；J3-乙 算得出的不足 10 对时 G2 记「还差」，交主 agent 定，不在这一份里放宽到更早的提交。
- G3 够判：J4-甲 有判定，或用户已按「立阶段」的读法定了 G3。
- 岔路单每一行都够判就停；没跑的量逐条标「够判后未跑」并写明它对应哪一行（J1-库 是附带量，G1 够判后不跑）。

### 射程：这一份答不了的

- G4（多设备档位、「N≥5 轮」）不罩。
- J2 量的是崩溃放量入口那一路（录入段加 GPU 判段、双机）；常规层要是落成崩溃枚举用例行，提交时走的是 `crash_state_enumeration` 那一路，判器同一份、录入与派活不同，吞吐可能不同，没量。
- R4 之外的 25 条登记流与候选共用多少块没量（J1-库 只在「部分」时当旁证）。
- R 的块键取决于版本表的取法（1.1 节与 5.1 写死的 `committed_versions_at_the_end`）；常规层真要落成时版本表另有取法，块键可能不同。复用键的共用占比不含版本表，当它的上界另报。
- 没有一个状态被判：这一份只数态数、块与键，不说常规层会不会红。
- T1（常规层 挂钟预算）不给值；挂钟数只进产物与报告，不进 kb（D13 已定项 11）。

## 十二、修订

装置写完、单测与产物都还没跑（这一位执行员只写装置，不编不跑），2026-10-19 写。判据、门槛、作废与停机条款原样保留；下面几条要么收窄停机 T2 的对象（修订 1，依据派发提示），要么把装置的落点补记在案（修订 2–4），要么列出没做的（修订 5）。修订 2 里 T2 的缩进替换比登记写的允许替换多出一种，请主 agent 认。

### 修订 1：`crash_points_of_the_steps` 直接调，停机 T2 缩到三份测试文件的 `prepare`，变异 V8 删去

依据：派发提示「`crash_points_of_the_steps` 已由主 agent 改成 `pub`（`crates/singlefs-checker-tier/src/crash_state_enumeration.rs`），装置直接调它，不照抄；登记里停机 T2 那条逐字比对随之不用」。

- 1.1 节「节点与块」末句（`crash_points_of_the_steps` 是私有函数，装置照抄，停机 T2 钉住，不改它的可见性）作废；装置调 `singlefs_checker_tier::crash_state_enumeration::crash_points_of_the_steps`。
- 停机 T2 剩下的对象：新池新建文件流、覆盖写固定脚本流（到 E 再正常卸载那一支）、多记录发布流三份测试文件里私有的 `prepare` 与它们调到的私有函数；建快照流直接调公共模块的 `prepare_the_snapshot_crash_flow`，不抄。
- 变异 V8（照抄的 `crash_points_of_the_steps` 里 `.skip(1)` 改 `.skip(0)`）删去：它的对象（装置里那份照抄的切法）不存在了。

### 修订 2：停机 T2 的实现（照抄的段、允许的替换）

四段，起止行是这一位现读的（`selfcheck` 的 `copied_text_verbatim_*` 逐段对原文查，位置由装置里的 `COPIED-BEGIN` / `COPIED-END` 两行标记定）：

| 段 | 出处（`crates/singlefs-checker-tier/tests/` 下） | 内容 |
|---|---|---|
| fixed_script_head | `crash_enumeration_fixed_script_stream.rs:53-90` | 第二份内容常量、`Prepared`、`Script` |
| fixed_script_body | 同上 `:131-561` | 谱系常量到 `prepare` 末尾（`overwrite`、`unmount_recorded`、`writes_and_segments_after_mkfs` 在内） |
| multi_record_publish | `crash_enumeration_multi_record_publish_stream.rs:74-218` | `MultiRecordPublish` 到 `prepare` 末尾 |
| new_pool_prepare_body | `crash_enumeration_new_pool_file_creation_stream.rs:795-813` | `prepare` 的前半，到被判的根槽写那一步 |

- 登记写的允许替换（`crate::` 换成 `singlefs_checker_tier::`、可见性词、函数名前缀）这三种一种也没用上。用到的是另一种：三条流各包进一个 `mod`（它们的 `prepare`、`Prepared`、`second_content` 同名，不包进模块会撞名），照抄的行每行多 4 个空格；检查时每行去掉这 4 格再在原文里整段逐字找。模块带 `#[rustfmt::skip]`，rustfmt 不重排照抄的行。
- 新池新建文件流的 `prepare` 后半（`TransactionPublishInRecordedStream::find_in` 的自检）属于那份测试文件的观察者，装置不用，不抄；被判的版本照该测试文件 `single_version` 的做法（被判的那条根槽写出的那一代，内容是第一个文件）在装置里现取。
- 单测里有一条拿装置自己的源文本改一行抄来的内容，T2 的检查必须由绿转红。

### 修订 3：装置的落点补记（判据与门槛不动）

- `keys`：`--dump` 可重复，另有 `--out`、`--controls`。`name=flow` 行比 5.3 多 `label=` 与 `two_state_agrees_with_crates=`（本地复算的两态闭式与 crates 的 `closed_form_state_count` 是否相同）；块键文件每块一行比 5.3 多第五列点序号（J2-乙 丁那一格按点排要用）；整条流态数装不进 u64 的流不排计划，`blocks_*` 写 `not_planned`。
- 5.2 五个对照的落点：自比在 `share`（候选与比较集的每条流）；基镜像改一扇区、同流两种展开、名字不进键、转储字节往返在 `keys --controls`（种子下标 0 的随机历史、覆盖写链每一档、冒烟层每一条）；转储往返在手写流上由 `record-fixed-chains` 做（用建好的池直接当基镜像排的块键，对写转储、读回、由转储重建基镜像排的块键）；在随机历史上由转储 bin 对每条新录的转储核「重建的基镜像 = 直接把录制流 mkfs 那几步叠到空盘上的基镜像」（`name=dump` 行 `base_matches_the_stream=`，收尾行 `base_mismatches=`）。任一没过退 4（X1）。
- 锚 A3 的「合成流」改成：纯函数 `blocks_needed_for` 在 65536、65537 上的单测，加每条冒烟层流的每个真实计划点的块数都等于它（`A3_blocks_per_point_*`）。
- `share`：`--candidates`、`--against` 可重复，另有 `--pinned-smoke <文件>`（一行 `<流名> <两态闭式> <最长段>`，给区间 S 里测试文件钉了常量的那几条流；执行员 grep 取常量、照闭式现算核过再写，取不到的不写、在产物里列「没钉」）。候选只收 `candidate` 层，比较集只收 `baseline` 层。
- 判定词的英文记号：重叠 overlap、分开 separate、换名 renamed、独立 independent、部分 partial、还差 still_missing、写路径提交大多作废过半 more_than_half_invalidated、块级复用省下大半 mostly_reused。门槛是第六节的原值（`INSIDE_FRACTION_AT_LEAST_FOR_OVERLAP` 等常量，装置里逐个写出处）。
- 8.2 的敏感性：每个取样点各自是一个范围（`scope=group:<组>`，组 = 流名去掉末尾 `-seedNN`），另有两个主取样的合并范围（`family:random_history_main` = BROAD、FourGibibytes 各步数档；`family:overwrite_chain_main` = u=4）；同一门槛在各范围上各判一次。`name=overlap` 另报写数不少于 1000 的流里不大于 S_max 的占比（F6 的第二条触发观测）。
- `compare`：另有 `--kind <commit-pair|patch>`、`--pair <名>`、`--judging-unit-key-changed`、`--expect-first-difference <journal-record|none>`（A9、A10、A11 的断言：`none` = 两份写流逐字相同且一块不作废；`journal-record` = 基镜像就不同，或第一处不同的写就是写表里第一条 journal 记录写）。A11 的独立一路在转储 bin：`name=dump` 行报 `compaction_operations=`（生成的历史操作表里挂着时整理的次数）。
- `compose`：J2 的输入是两份驱动日志原样（`--j2-small`、`--j2-medium`，要有一行 `CRASH_AMPLIFICATION_ITEM` 与至少一行 `CRASH_AMPLIFICATION_GPU_TOTAL`，两台各打一行时判段各取最大）、`du -sb` 的输出（`--library-bytes`）、读库程序的输出（`--recorded-blocks`）；另有 `--j1`（`keys` 的输出）、`--keys`（块键目录）、`--j3`（`compare` 的输出）、`--final-pair`（主工作区对最近一次提交那一对）。X3 与 T5 在 `compose` 里判；A7、A8 报 `name=anchor`，A8 对不上不作废。
- `selfcheck`：B1 对不上是第四节预计的，不退 3；A 开头的锚点对不上、任何一条停机检查没过退 3。
- 规模：最大的流有上千万块（400 步档两态闭式的 log2 到 40 附近，每块 65536 态）：块键文件边算边写，`share`、`library`、`compare` 一条流一条流地读，不把全部候选放进内存。`keys` 的线程数仍取 `E184_THREADS`，400 步档按内存上限酌减。

### 修订 4：变异表（第九节）

V8 删（修订 1）；V1–V7、V9–V13 共 12 条，写成 `crates/mutations.tsv` 的追加行（草稿在草稿目录 `mutation-rows-draft.tsv`，这一位没追加）。V5、V6、V13 改转储 bin，其余改主装置；每条的「必须红的测试名」是它所在 bin 里的一条单测。每条替换后的源文这一位都对着主工作区的 rlib 做过类型检查（零错）；一条也没跑过，三个数待提交时门禁 checker-tier-crates-mutation-replay。

### 修订 5：没做的

- 单测与变异都没跑（装置还没编）；`replay.sh` 的 `driver_e184` 登记行、实验页、索引行、`research/results/` 的产物都没写，留给编过之后续派的执行员。
- 停机 T6（`grep -rn "CrashFlow" crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-harness/src` 零命中）是开工的一条 shell 命令，装置里没有它。
- 第二段（J3-甲、J3-乙 的副本与提交清单）、第三段（J2 的崩溃放量）、第四段（J4）的外层脚本与命令这一位没写，只写了它们要用的 `compare`、`compose` 与转储 bin。

### 修订 6：覆盖写链 u = 4、16 两档改走顺序写（crates 里没有单事务的多数据单元覆盖写）

时点：装置单测已跑（主 agent 跑，70 条里 68 绿 2 红），产物一次没跑；2026-10-19 写。依据两处：单测日志 `unit-e184_regular_layer_cost_counts.log`（`~/.local/share/singlefs/work/m3-close-w4-e184-runner-b/` 下）里两条红，`panicked … 覆盖写: ContentExceedsDataUnit { bytes: 505827, capacity: 32634 }`（`an_overwrite_chain_runs_until_a_thousand_writes_and_keeps_one_version_per_overwrite`，u = 16）与 `chain_failed label=overwrite_chain-u04 panic=覆盖写: ContentExceedsDataUnit { bytes: 114219, capacity: 32634 }`、`label=overwrite_chain-u16 …`（`the_record_fixed_chains_command_writes_seven_dumps_and_skips_them_the_second_time`，u = 1 录成、`chains_done recorded=5 skipped=0 failed=2`）；和派发提示那一句「登记要的是 crates 今天就有的入口，那就改装置走那个入口；crates 今天没有多单元覆盖写，那就照登记修订的规矩写明」。

- 登记原文（臂 F）：「经 `overwrite_step`（`common_crash_points/mod.rs:419`，录崩溃点的那一路）发布 … u 取 1、4、16 三档（每次覆盖写的数据单元数）」。
- crates 的实际：`overwrite_step` 调 `publish_overwrite`（`crates/singlefs-core/src/transaction.rs:7477`），它的文档注释写「契约是一个数据单元：多个数据单元的内容走 `publish_sequential_write`」（`:7471`），内容装不进一个数据单元（32634 字节）就报 `ContentExceedsDataUnit`（`:7474`、`:8998` 起的 `refuse_a_file_that_does_not_fit_one_data_unit`，`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:932` 的用例钉了 32635 字节被拒）。会话层同样分两种改动：`UserChange::Overwrite`「覆盖写一个数据单元以内的内容」与 `UserChange::SequentialWrite`「顺序写整份内容，按切分纪律切成几个一单元事务」（`crates/singlefs-core/src/mounted_session.rs:100`–`:103`）。`content_needing(4, k)` 是 114219 字节、`content_needing(16, k)` 是 505827 字节，都超过一个数据单元：登记写的「经 `overwrite_step` 发布 u = 4、16」在 crates 里做不成。
- 零命中：`grep -rn 'fn [a-z_]*overwrit[a-z_]*' crates/singlefs-core/src` 只命中 `transaction.rs:7477` 的 `publish_overwrite` 与 `mount.rs:11177` 里一个测试辅助 `overwrite`；crates 里没有「按单元覆盖写多单元文件」的另一个入口，也没有单事务的多单元覆盖写。
- 多单元的整份内容替换（从偏移 0 写整份新内容，上一版比这一版多出来的单元同次发布释放）在 crates 里只有 `publish_sequential_write`（`transaction.rs:7533`；它的文档注释写「只要一个数据单元的内容，写出的字节与同内容的 `publish_overwrite` 一个不差」`:7523`，「不报 `ContentExceedsDataUnit`：多个数据单元正是这条路径要写的」`:7531`），录崩溃点的那一路用 `sequential_write_step`（`common_crash_points/mod.rs:75`）调它的同一份造法 `sequential_write`（`:53`）。
- 改法：u = 1 照登记走 `publish_overwrite`（不变）；u = 4、16 改走 `common_crash_points::sequential_write`。内容仍取 `content_needing(u, k)`，仍对同一个文件、同一个实例（1）、同一个写时间，仍写到写数 ≥ 1000，覆盖写次数 K 仍由装置数出来写进产物，三档与取样点（主取样 u = 4，方向相反的 u = 1 与 u = 16）不动，版本表仍是每次覆盖写一版。变的只有发布单位：u ≥ 2 的每次覆盖写是一次发布里的 u 个一单元事务、u 条 journal 记录（`publish_sequential_write` 的切分纪律），不是一个事务装 u 个单元（crates 里没有那一形）。这一条不是收严，是把 crates 里造不出的形换成今天有的入口，判据、门槛、臂与取样点一概不动；请主 agent 认。
- 不认的退路（先写在案，不预设）：u = 4、16 记成「造不出」（`name=chain_failed` 行原样进产物，附上面的 `ContentExceedsDataUnit`），臂 F 只剩 u = 1 一档；那样 8.2 里 F 族的主取样（u = 4）没有判定，G1 的 J1-乙、J1-丙（F）记「还差」，不能按「稳不稳定」判。
- 验收：`an_overwrite_chain_runs_until_a_thousand_writes_and_keeps_one_version_per_overwrite` 与 `the_record_fixed_chains_command_writes_seven_dumps_and_skips_them_the_second_time` 的原断言一条没改；前一条加了两条更严的断言（u = 16 的每一版超过一个数据单元；最后一版的内容就是第 K 次的 `content_needing(16, K)`）。变异 V14 钉这个分流。

### 修订 7：停机 T2 的缩进替换，主 agent 认的条件与核对

主 agent 认修订 2 里的那一种替换（三条流各包进一个 `mod`，照抄的行每行多 4 个空格），条件是：照抄的核对在去掉每行行首恰好 4 个空格之后与原文逐字相同。核对按这个条件另写了一份独立的 python（和 `selfcheck` 的 `copied_text_verbatim` 同一个条件，但按出处写的原文件起止行逐行比，不是在整份原文里找子串）：

```python
import hashlib, re, sys
source, root = sys.argv[1], sys.argv[2]
lines = open(source, encoding='utf-8').read().split('\n')
segments, current = [], None
for number, line in enumerate(lines, 1):
    stripped = line.lstrip()
    match = re.match(r'// COPIED-BEGIN (\S+) (\S+):(\d+)-(\d+)$', stripped)
    if match:
        assert current is None
        current = [match.group(1), match.group(2), int(match.group(3)), int(match.group(4)), [], number]
        continue
    match = re.match(r'// COPIED-END (\S+)$', stripped)
    if match:
        assert current is not None and current[0] == match.group(1)
        segments.append(current)
        current = None
        continue
    if current is not None:
        current[4].append(line)
assert current is None
failed = 0
for name, origin, first, last, body, begin_line in segments:
    original = open(f'{root}/{origin}', encoding='utf-8').read().split('\n')[first - 1:last]
    undented = []
    for line in body:
        if line == '':
            undented.append('')
        else:
            assert line.startswith('    '), (name, line)
            undented.append(line[4:])
    same = undented == original
    digest = hashlib.sha256('\n'.join(undented).encode()).hexdigest()[:16]
    original_digest = hashlib.sha256('\n'.join(original).encode()).hexdigest()[:16]
    print(f'{name} origin={origin}:{first}-{last} lines={len(undented)} original_lines={len(original)} verbatim={same} sha256_undented={digest} sha256_original={original_digest}')
    failed += 0 if same else 1
print(f'segments={len(segments)} not_verbatim={failed}')
sys.exit(1 if failed or len(segments) != 4 else 0)
```

输出（对修订后的装置、对写这一条时的 `crates/singlefs-checker-tier/tests/` 原文）见执行员报告；四段都要 `verbatim=True`、`not_verbatim=0` 才算过，任何一段对不上，T2 不过。

### 修订 8：两份 bin 补文件头的准入与运行条件，`main` 第一句调 `preflight`

依据：派发提示「文件头缺准入与运行条件、`main` 第一句不是 `preflight`：两份 bin 都照 `e161_crash_state_dedup_and_time_split.rs` 的写法补，依据 `preflight-discipline.md`「Rust 与别的语言」一行」。

- 两份文件头各写 `// admission: always <理由>` 与 `// run-condition: command python3`。准入写 `always`，不写 `inputs-changed`：两份 bin 每跑一次都在今天这份 crates 上重录、重排计划、重取块键，挂钟是新观测，登记的结果取决于的输入（代码、决策、登记）变了没有，不决定这一次跑有没有意义；已有写完的转储与块键文件由装置自己整份跳过（续跑）。
- `preflight` 只有一份实现，在转储 bin 里（`pub fn preflight(source_file_name)`）：直接起 `python3 <规范副本>/scripts/preflight.py check <源文件的绝对路径> [--force] -- <参数…>`（不经 `sh -c`），退出码 78 退 78、别的非 0 值退 1，stdout 第一行以 `forced` 起头时把最后一段摘要（空白换成下划线）交回；主装置经 `#[path]` 引转储 bin 当模块，`main` 第一句调 `write_dump::preflight(OWN_SOURCE_FILE_NAME)`。`--force` 在交给各子命令的参数解析之前摘掉。强制跑的那一次，标准输出第一行多一行 `E7RESULT name=preflight forced=<摘要>`，各收尾行的 `emitted=` 不数它；没强制跑的产物里没有这一行。
- 条件满足时 stdout 是 `met\t-`（`always` 不带指纹）：`preflight.py check` 对草稿目录里改完的两份源文件实跑过，都是 `met\t-`、退出码 0。
- 新增的纯函数与单测：`forced_summary_of_the_check_output`、`exit_code_after_a_failed_check`、`arguments_without_the_force_flag`，和两条「文件名指向的文件存在、文件头有两类声明」的单测。第二段在旧提交副本里编转储 bin 时，副本要带上 `.claude/singlefs-ai-sop/scripts/preflight.py`，否则 `preflight` 起不来（退 1）；缺了按「取不出」记，不写 0。

### 修订 9：变异表增补与一处 clippy 修正

- 变异表在修订 4 的 12 条之外增补 V14–V19 共 6 条（每处改法留一条）：V14 钉覆盖写链的分流（修订 6）；V15、V16、V17 钉 `preflight` 的退出码、摘要段与 `--force` 摘除；V18、V19 钉两份 bin 交给 `preflight` 的文件名。共 18 行，草稿 `mutation-rows-draft.tsv`，三个数仍待提交时门禁 checker-tier-crates-mutation-replay。每条替换后的源文这一位都对着主工作区的 rlib 做过类型检查（零错）。
- 主装置里三处只被单测用到的定义（`keys_file_text`、`KeysFile::from_text`、`OVERWRITE_AND_UNLINK_TEST_TEXT`）加了 `#[cfg(test)]`：不带 `--test` 的 clippy（`check.sh` 的 `--all-targets` 会编这一个目标）此前报 3 处 `dead_code`，修订 4 的「类型检查零错」只对着 `--test` 目标；改后两个目标都零错零警告。

### 修订 10：排计划前按块数上界拦（覆盖写链 u = 16 的全量要 2^44 字节的块表）

时点：装置单测已跑（主 agent 跑，76 条里 75 绿，`the_record_fixed_chains_command_writes_seven_dumps_and_skips_them_the_second_time` 没报结果、进程 SIGABRT），产物一次没跑；2026-10-19 写。依据：单测日志 `unit.log`（`~/.local/share/singlefs/work/m3-close-w4-e184-runner-c/` 下）第 81 行 `memory allocation of 17592186044416 bytes failed`（2^44 字节），和派发提示「找出哪一处要 16 TiB …补一条单测把这一处钉住，变异一条」「若查出它本来就要几十 GiB，写明量级（推的），并把单测缩到登记允许的小几何。缩了要写进登记修订」。

- 定位（读代码推的，没编译复现）：上一轮单测日志（`…/m3-close-w4-e184-runner-b/unit-e184_regular_layer_cost_counts.log` 第 88–104 行）里，覆盖写链 u = 1 与冒烟层四条都 `chain_recorded`，每条都过了 `direct_base_control`（用建好的池和由转储重建的池各排一遍全量计划，块键逐个比）；u = 4、16 当时死在 `ContentExceedsDataUnit`（修订 6）。所以这一次新进来的只有 u = 4、16 经 `publish_sequential_write` 造出的两条链，它们在 `direct_base_control` 的 `plan_blocks(…, Expansion::Full, …)` 里撞：`plan_blocks` 对每个点调 `PlannedCrashPoint::blocks`（`crates/singlefs-checker-tier/src/crash_amplification.rs:319`），它按 `STATES_PER_BLOCK`（`:175`，2^16）把点的态数切块、一次摆进一个 `Vec<Range<u64>>`（每块 16 字节），块数没有上限。
- 量级（推的）：2^44 字节 = 16 字节 × 2^40 块，2^40 块 × 2^16 态 = 2^56 态，也就是一个 56 写、全是两态单元写的段（`1 + Σ(2^|段| − 1)` 里 2^56 − 1 态）。多记录流测试文件 `crates/singlefs-checker-tier/tests/crash_enumeration_multi_record_publish_stream.rs:178`–`:180` 钉的单元段写数是 A（1 个单元）24、B（2 个）28、C（3 个）30；按「24 加每多一个单元约 2 写」粗外推，u = 16 的单元段约 56 写，u = 4 约 32 写。u = 16 一次覆盖写就是 2^56 态、2^40 块（一万亿块，块键文件写不下，`BlockOfAPlan` 每块 88 字节也放不进内存）；u = 4 一次覆盖写 2^32 态，几十次覆盖写合计按 10^7 块的量级估（推的，没量），在上限之内。单测本来就要的内存不是几十 GiB，是一个不该摆进内存的块表；单测不缩几何（覆盖写链的写数 1000、u 取 1、4、16、K 由装置数，一概不动）。
- 改法（装置，判据与取样点不动）：排计划之前先算块数上界 = 总态数除每块态数向上取整，加（段数 + 2）（每个点最多多一块的零头；点数不超过段数加末节点）。上界超过 2^26 块（`MOST_BLOCKS_PLANNED_PER_FLOW`）的展开不排计划、不取块键：`name=flow` 照报 `full=`、`quick_tier=`，`blocks_full=not_planned`（和 1.1 节「装不进 u64」的流同一种待遇；快档的块数少，照排）；`plan_blocks` 在这种流上返回 `Err` 而不是 abort。转储往返对照（`direct_base_control`）要同时摆两份计划，比块键的上限再小一档，2^24 块（`MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP`），超过的流只比基镜像（它本来对「装不进 u64」的流就是这么做的），`keys_equal=not_planned`。两个上限是工程上限，不是判据；2^26 块约占 7 GB（每块 `BlockOfAPlan` 88 字节加 `blocks()` 那份 16 字节，共 104 字节）。
- 这一条不是收严，是记明装置量不到的格子：一条块数上界超过 2^26 的全量计划，今天这台机器上排不了。后果请主 agent 认：① 没排计划的流不进 J1-丙 的族中位与族最大（`name=share_not_planned … reason=no_blocks_planned`，`name=share_group` 的 `planned_flows=` 数得出来），J1-乙 照用 `full=` 的态数判（态数照报，不受影响）；覆盖写链 u = 16 的 J1-丙 因此记「取不出」，不写 0，也不写「独立」。② 5.2 里要全量计划的三条对照（改一扇区、名字不进键、两种展开）对没排计划的流不跑（`controls_of_the_flow` 里 `reading.full` 为 `None` 就不跑，与「装不进 u64」的流相同）；转储往返的内容比对（`dump_round_trip`）照跑，每个臂每一档仍有它。③ u = 4 若也超过 2^26，臂 F 的主取样没有 J1-丙，按修订 6 里「不认的退路」同样的写法记「还差」；到时由产物里 u = 4 那一行的 `blocks_full=` 决定，不预设。可补的量（不在这一条里做，主 agent 定）：对没排计划的流，用「点的复用键与计划哈希是否出现在冒烟层各点里」给共用块数一个上界，不物化块。
- 验收：`the_record_fixed_chains_command_writes_seven_dumps_and_skips_them_the_second_time` 的断言一条没改；新增 6 条单测——`the_block_count_bound_adds_the_points_left_over_to_the_blocks_of_the_states`（上界的算术与两个常量）、`a_flow_whose_blocks_would_not_fit_in_memory_is_counted_but_not_planned`（56 写段的合成流：态数 2^56 + 1、不排计划、`plan_blocks` 在排计划之前返回 `Err`）、`the_two_limits_split_flows_by_the_blocks_of_their_states`（40 写、39 写两档与上界等于上限的边界）、`a_small_limit_refuses_the_full_plan_of_a_real_flow_and_leaves_the_quick_tier_plan`（新池新建文件流，小上限下全量 `not_planned`、快档照排）、`the_round_trip_control_compares_the_keys_only_when_both_plans_fit_under_its_limit`、`the_sixteen_unit_overwrite_chain_is_read_without_its_full_plan_and_with_its_quick_tier_plan`（真的 u = 16 链）。变异增补 V20–V27 共 8 条，草稿共 26 行，三个数仍待提交时门禁 checker-tier-crates-mutation-replay。没留变异的一处：`direct_base_control` 把两个上限里的哪一个交给 `_under`（要用真流凑到 2^24 与 2^26 之间的块数才分得出，单测里造不出）。
- 能推翻上面定位的现象：改后重跑单测，`the_record_fixed_chains_command_…` 仍以 `memory allocation … failed` 退出（那就不是块表，要去看 `plan_crash_points` 的别处）；或 `the_sixteen_unit_overwrite_chain_is_read_without_its_full_plan_and_with_its_quick_tier_plan` 的 `longest_segment >= 40` 断言红（u = 16 的单元段没有那么长，量级推错）。

## 十三、读过的文件与跑过的命令

### 13.1 规则、共用约束、记录、问题单与背景

- `.claude/agent-common.md` 1–106；`.claude/singlefs-ai-sop/rules/test-discipline.md` 1–122；`.claude/rules/three-way-inference.md` 1–166；`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 1–179；`.claude/rules/mutation-sampling.md` 1–93；`.claude/rules/implementation-first.md` 1–19；`.claude/rules/test-script-discipline.md` 1–106；`.claude/singlefs-ai-sop/rules/command-safety.md` 1–162；`.claude/singlefs-ai-sop/rules/writing-discipline.md` 1–133；`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 1–53。
- `research/prompts/m3-close-c409-cost-questions.md` 1–12；`research/prompts/m3-close-c409-r1-forks.md` 1–12；`research/prompts/_m3-close-c409-r1-body.md` 全文；`research/prompts/m3-close-c409-r1-main-verification.md` 全文；`research/prompts/m3-close-c409-r1-sonnet-output.md` 1–331；`research/prompts/m3-close-c409-r1-opus-output.md` 1–263；`records/2026-10-18-里程碑三收尾与近期欠账.md` 1–120。
- `research/prompts/e183-preregistration.md` 节标题（grep）与 1–30（只看登记的写法，没读它的结果）。
- `.claude/agents/experiment-runner.md`（grep `crates` 命中行 34、36、49）；`.claude/hooks/agent-write-scope.tsv`（grep 命中行 13–25）；`.claude/hooks/run-scope-guard.sh` 1–80 里 grep 命中行；`.claude/hooks/lib_long_run.py` 10–21、88–89；`.claude/heavy-thread-default` 全文；`scripts/kb/claim-experiment.sh` 1–17；`scripts/hosts/compile-then-swap.py` 1–40 里 grep 命中行；`scripts/experiment-apparatus/replay.sh` 71–72、1454；`crates/mutations.tsv` 第 1 行。

### 13.2 kb

- `.claude/kb/decisions/13-验证路线.md` 134–149、240–338、395–444（与 `quote-kb.py` 抄进第二节的八段同）；`.claude/kb/checks-owed.md` 第 294 行（C409，grep 命中）。

### 13.3 crates 与门禁、脚本

- `crates/singlefs-checker-tier/src/crash_identity.rs` 1–280、619（grep）；`crates/singlefs-checker-tier/src/crash_amplification.rs` 175、200–640、921–946、1061–1140、2439–3969 里 `seconds=` 的 grep 命中行、12173、12206、12470–12560、12936；`crates/singlefs-checker-tier/src/verdict_store.rs` 355–400、2071、2077、3255–3330；`crates/singlefs-checker-tier/src/crash_state_enumeration.rs` 1–200、716–1140；`crates/singlefs-checker-tier/src/crash.rs` 1349、2410–2475；`crates/singlefs-checker-tier/src/crash_library.rs` 120、327、593（grep）；`crates/singlefs-checker-tier/src/crash_injection.rs` 1480–1575、1980–2008 与 grep 命中行；`crates/singlefs-checker-tier/src/bin/crash_library_reader.rs` 1–60；`crates/singlefs-checker-tier/src/bin/e161_crash_state_dedup_and_time_split.rs` 1–40；`e170_o3_spec_executor_export.rs`、`e171_three_oracle_collusion*.rs` 的 `#[path]` grep 命中行；`crates/singlefs-checker-tier/Cargo.toml` 全文。
- `crates/singlefs-harness/src/memory_pool.rs` 520–684、843、853；`crates/singlefs-harness/src/history.rs` 120–160、561–567、700–760、794–858（grep）、915–980、1191、1402–1411、1527、1852–1860、3525–3540、7113–7125、7480–7560。
- `crates/singlefs-core/src/journal.rs` 24、112、127、200–265、296、306、340–360；`crates/singlefs-core/src/logged_operations.rs` 29、245、254、340（grep）；`crates/singlefs-core/src/deleted_inodes.rs` 1–60；`crates/singlefs-core/src/system_configuration.rs`、`instance_table.rs`、`pointer.rs`、`extent_tree.rs`、`make_filesystem.rs`、`bytes.rs`、`admission.rs`、`inode_tree.rs`、`unit.rs`、`recovery.rs` 的 `skip(` / 预留 / MAC grep 命中行；`crates/singlefs-format/src/lib.rs` 的 `format-const` grep 命中行；`crates/singlefs-checker/src/image.rs` 68、422、1505–1534（grep）。
- `crates/singlefs-checker-tier/tests/crash_enumeration_snapshot_creation_stream.rs` 1–120、180、283；`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs` 1–60 与 grep 命中行（44、53、75、86、115、139–848）；`crates/singlefs-harness/tests/common/mod.rs` 56、118、736、747；`crates/singlefs-harness/tests/common_snapshot_prefix/mod.rs` 475；`crash_enumeration_new_pool_file_creation_stream.rs` 762–792、794、920；`crash_enumeration_fixed_script_stream.rs` 99–166、255、1363；`crash_enumeration_multi_record_publish_stream.rs` 22、140、380；各 `crash_enumeration_*.rs` 与 `crash_points_of_the_instance_table_sweep.rs`、`record_checker_judges_absence_by_the_persisted_set.rs` 的 `prepare` grep 命中行。
- `.claude/gate.d/stage-inputs.tsv` 46–74；`.claude/gate.d/checker-tier-crash-point-replay.sh` 21、899（grep）；`scripts/crash/crash-amplification.sh` 1–120；`scripts/crash/lib_crash_point_replay_items.py` 115；`scripts/gate/admission.py` 的 `BLOCKS_QUERY` grep 命中行。

### 13.4 跑过的命令（原样）

1. `bash scripts/hosts/work-directory-ownership.sh claim ~/.local/share/singlefs/work/m3-close-w4-c409-cost-design --purpose '里程碑三收尾批 4：C409 常规层与大负载层的代价问题单 J1–J4 跑前登记'`
2. `bash scripts/crash/crash-amplification.sh --list-items`（两次；输出 20 行）
3. `python3 -c "s=[2,2,1,2,2,1,2,24,2,1,2,16,2,1,2]; print('snapshot_creation', len(s), sum(s), 1+sum(2**k-1 for k in s))"` 输出 `snapshot_creation 15 62 16842782`；`python3 -c "…p=s[:11]…"` 输出 `prefix11 [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2] 41 16777240`、`blocks_of_65537 2 blocks_of_65536 1`、`log2_16842782 24.0056`。
4. `bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts`（输出里有私有主机值，不抄；读到「双机、开 GPU」）
5. `python3 scripts/agents/dispatch-prompt.py experiment-runner`（只看「多机：」「分片：」「续跑：」三行的写法）
6. `bash scripts/kb/claim-experiment.sh --next`（输出 `E184`）；`bash scripts/kb/claim-experiment.sh E184 常规层与大负载层的代价计数`
7. `python3 scripts/kb/quote-kb.py <草稿目录>/quoted-clauses.md` 加八个取法（第二节；输出「8 段整抄进……回读逐字节一致」）
8. `git log --format='%h %ad %s' --date=short -n 14 -- crates/`；`git log --format='%h' -- crates/ | wc -l`（36）；`git log --format='%H %h' -n 13 -- crates/`
9. `sha256sum research/prompts/m3-close-c409-cost-questions.md research/prompts/m3-close-c409-r1-forks.md`
10. 读文件与定位用的 `grep -n`、`sed -n`、`wc -l`（命中行列在 13.1–13.3）；`grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv`（29）；`grep -n 'len(fields) != 4' scripts/crash/lib_crash_point_replay_items.py`（第 115 行）。
