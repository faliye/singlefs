# 核对表：d13-item4-fua-r1-local-defense 英文提示逐句核转述

机械对照，逐行列：英文项 / 原文文件:行 / 首稿缺的 / 定稿。这一轮只有 Q4 一问，没有分条目编号，全部对照集中在这一份表里。

| 英文项（提示里出现的句子，概括标注） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 崩溃状态定义整句（屏障切段；前若干段全持久+当前段任意子集持久+之后各段全不持久；枚举单位是一次整写） | `.claude/kb/decisions/13-验证路线.md:71` | 无（"枚举的单位是一次整写"与"前若干段全部持久，当前段任意子集持久，之后各段全部没持久"三个分句都保留，未省略任一项） | 见提示 BACKGROUND FACTS 第 1 段引号内前半 |
| 争议句本身："带 FUA 的写也是段边界：它自成一段，它之后的写不与它同段。" | 同上 `:71` | 无 | 见提示同段引号内后半，并在候选 A / 候选 B 两段各引一次 |
| 撕裂子集不枚举句（不在本轮判断射程，仅背景） | 同上 `:71` | 无 | 提示明写 "Everything else in the paragraph above is background only, not in dispute" |
| 「没持久」位置放旧字节，不是零，句 | 同上 `:71` | 无 | 同上，作为背景保留 |
| 镜像双写按设备计一次写句 | 同上 `:71` | 无 | 同上，作为背景保留 |
| 候选甲原文："它关掉自己所在的那一段，它之后的写不与它同段"（＝代码今天的做法） | `research/prompts/_d13-item4-fua-r1-background.md:73` | 无 | 见提示 Candidate A 段 |
| 候选乙定义：维持原文「它自成一段」；FUA 单独一段，前面普通写在更早的段，按全序前缀必须整段持久 | 同上 `:74` | 无 | 见提示 Candidate B 段 |
| crash.rs 切段函数文档注释："同上，再交回写表每一项在录制流里的下标……切段只有这一份实现，两个调用方不各切一遍。" | `crates/singlefs-harness/src/crash.rs:324-325` | 无 | 见提示 "How the code actually splits today" 段首引号 |
| crash.rs 切段算法本身（Barrier 分支关闭当前段；Write/WriteForceUnitAccess 分支先入段、FUA 再立刻关段），复述代码逻辑非逐字引用 | `crates/singlefs-harness/src/crash.rs:337-366`（现读代码，非注释） | 无 | 见提示同段中部 "The algorithm: it walks..." |
| segments.rs 切段规则文档注释（四句：屏障关掉前段；空段并入将开始的段；FUA 关掉自己所在段；流尾并入上一段） | `crates/singlefs-harness/src/segments.rs:71-74` | 无 | 见提示同段后半 "A second function...carries this exact doc comment" 引号内 |
| segments.rs 单测注释「FUA 不替它前面的普通写做持久：普通写与 FUA 同段」+ 断言值 "2" | `crates/singlefs-harness/src/segments.rs:266,271-273` | 无 | 见提示段落末句 "A unit test right below that comment..." |
| 「条款字面与代码今天的做法不一致」整句 | `research/prompts/_d13-item4-fua-r1-background.md:37` | 无 | 见提示 "The mismatch" 段，及 "THE ATTACK QUESTION" 段开头再引一次 |
| 发布路径五步（Barrier / WriteJournalRecordToEveryDevice / Barrier / WriteRootRecordForceUnitAccess / RotateSuperblockSlots） | `crates/singlefs-core/src/transaction.rs:548,549,553,554,558` | 无 | 见提示 "The real publish path" 段 |
| 变异表第 67 行标签："步 3：零单元发布在记录与根之间少一道屏障" | `crates/mutations.tsv:67` | 无 | 见提示 "A mutation in the project's mutation table" 段 |
| 判别力文档注释整段："判别力靠的就是这一条：段内真子集全枚举，所以……两类状态都摆得出来——少一道屏障……就把两段并成一段，后一类立刻摆得出，记录核对器判红。种子写死不随机：这一条判的是枚举域罩不罩得住，不是抽样运气。" | `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:275-277` | 首稿曾把"记录写持久了而根槽没持久"与"根槽持久了而它那次发布的记录一条都不在"两个分句合并简化，核对后两句都逐字保留、分别对应提示里的 "the record write is durable but the root slot is not" 与 "the root slot is durable but not a single record from that publish is present" | 见提示 "Why this mutation matters" 段 |
| root_without_record 字段语义："某次发布的根槽写已在盘上，而那次发布的 journal 记录一份都不在" | `crates/singlefs-harness/src/crash.rs:461-463` | 无 | 见提示 "The record checker this refers to flags..." 句，措辞对齐为 "root record is present and readable but not a single journal record belonging to that publish is present" |
| crash_points_withholding_write_kind 断言："一次都没把 journal 记录写扣下（少一道屏障那条变异就判不出来）" | `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs:333-342`（断言代码）、`:341`（消息字符串） | 无 | 见提示 "That test also asserts a second field..." 句 |
| 两种切法施加变异表第 67 行之后的实测结果表（甲两条判红；乙两条判绿；机理："有屏障时[记录写][根槽]，没屏障时 FUA 先关掉当前段、再推一段，还是[记录写][根槽]，段序列一模一样"） | `research/prompts/_d13-item4-fua-r1-background.md:41-46` | 无（"逐条日志在那个目录下"这类溯源细节不影响现象本身，未译入，因为提示不要求指路径） | 见提示 "Measured result of actually implementing the two candidates..." 段 |
| 块层契约 PREFLUSH 段落原文（英文，本来就是英文，不是翻译） | `research/prompts/_d13-item4-fua-r1-background.md:52`（原始出处 Documentation/block/writeback_cache_control.rst，Linux 6.17） | 无，逐字核对与背景材料的引用字节相同 | 见提示 "The block layer contract" 段第一处引号 |
| 块层契约 FUA 段落原文（英文原文） | 同上 `:54` | 无 | 见提示同段第二处引号 |
| blk-mq 段落原文（BLK_FEAT_FUA 未设时块层补发 REQ_OP_FLUSH，英文原文） | 同上 `:58` | 无 | 见提示 "Two measured devices" 段引号 |
| 宿主盘读数 fua = 1（原生直通） | `research/prompts/d13-item4-fua-r1-device-readings.md:13-14` | 无 | 见提示同段 "fua = 1" 句 |
| QEMU virtio-blk 读数 fua = 0（块层拿写完补发 FLUSH 模拟） | 同上 `:44,56-57` | 无 | 见提示同段 "fua = 0" 句 |
| 「攻击问题」结尾连接句：把条款字面不一致 + 实测结果 + 块层契约 + 宿主盘读数四样拼成"这个真实缺陷在候选乙下看不见"的论证 | 不是逐句转述，是本轮自己写的综合论证（组合上述已核实的四项事实） | 不适用 | 见提示 "THE ATTACK QUESTION" 段第二段，标注为自建论证，供主 agent 自行核实是否成立，不是从某一处原文整段抄来的 |
