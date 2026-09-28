# m3-prune-gpu-r2 本地攻方提示：逐句核对表

逐句英文项 / 原文文件:行 / 首稿缺的限定词 / 定稿。原文一律整句抄，不摘句；核对法见 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节。

## FACT TABLE 0（P6、P7 定义与已验证到的范围）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 0.1 P6 在 12 写单元段上叶子数=状态数=4095、真判定只有 1 类 | `research/prompts/m3-prune-gpu-r1-main-verification.md:55`「精确（叶子内判定一致、子集数之和等于状态数），但单元段上叶子数 = 状态数 = 4095、判定只有 1 类（p6.out:2）」；数据源 `research/prompts/m3-prune-gpu-r1-opus-model/outputs/p6.out:2`（`states_in_unit_segment=4095 distinct_judgment=1 leaves_recovery_plus_checker_normal=4095`） | 首稿漏写「候选槽内容随持久子集变」这条机理，只写了数 | 定稿补一句「the later checking pass rereads a candidate on-disk position whose content varies with exactly which writes ended up persisted」（对应主档 55 行「池级 checker 的扫描读候选槽，候选随持久子集变」） |
| 0.1 该 8 段前缀与真实首 8 段同类型、只把单元段从 24 写缩到 12 写 | 本条是我自己现算得出、不是原文直接摘句：`p6.out:1` `writes=24 segments=8`，`p6.out:2` `states_in_unit_segment=4095`（=2^12−1，故单元段本身 12 写，其余 7 段共 12 写，恰与首条流前 7 段 2+2+1+2+2+1+2=12 写相同），首条流段序列见 `crash_enumeration_new_pool_file_creation_stream.rs:466` | 首稿容易把这条现算结果误写成「已证明」 | 定稿用「built to have the same segment types in the same order as」+ 全句仍标出这是数出来的规模关系，不称其为直接测量结论 |
| 0.2 同一世界里 8 段合计的叶子数仍恰好 4095，其余 7 段没有多贡献一类 | 同上 `p6.out:2` 整行；「4128」（`enumerated_with_torn`）与「4095」（`leaves_...`）两个数并存，本句只处理后者 | 首稿把 `enumerated_with_torn=4128` 与 `leaves=4095` 混着说 | 定稿只引用 `leaves_recovery_plus_checker_normal=4095` 这一项，不提 4128，避免引入未澄清的差额 |
| 0.3 材料原话把这条外推到 2^136、2^300 那几段，且原话自己标了「推的」 | `m3-prune-gpu-r1-main-verification.md:55`「2¹³⁶、约 2³⁰⁰ 那几段罩不住（推的，按同一机理外推）」 | 首稿漏了「推的」这一限定词，写成了确定语气 | 定稿加整句「stating in so many words that this extension is inferred from the same mechanism, not measured at that scale」 |
| 0.4 P7 没有代码、没有测量、没有已定的类数公式；第二轮的开题原话 | `m3-prune-gpu-r1-main-verification.md:112`「①P7 在单元段上能不能把叶子数降到判定类数，且 p5 那一格照样抓得到」；P7 定义原话 `:98`「池级 checker 的扫描方向拆成逐槽判与聚合，P6 只对进得了聚合的内容分叉」 | 首稿漏了「p5 那一格照样抓得到」这半句（精确性约束），本条只管代价、不管精确性，因此这半句按范围排除、不抄进 FACT，但在此核对表标注去向 | 未抄入 FACT 0.4；理由：这半句是精确性判据，归云端攻方腿的攻击面（正文分工表「云端攻方」B1–B7、D1 的精确性），本地攻方只管代价，故此半句故意不抄、留白核对 |

## FACT TABLE 1（身份值的字节宽度）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 1.1 状态序号 u64，8 字节 | `crates/singlefs-checker-tier/src/crash.rs:1917`（`fn persisted_writes_of_state(&self, ordinal: u64)`） | 无 | 直译 |
| 1.2 今天内容去重键 128 位、16 字节，40.7 秒撞出一对 16 KiB 内容 | `m3-prune-gpu-r1-main-verification.md:51`「键用的 128 位摘要 40.7 秒撞出一对 16 KiB 内容（collide.out:1）」 | 首稿漏了「或键相同时逐字节比内容」这条并列出路 | 定稿补上「or comparing the full content byte-for-byte whenever two keys match」 |
| 1.3 抗碰撞替代：256 位、32 字节，同一份代码里已用了两次 | `crates/singlefs-checker-tier/src/verdict_store.rs:28`（`SHA256_DIGEST_BYTES: usize = 32`）与 `:35`、`:39`（`InputFingerprint`、`EnumerationPlanHash` 各一份） | 无 | 直译 |
| 1.4 判定编号 1 字节、最多 256 种、第 257 种今天拒绝 | `verdict_store.rs` 错误枚举（`:281` 附近 `MoreThan256VerdictVectorsNotDecidedInTheFirstVersion`，行号未逐一现查、只引了枚举名，故此行按函数名/常量名指，不写行号） | 无 | 直译，未写行号（按本轮规矩答复不写行号，此处核对表按常量名指） |
| 1.5 判定向量本身是变长字节串、代码里没有定宽 | `verdict_store.rs:103`（`EncodedVerdictVector(pub Vec<u8>)`） | 无 | 直译 |

