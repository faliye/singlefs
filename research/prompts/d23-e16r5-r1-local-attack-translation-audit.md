# d23-e16r5-r1 本地攻方 W3：译文核对表

来源均整行/整句核过。行号现查（`grep -n`），非从背景材料数偏移。

| 英文项（在提示里的编号） | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| G1a："Gate G23.2 is named gap-between-arm-and-does-not-converge; what it asks is whether, as batch size grows, the gap between arm 甲 (intent, no deferred ancestors) and arm 丙 does not converge." | `.claude/kb/decisions/23-journal的角色与格式.md:77`（简称列「甲丙差不收敛」+ 判什么列「攒批变大时，甲与丙的差不收敛」） | 首稿漏译简称列本身，只译了「判什么」列 | 定稿把简称与判什么两列都译出，合成一句 |
| G1b："Its current measured status is answered-for-乙, but bounded: in the E16 multistream workload, going from batch 1 to batch 10 the ratio grows from 3.40x to 3.71x — it does not converge, it widens instead." | `.claude/kb/decisions/23-journal的角色与格式.md:77`（「现在测得了吗」列前半段） | 首稿把「不收敛反而扩大」译成单纯 widens，漏了「不收敛」这个判断词，与「3.40×→3.71×」的对比关系没扣紧 | 定稿把「不收敛」译成显式的 it does not converge，再接 it widens instead，两个动作都在 |
| G1c："This only holds while batch ≤ stream count — the 56-cell scan shows the peak lands near batch ≈ streams; for the 16-stream row, by batch 50 the ratio has already dropped to 1.99, and by batch 200 to 1.24." | `.claude/kb/decisions/23-journal的角色与格式.md:77`（「而这只在...丙尚未测」前半段） | 无 | 同上，「16 流那一行」译成 the 16-stream row，两个具体数字 1.99 / 1.24 与对应批号都留了 |
| G1d："Warning: arm 丙 itself has never been measured." | `.claude/kb/decisions/23-journal的角色与格式.md:77`（「⚠️ 丙尚未测」） | 无 | 直译，不加限定词 |
| G2："Arm 丙 (the form named in E22, record-names-ancestor-location) likewise does not write ancestor content into the record; it is meaningful only if axis one (whether every fsync publishes the root) is reopened." | `.claude/kb/decisions/23-journal的角色与格式.md:56`（「丙（E22...）同样不写祖先内容，只在轴一被重开时才有意义」） | 首稿漏了「轴一」到底指什么，读者看不出「axis one」是什么决定；补充括注说明轴一＝「每次 fsync 发不发根」 | 定稿在 axis one 后面加括注 (whether every fsync publishes the root)，这是补的限定词，原文本身在别处（已定项 1 表格行）写过，这里补是为了不引入新事实、只是把同一份材料里已有的定义带过来 |
| Q1："Batching cannot absorb the benefit of deferring ancestor writes only while batch size is less than or equal to the concurrent stream count; once batch size exceeds the stream count, the ratio monotonically collapses toward 1." | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:75-76` | 无 | 逐句对应：「攒批吃不掉延后祖先的收益」→ batching cannot absorb the benefit of deferring ancestor writes；「只在批大小≤并发流数时成立」→ only while batch size ≤ concurrent stream count；「批一旦超过流数」→ once batch size exceeds the stream count；「比值单调塌向 1」→ the ratio monotonically collapses toward 1 |
| Q2："The two halves of that sentence, checked separately: batching-cannot-absorb-while-batch≤streams fails in 12 of the 21 rows of the real-baseline (G2, one_tebibyte_file) scan — adjacent-batch drops exceeding the 1 percent tolerance, by 1.3 to 4.7 percent, mostly near batch=2. Ratio-monotonically-collapses-beyond-batch=streams: both its sub-claims (no further rise, and settling within one quarter of the peak excess by batch=16×streams) hold in all 21 rows, but the phrase collapses toward 1 is imprecise — by batch=16384, 12 rows (streams 2 to 32) already dip below 1, down to 0.9927-0.9968, not merely close to 1." | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:104` | 首稿把「跌破 1」直接译成 falls below 1 没给出具体数值范围，判委托方 15 号规则「缺一个限定词就补上」——补了 0.9927-0.9968 这个区间，与「不只是接近 1」这句限定词一起留住 | 定稿把跌破的具体范围（0.9927-0.9968）与两个子句都留住，不省成一句话结论 |
| Q3："The stream count at which the interior peak disappears (the row becomes 丙行, i.e. NoInteriorPeak) shifts monotonically later as root-child count increases: at root_children=2, all 21 rows (streams 2 to 1024) are already 丙行; at root_children=11 (today's real baseline), 丙行 starts at streams=24; at root_children=17, it starts at streams=32; at root_children=147 (called G3 in the pre-registration), the near-diagonal arm (甲行) extends out to streams≈128, a wide plateau extends to streams=128, the moved-diagonal arm (乙行) extends to streams=256, and only at streams=384 does the row flip to 丙行 — under the same tree height, raising root_children from 11 to 147 delays the 丙行 starting point from streams=24 to streams=384, a 16-fold delay." | `.claude/kb/experiments/16-journal的角色WALvs意图日志.md:110` | 无 | 「同一个树高下」译成 under the same tree height，未删；「16 倍」译成 a 16-fold delay，未删 |

## 多出来的限定词与括注（比原文多，写明为什么加）

| 英文里多出的 | 加在哪一项 | 为什么加 |
|---|---|---|
| "(whether every fsync publishes the root)" 这个括注 | G2 | 「轴一」是这份材料没单独定义过的简称，不加括注模型读不出它指什么；括注内容取自同一份决策文件已定项 1 的表格行（「每次 fsync 发根」），不是新引入的事实 |
| "(called G3 in the pre-registration)" | Q3 | 原句用「G3」这个局部标号但这份提示没有给跑前登记的上下文；加这句括注只说明 G3 是登记里的本地标号，不引入登记文件之外的信息（`.claude/kb/experiments/16-journal的角色WALvs意图日志.md:96-98` 本身也写了这句「G0–G5 是跑前登记...本地标号」，只是那一行没被抄进这份提示） |
| "(i.e. NoInteriorPeak)" 等五个分类名后面的英文代号 | Q3、事实表说明段 | 装置源码（`research/e7-index-bench/src/bin/e16_fifth_run_peak_under_today_widths.rs` 的 `RowClassification` 枚举）把「甲行/乙行/丙行/宽平台/端点」映射到英文标识符 NearDiagonal/MovedDiagonal/NoInteriorPeak/WidePlateau/Endpoint；这五对不是转述、是产物里 `name=row_summary` 那一列 `classification=` 字段的原始取值translation 对照，事实表列直接用这五个中文词加英文代号双写，读者不必回读源码 |

## 未抄的部分

G23.2 一行里的「已答（对乙），但有界」这个整体判词，作为提示背景段单独给出（未拆句翻译，是标题式短语，逐字对应「answered-for-乙, but bounded」，未产生限定词缺失）。
