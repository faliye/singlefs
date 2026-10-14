# m3-c554-c572-r1 云端正推（Sonnet）报告（2026-10-12）

背景材料：`research/prompts/_m3-c554-c572-r1-background.md`（正文 `_m3-c554-c572-r1-body.md`）。分给这条腿的格：B1–B5。立场：从条款原文与 `crates/` 今天的代码推，不替主 agent 采纳或判决。

## 0　读法

- 代码行号一律按冻结副本 `~/.local/share/singlefs/work/m3-c554-c572-r1/repo/`（下文 `crates/...` 即该副本下的路径）；kb 行号是 `.claude/kb/` 下那份文件自己的行号，写进报告前逐句 `grep -nF` 过。
- 没编译、没跑 cargo、没跑重型测试：下面所有「代码怎么走」都是读码推的，没造盘面；要靠跑才能定的写进「要主 agent 跑的命令」。
- 量过的只有一样：B3 的小模型（`research/prompts/m3-c554-c572-r1-sonnet-model/`，复述 `crates/singlefs-core/src/mount.rs` 去重控制流，不读盘），不是入库装置上的数。
- 判定三档：还缺 / 已被吃掉 / 部分。「部分」后面写清哪一半缺、哪一半被哪条定案和哪处代码吃掉。
- 每格的「岔路」只写候选定义、翻面观测、够判条件，不带倾向。

## B1　单盘恢复择中被抛弃的根，暖机落满之前（C554 ①）

**判定：部分。** 「单盘恢复择中被抛弃的根」本身是已定条款写明的设计内结局；「那次挂载里被藏的根的单元被新实例写掉」这一半，在记录读得出时已被可写挂载时的预隔离吃掉；剩下的一半是「根槽与全部记录都读不出、系统配置又没见证」，盘上没有任何痕迹，没有一句条款写它。

推导（各条独立可核）：

1. 择中被抛弃的根是设计内：D16 已定项 8 的暖机把条件写成「新实例的根覆盖两块盘之前不让任何 fsync 返回」，E150 的依据行写明暖机之前一个故障就会落到被抛弃的根。
   - 「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回」（`.claude/kb/decisions/16-发布语义.md:216`）
   - 「E150（回退复用被抛弃的根引用的单元）：无暖机时 1 个故障就落到被抛弃的根上，暖机之后要 4 个」（`.claude/kb/decisions/16-发布语义.md:228`）
   - 幸存盘上没有新实例的根，也就没有那一行实例表：单盘上的择根看不出被抛弃，择根那一侧没有可用的信息（读 D16 已定项 8 推的，没造盘面）。
2. 登记行里 99 个读不出的成因在可写挂载那一次：被藏的根 R 在那次挂载里根槽和单元读回全零，影子账看不见它，写行取落点取到了它的槽（`research/prompts/m3-batch17-code-r1-opus-output.md:284` 起的 2.6 一节；读不出的槽号 1536）。这一数是 2026-10-05 的，早于下面这条定案：
   - 「可写挂载时的预隔离」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
