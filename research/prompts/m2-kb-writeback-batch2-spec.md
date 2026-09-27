# kb 写回规格（里程碑二收尾：第二批）

每条：文件、旧串（整段原文，在文件里恰好 1 次）、新串、依据。同一文件的各条旧串互不相交，按次序施加。

## 条 1

文件：`.claude/kb/decisions/13-验证路线.md`

旧串：
~~~~~
每个上只读恢复、问模型、池级 checker、记录核对器（只核挂载自己写出的那几次发布）。「比对的对象是实现恢复后的镜像」那一半，今天只有可写挂载之后的池级 checker 落在实现恢复后的镜像上；记录核对器在崩溃注入里与层 0 一样，入参的崩溃后镜像取的是崩溃态那一份。
~~~~~

新串：
~~~~~
每个上只读恢复、问模型、池级 checker、记录核对器：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成整条历史录制流 + 整条挂载流 + 挂载之后那次发布，持久集合逐段对应（历史那一段取第一次崩溃的持久集合，挂载那一段按二次崩溃取子集，之后那次发布全没落）。「比对的对象是实现恢复后的镜像」那一半：记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。
~~~~~

依据：用户 2026-09-27 定「崩溃注入层补可写挂载」之后代码审阅第 2 条收尾，实审 B3c-2（`research/prompts/m2-rev-b3c2-implementer-report.md` 第八节第 1 条给出这两处与新交法对不上、第三节「改法与依据」给出新交法的措辞、第一节结论第 1、2 条）；两个「先红后改」的对照在第四节。

## 条 2

文件：`.claude/kb/decisions-history/2026-09.md`

旧串：
~~~~~
### 2026-09-27（其二）：D13（验证路线） 已定项 4：按设备切段、FUA 只放行它那块盘、原地覆写多一态「新旧都读不出」
~~~~~

新串：
~~~~~
### 2026-09-27（其四）：D28（挂载期承诺量） 已定项 4：分配记录树那一项的式子从 K0 换成 K1（整棵树）

> 快查·改前：分配记录树、中央映射树进 Σ，按树高；实现仍是盘数 × 2 × (高 − 1) + 1（K0），A4c 报告量出它不是上界、会少扣。
>
> 快查·改后：分配记录树这一项按 K1（整棵树在单元区每层上罩到的位置数之和）算，恒是上界；中央映射树按现有的删插式子算，删插数跟着分配记录树这一项变；实现随 A4d。

- 改前：形态那一条写「分配记录树、中央映射树进 Σ，按树高」，没给出分配记录树具体怎么从树高算出节点数（代码里是 K0：盘数 × 2 × (高 − 1) + 1）。
- 改后：分配记录树这一项改按 K1 算：1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1，S_L = 812 × 169^L），不读这一版的状态、不依赖 ckpt_cost；中央映射树这一项照它现有的删插式子算，删插数取「分配记录树这一项 + 记账树节点数」，随分配记录树的式子换而跟着变。代价：4 GiB 以上每块盘多扣约 2.7–3.2%（1 TiB 约 33 GiB）；`admission.rs` 还没改，实现随 A4d。
- 依据：A4c 报告候选式子表（`research/prompts/m2-rev-a4c-implementer-report.md` 第二节「候选式子」）实测三种情形一块盘改 3–4 片叶、K0 不是上界、不动点不收敛；用户 2026-09-27 JST 09:5x 弹窗选「K1 整棵树」。

### 2026-09-27（其三）：D13（验证路线） 已定项 7：射程改写记录核对器收两份镜像、第三截交整条历史流 + 挂载流 + 挂载后那次发布

> 快查·改前：射程写「每个上只读恢复、问模型、池级 checker、记录核对器（只核挂载自己写出的那几次发布）」「记录核对器在崩溃注入里与层 0 一样，入参的崩溃后镜像取的是崩溃态那一份」。
>
> 快查·改后：第三截交给记录核对器的写表是整条历史录制流 + 整条挂载流 + 挂载之后那次发布；记录核对器改收两份镜像，崩溃态镜像判根在而记录在不在（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表。

