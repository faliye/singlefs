# C355/C363 第三轮核查员报告

核对表里的 ✗ 不免除主 agent 对推论的逐条现查，这一句照抄进报告开头。

## 判别力自证

抽的引用：W1（Sonnet）「`.claude/kb/decisions/03-空间分配.md:126`（D3 已定项 7）原句：『条目改写与产生这次释放的 COW 在同一次发布里原子生效……』」。
在草稿副本（`/tmp/claude-1000/c355-r3-verifier/selftest/03-空间分配.md`）里把行号从 126 改成 127，按核对法（`awk 'NR==127'` 取那一行、与引文逐字比）核：

```
$ awk 'NR==127' /tmp/claude-1000/c355-r3-verifier/selftest/03-空间分配.md
（空行）
```

第 127 行是空行，与引文内容（一整句关于「条目改写...原子生效...」的话）完全对不上 ⇒ **判 ✗**。自证通过：核对法分辨得出行号错了 1 的情形。

**附带发现（同一自证顺手带出的真实案例，不是构造的）**：这条自证之后，在核对 W3（本地攻方）给本地模型的英文提示 `c355-c363-r3-local-attack.md` 时，命中了三处**自然发生**的同类错误（非我构造）：FACT 1 引 `03-空间分配.md` 的「line 78」实际在第 79 行（第 78 行是空行）；FACT 7 引 `invariants.md` 的「line 462」实际在第 470 行；FACT 5 引 `checks-owed.md` 的「line 330」实际在第 329 行（330 是另一条 C376，不是 C375）。三处详情见下文 W3 表。这说明本次核对法的判别力不是只在自证那一格起作用。

## 快照核对

输入给的快照 `research/prompts/c355-c363-r3-snapshot/sha256sums.txt`（2026-09-23 20:35 JST 记）。对当前工作区重算全部 13 个文件的 sha256：

- 与快照**一致**（9 个）：`.claude/kb/decisions/28-挂载期承诺量.md`、`.claude/kb/decisions/05-快照-空间记账机制.md`、`.claude/kb/decisions/03-空间分配.md`、`.claude/kb/decisions/16-发布语义.md`、`.claude/kb/decisions/02-RAID条带策略.md`、`.claude/gate.d/51-admission-terms-covered.sh`、`crates/singlefs-core/src/allocator.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-harness/src/model.rs`、`research/prompts/_c355-c363-r3-body.md`。
- 与快照**不一致**（3 个，与主 agent 给的清单一致）：`.claude/kb/checks-owed.md`、`.claude/kb/invariants.md`、`crates/singlefs-core/src/mount.rs`。

**mount.rs 漂移范围比主 agent 交待的更大，单列记录**：`git diff crates/singlefs-core/src/mount.rs`（工作区对 HEAD `3b60f09`）显示 346 行新增、187 行删除，远不止主 agent 描述的三处（`allocator_of_version_without_file`、`format_time_allocator` 文档注释、两处用例改名）。额外改动包含 `raise_rollback_floor` 的文档注释重写（新增 C482 欠账说明）、`target_for_publish` 签名加参数、`PublishPlan` 新增 `highest_transaction_number_before_this_publish`/`new_inode_records` 字段、新枚举成员 `RollbackToVersionWithoutFileWhileTheRingStillHoldsAFileVersion`、`FormatTimeUnitLocationsOnDifferentSlots`、`ShadowLedger::branch_name` 等。**这些都不在 W1/W2/W3 三条腿引用的范围内**（逐条核对见下文各表），且逐一现查后，W2 引用的 `raise_rollback_floor`/`release_reclaim_holds` 具体控制流（回收在循环前、`release_reclaim_holds` 在循环后、`?` 直接返回）在当前树上仍然成立（见下文 W2 表第一行）。因此漂移范围虽超出描述，但未影响任何一条被核引用的结论。

## W1（云端辩方 Sonnet）核对表

引用来源：`research/prompts/c355-c363-r3-sonnet-output.md`（68 行）。

