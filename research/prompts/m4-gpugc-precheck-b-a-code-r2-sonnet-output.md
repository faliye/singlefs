# m4-gpugc-precheck-b-a-code-r2 辩方腿（Sonnet）报告（2026-11-12）

批次 `records/2026-10-19-里程碑四GPU垃圾回收开工.md#130`。立场：替第一轮判决里被判「打中」的两样（甲文档的前提、拒绝报里少的卸载重挂出路）与被判「这一份取样没撞上」的两样（乙槽对那一判、甲）辩护，复核判决够不够得着、是不是同样打中乙甲之前的现行形态；辩不住的照实写。禁读清单里的文件与目录没读。

读的树：冻结副本 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2/repo/`。被判的两份文件与主工作区、与快照清单逐字节相同（量的，`sha256sum`：`allocator.rs` 前 12 位 `00b3d99a0492`、`mounted_session.rs` 前 12 位 `62462e6b49b6`）。下文引代码的行号都是这棵树的；引探针的行号是攻方模型 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/` 的。探针读数取主 agent 复跑的 `~/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r1-main-check-opus-run/out/oracle.out`（sha256 前 16 位 `82a71415e825c942`，量的，本腿现算）。本腿自己没有量任何数：新模型写好了没编、没跑（子 agent 不交任务），下面凡标「推的，待主 agent 跑」的都是读码推出的。

## 总表：判决条目 / 辩方判定 / 证据

「站得住」：判决那一条够得着它说的那件事。「够不着」：证据与它下的结论之间差一步。「同样打中别的臂」：同一形状在乙甲之前的现行形态上同样成立，不分辨臂。

| 判决条目（第一轮判决第二节） | 辩方判定 | 证据（细节在 T1、T2、T3 各节） |
|---|---|---|
| 乙在槽对那一判上误拒：这一份取样没撞上，登欠账 | **站得住**；但判决写的「底 19351 有区分力」口径宽了，是全族的底，不是槽对那一判的底 | `base_published_after_a_series` 在 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:196` 数的是产品那一臂推过一串之后写进的格，任何触发都算；槽对那一判的有效分母是 1907 格里 O 那一臂真推了至少一串的格，装置没报（T2-2）。读码单调性论证另给支撑（T2-2） |
| 甲的前提「推了这次写也是被合取 7 拒」不成立，打中 | **站得住的是「前提没有被代码推出」；够不着「前提为假」**；**同样打中现行**：同一前提原本就写在 `refusal_is_short_of_space` 合取 7 那一臂（现行不推串的依据） | `series_product_raised=24` 只证推串会改全空段数（存在性），`series_*_crossed_the_floor=0` 的分母装置没报、产品那一臂结构上几乎走不到（T2-2）；修后的文档措辞「可能」「不另估」与证据等强（T1） |
| 甲误拒：这一份取样没撞上，分母太小 | **站得住**；分母 4（56681 次判里判出 4 次），且格间不独立（T2-2），有效分母可能更小 | 装置里 `fired_by_conjunct_before_any_series=4`、O 那一臂这 4 格 2 格仍被合取 7 拒、2 格仍放不下（oracle.out:41-42）（T2-2） |
| 乙、甲先拒之后卸载重挂写得进（打中，改法零轮交用户） | **站得住：这条出路在 1907 格里 12 格、甲 4 格里 2 格走得通**；**够不着「乙甲带来的」**：拒绝报没有出路字段是先算这一类报的原本形状，现行先算（式子那一判）同样不报，已登记 C1003；乙只是扩大了判出集合 | `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 无出路字段（`mounted_session.rs:691`）；式子那一判判出的 11514 格重挂写进 0（oracle.out:53-56，没有 `remount=published` 行）；现行形态推串之后的出路只在「被宽护卡住」时列（`mounted_session.rs:1990`）；本装置的池造不出宽护（T2-4，推的，待主 agent 跑） |
| 甲射程宽一形（先走空发成几步再判） | **站得住**（读码）；装置走不到这一形 | `conjunct_fired_with_versions_published=0` 是装置没有留着没走完的删除队列：13908 次 unlink 一次都没被拒（oracle.out:30），分母 0（T2-2） |
| 先乙后甲，两样同时成立只报乙：「不打中」 | **够不着**：判决引的两个 0 是类型状态计数，产品代码的两条构造路各填一样，恒为 0，不是「两样同时成立」的次数 | `trigger_of` 的 `both-empty`、`both-set` 两支（`research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:132-134`）；产品构造 `mounted_session.rs:6234-6235`（乙：`devices_short` 非空、`None`）与 `:6247-6248`（甲：`Vec::new()`、`Some`）。「两样同时成立」的真次数装置没数，新模型加了（T2-2） |
| 改 size、建快照不先判：成立（照推一边） | **站得住** | 读码，两腿一致；`mounted_session.rs:6154` 的穷举无 `_ =>` |
| 两处调用没有真池用例：打中（覆盖） | **站得住**，第 128 批已补两条用例与两行变异 | T1 |
| 每次发布克隆分配器多一张位图：不判违规 | **站得住** | `DeviceFreeMap` 派生 `Clone`（`allocator.rs:365`），读法 O(聚簇段数) |
| 检查单：类型与名字不成立；测试优先多线程不成立；其余五格成立 | 修后八格逐格见检查单一节 | |

三处同类先例（带出处，不用重新论证利弊顶替）：
- 「读成 0、底是 0 按没量到记」：`research/prompts/m4-gpugc-floor-raise-precheck-code-r3-main-verification.md:33`：「| 误拒 | `misrefused=0`，底是 0（第一节第 3 条） | 没量到，不算没打中 |」。
- 「先算判出放不下时报里没有卸载重挂」：`.claude/kb/checks-owed.md:607`（C1003）：「第三轮攻方探针跟着出路卸载重挂 663 次（卡住的 285、头一次判出放不下的 177、推过一串之后判出的 201）写进 0 次，推环 0」。这是乙甲之前的现行形态，177 格是现行先算头一次判出放不下之后跟着重挂。
- 「推了写得进的出路」这一形早先以「聚簇段那一项」撤回过：`research/prompts/m4-gpugc-floor-raise-precheck-code-r3-main-verification.md:27` 一行（撤回不丢出路那一格）。找不到别的先例，不用别家实现。