- 改前：射程末段写「每个上只读恢复、问模型、池级 checker、记录核对器（只核挂载自己写出的那几次发布）」「『比对的对象是实现恢复后的镜像』那一半，今天只有可写挂载之后的池级 checker 落在实现恢复后的镜像上；记录核对器在崩溃注入里与层 0 一样，入参的崩溃后镜像取的是崩溃态那一份」。
- 改后：第三截（二次崩溃）交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成「整条历史录制流 + 整条挂载流 + 挂载之后那次发布」，持久集合逐段对应；记录核对器改收两份镜像，崩溃态镜像判「根在而记录不在」（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表；第二截接的恢复后镜像是挂载与那次发布之后的池，恢复自称的那一版取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。
- 依据：代码审阅第 2 条收尾，实审 B3c-2（`research/prompts/m2-rev-b3c2-implementer-report.md` 第三节「改法与依据」、第八节第 1 条给的整行原样）；两个「先红后改」的状态在改前代码上先摆出来（第四节）。

### 2026-09-27（其二）：D13（验证路线） 已定项 4：按设备切段、FUA 只放行它那块盘、原地覆写多一态「新旧都读不出」
~~~~~

依据：新条目放在「## 历史版本」下最上面（decisions-history.md 文件头的规矩）；今天（2026-09-27）已有其一、其二两条（batch1 已写回），这两条分别记 D13 已定项 7（条 1）与 D28 已定项 4（条 3）的改动，续编其三、其四。

## 条 3

文件：`.claude/kb/decisions/28-挂载期承诺量.md`

旧串：
~~~~~
  - 分配记录树、中央映射树进 Σ，按树高；
~~~~~

新串：
~~~~~
  - 分配记录树这一项按 K1 算：1 + Σ_盘 Σ_{层级 L = 0 … 根层级 − 1}（这块盘单元区 [s, e) 在层级 L 上罩到的位置数 = ⌊(e − 1) ÷ S_L⌋ − ⌊s ÷ S_L⌋ + 1，S_L = 812 × 169^L），不读这一版的状态、不依赖 ckpt_cost，恒是上界；中央映射树这一项进 Σ，按它现有的删插式子算，删插数取「分配记录树这一项 + 记账树节点数」，随分配记录树这一项的式子变而跟着变；
~~~~~

依据：用户 2026-09-27 JST 09:5x 弹窗选「K1 整棵树」；式子取自 `research/prompts/m2-rev-a4c-implementer-report.md` 第二节「候选式子」K1 那一条（第 92 行）；中央映射树那一项不改是同一节标题括注给的口径（第 89 行）。

## 条 4

文件：`.claude/kb/decisions/28-挂载期承诺量.md`

旧串：
~~~~~
- `research/prompts/c363b-r1-main-verification.md` 第三节 V2-2、第四节第 4 条。
~~~~~

新串：
~~~~~
- `research/prompts/c363b-r1-main-verification.md` 第三节 V2-2、第四节第 4 条。
- 用户定案：分配记录树这一项换成 K1（整棵树），2026-09-27 JST 09:5x 弹窗选「K1 整棵树」；代价与候选式子表在 `research/prompts/m2-rev-a4c-implementer-report.md` 第二节「候选式子」（K0 不是上界、实测三种情形一块盘改 3–4 片叶、不动点不收敛）。
~~~~~

依据：新增依据指针，不增删实验号（A4c 报告不是 E 编号实验页）。

## 条 5

文件：`.claude/kb/decisions/28-挂载期承诺量.md`

旧串：
~~~~~
**欠**：C83（提交固定点没人回答）：实现后记固定点轮数，超过当时现算的量判红；C355（checkpoint 保留池池级扣而固定点每块要落两列）：逐设备扣那一格的准入对拍。
~~~~~

