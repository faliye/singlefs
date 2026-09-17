# 核对表：alloc-basis-r3-local-defense 提示英译逐句核对

每行：英文项（提示文件里出现的英译，摘出定位短语）/ 原文文件:行 / 首稿缺的 / 定稿。原文行号均为目标文件自身行号，现查（非从附录数）。

| 英文项（定位短语） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| Fact A: "Two of the statistics already have such an independent check... currently has no independent check at all."（转述，非逐字引用） | research/prompts/alloc-basis-r2-main-verification.md:31,36 | 无（转述而非逐字引用，核对内容与两处原文一致：只命中统计量 1、2；第 5 项 defer 没有任何检查在读） | 同首稿 |
| Fact B: "Candidate fix G8: from the allocation-record tree reachable from the newest root..."（引"收法候选 G8（从最新根的分配记录数『已释放 ∧ 释放代 > 这条根持久之后的回收门槛』，逐盘与第 5 项相等）"） | research/prompts/alloc-basis-r2-main-verification.md:36 | 无 | 同首稿 |
| Fact C: "The threshold it uses is exactly the runtime's reclaim predicate, which is in tension with..."（引"它用的门槛就是运行时的回收谓词，与 fs-design.md 第一格『运行时与 checker 必须用不同的算法』有张力"） | research/prompts/alloc-basis-r2-main-verification.md:36 | 无 | 同首稿 |
| Fact C 附注: "G8 has been attacked zero times so far"（引"被攻过零轮"） | research/prompts/alloc-basis-r2-main-verification.md:36 | 无 | 同首稿 |
| Quote 1: "What is forbidden is not 'traversal' itself, but two things..."（引"被禁止的不是「遍历」，是两件事：「在需要答案的那一刻还没有答案」，以及「审计与被审计用同一段代码」。第二条不依赖任何关于性能的假设——它要求的恰恰是运行时与 checker 必须用不同的算法。"） | .claude/rules/fs-design.md:34-36 | 无 | 同首稿 |
| Quote 2: "Runtime decision paths...checker / audit: traversal is required...collapse to zero on the spot."（引表格两行："运行时决策路径（...）\| 不许...\|不是慢...\|checker / 审计\|必须遍历\|若运行时也用遍历算...归零"） | .claude/rules/fs-design.md:23-24 | 首稿只译了「不是慢，是在被问到的那一刻没有答案」，漏译中间一句「准入控制要『进门前先算最坏情况』，而『释放空间这个操作本身不需要申请空间』也压在同一个数上」 | 补全该句："Admission control needs to compute the worst case before letting anything in, and the fact that the operation of releasing space itself should not require requesting space also rests on that same number." |
| Q2 引文 A: "The checker and the implementation share exactly one thing between them..."（引"checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（machine-first.md：重复要生成，不能手抄）。其余一律不共享：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用）已定的符号级判据，『格式解析与常量除外』那条例外就是这一项在用）。"） | .claude/kb/decisions/13-验证路线.md:369-372 | 首稿漏掉两处括注（machine-first.md「重复要生成，不能手抄」的引用理由；C12「格式解析与常量除外」那条例外的引用理由） | 补全两处括注，译文见提示文件第 31 行 |
| Q2 引文 B: "One boundary must be held: the generator only emits scalar values..."（引"一条边界要守住：生成器只发射标量值。它一旦开始发射『按字段表算出来的偏移函数』，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量。判据是发射物里有没有分支与算术。"） | .claude/kb/decisions/13-验证路线.md:397-398 | 无（"D13（验证路线）"译为 "this decision (D13, titled the verification route)"，语义保留） | 同首稿 |
| Q4 引文 A: "Each accounting statistic is independently maintained incrementally..."（引"记账的每个统计量各自独立增量维护（幂等完整值），而没有任何一条不变量盯住它们之间的和。"） | .claude/kb/checks-owed.md:80 | 无 | 同首稿 |
| Q4 引文 B: "Add a new invariant in the form of a vector equality..."（引"新增一条向量等式形态的不变量（形状照 I-3.5（引用区间的精确性） 对 I-3.1（已分配统计对得上） 的补法）：可用 = Σ设备(容量 − 已分配 − 不可回收 − defer 待释放) − 待删占用 − 已承诺预留，逐格判。"） | .claude/kb/checks-owed.md:80 | 首稿用省略号跳过中间括注「形状照 I-3.5 对 I-3.1 的补法」，中缺一段（摘句，违反"整行抄不许摘句"） | 去掉省略号，补全括注译文："in the same shape as how invariant I-3.5 supplements invariant I-3.1" |
| Q3 背景: "I-3.1 checks that the allocated-bytes statistic equals the sum obtained by actually walking every unit reachable from every root in the rollback candidate set."（转述"已分配空间统计 == 实际遍历所有引用得到的和"及其 checker 读法注） | .claude/kb/invariants.md:120 | 无（转述，核对内容与"2026-09-17 起『有效根』= 回退候选集里的根"一致） | 同首稿 |
| Q3 背景: "I-7.4 checks that no block referenced by any root in the rollback candidate set has had its physical range reassigned to something else or wiped by scrubbing."（转述 I-7.4 定义） | .claude/kb/invariants.md:50 | 无（转述内容对应"回退候选集里每一个根...所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头"） | 同首稿 |
| Q3 背景: "I-4.8 checks that, starting from any root in the rollback candidate set, every checksum along the walk matches what its parent pointer recorded."（转述 I-4.8 定义） | .claude/kb/invariants.md:153 | 无（转述内容对应"从回退候选集...里任一根出发遍历，所有块的校验和均与其父指针记录的一致"） | 同首稿 |

## 结论

首稿两处摘句/漏译问题（Quote 2 漏译中段一句；Q4 引文 B 用省略号跳过括注）在定稿写入 `research/prompts/alloc-basis-r3-local-defense.md` 之前已发现并补全；定稿文件本身不含省略号，逐条已核对无遗漏限定词。