## T2　产生结论的方法（主攻）

### T2-1　探针各读数怎么算（读码，对照模型源码；数取自 oracle.out）

| 读数 | 怎么算 | 模型源码位置 | 是不是判决说的那件事 |
|---|---|---|---|
| `fired_by_*`（1907、11514、4） | 产品那一臂被拒的就是 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 时，按成员里的两个字段分类 | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:208-210`、分类 `:112-136` | 是 |
| `misrefused_*` | O 那一臂在 `before.copy()` 上把两道先算关掉再发同一个改动，写进了才记 | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:227-228`（拷贝与发布）、`:244-245`（写进才记）；关先算的两处钩子 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/tools/patch_the_copy.py:44`（甲返回 `None`）、`:58`（乙返回空表） | 是：「推了本来写得进」就是这个定义；O 臂与产品臂走同一个推串循环（`mounted_session.rs:6306` 先算，`:6317` 之后是同一段推串），预算相同 |
| `remount_published_*` | 把 `before`（改动之前的池）正常卸载再可写挂载，先算照开，同一个改动写进了才记 | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:257-258`、`:269-270`；`research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/pool.rs:209-211`（卸载抬 F 到现行那一版） | 是出路的字面（卸载再可写挂载）；`before` 在 `series-before=0` 且没走空时等于被拒之后的池（oracle.out 里 `series-before=1` 共 83 格不是，推的，方向不明） |
| `series_*_raised` | 每推成一串抬 F 记推前、推后每块盘的 `empty_segments()`，有一块盘变多且没有变少记 raised | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:155`、`:157-162`；前后快照钩子 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/tools/patch_the_copy.py:77-92`；读数函数 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/core_probe_module.rs:35-41` | 是同一个数：甲、合取 7 读的就是 `DeviceFreeMap::empty_segments`（`allocator.rs:1162`）；记的是一整串（若干次空发布）的净变化，不是单次空发布 |
| `series_*_crossed_the_floor` | 推前有一块盘少于 `N₀`、推后每块盘都不少于 | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:164-168`；`N₀` 在每次改动开头读一次（`:181`） | 是「越过」的字面；分母见 T2-2 |
| `base_published_after_a_series` | 产品那一臂推过至少一串、写进了 | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:195-197` | 是全族的底，不分触发 |
| `fired_both_members_empty/set` | `trigger_of` 里两个字段（`devices_short` 空不空、`empty_cluster_segments_below_the_floor` 有没有）的两种别的组合 | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:132-134` | **不是**「两样同时成立」的次数，见 T2-2 |
| `conjunct_before_any_series_judged/found` | 甲那一判每跑到末尾一次记一笔、判出记 `found` | `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/tools/patch_the_copy.py:41` | 是：56681 次判、4 次判出（oracle.out） |

### T2-2　读成 0 的项：分母与装置走不走得到

下表的分母都取自 oracle.out；装置没报的写「没报」，不拿别的数顶。

| 读成 0 的项 | 分母 | 装置走得到吗 | 辩方读法 |
|---|---|---|---|
| `misrefused_slot_pairs=0` | 1907 格判出（oracle.out:64：O 那一臂在这 1907 格全以 `NoSpaceAfterRaisingTheFloor` 收场，其中可能一串都没推）；有效分母（O 那一臂真推了至少一串的格）**装置没按触发拆**，只有全部触发合计 7217（`oracle pushed-more-series-than-the-product`），占 13425 格判出的 53.8%，槽对那一判占其中多少没报：0 到 1907 都相容 | 走得到，但格间不独立：1907 格里 1874 格是 big-write（oracle.out），同一配置里同一个满池被反复拒；配置数没报 | 「这一份取样没撞上」站得住；「底 19351 有区分力」口径宽了（它是全族的底，任何触发都算）。单调性读码另给支撑：聚簇段集合只在 `allocator.rs:2930` 一处插入、只增不减，上界读的是推串之前的集合，推串只会让段外槽对更少（推的，读码，不独立于这一份读法） |
| `misrefused_formula=0` | 11514 | 走得到 | 这是乙之前就有的式子那一判，不归乙甲 |
| `fired_by_mixed=0`、`misrefused_mixed=0`、`remount_published_mixed=0` | 0 | **几乎走不到**：两块盘等大（`research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/shapes.rs:296` 的 `Pool::start([extra, extra])`）、每个单元落每一块盘，两块盘状态对称，短缺种类不会在两块盘上分开（推的） | 射程里「短缺种类混着来时可能推过一串才判出」没被这一装置罩到，读成 0 不是没撞上 |
| `misrefused_conjunct_before_any_series=0` | 4（oracle.out:38） | 走得到但极稀：56681 次判里 4 次判出 | 「分母太小」站得住；4 格里留了样例的两格同属一个配置（`REMOUNT_PUBLISHED` 两行样例都是 seed=60、`extra=1024`，oracle.out），格间不独立，有效分母可能更小 |
| `conjunct_fired_with_versions_published=0` | 0 | **这一族里没出现**：13908 次 unlink 一次都没被拒（oracle.out:30，没有 `kind=unlink refused` 行），全部 13425 格判出的 `versions-published` 都是 no（oracle.out），删除队列没有留下没走完的步，「走空发成几步之后甲才判出」这一形装置里没造出来（装置没有构造它的步骤，推的） | 判决「射程宽一形：成立（读码）」靠的是读码，装置读数一格都没罩；`a_walk_step_refused_for_space...`（第 128 批新用例）只罩接线 |
| `fired_both_members_empty=0`、`fired_both_members_set=0` | 判出 13425 格 | **恒 0**：产品只有两条构造路（`mounted_session.rs:6234-6235` 乙：`devices_short` 非空、`None`；`:6247-6248` 甲：`Vec::new()`、`Some`），另外两种组合构造不出 | 判决「先乙后甲，两样同时成立只报乙：不打中」引这两个 0 做依据，**够不着**：乙先判出时甲不跑（`mounted_session.rs:6232` 之后才到 `:6240`），所以「两样同时成立的格数」装置从来没数。读码结论（同时成立时只报乙、甲那一份丢掉）成立，但它是读码、不是量的（新模型加了 `count-below-floor` 一格去数） |
| `series_product_crossed_the_floor=0`（31219 串） | 推前有一块盘少于 `N₀` 的串数：**装置没报** | **产品臂结构上几乎走不到**：甲只在这次调用一串都没推时判（`mounted_session.rs:6195`：「if series_pushed_in_this_change > 0」是直接返回 `None` 的条件之一），产品臂在全空段为 0 时头一次被拒的前台写入被甲拦下、不推；剩下能让产品臂在 `N₀` 之下推串的只有删除类（装置里 unlink 从不被拒）与第一串推完全空段数掉到 0 之后的第二串 | 31219 不是分母；这个 0 不是比率 |
| `series_oracle_crossed_the_floor=0`（7300 串） | 推前少于 `N₀` 的串数：**装置没报**；来源只有 O 那一臂在甲判出的 4 格上推的串与别的触发里恰好全空段为 0 的格 | 走得到、极稀 | 同上，不是比率 |

