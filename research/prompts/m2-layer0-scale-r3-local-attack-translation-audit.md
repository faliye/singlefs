# m2-layer0-scale-r3 本地攻方腿：逐句核转述

按 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」：每一句转述与原文并排，缺一个限定词就补上；英文比原文多出来的限定词、括注单列一行并写明为什么加。

列：英文项（用于提示里的原样英文）/ 原文文件:行 / 首稿缺的 / 定稿说明。

## 一、Context 段与 Table 1（基线闭式、候选甲二定义）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Context 第一段："a crash state is: some prefix of segments fully persisted, one segment where the crash happens has some subset of its own writes persisted, and every later segment has nothing persisted." | `.claude/kb/decisions/13-验证路线.md:71`「一个崩溃状态是「前若干段全部持久，当前段任意子集持久，之后各段全部没持久」」 | 无遗漏（三个分句逐一对应） | 首稿即不补同一句里「枚举的单位是一次整写」「撕裂子集不枚举…」「「没持久」的位置上镜像放旧字节不是零」三处：这些讲撕裂语义与镜像取值，本轮十道题只问候选甲二与代价表的算术，不涉及撕裂概念；判定为「不抄，因为与本题算术无关」，不写进提示正文 |
| Context 第二段、Table 1 Row 2："Candidate A2 is defined as: for each segment, take every subset of that segment's in-place overwrites, combined with that same segment's COW writes restricted to exactly two choices (none of the COW writes persisted, or all of the COW writes persisted); the one combination where the entire segment persisted completely is still counted under the next segment, not under this one." | `research/prompts/m2-layer0-scale-r1-main-verification.md:28`「约简的候选只剩甲二（每段每个原地覆盖子集 × COW 取 {∅, 全集}，整段全落那一个仍归下一段）」 | 无遗漏；「仍」译成「still」 | 多出「not under this one」：由「归下一段」（去下一段）的对比义显式化，不是新增事实，是把原文隐含的对比讲白；不抄「攻方自己提的，被攻过零轮」——那半句讲候选的来历与验证状态，与本轮算术复算无关，判定为「不抄」 |
| Context 第二段句尾（Context 段独有，Table 1 Row 2 没有）："(the one case where the whole segment fully persisted is still counted under the next segment instead, not under this one)" | 同上 `r1-main-verification.md:28` | 不适用：这是 Context 段对同一定义的简写复述，不是新译一句 | 与 Table 1 Row 2 同一来源，两处措辞略有差异（Context 段更短）是为了先在背景段给一个概览、再在 Table 1 给可引用的完整定义，两处内容一致，没有分歧 |
| Table 1 Row 1："crash-state count = 1 + the sum, over all segments, of (2 raised to the power of w_i, minus 1), where w_i is segment i's own write count (barriers are not counted as writes; the function that implements this filters barriers out before counting)." | `crates/singlefs-harness/src/segments.rs:145`「E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。」 | 首稿未抄「E77（发布的持久顺序）」这个决策分项编号 | 定稿不补：这是给公式挂靠的分项引用编号，不是公式本身的限定词，补进去要求模型认一个与本题算术无关的编号；与第二轮本地攻方同一行的判法一致（该轮同样判「不算遗漏，是编号引用不是限定词」） |
| 同上，括注部分："(barriers are not counted as writes; the function that implements this filters barriers out before counting)" | `crates/singlefs-harness/src/segments.rs:151-155`（`closed_form_state_count` 函数体 `.filter(\|kind\| **kind != StepKind::Barrier)`） | 无遗漏 | 多出这句括注：文档注释本身只写「|段| 按写数算」，没有明说屏障算不算写；现读同一函数体的 filter 逻辑坐实屏障被过滤掉，加这句是为了不让模型把屏障也计入 w_i，已现查函数体、不是猜的 |

