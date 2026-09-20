# 翻译核对表：m2-supp3-item2-code-r1 本地攻方提示

每行：英文项（提示里的编号）/ 原文文件:行 / 首稿缺的或错的（已补/已改） / 定稿怎么写。行号现查（`grep -n`），不从背景材料数。

| 英文项 | 原文文件:行 | 首稿缺的/错的 | 定稿 |
|---|---|---|---|
| Q1 | `.claude/kb/decisions/04-校验和位置.md:287` | 无缺漏；"恒 32768 字节"与四个组成部分（自描述头/声明长度/净荷/补齐）全部对应到英文 | "The unit is always 32768 bytes; the self-describing header, the declared length, the payload, and the padding are all within these 32768 bytes." |
| Q2 | `.claude/kb/decisions/18-块里携带什么信息.md:944` | 首稿把「诞生代号」译成 "birth instance number"，与本提示第 4 节另行定义的 "instance"（实例/时间线）撞词——原文「诞生代号」与「实例代号」是两个不同字段（同文件 208 行 `(诞生代号, 实例代号, 事务号)` 并列为三个独立字段），用 "instance" 会让模型把这一个字段与后面的时间线字段混同。已改译 "birth generation number"，不再含 "instance" 这个词 | "...67 anchor offset, 75 birth generation number, 83 fsid..." |
| Q3 | `.claude/kb/decisions/18-块里携带什么信息.md:919` | 原句后半段还有码 3（打包记录单元）声明长度上界的重算（"已定项 11 的『声明长度 ≤ 32768 − 107』、cap(107, 56) = 583……按 136 重算：⌊(32768 − 136) / 56⌋ = ⌊32632 / 56⌋ = 582，与按 135 算的 582 同值，加这 1 字节之后 582 不变"）。这一段只管码 3，这一轮的题只问码 1、码 2，与 M1 算术那一半的分工无关（M1 只测分配记录树节点与数据单元内容上限两道算式），故略去，未译入提示 | 提示只到 "...the ceiling is 32768 minus 134 for code 1, and 32768 minus 136 for code 3." 为止 |
| Q4 | `.claude/kb/decisions/08-核心索引结构.md:512` | 原句括注「偏移表见 D18（块里携带什么信息） 已定项 18」是指路，不是数值限定词，略去无损 | "...The header including the 29-byte reserved area is 115 plus 2 times the key width." |
| Q5 | `.claude/kb/decisions/03-空间分配.md:422` | 无缺漏，key/value 两段的字节数、已释放标志位置、码 2 头按 key 宽 10 算，四句全部对应 | 见提示 Q5 |
| Q6 | `.claude/kb/decisions/18-块里携带什么信息.md:711-713` | 无缺漏；三种单元大小与依据（D4 已定项 1、已定项 11 登记表）、步进结论 16384 全部对应 | 见提示 Q6 |
| Q7 | `.claude/kb/decisions/16-发布语义.md:374` | 无缺漏；"min(...)"两项、"不足 4 个取最旧的有效根"、"一次处置的目标"三句全部对应；"处置"译成 "reclamation pass"是意译，不是逐字，已在本行注明——这句在本提示的计算规则（Section 4）里不实际使用，只为整行引用完整 | 见提示 Q7 |
| Q8 | `.claude/kb/decisions/16-发布语义.md:372` | 括注「（C282（环里最旧根没有定义） 要的定义）」是指路，不是数值限定词，略去无损 | 见提示 Q8 |
| Q9 | `.claude/kb/decisions/16-发布语义.md:375` | 无缺漏 | 见提示 Q9 |
| Q10 | `.claude/kb/decisions/16-发布语义.md:381` | 首稿用省略号「...」跳过了中段，其中恰好包含原文明写的「被抛弃时间线上的根带的 F 算不算」——这正是问题 7 要模型判断的那一件事，原文把它列成 2026-09-17 打回重议的未决项之一，删掉会让模型看不到"这件事本身在决策文本里是待定的"这条线索。已改写，把这句未决项与另外两条未决项（怎么防回落、checker 该用哪个 F）保留进去；「I-7.4 在这段历史上不成立」那段具体机制描述与 k_tol 一句、以及与本题无关的另外两件并议事项（空池非空校验、挂载内不回收）略去，因为它们不改变"F 算不算"这句未决项的可读性 | 见提示 Q10（"What is being redebated includes: ... whether F carried by a root on an abandoned timeline counts toward this computation; ..."） |
| Q11 | `.claude/kb/decisions/16-发布语义.md:383` | 无缺漏；"用户可见的树（inode 树、extent 树）"、"按 txg 排"、"有效 = 按实例表判仍然有效 ∧ txg ≥ 当前的 F"三处限定词全部保留——注意这一处"有效"的定义（多了 "txg ≥ 当前的 F" 一条）比 Q8/环里最旧有效根用的"有效"定义窄，提示原样两处都译出，不替模型合并 | 见提示 Q11 |
| Q12 | `research/prompts/m2-supp3-item2-implementer-report.md:76`（报告自己的行号，`grep -n` 现查） | 首稿的转述是"F_生效 = 各盘所带 F 最大值的最小值，被抛弃时间线上的根带的 F 也算——预想，跟收口表第 ② 行"；英文保留了"预想"（anticipated）与"收口表第②行"（open item 2 in the closing table）两个限定词，不当成已定案的口径 | "F effective equals the minimum, across disks, of the maximum F carried by that disk; a root on an abandoned timeline still counts toward this maximum. This is an anticipated reading, tied to open item 2 in the closing table." |

## 提示里多出来的限定词（原文没有，本提示自己加的）

| 位置 | 多出来的 | 为什么加 |
|---|---|---|
| Section 1 开头 | "these citations were checked against the source file before this prompt was written" | 告诉模型行号已核过，不是它要重新验证的对象；不影响原文任何一句的取值 |
| Section 4 全节 | 六条规则 V / X / P / Y / C / F 整节 | 原文「抬 F 的上限」「生效」两行本身没有把"每块盘上最新的有效根""第 4 新的非空有效根""各盘所带 F"这几个短语拆成可执行的算法——这是本轮攻方为了让题目"能落成数、能逐格判"而做的操作化改写，提示里已明写"这些是为这次练习写的白话复述，不是逐字引用，与 Section 1 分开看"，不冒充决策原文 |
| Table 1–5 全部数据 | 全部为本轮攻方按条款构造的合成数据，不是仓里任何已有的记录 | 任务要求"你按条款造"；已在每张表标题的括注里写明构造意图（基线、根数不足 4、某盘落后、连续空发布、抬 F 后回退） |