新串：
~~~~~
**欠**：C83（提交固定点没人回答）：实现后记固定点轮数，超过当时现算的量判红；C355（checkpoint 保留池池级扣而固定点每块要落两列）：逐设备扣那一格的准入对拍；分配记录树按 K1 的式子还没写进 `admission.rs`（今天仍是 K0：盘数 × 2 × (高 − 1) + 1），实现随 A4d。
~~~~~

依据：A4c 报告「K1、K3 都没写进 `admission.rs`」（第二节末段，第 116 行）；实现分给 A4d。

## 条 6

文件：`.claude/kb/checks-owed.md`

旧串：
~~~~~
用户 2026-09-26 定：暂时读不出的最新根不许当成被抛弃、「认下」这一支不走（原话「崩溃恢复把一条暂时读不出的最新根当成被抛弃 这个不能接受 这个要改」）；改法岔路单 `research/prompts/c554-fix-forks.md`（甲 拒可写、乙 重读后再判、丙 从记录重建，对照今天；三个候选都要同时罩住代码审阅第 22 条那一格），各臂的代价由 E158（择根与修复四岔路） 第 3 次跑量（跑前登记 `research/prompts/e158-r3-prereg.md`；实七复现种子在 `research/prompts/e158-r2-prereg.md` 主 agent 跑前修订那一段）：第一段的丢写读数按登记作废（阳性对照 PC-N 在今天那一臂上没过，那一句按实现今天的机理改写之后在九臂上重跑），第二段还没跑；量完交用户定。按 (区域, 槽) 让根槽读返回失败的开关已有（`singlefs_harness::fault_injection` 的 `RootRingSlotTarget` 与 `FaultSchedule::every_read_of_named_root_ring_slots_fails`，钉着今天行为的那条用例就用它）。主 agent 2026-09-25 判立欠账、不挡回退改形态那一轮，照用户同日对实例号撞号那一类「记成欠账」的定案（第一轮判决第 54 行）
~~~~~

新串：
~~~~~
用户已定（2026-09-27 JST 09:07 弹窗答复）：修法选乙 重读后再判，重读次数 R = 1（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行最后一格：取自 D16（发布语义） 已定项 1「根槽这一次读坏」那一行的「重读一次」；判「有一条更新的根读不出」用系统配置记的位置）；同一判据与重读要罩住代码审阅第 22 条那一格（重建影子账时第二次读实例表失败）。频率：门禁 74 号随机历史快档 96 段里 28 段是这一形，最短复现 3 步（`research/prompts/m2-investigate-gate74-reds-report.md`）。按 (区域, 槽) 让根槽读返回失败的开关已有（`singlefs_harness::fault_injection` 的 `RootRingSlotTarget` 与 `FaultSchedule::every_read_of_named_root_ring_slots_fails`，钉着今天行为的那条用例就用它）。实现在 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙手上。
~~~~~

依据：用户定案原话与手里的数在岔路单第 1 行最后一格（`research/prompts/c554-fix-forks.md`）；频率与最短复现在 `research/prompts/m2-investigate-gate74-reds-report.md`（历史 96 段、新发现 28、最短复现 3 步）。

## 条 7

文件：`.claude/kb/checks-owed.md`

旧串：
~~~~~
多次挂载的崩溃点重放里造这段历史（被抛弃实例的根重开时全部暂时读不出 → 新实例复用 → 撤故障 → 新实例的根全坏），断言恢复落到的根引用的单元没被复用，或挂载报错；判别力自证：今天的代码必须红
~~~~~