## FACT TABLE 2（KV 四张表键值布局）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 2.1 判定块键：`01` + 32B 指纹 + 2B 名长 + 名字 + 32B 计划哈希 + 8B 块起点 | `verdict_store.rs:14`（表行）、`:21`–`:22`（「块键 = 输入指纹（32 字节）‖ 流名或节点名的字节数（2 字节）‖ 流名或节点名 ‖ 枚举计划哈希（32 字节）‖ 块起点（8 字节）」） | 无 | 直译 |
| 2.2 判定块值：块里每状态 1 字节，块长由调用方定 | `verdict_store.rs:16`「块里每个状态一个字节：判定向量编号」；`:42`「块起点：这一块第一个状态在它那份枚举计划里的序号。块长由调用方定，不进键。」 | 无 | 直译 |
| 2.3 违例表键：块键 + 8B 块内序号；值 1 字节；只有违例的状态才有条目 | `verdict_store.rs:17`「违例 | `04` ‖ 块键 ‖ 块内序号（8 字节） | 那个状态的判定向量编号，一个字节」 | 首稿漏了「只有违例的状态才有条目、不违例的状态没有条目」这条隐含事实 | 定稿补一句「Only states that actually violate something are ever given an entry in this table; a state that violates nothing gets no entry at all」（推自表结构本身，非摘一行） |
| 2.4/2.5 判定向量按内容 / 按编号两张表 | `verdict_store.rs:15`、`:18` | 无 | 直译 |
| 2.6 三个真实名字的字节长（19/20/34） | `.claude/gate.d/stage-inputs.tsv:36`、`:37`、`:40`（三行的 `crash-case:` 名字）；长度现算（Bash `${#n}`，本次现查） | 无 | 直译，长度现算过 |

## QUESTION 1、3、4 里引用的三种块单位与实测数

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 片：第一条 257/片、第二条 25371/片、并行线一 20737/片 | `research/prompts/m3-prune-gpu-r1-facts-landscape.md:207`「第一条 257 个状态一片、65282 片；第二条 25371 个一片、65534 片；并行线一 20737 个一片、65533 片（`python3` 现算，没跑）」 | 首稿漏了「没跑，是现算的」这条限定 | 定稿写「already-computed slice sizes」，不称「测量」 |
| 固定 2^16=65536，独立于片长 | `m3-prune-gpu-r1-main-verification.md:69`「块 = 片（65536 个状态）」；`m3-prune-gpu-r1-facts-scale-up-inventory.md` 第九节 N3「续跑的片长...与 E162 的 2¹⁶ 块不是同一个量」 | 首稿把「块=65536」与「片长」混成一个词 | 定稿明确两者是两个不同的量、分别定义 |
| 节点：今天没有节点树，没有已定的数 | `m3-prune-gpu-r2-facts-scale-up-inventory.md` 第一节「今天没有节点」；`verdict_store.rs:12` 注释「名字写『流名或节点名』，今天没有节点」 | 无 | 直译 |

## FACT TABLE 3（两种按类存法与违例实测）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 3.1/3.2 一格一状态、按类两种定义（本条是本提示为练习自定的假设，非代码原文） | 定义沿用第一轮本地攻方提示同类假设的写法（`research/prompts/m3-prune-gpu-r1-local-attack.md:57`–`:61` FACT TABLE 1A 的定义方式），本轮改用真实 `verdict_store.rs` 键值宽度而非 E162 的假设宽度 | 无（本条本来就该标「为练习假设」而非「已实现」） | 定稿明写「for this exercise only, assume」 |
| 3.3 违例表 8+1=9 字节、非违例 0 字节 | `verdict_store.rs:17`、`:31`（`STATE_OFFSET_IN_BLOCK_BYTES = size_of::<u64>()`） | 无 | 直译 |
| 3.4 已有一次全量跑「oracle 违例 0、失败 0」，且原话自己标这是旧规模 | `research/prompts/e161-preregistration.md:161`（第 4.1 行「全量 oracle 违例 0、失败 0、journal 承重 3、验证跑过 6、验证失败 0」） | 首稿没有把「这条是旧规模、不代表今天四个目标」这条限定抄全 | 定稿整句「The source explicitly marks this particular recorded run as an older, smaller-scale version of the segment shapes discussed in this material, not a fresh measurement of today's exact segment sizes」 |

