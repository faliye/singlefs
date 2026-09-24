# 转述核对表：m2-keyspace-r1-local-attack（S6 算术表，四份文件）

四份提示文件：`m2-keyspace-r1-local-attack-ar-ptr.md`（分配记录树、候选=仅子指针 86 字节）、
`-ar-key.md`（分配记录树、候选=key+子指针 96 字节）、`-ex-ptr.md`（extent 树、候选=仅子指针 86 字节，
非已定案）、`-ex-key.md`（extent 树、已定案=key+子指针 110 字节）。四份共享同一批方法定义
（FACT 1、FACT 4、FACT 5、FACT 7、FACT 8、FACT 10、FACT 11 逐字相同），只有 FACT 2（key 宽/头宽）、
FACT 3（叶条目构成）、FACT 6（候选/已定案内部条目宽）、FACT 9（设备槽数或数据单元数）按树与候选各自不同，
逐条核对如下只登记一次、不同处另起一行说明。行号本次会话现查
`crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/unit.rs`、
`crates/singlefs-core/src/allocator.rs`、`.claude/kb/decisions/08-核心索引结构.md`，
不从背景材料 `_m2-keyspace-r1-background.md` 或正文 `_m2-keyspace-r1-body.md` 里数行号
（只把这两份文件当"某句话是不是来自这里"的核对对象，行号仍现查）。

提示本身不引用任何文件名加行号（按共用约束「英文提示里的每一句转述」一节与主 agent 派发提示
「提示里明令答复不写代码行号与文件行号」），只在这份核对表里写死来源文件与行号；核对表与运行记录里，
样本自带的行号（若模型自己写出行号）一律标「模型自给、未核」。

## 英文项 / 原文文件:行 / 首稿缺的 / 定稿