对这一组读数的辩护：

1. 判决从 `series_product_raised=24`、`series_oracle_raised=7` 推出「推串里的空发布能把全空段数推高，注释的前提没有被代码推出」，这一步**站得住**：这两个数是存在性证据，机制读码对得上（释放最后一条未释放记录时：「self.full_segments_without_an_unreleased_record += 1;」（`crates/singlefs-core/src/allocator.rs:877`），空发布换下固定点单元走这条路）。
2. 判决没有、也不该从 `crossed_the_floor=0` 推出「推串不会让写写得进」：分母没报、产品臂结构上走不到，这一格只能记「没量到」。判决文字「越过 `N₀` 的 0 串，甲误拒 0 格」把两个分母不明的 0 并排写，读者会当成比率，应补「分母装置没报」。
3. 同一份读数反过来也支持甲的方向：推串让全空段数变少的串是变多的 285 倍（产品臂 6846 对 24，O 臂 1543 对 7，oracle.out:75-82，算的）。甲的实际依据是「推串更常把它推低」，不是「推串不改它」；修后的文档措辞「可能」「不另估」（`mounted_session.rs:2120-2121`、`:6175-6176`）与这一读数等强，没有写过头。

### T2-3　O 那一臂是不是站得住的判别子

- 关得掉：两道先算在拷贝里各被一处钩子关掉（`research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/tools/patch_the_copy.py:44`、`:58`），自证 `name=self_check ok O 那一臂先算关得掉` 读的是「键里带 `oracle=NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 的计数为 0」（oracle.out 末尾自证行）。
- 推的上限与产品相同：O 臂与产品臂都走 `push_one_floor_raise_for_a_refusal_for_space`（`mounted_session.rs:6306` 起），预算参数 `publishes_pushed_in_this_admission` 与上限是同一份，差别只在先算那两段被短路；读码，没有第三处不同。所以 O 臂「写进了」就是产品推满之后写得进。
- 方向：O 臂推得不比产品少（产品臂判出时一串不推或只推一串，O 臂推到预算或上限），误拒只会多算不会漏算，对「乙误拒」这个问题 O 是**更强**的判别子（现行先算会放行的写 O 也放行）。
- 弱点一：O 臂不是「乙甲之前的现行」。它连式子那一判也关了（`:58` 把整个 `devices_short_even_with_everything_a_floor_raise_could_release` 交空表）。回答「乙比现行多挡了什么」要另一臂：只把槽对上界换回老算法、其余照产品。新模型加了这一臂（T2-5）。
- 弱点二：7217/13425 的判出格里 O 臂真推了串，其余 6208 格 O 臂一串都没推就以 `NoSpaceAfterRaisingTheFloor` 收场（F 已在上限、预算等），那些格上误拒不可能发生，读成 0 是平凡的。装置不按触发拆这一格（T2-2 第一行），新模型拆了。
- 弱点三：没有阳性对照。自证只证「O 关得掉」，没有「注入一个已知偏紧的上界、O 臂应当报出误拒」这一格（`.claude/rules/evidence-discipline.md@校验路径本身也要证明它会红`）。`misrefusal_base_zero` 那一行只在全族推串后写进为 0 时打，对「槽对那一判的有效分母为 0」无从报。这一条本腿没补（模型已大，补法推荐见「没做什么」）。

### T2-4　「拒绝报里少卸载重挂出路」够不够得着；现行丢不丢

读码事实（冻结副本）：

- 先算判出时交回的成员 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`（`mounted_session.rs:691`）只有 `devices_short`、`empty_cluster_segments_below_the_floor`、`last_refusal`、`floor_raises`、`refusals_that_pushed_the_floor_raises`、`versions_published` 六个字段，没有出路字段。这个形状在乙甲之前的式子那一判判出时就是这样。
- 现行推串之后交回的 `NoSpaceAfterRaisingTheFloor`（`mounted_session.rs:673`）带 `wide_protection_at_the_ceiling`（`:683`）；出路只在这一项是 `StuckBehindTheWideProtection` 时才列（`:872` 起 `ways_out_after`，卸载重挂一定列）。成为这一形要两件事：F 已在上限（「FloorRaiseStop::FloorAlreadyAtTheCeiling { .. } => {}」，`crates/singlefs-core/src/mounted_session.rs:1997`），且 C 那一圈这一刻隔离着槽（`:2007` 起的判）。别的推到不再推的结局（预算用完、抬 F 那一串自己被拒、C 一槽都没隔离）一律交 `NotWhatStopsTheAdmission`、不列出路。

所以「推串之后拒绝报里有卸载重挂」在乙甲之前本来就只覆盖被宽护卡住这一形；其余的拒绝（含式子那一判先算判出的）从来不报。

量的（先算判出之后跟着卸载重挂）：

| 先算 | 判出 | 重挂写进 | 来源 |
|---|---|---|---|
| 式子那一判（乙之前就有） | 11514 | 0 | oracle.out:53-56（没有 `remount=published` 行）；另 11472 格重挂之后先算又判出一次 |
| 槽对那一判（乙） | 1907 | 12（0.63%） | oracle.out:68 |
| 合取 7 那一判（甲） | 4 | 2 | oracle.out:43-44 |
| 现行先算头一次判出放不下（乙之前，别的探针） | 177 | 0 | `.claude/kb/checks-owed.md:607`（C1003）：「第三轮攻方探针跟着出路卸载重挂 663 次（卡住的 285、头一次判出放不下的 177、推过一串之后判出的 201）写进 0 次，推环 0」 |