## 二、Table 2（第二条流的两份段序列报告）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 2 表头与 Row 1 前半："One report was produced by a test program whose own doc comment says it reconstructs, on an in-memory disk, the same steps and the same parameters, step by step, as the prepare() function in a different test file named second_transaction_step_zero_layer0.rs -- it does not run that other test file directly." | `research/prompts/m2-layer0-scale-r1-opus-model/opus_scale_probe.rs:200`「固定脚本（照 second_transaction_step_zero_layer0.rs 的 prepare，逐步同名同参），内存盘上重建。」 | 首稿曾把这句写成笼统的「不是项目的真实测试装置」，没有点出这句判断的直接来源 | 定稿改为直接转述该函数自己的文档注释原话（重建、内存盘、逐步同名同参），并把来源从旁证（`m2-layer0-scale-r2-opus-output.md:330`「与第一轮同一套 Sim…不是仓里那份落文件的 prepare」）换成一手来源（`opus_scale_probe.rs:200` 那条函数自己的文档注释），現查后确认一手来源文字更精确、更不依赖推断 |
| Table 2 Row 1 引用行 "STREAM E writes=425 ... reduced_closed_form=174" | `research/prompts/m2-layer0-scale-r1-opus-model/outputs/p1.log:22` | 不适用：数据整行抄 | 已用 python 逐字节比对，原文与提示引文完全一致（见运行记录） |
| Table 2 Row 2："A different report was produced today by running that other test file (second_transaction_step_zero_layer0.rs) itself, directly, on the project's real test harness." | `records/2026-09-24-里程碑二收尾调度.md:162`「第二条流（固定脚本到 E）今天的段序列：`…second_transaction_step_zero_layer0.rs:287` 的断言当场红，实际 `[…]`」 | 首稿漏了「的断言当场红」这一层——今天量出的实际序列与该测试文件里钉死的旧数组不一致 | 定稿不补断言失败与旧数组这一细节：一是它要点名一个测试代码的行号（`:287`），本提示的格式要求 4 不许模型的答复出现代码行号，写进事实表容易被模型顺手照抄；二是「今天实际量出的序列是多少」这件事本身已经完整写进本行，断言失败与否不影响本轮要问的算术核对，判定为「不抄，因为与本题算术无关且涉及代码行号」 |
| Table 2 Row 2 数字子句："423 writes total, 54 segments total, and a closed form of 5575802973 states" | 同上 `records/2026-09-24-里程碑二收尾调度.md:162`「423 次写、54 段，闭式 5575802973 个状态」 | 无遗漏 | 未加字 |

## 三、Table 3（速率、外推、代价表的两轮记法）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 3 Row 1："run on a frozen copy of the project on 16 threads while the Opus attacker leg (a separate, concurrently running heavy task on the same machine) was competing for CPU" | `records/2026-09-24-里程碑二收尾调度.md:162`「速率（副本，16 线程，与 Opus 攻方抢着 CPU）」 | 首稿把「Opus 攻方」泛化成了「another concurrent heavy task (a separate attacker leg)」，丢了具体是哪一条腿 | 定稿改回点名「the Opus attacker leg」，与原文「Opus 攻方」一一对应，不再泛化 |
| Table 3 Row 1 数字子句："LAYER0_PROGRESS ... finished_states=4194320/67108885 elapsed_seconds=302.2" | 同上 `records/2026-09-24-里程碑二收尾调度.md:162` | 不适用：数据整段抄 | 已核对与原文逐字节一致（见运行记录） |
| Table 3 Row 1 补充句："The 67108885 figure in this measurement is the first stream's total state count, not the 5575606380-state second stream in Table 2 -- this rate was measured while enumerating a different stream than the one it is later applied to." | 本行没有单独一句原文对应；由同一行 162 内另一子句「第一条流…67108885 个状态」与本行分母 67108885 数值相同推出 | 不适用：这是本 agent 自己核对两个数字相同得出的连接句，不是逐句转述 | 这句是额外加的推论提醒，不是抄原文；加它是因为第五、六、七问要求模型判断代价表把「速率」与「状态总数」用在了两条不同的流上，不点破这个连接，模型未必会主动去查分母 67108885 是哪条流的数；已现查 162 行内「第一条流…67108885 个状态」这一子句坐实分母出处 |
| Table 3 Row 2："at this rate the second stream on 32 threads would take about 2.3 days" 与 "labels this figure as derived" | `records/2026-09-24-里程碑二收尾调度.md:162`「照这个速率第二条流 32 线程约 2.3 天（推的）」 | 无遗漏 | 「推的」译成「derived (not itself measured)」，是本轮沿用的固定译法（与「量过的」对应的 measured 相对） |
| Table 3 Row 3："on the frozen shape, 5575606380 states" labeled measured；"on 32 threads, about 2.3 days" labeled derived | `research/prompts/m2-layer0-scale-r2-main-verification.md:38`「全量：冻结形状 5575606380 个状态（量过的闭式），32 线程约 2.3 天（推的，按 13880 个 / 秒线性外推）」 | 无遗漏 | 未加字 |
| Table 3 Row 4："on the frozen shape, 210 states" labeled derived，"as using candidate A2 as an everyday fast lane instead of full enumeration" | `research/prompts/m2-layer0-scale-r2-main-verification.md:38`「甲二做平时快档：冻结形状 210 个状态（推的，按闭式算），秒级」 | 无遗漏 | 多出「instead of full enumeration」：原文这半句本身就是同一行两个选项的对照（「全量」与「甲二做平时快档」并列），加这几个字把并列关系显式化，不是新增事实 |
| Table 3 Row 5："candidate A2: frozen shape 210 states, second-level wall clock" | `research/prompts/m2-layer0-scale-r1-main-verification.md:45`「甲二 | 冻结形状 210 | 秒级（推的）」 | 无遗漏 | 三格表格行译成一句陈述，字段没丢 |
| Table 3 Row 6 整段 | `research/prompts/m2-layer0-scale-r2-main-verification.md:10`「甲二的 210 按段内写数算、没跑甲二的枚举器——本地攻方与核查员各自复算也是 210，但三处用的是同一条公式，不构成三条互不共享前提的路径，交用户时这个数照写「推的，按闭式算」。」 | 无遗漏 | 原文「本地攻方」在源文档（第二轮判决）自己的时间参照系里指「这一轮（第二轮）的本地攻方」；本提示是第三轮的材料，改写成「A local-attack leg from an earlier round」以保持对本轮读者（本条腿自己）而言时间参照正确，指的是同一件事、同一份产物，只是叙述视角从「那份文档的当轮」换成「相对本轮的更早一轮」，不是新事实 |

