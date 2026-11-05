# m3-close-b19-reason-r3 攻方腿（Opus）报告（2026-11-03）

立场：找反例。攻击面照正文第四节「攻方腿（Opus）」那一行：Q1、Q2 造误红与漏判的镜像；Q3 造写者与 checker 上限不同而判错的合法历史；Q4 造实现与条款字面不一致的盘面或历史、按槽认盘在真写者上走得到的历史；Q5 指出登记理由里站不住的一句。前两轮攻过的角度（丁的前提、三处简化、逐判混取拼镜像；改记读法本身的漏判与误红、C944 合法来源、丁的枚举与窗口外那一停、第 0 代根按区域认盘那一形）不重复。

开工快照：`research/prompts/m3-close-b19-reason-r3-snapshot/crates-sha256.txt` 36 份，`sha256sum -c` 在主工作区逐份 OK（开工时核的）。下文代码行号都是这一刻主工作区的，与快照一致。

这一条腿没写模型、没往服务交任务；下面每个打中都是按代码与条款逐行读出来的，标「推的（照代码读）」。被判的读数（r2 两腿模型的数）是那两份模型的产物原样，不是我量的。

## 各格判定一览

| 格 | 结论 | 一句话 | 依据的性质 |
|---|---|---|---|
| Q1-1 误红 / 漏判 | 造不出 | 取宽那一支只在这块盘上有 I-3.18 判红的记录时才进，合法镜像进不去；两条都已释放的相交在两种读法里一样，由 I-5.4 判 | 推的（照代码读） |
| Q1-2 与 I-5.4 重不重复 | **造得出（重复，且 I-3.1 的判定随一条不相干的记录翻面）** | 第五判比的是改记那一份里「至少一条未释放」的每一对，不只比被改记过的记录；一对按自述跨度就相交的记录（I-5.4 判红那一对），只要同盘别处有一条 I-3.18 判红的记录，I-3.1 也判红点名同一对；去掉那条不相干的记录，同一对上 I-3.1 成立 | 推的（照代码读），给了最小镜像 |
| Q1-3 说明里的数 | 造不出要分的情形 | 改记后相交现在判红、不进部分没判，部分没判的说明里不会再有「改记后相交」那一形 | 推的 |
| Q2-1 「0 条不适用」 | 造不出藏病 | 最新根树表 0 条只在第一个文件之前的那几版；回退目标要带文件，带文件之后最新根不会回到 0 条；那几版走到的引用由 I-5.1、I-7.4、I-2.1 罩着 | 推的 |
| Q2-2 记录集合 | 合法镜像造不出误红；1195 个 C944 形照构造全抓 | 走到的版本 txg 都不低于 θ_high，它们换下的单元释放代都高于 θ_high；实现的记录过滤与 r2 攻方模型 `c944_starts` 同一个式子 | 推的 |
| Q2-3 部分没判两形 | 造不出合法来源 | 「有条目而没有种类 3」由 I-9.16 ④ 判畸形；最新根的树表与分配记录树在发布次序上先于根落盘 | 推的 |
| Q3-1 第 0 代根两处 | 数可迁、合法环造不出变红 | 实现与 r2 辩方模型 `today_both_fixed` 那一臂三个旋钮逐个对上；只剩第 0 代根一种可能的槽，写者抬 F 之前读的也正是它 | 推的；r2 的数是辩方模型的产物，没在实现上复核 |
| Q3-2 写者与 checker 上限 | 误红造不出；坏写者越过字面上限而 checker 放过：造得出（判别子观测不到）；实现员那一句理由字面站不住 | 写者并进读坏、全零槽只会把上限压低；checker 看不到挂载那一刻的读数，那一槽盘上真是全零时，漏了「全零并」的坏写者 checker 判成立 | 推的；给了最小历史 |
| Q4-1 全零并 | 相符 | 并进的是 `AllZeroAtMount` 与 `BadAtMount` 两类，能住的最大 txg 与读坏同一个算法，回收门槛那一边没动 | 推的（照代码读） |
| Q4-2 推空 | 相符；停下的理由次序与 E173 装置不同（都拒，盘上同） | 「已推 + 1 + 1 ≤ 8」与 E173 推空B 臂定义同；预算放不下一次推空时实现先报预算用完、不算上限，装置先算上限、上限不高于 F 时报抬不动 | 推的（照代码读） |
| Q4-3 按槽认盘 | **造得出字面不一致（两处仍按 txg 认）；真写者上走不到结局不同** | 卸载当场放开那一截按 txg 认盘、抬 F 那一遍认「这一遍读出根的槽」按 txg 认槽；两处只在第 0 代根上与按槽不同，算出来的数相同。准入那一处按槽认盘，在真写者上与按 txg 认盘结局相同：推抬 F 排在暖机之后，每块盘上都有本实例的根 | 推的（照代码读） |
| Q4-4 理想模型 | **造得出胶水把推空当抬 F 交给模型（一旦走到就报假的不一致）**；今天的随机历史走不到推空 | `history.rs` 三处把 `RaisedFloor` 一律交 `answer_raise_rollback_floor`，不看 `floor_of_the_publishes`；推空那一份报的上限不高于现行 F，模型算的上限高于 F，`judge_reported_ceiling` 逐字比，必不等 | 推的（照代码读） |
| Q5 计数分配器 | E197：「离不开分配器」对判的那个数站不住；E183：站得住；两份都还有登记表结构上装不下的地方 | E197 判的是服务记的 cgroup 峰值，分配器只做互核；E183 的 copy_bytes 是一个进程里一次调用的分配字节差，cgroup 与 `/proc` 给不出 | 推的（照代码与登记读） |

