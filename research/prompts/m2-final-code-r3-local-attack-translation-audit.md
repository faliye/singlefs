# 转述核对表：m2-final-code-r3-local-attack（2026-09-25 UTC）

逐句核对 `research/prompts/m2-final-code-r3-local-attack.md`（本地攻方：分配记录树 W 与内部扇出、
根层公式在三种盘配置上各取几、extent 树三个宽度、145 单元文件下段几层，五道算术题）里每一条
fact / item 与原文。行号现查冻结副本 `/tmp/claude-1000/m2-final-code-r3/tree/crates/` 与 kb 快照
`/tmp/claude-1000/m2-final-code-r3/kb-snapshot/`（本轮材料员已核对这两份快照与主工作区当时逐字节
相同，`kb-sha256.txt`、`crates-src-sha256.txt`）。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节，以及派发提示「提示里
明令答复不写代码行号与文件行号」）；提示里指代源码一律只用常量名或函数名（如
`ALLOCATION_RECORD_BYTES`、`lower_root_level_for`），只在这份核对表里写死来源文件与行号。核对表
与运行记录里，样本自带的行号（若模型自己写出行号）一律标「模型自给、未核」。

事实表来源现查（不从背景材料数，行号已用 Read 工具的编号输出现核，见下方「命令与输出」一节）：
`crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/allocator.rs`、
`crates/singlefs-core/src/allocation_record_tree.rs`、`crates/singlefs-core/src/extent_tree.rs`，
均取冻结副本 `/tmp/claude-1000/m2-final-code-r3/tree/crates/` 下的版本。

