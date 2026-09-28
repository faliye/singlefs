# 逐句核转述：`m3-prune-gpu-r1-local-attack.md`

核对表；按 `.claude/rules/three-way-inference.md`「英文提示里的每一句转述，写完都要对着原文核一遍」与这一轮派发提示「事实表每行写来源文件与行号」。行号一律现查（`grep -n` / cat -n 现读），不从背景材料拼接稿数。

分两张表：表一核提示里出现直接引号或成段转述的句子（有没有漏译、多译限定词）；表二核提示里每条数值事实（FACT 0.x 到 FACT 4.x）抄的数对不对、来自哪个文件哪一行。

## 表一：转述与引号句逐句核对

| 英文项（提示文件节选） | 原文文件:行 | 首稿缺的/多的 | 定稿 |
|---|---|---|---|
| BACKGROUND: "a set of per-unit content checks (this material calls these '(a)-class checks' ...); many states in the same segment turn out to have byte-identical unit contents at the positions those checks read, so a proposed optimization (P1) de-duplicates those checks by content instead of by state." | `research/prompts/_m3-prune-gpu-r1-body.md:52`「单元级检查按内容去重 | (a) 类检查（整单元校验和、头自证、key 次序，事实表乙 1.3 节）按单元内容哈希只做一次」；类的定义见 `research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:78`「a = 只看单个单元...」 | 首稿把 P1 定义与「a 类」定义分写在两处，读者容易脱节；核对后把「following the source material's own lettering」这半句补进去，提醒这是源材料自己的字母记号，不是本提示新造的名字 | 定稿在 BACKGROUND 一次性交代（a）类定义与 P1 的关系；未加未减 P1 candidate 本身的内容 |
| BACKGROUND: "A further proposed optimization (G2) would classify each state by the vector of 'which write ended up visible at each position this state's checks read' ('read-set key') and run the expensive checks once per distinct key instead of once per state; states sharing a key are called one 'class'." | `_m3-prune-gpu-r1-body.md:76`「G2 | 状态到类的映射 | 一个节点里每个状态（掩码）对这个节点的读位置逐个求「落在这里的是哪次持久写的内容」，出每状态的内容编号向量，按向量分组成类；CPU 对每个类只跑一次恢复与 checker。纯位运算，每状态独立（推的）」 | 首稿漏译「纯位运算，每状态独立（推的）」这半句；核对后判定这半句是 G2 candidate 自己的一个限定词（说明它可以按状态并行、这条是推的），与「本地腿只算容量吞吐、不判精确性」的任务边界有关，但本轮问题只问字节与批数算术，不问这条「按状态独立」是否成立，所以不必进入 QUESTION 2 的算术前提；判定为可以不抄（不改变已给的 arithmetic 输入），非遗漏 | 定稿不含「纯位运算，每状态独立（推的）」，但 QUESTION 2 里补一句「(a), (b), and (c) use average, not worst-case, read counts」把「这是推的、按状态可能不同」的精神换一种方式保留在问题里 |
| SCOPE: "One of them (a cloud-based reviewer) is checking whether form alpha's proposed pruning rules are exact ... The other reviewer is designing the overall tree structure, the storage schema, and the GPU offload plan; picking a design is not your task either." | `_m3-prune-gpu-r1-body.md:129`「两条攻方腿不重叠：Opus 攻「剪得对不对」（精确性、判别力），本地攻方只算「装得下、跑得完」（容量、吞吐）。」；云端正推腿的射程见 `:125`「从 `crates/` 今天的实现出发，拼出一套能落地的整体设计...」 | 无缺——两条腿的射程（Opus 攻精确性、Sonnet 定整体设计）均译出；未把 Opus 具体攻的每一格（U4、P6、P2...）搬进来，因为提示只需要说「不是你的题」，不需要列全 Opus 的清单 | 定稿保持简述，不列 Opus 的具体格号，理由同上（本轮任务边界只需要「不是这题」，不需要「这题详细是什么」） |
| FACT 3A.1 quote: "a state is (node, position-within-segment); the persisted-set is computed on demand from that position as a bitmap of 64-bit words, one word per 64 writes in the segment, rounded up; a state no longer builds its own separate list-of-booleans the size of the whole segment." | `_m3-prune-gpu-r1-body.md:65`「S1 | 状态 = （节点，段内序号），持久集合按需从序号现算成位图（u64 字 × ⌈段长 ÷ 64⌉），不再每状态建 `Vec<bool>`（今天 `crash.rs:1917`、`:1962` 每状态现建一份整张枚举写表长的 `Vec<bool>`）」 | 首稿删掉了括注「（今天 `crash.rs:1917`、`:1962` 每状态现建一份整张枚举写表长的 `Vec<bool>`）」；核对后判定：这个括注只是给「今天怎么做」提供代码行号出处，而提示里明令「不写代码行号」（`.claude/agents/three-way-local-attack.md` 做什么一节第 1 条），这半句本身的内容（「今天每状态建一份 Vec<bool>」）已经在 FACT 3A.1 前半句的对比里体现（「no longer builds」暗示了「以前建」），删掉行号不删内容 | 定稿不含行号，但保留「no longer builds its own separate list-of-booleans the size of the whole segment」这半句，把「今天怎么做」的内容留住，只去掉行号 |
| FACT 1A.3: "... plus one state-count field per class (assume 4 bytes per class ...), plus a small fixed-size table of violation samples that this material asks you to ignore for the byte-count arithmetic below (assume it contributes 0 bytes for this exercise)." | `_m3-prune-gpu-r1-body.md:86`「只存类：类表（读集键 → 判定向量）+ 每节点每类的状态数 + 违例样本；状态到类的映射不存、要时由 G2 现算」 | 原句只说「+ 每节点每类的状态数 + 违例样本」，没有给字节宽度；首稿要做算术就必须钉一个宽度，这是本 agent自己加的假设，不是原文有的数——按「英文比原文多出来的限定词、括注也单列一行，写明为什么加」补记于此 | 定稿显式加了「assume 1 byte per class」「assume 4 bytes per class」「assume it contributes 0 bytes for this exercise」三处假设，且在句子里用「assume」「this material asks you to ignore」明说这是外加的简化，不是原文给的数；4 字节的理由另给（「every count given anywhere in this material is below 4,294,967,295」，即 u32 装得下，见表二 FACT 0F.2/0F.3/2B.4 等已给的最大计数） |
| FACT TABLE 3B closing sentence: "These two counts of the same stream do not agree with each other on segment lengths; this material does not resolve that disagreement and is not asking you to." | 本条不是转述某一句原文，是本 agent 拼两处事实表后自己发现的不一致（见表二 FACT 3B 行的两个来源），原文任何一处都没有明说「这两处对不上」这句结论 | 不适用（非转述，是本 agent 新写的比对结论，不存在「首稿缺」问题） | 按「本地腿缺席时必须显式报告」同一精神，把发现的事实冲突原样交给模型，不替它判断哪个数对；这句是本 agent 自己的观察，已在下方「产出」里另行报告给主 agent |