## 四、Table 4（M1 那一段：262144/32768）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿说明 |
|---|---|---|---|
| Table 4 Row 1 引用行 "Q9 seq=UOOUOMSU ... elapsed_seconds=208.7" | `research/prompts/m2-layer0-scale-r2-opus-output.md:314` | 不适用：数据整行抄 | 已用 python 逐字节比对，原文与提示引文完全一致（见运行记录） |
| Table 4 Row 2："for the UOOUOMSU segment, out of a full enumeration of 262144 states, 32768 were flagged red; under candidate A2, 0 were flagged red" | `research/prompts/m2-layer0-scale-r2-main-verification.md:9`「M1：`UOOUOMSU` 那一段全枚举 262144 个状态里 32768 个红、甲二 0 个红」 | 无遗漏 | 未加字 |
| Table 4 Row 3："in the history UOOUOMSU, the segment at txg 25 has 2 in-place writes and 16 COW writes; across the full enumeration, 32768 states were flagged red by the record checker's second condition, while none at all were flagged red when the COW writes were restricted to just the empty set or the full set (candidate A2's COW treatment); candidate A2 flagged 0 states red across the entire stream's 334 states." | `research/prompts/m2-layer0-scale-r2-main-verification.md:16`「历史 `UOOUOMSU`（…）txg 25 那一段 2 个原地写、16 个 COW 写，全枚举 32768 个状态被记录核对器第二条判红，COW 取 ∅ 或全集时一个都不红——甲二在整条流 334 个状态上 0 个红。」 | 首稿把「txg 25」写成泛化的「one particular commit point」，丢了具体的 txg 编号 | 定稿改回保留字面「txg 25」：这不是文件行号，是领域内的事务组编号，不受格式要求 4（不写代码/文件行号）约束，没有必要泛化掉 |
| 同一处未抄部分：历史 UOOUOMSU 的完整步骤括注「（A、B 之后卸载重挂、覆盖写两次、卸载重挂、覆盖写、进程重开、写 16 字节小文件、卸载重挂）」 | 同上 `r2-main-verification.md:16` | 首稿即未抄 | 定稿仍不抄：本轮十道题只问这一段的写数分解（2 个原地写、16 个 COW 写）与两个状态计数（262144、32768）的算术关系，不问这段历史本身的操作序列；判定为「不抄，因为与本题算术无关」 |
| 括注 "(candidate A2's COW treatment)" | 同上 `r2-main-verification.md:16` | 不适用：这是加的括注，不是缺的 | 多出这句括注：原句「COW 取 ∅ 或全集」本身就是候选甲二对 COW 写的处理方式（与 Table 1 Row 2 定义一致），加括注把两处呼应显式化，不是新增事实 |