辩方读法：

1. 判决这一条**打中**站得住（12 与 2 是实数），但它不分辨臂：先算这一类的报都没有出路字段，C1003 在乙甲之前就登记了这个缺口；按 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 表里「打中不分辨臂」那一行，应另立一笔账，不拿它判乙、甲。判决已把它并进 C1003、标零轮交用户，处置对；只是第二节那一行的措辞「先算判出放不下时，有一条走得通的出路没报」读起来像是乙甲新造的，应写明「现行先算同形」。
2. **乙甲新带来多少**是这一条真正没量的。一个可被推翻的读码预期（推的，待主 agent 跑）：本装置的池是干净 mkfs、暖机、进程内写、干净卸载的，没有读故障、没有被抛弃的根，C 那一圈不隔离槽，现行那一形（推到不再推、报被宽护卡住、列出路）在这个池族里造不出；所以那 12 格里乙之前的现行形态推串之后同样以 `NotWhatStopsTheAdmission` 收场、同样不报出路，用户看到的结局相同（ENOSPC、没有出路），乙只是省了一次白推。新模型数了 `fired by=slot-pairs isolated-before=some`、`oracle-wide-protection=stuck`（应为 0）与老上界一臂的结局（T2-5）。
3. 反过来，A/B 那一族里乙确实拿掉了现行报的出路：判决引的「乙少报 4801 格」出自 A/B 任务 13802 的日志（`outlet_fewer_than_as_coded=4801`，正推腿报告里的原样字段；本腿没复核），被跟着重挂的 `lost_first` 类共 295 格写进 0（4801 格里有几格被跟过日志没单列，上限 295）。所以现有的两份量法合起来是：A/B 族里拿掉了、但拿掉的出路 0/295 走得通；本探针族里出路 12/1907 走得通、但很可能现行也不报。**没有一份读数同时是「乙甲拿掉了」且「走得通」**。这一句是辩方最强的一句，也是需要主 agent 跑新模型才能升成「量的」的一句。

### T2-5　新模型与要量的（细节在「模型」与「要主 agent 跑的命令」两节）

在攻方模型上加了四样，都不改原有计数（同一批种子，原有 30 个读数应与 oracle.out 的 `PROBE_RESULT` 逐字段相同，可对照）：

- 每个触发拆「O 臂推了几串」「判出之前隔离着槽没有」「O 臂收场是不是被宽护卡住」「判出时全空段数是否已少于 `N₀`（两样同时成立）」「判出格落在几个配置」。
- 推前少于 `N₀` 的串数（产品、O 两臂），即 `crossed_the_floor` 的分母。
- 强推一臂：产品臂头一次被合取 7 自己拒的 413 格（oracle.out 里 `Publish:EmptyClusterSegmentsBelowTheFloor` 五行之和，算的）上，把合取 7 的被拒当成空间不够让会话推串，数写没写进、推几串、每串前后全空段数。这是甲前提的直接测量，分母 413 而不是 4，且现行形态本来就是不推。
- 老上界一臂：槽对那一判的上界换回「每个放得开的槽各补一对」（仓里变异表那一行的原文，`crates/mutations.tsv:7707`），其余照产品，在 1907 格上发同一个改动，记现行是同样判出、推了写进、推了仍被拒（带被宽护卡住与否）；并与重挂写进的 12 格交叉。

## T1　第一轮的修补本身

### T1-1　两处文档与代码、与探针读数逐句对

两处文字（冻结副本）：`crates/singlefs-core/src/mounted_session.rs:2120`：「// 抬 F 把 defer 里的槽回收不改这个数；推串里的空发布会换下固定点单元，可能把全空段数推高，判的是被拒那一刻的数，」与 `crates/singlefs-core/src/mounted_session.rs:2121`：「// 推高之后越不越过 `N₀` 不另估，不为它推串。原样交回这个成员，调用方分得出「先整理」与「真没空间」。」；甲那一判的文档 `crates/singlefs-core/src/mounted_session.rs:6175`：「/// 抬 F 回收 defer 里的槽不改这个数（defer 里的已释放槽本来就不算占用）；推串里的空发布会换下固定点单元，可能把全空段数推高，」与 `crates/singlefs-core/src/mounted_session.rs:6176`：「/// 推高之后越不越过 `N₀` 不另估。只读分配器（内存）。」。

| 句子 | 对得上什么 | 判 |
|---|---|---|
| 抬 F 回收 defer 里的槽不改这个数 | `empty_segments` 的文档与实现：defer 里的已释放槽不算占用（`allocator.rs:1157-1163`），回收只清分配位、不动未释放记录（第一轮正推腿读码；本腿只核了 `allocator.rs:877` 一带是释放那一支，没有另验回收路径） | 成立 |
| 推串里的空发布会换下固定点单元 | 机制：空发布换下上一版固定点单元，释放走 `mark_released`，未释放计数减到 0 且是整段时全空段数加一（`allocator.rs:877`）。探针只记一串的净变化，**没追到是哪一次释放**；全仓能让这个数变多的只有这一条路径，所以机制是读码推出的、效果是量的（24/31219，7/7300） | 成立；机制标「推的」更准 |
| 可能把全空段数推高 | 24/31219、7/7300（oracle.out:75-82，量的）；「可能」与存在性证据等强 | 成立，没写过头 |
| 判的是被拒那一刻的数 | 甲读 `device_map.empty_segments()`（`mounted_session.rs:6177` 起的函数体），不估推串之后 | 成立 |
| 推高之后越不越过 `N₀` 不另估 | 代码确实不估；**不是**「量到越不过」（越不越过的分母见 T2-2，装置没报） | 成立，文字只说了代码不估，没有说量到什么 |
| 不为它推串 | 现行设计：`refusal_is_short_of_space` 对合取 7 的被拒返回 `false`（`mounted_session.rs:2116` 起的匹配），原样交回 | 成立 |

