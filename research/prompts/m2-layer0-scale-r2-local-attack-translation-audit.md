# m2-layer0-scale-r2 本地攻方腿：逐句核转述

按 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」：每一句转述与原文并排，缺一个限定词就补上；英文比原文多出来的限定词、括注单列一行并写明为什么加。

列：英文项（用于提示里的原样英文）/ 原文文件:行 / 首稿缺的 / 定稿说明。

## 一、Context 段与 Table 1（背景与基线闭式）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Context 第一段："A write barrier (an fsync-like flush point) closes off the segment before it; a 'crash state' is defined as: some prefix of segments are fully persisted, the one segment where the crash happens has some subset of its own writes persisted, and every segment after that has nothing persisted." | `.claude/kb/decisions/13-验证路线.md:71`「屏障把录下来的写请求流切成段，一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」」 | 首稿漏了原文同一句里的「枚举的单位是一次整写」「撕裂子集不枚举，一次写的任何撕裂态一律并进「这次写没持久」」「「没持久」的位置上，镜像里放的是崩溃前那个位置的旧字节，不是零」三处 | 定稿不补：这三处讲的是撕裂态与镜像取值，本轮问题只问候选甲二的子集计数公式，不涉及撕裂语义；补进去会把模型的注意力引到与本题无关的撕裂概念上。判定为「不抄，因为与本题无关」，写在这里存查，不写进提示正文 |
| Table 1 Row 1（切段规则）："A barrier closes the segment before it (the barrier itself counts as belonging to the segment it closes); a segment with zero writes so far (only the very first barrier at the head of the stream) is folded into the segment about to begin instead. An FUA (force-unit-access) write always closes the segment it itself belongs to, because an FUA write does not make any earlier plain write persistent, so those earlier plain writes share its segment and any subset of them may be the ones that persisted. A trailing run at the very end of the stream that has only barriers and no writes is folded into the previous segment; every recorded step lands in exactly one segment." | `crates/singlefs-harness/src/segments.rs:90-92`「屏障关掉它之前那一段（屏障算在被关掉的那一段里）；段里还一个写都没有时（流首那道屏障）它并进即将开始的那一段；FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）；流尾只有屏障没有写的那一串并进上一段——录到的每一步都恰好落在一个段里。」 | 首稿漏了「（屏障算在被关掉的那一段里）」这个括注；核对后在定稿里补上 | 补的一处已列在「首稿缺的」列；另加「may be the ones that persisted」把原文「任意子集」的省略式展开成完整从句，同一件事，不算新增事实 |
| Table 1 Row 2（基线闭式）："The baseline (un-reduced) closed form is: crash-state count = 1 + the sum, over all segments, of (2^w_i minus 1), where w_i is segment i's own write count (barriers are not counted as writes)." | `crates/singlefs-harness/src/segments.rs:145`「E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。」 | 无遗漏 | 多出一句括注「(barriers are not counted as writes)」：这不是文档注释一行本身写的，是现读同一函数体（`closed_form_state_count` 内 `.filter(|kind| **kind != StepKind::Barrier)`）核实的，加是为了让模型不把屏障也算进 w_i；已现查函数体坐实，不是猜的 |

## 二、Context 第二段（候选甲与候选甲二）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| "An earlier reduction candidate for this project, call it candidate A, proposed collapsing every subset of a segment's COW writes into one single class equivalent to 'none of them persisted' -- so no per-COW-write subset ever needed enumeration at all." | `research/prompts/m2-layer0-scale-r1-local-attack.md:5`「if the recovered/visible file-system state does not depend on whether that new location is fully persisted or not persisted...then any subset of just those COW writes persisting gives the exact same recovered visible state as none of them persisting」 | 首稿的转述省了原句「because nothing that recovery could pick will ever reference that new block, and journal replay never reads it either」这个条件从句 | 定稿不补：本轮问题不要求判断候选甲的前提是否成立（那是第一轮已经判过、也是这一轮 M1 格云端攻方的事），这里只需要点出候选甲「把任意子集塌成一个类」这个計数规则本身，判定为「不抄，因为与本题的算术无关」 |
| "This project's own review process later found a mutation that is only caught when a state exists where a specific in-place write did not persist AND at least one COW write did persist; candidate A cannot represent such a state, so it missed that mutation entirely (zero detections on that mutation, versus a nonzero count for the refined candidate below on the same spot check)." | `research/prompts/m2-layer0-scale-r1-main-verification.md:19`「甲 0 个红、甲二 12 个红、全量取样 828 个红...要红须「取号的原地写没落∧某个COW写落了」，甲的代表状态摆不出来」 | 无遗漏：「0个红」「12个红」分别译成 zero detections、nonzero count（未写具体的 12，因为这句只用来交代背景，不是本轮问题的数） | 加了「(a mutation)」这个泛称，原文说的是「第304行」那一条具体变异，这里为了不引入本轮禁写的代码行号而改成不点名的「a mutation」，指路仍留在 Table 1 Row 3 的引用行 |
| "Candidate A was set aside, replaced by a refined candidate, called candidate A2 here, which is the one this leg's questions ask you to apply." | `research/prompts/m2-layer0-scale-r1-main-verification.md:28`「甲出局。约简的候选只剩甲二...」 | 无遗漏 |「甲」「甲二」音译成 candidate A / candidate A2，是本地腿英文提示的通用译法（与上一轮同名做法一致） |

