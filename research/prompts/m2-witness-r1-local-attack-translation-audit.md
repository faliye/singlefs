# m2-witness-r1 本地攻方提示：逐句核转述

核对表：英文项 / 原文文件:行 / 首稿缺的 / 定稿。每一句译自中文原文的句子都在这里对照一遍；英文比原文多出来的限定词、括注单列一行并写明为什么加。

## 1. Candidate A 定义句

英文项（定稿）：Placed into the system-configuration slot, allowed to grow past 512 bytes; relies on the system configuration's already-existing whole-slot checksum and two-slot rotation to detect a torn write.

原文文件:行：`research/prompts/_m2-witness-r1-background.md:14`，clause (a)：「放进系统配置槽、越过 512 字节，靠系统配置已有的校验和与两槽轮换认撕裂」。

首稿缺的：无——首稿与定稿同一版，一次写对。

多出来的（需要单列并写理由）：
- 「allowed to」（越过 512 字节译成「allowed to grow past」而不是直译「crosses past」）：原句字面没有「允许」二字，但同一份背景材料里 D22（单元原子性怎么合成） 已定项 9 讨论同一个系统配置槽时逐字写「内容允许跨过 512」（`.claude/kb/decisions/22-单元原子性怎么合成.md:197`）——加「allowed to」是把候选 A 明确放进这条已有政策的射程，不是新造的限定。
- 「whole-slot」（校验和译成「whole-slot checksum」而不是「checksum」）：原句字面只说「校验和」，「整槽」二字来自同一份决策 D22 已定项 9 的「整槽校验和」字段名（`.claude/kb/decisions/22-单元原子性怎么合成.md:195`）——加这个词是为了让候选 A 与 F8/F9 引用的同一个校验和字段对上号，避免模型把它当成一个新字段。

## 2. Candidate B 定义句

英文项（定稿）：An independent self-certifying unit, separate from the system-configuration slot, carrying its own generation number and its own whole-structure checksum (use the F9 widths: 8 bytes generation, 32 bytes checksum), with two-slot rotation of its own.

原文文件:行：`research/prompts/_m2-witness-r1-background.md:14`，clause (b)：「独立的自证单元，自己带校验和、两槽轮换」。

首稿缺的：无——首稿与定稿同一版。

多出来的（需要单列并写理由，这一条份量较重）：
- 「carrying its own generation number」：原句字面只说「自己带校验和」（carrying its own checksum），没有提「世代号」。加它的理由：D22（单元原子性怎么合成） 已定项 21 逐字「原地覆写的结构必须『整单元校验和 + 被实际检查的世代号』，两样缺一不可」——这是本项目对「自证结构」这个词的通用要求，候选 (b) 若只带校验和、不带世代号，按已定项 21 就不构成一个合格的自证结构；本提示的算式（F9 的 40 = 8 + 32）也要用到世代号这 8 字节。这个添加不是我凭空推的，但它确实超出了背景材料第 14 行这一句字面写的范围，所以单列在这里。
- 「separate from the system-configuration slot」：这是对「独立」（independent）一词的展开转述，未改变原意，只是把「独立」具体化成「与系统配置槽分开」，判为同义展开、不算新增限定词。
- 「of its own」（两槽轮换译成「two-slot rotation of its own」）：强调这两槽是候选 B 自己的两槽、不是系统配置的两槽，属于同义强调，不算新增事实。

## 3. Candidate C 定义句

英文项（定稿）：Split into several pieces, each piece no larger than 512 bytes, each piece self-certifying on its own (each piece carries its own generation number and its own whole-piece checksum, same F9 widths).

原文文件:行：`research/prompts/_m2-witness-r1-background.md:14`，clause (c)：「拆成几片、每片不超过 512 字节、各自自证」。

首稿缺的：无——首稿与定稿同一版。

