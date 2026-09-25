# 逐句核转述：`c363b-r1-local-attack.md`

核对表；按 `.claude/rules/three-way-inference.md`「英文提示里的每一句转述，写完都要对着原文核一遍」，以及这一轮派发提示「能落成数的问题给写死的事实表...核对表里写来源文件:行」。行号一律现查（`grep -n`），不从背景材料的拼接稿数；代码引用的行号同样现查冻结副本 `/tmp/claude-1000/c363b-r1/tree/crates/`（与工作区 `crates/` 当前内容一致，已用 `grep -n` 现查两处）。

分两张表：表一是提示里逐句转述、需要核对首稿有没有漏译或多译的四条 kb 引文；表二是提示里每一条 FACT 编号对应的原始来源（数值常量、函数、结构），供核对「事实表每行写来源文件与行号」这一条。

## 表一：kb 引文逐句核对

| 英文项（提示文件节选） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| FACT G quote: "How a multi-level code-two tree grows and shrinks (the accounting tree, the central mapping tree, and any tree that still uses the code-two btree): 1) ... no extra write-ordering step and no barrier; 2) when a leaf does not fit, it is split down the middle; 3) shrinking only removes empty nodes ... no below-half merging and no borrowing of entries; 4) separator keys are kept up to date ... 5) tree height equals the level field in the root node's code-two header, plus 1." | `.claude/kb/decisions/08-核心索引结构.md:321` 全句：「多层码 2 树怎么长、怎么收（记账树、中央映射树，以及照已定项 14 仍用码 2 btree 的树）：① 一次发布里分裂新建的节点与别的单元落在同一段，整条根到叶的路径照常 COW，不加写序步骤、不加屏障；② 叶装不下时从中间切；③ 收缩只摘空节点，根只剩一个孩子时降高一层，不做「低于一半合并」与借条目；④ 分隔 key 跟着维护：插到最左分隔 key 之下时把它压低，条目在兄弟间挪动之后把右边那个孩子的分隔 key 改成它新的最小 key；逐层可判：分隔 key ≤ 孩子节点头的最小 key，且 > 左邻孩子节点头的最大 key（D18（块里携带什么信息） 已定项 18 的 key 区间）；⑤ 树高 = 根节点码 2 头里的层级 + 1。」 | 首稿一度把括注「（D18（块里携带什么信息） 已定项 18 的 key 区间）」整段删掉，理由是「这个补充不影响高度算术」；核对后发现这句是①-⑤五条并列里第④条自己带的可判定标准的出处限定词，删掉会让「逐层可判」那句变成没有出处的断言——按「条款自己写了要连这句一起引」补回 | 定稿保留「(the key range in D18, decision item 18)」，五条全部逐条译出，编号①-⑤对应保留为 1)-5)，未删未增条目本身；括注挪到第 4 条分隔 key 的判定句之后（见提示文件 FACT G 全文） |
| FACT E1 quote: "Allocation record tree: addressed by absolute slot number, by position. Each leaf covers one fixed range of slots [k * W, (k + 1) * W) ... The checker checks, node by node, that the range a node claims to cover really is the range its position dictates, and that a record's last slot does not run past the last slot of the leaf it is in." | `.claude/kb/decisions/08-核心索引结构.md:377` 全句：「分配记录树：按绝对槽号按位置寻址。每片叶罩一段固定的槽 `[k × W, (k + 1) × W)`，W 取偶数（两槽数据单元永远不跨叶），且 W ≤ 叶条目容量 812（一条记录至少罩一个槽，叶永远装得下）；记录按起点槽落叶。往上各层按位置寻址，子节点罩哪一段由它在父节点里的位置算出。没有记录的一段 = 全空闲：那片叶不写，父节点那一格留空（用户 K1）。checker 逐节点核「这个节点罩的就是它的位置规定的那一段」与「记录的末槽不越过它所在叶的末槽」。」 | 首稿把「（用户 K1）」这个括注删了，理由是「跟本轮算术无关」；核对后判定这是一个可省的归属标注（不是判据、不是限定词，只说这条规则是哪次用户决定编的号），删掉不改变规则本身能不能用来算高度，按「只写指路不写数」的精神可以不抄这半句——非遗漏，列此说明为什么不抄 | 定稿不含「(user decision K1)」这个括注；规则本身（W 取偶数、W ≤ 812、按起点槽落叶、位置寻址、空段不写、checker 两条核验）逐句全译，未增未减 |
| FACT E3 quote: "The specific value of the leaf width W, the fan-out of the upper level, and the complete field layout of the upper-level leaf entries are things this decision item does not write down; whatever it does not write down, the implementer is to choose following the points above and state that choice in the hand-back; the lead agent will decide and add it into this section afterward." | `.claude/kb/decisions/08-核心索引结构.md:382`（一句，摘自射程段一整行里的一个 ⚠️ 分句）：「⚠️ 叶宽 W 的具体取值、上段扇出、上段叶条目的完整字段表，条款没写到的由实现员照上面几条取、交回里写明，主 agent 定后补进这一节。」 | 无缺——三件没写到的事（W 取值、上段扇出、上段叶条目字段表）与两步处置（实现员先取、交回写明；主 agent 定后补）均逐句译出 | 定稿保留，提示文件里明写这是「一句摘自更长段落」并说明该段落其余内容（write buffer 前端、已否掉的候选等）与本轮算术无关、未搬入提示——按「射程按每个要引的 kb 文件算」的精神，本轮只从这一大段摘两句（此句与下一行 FACT F1 引的那句），两句都在核对表单独列行、注明摘自同一段落，不整段抄的理由是段落其余部分（write buffer、已否掉候选、write buffer 前端豁免）与树高算术无关 |
| FACT F1 quote: "The accounting tree (the authoritative state), the central mapping tree, the tree table, the instance table, and the inode tree are all built with the one shared code-two btree (the splitting and shrinking rule)." | `.claude/kb/decisions/08-核心索引结构.md:382`（同一行，另一个 ⚠️ 分句之前的正文分句）：「记账树（权威态）、中央映射树、树表、实例表、inode 树照一套码 2 btree 做（D8（核心索引结构） 已定项 11 的分裂与收缩）。」 | 无缺——五棵树的名字（记账树、中央映射树、树表、实例表、inode 树）与括注「已定项 11 的分裂与收缩」均译出 | 定稿把括注意译成「(the splitting and shrinking rule)」，不点名「decision item 11」这个编号本身（因为提示已用 FACT G 指代同一条规则，不再让模型去查一个它读不到的决策编号），核对后确认这个编号只是回指 FACT G 自己讲的同一条规则，省略编号不丢规则内容 |
| FACT H quote: "Form: ckpt_cost = the sum, over each record tree, of that tree's current height, plus the number of accounting-tree nodes rewritten in this publish; recomputed at every publish from the current tree heights. Each code-two tree's current height equals the level field read right now from its root node's code-two header, plus 1; it is not cached anywhere in memory. How to read the height of the allocation-record tree and the extent tree, whose shapes are defined by key-space position, is left to a separate round of review. The unit is one 16 KiB metadata block; when substituted into a separate admission formula, which is written in bytes, it is multiplied by 16384. It is a quantity computed at runtime; there is no field for it on disk." | `.claude/kb/decisions/28-挂载期承诺量.md:94` 全句：「形态：ckpt_cost = Σ（每棵记录树当前的高）+ 记账树每发布的节点数，每次发布按当时的树高重算；每棵码 2 树当前的高 = 发布那一刻从它的根节点码 2 头里现读的层级 + 1（D18（块里携带什么信息） 已定项 18 的层级字段），不用内存里另存一份；按 key 空间定形状的分配记录树与 extent 树的高怎么读，随三方 `m2-keyspace-r1`。单位是 16 KiB 元数据块，代进已定项 1 那条按字节写的式子时乘 16384（式子里的「已分配」是「已分配字节」、「容量」是单元区大小，D5（快照 / 空间记账机制） 已定项 7）；它是运行时算出的量，盘上没有字段。」 | 首稿删了两处括注：「（D18（块里携带什么信息） 已定项 18 的层级字段）」与「（式子里的「已分配」是「已分配字节」、「容量」是单元区大小，D5（快照 / 空间记账机制） 已定项 7）」，理由都是「决策编号模型读不到，删了省字」；核对后判定：第一处括注只是回指 FACT G 已经给出的同一条「层级+1」规则，删掉不丢内容（FACT G 已完整给出这条规则），按「省略指向别处已给全的同一条规则」处理，非摘句；第二处括注解释了「已分配」「容量」在另一条式子（准入式）里的含义，这条式子本身不在本轮算术材料里，模型也不需要用到「已分配」「容量」这两个词去做树高与 ckpt_cost 的算术，删掉不影响本轮要算的东西 | 定稿两处括注均省略（非漏译，是判定与本轮算术无关之后的取舍，见前一列理由）；「m2-keyspace-r1」这个三方轮次名保留但改写成「a separate round of review」（不给模型一个它查不到的具体轮次名去搜索），其余每一句（Σ 是什么、树高怎么读、单位是什么、乘 16384、运行时量无盘上字段）逐句直译，未增未减 |