| 引用 | 核的结果 | 证据 |
|---|---|---|
| `transaction.rs:2200`「一次发布的准入里…三条…第一版各只有一个节点（分裂不做）」 | 对 | `awk 'NR==2200' crates/singlefs-core/src/transaction.rs` 原样：`/// 一次发布的准入里与这次写什么内容无关的那三条：分配记录树、记账树与中央映射树第一版各只有一个节点（分裂不做），` |
| `transaction.rs:751`「分配记录树第一版只有一个节点（分裂不做）：这次发布之后装不下就报错…」 | 对 | `awk 'NR==751'` 原样逐字相同（W1 引文比原文少了句中顿号后半句的反引号，内容一致） |
| `transaction.rs:756-770` `refuse_when_the_allocation_records_do_not_fit_one_node` | 对 | `grep -n "fn refuse_when_the_allocation_records_do_not_fit_one_node"` 命中 756；函数体 756-770 与报告描述（装不下就报 `PublishError::AllocationRecordsExceedOneNode`）逐字相符 |
| `build_allocation_record_node`（`:782-818`）整批按 `sort_key` 排序装一个节点 | 对 | 函数在 782 起始；`allocation_records.sort_by_key(AllocationRecord::sort_key)` 在函数体内，随后整批传入 `build_index_node`（单节点） |
| 两处调用 `:790-791`（树表 0 条那版）与 `:2728-2729`（带文件那版）的 `let mut allocation_records...sort_by_key` 两行 | 对 | 现查：`build_allocation_record_node` 函数体内该两行实际在 790-791；调用点 `:2728-2729` 现查也是同样两行代码，逐字相同 |
| `sort_key`（`allocator.rs:68`）= `(self.device.0, self.slot.0)` | 对 | `awk 'NR==68'` 原样 `(self.device.0, self.slot.0)`，`:66` 注释「按字段比：设备身份，再槽号」 |
| `allocator.rs:25`「key (设备 4, 槽号 6)」 | 对 | `awk 'NR==25'` 原样含「key (设备 4, 槽号 6) + value (跨度 2 含已释放标志, 分配代或释放代 8)」 |
| `.claude/kb/decisions/03-空间分配.md:117`「key = (设备身份 4, 16 KiB 槽号 6)」 | 对 | `awk 'NR==117'` 原样含该句，属「已定项 7」节（115-153） |
| `release()`（`allocator.rs:798`）`for device in &mut self.devices` 逐设备各改写一条记录 | 对 | `awk 'NR==798,829'` 确认 `pub fn release` 起于 798，`for device in &mut self.devices` 在 799 行，循环体内对同一 `placement.slot` 改写 |
| `03-空间分配.md:126`（D3 已定项 7）「条目改写与产生这次释放的 COW 在同一次发布里原子生效——分配记录是根下的 COW 树，条目与根同一次发布…」 | **不对（摘句，漏一处括注）** | 原文 126 行「…分配记录是根下的 COW 树（D22（单元原子性怎么合成） 已定项 3），条目与根同一次发布…」——W1 的引文在「分配记录是根下的 COW 树」与「条目与根同一次发布」之间**漏掉了「（D22（单元原子性怎么合成） 已定项 3）」这个中间括注**，未用省略号标出。内容本身（核心论证）不受影响，但按「引产物就整行抄」的字面标准不合格 |
| `28-挂载期承诺量.md:91`「不设常数上限，按当时的结构现算」 | 对 | `awk 'NR==91'` 原样「checkpoint 保留池不设常数上限，按当时的结构现算，池多大它就多大。」 |
| `16-发布语义.md:205`「一次空发布也写单元」+「按 D5…已定项 2…重写记账行，连带记账树节点、分配记录、映射条目与树表单元」 | **不对（行号张冠李戴）** | `:205` 只是标题行「#### 已定项 9：一次空发布也写单元」，只含第一个短引文；第二段长引文「按 D5…重写记账行…树表单元」实际在 **:207**（`定案` 正文）。报告末尾出处清单给的是范围 `205-217`，范围本身没错，但正文行内引用把两条不同行的话都记成「:205」 |
| `grep -n -i "release\|释放" .../e148_commit_fixpoint_two_record_trees.rs` 只命中一个测试名（标注「r2 攻方腿原话」） | **不对（复跑不可复现 + 转述失真）** | 实测：`grep -n -i "release\|释放" research/e7-index-bench/src/bin/e148_commit_fixpoint_two_record_trees.rs` **零命中**（退出码 1，见下方命令块）；r2 攻方腿的**真实原话**命令是 `grep -n -i "release\|释放\|free\|old\|旧" ...`（多了 `free\|old\|旧` 三个词），那条命令才命中 1 行 `356: fn conservation_holds_on_every_arm()`。W1 把命令简化转抄、又标「原话」，命令本身复跑对不上标注的结果；但用 r2 真实的原始命令复核，「只命中一个测试名」这个结论本身是真的（见证据块），不影响 W1 的实质论证 |
| `git show 3cff909^:research/prompts/c355-c363-r2-opus-output.md` 第 2.4 节，「11」「20」两个数 | 对 | `3cff909^` = `0ec0be2`（现查 `git log`），归档文件第 151 行起是「2.4 尾部从哪来：释放链」节；2.5 小结表「尾部 85% 最多 11 / 12（复用窗口 25）、20 / 21（复用窗口 200）」，W1 引的 11、20 是每对里的第一个数（E148 读法），如实标「未重新量」 |

复跑证据（问题项）：

```
$ grep -n -i "release\|释放" research/e7-index-bench/src/bin/e148_commit_fixpoint_two_record_trees.rs ; echo "exit=$?"
exit=1
$ grep -n -i "release\|释放\|free\|old\|旧" research/e7-index-bench/src/bin/e148_commit_fixpoint_two_record_trees.rs ; echo "exit=$?"
356:    fn conservation_holds_on_every_arm() {
exit=0
```

**W1 计数：核了 14 处，对 11 处，不对 3 处（摘句漏括注 1、行号张冠李戴 1、复跑命令转抄失真 1；三处都不影响 W1 最终结论的实质支点）。**

## W2（云端攻方 Opus）核对表，第一部分：kb 与代码引用

