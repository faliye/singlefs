# m3-close-b19-reason-r2 辩方腿（Sonnet）报告（2026-11-02）

轮名 m3-close-b19-reason-r2，立场：复核第一轮判决 `research/prompts/m3-close-b19-reason-r1-main-verification.md` 的 Q3「改法」「`!=` 改 `>` 够不够」「Q2 定」写成代码之后够不够得着、是不是同样打中被否掉的替代方案；Q1、Q2、Q3 全答。
读数口径：本腿不交任务、不跑 cargo、不跑模型，下面所有模型读数写「推的，待主 agent 跑」；「量的」只指读自第一轮已落盘产物的数（写明文件与行）。

## 零、开工核对与模型文件

- 开工快照：`sha256sum -c research/prompts/m3-close-b19-reason-r2-snapshot/crates-sha256.txt` 五份全 OK，`kb-sha256.txt` 六份全 OK（量的，本腿开工时跑的），下面引的代码行号都是对着这棵树现查的。
- 对勘误的处理：行号用今天的（`POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST` 8959、`possible_roots_in_the_ring_slot_at_the_raise` 8994、`rollback_floor_against_the_ring_reconstructed_at_the_raise` 9091、`judge_rollback_floor_raises_against_their_ceilings` 9437、两个 I-3.1 新函数 10491 与 10523）；WGSL 行号与正文一致（`c_root_in_the_ring_slot_at_the_raise` 1814、`judge_rollback_floor_raises_against_their_ceilings` 1969）。
- 模型目录 `research/prompts/m3-close-b19-reason-r2-sonnet-model/`。复跑命令（整条在第六节，主 agent 经任务服务跑）：`task-server/task submit --cwd /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model -- bash tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-snapshot --work <新的草稿目录>`。文件与 sha256（模型源码全部 `python3 -m py_compile` 过，语法之外没跑过）：

| 文件 | sha256 |
|---|---|
| `q1q2/q1q2_unit_class_reading.py` | a886289e23bf644b3b912c6dc464347c685778864c1b9b575186022b4163befc |
| `q3/ding_today.py` | 9d82ea84af46e17507be7fe4d50876c1e85f79d7ada01a7a55c3b0e5cbbd588a |
| `q3/history_today.py` | 9e104860650b913100bbc97c2c3e1213ae20c317044d199b2649287d39302346 |
| `q3/q3_today_run.py` | 95d1eab9003da71f0176e37788b752471fd22fc1e4dff93a26a64482db8e5e57 |
| `tools/rerun.sh` | 4ee52bdc02bca324e1848c2d2d802bb4c0722e62cabf88366b253ff022bbe72d |

- `q3/history_today.py` 拷自 `research/prompts/m3-close-b19-reason-r1-opus-model/history_ext.py`（sha256 23521c81f225e4cec3ad05f3070ad51eb8131a3d0cbaddf55b7d82775abee655），只加 `ring_at_raise` 两行；`q3/ding_today.py` 是按今天的 `walk.rs` 重写的丁，不是 `ding_port.py` 的改动（差别四条写在它的文件头）。
- 第一轮产物里我读到并引用的数：`research/prompts/m3-close-b19-reason-r1-opus-model/out/q2ext.txt` 第 3、6 行（admission，S=4 / S=8，J=128）。

## 一、总表

| 判决条目 | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| Q3「改法」：取宽那一支只认整份改记读法，不逐判混取 | **站得住**：实现逐句对上，字面满足的是「改记那一份整份过四判」这一个分句 | 10491–10514 改记，10523–10545 四判，21422–21454 调用点；第二节 Q1 |
| Q3「改法」：另加逐槽两判（Ω ⊆ E(θ_high)；E(t) ⊆ Ω ∪ U） | **站得住，但第二判在 Q 上有一处不对称**：隔离豁免的槽集 Q 按改记之前的跨度算，窄的隔离记录改记后多出的尾槽不在 Q 里，I-3.1 判红而不是部分没判（偏严，不放漏） | 10197、10457、21278；模型 `q1q2` 读数待跑 |
| Q3「`!=` 改 `>` 不够」（只挡宽方向，对窄方向不起作用） | **对新读法够不着**：判决的理由是对旧的逐判混取说的。换成整份改记加逐槽两判之后，`>` 与 `!=` 在两张最小镜像上同样判红；两者剩下的区别是「记录比单元宽、别处都对」的盘面报部分没判（`!=`）还是红（`>`） | 第二节 Q1 第 1 问的最后一段；`q1q2` 的 `triple` 与 `only_span_wrong` 行待跑 |
| I-3.1 部分没判说明补数 | **站得住，小缺口**：补了记账、遍历加 defer、两种读法各自下沿上沿六个数；没点名被改记的是哪几条（要去读 I-3.18 的违例说明） | 21454–21480；第二节 Q1 第 3 问 |
| Q2「定」：全放开那一形实现 | **实现比判决依据的模型保守两处**：认完算空着并记第 0 代根（模型只记第 0 代根一种）；第 0 代根树表记「判不了」（模型当已知空树）。方向都是只多报部分没判，不引入误红 | 9022、9023–9031、9030；第四节 Q3 第 3 问 |
| Q2 理由「两种都算按构造不引入误红」 | **站得住**：只要真环在枚举里，红要求每一种环都红 | 9143、9126 起的汇总；第四节 Q3 第 1 问 |
| Q2 的保守取法（实现员原版，30.0%–54.0%） | **同样打不中被攻的几形，只是更多部分没判**；没有它更安全的盘面 | 第四节 Q3 第 3 问 |
| C944 归属 | 推荐**新立一条（暂名 I-3.21）**，范围取最新根自己走读的引用；I-3.15 反方向次之（范围更宽但每个版本要单独设门） | 第三节 Q2 |
| 窗口外那一停（实现员读法第 1 条） | **站得住**：三项都先滤有效再算，低于 F 的根在两种表示里同样不进式子 | 9035、9143；第四节 Q3 第 2 问 |
| 组合上界 4096 | 判决没写这个数（来自两腿模型 `ding_port.py:18`）；第一轮产物里它几乎不咬（量的） | 第四节 Q3 第 3 问 |
| CPU 与 GPU | 逐支对上；往下认两层以上与第 0 代根被抬 F 盖掉两形 GPU 没有对拍 | 第四节 Q3 第 4 问 |

## 二、Q1　I-3.1 整份改记读法加逐槽两判

### 逐句对代码（判决 Q3「改法」那一行 → 实现）

判决那一行拆成五句，逐句找到字面满足它的分句：

