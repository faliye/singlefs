# 阶段同步第二段逐行判（组 F39、F40、F41、F44、F45；batch-2026-10-21）

候选表 `/home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv`，sha256 a2913e391a222ec455f068bb72d7123c94659be2da960181125ecf2bc132e6d9（开工时核过）。基准 1bb3a24dc，结束工作区。

## 数行
- 取出分到的行：`awk -F'\t' 'NR==1||$1~/^F(39|40|41|44|45)$/' candidates.tsv > mine.tsv`，数行 `wc -l`：202（含表头），即 201 行候选。F39=40、F40=21、F41=79、F44=27、F45=34。
- 派发提示写 F39-F45；F42、F43 是新立行（旧事实写「新立：」），候选表里没有行，`--check-report` 对 `--groups F39-F45` 退 2（「--groups 里有候选表没有的组：F42、F43」），所以实际跑的是 `--groups F39,F40,F41,F44,F45`。
- 判定分布（`grep -o '| \(要改\|要补\|事件句不改\|不相干\|要人看\) |' 逐行判定表 | sort | uniq -c`）：不相干 100、事件句不改 52、要改 49、要补 2（M1、M2，候选之外加的）、要人看 0。
- 分组要改：F39=6、F40=6、F41=27、F44=9、F45=1。

## 要点（主 agent 先看这几条）
1. **F40 的新事实本身写错了一半**：事实表新事实写「产品路径上没有调用者」，现查不真。`grep -rn "discard_at(" crates --include=*.rs` 命中 `crates/singlefs-core/src/mounted_session.rs:2465`，调用它的 `discard_the_segments_reusable_as_a_whole` 在同一文件 1921 行（用户改动的发布之后）与 1995 行（整理显式入口之后）被调，0b926bd3f 进仓（基准 1bb3a24dc 上 `discard_at` 在该文件 0 处）。所以 D26 已定项 5 射程句（184 行三语）里「产品路径上没有调用者」与第一版取法表「产品路径的调用者｜还没有」（296 行三语）要改，不是只加「已定」；kb 里没有一处记着这个函数（M1、M2）。同一行 D26:184 前半「没有整理的触发、发布入口」另归 F33。
2. **F45 C821 现查已不开**：`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl` 在（0b926bd3f），`crash_judge_conclusions.rs:438` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是 `&[]`，`tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs` 在，`grep -c "^C821" crates/mutations.tsv` = 24，`invariants/01-块自描述.md` 的 I-1.13 行已写「扫描内核的第三个入口 scan_rebuild_main 照判、置位」。checks-owed.md:536 的 C821 行（题目与现象列）还是旧的，判要改。这一次没跑对拍与变异（checker 档重型），所以只能写「已做、复跑归提交时」，移不移进已还清表由主 agent 定。
3. **C364 的字段表已定**：D26 已定项 2 有字段表与四个 format-const 标记，`crates/singlefs-core/src/logged_operations.rs` 在，`transaction.rs:12690` 起用 `logged_operations_root_entries` 写条目（基准上 0 处）。仍说「字段表未定」「没有写者」「仍欠实现」的有：D5:232、D8:110/265、freeze-layer-membership.md:33/106、milestone/04:59/64/191（三语）、checks-owed.md:267（三语）。账 C364 本身仍开着（I-4.4 的判法），但账的题目「整理意图记录没有字段表」已与现状不合，全仓十几处用它当简称，是否改题交主 agent（我判这些简称行为「不相干」）。
4. **F39 的 N₀**：旧值「第一版取 0」只剩在 `layout/01-first-txn`（三语 309/310 行）和 `milestone/04-fourth-txn` 59 行（三语）当作 D3 的逐字引文，判要改；D3、D26 本身已是新说法。
5. **F44**：D13:445 的「欠」行还列 C668、D19:175 还列 C296、D22:392 还有 C403 整行（自己写着「销账后此行删」）、checks-owed.md:689 写 C61「待提交落地销账」、milestone/03 的 72/107/124 行名单里 C296、C568、C572、C576 没标已还清；这些账都在已还清表（checks-owed.md 852-862 行）。
6. **三语译文落后于中文原文**（不在候选里的新发现）：checks-owed.en.md 与 .ja.md 各 861 行，中文 867 行；C296、C576、C587、C668、C827 在译文里仍在开着的区段（en:228/374/377/439/546），en 的 C827 行自己写着「Paid off」却还在开着的表里。F44 的 ja:374（C576）、ja:688（C41）按此判要改。候选正则写的是全角括号，所以 en 的 checks-owed.en.md:688（C41 同一句）、D13 / D19 / D22 的 en 与 ja 版（C668、C296、C403 各 1 处）、checks-owed.en/ja:540（C821）没进候选表，同样要改，我没逐行判。
7. 事实表 F46 一类里，我顺带看到 milestone/04:59 同一格还有「E10 没有跑前登记」（F46）、「COMPACTION_WATERMARK … 恒 0」等，这些不归我的组，没判。

## 没做什么
- 没改仓里任何文件；没跑重型测试（C821 的对拍与变异、C576 等的用例都没跑，凭代码与登记行判）。
- F42、F43 无候选行，没判。
- 「反向问」（第 10 步）只覆盖我的五组涉及的事：加了 M1、M2 两行要补；别组的事（F46 之后）没问。
- 候选表的「原文」列只有行首约 500 字，行尾超出的部分我用 `ctx.py`（草稿目录里的辅助脚本）读文件原行补看过；译文行（.en/.ja）的判法照中文原文，没有逐句核译文的措辞是否与中文逐字一致。
- 判成「不相干」的 100 行里有一批是「已是新说法」（如 D3 已定项 7 第 7 行），不是与检索词无关；这一格没有「已是新说法」的取值，照五种之一用了「不相干」并在理由里写明。
- 推翻判定的现象：上面第 1、2 条若 `mounted_session.rs` 那两处调用在提交前被删，或 C821 的对拍测试现在跑红，第 1、2 条要重判。