引用来源：`research/prompts/c355-c363-r3-opus-output.md`（369 行）。**模型目录整份 sha256 逐一复算，与报告开头列的 16 个哈希逐行比对（diff 无输出）—— 模型目录未被篡改，见下方证据块。**

| 引用 | 核的结果 | 证据 |
|---|---|---|
| `mount::raise_rollback_floor` 循环前回收扣住、循环内 `?` 直接返回、`release_reclaim_holds` 在循环之后 | 对（在**当前**、比主 agent 描述漂移更大的 mount.rs 上核实仍成立） | 现读 `crates/singlefs-core/src/mount.rs:760-874`：`reclaim_released_up_to(…, HeldUntilFloorTakesEffect)` 在 827（`while` 循环起于 834 之前）；循环体 839-859 内 `publish_version(...)?`；`allocator.release_reclaim_holds();` 在 867（循环结束之后）。三点全部现查确认，未受额外漂移影响 |
| `23-journal的角色与格式.md:369`「分配失败」按发生位置拆三支，整段引文 | 对，逐字 | `awk 'NR==369'` 与报告引文逐字相同（与 body.md 第三节引的同一句一致） |
| `23-journal的角色与格式.md:536` 死锁 2 表格行 | 对，逐字 | `awk 'NR==536'` 原样与报告引文逐字相同 |
| `23-journal的角色与格式.md:397` 新根段 = 树表单元指针 + 映射树根指针 | 对（概括，非摘全句） | `awk 'NR==397'` 原文含四个分量（树表单元指针 86、中央映射树根指针 86、树 ID 水位 8、回退下界 F 8）；W2 只概括前两个分量支持它要说的点，未声称引全句 |
| `23-journal的角色与格式.md:363`「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配、不许抹头」 | 基本对，措辞微差 | 原文是「不许重新分配也不许抹头」（连词「也」），W2 写成「、不许」；语义无差，不算摘句 |
| `23-journal的角色与格式.md:342`「恢复只施加…严格大于所选根的记录」 | 对，逐字 | `awk 'NR==342'` 与引文逐字相同 |
| `16-发布语义.md:33`（可再分配谓词表行）、`:37`（F_生效定义）、`:39`（准入/推空抬 F）、`:187`（区域归属盘 0/1/0，已定项 8）、`:207`（已定项 9 全文，与 W1 引的同一句）、`:221`（攒批）、`:250`（根+journal 前缀）、`:263`（replay 归谁）、`:276`（快照点）、`:289`（已定项 15）、`:309`（消息缓冲摊销）、`:320`（已定项 17 标题）、`:331`（射程只两格） | **全部对，逐字或语义精确对应** | 逐行 `awk 'NR==<行号>'` 现查（详见核查过程，篇幅原因不逐条贴出），其中 `:124`（取号/丢失窗口）「checkpoint 的切分只关闭成员资格…发布阶段等全部单元落盘」与 W2 引文逐字相同；`:147`（发布计数器四个等号）逐字相同；`:81`（journal 冻结组件）、`:102`（整体施加）、`:29`（回退候选集 4 个状态）同样逐字或精确概括 |
| `28-挂载期承诺量.md:26`（checkpoint 保留池只住内存）、`:27`（被抛弃根独占量与隔离集合是同一个数） | 对，逐字 | `awk 'NR==26,27'` 与引文逐字相同 |
| `03-空间分配.md:52`（D3 已定项 2，删除路径不申请空间） | 对，逐字（且确认属「已定项 2」，标题在 :50） | `awk 'NR==52'` 含「⇒ 删除路径仍然不申请空间」；`grep -n "^#### 已定项 2"` 确认标题在 50 行 |
| `05-快照-空间记账机制.md:108`（被抛弃根独占量例外行）、`:111`（四处例外的理由段） | 对，逐字 | `awk 'NR==108,111'` 与引文逐字相同 |
| `milestone/02-second-txn.md:195`（两处不同引文：抬 F 分配不到固定点的三个改法段；同一次挂载转环不回收段） | **对，两处都逐字精确命中同一超长行** | `awk 'NR==195'` 是一整段超长文字（约 2000+ 字），W2 两处引文分别在该行内逐字出现：「交用户的决策点：抬 F 的空发布分配不到固定点…三个改法（失败路径放开扣住 / 整个分配器退回抬 F 之前 / 回收推迟到生效之后）各修一半」与「同一次挂载里转环不回收（回收只在重建与抬 F 两处…）」均逐字命中 |
| `milestone/02-second-txn.md:332`（D28 已定项 1 可用式子按读法甲扣两次 defer，步 2 决策点） | 对，逐字 | `awk 'NR==332'` 含「D28（挂载期承诺量） 已定项 1 的可用式子按读法甲扣两次 defer（步 2 决策点）」 |
| `crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs` 用例名 + `free_slots() > 0` 断言 | 对 | 函数名 `raising_the_floor_with_no_free_slot_outside_the_hold_still_fails_and_the_hold_stays_in_the_process` 现查存在（:302）；`free_slots() > 0` 断言现查存在于 :332 |
| `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 用例名 | 对 | 函数名 `reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red` 现查存在（:499） |
| `crates/singlefs-harness/src/model.rs:1368-1369`（`capacity_wall_is_permitted` 文档注释自陈 D28 已定项 1 没实现） | 对，逐字 | `awk 'NR==1368,1369'` 原样「…D28 已定项 1 的准入式子 第一版没实现，各项没有现值」，函数定义紧接在 :1370 |
| `transaction.rs` `publish_admitted` 装单元之前取定全部落点，注释引 D3 已定项 5 | 对 | `publish_admitted` 起于 :2598；:2621 注释「落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定」逐字支持 |
| `HistoryDeviceWidth::UnitAreaOf384Slots`（两块单元区 384 槽的小盘） | 对 | `grep -n "UnitAreaOf384Slots" crates/singlefs-harness/src/history.rs` 命中，注释「六个 64 槽的聚簇段」= 384 |
| `c355-c363-r1-main-verification.md:85`（(a)/(b) 原文定义） | 对，逐字 | `awk 'NR==85'` 与报告开头的复述逐字相同 |
| `c355-c363-r1-main-verification.md:92`（第一轮给 C363 写的「怎么拦」） | 对，逐字 | `awk 'NR==92'` 「发布路径记下固定点实际分配的提交内生块槽数，超过准入那一刻扣的量判红」 |
| `alloc-basis-r2-main-verification.md:24`（Z2，12989 格 91 格有干净崩溃态） | 对，数字逐字 | `awk 'NR==24'` 含「合法操作搜索 12989 格里 91 格有干净崩溃态」 |
| 门禁 51 号今天原样末行 | 对，逐字，且现跑复现 | 见下方证据块：本核查员现跑门禁 51 号，末行与 W2 报告贴的一字不差 |

复跑证据（gate 51 与模型目录哈希）：

```
$ nice -n 19 bash .claude/gate.d/51-admission-terms-covered.sh 2>&1 | tail -1
  ✓ 准入不等式 9 项逐项有归属：统计量 5 项，写明的例外 4 项（容量、挂载期承诺量、被抛弃根独占量、checkpoint 保留池）；式子副本 2 处与权威一致