| 英文项 | 原文文件:行 | 首稿缺的/改动的 | 定稿理由 |
|---|---|---|---|
| FACT 1：格式 2 索引节点恒 16384 字节 | `crates/singlefs-format/src/lib.rs:14`（注释）与 `:15`（`pub const NODE_BYTES: u64 = 16384;`） | 无遗漏 | 按字面译 |
| FACT 2：头宽公式 86+2×key宽+29 | `crates/singlefs-format/src/lib.rs:46`（注释）、`:49`（注释）、`:50`-`:54`（函数体） | 无遗漏 | 按字面译 |
| FACT 2（AR）：key 宽 10、头宽 135 | `crates/singlefs-format/src/lib.rs:120`（`ALLOCATION_RECORD_KEY_BYTES`）、`:65`（注释）、`:66`（`ALLOCATION_RECORDS_TREE_INDEX_NODE_HEADER_BYTES = 135`） | 无遗漏 | 按字面译 |
| FACT 2（EX）：key 宽 24、头宽 163 | `crates/singlefs-format/src/lib.rs:96`（`EXTENT_KEY_BYTES`）、`:62`（注释）、`:63`（`EXTENT_TREE_INDEX_NODE_HEADER_BYTES = 163`） | 无遗漏 | 按字面译 |
| FACT 3（AR）：叶条目 20=key10+value10 | `crates/singlefs-format/src/lib.rs:119`（注释）、`:120`-`:123`（三个常量声明） | 无遗漏 | 按字面译 |
| FACT 3（EX）：叶条目 112=key24+数据指针88 | `crates/singlefs-format/src/lib.rs:95`-`:96`（key）、`:83`（注释）、`:84`（`DATA_POINTER_BYTES = 88`）、`:98`-`:99`（`EXTENT_LEAF_RECORD_BYTES = 112`） | 首稿加了一句区分「88 字节数据指针」与「86 字节子指针」不是同一个常量的限定，见下表 | 见下表「多出来的限定」 |
| FACT 4：子指针恒 86 字节 | `crates/singlefs-format/src/lib.rs:86`（注释）与 `:87`（`NODE_POINTER_BYTES = 86`） | 无遗漏 | 按字面译 |
| FACT 5：inode 树内部条目 120=8+26+86 | `crates/singlefs-format/src/lib.rs:104`（注释）、`:105`（`INODE_INTERNAL_ENTRY = 120`）；同一登记也见 `.claude/kb/decisions/08-核心索引结构.md:162` | 无遗漏 | 按字面译 |
| FACT 5：extent 树内部条目 110=24+86，已定案 | `.claude/kb/decisions/08-核心索引结构.md:321`（「extent 树内部节点的条目：key 24 + 子指针 86 = 110 字节。分配记录树、记账树、中央映射树的内部条目还没有条款」）与 `:330`（用户定案 2026-09-24） | 无遗漏 | 按字面译；同一行也是 AR 无已定案内部条目格式的出处 |
| FACT 6（ar-ptr）：候选=仅子指针 86 | 不是仓库文件，来源见下表 | 见下表 | 见下表 |
| FACT 6（ar-key）：候选=10+86=96 | 不是仓库文件，来源见下表 | 见下表 | 见下表 |
| FACT 6（ex-ptr）：候选=仅子指针 86（非已定案） | 不是仓库文件，来源见下表；已定案 110 已在上一行核过 | 见下表 | 见下表 |
| FACT 6（ex-key）：已定案=24+86=110 | 同 FACT 5 的 extent 行，`.claude/kb/decisions/08-核心索引结构.md:321`、`:330` | 无遗漏 | 按字面译 |
| FACT 7：叶容量=floor((16384−头)/条目宽) | `crates/singlefs-core/src/unit.rs:124`（注释「一个码 2 节点装得下多少条定宽条目：(16384 − 头 − 预留) / 条目宽」）与 `:126`-`:131`（`index_node_entry_capacity` 函数体） | 无遗漏 | 按字面译；「预留」已经算进 FACT2 的头宽公式（86+2×key宽+29 的 29 就是预留），未重复减一次 |
| FACT 8：内部扇出=同一公式换内部条目宽 | 同上 `crates/singlefs-core/src/unit.rs:126`-`:131`（同一函数，FACT 7 传叶条目宽、FACT 8 传内部条目宽） | 无遗漏 | 按字面译 |
| FACT 9（AR）：设备槽数=设备字节/16384−50176 | `crates/singlefs-format/src/lib.rs:230`（注释）、`:231`（`UNIT_AREA_START_SLOT = 50176`）；4 GiB→211968 的实测断言 `crates/singlefs-core/src/allocator.rs:1464`-`:1470`（测试名 `unit_area_of_a_4_gib_image_has_211968_slots_in_3312_segments`，`assert_eq!(map.unit_area_slots(), 211_968)`） | 64GiB/1TiB/16TiB 三个数字（4,144,128／67,058,688／1,073,691,648）不是仓库任何一处字面常量 | 这三个数字是本次会话按同一公式现算的算术（观测/算术，不需要三方），已用 `python3` 核对（4×1024³/16384−50176=211968；64×1024³/16384−50176=4144128；1024⁴/16384−50176=67058688；16×1024⁴/16384−50176=1073691648），过程未落盘于仓库，登记于本表 |
| FACT 9（EX）：数据单元净荷=32768−105−29=32634 | `crates/singlefs-format/src/lib.rs:17`（注释）、`:18`（`DATA_UNIT_BYTES=32768`）、`:32`（注释）、`:33`（`DATA_UNIT_HEADER_BYTES=105`）、`:29`（注释）、`:30`（`NONCE_MAC_ALGORITHM_RESERVED_BYTES=29`）；函数 `crates/singlefs-core/src/unit.rs:215`（注释含「32634」）、`:216`-`:221`（`data_unit_payload_capacity` 函数体） | 1MiB/1GiB/1TiB 三个数据单元数（33／32,903／33,692,212）不是仓库字面常量 | 这三个数字是本次会话按 ceil(文件字节/32634) 现算的算术，已用 `python3` 核对（1048576/32634 向上取整=33；1073741824/32634 向上取整=32903；1099511627776/32634 向上取整=33692212） |
| FACT 10：层链/树高/总节点数方法 | 不是任何一句原文的转述 | 见「没做什么」一节 | 见「没做什么」一节 |
| FACT 11：单次发布最坏情形写节点数方法 | 不是任何一句原文的转述 | 见「没做什么」一节 | 见「没做什么」一节 |

## 英文比原文多出来的限定词、括注（逐条写明为什么加）