## 逐条核对

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| Fact 1：节点字节数与槽字节数均 16384 | `crates/singlefs-format/src/lib.rs:11-12`（`/// 落点粒度：一个 16 KiB 槽...` `pub const SLOT_BYTES: u64 = 16384;`）与 `:14-15`（`/// 索引节点（码 2）恒 16 KiB...` `pub const NODE_BYTES: u64 = 16384;`） | 无遗漏；「两个分别命名、恰好同值」这句是我自己加的说明，见下方「多出来的限定」 | 按字面译两个常量的定义与数值，不摘句 |
| Fact 2：分配记录树节点头宽 135（含 29 字节预留位） | `crates/singlefs-format/src/lib.rs:65-66`（`/// 分配记录树节点（码 2）的头：86 + 2 × 10 + 29，key 宽 10...` `pub const ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES: u64 = 135;`） | 首稿未译出注释里的具体分解式「86 + 2 × 10 + 29」与「key 宽 10」，只译了最终值 135 与「含 29 字节预留位」这一限定 | 分解式（86/2×10/29 的构成）对本轮五道算术题不承重——题目只用 135 这个总值做减法，不需要知道它由哪几段相加；「含 29 字节预留位」这一限定必须保留，否则模型可能误以为 135 是不含预留位的「明文头」（另一个数，86 + 2×10 = 106），从而把 16384 − 135 与 16384 − 106 弄混 |
| Fact 3：extent 树节点头宽 163（同样含 29 字节预留位） | `crates/singlefs-format/src/lib.rs:62-63`（`/// extent 树节点（码 2）的头：86 + 2 × 24 + 29，key 宽 24...` `pub const EXTENT_TREE_INDEX_NODE_HEADER_BYTES: u64 = 163;`） | 同 Fact 2，未译分解式，只译总值 163 与「含预留位」限定 | 同 Fact 2 的理由 |
| Fact 4：分配记录树叶条目宽 20 | `crates/singlefs-format/src/lib.rs:122-123`（`/// 分配记录一条：key 10 + value 10...` `pub const ALLOCATION_RECORD_BYTES: u64 = 20;`） | 未译「key 10 + value 10」的分解 | 分解对本题不承重，题目只用 20 做除数 |
| Fact 5：分配记录树内部条目宽 96 | `crates/singlefs-format/src/lib.rs:140-141`（`/// 分配记录树内部节点条目：孩子罩的那一段的起点 key 10 + 子指针 86...` `pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;`） | 未译「key 10 + 子指针 86」的分解，也未译注释里提到的 D8 已定项 11 出处编号 | 分解与决策编号均不承重；决策编号会引入模型未加载的上下文 |
| Fact 6：extent 树上段叶条目宽 113 | `crates/singlefs-format/src/lib.rs:156-158`（`/// extent 树上段叶条目：key 24...+ 标签 1 + 载荷 88...` `pub const EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES: u64 = 113;`） | 未译标签字节与载荷字节的内部分解（标签 1、载荷 88、以及标签取值的三种含义） | 内部分解与「标签」语义对五道算术题（只做减法除法）不承重；只保留「上段叶、一条、113 字节」这一层 |
| Fact 7：extent 树下段叶条目宽 112 | `crates/singlefs-format/src/lib.rs:98-99`（`/// extent 树叶记录：key 24 + 数据指针 88...` `pub const EXTENT_LEAF_RECORD_BYTES: u64 = 112;`） | 未译「key 24 + 数据指针 88」分解 | 同上，不承重 |
| Fact 8：extent 树内部条目宽 110（两段共用） | `crates/singlefs-format/src/lib.rs:146-148`（`/// extent 树内部节点条目（上段与下段同一种）：孩子罩的那一段的起点 key 24 + 子指针 86...` `pub const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = 110;`） | 「上段与下段同一种」这一限定已译入（"the same figure for both segments"）；未译分解式与「用户 2026-09-24 定」的出处 | 分解式与定案日期不承重；「两段共用同一个数」这一限定必须保留，否则模型可能以为要分别求两个不同的内部扇出 |
| Fact 9：单元区起点 50176 与「单元区槽数」公式 | `crates/singlefs-format/src/lib.rs:284-285`（`/// 单元区从槽 50176（784 MiB）起...` `pub const UNIT_AREA_START_SLOT: u64 = 50176;`）与 `crates/singlefs-core/src/allocator.rs:217-223`（`/// 一块盘的单元区有几个 16 KiB 槽：从 UNIT_AREA_START_SLOT 起到设备末尾...` `pub fn unit_area_slots_of_device(device_bytes: u64) -> u64 { device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT }`） | 未译「784 MiB」这个换算旁注，也未译函数注释里「这是这个量唯一的一处定义...两处手抄会分叉」那句维护性告诫 | 两处都不影响本题的算术，「唯一定义处」的告诫是给实现者看的维护提示，与五道题的计算步骤无关 |
| Fact 10：4 GiB 盘单元区槽数实测值 211968 | `crates/singlefs-core/src/allocator.rs:1436-1439`（`fn unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments() { let map = DeviceFreeMap::new(DeviceIdentity(0), 4 << 30); assert_eq!(map.unit_area_slots(), 211_968); ... }`） | 未译同一测试里紧跟着的另外两条断言（`empty_segments()`＝3312、`free_runs()`＝1）——那两个数与本轮五道题无关 | 只取与「单元区槽数」这一个量相关的那一条断言，其余两条断言测的是别的量（空闲段数、空闲游程数），删去不算摘句（同一测试函数体内三条独立断言，各自可拆开引用） |
| Fact 11：根层几何函数实际吃的是盘的总槽数，不是单元区槽数 | `crates/singlefs-core/src/allocation_record_tree.rs:86`（`/// 从每块盘的绝对槽数（设备字节数 ÷ 16384）算。`）、`:91-113`（`fn of_devices(slots_of_each_device: &[(DeviceIdentity, u64)]) -> Self { ... }`）与 `:115-129`（`fn of_allocator(allocator: &PoolAllocator) -> Self { let slots: Vec<...> = allocator.devices.iter().map(|device_map| (device_map.device, UNIT_AREA_START_SLOT + device_map.unit_area_slots())).collect(); Self::of_devices(&slots) }`） | 无中文原句可逐句核对——这是我自己读三段源码合成的一条推论性事实，kb 的已定项 14 原文（见下方「多出来的限定」）从未点破这个区别 | 这条不是「转述某一句中文」，而是「把三段源码的逻辑关系写成一句英文陈述」；写法上我核对过：`of_devices` 的文档字符串逐字是「设备字节数 ÷ 16384」，`of_allocator` 逐字是 `UNIT_AREA_START_SLOT + device_map.unit_area_slots()`，两者相加确实抵消了 `unit_area_slots()` 内部的减法，我在 Fact 11 里逐步写出了这个抵消关系而不是直接下结论，模型可以自己核这一步代数 |
| Fact 12：两个真实测试用「总槽数」（未减单元区起点）构造 4 GiB×2 与 1 TiB×2 的假设盘池 | `crates/singlefs-core/src/allocation_record_tree.rs:885-891`（`fn geometry_of_two_four_gibibyte_devices() -> AllocationRecordTreeGeometry { let slots = (4u64 << 30) / SLOT_BYTES; AllocationRecordTreeGeometry::of_devices(&[(DeviceIdentity(0), slots), (DeviceIdentity(1), slots)]) }`）与 `:902-905`（`fn two_one_tebibyte_devices_give_a_tree_of_height_four() { let slots = (1u64 << 40) / SLOT_BYTES; ... }`） | **有意删掉**这两个测试函数体里紧接着的断言行（`:896-900` 的 `assert_eq!(geometry.root_level(), 2); assert_eq!(geometry.height(), 3); ...` 与 `:906-910` 的 `assert_eq!(geometry.height(), 4);`），只保留 `let slots = ...` 那一行 | **必须删**：那两行断言直接就是 item 3 要求模型独立算出的答案（4 GiB×2 的根层与树高、1 TiB×2 的树高），照抄等于替它交卷。这是本核对表里唯一一处「故意不抄全函数体」的地方，特此在这一行与下面「有意省略」一节重复标注 |
| Fact 13：extent 树下段分层的一般规则（span / root level / height 的定义） | `crates/singlefs-core/src/extent_tree.rs:79-85`（`/// 下段层级 L 上一个节点罩几个数据单元：144 × F^L...` `pub fn lower_span_in_data_units(level: u8) -> u64 { (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)) }`）、`:95-101`（`lower_root_level_for` 函数）与 `:521-527`（`lower_height` 函数，文档「第一个文件下段的高；内联（没有下段）时 0」） | **有意不点名** `EXTENT_TREE_LOWER_LEAF_DATA_UNITS`＝144、`EXTENT_TREE_INTERNAL_FANOUT`＝147 这两个具体数值，只译函数的形状（「层级 0 的跨度＝叶容量本身，层级 L 的跨度＝上一层跨度乘内部扇出」），把「叶容量」「内部扇出」都改写成「你在 item 4 算出的那个数」 | 这两个常量的具体值（144、147）正是 item 4 要模型自己算出来的两个答案；Fact 13 若直接写出 144、147，item 5 就变成照抄常量表，不再是独立推导。函数体本身（`fold` 逐层相乘、`find` 找最小满足的层级）不含具体数值，翻成一般规则是安全的 |