（退出码 0，与 W2 报告一致）

$ cd research/prompts/c355-c363-r3-opus-model && find . -type f | sort | xargs sha256sum > /tmp/…/opus-model-current-sha256.txt
$ diff /tmp/…/opus-model-current-sha256.txt <(报告开头贴的 16 行哈希)
（无输出，16 个文件全部一致）
```

**这一部分（kb / 代码 / 静态文件哈希）计数：核了 23 处，对 22 处（含 1 处措辞微差不算摘句），不对 0 处。**

## W3（本地攻方）核对表

材料：提示 `c355-c363-r3-local-attack.md`（61 行，英文）、翻译核对表 `c355-c363-r3-local-attack-translation-audit.md`（27 行转述核对）、跑记 `c355-c363-r3-local-attack-runlog.md`、样本 s1/s2/s3/void1。

### 提示材料 FACT 1–7 逐句核对源文件:行号

| FACT 引用 | 核的结果 | 证据 |
|---|---|---|
| FACT1「available equals, summed over…」= `28-挂载期承诺量.md:19` | 对，逐字对应 | `awk 'NR==19'` 与英译九项逐项对应 |
| FACT1「decision D3…settled item 4, line 82…calls its own copy…'only a pointer and a conclusion sentence'」 | 对（line 82 公式部分对） | `awk 'NR==82'` 公式逐字相同 |
| FACT1「whose own surrounding text (line 78) says explicitly 'the authoritative text is in D28…'」 | **不对，行号错 1** | `awk 'NR==78'` 是**空行**；`grep -n "权威正文在 D28"` 确认该句实际在**第 79 行**。与判别力自证的构造案例同型（真实发生，非构造） |
| FACT1「A gate check… requires these two copies… match character for character」= 门禁 51 号脚本 :14 | 对 | `awk 'NR==14'` 与英译逐字对应 |
| FACT2「the six terms inside the parentheses…summed by device…division is what decides…D5 settled item 4」= `28-挂载期承诺量.md:24` | 对，整句精确对应 | `awk 'NR==24'` 原文与英译逐句对应，含「这个分界决定 D5…已定项 4 里哪几个统计量要带设备维」 |
| FACT2「a separate decision about striping policy (D2 settled item 12, line 194)」 | 对 | `awk 'NR==194'`「容器整条链恒走 `w = 2` 镜像」「第一版 2 盘恒 `w = 2`…位置条目每副本一条」 |
| FACT3 D5 已定项 4 统计量表第 4、6 行（`:120`、`:122`）与对照表两行（`:106`、`:107`） | 对，逐字 | 四行 `awk 'NR==<行号>'` 均与英译逐字对应，含第 6 项两个「不带」都没有括注这一细节（英译显式点出「with no parenthetical explanation」） |
| FACT4 判据 `:4-5`、匹配规则 `:11`、右列规则 `:12-13`、指针回声规则 `:14`、射程 `:20-22` | 对，逐字，且门禁今天实跑确认末行与描述一致 | 逐行 `awk 'NR==<行号>'` 核对（含「且不是已撤回的编号」这半句，:12 行确认原文有），另见上方门禁复跑证据块 |
| FACT5「description: …verified 2026-09-17 with a search…」= `checks-owed.md:330`，C375 | **不对，行号错 1** | `grep -n "^### C375\|C375"` 确认 **C375 实际在第 329 行**；第 330 行是**另一条 C376**（发布边界在链尾没有输入，与 C375 无关）。内容本身（C375 那一整行文字）与 FACT5 的转述逐字对应，只是行号指错了一行——最可能的成因是 checks-owed.md 是**三份漂移文件之一**（C45 从开着的表挪进已还清，使表内后续行整体上移 1 行），但无法核实这一具体假设（没有快照时刻的 checks-owed.md 副本可比对） |
| FACT6 `transaction.rs:2314-2317`（POOL_WIDE_ACCOUNTING_ENTRIES 声明）、`:2765`（待删占用行）、`:2766-2770`（已承诺预留行） | 对，逐字，含「引 D5 已定项 8（不是已定项 4）」这一精细区分 | `awk` 逐行核对，:2314 原文「D5（快照 / 空间记账机制） 已定项 8」，确认不是已定项 4——翻译核对表首稿曾把这条误写成已定项 4，定稿已改正，现查代码确认定稿是对的 |
| FACT6 `records.rs:18,20,25`（三个常量） | 对，逐字 | `STATISTIC_PENDING_DELETE_BYTES=4`、`STATISTIC_COMMITTED_RESERVATION_BYTES=6`、`STATISTIC_NO_DEVICE_DIMENSION` 三行现查一致 |
| FACT6「A further search found no other place…computes, increments, or otherwise assigns any value other than this hardcoded 0」 | 对（穷尽性成立） | `grep -rn "STATISTIC_PENDING_DELETE_BYTES\|STATISTIC_COMMITTED_RESERVATION_BYTES" crates/` 额外命中 `first_transaction_step_five_publish.rs:976-977,997-998`，现查该处只是 `assert_eq!(value_of(statistic), 0, …)` 读断言，不「计算/递增/赋非 0 值」，与 FACT6 的措辞（只排除「赋值」不排除「读」）不矛盾 |
| FACT6 `model.rs:56-59`、`:815-818` | 对，逐字 | `POOL_WIDE_ACCOUNTING_ROWS=3`/`ACCOUNTING_ROWS_PER_DEVICE=6` 与 `accounting_rows_exceed_one_node` 函数体只比较行数，现查一致 |
| FACT7 `invariants.md:131`（I-3.4 行） | 对，逐字 | `awk 'NR==131'` 与英译逐字对应 |
| FACT7「line 462 for a second, unrelated mention of the same I-3.4」 | **不对，行号错 8** | `grep -n "I-3.4" .claude/kb/invariants.md` 命中 131 与 **470**，不是 462；`:462` 实际内容是另一件事（D8 断电后设计规则），不提 I-3.4。`:470`「补讲意图日志崩溃语义的四条…与 I-3.4（可用空间扣待删占用）」才是 FACT7 描述的「a list of invariants still owed narrative coverage」——语义描述对，行号错 |
| FACT7「A search…for the phrase 'committed reservation' returns no matches」 | 对 | `grep -n "已承诺预留" .claude/kb/invariants.md` 退出码 1，零命中 |

### 翻译核对表（27 行）抽核限定词

- 逐条核对上文 FACT1-7 已覆盖的行号与引文；额外重点抽查「首稿缺的」一列声称补回的限定词：
  - 「for the first version's two-device case」（针对「扣一半」的设备数限定）——现查 `checks-owed.md:330`（应为 329）原文「按副本之和记就每块盘扣一半」确实只在两块盘时成立，翻译核对表补上这个限定词的理由站得住。
  - 「with no parenthetical explanation given for either not carried mark on this row」（FACT3 第 6 行）——现查 `05-快照-空间记账机制.md:122` 确认原文两个「不带」都没有括注，第 4 项（`:120`）才有「（全池承诺量）」括注；核对表这条限定词补得对。
  - 「citing decision D5 settled item 8, not settled item 4」——现查 `transaction.rs:2314` 确认代码注释原文写的正是「已定项 8」，核对表这条限定词补得对，且指出了首稿一度混同的错误。
- 5 处 checks-owed.md 引用共享同一个错误行号（330，应为 329），是同一处漂移的重复引用，不是 5 个独立错误。

### 干净样本 s1 / s3 逐格核对

- 两份样本都：14 个标签齐全、次序与提示要求完全一致（`D5ROW.ONECOPY` … `SUMMARY.SUMCOPIES`）；不含被提示禁止的行号引用（`grep -nE "line [0-9]+|:[0-9]+"` 零命中）；不含 markdown 强调符号（`**`、`` ` ``、标题）零命中——三项格式要求都核实符合。
- s1 与 s3 在 `INVARIANT.BREAKS`/`D5ROW.BREAKS` 上给出不同判断（见下文「独立复推」一节），s1 判「一份副本会让 I-3.4 为假」，s3 判「都不会破坏」——两份样本内容不同，不是简单复读同一份答案，说明抽样不是摆设。