| 英文项 | 原文文件:行 | 多出来的限定 | 为什么加 |
|---|---|---|---|
| FACT 1「regardless of whether it is a leaf node or...an internal node」 | `crates/singlefs-format/src/lib.rs:15`（`NODE_BYTES` 只有一个全局常量，没有另立叶/内部两个值） | 加了「不论叶节点还是内部节点」这句限定 | `NODE_BYTES` 是唯一一个格式2节点尺寸常量，代码里没有第二个「内部节点尺寸」常量可供分岔；FACT 8 的内部扇出公式要用到「内部节点也是 16384 字节」这一步，不写清楚模型可能去问「内部节点是不是另一个尺寸」 |
| FACT 2「depends only on that node's key width...same header size as every other node in that same tree」 | `crates/singlefs-format/src/lib.rs:50`-`:54`（`index_node_header_bytes(key_width_in_bytes: u64)` 函数签名只有一个参数，没有层级参数） | 加了「头宽只看 key 宽、同一棵树每一层头宽相同」这句限定 | 函数签名本身没有「层级」这个参数，头宽公式在类型层面就不可能因层级不同而不同；FACT 8 要用「内部节点头宽=叶节点头宽」这一步做减法，不写清楚模型可能去猜内部节点是否需要一个不同的头 |
| FACT 3（EX）「different from, and 2 bytes wider than, the 86-byte child pointer defined in FACT 4」 | `crates/singlefs-format/src/lib.rs:84`（`DATA_POINTER_BYTES=88`）与 `:87`（`NODE_POINTER_BYTES=86`），两个常量在文件里紧邻声明、含义不同 | 加了「88 字节数据指针与 86 字节子指针是两个不同的常量」这句限定 | 两者只差 2 字节、紧邻声明，如果不显式区分，模型可能把 extent 叶记录的 88 字节数据指针与 FACT 6 内部条目要用的 86 字节子指针混为一谈，把 FACT 8 的内部扇出算成用 88 而不是 86，是全篇最容易踩的一个混淆点 |
| FACT 4「does not vary by tree and does not vary by level」 | `crates/singlefs-format/src/lib.rs:87`（`NODE_POINTER_BYTES` 是唯一一个全局常量，没有按树或按层再分） | 加了「不因树而变、不因层而变」这句限定 | 与 FACT 1/FACT 2 同一类推论：只有一个全局常量，没有第二个可供分岔的值 |
| FACT 5「there is no additional child pointer beyond the number of entries...unlike some other tree designs where an internal node with n keys has n plus 1 children」 | 无单一原文行；结构性推出（`crates/singlefs-format/src/lib.rs:104`-`:105` 的 120=8+26+86 与 `.claude/kb/decisions/08-核心索引结构.md:321` 的 110=24+86，两例都是「一条目一子指针」；`crates/singlefs-core/src/unit.rs:120`-`:124` 码 2 头字段表只有「条目数」一个字段，没有另立最左子指针字段） | 加了「条目数之外没有额外子指针（不同于经典 B 树 n+1 子指针）」这整句 | 两个已定案例都是条目自带子指针、总宽已经把子指针算进条目宽度里；不显式排除 n+1 的读法，FACT 8 的内部扇出公式（条目容量=子节点容量）就没有立足点，模型可能自己去猜是不是要多加 1 个子指针的空间 |
| FACT 6（四份文件各自的候选/已定案构造规则整段） | 不是仓库文件；来源是这次会话开头收到的派发消息原话：「候选几何按正文 S1、S2：叶管的 key 区间宽 = 一片叶的条目容量（分配记录 812 个槽、extent 144 个数据单元），上层每层扇出按『子指针 86』与『key + 子指针』两种各算一遍」 | 「仅子指针 86」与「key+子指针」两种候选宽度本身（AR 两份文件的 86/96，EX-ptr 的 86），以及「不判断该候选是否可行」这句免责声明 | 避免模型或复核者把候选宽度误当成 kb 或 `crates/` 里已经写死的条款去核对；分配记录树在 kb 里连内部条目的字面数字都没有（`.claude/kb/decisions/08-核心索引结构.md:321`「分配记录树...内部条目还没有条款」），两个候选数字是这条腿按派发消息的构造法算出、写进「写死的事实表」的，不是转述 |
| FACT 9（AR）「the worst case for how many allocation-record leaf entries a device could ever need is therefore one entry per slot」 | `research/prompts/_m2-keyspace-r1-body.md:13`（S1 那一行：「叶管多宽一段槽才永远装得下（一条记录至少 1 槽）」） | 把「一条记录至少 1 槽」倒过来表述成「最坏情形一槽一条记录，因此 N=槽数」 | body.md 原句只正面说「至少 1 槽」，没有直接说「因此最坏情形是槽数=记录数」；这一步等价改写是必要的桥接，否则模型不知道拿什么数当 FACT 10 的目标叶条目数 N；改写不改变原句的量化含义（「至少 1 槽」与「最多 1 槽/条」互为同一条件的两种表述） |
| FACT 9（AR）「64 GiB has 4,144,128 slots; a 1 TiB device has 67,058,688 slots; a 16 TiB device has 1,073,691,648 slots」与「4 GiB ÷ 16384 − 50176」公式注解 | `research/prompts/_m2-keyspace-r1-body.md:37` 只写了 211968 这一个数（「每盘槽数（4 GiB 盘 211968 槽）」），没有写公式，也没有写另外三个盘的数字；公式注解「4 GiB ÷ 16384 − 50176」与「其余盘按同一算式」都是这次会话开头收到的派发消息原话，不在 body.md 里 | 另外三个盘的数字、以及公式本身写法 | body.md 只给了一个已知点（4 GiB→211968），「其余盘按同一算式」是派发消息交代的做法，另外三个数字由这条腿本次会话按同一公式现算得出（算术，见上表），登记来源以免与「仓库里已经写死」混淆 |
| FACT 10、FACT 11 全文 | 不是任何一句原文的转述 | 见下方「没做什么」一节，不当限定词单列 | 见下方「没做什么」一节 |