一处没改的句子：`mounted_session.rs:2121` 的后半句「调用方分得出「先整理」与「真没空间」」在甲之后只剩一半：甲判出时 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease` 带 `Some(below_the_floor)`，合取 7 自己拒时是 `Publish`，调用方要读两个成员才分得出。这一句不在这两批改动里，条目在 T3-2。

### T1-2　`SlotPairCount` 换得干不干净

- 换了的：`DeviceFreeMap` 四个字段（`allocator.rs:415`、`:417`、`:424`、`:426` 的类型）、`pairs_inside_the_cluster_segments` 的入参与返回（`allocator.rs:511-515`）、`slot_pairs_without_an_unreleased_record_outside_the_cluster_segments`（`allocator.rs:800-806`）、`SlotsAFloorRaiseCouldRelease` 那一格（`mounted_session.rs:6056`）。
- 留着 `u64` 的：`unblocked_slot_pairs_outside_the_cluster_segments`（`allocator.rs:779-786`，返回 `u64`）。同一个量（槽对数）在同一个结构体的两个读数上两种表示。谁读它：`admission.rs:996` 装进公开字段 `slot_pairs_for_data_units: u64`（`admission.rs:980`），与数据单元个数比（一对一单元，同为 `u64`）；E180、E181 两份实验装置读那个公开字段；`allocator.rs`、`mounted_session.rs` 的单测与验证包用例。
- 混着比或混着加有没有：产品代码里没有一处。乙那一格先 `.0` 取出、乘一个数据单元的槽数再换字节（`mounted_session.rs:6090`），不与 `deferred`、`isolated`（槽数）相加；落得下那一判的 `u64` 只与单元个数比。两个读数只在测试里相遇，都用显式包装：`allocator.rs:3931-3933` 把 `u64` 读数包成 `SlotPairCount` 再与乙的上界比；验证包用例对乙的值取 `.0` 再与单元数比。换成类型的目的（槽数不与槽对数混加）在乙这一格已达到，没换的那个读数不削弱它。
- 辩方判：「留 `u64` 是主 agent 定的」站得住，不是该换没换。残留风险是 `pub u64` 字段的 `.0` 是个逃生口，仓里 `SlotNumber(pub u64)`、`CheckpointTxg(pub u64)`、`AvailableBytesOnOneDevice(pub i128)`（`address.rs:25`、`address.rs:33`、`admission.rs:706`）同一写法，不算新毛病。
- `.0` 取在哪几处（产品代码）：`allocator.rs:695-700`、`:910-915`（增减一对）、`:783`、`:805`（总数减段内和）、`mounted_session.rs:6090`；取出之后只做同类算术，没有与槽数相加相比。
- 溢出与下溢：`Sum` 用 `u64` 的 `sum()`（`allocator.rs:358-361`），根 `Cargo.toml:25` 的 `overflow-checks = true`，溢出会当场 panic、不回绕；槽对数不超过单元区槽数的一半，装得下。`.0 -= 1` 四处是原来的 `-= 1` 换写法，仍只在「这一对原来算着、现在不算」的状态翻转里减（`allocator.rs:693-701`、`:905-917` 的匹配），没有新的减法路径；段外数 = 总数减一组不重复的段内和的子集（`BTreeSet` 去重），不下溢。

### T1-3　两条真池用例钉的是不是它们名字说的那一形

- `a_publish_of_a_write_while_empty_cluster_segments_are_below_the_floor_is_refused_without_a_series`（`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1493`）：阳性对照删除队列空（被拒的是这次改动自己那一次发布），经 `publish_user_change` 单发，断言 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`、`devices_short` 空、合取 7 那一份与预期相同、一串没推、一版没发、F 不动、盘上逐字节不变、崩了再挂读得出旧内容。变异行 `crates/mutations.tsv:7700` 把 `publish_user_change` 那一处传成「免」，这条用例红在「照推一串」这一件上。钉的就是名字说的形。限度（Opus 腿 T3-2 已提）：这个池用只供测试的开关把 `N₀` 抬到比每块盘的全空段数多一段，全空段多、没有「全空段为 0 而产品 `N₀` 为 1」的回落形，所以钉的是接线，不是分布。
- `a_walk_step_refused_for_space_while_empty_cluster_segments_are_below_the_floor_is_pushed_for`（`crates/singlefs-verification/tests/accept_raises_the_floor_for_space.rs:1546`）：`mark_every_free_slot_allocated`（`:659`）把每个空闲槽记成已分配，每块盘全空段数按记录口径应为 0，`N₀` 取最大数加一，推出来是 1，**即产品值**（推的：用例没有断言这个数，待主 agent 在池上读一次才能升成量的）。期望 `NoSpaceAfterRaisingTheFloor` 只会从「先算没判出、推串尝试过」那一支出来（`mounted_session.rs:6317` 之后的 `Stopped` 分支），所以它钉的是「走空那一步先算不判出」。红的来源：变异行 `crates/mutations.tsv:7701`（走空传成前台写入）→ 甲判出 → 交回 `NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`，用例的 `let … else panic!` 红，是那一件。别的改坏也会让它红：让乙对提交内生块那一判也算上界的行 `crates/mutations.tsv:7472`（走空那一步被判成放不下）、走空那一步没被空间拒（用例 `panic!("单元区里一个空闲的槽都没有…")`）。会让它照样绿的：`of_the_user_change` 里除 `DataChangesPublishedTogether` 之外的前台变体传错（变异行 `:7697` 只点了这一个变体），走空用例本来就不过那个函数。这是覆盖限度，不是用例的毛病。

## T3　从未被看过的地方（读码；装置只走 `publish_user_change`，没有一格是量的）

### T3-1　`accept` 那一处传的 `change_of_the_refused_batch`