### 损坏判定（s1/s2/s3/void1）复核

自己重新数标点与跑 `corruption-check.py` / `oov-check.py`，与跑记逐项比对：

```
样本  逗号  句号  冒号  分号  wc -w  corruption exit  oov exit  words(自报)
s1    25    43    14    0     650    0                0         666 (生词=7: SUMCOPIES overestimation subtracts)
s2    0     16    0     0     753    0                0         774
s3    15    42    14    1     565    0                0         576 (生词=12)
void1  —     —     —    —     921    1(实词自复读=1 sentinel)  —   924
```

全部数字与 `c355-c363-r3-local-attack-runlog.md` 逐项相符（逗号/句号/冒号计数、`wc -w`、脚本自报 words、退出码、诊断词全部一致）。额外验证：s2 里要求逐字复现的短语「This would be refuted by:」，`grep -o "refuted by[^.]*"` 显示 s2 里这个短语**丢了冒号**（"refuted by the row description..."，无冒号），与跑记「连短语自带的冒号都被吞掉」的描述一致——**s2 判「带损坏」证据确凿，判定合理**（虽然不落在四类已登记签名里，跑记按规则从严处理，判据交代清楚）。

**W3 计数：核了 24 处（FACT 引用 17 + 抽查限定词 3 + 样本格式 3 + 损坏判定 1），对 21 处，不对 3 处（均为行号错 1 位数量级，内容本身无误）。**