## 没做什么

- FACT 7、FACT 8、FACT 10、FACT 11（叶容量、内部扇出、层链/树高/总节点数、单次发布最坏情形写节点数四条方法定义）、四份文件各自的 TASK 一节、FORMAT RULES 一节：不是任何一句中文原文（`_m2-keyspace-r1-body.md` 或这次会话开头的派发消息）的逐句转述，是这条腿自己写的可执行计算方法与交付格式，不对着某一句中文核对，本核对表因此不为它们的方法定义部分单列「缺什么/多什么」行（FACT 7、FACT 8、FACT 9 涉及的数字部分已在上一张表核过）。
- 未判本地模型答复的方向、未采信任何一格算出的数字、未判两次抽样之间是否一致——那是主 agent 的事；这份核对表只核提示与原文的对应关系。
- 未核对云端攻方腿（Opus，S1、S2、S4）与云端正推腿（Sonnet，S3、S5，及 S1/S2/S4 各一句）的提示与转述，按分工表不碰。
- 未通读 `_m2-keyspace-r1-background.md` 全文（1526 行）；只读了与 S6 算术表相关的部分（S1/S2 题面所在行、本地攻方分工那一行、附录里 812/144/32634 相关的现状描述行）。S6 是纯算术表，事实来源按派发要求「去 kb 或 `crates/singlefs-format/src/lib.rs` 现查，不从背景材料里数」，背景材料本身不作为引用来源，因此没有必要通读。
- 未读这一轮的正文/背景材料之外的其它文件（`_m2-keyspace-r1-appendix.md`、`_m2-keyspace-r1-checklist.md`、`m2-keyspace-r1-snapshot/`）；本轮派发消息未给出明确的「禁读清单」，按未特别禁读处理，只读了完成 S6 算术表任务必需的文件。

## 四份文件的拆分理由（这一轮从一开始按树×候选拆成四份，不是撞了预算之后才拆）

派发消息明说「上一道题的本地腿拆成四份按树分的提示才没撞网关输出预算，这一轮格数多就照拆」，指向
`m2-treesplit-r1-local-attack.md`（52 格单文件撞输出预算，退出码 3）与随后按树拆成的 `-ar.md`/`-ac.md`/
`-cm.md`/`-ex.md`（各 13 格、都是退出码 0）这一先例，见 `research/prompts/m2-treesplit-r1-local-attack-translation-audit.md`
「补记」一节与 `research/prompts/m2-treesplit-r1-local-attack-runlog.md`「原始 52 格文件」一节。

本轮 S6 只有两棵树（分配记录树、extent 树），但每棵树都要按「仅子指针」与「key+子指针」两种候选内部条目
各算一遍（派发消息原话，见上表 FACT 6 行），是「树 × 候选」两个维度，2×2=4，天然对应四份文件；若按「树」
一个维度拆成两份，每份要装下两个候选 × （4 或 3 个尺寸）× 2 个量 + 3 个写节点数 = 分配记录树 25 格、
extent 树 21 格，仍超过既有先例「≤16 格一份」的经验上限（见 `m2-treesplit-r1-local-attack-runlog.md`
第 27 行引用的先例说明）；改按「树 × 候选」拆成四份后，分配记录树两份各 13 格、extent 树两份各 11 格，
全部落在 ≤16 格以内。
