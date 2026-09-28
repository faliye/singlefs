# m3-prune-gpu-r3 本地攻方提示 逐句核转述

来源：`research/prompts/_m3-prune-gpu-r3-body.md` 第 19、51、57 行（A1 候选定义、本地攻方分工、本地攻方事实表要求）；`research/prompts/m3-prune-gpu-r1-facts-case-design.md` 表 2.1（第 72、75-87、92-93 行，节点数与全量枚举次数）；`research/prompts/e161-preregistration.md` 第 172 行（显卡容量读数）；`research/prompts/m3-prune-gpu-r1-facts-kv.md` 第 200 行（2¹⁶ 块假设）；`crates/singlefs-checker/src/image.rs:68`、`crates/singlefs-checker-tier/src/crash.rs:485`、`:487`、`:1168`、`:1190`、`:143`、`:159`（49+2+2 项目出处）；`research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` 第 48、147 行（E161 分段计时原始行）。

栏：英文项（提示文件行号） / 原文文件:行 / 首稿缺的 / 定稿。首稿即一次写成的定稿；没有二稿修订，这里记「无缺失，首稿即定稿」的按此写。

| 英文项（提示行） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| "identified by: <code file name>::<verification target name>::<process step number>::<hash string>"（提示 15 行） | `_m3-prune-gpu-r3-body.md:19`「被验的形态：`<代码文件名>::<验证目标名>::<流程步号>::<哈希串>`」 | 无缺失 | 逐字对应，四段用 :: 连接的原样保留 |
| "The process step number is composed of: which path on the operation tree, which operation step, which segment, and which write within the segment, plus a subset index, ending at one state. Step numbers are only ever appended, never reordered or reclaimed."（提示 15 行） | `_m3-prune-gpu-r3-body.md:19`「流程步号 = 操作树上第几条路径、第几步操作、第几段、段内第几次写，加子集序号到一个状态，只追加不重排、不回收」 | 无缺失 | 「只追加不重排、不回收」译成两句 only ever appended / never reordered or reclaimed，没有漏「不回收」（reclaimed） |
| "the content hash of the bytes written by that step, the code digest at recording time, the verdict digest (a digest of which checker code judged the result), and the version table (the oracle's expected values)"（提示 17 行） | `_m3-prune-gpu-r3-body.md:19`「写出的内容哈希、录入时的代码摘要、判法摘要、版本表（oracle 的期望）是属性」 | 「判法摘要」原文没有展开是什么摘要 | 补了括注 "a digest of which checker code judged the result"，这是我加的限定词，理由见下方「多出来的」一行 |
| "If only the checker changes: only re-check happens (no re-record)... If only the core changes: only the step whose content-hash attribute changed, plus every step below it in the same identity chain, gets re-recorded... If neither changed... zero re-recording and zero re-checking."（提示 19-21 行） | `_m3-prune-gpu-r3-body.md:19`「重录、重核按属性变没变判（只改 checker 只重核，只改 core 只重录内容哈希变了的那一步与它下面的分支，换机器换人身份相同、结果直接复用）」 | 「换机器换人身份相同、结果直接复用」没写「零重录零重核」这个显式数量 | 补了 "with zero re-recording and zero re-checking"，是我加的量化措辞，原文只说「直接复用」，理由见下方「多出来的」 |
| "candidate H1 is the flow definition only (this step's operation sequence and parameters, not the written bytes and not the code version)"（提示 25 行） | `_m3-prune-gpu-r3-body.md:19`「流程定义（这一步的操作序列与参数，不含写出的字节与代码版本）」 | 无缺失 | 逐句对应 |
| "candidate H2 is the written content (the bytes actually written by this step)"（提示 25 行） | `_m3-prune-gpu-r3-body.md:19`「写出的内容」 | 原文只有三个字，没有展开定义 | 补了括注 "the bytes actually written by this step" 消歧，理由见下方「多出来的」 |
| "candidate H3 is both of the above, concatenated"（提示 25 行） | `_m3-prune-gpu-r3-body.md:19`「两样都放」 | 原文没说怎么放（顺序、有没有分隔符） | 补了 "concatenated"，是我加的具体化措辞，原文「都放」没定形式，理由见下方「多出来的」 |
| "candidate R1 is freeze at registration (the identity keeps the file name it had at first registration; a separate registry records which file it lives in now)"（提示 27 行） | `_m3-prune-gpu-r3-body.md:19`「登记时冻结文件名（身份取第一次登记时的名字，登记表另记现住哪个文件，推的、没量过这个仓一年改几次名）」 | 「推的、没量过这个仓一年改几次名」这个限定没译 | 见下方「原文比英文多的」一行，故意不译进候选定义本身（那是对这个候选可信度的旁注，不是候选定义的一部分），但登记在这里以防漏检 |
| "candidate R2 is follow the current name (renaming the file changes the identity of every step recorded under that file name)"（提示 27 行） | `_m3-prune-gpu-r3-body.md:19`「跟着现名走（改名一次那个文件下的身份全变）」 | 无缺失 | 逐句对应 |
| 事实表分工来源（提示 SECTION 1-3、Q2-Q6）："改一次 core、改一次 checker、改一次文件名各要重录多少状态、重核多少状态（按今天三条整流的段数组与事实表戊的 61 个节点）" | `_m3-prune-gpu-r3-body.md:57` | 「61 个节点」在提示里没有整份列出，只挑了 T1、T4-T19、T32、T33 共 21 个代表节点 | 挑选理由写在提示 Q2-Q4 里（早期共享前缀、P2 中段、P2 尾段、P3 分支），运行记录里如实报「只挑代表节点、没有整份 61 行」 |
| "each 1 × 268435467 (four nodes, same count each)"（提示 50 行，T13-T16） | `research/prompts/m3-prune-gpu-r1-facts-case-design.md:84`「T13–T16 \| 依次 \| 覆盖写 txg 11–14（28 × 4）\| 同 T12 \| 各 1 × 268435467」 | 无缺失 | 「各」译成 each，四个节点同一个数这件事保留 |
| 显卡容量五行（提示 66-70 行） | `research/prompts/e161-preregistration.md:172`「本机两张卡：5090 total 32607 MiB...5060 Ti total 16311...另一台 5080 16303 MiB、5060 Ti 16311 MiB × 2 是主 agent 转述的读数，我没现查」 | 无缺失 | 「是主 agent 转述的读数，我没现查」译成 "relayed by the lead agent and not independently measured"，三张卡（5080、两张 5060 Ti）都标了这一句，没有漏标任何一张 |
| "49" pool-level checker invariants（提示 76 行） | `crates/singlefs-checker/src/image.rs:68` `IMPLEMENTED_INVARIANTS: [&str; 49]`（现查） | 无缺失（数字现查，不是转述） | 数字与出处现查得出，不是从中文材料译的 |
| "2" oracle passes、"2" record checker flags（提示 77-78 行） | `crates/singlefs-checker-tier/src/crash.rs:1168`、`:1190`（oracle 两遍）；`:485`、`:487`、`:143`、`:159`（记录核对器两条标记，现查） | 无缺失（现查） | 与 `_m3-prune-gpu-r3-body.md:57`「逐项原始判定每状态字节（49 条不变量 + 两遍 oracle + 记录核对器两条）」的计数口径一致 |
| E161 两行原始数据（提示 82、84 行） | `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:48`、`:147` | 无缺失，整行照抄字段名与数值 | 只选了 first_segment_head 与 second_segment_head 两格（各 4096 个状态），没有把另外三格（first_small、first_quick、second_quick）一起放进去；提示第 80 行已写明只取这两格，样本量偏小的警告也保留（"one machine, other load present, so treat as order-of-magnitude reference not a clean benchmark"，对应中文事实表乙「量级参考」一句） |
| "One KV block holds 2^16 = 65536 states."（提示 88 行） | `research/prompts/m3-prune-gpu-r1-facts-kv.md:200`「一块取 2¹⁶ 个状态」，标「算术 + 推的」 | 「推的」这个限定没有直接写进这一句 | 见下方「原文比英文多的」，在 SECTION 7 标题里补了 "marked as an assumption, not measured"，覆盖了这个限定 |