`accept` 只收数据写类：`ChangeAtTheAccept::Data { change, contract }`（`crates/singlefs-core/src/mounted_session.rs:7120`）；`WriteAtOffset` 与 `DataChangesPublishedTogether` 在这之前就拒（`mounted_session.rs:7112-7118`），屏障类走 `publish_a_barrier_alone`。所以 `change` 只会是整份写、建 inode 这几样，`of_the_user_change`（`mounted_session.rs:6154`，穷举、无 `_ =>`）都归「合取 7 判」。批并了几样时（`mounted_session.rs:7232`）：判单发的取 `change` 本身，判「待发加这一次」的取并成的 `DataChangesPublishedTogether`，同样归「合取 7 判」；并成的这一批至少含这一次改动（`together_with_this_change`），不会是空批。合取 7 对 `accept` 没有免的路径：它判前台写入的条件是「if writes_in_the_foreground && !is_a_rollback && relocations.is_empty() {」（`crates/singlefs-core/src/transaction.rs:13367`），`accept` 的批带文件写或新 inode 记录、不是回退、没有搬迁。结论：分类对，不会因并了几样而错，也不会把合取 7 免的批当成判的。限度：装置没有 `accept` 臂，这一条只靠读码与第 128 批 `accept` 真池用例。

### T3-2　甲判出时 `devices_short` 为空，读它的调用方拿到空表会不会走错支

- `devices_short`、`empty_cluster_segments_below_the_floor` 两个字段，非测试代码里在 `crates/singlefs-core/src/mounted_session.rs` 之外没有任何读它们的地方（`grep -rn 'devices_short' crates --include=*.rs` 在核心层之外只命中验证包与运行库的测试）。验证包模型 `crates/singlefs-verification/src/history.rs:5446`、`:5599`、`:5644` 与实验装置 `crates/singlefs-verification/src/bin/e158_root_choice_repair.rs:19902`、`crates/singlefs-verification/src/bin/e180_admission_reading_versus_writable_bytes.rs:642` 匹配这个成员，只读 `floor_raises`、`versions_published`、`last_refusal`，不看这两个字段；甲判出时 `last_refusal` 仍是空间不够那一种，模型归类不变。
- 测试帮手 `assert_refused_with_nothing_pushed`（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs:1013`）断言 `!no_space.devices_short.is_empty()`（`:1026`）：只适用乙的形。它的两处调用（`:1077`、`:1131`）放的是「比单元区还大的写」，上界必小于需求，乙先判出，甲不跑；先乙后甲的次序恰好保着这个帮手。以后有人把甲的形（全空段为 0 而上界够用）接进这个帮手会假红，这是帮手的限度，不是产品的毛病。
- 口径裂缝：「全空段少于 `N₀`」现有两条报法（合取 7 自己拒：`Publish` 带 `EmptyClusterSegmentsBelowTheFloor`；甲判出：`NoSpaceEven…` 带 `Some`）。今天没有产品调用方读这个差别，所以没有走错的支；T1-1 末尾那句「调用方分得出」是它的文字影子。判决第一轮已记，不另判。

### T3-3　`DeviceFreeMap` 克隆与重建

- 克隆：`DeviceFreeMap` 派生 `Clone`（`crates/singlefs-core/src/allocator.rs:365`），新加的三样（逐槽位图 `is_under_an_unreleased_record`、每段计数、总数）都是结构体字段，一起克隆；探针里 `copy_of_the_session` 整份克隆分配器（`research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/probe/src/pool.rs:70`），O 臂、重挂臂在拷贝上与产品臂得同形结局，旁证没有分叉（间接）。
- 重建：重开与管理员回退都走 `PoolAllocator::rebuild_from_records`（`crates/singlefs-core/src/allocator.rs:2522`）：`Self::new(devices)` 起步（`:2526`，三个数从全空重算：`allocator.rs:598-620` 一带的初值），再对每条记录调 `mark_allocated`、已释放的再调 `mark_released`，所以三个数是从记录经同一条增量路径重算的，不沿用旧值。
- 限度（推的）：`allocator.rs` 里钉增量与逐槽扫描相等的两条用例（`:4035`、`:4055`）都从 `PoolAllocator::new` 起步走随机历史，没有一条在 `rebuild_from_records` 重建之后核这三个数；重建路径的正确性靠「走同一条入口」而不是靠断言。这不判违规，补一条「随机历史之后按记录重建、三个数与增量相同」的小用例就够，交主 agent 定。

### T3-4　`SlotPairCount` 的 `Sum` 与算术的新路径

见 T1-2 末条：没有新的溢出或下溢路径，`overflow-checks = true`（`Cargo.toml:25`）下破了不变量会当场 panic。

## 检查单八格（修补后）

| 格 | 判 | 证据 |
|---|---|---|
| 走不到的理由二次查实 | **成立** | 两批加号行里新的 `assert!`、`expect`、`unreachable!`、`todo!`、`panic!` 只在测试代码里（第 128 批：库单测改写的一条 panic 文字，验证包用例里的断言、`.expect("有树")`、`.expect("池里有盘")`；第 129 批的加号行一条都没有），产品代码一条都没加（对 `research/prompts/_m4-gpugc-precheck-b-a-code-r2-diff.md` 按文件分段、只看加号行，用 awk 过滤 `assert`、`expect(`、`unreachable`、`todo!`、`panic!`、`unwrap(` 现查，命中 12 行，全在 `mounted_session.rs` 的测试模块与验证包用例里）。减法：四条 `.0 -= 1`（`allocator.rs:696`、`:911` 各一对）是原来的 `-= 1` 换写法，守卫不变（T1-2）；实现员第 128 批报告写「`crates/<包>/src/` 一行没动」与事实不符（改了注释与一条库单测的 panic 文字），不影响结论 |
| 类型与名字 | **成立** | `SlotPairCount` 无缩写，`pub` 字段与仓里 `SlotNumber(pub u64)` 同写法（T1-2）；`unblocked_slot_pairs_outside_the_cluster_segments` 留 `u64` 不算「同一个量不同表示」被混用（产品里两个读数不相遇）；新名字（用例名与帮手名）都是整词 |
| 不兼容旧代码与旧格式 | **成立** | 两批没碰盘上字节，没有版本分支或旧布局常量；变异行七行改名加后缀，旧名在 `crates/mutations.tsv` 里已删（按第一列整串比，七个旧名都没有同名行，awk 现查；`.claude/`、`scripts/`、`task-server/` 下没有文字点名它们，grep 现查） |
| 不新建判定库 | **成立** | 加号行没有新目录、TSV（变异表是行登记不是判定存放处）、sqlite、RocksDB |
| 所有任务多机可用 | **成立（限读到的加号行）** | 新用例在内存池上现造，不读环境、路径、主机名；池脚手架 `common_admission_pool` 内部本腿没读 |
| 测试优先多线程 | **成立** | 两条伪随机历史各一个定种子的 600 步（`crates/singlefs-core/src/allocator.rs:3955`），次序是被测对象（理由已写：`allocator.rs:4033`：「/// 不并行：每一步读上一步留下的分配器状态，操作的次序就是被测对象。」，`:4053` 同句）；cargo 测试框架本来就把各个 `#[test]` 并行跑。理由对一个种子成立；以后要多种子再按种子切片。真池两条是单场景 |
| 带 --item 的命令与项 | **成立** | 这两批没有新的崩溃点重放项与形式化项；变异行共 9 行（第 128 批 `crates/mutations.tsv:7700-7701` 两行，第 129 批改写 `:7707-7713` 七行），改写的旧名没有残留 |
| 新崩溃流多罩了什么 | **成立（不需要新流）** | 乙甲头一次就判出时一个写都不发，崩溃状态与上一次发布落盘之后相同；两条真池用例已钉盘上逐字节不变与崩了再挂读得出旧内容；走空那一形的写由已有的 unlink 流罩着（推的，没逐流比） |

## 辩得住与辩不住（给主 agent 判决时对着看）

| 项 | 结果 |
|---|---|
| 「打中：甲的前提不成立」 | 辩得住一半。站得住的是「注释的推理有缺口」（回收不改这个数，推不出推串不改这个数），证据是 `series_*_raised` 的存在性；够不着的是「前提为假」（`crossed_the_floor` 分母没报）；**不分辨臂**：同一前提在乙甲之前就是 `refusal_is_short_of_space` 合取 7 臂的依据，产品臂头一次被合取 7 自己拒的 413 格本来就不推。判决第三节第 1 条的处置（文档改成「可能」「不另估」）对，措辞不过头 |
| 「打中：拒绝报里少卸载重挂出路」 | 辩得住一半。12/1907、2/4 是实数，登 C1003、零轮交用户的处置对；够不着「乙甲带来」：先算这一类报没有出路字段、C1003 在乙甲之前已登，式子那一判 0/11514、现行先算 0/177 走得通。真正没量的是「乙拿掉了现行本来会报的出路」，只有 A/B 那一族有过（4801 格，0/295 走得通） |
| 「乙误拒这一份取样没撞上」 | 站得住；「底 19351 有区分力」应改口径；有效分母装置没拆 |
| 「甲误拒分母太小」 | 站得住（4/56681，格间不独立） |
| 「先乙后甲不打中」引的两个 0 | **辩不住**：那两个 0 恒为 0，不是「两样同时成立」的次数 |
| 「越过 `N₀` 的 0 串，甲误拒 0 格」并排写 | **辩不住**：两个 0 的分母一个没报、一个是 4，写成并排会被读成比率 |
| 射程宽一形、改 size 与建快照不先判、两处调用覆盖、位图克隆 | 站得住 |

## 模型

模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-model/`：在攻方模型 `research/prompts/m4-gpugc-precheck-b-a-code-r1-opus-model/` 上改，`probe/src/pool.rs`、`probe/src/device.rs`、`probe/src/main.rs`、`probe/Cargo.toml` 与攻方模型逐字节相同。没编、没跑（子 agent 不交任务、不起 cargo）；对三个改动的 Rust 文件跑过 `rustfmt --check`（只解析，没有语法错误，退出 1 只是排版差异），类型与借用没过编译器；`patch_the_copy.py` 的五处原文在冻结副本里各恰好命中 1 次（量的，本腿对冻结副本 `mounted_session.rs` 数过）。`bash -n` 两个脚本、`python3 -m py_compile` 补丁脚本都过。

| 文件 | sha256（`sha256sum` 现算） |
|---|---|
| `probe/Cargo.toml` | `879caa27846b1df02d9b5d96b28db0deb03c1a47f99e518be4a7a0a754bd1bb5` |
| `probe/core_probe_module.rs` | `a70215ff93d16f77a5e39b14cfd3c40989adb56b12675a55bfde016b4cc2fef9` |
| `probe/src/device.rs` | `81a8ae9934428a5a09ec1f59cc0f0567e49eb41aa7135ab584d9f956338959a3` |
| `probe/src/main.rs` | `358477d4da8f03a6d977eeb816cd96c8373be51bb97fd92d36304f422b6c3e74` |
| `probe/src/pool.rs` | `e4125643126be91539e6a445988eb0719c53d82f89c8429260d2dc3ddeb7b4ac` |
| `probe/src/shapes.rs` | `294999a0ee99723e688cb77e674a1a24a881ba6d1b0017c07d6928b6020d4f48` |
| `probe/src/tally.rs` | `c3485b63ff0310507a3f6123d2b45762b4e2489f4843f60586b70f259e8031f4` |
| `tools/patch_the_copy.py` | `a62fe5174a9149a6fb7069cc50cc9ba8a42254dd94afa0c8fb9676ba4f7eca82` |
| `tools/rerun.sh` | `5786b5aa93dc822e2a9e5d4df6fc88557f78ee845d8b22206270a50dba141d29` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

做的事（对攻方模型的增量，全部在拷贝里）：`patch_the_copy.py` 在原来三处钩子之外加两处：`refusal_is_short_of_space` 开头一道线程开关（开着时合取 7 自己的被拒当成空间不够，强推臂用）；`SlotsAFloorRaiseCouldRelease::of_the_device` 里槽对上界读数一道线程开关（开着时照 `crates/mutations.tsv:7707` 那一行的原文「每个放得开的槽各补一对」，老上界臂用）。`shapes.rs` 加：每个触发拆出的计数键（O 臂推了几串、判出前隔离着槽否、O 臂收场被宽护卡住否、判出时全空段数是否已少于 `N₀`、判出格落在几个配置、`series arm=* short-before`）；强推臂（产品臂头一次被合取 7 自己拒时）；老上界臂（槽对判出的格上）。原有计数键一个没动，同一批种子（`--seed-base 1 --configurations 310 --steps 400`）下原有 30 个读数应与 oracle.out 的 `PROBE_RESULT` 逐字段相同，可作自证。

## 要主 agent 跑的命令

一条（在仓根 `/home/fy5090/code/singlefs` 起，不给 `--cwd`）：

task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-sonnet-probe-run /home/fy5090/code/singlefs/research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-model

- 包装那一层先删运行目录 `…/m4-gpugc-precheck-b-a-code-r2-sonnet-probe-run`（只认工作根下、名字以 `-run` 收尾的，别的退 2），再调同一个模型目录里的 `tools/rerun.sh`；服务撞内存上限重排同一条命令时可安全重跑。
- 判过：末行 `TASK_SERVER_RESULT` 是 `type=succeeded exit=0`；日志里依次有 `PATCH_THE_COPY … probe_sources=5`、`name=build done binary=…`、`name=exit_code oracle=0`、`oracle ESTIMATE …`、`oracle WALL_SECONDS …`、`oracle PROBE_RESULT configurations=310 …`、四行 `name=self_check ok …`、`name=self_checks bad=0`、末行 `name=rerun_done …`；有 `✗` 就是没过。退 1 且日志是 `build.log` 末 80 行：编不过，新派同立场的腿照报错改 `probe/`；`name=exit_code oracle=3`：估时超过 40 分钟，缩 `rerun.sh` 里的 `--configurations`（新派腿改）。
- 内存与时间：时间推的，没量过本模型；攻方模型同一批种子的复跑 `oracle WALL_SECONDS 605.7`、`ESTIMATE threads=31`（量的，oracle.out），本模型多出的两臂在约 1907 + 413 次改动上各多发一次，相对 145155 次改动加 13425 次 O 臂与 13425 次重挂臂不到一成（推的）；内存没量过，攻方模型的估计是 4 至 10 GiB（推的，攻方腿报告）。每个配置跑完往标准错误打 `PROGRESS`（沿用攻方模型）。
- 读法（读 `oracle PROBE_RESULT` 与 `oracle COUNT` 行；整份输出在运行目录 `out/oracle.out`）：
  1. 自证：原有 30 个字段与 oracle.out 的 `PROBE_RESULT` 逐字段相同，不同就是新臂污染了原臂，整份作废。
  2. `fired by=slot-pairs oracle-series=0/1/2/3`（3 是三串及以上）：`misrefused_slot_pairs=0` 的有效分母 = 1、2、3 三行之和。`configs fired by=slot-pairs` 是格落在几个配置。
  3. `fired by=slot-pairs isolated-before=some`、`oracle-wide-protection=stuck`、`remount published by=slot-pairs oracle-wide-protection=stuck`：预期都是 0（推的）；不是 0，T2-4 第 2 点的预期被推翻。
  4. `fired by=slot-pairs old-bound=…` 与 `remount published by=slot-pairs old-bound=…`：十二格重挂写进的格上，老上界同样判出（`fires-by-slot-pairs`）、还是推了串（`pushed-then-no-space-wide-protection-not/stuck`）；`published` 的格数应为 0（与 O 臂同义）。
  5. `fired by=slot-pairs count-below-floor=yes`、`fired by=formula count-below-floor=yes`：乙判出时全空段数已少于 `N₀` 的格数，即「两样同时成立」的真次数。
  6. `series arm=product short-before`、`series arm=oracle short-before` 与各自的 `crossed-the-floor`、`short-before empty-segments=raised`：`crossed_the_floor` 的分母。
  7. `conjunct-first refused`（应是 413，算的）、`conjunct-first forced=published`、`conjunct-first forced-series=…`、`series arm=forced short-before` 与 `crossed-the-floor`：合取 7 自己拒的格上推串写不写得进；这是现行形态上那条前提的直接量法。
- 读数出来之前这一节所有「推的，待主 agent 跑」维持原状；跑出来后新派同立场的腿，派发提示带「承接：research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-output.md」与任务号，读 `task-server/task log <号> --all` 与上面那份输出。运行目录里 `copy/`（冻结副本的拷贝，服务没给绝对的编译目录时连编译目录）用完由主 agent 删。

## 什么现象会推翻这份报告里的辩护

- 「甲前提的打中不分辨臂」：强推臂里 `conjunct-first forced=published` 占 413 格相当比例 ⇒ 合取 7 自己拒的格上推串写得进，现行「不推」本身就是误拒，打中仍不分辨臂、但份量远大于第一轮判决估的（这时要另立一笔账，不再是改一段文字）。
- 「出路缺口不是乙甲带来的」：那 12 格里 `oracle-wide-protection=stuck` 大于 0，或老上界一臂在这些格上推了串、报被宽护卡住并列出路 ⇒ 现行会报、乙拿掉了，是乙带来的。
- 「乙误拒这一份取样没撞上」：`oracle-series` 的 1、2、3 三行之和很小（例如不到一百）⇒ 有效分母太小，欠账要写成「槽对那一判实际没被这一族测过」。
- 「甲方向对」（推串让全空段数变少是变多的 285 倍）：`series arm=oracle short-before empty-segments=raised` 占 `short-before` 的比例高 ⇒ 在甲该管的状态上推串常常把数推高，甲的依据反过来。

## 没做什么

- 没往统一任务服务交任何任务、没起 cargo、没编模型、没跑任何门禁与自证；上面所有「推的」的读数、新模型的任何读数都等主 agent 跑那一条命令。判决要引的数由主 agent 复跑。
- 没复核 A/B 任务 13802 的日志（正推腿报告里转述的 `outlet_fewer_than_as_coded=4801`、`lost_first_followed=295`）；没复核任务 13926 以外的任何探针读数。
- 没补「阳性对照」：注入一个已知偏紧的槽对上界、看 O 臂是否报出误拒，这一格模型里没有。推荐补法（零轮，推的）：对产品臂推过一串后写进的格（`base_published_after_a_series`，19351）逐格再发一次，把槽对上界减一对，数多少格因此被判出；这个数既是装置对「上界偏紧」的灵敏度，也是乙的安全余量。
- 没读 `common_admission_pool` 脚手架内部；没读 `crates/singlefs-core/src/mount.rs` 里抬 F 那一串自己的取落点逻辑；没核 `accept` 臂（装置只走 `publish_user_change`）；没数第 128 批两条用例各自在变异下红的真实退出（只读了 `crates/mutations.tsv` 的行与用例的断言，红不红是主 agent 任务 13955、14105、14167、14187 的读数，本腿没复核）。
- 没改任何仓里的文件：只写了这份报告与模型目录 `research/prompts/m4-gpugc-precheck-b-a-code-r2-sonnet-model/`；草稿目录 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-precheck-b-a-code-r2-sonnet` 里只有认领留下的 `.owner`、写者锁与改模型用的几个小脚本和规格文件（`du -sh` 为 68K），没有仓副本与编译目录，没有要删的。
