# 里程碑三设计轮第二轮 · 云端正推（Sonnet）报告

日期：2026-09-28。派发格：正文第四节「云端正推（Sonnet）」一行——B1–B9、C1–C8、D1 全部。

## 读法与总纲

**统一原则**：全部身份值按 `research/prompts/_m3-prune-gpu-r2-body.md:19`「身份值按磁盘格式那一级对待」对待——每一种身份值与每张表带定义版本号，定义一改旧数据读不到、不被读错。设计上做两件贯穿全部小题的事：

1. **节点身份（B1）与节点输入指纹（B2）都取「路径」口径，不取「镜像」口径**：路径写表本身（设备、种类、FUA、偏移、内容）是唯一忠实的信息源，父结束镜像会把「同一份字节、不同的写历史」（第一轮 T4 打中的 t4 世界）压扁成同一份输入，漏掉撕裂与去重相关的历史。这一条贯穿 B1、B2，也决定了 B9（块按节点分）、C5（单点 / 单线起点镜像从根重放而不是缓存镜像）、C6（复用按指纹，不按时限）的设计。
2. **KV 库（`crates/singlefs-checker-tier/src/verdict_store.rs`）今天没有任何调用方、没有布局版本**（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md` 第三节 `verdict_store.rs` 那一行「① 还没有任何调用方…… ④ 名字写「流名或节点名」，今天没有节点」）——这份报告把它当「今天的实现」的一部分设计接口，不改它已实现的字节布局（那是它自己的版本 0），只设计：加一个新的元数据键（B8）、给它接上节点身份（B1/B9）、把 C1–C8 的录入 / 核对逻辑摆在它周围。

以下 B1–B9 每一种身份值按六问作答；六问的问法照正文第一节：①字段与宽 ②罩住的输入 ③漏了会不会同键异判 ④谁先发现 ⑤版本号放哪 ⑥代价。每种身份值末尾附「十二世界判什么」一张表（第一轮攻方十个世界 + F1 + F2）。

## 附录范围内的实现现状（读到的，不是这一轮新查的）

- 节点树、剪枝、GPU 代码今天在 `crates/` 里都不存在（`grep -rln wgpu crates/` 零命中，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md` 第 66 行）。
- 崩溃点重放的段切割规则只有一份实现（`crates/singlefs-harness/src/segments.rs` 的 `SegmentClosingRule`，同一份规则同时喂两条切法——出种类串与出写表下标）：「两份切段（[split_into_segments] 出种类、crash::writes_and_segments_with_stream_indexes_and_entries 出写表下标）都经它判」（`crates/singlefs-harness/src/segments.rs:104`）「规则只有这一份」（`crates/singlefs-harness/src/segments.rs:105`）。
- `StepKind` 是封闭六种类枚举，声明序 `ZeroFill, UnitWrite, JournalRecord, RootRecordFua, SystemConfigurationSlot, Barrier`（`crates/singlefs-harness/src/segments.rs:21`–`:28`），「声明序就是种类串的规范序」（`crates/singlefs-harness/src/segments.rs:19`）。
- `layer0_plan_hash`（`crates/singlefs-checker-tier/src/crash.rs:2295`）已经把整条流的基线与写表按长度前缀拼接后取 SHA-256——`crates/singlefs-checker-tier/src/crash.rs:2286`「枚举计划哈希（进度文件分格的第三样，层 0 规模第三轮判决 U1）：基线的每个扇区、写表（盘、种类、FUA、偏移、内容」，这是 B2 节点输入指纹要复用的编码骨架，只是作用域从整条流缩小到一条路径前缀。
- 记录归属today纯按写表位置分桶，不读记录自己的字段：`publishes_of_one_recording`（`crates/singlefs-harness/src/memory_pool.rs:993`–`:1017`）逐个写表项扫，遇到 `JournalRecord` 只 `records.push(index)`，遇到 `RootRecordFua` 才 `root_identity_of_write(write)` 取根自己的身份、把攒的 `records`/`units` 打包成一次发布；journal 记录自己的 `(instance, checkpoint_txg)` 字段从未被读取用于归属。而记录头本身自描述这两个字段：`check_journal_record`（`crates/singlefs-checker/src/lib.rs:733`）解出的 `JournalRecordView` 里 `instance: read_u32(record, 16)`、`checkpoint_txg: read_u64(record, 26)`（`crates/singlefs-checker/src/lib.rs:793`、`:795`）。


## 第一题：身份值与数据结构

### B1 节点身份

**①字段与宽**：节点身份 = 种类串 ‖ 覆盖关系签名。种类串是这个节点自己那一段（按 T1「按操作」/「按段」already 站住的粒度，一个节点对应 `crates/singlefs-harness/src/segments.rs` 切出来的一段）里每一步的 `StepKind` 判别式，1 字节一步（六种，`u8` 够用，`crates/singlefs-harness/src/segments.rs:19`–`:28`）；覆盖关系签名与种类串等长，每步 1 字节位旗标，4 位在用（bit0=与更早的 `UnitWrite` 重叠、bit1=与更早的 `RootRecordFua` 重叠、bit2=与更早的 `JournalRecord` 重叠、bit3=与更早的 `SystemConfigurationSlot` 重叠），4 位预留（B8 版本升级时不移位、只加新位）。「与更早的写重叠」判的是路径上**从 mkfs 起、到这一步为止**、同一块盘上同一种类、字节区间有交集的任意一次写，不止是紧邻的上一次。

**②罩住的输入**：这个节点自己这一段的写序列（种类 + 是否与路径上更早的同类写重叠），不罩这一段写的具体偏移与字节内容（内容归 B6/B2）、不罩这一段之前路径的具体镜像内容（归 B2 的路径写表哈希）。