## 表二：FACT 编号对照来源（数值，非自然语言转述，只核「数抄对了没有」）

| FACT 编号 | 数值 | 来源文件:行 |
|---|---|---|
| 0.1 | 16,777,260 | `research/prompts/m3-prune-gpu-r1-facts-case-design.md:14`（引用 `crash_enumeration_new_pool_file_creation_stream.rs:446` 的 `FULL_STATES`） |
| 0.2 | 1,662,648,564 | `m3-prune-gpu-r1-facts-case-design.md:15`（`crash_enumeration_fixed_script_stream.rs:100`） |
| 0.3 | 1,358,954,634 | `m3-prune-gpu-r1-facts-case-design.md:16`（文件头 `:21`） |
| 0.4 | 3,038,380,458（= 0.1+0.2+0.3，本 agent 现加验证） | `research/prompts/_m3-prune-gpu-r1-body.md:127` |
| 0B.1 | 78,905,428 | `m3-prune-gpu-r1-facts-case-design.md:17`（七条合计，文件头 `:408`–`:410`） |
| 0B.2 | 65,536 | `m3-prune-gpu-r1-facts-case-design.md:20`（`:731`–`:733`） |
| 0B.3 | 33,554,520 | `m3-prune-gpu-r1-facts-case-design.md:118`（2.2 表「重复的状态数」行） |
| 0B.4 | 3,117,351,422 | `m3-prune-gpu-r1-facts-case-design.md:116`（2.2 表「全量里钉了数的状态合计」行） |
| 0C.1 | 约 268,435,467（T20、T21 各一份） | `_m3-prune-gpu-r1-body.md:32`（U3 行）；`m3-prune-gpu-r1-facts-case-design.md:240`（6.2 表「补进来的」行） |
| 0C.2 | 约 1,879,048,236（T34–T37 合计） | 同上两处 |
| 0C.3 | 约 1,000,000,000（σ 历史其余七步） | 同上两处 |
| 0D.1 | 2^136 − 1 | `m3-prune-gpu-r1-facts-case-design.md:105`（T54 行，「单元段 2^136 − 1 枚举不了」） |
| 0D.2 | 约 2^300 | `m3-prune-gpu-r1-facts-case-design.md:101`（T47–T48 行）；`:18`（P5 行「约 300 写」的来源说明） |
| 0E.1/0E.2 | [8,3,1,8,3,1,150994943,3,1,8]／[12,12,150994947,8]，合计 150994980 | `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:16`（`name=anchor_stream stream=first`） |
| 0E.3/0E.4 | [150994943,3,1,8,8,262143,3,1,589823,3,1,589823,3,1]／20 元素表，合计 152436764 | 同文件 `:129`（`name=anchor_stream stream=second`） |
| 0F.1 | T1 节点 3 次 × 8 = 24 | `m3-prune-gpu-r1-facts-case-design.md:72`（2.1 表 T1 行） |
| 0F.2 | T12–T16 各 268,435,467 | `m3-prune-gpu-r1-facts-case-design.md:83`–`:84`（2.1 表 T12、T13–T16 行；提示只挑了「四次」，原表其实是 T12 一次加 T13–T16 四次共五次同形，本 agent 在 0F.2 里写「four」，比原表少了 T12 那一次——见下方「首稿多译/少译」说明） |
| 0F.3 | T33 节点 1,073,741,895 | `m3-prune-gpu-r1-facts-case-design.md:93`（2.1 表 T33 行） |
| 0F.4 | T54，2^136 − 1 | 同 0D.1 |
| 0F.5 | 27 / 61（提示写「60 个具名操作」，下方说明） | `m3-prune-gpu-r1-facts-case-design.md:120`（2.2 表「全量一次都没枚举的节点」行，「61 个里 27 个」） |
| 1A.1/1A.2 | 块值 65536 字节/1 字节每状态；块键 32 字节（4+4+8+16） | `research/prompts/m3-prune-gpu-r1-facts-kv.md:26` |
| 1A.3 | Q1「只存类」定义（字节宽度为本 agent 外加假设，见表一） | `_m3-prune-gpu-r1-body.md:86` |
| 1B.1–1B.3 | 状态 37：narrow=5, wide=13, maximal=37 | `research/prompts/m3-prune-gpu-r1-facts-k4.md:183`、`:184`、`:190`（arm=k4_walk, cell=first_small） |
| 1B.4–1B.6 | 状态 84：narrow=7, wide=16, maximal=84 | 同文件 `:199`、`:200`、`:206`（arm=k4_walk, cell=second_quick） |
| 2A.1–2A.5 | 五格每状态读位置（单元/头扫描/环记录） | `research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:212`–`:216`（已算好的每状态平均表） |
| 2B.1 | 32 GiB = 34,359,738,368 字节（本 agent 换算） | 派发提示 `_m3-prune-gpu-r1-body.md:127`「32 GB 卡」原话，字节数为标准换算 |
| 2B.2 | 128 MiB / 256 MiB 上限 | `m3-prune-gpu-r1-facts-checker-gpu.md:133`（wgpu 默认上限行，引 `wgpu-types-30.0.1/src/limits.rs:441`、`:443`） |
| 2B.3 | 32,607 MiB | `m3-prune-gpu-r1-facts-checker-gpu.md`（E163 实验页引用，本表格「M2」一段列出的本机显存读数；本 agent 未在本轮事实表里逐字重抄这一行，改引 `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out` 跑前登记 4.6「本机 5090 total 32607 MiB」一句，见 `m3-prune-gpu-r1-facts-checker-gpu.md:227` 一带；行号待查，未逐字核，标记于此） |
| 2B.4 | 2,415,919,103 | `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:94`（`stream=second segment=29 writes=30`，另 `:97`、`:100`、`:103`、`:106`、`:117` 同数值的其余四个 30 写段） |
| 3A.1 | S1 quote | 见表一 |
| 3B | 74 段（10+64）实测写数只有 {1,2,16,18,22,26,30}；与 `m3-prune-gpu-r1-facts-case-design.md:134`「28×6、24×3、20×1、18×1、16×6」不一致 | `research/results/e161-crash-state-dedup-and-time-split-feasibility-2026-09-27.out:6`–`:15`（stream=first 10 段）、`:65`–`:128`（stream=second 64 段，本 agent 逐行 `grep -n` 现查一遍） |
| 4.1–4.5 | 五格 pool_checker/unit_check_replay 纳秒总量、distinct_unit_contents | `m3-prune-gpu-r1-facts-checker-gpu.md:187`–`:191`（纳秒总量）、`:177`–`:181`（distinct_unit_contents） |