## 表二：FACT 编号对照来源（数值、函数、结构，非自然语言转述，不涉及翻译，只核「数抄对了没有」）

| FACT 编号 | 数值或结论 | 来源文件:行（冻结副本与工作区一致，已各现查一次） |
|---|---|---|
| A1 | NODE_BYTES = 16384 | `crates/singlefs-format/src/lib.rs:15` |
| A2 | INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86 | `crates/singlefs-format/src/lib.rs:47` |
| A3 | NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29 | `crates/singlefs-format/src/lib.rs:30` |
| A4 | `index_node_header_bytes` 函数体 | `crates/singlefs-format/src/lib.rs:50-54` |
| A5 | `index_node_entry_capacity` 函数体 | `crates/singlefs-core/src/unit.rs:126-131` |
| A6 | NODE_POINTER_BYTES = 86 | `crates/singlefs-format/src/lib.rs:87` |
| A7 | `internal_entry_width_in_bytes` 函数体（key_width + 86） | `crates/singlefs-core/src/code_two_tree.rs:118-120` |
| B1 | ACCOUNTING_KEY_BYTES = 22；字段拆分 [2,8,4,8] | `crates/singlefs-format/src/lib.rs:126`；`crates/singlefs-core/src/code_two_tree.rs:37` |
| B2 | ACCOUNTING_ENTRY_BYTES = 34 | `crates/singlefs-format/src/lib.rs:128` |
| B3 | ACCOUNTING_INTERNAL_ENTRY_BYTES = 108 | `crates/singlefs-format/src/lib.rs:131` |
| B4/B5 | 叶容量 477、内部容量 150（本 agent 用公式 A5 现算，命令见运行记录） | 由 A5 代入 B1/B2、B1/B3 算出；本轮正文 `research/prompts/_c363b-r1-body.md:15` 独立给出同一对数 |
| B6 | `MultiLevelCodeTwoTree` 只有两个变体；角色映射函数只对这两个角色返回 Some | `crates/singlefs-core/src/transaction.rs:1275-1278`；`crates/singlefs-core/src/transaction.rs:2709-2726` |
| B7 | `height_read_from_the_root_node_header`：level+1，无内存旁路 | `crates/singlefs-core/src/transaction.rs:1775-1783` |
| C1 | MAPPING_KEY_BYTES = 27；字段拆分 [1,8,8,4,6] | `crates/singlefs-format/src/lib.rs:90`；`crates/singlefs-core/src/code_two_tree.rs:40` |
| C2 | MAPPING_ENTRY_BYTES = 55 | `crates/singlefs-format/src/lib.rs:93` |
| C3 | CENTRAL_MAPPING_INTERNAL_ENTRY_BYTES = 113 | `crates/singlefs-format/src/lib.rs:134` |
| C4/C5 | 叶容量 294、内部容量 143（本 agent 现算） | 由 A5 代入 C1/C2、C1/C3 算出；本轮正文同一处 |
| C6/C7 | 同 B6/B7 | 同上 |
| D1 | ALLOCATION_RECORD_KEY_BYTES = 10 | `crates/singlefs-format/src/lib.rs:120` |
| D2 | ALLOCATION_RECORD_BYTES = 20 | `crates/singlefs-format/src/lib.rs:123` |
| D3 | 单节点、level 恒 0、装不下就拒绝发布（812 由 A5 代入 D1/D2 算出） | `crates/singlefs-core/src/transaction.rs:841-858` |
| D4 | `AllocationTree` 角色在角色映射函数里返回 None；`height_read_from_the_root_node_header` 只接受两个变体 | `crates/singlefs-core/src/transaction.rs:2709-2726`（`AllocationTree` 那一支）；`crates/singlefs-core/src/transaction.rs:1775` |
| E1/E3 | kb 引文，见表一 | `.claude/kb/decisions/08-核心索引结构.md:377`；`:382` |
| E2 | 812 与 D3 同一个数 | 由 A5 代入 D1/D2 算出，与 D3 引用同一次计算 |
| E4 | 「实现员还没开工」 | 本轮正文 `research/prompts/_c363b-r1-body.md:18`；`crates/singlefs-core/src/transaction.rs:1275-1278` 里没有第三个变体，交叉核实 |
| F1 | 树表 level 恒 0；角色映射返回 None；kb 引文见表一 | `crates/singlefs-core/src/transaction.rs:4871-4873`（build_index_node 的 level 实参）；`crates/singlefs-core/src/transaction.rs:2709-2726`（`TreeTable` 那一支）；`.claude/kb/decisions/08-核心索引结构.md:382` |
| F2 | TREE_TABLE_KEY_WIDTH = 8 | `crates/singlefs-core/src/make_filesystem.rs:47` |
| F3 | TREE_TABLE_ENTRY_BYTES = 200 | `crates/singlefs-format/src/lib.rs:140` |
| F4 | 断言文本；未找到树表专属的拒绝函数 | 断言：`crates/singlefs-core/src/unit.rs:160-163`；未找到的检索命令见运行记录 |
| G | kb 引文，见表一 | `.claude/kb/decisions/08-核心索引结构.md:321` |
| G2 | 代码注释「⌈n÷2⌉」，非 kb 决议，仅代码引用 | `crates/singlefs-core/src/code_two_tree.rs:10` |
| G3 | 本 agent 自己从 G 的规则 3 推出的观察，非引文 | 不适用（对 FACT G 内容本身的读法说明，不是另一处来源） |
| H | kb 引文，见表一 | `.claude/kb/decisions/28-挂载期承诺量.md:94` |
| H2 | 本 agent 核实：全篇给的材料里没有一处给出这个量的公式 | 通读本轮附录 `research/prompts/_c363b-r1-appendix.md` 全文（含 D28 已定项 1、4，D16 已定项 1、9）与 kb 决策 03/08/16/28 号原文，未见对应公式 |