## Item 1 至 Item 5 与原文的对照

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| Item 1：W = (16384 − 135) ÷ 20 取整 | `research/prompts/_m2-final-code-r3-background.md:102`（分工表本地攻方一行：「① W = (16384 − 135) ÷ 20 取整」）；算式的字面出处是 `.claude/kb/decisions/08-核心索引结构.md:390`（kb 快照，「叶宽 W = 812（取叶条目容量 (16384 − 135) ÷ 20 本身）」） | **有意删掉** kb 原文里已经写明的答案「812」，只保留算式本身（16384 − 135 ÷ 20 取整） | 812 是 item 1 要模型自己算出的答案；kb 原文把算式和答案写在同一句里，我只取算式、把答案切掉，这是本轮翻译里第二处「有意不抄全句」，与 Fact 12/13 同族 |
| Item 2：分配记录树内部扇出 (16384 − 135) ÷ 96 | `research/prompts/_m2-final-code-r3-background.md:102`（「② 分配记录树内部扇出 (16384 − 135) ÷ 96」）；字面出处同上 kb 快照 `:390`（「内部条目 96 字节...扇出 169」） | 同 Item 1，**有意删掉**答案「169」，只留算式 | 169 是 item 2 的答案；额外要求模型在 item 3 里核对自己算出的 169 是否与 item 3 公式里字面写的「169」一致——这一句核对要求是我加的，原文没有，见下方「多出来的限定」 |
| Item 3：根层公式「最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169」在 240 槽单盘、4 GiB × 2、1 TiB × 2 上各取几 | 本轮派发任务原文（主 agent 消息第四点「本地攻方」一行）与字面出处 `.claude/kb/decisions/08-核心索引结构.md:395`（kb 快照，「根罩整个 key 空间、按盘分流；根层取最小的 R ≥ 1，使 Σ_盘 ⌈盘上槽数 ÷ (W × 169^(R−1))⌉ ≤ 169。4 GiB × 2 时根在第 2 层、树高 3。」） | **有意删掉**同一句后半「4 GiB × 2 时根在第 2 层、树高 3」——这正是 item 3 里三个配置之一要独立算出的答案 | 与 Item 1/2 同族的必要省略；公式本身（Σ、⌈⌉、W × 169^(R−1)、≤169）逐字译出，只切掉了紧跟其后的一个已算好的例子 |
| Item 3 补充：「盘上槽数按 kb 快照与冻结副本现查」——「总槽数」与「单元区槽数」两种读法都要算、都要报是否一致 | 派发任务原文的括注「（盘上槽数按 kb 快照与冻结副本现查）」 | 这句括注本身没有指明「盘上槽数」是总槽数还是单元区槽数；**多出来的**是 Fact 11/12 揭示的「两种读法」与 item 3 里「两种都算、报告是否一致」的要求 | 见下方「多出来的限定」一节的详细说明；这不是翻译遗漏，是我从源码里现查出的一个原文没有点破的岔路，必须让模型自己把两种读法都摆出来，不能替它先选一种 |
| Item 4：extent 上段叶 (16384 − 163) ÷ 113、下段叶 (16384 − 163) ÷ 112、内部扇出 (16384 − 163) ÷ 110 | `research/prompts/_m2-final-code-r3-background.md:102`（「④ extent 上段叶 (16384 − 163) ÷ 113、下段叶 (16384 − 163) ÷ 112、内部扇出 (16384 − 163) ÷ 110」）；字面出处 kb 快照 `:397、399、403`（「上段叶条目 113 字节...」「上段叶罩 143 个 inode，内部条目 110 字节...扇出 147」「下段叶罩 144 个单元...」） | **有意删掉** kb 原文三处已经写明的答案「143」「147」「144」，只留三条算式 | 143/147/144 正是 item 4 要模型自己算出的三个答案，与 Item 1/2/3 同一族的必要省略 |
| Item 5：145 个单元的文件下段几层 | `research/prompts/_m2-final-code-r3-background.md:102`（「⑤ 145 个单元的文件下段几层」） | **有意不引** `crates/singlefs-core/src/extent_tree.rs:1130-1146` 那条现成的单元测试（`a_lower_segment_grows_a_level_past_one_hundred_and_forty_four_data_units`，其中 `lower_segment_nodes_of_a_file_without_holes(145)` 直接断言出「层级 0 两个、层级 1 一个」这个答案） | 那条测试的断言就是 item 5 的标准答案；一个字都不能进提示，否则整道题变成照抄测试文件。Fact 13 只给一般规则（span 的递推定义、root level 的搜索定义），不给这条测试的具体断言 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| Fact 1 结尾「These are two separately named quantities... that happen to carry the same numeric value」 | `crates/singlefs-format/src/lib.rs:11-12`、`:14-15` 均无这句话 | 我自己加的一句说明 | `SLOT_BYTES` 与 `NODE_BYTES` 是两个独立命名的常量，源码里从未写过一句「这两个常量恰好同值」；不加这句，模型可能以为两者本来就是同一个符号的两个名字，从而在后续算式（例如 item 3 需要用槽字节数换算盘容量、item 1 需要用节点字节数做减法）时分不清该用哪一个名字所指的量，虽然数值相同、结果不受影响，但这个说明能防止模型在解释步骤里把两件事混成一件事 |
| Fact 11 全文（见上一节「Item 3 补充」一行已展开的理由） | 无对应的单一 kb 原句，是我合并 `allocation_record_tree.rs:86、91-113、115-129` 三段源码代数关系得出的一条陈述 | 整条 Fact 11 都是「多出来的」，kb 的已定项 14（`.claude/kb/decisions/08-核心索引结构.md:395`）只写「Σ_盘 ⌈盘上槽数 ÷ ...⌉」，从未说明「盘上槽数」具体是总槽数还是单元区槽数 | 不加这条，模型只能靠自己去猜「盘上槽数」该用总槽数还是单元区槽数，猜错了会导致 item 3 的计算前提本身就错，而这不是模型的算术错误、是提示没把岔路交代清楚；加上这条并把两种读法都摆出来（item 3 主体部分明确要求两种都算），把这个岔路留给模型自己在答案里报告，而不是替它先做决定 |
| Item 2 结尾「State explicitly whether this result equals the number 169 that appears literally inside item 3's formula below」 | `research/prompts/_m2-final-code-r3-background.md:102` 与 kb 快照 `:395` 均无这句交叉核对的要求 | 我自己加的一句核对要求 | item 2 的答案（内部扇出）与 item 3 公式里字面写的常数「169」理论上应该是同一个数；加这句是为了让模型自己检查这两处是否一致，这正是「本地攻方·算术」这条腿本身要做的事——不是我替它下结论，是我把「你应该检查这一点」写清楚，模型检查出的结果（一致或不一致）原样收回，不由我判断 |
| Item 3 主体部分「for configurations B and C, apply it twice... state explicitly... whether the total-slot-count way and the unit-area-slot-count way produce the same value of R or two different values of R」 | 同上，派发任务原文与 kb 快照都没有要求「两种算法都跑一遍并报告是否一致」这句话 | 我自己加的一套计算与核对要求 | 与 Fact 11 那条「多出来的事实」配套：既然源码里存在「盘上槽数」两种可能读法的岔路，就必须让模型把两条路都走一遍、如实报告两条路的结果是否一致，而不是替它挑一条路。这不是我替它做判断，只是把「两条路都要走」这件事从「我可能会漏掉的一步」变成「提示里写明必须做的一步」 |

