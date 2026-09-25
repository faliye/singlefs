# 转述核对表：m2-final-code-r2-local-attack（2026-09-25）

逐句核对 `research/prompts/m2-final-code-r2-local-attack.md`（本地攻方：正文第四节「分工」表本地攻方那一行
①-⑤ 五个算术格，拆成 FACT 1 至 FACT 9 与 10 个任务标签）里每一条 FACT 与中文原文。行号本次会话现查
各自源文件（`crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/unit.rs`、
`crates/singlefs-core/src/code_two_tree.rs`、`crates/singlefs-core/src/recovery.rs`、
`crates/singlefs-core/src/transaction.rs`、`.claude/kb/decisions/08-核心索引结构.md`，均取自冻结副本
`/tmp/claude-1000/m2-final-code-r2/tree/crates/` 与 kb 快照 `/tmp/claude-1000/m2-final-code-r2/kb-snapshot/`），
不从背景材料 `_m2-final-code-r2-background.md` 或正文 `_m2-final-code-r2-body.md` 里数。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与派发提示「提示里明令答复
不写代码行号与文件行号」），只在这份核对表里写死来源文件与行号；核对表与运行记录里，样本自带的行号
（若模型自己写出行号）一律标「模型自给、未核」。

FACT 7（树高 height(N) 的方法定义）与 TASK、FORMAT RULES 两节不是任何一句中文原文的转述——它们是这条腿
自己为了把「逐格填、每格写算式」落成可执行方法而写的计算规则，只在下方「没做什么」一节说明，不单独列行。

