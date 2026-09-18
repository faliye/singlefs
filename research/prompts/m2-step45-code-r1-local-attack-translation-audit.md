# 核对表：m2-step45-code-r1 本地攻方腿英文提示的逐句核转述

针对 `research/prompts/m2-step45-code-r1-local-attack.md` 里每一句由中文源（kb 决策 / 不变量 / 里程碑正文，或代码里的中文注释）转述出的英文句子，逐句与原文并排核对。格式：英文项（prompt 里的标签）/ 原文文件:行 / 首稿缺的 / 定稿怎么补。

首稿是写提示文件之前脑内起草的直译，与最终写进提示文件的定稿逐句比较；「首稿缺的」记的是脑内直译里遗漏、最终定稿里补回的限定词。

## D1（回退下界 F 与可再分配）

原文：`.claude/kb/decisions/16-发布语义.md:371-376`（D16 已定项 1 的形态表，"可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根)" 起五行）。

- 首稿缺的：漏了「平时不动，只在准入不够时抬」这半句的「平时」二字——首稿写成 "F is only ever raised when admission control is short"，把「正常情况下它从不变」这个方向丢了，只剩"只在...时抬"，读者会以为它平时也可能变，只是这一条触发路径。
- 定稿：加回「under normal operation it never changes」一句，与"is only ever raised when..."并列，对应原文"平时不动"与"只在准入不够时抬"两个分句。
- 首稿缺的：漏了"每块幸存盘上都有带新 F 的持久根才生效"里的"幸存"（原文明确排除掉线的盘，只算 survive 下来的盘）。首稿写成 "every device"。
- 定稿：改成 "every surviving device"，对应 D16 已定项 1 表格"生效"那一行原文的"幸存盘"。
- 核对：候选集那句"按实例表判仍然有效 ∧ txg ≥ F_生效"逐字保留了合取符号对应的两个条件，未删减。

## D2（分配记录条目不删、改写）

原文：`.claude/kb/decisions/03-空间分配.md:175`（索引行）与 `:177-180`（正文）。

- 首稿缺的：首稿只写"the record is rewritten in place"，漏了"释放代仍写进那 8 字节"这类字段细节——但这些细节在 F 系列事实（F2、F13）里已经用代码行号覆盖，D2 本身只需要保留"改写而不删除、留到再分配时再改写一次"这条口径，不需要重复字段宽度。
- 定稿：D2 只保留"rewritten in place... rather than being deleted... left in the tree until the placement is reallocated, at which point the same record is overwritten again"，与原文"条目不删...改写成已释放 + 释放代...条目留到该落点被重新分配时覆盖"逐句对应，没有遗漏"改写"发生两次（释放一次、复用一次）这一点。

## D3（记账行：有值才写行）

原文：`.claude/kb/decisions/05-快照-空间记账机制.md:333`（已定项 4 标题，"其中一项带树维"）；"有值的统计量就写一行，没有对象的统计量不写行（读空 = 从未维护）"见附录背景材料转述、原文见同文件已定项 8 段（第一个事务写 15 行那段，appendix 行 254 对应源文件行号需另查，此处只用其口径句，不涉及具体行数，未逐字核对源文件行号——标记为待核）。
- 首稿缺的："读空 = 从未维护"这个等价关系首稿漏写，只写了"only writes a row when it has a value"，没说清"没有行"要读成什么。
- 定稿：加上"reading a key that has no row means that statistic was never maintained for that key, and must not be read as zero"，对应原文"读空 = 从未维护，不是 0"。
- 待核记录：D3 引用的"十四个统计量、有值才写行"这条口径的确切行号本核对表未逐行复核（appendix 未整段抄出该句所在的确切源文件行区间），只核对了语义方向；下一轮若引用 D3 的具体数字（十四、十五）要重新现查源文件行号。

## D4（I-2.1 校验和与内容匹配，2026-09-17 新注）

原文：`.claude/kb/invariants.md:107`。

- 首稿缺的："不抬 F 就回收复用时 A 的根还在候选集里、这一条红，是 C22 的必红"这句里的"必红"（mandatory regression，不是"可能红"）首稿写成"should go red"，弱于原文"必红"的强制语气。
- 定稿：改成"must make this invariant go red"，与"必红"的强制口径一致。
- 核对：""被引用"按 I-3.1 那个候选集里的根算"这半句在英文里对应"referenced is now defined relative to the I-3.1 candidate set of roots"，把"按...算"译成"is now defined relative to"，方向未变。

## D5（I-3.1 已分配统计对得上，候选集并集口径）

原文：`.claude/kb/invariants.md:120`。