## 有意省略的清单（避免泄题，逐条重复标注一次）

以下五处答案性质的原文内容，**一个字都没有进入提示**：kb 快照 `08-核心索引结构.md:390` 里的「812」、
`:395` 里的「4 GiB × 2 时根在第 2 层、树高 3」、`:399` 里的「143」与「147」、`:403` 里的「144」，
以及冻结副本 `crates/singlefs-core/src/extent_tree.rs:1130-1146` 那条 145 单元测试的具体断言、
`crates/singlefs-core/src/allocation_record_tree.rs:893-900` 与 `:902-910` 两条测试里根层/树高的
断言行。这些内容原本都可以直接摘出来充当「事实」，但它们本身就是本轮五道算术题要求模型独立算出的
答案，一旦进入提示就等于替模型完成任务；上面两张核对表已在各自那一行重复标注过一次「有意删掉」。

## 命令与输出（Fact 1–13 与 Item 1–5 依据，本次会话现查，留痕核验）

```
$ grep -n "" /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-format/src/lib.rs | sed -n '11,15p;62,66p;98,99p;122,123p;136,161p;284,285p'
11:/// 落点粒度：一个 16 KiB 槽（D3（空间分配） 已定项 7；D19（块指针的结构与宽度预算） 已定项 4 的物理偏移就是槽号）。
12:pub const SLOT_BYTES: u64 = 16384;
13:
14:/// 索引节点（码 2）恒 16 KiB（D8（核心索引结构） 已定项 2）。format-const: NODE_BYTES
15:pub const NODE_BYTES: u64 = 16384;
62:/// extent 树节点（码 2）的头：`86 + 2 × 24 + 29`，key 宽 24（D18（块里携带什么信息） 已定项 2 / 已定项 16；D8（核心索引结构） 已定项 11）。format-const: EXTENT_TREE_INDEX_NODE_HEADER_BYTES
63:pub const EXTENT_TREE_INDEX_NODE_HEADER_BYTES: u64 = 163;
64:
65:/// 分配记录树节点（码 2）的头：`86 + 2 × 10 + 29`，key 宽 10（D8（核心索引结构） 已定项 11；D18（块里携带什么信息） 已定项 16）。format-const: ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES
66:pub const ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES: u64 = 135;
98:/// extent 树叶记录：key 24 + 数据指针 88（D19（块指针的结构与宽度预算） 已定项 7）。format-const: EXTENT_LEAF_RECORD_BYTES
99:pub const EXTENT_LEAF_RECORD_BYTES: u64 = 112;
122:/// 分配记录一条：key 10 + value 10（D3（空间分配） 已定项 7 / 已定项 11）。format-const: ALLOCATION_RECORD_BYTES
123:pub const ALLOCATION_RECORD_BYTES: u64 = 20;
136:/// 分配记录树一片叶罩几个槽（W，D8（核心索引结构） 已定项 14「分配记录树」：按绝对槽号按位置寻址，叶 k 罩 `[k × W, (k + 1) × W)`，
137:/// W 取偶数、W ≤ 叶条目容量 812）：取叶条目容量本身 812。条款把具体取值交给实现员，交回里写明，主 agent 定后补进已定项 14。
138:pub const ALLOCATION_RECORD_TREE_LEAF_SLOTS: u64 = 812;
139:
140:/// 分配记录树内部节点条目：孩子罩的那一段的起点 key 10 + 子指针 86（D8（核心索引结构） 已定项 11 那一行「照码 2 btree 做时它们的内部条目是 96」）。
141:pub const ALLOCATION_RECORD_TREE_INTERNAL_ENTRY_BYTES: u64 = 96;
142:
143:/// 分配记录树内部节点的扇出：(16384 − 135) ÷ 96 的整数部分。一个内部节点罩 169 个孩子那么宽的一段。
144:pub const ALLOCATION_RECORD_TREE_INTERNAL_FANOUT: u64 = 169;
145:
146:/// extent 树内部节点条目（上段与下段同一种）：孩子罩的那一段的起点 key 24 + 子指针 86（D8（核心索引结构） 已定项 11 那一行，
147:/// extent 110 是用户 2026-09-24 定的）。
148:pub const EXTENT_TREE_INTERNAL_ENTRY_BYTES: u64 = 110;
149:
150:/// extent 树内部节点的扇出（上段与下段同一种）：(16384 − 163) ÷ 110 的整数部分（D8（核心索引结构） 已定项 14「内部扇出 147」）。
151:pub const EXTENT_TREE_INTERNAL_FANOUT: u64 = 147;
152:
153:/// extent 树下段一片叶罩几个数据单元（D8（核心索引结构） 已定项 14「叶罩 144 个单元」）：(16384 − 163) ÷ 112 的整数部分。
154:pub const EXTENT_TREE_LOWER_LEAF_DATA_UNITS: u64 = 144;
155:
156:/// extent 树上段叶条目：key 24（locality 0、inode 号、offset 0）+ 标签 1 + 载荷 88（标签 1 时是下段根的节点指针 86 加 2 字节零，
157:/// 标签 2 时是那个数据单元的数据指针 88，标签 0 时全零）。D8（核心索引结构） 已定项 14 把上段叶条目的完整字段表交给实现员，交回里写明。
158:pub const EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES: u64 = 113;
159:
160:/// extent 树上段一片叶罩几个 inode 号：(16384 − 163) ÷ 113 的整数部分（一个 inode 号至多一条上段叶条目，叶永远装得下）。
161:pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;
284:/// 单元区从槽 50176（784 MiB）起（D3（空间分配） 已定项 10）。
285:pub const UNIT_AREA_START_SLOT: u64 = 50176;
```