## 独立复推：一份副本读法会不会让 D28 已定项 1 与 I-3.4 为假

不看 s1/s3 的答案，只看 FACT 与 kb 原文自己重新推一遍：

1. `05-快照-空间记账机制.md:187`（D5 已定项 7）：「已分配字节」= 分配记录树里落点之和。
2. 分配记录树的记录结构（`allocator.rs:25`，D3 已定项 7）：key = (设备, 槽号)，**每份物理副本各有一条独立记录**——`release()`（`allocator.rs:799` `for device in &mut self.devices`）对同一个逻辑落点在**每块盘上分别**改写一条记录，证实一个 2 副本单元在分配记录树里确实落两条记录，一盘一条。
3. ⇒ 「已分配」按盘算、再 Σ设备(...) 求和（`28-挂载期承诺量.md:19` 公式），这个求和结果**已经是两份副本的物理字节之和**——这是数据结构逼出来的，不是自由选择。
4. 待删占用与已承诺预留在公式里是 Σ设备(...) **括号外**、只扣一次的池级标量（同一行公式）。
5. 若把这两项按「一份副本」（逻辑大小，只算一份）记账：对于一个占用两份物理副本的单元，公式只扣了「本该扣两份」里的一份，另一份物理空间被算作「未扣除、仍可用」。
6. I-3.4（`invariants.md:131`）定义：「可用空间统计**已扣除**『待删除但意图未完成』占用的空间」——若只扣了一半物理占用，「已扣除…占用的空间」这句断言为假（只扣了一半，不是全部）。
7. **推得出**：一份副本读法确实会让 I-3.4 的断言与 D28 已定项 1 的公式（在「已分配」已经是逐盘物理字节和的前提下）不一致——与主 agent 的核法结论相同。反过来，副本之和读法与「已分配」的记账口径一致，不产生这个矛盾。
8. 与 s1/s3 对照：s1 的 `INVARIANT.BREAKS`「adopting the one-copy convention causes I-3.4 to be false…as the formula subtracts only the logical size」与本复推结论一致；s3 的 `D5ROW.BREAKS`/`INVARIANT.BREAKS`「no changes are required…neither change causes any other sentence to become false」**没有推出这一层**，是较弱/不完整的答案。

**结论：推得出，与主 agent 的核法一致。这条不算三方论证意义上的「推论」（没有用来推翻或确立 decisions.md 的任何一句），是核对表内的一次观测性复算，因此不额外走三方。**

## W2 复跑：计数模型 `w2_count.rs`（在本核查员草稿目录独立重跑）

命令（腿的模型目录先拷进草稿目录，再在副本里跑，未用腿的原目录）：

```
$ cp -r research/prompts/c355-c363-r3-opus-model/* /tmp/claude-1000/c355-r3-verifier/opus-model/
$ cd /tmp/claude-1000/c355-r3-verifier/count-rerun
$ nice -n 19 rustc -O --edition 2021 -o w2_count /tmp/claude-1000/c355-r3-verifier/opus-model/w2_count.rs
$ ./w2_count > w2_count.out
$ diff w2_count.out .../results/w2_count.out    # 无输出，逐字节相同
$ sha256sum w2_count.out
eabb701b05dccfb23b344b25a6fda4a7d09406b121c8ab134ee43f20e55a8e5d   # 与报告原文一致
```

两条变异复跑：

```
$ ./w2_count --mutate-ignore-device-cover      ; exit=101（断言 panic）
$ ./w2_count --mutate-split-pays-no-fixed-point ; exit=101（断言 panic）
```