- 首稿缺的：首稿把"按最新根指着的实例表有效"简化成"valid"，丢了"按最新根"这个限定——若不点明是"最新根"的实例表，读者可能以为是任一根自己的实例表。
- 定稿：改成"valid per the newest root's own instance table"，对应原文"按最新根实例表有效"。
- 首稿缺的：漏了"F 之下的根引用的已释放单元已可再分配"这句里的"已释放"这个前提——不是所有 F 之下的根引用的单元都被排除，是"已释放的"那些。核对原文："被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里"——这里两个从句主语不同（被抛弃时间线的根 vs F 之下的根），宾语也不同（引用的单元 vs 引用的已释放单元）。
- 定稿：拆成两句，"Units referenced only by roots on an abandoned timeline are isolated by the shadow ledger and are not counted; units referenced only by roots below F may already be reclaimed and reused and are likewise not counted"，把两个从句的主语与限定词分别对齐原文的两个分句，不再合并成一句丢限定词。

## D6（I-5.2 空闲统计对得上，独立维护）

原文：`.claude/kb/invariants.md:167`（不变量本身）；"两个数要走两条不共享的加减路径"见 appendix 行 690（`.claude/kb/decisions/05-快照-空间记账机制.md` 已定项 4 附近的 ⚠️ 注，对应源文件同一文件已定项 4 段落，本核对表未单独现查该 ⚠️ 注在源文件里的确切行号，只核对了 invariants.md:167 那一条不变量本身的文字）。
- 首稿缺的：首稿写"must equal total minus allocated"，没有强调"必须由两条独立路径分别维护、不能靠减法定义"这条附加要求（这条要求其实来自 D5 附近的 ⚠️ 注，不是 I-5.2 行本身；首稿把两处混在一起写成一句）。
- 定稿：保留 I-5.2 本身的等式陈述，另加一句"this equality must be demonstrated by two independently maintained counting paths, never by defining one in terms of the other"，对应"两个数要走两条不共享的加减路径"和"若空闲就是这么算出来的，那条不变量是恒真式，判别力为零"这条依据；因为这条要求的源头（05-快照-空间记账机制.md 已定项 4）未单独现查行号，D6 条目标注为"来自 I-5.2 本身 + 一条相邻依据合并转述"，未来引用要分开标源。

## D7（步 5 现状段末尾两个决策点）

原文：`.claude/kb/milestone/02-second-txn.md:189`（该段最后一句"决策点：checker 怎么认...F 只在一块盘上时新实例的根写 F_生效...是不是条款的意思"）。

- 首稿缺的：首稿把"F 只在一块盘上时"这个条件漏掉，直接写成"whether a newly established instance's root writing F_生效 conflicts with..."，没有点出这是"只有一块盘带着新 F"这种局部生效场景下才会出现的写法。
- 定稿：加回"an individual surviving device had already been carrying"这句里隐含的"只在一块盘上"语境，并在 F11 与 Q2 系列问题里显式追问"if the process crashes after the first new-floor publish... but before the second"，把"F 只在一块盘上"这个条件坐实成一个具体可核的操作序列，不再是抽象转述。

## D8（影子账窄读法，D23 已定项 14 与步 4 现状段）

原文：`.claude/kb/decisions/23-journal的角色与格式.md:1209`（"查账的集合 2026-09-16 用户定案取窄读法...宽读法...会让抬 F 在第一版 24 槽里买不到任何东西"）；`.claude/kb/milestone/02-second-txn.md:160`（步 4 现状段"影子账查的是「(txg, 实例) 大于 R_old 的可读根」的账（窄读法）"）。

- 首稿缺的：首稿最初写成"both readings are recorded as live options"（源自 appendix 附录二里 991 行"『查谁的账』条款有两种读法"这句转述，appendix 那句本身是步 4 的"预想的细节"段，写的是决策过程，不是最终结论）——这是本轮核对时发现的最大一处偏差：appendix 里确实写着"两种读法"，但现查决策文件本体（`23-journal的角色与格式.md:1209`）之后看到，2026-09-16 用户已经定案取窄读法，宽读法是被否掉的候选，不是"仍然并存的两个选项"。
- 定稿：改写成"this narrow reading was chosen, on 2026-09-16, over an alternative broad reading... the recorded reason for rejecting the broad reading is..."，把"两种读法都在"改成"其中一种已被否掉、原因是什么"，与决策文件原文的定案语气一致，不再暗示这是一个仍然开着的分支。这处修正直接影响 Q3a 与相关问题的措辞：提示里没有再问"选哪种读法"，而是问"这个已经选定的窄读法本身有没有可达的反例"。

## 其余（F 系列，源头是代码里的中文注释，非 kb 决策文本）

F3、F7、F11、F12 四条引用的代码注释（allocator.rs:511-512、mount.rs 169 附近的函数说明、mount.rs:291-293、transaction.rs:926-927）在写英文事实时逐句核对过，具体做法：先把中文注释整句抄进核对草稿，再逐词翻译，检查"仅""从不""恰好一次""在...之前""窄""大于等于"这几类限定词是否原样保留在英文里；F3 额外把"第一版根环 24 槽、种子 txg 0，环里最旧有效根恒 0，floor 就是 F_生效"整句按引号原样嵌入英文事实（未转述，直接给出中文原文加括注翻译），避免转述丢字。这几条因为直接对应单一函数的单一注释、上下文短，核对时没有发现遗漏限定词的情况；核对方式记录于此，供复查。

## 历史版本

（无。这是这一轮唯一一版核对表。）
