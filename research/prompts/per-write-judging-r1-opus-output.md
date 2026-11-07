# 逐写独立判第一轮攻方腿（opus，2026-11-06）

轮名 per-write-judging-r1；攻击面：①「对所有满足判据的段等价」是不是外推过头；② 判据与「被选中的根只能是上一版」「在飞单元靠诞生代号挡在外面」有没有循环；③ 最脆弱的一条：给一个判据判为可以逐写判的段上、判定与逐写判代表它的那个状态不同的具体崩溃状态。方案按 `crates/` 今天的实现谈。本报告里**没有一个数是我量的**：引的 `PROBE_*` 行是主 agent 交的任务 8321 的日志（量的，主 agent 跑的），其余全是读代码推的；我写的模型没编、没跑，命令在「要主 agent 跑的命令」一节。

## 各格判定一览

| 格 | 判定 | 依据（细节在各节） |
|---|---|---|
| ① 全称结论外推过头 | 打中（推的） | 探针三段与阳性对照都落在同一类：在飞写全被诞生代号挡住、前缀没有指进本段的引用、本段那次发布的记录一条都没落、没有复用。阳性对照自己在被判据排除的那一类里（`PROBE_WRITE … over_an_earlier_write=true`），证明不了判据收下的段上逐写判对。四类没取样：前缀引用本段槽（「补回」方向）、记录先于单元（恢复的「两份都验过才施加」是合取门）、诞生代号不挡的在飞单元（搬迁副本）、读持久集合与真跑实现的观察者 |
| ② 循环 | 打中（推的） | 判据的可读段表形式「盖的扇区写表里没人写过、基线是 0 ⇒ 不在前缀读集里」靠的是 D16（发布语义） 已定项 7 的持久顺序成立，而那正是崩溃点重放要验的东西；「在飞单元靠诞生代号挡在外面」对搬迁副本不成立（头不盖戳、birth 不变），搬迁副本靠的是另一条机制（I-1.13 的子集规则与 I-1.8 的同组归并），探针没碰过 |
| ③ 候选甲：红不红（逐状态） | 打中（推的，模型待跑：`late_unit`、`record_before_units` 两臂） | 反例 A：C1 的一个被引用单元两份副本写晚到 C2 的单元段（判据照收）；状态 S = {这两份} 全量判绿（推的），甲按单写合成判红。反例 B（视记录点名的单元是不是本段全部写而定）：记录先于单元时，全量里「点名单元都落了、别的没落」的状态由记录核对器判红，每个单写都与空集相同，甲判绿——漏红 |
| ③ 候选甲：红项 | 打中（推的） | 反例 A 的同一族状态，红项多出走读断那一串；见第三节表 |
| ③ 候选乙：红不红 / 红项 | 一对副本时不中，两对时打中（推的，模型 `late_two_units` 臂） | 组内全组合把 {两份} 判到了；跨组是合取（引用的每个单元都要至少一份在）：S = {甲单元两份, 乙单元两份}全量判绿，乙按组合成判红 |
| 六类破坏条件漏项 | 漏 5 类（各指到 文件:行，第四节） | 读持久集合的观察者；恢复施加记录的合取门；诞生代号不挡的在飞单元；系统配置轮换与单元写同段；I-1.8 判过次数对两份副本是合取（b5 写成逐写累加不对）；另有真跑实现的续做观察者没证 |
| 探针取样点不敏感 | 给出 4 个取样点，其中 3 个写成了能跑的命令（第五节、「要主 agent 跑的命令」） | 搬迁那一个只给规格，原型没写 |

## 复跑命令与模型文件

模型目录 `research/prompts/per-write-judging-r1-opus-model/`（原型没编过、没跑过；`out/` 还没有）：

| 文件 | sha256 |
|---|---|
| `proto/Cargo.toml.in` | `8fd80df070d0097e54f8bafa09191543871ffb8b0e5da54082ee55c1e859933d` |
| `proto/src/main.rs` | `801c323c919cbf3e08e0632e8cf852ec9875a4af319129814d8390d9991b6f3a` |
| `tools/rerun.sh` | `e50be0a375f2a52d08080135a448cc209da52fe13d659a844927c627bb09d8c1` |

