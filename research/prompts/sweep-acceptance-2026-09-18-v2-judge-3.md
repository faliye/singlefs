# sweep-acceptance-v2-judge-3 逐行判定

阶段：sweep-acceptance-v2-judge-3。候选表：`research/prompts/sweep-acceptance-2026-09-18-v2-candidates.tsv`（事实表在同目录 `-v2-facts.tsv`）。分到的组：B9、B10、B11、B12、B13、B14、B16，共 210 行，逐行判过，不整组放行。

结束提交 `00c9d4f`，读上下文一律 `git show 00c9d4f:路径`（不读工作区，工作区在这之后又改过）。

判据：把这一行里说到的那件事换成候选表给出的「新事实」，原句还是不是真话。真话 ⇒ 事件句不改（说的是那一次发生的事）或不相干（提到了检索词，说的却不是这件事实）；不真 ⇒ 要改（给改后的句子）；该登记而没登记 ⇒ 要补；判不清楚两处决策之间的关系 ⇒ 要人看。

## 逐行判定
| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| B9 | .claude/kb/checks-owed.md:292 | 不相干 | 说的是 C314 的回退例外与 I-7.4 逐字，不是「查谁的账」读法本身 |
| B9 | .claude/kb/checks-owed.md:296 | 不相干 | 说的是 C318 记账没进准入不等式，不涉及读法定没定 |
| B9 | .claude/kb/checks-owed.md:328 | 不相干 | 说的是 C356 第九项按统计量之差算、钳位问题，不是读法 |
| B9 | .claude/kb/checks-owed.md:351 | 事件句不改 | 已写「影子账窄读法只隔离『只被被抛弃根引用』的槽」，与新事实一致，不受影响 |
| B9 | .claude/kb/decisions/03-空间分配.md:45 | 不相干 | 只是准入不等式权威形式的登记位，不提读法 |
| B9 | .claude/kb/decisions/03-空间分配.md:146 | 不相干 | 准入不等式公式本身，不提读法 |
| B9 | .claude/kb/decisions/05-快照-空间记账机制.md:351 | 不相干 | 被抛弃根独占量的取值公式，不提读法争议 |
| B9 | .claude/kb/decisions/05-快照-空间记账机制.md:354 | 不相干 | 四处例外不进持久记账的说明，不提读法 |
| B9 | .claude/kb/decisions/23-journal的角色与格式.md:1209 | 事件句不改 | 已写「查账的集合 2026-09-16 用户定案取窄读法：按主语『被抛弃时间线的根』读」，与新事实一致 |
| B9 | .claude/kb/decisions/28-挂载期承诺量.md:11 | 不相干 | 准入不等式权威形式登记位，不提读法 |
| B9 | .claude/kb/decisions/28-挂载期承诺量.md:21 | 不相干 | 准入不等式公式本身 |
| B9 | .claude/kb/decisions/28-挂载期承诺量.md:24 | 不相干 | 式子逐项对照表的指引，不提读法 |
| B9 | .claude/kb/decisions/28-挂载期承诺量.md:30 | 事件句不改 | 已写窄读法下的取值公式「只被被抛弃根引用的槽数 = … − 回退候选集里的根引用的槽」，与新事实一致 |
| B9 | .claude/kb/decisions/28-挂载期承诺量.md:46 | 不相干 | 被抛弃根独占量的定案摘要，不提读法争议 |
| B9 | .claude/kb/decisions/28-挂载期承诺量.md:49 | 不相干 | 按设备求和的分界说明 |
| B9 | .claude/kb/decisions/28-挂载期承诺量.md:100 | 不相干 | ckpt_cost 归属说明，不提读法 |
| B9 | .claude/kb/experiments.md:171 | 不相干 | E153 跑批摘要，讨论的是判红次数不是读法定没定 |
| B9 | .claude/kb/experiments/104-扫描重建的现行版本判定.md:94 | 不相干 | 讨论 kind 0 行回收模型改动，不是查谁的账读法 |
| B9 | .claude/kb/experiments/150-回退复用被抛弃的根引用的单元.md:20 | 不相干 | 描述第二次跑的独立算法，不主张读法未定 |
| B9 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:22 | 不相干 | 讨论 I-3.1 判红与 Q4ii 违例，不是读法定没定 |
| B9 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:25 | 不相干 | 讨论 G12 谓词的根因，不涉及读法状态 |
| B9 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:34 | 不相干 | 讨论 Q6b 模型字段，不涉及读法状态 |
| B9 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:37 | 不相干 | 讨论隔离槽数超过上界的发现，不涉及读法状态 |
| B9 | .claude/kb/experiments/153-账本形态与环上有洞的代价.md:110 | 不相干 | 讨论 G12 判红根因两层，不涉及读法状态 |
| B9 | .claude/kb/experiments/154-两道闸串-重判与回收时点的代价.md:137 | 事件句不改 | 已写「影子账按窄读法隔离…的槽」，与新事实一致 |
| B9 | .claude/kb/layout/02-second-txn.md:15 | 不相干 | 只住内存两样的登记说明，不涉及读法状态 |
| B9 | .claude/kb/milestone/02-second-txn.md:40 | 不相干 | 说的是已定项14前缀第五条与回退行落点，不是查谁的账读法 |
| B9 | .claude/kb/milestone/02-second-txn.md:144 | 不相干 | 描述测试脚本产生的场景，不涉及读法状态 |
| B9 | .claude/kb/milestone/02-second-txn.md:166 | 事件句不改 | 已写「对用户 2026-09-16 定的窄读法措辞…的收严」，与新事实一致 |
| B9 | .claude/kb/milestone/02-second-txn.md:176 | 事件句不改 | 已完整叙述两种读法之争并写明 2026-09-16 用户定案取窄读法，与新事实一致 |
| B9 | .claude/kb/milestone/02-second-txn.md:178 | 不相干 | 被抛弃根独占量清零时机的说明，不涉及读法争议本身 |
| B9 | .claude/kb/milestone/02-second-txn.md:195 | 不相干 | 可再分配谓词与抬 F 的实现现状，不涉及查谁的账读法 |
| B9 | .claude/kb/milestone/02-second-txn.md:222 | 不相干 | 层 0 崩溃点重放现状汇总，不涉及读法 |
| B9 | .claude/kb/milestone/02-second-txn.md:311 | 不相干 | F 生效事后回落的打回重议项，不是查谁的账读法 |
| B9 | .claude/kb/milestone/02-second-txn.md:312 | 不相干 | 被抛弃根账读不出时的修复路径打回重议，不涉及读法 |
| B9 | .claude/kb/milestone/02-second-txn.md:338 | 不相干 | checker 欠账清单，不涉及读法 |
| B9 | .claude/kb/milestone/02-second-txn.md:339 | 不相干 | 今天不可达的欠账清单，不涉及读法 |
| B9 | .claude/kb/verification-build.md:74 | 不相干 | 准入对分配器发得出的槽数的验证项，不涉及读法定没定 |
| B9 | records/2026-09-13-总审核.md:168 | 不相干 | D28 已定项相关问题清单，不涉及查谁的账读法 |
| B9 | records/2026-09-13-总审核.md:197 | 不相干 | 提议 6 改口径的讨论，不涉及读法 |
| B9 | records/2026-09-16-subagent拆分提案.md:518 | 不相干 | 讨论 E153 报告执行问题，只是提到被抛弃根，与读法无关 |
| B9 | records/2026-09-17-已分配口径三方与两个实验.md:45 | 不相干 | 甲-T1×G5″臂发现记录，不是读法定没定的陈述 |
| B9 | records/2026-09-17-已分配口径三方与两个实验.md:46 | 不相干 | G12 臂发现记录，不是读法定没定的陈述 |
| B9 | records/2026-09-17-已分配口径三方与两个实验.md:97 | 要人看 | 该行「三轮站住的」写『保守读法』、「状态」写『用户已定（G5）』，与决策正文『取窄读法』是不是同一件事说法不一致，需要人核对『保守读法／G5』与『窄读法』的关系 |
| B10 | .claude/kb/checks-owed.md:75 | 不相干 | 讨论 C65 分配器提示矛盾，locality_id 只作举例，不涉及 D18 提议决没决 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:71 | 不相干 | D8 已定项3 key 布局（2026-08-25），与 D18 已定项3 加字段提议是两条不同决策 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:133 | 不相干 | key 布局细节，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:137 | 不相干 | 局部性退化说明，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:158 | 不相干 | 实验数字引用，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:174 | 不相干 | 收益说明，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:180 | 不相干 | 可写头覆盖问题，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:197 | 不相干 | locality_id 是提示的性质说明，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:203 | 不相干 | 三方论证挖出的要求，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:206 | 不相干 | inode 记录要存 locality_id 的要求，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:399 | 不相干 | 字段表条目，非 D18 提议状态 |
| B10 | .claude/kb/decisions/08-核心索引结构.md:426 | 事件句不改 | 已写「2026-09-16 用户定案…三轮三方后不收」并给出重建规则，与新事实一致 |
| B10 | .claude/kb/decisions/09-加密.md:553 | 事件句不改 | 已写「2026-09-16…用户定案不收，这一行维持」，与新事实一致 |
| B10 | .claude/kb/decisions/09-加密.md:672 | 不相干 | 只是罗列现查过的四条依据，不涉及提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:96 | 不相干 | track 与 locality_id 抢排序权，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:195 | 不相干 | D14 决策理由，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:214 | 不相干 | D14 第三轮判据讨论，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:281 | 不相干 | D14 判据取 locality_id 讨论，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:283 | 不相干 | 冻结分层说明，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:284 | 不相干 | D14 判据讨论，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:286 | 不相干 | 第一版恒 0 的引用，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:287 | 不相干 | 字段表引用，非 D18 提议状态 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:295 | 不相干 | 陈述 D18 自描述头五元组不含它，这是决策不收之后仍然成立的事实，不是提议状态本身 |
| B10 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:307 | 不相干 | 住超级块/指针形态枚举，非 D18 提议状态 |
| B10 | .claude/kb/decisions/15-格式冻结政策.md:62 | 不相干 | D8 key 编码冻结分层，非 D18 提议 |
| B10 | .claude/kb/decisions/18-块里携带什么信息.md:398 | 事件句不改 | 这一行就是「2026-09-16 用户定案（三轮三方）：不加第六个字段 locality_id，提议不收」，正是新事实本身 |
| B10 | .claude/kb/decisions/18-块里携带什么信息.md:401 | 事件句不改 | 已写重建规则「取值 inode 记录 > 幸存 extent key > 0」，与新事实一致 |
| B10 | .claude/kb/decisions/18-块里携带什么信息.md:820 | 要改 | 六元组的两个候选不在这次定案里：`locality_id`（D18（块里携带什么信息） 已定项 3 那条已定案不收的提议）与出生快照维（D9（加密） 已定项 6 索引行欠的那次重开），都是 AAD 组成的永久变更，各归其分项。 |
| B10 | .claude/kb/decisions/26-后台整理与放置回收.md:218 | 不相干 | 抗老化形态讨论里提到 locality_id 是替代还是叠加，非 D18 提议状态 |
| B10 | .claude/kb/decisions/26-后台整理与放置回收.md:233 | 不相干 | 抗老化形态判定，非 D18 提议状态 |
| B10 | .claude/kb/experiments/09-key编码对遍历局部性的影响.md:3 | 不相干 | D8 已定项3 key 布局背景，非 D18 提议 |
| B10 | .claude/kb/experiments/09-key编码对遍历局部性的影响.md:4 | 不相干 | 实验背景说明，非 D18 提议 |
| B10 | .claude/kb/experiments/09-key编码对遍历局部性的影响.md:32 | 不相干 | 实验发现，非 D18 提议 |
| B10 | .claude/kb/experiments/09-key编码对遍历局部性的影响.md:43 | 不相干 | 实验结论呼应 D8 已写的句子，非 D18 提议 |
| B10 | .claude/kb/experiments/35-多可写头两种形态.md:9 | 不相干 | D8 已定项3 key 布局引用，非 D18 提议 |
| B10 | .claude/kb/experiments/98-inode记录与inode树的几何.md:14 | 不相干 | D8 已定项3 引用，非 D18 提议 |
| B10 | .claude/kb/experiments/98-inode记录与inode树的几何.md:28 | 不相干 | locality_id 对照判别力设计，非 D18 提议 |
| B10 | .claude/kb/experiments/98-inode记录与inode树的几何.md:65 | 不相干 | 不能重算字段清单，非 D18 提议 |
| B10 | .claude/kb/experiments/98-inode记录与inode树的几何.md:112 | 不相干 | 对照实验数字，非 D18 提议 |
| B10 | .claude/kb/invariants.md:270 | 事件句不改 | I-9.9 已按 2026-09-16 的按侧判读法描述，与新事实一致 |
| B10 | .claude/kb/layout/01-first-txn.md:253 | 不相干 | inode 记录字段表，非 D18 提议状态 |
| B10 | .claude/kb/layout/01-first-txn.md:272 | 不相干 | extent 叶记录 key 布局，非 D18 提议状态 |
| B10 | .claude/kb/milestone/01-first-txn.md:114 | 不相干 | 第一个文件预想参数，非 D18 提议状态 |
| B10 | .claude/kb/milestone/01-first-txn.md:115 | 不相干 | extent 树记录宽度，非 D18 提议状态 |
| B10 | .claude/kb/milestone/02-second-txn.md:439 | 不相干 | extent 树设想实现，非 D18 提议状态 |
| B10 | records/2026-08-29-组合对攻轮.md:226 | 事件句不改 | 2026-08-26 的 D8 已定项3 历史记录 |
| B10 | records/2026-08-29-组合对攻轮.md:231 | 事件句不改 | 2026-08-29 的历史论证记录，属历史陈述 |
| B10 | records/2026-09-05-C113定案.md:51 | 事件句不改 | 2026-09-05 记录当时候选排队顺序，历史陈述 |
| B10 | records/2026-09-05-inode树定案.md:39 | 事件句不改 | 2026-09-05 历史陈述，排队顺序 |
| B10 | records/2026-09-09-D14项2第二轮.md:18 | 事件句不改 | D14 第二轮论证历史记录 |
| B10 | records/2026-09-09-D14项2第二轮.md:63 | 事件句不改 | 历史论证依据小标题 |
| B10 | records/2026-09-09-D14项2第二轮.md:65 | 事件句不改 | 历史论证引用的三处坐实 |
| B10 | records/2026-09-09-D14项2第二轮.md:66 | 事件句不改 | 2026-09-09 的历史论证引用，属历史陈述 |
| B10 | records/2026-09-09-D14项2第二轮.md:78 | 事件句不改 | 历史论证引用，且陈述内容今天仍然成立 |
| B10 | records/2026-09-09-D14项2第二轮.md:87 | 事件句不改 | 历史论证反向条款要求 |
| B10 | records/2026-09-09-D14项2第二轮.md:88 | 事件句不改 | 历史 grep 结果记录（31 行，明写的没有） |
| B10 | records/2026-09-09-D14项2第二轮.md:117 | 事件句不改 | 2026-09-09 的历史论证记录，属历史陈述 |
| B10 | records/2026-09-09-D14项2第二轮.md:153 | 事件句不改 | 历史论证表格记录 |
| B11 | .claude/kb/checks-owed.md:90 | 不相干 | C80 记账更新问题，C161 只作旁引不涉及其状态 |
| B11 | .claude/kb/checks-owed.md:163 | 事件句不改 | 这一行就是 C161 的定案摘要「一棵树只准一级…」，正是新事实本身 |
| B11 | .claude/kb/checks-owed.md:202 | 不相干 | C211 节点落盘形态问题，只说与 C161 同族，不涉及 C161 状态 |
| B11 | .claude/kb/checks-owed.md:315 | 不相干 | C344 是新立欠账，引用 C161 只是出处，不涉及 C161 状态 |
| B11 | .claude/kb/checks-owed.md:316 | 不相干 | C345 出处引用 C161 轮次，不涉及状态 |
| B11 | .claude/kb/checks-owed.md:319 | 不相干 | C348 出处引用 C161 轮次，不涉及状态 |
| B11 | .claude/kb/checks-owed.md:320 | 不相干 | C349 出处引用 C161 轮次，不涉及状态 |
| B11 | .claude/kb/decisions/08-核心索引结构.md:452 | 要改 | ⚠️ 本题之外挖出的更大空白已立账 C161（缓冲两级并存而衔接没定），2026-09-16 定案：一棵树只准一级——走前端的三棵树 ε 恒 0、写一律进前端、发布前排空到叶；仍欠的是检查，新立 C344 到 C349。 |
| B11 | .claude/kb/decisions/08-核心索引结构.md:534 | 不相干 | 描述 C161 检查的设计规格，不断言 C161 是否已定案 |
| B11 | .claude/kb/decisions/11-索引节点要不要留消息缓冲区.md:233 | 事件句不改 | 已写「2026-09-16 C161 第三轮写明」，与新事实一致 |
| B11 | .claude/rules/three-way-inference.md:257 | 不相干 | 以 C161 第三轮为例讲周限额应对，不涉及 C161 状态 |
| B11 | .claude/rules/three-way-inference.md:259 | 不相干 | 以 C161 第二轮为例讲草稿目录隔离，不涉及 C161 状态 |
| B11 | records/2026-09-13-总审核.md:269 | 事件句不改 | 历史记录：第一版直落叶时 C161 不触发，历史陈述 |
| B12 | .claude/kb/checks-owed.md:146 | 事件句不改 | 这一行就是 C143 的定案摘要，正是新事实本身 |
| B12 | .claude/kb/checks-owed.md:309 | 不相干 | C338 预留余量问题，C143 甲臂只是历史引用 |
| B12 | .claude/kb/checks-owed.md:310 | 不相干 | C339 故障计数口径问题，引用 C143 三轮的模型结果作证据，不涉及 C143 状态 |
| B12 | .claude/kb/checks-owed.md:311 | 不相干 | C340 回退记录链接续问题，不涉及 C143 状态 |
| B12 | .claude/kb/checks-owed.md:312 | 不相干 | C341 点删跳号问题，出处引用 C143 轮次，不涉及状态 |
| B12 | .claude/kb/checks-owed.md:313 | 不相干 | C342 树 ID 水位问题，与 C143 同形但另立，不涉及 C143 状态 |
| B12 | .claude/kb/checks-owed.md:314 | 不相干 | C343 级1重建问题，出处引用 C143 轮次，不涉及状态 |
| B12 | .claude/kb/decisions/05-快照-空间记账机制.md:377 | 事件句不改 | 已写「2026-09-16 三轮三方+用户定案：不搬水位，改收窄唯一性…」，与新事实一致 |
| B12 | .claude/kb/decisions/08-核心索引结构.md:74 | 事件句不改 | 已写「2026-09-16 随 C143…定案：唯一性…收窄到『已发布的(inode号,对象出生代)不复用』」，与新事实一致 |
| B12 | .claude/kb/decisions/08-核心索引结构.md:380 | 事件句不改 | 已写收窄后的唯一性定义与回退出生代规则，与新事实一致 |
| B12 | .claude/kb/decisions/08-核心索引结构.md:381 | 事件句不改 | 已写 2026-09-16 再收窄之后仍被禁的范围，与新事实一致 |
| B12 | .claude/kb/decisions/08-核心索引结构.md:479 | 不相干 | 树 ID 水位定案，落点引用 C143，不涉及 C143 状态本身 |
| B12 | .claude/kb/decisions/23-journal的角色与格式.md:691 | 事件句不改 | 已写「2026-09-16 随 C143 定案取 CJ2」的 checkpoint_txg 取值，与新事实一致 |
| B12 | .claude/kb/decisions/23-journal的角色与格式.md:1209 | 事件句不改 | 已写「2026-09-16 随 C143…」的 checkpoint_txg 取值，与新事实一致 |
| B12 | .claude/kb/decisions/23-journal的角色与格式.md:1240 | 事件句不改 | 已写「2026-09-16 C143 定案 CJ2」及 D5 已定项2 点删规则改法，与新事实一致 |
| B12 | .claude/kb/milestone/02-second-txn.md:169 | 事件句不改 | 已写「2026-09-16…随 C143 定案改成 max(…)+1」，与新事实一致 |
| B12 | .claude/kb/milestone/02-second-txn.md:191 | 不相干 | 决策点清单里列出 C143 作为会碰到的项，不涉及状态 |
| B12 | .claude/kb/milestone/02-second-txn.md:319 | 不相干 | 说的是 C143「另一半」——崩溃丢弃孤儿单元 inode 号能不能重发——是仍未决的独立子问题，不是新事实已定案的那一半，不受影响 |
| B12 | .claude/rules/three-way-inference.md:203 | 不相干 | 以 C143 第三轮为例讲模型放开动作重扫的方法论，不涉及状态 |
| B12 | records/2026-09-06-D8项8第三轮对抗.md:87 | 事件句不改 | 历史记录（2026-09-06，早于定案） |
| B12 | records/2026-09-06-树ID水位臂比较重做.md:68 | 事件句不改 | 历史记录（2026-09-06） |
| B12 | records/2026-09-06-树ID水位臂比较重做.md:105 | 事件句不改 | 历史记录（2026-09-06），当时的回扫结论 |
| B12 | records/2026-09-16-subagent拆分提案.md:25 | 不相干 | 以 C143 第三轮材料体量为例，不涉及状态 |
| B12 | records/2026-09-16-subagent拆分提案.md:225 | 不相干 | 以 C143 第三轮为例讲族一输入产出，不涉及状态 |
| B12 | records/2026-09-16-subagent拆分提案.md:508 | 不相干 | 以 C143 第一轮报告为例讲损坏闸回扫，不涉及状态 |
| B13 | .claude/kb/checks-owed.md:30 | 不相干 | C18 解耦字段污染只读维度，deadlist 只作举例，不涉及条目形态定没定 |
| B13 | .claude/kb/checks-owed.md:94 | 不相干 | C84 墓碑粒度问题，deadlist 只作回收节奏引用，不涉及条目形态 |
| B13 | .claude/kb/checks-owed.md:148 | 事件句不改 | 已写「2026-09-16 D5 已定项12 定了条目形态…」，与新事实一致 |
| B13 | .claude/kb/decisions.md:150 | 事件句不改 | 索引条目「12. deadlist 条目的形态 —— 已定」，与新事实一致 |
| B13 | .claude/kb/decisions/05-快照-空间记账机制.md:1 | 事件句不改 | 已写「deadlist 条目形态…2026-09-16 三轮三方+用户定案」，与新事实一致 |
| B13 | .claude/kb/decisions/05-快照-空间记账机制.md:36 | 不相干 | 按落点记的搬迁代价讨论，不涉及条目形态定没定 |
| B13 | .claude/kb/decisions/05-快照-空间记账机制.md:255 | 事件句不改 | 这一行就是已定项12 的定案全文，正是新事实本身 |
| B13 | .claude/kb/decisions/05-快照-空间记账机制.md:475 | 不相干 | 讨论的是稀疏旁表（原已定项6）的落点与字段表，不是 deadlist 条目形态 |
| B13 | .claude/kb/decisions/05-快照-空间记账机制.md:519 | 事件句不改 | 已定项12 标题行「2026-09-16，用户定案+三轮三方」，与新事实一致 |
| B13 | .claude/kb/decisions/06-快照实现模型.md:214 | 不相干 | 讨论每头持有 deadlist 与 previous_snapshot_txg 的结构性要求，不是条目形态 |
| B13 | .claude/kb/decisions/08-核心索引结构.md:467 | 要人看 | 逐字引用 D5 旧措辞「销毁快照时把它的deadlist合并到下一个更新的那一侧」作为 2026-09-06 论证证据，与已定项12现行的『按批放掉区间条目、不改键』写法不同，是否需要跟着改需要人判断这段引用是历史证据还是仍在断言现状 |
| B13 | .claude/kb/decisions/08-核心索引结构.md:477 | 不相干 | 只是提到 I-3.6（deadlist 紧界）的名字，不涉及条目形态定没定 |
| B13 | .claude/kb/decisions/08-核心索引结构.md:505 | 不相干 | 码8（deadlist）day-1 注册的说明，指向『条目形态另立 D5 已定项12』，属正确的指针引用，不受影响 |
| B13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:122 | 不相干 | 讨论指针字段消费者，deadlist 只作举例 |
| B13 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:144 | 不相干 | 讨论码2/码3 是否进 deadlist/livelist 的正确性问题，不涉及条目形态定没定 |
| B13 | .claude/kb/decisions/26-后台整理与放置回收.md:208 | 事件句不改 | 这一段显式标注『定案之前那一格…逐字保留』，是有意保留的冻结前文本，不代表现状 |
| B13 | .claude/kb/decisions/26-后台整理与放置回收.md:234 | 要人看 | 该行『deadlist条目内容随位置权威一起定』是否已被 D5 已定项12（key 不因搬迁改写）解决，需要人对照两处决策判断是否已经过时 |
| B13 | .claude/kb/experiments/94-搬共享落点的触碰面.md:28 | 不相干 | 实验模型描述 ptr_rewrite 臂机制，不涉及条目形态决策状态 |
| B13 | .claude/kb/experiments/94-搬共享落点的触碰面.md:29 | 不相干 | 实验模型描述 central_map 臂机制，不涉及条目形态决策状态 |
| B13 | .claude/kb/invariants.md:126 | 不相干 | I-3.5 引用区间精确性定义，deadlist 只作归属区间概念引用 |
| B13 | .claude/kb/layout/01-first-txn.md:373 | 要改 | 把该行第三列里的「条目形态未定（D5（快照 / 空间记账机制） 已定项 12），第一个事务一条都不写」改成「条目形态已定（D5（快照 / 空间记账机制） 已定项 12：key = 头的树 ID 8 + 纪元 8 + 中央映射 key 27，value 空），第一个事务一条都不写」，其余列（deadlist 树 / 200 / 后两列）不变。 |
| B13 | .claude/kb/layout/01-first-txn.md:409 | 事件句不改 | 历史陈述：2026-09-14 新立时判『否』（不改第一个事务字节），与新事实不冲突（该判定本身仍然成立） |
| B13 | .claude/kb/milestone/01-first-txn.md:106 | 不相干 | 树 ID 编号列举，不涉及条目形态 |
| B13 | .claude/kb/milestone/01-first-txn.md:130 | 不相干 | day-1 进树表说明，指向『条目形态是 D5 已定项12』，属正确指针引用 |
| B13 | .claude/kb/milestone/02-second-txn.md:137 | 事件句不改 | 已写『D5 已定项12…2026-09-16 用户定案+三轮三方』，与新事实一致 |
| B13 | records/2026-09-11-D19项6对抗第二轮.md:36 | 事件句不改 | 历史记录（2026-09-11，早于定案），当时属共有欠账 |
| B13 | records/2026-09-13-总审核.md:362 | 事件句不改 | 历史记录（2026-09-13，早于定案），当时条目形态另立未定项 |
| B14 | .claude/kb/checks-owed.md:90 | 事件句不改 | 已写『fsync = 一次全量发布（2026-09-16 用户定案…）』，与新事实一致 |
| B14 | .claude/kb/decisions/08-核心索引结构.md:532 | 事件句不改 | 已写『fsync = 一次全量发布』作为发布集边界定义的一部分，与新事实一致 |
| B14 | .claude/kb/decisions/23-journal的角色与格式.md:974 | 事件句不改 | 已写『脏叶是全部脏叶…fsync = 一次全量发布，2026-09-16 用户定案，C80 那句『部分发布』随之改』，与新事实一致 |
| B16 | .claude/kb/checks-owed.md:24 | 事件句不改 | 写的是『2026-09-14 第一套布局的形态有了：…23 条…』，是带日期的历史快照，不断言今天的总数 |
| B16 | .claude/kb/checks-owed.md:403 | 不相干 | C374 已还清记录，未提总条数，只是提到『池级 checker』 |
| B16 | .claude/kb/invariants.md:12 | 事件句不改 | 这一行就是当前总数『判 29 条』及其逐次演进历史，正是新事实本身 |
| B16 | .claude/kb/invariants.md:24 | 不相干 | I-1.1 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:25 | 不相干 | I-1.2 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:26 | 不相干 | I-1.3 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:27 | 不相干 | I-1.4 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:29 | 不相干 | I-1.6 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:30 | 不相干 | I-1.7 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:50 | 不相干 | I-7.1 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:51 | 不相干 | I-7.2 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:53 | 不相干 | I-7.4 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:55 | 不相干 | I-7.6 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:56 | 不相干 | I-7.7 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:57 | 不相干 | I-7.8 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:110 | 不相干 | I-2.1 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:112 | 不相干 | I-2.3 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:114 | 不相干 | I-2.5 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:123 | 不相干 | I-3.1 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:157 | 不相干 | I-4.8 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:170 | 不相干 | I-5.1 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:171 | 不相干 | I-5.2 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:262 | 不相干 | I-9.1 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:263 | 不相干 | I-9.2 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:265 | 不相干 | I-9.4 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:268 | 不相干 | I-9.7 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:271 | 不相干 | I-9.10 单条不变量定义，未提总条数 |
| B16 | .claude/kb/invariants.md:274 | 不相干 | I-9.13 单条不变量定义，未提总条数 |
| B16 | .claude/kb/milestone/01-first-txn.md:217 | 事件句不改 | 写的是『三个判决器（2026-09-14）：…23 条不变量…』，是带日期的里程碑历史快照，不断言今天的总数 |
| B16 | .claude/kb/milestone/02-second-txn.md:117 | 不相干 | 分配记账演示，未提总条数 |
| B16 | .claude/kb/milestone/02-second-txn.md:224 | 不相干 | 设想实现描述，未提总条数 |
| B16 | .claude/kb/milestone/02-second-txn.md:334 | 不相干 | 增补1第②行讨论，未提总条数 |
| B16 | .claude/kb/milestone/02-second-txn.md:390 | 事件句不改 | 已写『池级 checker（29 条不变量）』，与新事实一致 |
| B16 | .claude/kb/milestone/02-second-txn.md:400 | 不相干 | 历史生成器方法论描述，未提总条数 |
| B16 | .claude/kb/tooling.md:607 | 不相干 | 门禁54号跑法说明，未提总条数 |
| B16 | .claude/kb/verification-build.md:4 | 不相干 | 四个 crate 现状描述，未提总条数 |
| B16 | .claude/kb/verification-build.md:139 | 事件句不改 | 已写『池级 checker…29条：第一版23条加…』完整演进，与新事实一致 |
| B16 | .claude/rules/implementation-workflow.md:12 | 不相干 | 三方论证工作流描述，未提总条数 |
| B16 | records/2026-09-13-总审核.md:443 | 不相干 | 小节标题，未提总条数 |
| B16 | records/2026-09-13-总审核.md:447 | 事件句不改 | 历史记录（2026-09-14 事件）：『判第一版23条不变量』，带『第一版』限定词，历史陈述 |