新串：
~~~~~
多次挂载的崩溃点重放里造这段历史（被抛弃实例的根重开时全部暂时读不出 → 新实例复用 → 撤故障 → 新实例的根全坏），断言恢复落到的根引用的单元没被复用，或挂载报错；判别力自证：今天的代码必须红。**验收**（乙改完之后要绿的三条）：调查员最短复现（`SINGLEFS_RANDOM_HISTORY_SHRINK_SEED=7463871032432355115` 跑 `shrink_one_failing_seed_from_the_environment`，3 步，`research/prompts/m2-investigate-gate74-reds-report.md`）不再判红；`second_transaction_step_three_formatted_pool.rs` 里那条 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 用例改判「只红 I-7.4（近 K 代块未被复用）」；`second_transaction_supplement_two_unreadable_abandoned_root_slot.rs` 的 `an_abandoned_root_whose_own_root_slot_is_unreadable_is_neither_isolated_nor_counted` 改名、按新行为钉（暂时读错、重读成功时择对根、不红；R 轮之后仍读不出则拒可写、只读照常）
~~~~~

依据：岔路单第 1 行；实现规格 `research/prompts/c554-fix-forks.md` 挪出的验收三条照实现员那份「验收」一节第 1 条三个子项（乙那一路的实现规格，主 agent 另派 C554 乙落地）。

## 条 8

文件：`.claude/kb/checks-owed.md`

旧串：
~~~~~
- C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）：补 E158（择根与修复四岔路） 第 3 次跑第一段查到的丢写机理（新实例写行那次发布取了与被藏根相同的 txg、把它的根槽盖掉），代码审阅第 22 条并进来，前置改指改法岔路单 `research/prompts/c554-fix-forks.md` 与第 3 次跑，补 checker 那一侧 I-7.4（近 K 代块未被复用） 照实红。
~~~~~

新串：
~~~~~
- C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）：补 E158（择根与修复四岔路） 第 3 次跑第一段查到的丢写机理（新实例写行那次发布取了与被藏根相同的 txg、把它的根槽盖掉），代码审阅第 22 条并进来，前置改指改法岔路单 `research/prompts/c554-fix-forks.md` 与第 3 次跑，补 checker 那一侧 I-7.4（近 K 代块未被复用） 照实红。
- C554（崩溃恢复抛弃的根暂时读不出时影子账算不到）：用户 2026-09-27 JST 09:07 弹窗定修法乙 重读后再判，R = 1（岔路单 `research/prompts/c554-fix-forks.md` 第 1 行最后一格）；频率：门禁 74 号随机历史快档 96 段里 28 段是这一形，最短复现 3 步（`research/prompts/m2-investigate-gate74-reds-report.md`）；实现分给 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙那一路。
- 新立 C570（抬 F 回收扣住的槽计数上空闲但分配器不发）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）（A4c 报告「要改成准入先拒需要的几样」第 4、5 条，推的，没实现、没量 `df`）。
~~~~~

依据：checks-owed.md 自己的「## 历史版本」按当天日期堆叠，与 decisions-history 那份「快查」两行不是同一套规矩（这份文件的既有条目没有快查行），照它现有格式追加。

## 条 9

文件：`.claude/kb/checks-owed.md`

旧串：
~~~~~
| C419 | F 生效之后靠什么保持不回落 | 取 SysPre：抬 F 那一串先逐盘把新 F 写进系统配置、加屏障再发根（`crates/singlefs-core/src/mount.rs` 的 `raise_the_floor_through`），恢复取 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 系统配置里读得出的 F 的最大值)（`crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor`）；会红的检查是 I-7.12（系统配置 F 不低于同盘根上的 F） 的池级判定与它的坏镜像，以及 C556（checker 与层 0 不读系统配置里的 F） 那一侧 checker 读系统配置里的 F | 2026-09-26 |
~~~~~