复跑（整份，交主 agent 跑，见「要主 agent 跑的命令」第 3 条）：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/copy --work /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/rerun-work`。

原型做什么：两块内存稀疏盘（`SparseBlockDevice`，不建镜像文件），mkfs 只跑一次、之后的写全录进一条流（取号 → 暖机 → 新池新建文件 A → `publish_overwrite` 覆盖写 C1 → 覆盖写 C2，文件一直在）；流是原型自己调 core 的公开入口造的，没从崩溃枚举用例拷。五条臂改录下来的写表次序（等于一份把这一处写早了或写晚了的实现）：`unmodified`（原样）、`late_unit`（C1 一对副本挪进 C2 单元段开头）、`late_two_units`（两对）、`record_before_units`（C2 的记录段与单元段对调）。每条臂上从 C1 单元段起每个只有单元写的段，把段内全部真子集逐个判（`judge_crash_image`：两遍恢复、两份 oracle、池级 checker、记录核对器），与四种代表法逐状态比（`union`、`delta`、`group_delta`、`baseline_if_singles_equal`，定义在 `main.rs` 文件头）。跑前先用 `crash_state_count_with_torn_in_place_overwrites` 算出要判的段的闭式状态数与逐个判的数对一遍（`PROTO_STATE_COUNT`、`PROTO_STATE_COUNT_AGREES`），合起来超过 10⁶ 就退 3 不跑；再在第一段上判空集与各单写估挂钟，估出来超过 40 分钟就退 3 不跑。一段写数超过 19 的不展开（打 `PROTO_SEGMENT_SKIPPED`）。

**与派发定义第 3b 步的出入（交主 agent 定）**：崩溃状态没有经 `enumerate_crash_states` / `enumerate_crash_states_versions` 走——那两个入口经判定库、只交回计数，比不了逐状态的红项；原型直接对每个状态调 `judge_crash_image`，状态集合按 D13（验证路线） 已定项 4 的定义取「更早的段全落、本段任意真子集、之后一个不落」。只展开只有单元写的段（逐写判的候选段），记录、根、系统配置段不判：它们不是这一题的被判对象，但这确实不是「这段历史的崩溃状态一个不落」。

## 一、攻击面 ①：「对所有满足判据的段等价」是从哪几个点外推的

探针量的（任务 8321 日志，主 agent 跑的）：删除流段 12、16、20 原样各取单写、成对、只差一个、随机 2000 个子集，逐态与 S = ∅ 比，`red=0 red_differs=0 red_items_differ=0 full_differs=0`；阳性对照「把一个能到达的单元两盘副本都盖掉」红。这几个点共有的性质（读代码推的，每条指到代码）：

1. **在飞写全被诞生代号挡在扫描之外**。三段都是一次普通发布新写的单元，诞生代号是下一版的 txg，扫描方向 `crates/singlefs-checker/src/walk.rs:11256` 的 `return birth_txg <= self.mount_root_checkpoint_txg;` 把它们挡掉，I-7.8 那一路 `crates/singlefs-checker/src/walk.rs:5577` 的 `if birth_txg <= newest_published_txg` 同样挡掉。诞生代号**不**挡的在飞单元一个都没取到（第二节）。
2. **前缀没有一个引用指进本段要写的槽**，空集状态是绿的（`PROBE_BASELINE label=unmodified segment=16 red=false red_items=[]`）。所以三段只量了「在飞写只会往镜像上加东西、加的东西没人读」这一种形状；「前缀已经引用了本段的槽、本段的写把缺的东西补回来」这一方向（落的写越多红项越少）一次都没量过。
3. **本段那次发布的记录一条都没落**：三段之后才是记录段（`PROBE_SEGMENT segment=13 writes=4 kinds={"JournalRecord"}` 在段 12 之后）。恢复里唯一一道「几个写全落才变」的门——施加记录前点名单元两份都要验过（`.claude/kb/decisions/23-journal的角色与格式.md:364`「一个点名单元的两份（两条位置条目）都读得出、校验和都对才施加」）——在这三段上没有对象。
4. **没有复用**：三段 `over_an_earlier_write=0 over_nonzero_base=0`，判据的分类器在三段上只判过「收」，没判过「不收」，它把复用段挡在外面这一半没被量过。
5. **阳性对照落在被判据排除的那一类**：`PROBE_WRITE label=planted segment=16 position=0 … over_an_earlier_write=true`。它说明探针的比法会红，但它造的那一形判据本来就不收，回答不了「判据收下的段上逐写判对不对」。
6. **观察者没进探针**：可写挂载续做观察者在每个状态上真跑实现的挂载与续做（`crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:326`「它交回的只随镜像字节、恢复报告与另读的输入变」），恢复报告一变它的结果就变（第三节反例 B 正好改恢复报告）；删除停在中间观察者读持久集合（`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:110`）。

⇒ 「对所有满足判据的段等价」是从「只往镜像上加、没人读、没有合取门」这一种形状外推到判据收下的全部段。判据字面（单元写、不盖旧写、基线为 0）收下的段里，第三节给出两种落在 2、3 两条之外的形状。

什么现象会推翻这一格：主 agent 在第五节的取样点上跑出四种代表法全部 `red_items_differ=0`，且 `late_unit`、`record_before_units` 两臂的段被分类器判「收」。

## 二、攻击面 ②：判据与它依赖的两句有没有循环

**② 之一：「被选中的根只能是上一版」与「前缀里的根只指向已经落了的单元」。** 调查员的可读段表核法原文（`/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md:223`）：「写「只判 c1 到 c4」的一个简便核法（推的）：单元段里每个写盖到的扇区，在写表里更早的写都没碰过，基镜像在那里也是 0。这样 c1、c3、c4 自动成立，因为前缀里没有谁指向一个从没写过的槽。前缀里的根只指向已经落了的单元；…」。「前缀里的根只指向已经落了的单元」就是 D16（发布语义） 已定项 7 的持久顺序（`.claude/kb/decisions/16-发布语义.md:17`「单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」）在实现里成立。崩溃点重放要抓的正是实现违反这一条（还有删除流段 20 那一类实现写错的）。所以：

- 实现是对的时候，可读段表形式与 c1 等价；
- 实现把一个单元写晚了（反例 A）或把记录写早了（反例 B）时，可读段表形式照收，c1 不成立——判据收不收这一段，取决于被测的那一条性质成立不成立。这是循环：判据用被测性质的结论来决定怎么测它。

c0「被选中的根恒是上一版」本身不循环：它只靠段里没有根槽写（读段表就知道）。循环的是 c1 的可读形式。

**② 之二：「在飞单元靠诞生代号挡在外面」。** 搬迁（整理）把单元原样写到目的地：`crates/singlefs-core/src/transaction.rs:10564`「原样字节写到目的地（头不盖戳、birth 不变）、…」；`.claude/kb/decisions/19-块指针的结构与宽度预算.md:192` 同句「被搬单元逐字节不变、birth 不变」。搬迁批的单元段里，搬迁副本的诞生代号是源那一版的、不大于挂载根，`crates/singlefs-checker/src/walk.rs:11256` 收它进扫描（实例相等支），I-7.8 那一路 `crates/singlefs-checker/src/walk.rs:5577` 也收（码 2 节点同样搬，D19 同一行「搬一个被只读快照引用的码 2 / 码 3」）。目的地是分配器现取的空闲槽，没写过就过判据。这些在飞单元不是被诞生代号挡掉的；它们不改判定靠的是另两处：

- I-1.8 归并键含写序，搬迁副本与源同键，并进源那一组（`crates/singlefs-checker/src/walk.rs:11472` `if members.len() < 2 {` 那一组本来就 ≥ 2）；
- I-1.13 中央映射比对：`crates/singlefs-checker/src/walk.rs:17931` `if walked_placements.is_subset(rebuilt_placements) {` 放过多出来的副本，代码注释原文（`crates/singlefs-checker/src/walk.rs:17927`）「…搬迁把单元原样写到目的落点、…」。这一判的另一支 `crates/singlefs-checker/src/walk.rs:17966` `if readable_rebuilt != readable_walked {` 是**相等**比较：前缀上遍历方向有一份读不出时走这一支，再多一份读得出的搬迁副本就从绿翻红。单写就翻、对多写是「或」，所以甲、乙按单写合成不出错（推的）；但它说明「在飞单元不进判定」这句前提在搬迁段上是假的，等价靠的是两条没被探针碰过的判法。

什么现象会推翻 ② 之二：搬迁批的单元段在 `crates/singlefs-checker/src/walk.rs:11252` 之前另有一道把搬迁副本挡掉的判断（我只读了 `admits_the_header` 与 `content_unit_of_header`，没找到）。

## 三、攻击面 ③：判据收下的段上、判定与逐写判代表它的状态不同的崩溃状态

两段历史共用前缀：mkfs → 取号 → 暖机 → 新池新建文件 A → 覆盖写 C1（`publish_overwrite`）→ 覆盖写 C2，两块盘，每个单元两盘各一份。记 C2 的单元段写集为 W2（n2 个写），C2 的记录两份为 r0、r1。故障都是「实现把一次写的次序放错了」，用户动作（这条流做哪几次发布）在原型里写死成上面那一串；放开扫用户动作没做，见「这条腿自己的限度」。

### 反例 A：单元晚于引用它的根（`late_unit` 臂）

- 历史：C1 有一个被 C1 的根引用的单元 X（原型优先挑数据单元），两份 x0（盘 0）、x1（盘 1）。实现把 x0、x1 推迟到 C1 的根槽写与系统配置轮换之后、跟 C2 的单元一起发。C2 的单元段变成 W2' = {x0, x1} ∪ W2。
- 判据收不收：W2' 只有单元写；x0、x1 落在分配器给 C1 的新槽上，写表里更早没人写过、基线是 0，`over_an_earlier_write=0 over_nonzero_base=0`（推的；原型打 `PROTO_SEGMENT arm=late_unit …` 这一行）⇒ 照可读形式收。调查员 c1 原文（`/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md:214`）「W 里每个写都不碰 R(P)」不成立：X 的槽在 R(P) 里（C1 的根走到它）。
- 前缀 P（C1 的系统配置轮换全落，x0、x1 不落）：C1 的根引用的 X 两份都不在。
- 具体状态：**S = {x0, x1}**（W2 一个都没落）。全量判定：X 两份都在，C1 那一版整棵树读得通，W2 是没人读的在飞垃圾（诞生代号是 C2 的 txg，`crates/singlefs-checker/src/walk.rs:11256` 挡掉）——推的：**绿**。记录核对器也不红：x0、x1 在写表里排在 C1 的根之后，按 `crates/singlefs-verification/src/crash.rs:237`「写表下标 `index` 那次写属于哪次发布：它后面第一条根槽 FUA 写写出的那条根的 checkpoint_txg」归到 C2，恢复落在 C1，C2 那次不在判据二的射程里。
- 甲怎么代表它：J(∅) 是「X 两份都不在」那一串（与探针 `both_planted` 同形：`journal_*_oracle:read_failed`、I-2.1、I-4.8、I-7.2、I-7.4、I-1.13 …，推的）；J({x0}) 是「x1 不在」（I-2.1 每份副本各判一次，`crates/singlefs-checker/src/image.rs:2076` `judgements.judge("I-2.1", matches, || {`），与探针 `planted_one_only` 同形。按 `union` 或 `delta` 合成，S 的代表都留着 I-2.1 等项 ⇒ **红**。红不红不同，红项也不同。同一族里每个 S ⊇ {x0, x1}、S ≠ W2' 都这样，共 2^n2 − 1 个状态（推的）。
- 乙：组 {x0, x1} 组内全组合判到了 S 本身，代表与全量相同（推的）。**两对**（`late_two_units` 臂，X、Y 各两份）时 S = {x0, x1, y0, y1}：全量绿；乙的组 X 代表 J({x0, x1})＝「Y 两份不在」那一串，与 J(∅)「X、Y 都不在」项名相同（都是 I-2.1、I-4.8、read_failed …），按组的增减合成之后仍是 J(∅) ⇒ 红。跨组是合取（引用的每个单元都至少要一份在），组内全组合罩不住。

四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）：

1. 分不分辨臂：一对副本时分辨（甲中、乙不中）；两对时甲乙一起中——共用前提是「可读形式 ⇒ c1」，这一笔另立账先修，判甲乙只看一对那一格。
2. 系统当时看不看得到：只读段 k 的段表看不到——这份坏实现与对的实现在段 W2' 上的段表（种类、盖旧写数、基线非零数）逐项相同。读前缀镜像（P 的根走到哪些槽）看得到，读 J(∅) 也看得到，但正文第四节第 3 条要「读段表就能判、不读判定结果」。按字面是「判别子观测不到」：改判据是新一轮。
3. 满足的是判据字面哪一分句：c1 的语义分句不满足，满足的是调查员给的可读形式（`/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md:223` 那一句）。打中的是「判据写成引擎能逐段判定的形式」那一格（正文第四节第 3 条），不是 c1 本身。
4. 改法在这几格上中不中：见本节末的表。

整条用例红不红不变（P 本身就红）；变的是逐状态的红绿与红项——入库的判定块与 GPU、CPU 对拍比的就是逐状态的这一串。

### 反例 B：记录先于它点名的单元（`record_before_units` 臂）

- 历史：实现把 C2 的记录 r0、r1 发在 C2 的单元之前，中间一道屏障：记录 → 屏障 → 单元 → 屏障 → 根槽。段：[r0, r1]、W2、[根]、…
- 判据收不收：W2 与对的实现一模一样（只有单元写、新槽）⇒ 收。
- 前缀 P（r0、r1 落了，W2 一个没落）：看 journal 的那一遍恢复扫到 C2 的记录，施加前验点名单元两份都在——都不在，不施加（`.claude/kb/decisions/23-journal的角色与格式.md:364` 那一句），落在 C1。推的：P 绿（池级 checker 的 journal 那几条与 I-4.1 怎么对待「施加不了的记录」我没读到，原型的 `baseline_red_items` 会答）。
- 记 N 为 C2 的记录点名的那几个单元的全部副本。每个点名单元两份，单写永远凑不齐 ⇒ **每个 J({w}) 都等于 J(∅)**（推的）。
- 具体状态：**S = N**（点名单元的副本全落、W2 里其余的一个没落），要 N ⊊ W2 才有。这时恢复施加 C2 的记录、自称落在 C2；记录核对器判据二按偏移分组，C2 那次发布有一组副本全缺（没被点名、没落的那个单元）⇒ `record_checker:claimed_state_missing_unit` **红**（`crates/singlefs-verification/src/crash.rs:148` `.any(|copies| copies.iter().all(|copy| copy_is_missing(*copy)))`）。甲四种代表法都给 J(∅)＝绿 ⇒ **漏红**。乙：N 只含一个单元时组内判到、不漏；N 跨两个以上单元时跨组合取，漏。
- 这一形如果成立，整条用例红不红就变了：坏实现的红状态只在这一段（下一段 W2 全落，判据二不缺），逐写判一个都判不到。
- 构造得出的条件只有一条没核：`publish_overwrite` 的记录点名的是不是 W2 的全部单元。D23（journal 的角色与格式） 已定项 12 的射程原文提到「seq 批 1 是 8 条记录、每条只装 1 到 5 个点名项」（`.claude/kb/decisions/23-journal的角色与格式.md:324`），点名项没有逐发布列全，我没去 core 里数。N = W2 时这条路构造不出（全量也看不见那道合取门，两边一样）。原型的 `PROTO_COMPARE arm=record_before_units` 行直接答。

四句：分辨臂——N 一个单元时分辨（甲中乙不中），多个单元时不分辨；系统看不看得到——段 W2 的段表与对的实现相同，差别在更早一段（记录已落），只读本段段表看不到，读前缀里「最新根之上已落的记录点名了哪些槽」看得到、不用读判定结果；满足的分句——R(P) 的定义（`/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md:207`）本来含「journal 点名项走到的单元槽」，c1 的语义分句不满足，可读形式满足；改法见下表。

### 改法各修哪一格（只在我的推理上过过，没实现、没跑，被攻过零轮）

| 改法 | 反例 A 一对 | 反例 A 两对 | 反例 B（N 一个单元） | 反例 B（N 多个单元） | 代价 / 违反什么 |
|---|---|---|---|---|---|
| 甲（现状） | 中 | 中 | 中 | 中 | — |
| 乙（按副本组） | 不中（推的，待主 agent 跑） | 中（推的，待主 agent 跑） | 不中（推的） | 中（推的） | 态数见调查员 R2 |
| 只认段种类全是单元写 | 中 | 中 | 中 | 中 | 改不到：两个反例的段种类都是单元写 |
| J(∅) 红就退回全组合 | 不中（推的） | 不中（推的） | 中（P 绿，推的） | 中 | 读了判定结果，违反正文第四节第 3 条 |
| 引擎在选展开之前走一遍前缀镜像，算 R(P) =（根环里每条读得出的根走到的槽）∪（最新根之上已落的记录点名的槽）∪（分配记录罩住的槽），W ∩ R(P) ≠ ∅ 的段退回全组合 | 不中（推的） | 不中（推的） | 不中（推的） | 不中（推的） | 读镜像不读判定；R(P) 本身就是池级 checker 那一遍走读的读集，它要和 checker 读法同步、要另立会红的检查；回退候选集与被抛弃根的读法要不要并进来没想全 |

这几格全是「推的」；`late_unit`、`late_two_units`、`record_before_units` 三臂跑出来之后，甲、乙两行能改成「量过」。

### 跑前写死的预测（写在跑之前，跑出来之后不改，只往下另记）

| 号 | 预测 | 推翻它的观测 | 推翻之后怎么改 |
|---|---|---|---|
| Q1 | `unmodified` 臂两段四种代表法 `red_items_differ=0`，`singles_equal_baseline=true` | 任一 >0 | 原型或比法写错了（探针在删除流上量过同形），先查原型，其余预测作废 |
| Q2 | `late_unit`、`late_two_units` 的 C2 单元段 `over_an_earlier_write=0 over_nonzero_base=0`（判据收） | 任一 >0 | 两个反例 A 不在判据收下的段上，撤回反例 A |
| Q3 | `late_unit` 的 C2 单元段：`baseline_red_items` 非空；`union` 与 `delta` 的 `red_or_not_differs` = 2^n2 − 1（n2 = `unmodified` 臂那一段的写数）；`group_delta` 的 `red_or_not_differs=0` | union/delta 为 0 | 反例 A 的「S ⊇ {x0, x1} 全量绿」不成立，撤回甲的那一格 |
| Q4 | `late_two_units` 的 C2 单元段：`group_delta` 的 `red_or_not_differs` ≥ 2^n2 − 1 | 为 0 | 撤回乙的那一格 |
| Q5 | 三条 late 臂的红项里一次都不出现 `record_checker:claimed_state_missing_unit` | 出现 | 记录核对器的归法不是我读的那样（`crates/singlefs-verification/src/crash.rs:237`），反例 A 第二条依据改 |
| Q6 | `record_before_units` 的 C2 单元段：`singles_equal_baseline=true`；若有红状态，`baseline_if_singles_equal` 的 `red_or_not_differs` 等于红状态数，红项含 `record_checker:claimed_state_missing_unit` | 红状态为 0 | N = W2，反例 B 这条路构造不出，写进「没打中的形状」 |
| Q7 | `record_before_units` 的 C2 单元段 `baseline_red_items=[]` | 非空 | P 本身红，反例 B 降为与 A 同级（整条用例照红） |

## 四、六类破坏条件漏了哪几类

调查员第三节列的是 b1 镜像双副本、b2 记录核对器全部副本缺、b3 I-7.8 认领跨相邻槽、b4 同段重叠写、b5 判过 0 次报不适用、b6 根与单元同段。漏的（每一类都在判据可读形式收下的段上能出现，除了标明的）：

| 漏项 | 指到哪 | 打破什么 | 局部组合够不够 |
|---|---|---|---|
| L1 前缀引用本段要写的槽（「补回」方向：落得越多红项越少） | 可读形式的出处 `/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md:223`；持久顺序 `.claude/kb/decisions/16-发布语义.md:17` | c1，且可读形式查不出（第二节） | 不够：引用的每个单元至少要一份在，是跨组合取（反例 A 两对） |
| L2 恢复施加记录前「点名单元两份都验过」：前缀已落、点名本段槽的记录 | `.claude/kb/decisions/23-journal的角色与格式.md:364`；判据二 `crates/singlefs-verification/src/crash.rs:148` | c1（R(P) 含记录点名项），可读形式查不出 | 要对 N 做全组合；N 跨几个单元就是 2^|N| − 1 |
| L3 诞生代号不挡的在飞单元（搬迁副本：头不盖戳、birth 不变） | `crates/singlefs-core/src/transaction.rs:10564`；收它的 `crates/singlefs-checker/src/walk.rs:11256`；不改判定靠 `crates/singlefs-checker/src/walk.rs:17931` 与 `crates/singlefs-checker/src/walk.rs:11472` | c2 的前提句；I-1.13 的 `crates/singlefs-checker/src/walk.rs:17966` 那一支是相等比较，前缀上有一份读不出时单写就翻红 | 单写翻、对多写是「或」，甲按单写合成不出错（推的）；但要把这两处判法写进判据的依据，不能再说「在飞单元不进判定」 |
| L4 系统配置轮换与单元写同段（b6 只写了根） | `crates/singlefs-verification/src/crash.rs:329`「上一次发布的系统配置槽轮换与这一次的单元写」…「之间没有屏障、并在同一段时」；I-7.7 按系统配置的实例代号比每个载体 `crates/singlefs-checker/src/walk.rs:5741`，两盘不等时 `crates/singlefs-checker/src/walk.rs:5748` | 种类检查会挡下（段里有系统配置槽写），不是可读形式的漏洞；但局部组合表里没有这一类，退回全组合时也没写局部范围：系统配置写取三态，I-7.7 ② 在「一盘轮换了、一盘没轮换」加「单元落了」时红，是单元写与系统配置写的合取 | 系统配置两写 × 单元：3² ×（本段单元写全组合或按实例代号分组）|
| L5 I-1.8 判过次数对两份副本是合取（b5 写成「逐写累加」不对） | `crates/singlefs-checker/src/walk.rs:11472` `if members.len() < 2 {`；不适用 `crates/singlefs-checker/src/walk.rs:21378`；b5 原文 `/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md:233`「要 E3 就得把这类条目的判过次数逐写累加」 | E3：一组只有在飞副本、诞生代号又被收的单元（坏实现盖错诞生代号），两份都落才凑成一组、判过次数加一，单写都不加；前缀里一组都没有时就是「不适用 → 成立」。只在坏实现上出现 | 逐写累加给不出；要按副本组合 |
| L6 读持久集合的观察者 | `crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:110` | E3 计数（调查员在 c5 写了，但没进六类） | 按 [S ≠ ∅] 计可以 |
| L7 真跑实现的可写挂载续做观察者 | `crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:326`；恢复报告里改按映射读的次数 `crates/singlefs-core/src/recovery.rs:876` `pub mapping_fallbacks: usize,` | c5 没证；它随恢复报告变，L2 一出现它就跟着变；挂载要写盘、分配器可能把新单元写在在飞垃圾的槽上 | 没核 |

L1、L2 两类合起来是「前缀里有指进本段写集的引用」，它不是本段段表的性质，是前缀的性质；按正文第四节第 3 条「读段表就能判」写的判据看不到它（第三节四句的第 2 句）。

## 五、探针的取样点不敏感在哪、该加哪几个

探针（任务 8321）的取样点全在「只往镜像上加、没人读、没有合取门」那一形上（第一节 1–5 条）。它在那一形上的 0 是可信的（阳性对照红了），但那一形上逐写判本来就对：在飞写不进任何判法时，所有代表法都等于 J(∅)。会让逐写判与全组合不同的取样点，按能不能跑分：

| 取样点 | 为什么敏感 | 怎么跑 |
|---|---|---|
| T1 前缀引用本段的槽，「补回」方向（`late_unit`） | 落的写越多红项越少，单写合成给不出「全补齐」那一态 | 原型，命令第 1、2 条 |
| T2 跨组合取（`late_two_units`） | 乙的组内全组合罩不住跨组的「每个都至少一份」 | 同上 |
| T3 记录先于单元（`record_before_units`） | 恢复的「点名单元两份都验过才施加」是合取门，单写永远凑不齐 | 同上 |
| T4 搬迁批的单元段（诞生代号不挡） | 在飞副本进扫描，等价靠 `crates/singlefs-checker/src/walk.rs:17931` 子集规则与 I-1.8 同组归并；`crates/singlefs-checker/src/walk.rs:17966` 那一支是相等比较 | **没写成命令**（推的，待写）：已有的崩溃枚举流 `crates/singlefs-verification/tests/crash_enumeration_compaction_relocation_stream.rs` 里有搬迁批；规格：在自己的原型里自造一次搬迁批（要先签发整理意图，`publish_compaction_step` 再 `publish_relocation_batch`），对它的单元段打段表与 `over_*`，再按本原型的比法逐状态比；另造一臂把前缀上一份被遍历方向引用的副本清零（走 `crates/singlefs-checker/src/walk.rs:17966` 那一支），看单写是不是就翻红 |
| T5 有复用的段（判据该判「不收」的那一半） | 三段 `over_*` 全 0，分类器的「不收」从没被量过 | 没写成命令（推的，待写）：`crates/singlefs-verification/tests/crash_enumeration_compaction_deadlist_reclamation_stream.rs` 那条流回收复用槽，规格是打它每段的 `over_an_earlier_write`，核复用段被判「不收」 |
| T6 续做观察者 | 每个状态真跑挂载、随恢复报告变（L7） | 没写（推的，待写）：T3 那一臂的状态上挂 `observe_the_mount_continuation`，比 S = N 与 S = ∅ 的续做步数与挂完的 checker |

## 没打中的形状

试过、构造不出（全是读代码推的，没跑）：

- **前缀绿、在飞写都落在新槽且没被前缀引用时，池级 checker 的合取**：扫描方向三处（`crates/singlefs-checker/src/walk.rs:11415` 起的码 1 / 码 3、`crates/singlefs-checker/src/walk.rs:5537` 起的 I-7.8、`crates/singlefs-checker/src/walk.rs:5628` 起的 I-7.7）逐槽判；I-7.7 对载体是有一个超过就红（`crates/singlefs-checker/src/walk.rs:5741` `.find(|(_, instance)| *instance > pool_highest);`）；I-7.8 认领规则 `crates/singlefs-checker/src/walk.rs:5569` `first_slot_not_claimed = slot + claimed_slots;` 只往后跨本单元自己那一次写盖住的槽，两个不重叠的新写互不跨；I-1.13 不适用的门 `INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`（`crates/singlefs-checker/src/walk.rs:18033`）七项全是遍历方向的、由前缀定，在飞写遮不住别的红项。取样范围：按 114 条里扫描三处与 I-1.13、I-1.8 读代码，别的 T 类、U 类条目只读前缀引用到的槽，没逐条追。
- **候选槽集合**：`crates/singlefs-verification/src/memory_pool.rs:838` 只把落了的单元写盖住的槽加进候选，没落的不进；不构成合取。
- **撕裂单元写**：单元写的撕裂态按 D13（验证路线） 已定项 4 并进「没持久」，可是头扇区落了、体没落时头校验和照过（I-7.7 只验头），这一格全量与逐写判同样看不见，是「打中不分辨臂」，不拿来判甲乙。
- **续做观察者在搬迁垃圾槽上**：续做是新实例，旧实例的在飞单元按实例表那一支（`crates/singlefs-checker/src/walk.rs:11252` 之后的实例比挂载根小那一支）判没发布；I-7.8 的认领规则会让旧垃圾头把续做新写的下一槽跳过，只会让 I-7.8 少数一个树 ID、红不起来。

「没打中」按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」只抽了这一次，不支撑「没有」。

## 这条腿自己的限度

- 原型没编、没跑；API 签名逐个 grep 过，但编不编得过要主 agent 编了才知道。编不过请新派攻方腿承接修。
- 两个反例都改的是写表次序（造的是坏实现），不是今天的实现走得出的历史；今天的实现上（`unmodified` 臂）我预测不中（Q1）。所以「判据在今天对的实现上会不会错」这一句我没打穿，打穿的是「判据收下的段上，逐写判能不能抓住它该抓的坏实现」。
- 用户动作写死：流只有 A → C1 → C2 三次发布，没放开扫「C2 做什么」（覆盖写、建 inode、删除意图）。反例 B 构造不构造得出取决于 C2 那次的记录点名几个单元，换一种发布结论可能翻，第三节已写成条件。
- 只展开只有单元写的段，没按第 3b 步走 `enumerate_crash_states`（理由在开头）。
- 别家实现一样没引。

## 要主 agent 跑的命令

编译与跑都在我建的仓副本里（`/home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/copy`，`python3 scripts/hosts/minimal-copy.py … --with-test-inputs` 建的，50M，草稿目录的 `.deliverables` 里写成「交接」）。原型包是副本里的 `per-write-judging-attack-proto/`（自己一个 workspace，path 依赖指副本的 `crates/`，锁文件拷自副本的 `Cargo.lock`；源与模型目录 `proto/src/main.rs` 逐字节相同）。

1. 编原型
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/copy/per-write-judging-attack-proto -- cargo build --release`
   - 在哪起：上面 `--cwd` 那个目录（副本里）。
   - 看什么：`TASK_SERVER_RESULT` 的 `type=succeeded exit=0`；记下末行的 `build=` 目录。编不过（`exit=101` 一类）请新派攻方腿承接，派发提示带「承接：research/prompts/per-write-judging-r1-opus-output.md」与任务号。
2. 跑原型（第 1 条过了才跑）
   - 命令：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/copy/per-write-judging-attack-proto -- <build= 目录>/target/release/per-write-judging-attack`（`<build= 目录>` 照第 1 条末行填）
   - 看什么：`TASK_SERVER_RESULT exit=0` 且末行 `PROTO_FINISHED`；退 2 是流的段形状与预想不同（会打 `PROTO_FLOW_SEGMENT` 全表与 `PROTO_REFUSED`），退 3 是状态数或估时超上限（`PROTO_PLANNED_STATES`、`PROTO_ESTIMATE` 那两行）。读 `PROTO_STATE_COUNT_AGREES … agrees=true`（每条臂一行，`false` 说明我对引擎闭式的理解错了，比法照看、状态数另核）、各臂 `PROTO_SEGMENT` 的 `over_*`（Q2）、`PROTO_STATES` 的 `baseline_red_items` 与 `singles_equal_baseline`（Q3、Q6、Q7）、`PROTO_COMPARE` 四种代表法的 `red_or_not_differs` 与 `red_items_differ`（Q1、Q3、Q4、Q6）、`PROTO_EXAMPLE` 给的具体状态；对着第三节「跑前写死的预测」逐条判。状态数按 `publish_overwrite` 一段约 12 写推，四条臂合起来约 10 万态（推的；原型跑前自己打 `PROTO_PLANNED_STATES`）。
3. 整份复跑（判决要引的数用这一条）
   - 命令：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/copy --work /home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/rerun-work`
   - 在哪起：随意（脚本只从参数取路径）；主 agent 照 `.claude/rules/three-way-inference.md` 复跑时把模型目录拷进自己的草稿目录、`--frozen` 照给这个副本或开工快照倒推出的树。要在新 shell 里起。
   - 看什么：`TASK_SERVER_RESULT exit=0`；`name=rerun_output … lines=` 与 `name=rerun_compare`（模型目录还没有 `out/out.txt`，这一趟打 `same=not_compared`；第 2 条的输出存成 `out/out.txt` 之后再复跑才比）。

## 没做什么

- 没往统一任务服务交任务；原型没编、没跑；判决要引的数全都还没有（命令在上一节）。
- 不判正推、辩方那几格；不替主 agent 采纳。改法表里每一格都是推的，被攻过零轮。
- 搬迁（T4）、复用（T5）、续做观察者（T6）三个取样点只给了规格，原型没写。
- 没放开扫用户动作（C2 做哪种发布）。
- 没读池级 checker 114 条的每一条判法，只读了扫描三处、I-1.8、I-1.13、I-2.1 与记录核对器。
- 引的调查员报告在仓外（`/home/fy5090/.local/share/singlefs/work/per-write-judging-facts/result.md`），行号是今天现查的。

## 草稿目录里留的

- `/home/fy5090/.local/share/singlefs/work/per-write-judging-r1-attack/copy`：仓副本，50M，`.deliverables` 里写成「交接」（主 agent 在里面编、跑原型，用完删）。里面没有编译目录：编译由任务服务在它自己的目录里做。
- `log8321.txt`（任务 8321 日志的拷贝，我读的那一份）、`minimal-copy.txt`、`fmt*.txt`：普通文件。