1. 「取宽那一支只认整份改记读法，不逐判混取」：三判不成立才进这一支（「let bounds_with_the_spans_set_to_the_unit_classes = if lies_within_its_bounds {」(`crates/singlefs-checker/src/walk.rs:21422`)）；进了之后先一次改出整份记录（「some_span_was_set.then_some(set_to_the_unit_classes)」(`crates/singlefs-checker/src/walk.rs:10514`)），再整份重算四个界、整份过一个函数（「allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading(」(`crates/singlefs-checker/src/walk.rs:21448`)）。这个函数里读的全是改记那一份的界（10523 起），没有一处回头读记录自述那一份；记录自述那一份的界只在「成立」那个 else 支和说明文字里读（21454 起）。字面满足它的分句是「改记那一份整份过四判」，不是任何一判单独取宽。
2. 改记范围「未释放、起点单元头用得了、跨度与那一类的槽数不符」：已释放不改（「if record.is_released {」(`crates/singlefs-checker/src/walk.rs:10499`)），头用不了不改（「Some(_) | None => *record,」(`crates/singlefs-checker/src/walk.rs:10510`)），不符的改成那一类（「Some(slots_of_the_class) if slots_of_the_class != record.span_slots => {」(`crates/singlefs-checker/src/walk.rs:10503`)，两个方向都改）。「头用得了」与 I-3.18 共用一个函数（「fn class_and_slots_of_the_unit_whose_usable_header_starts_at(」(`crates/singlefs-checker/src/walk.rs:19369`)；I-3.18 的比较在「record.span_slots == slots_of_the_class,」(`crates/singlefs-checker/src/walk.rs:19445`)）。
3. 「遍历走到的每一槽都在改记那一份 E(θ_high) 里」：`every_walked_slot_lies_in_the_unit_class_reading`（10540 起）读 `walked_slots_outside_the_records`，那一份由 `counted_at_the_checker_reclaim_threshold`（10418 起算）算出。
4. 「改记那一份 E(t) 的每一槽都在 Ω ∪ U 里」：`every_slot_of_the_unit_class_reading_is_walked_or_deferred`（10543）读 `slots_counted_at_the_root_reclaim_threshold_outside_the_walk_and_the_deferred`，算它的地方排除了遍历走到的槽、隔离豁免的槽（「!quarantined_exempted_slots.contains(&(device, **slot))」(`crates/singlefs-checker/src/walk.rs:10457`)）和还在 defer 的槽（「!still_deferred_outside_the_walk.contains(slot)」(`crates/singlefs-checker/src/walk.rs:10458`)）。
5. 「不成立判红；成立报部分没判」：「if the_misfit_lies_in_units_the_span_invariant_reddens {」（`crates/singlefs-checker/src/walk.rs:21454`）走部分没判，else 支照 `lies_within_its_bounds` 判红。
6. 两张字节级坏镜像：宽的「set_allocation_record_span(bytes, MAPPING_ROOT, 3)」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:3846`）、窄的 3934 起，断言「红 I-3.1 与 I-3.18：{verdicts:?}」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:3864` 与 3963）。
7. GPU 同改：窄记录的改记在 2915 起（「if (slots_of_the_unit_class == sw(record + 3u)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2926`)），逐槽上沿只在按单元那一类数的那一份判（「if (reads_the_unit_classes && at_the_root_threshold && !still_deferred」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3003`)），两种读法共用一个调用点（「&& bounds.every_slot_counted_at_the_root_threshold_is_walked_or_deferred;」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3098`)）。

实现与判决的两处读法差，不影响判决：隔离豁免的槽集合 Q 按改记**之前**的记录算（「if record.is_released || referenced_start_slots.contains(&(record.device, record.slot)) {」(`crates/singlefs-checker/src/walk.rs:10197`)，调用在 21278），改记那一份的界读的是同一个 Q；改记只看这一块盘上的记录（「.filter(|record| record.device == *device)」(`crates/singlefs-checker/src/walk.rs:21427`)）。

### 第 1 问　还漏不漏

**结论：造不出一张「改记读法本身把真病藏起来而没有别的红」的盘面（推的，待模型）；造得出一个偏严的不对称（隔离的窄记录）。**

I-3.1 判部分没判的前提是 I-3.18 红：改记集合非空 ⇒ 有一条最新根里未释放、头用得了、跨度不符的记录；I-3.18 对同一批记录用同一个函数、同一个比较，而且最新根恒在它的版本清单里（「let walked = *index == newest_index || (!abandoned && !below_floor);」(`crates/singlefs-checker/src/walk.rs:20703`)；I-3.18 读的是一版的全部分配记录树，比 I-3.1 读的最新根树表种类 3 那几棵多一棵根记录持有的树）。所以部分没判的盘面上 I-3.18 必红，部分没判不会让镜像变绿；剩下的问题只是 I-3.1 这一格有没有把「统计漏了或多了的那几槽」说全。

在部分没判成立时，四判把盘面压成：E_改记(θ_high) ⊇ Ω（P）、|E_改记(θ_high)| ≤ A ≤ min(|E_改记(t)|, |Ω| + |Q| + |U|)、E_改记(t) ⊆ Ω ∪ Q ∪ U。只有未释放记录、θ_high = t 时三式夹出 E_改记 = Ω ∪ (E ∩ Q) 且 A = |E_改记|：记账与「遍历走到的单元加隔离单元」逐槽相等，统计本身没有留下可藏的数。t < θ_high 时 A 在 [|E(θ_high)|, min(|E(t)|, |Ω ∪ U|)] 里可以松，松的就是释放代落在 (t, θ_high] 的那几个已释放记录罩的槽，那是 RCW 设计里的松（`crates/singlefs-checker/invariants/05-空间记账.md` 第 8 行 I-3.1 的口径），不是改记带出来的。

题面点的三个形，逐个：

| 形 | 能不能造 | 挡住它的那一行 / 它为什么没有逃出去 |
|---|---|---|
| 改记之后两条记录罩同一槽（I-5.4 只按自述跨度判，看不见：「只看重叠的 I-5.4（分配记录罩住的槽互不相交） 也看不见」(`crates/singlefs-checker/invariants/05-空间记账.md:25`)） | 造得出「两条改记后重叠的记录」，造不出「只有 I-3.1 部分没判说它、别处没红」 | 两个单元都被走读走到时，引用的跨度是走读按被指单元那一类给的（「let span_in_slots = u64::try_from(unit_bytes).expect("单元字节数") / SLOT_BYTES;」(`crates/singlefs-checker/src/walk.rs:2824`)；索引节点给 1，2449），重叠的两个单元引用也重叠，I-5.1 红（「judgements.judge("I-5.1", first_slot + first_span <= second_slot, || {」(`crates/singlefs-checker/src/walk.rs:21241`)）。一个被走到、一个没被走到：没被走到的是泄漏或隔离，泄漏在逐槽上沿或数里红，隔离的带记号记录罩着被引用的槽时 I-3.17 ② 红 |
| 改宽之后罩进一个隔离豁免的槽 | 造得出，结果偏严 | 窄的隔离记录（未释放、没人引用、带记号）：Q 按自述跨度算，改记那一份多出的尾槽不在 Q、不在 Ω、不在 U，逐槽上沿红，I-3.1 判红而不是部分没判。同一个窄记录的单元若是被走读走到的，改记后部分没判。两种盘面 I-3.18 都红，镜像都是红的；偏严只改 I-3.1 这一格的颜色 |
| 改宽之后罩进只有被抛弃根引用的槽 | 造不出新的漏 | 被抛弃根引用的单元不在 Ω（候选集滤掉，20703），也不在 Q、U：改记那一份若把它罩进 E(t)，逐槽上沿红（10543）。这一判只比记录自述那一份严 |
| E(θ_high) 与 E(t) 在释放代落在 (t, θ_high] 的槽上不同时，U 里的槽被逐槽上沿当成合法 | 造得出，但那是该合法的 | U 是已释放、释放代高于 t、没被走到的记录罩的槽，写者在 t 之上照算已分配，所以合法；改记只动未释放记录，不动这一块 |

**模型**（推的，待跑）：`q1q2/q1q2_unit_class_reading.py` 把今天的实现（`!=`）、`>`、不取宽放进同一批小世界逐个盘面比，再对着单元算真值。要读的数在第六节；本条结论被推翻的现象是：`name=partial_ne` 里出现 `kinds` 不是 `only_span_wrong`、且尾字段是 `only_I-3.18` 的行（部分没判成立、盘上有 I-3.18 之外的病、除 I-3.18 外没有别的已建不变量点名），或 `name=healthy_but_not_holds` 非空（健康盘面被判不成立，说明移植或实现有错）。

### `!=` 改 `>` 在新读法下是不是同样被打中

判决说「`!=` 改 `>` 不够：只挡宽方向，对窄方向那一形不起作用」，那是对旧的逐判混取说的。换到整份改记加逐槽两判之后：

- 窄的镜像：`>` 照样改记窄记录（c > span），与 `!=` 完全同一份记录，判红相同。
- 宽的镜像（3834）：`>` 不改记宽记录，改记集合为空，三判不成立即判红（21430 一带，`None` 不进部分没判）；`!=` 改记后遍历走到的 50252 不在 E 里，也判红。两个都红。
- 两者剩下的差别：记录比单元宽、统计照单元记、别处都对时，`!=` 报部分没判，`>` 判红。这时 I-3.18 红，镜像两种写法都是红的；差别只在 I-3.1 这一格是不是替 I-3.18 的病挨红。

所以判决否掉 `>` 的理由对新读法**够不着**。`!=` 仍站得住，理由换成第二轮判决 Q9 的条件（配不上的那块畸形别的不变量判红，那一处报部分没判）：`>` 在宽记录上违反它。模型的 `triple`、`only_span_wrong`、`partial_ne_but_gt_says` 几行给两者的盘面数。推翻「`!=` 仍站得住」的现象：`partial_ne_but_gt_says ... other_kinds` 里 `gt` 报红而 `ne` 报部分没判、且 `kinds` 含 I-3.18 之外的病——那才说明 `!=` 把别的病藏进了部分没判。

### 第 2 问　合法写者的镜像进不进得了这一支、进了会不会误红

**结论：进不了（推的）；进了也不会因四判而误红（没有可进的合法镜像，没法反过来量）。**

- 进支两道闸：三判不成立，且改记集合非空（10514）。改记集合非空 ⇒ I-3.18 红，所以合法镜像上两道闸里第二道本身就关着，前提「I-3.18 在合法镜像上不红」要核的就是「头用得了」这一句在处处成立。
- 「起点槽上是一个已被合法复用的别类单元，而那条记录是未释放的」：合法写者复用的落点，记录先是已释放且释放代够低，复用时同一次分配把罩着这一槽的已回收记录删掉（「复用一个已回收的落点时，新记录罩住的槽上别的已回收记录要随这次分配一起删掉」(`crates/singlefs-checker/invariants/07-分配器.md:9`)）。未释放记录的起点槽上不会是别的单元；已释放的记录根本不进改记（10499）。崩溃点重放的崩溃态里，下一次发布已写下盘、根还没落的单元写在已回收的落点上，最新根的树里罩着它们的是已释放记录，同样不进。
- 「头用得了」的认法：I-3.18 与 I-3.1 同一个函数（19369），GPU 一侧同一个口径（2897 起 `c_slots_of_the_class_of_the_unit_whose_usable_header_starts_at`，I-3.18 的 GPU 判在「judge(INVARIANT_I_3_18, sw(record + 3u) == slots_of_the_class);」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:7647`)）。读不出、头用不了都交回无对象，不改记。
- 反过来想：就算有合法镜像进来，改记那一份是把那几条记录改成单元真占的槽数，统计是写者按单元记的，改记之后四判只会更容易成立；误红要靠「统计与单元也对不上」，那已经是别的病。
推翻它的现象：任一张合法写者镜像（含 `crash_point_replay` 的崩溃态）上 I-3.18 红。这一条在 checker 档的现有回归里是绿的（我没跑，记「没验」）。

### 第 3 问　部分没判说明里的数够不够

**结论：够判「没判的那块最多多大」，不够点名「是哪几条记录」（推的，读代码；说明文字没有跑出来过）。**

说明带的六个数（字节，都在 21471–21472 与 21461 的格式串里）：记账的已分配；遍历全部有效根得到（Ω 加隔离豁免加还在 defer 的，两种读法同一个数，因为已释放记录不改记）；按记录自述的下沿、上沿；改记那一份的下沿、上沿（「；按 I-3.18 判红的记录改记成单元那一类：下沿 {}、上沿（门槛 t 之上）{}，整份三判与逐槽两判都成立」(`crates/singlefs-checker/src/walk.rs:21461`)）；外加按记录自述那一份遍历走到而不在记录里的槽（21473），门槛 t 与现算的门槛都在串里。五个数不够说「没判的那块是哪几条记录」：改记前后两个下沿的差是改记净增的槽数（上限，重叠时更小），但哪几条被改了、各改几槽要去读 I-3.18 的违例说明。

题面问的「两种读法的下沿上沿都把真病包在里面、读说明的人却看不出来」：
- 一窄一宽两条记录同时改，净增为 0，两个下沿、上沿相等。读的人看得出，因为「遍历走到而不在记录里的槽」非空（窄的尾槽）；看不出的是宽的那条多罩了哪一槽。
- t < θ_high 时区间 [下沿, 上沿] 本来就有松（释放代落在 (t, θ_high] 的已释放记录），记账落在松里的泄漏看不出，与健康镜像同样看不出，改记不新增。
- GPU 一侧只打部分没判的标记（3106、3111），没有数；CPU 与 GPU 对拍的是违例位，不比说明。
这条不翻面。补法（小）：说明里加「改记 n 条、净增 k 槽」，不改判。推翻「够判大小」的现象：存在一张部分没判的盘面，说明里六个数夹不出没判那块的大小（模型里 `partial_ne` 的盘面逐个核：改记前后下沿之差 ≥ 真正被改记的槽数）。

## 三、Q2　C944：走到的单元起点没有自己的记录归哪条

### 先说今天看得到什么

C944 的病形是「走到的单元起点没有一条起点就在这里的未释放记录，而邻居记录的跨度罩着它」。I-3.1 的 (P) 是槽级的：遍历走到的槽在某条 E(θ_high) 记录罩住的槽里就过，不要求起点对齐。I-3.18 从记录往单元看，I-3.15 也从记录往单元看（`crates/singlefs-checker/invariants/05-空间记账.md` 第 22 行：「头用不了的那一格没有对象」）。

先把病形收窄。在**走到的世界**里（单元互不相交，I-5.1 判引用重叠），罩着邻居单元起点的那条记录 R 起点在邻居之前，只有三种：R 的跨度等于它的类（R 的单元被走到时与邻居的引用重叠，I-5.1 红；没被走到时 R 是泄漏，I-3.1 的个数或逐槽上沿红）；R 的跨度不等于它的类且头用得了（I-3.18 红）；R 起点槽头用不了（那一槽若被走到，I-2.1 红；没被走到，R 是泄漏，数里或逐槽上沿红）。R 是已释放而释放代高于 θ_high 的记录时，E 数它，那是「活单元被记成已释放放进 defer」，已分配与 defer 的算术另有判（I-3.11，没移植进模型）。所以纯的 C944（只有它、别处不红）在这些单元互不相交的盘面上造不出来（推的），它的价值是**诊断完整**：I-3.18 红、把那条记录跨度改回去之后，被罩的邻居没有记录这件事才冒出来，由 I-3.1 红说。`q1q2` 的 `name=c944_pure` 行就是在小世界里找反例，读法见第六节。

### 第 1 问　合法写者写得出「走到的单元起点上没有未释放记录」吗

**结论：只在没有分配记录树的版本里写得出（推的）；有分配记录树的最新根上写不出。**

| 来源 | 写者代码那一行 / 合法历史 | 写得出吗 |
|---|---|---|
| 中央映射换位 | 引用按映射位置记（`crates/singlefs-checker/invariants/05-空间记账.md` 第 8 行：「按槽数加总对得上，这一条看不见，由 G3 的新判 N2 判红」说的是源记录还在册、目的槽没有记录——那是坏镜像）；合法搬迁在目的槽有记录（`crates/singlefs-harness/tests/checker_relocated_images_judged_by_the_mapping_position.rs` 钉成立） | 写不出 |
| 隔离 | 带记号、没有根引用；I-3.17 ② 要求带记号记录罩住的槽不被最新根引用（「② 带记号的记录罩住的槽不被最新根引用」(`crates/singlefs-checker/src/walk.rs:10553`)） | 不走到，不涉及 |
| 由记录施加出来的那几版 | 它们新分配的单元记录在那一版自己的新根段树里；最新根的树里这些单元要么已释放要么不存在 | 写得出（推的），但在 I-3.1 的最新根范围之外；范围取最新根自己走读的引用就不碰它 |
| 只被更早候选根走到（记录已释放） | I-3.1 的 Ω 是全部候选版本的并集（「let walked_slots_of_every_walked_version: BTreeSet<(u32, u64)> = walk」(`crates/singlefs-checker/src/walk.rs:21284`)），最新根自己那一遍另有一份快照（「let start_slots_referenced_by_the_newest_root: BTreeSet<(u32, u64)> = walk」(`crates/singlefs-checker/src/walk.rs:20598`)，20604 的 `slots_covered_by_the_newest_root` 同一刻）。更早候选根独有的单元在最新根树里是已释放记录 | 写得出；用并集做范围会误红，用最新根那份快照不会 |
| 打包记录单元 | 码 3 与码 1 同占两槽、一条记录一个单元（「码 1 数据单元与码 3 打包记录单元 32768 两槽」(`crates/singlefs-checker/invariants/05-空间记账.md:25`)） | 写不出 |
| 第 0 版树表单元与 mkfs 的实例表第 0 片 | 记录要到第一个文件版本才落盘（「树表 0 条的一版上没有分配记录树，mkfs 写的两个单元的记录要到」(`crates/singlefs-checker/src/walk.rs:10654`)）；mkfs 种的树表单元第 0 版（「树表单元第 0 版（0 条条目）」(`.claude/kb/layout/01-first-txn.md:59`)） | **写得出**（范围：树表里没有种类 3 的版本） |
| 树表 0 条、写过行的那一版 | 分配记录树的根住根记录（`crates/singlefs-checker/src/walk.rs` 2543 的注释）；I-3.1 读最新根记录时树表指针全零只回空表（「if tree_table_pointer.all_zero {」(`crates/singlefs-checker/src/walk.rs:10322`)），不读根记录持有的那一棵；mkfs 的树表单元在这一版里仍被引用却没有记录 | **写得出**（同上；这一版没有记账树，I-3.1 本来就不判） |
| 暖机的零单元发布 | 第一个文件版本补记前面零单元发布的根（「pub fn record_zero_unit_roots_leading_to(&mut self, version_to_build_on: CheckpointTxg) {」(`crates/singlefs-core/src/allocator.rs:2931`)） | 同上，补记之后的文件版本写不出 |

小结：必须设的门是「这一版的树表里有种类 3 的分配记录树条目」；设了之后，最新根自己走读的引用都要有起点对齐的未释放记录。**推翻它的现象**：一个合法写者造出的、树表里带种类 3 的最新根，走读到的单元起点上没有未释放记录（崩溃点重放的崩溃态也算）。

### 第 2 问　归属：四个候选

「单元往记录看」要一个单元集合。今天能拿到的集合只有三份：全部候选版本的并集 Ω（21284）、最新根自己走读的快照（20598、20604）、I-3.15 每一版现成的 `pointed_directly_by_the_root_or_a_parent`（「let pointed_directly_by_the_root_or_a_parent: BTreeSet<(u32, u64)> = [」(`crates/singlefs-checker/src/walk.rs:19198`)）加映射叶里的位置（19216）。候选按这三份分。

| 候选 | 射程 | 在第 1 问的来源上误不误红 | 在 Q1 两张镜像上多抓到什么 |
|---|---|---|---|
| α　I-3.18 加反方向 | I-3.18 的版本清单（候选根、记录施加出来的版、下一次挂载先施加的版，`crates/singlefs-checker/src/walk.rs:20909` 起；判的函数在 19388），只看未释放 | I-3.18 的循环里没有每一版自己的单元集合：取并集 Ω 会在「只被更早候选根走到」那一行误红；取最新根快照就退化成 γ；取 I-3.15 那份就退化成 β。三种取法里第一种不能用 | 同 γ |
| β　I-3.15 加反方向 | I-3.15 的版本清单，每一版用自己的映射与根记录直接指的单元（19127 起），要求每个 (盘, 槽) 在这一版自己的分配记录树里有起点就在这里的未释放记录。**比 γ 宽：更早候选版本自己的树漏记录也会红**，那是 I-3.1 看不见的 | 第 0 版树表单元、树表 0 条写过行的版、暖机的零单元发布版会误红（这些版 `pointed_directly...` 里有树表单元与实例表第 0 片，树里没有它们的记录）；必须按版设门「这一版树表里有种类 3 的条目」，设了之后其余来源不红（推的，没有造盘面验）。老的候选版本是否都满足（年轻池第一圈里一串树表 0 条的版）靠这道门，没量 | 同 γ |
| γ　新立一条（暂名 I-3.21，下一个没用过的号：`crates/singlefs-checker/invariants/05-空间记账.md` 现有到 I-3.20） | 只看最新根自己走读的引用：`start_slots_referenced_by_the_newest_root` 里每个 (盘, 起点槽) 在最新根树表种类 3 那几棵里有一条起点就是这一槽的未释放记录；前置：树表带种类 3 条目（`allocation_record_tree_entries > 0`）且 `allocation_records_of_the_newest_root_read_in_full_without_judging`（10271）读全，读不全报部分没判，没有种类 3 的版本报不适用 | 来源 3、4 在范围外；来源 6、7、8 被前置挡在外；其余不涉及。表里没有误红的来源（推的） | 宽镜像（3834）多一条红，点名被罩的邻居 50252（两盘）。窄镜像（3934）不多：没有被走到而没有记录的单元 |
| δ　把 I-3.1 的 (P) 收紧成起点对齐 | 同 I-3.1（最新根、有记账） | 同 γ | 不推荐：宽记录盖住邻居的盘面上 I-3.1 自己变成红，破坏第二轮 Q9 的条件（别的不变量红时那一处报部分没判），而且要在改记那一份里再解释一遍起点 |

推荐 γ：它需要的数据（最新根快照、读全的最新根分配记录）今天都在 `check_pool_image` 里现成；范围最窄，因此第 1 问表里每一个合法来源都能指出它在范围外或被前置挡住；β 的额外价值（更早候选版本自己的树）是真的，但要先设计门、先量年轻池，记成 γ 之后的扩展，不和 γ 绑在一起。GPU 一侧要新增 `IMPLEMENTED_INVARIANTS` 的一格（现在 113，「pub const IMPLEMENTED_INVARIANTS: [&str; 113] = [」(`crates/singlefs-checker/src/image.rs:176`)）和覆盖表，I-3.18 立项时漏过这一步（「GPU 判器还没判：下标 107 不在覆盖表里」(`crates/singlefs-checker/invariants/05-空间记账.md:25`)；今天 I-3.18 已有 GPU 判，7647）。

**推翻推荐的现象**：`name=c944_pure` 在小世界里找到 `holds` 或 `partial` 且没有别的已建不变量红的盘面——那说明 C944 不只是诊断完整，是真缺口，且那张盘面就是 γ 的最小反例，推荐不变但优先级升；反过来 `c944_pure` 为空，则 γ 是诊断条，可以排在 GPU 对拍之后。

### 第 3 问　γ 的字节级坏镜像

照 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 3834 那张宽镜像改，**跨度改成 2 而不是 3**，记账不动：

- 基底：`build_pool("…").memory_pool()`，写完新池新建文件那份（同 3835）。
- 改法：`mutate_allocation_leaves(&mut image, &UNITS, ALLOCATION_LEAVES, |bytes| set_allocation_record_span(bytes, MAPPING_ROOT, 2) + remove_the_last_allocation_record(bytes, TREE_TABLE))`。映射树根 `MAPPING_ROOT`（50251，码 2 一槽）那条未释放记录跨度 1 → 2，罩 50251、50252；树表 `TREE_TABLE`（50252，一槽）自己那条记录整条拿掉；两盘各 2 处，共 4 处（与 3834 同一个断言「4」）。记账一个字不动。
- 预测（推的，没造出来跑）：50251 + 50252 两槽恰好是原来两条记录罩的两槽，E 与 Ω 集合不变、个数不变，I-3.1 的三判按记录自述就成立（不进部分没判）；I-5.4 无重叠（50253 没有记录）；I-3.15 对 50251 那条记录：它是映射树根，根记录直接指着，成立。今天红的只有 I-3.18（50251 跨 2、码 2 一槽）。γ 立了之后红 [I-3.18, γ]：γ 点名 (盘 0, 50252) 与 (盘 1, 50252)。
- 断言写法照 3864 的：`violated_invariants(&verdicts) == vec![I-3.18, γ]`，`verdicts_that_differ_outside` 对干净镜像逐条相同。
- 阴性对照两张：干净镜像上 γ 判到且成立；只做过 mkfs 的池（`crates/singlefs-harness/tests/` 里 formatted pool 那一族）γ 报不适用。
- 判法变异（照 `crates/mutations.tsv` 的写法）：γ 改成恒成立 / 把「起点对齐」改回「槽级覆盖」都应让这张红的断言红。

这张镜像也是 C944 病形的字节级版：它表明「I-3.18 红之后，被罩邻居没有记录」今天没有任何一条不变量说出口。

## 四、Q3　I-7.9 丁全放开那一形

### 第 1 问　枚举全不全

**结论：真住户一定在 `possible_roots_in_the_ring_slot_at_the_raise` 交回的几种里（推的），条件三个：「实例中间的一版根槽一定落了盘」、一槽里先后住的 txg 只增、mkfs 把第 0 代根种在每个区域的槽 0。三个今天都成立；第一个是判决 Q1 第 1 问已经挂在 D23 已定项 14 第 406 行那一段上的前提，这里不重复攻。**

「全部可能的环都成立才判成立」只在真环在枚举里时才不放坏写者。红要求每一种环都红，成立要求每一种环都成立，所以只要真环是枚举里的一种：真环红 ⇒ 枚举判不出「成立」、真环成立 ⇒ 判不出「红」；合法写者的真环成立，红不可能；坏写者的真环红，成立不可能。逐一核题面点的三处：

1. **逐层停下的条件**。链从 u 往下，停在三处：认作落了盘（「let its_root_landed = version.instance == raising_instance」(`crates/singlefs-checker/src/walk.rs:9054`)，另一半「|| (!rows_of_its_instance.is_empty()」(`crates/singlefs-checker/src/walk.rs:9055`)）；窗口外（「if overwritten_txg < floor_before_the_raise {」(`crates/singlefs-checker/src/walk.rs:9035`)）；落点式的正 txg 认完。抬 F 那一刻这一槽真住的是链上从 u 往下**第一个真落了盘**的那一层；停在「认作落了盘」的那一层只要它真落了（前提），真住户就在它或它之上的某层，都已枚举；没停的层一律把「它没落，往下」也枚举进去。所以真住户跑到链外只有一条路：被认作「落了盘」的那一层其实没落——前提失效。
2. **「落点式的正 txg 认完算空着」**。槽不是 0：这一槽没有更早的落点式 txg，从没写过，空着（9022）。槽 0：mkfs 在每个区域的槽 0 种第 0 代根（「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」(`crates/singlefs-core/src/make_filesystem.rs:822`)），枚举里第 0 代根只在抬之前的 F 为 0 时记（「if slot == RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION」(`crates/singlefs-checker/src/walk.rs:9023`)）；F 大于 0 时第 0 代根 txg 0 低于 F，无效，与空着等价。区域 1、2 的槽 0 在 txg 1、2 落盘时被盖，链里 u = 1、2；那一层要在 journal 里恰有一个实例带提交标记的记录，否则 Err。
3. **journal 里有几个实例**。0 个或多于 1 个都 Err（「let [version] = versions else {」(`crates/singlefs-checker/src/walk.rs:9043`)），整条报环重建不出，不判成立也不判红。GPU 同一支（「if (version_instance == NONE || more_than_one_instance) {」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1909`)）。

另有两个隐含的读法要点名：镜像上坐着 txg 小于抬 F 那条根的根，按「抬 F 那一刻它就在这一槽」处理（「Some((_, _, root)) if root.checkpoint_txg >= raising_txg => {」(`crates/singlefs-checker/src/walk.rs:9243`)，其对面是照读的一支）——靠「一槽里先后住的 txg 只增」，新实例接在环里最大 txg 之后；全零的槽按抬 F 那一刻空着（`AllZero` 一支），靠「写过的槽撕了也不会是全零」。

构造的反例要的现象：一段历史，某一层 u 的根槽写失败而同一实例接着发（D23 第 406 行那一段落地之后的写法），在 `q3_today_run.py` 里 `p_rootfail` 取 0.03 造。**模型读数（推的，待跑）**：`name=q3enum` 行里，`p_rootfail=0` 的 admission 与 bad 两写者 `NOT_COVERED=0`；`p_rootfail=0.03` 的行 `NOT_COVERED>0`（后者证明这一核会红，不是空核）。`NOT_COVERED>0` 出现在 `p_rootfail=0` 的行上就推翻本条。

### 第 2 问　窗口外那一停与「住着一条 txg 低于 F 的无效根」是不是处处等价

**结论：等价（推的）。**

丁先把每一种可能的环的根滤成有效根，再交给上限式子（「let is_valid = root.checkpoint_txg >= floor_before_the_raise」(`crates/singlefs-checker/src/walk.rs:9143`)，滤完才 push 进 `valid_roots_oldest_first`）。算上限的函数只吃滤过的列表，三项逐项：

| 项 | 住着一条 txg < F 的无效根 | 窗口外那一停记一种空着 |
|---|---|---|
| 最旧有效根 `oldest_valid_root_txg` | 被 9143 滤掉，不进列表 | 空着，不进列表 |
| 每块盘最新有效根 | 同上 | 同上 |
| 非空有效根去重计数（比的是前一条**有效**根的树表指针，最旧那条跟 ABSENT 比，8784 起） | 同上；前一条的认法只看列表，被滤掉的根不当「前一条」 | 同上 |

所以式子的输入逐项相同。停的另一个效果是**不去读 journal**：u 低于 F 的层不要求 journal 有记录，不报 Err（见 CPU 单测 `an_overwritten_slot_stops_recognizing_once_the_layer_lies_below_the_floor_before_the_raise`，`crates/singlefs-checker/src/walk.rs` 第 23633 行起，它的文档写着 journal 里没有 14 那一版也不卡），这是只多判不少判的方向。GPU 同一停（「if (lt64(overwritten_txg, floor_before)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1881`)）。

不等价的盘面要满足：u ≥ F 的那一层没停、其下一层 < F 记空着，而真住户是一条 txg ≥ F 的根——真住户 txg 比 u 小又 ≥ F，但它在 u 之下同一槽，落点式要求它 ≡ u mod 3S 且 < u，即 ≤ u − 3S；u − 3S ≥ F 时链没停，< F 时它自己也无效。所以不存在。**推翻它的现象**：`q3stop` 行里 `DIFFERENT_*` 非零（今天的停与不停在两边都判得出结论时判词不同）；`stop_resolves_what_no_stop_leaves_partial_*` 非零只说明停多救回了一些盘面，是预期的。

### 第 3 问　第 0 代根树表判不了、组合上界 4096、合起来的部分没判比例

**3a　第 0 代根记成「判不了」会不会让本该判红的环落到部分没判：会（推的，下面一张手算的盘面，模型待跑）。**

今天的实现对「落点式的正 txg 认完了、槽 0、抬之前 F 为 0」记**两种**：空着，和第 0 代根；第 0 代根的树表记 `None`（「user_visible_tree_root_pointers: None,」(`crates/singlefs-checker/src/walk.rs:9030`)），在上限式子里属于「空不空判不了」（「(None, _) | (_, None) => {」(`crates/singlefs-checker/src/walk.rs:8784`)），它和它后面那一条有效根都进「判不了」一档，下沿只数判得了的、上沿把判不了的一律当计入。D16 的上限式子里「最旧有效根」是兜底，第 0 代根在环里时它就是 txg 0。

手算盘面（S = 8，环长 24，区域→盘 (0, 1, 0)，单实例；这是简化的状态约定，与 `ding_today.py` 的一致）：txg 1、2 是暖机的空树根，txg 3 起树里有文件，状态 A（3、4）、B（5–7）、C（8、9），共 3 个不同的非空状态（不足 4）；txg 10 抬 F 0 → 1；之后写到 txg 26，24 盖区域 0 槽 0（第 0 代根的种子，抬 F 时它还在），25 盖 (1,0)，26 盖 (2,0)。窗口里（txg > 26 − 24 = 2）之前的根是 3–9，去掉最旧的 3 之后只有 B、C 两个状态，窗口定不住，走丁。可能的环：(0,0) 两种 [空着, 第 0 代根]；(1,0) 是 txg 1、(2,0) 是 txg 2（同一实例，认作落了盘，一种）；10–23 盖的槽抬 F 时都是空着。
- 空着那一种：有效根 1…9，最旧 1，3 个非空状态不足 4，上限 = min(盘上最新, 1) = 1，F = 1 成立。
- 第 0 代根那一种：有效根 0（判不了）、1…9。下沿：判得了的非空状态 3 个，不足 4，取最旧 0，下沿 0。上沿：3 个加判不了的 {txg 0, txg 1} 共 5 条计入，第 4 新的是 1，上沿 1。F = 1 落在 (0, 1] 之间：「F 落在上限两沿之间」。
- 两种环判得不一致（成立 / 两沿之间），报可能的环判得不一致，部分没判。

真相：抬 F 那一刻 (0,0) 里就是第 0 代根，树表 0 条，等于 ABSENT，不是「判不了」。把它当已知的空树根：有效根 0…9，最旧 0，不足 4 个非空状态，上限 = min(盘上最新, 0) = 0，F = 1 越过它，**红**。今天落到部分没判。这一形只发生在年轻池第一圈内（槽 0 的第 0 代根要等 txg 3S 才被盖），与第一轮腿说的「第 0 代根只在年轻池第一圈内」同一个范围。落到部分没判不会翻成绿：它是漏红，不是误绿。

**3b　用 mkfs 树表 0 条这条事实能不能判：能，不引实现。** `UserVisibleTreeRootPointers::ABSENT` 的文档已经写着这个事实（「没有前一条有效根时拿它比：两棵树的条目都没有、一个快照都没有（mkfs 种的第 0 版树表就是这样）」(`crates/singlefs-checker/src/walk.rs:8485`)），kb 的 mkfs 清单也有（「树表单元第 0 版（0 条条目）」(`.claude/kb/layout/01-first-txn.md:59`)）；池级 checker 已经另记过第 0 代根的另外两个常量（`GENESIS_ROOT_INSTANCE` 8963、`RING_SLOT_THE_GENESIS_ROOT_IS_SEEDED_IN_EVERY_REGION` 9863），再记「第 0 代根的树表指针 = ABSENT」同一类。改两处：9030 记 `Some(UserVisibleTreeRootPointers::ABSENT)`；9022 的「空着」只在不是「槽 0 且抬之前 F 为 0」时记（那一槽 mkfs 一定种了，空着不可能：种子在每个区域的槽 0，`crates/singlefs-checker/invariants/02-根环.md` 第 24 行 I-7.17：「mkfs 把第 0 代种进全部区域」）。两处都是**收窄**枚举，第 0 代根被盖掉的种子的存在由 mkfs 保证，不是猜。GPU 对应：1865–1876 的 EMPTY 与第 0 代两项、`c_user_visible_pointers` 对基为 NONE 的返回（1602–1605）要有「判得了的空树」一档。**推翻它的现象**：某种格式的池 mkfs 没把第 0 代根种进每个区域的槽 0——那样「空着」才是真的可能；`crates/singlefs-core/src/make_filesystem.rs:822` 是现在的种法。

模型：`cfg=today`（今天）、`today_genesis_only`（只去掉第 0 代那一格的「空着」）、`today_both_fixed`（两处都改）、`verdict`（第一轮模型的口径）。要读的是 `writer=bad`（非法抬 F）那几行 `red`：`today_both_fixed` 比 `today` 多出来的红就是 3a 漏掉的数；`writer=admission`（合法）那几行 `red` 三者都必须是 0（`red>0` 推翻「两种都算按构造不引入误红」）。

**3c　4096 与合起来的部分没判比例。**

- 量的（第一轮攻方 `research/prompts/m3-close-b19-reason-r1-opus-model/out/q2ext.txt`，admission，J = 128，`all` 变体，我读出来、比值是我算的）：S = 4 第 3 行 partial 1293 / 合法抬 F 根 15627 = 8.3%，`partial_why` 只有 `ambiguous:1293`，没有 `combos`；S = 8 第 6 行 partial 604 / 25678 = 2.4%，`combos:3`，`ambiguous:601`。4096 的贡献是 0 与 3 条（0 与 0.012%），部分没判几乎全是「判得不一致」。同文件 bad 写者（非法）S = 8 那一行 `combos:1`。
- 这两个数（8.3% / 2.4%）带第 0 代根与恢复退回（`q2ext` 的历史），和判决里的 5.0% / 2.1%（`q2`，没有第 0 代根）不是同一批历史，不能相加；4096 的 `combos` 本来就计在 `partial` 里，更不能再加一遍。
- 判决没写上界数；4096 来自两腿模型（`MAX_COMBOS = 4096`，`research/prompts/m3-close-b19-reason-r1-opus-model/ding_port.py:18`）。
- 今天的实现报部分没判的比例：**推的，待跑**。方向：今天比 `all` 变体保守两处（3a），部分没判只多不少。读数：`q3-admission-4-128.txt`、`q3-admission-8-128.txt` 里 `cfg=today` 与 `cfg=verdict` 两行的 `legal_raise_roots`、`partial_ratio`、`partial_why`；`cfg=verdict` 的 `partial_ratio` 应当和上面 8.3% / 2.4% 接近但不必逐位相等（我的丁多了区域号纠正，见 `ding_today.py` 文件头第 4 条），差多于一两个百分点说明我的移植有错，读数作废。

### 第 4 问　CPU 与 GPU

**结论：逐支对上（推的，读代码）；往下认两层以上、第 0 代根被抬 F 盖掉两形 GPU 没有对拍镜像（量的：对拍文件里没有）。**

| 支 | CPU | GPU |
|---|---|---|
| 窗口先判，F 落两沿之间不判 | 「if raised_floor > ceiling.lowest_possible && raised_floor <= ceiling.highest_possible {」(`crates/singlefs-checker/src/walk.rs:9511`) | 「if (lt64(lowest, raising.floor) && le64(raising.floor, highest)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2185`) |
| 窗口定得住就用窗口的判词，定不住才走丁 | `window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise`（8838），F ≤ 下沿且定不住才走丁 | 「if (lt64(lowest, raising.floor) || window_bounds_the_ceiling_at_the_raise) {」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2204`)，其后 break；不满足才进下一遍 |
| 窗口外一停 | 9035 | 1881 |
| 第 0 代根的两项（空着 + 第 0 代根） | 9022、9023 | 「if (slot == 0u && is_zero64(floor_before)) {」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1870`)，基 NONE（1875） |
| journal 0 个或多个实例 | 9043 | 1909 |
| 组合上界 | 9126 | 「if (possible_ring_count > C_POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST) {」(`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2053`) |
| 每种环的判词聚合 | 五种结局的 `discriminant` 比对 | 位 0 / 位 1 的或（2193），3 即不一致；任一种环两沿之间 / 没有有效根，立刻部分没判，CPU 是全算完再比，结果同为部分没判 |

「窗口定得住、丁又会判出不一样」的盘面：两边在窗口定得住时都不走丁（CPU 9511 之后直接判窗口，GPU 2204 break），所以在这类盘面上两边不可能因丁而不同；窗口定得住是否真等于抬 F 那一刻的下沿，是 `window_lowest_ceiling_is_the_lowest_ceiling_of_the_ring_at_the_raise`（8832 起的文档）的前提，不归 CPU 与 GPU 对拍。

现有 GPU 对拍文件 `crates/singlefs-checker-tier/tests/crash_judge_gpu_rollback_floor_ceiling_rebuilt_from_the_journal.rs` 七个用例（799、825、851、881、916、949、985）：单实例一层、跨实例一层、u = T 的两种环；文件里没有「第 0 代」被抬 F 盖掉的用例（`grep` 第 0 代只命中第 10 行、第 0 代根还在环里的描述），也没有往下认两层的。CPU 一侧两形都有单测（23530 第 0 代；23585 往下三层）。

**该补的镜像一：第 0 代根被抬 F 盖掉。** 基底：该文件开头那条流（新池新建文件，环长 3S = 24，根环 txg 0 种子、1、2 暖机、(1, 3) 新建文件）；流尾只抄一条 (1, 3) 的根写成 txg 24（实例 1，F 0 → 1，不带卸载记号）——24 mod 3 = 0，(24 div 3) mod 8 = 0，正好盖区域 0 槽 0 的第 0 代根种子。镜像上环：(0,0) = 24，(1,0) = 1，(2,0) = 2，(0,1) = 3，别的全零。窗口（txg > 0）里之前的根 1、2、3，非空状态只有 3 那一版，定不住，走丁；(0,0) 两种 [空着, 第 0 代根]，其余一种。
- 今天的判词（两边应当相同）：空着那一种上限 1、F = 1 成立；第 0 代根那一种上下沿都是 0（有效根 0、1、2、3，非空 1 个，不足 4，取最旧 0），F = 1 越过，红；两种不一致，I-7.9 部分没判。
- 改成「第 0 代根记已知空树、(0,0) 只有第 0 代根一种」之后：单环，上限 0，F = 1 红，两边应判红。
- 阴性对照一张：同基底，抬之前的 F 大于 0（前一条根带 F = 1、本条抬到 2）。第 0 代根 txg 0 低于 F，无效，(0,0) 只有「空着」一种，不涉及第 0 代根的树表，两边应得同一个结论，改法前后不变。

**该补的镜像二：往下认两层以上。** 要三层都落在同一槽（txg ≡ 同一个数 mod 3S）。环长 24 要 55 次发布左右，太长；`make_filesystem.rs:836` 的 `parameters.geometry.root_ring_slots_per_region` 是现成的旋钮，若 S = 4（环长 12）能用：实例 1 到 txg 7（正常卸载，行 T = 7），实例 2 从 8 发到 19（崩溃，行 T = 19），实例 3 重开，抬 F 的根在 20 与 30 之间，31 盖掉 (1, 2) 槽（7 mod 3 = 1，(7 div 3) mod 4 = 2；19 mod 3 = 1，(19 div 3) mod 4 = 2；31 同）。(1,2) 在抬 F 那一刻可能住 [txg 19（实例 2，u = T）, txg 7（实例 1，u = T）, 空着] 三种，journal 里要留 txg 7 与 19 带提交标记的记录。F 取三个环上限的最小值 → 三环都成立；F 越过三个上限 → 三环都红；F 夹在中间 → 不一致，部分没判。CPU 单测 23585 已钉「往下三层认出 [26, 14, 2]」，GPU 缺的是整池镜像上同一个结论的逐状态对拍。

## 五、每一问一个结论

| 问 | 结论 | 类别 | 什么现象推翻 |
|---|---|---|---|
| Q1.1 改记读法本身带出来的漏 | 造不出「部分没判而除 I-3.18 外没有别的红、盘上又有 I-3.18 之外的病」；造得出隔离窄记录判红不判部分没判的不对称（偏严） | 推的（读代码；模型待跑） | `name=partial_ne` 里 `kinds` 非 `only_span_wrong` 且 `only_I-3.18` 的行；`healthy_but_not_holds` 非空 |
| Q1.1 附：`!=` 改 `>` 是否同样被打中 | 在新读法下两者在两张最小镜像上同判红；差别只在宽记录说得全时部分没判（`!=`）还是红（`>`）；`!=` 站得住的理由是 Q9 的条件，不是「不挡窄向」 | 推的 | `partial_ne_but_gt_says ... other_kinds` 里 `gt` 红而 `ne` 部分没判、且含 I-3.18 之外的病 |
| Q1.2 合法镜像误红 | 进不了这一支（改记集合非空 ⇒ I-3.18 红） | 推的 | 任一合法写者镜像（含崩溃态）上 I-3.18 红 |
| Q1.3 说明里的数 | 够判大小，不够点名被改记的记录 | 推的 | 存在部分没判盘面，改记前后下沿之差小于真被改记的槽数 |
| Q2.1 合法来源 | 有分配记录树的最新根上写不出；树表里没有种类 3 的版本（mkfs 第 0 版、树表 0 条写过行的版、暖机零单元发布版）写得出 | 推的（读代码） | 有种类 3 的最新根上走读到的单元起点没有未释放记录 |
| Q2.2 归属 | 推荐新立一条（暂名 I-3.21，最新根范围，前置：树表带种类 3 且读全）；I-3.15 反方向（β）范围更宽、要按版设门，记为扩展；I-3.18 反方向（α）在今天的循环里没有每版的单元集合，退化成前两者；收紧 I-3.1 的 P 不推荐 | 推的 | `name=c944_pure` 有行（缺口是真的，优先级升）/ 为空（诊断条，优先级降） |
| Q2.3 字节级坏镜像 | 3834 那张宽镜像改跨度 3 → 2，其余不动；今天只红 I-3.18，γ 立后红 [I-3.18, γ] | 推的（预测） | 造出来跑，今天红的不止 I-3.18，或 γ 立后不红 |
| Q3.1 枚举全不全 | 全，条件是三个前提；三处（逐层停、认完算空着、journal 实例数）逐一核过 | 推的 | `q3enum` 的 `NOT_COVERED` 在 `p_rootfail=0` 的行上非零 |
| Q3.2 窗口外一停 | 与「住着无效根」处处等价，三项逐项核 | 推的 | `q3stop` 里 `DIFFERENT_*` 非零 |
| Q3.3a 第 0 代根判不了会不会让本该红的环落到部分没判 | 会（一张手算盘面，年轻池第一圈内） | 推的 | `bad` 写者 `today_both_fixed` 的 `red` 不比 `today` 多 |
| Q3.3b 能不能用 mkfs 树表 0 条判 | 能，不引实现；改两处（9030、9022），GPU 对应 1865–1876 与 1602–1605 | 推的 | 某种格式的池 mkfs 不把第 0 代根种进每个区域槽 0 |
| Q3.3c 4096 与合起来的比例 | 4096 几乎不咬（量的：0 与 3 条）；今天实现的部分没判比例要跑 | 量的（第一轮产物）+ 推的（今天） | `cfg=today` 与 `cfg=verdict` 的 `partial_ratio` 差反方向 |
| Q3.4 CPU 与 GPU | 逐支对上；缺两张对拍镜像（第 0 代根被抬 F 盖掉；往下认两层以上），造法与两边应判什么写在第四节 | 推的（读代码） | 补的镜像上 CPU 与 GPU 违例位或判到位不同 |

## 六、要主 agent 跑的命令

**命令一（非重型：纯 python，不起 cargo、不碰 GPU、不进 checker 档；耗时推的：十几分钟到一小时）**

```
task-server/task submit --cwd /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model -- bash tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-snapshot --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-reason-r2-sonnet-run1
```

- 在哪个目录起：`--cwd` 就是模型目录 `/home/fy5090/code/singlefs/research/prompts/m3-close-b19-reason-r2-sonnet-model`。`--work` 要一个不存在或空的新目录（上面这条是例子，换一个没用过的也行）。`--frozen` 这个模型不读，指到开工快照目录只为过参数检查。
- 跑完看哪几行：`TASK_SERVER_RESULT` 那一行 `type=` 与 `exit=0`；`task-server/task log <号>` 里九行 `name=<名>_no_reference lines=<数>`（`q1q2`、`q3-admission-4-128`、`q3-bad-4-128`、`q3-admission-4-32`、`q3-rootfail-4-128`、`q3-admission-8-128`、`q3-bad-8-128`、`q3-admission-8-32`、`q3-rootfail-8-128`），没有 `_exit_`、`_differs`、`rerun_batch_incomplete`。第一次没有 `out/` 参照，所以是 `no_reference`；主 agent 核过输出之后把 `--work` 里九个 `.txt` 拷进模型目录的 `out/`，之后复跑就逐行比。
- 要读的数（`--work` 目录里的文件；写之前的预测放在括号里，读到相反的数就按「什么现象推翻」那一栏改结论）：
  - `q1q2.txt`：`name=q1q2 ... worlds=`；`name=healthy_but_not_holds`（预测：没有）；`name=verdict mode=ne|gt|none`；`name=partial_ne kinds=… only_I-3.18|named_by_another_invariant worlds=`（预测：`kinds` 里没有 `only_I-3.18` 配 I-3.18 之外的病）；`name=partial_ne_i318_red`（预测：只有 `True`）；`name=side upper_slot_judge_alone_turns_partial_into_red_mode_ne=`（逐槽上沿单独起作用的盘面数，为 0 说明它在这批小世界里被个数夹住）；`name=only_span_wrong mode=… verdict=…`、`name=partial_ne_but_gt_says`、`name=partial_gt_but_ne_says`、`name=triple`；`name=c944_pure`、`name=c944_ne_not_red`；以及 `example` 行。
  - `q3-*.txt`：`q3 ... cfg=<名> legal_raise_roots=… reaches_ding=… final[...] partial_why[...] partial_ratio=` 的 `today`、`verdict`、`today_genesis_only`、`today_both_fixed` 四行（admission 写者 `red` 预测全为 0；bad 写者 `today_both_fixed` 的 `red` 预测不少于 `today` 的）；`q3enum ... NOT_COVERED=`（`p_rootfail=0` 预测为 0，`p_rootfail=0.03` 预测大于 0）；`q3stop ...`（`DIFFERENT_*` 预测没有）；`cfg=verdict` 的合法 `partial_ratio` 与第一轮 8.3%（S = 4）/ 2.4%（S = 8）的差。
- 主 agent 跑完，新派一条同立场的腿，派发提示写「承接：research/prompts/m3-close-b19-reason-r2-sonnet-output.md」与任务号，读 `task-server/task log <号>` 与 `--work` 里的文件，把本报告里标「推的，待主 agent 跑」的读数落成数。

不需要交的命令：命名检查（本腿没改 `crates/` 或 `scripts/` 下的 `.rs`）；`--selftest` 与门禁阶段（没碰）。

## 七、没做什么

- 没往统一任务服务交任务，没跑 cargo，没碰 GPU 与 checker 档；模型一次都没跑（只做了 `python3 -m py_compile` 与 `bash -n`），所以本报告所有模型读数都是「推的，待主 agent 跑」，判决要引的数由主 agent 复跑（命令在第六节）。
- 没造入库的字节级坏镜像，Q2.3 与 Q3.4 只给造法与预测。
- 没核 I-3.18 在合法镜像上绿这一句的 checker 档回归（没跑）；Q1.2 的「进不了」是读代码推的。
- 没量 4096 在今天实现上的比例；没量 β（I-3.15 反方向）的门在年轻池上的行为。
- 没读禁读清单里的两个路径（`research/prompts/m3-close-b19-reason-r2-opus-output.md`、`research/prompts/m3-close-b19-reason-r2-opus-model/`）。
- 模型的状态约定是第一轮攻方模型的：历史生成器里初始状态 0 既当「空树」又在写者一侧数成非空，我手算的 3a 盘面按真 checker 的 ABSENT 约定写，与 `ding_today.py` 里 `cfg` 的 `genesis_state=0` 不逐位一致；模型读数只用来比相对差。
- 没替主 agent撤销或维持判决，只交复核结果。