新串：
~~~~~
| C419 | F 生效之后靠什么保持不回落 | 取 SysPre：抬 F 那一串先逐盘把新 F 写进系统配置、加屏障再发根（`crates/singlefs-core/src/mount.rs` 的 `raise_the_floor_through`），恢复取 F_生效 = max(各幸存盘最新持久有效根所带 F 的最大值, 系统配置里读得出的 F 的最大值)（`crates/singlefs-core/src/recovery.rs` 的 `effective_rollback_floor`）；会红的检查是 I-7.12（系统配置 F 不低于同盘根上的 F） 的池级判定与它的坏镜像，以及 C556（checker 与层 0 不读系统配置里的 F） 那一侧 checker 读系统配置里的 F | 2026-09-26 |
| C570 | 抬 F 回收扣住的槽计数上空闲但分配器不发 | 抬 F 回收槽从隔离集摘除之后到 F 生效之前，这些槽在准入读数上已经算回空闲，但分配器实际还不发它们；式子（D28（挂载期承诺量） 已定项 1）没有一项装这段「扣住」的量，那一刻的准入判断比分配器真会给的更松（`crates/singlefs-core/src/admission.rs` 的 `AdmissionReading::of_allocator` 文档 ⚠️ 那一条） | 造一次挂载：抬 F 回收一批槽之后、F 生效之前立刻发起一次会把这些槽用掉的分配，断言分配器真的不发它们；准入放行、分配却失败（或反过来）判红；判别力自证：把「扣住」那一步去掉（分配器改成照发），检查必须变 | A4c 报告「要改成准入先拒需要的几样」第 4 条推给主 agent（推的，没实现）；准入先拒（`transaction.rs` 的 `settle_the_allocation_record_tree` 前置判）本身还没做，这一条与它同批 | 2026-09-27 A4c 报告（`research/prompts/m2-rev-a4c-implementer-report.md` 第三节第 4 条） |
| C571 | df 算不算聚簇段内关着的空槽没量，假性 ENOSPC 待判 | 这次挂载里，D3（空间分配） 已定项 8 第 2 条为聚簇提交对用户数据关着的段内空槽（A4c 报告量到一段 79 对），`df` 报的可用空间要不要把这些槽算进去没有条款；若算，用户可能看着 `df` 报有空间却写入 ENOSPC，那会是 D3（空间分配） 已定项 9 第 1 条说的假性 ENOSPC 那一类（A4c 报告第三节第 5 条，推的，没量 `df` 的实现） | 量一次 `df`（或它对应的容量查询路径）在段内空槽被关着、真实可分配对已经不够时报的可用字节，与那次写实际拿到的 ENOSPC 对照：`df` 报的可用 > 0 而写入 ENOSPC，且那些空槽正是本条「段内关着」的空槽，判为假性 ENOSPC 命中；没有就把这一条改判「量过、不成立」。判别力自证：把「段内关着」那一步去掉（不预留段外成对空槽），检查必须变 | 先要有「段内关着的空槽计数」这个读数（C570（抬 F 回收扣住的槽计数上空闲但分配器不发） 或 A4c 报告第三节第 3 条：按段增量维护整对都空的槽对数），才谈得上量 `df` 会不会把它们算进去 | 2026-09-27 A4c 报告（`research/prompts/m2-rev-a4c-implementer-report.md` 第三节第 5 条） |
~~~~~

依据：C 编号取全仓现查最大值 + 1（`grep -rohE 'C[0-9]{3,4}' 排除 .claude/gate.d/fixtures/ 之后最大 C569`，本批新立 C570、C571）；两条都出自 A4c 报告「要改成准入先拒需要的几样」第 4、5 条，主 agent 定各立一行、前置写清（派发提示）。

## 条 10

文件：`.claude/kb/checks-owed.md`

旧串：
~~~~~
D23（journal 的角色与格式） 已定项 19 ③ 定「环 ≤ 设备容量 ÷ 4」，预想镜像从 1 GiB 改成 4 GiB ⇒ [layout/01-first-txn.md](layout/01-first-txn.md) 五那一节的空闲字节改成 3 472 670 720、全空段数改成 3310、runs 改成 4，三个数仍然全是镜像大小的函数 | 2026-09-13 E142（第一个事务的干跑） 第六次跑 G22 |
~~~~~