## 三、Table 2（写种类的分类规则）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 2 Row 2（模块文档注释六种写）："a whole-segment zero-fill, a unit write, a journal-record write, a root-slot FUA write, a system-configuration-slot in-place overwrite, and a barrier." | `crates/singlefs-harness/src/segments.rs:6`「整段清零、写单元、写 journal 记录、根槽 FUA 写、系统配置槽原地覆写、屏障。」 | 无遗漏，六项逐一对应 | 未加字；「原地覆写」直译成 in-place overwrite，与本提示分类规则用的同一个词一致 |
| Table 2 Row 4（journal 环定长）："The journal is implemented as a fixed-size ring buffer; its record slots are reused (overwritten in place) as the ring wraps around, rather than the journal growing without bound." | `.claude/kb/decisions/23-journal的角色与格式.md:93`（已定项 2 定案）「取定长环」，及同一已定项标题「journal 是定长环还是链式」 | 首稿只译了「定长环」三个字，缺「链式」候选被否掉这一层对比 | 定稿不补链式候选的细节：本题只需要「journal 是定长环、槽位被复用」这一个事实支持「journal 环记录槽是原地覆盖」，链式候选为什么被否与本题无关，判定为「不抄，因为与本题的分类判断无关」 |
| Table 2 Row 5（分类规则本身）："a write's kind counts as an in-place overwrite if and only if it is exactly named root_slot, or it is exactly named system_configuration_slot, or it is a record-slot write inside the journal ring; every other kind counts as a copy-on-write (COW) write. If...you cannot decide with confidence...answer 'cannot determine'...do not guess." | 本轮派发指令（主 agent 消息，非仓内文件）：「每段哪些写算原地覆盖，照 KINDS 行的种类判：root_slot、system_configuration_slot、journal 环记录槽是原地，其余是 COW。判不了的写「判不了」，不猜。」 | 无遗漏 | 加了「exactly named」，把原文反引号标出的 `root_slot`、`system_configuration_slot`（看起来是字面标识符）与未加反引号的「journal 环记录槽」（描述性短语）的区别显式化，是为了不让模型把三者都当成字面字符串比对；这一处来源不是仓内文件，是主 agent 本轮直接给这条腿的任务定义，已在提示行内注明「it is not a quotation from any repository file」 |
| Table 2 Row 6（root slot 命名先例）："An earlier round of this same three-way check...already used the English name 'the root slot' for this exact in-place-overwrite category...root-slot writes, system-configuration-slot writes, and journal-ring record-slot writes were named together as 'exactly three kinds of in-place overwrite slot in this system'." | `research/prompts/m2-layer0-scale-r1-local-attack.md:5`「There are exactly three kinds of in-place overwrite slot in this system: the root slot, the system-configuration slot, and the record slot inside the journal ring」 | 无遗漏 | 加了一句提醒「it does not by itself tell you whether...root_record_fua...is the same category as...root_slot -- you still have to decide that」：这是主动加的限定，防止模型把这条先例事实误当成已经替它做完分类判断，属于为避免误导而加的限定词，不是新增事实 |

## 四、Table 3（原样数据行）与 Table 4（对照数字）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 3 Row 1 STREAM E 整行 | `research/prompts/m2-layer0-scale-r1-opus-model/outputs/p1.log:22` | 不适用：数据整行抄，非叙述性转述 | 已用 python 逐字节比对，STREAM E 原文与提示引文完全一致 |
| Table 3 Row 2 KINDS E 整行 | `research/prompts/m2-layer0-scale-r1-opus-model/outputs/p1.log:23` | 不适用：数据整行抄 | 已用 python 逐字节比对，KINDS E 原文与提示引文完全一致 |
| Table 3 Row 3（对应关系说明）："The Nth bracketed group in Row 2's list...describes exactly the writes in the Nth segment of Row 1's sizes list...in the same order" | `crates/singlefs-harness/src/segments.rs:160`「登记表的「种类」串：每段一个多重集，按枚举声明序排成规范形、`×N` 计数，段之间用 `|` 隔开」 | 无遗漏（本行只借用「每段一个多重集」这一结构性事实，不逐字翻译原句） | 补充的「in the same order, left to right」是本 agent 自己用 python 解析 55 个分组、55 个大小逐位核对之后确认的观测（见运行记录），不是原文字面，已在提示 Question 1 里要求模型自己重做一遍这个核对，不是把结论喂给它 |
| Table 4 各行数字（5575606380、174、210）与「第二条流」说法 | `research/prompts/m2-layer0-scale-r1-main-verification.md:22, 42, 45` | 不适用：数据整行引用 | 210 出自第 45 行「甲二 | 冻结形状 210」，与第 42 行表头「第二条流（新形状待量）」、第 22 行「到 E 425 写/55 段/全量 5575606380/甲174/甲二210」三行合看，确认这三个数字对应的正是 STREAM E / KINDS E 这同一条流，已现查三行原文 |

## 五、Table 2 Row 1、Row 3（辅助事实，非叙述性转述）

Table 2 Row 1（四个种类字符串枚举）与 Row 3（`write_accounting.rs` 里存在字面 `root_slot` 标识符）都是直接读代码/日志得到的观测事实，不是中文原文的转述，不逐句列入本表；来源分别是 `research/prompts/m2-layer0-scale-r1-opus-model/outputs/p1.log:23`（KINDS E 行本身）与 `crates/singlefs-core/src/write_accounting.rs:87`，均已现查坐实。