```
$ grep -n "" /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/allocator.rs | sed -n '217,223p;1436,1442p'
217:/// 一块盘的单元区有几个 16 KiB 槽：从 [`UNIT_AREA_START_SLOT`] 起到设备末尾。
218:/// **这是这个量唯一的一处定义**：[`DeviceFreeMap::new`] 与恢复那一侧「分配记录的跨度在不在单元区内」
219:/// 那一道判（`recovery::allocation_records_fit_the_pool_geometry`）都读它，两处手抄会分叉。
220:#[must_use]
221:pub fn unit_area_slots_of_device(device_bytes: u64) -> u64 {
222:    device_bytes / SLOT_BYTES - UNIT_AREA_START_SLOT
223:}
1436:    #[test]
1437:    fn unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments() {
1438:        let map = DeviceFreeMap::new(DeviceIdentity(0), 4 << 30);
1439:        assert_eq!(map.unit_area_slots(), 211_968);
1440:        assert_eq!(map.empty_segments(), 3312);
1441:        assert_eq!(map.free_runs(), 1);
1442:    }
```

```
$ grep -n "" /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/allocation_record_tree.rs | sed -n '85,113p;115,129p;885,911p'
85:    /// 从每块盘的绝对槽数（设备字节数 ÷ 16384）算。
86:    ///
87:    /// # Panics
88:    /// 池里一块盘都没有，或盘多到连 255 层的根都装不下每块盘各一格（多于 169 块盘）：池里恒有两块盘（mkfs 断言过）。
89:    #[must_use]
90:    pub fn of_devices(slots_of_each_device: &[(DeviceIdentity, u64)]) -> Self {
91:        assert!(
92:            !slots_of_each_device.is_empty(),
93:            "池里至少一块盘：mkfs 断言过两块"
94:        );
95:        let mut slots_of_each_device = slots_of_each_device.to_vec();
96:        slots_of_each_device.sort_by_key(|(device, _)| *device);
97:        // 迭代上界是 255 层；每一轮 S_{R−1} 乘 F，槽数有上界 2^48，至多 6 轮就到每块盘一格。
98:        let root_level = (1..=u8::MAX)
99:            .find(|candidate_root_level| {
100:                let child_span = span_in_slots_at_level(candidate_root_level - 1);
101:                let cells: u64 = slots_of_each_device
102:                    .iter()
103:                    .map(|(_, slots)| slots.div_ceil(child_span))
104:                    .sum();
105:                cells <= ALLOCATION_RECORD_TREE_INTERNAL_FANOUT
106:            })
107:            .expect("盘数不多于扇出 169 时，槽数有上界 2^48、层级至多 6 就装得下每块盘各一格");
108:        Self {
109:            slots_of_each_device,
110:            root_level,
111:        }
112:    }
113:
115:    /// 写者那一侧：分配器里每块盘的单元区末端就是这块盘的绝对槽数（单元区从 [`UNIT_AREA_START_SLOT`] 起到设备末尾）。
116:    #[must_use]
117:    pub fn of_allocator(allocator: &PoolAllocator) -> Self {
118:        let slots: Vec<(DeviceIdentity, u64)> = allocator
119:            .devices
120:            .iter()
121:            .map(|device_map| {
122:                (
123:                    device_map.device,
124:                    UNIT_AREA_START_SLOT + device_map.unit_area_slots(),
125:                )
126:            })
127:            .collect();
128:        Self::of_devices(&slots)
129:    }
885:    fn geometry_of_two_four_gibibyte_devices() -> AllocationRecordTreeGeometry {
886:        let slots = (4u64 << 30) / SLOT_BYTES;
887:        AllocationRecordTreeGeometry::of_devices(&[
888:            (DeviceIdentity(0), slots),
889:            (DeviceIdentity(1), slots),
890:        ])
891:    }
902:    #[test]
903:    fn two_one_tebibyte_devices_give_a_tree_of_height_four() {
904:        let slots = (1u64 << 40) / SLOT_BYTES;
905:        let geometry = AllocationRecordTreeGeometry::of_devices(&[
906:            (DeviceIdentity(0), slots),
907:            (DeviceIdentity(1), slots),
908:        ]);
909:        assert_eq!(geometry.height(), 4);
910:    }
911:
```