多出来的：
- 括注「each piece carries its own generation number and its own whole-piece checksum, same F9 widths」：与候选 B 同一处理由——原句字面只说「各自自证」，没有展开自证具体靠哪几个字段；加这个括注是把 D22 已定项 21「校验和 + 世代号缺一不可」这条通用要求套到候选 C 的每一片上，为了让 W5 的算式（per_piece_overhead = 41）有依据、不是凭空钉一个数。这个添加同样超出第 14 行字面范围，单列在此。

## 4. F5（N_w 的数）

英文项（定稿）：Across four tested geometries, three of them (named GEOMETRY_PRIMARY, S16, and a small-ring geometry) each need at most 3 concurrently-alive rollback witnesses at their peak; the fourth geometry (named S4, whose root ring has a smaller capacity of 12 slots, tested with an extra search depth of 4) needs at most 4 concurrently-alive rollback witnesses at its peak.

原文文件:行：`.claude/kb/experiments/158-择根与修复四岔路.md:14`：「GEOMETRY_PRIMARY/S16/小环 三个几何点历史数、峰值、正步数逐字段相同（856 段历史，峰值 3），S4 因根环容量小（12 槽）在 L=4 下出现峰值 4（7264 段历史）。四个几何点合计 N_w = max(3,3,3,4) = 4。」

首稿缺的：首稿只写了「tested with an extra search depth of 4」，漏了原句「S4 因根环容量小（12 槽）」这半句——S4 峰值更高的原因原文给了两个共存的限定：根环容量小（12 槽）与搜索深度 L=4，首稿只留了后一个。定稿已补上「whose root ring has a smaller capacity of 12 slots」。

定稿：已把两个限定词都写进 F5（见上）。

省略（判为不改变数值、不算漏限定词）：原句括注的「856 段历史」「7264 段历史」是枚举总量，不是 N_w 本身的定义输入，省略不改变 N_w = 3 或 4 这个数，未补回。

## 5. F8 第一句（自证结构点名哪几类）