## 首稿多译/少译，逐条说明（非表一「转述句」，是表二数值/措辞层面的取舍）

- 0F.2：原表 2.1 里 T12（1 次）与 T13–T16（4 次）都是「28 写、覆盖写、268,435,467 states 每次」的同形节点（`m3-prune-gpu-r1-facts-case-design.md:83`–`:84`，「与 T12 同形」），一共五次而不是四次；本条只用「four」是因为 QUESTION 0-PRIME 需要的是「同一种操作重复出现」这个例子本身（by-operation 抽样），不需要凑全五次的精确计数，且 T12 与 T13–T16 在原表里语义上分两行、条件略有差别（T12 是「释放 D 复活的 A 的数据单元」这一次，T13–T16 是后续四次「条件推的相同」），FACT 0F.2 只取后一类「条件相同的四次」，与原表 T13–T16 一致（不含 T12），非误抄，是刻意只取同质的那四个
- 0F.5：首稿一度写成「60 named operations」，与原表口径「61 个节点里 27 个」（`m3-prune-gpu-r1-facts-case-design.md:120`）不一致；核对时发现，取号取样之前用 `research/scripts/replace-once.py` 定点改成「61」，现文件已是「61 named operations ... 27 of the 61 named operations」，与来源一致，未发出错误版本