## Q1　I-3.1 取宽第五判

### Q1-1　误红与「两条都已释放而相交」的漏判：造不出

- 取宽那一支要先过一判：改记那一份要存在，即这块盘上至少一条未释放记录被 I-3.18 判红、改记成那一类的槽数。`crates/singlefs-checker/src/walk.rs:21731`：「bounds_with_the_spans_set_to_the_unit_classes.as_ref().is_some_and(」——没有改记那一份时这一判是假，直接走判红，不进第五判。合法写者的镜像上 I-3.18 不红（崩溃点重放 2026-10-22 起逐状态判它），所以合法镜像进不了第五判，第五判在合法镜像上误红造不出。
- 「两条都已释放而相交、藏着真病」：改记只改未释放记录（I-3.18 只判未释放的，`crates/singlefs-checker/invariants/05-空间记账.md:25`：「已释放的记录不判（落点可以已被别的类的单元合法复用）」），两条已释放记录在改记那一份与自述那一份里跨度一样，它们相交在自述读法里就相交，归 I-5.4：`crates/singlefs-checker/invariants/07-分配器.md:9`：「同一块盘上任意两条分配记录（不论已分配、已释放、释放代是否已低于回退下界）罩住的槽区间 [槽号, 槽号 + 跨度) 互不相交」。第五判不比这一对（`crates/singlefs-checker/src/walk.rs:10595`：「(true, true) => continue,」）不漏病：I-5.4 在同一张镜像上判红。
- 什么会推翻：一张合法镜像上 I-3.18 判红（那样取宽那一支就进得去），或 I-5.4 不判最新根的分配记录树。

### Q1-2　第五判与 I-5.4：造得出重复，且 I-3.1 的判定随一条不相干的记录翻面

第五判的实现比的是改记那一份里「至少一条未释放」的每一对，不论这一对里有没有被改记过的记录：`crates/singlefs-checker/src/walk.rs:10593`：「(false, false | true) => (*earlier, *later),」。不变量条文照这样写：`crates/singlefs-checker/invariants/05-空间记账.md:8`：「改记之后每条未释放记录罩住的槽区间不与这块盘上别的记录（未释放或已释放）罩住的槽区间相交」。一对按自述跨度就相交的记录，两种读法里都相交，本来归 I-5.4；只要同一块盘上别处有一条 I-3.18 判红的记录让取宽那一支进得去，第五判也点名这一对、把 I-3.1 判红。

最小镜像（一块盘、最新根的分配记录，θ_high = t = 2；槽宽记作 w）：

| 记录 | 起点槽 | 自述跨度 | 状态 | 起点上的单元 | 改记之后 |
|---|---|---|---|---|---|
| A | 0 | 1 | 未释放 | 码 1 数据单元（两槽），头用得了 | [0, 2)（I-3.18 判红） |
| B | 5 | 2 | 未释放 | 码 1 数据单元（两槽） | [5, 7)（不变） |
| C | 6 | 1 | 已释放，释放代 1（≤ t） | 不看 | [6, 7)（不变） |

遍历走到 A 那个单元（槽 0、1）与 B 那个单元（槽 5、6）；没有隔离记号；记账的已分配 = 4w。

逐判（照 `walk.rs` 的取宽那一支，推的）：
1. 自述那一份：E(θ_high) = {0, 5, 6}（C 释放代 ≤ θ_high，不算），遍历走到的槽 1 不在里面，(P) 不成立，进取宽那一支。
2. 改记那一份：E(θ_high) = E(t) = {0, 1, 5, 6}；(L) 4w ≥ 4w；(P) 走到的四槽都在；(U) Ω ∪ U = 4 槽（U 空：没有释放代高于 t 的已释放记录），4w ≤ min(4w, 4w)；逐槽两判都成立。`allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading` 成立。
3. 第五判：按起点排成 A(0,2)、B(5,2)、C(6,1)。B 往后看到 C（6 < 7），(未释放, 已释放) ⇒ 记一对 (B, C)，相交 [6, 7)。表非空，`crates/singlefs-checker/src/walk.rs:21740`：「if records_overlapping_in_the_unit_class_reading.is_empty() {」不成立 ⇒ I-3.1 判红，点名 B、C 两条记录的起点。
4. I-5.4：B [5, 7) 与 C [6, 7) 相交，判红，点名同一对。
5. 拿掉 A（别的不动，记账改 2w）：自述那一份 E(θ_high) = {5, 6}，(L)(P)(U) 都成立 ⇒ I-3.1 成立；I-5.4 照旧判红那一对。

同一对 (B, C) 的相交，I-3.1 判不判红取决于同盘上一条不相干的窄记录 A 在不在。C 换成未释放（释放标志去掉、别的不变）时，判决原话「改记之后各条未释放记录罩住的槽区间两两不相交」的字面读法同样打中（B、C 都未释放）。

四句：
- 分不分辨臂：分辨。实现的读法（至少一条未释放就比）与判决字面（只比未释放两两）都打中；「只比至少一条被改记过、且相交落在改记多出来的那几槽上的那一对」不中。
- 系统看不看得到：看得到。`records_on_this_device` 与 `records_set_to_the_unit_classes` 同次序（`walk.rs` 里 `allocated_statistic_left_to_the_span_invariant` 的文档写明），哪几条被改记、多罩了哪几槽在判的那一刻都在手上。
- 满足的分句：I-3.1 条文（`05-空间记账.md:8`）取宽那一支的第五判「…不与这块盘上别的记录（未释放或已释放）罩住的槽区间相交」，「相交判红」。
- 改法在这一格上：判决给的两张验收镜像（攻方「隔离单元尾槽压在释放代落在 (t, θ_high] 的记录上」、辩方「窄记录加尾槽上另一条记录」）里相交的那一对都含一条被改记、相交落在改记多出来的槽上，收窄之后照样红。

改法（只在我的读法上推过、被攻过零轮）：第五判只收「其中一条是被改记过的未释放记录、相交的槽落在它改记多出来的那几槽里」的那一对；自述读法里就相交的对交 I-5.4，不进 I-3.1。代价：`walk.rs` 里那几条第五判单测里比「未改记的未释放记录与已释放记录」的那一格要改期望（推的，没逐条核）。

| 改法 | A 在、B/C 相交那一格 | 攻方验收镜像 | 辩方验收镜像 |
|---|---|---|---|
| 今天的实现（至少一条未释放） | 红（推的） | 红（推的） | 红（推的） |
| 判决字面（未释放两两） | C 未释放时红（推的） | 不红：那一对里另一条已释放（推的；实现员报告第 75 行也这样写） | 视尾槽上那条释放与否（推的） |
| 只比被改记的、相交落在多罩的槽上 | 不红（推的） | 红（推的） | 红（推的） |

什么会推翻：`crates/singlefs-checker/src/walk.rs:10593` 之前另有一道按「被改记过」过滤记录的判；或取宽那一支在 I-5.4 判红的盘上不进。

### Q1-3　说明里的数：造不出要分的情形

改记后相交现在判红（`crates/singlefs-checker/src/walk.rs:21740` 那一判），不进部分没判，部分没判的说明里不会出现「改记后相交」；第二轮那几形（宽记录罩活邻居、窄记录漏数尾槽抵掉泄漏）落进部分没判时说明带改记的记录与两种读法差的槽，读的人分得出。Q1-2 那一形落在判红一侧，不在说明里。推的。

## Q2　I-3.21

### Q2-1　「0 条不适用」：造不出藏病

- 最新根树表 0 条只出现在第一个文件之前（mkfs 第 0 版、照抄它的暖机根、树表 0 条写过行的那一版）。带文件之后最新根回不到 0 条：管理员回退的目标要带文件（`crates/singlefs-core/src/mount.rs` 卸载那一段文档写「回退候选要带文件」，推的，没逐个核回退入口）。
- 那几版里没有一条分配记录，「起点对齐」这件事没有对象；走到的引用（第 0 版树表单元、写行那一版的实例表页）互相压槽、或压在被更早候选根引用的单元上，由 I-5.1（走到的引用两两不重叠，`walk.rs` 里 I-5.1 那一段对全部候选版本的引用取并集排序比）与 I-7.4 / I-2.1（候选根走进被复用或抹头的单元）判。
- 什么会推翻：一条带文件之后、最新根树表 0 条的合法发布路径（例如回退到第一个文件之前的那一版被放开）。

### Q2-2　「未释放或释放代高于 θ_high」：合法镜像误红造不出，1195 个 C944 形照构造全抓

误红那一边，挡住它的是下面这串不等式（推的，每一步指到代码）：
1. 走到的版本：最新根、不被抛弃且 txg ≥ F_生效 的根（`crates/singlefs-checker/src/walk.rs:20962`：「let walked = *index == newest_index || (!abandoned && !below_floor);」），与由记录施加出来的那几版（`crates/singlefs-checker/src/walk.rs:12715`：「let at_or_above_the_floor = record.checkpoint_txg >= rollback_floor;」，调用处传的是 F_生效；`crates/singlefs-checker/src/walk.rs:12716`：「let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;」）。
2. θ_high = max(F_生效, 环里最旧有效根)（`crates/singlefs-checker/src/walk.rs:9826`：「oldest_valid_root_in_the_ring.max(effective_rollback_floor)」）。除最新根外每一版的 txg r 都 ≥ F_生效 且 ≥ 环里最旧有效根 ⇒ r ≥ θ_high。
3. 一个单元被 txg r 的那一版走到、在最新根里已释放 ⇒ 它的释放代 g > r ≥ θ_high ⇒ 它的记录在 E(θ_high) 里（`crates/singlefs-checker/src/walk.rs:10790`：「!record.is_released || record.generation > reclaim_threshold_computed_by_the_checker」）。最新根自己走到的单元在它自己的记录里未释放。
4. 剩下能误红的只有「有 E(θ_high) 记录罩着、起点却不对齐」：合法写者一条记录记一个单元、引用指单元起点（D3 已定项 7「一条记录记一个存储单元」，I-3.18 的条文照它写），I-3.1 的 (P) 已经在每个崩溃状态上判「走到的槽都被 E(θ_high) 罩着」。

漏判那一边：实现的过滤（第 3 步那一行）与 r2 攻方模型 `research/prompts/m3-close-b19-reason-r2-opus-model/i31_q1_sweep.py` 的 `c944_starts` 同一个式子（未释放或 gen > θ_high 的记录起点集合，走到的起点不在里面就算），1195 个形在模型口径下全抓；这一条在实现上判在 I-3.1 那一段之前、不看有没有记账，不被 I-3.1 的分支挡。没在整池镜像上量。

什么会推翻：一段合法历史里，某个候选版本走到的引用起点落在一个多槽单元的中间（例如映射位置条目指到单元内某一槽），或最新根带的记录跨度与它罩住的单元起点错开一槽。

### Q2-3　「有条目而没有种类 3」与「树表读不出」：造不出合法来源，不至于形同虚设

- 前者由 I-9.16 ④ 判畸形（`walk.rs` 里 I-3.21 那一判的文档点名），合法写者的第一个文件那一版就建种类 3 的树。
- 后者：发布次序是单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）（D16 已定项 7），最新根的树表与分配记录树在它落盘之前已落盘，崩溃枚举的状态里最新根这几块完整；读不出只在注入的坏块上。
- 比例：不适用（0 条）占的是新池前几版，部分没判只在注入坏块时出，崩溃点重放各条流上 I-3.21 判到的状态集应与 I-3.1 那一类相同（实现员报告这样推，我没核）。

## Q3　第 0 代根两处与 I-7.9 的上限

### Q3-1　ABSENT 加「槽 0 而 F 为 0 不记空着」：r2 的数可迁到实现的前提逐个对上；合法环变红造不出

- r2 辩方的 +7 / +3 / −55 / −370 出自它模型的 `today_both_fixed` 那一臂（`research/prompts/m3-close-b19-reason-r2-main-verification.md:25`：「坏写者红 +7（S = 4）、+3（S = 8），合法部分没判降 55、370」）。那一臂的三个旋钮：`research/prompts/m3-close-b19-reason-r2-sonnet-model/q3/ding_today.py:28`：「'today_both_fixed': dict(none_with_genesis=False, genesis_state=0, window_out_stop=True),」。实现逐个对上：槽 0 且抬之前 F 为 0 时只交第 0 代根（`crates/singlefs-checker/src/walk.rs:9028`：「if slot == RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION」），它的指针记 ABSENT（`crates/singlefs-checker/src/walk.rs:9036`：「UserVisibleTreeRootPointers::ABSENT,」），窗口外那一停照留。模型里第 0 代根的状态记 0、「前一条」起点记 ABSENT，第 0 代根算一个非空状态；实现里两者都是 ABSENT、第 0 代根不算非空。两种记法只在「不同状态不足 4 个」时可能不同，那时两边都落到最旧有效根（第 0 代根自己，txg 0），取值相同（推的）。
- 在实现上复核：没做。要在实现上复核得把模型那几万段环逐个造成镜像喂池级 checker，这条腿没做；实现员的单测只钉了两张环（`walk.rs` 测试 24015、24035 两条，行号照实现员报告，我没逐条核）。这几个数在判决里应照旧标「模型上量的」。
- 合法环从部分没判变成红：造不出（推的）。按 journal 往下认一槽时，可能的列表里总含「那一刻真住着的那一条」：认到落了盘的那一版就停，一版都没落就是 mkfs 种的第 0 代根。合法写者抬 F 之前读的就是那一槽的真内容，F 不越过按真内容算的上限，于是在真内容那一种可能上成立；判红要每一种可能都不成立，去掉「空着」那一种不会让合法写者每一种都不成立。r2 辩方模型合法写者 `today_both_fixed` 在 `p_rootfail=0` 下红 0：`research/prompts/m3-close-b19-reason-r2-sonnet-model/out/q3-admission-4-128.txt:5`：「cfg=today_both_fixed legal_raise_roots=15627 reaches_ding=13876 final[holds=13977 red=0 partial=1293」。
- 什么会推翻：一张合法镜像上，按 journal 往下认的那一槽真内容不在可能列表里（枚举不全，r2 判过全，前提是 C934、txg 只增、mkfs 种子齐全）。

### Q3-2　写者并进全零槽、checker 不并

**误红：造不出（推的）。** 写者的上限 = min(只按这一遍读得出的根算的上限, 挂载那一刻读坏或读回全零、这一遍又读坏的槽能住的最大 txg)（`crates/singlefs-core/src/mount.rs:4086`：「.filter_map(|ring_slot| ring_slots_bad_since_the_mount.get(ring_slot))」）。并进一个槽只会把写者的上限压低。把那一槽真住着的一条根（txg 不高于它能住的最大 txg h）放回去，按 D16 的式子重算：每块盘最新的有效根只升不降；不同状态从新到旧数，新加的那一条最多把它后面那一条同状态的根挤掉、让那个状态记在更早的 h 上，第 4 新的不会低于 min(原来的第 4 新, h)。所以写者的 F ≤ min(按读得出的算, h) ≤ 按真内容算的上限，checker 按真内容那一种可能判成立。

**坏写者越过字面上限而 checker 放过：造得出（判别子观测不到）。** D16 字面：`.claude/kb/decisions/16-发布语义.md:38`：「挂载那一刻读回全零、之后这个进程没写过的槽，同读坏的槽一样当作可能是一条非空有效根，F 不越过它」。最小历史（R = 3、S = 4、环长 12、区域归属 0 / 1 / 0，落点 区域 = txg mod 3、槽 = (txg div 3) mod 4；全是真写者走得到的步，推的，没跑）：

| 步 | 做什么 | 盘上那一槽（区域 1 槽 2，在盘 1 上） |
|---|---|---|
| 1 | mkfs；实例 1 可写挂载、写第一个文件，发到 txg 7 时崩在 journal 记录之后、根槽之前 | txg 7 落这一槽，根槽没写：全零（mkfs 清过） |
| 2 | 实例 2 可写挂载（恢复施加记录 7），接着发，发到 txg 19 时同样崩在记录之后、根槽之前 | txg 19 也落这一槽：仍全零 |
| 3 | 实例 3 可写挂载：N = 18，新实例第一次发布 txg 20；挂载读这一槽读回全零 ⇒ `AllZeroAtMount`，能住 (6, 19] 里的 7 与 19，最大 19 | 全零 |
| 4 | 实例 3 发 20–30，其中 22、25、27、29 是改变用户可见状态的覆盖写；30 那一次准入不够 | 全零 |
| 5 | 准入抬 F：按读得出的根算，每块盘最新 ≥ 28、第 4 新的非空状态 22 ⇒ 22；并进这一槽 ⇒ min(22, 19) = 19 | 抬 F 那一串的 txg 31 写掉这一槽 |

今天的写者抬到 19。漏了「全零并」的坏写者（这一批之前的代码）抬到 22，越过字面「F 不越过它」。池级 checker 在抬 F 那条根上按 journal 往下认这一槽：可能住 19（实例 2 那一版，行 (2, 19) 里 19 不小于 19，没落）、7（同理）、空着；三种可能里第 4 新的非空状态都是 22（19、7 都比 22 旧），上限都是 22，F = 22 判成立。checker 判不出：区分「写者并没并」要的是挂载那一刻的读数，镜像上没有。盘上那一槽没有持久根，F = 22 不越过任何一条真根，这一格没有数据后果。

四句：分辨「并」与「不并」两种写者（并的 19、不并的 22，checker 都判成立）；判别子观测不到（checker 只有镜像）；满足的是 D16:38「F 不越过它」那一分句；改 checker 碰不到这一格，只有写者的单测钉得住（`crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs` 那几条照新规则改了期望，正文第二节）。

**实现员那一句理由字面站不住。** `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-impl-checker/result.md:60`：「按条款式子本来就不进上限」。条款式子（D16:38）恰恰把读回全零的槽并进上限；站得住的是另一句理由：checker 看不到挂载那一刻的读数，那一槽盘上真全零时没有可越过的根，真住着根时 checker 读得到它、照式子判。结论（I-7.9 不改）照旧成立，理由要换成「判别子观测不到，归写者单测」。

什么会推翻：一段合法历史里写者的上限高于 checker 按真内容那一种可能算的上限（上面那串不等式哪一步错了），或 checker 的重算只取一圈窗口（D16:65）让真内容那一种可能里少了一条写者算进去的根。

## Q4　代码轮：写者一侧

### Q4-1　全零并：相符

| 条款（D16 已定项 1） | 代码 | 判 |
|---|---|---|
| `.claude/kb/decisions/16-发布语义.md:38`：「读坏槽与读回全零的槽都并进上限」 | `crates/singlefs-core/src/allocator.rs:1693-1694` 两类同一个分支：「RootRingOccupant::BadAtMount(txgs_it_can_hold)」与「| RootRingOccupant::AllZeroAtMount { txgs_it_can_hold } => {」 | 相符 |
| `.claude/kb/decisions/16-发布语义.md:38`：「能住的最大 txg 的取法同读坏的槽」 | 补丁把全零那一份改成直接交 `txgs_ring_slots_bad_at_mount_can_hold` 的整个结果（原来只取最旧那个数），与读坏那一份同一个函数、同一个窗口锚 | 相符 |
| 回收门槛钉最旧 txg（D16:39「回收门槛三处 … 钉在它可能住过的最旧 txg」） | 全零那一项照旧取 `oldest_txg_it_can_have_held()`，进程内记根的 E2 重读取 `ring_slots_pinned_since_the_mount_at_the_oldest_txg_they_can_have_held`，两处都没改 | 没被一起动 |
| 只并「这一遍又读坏」的 | `crates/singlefs-core/src/mount.rs:4086` 只在这一遍当没有根的槽里找；`crates/singlefs-core/src/mount.rs:4088`：「.filter(|highest_txg_it_can_hold| *highest_txg_it_can_hold >= current_floor)」 | 窄于字面「挂载那一刻读回全零、之后这个进程没写过」，与读坏那一类原来的写法同；这一遍读得出就照读出来的根算，F 照式子不越过它 |

造不出不一致。Q3-2 那段历史说明这一并在真写者上走得到（崩在记录之后、根槽之前两次落同一槽）。

### Q4-2　推空：相符；停下的理由次序与 E173 装置不同

- 预算：`crates/singlefs-core/src/mount.rs:4810`：「publishes_pushed + 1 + 1 > PUBLISHES_PER_ADMISSION_AT_MOST;」，与 `.claude/kb/decisions/16-发布语义.md:44`：「上限卡在读坏槽或读回全零的槽上推的空发布也算在内」一致；调用处按 `raised.publishes.len()`（推空是 1）加进已推次数。
- 臂定义：`.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md:359`：「推空B 在上限卡在读坏槽上（C并 ≤ F 而 C0 > F）时推一次不抬 F 的空发布、算进已推次数」；实现判「卡住」是 `crates/singlefs-core/src/mount.rs:3975`：「self.not_passing_the_ring_slots_bad_since_the_mount <= floor」与 `crates/singlefs-core/src/mount.rs:3976`：「&& self.from_the_roots_read > floor」。相符。推的那一次带现行 F（`mount.rs` 推空函数里 `assert_eq!` 钉 F 不变）、txg 加一、不写系统配置、不回收。
- 次序：推空发布写掉那一槽时进程记根，根环表里那一项换成这个进程写过的根，下一次算上限不再并它，于是照抬 F 那一串抬（`crates/singlefs-core/src/mount.rs:4835`：「let pushed = if ceilings.are_held_by_ring_slots_bad_since_the_mount(current.root.rollback_floor)」先判卡没卡，再判抬）。没找到次序错。
- 与装置不同的一处：预算连一次推空都放不下时，实现在算上限之前就报预算用完（`crates/singlefs-core/src/mount.rs:4813`：「if an_empty_publish_keeping_the_floor_exceeds_the_budget {」）；装置先算两个上限，都不高于 F 时报抬不动（`scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs:1469`：「if reading.merged > floor {」，不中、不卡时走到 `StepOutcome::Stopped(RejectionReason::FloorAlreadyAtTheCeiling)`）。两边都拒这次请求、盘上一个字节不写，只差交回的停下理由（`PublishesPerAdmissionWouldBeExceeded` 对 `FloorAlreadyAtTheCeiling`）。抬 F 那一臂（照今天）在这一点上本来就是实现先判预算，这一批没改它。四句：不分辨臂（推空B 与照今天在这一格同样先判预算）；系统看得到；不满足条款任何一句的字面（条款只说拒）；没有改法要给。

### Q4-3　按槽认盘：两处仍按 txg 认，结局相同；准入那一处在真写者上与按 txg 认结局相同

仍按落点公式从 txg 推的两处（只在第 0 代根上与按槽不同：mkfs 在三个区域的槽 0 各种一份 txg 0 的根，公式把三份都推到区域 0）：

1. 正常卸载当场放开那一截（D16:43「只有释放代不高于「每块盘上最新有效根的 txg」最小者的那一截当场可用」）：`crates/singlefs-core/src/mount.rs:5275`：「.map(|root| (device_of_txg(root.checkpoint_txg), *root))」。差别只在某块盘上唯一的有效根是第 0 代根时：按 txg 认那块盘一条都没有、整截不放；按槽认取 0、放到释放代 ≤ 0 的槽。释放代 ≤ 0 的已释放槽不存在（mkfs 是 txg 0，最早的释放在 txg 1 之后），两种算法放开的槽相同（推的）。
2. 抬 F 那一遍认「挂载时读坏、这一遍也没读出根的槽」：`crates/singlefs-core/src/mount.rs:5044`：「.map(|root| target_for_publish(root.checkpoint_txg, slots_per_region))」再 `crates/singlefs-core/src/mount.rs:5053` 过滤。区域 1、2 槽 0 读出第 0 代根时被认成区域 0 槽 0：那两槽要是挂载时读坏、这一遍读出了第 0 代根，照旧当没读出、照钉的最旧 txg 并进最小值；区域 0 槽 0 挂载时读坏、这一遍又读坏，却因为别的区域读出第 0 代根被当成读出、从钉里拿掉。两种情形 `valid_root_txgs` 里都已有第 0 代根的 txg 0（它不按 F 滤），最小值是 0 不变（推的）。

准入那一处（`crates/singlefs-core/src/mount.rs:4024` 起按槽认盘）在真写者上：推抬 F 排在写行与暖机之后（`crates/singlefs-core/src/mount.rs:8356`：「warm_up_publishes.push(next.clone());」在前，`crates/singlefs-core/src/mount.rs:8415`：「push_floor_raises_after_the_row_publish(」在后；会话里的推更在挂载交回之后），D16 已定项 8 要暖机推到每块盘上都有本实例的根才往下走。本实例的根 txg > 0、不被实例表抛弃、txg ≥ 现行 F，这些槽是「这个进程知道住着根的槽」，这一遍读坏就重读、仍坏就拒，不会被当成没有根。所以准入算上限时每块盘上最新的有效根都 ≥ 本实例暖机那一条 > 0，第 0 代根算在哪块盘都不改最大值，按槽与按 txg 结局相同（推的）。r2 攻方那段历史要求盘 1 上只剩第 0 代根时还在抬 F，真写者走不到：那段历史里每次区域 1 崩在根槽之前，进程就没了，下一次挂载的暖机要先在盘 1 落一条根，落不下这个进程也走不到推抬 F。用例 `crates/singlefs-harness/tests/admission_floor_ceiling_counts_each_genesis_root_on_the_device_of_its_ring_slot.rs` 的文件头已写明它只在算上限的式子上钉。

什么会推翻：一条不经暖机就推抬 F 的路径（实验装置、checker 档运行模式自算上限的那几处调的是公开函数、可以喂任意根环，不算真写者）；或暖机在某块盘上落根失败而挂载照样做成可写。

### Q4-4　理想模型与胶水：造得出「推空一旦走到就报假的不一致」；今天的随机历史走不到推空

胶水（`crates/singlefs-harness/src/history.rs`）把会话、可写挂载、挂载续做三处交回的每一份 `RaisedFloor` 都当一串抬 F 交给模型，不看这一批新加的 `floor_of_the_publishes`：
- 会话那一处：`crates/singlefs-harness/src/history.rs:5152`：「let raise_answer = model.answer_raise_rollback_floor(ModelCheckpointTxg(raised_floor.0));」，报的上限 `crates/singlefs-harness/src/history.rs:5162`：「reported_ceiling: Some(ModelCheckpointTxg(raised.ceiling.0)),」。挂载续做那一处调同一个函数：`crates/singlefs-harness/src/history.rs:5797`：「let raise_verdict = judge_one_floor_raise_pushed_by_the_session(model, raised);」。
- 可写挂载那一处：`crates/singlefs-harness/src/history.rs:7336`：「.answer_raise_rollback_floor(ModelCheckpointTxg(raised_floor.0));」，报的上限 `crates/singlefs-harness/src/history.rs:7346` 同一句。

推空那一份里：`raised_floor` 是现行 F（推空不抬）；`raised.ceiling` 是并进那几槽之后的上限（`crates/singlefs-core/src/mount.rs:4916`：「ceiling: ceilings.not_passing_the_ring_slots_bad_since_the_mount,」），按「卡住」的定义不高于现行 F。模型不认读坏、全零槽，`rollback_floor_ceiling` 算的是不并那几槽的上限，按「卡住」的定义高于现行 F。`crates/singlefs-harness/src/model.rs:8790`：「if answer.rollback_floor_ceiling == Some(reported) {」两者必不等 ⇒ 交回 `RollbackFloorCeiling` 那一格的不一致。上限那一格过了的话，模型还按「推到每块盘上都有一条带新 F 的根」答（`crates/singlefs-harness/src/model.rs:8238`：「let roots = self.empty_publishes_carrying_the_floor_to_every_device(session, new_floor);」），推空只发一条，根的条数也对不上。

走不走得到（推的，按 `HistoryOperation` 逐个看的文档）：推空要挂载那一刻有读坏或读回全零、窗口里能住 txg 的根槽。随机历史里能留下这种槽的步：`CrashRecoveryAbandoningTheNewestRoot` 让最新那条根的槽挂载时读回全零，但那一步的文档写明可写挂载在取号之前就拒（`crates/singlefs-harness/src/history.rs:527`：「这一步不再造出被抛弃的根」）；掉电截在第一条 journal 记录之前的那一步（`OverwriteLosingPowerBeforeItsJournalRecordThenMountWritable`）留下的槽不在 (N − 3S, 第一次发布 − 1] 里；整理崩溃那一步崩在批与批之间、根都落了。随机历史里没有「崩在记录之后、根槽之前再可写挂载」那一步，走不到推空，所以今天不报假红，也一次都没比过推空。故障注入里说谎设备吞掉根槽写（`fault_injection.rs` 的 `swallowed_root_slot_writes`）之后有没有再可写挂载、交模型比，我没核。

四句：分辨「胶水分不分推空」两种写法；胶水看得到（`floor_of_the_publishes` 就在 `RaisedFloor` 里）；满足的是 `judge_reported_ceiling` 的逐字相等那一判；补模型（让模型认读坏、全零槽与推空）与只改胶水（`KeptWhileTheCeilingIsHeldByRingSlotsBadSinceTheMount` 那一份不交 `answer_raise_rollback_floor`，另比「F 不变、txg 加一、一条根」）都修得到这一格。

要不要同批补：模型那一侧补「认读坏、全零槽」是另一件大活（模型的上限还按 txg 认盘：`crates/singlefs-harness/src/model.rs:3943`：「.entry(device_holding_the_root_of(root.key.checkpoint_txg))」），推空在随机历史里走不到；最小的同批改法是胶水按 `floor_of_the_publishes` 分开，推空那一份走到就明说「模型不答」（`ModelDisagreementAspect::NotModeled` 那一类），不当抬 F 比。推的，被攻过零轮。

## Q5　两份装置的计数分配器

- **E197：「离不开这个分配器」对判的那个数站不住。** 登记写明判的是服务记的 cgroup 峰值：`research/prompts/e197-preregistration.md:21`：「进程内的分配器峰值与 VmHWM 两个数只用来互核，不判」。分配器撑的是互核那几格（A7 推算值 ±5%、A9、A11、停机条款 S3 里「两条量法互不包含」那一判），这几格要按 `Layout` 字节比，VmHWM（同一份装置已在读 `/proc/self/status`）按页、含 malloc 块头，撑不起 ±5% 的推算比对（推的）。登记理由要写成「判的数不靠它；去掉它要改 A7、A9、A11 与 S3 的口径（改登记），判的数不变」，不能写成「量内存非它不可」。
- **E183：站得住。** 它的 `copy_bytes` 是同一个进程里一次 `mounted.allocator.clone()` 前后本线程的在用字节之差（`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs` 第 3370 行起那一段），一个进程里多次调用各要一个数；服务的 cgroup 峰值是整个进程一个数，`/proc/self` 是页粒度、不分调用。仓根 `Cargo.lock` 里没有现成的计数分配器包（`grep` stats_alloc、dhat、tikv-jemalloc、cap 等名字 0 命中）。12.2 那一句「被测的那一调自己不起线程」我核过：`crates/singlefs-core/src`、`crates/singlefs-format/src`、`crates/singlefs-replay-prefix/src` 里 `thread::spawn`、`thread::scope`、`rayon`、`par_iter`、`std::thread` 0 命中。
- **两份都有的、登记表装不下的地方**（推的，没读门禁那一格的判法全文）：
  1. 登记表第 2 段是 crate 根 `src/lib.rs` 的级别（`.claude/gate.d/unsafe-allowances.tsv:6`：「2 crate 根（src/lib.rs）允许写的级别」），`crates/singlefs-checker-tier/src/lib.rs:14` 是 `#![forbid(unsafe_code)]`，表里这个 crate 今天是 forbid 一行（`.claude/gate.d/unsafe-allowances.tsv:31`）。两份装置是 `src/bin/` 下各自的 crate 根，lib.rs 的级别管不到它们；照表的写法登，第 2 段要写 forbid,deny，等于把整个 checker 档库的根也放成 deny，或者门禁要另认「bin 根」。理由里要写明选哪一边。
  2. 两份各 9 处里，`unsafe impl GlobalAlloc for CountingAllocator` 那一处不在任何函数里（E183 第 207 行、E197 第 140 行），一处放开一行的表（第 4 段是函数名）装不下它。
  3. 四个 `unsafe fn` 那一行上方没有连着的 `// SAFETY:` 注释（E183 第 208、217、226、232 行，E197 第 141、150、159、165 行；第一个的上一行是 `unsafe impl` 那一行，另三个的上一行是空行），表头判法要「每一处所在那条语句的开头一行上方连着的 // 注释里有一行 // SAFETY:」（`.claude/gate.d/unsafe-allowances.tsv:18`）。只登表不改代码，门禁照样红。

## 没打中的形状

| 格 | 试过的形状 | 取样范围 |
|---|---|---|
| Q1 | 合法镜像进取宽那一支（要 I-3.18 判红，合法写者不出）；两条已释放相交（I-5.4 罩）；改记把宽记录收窄后去掉自述读法里的相交（I-5.4 照判）；第五判的排序扫描漏对（按起点升序、`take_while(later.slot < end)` 对任一相交对都扫到；跨度 0 的记录罩空区间） | 按代码逐分支推，没有枚举 |
| Q2 | 走到的版本里有 txg 低于 θ_high 的（最新根除外都被候选过滤挡住）；由记录施加出来那一版的 F 过滤用哪个 F（调用处传 F_生效）；「下一次挂载先施加的那一版」进不进遍历（只进 I-5.4 / I-3.10，不进 `walk.references`）；最新根低于 F_生效 时它自己的引用（在它自己的记录里未释放） | 按代码逐分支推 |
| Q3 | 槽 0 只剩第 0 代根一种可能时合法写者越过（写者读的就是那一槽真内容）；写者并读坏槽之后上限高于 checker（上面那串不等式挡）；r2 辩方 `p_rootfail` 那一组（今天的写者走不到，r2 已判） | 推的 |
| Q4 | 推空之后抬 F 的次序；推空写掉卡住的那一槽之后 E2 重读与钉的解开；预算在挂载续做里随删除步清零（这一批之前就这样，`crates/singlefs-core/src/mount.rs:8013`）；推空与暖机、写行之前不推那一条的关系；按槽认盘在准入、重建分配器（`mount.rs` 的 `roots_on_the_devices_they_were_read_from` 已按槽）、回退判候选与 F_生效（取各盘最新根带的 F 的最大值，第 0 代根带 F 0、不改最大值）几处 | 按代码逐处推 |
| Q5 | E183 第 12.2 节「被测的那一调自己不起线程」（grep 核过，站得住）；E197 线程本地计数的「扫环在主线程里跑」（`scan_journal` 所在的 `singlefs-core` 里没有起线程的调用，站得住） | grep 三个 crate 的 `src` |

## 这条腿自己的限度

- 没写模型、没交任务，全部结论是按代码与条款读出来的，标「推的」。Q1-2、Q3-2、Q4-4 三个打中都给了具体的镜像或历史，但没有在副本或模型上量过；主 agent 现查时读 `crates/singlefs-checker/src/walk.rs:10585-10600`、`crates/singlefs-checker/src/walk.rs:21725-21745`、`crates/singlefs-harness/src/history.rs:5141-5165`、`crates/singlefs-harness/src/history.rs:7318-7348` 这几段即可坐实或推翻。
- Q3-2 那段历史的每一步是不是真写者都走得到（崩在记录之后、根槽之前那一点在真写者上是一个崩溃点；恢复施加那一版之后写行的 txg）是按条款推的，没跑。
- Q4-4 只看了随机历史的操作清单文档，没看每个操作的执行体；故障注入里吞掉根槽写之后的那一段没核。
- Q5 关于门禁会不会红的三条是按登记表表头读的，没读 `code-source-discipline.sh` 里那一格的解析代码；那份脚本此刻在工作区里有没提交的改动（另一个会话的工具员在改这一格），以它改完的样子为准。
- 正文第一节说的「实现员把判决两处措辞写宽或写窄」两处，我只在 Q1-2（第五判射程）与 Q2-2（E(θ_high)）里各攻了一个角度；GPU 扫描内核同一判（`c_every_unreleased_record_of_the_unit_class_reading_is_disjoint_from_the_other_records`）我没读，推它与 CPU 同射程。

## 要主 agent 跑的命令

无。这一条腿没写模型、没拷副本；打中的三处都可以靠读上面点名的几段代码现查。

## 没做什么

- 不判正推腿、辩方腿那几格；不替主 agent 采纳；没有副本上量出的数。
- 没往统一任务服务交任务；判决要引的数（r2 辩方模型的 +7 / +3 / −55 / −370 与 `red=0`）是 r2 模型的产物原样，没复跑。
- 没建仓副本、没建编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r3-opus` 里只有认领与写者锁、引文核对的输出 `cite-check.txt` 与改路径用的替换规格 `paths.json`，没有仓副本与编译目录要删。