**③漏了会不会同键异判**：漏覆盖关系签名（只用种类串）会同键异判——第一轮攻方 kinds 世界实测：`distinct_kind_strings=3`、其中 1 种种类串下有 2 个条件签名，「同一种类串下并着「根槽第一次写」与「根环回卷」」（`research/prompts/m3-prune-gpu-r1-main-verification.md:44`）；两个条件签名的差异点就是 `root_slot_rewrite`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/kinds.out:1`：`detail=["SS|UUU…|JJ|R|SS=>{"reuse=false,root_slot_rewrite=false,…", "reuse=false,root_slot_rewrite=true,…"}"]`），恰好是覆盖签名 bit1（与更早的 `RootRecordFua` 重叠）要分开的那一位。用 `/tmp/claude-1000/m3-prune-gpu-r2-sonnet/derive_b1_overlap_signature.py` 解析这一行原始输出（只做字符串切分与算术，不编译不跑代码）核过：加上这一位之后，`distinct_node_identities_with_overlap_bit=2`，与两条条件签名一一对应（脚本输出 `PASS: 加一位 RootRecordFua 重叠位之后，kinds 世界这一条种类串下的两个条件签名分成了两个不同的节点身份`）。

**④谁先发现**：新增一条 harness 档断言 `node_identity_distinguishes_first_root_write_from_root_ring_wraparound`：对 kinds 世界那条种类串重算「种类串 + 覆盖签名」，断言 `distinct_node_identities == distinct_condition_signatures`（今天种类串单独算出来的数是 `distinct_kind_strings=3` 而 `kind_strings_with_several_condition_signatures=1`，改法之后两者应相等）。

**⑤版本号放哪**：进 B8 设计的「节点身份编码版本」子字段（`node_identity_version`）；种类串本身的规范序随 `StepKind` 声明序（`crates/singlefs-harness/src/segments.rs:19`「声明序就是种类串的规范序」）——新增第七种 `StepKind` 时这个版本要跟着抬，因为它改变了种类串每一步的取值范围与编码宽度假设。

**⑥代价**：每节点字节数 = 2 × 段长（种类串 1 字节/步 + 覆盖签名 1 字节/步）。第一轮攻方实测的段长序列：interior 与 order、small 三个世界共享同一条前 11 段的路径，`segment_lengths=[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/interior.out:1`）；reuse 世界的路径更长，`segment_lengths=[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 24, 2, 1, 2, 2, 2]`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/reuse.out:1`）——最长的段 24 步，节点身份 48 字节；最短的段 1 步，节点身份 2 字节。

#### B1 在十二世界上判什么

| 世界 | 种类串单独判 | 加覆盖签名之后判 |
|---|---|---|
| kinds | 3 种串，1 种下 2 个条件签名（合并） | 4 种节点身份，一一对应（本节脚本核过） |
| collide | 不涉及（这是内容碰撞世界，见 B6） | 同左 |
| interior | 不涉及节点身份合并；打中的是 B4/F1 的扫描边界问题 | 同左，不受影响 |
| order | 不涉及节点身份合并；打中的是 B5 记录归属 | 同左，不受影响 |
| p5 | 不涉及（P5 是按论证跳过剪枝的世界，见 B4） | 同左 |
| p6 | 不涉及（P6 是等价类聚合世界，见 B4） | 同左 |
| reuse | 段序列 `[2,2,1,2,2,1,2,24,2,1,2,24,2,1,2,2,2]`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/reuse.out:1`）——两段都是 24 步的单元段，种类串相同（都是同一种整段写），覆盖签名要分：第一次写这个位置（无覆盖）与提前复用再写同一位置（有覆盖）区分开，否则 I-7.4（近 K 代块未被复用） 类违例会被并进同一个节点 | 分成两个节点身份（覆盖签名 bit0 不同） |
| small | 两条路径前 11 段共享（`shared_prefix_identical=true`，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/small.out:1`） | 共享段的节点身份逐段相同（种类串与覆盖签名都相同），第 12 段起分叉 |
| t4 | 不涉及节点身份（涉及 B2 指纹） | 同左 |
| torn | 单一段内枚举，不涉及跨节点合并 | 同左 |
| F1（`m3-prune-gpu-r1-f1-i78-report.md`） | 不涉及节点身份（是 B4 扫描边界问题） | 同左 |
| F2（`m3-prune-gpu-r1-f2-order-report.md`） | 不涉及节点身份（是 B5 记录归属问题） | 同左 |


### B2 节点的输入指纹

**①字段与宽**：递归定义，32 字节（SHA-256）：`fingerprint(根节点) = SHA256("singlefs node fingerprint 1" ‖ mkfs 写表)`；`fingerprint(子节点) = SHA256(fingerprint(父节点) ‖ 本节点自己的写表 ‖ 判法摘要)`。「写表」按 `layer0_plan_hash` 已经在用的编码（设备号、种类名、FUA 位、偏移、内容，逐项长度前缀拼接，`crates/singlefs-checker-tier/src/crash.rs:2286`）取这个节点自己那一段的写，不是整条路径重新编一遍——父指纹已经把父节点及其以上的路径吸收掉，逐节点只需把自己这一段追加进去（增量哈希，O(1) 而不是 O(路径深度)）。判法摘要是 B8 设计的「判法版本」标量（oracle、池级 checker 走的不变量集合、记录核对器的版本各一个小整数，拼成固定字节序列）。

**②罩住的输入**：从 mkfs 起、到这个节点结束为止的**全部写**（设备、种类、FUA、偏移、内容），以及产生判定要用的判法版本；不罩：这个节点的展开方式（P6/P7 怎么把状态分类，那是 B4 的键）、哪个根被判（`judged_root_index`）与版本表——这两样如果也影响判定，需要在判法摘要里额外带上（见「④谁先发现」的后半段）。

**③漏了会不会同键异判**：**打中过**——第一轮 T4 世界证明「父结束镜像哈希 + 子节点写表」不够：三个变体（a_plain、b_bytes_then_zero_fill、c_zero_bytes）里 a 与 b/c 的 `parent_end_bytes_equal_to_a=true`、`child_writes_identical=true`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/t4.out:1`–`:3`），但 b/c 的 `child_first_write_tearable_in_overlay=true` 而 a 是 `false`，导致 `child_states_overlay` 是 5（b/c）对 3（a）——如果指纹只取「父结束镜像的字节内容 + 子节点写表」，a 与 b/c 会被判成同一个指纹却应该有不同的枚举域。本设计的修法（父指纹 = 父路径**写表**哈希，不是父镜像**内容**哈希）能分开 a 与 b/c：variant b/c 的父路径比 a 多一次 `ZeroFill`/覆盖写（`b_bytes_then_zero_fill`、`c_zero_bytes` 这两个变体名本身就点出了那一次额外的写），写表哈希会因为这次额外的写而与 a 不同，即使父结束时的镜像字节相同。

**④谁先发现**：新增一条 harness 档断言 `node_fingerprint_separates_t4_variants_with_identical_parent_mirror`：对 t4 三个变体分别按「父结束镜像哈希方案」与「父路径写表哈希方案」各算一遍父指纹，断言前者 a==b==c（复现问题）、后者 a≠b、a≠c（本设计修复）。第二层发现：如果 `judged_root_index`/版本表确实影响某节点的判定而没有进判法摘要，新增一条断言让两个只在 `judged_root_index` 上不同、别的都相同的节点在这个字段缺失时共享同一个指纹却给出不同判定——`crates/singlefs-checker-tier/src/crash.rs` 里 `enumerate_layer0_in_state_slices` 一族函数固定了 `judged_root_index`（层 0 只判「实际走的那条根」，D13（验证路线） 已定项 7 已经把「被判的根」钉成「恢复实际走的那条」而不是给调用方选择），所以今天的层 0 场景下这一项不随节点变化，可以先不进指纹；一旦以后要支持「同一份历史判不同的根」，要把 `judged_root_index` 摘要补进判法摘要。

**⑤版本号放哪**：判法摘要本身自带三个子版本（oracle 版本、checker 不变量集合版本、记录核对器版本），这三个子版本又都收进 B8 的库级布局版本记录；写表编码格式版本（设备号/种类名/FUA/偏移/内容各字段的长度前缀写法）另开一个子版本，因为它决定「同一段写历史算出同一个哈希」这条性质本身。

**⑥代价**：每节点一次 SHA-256（32 字节输出），输入长度 = 父指纹 32 字节 + 本节点写表字节数（写表条目按 `layer0_plan_hash` 的写法，每条至少 1（设备号）+ 若干（种类名长度前缀）+ 1（FUA）+ 8（偏移）+ 内容长度前缀，内容可达 32 KiB 一个单元）+ 判法摘要几十字节。存储只需存 32 字节指纹本身（不需要存中间写表），代价与节点数成正比，不与路径深度成正比（增量哈希）。

#### B2 在十二世界上判什么

| 世界 | 判什么 |
|---|---| 
| t4 | 直接针对的世界：三变体下父路径写表哈希把 a 与 b/c 分开（见③） |
| small | 两条路径前 11 段共享，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/small.out:1`「shared_prefix_identical=true」——共享段的指纹逐段相同（递归定义下父指纹相同、子段写表相同 ⇒ 子指纹相同），第 12 段起 `path_b` 多出的写让指纹分叉 |
| reuse、order、interior | 路径更长（11–17 段），指纹递归延伸，不需要重新验证更早的段 |
| kinds | 不涉及指纹（涉及 B1 种类串合并） |
| p5、p6、torn | 不涉及跨节点指纹比较，涉及的是同一节点内的状态分类（B3/B4） |
| collide | 不涉及指纹（涉及 B6 内容身份） |
| F1（`research/prompts/m3-prune-gpu-r1-f1-i78-report.md`） | 不涉及（B4/扫描边界） |
| F2（`research/prompts/m3-prune-gpu-r1-f2-order-report.md`） | 不涉及（B5 记录归属） |


### B3 状态身份与持久集合表示

**①字段与宽**：状态身份 = (节点身份 B1/指纹 B2 关联的 `node_id`，节点内序号)。`node_id` 是 B9 设计里给节点分配的一个紧凑整数（不是 32 字节指纹本身，指纹只用来查表拿 `node_id`，见 B8 元数据表的新增第五张表）；节点内序号沿用今天 `crash.rs:1917` `persisted_writes_of_state` 的混合进制算法（原地覆写取三态、其余写取两态），只是把作用域从「整条流的全局序号」缩小成「这一个节点自己的段」。持久集合不单独存：由 `(node_id, 节点内序号)` 按同一套混合进制现算，与今天「持久集合由序号现算，不存」（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md` 第 122 行「持久集合由序号现算，不存」）的做法一致，只是序号的作用域收窄。

**②罩住的输入**：这个节点自己段内每次写的持久 / 不持久 / 撕裂三态取值；不罩这个节点之前路径的持久状态（那已经是「全部持久」，是这个节点存在的前提）。

**③漏了会不会同键异判**：本设计不改变现有算法本身（只改作用域），所以「同键异判」的风险点在于**四条路径（单线、单点、分片、GPU 批）算序号时有没有各用一套**——今天分片按片交错（`slice_index % 2`，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md` 第 150 行），这与「节点内序号」是两个不同维度：分片是「这一批状态里选哪些算」，节点内序号是「这个状态在自己节点里排第几」，两者正交，不冲突。

**④谁先发现**：新增一条 checker-tier 档断言 `state_ordinal_agrees_across_single_line_single_point_shard_and_gpu_batch`：同一个具体状态分别用四种入口（`--node --state` 单点、整节点单线跑、`SINGLEFS_LAYER0_SHARD=i/n` 分片、GPU 批派发）各自算出的 `(node_id, 节点内序号)` 必须逐字段相同。

**⑤版本号放哪**：混合进制算法版本进 B8（今天没有改动这个算法，版本号先钉在「与今天相同」这一档，留位置给以后改撕裂态种数时抬）。

**⑥代价**：位图 8 字节/状态、段长 16–28（第一轮 S1 站住的实测：「本地攻方两份干净样本：每状态 8 字节（段长 16–28）」，`research/prompts/m3-prune-gpu-r1-main-verification.md:62`），本设计沿用，不改。

#### B3 在十二世界上判什么

| 世界 | 判什么 |
|---|---|
| 全部十个攻方世界 + F1 + F2 | 都不直接攻状态身份的编码算法本身（S1 站住、被攻过一轮未打中，`research/prompts/m3-prune-gpu-r1-main-verification.md:62`）；本设计的改动只是把作用域从「整条流」缩小到「节点」，十二世界里没有一个世界的路径长到需要跨节点的状态序号连续性——各世界的段数最多 17（reuse），远小于会暴露「节点边界处序号断裂」这类问题的规模，这一格留给第二、三轮用更长路径攻 |


### B4 恢复与池级 checker 的等价类身份（剪枝的键）

**①字段与宽**：两段拼接，都是定长 32 字节（SHA-256，抗碰撞散列，见 B6）：

- **读集段**：对这个状态跑恢复（两遍 `JournalPolicy`）、oracle、池级 checker 走读时，每一次「读一个位置」都把 `(设备, 偏移, 长度)` 与读到的内容一起喂进一个滚动 SHA-256（不是先攒成 `Vec` 再整个哈希，避免读次数很多时中间列表本身占内存）——这与今天「整串调用当键」（`research/prompts/m3-prune-gpu-r1-main-verification.md:52`「键 = 整串调用的读（位置 + 内容）」）同一个信息集合，只是编码从「存整份列表」改成「滚动摘要」。
- **候选槽聚合段**（P7 新增，修 C587）：池级 checker 扫描方向不再把每个候选槽的读判定单独进键，改成先跑一遍「聚合」——扫描按候选槽升序处理，维护一个「已消费到」水位线；候选槽偏移 < 水位线的**跳过**，不读它的头、不进键；候选槽偏移 ≥ 水位线且头校验和过、magic 对的，**接受**为一个真单元，把水位线推进到「这个单元的声明宽度」之后（数据单元固定跨度，索引 / 打包记录单元按头里的声明宽度算），只有被接受的这些单元的头内容与位置进这一段的键。这直接对应 P7「扫描拆成逐槽判与聚合……扫描只按进得了聚合的内容进键」的设计。

**②罩住的输入**：恢复 + oracle + 池级 checker 走读的全部读位置与内容（读集段），加上扫描方向**接受**的那些候选单元（聚合段）；**不罩**：记录核对器的读（它要另外的持久集合信息，另算，见「③」）、扫描方向**跳过**的候选槽（它们不该影响判定，因为它们本来就不是真单元）。

**③漏了会不会同键异判**：

- **只有读集段、没有聚合段**（今天的做法）：**打中且已实现验证**——F1 攻方在合法 32K 数据单元内容第二个槽开头放一个校验和自洽的伪造码 2 头，扫描把它当真单元头解析，让 I-7.8 在健康池上假红（`.claude/kb/checks-owed.md:467`「池级 checker 扫描方向对每个候选槽开头都当单元头解析，不判它是不是落在另一个单元中间」）；机理在 `crates/singlefs-checker/src/walk.rs:2405`（候选槽枚举）、`:2413`（只判 magic 与类别字节，不判位置合法性）、`:2417`（只判头自身校验和）、`:2419`（读诞生代号）、`:2427`（写进 identifiers 集合，未核对它是不是落在另一单元里面）。
- **只用抗碰撞散列而不加聚合水位**：不修 C587——B6 的哈希算法升级管的是「内容碰撞」，管不了「扫描把伪造头误当真头」，两个问题在不同层。
- **P6 无聚合时不裁剪**（另一种「漏了」）：单元段上 `leaves_recovery_plus_checker_normal=4095`、`distinct_candidate_unit_slot_answers=4095`，与 `states_in_unit_segment=4095` 相等（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/p6.out:2`），说明候选槽答案随每个状态的持久子集变化，逐状态各不相同——不加聚合，读集键在单元段上一个状态都并不拢，P6「省」不下来这条第一轮已经打中挂起（`research/prompts/m3-prune-gpu-r1-main-verification.md:55`「单元段上叶子数 = 状态数 = 4095、判定只有 1 类」）。

**④谁先发现**：F1 报告已给出判别力自证做法：`.claude/kb/checks-owed.md:467`「造一份健康池（写一个用户数据单元，内容第二个槽开头放伪造码 2 头），断言……与别的不变量都判绿；今天的扫描代码在这份镜像上会误判……违反；判别力自证：把伪造头去掉，检查须转绿」——这条断言原样收进 checker 档新增用例 `scan_direction_rejects_a_forged_header_inside_another_units_content`。P6 省不下来那条由 `research/prompts/m3-prune-gpu-r1-opus-model/attack.rs` 的 `attack_p6` 世界改造：加聚合水位之后重算 `leaves_if_scan_dropped_walk_key` 一类指标，断言单元段上的叶子数显著小于状态数（今天 `leaves_if_scan_dropped_walk_key=1`，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/p6.out:2`，说明「丢掉扫描键之后」单元段其实只有 1 类——本设计要的聚合恰好是「丢掉逐候选槽的读集键、换成聚合后的单元级键」，与这一列已经量出的数字方向一致，可以直接复用这个世界验证）。

**⑤版本号放哪**：进 B8：读集编码版本（滚动哈希的输入编排）、聚合算法版本（P7 的水位推进规则——尤其是「单元声明宽度怎么算」这条规则一旦改变，旧键全部作废）。

**⑥代价**：32 字节的键，但**计算代价**是这个状态被恢复 / oracle / checker 走读实际读了多少次、多少字节——E161 可行性档量过（`research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:212`–`:216` 那张表）：`second_quick` 格每状态 227.0 次单元读、198.7 次头扫描读、18.4 次环记录读，池级 checker 占每状态总耗时 91.1%——这与 P6「精确但省不下来」的判决一致：加密钥的滚动哈希本身不贵（O(读字节数) 一次遍历），贵的是「读」这件事本身没有变少，聚合只解决「同键异判」的正确性问题，不解决 P6「省」的性能问题（那要靠 P7 之后再叠加「类判定」，属于第二、三轮要继续攻的题）。

#### B4 在十二世界上判什么

| 世界 | 判什么 |
|---|---|
| F1（`research/prompts/m3-prune-gpu-r1-f1-i78-report.md`） | 直接针对：加聚合水位后伪造头不再被接受为真单元 |
| p6 | 直接针对：单元段上 `distinct_candidate_unit_slot_answers=4095`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/p6.out:2`）说明不加聚合就并不拢；加聚合之后单元段应退化为 `leaves_if_scan_dropped_walk_key=1` 那一档（同一行） |
| p5 | 键罩全部读位置这一格同键异判 0（`research/prompts/m3-prune-gpu-r1-main-verification.md:52`「p5 的 63 个、p6 的 4095 个单元段状态同键异判 0」，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/p5.out:4`：`full_key_inconsistent=0`）在加了聚合段之后要重新核一遍——聚合段只增加信息量，不减少读集段已经罩住的东西，不会让原本一致的两个状态变得不一致 |
| torn | 撕裂态要求聚合的水位推进规则与枚举器的撕裂写表一致（`research/prompts/m3-prune-gpu-r1-main-verification.md:55`「撕裂态必须照搬枚举器的撕裂写表」），否则 `research/prompts/m3-prune-gpu-r1-opus-model/outputs/torn.out:2` 里 `states_where_enumerated_slot_differs_from_physical_tearing=2` 那两个状态会在聚合水位判定上出现分歧 |
| kinds、t4、collide、interior、order、reuse、small | 不直接攻 B4 的聚合机制，但间接依赖它：interior 与 order 世界本身就是「合法内容里嵌了看起来像头的字节」的两种变体（interior 是数据单元内容、order 是记录顺序），扫描方向的聚合修的是 interior 那一类的机理 |
| F2（`research/prompts/m3-prune-gpu-r1-f2-order-report.md`） | 不涉及 B4（涉及 B5） |


### B5 记录核对器的身份与记录归属

**①字段与宽**：记录归属改按记录自己的字段判，不按写表位置判。每条 journal 记录解码出 `(instance: u32, checkpoint_txg: u64)`（`check_journal_record` 已经解这两个字段：「instance: read_u32(record, 16)」「checkpoint_txg: read_u64(record, 26)」，`crates/singlefs-checker/src/lib.rs:793`、`:795`），记录归属 = 与它 `(instance, checkpoint_txg)` 相同的那次发布（那次发布的身份同样来自根写的 `root_identity_of_write`，字段布局同源：`crates/singlefs-harness/src/memory_pool.rs:828`「实例代号在偏移 24（4 字节）、checkpoint_txg 在偏移 28（8 字节）」）。**两样都算、对拍**（B5 岔路单第三个候选）：同时保留今天「按写表次序归给排在它后面的第一条根」（`publishes_of_one_recording`，`crates/singlefs-harness/src/memory_pool.rs:993`）算出的归属，两者不一致本身就是一条新判据（见「③」）。

**②罩住的输入**：记录头里的 `(instance, checkpoint_txg)` 字段（内容归属）+ 这条记录在写表里的位置相对哪次根写（位置归属）+ 这条记录与它对应发布的持久状态（是否持久）。

**③漏了会不会同键异判**：**今天（只有位置归属）已经打中**——F2 世界实测：段 0..7 全持久、txg 3 的根持久、txg 3 的两条 journal 记录都不持久，写表按「攻方次序」（根在记录前）交给记录核对器时判 `root_without_record: false`（应该是 `true`），机理是 `publishes_of_one_recording`（`crates/singlefs-harness/src/memory_pool.rs:993`，函数体 `:1000`–`:1009` 遇到 `JournalRecord` 只 `records.push(index)`、遇到 `RootRecordFua` 才把攒的 `records` 打包）按写表次序归发布：`research/prompts/m3-prune-gpu-r1-f2-order-report.md:10`「记录只归给排在它后面的第一条根」，根在记录之前时这两条记录要么归给**下一次**发布、要么（新建文件是最后一次发布时）**谁都不归**，`crash.rs:139` 那道 `!publish.records.is_empty()` 守卫因此整个跳过这次发布的判定（`.claude/kb/checks-owed.md:468`「记录核对器按写表次序把记录归给排在它后面的第一条根，不看记录自己的 txg……层 0 的五样逐状态判定……一样都不红」）——45 个枚举状态里已知 27 个是这一类真洞。改按内容归属（记录自己的 `checkpoint_txg` 与根的 `checkpoint_txg` 比）之后，这条记录归给 txg 3 那次发布，`publish.records` 非空，`!in_place(record)` 为真（两条记录都没持久）而 `in_place(root)` 为真（根持久），`root_without_record` 正确判 `true`。**新的风险**：只换成内容归属，会不会漏掉「记录顺序错乱」这类今天位置归属还能抓到的东西？答：位置归属抓的从来不是「顺序错」，是「凑够记录数」；只要记录内容里的 `(instance, checkpoint_txg)` 本身没被破坏，内容归属永远精确——这是它比位置归属更强的地方，不是弱的地方。

**④谁先发现**：F2 报告已给出复现与判别力自证：「造一份「根持久、它自己的两条 journal 记录都没持久」的崩溃镜像，按攻方次序……断言它判红；判别力自证：把真实现也改成先根后记录，同一形态的 45 个状态里已知 27 个是真洞，改法接上之后须判红这 27 个之一，不许仍是 0」（`.claude/kb/checks-owed.md:468`）。**再加一条新断言**（利用「两样都算、对拍」）：`record_attribution_position_and_content_agree_on_healthy_streams`——对层 0 全部健康（未被攻方改坏）的录制流，位置归属与内容归属必须给出相同的发布分组；一旦某处流出现「同一条记录，位置归属与内容归属指向不同的发布」，这本身就是一个新判定信号（暂命名 `attribution_mismatch`），意味着这条记录在写表里的位置违反了 D16（发布语义） 已定项 7 的持久顺序（记录先于根写、但落在写表位置上却排在另一次发布的根之后——例如乱序重放、或未来某条布局把记录批量提前写），这是**现有代码今天完全没有的一条判定**，是本设计新增的覆盖面。

**⑤版本号放哪**：进 B8：记录归属算法版本（位置 / 内容 / 两者对拍，三选一或全选，写死在这个版本里）；记录头 `(instance, checkpoint_txg)` 字段偏移版本（今天是偏移 16、26，这两个数字随 D23（journal 的角色与格式） 的记录格式走，记录格式若改了偏移，这里要跟着抬）。

**⑥代价**：每条记录多算一次「内容归属 vs 位置归属」比较（两个整数比较，几乎零代价）；`attribution_mismatch` 新判定信号占用判定向量表（B7）里一个新的位。

#### B5 在十二世界上判什么

| 世界 | 判什么 |
|---|---|
| F2（`research/prompts/m3-prune-gpu-r1-f2-order-report.md`） | 直接针对：内容归属把「先根后记录」的 27 个真洞里至少一个判红（③ 已给出机理） |
| order（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/order.out`） | F2 世界的原始复跑（`healthy_first_stream` 与 `root_before_records_mutant` 两个变体，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/order.out:1`、`:13`），改法之后 `root_before_records_mutant` 那一支应从「五样判定都不红」变成「记录核对器判 `root_without_record: true`」 |
| reuse | `red_names_full_local_localimage_fullrecords={"checker:I-2.1": (5, 5, 5), "checker:I-4.8": (5, 5, 5), "checker:I-7.4": (5, 5, 5), "records:claimed_state_missing_unit": (3, 0, 3)}`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/reuse.out:19`）——记录核对器今天已经能判 `claimed_state_missing_unit`，内容归属不改变这一条（它读的是单元副本缺席，不是记录归属），只新增 `attribution_mismatch` 这条独立信号，两者并存、不冲突 |
| 其余（kinds、collide、interior、p5、p6、torn、small、t4、F1） | 不直接攻记录归属；small 世界的 `attack_u4_shared_node` 那一行「segments_whose_publish_label_differs=["10:after_the_last_root->instance1_txg4"]」（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/small.out:3`）提示「发布标签」这个概念已经在攻方代码里存在，内容归属可以直接复用它的计算方式 |


### B6 内容身份（单元内容去重、读集键里的「内容」）

**①字段与宽**：SHA-256，32 字节。放弃今天 E161 装置用的 128 位非密码学摘要，改用抗碰撞散列（B1/B4/B5/B7 全部沿用同一颗 SHA-256 实现，不再各写一份）。不保留「键相同时逐字节比内容」这条回退：在这个规模下（单个内容体 4 KiB–32 KiB，10¹⁰ 级状态）逐字节回退等于要求把全部去重前内容留着以备比对，直接抵消去重的意义；SHA-256 的碰撞概率在这个规模下可忽略，按 B8 的「知情接受」记法记一笔（D24（后台重活能不能卸给 GPU） 已定项 2 那种「绝不能」句式不适用于这里——这是一句拍脑袋，本设计标它是**推的**：残余碰撞概率没有专门算过，只按 SHA-256 的一般密码学强度推）。

**②罩住的输入**：一个「单元」的原始字节（码 1/2/3 头 + 载荷，按声明宽度取，不含它以外的字节）；是内容的纯函数，不依赖被谁读、从哪个位置读。

**③漏了会不会同键异判**：**打中过、已被这条设计吸收**——第一轮攻方在 40.7 秒内用 rho 方法撞出一对 16 KiB 内容共享同一个 128 位摘要（`digests_equal=true`，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/collide.out:1`），「键用的 128 位摘要 40.7 秒撞出一对 16 KiB 内容」（`research/prompts/m3-prune-gpu-r1-main-verification.md:51`）；同一份世界里两份内容的 CRC-32C 不同（`crc32c_first=1ec1ebae crc32c_second=d30c8dc9`，同一行），也就是说「端到端」（走完整个校验和链路）没有被这对碰撞内容骗过，只有「拿 128 位摘要当去重键」这一步会被骗。换成 SHA-256（256 位）之后，40.7 秒能找到的构造方法（rho，代价 O(2^(n/2))）代价从 2^64 抬到 2^128，超出任何可行算力。

**④谁先发现**：collide 世界本身就是判别力自证：`must_be_nonzero=1`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/collide.out:1`）标着「这条攻击必须撞出东西」；本设计新增一条「换算法之后同一攻击代码在同样的挂钟预算内 rho 方法找不到 SHA-256 碰撞」的反向断言（跑一个较短的挂钟上限，比如仍是几十秒，断言 `digests_equal=false`——这只证明「这条特定攻击不再在这个挂钟预算内命中」，不是数学证明，要在报告里如实标「推的」）。

**⑤版本号放哪**：进 B8：内容哈希算法版本（128 位 / SHA-256 二选一，钉死当前版本），一旦以后再换算法（比如换成别的抗碰撞散列），旧库里存的「内容 → id」映射表要整表按新算法重算，version 不匹配时直接拒绝打开（同 B8 设计）。

**⑥代价**：单条记录从 16 字节涨到 32 字节；给内容一个小整数 id 的做法完全照抄 `verdict_store.rs` 已经实现的「判定向量按内容 / 按编号」两张表模式（`crates/singlefs-checker-tier/src/verdict_store.rs:17`「判定向量按内容 | `02` ‖ 判定向量的字节 | 编号，一个字节」、`crates/singlefs-checker-tier/src/verdict_store.rs:18`「判定向量按编号 | `03` ‖ 编号，一个字节 | 判定向量的字节」），只是键从「判定向量字节」换成「内容的 SHA-256」、值从 1 字节编号换成合适宽度的整数（内容去重表的种类数按 E161 g1 表的量级看只有几十种，`research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:181`「second_quick states=84 distinct_unit_contents=58」，1 字节编号在今天的取样域上够用，但没有像 B7 一样有硬编号上限的攻击面证据，仍按 2 字节留余量）。

#### B6 在十二世界上判什么

| 世界 | 判什么 |
|---|---|
| collide | 直接针对：SHA-256 让这条攻击的挂钟预算失效（③④ 已给出） |
| interior、order、reuse、small、t4、torn | 都涉及具体内容的读，但都不是针对哈希算法本身构造攻击，换算法不改变这些世界的既有判定；本设计要求换算法之后 F1/F2/torn 等世界重跑一遍产物逐字节相同（算法只影响「键」，不影响「值」——同一份内容任何哈希算法下判定应该相同） |
| kinds、p5、p6、F1、F2 | 不直接针对内容身份 |


### B7 判定向量身份（一个状态判成什么）

**①字段与宽**：编号从 1 字节改宽到 **2 字节**（大端），块值从「每状态 1 字节」改成「每状态 2 字节」；判定向量表结构不变（`crates/singlefs-checker-tier/src/verdict_store.rs:17`「判定向量按内容 | `02` ‖ 判定向量的字节 | 编号，一个字节」、`crates/singlefs-checker-tier/src/verdict_store.rs:18`「判定向量按编号 | `03` ‖ 编号，一个字节 | 判定向量的字节」两张表的「编号，一个字节」都改成「编号，两个字节」），只是把编号上限从 256 抬到 65536。判定向量内容（`EncodedVerdictVector(Vec<u8>)`）本身不定长，本设计把它的位布局钉死成一个显式的位字段结构：`root_persisted`(1 位)、`recovery_outcome`(2 位：no_file / file_read / failed)、`journal_differing`(1 位)、`verification_ran`(1 位)、`verification_failed`(1 位)、`oracle_violation`(1 位)、`ignored_oracle_violation`(1 位)、`record_root_without_record`(1 位)、`record_claimed_state_missing_unit`(1 位)、`record_attribution_mismatch`(B5 新增，1 位)、`checksum_pass`(GPU/CPU 共用位，C3，1 位)——以上共 12 位，2 字节装得下；**49 条不变量各自的评估 / 违例 / 不适用状态不进这个定长位字段**，理由见「③」。

**②罩住的输入**：这个状态在层 0 五样判定（两遍恢复 + oracle、池级 checker 违例汇总、记录核对器）里、除「逐条不变量」以外的全部结局；不罩：哪一条具体不变量违例（那是「事后要答」的问题，见④）。

**③漏了会不会同键异判**：**不是同键异判问题，是编号耗尽问题**——`register_verdict_vector` 今天在第 257 种判定向量时拒绝写入（`crates/singlefs-checker-tier/src/verdict_store.rs:281`「MoreThan256VerdictVectorsNotDecidedInTheFirstVersion,」，`:508` 触发点），已经有单测钉死这个边界（`the_257th_distinct_verdict_vector_is_refused_and_the_library_is_unchanged`，`crates/singlefs-checker-tier/src/verdict_store.rs:1067`）。风险点是「49 条不变量的评估 / 违例 / 不适用状态，能不能全部塞进同一个判定向量」——摸底 Q2 已经把这问题摆明：「今天 `Layer0Tally` 的各类计数……能不能全部从逐状态向量重算出来」（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:178`）。答案是**不能全塞进定长位字段**：49 条不变量若各自独立地在「评估过 / 违例 / 不适用」三态之间变化，组合数上限是 3^49（远超 65536），而实践中同一个崩溃状态触发的具体是「哪几条不变量违例」——这是一个**变长**集合（0 到 49 条都可能），不是一个可枚举的小型有限集合，用定长编号去逼近它，编号数会随着不变量种类和组合方式膨胀，逼近甚至超过 65536 的风险无法排除（本设计**没有量过**实际会出现多少种组合，这是推的）。

**④谁先发现**：**重构断言**（不是判红断言，是「重构完整性」断言）：新增一条 checker-tier 档测试 `layer0_tally_is_fully_reconstructible_from_stored_verdicts`，跑一段小域，同时用今天的直接计数（`Layer0Tally` 各字段直接 `+= 1`）与「从存进 KV 的判定向量 + 违例表反推」两条路径各自统计一遍，断言二者相等——**这条断言目前会红**：因为③已经论证了 49 条不变量的细粒度信息进不了定长判定向量，所以「哪几条不变量违例」这部分的重构必须走一条独立的路（见「设计」段）。**设计**：判定向量只装「粗粒度」的 12 位（②列的字段），**逐条不变量的违例明细单独进违例表**——`verdict_store.rs` 今天的违例表（`04` ‖ 块键 ‖ 块内序号 → 判定向量编号，`crates/singlefs-checker-tier/src/verdict_store.rs:19`）在这个设计里的值改成「判定向量编号 ‖ 违例的不变量名字列表（变长，逗号分隔的 ASCII 字节串，或更紧凑地一个 64 位 bitset，因为 49 条不变量正好装得进一个 `u64`）」——用一个新的 8 字节字段（不变量违例位图，`u64`，每条不变量占 1 位，49 条用 49 位）随附在违例表的值里，专门解决「哪条不变量违例」这个只在违例状态才需要回答的问题，不进判定向量（判定向量只用一个 `not_applicable_or_violated`（1 位）标「这个状态有没有不变量层面的违例」，具体是哪几条，走违例表的位图字段）。

**⑤版本号放哪**：进 B8：判定向量编号宽度版本（1 字节 / 2 字节）、判定向量内容位字段布局版本（12 个具名位分别占哪一位）、违例表值布局版本（是否带不变量违例位图这个新字段）。

**⑥代价**：块值从 1 字节/状态涨到 2 字节/状态（对应丁事实表已经预留的「或「30GB」要每状态 2 字节」那一档，`research/prompts/m3-prune-gpu-r1-facts-kv.md:200`「g | 「一块取 2¹⁶ 个状态，150 亿状态约 23 万个键、15–30 GB 值」……每状态 2 字节时「30 GB」」——本设计的 2 字节选择与这个既有预估直接对上）；违例表新增 8 字节/违例状态（不变量位图），只在违例状态上付出这个代价，绝大多数判绿状态不受影响。

#### B7 在十二世界上判什么

| 世界 | 判什么 |
|---|---|
| reuse | `red_names_full_local_localimage_fullrecords={"checker:I-2.1": (5, 5, 5), "checker:I-4.8": (5, 5, 5), "checker:I-7.4": (5, 5, 5), "records:claimed_state_missing_unit": (3, 0, 3)}`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/reuse.out:19`）——这条世界同一个状态上**同时**有三条不变量违例（I-2.1、I-4.8、I-7.4），是「不变量违例位图」这个设计最直接的验证场景：判定向量的 12 位只标「有不变量违例」，位图字段要能同时置上这三条不变量各自的位 |
| interior | `red_names_full_local_localimage_fullrecords={"checker:I-7.8": (0, 12, 12)}`（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/interior.out:12`）——单条不变量违例，位图只置一位 |
| p5、p6、torn、collide、kinds、order、small、t4 | 不直接攻判定向量的编号或位布局 |
| F1、F2 | 各自新增的判定信号（F1 归入池级 checker 的 I-7.8 违例，走位图；F2 的 `record_attribution_mismatch` 走判定向量本身的位，见 B5） |


### B8 版本号（每一种身份值与每一张表的定义版本放哪、怎么核）

**①字段与宽**：一个新的元数据键（键标签 `0x00`，在今天已用的 `01`–`04` 之前，`crates/singlefs-checker-tier/src/verdict_store.rs` 键标签表 `:16`–`:19` 目前只用到 `01`–`04`），值是一个固定顺序、只许追加不许改序的 `u16` 数组，每个身份值 / 表布局各占一个位置：`node_identity_version`（B1）、`node_fingerprint_version`（B2）、`state_encoding_version`（B3）、`class_key_version`（B4，含扫描聚合算法版本）、`record_attribution_version`（B5）、`content_hash_version`（B6）、`verdict_vector_layout_version`（B7，含编号宽度与位字段布局）、`block_layout_version`（B9，块键/值布局）。共 8 个 `u16`，16 字节。

**②罩住的输入**：每一个会被固定偏移 / 固定位置解码的字节布局；不罩「会自然产生新键」的哈希类字段（B2 的节点指纹、B6 的内容哈希）本身的算法选择——算法一变，产生的哈希值本身就不同，天然形成新键，不会被静默错读成旧格式（这与 `.claude/rules/fs-design.md:194`「参数化布局的失败模式是**静默误读**，incompat 位的失败模式是**拒绝挂载**」是同一个道理：哈希值改变本身就是「挡在门外」，不需要额外一层版本号去挡）；但算法版本仍然要记（放进对应的身份值版本字段），因为要能回答「这个库是用哪种算法建的」这个问题，也为了让「打开库」这一步能显式核对「我这次编译支持的算法」与「这个库当初写入时用的算法」是不是同一种。

**③漏了会不会同键异判**：**会静默错读，不是同键异判**——如果不设版本号，改了判定向量的位字段布局（B7）之后重新打开一个旧库，旧库里 2 字节判定向量编号会被新代码按新的位字段语义重新解释，得到一个看起来合法、实际错误的判定（这正是 `.claude/rules/fs-design.md:167`–`:170` 那段「一套布局 + 参数」反面例子的同构：「若某个版本改了……的口径，旧读者照样算得出一个偏移、照样读出「看起来合法」的字节 —— 不报错，而数据是错的」）。

**④谁先发现**：打开库时（`VerdictStore::open_existing`，`crates/singlefs-checker-tier/src/verdict_store.rs:453`）新增一步：读元数据键，逐项核对 8 个子版本与编译期常量是否完全相同，任一处不同就返回一个新的错误成员（`LibraryLayoutVersionMismatch { stored: [u16; 8], compiled: [u16; 8] }`），拒绝打开——不修、不猜、不部分兼容，仿照 B7 里 `open_existing` 已有的「打不开修一次」逻辑（`crates/singlefs-checker-tier/src/verdict_store.rs:449`「开一个已有的库；第一次打不开时调一次 `DB::repair` 再开」）单独走一条新分支（版本不对不是「打不开」，是「打开了但不敢信」，两种失败要分开报）。新增单测：建一份库、手改元数据键其中一个子版本字节，断言 `open_existing` 返回 `LibraryLayoutVersionMismatch`（仿照今天已有的 `an_existing_library_whose_current_file_is_gone_opens_after_one_repair` 这类「构造一个坏库、断言特定错误」的测试写法，`crates/singlefs-checker-tier/src/verdict_store.rs:1097`）。

**⑤版本号放哪**：这个问题问的就是它自己——放在库里一个固定键（`0x00`），不进块键、不进每张表分别存一份（分别存会导致「这次写的块」与「库的版本」脱节，写块时忘了检查版本仍能写进去；固定在打开库那一刻检查一次，比每次读写都检查更省，也更不会被绕过——`VerdictStore` 结构体今天已经是「打开一次、后续操作都走 `&mut self`」的形状，`crates/singlefs-checker-tier/src/verdict_store.rs:425`「一个打开着的判定库。同一时刻一个库只许一个进程开（RocksDB 的 LOCK 文件），进程里改库的方法都要 `&mut self`」，版本检查天然只用做一次）。

**⑥代价**：16 字节，一次性，与库大小无关。

#### B8 在十二世界上判什么

十二世界都不直接构造「版本号缺失导致误读」的场景（今天的库还没有任何调用方，见「读法与总纲」一节，没有旧数据可供误读）；这一格留给第二、三轮设计「先建一份旧版本库、再用新代码打开」的世界去攻。

### B9 数据结构（块的单位）

**①字段与宽**：块单位维持「固定 2¹⁶（65536）个状态」不变（沿用 E162 选型时的主取样点；块单位的三个候选——片 / 固定 2¹⁶ 个状态 / 节点——`research/prompts/m3-prune-gpu-r2-forks.md:17`「**片**（续跑与分片的单位）。**固定 2¹⁶ 个状态**。**节点**」），但**寻址方式改成「节点内」**：`VerdictBlockKey.stream_or_node_name`（`crates/singlefs-checker-tier/src/verdict_store.rs:49`「流名（里程碑三以后是节点名）」）填 B1/B9 分配给节点的紧凑 `node_id` 的十进制或定长字节表示；`block_start`（`BlockStartStateIndex`，`crates/singlefs-checker-tier/src/verdict_store.rs:41`「块起点：这一块第一个状态在它那份枚举计划里的序号」）此后解读为「这个节点自己的状态序号」，不再是整条流的全局序号。一个节点如果状态数超过 65536，切成 `⌈状态数 / 65536⌉` 块，块起点是 0、65536、131072……；状态数不足 65536 的节点只有一块，不满。

**②罩住的输入**：一个节点内一段连续状态区间的判定向量。

**③漏了会不会同键异判**：不是同键异判问题，是**效率问题**——粒度很细的节点（段长 1–2 步的节点，比如 order/interior 那条 11 段路径里 5 个段长只有 1 或 2，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/interior.out:1`「segment_lengths=[2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2]」）如果状态数远小于 65536，每个节点只有一个远未写满的块，浪费块键开销（每块键 32+2+名字+32+8 字节的固定部分）但不产生错误。

**④谁先发现**：一条断言 `block_start_plus_block_length_never_exceeds_the_nodes_own_state_count`（越界即 panic，仿照今天已有的「违例状态的块内序号必须在块里」那条 panic 契约，`crates/singlefs-checker-tier/src/verdict_store.rs:573`「违例状态的块内序号必须在块里（调用方给的序号超出了块长）」）。

**⑤版本号放哪**：块布局版本（B8 的 `block_layout_version`）——它现在同时管「块键字节布局」与「块起点的语义（全局序号 vs 节点内序号）」，是本设计对现有代码唯一改变了「已实现且被单测钉住的字节含义」的一处，必须显式抬版本号。

**⑥代价**：键数 = Σ_节点 ⌈该节点状态数 / 65536⌉。段长 1–24 步的节点（十个攻方世界的量级）状态数远小于 65536（p6 世界单元段状态数 4095，`research/prompts/m3-prune-gpu-r1-opus-model/outputs/p6.out:2`），每个节点恰好 1 块；只有真正的大段（U3 提到的 2¹³⁶、约 2³⁰⁰ 那几段）才会切出很多块。

#### B9 在十二世界上判什么

| 世界 | 判什么 |
|---|---|
| p6 | 单元段状态数 4095（`research/prompts/m3-prune-gpu-r1-opus-model/outputs/p6.out:2`），换算到节点内寻址后应恰好落在一块里（4095 < 65536） |
| 其余世界 | 段长普遍在 1–24 步，状态数远小于块大小，都只产生「一个节点一块、不满」的情形，不触发跨块拼接逻辑；跨块拼接的正确性留给第二、三轮用大段（2¹³⁶ 级）攻 |


### 1.10 KV 表全貌（键与值的字节布局，整库汇总）

沿用 `crates/singlefs-checker-tier/src/verdict_store.rs` 已实现的「键的第一个字节是表标签、都放默认列族」这个形状（`crates/singlefs-checker-tier/src/verdict_store.rs:6`「四张表都放在默认列族里、靠键的第一个字节（表标签）分开，不用列族」），本设计新增四张表、扩宽一张已有表的值、新增一个元数据键，整数一律大端：

| 标签 | 表 | 键 | 值 | 这一轮改动 |
|---|---|---|---|---|
| `00` | 库元数据（新增） | 空（整张表只一行，键就是标签本身） | 8 个 `u16`：`node_identity_version、node_fingerprint_version、state_encoding_version、class_key_version、record_attribution_version、content_hash_version、verdict_vector_layout_version、block_layout_version`（B8） | 新增 |
| `01` | 判定块 | 输入指纹 32 字节 ‖ 节点名长 2 字节 ‖ 节点名 ‖ 枚举计划哈希 32 字节 ‖ 块起点 8 字节（今天已实现的布局，`crates/singlefs-checker-tier/src/verdict_store.rs:21`「块键 = 输入指纹（32 字节）‖ 流名或节点名的字节数（2 字节）‖ 流名或节点名 ‖ 枚举计划哈希（32 字节）‖ 块起点（8 字节）」；「节点名」= B1/B9 的紧凑 `node_id`，「块起点」= B9 的节点内序号） | 块里每个状态 **2 字节**（B7，判定向量编号，宽度从 1 字节改成 2 字节） | 值宽度改宽（B7） |
| `02` | 判定向量按内容 | 判定向量的字节（B7 的 12 位定长位字段） | 编号，**2 字节** | 编号宽度改宽（B7） |
| `03` | 判定向量按编号 | 编号，**2 字节** | 判定向量的字节 | 同上 |
| `04` | 违例 | 块键 ‖ 块内序号 8 字节 | 判定向量编号 2 字节 ‖ 不变量违例位图 8 字节（B7 新增字段，49 条不变量各占一位） | 值新增位图字段（B7） |
| `05` | 内容按摘要（新增） | SHA-256 摘要 32 字节（B6） | 内容 id，2 字节 | 新增 |
| `06` | 内容按 id（新增） | 内容 id，2 字节 | SHA-256 摘要 32 字节 | 新增，与 `02`/`03` 同一种「按内容 / 按编号」互逆模式 |
| `07` | 节点按指纹（新增） | 节点指纹 32 字节（B2，SHA-256） | 节点 id（B1/B9 的紧凑整数，4 字节，留够余量给远超 65536 的节点数） | 新增 |
| `08` | 节点按 id（新增） | 节点 id 4 字节 | 节点指纹 32 字节 ‖ 节点身份（B1 的种类串 ‖ 覆盖签名，变长） | 新增，`07`/`08` 同样互逆 |

**版本核对时机**：打开库（`open_existing`）时先读 `00` 号键，8 个子版本逐项与编译期常量比对，任一处不同即 `LibraryLayoutVersionMismatch`（B8 ④）；`create_empty` 建库时写入当前编译期的 8 个子版本。


## 第二题：录入与核对的二元体系

### C1 录入写什么、按什么单位写

**设计**：每状态一格判定向量编号（B7）与「只存类」两样都写，不二选一。理由是它们答的是不同的问题：判定向量表按块存（`01` 号表）答「这个具体状态判成什么」；本设计新增一张**类表**（键标签 `09`，键 = 节点 id 4 字节 ‖ 判定向量编号 2 字节，值 = 这个节点里这种判定向量出现的状态数，4 字节）答「这个节点里各类各有多少个状态」——这张表从已经写进 `01` 号表的块直接聚合算出，不需要额外的录入路径，只是多一步「写完一块之后顺手累加一次类表」。理由是摸底 Q1「录入写下来的原子单位是什么」（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:177`）与 N1「结果只留计数，不留逐状态判定」（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:156`）合起来看：只留计数答不出「这个具体状态判了什么」，只留逐状态又要额外一次扫描才能答「这一类有多少个」——两样都写，各自答各自的问题，代价只是类表本身（远小于块表：类数远小于状态数）。

### C2 核对判据住哪、判据收成一处

**设计**：新增一个「判定分类」纯函数库（`crates/singlefs-checker-tier/src/verdict_classification.rs`，checker 档新文件），输入是原始信号（恢复结局、oracle 结局、checker 违例集合、记录核对器结局、变异跑测结局……），输出是一个封闭枚举 `VerdictClass { Caught, Ineffective, NotRed, Timeout, MemoryExceeded, ProcessIncomplete }`（穷举 `match`，不写通配臂，照 `.claude/singlefs-ai-sop/rules/code-discipline.md`「分支」一节的写法）；`crates-mutation-rows.py`、`prove-red.sh`、`mutate.sh` 三份「各抄一份」的分类逻辑（摸底 N7「结局判法各抄一份。变异的「抓到 / 无效 / 没红」：`crates-mutation-rows.py:649`、`prove-red.sh:75`–`:90`、`mutate.sh:536`，规则 `mutation-sampling.md:80` 又用文字写一遍」，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:162`）改成各自调一个共同的分类二进制（`crates/singlefs-checker-tier/src/bin/verdict-classify`），Python 侧解析它单行 stdout 而不是各自重新判断字符串。这是「一个核对模块，别的都调它」这个候选，理由是穷举 `match` 能让编译器强制三处调用点在新增分类成员时一起补分支，而三份手写字符串匹配今天做不到这一点。

### C3 纯 CPU 与 CPU 加 GPU 逻辑相同到哪一层

**设计**：一个 trait `UnitChecksumBackend`，两个实现（`CpuChecksumBackend`、`GpuChecksumBackend`，后者挂在 GL 新建 crate 里、挂可选特性），两者写**同一种记录**——校验和结果本身只是判定向量（B7）12 位里的 `checksum_pass` 一位，不单独开一张表，GPU 只接这一位、别的 11 位（root_persisted、恢复结局……）仍旧全部在 CPU 侧算（D13（验证路线） 已定项 15「GPU 只接校验和那一截」，`.claude/kb/decisions/13-验证路线.md:318`）；D24（后台重活能不能卸给 GPU） 已定项 2 第三条`.claude/kb/decisions/24-后台重活能不能卸给GPU.md:34`「它是一个新的失败域，卸出去的结果必须能被 CPU 独立复核」（与 `evidence-discipline.md`「校验的两条路径不许共享同一段代码」同构）要求两份实现互相独立、抽样复核——本设计把这条落成 C4 里的「四种跑法在重叠处结果逐项相同」检查的一部分：`checksum_pass` 这一位无论走 CPU 还是 GPU 都必须相同。

### C4 四种跑法

**设计**：单机 CPU / 双机 / 单机 GPU / 双机 GPU 四种模式按 `ENABLE_ACROSS_MACHINES`、`ENABLE_GPU` 两个独立布尔组合（`research/scripts/layer0-shard-configuration-check.sh:16`「ENABLE_ACROSS_MACHINES 双机：写 1 才开；没写或写 0 算关（用户 2026-09-28 定：不设参数不主动开）」、`:17`「ENABLE_GPU GPU：写 1 算开，没写或写 0 算关」），不写就不开（用户定案，见正文第一节表）。新增一条 checker 档断言 `four_run_modes_agree_on_the_overlap_domain`：取一个所有四种模式都能全量跑完的小域，四种模式各跑一遍，断言这个域里每个状态的 2 字节判定向量逐字节相同——这条断言把「GPU 显存不够退回 CPU 那一片的记录与 GPU 那一片同不同形」也一并覆盖：GPU 不可用时退回 CPU 计算 `checksum_pass` 这一位，写出的判定向量与另一台机器用真 GPU 算出的必须相同（因为判定向量的位布局不区分「这一位是 CPU 算的还是 GPU 算的」，B7 设计里故意不留这个区分位）。


### C5 一个入口、三档

**设计**：单点 / 单线的起点镜像**从根重放到父节点**，不从 KV 缓存父节点结束镜像——理由是缓存需要一张新表存整份池镜像字节（可能几十 MiB 一份、按节点数乘），而重放的代价只是 O(路径深度) 次写应用，E161 已经量过「每状态总耗时」在微秒到毫秒级（`research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:216`「second_quick | 79.9 | 227.0 | 198.7 | 18.4 | 8964.6」，单位微秒），单点调试场景对着几十个节点深的路径重放，代价在这个量级下完全可以接受，不必为离线调试用途单独维护一份大体积缓存。三档在重叠处必须相同：新增断言 `single_point_replay_agrees_with_full_enumeration`——同一个目标状态分别用「全量」入口（跑完整棵树，从 KV 读它落进的判定向量）与「单点」入口（`--node <node_id> --state <序号>`，只重放到这个节点、只评这一个状态）各得到一份判定向量，断言逐字节相同。今天最接近的入口只到测试函数一级：「`rerun-failed-tests.py` 按测试函数」（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:164`），用户要的「全量 / 单条线 / 单点」三档「今天都没有」（同一行）。

### C6 复用与作废

**设计**：复用判据统一成「节点指纹（B2）+ 库布局版本（B8）都不变」，**不设时限**——今天至少五种复用口径、时限不一（摸底 N5「复用口径至少五种，时限不一。R1、R2、R7 过 24 小时作废；R4 不过期」，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:160`），这条设计选「不过期」这一档（同 R4 崩溃枚举用例标记「不过期」的先例），因为指纹是精确的（同一路径写表 + 判法版本 ⇒ 同一指纹，B2 ③ 已论证），没有「过陈旧」这回事，只有「指纹变没变」这回事——时限本质上是在没有精确指纹时的一种保守猜测，本设计既然有了精确指纹，不需要再叠加一层猜测。**留下的限制**：本设计**没有解决**「节点走到哪些代码模块没有机械登记」这个问题（摸底 Q5「复用单位与作废范围怎么对上……算不到时作废的粒度取多大」，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:181`）——判法版本（B8 的一个子版本）改一次，就让全部节点的指纹级联失效（因为 B2 的递归定义里，判法摘要是每个节点指纹计算的输入之一），这比「只让真正受影响的节点失效」更粗，但比今天精确（摸底 N4 `research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:159`「改一行 core、加一个依赖、加 `.cargo/config.toml`，8 条用例全部重跑」）：至少改一行不涉及判定逻辑的代码（比如改一行注释、改一份不参与编译的脚本）不会级联失效，因为判法版本是一个显式维护的整数，不是「整棵 `crates/` 的文件哈希」。

### C7 门禁 59 号怎么放进这一套

**设计**：**只共用调度与 C2 的判定分类模块，不共用存储与分片单位**——变异行（`crates/mutations.tsv` 的一行）与崩溃状态（节点 + 序号）是两种不同的寻址空间，没有自然的交叉点，59 号继续用它自己的按条记录（`crates-mutation-rows.py`）而不是 KV 判定块；但 59 号的「抓到 / 无效 / 没红」判定改调 C2 的分类二进制（今天「结局判法各抄一份」，摸底 N7，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:162`），59 号的双机分片改迁到 `peer-host-lib.sh`——今天只有 `mutation-shard-run.sh` 与 `multi-host-run.sh` 用它，`layer0-shard-run.sh` 自己一份 ssh / rsync（摸底 N8「跨机驱动三份。`layer0-shard-run.sh` 自己一份 ssh / rsync / 清场 / 停进程树」，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:163`），本设计把 `layer0-shard-run.sh` 也迁到 `peer-host-lib.sh` 上，两条跨机驱动收成一条公共库 + 两个薄壳。这正面回答了摸底 Q13「变异整表复跑（59）与崩溃状态放量是不是同一套……研究侧 `mutate.sh` 并不并进来」（`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:189`）：并的是调度基础设施（跨机原语、判定分类），不并存储；`research/scripts/mutate.sh`（研究侧变异表，不是 `crates/mutations.tsv` 那一份）保持独立，不并进来（它的对象是实验二进制，不是 checker 档，两者的重型测试闸判法本来就不同）。

### C8 散在外面的与收尾

- **崩溃注入 / 坏盘输入 / 故障注入大档**：今天没有门禁跑（摸底 N10「有几块放量没有门禁跑。崩溃注入大档、坏盘输入大档、故障注入大档都标 ignore、没登记」，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:165`）。**设计**：把它们的「种子区间」当成一种探路型叶子节点（不进精确剪枝的主树，但共用同一个 KV 库、同一套判定分类），这样它们的发现也进同一份「按签名去重的发现日志」，不再是「只在 git 忽略的 `research/scripts/memory-peaks.tsv` 里有手跑留下的峰值行」（同一行）这种没有登记的状态。
- **KV 特性默认编不编**（摸底 Q15，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:191`）：**维持默认不编**，理由是冷编译代价（「冷编 release 不开特性 31.9 秒、开特性 2 分 41 秒到 3 分 7 秒」，见「读法与总纲」一节引的实现现状）；本设计要求接入完成之后，`crash-verifier` 跑全量（`54 --full`）时的命令改成显式带 `--features verdict-store`，把「谁在跑它」（摸底 N12「54 快档、`check.sh` 的 `cargo test --all` 都不带 `--features verdict-store`」，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:167`）从「没人跑」改成「全量档才跑」。
- **门禁改全名之后按编号认阶段**（摸底 N11，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:166`）：**设计**——把 `.claude/hooks/lib_heavy_tests.py` 认阶段用的正则从要求前导编号改成允许有编号或没有（`\.claude/gate\.d/(?:\d+-)?[^/]+\.sh$`），把 `crates-mutation-rows.py:96` 写死的 `STAGE = "59-crates-mutation-replay.sh"` 改成从自己的文件路径动态取（`os.path.basename(...)` 或等价），把 `stage-owners.tsv` 里旧编号的两行（`:17`、`:27`）改成合并之后的现名——这三处是具体改哪个文件的哪一步，属于回扫，不属于这一轮设计题本身，交给下一轮实现。
- **整目录登记要不要收窄**（摸底 Q16，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:192`）：**设计建议，标「推的」**——55、87 把 `research/scripts/`、`research/results/` 整目录登记进指纹（摸底 N15「登记路径宽到整目录……改任何研究脚本、入库任何产物，55 的六台虚机都重起」，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:170`）可以收窄成「这个阶段实际 `source`/调用到的脚本清单」（一份显式 allowlist），但这条改动的收益要用「过去N次门禁运行里因为改动了目录里与这个阶段无关的脚本而被迫重跑」这个数来判断值不值得，本设计没有量这个数，留给以后一轮决定要不要做。
- **限时算法两种**（摸底 N16，`research/prompts/m3-prune-gpu-r2-facts-scale-up-inventory.md:171`）：并入 C2 的判定分类模块——`mutate.sh` 在包装外面套 `timeout`（含排队时间）与 59 号把限时交给包装（不含排队）算的是两个不同的量，本设计要求两者都改成「限时只算包装内的挂钟，不含排队」这一种口径（与 59 号今天已经在用的口径一致），`mutate.sh` 那一处改法属于实现，交下一轮。


## 第三题：分片读写成立

### D1 两台机器往 KV 录入

**设计**：**各写一份，跑完按块导入**（不是「一个库经网络送块」——E162 S3 那条路「没有有效数」，`research/prompts/_m3-prune-gpu-r2-body.md` 第 56 行「今天三种合法：账本 merge、记录 import、查缺查重」）。每台机器各自开一份**独立的 RocksDB 库实例**，分工按 **节点 id 取模**（`node_id % 2`，与今天 `layer0_progress.rs:186`「`shard_owning_slice`」按切片序号取模同一形状，只是把「片号」换成「节点号」）——这个分工只依赖 `node_id`，与线程数无关，换线程数重跑不改变哪台机器负责哪些节点。跑完之后，第二台把它的库目录整份（或用 RocksDB 自带的 `Checkpoint`/`Ingest` 机制，具体选哪种留给实现）传回本机，本机对每一块调用 `store_verdict_block`（B7 已实现的接口）逐块导入——**这一步天然复用了 `verdict_store.rs` 已经实现的冲突检测**：同一个块键下内容相同 ⇒ `AlreadyStoredIdentically`（`crates/singlefs-checker-tier/src/verdict_store.rs:251`「这个块键下已经存着逐字节相同的块与相同的违例，什么都没写（续跑时重写同一块走这一格）」）；内容不同 ⇒ `DifferentVerdictsUnderAStoredBlockKeyNotDecidedInTheFirstVersion`（同一份的一个变体，`crates/singlefs-checker-tier/src/verdict_store.rs:284`）——**导入过程本身就是「合完与单机逐项相同」的检查**：如果两台机器各自算出的同一个节点（理论上不该被两台都算到，因为分工按 `node_id % 2` 互斥，但如果分工有 bug 导致重叠）给出不同判定，导入这一步会直接报错，不会静默覆盖。

**某台被杀、重跑、换线程数**：块不重不漏不变——「不重不漏」由块键的确定性寻址保证（B1/B9：同一个节点、同一个块起点永远映射到同一个块键，与哪台机器、跑几次、用几个线程无关）；「不变」由 `AlreadyStoredIdentically` 的字节级比较保证（同一个块键下重算出逐字节相同的内容才允许免写，不同就报错，不会用新计算结果覆盖旧的）。新增一条 checker-tier 档测试 `killed_and_restarted_with_a_different_thread_count_produces_the_same_library_content`：模拟「跑到一半杀掉、换线程数重跑、导入」，与「单机一次性跑完」的库内容逐块（键 + 值）比对，仿照今天 `layer0-shard-run.sh` 的 merge 自检形状（「⑥ 本机 `SINGLEFS_LAYER0_SHARD=merge/2` 再跑同一条用例：不枚举，读 n 份账本、核文件头逐字段相同、核切片 0..S 每个恰好一次」，`research/prompts/m3-prune-gpu-r1-facts-kv.md:153`）。

**第二台的内存上限与 GPU 卡进同一份多机配置**：`multi-host.env.example` 已经有 `PEER_MEMORY_CAP=24G`（`multi-host.env.example:15`）这一种「第二台的资源上限写在同一份配置里」的先例，本设计照这个形状新增一行 `PEER_GPU_ADAPTER_INDEX`（对应 E163 装置 `--adapter-index` 参数选哪张卡，`research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:148`「多卡切片 `command_run_shard` | rs:656 起 | 一张卡一个进程：`--input 批文件 --start --count --adapter-index --output`」），归进 `MULTI_HOST_SWITCH_KEYS`/配置读取脚本同一张表（具体改哪一行属于实现，交下一轮）。

**共用问句**：①精确性——D1 不改变精确剪枝的定义域，只改变「谁算哪个节点」，精确性由 B1–B9 的身份值设计保证，不因分片而降低；②省下多少——本设计**没有量**双台并行相对单台的实际加速比（要量的是两台各自的判定产出速率，「第二台 24 800 状态/秒」这类数已有先例，`research/prompts/m3-prune-gpu-r1-facts-kv.md:259`「门禁 54 号第二条流「本机 23,700 个/秒、第二台 24,800 个/秒」」），本设计推的理论上界是「两台之和」，实际要扣掉传库、导入、`AlreadyStoredIdentically` 逐块比对的开销，这部分代价没有量过，标「缺这个数」；③坏了谁先发现——`killed_and_restarted_with_a_different_thread_count_produces_the_same_library_content` 与导入步骤自带的冲突检测。


## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| B1 | 设计：种类串 ‖ 覆盖关系签名（4 位重叠标志），用脚本核过分开了 kinds 世界的合并 | 只用种类串会把「根槽第一次写」与「根环回卷」并成一个，加一位「与更早的 RootRecordFua 重叠」就分开 |
| B2 | 设计：父指纹 = 父路径写表哈希（递归定义），不是父结束镜像内容哈希 | t4 世界证明「镜像内容相同」盖不住「写历史不同」，改成写表哈希即可分开 |
| B3 | 设计：作用域从整条流缩小到节点，编码算法不变（S1 已站住） | 沿用今天的位图与混合进制算法，只改「哪个范围内编号」 |
| B4 | 设计：读集键（滚动哈希）+ 扫描聚合段（水位推进，修 C587） | 只有读集键仍会被 F1 的伪造头攻破，聚合段是必需的第二部分 |
| B5 | 设计：记录归属改按记录自己的 `(instance, checkpoint_txg)` 字段判，位置归属保留做对拍 | 修 F2：先根后记录时今天全部判定都不红，内容归属能抓住 |
| B6 | 设计：128 位摘要换成 SHA-256，不留逐字节回退 | collide 世界 40.7 秒撞出 128 位碰撞，换算法把攻击代价抬到不可行 |
| B7 | 设计：编号 1 字节改 2 字节；49 条不变量明细搬出判定向量、进违例表的位图字段 | 49 条不变量独立变化的组合数远超 256／65536，定长向量装不下，拆成「粗粒度向量 + 细粒度位图」两截 |
| B8 | 设计：新增元数据键（`00`），8 个 `u16` 子版本，任一不匹配即拒绝打开 | 今天的库完全没有版本号，这是从零设计 |
| B9 | 设计：块单位维持固定 2¹⁶，寻址方式从「全局序号」改成「节点内序号」 | 只改寻址语义，不改块大小本身 |
| C1 | 设计：每状态一格 + 类表两样都写 | 只写一样答不出另一类问题，两样都写代价可控 |
| C2 | 设计：新增判定分类模块，59/54/`mutate.sh` 都调它 | 修「结局判法各抄一份」（N7），穷举 `match` 逼调用点同步 |
| C3 | 设计：一个 trait 两份实现，只把 `checksum_pass` 一位交给 GPU | 与 D13 已定项 15、D24 已定项 2 一致，GPU 只接校验和那一截 |
| C4 | 设计：四种跑法在重叠域上判定向量逐状态必须相同 | 覆盖 D24 已定项 2 第三条「结果必须能被 CPU 独立复核」 |
| C5 | 设计：单点 / 单线从根重放，不缓存父节点镜像 | 代价可接受（微秒到毫秒级/状态），避免维护一份大体积镜像缓存表 |
| C6 | 设计：复用判据 = 节点指纹 + 库版本，不设时限 | 指纹精确，时限是没有精确指纹时的保守猜测，不再需要 |
| C7 | 设计：59 只共用调度（`peer-host-lib.sh`）与判定分类模块，不共用存储 | 变异行与崩溃状态是两种寻址空间，没有自然交叉点 |
| C8 | 设计：多项散点各给出具体改法（大档接入探路型节点、KV 特性默认不编、按编号认阶段改正则、整目录登记收窄留待以后量数） | 逐条列在正文，多数是「改哪个文件的哪一步」的回扫级建议 |
| D1 | 设计：各写各的库、按 `node_id % 2` 分工、导入时复用现成的字节级冲突检测 | 分工与线程数无关，导入步骤本身就是「合完与单机相同」的检查 |

## 没做什么

- **不判别的腿的格**：不判本地攻方的 B3、B6、B7、B9、C4 代价算术，不判云端攻方对 B1–B7、D1 精确性的攻击面，那两条腿各自派发、各自出报告。
- **不实测**：本设计的十几处「新增断言」（`node_identity_distinguishes_first_root_write_from_root_ring_wraparound`、`node_fingerprint_separates_t4_variants_with_identical_parent_mirror`、`layer0_tally_is_fully_reconstructible_from_stored_verdicts`……）**一条代码都没写、没编译、没跑**——只在 `/tmp/claude-1000/m3-prune-gpu-r2-sonnet/derive_b1_overlap_signature.py` 用纯 Python 字符串解析 + 算术核过 B1 那一条（对已有的 `kinds.out` 原始输出重算，不改代码、不跑 `crates/`），其余全靠对第一轮攻方十个世界产物文件的原始字段做算术推导；这是设计轮，不是实现轮，跑前登记与实现留给下一轮。
- **不判本轮之外的题**：正文第一节表里「用户定案」那几条（不抽样、崩溃放量集中一处、录入核对二元体系、四种跑法开关只从配置读、KV 选 RocksDB）不重新论证，只设计怎么做。
- **不改仓里任何文件**：这一轮只读 `crates/`、kb、`research/scripts/`、`research/prompts/` 里已有的文件，没有 `Edit`/`Write` 工具，草稿目录 `/tmp/claude-1000/m3-prune-gpu-r2-sonnet/` 只放了一个 python 脚本，没有编译目录、没有仓副本、没有工作树。
- **没有跑 E161 装置副本**：派发提示允许「在 E161 装置或第一轮攻方模型的副本上跑小域」，但本轮的验证需求（核对「加一位重叠签名能不能分开两个条件签名」）已有第一轮攻方产出的原始文本可用（`kinds.out`），不需要重新编译运行来取得这个数；其余身份值的验证（B2/B4/B5/B6/B7/B9）引用的都是第一轮攻方与摸底事实表已经跑出的数字，本轮没有再跑一遍。
- **B4/C7/C8 里标「推的」的几处**（SHA-256 残余碰撞风险接受、59 号只共用调度不共用存储的取舍、整目录登记收窄的收益）都还没有量出支撑的数字，交下一轮或交用户定。