3. 预隔离今天的实现：水位之上、没施加的记录点名的落点进 C 圈。R 与所选那一版同实例，所以它的记录在水位之上：
   - 「> (selected_version.instance, selected_version.checkpoint_txg)」（`crates/singlefs-core/src/mount.rs:1824`）
   - 「let placements_named_by_unapplied_journal_records = match shadow_ledger {」（`crates/singlefs-core/src/mount.rs:3030`）
   - 记录点名的单元与新根段四条指针（树表、映射根、实例表、分配记录树根）都进 C 圈（`crates/singlefs-core/src/mount.rs:1826-1849`）。读码推：登记行里读不出的那个槽 1536 若是 R 的映射根或树表，今天会被 C 圈隔离；这一步要重跑才能定。
4. 剩下的一半：根槽读不出、R 的记录也读不出时，C 圈和影子账的输入里都没有 R，只剩回收门槛的钉（挂载时读坏、读回全零的槽钉在可能住过的最旧 txg）：
   - 「let oldest_valid_root_in_the_ring = roots」（`crates/singlefs-core/src/mount.rs:2980`）
   - 钉只管「已释放、释放代高于钉」的槽。R 自己新分配的槽没被释放过，不受钉管；F 也可以越过读回全零的槽：「读回全零的槽不并进上限、F 可能越过住在全零槽里的有效根（用户认的代价）」（`.claude/kb/decisions/16-发布语义.md:36`）。
   - 这一半在盘上没有痕迹（根槽、记录、系统配置都没见证到它），没有什么可隔离的集合；条款也没写它。D23 已定项 14 射程的已知边角是「挂着时回退的已知边角三样」（`.claude/kb/decisions/23-journal的角色与格式.md:413`），三样里没有这一条。

候选写法（各定义一句）：

- B1-a　不改码：条款加一句射程，写明「R 的根槽与全部记录都读不出、系统配置没见证时无从隔离，单盘恢复在暖机落满之前可能择中单元已被复用的根」。动 kb 一句，`crates/` 零改动；要先在今天的树上重跑登记行 ① 的造法，确认读得出记录的那一半 99 个降到 0。
- B1-b　择根侧：所选根走读失败、系统配置里 c_见证 比它新而「见证到的那一版」的根在另一块盘上时，退到比所选根更旧、不被抛弃的最新一条根。动 `crates/singlefs-core/src/recovery.rs` 的 `newest_root_from_the_version_the_system_configuration_witnessed_up_to_the_chosen_root_to_fall_back_to`（`crates/singlefs-core/src/recovery.rs:5564`）及其三处调用（恢复、可写挂载读阶段、`mounted_read::mount_read_only`）。
  - 代价：它的注释写明现行取法不退到更旧的根，理由是已确认的那一版根槽读回全零时会退过它：「不退到比它更旧的根（`F2_EXACT`」（`crates/singlefs-core/src/recovery.rs:5542`）。
- B1-c　影子账侧退到更保守的集合：对挂载时读坏、读回全零的根槽（`RootRingOccupant::BadAtMount`、`AllZeroAtMount`，`crates/singlefs-core/src/mount.rs:3122-3136`），无法按槽认出引用时，把现行账里全部已释放的落点并进 C 圈（同回退那一支的宽护）。
  - 代价：同函数里记着一次同类退法的量：「384 槽盘上隔离约 370 个槽、600 段里 588 段被拒」（`crates/singlefs-core/src/mount.rs:2157`）。

要交用户的岔路：

- B1-a / B1-b / B1-c 三选一（B1-c 与 B1-a 可叠加）。
- 翻面观测：今天的树上重跑登记行 ① 的造法，「读不出」的判定数是否仍大于 0；大于 0 且成因是「记录也读不出」，B1-a 不够；等于 0，B1-a 够。
- 够判条件：同一造法（见「要主 agent 跑的命令」第 1 条）在冻结副本上跑完 `AB2TALLY` 汇总。
- 推翻本格「部分」判定的现象：重跑得到 `MappingStillUnreadable` 仍有、且读得出记录（则判「还缺」，缺的是 C 圈漏了哪一类）；或重跑全 0 且 R 的记录读不出的变体也全 0（则判「已被吃掉」，意味着有第三条未读到的保护）。

## B2　水位之上没施加的发布账读不出（C554 ②）

**判定：还缺（部分）。** 缺的是一句条款加对应的一处代码；R 自己新分配、记录点名的那一份落点今天已经进 C 圈，所以登记行写的「一个落点都不进 C 那一圈」不再准确，缺的只是账里有、记录没点名的那部分。

推导：

1. `None => continue` 今天有两处，只有一处没有条款撑腰。
   - 「None if readable_records_of_the_publish_leave_a_gap_or_miss_the_last(」（`crates/singlefs-core/src/mount.rs:1932`）：账读不出且记录不齐，走从中央映射现算的那一支（`C_MAPPING`），仍不成拒可写。
   - 「None => continue,」（`crates/singlefs-core/src/mount.rs:1944`）：账读不出且记录齐，什么都不交。这一处是登记行 ② 说的那一处。
   - 「None => continue,」（`crates/singlefs-core/src/mount.rs:2034`）：较新可读根一条记录都读不出、映射现算不成、系统配置里取过号的实例代号比它新。这一处有条款：「现算不成不隔离、不拒（主 agent 2026-10-08 定）」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。
2. 1944 那一处的条款是自指落空的：
   - 预隔离一段把记录齐的情形交给 F1 乙：「那次发布读得出的记录齐（本次发布内序号 1..N 连着、带末条）就不加」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。
   - F1 乙的触发条件写的是记录不齐：「读得出的那几条记录本次发布内序号不连续或缺末条、而它们点名的单元里有一个两份副本都验不过」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）。
   - 实现与之一致：F1 乙的判法里要求记录不齐（`crates/singlefs-core/src/mount.rs:6874`）。于是「记录齐、账读不出」这一形，两边都不接，条款指向的那一判在这一形上不触发。
3. 同一形在邻近一支有处理，对照出不对称：较新可读根（根槽读得出）记录齐、账读不出时走「丙」，把现行账里仍分配的落点并进 C 圈，并登记要并中间那几次发布、要认回退复活：
   - 「txgs_of_the_roots_taking_the_current_account.push(root.checkpoint_txg);」（`crates/singlefs-core/src/mount.rs:2061`）
   - 「placements_still_allocated_in_the_account(current_records);」（`crates/singlefs-core/src/mount.rs:2063`）
   - 根槽读不出的发布走 `placements_in_the_accounts_of_unapplied_publishes`（`crates/singlefs-core/src/mount.rs:1895`），没有这一支。
4. 为什么「记录齐」的账读不出会真走到：账读不出是树表或账根两份副本都验不过，另加根槽读不出（读回全零），至少三处同坏；崩溃镜像上的判法写在 `account_unreadable_on_the_crash_image`。R 自己写的单元由记录点名兜着（`crates/singlefs-core/src/mount.rs:1814` 起的函数；点名项取这次重写的角色，`crates/singlefs-core/src/transaction.rs:1932` 起 `named_roles`），漏的是 R 从所选那一版继承、不在这次重写里的单元：它们在现行账里仍分配，C 圈豁免现行账里仍分配的槽（`crates/singlefs-core/src/mount.rs:1686`），这次挂载里释放、回收之后才需要补隔离，而补隔离只补 C 圈里有的（读码推，没造盘面）。
5. 回收门槛的钉只在 F 不越过时护得住继承单元，读回全零的槽不并进 F 的上限：
   - 「读回全零的槽不并进上限、F 可能越过住在全零槽里的有效根（用户认的代价）」（`.claude/kb/decisions/16-发布语义.md:36`）。这句认下的是 F 越过有效根、回退候选少一个；没有写「它引用的继承单元被重新发出去」。
6. 已有的会红的东西把「拒可写」这条路卡住了：
   - 「"R 的根槽全零、末条点名的树表验不过：R 不施加，只读恢复落到 P"」（`crates/singlefs-harness/tests/writable_mount_refusal_for_an_unreadable_account_with_incomplete_records_judged_on_the_d23_list.rs:176`）所在用例，对记录齐、账读不出的崩溃镜像判「拒可写不合法」（同文件 `WritableMountRefusalVerdict::Illegal {` 在 195 行）。
   - 「记录齐的不走这一支（照被抛弃根那一份不收窄会在 `ur` 那几形上误拒，攻方量到入库 7 条）」（`crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs:7`）。

候选写法：

- B2-a　拒可写：记录齐、账读不出、映射现算不成、重读一次仍不成，取号之前拒（拒因复用 `AccountOfAnUnappliedPublishWithIncompleteReadableRecords` 或另起成员）。
  - 动：`crates/singlefs-core/src/mount.rs` 1932–1944（去掉记录不齐这个前件）、2339 起的函数注释；`crates/singlefs-harness/src/mount_transparency.rs` 的复算判据（现行判「记录齐 ⇒ 拒不合法」）；翻面上面第 6 点两处用例；`crates/mutations.tsv` 里点名「C_MAPPING 判据复算不看记录齐不齐」的那一行。
  - 代价：上面 `ur` 那 7 形翻成被拒（登记在案的数）；其余拒的频率没量。
- B2-b　退到现行账里全部已释放的落点（同回退那一支的宽护）：1944 改成交 `all_released_in_the_current_account`（现在是 `placements_a_rollback_publish_revived_released_in_the_current_account` 里的闭包，`crates/singlefs-core/src/mount.rs:2179` 起，要提成函数）。
  - 动：mount.rs 1895 起的函数与它的两处调用（3040 起）。
  - 代价：同类退法在 384 槽盘上量过「隔离约 370 个槽、600 段里 588 段被拒」（`crates/singlefs-core/src/mount.rs:2157`，那是回退复活的对照臂，不是这一处的数）；宽护卡住准入时发布报 `UserChangeRefused::NoSpaceAfterRaisingTheFloor`（`.claude/kb/decisions/16-发布语义.md:37`）。
- B2-c　复用丙：把 2046–2076 那一支（现行账里仍分配的落点、丙并中间、回退复活）搬到 1944。
  - 动：mount.rs 1895 起的函数加 `roots`、`is_abandoned` 参数并回传走丙的 txg；3040–3086 的调用与「丙并中间」那一步（`crates/singlefs-core/src/mount.rs:2110`）。
  - 代价：回收时补隔离的槽数随现行账里被释放的槽涨，没量；比 B2-b 窄，因为不并全部已释放的槽。
- B2-d　先重读一次再判：1944 前调 `before_the_one_reread.before_rereading()`，再读一遍账，读得出就用，读不出再落 B2-a/b/c 之一。单独不成一个结局，只区分瞬时与持续；代价是 None 时多读一棵账。

要交用户的岔路：

- 候选：B2-a / B2-b / B2-c，B2-d 可叠在任一条前面。
- 翻面观测：(i) `ur` 7 形与上面第 6 点的 Illegal 判是否要翻面；(ii) 384 槽小盘上 B2-b 的隔离槽数与被拒段数；(iii) B2-c 的补隔离槽数。
- 够判条件：三个数都在冻结副本上量出（B2-a 数被翻面的入库用例数，B2-b / B2-c 数 `isolated_slots_per_device` 与被拒历史段数）。
- 推翻本格「还缺」的现象：造出一段记录齐、账读不出、根槽读回全零的历史，今天的树上可写挂载之后继承单元没被重新发出去、池级 checker 不红 I-7.4（则这一形今天已有别的保护）。

## B3　C 那一圈按起点槽去重（C554 ③）

**判定：部分。** 实现偏离条款（条款写在槽的粒度，代码按「盘 + 起点槽」去重），这一半是还缺；后果这一半——被漏的槽有没有合法引用——两条腿此前都没造出，今天也没造出，所以没有可达的红状态。

推导：

1. 条款是槽的粒度：
   - 「只隔离其中只被被抛弃根引用的槽」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）
   - 「只除去现行账里仍分配着的槽」（`.claude/kb/decisions/23-journal的角色与格式.md:392`）
   - 条款没有「按起点槽去重」这一句，也没有「按单元去重」这一句；「隔离按单元还是按槽」条款只写了槽。
2. 实现按起点槽去重，三个圈共用一个集合，先到的赢：
   - 「let mut isolated: BTreeSet<(u32, u64)> = BTreeSet::new();」（`crates/singlefs-core/src/mount.rs:1591`）
   - 「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」（`crates/singlefs-core/src/mount.rs:1641`），同一句在 1665 行；C 圈那一处是「if referenced_by_the_current_account.contains(&key) || !isolated.insert(key) {」（`crates/singlefs-core/src/mount.rs:1686`）。
   - 豁免集也只存起点槽：「.map(|record| (record.device.0, record.slot.0))」（`crates/singlefs-core/src/mount.rs:1573`）、「.map(|(device, slot, _)| (device.0, slot.0)),」（`crates/singlefs-core/src/mount.rs:1587`）。
3. 分配器那一侧本来就幂等，所以 mount.rs 这个集合对「隔离了几个槽」是多余的，它真正决定的是先到的那一份走 `isolate` 还是 `isolate_the_slots_not_allocated`（前者连已分配的槽一起置位，后者跳过已分配的槽，D28 第九项不愿双扣）：
   - 「pub fn isolate_the_slots_not_allocated(&mut self, slot: SlotNumber, span: u64) {」（`crates/singlefs-core/src/allocator.rs:598`）
   - 「if !self.isolated[index] {」（`crates/singlefs-core/src/allocator.rs:614`）
4. 小模型（`research/prompts/m3-c554-c572-r1-sonnet-model/`，复述上面的控制流，不读盘）原样输出：
   ```
   start-key  : [(0, 1544)]
   span-key   : [(0, 1544), (0, 1545)]
   per-slot   : [(0, 1544), (0, 1545)]
   start-key (wide first): [(0, 1544), (0, 1545)]
   exempt by start slot  : [] (候选只罩 1544 时 1545 也被一并豁免)
   ```
   同一起点两份、窄的先到：现行少罩一槽；宽的先到：不少。结果依赖进集合的先后，这是实现偏离条款的直接证据。
5. 真盘上出现过：「改前同形 64/640」（`research/prompts/m3-batch10z-no-code-r1-main-verification.md:28`），被漏的槽没有候选根合法引用，当时判没打中。
6. 池级 checker 为什么没红：它的 I-7.4 被抛弃根那一半是读盘走读、看被抛弃根引用的单元校验和：「有一个单元校验和对不上或头用不了，就是它指着的块被重新分配或抹头了，I-7.4 按这条根判一格红」（`crates/singlefs-checker/src/walk.rs:8494`）。少罩的那一槽若没有任何根引用，读盘走读碰不到它；隔离位是内存里的分配器状态，镜像上没有。所以这一条在镜像层本来就判不了，该加的是挂载层的判据，不是镜像 checker 的。

候选写法：

- B3-a　逐槽去重：`isolated` 与两个豁免集改成逐槽的集合，每个落点只对还没进集合的子段调 `isolate*`。动 `crates/singlefs-core/src/mount.rs` 1573–1695 共 5 处；同步 `crates/mutations.tsv` 第 41 与 3149 行（旧串写的就是现行那句）。代价：集合大小从落点数涨到槽数（单元区槽数以内），无读盘；先到变体的语义变成逐槽。
- B3-b　键带跨度：`(盘, 起点槽, 跨度)`。动同 5 处里 3 处去重句；豁免集仍按起点槽，较宽的被豁免份仍可能少罩（对称的一形还在）。代价最小。
- B3-c　删 mount.rs 的去重集，全交分配器幂等。动 3 处去重句；风险是 `isolate` 与 `isolate_the_slots_not_allocated` 两个变体不再有先到先得，D28 第九项双扣防线（1679–1683 的注释）要重核。
- B3-d　维持现行，条款加一句「同起点不同跨度时较宽一份少罩的槽没有合法引用，理由是……」。要先有不可达的证明，两条腿此前都只是读码推。

要交用户的岔路：

- 候选：B3-a / B3-b / B3-c / B3-d。
- 翻面观测：能否造出「宽的一份被合法候选根引用、窄的一份先到」的历史；造得出则 B3-d 出局。
- 够判条件：挂载层判据——对 C 表给 `[(d,s,1),(d,s,2)]` 与反序两种输入，断言隔离的槽集相同；今天会红（小模型已示）。可放在 `crates/singlefs-harness/src/mount_transparency.rs` 或 `crates/singlefs-core/src/mount.rs` 的单测里，用 `allocator.rs:2254` 起的 `isolated_slots_covered_by` 数。
- 推翻本格「部分」判定的现象：读码时发现某处在 `isolate*` 之外又按槽补隔离了（则偏离不成立）；或造出合法引用的反例（则「后果」那一半也是还缺）。

## B4　被抛弃实例的根重开时全部暂时读不出（C554 机理那一形）

**判定：部分。** 机理今天仍成立，并且有一条入库的钉子用例钉着；已知边角 ② 那一句写的「乙-配置续之后只剩……」落后于 2026-10-07/08 的预隔离，没说清为什么预隔离在这一形上不管用。站不站得住取决于要不要那句话再多说半句。

推导：

1. 条款现状：
   - 「崩溃恢复抛弃的根在重开时全部暂时读不出，影子账读不到它们的账、隔离 0」（`.claude/kb/decisions/23-journal的角色与格式.md:413`）
   - 「乙-配置续之后这一形只剩被抛弃根是系统配置没见证到的那一种」（`.claude/kb/decisions/23-journal的角色与格式.md:413`）
2. 预隔离管不到：C 圈只收记录的 (实例代号, checkpoint_txg) 大于所选那一版的，比较以实例代号为主：
   - 「按实例代号为主比」（`.claude/kb/decisions/23-journal的角色与格式.md:370`）
   - 「> (selected_version.instance, selected_version.checkpoint_txg)」（`crates/singlefs-core/src/mount.rs:1824`）
   - 被抛弃实例的发布，实例代号比所选那一版小，它们的记录在水位之下，C 圈的输入里没有（读码推）。
3. 影子账管不到：它遍历的是读得出的根：「for root in roots.iter().filter(|root| is_abandoned(root)) {」（`crates/singlefs-core/src/mount.rs:1594`）。根槽读不出的被抛弃根不在 `roots` 里。
4. 钉子用例今天仍钉着这一格，且注释写明条款没写：
   - 「vec![(DeviceIdentity(0), 0), (DeviceIdentity(1), 0)],」（`crates/singlefs-harness/tests/unreadable_abandoned_root_slot.rs:284`）
   - 「⚠️ 被抛弃根的根槽读不出这一格该是什么样还没有条款」（`crates/singlefs-harness/tests/unreadable_abandoned_root_slot.rs:18`）
   - 回收门槛的钉护已释放的槽（`crates/singlefs-core/src/mount.rs:2980`、`:3007`），不护被抛弃根自己新分配、没释放过的槽，所以 14 个槽掉出（用例 225–228 行列了槽号）。
5. 「撤故障后新实例写行那次发布取了与被藏根相同的 txg」这一句的含义（`.claude/kb/checks-owed.md:392`）：新实例第一次发布的 txg 下界是读得出的根与记录的最大值加一（`.claude/kb/decisions/23-journal的角色与格式.md:371`），读不出的根和记录不进这个最大值，所以可以撞上被藏根的 txg；根槽位置只由 txg 定，撞上就把被藏根盖掉。
   - 读码推它的两面：撞上 ⇒ 被藏根立刻消失，之后恢复落不到它上面，但盖掉之前它的单元已无保护；避开 ⇒ 被藏根留在环里直到轮转覆盖它，期间恢复有可能落到它上面。所以「新实例的第一次发布避开读不出的槽对应的 txg」按机理不缩小窗口，反而可能拉长（推的，没量；要用 E158 的阳性对照 PC-N 两臂对比才能定）。

候选写法：

- B4-a　维持，条款改写：射程 ② 去掉「乙-配置续之后只剩……」，改写成现状——被抛弃实例的发布在水位之下，预隔离与影子账都不管根槽读不出的那一条，钉子用例钉着。`crates/` 零改动。
- B4-b　C 圈扩到被抛弃发布的记录：记录按最新根的实例表判被抛弃（现成函数 `publish_is_abandoned_by_the_instance_table`，`crates/singlefs-core/src/recovery.rs:1559`，F2 判记录用的就是它）且根不在 `roots` 里的，其点名的落点与记录拼根读到的账也进 C 圈。
  - 动：`crates/singlefs-core/src/mount.rs` 的 `placements_named_by_unapplied_journal_records`（1814）与 `placements_in_the_accounts_of_unapplied_publishes`（1895）的过滤，各加 `newest_table` 参数（`rebuilt_allocator` 里已有，2954 行附近）；3030–3040 的调用；解开时机仍按 `target_for_publish(记录的 txg)` 那一槽被这个进程写掉。
  - 代价：多隔离的槽数随环里还留着的被抛弃发布数涨；入库装置上一次被抛弃发布独占 14 个槽（上面第 4 点两块盘各 14），环里被抛弃发布的条数没量；依赖那些记录还没被记录环转覆盖（覆盖了就回到 B4-a 的状态）；钉子用例 284 行的 `(…, 0)` 要翻面。
- B4-c　拒可写：环里有挂载时读坏（非全零）的槽，其可能住的 txg（`crates/singlefs-core/src/mount.rs:2960` 起 `txgs_ring_slots_bad_at_mount_can_hold` 的结果）落进最新根实例表某一行的被抛弃区间 (Ti, N]，重读一次仍如此，取号之前拒。
  - 动：mount.rs `rebuilt_allocator` 里 2960–3007 之后加一道判；`MountError` 加一个拒因；挂载透明（`crates/singlefs-harness/src/mount_transparency.rs`）加复算。
  - 代价：任何有一个老的读坏槽、又有被抛弃区间的池每次可写挂载都拒，只剩只读（同 C596 那一类，`.claude/kb/decisions/23-journal的角色与格式.md:412` 已写的代价）；拒的频率没量。
- B4-d　收严新实例第一次发布的 txg：下界再加「读坏槽能住的最大 txg」。按上面第 5 点机理不缩窗口，列出来供用户看，不是推荐项。动 `warm_up_publish_txgs` 一带（`crates/singlefs-core/src/mount.rs`）。

要交用户的岔路：

- 候选：B4-a / B4-b / B4-c / B4-d。
- 翻面观测：(i) 在钉子用例装置上预期 B4-b 的隔离槽数（读码推 14 槽 / 盘，没跑）；(ii) E158 阳性对照 PC-N 在「撞上」与「避开」两臂的 `hidden_in_ring_after` 与恢复落点。
- 够判条件：(i) 在冻结副本拷贝上改 B4-b 的过滤跑钉子用例，得隔离槽数；(ii) 同一造法重跑 PC-N 两臂。
- 推翻本格「部分」判定的现象：造出「被抛弃实例的记录在水位之下」之外的第二条保护，使钉子用例 284 行不再是 0（则「已被吃掉」）；或重跑发现登记行的历史（新实例的根全坏之后恢复落到被抛弃的根）在今天的树上被可写挂载的拒挡在更早一步（则这一形不可达，判「已被吃掉」）。

## B5　回退目标引用隔离单元（C572）

**判定：还缺。** 这一格缺的是一个检查，不是一句条款：条款明说「在任何写之前拒」那一格的理由是「cur 的账里仍分配的单元分配器不会发出去」，隔离单元恰是这句话的反例；实现按条款写，反例没有被任何用例碰过。

推导：

1. 条款的理由句与隔离单元的定义：
   - 「cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）
   - 「任一份核出对不上，这个单元在每块盘上的分配记录都留在」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）：隔离单元是「已经核出一份坏的、两块盘的记录一起留在已分配」的单元。理由句只保「槽不会被发给别人」，不保「两份都读得出」。
   - 逐盘验为什么要有：「1 就是fsync失败后 两个盘掉一个盘。这个不能认，这个违背我们数据安全的承诺」（`.claude/kb/decisions/23-journal的角色与格式.md:445`）。
2. 实现按条款：
   - 同槽同代的记录落在「两版共用」分支，什么都不做：「Some(_shared_by_both_versions) => {}」（`crates/singlefs-core/src/mount.rs:7743`）；已释放的才进复活集（`:7720`）；分配代不同才拒（`:7732`）。
   - 逐盘验只读复活集里的单元：「every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;」（`crates/singlefs-core/src/mount.rs:8107`）。
3. 回退之后隔离单元会重新被用户引用：新根的用户可见映射条目取回退目标的：「let mapped_units = target_version」（`crates/singlefs-core/src/mount.rs:7898`）。回退不读数据单元的内容：「读不出的数据单元照抄它的位置项、不读内容，挂载照常，读到那个文件时才报错」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`，说的是从盘上重建上一版）。
4. 隔离单元在 cur 里有什么特征：两块盘的记录一起留在已分配，分配代是原来的，映射条目已去掉，所以「cur 的用户可见引用集里没有它，而账里有一条同槽同代的已分配记录」。
   - 两块盘的记录一起留：「if !some_copy_failed {」（`crates/singlefs-core/src/transaction.rs:3721`）之后交回两份；`release_leaving_the_record_allocated_on` 对 `devices_whose_record_stays_allocated` 里的盘不动（`crates/singlefs-core/src/allocator.rs:1865`）。
   - cur 的用户可见引用集今天已经在算，只是在逐盘验之后：「user_visible_units_through_the_mapping(current).map_err(not_releasable)?」（`crates/singlefs-core/src/mount.rs:8122`）。
   - 重挂之后分配记录里没有「隔离」标记，只剩「已分配而没有根引用」这一个特征；池级 checker 认隔离记录也是这个特征：「已分配而没有根引用的记录，checker 自己读它罩住的那一份」（`crates/singlefs-harness/tests/release_checksum_quarantine.rs:18`）。
5. 没有用例碰过：`grep -n 'quarantin' crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 在冻结副本里命中 487、1854、1858、1864 四行；487 行是「assert!(rolled_back.quarantined_user_visible_copies.is_empty());」（`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:487`），1854–1864 说的是固定点单元（记账树根）的隔离，不是用户可见单元。
6. 一份坏拷贝的单元回退之后：只一份坏时，另一份好，读得成但只剩一份冗余，丢那块盘就丢数据；两份都坏时回退「成功」而读该文件报错。「读不出」的判定在隔离那一刻是两次读都没对上（`crates/singlefs-core/src/transaction.rs:3672` 起的函数）。

候选写法：

- B5-a　隔离单元也进逐盘验：`resurrection_against_the_current_account`（`crates/singlefs-core/src/mount.rs:7670`）加一个参数「cur 的用户可见引用集」，7743 分支里 (盘, 槽) 不在其中的记录，并进 `units_to_check_on_every_device`（7746 的条件改成「复活 或 隔离」）；调用方 `roll_back_by_a_forward_publish`（7998）把 8121–8140 取引用集那一步提到 8098 之前。拒因复用 `ResurrectedUnitCopyUnreadableOrMismatched` 或另起成员（`RollbackError`）。
  - 代价：读盘 = 回退目标引用的隔离单元数 × 2 份 × 2 槽；平时 0 个单元。拒的频率 = 回退目标引用着隔离单元、且那单元今天仍有一份读不出或对不上。
- B5-b　回退目标引用隔离单元直接拒：识别同 B5-a，不读盘，新拒因。代价：比 B5-a 多拒一种——隔离单元的两份今天都读得好（隔离时是瞬时坏读的残留）；少读盘。
- B5-c　维持现行，改理由句：把「cur 的账里仍分配的单元……」改成「……其中隔离单元不在逐盘验之内，回退后它按隔离那一刻的样子（至少一份坏）重新被引用」，写成已知边角。`crates/` 零改动。
- B5-d　只拒两份都坏、放行一份坏：识别同 B5-a。与「有一份不行就拒」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）口径不同，是改口径，不是补漏。

要交用户的岔路：

- 候选：B5-a / B5-b / B5-c / B5-d。
- 翻面观测：用 `crates/singlefs-harness/tests/release_checksum_quarantine.rs` 第 2 个用例的做法造一个只一份坏的隔离单元（该文件第 10 行描述），再回退到引用它的那一版：今天回退成功、之后读该文件走好的那一份（读码推，没跑）；候选改完后回退该被拒（B5-a/b）或被放行（B5-c/d）。
- 够判条件：这一条用例在今天的树上通过（回退成功）、候选改完后翻成被拒；加两份都坏的变体。
- 推翻本格「还缺」的现象：上面造法在今天的树上回退就被拒（则有别的检查吃掉了它，判「已被吃掉」，并要指出是哪一处）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| B1 | 部分 | 单盘择中被抛弃的根是 D16 已定项 8 / E150 写明的设计内结局；被藏根的单元在记录读得出时已被预隔离罩住（今天的树上没重量）；剩「根槽与全部记录都读不出、系统配置没见证」一半盘上无痕迹、没有条款 |
| B2 | 还缺（部分） | 记录点名的落点已进 C 圈；记录齐、账读不出时 `mount.rs:1944` 什么都不交，预隔离一段指向的 F1 乙在记录齐时不触发，条款自指落空；邻近的较新可读根那一支有「丙」，这一支没有 |
| B3 | 部分 | 条款写槽、实现按起点槽去重，偏离成立（小模型示一槽之差、与进集合先后有关）；被漏的槽有无合法引用至今没造出，镜像层 checker 本来判不了，该加的是挂载层判据 |
| B4 | 部分 | 机理成立、钉子用例钉着（14 槽掉出、计数 0）；被抛弃实例的记录在水位之下，预隔离管不到；射程 ② 的旧句落后；「避开读不出槽的 txg」按机理不缩窗口 |
| B5 | 还缺 | 隔离单元落在「两版共用、不动」分支，不进逐盘验；回退后用户重新引用它，至少一份坏；无任何用例碰过 |

## 要主 agent 跑的命令

1. B1 的重量（登记行 ① 的造法在今天的树上还剩几个读不出）。这是攻方腿模型 `research/prompts/m3-batch17-code-r1-opus-model/` 的复跑，只编跑 harness 档与它自己的原型 crate；`jia` 臂不打补丁。它的探针在今天的树上能不能编过我没核（那份探针写于 2026-10-05，今天的 `mount.rs` 之后改了很多），编不过要先改探针，那是另一件事。
   - 工作目录：仓根。冻结副本拷一份（拷之前 `find crates -type f -exec touch {} +`），环境变量 `FROZEN_TREE=<拷贝>`；`<W>` 是新的空目录。
   - 命令（经内存包装，线程数该模型写死 15）：
     ```
     bash scripts/hosts/run-with-memory-cap.sh <上限> env FROZEN_TREE=<拷贝> bash research/prompts/m3-batch17-code-r1-opus-model/rerun.sh build-jia <W>
     bash scripts/hosts/run-with-memory-cap.sh <上限> env FROZEN_TREE=<拷贝> bash research/prompts/m3-batch17-code-r1-opus-model/rerun.sh tables <W>
     bash scripts/hosts/run-with-memory-cap.sh <上限> env FROZEN_TREE=<拷贝> bash research/prompts/m3-batch17-code-r1-opus-model/rerun.sh ab2 <W>
     bash research/prompts/m3-batch17-code-r1-opus-model/rerun.sh summary <W>
     ```
   - 看哪几行：`<W>/summary.txt` 里 `[ab2-jia]` 一行（按类的计数）与 `<W>/out/ab-jia-*.log` 里 `AB2CHOSE` 行有没有 `ro:Open(Walk(MappingStillUnreadable`；2026-10-05 的数是 464 个择中、99 个读不出（`research/prompts/m3-batch17-code-r1-opus-output.md:284` 起的 2.6 一节）。
2. B4 的 PC-N 两臂（撞上 / 避开）用 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs`，是 checker 档包的二进制，按共用约束归重型，我没起；是否值得起由主 agent 定。

## 复核留痕

- 冻结副本六个点名文件的 sha256 现核（2026-10-12，命令 `sha256sum -c research/prompts/m3-c554-c572-r1-snapshot/frozen-copy-sha256.txt`，在副本根下跑）：六行全 OK。
- 小模型复跑（`research/prompts/m3-c554-c572-r1-sonnet-model/tools/rerun.sh`，经 `scripts/hosts/run-with-memory-cap.sh 1G`，排队 178 秒后起跑）末行原样：`compare: b3_model.out 与 out/ 里那份逐行相同`，退出码 0。
  - `sha256sum`：`7f7423d2bfb8d14e641f0a09dd2da7b13ebaf0ad1b3f4d68319e23bd6e575376` `b3_model.py`；`0af1a84e8a723f57e6fb38ddca8f6f3fb9d45c6385f5634ba66dcdb67df5b2ce` `out/b3_model.out`；`2406ccf230a55fa4d81532ac17653647ac62877c9d8d9c3c9a70fe37fdcd27ea` `tools/rerun.sh`。
  - 这个模型只复述我读到的控制流，不是被测代码本身；它会红的反例是把 `start_key` 换成 `span_key`，输出里第一行就变成两个槽。
- 现查的几条 grep（冻结副本里）原样：`grep -nF 'None => continue,' crates/singlefs-core/src/mount.rs` 命中 `1944` 与 `2034` 两行；`grep -nF 'isolated.insert(key)' crates/singlefs-core/src/mount.rs` 命中 `1641`、`1665`、`1686` 三行。

## 没做什么

- 没编译、没跑 cargo、没跑任何重型测试；B1 的重量、B4 的 PC-N 两臂、B2 / B5 的反例历史都没造盘面，所有「代码怎么走」是读码推。
- 没替主 agent 采纳或判决；每格的岔路没带倾向。
- 没读攻方腿这一轮的任何东西（禁读清单）。
- 没核池级 checker 对「回退后重新被引用的隔离单元」怎么判（B5 之后它会不会红 I-4.8 一类没读）。
- 没量 B2-c、B4-b 的补隔离槽数与 B2-a 在 `ur` 7 形之外的拒频率。
- 没核 `crates/mutations.tsv` 里对应行在候选改完后要怎么改，只点了行号（B3 的第 41、3149 行）。
- 没读 `mount_transparency.rs` 的复算判据全文，B2-a 动它的说法来自测试文件头与函数名。
