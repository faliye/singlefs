# 本地攻方提示译文核对表：abandoned-floor-r1（X3　10 槽的账）

提示：`research/prompts/abandoned-floor-r1-local-attack.md`。代码行号按冻结副本 `/tmp/claude-1000/abandoned-floor-r1/tree/crates/` 现取；kb 行号在 `.claude/kb/` 里现取；调查报告指 `research/prompts/closeout-recheck-2026-09-28/row43-investigator-report.md`；正文指 `research/prompts/_abandoned-floor-r1-body.md`。

## 一、逐句对照

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| 开头指令（不用 markdown 强调、按题号答、每答一句 This would be refuted by、只用英文、不写行号） | `.claude/agents/three-way-local-attack.md`「做什么」第 1 条；`.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」 | 无 | 同首稿；另加「每个表格行也要写推翻句」 |
| 每格填数或「not determinable」加缺的事实、或上下界；不许只答 yes / no；格后方括号写 FACT 号 | 定义「做什么」第 1 条；`three-way-inference.md`「本地腿只问能落成数、能逐格判的题」 | 无 | 同首稿 |
| SCOPE：别的腿在副本上跑候选、别的腿对已定分项判相容；不答那两题，不造别的历史 | 正文第 44–48 行（X1、X2）、第 61 行 | 无 | 同首稿 |
| VOCABULARY：txg、根环、实例、F、F_eff 指 FACT 2 | 背景用语，出处落在各 FACT；F_eff 定义见 FACT 2 | 无 | 同首稿 |
| VOCABULARY：slot 16384 字节 | `crates/singlefs-format/src/lib.rs:14` `pub const SLOT_BYTES: u64 = 16384;` | 无 | 同首稿 |
| VOCABULARY：segment 64 槽 | `crates/singlefs-format/src/lib.rs:271` | 无 | 同首稿 |
| VOCABULARY：释放代是换下它的那次发布的 txg | 调查报告:133「实例 2 第一次发布（写行，txg 6，在 B 上面接）把它们换下、记成释放代 6」 | 无 | 同首稿 |
| VOCABULARY：checker 从盘上读镜像 | `.claude/kb/invariants.md:144`（I-3.11 行「读的是镜像里的记账行、不读内存里的分配器」） | 无 | 同首稿 |
| FACT 1 定义句 | `.claude/kb/invariants.md:134`「已分配空间统计 == 实际遍历所有引用得到的和」 | 无 | 同首稿 |
| FACT 1 checker 读法、有效根 = 候选集（实例表条件、txg ≥ F_生效） | invariants.md:134「按根环里全部有效根的引用取并集」「2026-09-17 起「有效根」= 回退候选集里的根：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效」 | 无 | 同首稿 |
| FACT 1 理由句 | invariants.md:134「被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里」 | 无 | 同首稿 |
| FACT 1 不适用句 | invariants.md:134「最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」」 | 无 | 同首稿 |
| FACT 1 隔离豁免句 | invariants.md:134「隔离的记录：已分配而没有任何根引用的记录，按单元豁免——它罩住的那个单元只要有任何一份副本 checker 自己读出来读不出或校验和对不上，这个单元在每块盘上的那几条记录都豁免；每一份都读得出且对得上，照旧判违例」 | 无 | 同首稿 |
| FACT 2 F_生效 的算法 | `crates/singlefs-checker/src/walk.rs:5843`–5846 注释、5849–5874；invariants.md:134「池里自证过、fsid 与本池相同的系统配置槽带的 F」 | 缺「幸存盘」「持久」、缺「每块盘取落在它上面的根环区域里 (txg, 实例) 最大的那一条」、缺「最新根的实例表读不出时一行都没有、不滤」 | 补了 surviving device、durable、largest (txg, instance) 那一句、instance table cannot be read 那一句 |
| FACT 2 被抛弃根的 F 不算 | walk.rs:5845「被抛弃时间线上的根带的 F 算不算，D16 写着仍开着，这里照「有效根」的字面不算」、5850 | 无 | 同首稿 |
| FACT 2 被抛弃的判法 | walk.rs:5840 `row.instance == root.instance && root.checkpoint_txg > row.published_checkpoint_txg` | 无 | 同首稿 |
| FACT 3 低于下界、走不走 | walk.rs:5884 `let below_floor = root.checkpoint_txg < effective_rollback_floor;`、5885 `let walked = *index == newest_index \|\| (!abandoned && !below_floor);` | 无 | 同首稿 |
| FACT 4 已分配行、defer 行 | `crates/singlefs-core/src/transaction.rs:6292`、6299 | 无 | 同首稿 |
| FACT 4 defer 注释 | transaction.rs:6298「第 5 项：已释放、还在 defer 窗口里的（它们仍算在已分配里：占着空间、被根环里的有效根引用）。」 | 无 | 同首稿 |
| FACT 5 可再分配谓词 | `.claude/kb/decisions/16-发布语义.md:33`「\| 可再分配 \| 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) \|」 | 无 | 同首稿（英文写 the settled rule：同文件第 9 行已定项 1 状态「已定」） |
| FACT 5 回收挑哪些记录 | `crates/singlefs-core/src/allocator.rs:1567` `.filter(\|record\| record.is_released && record.generation <= floor)` | 无 | 同首稿 |
| FACT 5 回收一槽做什么 | allocator.rs:655–676（`mark_reclaimed`：清分配位、used_per_segment 减、allocated_slots 与 deferred_slots 减、free_slots 加、free_runs 改） | 首稿写「函数体不碰别的」，漏了 free_runs 的改动，说过了头 | 改成「还改 free-run 计数；只清分配位，不清隔离位、扣住位」 |
| FACT 5 回收下界 | `crates/singlefs-core/src/mount.rs:1147`–1152 `oldest_valid_root.map_or(effective_floor, \|oldest\| effective_floor.max(oldest))` | 无 | 同首稿 |
| FACT 5 最旧有效根 | mount.rs:2346–2350（没被实例表判抛弃的根里 txg 最小的） | 无 | 同首稿 |
| FACT 5 抬 F 回收扣到生效、放开 | mount.rs:2367–2370 `ReclaimedReuse::HeldUntilFloorTakesEffect`；allocator.rs:1583–1586；allocator.rs:1593「抬 F 生效（带新 F 的根落满每块盘）之后，把扣住的回收槽放开。」 | 无 | 同首稿 |
| FACT 5 挂载时回收立即可用 | mount.rs:1433–1436 `ReclaimedReuse::Immediately` | 无 | 同首稿 |
| FACT 6 D23 已定项 14 那条 ⚠️ 第一句（英文写 A note under decision D23 settled item 14，不说「settled」以外的强度） | `.claude/kb/decisions/23-journal的角色与格式.md:402`「影子账与按实例表判抛弃（`abandoned_by_table`）留着，理由是崩溃恢复，不是管理员回退：一段没有任何管理员回退的历史里，崩溃恢复因暂时读错落到旧根、实例表抛弃了较新的根，关掉影子账，被抛弃的根引用的数据单元被复用，之后恢复落在它上面读不出」 | 无 | 同首稿 |
| FACT 6 C554 乙-配置续那一句 | D23:402「C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续（用户 2026-09-27 定）之后，系统配置见证过的较新的根暂时读不出不再被抛弃：可写挂载重读一次，仍读不出就拒可写；影子账留着的理由收窄成系统配置没见证到的那一形（那次发布的系统配置轮换没落盘）。」 | 无 | 同首稿 |
| FACT 6 代码规则：只被被抛弃根引用的槽 = … | mount.rs:1209「只被被抛弃根引用的槽 = 被抛弃根引用的槽 − 候选集里的根引用的槽 − 当前这一版账里」–1210「还分配着的槽；候选 = 可读 ∧ 按实例表有效 ∧ txg ≥ F」；代码 1233、1238、1258 | 无 | 同首稿 |
| FACT 6 用户窄读法措辞与收严 | mount.rs:1210–1213「用户 2026-09-16 定的窄读法措辞是「仍被有效根引用的槽不在其内」、有效只按实例表判，豁免只给候选集里的根（多了 txg ≥ F、抬 F 之后按新 F 重算）是对那句措辞的收严……预想、偏离用户定案的措辞，交用户。」 | 首稿整句漏了：只写了代码规则，没写它是对用户措辞的收严、标着预想、交用户 | 补了 The same comment adds … to be put to the user 一句 |
| FACT 6 一条根引用的是什么 | mount.rs:1214「一条根引用的 = 它那一版账里还分配着的落点；账里已释放的是它换下的上一版的，不算它引用。」 | 无 | 同首稿 |
| FACT 6 隔离位独立、只增不撤、清的时机 | mount.rs:1216–1217「隔离位独立于分配位（`DeviceFreeMap::isolate`），所以候选集缩小（抬 F）之后重算只会多隔离几个槽，不用撤销；清只在被抛弃的根离开根环的那一次发布里清」 | 无 | 同首稿 |
| FACT 6 抬 F 时先按新 F 重算影子账 | mount.rs:2351–2352「候选集按新 F 缩小，只被被抛弃根引用的槽会变多（一个槽此前靠一条低于新 F 的根豁免），回收之前先按新 F 重算影子账」 | 无 | 同首稿 |
| FACT 6 隔离槽数只住内存 | mount.rs:824「影子账隔离的槽数，逐盘（D28（挂载期承诺量） 已定项 1 第九项「被抛弃根独占量」，只住内存）」 | 无 | 同首稿 |
| FACT 6 isolate 做什么 | allocator.rs:576–591 | 无 | 同首稿 |
| 事实段标题：谁量的、代码改没改 | 调查报告:89「副本 checker 另打印明细」、:238「副本相对主工作区在 history.rs 与 walk.rs 上的全部改动，都是探针、打印与 ROW43_EXPERIMENT 开关」 | 首稿写「all measured on unmodified code」，说过了头：checker 是调查员的副本 | 改成 filesystem code unmodified、checker 是调查员副本（探针、打印、实验开关） |
| FACT 7 步骤 | 调查报告:103「A（mkfs 进程 txg 3）→ 覆盖写 B（4）→ 覆盖写 C（5）→ 崩在 C 的轮换之前 → 可写挂载，C 一时读不出（实例 2，写行 6、暖机 7）→ 关掉、再可写挂载（实例 3，写行 8、暖机 9、10）→ 覆盖写四次（11–14）→ 抬 F 到 5（空发布 15、16）」；:92 C=(inst 1, txg 5)、落到 txg 4；:113、:116 txg 3、4 是实例 1 | 首稿「读不出」没写全 0、重读一次仍读不出、挂载交回后写回 | 按 :56「C 的根槽与数据单元读回全 0（重读一次仍读不出），挂载交回后原样写回」补上；:82 说最小复现用的就是这一形 |
| FACT 7 再挂载时 C 被判抛弃、影子账隔离 | 调查报告:80「再挂载时 C 读得出，按实例 2 的表判被抛弃，影子账把它的单元隔离，后面的覆盖写不再复用它们」 | 无 | 同首稿 |
| FACT 7 抬 F 之前无违例、抬 F 的回报 | 调查报告:95 `violations_before=[]`、:96 `raise=Ok(publishes 2, reclaimed 13, ceiling 11, abandoned_roots_unreadable 0)` | 见第二节「不译」 | 同首稿 |
| FACT 8 机理数 | 调查报告:97「根环槽数 24、最新根 txg 16、环里自证过的根槽 17 个、最老的自证过的根 txg 0、遍历的候选根槽 11 个、并进遍历的由记录施加出来的版本 0 个、被实例表判抛弃的根槽 1 个、回退下界 F 5、低于 F 的根槽 5 个」 | 无 | 同首稿 |
| FACT 8 三条根 | 调查报告:112–115 | 无 | 同首稿（区域号不译，见第二节） |
| FACT 9 10 槽与引用它们的根 | 调查报告:116–124、:130「释放代 6 的记录合计 2 + 8 = 10 槽」、:132、:133「候选集里的根（txg ≥ 5、没被抛弃的：6–16）一条都不引用它们」 | 无 | 同首稿 |
| FACT 9 盘 1 相同 | 调查报告:109「盘 1 逐项相同」 | 无 | 同首稿 |
| FACT 10 盘 0 总数 | 调查报告:125 `device=0 allocated=Some(2326528) walked=2162688 quarantined_exempted=0 defer=Some(2031616)`；:103「差 2326528 − 2162688 = 163840 = 10 × 16384，两盘相同」；:97 抬后只有 I-3.1 | 无 | 同首稿 |
| FACT 11 另 15 槽、豁免只看未释放 | 调查报告:128、:219「豁免只看未释放的记录」 | 无 | 同首稿 |
| FACT 12 扫描二 | 调查报告:80「98 格里 50 格抬 F 被上限拒、盘面不变且全绿；48 格抬 F 做成，其中 42 格跑完全绿，6 格以已知红第 0 条收尾：两种造法各 3 格，都是覆盖写 4、5、6 次、F = 5（C 那个 txg，空档）。同一段只换 F：3、4、6、7、8、9 全绿，只有 5 红。」 | 首稿只写了「只换 F 只有 5 红」，漏了 98 格的拆分（50 被上限拒、48 做成、42 绿、6 红、各 3 格、覆盖写 4–6 次） | 补全 |
| FACT 13 甲 | 正文:33「候选集下界取「txg ≤ F_生效 的最大一条没被抛弃的根」也遍历」 | 无 | 同首稿（括注「副本探针：99 条全绿」不译，见第二节） |
| FACT 13 乙（YI1、YI2） | 正文:34「抬 F 的目标 txg 上的根全属于被抛弃实例时，改取下一个不全被抛弃的 txg，或拒这次抬 F。」 | 无 | 同首稿 |
| FACT 13 丙 | 正文:35「抬 F 时把只被 F 之下的根与被抛弃根引用、已释放的槽回收（写明它与影子账「被抛弃根引用的单元不复用」怎么相容）」 | 无 | 同首稿 |
| FACT 13 丁（DING1、DING2） | 正文:36「checker 侧的「实际遍历」并上影子账隔离集，或并上 F 之下仍在 defer 里的释放记录。」 | 无 | 同首稿 |
| FACT 14 C379 题面 | `.claude/kb/checks-owed.md:315` 第二、三列 | 无 | 同首稿 |
| FACT 14「那三个数来自早先攻方的副本、另一个池面」 | checks-owed.md:315 末列「攻方副本上的数，入库装置上没重做」；调查报告:178「数与欠账行登记的攻方数（3310、50304 → 50368）不同，是另一个池面」 | 无 | 同首稿 |
| FACT 15 段数向上取整、末段可能不满 | allocator.rs:372 `let segments = unit_area_slots.div_ceil(CLUSTER_SEGMENT_SLOTS);`、:373 注释「末尾那一段可能不满 64 槽」、:386 | 无 | 同首稿 |
| FACT 15 empty_segments | allocator.rs:683–692（`.filter(\|used\| **used == 0)` 在 687） | 无 | 同首稿 |
| FACT 15 lowest_empty_segment | allocator.rs:754 注释「单元区内最低的 64 槽对齐全空段的起点」、756–769（758 `full_segments = self.allocated.len() / per_segment`，761–763 三个条件，768 起点槽） | 无 | 同首稿 |
| FACT 15 记账行取 empty_segments | transaction.rs:6301 | 无 | 同首稿 |
| FACT 16 造镜像 | 调查报告:161 | 首稿没写读记账行的是调查员副本 checker | 补 the same investigator copy |
| FACT 17 S1–S4 | 调查报告:166–169、173–175、:178「盘 1 逐项相同」 | 无 | 同首稿（S3 的 defer 1228800 不译，见第二节） |
| FACT 18 C379 三个改法 | 正文:40 | 无 | 同首稿 |
| Q1–Q9 题目 | 正文:51「按写死的事实表逐格算：复现镜像上那 10 槽，在今天与甲、乙、丙、丁下，记账已分配、checker 遍历、defer、影子账隔离各是多少，I-3.1 两边差多少；C379 那一格按事实表算两个口径各给多少段」 | 无 | 同首稿；多出来的见第三节 |

## 二、原文有、英文不译

| 原文 | 文件:行 | 为什么不译 |
|---|---|---|
| I-3.1 行的实现状态、坏镜像用例、C556 已还清 | invariants.md:134 | 不是口径，不进账 |
| I-3.1 行「不适用」那一格后面的括注（实二的读法、主 agent 2026-09-26 暂认）与为什么照判会误红那一段 | invariants.md:134 | 这段历史最新根 F 5 = F_生效 5（调查报告:112），不走这一格；留了「不适用」的判据句 |
| I-3.1 行中央映射条目那一句、由记录施加出来的那一版那一句、已知缺口那一句 | invariants.md:134 | 调查报告:97「并进遍历的由记录施加出来的版本 0 个」，已作为 FACT 8 的数给出；映射条目这段历史没有出场的观测 |
| D23:402 中间两句（只在那次挂载里隔离挡不住、落 C314） | D23:402 | 讲为什么要留影子账，不改这 10 槽的数 |
| mount.rs:1215 读不出的被抛弃根只计数、候选根读不出当不豁免 | mount.rs:1215 | 这次抬 F `abandoned_roots_unreadable 0`（调查报告:96，已在 FACT 7） |
| 抬 F 回报里的 `reclaimed 13` | 调查报告:96 | 它是回收函数交回的记录条数（allocator.rs:1541–1554 只交回第一块盘的），与 FACT 11 的「15 槽」单位不同，给出来模型会拿槽数去对条数；FACT 11 已给那 15 槽的去向 |
| 三条根的区域号（region 0 / 1 / 2） | 调查报告:113–115 | 与账无关 |
| 甲后面的括注「副本探针：99 条全绿」；调查报告 2.3 表的各探针结局与第三节末段 3307 那次反事实 | 正文:33；调查报告:147–155、:180 | 是在改过的副本上量出的候选结局，正是这一格要本地腿自己算的数；给了就不是独立算出来的。今天代码上只换 F 的观测（FACT 12）照给 |
| S3 那一行的 defer `1228800` | 调查报告:169 | C379 那一格不问 defer |
| C379 行第三、四列（造镜像断言、判别力自证、与 D16 与 I-3.1 一起判、不在 C318 射程） | checks-owed.md:315 | 是欠账的还法与归属，不是口径；两个口径在第二列 |
| 调查报告第一节随机历史的计数、第 ② 行「被抛弃根的 F」那一节 | 调查报告:14–49、:139–155 | 不在 X3 攻击面；X1、X2 的题 |

## 三、英文比原文多出来的

| 英文多出来的 | 为什么加 |
|---|---|
| 候选标签 JIA、YI1、YI2、BING、DING1、DING2、TODAY | 中文甲乙丙丁不能进英文提示；乙、丁各有两个分支，拆成两行才能逐格填；TODAY 是正文「今天」 |
| VOCABULARY 一节（txg、实例、记录的字段、checker 读盘） | 本地腿读不到仓，名词要先给定义；每个定义都落在第一节的出处 |
| Q1 各列 (a)–(g) 与「x + y」拆法 | 正文:51 只说「各是多少」，拆成 50176–50177 与 50257–50264 两段，是因为两段被引用的根不同（FACT 9），主 agent 比数时能看出差在哪一段 |
| Q1 (g) 全盘字节差 | 正文:51「I-3.1 两边差多少」没说只算 10 槽；全盘的数由 FACT 10 起算，事实不够时要它说缺哪条 |
| Q3 DING2 的第二种读法「只被 F 之下的根引用的记录」 | 正文:36「F 之下仍在 defer 里的释放记录」可以读成释放代在 F 之下，也可以读成被 F 之下的根引用；两种都要它算，不替它选 |
| Q5 用 FACT 10 的字节核 TODAY 行 | 给主 agent 一格能直接对 FACT 10 的自检 |
| Q6 (b)「按与 (a) 同一个段数组数」 | 两个函数数的段集合不同（FACT 15），不说清楚 (b) 就有两种数法 |
| Q7 末段不满那一题 | FACT 15 的代码事实让两个口径还有一处与隔离位无关的差，要它单独算上限 |
| Q8 C379 三个改法下 S3 的数 | 正文:51「两个口径各给多少段」落到正文:40 的三个改法上 |
| Q9 两次跳段的槽数与段数 | 把 C379 欠账行（另一个池面）与这次的数放到同一个单位上比 |
| ASSUMPTION 行的要求 | 事实表没写的前提（例如某些隔离槽落在哪一段）要它写出来，主 agent 才核得到 |