## FACT TABLE 4（GPU 显存与已测每状态字节）

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 4.1 理想 32 GiB 与实测卡 32607 MiB | 理想值为本轮任务原话「一张 32 GB 卡」的字面换算（本条括注写明是本轮任务措辞本身，不是仓里的数）；实测卡 `research/prompts/e161-preregistration.md:164`（第 4.6 行「本机两张卡：5090 total 32607 MiB」） | 首稿直接抄了第一轮本地提示 2B.3 算出的字节数（34,192,932,864），现查 32607×1048576=34,190,917,632，与第一轮那份不一致 | 定稿不抄第一轮算出的字节数，改成只给 MiB 原始读数、要求模型自己现乘，避免把上一轮的算术误差带进这一轮 |
| 4.2 wgpu 默认绑定 128 MiB、缓冲 256 MiB | `.claude/kb`（无直接 kb 行，出自研究材料）`research/prompts/m3-prune-gpu-r1-facts-checker-gpu.md:133`「`wgpu-types-30.0.1/src/limits.rs:441` `max_storage_buffer_binding_size: 128 << 20`...`:443` `max_buffer_size: 256 << 20`」 | 无 | 直译 |
| 4.4 第一条流 7 字节/状态（W=49）、第二条流 66 字节/状态（W=523） | `research/prompts/e161-preregistration.md:175`（第 4.9 行「第一条流 W = 49、7 字节/状态...第二条流整条 W = 523、66 字节/状态」） | 首稿漏了「这是显式存逐状态子集位图的代价，不是 1 字节判定码的代价」这条区分 | 定稿整句「not the compact 1-byte verdict code of fact 4.3」 |
| 4.5 单卡 3.419 字节/状态、五卡 10.26 字节/状态，10^10 假设规模 | `e161-preregistration.md:174`（第 4.8 行「显存预算：10¹⁰ 个状态时单卡每状态 3.419 字节、五卡每状态 10.26 字节」） | 首稿把这两个数当成「某种编码的实际大小」 | 定稿整句「The source states these two numbers as the limit itself...not as a chosen encoding's actual size」（原文本身就是这么写的：「凡按状态存下来的部分...每状态超过 10.26 字节，一次装入在五卡上也装不下」） |

## FACT TABLE 2（问题 2）目标 A–D 的段结构

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 目标 A：11 段与 16,777,260 | `crash_enumeration_new_pool_file_creation_stream.rs:446`（`FULL_STATES`）、`:466`（段数组）；段状态数拆分 `research/prompts/m3-prune-gpu-r1-facts-landscape.md:107`–`:121` 表 | 无 | 直译 |
| 目标 B：136+4+1+2、25 个状态 | `crash_enumeration_record_spill_over_stream.rs:7`–`:16`（头注）、`:382`（`1 + ((1 << 4) - 1) + ((1 << 1) - 1) + (3 * 3 - 1)`） | 无 | 直译 |
| 目标 C：78 段、1,662,648,564；17 个单元段（24×3、18×1、16×6、20×1、28×6） | `crash_enumeration_fixed_script_stream.rs:100`（`FULL_STATES_THROUGH_THE_UNMOUNT`）、`:388`–`:393`（注释「A 与 B 与 C 各 24、写行发布 18、写行之后的暖机各 16、回退 D 20、覆盖写与 E 各 28、抬 F 与卸载推的空发布各 16」）、`:439`–`:442`（段数组）；单元段计数由本次现数组逐项核对（python 现算，非仓里直接给出的汇总数） | 首稿曾把「17」误数成「18」，本次重新逐项计数并用 python 核验总和等于已定的 1,662,648,564 才定稿 | 定稿只给「exactly 3／1／6／1／6」五个计数，不给具体是哪一段、不给非单元段的 config/record/root 细分（61 段的内部拆分未在源码注释里逐段点名，本材料不代它推算，只给「剩下 61 段都是同类小段」这句話并留给模型用减法） |
| 目标 D：19 段、1,358,954,634；三个单元段 24/28/30、4 写记录段 15、6 写记录段 63 | `crash_enumeration_multi_record_publish_stream.rs:21`（头注合计数）、`:185`（段数组）、`:393`（`assert_eq!(closed_form, 1_358_954_634)`） | 无 | 直译 |
