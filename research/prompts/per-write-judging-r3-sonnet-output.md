# 逐写独立判 第三轮 辩方腿报告（Sonnet）

批次：records/2026-11-05-删轻型并入重型.md#并包。立场：辩方，替「逐写独立判」辩护到底；复核第二轮判决 `research/prompts/per-write-judging-r2-main-verification.md`（连同第一轮 `research/prompts/per-write-judging-r1-main-verification.md`）。
没读禁读清单里的任何一份（别的腿这一轮的报告）。读过的：背景材料正文与附录（附录整份是 kb E1 到 E6 实验页的抄录，与这一轮的问题无关）、两轮判决、第二轮辩方两份报告与模型、第二轮攻方两份报告里被引的几处（464438 那一行、乙·增减那一行、攻方原型的 `delta_of`）。
标注：「量」= 产物文件里的一行或我用 `awk`、`grep` 对产物数出来的；「读」= 读源码；「算」= 对量到的数做算术；「推」= 没有现场读数，等主 agent 跑。引文都是 `grep -nF` 现查的行号。
草稿目录 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r3-defense`（认领过；里面只有桩自检的源码，编出来的二进制与库已删，没建仓副本、没建 cargo 编译目录）。模型目录 `research/prompts/per-write-judging-r3-sonnet-model/`。
没起 cargo、没往任务服务交任务；类型对不对的把握来自一份桩自检（第七节末尾写了它是什么、没证明什么）。

## 〇、结论（先给）

1. **16 段的 E6 读数 2，推的结论是 16 段全是宽度误报，没有一段是真点名**（「推的，待主 agent 跑」，探针命令在第七节）。依据是两条互不共用的路：
   ① 产物那一路（量）：24 个真实单元段里 `e2_over_an_earlier_write`、`e2_over_nonzero_base` 全是 0，读数 2 的 16 段每段恰好是一条记录的两份副本；
   ② 代码那一路（读）：记录点名的位置条目就是那个单元自己的落点，类标签 2 的节点只占 16384，而 r2 的 E6 对每个点名项一律按 32768 算。前缀里的点名单元在写表更早处写过、E2 又说这一段的写不盖任何更早的写，所以与这一段相交的只可能是被多算的那 16384。
   两条路共用的前提只有「记录点名的位置条目等于它点名的那个单元的写落点」（第二节第 1 步，读 `transaction.rs`），探针逐条核它（`prefix_matched_class_and_birth`）。
2. **收窄后的 E6（我叫 E6n）按点名项自己的类取宽度**（类 1、3 是 32768，类 2 是 16384，与恢复、池级 checker 读点名单元用的宽度同一份），点名类不认识或记录解不出就整段退回。对 r2 的 E6，**这是字面上的放宽**（类 2 的范围缩了一半），不是「只收严」：按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」要先记一次输（r2 的 E6 在 24 段里退回了 16 段），E6n 当成新臂、跑前写预测（第六节），不当成「澄清」。它在反例 B 的几条臂上要拒（预测见第六节）。
3. **态数**（算的，前提是 E6n 在 24 个真实单元段上全过，探针才能把它升成量）：真实删除流 1,463,812,286 → 471；对拍三条流 1,178,599,580 → 355、1,107,689,646 → 378、1,195,376,807 → 404。四个数与 r2 报告里「全过」的数逐位相同（第三节表）。
4. **逐态全量核乙**：命令在第四节。三件要紧的事：（a）真实流上的段全是「全绿对全绿」的可能性很大，比较没有牙，所以给了造坏（`PROBE_EXHAUST_FAULT`）与阳性对照；（b）24 个段合计 4,945,477,608 个状态（算的），按第二轮读到的下界速度（≥1.6×10³ 状态/秒，量的下界）单进程要 35 天以上，所以分层：≤22 写的 6 个段一次交完，24 写以上要切片，26 写以上要么上几十个片要么另走 GPU（GPU 一路我没读够，不写命令）；（c）另给「按阶全量」（至多 k 个组里有写落下的状态一个不漏），它不是逐态全量，只覆盖低阶相互作用，写明它不证明什么。
5. **对第二轮判决**：判决表 10 行没有一行要撤销（第五节逐行）。行 2、6、9 原样站得住；行 1、3 站得住但要加限定（只打 E6 的字面写法；那 3072 个状态可能是全绿对全绿）；行 4、5、7、8 的数站得住、够不着判据内（它们打的是 E4、E2、E1 本来就拒的形状）；行 10 数对、说法把小段全量与取样并成了一句。
   **「这是第三轮唯一要定的事」不对**（第 5.2 节列了九条，最要紧的三条）：E5（读迹）没有实现，所以 471 那一族的数在 E5 上没有读数，那一份报告自己写了不能据此降态；E3「落在单元区」那半句没量；真实流上的单元段若全绿，「乙与全组合等价」的任何数都没有牙，只有造坏与阳性对照能给牙。另有一处结论接错了射程：第二轮结论第 2 条把「两组同时被盖」（91 对里 78 对）接到「判据放宽」上，这 91 对是挪到更早写的偏移上造出来的，E2 与 E6n 都拒，它们与 E6 放宽与否无关（第 5.2 节第二条）。

## 一、模型、复跑命令与 sha256

模型目录 `research/prompts/per-write-judging-r3-sonnet-model/`。探针没编过、没跑过。

| 文件 | sha256 |
|---|---|
| `probe/named_slots_classifier.rs.in`（逐段读 E6 点名的槽；旧宽读法与 E6n 并排；581 行） | `37d41f1d0faa69aab9e50c8a87e948d85ec4cbe59493e025ed705628e52f947e` |
| `probe/exhaustive.rs.in`（逐态全量比较器、按阶全量、造坏、切片） | `90b1d402d9f5925588a6575fb59513074e29d8c633112989e523878ec51fbbac` |
| `probe/named_slots_unlink.rs.in`（删除流上的普查、臂、逐态全量、阳性对照三条测试） | `c13f3bbac68c406292f261b90d9cb3cbdbdf018d5bfbcbc0207c05f4df0eadc8` |
| `probe/named_slots_pairing.rs.in`（对拍三条流上的普查、逐态全量两条测试） | `7f3fbaa3683c007dab189c74aa1a4fa2506a395d75392d96d340dd11aa66810e` |
| `tools/rerun.sh`（拷自 `scripts/agents/three-way-rerun-template.sh`，只改 `rerun_steps` 那一段） | `8809326ff2eb3ae2ce499aa57a34478dfd8c79aa3026953c78badb1514388d26` |
| `tools/stub-selfcheck/stub_verification.rs`、`selfcheck_tests.rs.in`、`build.sh`（桩自检，第七节末尾） | `c346c5abfa4057c59087bad2c5ca1f34222b0c03f8958682dbb4a08c910e37a2`、`71d7d8c3a85a7705af126430bc8eb884f96d0e2c346753962b77486c20258020`、`02062e2b9b6b3d0d8b0d814ea741313bc52599366e02fc7c68dd1c6e316db870` |

`tools/rerun.sh` 与第二轮那份的两处不同（都在 `rerun_steps` 里）：
- **能整条重起接着跑**：被服务撞内存上限重排时，同一条命令原样重起不会被「--work 已存在且非空」挡住。做法：状态（仓副本、各步输出、完成记号）放在 `<work>.state`，`<work>` 本身到最后一步才写；已完成的步按 `<work>.state/done/<步名>` 跳过。
- **汇总不漏行**：每个测试第一行先打一个空行，使每条 `PROBE_` 行都在行首；汇总用 `grep -ho 'PROBE_.*$'`（从第一个 `PROBE_` 取到行尾，不要求在行首），并另数 `grep -c '^PROBE_'` 报出来（`probe_lines_at_line_start`），与 `grep -c 'PROBE_'` 不等时输出里写 `complete=false`、退 1。
- 步由环境变量 `PROBE_STEPS` 选（默认 `census-unlink,census-pairing,control`），逐态全量那两步另给 `PROBE_EXHAUST`、`PROBE_EXHAUST_FLOW`、`PROBE_EXHAUST_FAULT`、`PROBE_EXHAUST_MAX_GROUPS`，命令写在第七节。
- 撞重排之外的失败（编不过、测试红）：输出留在 `<work>.state/out-<步名>.txt`，`<work>` 仍是空的。
- 它的 `--selftest` 对我这份会红（模板自证拷的是模板那一段 `rerun_steps`），所以不列。

## 二、问题 1：16 段里 E6 点名的槽与点名它的记录，逐段判

### 2.1 推导链（每一步写依据；哪一步是读、哪一步是量）

1. **记录点名的位置条目就是那个单元自己的落点**（读）。`crates/singlefs-core/src/transaction.rs:16738`「locations: pool.location_entries(unit.slot, &unit.bytes),」；诞生代号是这次发布的 txg：`crates/singlefs-core/src/transaction.rs:16741`「birth_txg: txg,」；类标签取点名角色自己的：`crates/singlefs-core/src/transaction.rs:16739`「unit_class: identity.unit_class(),」。每一发布里谁点名谁：`crates/singlefs-core/src/transaction.rs:13440`「第 k 个事务（k < N − 1）一条记录、只点名文件第 k 个数据单元」，共享的节点 `crates/singlefs-core/src/transaction.rs:13441`「只在最后一个事务的记录里点名」。零单元发布的记录一项都不点名：`crates/singlefs-core/src/transaction.rs:1772`「named: Vec::new(),」（段 2、5 的记录就是这一种）。
2. **单元写先于点名它的记录**（读）。`.claude/kb/decisions/16-发布语义.md:215`「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」。所以前缀里每条记录点名的单元，写表里都有一次更早的、同盘同偏移的单元写（偏移 = 槽号 × 16384：`crates/singlefs-core/src/address.rs:103`「offset.0 == self.0 * SLOT_BYTES,」，槽宽 `crates/singlefs-core/src/pointer.rs:33`「16 KiB 槽号 6」）。探针逐条核这一步：`prefix_matched_earlier_unit_write`、`prefix_matched_class_and_birth`。
3. **r2 的 E6 对每个点名项一律按 32768 算**（读）。`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:248`「let named_end = named_start + DATA_UNIT_BYTES_OF_THE_POOL;」；而码 2 节点只占 16384：`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:62`「claimed_bytes: NODE_BYTES_OF_THE_POOL,」。恢复与池级 checker 读点名单元都按类取宽度：`crates/singlefs-core/src/recovery.rs:909`「UNIT_CLASS_INDEX_NODE => Some(usize::try_from(NODE_BYTES).expect("16384")),」、`crates/singlefs-core/src/recovery.rs:906`「UNIT_CLASS_DATA | UNIT_CLASS_PACKED => {」；`crates/singlefs-checker/src/walk.rs:13207`「MappingKeyUnitClass::IndexNode => node_bytes(),」、`crates/singlefs-checker/src/walk.rs:13212`「reader.read(location_device, location_slot * SLOT_BYTES, unit_bytes)」。类不认识：检查器 `crates/singlefs-checker/src/walk.rs:13411`「MappingKeyUnitClass::Unregistered(_) => return false,」当验不过。
4. **这一段的写不盖任何更早的写、基镜像在那段范围全零**（量）。24 个真实单元段（`label=unmodified` 的 7 行、对拍三条流的 5+6+6 行；我取出存在 `real-unit-lines.txt`，sha256 `b6bad10a0828712d664dbe01cedd46ffe8d2533dbdb614089029c87314260ddb`，与第二轮读产物那条腿存的同名文件逐位相同）上，我用 `grep -o '<列>=[0-9]*'` 加 `awk` 求和：`e2_over_an_earlier_write` 0、`e2_over_nonzero_base` 0、`e3_without_header` 0、`e3_unaligned` 0、`e3_claimed_bytes_differ` 0、`e4_not_beyond_mount_root` 0、`diag_records_of_same_identity` 0（七列各 24 行都是 0）；`e6_records_in_prefix_naming_a_slot` 总和 32、非 0 的 16 行（每行恰 2）；`group_sizes={2: ` 24 行。非单元段（删除流 29 段、对拍三条流各自的非单元段）的 `kinds` 里一个都没有 `UnitWrite`（`grep -c` 都得 0），所以流里的每个单元写都在这 24 段里、都受 E3 等长那一列管。
5. **合起来**（推，由 1 到 4 演绎）：设前缀记录 r 点名单元 u，位置 (d, s)，类 c。由 2，写表更早处有同盘同偏移的单元写 w_u，由 4 的 E3 等长，w_u 的字节数就是 c 的宽度，所以 u 的真实范围 = w_u 的范围。由 4 的 E2，这一段每个写都与同盘更早的每个写（含 w_u）不相交。所以 u 的真实范围与这一段任何写都不相交，E6 的命中只能来自宽读法多算的那一截——只有 c = 2（宽 16384）的点名项有这一截。读数 2 恒为 2，不是 1、也不是 3，也对得上：一条记录的两份副本（两块盘各一份，段 9、17、21 这类记录段的写数都是 2）各自都点名两块盘上的同一个槽，各命中本盘紧挨着的下一个槽。
6. **这条推导没用到而探针要量的**：命中的那条记录是哪一条（记录序号、写表下标）、点名项的类是不是 2、被命中的写与点名单元是不是紧挨（`touching=true`）、槽号差 1、诞生代号不同。这些是推导的预言，不是它的前提。

### 2.2 逐段（16 行；读数行取自 `out-classifier-and-arms.txt`（删除流，记 U）与 `out-pairing-flows.txt`（对拍三条流，记 P），都在 `research/prompts/per-write-judging-r2-sonnet-model/out/` 下）

「点名它的记录」一列是从段表读的（量：各流的段次序与每段写数，见 `P:` 各行）：单元段 k 前面最近的一次发布的记录段是 k−3，写数 2 就是一条记录的两份副本。写表下标、记录计数器、点名项序号要探针（`PROBE_E6_HIT` 行）才有，现在写「待跑」。

| 流 | 段 | 读数行 | 点名它的记录所在段（写数） | 判（推，待主 agent 跑） | 依据 |
|---|---|---|---|---|---|
| 删除流 | 12 | U:85 | 9（2） | 误报 | 2.1 第 5 步；前缀里只有零单元发布的记录（段 2、5）与段 9 |
| 删除流 | 16 | U:89 | 13（4） | 误报 | 同上；段 13 两条记录，只有点名紧挨着本段第一个写的那个节点的那一条命中 |
| 删除流 | 24 | U:97 | 21（2） | 误报 | 同上 |
| 删除流 | 28 | U:101 | 25（2） | 误报 | 同上 |
| 删除流 | 32 | U:105 | 29（2） | 误报 | 同上 |
| 第二批隔离流 | 12 | P:95 | 9（2） | 误报 | 同上 |
| 第二批隔离流 | 20 | P:103 | 17（2） | 误报 | 同上 |
| 第二批隔离流 | 24 | P:107 | 21（2） | 误报 | 同上 |
| 只靠记录施加流 | 12 | P:128 | 9（2） | 误报 | 同上；这一流段 12 的 30 个写 `diag_birth_differs_from_publish=30`（量，P:128），见第五节第 1 行 |
| 只靠记录施加流 | 20 | P:136 | 17（2） | 误报 | 同上 |
| 只靠记录施加流 | 24 | P:140 | 21（2） | 误报 | 同上 |
| 只靠记录施加流 | 28 | P:144 | 25（2） | 误报 | 同上 |
| 删到最后一批流 | 12 | P:165 | 9（2） | 误报 | 同上 |
| 删到最后一批流 | 20 | P:173 | 17（2） | 误报 | 同上 |
| 删到最后一批流 | 24 | P:177 | 21（2） | 误报 | 同上 |
| 删到最后一批流 | 28 | P:181 | 25（2） | 误报 | 同上 |

每行的依据是同一组：该行读数行里 E2、E3、E4 各列全 0 且 `e6_records_in_prefix_naming_a_slot` 恰为 2（量，读数行本身）；代码依据三处（读）：`crates/singlefs-core/src/transaction.rs:16738`「locations: pool.location_entries(unit.slot, &unit.bytes),」、`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:248`「let named_end = named_start + DATA_UNIT_BYTES_OF_THE_POOL;」、`crates/singlefs-core/src/recovery.rs:909`「UNIT_CLASS_INDEX_NODE => Some(usize::try_from(NODE_BYTES).expect("16384")),」。「与这一段在飞的写是不是同一个槽同一个诞生代号」这一问，16 行的预测都是**否**（点名单元的槽比在飞的写早一槽、诞生代号是上一次发布的 txg，在飞的写带这一次发布的 txg），读数在 `PROBE_E6_HIT` 行的 `same_slot`、`same_birth_txg`（Q2）。

16 行里没有「真点名」。8 个读数 0 的段（U:81、U:93，P:91、P:99、P:124、P:132、P:161、P:169）：4 个是段 8（每流一个）——前缀里只有零单元发布的记录，没有点名项；另 4 个（删除流段 20，三条对拍流各自的段 16）前一次发布的记录段写数是 6（三条记录）。三条记录的发布为什么没有「点名单元紧挨着下一个写」我没追（推：节点由开放段按游标连续取、容器要两槽对齐档，`crates/singlefs-core/src/allocator.rs:2433`「提交内生块：从开放段 bump；容器按 32768 对齐档，节点取游标处最低空槽。」、`crates/singlefs-core/src/allocator.rs:2513`「if footprint == UnitFootprint::TwoSlotsAligned && !start.is_multiple_of(2) {」，游标奇偶不同就会留一个空槽）。这 8 段不影响 E6n 的结论；探针的 `prefix_nearest_gap_after_a_named_extent` 把「紧挨」与「隔一个槽」直接读出来。

### 2.3 什么现象会推翻第 2.2 节

- 任一 `PROBE_E6_HIT` 行 `overlaps_by_class=true`：那一段是真点名（点名单元自己的范围与这一段的写相交），这一段的误报判断撤回，2.1 第 5 步的前提（E2 加 E3 等长加点名项等于更早的写）有一环不成立，要查哪一环。
- 任一 `PROBE_E6N_SEGMENT` 行 `prefix_matched_class_and_birth` 小于 `prefix_named_locations`：第 2 步（点名的单元在写表更早处写过）不成立，E6n 的充分性论证要改（点名了写表里没有的单元：基镜像里的由 `prefix_no_earlier_write_but_base_nonzero` 单数，两样都不是的由 `prefix_unmatched` 数）。
- 命中的 `named_class` 不是 2：宽度解释对这一行不成立，要另找原因。
- `prefix_unparsable_record_writes` 或 `prefix_unknown_class_locations` 非 0：记录点名区或类标签有这里没料到的形状。

## 三、问题 2：收窄后的 E6（E6n）、它在反例 B 的臂上拒不拒、态数

### 3.1 E6n 的定义

输入同 r2：写表、基镜像、段表。对单元段 k 的写集 W（写表里最低的下标记 i）：前缀 = 写表下标小于 i 的写。**E6n：前缀里每条 `JournalRecord` 写，同时满足三条，任一条不满足就整段退回（退回的意思同 r2：照今天的展开，不会比今天多一个状态）。**

- **a. 解得出。** 魔数 `SFSJ`、头 496 字节、点名项数（偏移 12 起 4 字节）乘 56 加 496 不越过 4096 字节（`crates/singlefs-core/src/journal.rs:725`「writer.put_u32(u32::try_from(self.named.len()).expect("点名项数 4 字节"));」是写者的次序；点名项 56 字节的字段表 `crates/singlefs-core/src/journal.rs:159`「点名项 56（D23（journal 的角色与格式） 已定项 17）：位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1。」）。
- **b. 类认得。** 每个点名项的类标签是 1、2、3 之一（`crates/singlefs-core/src/unit.rs:30`「pub const UNIT_CLASS_DATA: u8 = 1;」、`:31` 类 2 节点、`:32` 类 3 打包）。
- **c. 点名的范围不盖这一段的写。** 每个点名项的每条位置条目 (盘身份, 槽号)：范围 = [槽号 × 16384，槽号 × 16384 + 宽度(类))，宽度(类 1、3) = 32768、宽度(类 2) = 16384；它与 W 里任何同盘的写罩住的范围都不相交。

### 3.2 与 r2 的 E6 比，放宽在哪、收严在哪（写明，不当「澄清」）

- **放宽（字面上）一处**：类 2 的点名范围从 32768 缩到 16384。r2 的 E6 原文就写了「点名单元按 32768 字节算」（`research/prompts/per-write-judging-r2-sonnet-output.md:97` 那一行；探针写法 `research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:248`「let named_end = named_start + DATA_UNIT_BYTES_OF_THE_POOL;」）。所以 E6n 在 r2 的 E6 退回的 16 段上收下，是改了判据，**不是**判据在原条款下过了。
- **收严两处**：a（r2 的 `named_locations_of_a_record` 对越界点名区是 `break` 悄悄丢项、对解不出的记录返回空表）与 b（r2 对不认识的类按 32768 算，不退回）。两处在真实流上预测都不触发（第六节），收严的意义在臂上。
- **按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步**：①记一次输：E6 按最弱读法在 24 个真实单元段上退回 16 段，真实删除流 1,463,812,286 只降到 1,379,926,285（`research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:109`「passing_unit_segments=2 engine_total=1463812286 reduced_total_group_form=1379926285」）；②只许收严：E6n 不满足这一条，所以不算 E6 的修补，算新臂，要独立被攻，攻方只剩零轮（这一轮是最终轮，按 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」，写进判决的交用户表标「零轮」）；③写明收严在哪：上面。
- **打中之后先问四句**：它分不分辨臂？r2 的 E6 不分辨（基线 2，B 臂 2 变 6 全靠增量，`research/prompts/per-write-judging-r2-sonnet-s2-output.md` 第二节 2.1 表）；E6n 基线预测 0，B 臂 0 变 4，分辨。被判的系统当时看不看得到？看得到：恢复施加前验点名单元读的范围 `crates/singlefs-core/src/recovery.rs:904`「fn unit_bytes_for_class(unit_class: u8) -> Option<usize> {」，池级 checker 读的范围 `crates/singlefs-checker/src/walk.rs:13418`「.read(*location_device, location_slot * SLOT_BYTES, unit_bytes)」，都按类取宽度，与 E6n 的 c 同一个范围。满足的是字面的哪一个分句？E6 字面的「点名单元按 32768 字节算」。跑前条款给的改法在打中的格上还中不中？E6n 在那 16 格上预测不中。

### 3.3 E6n 没有覆盖什么

E6n 只管「恢复施加前验点名单元」与「池级 checker 验点名项对单元头」这两处读点名单元的范围（上面两处行）。别的读（池级 checker 扫整个单元区、读迹）不归 E6n：归 E2、E4，与没有实现的 E5。我没逐行追池级 checker 里别的读点名单元的路径（推：`crates/singlefs-checker/src/walk.rs` 里 `named_entries` 出现在 11632、11900、11912、13202、13405、22476 几行，我只读了 13194 与 13401 两个函数）。

### 3.4 E6n 放不放过反例 B（不放宽到放过它）

反例 B = 记录先落、它点名的单元后落（候选甲在这一形上漏红）。B 里记录在前缀、点名的单元在 W 里，所以点名项的位置条目与 W 里那个写同盘同槽、类标签相同（写者造的记录），范围起点相同：E6n 的 c 命中。点名类标签写错（类 2 点名 32768 的单元）起点仍相同，仍命中；点名的槽与 W 里任何写都不重叠的记录点名的不是 W 的单元，恢复验它读的范围与 S 无关，不归 E6n。所以 E6n 对 B 的拒绝**不依赖宽度取哪个**，只依赖起点。在真实流上造四条臂验这一句（`probe/named_slots_unlink.rs.in` 的 `probe_named_slot_census_over_the_unlink_stream_and_the_misordered_arms`）：

| 臂 | 怎么造 | E6n 预测（第 12 段或第 16 段） |
|---|---|---|
| record_before_units | 第 13 段四个记录写整段挪到第 12 段单元之前（r2 的 B 臂，r2 读到旧 E6 是 2 变 6，`research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:161`「label=record_before_units segment=12 … e6_records_in_prefix_naming_a_slot=6」） | 第 12 段 `e6n_narrow_record_writes=4`（基线 0）、`passes_e6n=false`；旧写法 6 |
| record_before_units_single | 只挪点名一个单元的那条记录的两份 | 第 12 段 `e6n_narrow_record_writes=2` |
| record_before_units_shared_nodes | 只挪点名共享节点的那条记录的两份 | 第 12 段 `e6n_narrow_record_writes=2` |
| late_unit | 第 12 段一个单元的两份副本挪到第 16 段开头（r2 的 A 臂） | 第 16 段 `e6n_narrow_record_writes=2`，且 `e4_not_beyond_mount_root=2`（r2 读到 `research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:127`「label=late_unit segment=16 … e4_not_beyond_mount_root=2」） |
| planted_one_group、planted_two_groups | 第 16 段的组挪到第 12 段单元的偏移上（r2 的两组同被盖） | 第 16 段 `e2_over_an_earlier_write` 为 2、4；E6n 也 ≥ 2：这类形状 E2、E6n 各自都拒 |

### 3.5 态数（算；前提：E6n 与 E1 到 E4 在这些段上成立，探针 `PROBE_E6N_SEGMENT` 的 `passes_e6n` 才把它升成量）

每段乙的态数 = 1 + Σ 各组（2^组内写数 − 1），取自 r2 的 `states_group_form` 列（量，`real-unit-lines.txt`）。整条流 = 1 + 非单元段的引擎态数 + 各单元段（过的取乙、不过的取引擎态数）。我用 `awk` 对 r2 输出重算了「全过」那一格：

| 流 | 单元段各自乙的态数 | 非单元段引擎态数和（量） | 现在（量） | **E6n 全过** | r2 的 E6（量） |
|---|---|---|---|---|---|
| 删除流 | 段 8 = 37、12 = 43、16 = 46、20 = 40、24 = 37、28 = 34、32 = 37 | 196 | 1,463,812,286 | **471** | 1,379,926,285 |
| 第二批隔离流 | 段 8 = 37、12 = 46、16 = 40、20 = 37、24 = 34 | 160 | 1,178,599,580 | **355** | 1,094,713,579 |
| 只靠记录施加流 | 段 8 = 37、12 = 46、16 = 28、20 = 25、24 = 25、28 = 37 | 179 | 1,107,689,646 | **378** | 1,090,650,353 |
| 删到最后一批流 | 段 8 = 37、12 = 46、16 = 40、20 = 37、24 = 34、28 = 37 | 172 | 1,195,376,807 | **404** | 1,111,490,806 |

某一段 E6n 不过时：整条流的数 = 全过那一格 − 该段乙的态数 + 该段的引擎态数（例：删除流只有段 12 不过是 471 − 43 + 268,435,455 = 268,435,883）。这四个 E6n 数只在「24 段全过」下成立；预测是全过，推翻它的是 `passing_e6n` 小于 `unit_segments`。
这些数还**不是**「逐写独立判能用的态数」：E5（S=∅ 那一遍的读迹）没有实现，D5 在这里没读数（第 5.2 节第三条的第 1 点）。

## 四、问题 2 后半：收下的段上逐态全量核候选乙与全组合等价

### 4.1 比什么

- **全组合**：段（或宇宙）里每个写落或不落的 2^n 个组合，每个都真判。真判走 `judge_crash_image`（CPU，与第二轮探针同一条）；前面各段全落、之后各段全不落（`.claude/kb/decisions/13-验证路线.md:141`「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」）。
- **候选乙**：按副本组分（同偏移、同归并键两条边连通，`groups_of_segment`；真实流上每组恰两个写，量的 24 行 `group_sizes={2: N}`）；S=∅ 判一次，每组每个非空子集判一次（每组 2 个写时每组 3 次）；状态 S 的合成有两种，都比：**并**（S=∅ 的红项并上每个组内子态的红项）与**增减**（S=∅ 的红项，减去各部分比 S=∅ 少掉的项、加上各部分比 S=∅ 多出的项，与第二轮攻方原型同一个写法，`research/prompts/per-write-judging-r2-opus-model/proto/src/main.rs:1256`「fn delta_of(baseline: &StateVerdict, parts: &[&StateVerdict]) -> BTreeSet<String> {」）。
- **两层**：红不红（合成的红项空不空对真判的空不空）、红项（两份集合相等不相等）；另数两个方向：真判有而合成没有的项（漏项，不安全的方向）、合成有而真判没有的（多报）。
- **不取样**：`exhaustive::run` 对序号区间 [起, 终) 每个状态判一次，末尾断言判的个数等于终减起；切片 i/N 的起止用整数乘除算，几片的区间首尾相接，合起来恰好盖满（桩自检里 5 片的状态数相加得 64 = 全量）。

### 4.2 成本（算；速度是下界，不是量到的速度）

24 个单元段合计 2^n − 1 共 **4,945,477,608** 个状态（算：四条流的 `engine_states` 单元段和 1,463,812,089 + 1,178,599,419 + 1,107,689,466 + 1,195,376,634，取自 `real-unit-lines.txt`）。
速度只有一个下界：`research/prompts/per-write-judging-r2-sonnet-model/out/out-universes.txt:33`「finished in 7.75s」，这一趟里含录流与 12 个格 × 1024 = 12,288 个状态的真判（`research/prompts/per-write-judging-r2-sonnet-model/out/out-universes.txt:17`「PROBE_SYNTH label=unmodified-trial0 segment=12 groups=5 writes=10 states=1024 baseline_red=false」起的 12 行），所以 **≥ 1,586 状态/秒**（算）；线程数由服务定、我不知道。按这个下界：

| 段的写数 n | 段数 | 状态数合计（2^n 个/段） | 按 1.6×10³ 状态/秒单进程的钟点（下界速度，所以是上界钟点） |
|---|---|---|---|
| 16 | 2 | 131,072 | 0.02 |
| 18 | 1 | 262,144 | 0.05 |
| 22 | 3 | 12,582,912 | 2.2 |
| 24 | 10 | 167,772,160 | 29.1 |
| 26 | 3 | 201,326,592 | 35.0 |
| 28 | 1 | 268,435,456 | 46.6 |
| 30 | 4 | 4,294,967,296 | 745.7 |

所以全部 24 段逐态全量单进程是 35 天以上的量级（算），**没有哪一层是「跑不起」的理由**（`.claude/agent-common.md`「崩溃点测试准确率和正确率优先，其次再衡量时间成本」），但 30 写的 4 段要靠切片的片数（每片 ≤ 4×10⁶ 状态就是每段 256 片）或另走 GPU；我不替主 agent 定要不要跑。**先跑第七节命令 C2 读 `states_per_second`，再定后面几层怎么切。**

GPU 一路我没读够，不写命令：现有 GPU 对拍把每个状态的结果存进内存里的数组（`crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:2479`「cpu_red_items_of_each_state: Vec<String>,」、`:2484`「gpu_words_of_each_state: Vec<[u32; VERDICT_WORDS]>,」），十亿级的段放不下，要走 GPU 得另写流式比较。

### 4.3 命令的分层（命令整条在第七节）

- **层 A（≤ 22 写，6 段，合计 12,976,128 个状态：上表 16、18、22 三行之和）**：删除流段 28；第二批隔离流段 24；只靠记录施加流段 16、20、24；删到最后一批流段 24。一次任务一条流。
- **层 B（24 写，10 段，每段 16,777,216 个状态）**：删除流段 8、24、32；第二批隔离流段 8、20；只靠记录施加流段 8、28；删到最后一批流段 8、20、28。每段切 8 片（`24@3/8` 写法）。
- **层 C（26 写以上，8 段）**：删除流段 20（26 写）、12（28 写）、16（30 写）；第二批隔离流段 12（30）、16（26）；只靠记录施加流段 12（30）；删到最后一批流段 12（30）、16（26）。要么切很多片，要么只走下面的按阶全量。

### 4.4 比较器自己会不会红（校验路径要证明它会红）

`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」。两个办法，都在探针里：
- **阳性对照**（`probe_exhaustive_comparer_flags_the_planted_two_group_pairs`）：第二轮「两组同被盖」的 91 对，这一趟经同一个比较器 `run` 判一遍。第二轮量到并合成 91 对里 78 对红项不同、共 243 个状态（我对 `out-two-groups.txt` 再数了一遍：91 行 `PROBE_SYNTH label=two-groups` 的 `yi_items_differ` 之和 243、非 0 的 78 行，`research/prompts/per-write-judging-r2-sonnet-model/out/out-two-groups.txt:325`「pairs=91 skipped=0 pairs_where_yi_items_differ=78 pairs_where_yi_red_differs=0 pairs_where_jia_items_differ=91」）。这一趟并合成的数要逐位相同（91、78、243），不同就是比较器与第二轮那条路不是同一件事。基线是绿的（`baseline_red=false`），所以增减合成与并合成在这一格相等（推）。
- **造坏**（`PROBE_EXHAUST_FAULT`）：真实流上的段多半是全绿对全绿——第二轮读产物那条腿自己写过：未改动段那三格的红不红一层可能是全绿对全绿（转述，出处 `research/prompts/per-write-judging-r2-sonnet-s2-output.md` 第二节 2.2「读法与限定」第 1 点），产物里没有「有多少个状态是红的」那一列。全绿对全绿时任何合成都对，比较没有牙。造坏把一份（`copy:`）或一组两份（`group:`）副本的最后一个字节取反：单元头不动，E1 到 E4 与 E6n 读的东西都不变，段照样被收下，状态里却有红。逐态全量比的是「有红的段上乙还对不对」。每个逐态全量的任务都要有「不造坏」和「造坏」各一份读数。

### 4.5 按阶全量（不是逐态全量）

`PROBE_EXHAUST_MAX_GROUPS=k`：只判至多 k 个组里有写落下的状态，每个被碰到的组取它的每个非空子集，一个不漏、一个不重。15 组时的状态数（算）：k=1 46、k=2 991、k=3 13,276、k=4 123,841、k=5 853,570、k=6 4,502,215、k=7 18,575,560、k=8 60,795,595，k=15 是全部 4^15 = 1,073,741,824。它覆盖「任意至多 k 个副本组联合的相互作用」，**不证明** k+1 个及以上组联合的相互作用，所以不能写成逐态全量；写进判决时只写「k 阶以内全量」。它是层 C 在没有 GPU 时唯一一个今天交得起的全量形态。

### 4.6 什么现象会推翻第四节

- 任一 `PROBE_EXHAUST_SLICE` 行的 `union_items_differ` 或 `delta_items_differ` 非 0：乙在这一段这一片与全组合不等价；`PROBE_EXHAUST_MISMATCH` 行给前 8 个状态。`union_items_missing_from_the_synthesis` 非 0 是漏项（候选乙漏红，最重的一种）。
- `PROBE_EXHAUST_CONTROL` 不是 91、78、243：比较器与第二轮不是同一件事，第四节的数都先不引。
- 不造坏时 `real_red=0` 且造坏时 `real_red` 也是 0：造坏没让红出现（被造坏的副本没被池级 checker 判到），这一族「等价」读数没有牙；换 `PROBE_EXHAUST_FAULT=flip:` 另选位置，或另找造坏办法。

## 五、问题 3：复核第二轮判决

判决表 10 行在 `research/prompts/per-write-judging-r2-main-verification.md` 第 17 到 26 行；下面逐行写：判决站不站得住、它说的那句话够不够得着它想打的对象、是不是同样打中别的臂、什么现象推翻我这一行。

| # | 判决行 | 辩方判定 | 依据 | 什么现象推翻我这一行 |
|---|---|---|---|---|
| 1 | `research/prompts/per-write-judging-r2-main-verification.md:17`「辩方收严判据 E1–E6 在真实删除流上能把态数降到 471」不采 | **站得住；够不着「这条路降不下去」**：它判的只是 E1–E6 的字面写法（E6 按 32768 算），E6n 不在它的射程里 | 量：24 行里 E 列除 E6 外全 0（第二节第 4 步）；同一行写的「只有 E6 有读数，16 段为 2」（`research/prompts/per-write-judging-r2-main-verification.md:17`「只有 E6 有读数，16 段为 2」）逐字成立。限定：非 E 列的 `diag_birth_differs_from_publish` 在只靠记录施加流段 12 是 30（量，`research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:128`「segment=12 writes=30 … diag_birth_differs_from_publish=30」），判决那句只说 E 列，没错，但读的人别当它「全 0」 | E6n 在 24 段上不是全过（`passing_e6n` 小于 `unit_segments`） |
| 2 | `research/prompts/per-write-judging-r2-main-verification.md:18`「E6 在真实流上的基线 2 是判据宽度造成的误报，还是真有记录点名这一段要写的槽」未定，第三轮核 | **站得住（挂起对）；现在能往前推一步**：推的结论是 16 段全是误报（第二节），等探针 | 当时产物确实分不开两种读法（第二轮读产物那条腿第二节「E6 的基线 2 是什么，产物答不了」） | 第二节 2.3 的任一条 |
| 3 | `research/prompts/per-write-judging-r2-main-verification.md:19`「收严之后判据收下的段上，候选乙与全组合等价」只在极少取样上量过 | **站得住；还要加一句：这极少的取样也没有牙** | 判据之内有量的只有删除流段 20 的 3 次、3072 个状态（`research/prompts/per-write-judging-r2-main-verification.md:19`「判据之内有量的只有段 20 的 3 次、共 3072 个状态」；`research/prompts/per-write-judging-r2-sonnet-model/out/out-universes.txt:25`「PROBE_SYNTH label=unmodified-trial0 segment=20 groups=5 writes=10 states=1024 baseline_red=false」起三行，量）；这三次 `baseline_red=false`、四个差全 0，但产物没有「有多少个状态是红的」那一列，红不红一层可能是全绿对全绿（第二轮读产物那条腿自己写的推）。对拍三条流被 E 判据放行的段 8、16 与删除流段 8 一个逐态对照都没有 | 探针的 `real_red` 在未造坏的段上非 0（那就有牙了，这句要改） |
| 4 | `research/prompts/per-write-judging-r2-main-verification.md:20`「反例 A（一对）：甲的两种合成、乙的并合成在红项层与全组合不同，乙的增减合成两层都不差」采（取样） | **站得住；够不着 E1 到 E6 加乙**：A 在判据里被 E4 拒 | 量：`late_unit` 臂段 16 `e4_not_beyond_mount_root=2`（`research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:127`「segment=16 writes=32 … e4_not_beyond_mount_root=2」）。这一行说明「乙增减」比别的合成强，但打的是没有判据的乙，打不到判据收下的段；而 A 一对那一格上乙·增减不中的读数里含一个 4094 状态的小段全量，不全是取样（`research/prompts/per-write-judging-r2-opus-s2-output.md:407`「是取样 15954 + 16405 + 7269 态加小段全量 4094 态上的 0」） | `late_unit` 臂上 E4 与 E6n 都判「收」 |
| 5 | `research/prompts/per-write-judging-r2-main-verification.md:21`「反例 A（两对）分辨四种合成」分辨不了 | **站得住（判别子观测不到）；同样打中所有臂；也在判据之外** | 同第 4 行：A 的单元诞生代号不超挂载根，E4 拒；两对形没造过静态判定。四种合成逐格同数就是 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」的「打中不分辨臂」：病根是各臂共用的前提，不拿它判臂 | 在判据收下的段上造出两对形且判出差 |
| 6 | `research/prompts/per-write-judging-r2-main-verification.md:22`「反例 B（记录先写、单元后写，甲漏红）」没造出，不采为「没有」 | **站得住（一次不算数）；第三轮这一格仍没有动态读数** | 两轮的 B 臂都只做静态判定（`engine_states=None`，`research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:161`「label=record_before_units segment=12 … e6_records_in_prefix_naming_a_slot=6」），没有枚举状态，所以「甲漏红」没有量过。我的臂（第三节 3.4）也只是静态判 E6n 拒不拒，不造 B 的状态；不替它加动态读数 | 在 B 臂上枚举状态、候选甲全组合判红而代表判绿 |
| 7 | `research/prompts/per-write-judging-r2-main-verification.md:23`「两组同时被盖时，乙在红项层与全组合不同」采（量的） | **数站得住，射程够不着判据内**：我重数了 | 量：91 行 `PROBE_SYNTH label=two-groups` 的 `yi_items_differ` 之和 243、非 0 的 78 行、状态 91×16 = 1456（`research/prompts/per-write-judging-r2-sonnet-model/out/out-two-groups.txt:325`「pairs=91 skipped=0 pairs_where_yi_items_differ=78」）。但这 91 对是把第 16 段的组挪到第 12 段单元的偏移上造出来的（`plant_over`），按构造 E2 拒；第二轮没有对挪过的写跑分类器（第二轮读产物那条腿第二节「判据元素 × 取样点与臂」前的说明），所以「E2 拒它」是推的。我的探针对挪过的写跑了（`planted_one_group`、`planted_two_groups` 两臂，预测 `e2_over_an_earlier_write` 为 2、4，E6n 也 ≥ 2）。另：差有两个方向，真判有而乙没有的项（`I-3.1`、`I-3.11` 各 3 行，第二轮读产物那条腿的 `tabulate-out.txt`）是乙漏项，比多报重 | `planted_two_groups` 臂上 `e2_over_an_earlier_write` 为 0（那就是判据收下了它们） |
| 8 | `research/prompts/per-write-judging-r2-main-verification.md:24`「混合种类段、多组被盖两臂上四种代表法都有差」采（量的），判据不收 | **站得住；同样打中所有臂；判据不收** | E1 拒混合段（`research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:235`「label=mixed_segment segment=12 writes=30 … e1_only_unit_writes=false」）；多组被盖同第 7 行。这一行没有说的：不收 = 退回今天的 2^n，不是「乙不对就算了」 | 混合段上 E1 判「收」 |
| 9 | `research/prompts/per-write-judging-r2-main-verification.md:25`「第一轮判决 11 行都站得住，4 行打中的射程要缩小」采 | **站得住**。我现查了第二轮辩方逐行核过的两处：`crates/singlefs-core/src/transaction.rs:10564`「原样字节写到目的地（头不盖戳、birth 不变）」、`crates/singlefs-checker/src/walk.rs:17931`「if walked_placements.is_subset(rebuilt_placements) {」，原文逐字在 | 另 `crates/singlefs-checker/src/walk.rs:11256`「return birth_txg <= self.mount_root_checkpoint_txg;」、`crates/singlefs-checker/src/walk.rs:11252`「if instance > self.mount_root_instance {」也在原行 | 其中任一行改了（它们在 `crates/` 里，随实现变） |
| 10 | `research/prompts/per-write-judging-r2-main-verification.md:26`「方法限定：判的态只占被判段全量约万分之一到万分之二，合计 464438 态」采 | **数对，说法有一处够不着**：合计里混着全量 | 量：96190 + 130413 + 148476 + 89359 = 464,438（算，`research/prompts/per-write-judging-r2-opus-model/out/small_segments_exhaustive.txt:85`「judged=89359 full_domain_of_the_judged_segments=89359」是 100%）。取样的 375,079 个占各自被判段全量 0.0236%、0.0077%、0.0203%（算：96190/407896056、130413/1692401652、148476/729808888），「约万分之一到万分之二」的上沿偏小（2.36）。那句话把小段全量（89,359，不取样）与取样（375,079）并成一个「判的态只占约万分之一到万分之二」 | 重算 |

### 5.2 判决正文三句结论，逐句

- **`research/prompts/per-write-judging-r2-main-verification.md:32`「态数只降了约 6%」**：数成立（算：1,463,812,286 − 1,379,926,285 = 83,886,001，占 5.73%；`research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:109`「passing_unit_segments=2 engine_total=1463812286 reduced_total_group_form=1379926285」；降的就是通过的段 8、20：16,777,215 − 37 加 67,108,863 − 40）。「几乎什么都收不下」只对 E6 的字面写法成立（第一行已写）。
- **`research/prompts/per-write-judging-r2-main-verification.md:33`「判据放宽到能收下那些段，两组同时被盖时按单元副本组判在红项层就会与全组合不同（91 对里 78 对）」**：**接错了射程，够不着**。那 16 段卡在 E6，放宽 E6（E6n）只是把点名范围按类取宽度；「两组同时被盖」的 91 对要把第 16 段的写挪到第 12 段单元的偏移上，盖的是更早的写，E2 拒（预测 `e2_over_an_earlier_write` 为 2、4），E6n 也拒。放宽 E6 不放宽 E2，所以这 91 对与「E6 误报还是真点名」无关，不构成「放宽 E6 的代价」。
- **`research/prompts/per-write-judging-r2-main-verification.md:35`「这是第三轮唯一要定的事。」**：**不对，至少还漏了下面几处**（逐处给出处）。
  1. **E5（S=∅ 那一遍的读迹）没有实现，没有读数**：`research/prompts/per-write-judging-r2-sonnet-output.md:96`「当前没有机器检查它的实现…不能据此降态」。就是说就算 E6n 在 24 段全过，471、355、378、404 在 E5 上仍是「未满足」，那一份报告自己写了不能据此降态。「唯一」把它漏了。
  2. **E3「落在单元区」那半句没量**：第二轮读产物那条腿的前提表标「半量，仍是推」（`research/prompts/per-write-judging-r2-sonnet-s2-output.md:195`「落在单元区内」那一行：只量到写集分开，区边界没读）。
  3. **比较没有牙**：真实流的段若全绿，乙与全组合「等价」是全绿对全绿，任何数都不构成证据（第四节 4.4）。要有牙只有造坏与阳性对照。
  4. **动态对照覆盖面小**：对 E 判据放行的段，逐态对照只做了删除流段 20 三次；删除流段 8、对拍三条流各自的段 8 与 16 一个都没做（`research/prompts/per-write-judging-r2-sonnet-s2-output.md` 第四节第 5 点，第二轮读产物那条腿写的）。
  5. **E4 的两处没走到**：前缀里没有根槽写那一支、写表里挂载根与池级 checker 的挂载根是不是同一个，都是推（`research/prompts/per-write-judging-r2-sonnet-s2-output.md:196`「E4 实例与诞生代号超过挂载根」那一行）；池级 checker 取的是 `crates/singlefs-checker/src/walk.rs:21364`「mount_root_checkpoint_txg: roots[newest_index].2.checkpoint_txg,」，我没追它与写表里「最后一条根槽写」逐项相同。
  6. **取样点没写的四类**（σ 段、整理与快照流的复用段、已准入的在飞头、GPU 判器一侧）还开着：`research/prompts/per-write-judging-r2-sonnet-output.md:237`「没写 σ 段、整理与快照流的复用段、已准入的在飞头」。四条流都是删除类发布、没有搬迁批（推，`research/prompts/per-write-judging-r2-sonnet-output.md:120`「删除流与对拍三条流是删除类发布，没有搬迁批」那一行写的），所以覆盖面只是这四条流。
  7. **非单元段不降**：471 里 196 个态是非单元段的引擎态数（量，`awk` 对 `PROBE_TIGHT_SEGMENT` 行的 `engine_states` 求和），逐写判碰不到它们。
  8. **入库形态我没查**：合成出来的状态判定怎么进那一份判定库、GPU 与 CPU 对拍比的是不是红项层（`research/prompts/per-write-judging-r1-main-verification.md:31`「要保红项」），我没读 `verdict_store` 与对拍那一侧，不知道逐写判在入库上还欠什么。
  9. **E6n 是零轮的新臂**（最终轮，不再开第四轮）：写进判决的交用户表标「零轮」，`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」。

### 5.3 第二轮各格报出的数，能不能从产物核出来（正文问题 3）

| 数 | 核法 | 结果 |
|---|---|---|
| 6%（约） | 上面的算：5.73% | 核出来，「约 6%」成立 |
| 78/91 | `grep -c 'PROBE_SYNTH label=two-groups'` 91；`yi_items_differ` 非 0 的 78 行、总和 243、状态 91×16 = 1456；汇总行 `research/prompts/per-write-judging-r2-sonnet-model/out/out-two-groups.txt:325`「pairs=91 skipped=0 pairs_where_yi_items_differ=78」 | 核出来 |
| 3072 | 段 20 三次 × 每次 1024（`research/prompts/per-write-judging-r2-sonnet-model/out/out-universes.txt:25`「label=unmodified-trial0 segment=20 groups=5 writes=10 states=1024」起三行） | 核出来；限定：这 3072 个状态是不是都绿读不出（第 3 行） |
| 464,438 | 四个 `PROTO_PLANNED_STATES` 行相加 | 核出来；限定：含 89,359 个全量（第 10 行） |
| 汇总漏 4 行 | `research/prompts/per-write-judging-r2-sonnet-model/tools/rerun.sh:95`「grep -h '^PROBE_'」；四个 `out-*.txt` 里含 `PROBE_` 的行 190 + 107 + 15 + 310 = 622，行首是 `PROBE_` 的 189 + 106 + 14 + 309 = 618 = `out.txt` 行数 | 核出来 |

没有对不上的数；够不着的地方是第 7 行（射程）与第 10 行（说法）。

### 5.4 交用户的表（判决文件第 41 到 46 行）逐行

- 「收严判据 E1–E6 | 被攻过两轮，没立 | 真实删除流只降约 6%（量的）」：站得住，补「E6 字面写法」。
- 「E6 基线 16 段 | 未定」：推的结论是 16 段全误报，等探针；E6n 是新臂。
- 「候选乙 | 被攻过一轮 | 反例 A 一对两层不差（取样）；两组同被盖时红项有差（量的）」：**第二句够不着判据内**（第 7 行）。
- 「候选甲 | 红项层不够（量的）」：站得住（两轮都量到）。

## 六、跑前写死的预测（跑出来之后不改，只往下另记；都是推的，待主 agent 跑）

「推翻」栏写读到什么就推翻；推翻之后怎么改写在最后一栏。

| 号 | 预测 | 依据 | 推翻它的读数 | 推翻之后 |
|---|---|---|---|---|
| Q1 | `census-unlink`，`label=unmodified` 的 7 个单元段：段 12、16、24、28、32 是 `e6_old_wide_record_writes=2 e6n_narrow_record_writes=0 wide_only_record_writes=2 passes_old=false passes_e6n=true`；段 8、20 三项全 0、`passes_old=true passes_e6n=true` | 第二节 2.1；r2 的 E6 读数照复现 | 任一段 `e6n_narrow_record_writes` 非 0，或 `e6_old_wide_record_writes` 与第二轮的 2 / 0 对不上（探针与第二轮不是同一件事） | 非 0 的那段是真点名，第二节那一段的误报判断撤回；对不上先查探针 |
| Q2 | `PROBE_E6_HIT` 行：删除流 5 段每段 4 行、对拍三条流 3、4、4 段每段 4 行，共 64 行；每行 `named_class=2 touching=true same_slot=false same_birth_txg=false overlaps_by_class=false`，`named_end_by_class` 等于 `in_flight_offset` | 一条记录两份副本各点名两块盘的同一个槽，各命中本盘紧挨着的下一个槽 | 任一行 `overlaps_by_class=true`、`named_class` 不是 2、`touching=false` | 同 Q1 |
| Q3 | 每个 `PROBE_E6N_SEGMENT`：`prefix_matched_class_and_birth` 等于 `prefix_named_locations`，`prefix_no_earlier_write_but_base_nonzero=0 prefix_unmatched=0 prefix_unparsable_record_writes=0 prefix_unknown_class_locations=0` | 第二节 2.1 第 1、2 步 | 任一格不等或非 0 | 2.1 第 2 步不成立，E6n 的充分性论证要改（点名的单元不全在写表更早处） |
| Q4 | `PROBE_E6N_FLOW`：删除流 `passing_old=2 passing_e6n=7 reduced_total_old=1379926285 reduced_total_e6n=471`；第二批隔离流 `passing_old=2 passing_e6n=5 … 1094713579 … 355`；只靠记录施加流 `passing_old=2 passing_e6n=6 … 1090650353 … 378`；删到最后一批流 `passing_old=2 passing_e6n=6 … 1111490806 … 404` | 第三节 3.5 的算 | 任一数不等 | 看 `passing_e6n` 与 `unit_segments` 差几段、哪几段 |
| Q5 | 臂：`record_before_units` 段 12 `e6_old_wide_record_writes=6 e6n_narrow_record_writes=4 passes_e6n=false`；`_single` 段 12 narrow=2；`_shared_nodes` 段 12 narrow=2；`late_unit` 段 16 narrow=2、`e4_not_beyond_mount_root=2`；`planted_one_group` 段 16 `e2_over_an_earlier_write=2`、narrow ≥ 2；`planted_two_groups` 段 16 `e2_over_an_earlier_write=4`、narrow ≥ 2；`PROBE_E6N_ARM_SHAPE` 的 `single_named_writes=2 shared_nodes_writes=2`、`named_counts` 是 {1,1,13,13} 的某个排列（13 是推：第 12 段 14 个单元里 2 个数据单元、12 个共享） | 3.4 | 任一臂的段上 narrow 为 0（E6n 放过了那一形） | E6n 对那一形不起作用：要么起点比较写错，要么那一形的点名在今天的流上构造不出；写进「没打中的形状」 |
| Q6 | `control`：`PROBE_EXHAUST_CONTROL pairs=91 skipped=0 states=1456 pairs_union_items_differ=78 states_union_items_differ=243 pairs_delta_items_differ=78 states_delta_items_differ=243` | 第二轮量到并合成 78、243；基线绿时增减等于并 | 任一数不等 | 比较器与第二轮不是同一件事，第四节的数先不引 |
| Q7 | 逐态全量，不造坏：每个收下的段 `real_red=0 union_items_differ=0 delta_items_differ=0 union_items_missing_from_the_synthesis=0` | 第二轮 Q7 九次取样全 0；8321 在三段上单写、成对、只差一个、随机 2000 个与 S=∅ 逐态相同 | 任一非 0 | 非 0 的片就是乙在这一段不等价的证据，写出 `PROBE_EXHAUST_MISMATCH` 前 8 个状态；`real_red` 非 0 则第三轮判决「全绿对全绿」那句要改 |
| Q8 | 逐态全量，造坏 `copy:0`（一份副本最后字节取反）：`real_red` = 2^(n−1)（含这一份副本的状态都红），`union_items_differ=0 delta_items_differ=0`；`group:0`：`real_red` = 3 × 2^(n−2) | 点名项校验和对不上，池级 checker 对每份副本各判一次 I-2.1（`crates/singlefs-checker/src/image.rs:2076`「judgements.judge("I-2.1", matches, || {」）。推的：在飞单元是否被判到我没追 | `real_red=0`：造坏看不见；`real_red` 对而 `union_items_differ` 非 0：乙漏了这一类 | `real_red=0` 时造坏另找办法（造到头里），在此之前这一格没有牙 |
| Q9 | 第一个逐态全量任务的 `states_per_second` 不低于 1,586 | 第四节 4.2 的下界 | 低于 | 4.2 的钟点表按新速度重算 |

## 七、要主 agent 跑的命令

`<冻结副本>` = 一棵两份测试文件里还没有 `per_write_judging_named_slots` 的树（开工快照倒推出的树，或主工作区的一份拷贝）；`<新的草稿目录N>` = 工作根下一个不在的目录，每条命令一个，不要复用。在哪个目录起：仓根 `/home/fy5090/code/singlefs`（脚本只从参数取路径）。每条命令的通用判法：`TASK_SERVER_RESULT` 那一行 `type=succeeded exit=0`；`task-server/task log <号>` 里 `name=rerun_output … complete=true`（`lines`、`probe_lines_in_the_per_step_files` 相等）；读 `<新的草稿目录N>/out.txt`；跑完 `<新的草稿目录N>.state/tree`（仓副本）与编译目录当场删（`task-server/task clean <号>`）。撞内存上限被重排时同一条命令原样重起即可，已完成的步不重跑。退 101 是编不过：错误在 `<新的草稿目录N>.state/out-<步名>.txt`，请新派辩方腿承接修（派发提示写「承接：research/prompts/per-write-judging-r3-sonnet-output.md」并给任务号）。

**C1 普查、臂、阳性对照（默认三步；问题 1 与问题 2 的前半）**
- 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录1>`
- 看：Q1 到 Q6。先读 `PROBE_E6N_FLOW`（四行）、`PROBE_E6N_SEGMENT`（`passes_e6n`）、`PROBE_E6_HIT`（64 行）、`PROBE_E6N_ARM_SHAPE`、各臂的 `PROBE_E6N_SEGMENT`、`PROBE_EXHAUST_CONTROL`。

**C2 逐态全量：最小三段，不造坏（只靠记录施加流段 16、20、24，共 393,216 个状态）**
- 命令：`task-server/task submit -- env PROBE_STEPS=exhaust-pairing PROBE_EXHAUST_FLOW=applied-by-records PROBE_EXHAUST=20,24,16 bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录2>`
- 看：Q7、Q9：三行 `PROBE_EXHAUST_SLICE`（`judged` 依次 65536、65536、262144，`real_red`、`union_items_differ`、`delta_items_differ`、`union_items_missing_from_the_synthesis`），`states_per_second`。段被 E6n 判退回时是 `PROBE_EXHAUST_SKIPPED`，那就是 Q1 到 Q4 红了。

**C3 同三段，造坏两种（各一条命令）**
- 命令：`task-server/task submit -- env PROBE_STEPS=exhaust-pairing PROBE_EXHAUST_FLOW=applied-by-records PROBE_EXHAUST=20,24,16 PROBE_EXHAUST_FAULT=copy:0 bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录3>`
- 命令：同上，`PROBE_EXHAUST_FAULT=group:0`，`--work <新的草稿目录4>`
- 看：Q8。`PROBE_EXHAUST_FAULT` 行点名改了哪几个写表下标。

**C4 逐态全量：层 A 其余三段（22 写，各 4,194,304 个状态）**
- 命令（删除流段 28）：`task-server/task submit -- env PROBE_STEPS=exhaust-unlink PROBE_EXHAUST=28 bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录5>`
- 命令（第二批隔离流段 24）：`task-server/task submit -- env PROBE_STEPS=exhaust-pairing PROBE_EXHAUST_FLOW=quarantined PROBE_EXHAUST=24 bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录6>`
- 命令（删到最后一批流段 24）：把上一条的 `PROBE_EXHAUST_FLOW` 换成 `last-batch`，`--work <新的草稿目录7>`
- 每条再各配一条 `PROBE_EXHAUST_FAULT=copy:0` 的（`--work` 另给）。先跑 C2 读速度再交这几条。

**C5 层 B、层 C 的切片与按阶全量（模式，先 C2 读速度再定交哪些）**
- 层 B 每段切 8 片：`PROBE_EXHAUST=<段号>@<0..7>/8`，其余同 C4。层 C 的片数自定（每片 ≤ 4×10⁶ 状态就是 26 写 16 片、28 写 64 片、30 写 256 片）。
- 按阶全量（例：删除流段 16，至多 5 个组里有写落下，853,570 个状态）：`task-server/task submit -- env PROBE_STEPS=exhaust-unlink PROBE_EXHAUST=16 PROBE_EXHAUST_MAX_GROUPS=5 bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r3-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结副本> --work <新的草稿目录8>`，另配 `PROBE_EXHAUST_FAULT=copy:0` 的一条。`PROBE_EXHAUST_SLICE` 行的 `max_groups=Some(5)` 与 `of=853570` 要对上。
- 判决要引的数，主 agent 在入库的装置上重做一次才能引（`.claude/rules/three-way-inference.md` 判决由主 agent 做那一节第 3 条）；这份副本上的读数在主 agent 跑出之前一律是「推的，待主 agent 跑」。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果：第五节每行的「辩方判定」是辩方的读法。没往统一任务服务交任务；判决要引的数没复跑（命令在第七节）。
- 探针没编、没跑：没有一个 `PROBE_` 行有读数。我只做了一份**桩自检**：用 `rustc` 把一份假的 `singlefs_verification`（`tools/stub-selfcheck/stub_verification.rs`，只有探针用到的类型与函数签名，签名照源码抄、函数体是空的）与探针里的分类器、比较器拼成测试程序，编过、各跑了三个测试（记录解析、宽读法与按类取宽度分开一个宽度造成的命中、比较器在加性判定上不报、在跨组判定上报红、5 片相加得 64、按阶 1/2/3 阶得 10/37/64 个状态、造坏三种写法）全过；`research/prompts/per-write-judging-r3-sonnet-model/tools/stub-selfcheck/build.sh` 我没跑（项目代码闸只认经服务交），手工等价的 `rustc` 序列在草稿目录 `stubcheck/` 里跑的，二进制与库已删。**它没证明**：探针对真实 `Prepared`、`RecordedFlow`、`judge_crash_image` 的类型对不对（桩的签名是我读源码抄的）、探针在真实流上读出什么。三个文件的 `use` 路径（`singlefs_verification::crash::…`、`memory_pool::…`）照第二轮编过的探针写，真实编译可能还有我没料到的类型错。
- 没追池级 checker 里除 13194 与 13401 两个函数之外读点名单元的路径（第三节 3.3）；没读 GPU 对拍的流式比较能不能做（第四节 4.2）；没读 `verdict_store` 与判定库入库那一侧（第五节第 5.2 第 8 点）。
- 没量「真实流的段是不是全绿」：产物里没有这一列（第四节 4.4），探针的 `real_red` 才有。
- 没验两对副本形的 A、点名一个单元形的 B 的动态读数（第五节第 6 行）；我造的 B 臂只判 E6n 拒不拒，不枚举状态。
- E5（读迹）没有实现，也没写：库里没有对池级 checker 的记读包装，要加库里的件，不在我的写范围里。
- 没碰别的腿这一轮的报告（禁读清单）；没做 git 写操作。草稿目录里只有 `stubcheck/` 的桩源码与 `real-unit-lines.txt`、`e6-table.txt`、`steps_body.sh`、`splice.py`（`du -sh` 232K，不用清）。