## 逐行判定
| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F39 | .claude/kb/checks-owed.en.md:116 | 不相干 | C139 行里的 N₀ 只是第二个合取式的符号写法，没写取多少，不受 N₀ 第一版改取 1 影响（原行：「**D3 (Space Allocation)」） |
| F39 | .claude/kb/checks-owed.en.md:539 | 不相干 | C820 行就是这件事实的欠账（N₀ 的算式没有计数模型），与 D3 已定项 7 现写的「第一版取 1，算式等计数模型验过再定」一致，仍是真话（原行：「底额 N₀ 的算式没有计数模型」） |
| F39 | .claude/kb/checks-owed.ja.md:116 | 不相干 | C139 行里的 N₀ 只是第二个合取式的符号写法，没写取多少，不受 N₀ 第一版改取 1 影响（原行：「**D3（空間割り当て） 那条立项原则「解放空间」） |
| F39 | .claude/kb/checks-owed.ja.md:539 | 不相干 | C820 行就是这件事实的欠账（N₀ 的算式没有计数模型），与 D3 已定项 7 现写的「第一版取 1，算式等计数模型验过再定」一致，仍是真话（原行：「底额 N₀ 的算式没有计数模型」） |
| F39 | .claude/kb/checks-owed.md:116 | 不相干 | C139 行里的 N₀ 只是第二个合取式的符号写法，没写取多少，不受 N₀ 第一版改取 1 影响（原行：「**D3（空间分配） 那条立项原则「释放空间这个」） |
| F39 | .claude/kb/checks-owed.md:535 | 不相干 | C820 行就是这件事实的欠账（N₀ 的算式没有计数模型），与 D3 已定项 7 现写的「第一版取 1，算式等计数模型验过再定」一致，仍是真话（原行：「底额 N₀ 的算式没有计数模型」） |
| F39 | .claude/kb/decisions/03-space-allocation.en.md:148 | 不相干 | D3 已定项 7 登记表第 7 行本身已是新说法（N₀ 第一版取 1、算式、只拦前台写入类发布）（原行：「t ≥ N₀」） |
| F39 | .claude/kb/decisions/03-space-allocation.en.md:159 | 不相干 | 判决回看行已写「N₀ 的算式先建计数模型验，验之前取 1」，是新说法，不含旧值 0（原行：「t; N₀ formula verified」） |
| F39 | .claude/kb/decisions/03-領域割り当て.ja.md:148 | 不相干 | D3 已定项 7 登记表第 7 行本身已是新说法（N₀ 第一版取 1、算式、只拦前台写入类发布）（原行：「数 ≥ N₀」） |
| F39 | .claude/kb/decisions/03-領域割り当て.ja.md:159 | 不相干 | 判决回看行已写「N₀ 的算式先建计数模型验，验之前取 1」，是新说法，不含旧值 0（原行：「状態；N₀ の算式は計数モデルで先に検証し、」） |
| F39 | .claude/kb/decisions/03-空间分配.md:148 | 不相干 | D3 已定项 7 登记表第 7 行本身已是新说法（N₀ 第一版取 1、算式、只拦前台写入类发布）（原行：「数 ≥ N₀」） |
| F39 | .claude/kb/decisions/03-空间分配.md:159 | 不相干 | 判决回看行已写「N₀ 的算式先建计数模型验，验之前取 1」，是新说法，不含旧值 0（原行：「态；N₀ 的算式先建计数模型验，验之前取」） |
| F39 | .claude/kb/decisions/14-dual-track-large-small-files-persistent-transient.en.md:38 | 不相干 | 这是 E151 第四、五次跑的实测线索（家固定的目录提示、uniform 上留 24 个尾部段回落 0），没有写整理底额 N₀ 第一版取几，不受取 1 影响（原行：「ain N₀" registered as me」） |
| F39 | .claude/kb/decisions/14-大小ファイル・永続-一時デュアルトラック.ja.md:38 | 不相干 | 这是 E151 第四、五次跑的实测线索（家固定的目录提示、uniform 上留 24 个尾部段回落 0），没有写整理底额 N₀ 第一版取几，不受取 1 影响（原行：「定 + N₀留保」は候補とせず、測定済み手がかり」） |
| F39 | .claude/kb/decisions/14-双轨大小文件-持久临时.md:38 | 不相干 | 这是 E151 第四、五次跑的实测线索（家固定的目录提示、uniform 上留 24 个尾部段回落 0），没有写整理底额 N₀ 第一版取几，不受取 1 影响（原行：「+ 留 N₀」不作候选，作已量过的线索登记**：」） |
| F39 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:92 | 不相干 | D26 已定项 3 写「N₀ 取能让棘轮不发散的最小值，数值等 E10」；D3 现写算式等计数模型验过再定（欠 C820），数值仍在等 E10，仍是真话（原行：「t ≥ N₀). **Floor quota」） |
| F39 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:96 | 不相干 | 这是已定项 3 的射程句，只说「不管 N₀ 取多少」，没写 N₀ 的值（原行：「of N₀. Which term of a」） |
| F39 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:283 | 不相干 | 这是读法推论，只把 N₀ 当第二合取式的符号用，没写取值（原行：「t ≥ N₀) the second conj」） |
| F39 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:92 | 不相干 | D26 已定项 3 写「N₀ 取能让棘轮不发散的最小值，数值等 E10」；D3 现写算式等计数模型验过再定（欠 C820），数值仍在等 E10，仍是真话（原行：「数 ≥ N₀) とする。**下限枠は不等式のい」） |
| F39 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:96 | 不相干 | 这是已定项 3 的射程句，只说「不管 N₀ 取多少」，没写 N₀ 的值（原行：「管轄；N₀ の値は管轄外。下限枠自身が占める」） |
| F39 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:283 | 不相干 | 这是读法推论，只把 N₀ 当第二合取式的符号用，没写取值（原行：「数 ≥ N₀) を割り当て受入の第2連言節とし」） |
| F39 | .claude/kb/decisions/26-后台整理与放置回收.md:92 | 不相干 | D26 已定项 3 写「N₀ 取能让棘轮不发散的最小值，数值等 E10」；D3 现写算式等计数模型验过再定（欠 C820），数值仍在等 E10，仍是真话（原行：「数 ≥ N₀)。**底额进不了不等式的任何一项」） |
| F39 | .claude/kb/decisions/26-后台整理与放置回收.md:96 | 不相干 | 这是已定项 3 的射程句，只说「不管 N₀ 取多少」，没写 N₀ 的值（原行：「不管 N₀ 取多少。底额自己占的空间算准入不」） |
| F39 | .claude/kb/decisions/26-后台整理与放置回收.md:284 | 不相干 | 这是读法推论，只把 N₀ 当第二合取式的符号用，没写取值（原行：「数 ≥ N₀) 做成了分配准入的第二个合取式；」） |
| F39 | .claude/kb/layout/01-first-txn.en.md:309 | 要改 | 原：「7, first version N₀ =」；改为：准入行改为：合取式只登记在 D3（空间分配） 已定项 7，第一版 `N₀` = 1（算式等计数模型验过再定，欠 C820）。旧句「第一版 `N₀` = 0」已与 D3 已定项 7 现写的取 1 相反；「新池新建文件恒过」一格不必改（新池空段远多于 1，需主 agent 核一眼） |
| F39 | .claude/kb/layout/01-first-txn.ja.md:310 | 要改 | 原：「1版 N₀ = 0」；改为：译文照中文改后的句子译：合取式只登记在 D3 確定項目 7，第 1 版 `N₀` = 1（算式は計数モデルの検証後に確定、C820 で未済）。旧句は 0 のままで D3 と矛盾 |
| F39 | .claude/kb/layout/01-first-txn.md:310 | 要改 | 原：「一版 N₀ = 0」；改为：准入行改为：合取式只登记在 D3（空间分配） 已定项 7，第一版 `N₀` = 1（算式等计数模型验过再定，欠 C820）。旧句「第一版 `N₀` = 0」与 D3 已定项 7 现写的取 1 相反；「新池新建文件恒过」一格不必改（新池空段远多于 1，需主 agent 核一眼） |
| F39 | .claude/kb/milestone/04-fourth-txn.en.md:18 | 不相干 | 这是里程碑四出口项，要的是 N₀ 重开并定值；D3 现在只是暂取 1、终值等计数模型验过（C820 仍开着），「定值」仍是没做完的出口项，句子没写旧值 0，不受影响（想把「重开」改成「验过后定终值」属润色，不必）（原行：「d; N₀ in row 7 of the」） |
| F39 | .claude/kb/milestone/04-fourth-txn.en.md:59 | 要改 | 原：「- **Existing in reposito」；改为：把「D3 已定项 7 登记表第 7 行逐字「`N₀` takes 0 in first version（整理未实现，整理进里程碑时重开）」」改成逐字「`N₀` takes 1 in version 1，算式 ⌈(3S − 1) × c ÷ 64⌉ + 1 等计数模型验过再定」（欠 C820）；同一句里的 discard 零命中、没有写者等另归 F40、F41、F33 判 |
| F39 | .claude/kb/milestone/04-fourth-txn.en.md:64 | 不相干 | 二·1 设计轮列的是「三条水位与 N₀ 的取值（E10 跑前登记）」，N₀ 终值仍等计数模型（C820 开着），是待办不是旧值，仍是真话（原行：「nd N₀ (pre-run registr」） |
| F39 | .claude/kb/milestone/04-fourth-txn.ja.md:18 | 不相干 | 这是里程碑四出口项，要的是 N₀ 重开并定值；D3 现在只是暂取 1、终值等计数模型验过（C820 仍开着），「定值」仍是没做完的出口项，句子没写旧值 0，不受影响（想把「重开」改成「验过后定终值」属润色，不必）（原行：「行の N₀ の再開と値決定；スナップショット」） |
| F39 | .claude/kb/milestone/04-fourth-txn.ja.md:59 | 要改 | 原：「- **リポジトリ内の既存要素**：D26 (バ」；改为：译文照中文改后的句子译：登记表第 7 行逐字「`N₀` は第 1 版で 1、算式 ⌈(3S − 1) × c ÷ 64⌉ + 1 は計数モデルの検証後に確定」（欠 C820）。旧句「第一版で 0 を採用」已不是 D3 现文 |
| F39 | .claude/kb/milestone/04-fourth-txn.ja.md:64 | 不相干 | 二·1 设计轮列的是「三条水位与 N₀ 的取值（E10 跑前登记）」，N₀ 终值仍等计数模型（C820 开着），是待办不是旧值，仍是真话（原行：「位と N₀ の値決定（E10 (エージング：」） |
| F39 | .claude/kb/milestone/04-fourth-txn.md:18 | 不相干 | 这是里程碑四出口项，要的是 N₀ 重开并定值；D3 现在只是暂取 1、终值等计数模型验过（C820 仍开着），「定值」仍是没做完的出口项，句子没写旧值 0，不受影响（想把「重开」改成「验过后定终值」属润色，不必）（原行：「行的 N₀ 重开并定值；能搬被快照共享的块；」） |
| F39 | .claude/kb/milestone/04-fourth-txn.md:59 | 要改 | 原：「N₀ 第一版取 0（整理未实现，整理进里程碑时」；改为：把「D3（空间分配） 已定项 7 登记表第 7 行逐字「`N₀` 第一版取 0（整理未实现，整理进里程碑时重开）」」改成逐字「`N₀` 第一版取 1，算式 ⌈(3S − 1) × c ÷ 64⌉ + 1 等计数模型验过再定」（欠 C820）；同一格里「discard 零命中」「没有写者」另归 F40、F41、F33 判 |
| F39 | .claude/kb/milestone/04-fourth-txn.md:64 | 不相干 | 二·1 设计轮列的是「三条水位与 N₀ 的取值（E10 跑前登记）」，N₀ 终值仍等计数模型（C820 开着），是待办不是旧值，仍是真话（原行：「位与 N₀ 的取值（E10（老化：结构抗老化」） |
| F39 | records/2026-09-13-总审核.md:64 | 事件句不改 | 这是 2026-09 至 10 月那一天的审核或派发或交回记录（问题清单、「第一版 N₀ = 0」当时的处置、欠账写回、`N₀` 压成 ForcedForTests(0) 的用例），说的是那一次发生的事，换成新说法就不是原来发生的事（原行：「数 ≥ N₀)、里程碑的「不在开放段」四处不同」） |
| F39 | records/2026-09-13-总审核.md:254 | 事件句不改 | 这是 2026-09 至 10 月那一天的审核或派发或交回记录（问题清单、「第一版 N₀ = 0」当时的处置、欠账写回、`N₀` 压成 ForcedForTests(0) 的用例），说的是那一次发生的事，换成新说法就不是原来发生的事（原行：「数 ≥ N₀) 第一版 N₀ = 0（整理未实」） |
| F39 | records/2026-10-13-里程碑四预开发.md:66 | 事件句不改 | 这是 2026-09 至 10 月那一天的审核或派发或交回记录（问题清单、「第一版 N₀ = 0」当时的处置、欠账写回、`N₀` 压成 ForcedForTests(0) 的用例），说的是那一次发生的事，换成新说法就不是原来发生的事（原行：「九行（N₀ 计数模型、GPU 扫描重建内核、」） |
| F39 | records/2026-10-13-里程碑四预开发.md:204 | 事件句不改 | 这是 2026-09 至 10 月那一天的审核或派发或交回记录（问题清单、「第一版 N₀ = 0」当时的处置、欠账写回、`N₀` 压成 ForcedForTests(0) 的用例），说的是那一次发生的事，换成新说法就不是原来发生的事（原行：「开关、N₀ 压成 ForcedForTes」） |
| F40 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:184 | 要改 | 原：「**Scope**: Governs wheth」；改为：把「块设备 trait 有 `discard_at`……产品路径上没有调用者（`discard_at(` 只在 `block_device.rs` 的单测、harness 的录制器与故障注入设备转发里命中）」改成：块设备 trait 有 `discard_at`（`crates/singlefs-core/src/block_device.rs`），产品路径上由 `crates/singlefs-core/src/mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole` 在用户改动的发布（1921 行）与整理显式入口（1995 行）之后调用，`discard_at(` 另在 `block_device.rs` 单测、harness 录制器、故障注入转发与 `mounted_session.rs:2465` 命中。现查：`grep -rn "discard_at(" crates --include=*.rs` 命中 mounted_session.rs:2465，该函数 0b926bd3f 进仓。译文照改。同一行前半「没有整理的触发、发布入口」另归 F33 判 |
| F40 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:285 | 不相干 | 「discard 的时机」一段已是新说法（发布返回之后、可再分配谓词为真的槽按 64 槽整段发、trait 加 discard_at 默认返回不支持并计数），不含旧口径（原行：「of discard**: Disk FTL」） |
| F40 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:291 | 不相干 | 第一版取法表的「discard_at 交回什么」一行（Applied 或 Unsupported，Unsupported 计数）与新事实一致，现查 block_device.rs 有 DiscardOutcome（原行：「discard_at returns」） |
| F40 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:295 | 要改 | 原：「Callers on product path」；改为：「产品路径的调用者」一格改为：`mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole`，用户改动的发布与整理显式入口之后各调一次（装着还没清扫孤儿头的段不发）。旧句「还没有：发布路径上没有一处调 discard_at」现查不真：`grep -n "discard_at(" crates/singlefs-core/src/mounted_session.rs` 命中 2465 行，调用者在 1921、1995 行。译文照改；下面「还开着两问」是否照旧由写的人核 |
| F40 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:184 | 要改 | 原：「**射程**: 回収の系列分離要否および選定メカ」；改为：把「块设备 trait 有 `discard_at`……产品路径上没有调用者（`discard_at(` 只在 `block_device.rs` 的单测、harness 的录制器与故障注入设备转发里命中）」改成：块设备 trait 有 `discard_at`（`crates/singlefs-core/src/block_device.rs`），产品路径上由 `crates/singlefs-core/src/mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole` 在用户改动的发布（1921 行）与整理显式入口（1995 行）之后调用，`discard_at(` 另在 `block_device.rs` 单测、harness 录制器、故障注入转发与 `mounted_session.rs:2465` 命中。现查：`grep -rn "discard_at(" crates --include=*.rs` 命中 mounted_session.rs:2465，该函数 0b926bd3f 进仓。译文照改。同一行前半「没有整理的触发、发布入口」另归 F33 判 |
| F40 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:285 | 不相干 | 「discard 的时机」一段已是新说法（发布返回之后、可再分配谓词为真的槽按 64 槽整段发、trait 加 discard_at 默认返回不支持并计数），不含旧口径（原行：「**discard のタイミング**: ディスク」） |
| F40 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:291 | 不相干 | 第一版取法表的「discard_at 交回什么」一行（Applied 或 Unsupported，Unsupported 计数）与新事实一致，现查 block_device.rs 有 DiscardOutcome（原行：「discard_at の返り値」） |
| F40 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:295 | 要改 | 原：「プロダクトパスからの呼び出し元」；改为：「产品路径的调用者」一格改为：`mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole`，用户改动的发布与整理显式入口之后各调一次（装着还没清扫孤儿头的段不发）。旧句「还没有：发布路径上没有一处调 discard_at」现查不真：`grep -n "discard_at(" crates/singlefs-core/src/mounted_session.rs` 命中 2465 行，调用者在 1921、1995 行。译文照改；下面「还开着两问」是否照旧由写的人核 |
| F40 | .claude/kb/decisions/26-后台整理与放置回收.md:184 | 要改 | 原：「**射程**：管回收分不分线与按什么选线，不管怎」；改为：把「块设备 trait 有 `discard_at`……产品路径上没有调用者（`discard_at(` 只在 `block_device.rs` 的单测、harness 的录制器与故障注入设备转发里命中）」改成：块设备 trait 有 `discard_at`（`crates/singlefs-core/src/block_device.rs`），产品路径上由 `crates/singlefs-core/src/mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole` 在用户改动的发布（1921 行）与整理显式入口（1995 行）之后调用，`discard_at(` 另在 `block_device.rs` 单测、harness 录制器、故障注入转发与 `mounted_session.rs:2465` 命中。现查：`grep -rn "discard_at(" crates --include=*.rs` 命中 mounted_session.rs:2465，该函数 0b926bd3f 进仓。译文照改。同一行前半「没有整理的触发、发布入口」另归 F33 判 |
| F40 | .claude/kb/decisions/26-后台整理与放置回收.md:286 | 不相干 | 「discard 的时机」一段已是新说法（发布返回之后、可再分配谓词为真的槽按 64 槽整段发、trait 加 discard_at 默认返回不支持并计数），不含旧口径（原行：「**discard 的时机**：盘的 FTL 只」） |
| F40 | .claude/kb/decisions/26-后台整理与放置回收.md:292 | 不相干 | 第一版取法表的「discard_at 交回什么」一行（Applied 或 Unsupported，Unsupported 计数）与新事实一致，现查 block_device.rs 有 DiscardOutcome（原行：「discard_at 交回什么」） |
| F40 | .claude/kb/decisions/26-后台整理与放置回收.md:296 | 要改 | 原：「产品路径的调用者」；改为：「产品路径的调用者」一格改为：`mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole`，用户改动的发布与整理显式入口之后各调一次（装着还没清扫孤儿头的段不发）。旧句「还没有：发布路径上没有一处调 discard_at」现查不真：`grep -n "discard_at(" crates/singlefs-core/src/mounted_session.rs` 命中 2465 行，调用者在 1921、1995 行。译文照改；下面「还开着两问」是否照旧由写的人核 |
| F40 | .claude/kb/decisions/26-后台整理与放置回收.md:306 | 不相干 | 这句只说发给盘的指令里至少有 discard 并指向未定项 9 的时机一段，没有说 discard 没调用者或时机未定（原行：「至少有 discard，理由见未定项 9 的「d」） |
| F40 | .claude/kb/milestone/04-fourth-txn.md:64 | 不相干 | 二·1 行里已写「discard 什么时候发（已定：发布返回之后……按整段发）」，与新事实一致（原行：「登记）；discard 什么时候发（已定：发布返」） |
| F40 | records/2026-09-24-里程碑二收尾调度.md:315 | 不相干 | `discarded_because=none` 是分片续跑进度字段（跑过的片被丢弃的原因），与块设备 discard 无关（原行：「用户：「预演一次分片流程跑一次 然后中断 然后续」） |
| F40 | records/2026-09-24-里程碑二收尾调度.md:318 | 不相干 | `discarded_because=none` 是分片续跑进度字段（跑过的片被丢弃的原因），与块设备 discard 无关（原行：「28、discarded_because=no」） |
| F40 | records/2026-10-12-里程碑四关键信息调研.md:53 | 事件句不改 | 这是 2026-10-12 调研里记下的一处 kb 腐烂（当时 `discarded_because` grep 0 行），说的是那天的现查（原行：「 说 discard 的 grep 会命中 」） |
| F40 | records/2026-10-13-里程碑四预开发.md:60 | 事件句不改 | 这是 2026-10-13 里程碑四预开发的派发与交回记录（第一批 discard_at 口子、实现员交回、写回员交回的判点），说的是那一次做了什么（原行：「设备 discard_at 口子、录制与崩溃」） |
| F40 | records/2026-10-13-里程碑四预开发.md:67 | 事件句不改 | 这是 2026-10-13 里程碑四预开发的派发与交回记录（第一批 discard_at 口子、实现员交回、写回员交回的判点），说的是那一次做了什么（原行：「 指「还缺一个前提」的指针已改。它列的 D26」） |
| F40 | records/2026-10-13-里程碑四预开发.md:79 | 事件句不改 | 这是 2026-10-13 里程碑四预开发的派发与交回记录（第一批 discard_at 口子、实现员交回、写回员交回的判点），说的是那一次做了什么（原行：「二·2 discard 实现员交回（103 分，」） |
| F40 | records/2026-10-13-里程碑四预开发.md:120 | 事件句不改 | 这是 2026-10-13 里程碑四预开发的派发与交回记录（第一批 discard_at 口子、实现员交回、写回员交回的判点），说的是那一次做了什么（原行：「份、discard_at 产品路径上没有调」） |
| F41 | .claude/kb/checks-owed.en.md:254 | 不相干 | C346 行里「logged_ops 回退时取哪一版还没定」说的是回退取版，不是字段表；字段表已定不使它变真或变假（原行：「D8 (Core Index Structure」） |
| F41 | .claude/kb/checks-owed.en.md:268 | 要改 | 原：「字段表已定 (判决 research/pro」；改为：把「仍欠实现（`crates/singlefs-core/src/logged_operations.rs` 在第 2 批补丁里）与 I-4.4（意图只描述目标态） 的判法（字段白名单）」改成「仍欠 I-4.4（意图只描述目标态） 的判法（字段白名单）与会红的检查」。现查：`ls crates/singlefs-core/src/logged_operations.rs` 在，0b926bd3f 进仓，`transaction.rs:12690` 已用 `logged_operations_root_entries` 写条目；译文照改 |
| F41 | .claude/kb/checks-owed.en.md:272 | 不相干 | C372 行里的「C364 第一轮攻方腿的 64 槽扫描」只是引那一轮攻方腿的名字，说的是 2026-09-17 那一轮；不涉及字段表是否已定（原行：「D3 (Space Allocation) se」） |
| F41 | .claude/kb/checks-owed.en.md:273 | 不相干 | C373 行只用 C364 当简称；行里「crates/ 里还没有整理代码」这一半另归 F33/F37 判，不属字段表这件事实（原行：「D26 (Background Compacti」） |
| F41 | .claude/kb/checks-owed.ja.md:254 | 不相干 | C346 行里「logged_ops 回退时取哪一版还没定」说的是回退取版，不是字段表；字段表已定不使它变真或变假（原行：「D8（中核インデックス構造） 確定項目 3 第」） |
| F41 | .claude/kb/checks-owed.ja.md:268 | 要改 | 原：「字段表已定（判决 research/prom」；改为：把「仍欠实现（`crates/singlefs-core/src/logged_operations.rs` 在第 2 批补丁里）与 I-4.4（意图只描述目标态） 的判法（字段白名单）」改成「仍欠 I-4.4（意图只描述目标态） 的判法（字段白名单）与会红的检查」。现查：`ls crates/singlefs-core/src/logged_operations.rs` 在，0b926bd3f 进仓，`transaction.rs:12690` 已用 `logged_operations_root_entries` 写条目；译文照改 |
| F41 | .claude/kb/checks-owed.ja.md:272 | 不相干 | C372 行里的「C364 第一轮攻方腿的 64 槽扫描」只是引那一轮攻方腿的名字，说的是 2026-09-17 那一轮；不涉及字段表是否已定（原行：「全不同（C364（整理意图记录没有字段表） 第一」） |
| F41 | .claude/kb/checks-owed.ja.md:273 | 不相干 | C373 行只用 C364 当简称；行里「crates/ 里还没有整理代码」这一半另归 F33/F37 判，不属字段表这件事实（原行：「D26（バックグラウンドデフラグと配置回収） 確」） |
| F41 | .claude/kb/checks-owed.md:253 | 不相干 | C346 行里「logged_ops 回退时取哪一版还没定」说的是回退取版，不是字段表；字段表已定不使它变真或变假（原行：「D8（核心索引结构） 已定项 3 第 3 条「后」） |
| F41 | .claude/kb/checks-owed.md:267 | 要改 | 原：「字段表已定（判决 research/prom」；改为：把「仍欠实现（`crates/singlefs-core/src/logged_operations.rs` 在第 2 批补丁里）与 I-4.4（意图只描述目标态） 的判法（字段白名单）」改成「仍欠 I-4.4（意图只描述目标态） 的判法（字段白名单）与会红的检查」。现查：`ls crates/singlefs-core/src/logged_operations.rs` 在，0b926bd3f 进仓，`transaction.rs:12690` 已用 `logged_operations_root_entries` 写条目；译文照改 |
| F41 | .claude/kb/checks-owed.md:271 | 不相干 | C372 行里的「C364 第一轮攻方腿的 64 槽扫描」只是引那一轮攻方腿的名字，说的是 2026-09-17 那一轮；不涉及字段表是否已定（原行：「全不同（C364（整理意图记录没有字段表） 第一」） |
| F41 | .claude/kb/checks-owed.md:272 | 不相干 | C373 行只用 C364 当简称；行里「crates/ 里还没有整理代码」这一半另归 F33/F37 判，不属字段表这件事实（原行：「D26（后台整理与放置回收） 已定项 1 第 2」） |
| F41 | .claude/kb/decisions/05-snapshot-space-accounting-mechanism.en.md:232 | 要改 | 原：「**Settlement**: Writable」；改为：把「logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表））」改成「logged_ops 写 0（整理意图条目的字段表已定，见 D26 已定项 2；C364 仍欠 I-4.4 的判法）」。D26 已定项 2 现有 key 10、value 34 的字段表与四个 format-const 标记，「字段表未定」已不真。译文照改 |
| F41 | .claude/kb/decisions/05-スナップショット-空間会計メカニズム.ja.md:232 | 要改 | 原：「**決定事項**: 書き込み可能ヘッドの ID」；改为：把「logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表））」改成「logged_ops 写 0（整理意图条目的字段表已定，见 D26 已定项 2；C364 仍欠 I-4.4 的判法）」。D26 已定项 2 现有 key 10、value 34 的字段表与四个 format-const 标记，「字段表未定」已不真。译文照改 |
| F41 | .claude/kb/decisions/05-快照-空间记账机制.md:232 | 要改 | 原：「**定案**：可写头的 ID = 该头 inod」；改为：把「logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表））」改成「logged_ops 写 0（整理意图条目的字段表已定，见 D26 已定项 2；C364 仍欠 I-4.4 的判法）」。D26 已定项 2 现有 key 10、value 34 的字段表与四个 format-const 标记，「字段表未定」已不真。译文照改 |
| F41 | .claude/kb/decisions/08-core-index-structure.en.md:110 | 要改 | 原：「**Settlement**: Deleting」；改为：把「字段表、回退时取哪一版、态别留到第一个写者（整理，D26 已定项 2）落地时定（C364）」改成「字段表已定（D26 已定项 2 的整理意图条目字段表），回退时取哪一版、态别留到第一个写者（整理）落地时定（C364）」。回退取版与态别是否也已定，请主 agent 核 D26 与 D23 后再删。译文照改 |
| F41 | .claude/kb/decisions/08-core-index-structure.en.md:265 | 要改 | 原：「0 (field table unsettle」；改为：把「logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表））」改成「logged_ops 写 0（整理意图条目的字段表已定，见 D26 已定项 2；C364 仍欠 I-4.4 的判法）」。译文照改 |
| F41 | .claude/kb/decisions/08-コアインデックス構造.ja.md:110 | 要改 | 原：「**決定事項**：大ディレクトリの削除や大ファイ」；改为：把「字段表、回退时取哪一版、态别留到第一个写者（整理，D26 已定项 2）落地时定（C364）」改成「字段表已定（D26 已定项 2 的整理意图条目字段表），回退时取哪一版、态别留到第一个写者（整理）落地时定（C364）」。回退取版与态别是否也已定，请主 agent 核 D26 与 D23 后再删。译文照改 |
| F41 | .claude/kb/decisions/08-コアインデックス構造.ja.md:265 | 要改 | 原：「は 0（フィールド表未定、C364 (整理意図レ」；改为：把「logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表））」改成「logged_ops 写 0（整理意图条目的字段表已定，见 D26 已定项 2；C364 仍欠 I-4.4 的判法）」。译文照改 |
| F41 | .claude/kb/decisions/08-核心索引结构.md:110 | 要改 | 原：「根注册，字段表、回退时取哪一版、态别留到第一个写」；改为：把「字段表、回退时取哪一版、态别留到第一个写者（整理，D26 已定项 2）落地时定（C364）」改成「字段表已定（D26 已定项 2 的整理意图条目字段表），回退时取哪一版、态别留到第一个写者（整理）落地时定（C364）」。回退取版与态别是否也已定，请主 agent 核 D26 与 D23 后再删。译文照改 |
| F41 | .claude/kb/decisions/08-核心索引结构.md:265 | 要改 | 原：「写 0（字段表未定，C364（整理意图记录没有字」；改为：把「logged_ops 写 0（字段表未定，C364（整理意图记录没有字段表））」改成「logged_ops 写 0（整理意图条目的字段表已定，见 D26 已定项 2；C364 仍欠 I-4.4 的判法）」。译文照改 |
| F41 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:43 | 不相干 | 「欠」一行只点名 C364 这笔账，账仍开着（checks-owed.md:267 仍欠 I-4.4 判法与会红的检查），句子没说字段表有没有；账的题目「没有字段表」已与现状不合，是否改题交主 agent（原行：「y); C364 (Compaction int」） |
| F41 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:70 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:71 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:72 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:73 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-background-compaction-and-placement-reclamation.en.md:84 | 不相干 | 「欠」一行只点名 C364 这笔账，账仍开着（checks-owed.md:267 仍欠 I-4.4 判法与会红的检查），句子没说字段表有没有；账的题目「没有字段表」已与现状不合，是否改题交主 agent（原行：「d); C364 (Compaction int」） |
| F41 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:43 | 不相干 | 「欠」一行只点名 C364 这笔账，账仍开着（checks-owed.md:267 仍欠 I-4.4 判法与会红的检查），句子没说字段表有没有；账的题目「没有字段表」已与现状不合，是否改题交主 agent（原行：「ない)；C364 (整理意図レコードにフィールド」） |
| F41 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:70 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:71 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:72 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:73 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-バックグラウンド整理と配置回収.ja.md:84 | 不相干 | 「欠」一行只点名 C364 这笔账，账仍开着（checks-owed.md:267 仍欠 I-4.4 判法与会红的检查），句子没说字段表有没有；账的题目「没有字段表」已与现状不合，是否改题交主 agent（原行：「ない)；C364 (整理意図レコードにフィールド」） |
| F41 | .claude/kb/decisions/26-后台整理与放置回收.md:43 | 不相干 | 「欠」一行只点名 C364 这笔账，账仍开着（checks-owed.md:267 仍欠 I-4.4 判法与会红的检查），句子没说字段表有没有；账的题目「没有字段表」已与现状不合，是否改题交主 agent（原行：「轨迹）；C364（整理意图记录没有字段表）；C3」） |
| F41 | .claude/kb/decisions/26-后台整理与放置回收.md:70 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-后台整理与放置回收.md:71 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-后台整理与放置回收.md:72 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-后台整理与放置回收.md:73 | 不相干 | 这是新事实本身的 format-const 标记（LOGGED_OPERATIONS_KEY_BYTES = 10、VALUE_BYTES = 34、ENTRY_BYTES = 44、ENTRIES_PER_NODE = 369），已是新说法（原行：「!-- format-const: LOGGED」） |
| F41 | .claude/kb/decisions/26-后台整理与放置回收.md:84 | 不相干 | 「欠」一行只点名 C364 这笔账，账仍开着（checks-owed.md:267 仍欠 I-4.4 判法与会红的检查），句子没说字段表有没有；账的题目「没有字段表」已与现状不合，是否改题交主 agent（原行：「腾出）；C364（整理意图记录没有字段表）。」） |
| F41 | .claude/kb/freeze-layer-membership.en.md:33 | 要改 | 原：「on, field table unsettle」；改为：把「logged_ops 树（树 ID 65547）的 key 编码（零根注册，字段表未定，C364）」改成「……（零根注册，字段表已定：D26 已定项 2，key 10 = 种类 2 + 签发 txg 8；C364 仍欠 I-4.4 的判法）」。key 编码既已定，「判不动」那一格的推导（判据哪一句卡在哪）要不要重推，请主 agent 看。译文照改 |
| F41 | .claude/kb/freeze-layer-membership.en.md:106 | 要改 | 原：「es; field table, rollbac」；改为：把「字段表、回退取哪一版、态别都留到整理落地那天」改成「字段表已定（D26 已定项 2），回退取哪一版、态别留到整理落地那天」；其余「一棵树装哪一类……没有一条已定分项说」不动。译文照改 |
| F41 | .claude/kb/freeze-layer-membership.ja.md:33 | 要改 | 原：「ト登録、フィールド表未定、C364（コンパクショ」；改为：把「logged_ops 树（树 ID 65547）的 key 编码（零根注册，字段表未定，C364）」改成「……（零根注册，字段表已定：D26 已定项 2，key 10 = 种类 2 + 签发 txg 8；C364 仍欠 I-4.4 的判法）」。key 编码既已定，「判不动」那一格的推导（判据哪一句卡在哪）要不要重推，请主 agent 看。译文照改 |
| F41 | .claude/kb/freeze-layer-membership.ja.md:106 | 要改 | 原：「れない；フィールド表、どのバージョンへロールバッ」；改为：把「字段表、回退取哪一版、态别都留到整理落地那天」改成「字段表已定（D26 已定项 2），回退取哪一版、态别留到整理落地那天」；其余「一棵树装哪一类……没有一条已定分项说」不动。译文照改 |
| F41 | .claude/kb/freeze-layer-membership.md:33 | 要改 | 原：「根注册，字段表未定，C364（整理意图记录没有字」；改为：把「logged_ops 树（树 ID 65547）的 key 编码（零根注册，字段表未定，C364）」改成「……（零根注册，字段表已定：D26 已定项 2，key 10 = 种类 2 + 签发 txg 8；C364 仍欠 I-4.4 的判法）」。key 编码既已定，「判不动」那一格的推导（判据哪一句卡在哪）要不要重推，请主 agent 看。译文照改 |
| F41 | .claude/kb/freeze-layer-membership.md:106 | 要改 | 原：「都不写；字段表、回退取哪一版、态别都留到整理落地」；改为：把「字段表、回退取哪一版、态别都留到整理落地那天」改成「字段表已定（D26 已定项 2），回退取哪一版、态别留到整理落地那天」；其余「一棵树装哪一类……没有一条已定分项说」不动。译文照改 |
| F41 | .claude/kb/milestone/02-second-txn.en.md:359 | 事件句不改 | 这是里程碑二收尾 2026-09-28 的处置记录（那一轮判完了谁、c364-r3 也在同批），说的是当时发生的事，C364 在这里只是账号简称（原行：「Two rounds of tripartite」） |
| F41 | .claude/kb/milestone/02-second-txn.ja.md:359 | 事件句不改 | 这是里程碑二收尾 2026-09-28 的处置记录（那一轮判完了谁、c364-r3 也在同批），说的是当时发生的事，C364 在这里只是账号简称（原行：「ゲート doc-process-records」） |
| F41 | .claude/kb/milestone/02-second-txn.md:373 | 事件句不改 | 这是里程碑二收尾 2026-09-28 的处置记录（那一轮判完了谁、c364-r3 也在同批），说的是当时发生的事，C364 在这里只是账号简称（原行：「门禁 doc-process-records 的」） |
| F41 | .claude/kb/milestone/04-fourth-txn.en.md:18 | 不相干 | 里程碑四出口项要的是 C364 等账「还清或改写」，账仍开着，句子没说字段表有没有，仍是真话（原行：「s), C364 (compaction int」） |
| F41 | .claude/kb/milestone/04-fourth-txn.en.md:59 | 要改 | 原：「- **Existing in reposito」；改为：把「GC 意图的载体是 logged_ops 树（……用户定案 K-I1：零根注册、字段表等整理落地时再定……），今天代码里只有零根与种类码 `TREE_KIND_LOGGED_OPERATIONS`（`crates/singlefs-core/src/records.rs`），没有写者……C364 是二·1 的第一题（2026-10-12 现查）」改成：用户定案 K-I1 让它零根注册；字段表后来由判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 在 D26 已定项 2 定下（key 10 + value 34，条目 44）；`crates/singlefs-core/src/logged_operations.rs` 已有字段表的 `to_bytes` 与 `parse`，`transaction.rs` 已有写 logged_ops 条目的代码（12690 行 `logged_operations_root_entries`，基准 1bb3a24dc 上 0 处）；C364 余下 I-4.4 的判法与会红的检查。同一句 D3 已定项 7 的 N₀ 引文另归 F39，discard 零命中另归 F40。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.en.md:64 | 要改 | 原：「nd: field table for comp」；改为：把二·1 的「设计轮：整理意图记录的字段表」改成「设计轮：整理意图记录的字段表（已定：D26 已定项 2 的字段表，key 10 + value 34）；仍欠 I-4.4 的判法」；三条水位与 N₀ 的取值等 E10，不动。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.en.md:188 | 要改 | 原：「ows logged_ops tree; use」；改为：三问答案一栏的现状改为：用户定案 K-I1 让它零根注册，字段表已由判决 m4-gc-r1 G1 定在 D26 已定项 2；出处一栏「`TREE_KIND_LOGGED_OPERATIONS` 没有写者」改为 `transaction.rs` 已有写 logged_ops 条目的代码（12690 行 `logged_operations_root_entries`）；「里程碑四要做的」一栏「二·1 先定 logged_ops 的字段表」改为「字段表已定，仍欠 I-4.4 判法（C364）」。这一节标题「三问的答案现状（2026-10-12 现查）」带日期，改完要么重写日期要么注明哪几格后来变了。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.ja.md:18 | 不相干 | 里程碑四出口项要的是 C364 等账「还清或改写」，账仍开着，句子没说字段表有没有，仍是真话（原行：「ない)、C364 (整理インテントレコードにフィ」） |
| F41 | .claude/kb/milestone/04-fourth-txn.ja.md:59 | 要改 | 原：「- **リポジトリ内の既存要素**：D26 (バ」；改为：把「GC 意图的载体是 logged_ops 树（……用户定案 K-I1：零根注册、字段表等整理落地时再定……），今天代码里只有零根与种类码 `TREE_KIND_LOGGED_OPERATIONS`（`crates/singlefs-core/src/records.rs`），没有写者……C364 是二·1 的第一题（2026-10-12 现查）」改成：用户定案 K-I1 让它零根注册；字段表后来由判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 在 D26 已定项 2 定下（key 10 + value 34，条目 44）；`crates/singlefs-core/src/logged_operations.rs` 已有字段表的 `to_bytes` 与 `parse`，`transaction.rs` 已有写 logged_ops 条目的代码（12690 行 `logged_operations_root_entries`，基准 1bb3a24dc 上 0 处）；C364 余下 I-4.4 的判法与会红的检查。同一句 D3 已定项 7 的 N₀ 引文另归 F39，discard 零命中另归 F40。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.ja.md:64 | 要改 | 原：「コードのフィールド表；三つの水位と N₀ の」；改为：把二·1 的「设计轮：整理意图记录的字段表」改成「设计轮：整理意图记录的字段表（已定：D26 已定项 2 的字段表，key 10 + value 34）；仍欠 I-4.4 的判法」；三条水位与 N₀ 的取值等 E10，不动。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.ja.md:188 | 要改 | 原：「ントは logged_ops ツリーを踏襲；ユー」；改为：三问答案一栏的现状改为：用户定案 K-I1 让它零根注册，字段表已由判决 m4-gc-r1 G1 定在 D26 已定项 2；出处一栏「`TREE_KIND_LOGGED_OPERATIONS` 没有写者」改为 `transaction.rs` 已有写 logged_ops 条目的代码（12690 行 `logged_operations_root_entries`）；「里程碑四要做的」一栏「二·1 先定 logged_ops 的字段表」改为「字段表已定，仍欠 I-4.4 判法（C364）」。这一节标题「三问的答案现状（2026-10-12 现查）」带日期，改完要么重写日期要么注明哪几格后来变了。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.md:18 | 不相干 | 里程碑四出口项要的是 C364 等账「还清或改写」，账仍开着，句子没说字段表有没有，仍是真话（原行：「出去）、C364（整理意图记录没有字段表）、C9」） |
| F41 | .claude/kb/milestone/04-fourth-txn.md:59 | 要改 | 原：「- **仓里已有的**：D26（后台整理与放置回」；改为：把「GC 意图的载体是 logged_ops 树（……用户定案 K-I1：零根注册、字段表等整理落地时再定……），今天代码里只有零根与种类码 `TREE_KIND_LOGGED_OPERATIONS`（`crates/singlefs-core/src/records.rs`），没有写者……C364 是二·1 的第一题（2026-10-12 现查）」改成：用户定案 K-I1 让它零根注册；字段表后来由判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1 在 D26 已定项 2 定下（key 10 + value 34，条目 44）；`crates/singlefs-core/src/logged_operations.rs` 已有字段表的 `to_bytes` 与 `parse`，`transaction.rs` 已有写 logged_ops 条目的代码（12690 行 `logged_operations_root_entries`，基准 1bb3a24dc 上 0 处）；C364 余下 I-4.4 的判法与会红的检查。同一句 D3 已定项 7 的 N₀ 引文另归 F39，discard 零命中另归 F40。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.md:64 | 要改 | 原：「图记录的字段表；三条水位与 N₀ 的取值（E」；改为：把二·1 的「设计轮：整理意图记录的字段表」改成「设计轮：整理意图记录的字段表（已定：D26 已定项 2 的字段表，key 10 + value 34）；仍欠 I-4.4 的判法」；三条水位与 N₀ 的取值等 E10，不动。译文照改 |
| F41 | .claude/kb/milestone/04-fourth-txn.md:191 | 要改 | 原：「意图走 logged_ops 树；用户定案 K-」；改为：三问答案一栏的现状改为：用户定案 K-I1 让它零根注册，字段表已由判决 m4-gc-r1 G1 定在 D26 已定项 2；出处一栏「`TREE_KIND_LOGGED_OPERATIONS` 没有写者」改为 `transaction.rs` 已有写 logged_ops 条目的代码（12690 行 `logged_operations_root_entries`）；「里程碑四要做的」一栏「二·1 先定 logged_ops 的字段表」改为「字段表已定，仍欠 I-4.4 判法（C364）」。这一节标题「三问的答案现状（2026-10-12 现查）」带日期，改完要么重写日期要么注明哪几格后来变了。译文照改 |
| F41 | records/2026-09-13-总审核.md:169 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「段表；logged_ops 树无树 ID /」） |
| F41 | records/2026-09-28-里程碑三第十项清单.md:219 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「C364（整理意图记录没有字段表）」） |
| F41 | records/2026-10-01-删除与并发事务.md:193 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「现在定 logged_ops 树、C 合一推翻」） |
| F41 | records/2026-10-01-删除与并发事务.md:202 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「定的是 C364 留的岔；C 中三处（D8 已定」） |
| F41 | records/2026-10-01-删除与并发事务.md:203 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「写回：logged_ops 树与 delet」） |
| F41 | records/2026-10-01-里程碑三形式化接入.md:984 | 不相干 | 这一行讲的是把 I-9.16 ② 的循环补到 11 种、常量 `TREE_KIND_LOGGED_OPERATIONS = 12u`，谈的是树种类码，不涉及字段表（原行：「IND_LOGGED_OPERATIONS =」） |
| F41 | records/2026-10-12-里程碑四关键信息调研.md:33 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「5 / C364 / C412 都在 chec」） |
| F41 | records/2026-10-12-里程碑四关键信息调研.md:35 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「1 让 logged_ops 树零根注册、字段表」） |
| F41 | records/2026-10-12-里程碑四关键信息调研.md:44 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「的载体 logged_ops 只有零根与种类码，」） |
| F41 | records/2026-10-13-里程碑四预开发.md:25 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「计，含 logged_ops 字段表 C364（」） |
| F41 | records/2026-10-13-里程碑四预开发.md:66 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「乙改写 C364、C372、C140、C417、」） |
| F41 | records/2026-10-13-里程碑四预开发.md:67 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「那一批（C364、C372、C373、C140、」） |
| F41 | records/2026-10-13-里程碑四预开发.md:71 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「：改写 C364、C372、C140、C417、」） |
| F41 | records/2026-10-13-里程碑四预开发.md:78 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「- 第 2 批 二·2 GC 纯函数实现员交回（」） |
| F41 | records/2026-10-13-里程碑四预开发.md:84 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「（四个 logged_ops 常量挂 C364）」） |
| F41 | records/2026-10-13-里程碑四预开发.md:120 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「认；② C364 不还——认（会红的检查没建）；」） |
| F41 | records/2026-10-13-里程碑四预开发.md:189 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「- 写段收尾件交回（Opus，47 分，45.4」） |
| F41 | records/2026-10-13-里程碑四预开发.md:377 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「哪边」、C364 逐格并；C820–C829 是」） |
| F41 | records/2026-10-18-审查报告核实与修复.md:53 | 事件句不改 | 这是总审核、里程碑三清单、删除与并发事务、里程碑四调研与预开发、审查报告核实等记录里某一天的处置或现查（C364 当时无字段表、K-I1 当时的定案、写回员与实现员交回的判点），说的是那一次发生的事（原行：「了的 LOGGED_OPERATIONS_KE」） |
| F44 | .claude/kb/checks-owed.ja.md:374 | 要改 | 原：「C576」；改为：译文这一行（C576）还在开着的表里；中文原文已把它移进已还清表（checks-owed.md:859，证据两条用例的结局已钉、变异已按名字登记）。把 ja 的这一行移进已还清表，证据列照中文译。现查：ja 与 en 的 checks-owed 各 861 行，中文 867 行，C296、C576、C587、C668、C827 在译文里仍在开着的区段，比中文落后 |
| F44 | .claude/kb/checks-owed.ja.md:378 | 不相干 | 这是 C588 行的去向列，C587 只是「同属崩溃放量剪枝与 GPU 第二轮判决」的简称；该句说的是 2026-10-11 判的去向，不说 C587 开没开（原行：「5（与 C587（扫描候选槽头被伪造头骗过） 同」） |
| F44 | .claude/kb/checks-owed.ja.md:391 | 事件句不改 | 这是 C600 行里 2026-10-15 重跑大档的结果（红在另一个签名上，C572 的新拒因），说的是那一次重跑的事（原行：「已做：设备吞写之后書き込み可能マウント该成还是该」） |
| F44 | .claude/kb/checks-owed.ja.md:688 | 要改 | 原：「实现，待提交落地销账）」；改为：把「（2026-10-15 实现，待提交落地销账）」改成「（2026-10-15 实现，C61 已还清）」：C61 现已在已还清表（checks-owed.md:855，证据 doc-decisions 的 cross-decision-status 格放宽与三份样本），「待提交落地销账」已不真。译文照改 |
| F44 | .claude/kb/checks-owed.md:375 | 不相干 | 这是 C588 行的去向列，C587 只是「同属崩溃放量剪枝与 GPU 第二轮判决」的简称，说的是 2026-10-11 判的去向，不说 C587 开没开（原行：「记录归哪次发布的判据要改（按 txg 还是按写表」） |
| F44 | .claude/kb/checks-owed.md:388 | 事件句不改 | 这是 C600 行里 2026-10-15 重跑大档的结果（「覆盖写吞写之后挂着回退，C572 的新拒因」），说的是那一次重跑发生的事（原行：「已做：设备吞写之后可写挂载该成还是该拒的条款已定」） |
| F44 | .claude/kb/checks-owed.md:689 | 要改 | 原：「实现，待提交落地销账）」；改为：把「（2026-10-15 实现，待提交落地销账）」改成「（2026-10-15 实现，C61 已还清）」：C61 在已还清表 checks-owed.md:855，证据 `doc-decisions.sh` 放宽后的三份样本，「待提交落地销账」已不真 |
| F44 | .claude/kb/decisions/02-RAID条带策略.md:317 | 不相干 | 这句交代依据是哪份判决和哪笔账（欠账 C568），C568 是 io_min 1 MiB 不接受那一笔，射程已写进已定项 19，句子不说账开没开（原行：「**射程**：io_min = 4096、」） |
| F44 | .claude/kb/decisions/13-验证路线.md:445 | 要改 | 原：「能派）、C668（随机历史脚手架的引入者白名单能」；改为：「欠」一行删去「C668（随机历史脚手架的引入者白名单能被别名写法绕过）、」：C668 已在已还清表（checks-owed.md:862，前提不在，判定复用改按函数单元键） |
| F44 | .claude/kb/decisions/19-块指针的结构与宽度预算.md:175 | 要改 | 原：「路径）、C296（journal 重放时凑不齐映」；改为：「欠」一行删去「C296（journal 重放时凑不齐映射 key）、」：C296 已在已还清表（checks-owed.md:860，I-8.11 已实现，6 行变异） |
| F44 | .claude/kb/decisions/22-单元原子性怎么合成.md:392 | 要改 | 原：「欠**：C403（引三方论证而没有腿况记录）（检」；改为：整行删掉：这行自己写着「检查已做进 doc-decisions 的 three-way-leg-record 格，待提交落地销账，销账后此行删」，C403 已在已还清表（checks-owed.md:854），销账已发生，该删 |
| F44 | .claude/kb/decisions/28-挂载期承诺量.md:140 | 事件句不改 | 这是 A4d 实审当时交给主 agent 的三处设计问题「记欠」（C573 至 C576），说的是那一次记了哪几笔账，不说它们现在开没开（原行：「载荷）、C576（三条用例被准入先拒连带红，改法」） |
| F44 | .claude/kb/milestone/03-third-txn.md:72 | 要改 | 原：「其余归项二的欠账簇，开工日逐条现查还开不开、按要」；改为：二·5 的欠账簇里「C296（journal 重放时凑不齐映射 key）」后补「（已还清）」，与同一行其余已还清的写法一致；C296 在 checks-owed.md:860 已还清 |
| F44 | .claude/kb/milestone/03-third-txn.md:107 | 要改 | 原：「「第十项自留」45 行，开工日逐条判「这个里程碑」；改为：五·1 的 45 行名单里 C568、C572、C576 后各补「（已还清）」，写法同本行 C355、C455 等；三笔都在 checks-owed.md 已还清表（858、852、859 行） |
| F44 | .claude/kb/milestone/03-third-txn.md:112 | 事件句不改 | 这是 2026-10-15 重跑大档红在别的签名上的结果（C572 的新拒因），说的是那一次重跑，不说 C572 开没开（原行：「大档故障注入 2 条新发现（用户 2026-09」） |
| F44 | .claude/kb/milestone/03-third-txn.md:124 | 要改 | 原：「C356（第九项按统计量之差算，净释放时为负且没」；改为：把「C576（三条用例被准入先拒连带红，改法待选）（判决：行改成按变异名登记，第 149 行漂移、第 12 行不红）」后补「（已还清：两条用例的结局已钉、变异已按名字登记，checks-owed.md:859）」；其余 C568 那一条用户已定的写法不动 |
| F44 | .claude/kb/milestone/03-third-txn.md:125 | 不相干 | 这句写的是 C572 条款已定、实现与会红用例 2026-10-14 已合入，不说账开着，已还清（checks-owed.md:852）与它不冲突（原行：「C355（checkpoint 保留池池级扣而固」） |
| F44 | .claude/kb/milestone/03-third-txn.md:132 | 不相干 | 这句写的是 C568 条款已定（不接受）、补拒绝用例已落，不说账开着，已还清（checks-owed.md:858）与它不冲突（原行：「C568（固定结构取严之后 io_min 1」） |
| F44 | .claude/kb/milestone/03-third-txn.md:135 | 不相干 | 这句写的是 C568 用户已定、其余岔路还开着，C568 已还清与「用户已定」不冲突（原行：「1 与 C568（固定结构取严之后 io_min」） |
| F44 | records/2026-09-13-总审核.md:238 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「C296（journal 重放时凑不齐映射 k」） |
| F44 | records/2026-09-19-决策瘦身与双向登记.md:189 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「格立成 C420（用私有索引提交之后共用索引过期」） |
| F44 | records/2026-09-28-里程碑三第十项清单.md:116 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「C568（固定结构取严之后 io_min 1」） |
| F44 | records/2026-09-28-里程碑三第十项清单.md:119 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「C572（回退目标引用隔离单元不进逐盘验）」） |
| F44 | records/2026-09-28-里程碑三第十项清单.md:123 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「C576（三条用例被准入先拒连带红，改法待选）」） |
| F44 | records/2026-09-28-里程碑三第十项清单.md:265 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「C296（journal 重放时凑不齐映射 k」） |
| F44 | records/2026-10-01-删除与并发事务.md:371 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「）：加 C668（随机历史脚手架的引入者白名单能」） |
| F44 | records/2026-10-19-里程碑四GPU垃圾回收开工.md:31 | 事件句不改 | 这是总审核、2026-09 至 10 月各记录里某一天的处置（登记 C296 要的是这几段、立成 C420、第十项清单逐行判「不相关」、写回员加 C668、C827 改三语标已还），说的是当时发生的事（原行：「起改」：C827（扫描重建对删除中与覆盖写变短的」） |
| F45 | .claude/kb/checks-owed.md:536 | 要改 | 原：「C821」；改为：改写 C821 行：题目「GPU 判器没有扫描重建内核」与现象列「内核不置位，GPU 与 CPU 对拍对这一条不比」都已不真。现查：`crates/singlefs-checker-tier/src/crash_judge_checker_scan_rebuild.wgsl` 在（0b926bd3f 进仓），`crash_judge_conclusions.rs:438` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是 `&[]`，`tests/crash_judge_gpu_scan_rebuild_agrees_with_the_cpu_checker.rs` 在，`crates/mutations.tsv` 有 24 行 C821 变异，invariants/01-块自描述.md 的 I-1.13 行已写「扫描内核的第三个入口 scan_rebuild_main 照判、置位」。改为「已做：内核与对拍用例已入库，变异 24 行；对拍与变异整表复跑归提交时（这一次没跑）」，是否移进已还清表等复跑绿了由主 agent 定。en、ja 的 checks-owed.md:540 同一行也要改 |
| F45 | .claude/kb/checks-owed.md:553 | 不相干 | C840 行说的是系统配置节点大小字段没人判，GPU 恢复内核不读偏移 301 那 4 字节；不涉及扫描重建内核（原行：「代码批；GPU 判器的恢复内核不读这 4 字节（」） |
| F45 | .claude/kb/checks-owed.md:559 | 不相干 | C846 行说的是每个状态草稿区按最大场景给，大卡填不满；不涉及扫描重建内核或覆盖表（原行：「GPU 判器每个状态的草稿区按最大场景给，大卡」） |
| F45 | .claude/kb/checks-owed.md:560 | 不相干 | C852 行说的是 GPU 恢复内核每盘落点下限写死 10；不涉及扫描重建内核或覆盖表（原行：「GPU 恢复内核每盘落点下限写死 10」） |
| F45 | .claude/kb/checks-owed.md:754 | 不相干 | C608 行是已还清的证据（GPU 变异罩内核与对拍），不是扫描重建内核缺不缺（原行：「GPU 变异不罩内核与 GPU 对 CPU 的」） |
| F45 | .claude/kb/checks-owed.md:757 | 不相干 | C676 行是已还清的证据（树表发号次序判八种），不涉及扫描重建内核（原行：「GPU 判器的树表发号次序只判七种」） |
| F45 | .claude/kb/milestone/04-fourth-txn.md:130 | 不相干 | 这是截断那份的实现进度（GPU 判器与恢复内核已打进、验证段没开），不涉及扫描重建内核缺不缺（原行：「、接线、GPU 判器与恢复内核、三条崩溃流已打进」） |
| F45 | .claude/kb/milestone/04-fourth-txn.md:158 | 不相干 | 这是里程碑四出口里一串仍欠账的名单，其中 C735、C744 谈 GPU 对拍流，不是 C821，不受内核补上影响（原行：「重放与 GPU 判器的判别力：已登记的流与 GP」） |
| F45 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:47 | 不相干 | 这是里程碑三 2026-09-28、29 GPU 判器分批接进内核与对拍的进展或理由，早于扫描重建内核，不涉及它（原行：「GPU 判器：三批 49 条全罩、17 条流逐」） |
| F45 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:48 | 不相干 | 这是里程碑三 2026-09-28、29 GPU 判器分批接进内核与对拍的进展或理由，早于扫描重建内核，不涉及它（原行：「GPU 判器：池级 checker 三批写完、」） |
| F45 | records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md:175 | 不相干 | 这是里程碑三 2026-09-28、29 GPU 判器分批接进内核与对拍的进展或理由，早于扫描重建内核，不涉及它（原行：「re」。GPU 判器对不对由两样全量的东西罩着：」） |
| F45 | records/2026-09-30-里程碑三开工第一轮.md:474 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「弹窗定：GPU 判器内核的旧格式由本线第 8 批」） |
| F45 | records/2026-09-30-里程碑三开工第一轮.md:498 | 事件句不改 | 这是 2026-09 至 10 月某一天关于 GPU 判器覆盖表「要求全覆盖」或补登的决定与现查，说的是当时怎么定、怎么排的（原行：「弹窗）：GPU 判器与 I-3.5 选「改回要求」） |
| F45 | records/2026-09-30-里程碑三开工第一轮.md:1001 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「批要改 GPU 判器内核，随后「用户刚定」把第」） |
| F45 | records/2026-10-01-删除与并发事务.md:730 | 事件句不改 | 这是 2026-09 至 10 月某一天关于 GPU 判器覆盖表「要求全覆盖」或补登的决定与现查，说的是当时怎么定、怎么排的（原行：「排期：GPU 判器覆盖表要求全覆盖（用户 20」） |
| F45 | records/2026-10-01-删除与并发事务.md:947 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「G：GPU 判器——九个新下标（I-9.17–」） |
| F45 | records/2026-10-01-对接上游测试脚本规范.md:665 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「早有）；GPU 判器五条在 HEAD 上绿、r2」） |
| F45 | records/2026-10-01-对接上游测试脚本规范.md:671 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「0 条：GPU 判器 6 条（别的会话暂存的内核」） |
| F45 | records/2026-10-01-对接上游测试脚本规范.md:683 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「1 条，GPU 判器对 CPU 判器那份测试二进」） |
| F45 | records/2026-10-01-里程碑三形式化接入.md:817 | 事件句不改 | 这是 2026-09 至 10 月某一天关于 GPU 判器覆盖表「要求全覆盖」或补登的决定与现查，说的是当时怎么定、怎么排的（原行：「进主仓（GPU 判器删「内核还没判」表、覆盖断言」） |
| F45 | records/2026-10-13-复用键与全仓改名.md:398 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「GPU 判器四处缺口：基线系统配置择槽、判表落」） |
| F45 | records/2026-10-13-复用键与全仓改名.md:867 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「不许绑 GPU 判器的实现（内核源码、派活代码）」） |
| F45 | records/2026-10-13-复用键与全仓改名.md:1030 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「。核：GPU 判器组的单元键在集成树与旧全量树」） |
| F45 | records/2026-10-13-里程碑四预开发.md:167 | 事件句不改 | 这是 2026-09 至 10 月某一天关于 GPU 判器覆盖表「要求全覆盖」或补登的决定与现查，说的是当时怎么定、怎么排的（原行：「GPU 覆盖与不比表合入时主 agent 补登）」） |
| F45 | records/2026-10-13-里程碑四预开发.md:239 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「- 丙交回（Opus，41 分，36.1 万 t」） |
| F45 | records/2026-10-13-里程碑四预开发.md:312 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「- 截戊交回（Opus，122 分，85.4 万」） |
| F45 | records/2026-10-13-里程碑四预开发.md:358 | 事件句不改 | 这是 2026-10-13 预开发记录里当天的现查或待办（I-1.13 与 I-3.15 在 GPU 判器里内核不置位、表是空表对不上），说的是那天的发现；现在 I-1.13 已有内核（原行：「3 在 GPU 判器里内核不置位（判决 m4-t」） |
| F45 | records/2026-10-13-里程碑四预开发.md:664 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「动不动 GPU 判器与内核文件，判出再回 别的会」） |
| F45 | records/2026-10-13-里程碑四预开发.md:709 | 事件句不改 | 这是 2026-09 至 10 月某一天关于 GPU 判器覆盖表「要求全覆盖」或补登的决定与现查，说的是当时怎么定、怎么排的（原行：「行说「GPU 判器内核那一位只判出现规则、不判」） |
| F45 | records/2026-10-13-里程碑四预开发.md:798 | 不相干 | 这一行说的是别的 GPU 判器内核改动、容量缺口、对拍红或键复用的决定，不涉及扫描重建内核缺不缺（原行：「。这是 GPU 判器容量缺口（照「两段没」） |
| F45 | records/2026-10-13-里程碑四预开发.md:799 | 事件句不改 | 这是 2026-10-13 预开发记录里当天的现查或待办（I-1.13 与 I-3.15 在 GPU 判器里内核不置位、表是空表对不上），说的是那天的发现；现在 I-1.13 已有内核（原行：「义行写「GPU 判器下标 95：内核不置位、对拍」） |
| F45 | records/2026-10-18-里程碑三收尾与近期欠账.md:165 | 事件句不改 | 这是 2026-10-18 里程碑三收尾记录里当天的现查与用户定案（差在 I-1.13 因 C821、补 GPU 扫描重建内核、派实现员），说的是那一次发生的事（原行：「.13，C821（GPU 判器没有扫描重建内核）」） |
| F45 | records/2026-10-18-里程碑三收尾与近期欠账.md:166 | 事件句不改 | 这是 2026-10-18 里程碑三收尾记录里当天的现查与用户定案（差在 I-1.13 因 C821、补 GPU 扫描重建内核、派实现员），说的是那一次发生的事（原行：「」（还 C821（GPU 判器没有扫描重建内核）」） |
| F45 | records/2026-10-18-里程碑三收尾与近期欠账.md:239 | 事件句不改 | 这是 2026-10-18 里程碑三收尾记录里当天的现查与用户定案（差在 I-1.13 因 C821、补 GPU 扫描重建内核、派实现员），说的是那一次发生的事（原行：「为底派 C821（GPU 判器没有扫描重建内核）」） |
| M1 | .claude/kb/milestone/04-fourth-txn.md:66 | 要补 | 二·2 行（实现）或「实现进度」要补记：发布返回之后的 discard 已接上产品路径，调用者是 `crates/singlefs-core/src/mounted_session.rs` 的 `discard_the_segments_reusable_as_a_whole`（用户改动的发布与整理显式入口之后各一次，0b926bd3f 进仓）；文件后端真打洞报不支持那一支没有用例仍欠 C857（checks-owed.md:565）。现查：`grep -rn "discard_the_segments_reusable_as_a_whole" .claude records` 零命中，kb 里没有一处记着它 |
| M2 | .claude/kb/decisions-history.md:14539 | 要补 | D26 同日块只记了「第一版取法表」，没记产品路径已接上 discard：补一项「D26 未定项 9 第一版取法表里『产品路径的调用者』一格由『还没有』改为 mounted_session.rs 的 discard_the_segments_reusable_as_a_whole」，与上面 F40 的 42-53 号三语六行同批改（门禁 doc-decisions 对变更史的要求照它的规格） |

## --check-report 输出末行
命令：`python3 scripts/kb/stale-candidates.py --check-report /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-facts/candidates.tsv /home/fy5090/.local/share/singlefs/work/batch-2026-10-21-sync-judge-6/report.md --groups F39,F40,F41,F44,F45`，退出码 0，末行：
  ✓ 报告判全了：201 行候选都有逐行判定