英文项（定稿）：For a self-certifying structure in this format (the source line names three examples: root-ring slots, journal record headers, and the system-configuration slot; the general property they share, given elsewhere in the same decision, is that they are not protected by a parent pointer's checksum and must authenticate themselves), the tear-detection resolution equals whichever physical block size is detected at runtime, and this width must never be hardcoded into the format.

原文文件:行：`.claude/kb/decisions/20-承重面单元的原子性与自包含.md:68`：「自证单元（根槽、journal 记录头、系统配置槽） | 等于运行时探测到的 `physical_block_size`，不许硬编码。这一句说的是撕裂判定的分辨率，不是结构尺寸……」

首稿缺的：首稿把「根槽、journal 记录头、系统配置槽」这三个点名的例子换成了一句自己归纳的通用定义（「a structure that is not protected by a parent pointer's checksum and must authenticate itself」），没有把原文点名的三个例子写出来。定稿已把三个例子加回来，通用定义降级为括注里的补充说明（并注明它「given elsewhere in the same decision」，与原句本身分开）。

## 6. F8 第二、三句（跨过 512 仍可检测）

英文项（定稿）：A write no larger than one physical block lands as a single atomic physical write. A write larger than one physical block can be partially applied, but the partial application is still fully detectable, because the structure carries its own whole-structure checksum together with a generation number that is actually checked, and either one alone is not enough.

原文文件:行：`.claude/kb/decisions/22-单元原子性怎么合成.md:197`：「内容允许跨过 512（探测到的 `physical_block_size`）。跨过之后一次写可能落一半，由整槽校验和（覆盖整槽 4096 含补齐）判出，择槽规则（校验和过且世代号最大，已定项 16）自动回退到另一个槽。可检测、可恢复。」

首稿缺的：首稿把这句的出处误标成 `decisions/20-承重面单元的原子性与自包含.md:84`（D20 已定项 4 的「射程」段），已核对发现「内容允许跨过 512……可检测、可恢复」这句实际出自 D22（单元原子性怎么合成） 已定项 9、在 `decisions/22-单元原子性怎么合成.md:197`，不在 D20。定稿已改到正确的文件与行号（见上文 F8 的 Source 行）。

定稿补充：原文「世代号最大」是择槽规则的一部分（怎么回退到另一个槽），我的转述简化成「a generation number that is actually checked」，没有展开「择槽取世代号最大」这一步——这一步对 F8 要证明的「可检测」结论不是必需的（可检测靠校验和，不靠择槽规则本身），判为合理简化、不算漏限定词。

## 7. F9（系统配置自证开销 8+32）

英文项（定稿）：The system-configuration slot's own self-certifying overhead, which is the established pattern this project already uses, is an 8-byte generation number (named "slot generation number") plus a 32-byte whole-structure checksum (named "whole-slot checksum"), 40 bytes together.

原文文件:行：`.claude/kb/decisions/22-单元原子性怎么合成.md:195`：「系统运行量（不是配置） | 槽世代号 8、整槽校验和 32、journal tail 8、实例代号 4 | 52」。

首稿缺的：无——首稿与定稿同一版。

省略（判为有意的取舍范围、单列说明）：原表这一行合计 52 字节，除了「槽世代号 8」「整槽校验和 32」，还有「journal tail 8」「实例代号 4」两个字段。英文项只取了前两个（8+32=40），没有把 journal tail 与实例代号也算进「自证开销」。理由：journal tail 与实例代号是运行状态字段，不是「结构自己证明自己没撕裂」这件事需要的字段（对应 F8 的校验和 + 世代号两件套），排除它们没有把原文的 52 字节谎报成 40——英文项从未声称「52 = 自证开销」，只取了这一行里与自证直接相关的子集，判为合理的范围选择，不是漏限定词。

## 8. F10（第一版跑几块盘）

英文项（定稿）：The first runnable version of this filesystem targets exactly 2 devices.

原文文件:行：`.claude/kb/decisions/02-RAID条带策略.md:148-150`：「#### 已定项 9：第一个可运行目标跑 2 块盘 / 定案：第一版不跑单盘，第一个可运行目标跑 2 块盘；单盘走单独一条线，那条线开线时再定。」

首稿缺的：无——首稿与定稿同一版。

多出来的：
- 「exactly」：148-150 行字面没有「恒」或「exactly」这个词。加它的理由：同一份决策文件更下面的正文（`decisions/02-RAID条带策略.md:181`）逐字「w = 2 的条带成员一死即整条回收（1 数据 + 1 副本，无跨单元耦合）⇒ 第一版（2 盘恒 `w = 2`）自动满足」——「恒」字明确写在同一份决策里，只是不在 148-150 这两行内；加「exactly」是为了让 devs=2 在本提示的算式里被当成一个不随行代入变化的常量，不是我凭空推的。

省略（判为与本轮算式无关、不算漏限定词）：原句「单盘走单独一条线，那条线开线时再定」——单盘路径是另一条独立开发线，与本轮 W5 只需要 devs=2 这一个常量无关，未译入。

## 9. F6（C331 候选乙的名字）

英文项（定稿）：A second, separate change under discussion (identified as C331 candidate B) proposes adding one extra checkpoint-txg-typed field to the system-configuration slot.

原文文件:行：`.claude/kb/checks-owed.md:285`，C331 行内「乙『系统配置带 txg』挡不住且零故障（取号那一步 `write_system_configuration_slot` 的 txg 参数是占位的 0）」。

首稿缺的：无——首稿与定稿同一版。

省略（判为与 W5 算式无关、不算漏限定词）：原文括注「挡不住且零故障（取号那一步……占位的 0）」是候选乙在 C331 那道题（择根倒挂）上的判决结果，与本轮 W5 只借用它的「8 字节」这个宽度事实无关，未译入；F6 的英文项只陈述「有这个候选、宽度 8 字节」两件事，没有替候选乙在 C331 上是否挡得住下判断。