（注：上面第三段命令输出里 `:893-900` 一段——`two_four_gibibyte_devices_give_a_tree_of_height_three`
测试函数体本身，含 `assert_eq!(geometry.root_level(), 2)` 等断言——本次核查时**特意没有**用 `sed`
取进这份留痕记录，理由与 Fact 12 那一行「有意删掉」相同：那条断言就是 item 3 配置 B 要求模型自己
算出的答案之一，连留痕材料里都不放，避免有人从这份核对表里间接读到答案。同理 `:1130-1146` 的
145 单元测试与 `:521-527` 的 `lower_height` 函数体也不在这里贴出，只在上方表格里点名文件与行号
区间，不贴正文。）

```
$ grep -n "" /tmp/claude-1000/m2-final-code-r3/tree/crates/singlefs-core/src/extent_tree.rs | sed -n '79,101p'
79:/// 下段层级 L 上一个节点罩几个数据单元：144 × F^L，乘到装不进 u64 就停在 u64 的最大值。
80:#[must_use]
81:pub fn lower_span_in_data_units(level: u8) -> u64 {
82:    (0..level).fold(EXTENT_TREE_LOWER_LEAF_DATA_UNITS, |span, _| {
83:        span.saturating_mul(EXTENT_TREE_INTERNAL_FANOUT)
84:    })
85:}
86:
87:/// 上段罩得住 inode 号 `largest_inode` 的最低一层（上段根的层级）。
88:#[must_use]
89:pub fn upper_root_level_for(largest_inode: u64) -> u8 {
90:    (0..=u8::MAX)
91:        .find(|level| upper_span_in_inodes(*level) > largest_inode)
92:        .expect("第 8 层起罩的 inode 号已超过 u64 的上界")
93:}
94:
95:/// 下段罩得住 `data_units` 个单元（单元号 0 .. data_units − 1）的最低一层（下段根的层级）。
96:#[must_use]
97:pub fn lower_root_level_for(data_units: u64) -> u8 {
98:    (0..=u8::MAX)
99:        .find(|level| lower_span_in_data_units(*level) >= data_units)
100:        .expect("第 8 层起罩的单元数已超过 u64 的上界")
101:}
```

## 没做什么（本核对表）

- 未核对云端攻方腿（Z13、Z16、Z17）与云端正推腿（Z14、Z15、Z18）各自提示的 fact 与判断——本地攻方
  按分工表只碰算术这一格，不碰另外六格。
- 未判两次抽样之间答复方向是否一致——那是运行记录与主 agent 的事，不是这份核对表的事。
- 未解读、未采纳提示预期会得到的具体数值——这份核对表只核「英文提示与原文逐句对得上」，不核算术
  本身对不对。