两条都退 101，与报告「本次两条都退 101」一致。`diff` 与原始 `results/` 里的对应文件比对：**唯一差异是 panic 消息里嵌的线程号与源文件路径**（我的草稿路径 vs 腿的原路径），断言的 `left`/`right` 值、行号（`:105:9`、`:109:17`）逐字节相同——按「输出里嵌着临时路径的按字段比，不按整份哈希判 ✗」，**判对**。

**判别力自证（这条变异断言本身）**：两条变异命令本身就是「注入已知故障、看校验路子是否真的抓得到」的例子——腿的 `run.sh` 里写死「变异退出码为 0 就报错退出」，本核查员复跑得到的也是非 0（101），说明这条变异检查没有被摆设化。

## 快照漂移影响了哪几条

- **`.claude/kb/checks-owed.md`**（C37 改、C45 移出开着表、C236 链接改纯文本）：受影响的是 W3 材料对 C375 的行号引用（"line 330"，实为 329）——内容本身未变，只是行号随表内行数变化而偏移，判「不对」但不影响 W3 任何一格答案的实质（本地模型没被要求引行号）。C355/C363/C370/C84 四行主 agent 已核过与附录逐字节相同，三条腿均未直接引用这四行本身的行号（只作为编号在正文里提及）。
- **`.claude/kb/invariants.md`**（插入 I-8.8、改写 I-8.3/I-9.14、条数句 72→73、历史节新增两条）：三条腿里只有 W3 直接引用 `invariants.md` 的行号（`:131`、`:462`）。`:131`（I-3.4 行）核对无误——I-3.4 排在改动点（第 84 行插入）**之前**，不受影响。`:462` 本身是错误引用（应为 470），但这处误差与插入改动无关：文本内容确认第 462/470 行都不在 I-8.8/I-8.3/I-9.14 涉及的区域附近，是原始编号本来就没找准，不是漂移造成的。
- **`crates/singlefs-core/src/mount.rs`**：详见开头「快照核对」一节。漂移范围比主 agent 描述的大得多，但 W1/W2/W3 三条腿引用的具体函数（`raise_rollback_floor`、`release_reclaim_holds`、`reclaim_released_up_to`）与改动点（`allocator_of_version_without_file`、`format_time_allocator`、`RollbackToVersionWithoutFile*`、`ShadowLedger` 等）不重叠，逐一现查确认三条腿的结论不受影响。

## 没做什么

- 不判三条腿任何一条打中成不成立、该不该采纳；只核引用、产物与复跑，判决交主 agent。
- W2 报告里标「推的」「没在装置上跑」的部分（1.1 可达历史、2.1 改另一个单元要改三个的论证、2.4 部分表格行）本核查员未独立重新推导，只核它标注的条款引用逐字对不对；这些格子本身是否正确的判断权在主 agent。
- W2 的「零轮形态」Z1（`v2-reclaim-old-floor-immediately.patch`）与「没打中的形状」表未逐条复跑（V2 分支的 75 段本次复跑会覆盖，见下节；396 段回退扫描未重跑，时间成本过高，只核了补丁本身能不能干净应用于当前树——能，见「快照核对」一节）。
- 未对 `crates/` 做崩溃点重放或额外编译单测；未跑 `cargo clippy` 或门禁全量；只跑了登记给本核查员的门禁阶段（`stage-owners.tsv` 里没有登记给 `three-way-verifier` 的阶段——现查 `awk` 命令空结果——所以没有该由本核查员先跑的门禁）。
- W1「没做什么」一节自陈的缺口（未建模型量释放链在多叶稳态下的真实大小、未实现分裂、未回答 W2/W3）照实转述，未替它补做。
- 未核对 W2 报告里「c355-c363-r1-opus-output.md 第 260 行」（归档文件，本轮开头引用第一轮攻方腿提 (b) 时括注「从哪里借、发布怎么拆」）——该文件在 `git show 3cff909^:...` 之前的更早某次提交归档，未追查具体是哪一次提交，只信了 W2 转述的内容本身与本轮内容一致，未逐字回查那份更早的归档。
- 未独立验证 W2 的「计数模型逐相位数出来」表格（`w2_count.out` 里 7 行的具体数字含义、`w2_probe.rs` 379 行代码逻辑）本身建模对不对——只核了它能不能重新编译跑出逐字节相同的产物（能），不核建模逻辑是否正确反映了 D16 已定项 9 与 raise_rollback_floor 的真实约束。

## W2 复跑：全量 copy-probe（`run.sh` 全部，在本核查员草稿目录独立重跑，非腿的原目录）

命令：`nice -n 19 bash /tmp/claude-1000/c355-r3-verifier/opus-model/run.sh /home/fy5090/code/singlefs /tmp/claude-1000/c355-r3-verifier/opus-rerun-draft /tmp/claude-1000/c355-r3-verifier/opus-rerun-output`（模型目录先整份拷进草稿目录，`$SRC` 指向入库仓只读 rsync 源，`$DRAFT`/`$OUT` 全部落在本核查员自己的草稿目录，退出码 0，两处 `patch` 均无 fuzz/reject 干净应用）。

