# 逐写独立判 第一轮 正推腿报告（Sonnet）

批次：records/2026-11-05-删轻型并入重型.md#并包。立场：正推（从 `crates/` 的实现推「应该是什么」，再与背景材料的量、候选逐格比）。
背景材料：research/prompts/_per-write-judging-r1-background.md（正文、附录），调查员事实 /home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md。
草稿目录：/home/fy5090/.local/share/singlefs/work/per-write-judging-r1-forward（认领过，没写文件进去，没建副本、没建模型）。
行号都是今天的工作区现查的（工作区有未提交改动，`crates/singlefs-checker-tier` 已并入验证包）；调查员报告里的几个行号已移位，下面以今天的为准。
标注：「量」= 背景材料正文第三节主 agent 的探针读数；「读」= 我读代码得到；「推」= 我推的，没有现场观测。

## 总览（先说结论，细节在各节）

1. 判据要保的是「判定 = 它读到的字节与持久集合的纯函数」。一个状态判得与 S=∅ 相同，当且仅当 S 里的写没动 S=∅ 那一遍读过的字节、也没改变扫描候选带来的新读。把这句拆开，单元段上充分条件是五条（第 1 节的 D1 到 D5），前四条读写表与基镜像就能判，第五条要 S=∅ 那一遍的读迹（只记读过哪几个扇区，不看判定结果）。
2. 候选甲在「判据判为可以逐写判」的段上，「红不红」「红项」两层都与全组合等价，但只在 D1 到 D5 全成立时；候选乙在这样的段上也等价，是甲的超集。两者分开的地方在判据之外：乙能保「同一单元两份副本一起」的组内相互作用，甲保不了（第 2 节）。
3. 调查员的六类没列全：缺「已准入单元的副本组」（I-1.8 / I-1.13 按两份副本成组）、「单元写与系统配置槽同段」（I-7.7 门槛耦合）；六类里的 b3 条件写宽了，良构的相邻单元写不互相认领（第 3 节）。
4. 观察者：三类（只记计数且读持久集合、交回镜像且按读集去重、读恢复报告），逐写判下处置不同；续做观察者的行为随读集不变、挂完的盘由「崩溃镜像 + 观察者发的写」逐状态重建，所以后者仍是每状态一份判（第 4 节）。
5. 取样点不够撑全称结论：三个段都是「无旧内容可盖」的段，探针没有取到任何一个判据应当拒掉的段、也没取到准入分支、混合种类段、观察者（第 5 节）。

## 0. 记号