多出来的（英文比原文多的限定词或括注，及为什么加）：

1. "a digest of which checker code judged the result"（提示 17 行，判法摘要的括注）：原文「判法摘要」四个字没展开，本地模型读不到中文附录里对「判法摘要」的定义，不加括注会不知道摘要的是什么，加这句是为了让候选定义可执行地被模型使用，不改变判法摘要本身「是一个属性、不进身份」这件事。
2. "with zero re-recording and zero re-checking"（提示 21 行）：原文「结果直接复用」没有显式给出「零重录零重核」这个数量表述，但这正是提示要模型在 Q2-Q6 里填的那类数字格的对照基线（身份相同时两个数都是 0），加这半句是为了让模型知道这一分支在数字表里该填 0、不是「不适用」，不改变原文语义。
3. "the bytes actually written by this step"（H2 括注，提示 25 行）与 "concatenated"（H3，提示 25 行）：原文「写出的内容」「两样都放」都没有展开到可执行的程度（写出的内容具体指哪些字节、两样怎么放到一起），这两处括注是为了让 H1/H2/H3 三个候选在 Q2-Q6 的表格题里可以被逐格区分开填数，不改变候选本身「哈希什么」的意思。
4. "marked as an assumption, not measured"（SECTION 7 标题，提示 86 行）：覆盖了原文戊表 200 行「算术 + 推的」里「推的」这个限定，见上表 R1 与 KV 块两行标出的缺口，在标题里统一补上，不放进候选定义句子本身。

原文比英文多、英文没译出来的（登记在案，不动候选定义本身）：

1. `_m3-prune-gpu-r3-body.md:19` R1 候选原文有「推的、没量过这个仓一年改几次名」这个可信度限定，提示 27 行的 R1 候选定义句里没有这半句——因为这半句评的是「这个候选是不是站得住」，不是「这个候选是什么」，放进候选定义会让模型把「没量过」当成候选定义的一部分去做数字题，故意留在候选定义之外；已在上表单独登记，运行记录里会注明这一处不译的理由。