`results/summary.txt`（报告里引的全部数字的来源）**与腿原始产物逐字节完全相同**（`diff` 零输出，退出码 0）：

```
## P1（base）：抬 F 被拒且原因是每块盘上都没有，按这一步写了几次分组
raise_writes=0 35
raise_writes=15 25
raise_writes=30 5
## P1（base）总段数 / 抬 F 做成
75
10
## P2（base）
# P2 summary: histories=3978 escaped=2127
in-process suffixes: 1430 escaped: 0
escaped with a mount in the suffix: 2127 escaped without: 0
## V1 对拍：各臂结局
base     990 Completed 
v1     726 Completed     264 NewFinding 
v2     990 Completed 
## V1（v1 臂）判红的段按抬 F 那一步的结局分组
RaisedFloor(F=20,publishes=2)(30) 66 0
RaisedFloor(F=36,publishes=2)(30) 66 0
Refused(MountError::Publish(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)))(0) 462 264
Refused(MountError::Publish(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)))(15) 330 0
Refused(MountError::Publish(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)))(30) 66 0
## V2：P1 里抬 F 的结局
     55 RaisedFloor(F=*,publishes=2)
     20 RaisedFloor(F=*,publishes=3)
```

逐条对主 agent 点名要核的三个数：

| 数 | 报告原文 | 复跑结果 | 判 |
|---|---|---|---|
| 「30 / 75 段历史在拒绝之前已经写了 1–2 次空发布」 | raise_writes=15（25 段）+ raise_writes=30（5 段）= 30，总段数 75 | 复跑：25+5=30，总段 75，逐字节相同 | 对 |
| 「拒绝之后 1430 个留在进程里的后缀 0 个走得出去」 | `in-process suffixes: 1430 escaped: 0` | 复跑：`in-process suffixes: 1430 escaped: 0`，逐字节相同 | 对 |
| 「借扣住的槽（V1）990 段里 264 段 I-3.1 红」 | `v1     726 Completed     264 NewFinding`（726+264=990） | 复跑：同一行逐字节相同 | 对 |

五个 `results/*.out` 产物文件逐个 `diff`：**每个文件唯一的差异是编译耗时行（`Finished ... in X.XXs`）与测试耗时行（`finished in X.XXs`）——这是运行环境的字段（本机当时有其他并发 `cargo test --all` 在跑，挂钟比腿原始的慢约 1.1–5.4 倍），所有数据行（含 `results/w2_v1.v1.out` 里报告正文引用的那条判红整行 `V1[v1] 1 0 20 1 MountWritable Refused(...)... NewFinding(CheckerViolations { invariants: ["I-3.1"] }) at Operation(42)...`）逐字节相同**。按「输出里嵌着…按字段比，不按整份哈希判 ✗」，判**对**。

**W2 复跑计数：核了 4 个产物文件的完整数据内容（summary.txt 全文 + 4 个 .out 文件的数据行）+ 3 个主 agent 点名的具体数字，全部对，0 处不对。**

## 三条腿合计与最终结论

| 腿 | 核了 | 对 | 不对 | 核不动 |
|---|---|---|---|---|
| W1（Sonnet） | 14 | 11 | 3（摘句漏括注 1、行内引用行号张冠李戴 1、复跑命令转抄失真 1；均不影响实质结论） | 0 |
| W2（Opus，kb/代码/静态哈希部分） | 23 | 22 | 0（1 处措辞连词微差，不计入不对） | 0 |
| W2（Opus，复跑部分：count 模型 + 全量 copy-probe） | 两轮复跑共核对 summary.txt 全文 + 5 个 .out 文件的数据部分 + 3 个 count 模型产物 | 全部对 | 0 | 0 |
| W3（本地攻方，含翻译核对表与样本） | 24 | 21 | 3（均为提示材料 FACT 行号错 1 位数量级：FACT1 差 1 行、FACT5 差 1 行、FACT7 差 8 行；C375 那处 5 行转述共享同一错误行号，按 1 处计） | 0 |
| **合计** | **≈88 处（不含逐字节 diff 的产物核对，那部分按文件计另算）** | **≈81** | **6** | **0** |

**复跑的数对不对得上**：全部对得上（W1 的 grep 命令转抄除外——那条命令本身转抄失真，但换回原始命令复跑，结论仍然成立；W2 的 count 模型与全量 copy-probe 两轮复跑，数据部分逐字节 100% 相同）。

**快照漂移影响了哪几条**：见前文「快照漂移影响了哪几条」一节——`checks-owed.md` 的 C375 行号引用（W3，5 处转述共享同一错误）、`mount.rs` 的漂移范围核实比主 agent 描述的大（W1/W2/W3 均未受影响，逐一现查确认）；`invariants.md` 的插入改动未影响任何一条被引用的行。

核对表里的 ✗ 不免除主 agent 对推论的逐条现查——本报告只核引用、产物与复跑，不判三条腿任何一条的推论本身对不对、该不该采纳。