- P：前面的段全部持久后的镜像（含基镜像）；W：当前段的写；S ⊊ W：当前段落了的写；B(w)：写 w 罩住的字节范围；J(S)：一个状态的整份判定。
- 一个状态的镜像 = 基镜像 + 持久写按写表次序叠上去：`crates/singlefs-verification/src/memory_pool.rs:461` 「pub struct CrashImage<'base> {」。
- 三层「一样」沿用调查员的叫法：E1 红不红，E2 红项（`red_items()` 的串），E3 整份（含成立 / 不适用 / 部分没判与各种计数）。`red_items()` 只收 oracle 违例的种类、池级 checker 判到 Violated 的不变量编号、记录核对器两项（读：`crates/singlefs-verification/src/crash.rs:1151` 「pub fn red_items(&self) -> Vec<String> {」，`crates/singlefs-verification/src/crash.rs:1165-1174` 里 `NotApplicable`、`PartiallyJudged` 不进红项）。所以 E2 比的是不变量编号的集合，不是违例文字。

## 1. 问题 1：充分条件与判据

### 1.1 引擎怎么判一个状态（读）

`crates/singlefs-verification/src/crash.rs:1242` 「fn judge_crash_image_against_the_version_table(」只有这几样输入：镜像的读答案、`image.persisted`、写表。
- `crates/singlefs-verification/src/crash.rs:1247` 「let root_persisted = image.persisted[judged_root_index];」与 `crates/singlefs-verification/src/crash.rs:1248` 「let newest_persisted = newest_persisted_root(image.writes, &image.persisted);」：只随根槽写落没落变。单元段里没有根槽写（D1），这两样对一段内所有状态是常量。
- 两遍恢复、两份 oracle：只经 `PoolReader` 读（`crates/singlefs-core/src/recovery.rs:8258` 「pub fn recover(reader: &dyn PoolReader, policy: JournalPolicy) -> RecoveryReport {」），读到的位置由前面读到的字节决定。
- 池级 checker：`crates/singlefs-verification/src/crash.rs:1263` 「let pool_checker = check_pool_image(image);」，经 `ImageReader` 读，另外要候选槽表（`candidate_unit_slots`、`candidate_journal_slots`）。
- 记录核对器：`crates/singlefs-verification/src/crash.rs:1264` 「let records = check_records(image, consulted.effective_root);」，读镜像也读 `persisted`。
所以：判定是（读答案，持久向量，写表）的确定性函数。读答案在 S 与 S=∅ 上一致的位置，两边的执行逐步相同，这是下面所有论证的底。

### 1.2 五条引理（读）

引理一，被引用单元的读由校验和把门。走读往下读一个单元，两份位置条目各判一次 I-2.1，对得上才用：`crates/singlefs-checker/src/image.rs:2076` 「judgements.judge("I-2.1", matches, || {」。指针带提示时先按提示读、只读不判，要校验和与出生身份都对得上才用：`crates/singlefs-checker/src/walk.rs:1542` 「fn copy_at_a_location_hint(」，再不行回到映射位置读：`crates/singlefs-checker/src/walk.rs:2181` 「fn read_a_unit_to_walk_and_the_placement_it_was_read_from(」。后果：一个槽里的字节只经「校验和对得上没有」与「对得上时的内容」影响判定。P 上读到的是「没写过的零」（对不上）时，S 里某个写落在那个槽上，只要它不是指针要的那个单元，读出来仍是对不上，I-2.1 的红绿和后续路径都不变。例外只有一个：指针要的恰好就是 W 里的那个单元（指针先于单元落盘，悬空引用）。这种 P 在 S=∅ 就读到零、判红。

引理二，扫描方向只读候选槽，候选槽 = 基镜像写过的槽 ∪ 持久的单元写罩住的槽。读：`crates/singlefs-verification/src/memory_pool.rs:838` 「if *is_persisted && write.device.0 == device && write.kind == StepKind::UnitWrite {」。候选槽表的消费者只有三处：`crates/singlefs-checker/src/walk.rs:5547`（I-7.8 的树 ID 扫描）、`:5668`（I-7.7 的载体）、`:11423`（I-1.8、I-1.13 的已发布单元扫描）（读：`grep -rn candidate_unit_slots crates` 在 `crates/singlefs-checker/src/walk.rs` 只命中这三处用法，`crates/singlefs-core` 零命中）。恢复（core）不读候选槽表。GPU 判器的落点表同理：没有写碰过的单元槽读成全零（`crates/singlefs-verification/src/crash_judge_tables.rs:970` 「GPU 读没有落点的槽，结局与读全 0 相同」）。

引理三，扫描方向对一个候选槽的处理是逐槽的，跨槽的耦合只有三处，逐处读：
- 已发布准入：`crates/singlefs-checker/src/walk.rs:11252` 「if instance > self.mount_root_instance {」整支判未发布；`crates/singlefs-checker/src/walk.rs:11256` 「return birth_txg <= self.mount_root_checkpoint_txg;」，实例等于挂载根时要诞生代号不超挂载根 txg；实例更小的一支看实例表那一行（`crates/singlefs-checker/src/walk.rs:11245` 起的 `fn admits_the_header(`）。在飞单元的实例大于挂载根、或实例相等而诞生代号大于挂载根 txg 时不进 I-1.8、I-1.13。
- I-7.8 的认领：`crates/singlefs-checker/src/walk.rs:5560` 「if slot < first_slot_not_claimed {」与 `crates/singlefs-checker/src/walk.rs:5569` 「first_slot_not_claimed = slot + claimed_slots;」，槽号升序，一个头认领它声明的跨度（`crates/singlefs-checker/src/walk.rs:5510` 「fn slots_claimed_by_a_unit_header(」：码 1、码 3 两槽，码 2 一槽，声明来自头里的类标签，不来自写长）。再要诞生代号不超已发布的最新根（`crates/singlefs-checker/src/walk.rs:5577` 「if birth_txg <= newest_published_txg」）。
- I-1.8 成组：`crates/singlefs-checker/src/walk.rs:11466` 「let mut groups: BTreeMap<&Vec<u8>, Vec<&ScannedContentUnit>> = BTreeMap::new();」，`crates/singlefs-checker/src/walk.rs:11472` 「if members.len() < 2 {」，成员少于两份的组不判；两盘各一份的副本只有两份都落（且都准入）才成组，然后比载荷：`crates/singlefs-checker/src/walk.rs:11480` 「let payloads_are_the_same = every_member_carries_the_same_payload_checksum || {」。
- I-7.7 载体：`crates/singlefs-checker/src/walk.rs:5739` 「let above_every_disk = carriers」，`crates/singlefs-checker/src/walk.rs:5749` 「let witnessed_by_one_disk = carriers」，对每个载体逐个比门槛，是「存在一个超过就红」的或，没有成组、没有准入过滤。

引理四，记录核对器只罩落到的那一版及更早的发布：`crates/singlefs-verification/src/crash.rs:136` 「if effective_root.is_some_and(|(_, txg)| txg.0 >= publish.checkpoint_txg)」；单元段里在飞的发布 txg 比落到的那一版大，它自己的单元不判。更早发布的副本组（按偏移分组，两盘同偏移一组）全缺才红：`crates/singlefs-verification/src/crash.rs:146` 「if copies_by_offset」接 `crates/singlefs-verification/src/crash.rs:148` 「.any(|copies| copies.iter().all(|copy| copy_is_missing(*copy)))」。一份副本缺不缺看它的扇区上有没有持久、过了回收谓词的更晚的写：`crates/singlefs-verification/src/crash.rs:211` 「let explained_by_a_persisted_legal_later_write =」。W 的写不碰更早发布的任何副本扇区时，这一格对 S 恒与 S=∅ 相同。

引理五，里程碑 17 之前恢复与分配都由记录驱动，不盲扫单元区：区域扫描「已释放或没有记录的槽跳过」（`crates/singlefs-core/src/region_scan.rs:2`）；checker 里按单元区范围过滤槽号的两处（调查员「认不准」的 `crates/singlefs-checker/src/walk.rs:20575`、`:20625`）读的是最新根的分配记录行，不是单元字节（读：`crates/singlefs-checker/src/walk.rs:20526-20530` 的文档「从记录现扫」与 `:20575` 的 `.filter(|slot| (unit_area_start_slot..device_end_slot).contains(slot))` 作用在 `occupied_slots` 上，不读 S 的字节）。

### 1.3 判据 D（单元段可以逐写判的充分条件，写成引擎读段表的形式）

输入：枚举用的写表 `writes`（种类、盘、偏移、内容字节）、基镜像 `base`、段表 `segments`、段号 k。输出：`Independent`（交回要单判的写）或 `Fallback(原因)`（这一段照旧全组合或局部组合）。D1 到 D4 只读写表与基镜像；D5 要 S=∅ 那一遍的读迹。

| 号 | 条件 | 依据（文件:行） | 要读的 |
|---|---|---|---|
| D1 | 段里每个写的种类都是 `UnitWrite` | 单元写恒两态：`crates/singlefs-verification/src/crash.rs:2190` 「if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {」；段内混了根槽写、journal 记录、系统配置槽、清零、discard 的段不归这一判据（它们不是两态、或让「落到哪一版」随 S 变） | 写表 |
| D2 | 每个写 w：同盘上写表里下标更小的写（含同段更早的）没有一个与 B(w) 重叠，基镜像在 B(w) 上没有非零字节 | 与撕裂判据同形的两条现成代码：`crates/singlefs-verification/src/crash.rs:2194` 「let base_holds_nonzero_bytes_there」与 `crates/singlefs-verification/src/crash.rs:2200` 「let an_earlier_write_with_bytes_overlaps = writes[..write_index].iter().any(|earlier| {」；同段更早的写也算，所以 D2 蕴含段内写两两不重叠。探针的 `over_an_earlier_write`、`over_nonzero_base` 就是这两条的计数（量：删除流单元段都是 0） | 写表、基镜像 |
| D3 | 每个写 w：偏移按槽对齐；头解得出（`slots_claimed_by_a_unit_header` 同一份解析）时，声明跨度 × 槽宽 ≤ 写长；解不出头的写记「无头」 | 认领按头声明的跨度往后跳：`crates/singlefs-checker/src/walk.rs:5569` 「first_slot_not_claimed = slot + claimed_slots;」，声明不超过写长就不会罩到别的写的槽；超过了才跨写耦合（调查员 b3 的真正前提） | 写表字节 |
| D4 | 每个解得出头的写 w：头里的 (实例, 诞生代号) 在 P 的挂载根下判成「未发布」，即实例 > 挂载根实例，或实例 = 挂载根实例且诞生代号 > 挂载根 txg；落到实例更小那一支（要看实例表行）的，判据拒绝 | `crates/singlefs-checker/src/walk.rs:11252`、`:11256`；挂载根取写表里本段之前最后一条 `RootRecordFua` 写写下的根，没有就取基镜像的根（`crates/singlefs-verification/src/memory_pool.rs:1100` 「pub fn publishes_in(」里有现成的根身份取法） | 写表字节 |
| D5 | S=∅ 那一遍判定（恢复两遍、两份 oracle、池级 checker、记录核对器）读过的扇区集合，与每个 B(w) 都不相交（按扇区） | 引理一、四的悬空引用例外与 D2 管不到的读：P 若红于悬空指针，会读到 W 要写的槽，写落下就改了校验和结局 | S=∅ 的读迹（只记读过哪几个扇区，不看判定结果） |

退回规则：任一条不成立，这一段照旧（小段全量；写数大的段取调查员 b1 到 b4 的局部组合，见第 3 节）；D3、D4 只要有一个写不过，就只把那个写（连同第 3 节点名的相关写）单拎出来做局部组合，其余写仍可按 D 判（判据按写算，不按段算）。
D5 的读迹怎么取：`crates/` 里没有对 `CrashImage` 的记读包装（读：库代码里 `RecordingBlockDevice` 记的是块设备读，用在观察者上，`crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:339` 「let stream = SharedStream::retaining_contents_and_reads();」），要加；实验代码里有现成的记读读者可参照（`crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs` 的 `RecordingReader`、`crates/singlefs-verification/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs` 的 `ReadRecordingPoolReader`），只是实验装置，不是库里的件。没有读迹时可以退一步：要求 S=∅ 判绿（悬空引用必定判红），但那是读判定结果，与「读段表」的提法不同。

挂在实现上的位置（改法，读）：
- 与撕裂判据同形的一份「每写一格」的表加在 `crates/singlefs-verification/src/crash.rs:2107` 「pub fn of(base: &MemoryPool, writes: &[RetainedWrite]) -> Self {」旁（`TearableInPlaceOverwrites::of` 就是 (base, writes) → 每写一个 bool）。
- 展开方式加在 `crates/singlefs-verification/src/crash.rs:1937` 那个枚举（`CrashPointReplaySegmentExpansion`，现有 `NotExpanded`、`EveryProperSubset`、甲二）里，甲二的注释自己写了它「替不了全量展开」；新判据与甲二方向相反：甲二把所有单元写并成全落或全不落，D 只把过了 D2 到 D5 的写拿出去、其余照旧全组合。
- 选择点现在是 `crates/singlefs-verification/src/crash_amplification.rs:270` 「pub fn expansion_of(self, segment: &[usize]) -> CrashPointReplaySegmentExpansion {」，它只看段的写数，看不到写表与基镜像；`crates/singlefs-verification/src/crash_facts.rs:294` 「pub fn in_place_and_copy_on_write_writes_of_the_segment(」已经把单元写与别的写分开。D 要写表与基镜像，选择点得改成在建计划处（那里有 base 与 writes）算。
- `crates/` 里没有「逐写独立」的任何一块：`grep -rnE 'independent_unit|IndependentWrites|virgin|逐写独立' --include=*.rs crates` 零命中（读）。

### 1.4 为什么 D1 到 D5 够用（推导，骨架）

设 D1 到 D5 对段 k 的所有 w 成立。取任意 S ⊆ W，比较 J(S) 与 J(∅)：
1. 非扫描部分：D5 说 J(∅) 读过的扇区与每个 B(w) 不交，所以 I(S) 与 P 在这些扇区上逐字节相同；判定是确定性的（引理一的读由校验和把门，路径只随读到的字节走），所以两边的执行逐步相同：恢复两遍、两份 oracle、走读、实例表链都读到同样的东西、得同样的结论。
2. 常量部分：D1 保证 `root_persisted`、`newest_persisted` 不随 S 变（`crates/singlefs-verification/src/crash.rs:1247`、`:1248`）；引理四保证记录核对器不变（D2 不碰更早的副本扇区，在飞发布 txg 大于落到的那一版）。
3. 扫描部分（引理二、三）：S 里每个写给候选槽表加上它罩住的槽。逐写看这些新槽上的三处判法：
   - I-1.8、I-1.13：D4 说准入判未发布，不进扫描项、不成组，判过次数也不加；
   - I-7.8：D4 的诞生代号 > 最新已发布根的诞生代号，被 `crates/singlefs-checker/src/walk.rs:5577` 的过滤挡掉；D3 说认领跨度不超过写长，D2 说别的写不在这个跨度里，认领不跨写；
   - I-7.7：每个载体逐个比门槛，是或。S 里的写各自贡献「有没有一个载体超门槛」，并集等于 S 的结论。
4. 所以 J(S) = J(∅) 加上每个写各自带来的、只随该写字节变的那几项（只可能出在 I-7.7 这一类逐载体判法上），而这些项的合成方式是或 / 集合并。逐写判（S=∅ 与每个单写）能还原任意 S 的 E1、E2；E3 里的次数类要另算（见第 2 节）。

删除流上这条推导的预言与量对得上：D2 的计数是 0（量：段 12、16、20，`over_an_earlier_write=0`、`over_nonzero_base=0`），预言「S 与 S=∅ 整份相同」，量到 `full_differs=0`（量：单写、成对、只差一个、随机 2000 个子集）。

### 1.5 共用问句的回答

在判据 D 判为可以逐写判的段上，有没有一个具体状态，它的判定与它被代表的那个状态不同：我没有构造出来。下面是我试过的反例与它们为什么不成立，以及判据管不到的两处。
- 试一：C561 的 σ 段（`crates/singlefs-verification/tests/record_checker_judges_absence_by_the_persisted_set.rs:9` 「2 个原地写 + 16 个单元写」那一段）里「只落 y（两盘）」的状态 X。X 的判定（记录核对器假红 / 假绿）取决于 y 与 u2、l 的旧扇区的关系，与别的写落没落无关，所以逐写判单拎 y 也许就够；但 y 盖在更早发布的节点 u2 与数据单元 l 的扇区上，D2 不成立，判据把这一段判为不可逐写判，不在问句范围里。
- 试二：两个相邻槽的单元写，前一个的头声明两槽（码 1 数据单元）、后一个紧跟在第二槽上。写长是 32K 时后一个必然落在第三槽起，D2 蕴含不重叠、D3 蕴含不被认领；写长只有 16K 而头声明两槽时 D3 拒绝。
- 试三：在飞单元的头判成已发布（实例更小、实例表无行）。D4 拒绝；这一支在本仓的在飞单元上没有量到（探针只在三个段上量，未覆盖续做之后换了实例的段，见第 5 节）。
- 试四：P 红于悬空指针，W 里恰好有它要的单元。S=∅ 判红，S ∋ 该写判绿，且红项不同。D5 拒绝。若只实现 D1 到 D4 而不做 D5，这是一个活的反例，也就是说 D5 不能省。
- 判据管不到的一：观察者对 S 的谓词（第 4 节）与 E3 里的各种次数（`crates/singlefs-checker/src/walk.rs:21378` 「if judge_merged_version_total_order(reader, &content_units, &mut judgements) == 0 {」只改成立 / 不适用的区分，不改红项，要次数的话逐写累加）。
- 判据管不到的二：D4 用头字节判准入，头字节若被实现写错（诞生代号写成不超挂载根的值），在飞单元就变成已准入——此时 D4 拒绝，不会逐写判；但这也说明「未发布」那一条是靠实现写对头才成立的前提，不是被检出来的性质。

什么现象会推翻第 1 节：任一个满足 D1 到 D5 的段上，有一个状态 S 的 `red_items()` 或整份签名与「S=∅ ∪ 各单写」的合成不同（探针里的 `full_differs`、`PROBE_PAIRS_NOT_UNION` 非零，且那个写通过了 D2 到 D4 的静态判）。

## 2. 问题 2：候选甲、乙在「红不红」「红项」两层上等不等价

定义（按正文第一节）：
- 甲：判 S=∅ 与每个单写，S 的结论由 red(S) = red(∅) ∪ 并_{w∈S} red({w}) 还原（红不红就是这个集合是否非空）。态数 n+1。
- 乙：同一单元的几份副本算一组（分组键与记录核对器同一个：同偏移，`crates/singlefs-verification/src/crash.rs:146` 「if copies_by_offset」所在循环按偏移把两盘的副本归一组），组内取全部非空子集，组间各判一次，red(S) = red(∅) ∪ 并_g red(S∩G_g)。每组两份副本时态数 1 + 3·(n/2)。

「判据判为可以逐写判」= 段满足第 1 节 D1 到 D5。

### 2.1 在判据内（D1 到 D5 全成立）：两层、两候选都与全组合等价

- 红不红（E1）：第 1.4 节第 3 步说明 S 对 J 的影响只来自逐载体的或（I-7.7）与不进集合的写（I-1.8、I-1.13、I-7.8），红不红是「∅ 红，或某个单写红」。甲还原出的就是它；乙是甲的超集（乙的组内状态含两份单写），还原出同一个集合。
- 红项（E2）：红项是不变量编号 / oracle 种类的集合（`crates/singlefs-verification/src/crash.rs:1151` 起的 `red_items`），各写贡献的项合成是集合并，两候选都还原得出。
- 量的印证（正文第三节）：删除流段 12、16、20，单写、成对、只差一个、随机 2000 个子集，对 S=∅ 比，`red=0 red_differs=0 red_items_differ=0 full_differs=0`，`PROBE_PAIRS_NOT_UNION pairs=0`。这说明在这三段里 J(S) 整份等于 J(∅)，连逐写判的「单写」都没有带来任何差：此处甲、乙都与只判 S=∅ 一个状态等价。这是三个段的量，不是全称（第 5 节）。
- 态数（算的，不是量的，沿用调查员第四节）：段 12、16、20 甲 29、31、27，乙 43、46、40。判据内两者结论相同，乙多出的态没有额外信息，所以判据内选甲；乙的价值全在判据外。

### 2.2 在判据外：两层分开，甲与乙开始不同

红不红（E1）：
- 甲保 E1 的前提是「每个单写自己就红」。读代码：被引用单元只要有一份副本被盖，I-2.1 就单独红（`crates/singlefs-checker/src/image.rs:2076` 「judgements.judge("I-2.1", matches, || {」逐份判），量也印证：只盖一份时也红（I-2.1，正文第三节阳性对照）。所以阳性对照里甲的 E1 与全组合相同（单写已红）。
- 但这个相等是靠池级 checker 对单写自己的冗余得来的，不是靠记录核对器的结构：记录核对器判「同组副本全缺才红」，写成代码是 `crates/singlefs-verification/src/crash.rs:148` 「.any(|copies| copies.iter().all(|copy| copy_is_missing(*copy)))」，是一个与：单写不红、两份都缺才红。记录核对器是当作独立 oracle设的（D13 已定项 7，`13-验证路线.md:203`），目的就是抓池级 checker 抓不到的，所以「池级 checker 恰好对单写也红」不能当结构性保证。哪一个流上两者分歧，我没有现场观测，推的。
- 成组的另一处是 I-1.8：两份副本都落（都准入）才成组，才判载荷一致（`crates/singlefs-checker/src/walk.rs:11472` 「if members.len() < 2 {」、`crates/singlefs-checker/src/walk.rs:11480`）；两份载荷不同的实现错误，单写不红、成对才红。这时甲的 E1 与全组合不同，乙保得住。条件是副本准入，即 D4 不成立，在飞单元在本仓流上没有量到准入（推）。
- 结论：判据外，乙的 E1 ⊇ 甲的 E1，甲在记录核对器的与、I-1.8 的成组两处有可构造的漏。

红项（E2）：
- 量：阳性对照两份都盖时红项多出 `journal_*_oracle:read_failed` 等，`PROBE_PAIRS_NOT_UNION pairs=1`（正文第三节），即两份都盖的红项不等于两个单写红项的并。甲还原出的是单写红项的并，少了走读断带出来的项。乙把「只落一份 ×2、两份都落」三个状态都判了，该组的红项精确。
- 同一阳性对照里，段 16 其余 28 个写与这一对一起、两两比，只有这一对不是并（`pairs=1`，段 16 的对数是 435，算的）：说明在这一段里，相互作用只在这一组的两份副本之间，组间没有。这是乙在这一个取样点上成立的量。
- 乙的红项在两种情形下也会不精确（推，读代码）：（a）两个组在引用关系上是祖先与后代，祖先组两份都坏时走读不往下读，后代组的违例被遮住：索引节点两份都读不到对得上的就记走读失败并交回不可走（`crates/singlefs-checker/src/walk.rs:2483` 「.push(format!("{what} 两份都读不到对得上的"));」，`crates/singlefs-checker/src/walk.rs:2484` 「return IndexNodeReading::NotWalkable;」），孩子不再读；记走读失败后 I-4.8、I-7.2、I-7.4 跟着红（`crates/singlefs-checker/src/walk.rs:3805` 「两份都对不上记走读失败（I-4.8、I-7.2、I-7.4 照旧跟着）」）；红项随祖先组的状态变，组间不独立，要把有引用关系的组并成一个大组做全组合；（b）一份副本被 k 个写分段盖住（32K 数据单元被两个 16K 写盖），k > 1，组要按「盖住同一份副本的写」算，不是按「一写一份」算。
- 两个候选在判据外都需要「相互有作用的写」的闭包：写集的并查集，边是共享被盖扇区、同组副本、祖先后代引用。引用关系读不出写表（要读 P 的指针），所以乙的分组只有「同偏移两份」这一层，祖先后代那一层要读 P 的结构，写表上没有（推）。

### 2.3 小结表（两层 × 两候选 × 判据内外）

| | 判据内 E1 | 判据内 E2 | 判据外 E1 | 判据外 E2 |
|---|---|---|---|---|
| 甲（n+1 态） | 等价 | 等价（红项并） | 仅当单写自己红时等价（I-2.1 冗余）；记录核对器的与、I-1.8 成组两处漏 | 不等价：量到 `pairs=1`，丢走读断带出的项 |
| 乙（1+3G 态） | 等价（甲的超集） | 等价 | 组内保住；组间只有「同偏移两份」一层 | 组内保住；祖先后代两组同时被盖会被遮，要并组 |

什么现象会推翻第 2 节：判据内的一个段上找到甲与乙还原出来的红项不同（说明合成不是并），或判据外找到一个组内以外的相互作用（说明乙的组划少了）。

## 3. 问题 3：六类破坏条件列全没有，局部组合的范围与态数

先给方法：判据 D 的五条里每一条被破坏，都对应「某个写的影响不再只随它自己的字节变」。把五条各自被破坏的方式列出来，与调查员的六类对。我把调查员的六类逐类对到代码，列出多出来的两类与一处改写，再给每类的局部组合与态数。

### 3.1 逐类对照（读 / 推 分开）

| 类 | 破坏的是 | 代码（读） | 局部组合的范围 | 态数 |
|---|---|---|---|---|
| b1 镜像双副本（同一被引用单元的两份都被盖） | D2、D5 | 每份副本各判一次 I-2.1：`crates/singlefs-checker/src/image.rs:2076`；两盘各一份：「按设备各算一次写」（`.claude/kb/decisions/13-验证路线.md:141`）；删除流意图段「13 个单元两盘各一份」（`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:249`） | 盖住同一单元两份副本的写；一份副本被 k 个写分段盖住时，按「盖住这一单元任一份副本的写」算 | 2 个写：2^2 − 1 = 3；一般 2^(k₀+k₁) − 1。量：阳性对照三个状态 |
| b2 记录核对器「同组副本全缺才红」加回收谓词 | D2 | 与：`crates/singlefs-verification/src/crash.rs:148`；一份缺不缺看有没有持久且合法的更晚的写解释那一扇区：`crates/singlefs-verification/src/crash.rs:211`；每个后写合法与否只看它之前的写表：`crates/singlefs-verification/src/crash.rs:295` 「for write in &writes[..later_index] {」，所以合法性是每写一个常量，不随 S 变 | 盖住同一发布同一偏移的各副本扇区的全部在飞写（被盖的副本、更晚的写同在这一组里） | 2^k − 1，k 为组内写数。量：C561 那一段 262144 个状态，旧判法 32768 个红（`.claude/kb/checks-owed.md:666` 「第 79 段 262144 个状态」「换回旧判法 32768 个红」）。32768 = 2^(18−3)，红条件只约束 3 个写（推），所以 k ≥ 3 |
| b3 I-7.8 认领跨相邻槽 | D3 | `crates/singlefs-checker/src/walk.rs:5560` 与 `crates/singlefs-checker/src/walk.rs:5569`，跨度由头里类标签定（`crates/singlefs-checker/src/walk.rs:5510`） | 头声明跨度大于写长、或认领范围罩住前缀已有候选槽的那个写，与被罩的写（或前缀槽）；改写见 3.2 | 一对：2^2 − 1 = 3；良构写（声明跨度 = 写长）不需要 |
| b4 同段重叠写 | D2 | 一个落点的字节由罩它的几个写共同定：`crates/singlefs-verification/src/crash_judge_tables.rs:56` 的 `MAXIMUM_WRITES_PER_LOCATION`（256）、`:64` 的 `MAXIMUM_VERSIONS_PER_LOCATION`（4096）说明建表时每落点要组合；单元段本身 D2 已蕴含不重叠 | 重叠写的连通分量 | 分量里 k 个写：2^k − 1；原地覆写取三态时 3^m · 2^(k−m) − 1（`crates/singlefs-verification/src/crash.rs:1932`） |
| b5 全局计数「判过 0 次报不适用」 | 不破坏 D，只是 E3 | `crates/singlefs-checker/src/walk.rs:21378`；`NotApplicable` 不进红项（`crates/singlefs-verification/src/crash.rs:1165`、`:1172`） | 无（红不红、红项不受影响；成立 / 不适用的区分与判过次数要逐写累加） | 不加态 |
| b6 根与单元同段 | D1 | 一个 FUA 写只放行它自己那块盘（「带 FUA 的写只放行它自己那块盘」，`.claude/kb/decisions/13-验证路线.md:141`），所以一块盘的根槽写与另一块盘上还没被屏障隔开的单元写可以同段（段的关法：`crates/singlefs-verification/src/segments.rs:124` 「impl SegmentClosingRule {」）；根落了，新根能走到的单元落没落都要判（推：本仓流上这样的段没有量到） | 根写落 / 不落 × 新根能走到的单元的副本组（每组按 b1） | 根 2 态 × 每组 3 态；E2 要这些组的全组合（父组坏了遮住子组，见第 2 节） |
| **b7（新）已准入单元的副本组** | D4 | I-1.8 成组：`crates/singlefs-checker/src/walk.rs:11472`；准入第三支，实例更小、实例表无行算已发布，有行时比诞生代号或事务号水位：`crates/singlefs-checker/src/walk.rs:11265` 「let Some(row) = row_of_the_instance else {」与 `crates/singlefs-checker/src/walk.rs:11271` 「birth_is_published || transaction <= row.applied_transaction_high_water_mark」；I-1.13 的中央映射按「扫到它的副本的落点」重建（`crates/singlefs-checker/invariants/01-块自描述.md:19`「扫到它的副本的落点」），只落一份副本时扫描方向的映射项只有一个落点（推：没有逐行追到 I-1.13 的比对代码） | 同一合并键的两份副本 | 每个单元 2^2 − 1 = 3；红只在两份都落（载荷不同）时才有，甲单写看不到 |
| **b8（新）单元写与系统配置槽（原地覆写）同段** | D1 | I-7.7 的门槛取各盘两槽里自证过的最大者再取最小，载体逐个比：`crates/singlefs-checker/src/walk.rs:5739`、`:5749`；系统配置槽写是原地覆写取三态；C577 之前上一次发布的系统配置槽轮换与单元写同段：`crates/singlefs-verification/tests/record_checker_judges_absence_by_the_persisted_set.rs:9` 「2 个原地写 + 16 个单元写」 | 系统配置槽写（盘数 m 个）的全部三态组合 × 每个载体写（∅ 或单写） | 约 3^m · (n + 1)（单元写之间仍独立）；m = 2、n = 16：9 × 17 = 153（算的，推）对 2^18 |

### 3.2 对调查员六类的改写与补充

- b3 写宽了。调查员写「盖住相邻两槽的写要一起看，2^2−1=3 态」。读认领代码：一个头认领它声明的跨度，下一个写从跨度之后起才是候选（`crates/singlefs-checker/src/walk.rs:5560`、`:5569`）；良构的单元（声明跨度 = 写长，码 1、码 3 两槽 = 32K，码 2 一槽 = 16K）与它后面相邻的写并不互相认领。相邻两槽「互相影响」要的是头声明与写长不符、或前缀里本来就有候选槽被罩，前者由 D3 判，后者就是 D2。所以 b3 的局部组合只在这两种情形下出现，不是「凡相邻都出」。
- 多出 b7、b8 两类（上表）。b7 是调查员 b5 附近漏的：b5 说「成组只影响判过次数」，那是对未准入的单元；对已准入的，成组与否改变 I-1.8 ① 红不红（载荷不同只有成对才红）。b8 是把 D1 破坏的方式只写了「根与单元同段」，没写系统配置槽与单元同段（它在 C577 之前是常态，C577 之后只在一块盘没放行时出现，推）。
- 候选槽表本身随 S 变（引理二）不是一类：它只是扫描方向的输入，影响经 b3、b7 以及逐载体的或体现。
- 观察者与续做写的相互作用，不是判定的破坏类，放在第 4 节。

### 3.3 删除流今天的实际分布（量 + 读）

- 单元段 12、16、20 与其余单元段：`over_an_earlier_write=0`、`over_nonzero_base=0`（量），所以 b1、b2、b4 在这些段上的局部组合是 0；b3 靠 D3、b7 靠 D4（读头字节，没有量）；这几类全部为空的前提是 D3、D4 在这些段上成立，本轮没有谁去判（见第 5 节）。
- 系统配置槽段 `over_an_earlier_write=2`（量，正文第三节）：这些段不是单元段，D1 拒绝，照旧：取三态的 2 个写，3^2 − 1 = 8 态（算的：`crates/singlefs-verification/src/crash.rs:1932` 的闭式；调查员写明没有逐写核，推）。
- 对拍三条流里「单元段 `over_an_earlier_write=0`、系统配置槽段 `=2`」（量，任务 8322），同上。

什么现象会推翻第 3 节：（a）某个满足 D1 到 D5 的段上，探针量到 `PROBE_PAIR_INTERACTIONS` 非零，说明还有一类；（b）D3 通过而相邻写的认领仍然影响 I-7.8，说明 b3 的改写错了；（c）已准入的在飞单元在本仓某条流上真的出现（b7 变成实的）。

## 4. 问题 4（正文第四节第 4 条）：观察者在逐写判下怎么判

先把观察者按「读什么、记什么」分成三类（读代码），再逐类答。

| 类 | 例 | 读什么 | 去不去重 | 代码 |
|---|---|---|---|---|
| 只记计数、可以读持久集合的结构 | 发布边界观察者、删除停在中间观察者 | 持久集合；断言里读恢复报告 | 逐状态看，不去重：`crates/singlefs-verification/src/crash_amplification.rs:20174` 「逐状态看，不去重（它可以读持久集合的结构）」 | `crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:157` 「if !publish.is_cut_before_its_last_record(&image.persisted) {」；`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:367` 「if !publish.is_cut_after_some_unit_and_before_its_record(&image.persisted) {」 |
| 交回镜像（可写挂载续做） | 续做观察者，挂在整条流与标准前缀三步上 | 在镜像上真跑可写挂载，经录制设备记下读过的段 | 按恢复报告分桶、按读集见证去重：`crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:209` 「deduplication: ObserverDeduplication::ByTheRecoveryReportAndTheReadSet,」 | `crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:339` 「let stream = SharedStream::retaining_contents_and_reads();」，交回读集 `crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:422` 「ranges_read_on_the_crash_image: ReadRangeOnTheCrashImage::of_the_recorded_read_ranges(」 |
| 只记计数、读恢复落到的那一版（快照、整理那几步的观察者） | 登记在 `crates/singlefs-verification/tests/common_crash_points/mod.rs:297` 起的那几张表里 | 恢复报告落到的版本上的树（按指针读，引理一） | 同第一类，逐状态不去重 | 没有逐个读（推：只读了登记表与文档，没有追进它们的实现） |

### 4.1 只记计数的观察者（第一类）

- 它们的判定有两部分：激活谓词读 S（`is_cut_after_some_unit_and_before_its_record` 在 `crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:110`：单元写至少落一个、记录一份都没落、根没落），断言读恢复报告。
- 判据 D 成立时恢复报告随 S 是常量（第 1.4 节第 1 步），所以断言在 S=∅ 与任一单写上各判一次就覆盖全部 S；但谓词随 S 变，不能用代表去算它的计数。这个观察者的计数要按组合算：段 20 全部 S ≠ ∅ 的状态数是 2^n − 1（n = 26，再加下一段的 S=∅ 那个全落状态），量到 67,108,863 = 2^26 − 1（量，正文第三节没写这个数，出自调查员事实表 E1，我没有重量）。也就是说逐写判不能「少判一个计数」：计数得有闭式或逐状态的位运算。逐状态只做谓词、不做判定很便宜（对掩码做位运算，不判状态），但 `crates/` 里现在谓词与断言写在同一个函数里，没有拆开（读：上面两处函数体），要拆。
- 能不能只在 S=∅ 与单写上跑观察者：谓词的取值类只有「S 与那次发布的单元写集交集是否为空」两类，S=∅ 与任一单写各占一类，所以代表选 {∅} ∪ 单写就够；要保计数时，按类加权（空类 1 个状态，非空类 2^n − 2 个），不能按代表一个一个数。
- 第一类里没有读读集见证的东西，D5 对它们无法机械判：它们读镜像是随便读的（没经录制设备）。所以对第一类，D5 做不了，只能保守地对每个单写都真跑一遍（n + 1 次，不是 1 次）。

### 4.2 续做观察者（第二类）：行为与挂完的盘要分开

- 行为（挂载发的写、恢复报告、计数）：它只随「读到的字节、恢复报告、另读的输入」变（文档：`crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:326` 起「它交回的只随镜像字节、恢复报告与另读的输入变，录入段按恢复镜像去重」），所以它已经是读集判据：读集见证相同的状态取同一次运行的结果。D5 对续做观察者因此可以机械判：S=∅ 那一次交回的读集与每个 B(w) 不相交即可，这是现成的件（`ObservedReadSet`，`crates/singlefs-verification/src/crash_amplification.rs:21940` 附近用它算见证），不用新加。
- 挂完的盘不能取代表：`crates/singlefs-verification/src/crash_amplification.rs:22588` 「fn pool_after_the_observer(」里 `crates/singlefs-verification/src/crash_amplification.rs:22594` 「pool.apply_writes(&image.image.writes_on_the_crash_image);」是「这个状态的持久写叠基底，再叠观察者发的写」，建盘时各状态各用自己的持久集合（`crates/singlefs-verification/src/crash_amplification.rs:21963` 「rebuilt.apply_writes(&image_after.writes_on_the_crash_image);」），池级 checker 对每个不同的镜像判一次。观察者发的写 C 在 D5 成立时对所有 S 相同，但在飞的垃圾 w 还留在 C 之下或之外：挂完的盘 = P ⊕ C ⊕ （w 被 C 盖掉后剩下的部分）。
- 所以 D 要对挂完的盘再判一遍（判据对象换成：写表 ∪ C，w 的有效范围换成 B(w) 去掉被 C 盖住的部分）。D1 到 D3 仍读写表；D4 的第三支在这里会走到：续做取了新实例、挂载写行与暖机各发布一次，根的实例是新实例，比在飞单元的大（推），落到实例更小那一支，要看实例表行（`crates/singlefs-checker/src/walk.rs:11265`、`:11271`），行在 P ⊕ C 的字节里，不在写表里。`admits_the_header` 是 checker 里的私有函数（读：`crates/singlefs-checker/src/walk.rs:11245` 「fn admits_the_header(」没有 pub），crates 里没有「报出扫描收进了哪几个头」的接口。这是判据 D 里唯一必须读 P ⊕ C 结构的一条。
- 两个出路：（一）给 checker 加一个只读的出口，报扫描方向收进了哪些槽，D4 在单写的挂完的盘上动态判，收进任何一个在飞的头就退回；（二）续做之后挂完的盘不取代表，逐状态判（今天就是这样，不省）。
- 续做写 C 与 w 部分重叠（C 的 16K 写盖在 w 的 32K 数据单元的第一槽上，第二槽的残余留着）：C 在 D5 成立时对所有 S 相同，残余只随 w 变，仍是逐写的；{w} 单独的挂完的盘就带着这个残余，单写判得到。不成立的只有 C 写到半个槽或一个槽里的一部分时（读：这时 D 的 D3 对 C 的写同样要判）。
- 恢复路径（core）一侧：续做的分配器取空闲槽靠分配记录，不靠扫字节（`crates/singlefs-core/src/region_scan.rs:2` 「已释放或没有记录的槽跳过」是整理那一支；分配一支我没有逐行读，推）。

### 4.3 几句要写明的保留

- 调查员第二节的「可写挂载续做观察者按镜像去重所以并不能把单元段里的组合并掉」是对的，但原因要更精确：去重键是读集见证（行为部分已按读集去重），而挂完的盘的 checker 仍按镜像各判一次，那一半才是并不掉的。
- 续做观察者本轮没有放进探针（正文第三节只量了 CPU 判器），所以第 4 节的结论全是读代码加推，没有量。

什么现象会推翻第 4 节：（a）续做观察者在某个满足 D 的段上，S=∅ 与某个单写的读集（`ranges_read_on_the_crash_image`）出现与 B(w) 的交；（b）挂完的盘上，某个状态 S 的 checker 红项不等于 S=∅ 与各单写在挂完的盘上红项的并；（c）第一类里有一个观察者的断言读了持久集合以外随 S 变的东西（不止谓词）。

## 5. 问题 5（正文第四节第 5 条）：取样点够不够

### 5.1 够支撑的与不够支撑的

量到的（正文第三节）：删除流段 12（28 写）、16（30 写）、20（26 写），都是 `UnitWrite`、`over_an_earlier_write=0`、`over_nonzero_base=0`；单写、成对、只差一个、随机 2000 个子集与 S=∅ 比，整份相同；一个阳性对照（把一个能到达的单元两盘副本都盖掉），探针分得出差别（红、红项差、`pairs=1`）。

够支撑的只有这一句：「在这三个段上，J(S) 整份等于 J(∅)，逐写判与全组合等价」。对「所有满足判据的段」的全称，不够。理由逐条（都是读探针源码 调查员仓副本里的 `/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/repo/crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs` 第 1972 行 起的 `mod per_write_judging_probe` 加正文第三节读出来的，不是量）：
- 探针判的是 `judge_crash_image`（`crates/singlefs-verification/src/crash.rs:1242` 那一层），没有挂任何观察者，没有跑挂完的盘上的池级 checker；`full` 签名是 `consulted`、`ignored`、两份 oracle、`pool_checker`、`records` 的 Debug 串（读：探针 `signature_of`）。第 4 节的结论一个取样点也没有。
- 三个段的样子太像：同一条流、同一个 mkfs 几何、同一种「新建、没有旧内容可盖」的写。判据的反面（D2 拒绝的段）一个都没取，所以没有一格证明判据会拒绝该拒的段。
- 随机 2000 个子集是均匀取的，对「写 a 与写 b 同在时才显出」这类结构性相互作用没有力度；成对是穷举的，覆盖了二阶相互作用，三阶及以上只有随机与只差一个在碰。判据内的结构推出没有三阶作用（第 1.4 节），判据外有（C561 红条件约束 3 个写，第 3 节）。

### 5.2 还缺的敏感取样点（每一格说明它推翻哪一句话）

| 取样点 | 为什么敏感 | 推翻什么 | 现成的件 |
|---|---|---|---|
| σ 段（C561 复用链，16 个单元写，写盖在更早发布的 u2、l 的扇区上） | D2 的反面：判据应当拒绝；逐写判单拎 y 会漏「y 与 l 同在 / y 与 l 不在」这类取决于复用合法性与副本全缺的状态 | D2 的必要性；b2 的局部组合态数（k ≥ 3） | `crates/singlefs-verification/tests/record_checker_judges_absence_by_the_persisted_set.rs` 的 σ 全量（262144 → C577 之后 2^16 个状态，旧判法 32768 个红，`.claude/kb/checks-owed.md:666`）；缺的是一个「判据判这一段」的探针 |
| 已准入的在飞头（D4 第三支）：续做之后挂完的盘上，在飞单元的实例比挂载根小、实例表有行或无行 | D4 的反面，也是第 4 节的核心；只有两份副本都落才显出（I-1.8 ①），单写看不到 | D4 的充分性；b7 的存在 | 标准前缀三步上的续做观察者；缺的是在挂完的盘上取单写与成对的探针 |
| 混合种类的段（单元写与系统配置槽写、根槽写同段） | D1 的反面；b6、b8 | D1；b8 的 3^m·(n+1) | 对拍三条流的段表（任务 8322）只列了写数与「盖旧内容的写数」，没有列种类串；要加 |
| 多组同时被盖、且有引用关系（祖先组与后代组） | 乙的组间独立这一句的反面（第 2.2 节） | 乙的组间独立 | 阳性对照改成盖两层：把 B 的上层节点与下层节点的两份副本都盖 |
| 写长与头声明不符的单元（16K 写、头声明两槽）紧挨另一个写 | D3 的反面；I-7.8 认领跨写 | D3、b3 的改写 | 探针里造；`slots_claimed_by_a_unit_header` 是私有函数，造输入时按头字节写 |
| 整理（GC）流、快照流的单元段（单元落在释放后复用的槽上） | 这些流里单元写盖旧内容是常态，D2 应当大面积拒绝；检验判据不会把它们放过去 | D2 | 登记在 `crates/singlefs-verification/tests/common_crash_points/mod.rs:297` 起的快照、整理几步；要先看每段的 `over_an_earlier_write` |
| 小段全量对照：对每个满足 D 且 n ≤ 16 的段，全组合（≤ 65,535 个状态）逐状态与「S=∅ ∪ 各单写」的合成比 | 便宜、穷举、不依赖随机；是全称命题在小规模上的真检验 | 第 1.4 节的合成式 | 引擎本来就把写数小于 10 的段全量展开（197 个状态那一域）；这些状态的判定库结论已经有，缺的是比较程序 |
| GPU 判器一侧：同一批取样点 | 判器输入把没有落点的槽当全零（`crates/singlefs-verification/src/crash_judge_tables.rs:970`），与 D2 的前提一致，但探针是 CPU 一侧；GPU 与 CPU 对拍比的是红位、违例位、恢复字（调查员第三节） | 逐写判对 GPU 对拍这一半是否同样成立 | `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 目标，探针只录了段表 |

### 5.3 一条「怎么读这次的 0」

探针里 `full_differs=0` 只有在阳性对照红了的前提下才算数（调查员预言表 P2 写了这条，阳性对照也红了，`PROBE_PLANTED_STATE … red_differs=true`）。所以这三个段的 0 是可信的 0，但它是「这三个段、这组子集」的 0。

## 要主 agent 跑的命令

本轮没有写模型，没有起任何脚本。下面一条只是核第 2、3 节里引的记录核对器几格（同偏移副本全缺才红、按扇区判缺席）今天是否还成立，现成测试，不是我新写的装置。

1. 命令（在仓根起）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 record_checker_judges_absence_by_the_persisted_set 里 C561 四格; 范围=4 个测试; 为什么=核逐写独立判第二、三节引的记录核对器按扇区、同偏移副本全缺才红的几格今天是否成立' task-server/task submit -- cargo test -p singlefs-verification --test record_checker_judges_absence_by_the_persisted_set -- --exact the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing a_data_unit_only_half_covered_by_a_landed_later_node_is_missing a_node_whose_every_later_write_to_the_same_offset_never_landed_is_missing partial_landing_of_the_two_later_nodes_never_reports_a_unit_missing`
   起在：`/home/fy5090/code/singlefs`。
   看：`TASK_SERVER_RESULT` 那行 `type=succeeded exit=0`，`task-server/task log <号>` 里 `test result: ok. 4 passed; 0 failed`。

要新派的活（写探针，不是命令）：第 5.2 节表里的 σ 段判据判定、续做之后挂完的盘上的准入、混合种类段的段表、两层相关组的阳性对照。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1 条件（D1 到 D5）每条指到文件:行 | 推出 | 五条里 D1 到 D4 读写表与基镜像就能判，D5 要 S=∅ 的读迹（读迹包装 crates 里没有，实验装置里有） |
| 1 判据可实现形式 | 给出 | 加在 `crates/singlefs-verification/src/crash.rs:2107` 旁与 `CrashPointReplaySegmentExpansion`；选择点 `crates/singlefs-verification/src/crash_amplification.rs:270` 看不到写表与基镜像，要改到建计划处 |
| 1 共用问句 | 没构造出反例 | 试了 C561 的 σ、相邻认领、已准入头、悬空指针；悬空指针例子在没有 D5 时是活反例，D5 不能省 |
| 2 判据内 E1、E2，甲与乙 | 一致 | 两者都与全组合等价；判据内甲更省（n+1 对 1+3G） |
| 2 判据外 E1 | 不一致 | 甲在记录核对器的「与」与 I-1.8 成组两处会漏，乙保组内 |
| 2 判据外 E2 | 不一致 | 量：`pairs=1`，甲丢走读断带出的项；乙组内精确，祖先后代同被盖时也不精确（推） |
| 3 六类列全 | 没列全 | 多出 b7（已准入单元的副本组）、b8（单元写与系统配置槽同段）；b3 条件写宽了 |
| 3 各类态数 | 给出 | b1 3、b2 2^k−1（C561 约束 3 个写，推）、b3 3 且只在不良构时、b4 2^k−1、b5 不加、b6 根 2 × 组 3、b7 3、b8 约 3^m·(n+1) |
| 4 观察者 | 分三类答 | 第一类谓词要加权算、不能取代表；续做观察者行为按读集、挂完的盘逐状态；D4 第三支在挂完的盘上要读 P ⊕ C 的结构 |
| 5 取样点 | 不够全称 | 三段加一个阳性对照只支撑「这三段」；缺 8 类敏感点（5.2 表） |
| 调查员「认不准」里 `crates/singlefs-checker/src/walk.rs:20575`、`:20625` | 已核 | 读分配记录行不读单元字节，不是扫描方向 |
| 调查员 b3「相邻两槽 3 态」 | 改写 | 良构单元（跨度 = 写长）互不认领 |

## 没做什么

- 不判别的腿的格：不替主 agent 采纳、不出判决；没往统一任务服务交任务（只在上面写了一条命令）；没写模型、没建仓副本、没起脚本。
- 判决要引的数没有复跑：正文第三节的量（任务 8321、8322 的读数）我没有重读日志，原样引用；`full_differs`、`pairs` 等数一律是主 agent 的量。第 2 节的 `pairs=1` 对应的对数 435 是我算的（C(30,2)），不是量。
- 没有读 `crates/singlefs-core` 的恢复、分配、挂载的全部代码：第 1 节引理一的「恢复一侧读由校验和把门」只读了 `recover` 的头与 checker 一侧的读法，core 走读是否也一样我没有逐行核（调查员也没读），推的。
- 没有逐条追 114 条不变量到判法：本报告用读集论证绕开了「按读几个单元分类」；这条论证成立要靠引理一到五是全部的读路径，这一点靠 `grep candidate_unit_slots` 只有三处用法与 `PoolReader` / `ImageReader` 的实现只有 `CrashImage` 与基镜像两种这两处读得，其余的读路径（例如 checker 某条不变量自己直接读盘）没有逐条查，推的。
- 没有碰别的腿这一轮的提示与产出（禁读清单）。