新串：
~~~~~
D23（journal 的角色与格式） 已定项 19 ③ 定「环 ≤ 设备容量 ÷ 4」，预想镜像从 1 GiB 改成 4 GiB ⇒ [layout/01-first-txn.md](layout/01-first-txn.md) 五那一节的空闲字节改成 3 472 670 720、全空段数改成 3310、runs 改成 4，三个数仍然全是镜像大小的函数；主 agent 2026-09-27 定：`TEST_IMAGE_DEFAULT_BYTES`（`research/` 里镜像默认大小 4 GiB 的字面常量）不登记 format-const（C11b 报告候选表最后一行「没有条款落点」），这一笔仍是它唯一的落点 | 2026-09-13 E142（第一个事务的干跑） 第六次跑 G22 |
~~~~~

依据：C11b 报告「kb 要加的 format-const 标记」表最后一行与「设计问题」第 2 条（`research/prompts/m2-rev-c11b-implementer-report.md`）；主 agent 判断（派发提示）。

## 条 11

文件：`.claude/kb/invariants.md`

旧串：
~~~~~
⚠️ 代码里暂用占位名 `I-MAPPING-KEY`（`crates/singlefs-checker/src/image.rs` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER`），主 agent 另派人把它换成 I-1.11（映射 key 与单元头相符）：清单、判定点、用例三处一起换，变异表点着旧名的锚点跟着改）
~~~~~

新串：
~~~~~
编号已落到代码：`crates/singlefs-checker/src/image.rs:38` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 现在写的值就是这个编号本身，I-1.11（映射 key 与单元头相符））
~~~~~

依据：`research/prompts/m2-rev-i111-implementer-report.md`（实审 B2 编号落定：I-MAPPING-KEY → I-1.11）；`crates/singlefs-checker/src/image.rs:38` 现查 `pub const MAPPING_KEY_MATCHES_THE_UNIT_HEADER: &str = "I-1.11";`。

## 条 12

文件：`.claude/kb/invariants.md`

旧串：
~~~~~
判定照第二份报告第八节，代码暂用占位名 `I-MAPPING-KEY`，由主 agent 另派人换成新号。
~~~~~

新串：
~~~~~
判定照第二份报告第八节，代码里的常量当时暂用占位名 `I-MAPPING-KEY`，2026-09-27 由实审 B2（`research/prompts/m2-rev-i111-implementer-report.md`）换成 I-1.11（映射 key 与单元头相符）。
~~~~~

依据：同条 11；这一句在「## 历史版本」2026-09-27 条目里，按事件句改写、不改事实。

## 条 13

文件：`.claude/kb/layout/01-first-txn.md`

旧串：
~~~~~
| 单元头 | nonce 代号、MAC 与算法类型的预留位（偏移 105，算进头 ⇒ 头 134） | 12 + 16 + 1 | 全 0 | D9（加密） 已定项 10；D18（块里携带什么信息） 已定项 14 / 已定项 16（算法类型那 1 字节 2026-09-14 用户定案加） | 已定（第一版不写值） |
~~~~~

新串：
~~~~~
| 单元头 | nonce 代号、MAC 与算法类型的预留位（偏移 105，算进头 ⇒ 头 134） | 12 + 16 + 1 <!-- format-const: NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29 --> | 全 0 | D9（加密） 已定项 10；D18（块里携带什么信息） 已定项 14 / 已定项 16（算法类型那 1 字节 2026-09-14 用户定案加） | 已定（第一版不写值） |
~~~~~

依据：C11b 报告「kb 要加的 format-const 标记」表第 1 行（`research/prompts/m2-rev-c11b-implementer-report.md` 第 75 行）。

## 条 14

文件：`.claude/kb/layout/01-first-txn.md`

旧串：
~~~~~
含 29 字节预留位的头是 **134**，载荷从 134 起（D18（块里携带什么信息） 已定项 16）。
~~~~~

新串：
~~~~~
含 29 字节预留位的头是 **134** <!-- format-const: DATA_UNIT_PAYLOAD_OFFSET = 134 -->，载荷从 134 起（D18（块里携带什么信息） 已定项 16）。
~~~~~

依据：C11b 报告表第 2 行（第 76 行）。

## 条 15

文件：`.claude/kb/layout/01-first-txn.md`

旧串：
~~~~~
含预留位的头 136，记录区从 136 起
~~~~~

新串：
~~~~~
含预留位的头 136 <!-- format-const: PACKED_UNIT_RECORDS_OFFSET = 136 -->，记录区从 136 起
~~~~~

依据：C11b 报告表第 3 行（第 77 行）。

## 条 16

文件：`.claude/kb/decisions/18-块里携带什么信息.md`

旧串：
~~~~~
52 key 区间 2k / 52 + 2k 诞生代号
~~~~~

新串：
~~~~~
52 key 区间 2k <!-- format-const: INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT = 2 --> / 52 + 2k 诞生代号
~~~~~

依据：C11b 报告表第 5 行（第 79 行）。

## 条 17

文件：`.claude/kb/decisions/18-块里携带什么信息.md`

旧串：
~~~~~
⇒ 明文头 86 + 2k、含预留位 115 + 2k
~~~~~

新串：
~~~~~
⇒ 明文头 86 + 2k <!-- format-const: INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE = 86 -->、含预留位 115 + 2k
~~~~~

依据：C11b 报告表第 4 行（第 78 行）。

## 条 18

文件：`.claude/kb/decisions/23-journal的角色与格式.md`

旧串：
~~~~~
= 188 字节（实例表单元指针照所选根）
~~~~~

新串：
~~~~~
= 188 字节 <!-- format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES = 188 -->（实例表单元指针照所选根）
~~~~~

依据：C11b 报告表第 6 行（第 80 行）。

## 条 19

文件：`.claude/kb/decisions/23-journal的角色与格式.md`

旧串：
~~~~~
flags 1（55）= 56；不另带载荷 CRC
~~~~~

新串：
~~~~~
flags 1（55）= 56 <!-- format-const: JOURNAL_NAMED_ENTRY_BYTES = 56 -->；不另带载荷 CRC
~~~~~

依据：C11b 报告表第 7 行（第 81 行）。

## 条 20

文件：`.claude/kb/decisions/23-journal的角色与格式.md`

旧串：
~~~~~
4 KiB 记录装 67 个点名项（已定项 4）；再大只抬高
~~~~~

新串：
~~~~~
4 KiB 记录装 67 个点名项 <!-- format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD = 67 -->（已定项 4）；再大只抬高
~~~~~

依据：C11b 报告表第 8 行（第 82 行）；⚠️ 登记这个名字之后门禁 27 号会红在 `research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs:74`（值写成算式），见条 24。

## 条 21

文件：`.claude/kb/layout/01-first-txn.md`

旧串：
~~~~~
| 几何 | journal 环长度（字节） | 8 | 805 306 368 | D23（journal 的角色与格式） 已定项 2 / 已定项 19（mkfs 参数、默认 768 MiB、约束环 ≤ 设备容量 ÷ 4，2026-09-14 用户定案） |
~~~~~

新串：
~~~~~
| 几何 | journal 环长度（字节） | 8 | 805 306 368 <!-- format-const: JOURNAL_RING_DEFAULT_BYTES = 805306368 --> | D23（journal 的角色与格式） 已定项 2 / 已定项 19（mkfs 参数、默认 768 MiB、约束环 ≤ 设备容量 ÷ 4，2026-09-14 用户定案） |
~~~~~

依据：C11b 报告表第 9 行（第 83 行）。

## 条 22

文件：`.claude/kb/layout/01-first-txn.md`

旧串：
~~~~~
| 几何 | 根环 chunk | 4 | 1 048 576 | D22（单元原子性怎么合成） 已定项 2 |
~~~~~

新串：
~~~~~
| 几何 | 根环 chunk | 4 | 1 048 576 <!-- format-const: ROOT_RING_CHUNK_BYTES = 1048576 --> | D22（单元原子性怎么合成） 已定项 2 |
~~~~~

依据：C11b 报告表第 10 行（第 84 行）。

## 条 23

文件：`.claude/kb/experiments/142-第一个事务的干跑.md`

旧串：
~~~~~
`crates/singlefs-format/src/lib.rs:50-54` 与这个装置的 `index_node_header_bytes` 逐项核过
~~~~~

新串：
~~~~~
`crates/singlefs-format/src/lib.rs:47`／`:56` 两个标量 `INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE`／`INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT`（2026-09-27 起 format 只留这两个标量，不再有可调的共享 `index_node_header_bytes`，三方各自算一份、互不调用，代码审阅第 11 条）与这个装置自己的 `index_node_header_bytes`（`e142_first_transaction_dry_run.rs:1040`）逐项核过；⚠️ 这个装置 `:106` 的 `JOURNAL_NEW_ROOT_SEGMENT_BYTES` 写成算式（`2 * NODE_POINTER_BYTES + 8 + 8`），登记 `format-const: JOURNAL_NEW_ROOT_SEGMENT_BYTES` 之后门禁 27 号会红在这一行，是已知的（C11b 报告设计问题第 3 条），要改成字面量重跑产物或先不登记这个名字
~~~~~

依据：`research/prompts/m2-rev-c11-implementer-report.md` 设计问题第 4 条（`index_node_header_bytes` 三方各算一份）；`crates/singlefs-format/src/lib.rs:47,50-56` 与 `crates/singlefs-checker/src/lib.rs:301` 现查；`research/prompts/m2-rev-c11b-implementer-report.md` 「书记员落位时要一起看的两件事」第 1 条给出 `e142_first_transaction_dry_run.rs:106` 会红。

## 条 24

文件：`.claude/kb/experiments/157-并行线一两条条款的计数模型.md`

旧串：
~~~~~
`index_node_header_bytes(24)=163`（`86+2×24+NONCE_MAC_ALGORITHM_RESERVED_BYTES`，那个预留位那边是 29）
~~~~~

新串：
~~~~~
`EXTENT_NODE_HEADER_BYTES=163`（自己按 `86+2×24+29` 算的一份，2026-09-27 起不调 format 的函数——format 只留 `INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE`／`INDEX_NODE_HEADER_KEY_RANGE_KEY_COUNT` 两个标量，`index_node_header_bytes` 这个名字在 format、checker、每个实验装置里各自私有一份，互不调用，代码审阅第 11 条）；⚠️ 这份装置 `:74` 的 `JOURNAL_NAMED_ENTRIES_PER_RECORD` 写成算式（`(JOURNAL_RECORD_BYTES - JOURNAL_HEADER_BYTES) / JOURNAL_NAMED_ENTRY_BYTES`），登记 `format-const: JOURNAL_NAMED_ENTRIES_PER_RECORD` 之后门禁 27 号会红在这一行，是已知的（C11b 报告设计问题第 3 条），要改成字面量重跑产物或先不登记这个名字
~~~~~

依据：同条 23；`e157_parallel_line_one_clauses.rs:38,67`（`EXTENT_NODE_HEADER_BYTES` 的算法与值）与 `:74`（`JOURNAL_NAMED_ENTRIES_PER_RECORD` 的算式）现查；`research/prompts/m2-rev-c11b-implementer-report.md`「书记员落位时要一起看的两件事」第 1 条给出这一行会红。

## 第 7 项：层 0 钉值与登记句 —— 本批不写

`.claude/kb/layout/01-first-txn.md` 第八节层 0 钉值与登记句，等 E142 重出之后再写回，本批只占位、不生成改动条。