## 英文项 / 原文文件:行 / 首稿缺的 / 定稿

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| FACT 1：格式2索引节点恒16384字节，不论叶/内部 | `crates/singlefs-format/src/lib.rs:14`（注释「索引节点（码 2）恒 16 KiB（D8（核心索引结构） 已定项 2）。format-const: NODE_BYTES」）与 `:15`（`pub const NODE_BYTES: u64 = 16384;`） | 无遗漏；「恒」译作 always，16 KiB 换算成 16384 字节写出 | 按字面译；换算成字节是等值改写，避免模型再自己换算一次引入误差 |
| FACT 2：码2头宽公式、记账树头159、映射树头169 | `crates/singlefs-format/src/lib.rs:49`（注释「码 2 索引节点含 key 区间与预留位的头宽：`86 + 2 × key 宽 + 29`」）；`:68`-`:69`（记账，key 22，头159，「记账树节点（码 2）的头：86 + 2 × 22 + 29」）；`:71`-`:72`（映射，key 27，头169） | 无遗漏；公式与两组数字逐一对应 | 按字面译 |
| FACT 3：记账树叶条目34、映射树叶条目55的构成 | `crates/singlefs-format/src/lib.rs:127`-`:128`（记账「记账条目一条：key 22 + value 8 + seq 4」=34）；`:92`-`:93`（映射「中央映射条目：key 27 + 位置条目 14 × 2」=55） | 无遗漏；两条目各自的字段构成原样保留 | 按字面译；两个构成都进了后续格子要用的数（key宽22/27、entry宽34/55），不压缩 |
| FACT 4：记账树内部条目108、映射树内部条目113 | `crates/singlefs-format/src/lib.rs:130`-`:131`（「记账树内部节点条目：分隔 key 22 + 子指针 86」=108）；`:133`-`:134`（「中央映射树内部节点条目：分隔 key 27 + 子指针 86」=113）；`.claude/kb/decisions/08-核心索引结构.md:327`（「内部节点的条目 = 本树 key + 子指针 86：记账树 22 + 86 = 108、中央映射树 27 + 86 = 113。」） | 首稿加了一句原文三处都没有逐字写出的话：「没有额外的、条目数之外的子指针（不同于某些其他树设计里 n 个 key 对应 n+1 个子指针）」 | 这句是结构性推出的桥接句，不是某一行注释的转述，见下表「多出来的限定」 |
| FACT 5：单节点容量公式=⌊(16384−头)÷条目宽⌋，叶用叶条目宽、内部用内部条目宽、头相同 | `crates/singlefs-core/src/unit.rs:120`-`:124`（注释「码 2 索引节点 16384……一个码 2 节点装得下多少条定宽条目：(16384 − 头 − 预留) / 条目宽」）与 `:126`-`:131`（`pub fn index_node_entry_capacity`）；`crates/singlefs-core/src/code_two_tree.rs:94`-`:95`（「一个码 2 节点最多装几条条目：叶按这棵树的叶条目宽算，内部节点按内部条目宽算」）与 `:103`（「格式算出来的容量：(16384 − 含预留位的头) ÷ 条目宽」） | 无遗漏；floor 除法与「叶/内部用各自条目宽、头不变」两点都在原文里 | 按字面译；「内部节点容量也等于最大子节点数」那句单列进下表「多出来的限定」（是 FACT 4 一一对应关系的推论，不是这几行注释本身说的） |
| FACT 6：记账行数 = 3 + 6×设备数，两盘15行 | `crates/singlefs-core/src/recovery.rs:2369`（`if roots.accounting.leaf_entries_in_key_order.len() != 3 + 6 * device_count`）与 `:2371`（报错文案「记账条目数不是 3 + 6 × 盘数」）；`crates/singlefs-core/src/transaction.rs:5070`-`:5071`（注释「t6 记账树（D5（快照 / 空间记账机制） 已定项 8）：两盘 15 行——带设备维的六项每盘一行、池级三行」）；`crates/singlefs-format/src/lib.rs:136`-`:137`（「第一个事务这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。」`pub const FIRST_TRANSACTION_ACCOUNTING_ROWS: u64 = 15;`） | 首稿略去了 transaction.rs:5071 后半句「全部来自分配器在分配那一刻增量维护的数，不扫盘」 | 略去的半句讲的是这个数怎么算出来（增量维护、不扫盘），不影响「3+6×D」这条算术本身，登记为有意省略 |
| FACT 8：见证表布局（计数1字节、条目16字节、上限=R×S−1、偏移481、槽4096） | `crates/singlefs-format/src/lib.rs:195`-`:196`（`SYSTEM_CONFIGURATION_BYTES: u64 = 481`）；`:232`-`:235`（「回退见证表……从槽内偏移 481（`SYSTEM_CONFIGURATION_BYTES`）起」，`ROLLBACK_WITNESS_TABLE_OFFSET_IN_THE_SYSTEM_CONFIGURATION_SLOT = SYSTEM_CONFIGURATION_BYTES`）；`:204`-`:205`（`SYSTEM_CONFIGURATION_SLOT_BYTES: u64 = 4096`）；`:238`-`:239`（「见证表头：条数 1 字节」）；`:241`-`:242`（「一个见证条目：新实例代号 4 + 回退目标 R_old 的实例代号 4 + R_old 的 txg 8」=16）；`:244`-`:247`（「见证表按 S 的上界定宽：条数上限 = 根环槽数减 1（R × S − 1，只由根环几何定），S 取格式承诺区间的上界 16 时是 47 条」，`ROLLBACK_WITNESS_ENTRIES_MAXIMUM = ROOT_RING_REGIONS * ROOT_RING_SLOTS_PER_REGION_MAXIMUM - 1`）；`:208`-`:209`（`ROOT_RING_REGIONS: u64 = 3`）；`:211`-`:218`（`ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = 4`）；`:220`-`:223`（`ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16`） | 首稿略去「罩在整槽校验和里、越过 512 字节」（`:233`）；「S 是系统配置字段，挂载时判区间」那半段（`:214`-`:216`）压成了一句「区间的两条边是已定的格式常量」 | 「罩在整槽校验和里、越过 512 字节」讲的是撕裂检出机制，与「偏移+表长 ≤ 槽宽」这条纯算术无关，登记为有意省略；S 是运行期字段还是格式常量那段背景压缩后只保留「区间上下界 4 和 16 是已定常量」这个算术要用的事实，登记为有意省略，不是漏译 |
| FACT 9：journal记录4096、头311、点名项56 | `crates/singlefs-format/src/lib.rs:161`-`:162`（「journal 记录定长 4 KiB」`JOURNAL_RECORD_BYTES: u64 = 4096`）；`:170`-`:171`（「journal 记录头：78 + 事务号 8 + …」`JOURNAL_HEADER_BYTES: u64 = 311`）；`:173`-`:176`（「journal 点名项：位置条目 14 × 2 + 类标签 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1」`JOURNAL_NAMED_ENTRY_BYTES = …`，值56，`:351` 测试断言 `assert_eq!(JOURNAL_NAMED_ENTRY_BYTES, 56, ...)`） | 首稿略去了 311 字节头与 56 字节点名项各自的十几个子字段逐项列表 | 子字段列表不进任何一格算术（10 个任务项只用得到 4096、311、56 三个总数），登记为有意省略，与 FACT 4 对子指针 86 字节构成的省略同一条理由 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| FACT 1「regardless of whether it is a leaf node or an internal node」 | `crates/singlefs-format/src/lib.rs:15`（`NODE_BYTES` 只有一个全局常量，没有另立叶/内部两个值）；`crates/singlefs-core/src/code_two_tree.rs:94`-`:95`（容量公式对叶、内部共用同一个 16384） | 加了「不论叶节点还是内部节点」这句限定 | `NODE_BYTES` 是唯一一个格式2节点尺寸常量，代码里没有第二个「内部节点尺寸」常量可供分岔；不写清楚，模型可能去猜内部节点是不是另一个尺寸，而 FACT 5 的容量公式两种节点都要用到 16384 |
| FACT 2「the same value whether that node is a leaf node or an internal node」「every node at every level uses this same header size」 | `crates/singlefs-format/src/lib.rs:50`（`index_node_header_bytes(key_width_in_bytes: u64)` 函数签名只有一个参数，没有层级参数）；`crates/singlefs-core/src/code_two_tree.rs:94`-`:95`、`:103`（容量公式对叶、内部都从同一个「含预留位的头」减） | 加了「头宽不分叶/内部、同一棵树每层头宽相同」这句限定 | 函数签名没有「层级」参数，头宽公式在类型层面就不可能因层级不同而不同；FACT 5 要用「内部节点头宽=叶节点头宽」这一步做减法，不写清楚模型可能去猜内部节点是否需要一个不同的头，从而算错 AC.internal_cap / MP.internal_cap |
| FACT 4「there is no additional child pointer beyond the number of entries in the node, unlike some other tree designs where an internal node with n keys has n plus 1 children」 | 无单一原文行；结构性推出（`crates/singlefs-format/src/lib.rs:46` 码2头字段表只列一个「条目数」字段，没有另立「最左子指针」字段；`.claude/kb/decisions/08-核心索引结构.md:327` 与 `crates/singlefs-format/src/lib.rs:130`-`:131`、`:133`-`:134` 三处「内部条目=本树key+子指针86」都是一条目一子指针） | 加了「条目数之外没有额外子指针（不同于经典 B 树 n+1 子指针）」这整句 | 码2头里只有一个「条目数」字段，没有第二个字段能装「最左子指针」；三处内部条目定义都是把子指针直接编进条目本身。不显式排除 n+1 的读法，ROWS.height_at_15 与 ROWS.min_devices_to_split 都要用「内部节点容量=最大子节点数」这一步，模型可能自己去猜是不是要多留一个子节点的位置 |
| FACT 5「an internal node's entry capacity also equals the maximum number of child nodes that one internal node can point to」 | 同上（FACT 4 的推论），另见 `crates/singlefs-core/src/code_two_tree.rs:103`-`:112`（`of_the_node_format` 直接把 `index_node_entry_capacity` 的返回值同时赋给 `leaf_entries` 与 `internal_entries`，内部那份就是子节点容量，没有另一条「子节点数」的独立算法） | 加了「内部节点容量=能指向的最大子节点数」这句换算 | ROWS.height_at_15 的高度公式（叶容量 × 内部扇出^(h-1)）里「内部扇出」指的是子节点数，不是条目数；两者在这份代码里数值相同（FACT 4 的一一对应），但概念不同，不写这句换算模型可能分不清任务里 AC.internal_cap 该填哪一个 |
| FACT 6「right after a pool's very first publish」 | `crates/singlefs-core/src/transaction.rs:5070`（注释标题「t6 记账树」出现在描述「第一个事务」发布写出的行的上下文里）与 `crates/singlefs-format/src/lib.rs:136`（「第一个事务这次发布写出的记账行数」） | 加了「紧接着池的第一次发布之后」这句限定 | `FIRST_TRANSACTION_ACCOUNTING_ROWS` 与 `transaction.rs` 那段注释都是在讲「第一个事务」这一次发布之后树里有几行，不写清楚时间点，模型可能以为 15 是任意时刻的行数，而记账树后续发布可能改代（`checks-owed.md` C548 记的整批换代问题，这份提示不引用 C548，只借这一点把「哪个时刻」说清楚） |
| FACT 8「a closed interval whose lower bound is 4 and whose upper bound is 16」 | `crates/singlefs-format/src/lib.rs:211`-`:218`（`ROOT_RING_SLOTS_PER_REGION_MINIMUM: u64 = 4`）与 `:220`-`:223`（`ROOT_RING_SLOTS_PER_REGION_MAXIMUM: u64 = 16`），两个常量各自被用作可取到的边界值（`:246`-`:247` 用 16 算出 47） | 把「下界」「上界」两个中文词统一改写成「闭区间」这句英文表述 | 原文两处分别叫「下界」「上界」，没有单独用一个词说「闭区间」；但 `:247` 的 `ROLLBACK_WITNESS_ENTRIES_MAXIMUM` 公式直接用 16（上界本身）代入算出 47，说明上界是可取到的值，不是一个够不到的极限，「闭区间」是这个用法的等值改写，不是新信息 |

## 没做什么

- FACT 7（height(N) 的方法定义）、TASK 一节、FORMAT RULES 一节：不是任何一句中文原文的转述，是这条腿
  自己写的可执行计算方法与交付格式，不对着某一句中文核对，本核对表不为它们单列行。
- 未判本地模型答复的方向、未采信任何一格算出的数字——那是主 agent 的事，不是这份核对表的事。
- 未核对云端攻方腿（Opus，Z8、Z9、Z12）与云端正推腿（Sonnet，Z7、Z10、Z11）各自的提示与转述，按
  正文第四节分工表不碰；本地攻方的攻击面明令只在算术上、与云端攻方的攻击面不重叠，本文件与它们无重叠。
- 未把 `.claude/kb/checks-owed.md` C548（记账树每次发布整批换代与已定项不符、「80 块盘起才分裂」那句）
  抄进提示或核对表：正文分工表给本地攻方的题面已经把算术边界写清楚（③④两格），C548 是另一条尚在
  讨论的欠账，把它的结论抄进提示会让模型直接抄现成数字而不是自己推导，因此只在这份核对表里提一句
  它存在（`.claude/kb/checks-owed.md:482`），不作为提示的输入事实。
