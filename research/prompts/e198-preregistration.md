# E198 跑前登记：取号跳过认出生代号的前提

写于 2026-10-29，装置写之前。

## 一、问题

英文名：instance_table_pointer_birth_skip_premise

主 agent 给的问题（派发提示「关的是哪几条」一节逐字）：

> 问题单两行：① 加第四条件（R 的实例表指针出生 txg 等于 R.txg）之后，「行 + R 的实例表指针出生代号」能不能把取号跳过与存在过的版本分全（含根环转一圈留洞、可写挂载末尾清扫、读故障；装置判据排除实例 1）；② C698 两组镜像（nrb、rs1）随 ① 改后是否 0 误红。取新实验号、写跑前登记：臂、对照、判据、失败条款、取样域（C699 一行点名的短环 ss 432 盘面与 skr 45 步，加清扫那一形）、装置落点（照 `.claude/rules/implementation-first.md` 先读 `crates/` 今天的判法与写法：`crates/singlefs-checker/src/walk.rs` I-3.5 那一段、`crates/singlefs-core/src/mount.rs` 写行与 `crates/singlefs-core/src/instance_table_sweep.rs` 清扫）。两腿报告里的线索（第二轮攻方 `research/prompts/m3-close-b19-design-r2-opus-output.md` Q6、辩方 `research/prompts/m3-close-b19-design-r2-sonnet-output.md` Q6、第一轮正推腿 `research/prompts/m3-close-b19-checker-r1-sonnet-output.md` K7 的 P1–P3）只当线索。

问题单（`research/prompts/m3-close-b19-c699-forks.md`，逐行抄）：

```
# m3-close-b19-c699 问题单（2026-10-29）

判决 `research/prompts/m3-close-b19-design-r2-main-verification.md` 第二节 Q6：C699（取号跳过改认出生代号）先做量前提的实验，再实现。被测的是池级 checker 今天的判法（`crates/singlefs-checker/src/walk.rs` 里 I-3.5 认取号跳过的那一段）与写者今天的写法（`crates/singlefs-core/src/mount.rs` 写行那一处 `InstanceTablePlan::Rewrite(`、`crates/singlefs-core/src/instance_table_sweep.rs` 清扫那一处）。

| # | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 1 | 前提「写行之后、同一个实例里每一版照抄同一张实例表」：甲 加第四条件（R 的实例表指针出生 txg 等于 R.txg）之后，「行 + R 的实例表指针出生代号」把取号跳过与存在过的版本分全 ／ 乙 还有第四条件挡不住的形 | 在合法历史（含根环转一圈留洞、可写挂载末尾清扫、读故障）上，按甲的判法认作取号跳过的版本里有存在过的，或存在过的被认成取号跳过；量的装置判据排除实例 1（第一次可写挂载不写行） | 短环 ss 432 盘面与 skr 45 步（C699 一行点名的两组）加清扫那一形，分类逐个对：错分 0 就判甲，错分非 0 就判乙并给出那一形 | 开着 |
| 2 | C698（读坏一个根槽的镜像上的误红）随 1：甲 按 1 的判法改后，攻方 nrb、rs1 两组镜像上 I-3.5 不把存在过的版本认成取号跳过、I-3.9 不红 ／ 乙 还有误红 | 两组镜像上改后的判法报的结局 | 两组都重造并逐张判完 | 开着 |
```

### 读法写死

- **版本**：(实例代号 i, checkpoint_txg t)。**存在过的版本**：历史里写者发成、并且落到持久状态的那一版——它的根槽写（录制流里的 `RootRecordFua` 那一步）落进了镜像，或之后某一次可写挂载的恢复把它从 journal 记录里施加了出来（那次挂载交回的 `MountOutput::chosen_root` 的 txg < t ≤ `MountOutput::effective_root` 的 txg）。记录已落、根槽没落、之后没有哪一次挂载施加它的那一版（skr 里实例 2 写行那一次）**不算存在过**；崩在记录提交之前的不算。
- **祖先链**：每个存在过的版本记下它接在哪一版上发出（父版本）：普通发布、暖机、抬 F、清扫、回退发布的父版本是发它之前那个会话的现行那一版；一次可写挂载写行那次发布的父版本是那次挂载恢复之后的有效根（`effective_root`）。R 的祖先链 = 从 R 沿父版本一路往前。
- **候选集与相邻对**：照 I-3.5 的出现规则（第二节 `05-空间记账.md:11`）与 `walk.rs` 今天的取法（第三节）：回退候选集里的根（自证过、不被最新根的实例表判抛弃、txg ≥ F_生效，最新根恒收）加由记录施加出来、根槽从没落盘、四条同时成立的那几版（`walk.rs:11410` 起的文档注释 ①–④），按 txg 升序，同 txg 的几条并成一组；相邻两组 (R⁻, R) 为一对，R 侧逐条各算一对。**洞对**：R.txg > R⁻.txg + 1 的那一对。只有洞对进分类。
- **真值**（只从历史账本取，不读镜像）：洞对 (R⁻, R) 的真值是「**直接接上**」⟺ R 的父版本是 R⁻ 那一组里的某一版；否则是「**中间有存在过的版本**」（R⁻ 在 R 的祖先链上，链上 R⁻ 与 R 之间至少一版）或「**R⁻ 不在 R 的祖先链上**」——后两样合称「不是直接接上」，分开计数、合并判。
- **两条判法**（各自只读镜像）：
  - **H1-A（今天）**：`walk.rs:858-900` 的 `hole_in_the_candidate_set_between` 认作「行接得上」的三条件——最新根的实例表里有一行 (R⁻ 的某个实例 i, T = R⁻.txg)；R 的实例 v > i；i 与 v 之间每个实例 j 都有一行、都是 (j, 0, 0)。
  - **甲**：H1-A 的三条件，再加第四条件「R 的根记录里实例表指针的出生 txg 等于 R.txg」。R 是由记录施加出来的版本时，指针取那条记录新根段里的实例表指针（`walk.rs:2573` 的取法）。指针全零或出生 txg 读不出时，这一对按「不认作取号跳过」处理，另计一格「指针读不出」。
- **认作取号跳过** = 该判法认这一对「行接得上」。**E1**（把存在过的认成取号跳过）= 认作取号跳过、而真值不是直接接上的洞对数；**E2**（把取号跳过认成存在过）= 真值直接接上、而判法不认作取号跳过的洞对数。问题单第 1 行「错分」= E1 与 E2，两样各占一行、各自判，任一非 0 即「错分非 0」。
- **前提**（第 1 行候选的名字）读成三句，只在**写过行的实例**（实例代号 ≥ 2）上判，实例 1 排除（第一次可写挂载不写行，照抄 mkfs 那张表）：P1 一个写过行的实例的第一版（写行那次发布）指针出生 txg 等于它自己的 txg；P2 同一实例相邻两个存在过的版本，后一版的指针出生 txg 等于前一版的，或后一版是清扫发布、指针出生 txg 等于它自己的 txg；P3 指针出生 txg 不大于自己的 txg。前提的数从历史账本取（写者交回的根记录），镜像上读到的那一份只作回比（第十一节停机条款）。
- **第 2 行「按 1 的判法改后」** 读成：只把 I-3.5 的缺口判法从 H1-A 换成甲；I-3.9 的判法不改，用 `crates/singlefs-checker` 今天的 `check_pool_image` 判（`walk.rs:7619` 的 `judge_release_generations` 不读洞的分类，第三节）。「误红」：I-3.5 一半 = 甲在这两组镜像上的 E1；I-3.9 一半 = 今天的池级 checker 报 I-3.9 违例的镜像张数。
- **取样域**（第五节逐组写构造）：ss（432 张）、skr（45 张）、清扫那一形（sw-crash，432 张，主取样）、读故障加清扫（sw-read，315 张，甲的阳性对照）、不崩的清扫对照（sw-none，432 张，附带）、nrb（6 张）、rs1（285 张），加第八节的几何敏感性点。组名与张数的出处见第四节、第七节。

### 待主 agent 认的项（不改判据，装置写之前认）

1. **第 2 行 I-3.9 一半的读法**：照上面写死的读法，甲不碰 I-3.9，第 2 行的 I-3.9 一半就是今天的池级 checker 在重造的 rs1、nrb 上报的数（第四节第 6 条：按读码推，它与甲无关）。主 agent 若要的是「I-3.5 与 I-3.9 一起改」，I-3.9 的改法问题单里没有定义，这一半要先定改法再登记，这份登记不量它。
2. **sw-read 进不进第 1 行的翻面**：读故障加清扫（R 正是清扫那一版、它前面同实例的版本读坏）是正推腿 K7 自己写的「已知口子」，按甲的字面必中 E1（第四节第 3 条）。登记里它单占一行、同时当甲的阳性对照；照问题单字面（「含…读故障」）它的 E1 也让第 1 行翻乙。要把已知口子从第 1 行剔掉，由主 agent 在装置写之前改问题单。
3. **清扫阈值**：产品路径阈值是占位 369（C690 默认值没定），要在挂载末尾清扫就得池里有 370 行以上。主取样点取 `InstanceTableRowCountThreshold::ANY_ROW`（0，同一个配置项的另一个取值，走 `mount_writable_with_instance_table_sweep_configuration`）；369 那一档放在第八节几何敏感性。主 agent 认 0 这一档算合法历史。
4. **张数对不上时**：取样域按参数格定（第五节）；今天的写者在某一格上返回 `Err`（合法结局）而少了镜像时，照实报少了哪几格、为什么，第 1 行、第 2 行按实到的张数判，够不够判由主 agent 定（第十节失败条款 F5）。

## 二、被测条款与它引的定义

`python3 scripts/kb/quote-kb.py /home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-design/quoted-clause.md '.claude/kb/checks-owed.md~^\| C698 \|' '.claude/kb/checks-owed.md~^\| C699 \|' 'crates/singlefs-checker/invariants/05-空间记账.md~^\| I-3\.1 \|' 'crates/singlefs-checker/invariants/05-空间记账.md~^\| I-3\.5 \|' 'crates/singlefs-checker/invariants/05-空间记账.md~^\| I-3\.9 \|' 'research/prompts/m3-close-b19-design-r2-main-verification.md~^\| Q6 C699 \|'` 抽出（「✓ 6 段整抄进 …，回读逐字节一致」），整份追加在下面。I-3.1 一行整抄是因为前提那一句（「隔一条也是同一张实例表」…「没量」）住在它里面；I-3.5 一行是被改的缺口条款；I-3.9 一行是第 2 行的另一半；判决 Q6 一行是派这条实验的那一格。

**出处 `.claude/kb/checks-owed.md:422-422`（整段抄，未转述）**

```markdown
| C698 | 读坏一个根槽的镜像上的误红 | 读坏一个根槽的镜像算池级 checker 的合法镜像（用户 2026-10-04 定：checker 不许因读不出的根槽误红）。现行判法在这类镜像上：I-3.5（引用区间的精确性）「取号跳过」那一判只认实例表行，最新实例写行那条根读坏时把它存在过的那一版认成取号跳过（攻方副本上 nrb 6 / 6、rs1 285 / 285；只有 I-3.1（已分配统计对得上）按分类判时才红，I-3.5（引用区间的精确性）在攻方 21072 行判定里红 0 行，一次抽样）；I-3.9（释放代落在停止引用它的那一格区间里）在回退加根槽读坏的 rs1 285 张镜像里红 150 张（回退复活的单元，回退那一版读不出）。这些都按误红记 | 下一轮修：这类镜像上取号跳过不把存在过的版本认成跳过（改法见 C699（取号跳过改认出生代号）），I-3.9（释放代落在停止引用它的那一格区间里）不红；判别力自证：攻方 nrb、rs1 两组镜像现行判法认错 / 红、改完 0 | 用户 2026-10-04 定算合法镜像，下一轮实现；验收要等 C699（取号跳过改认出生代号） 的实现；去向：里程碑三二·5（I-3.5（引用区间的精确性）、I-3.9（释放代落在停止引用它的那一格区间里） 读坏一个根槽的镜像误红，随 C699（取号跳过改认出生代号））（五·11，2026-10-12 判）。 | `research/prompts/m3-candidate-hole-r1-main-verification.md` 第二节「「取号跳过」认什么证据」表第 1 行（只认实例表行）、第五节第 2 行与第六节「读坏一个根槽的镜像算不算合法镜像」行；攻方报告 `m3-candidate-hole-r1-opus-output.md`「四、旁及」那一节（rs1 上 150 / 285）；`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 3 行「不算合法镜像上的红」那一判被这一定案替换（冻结证据不回改） |
```

**出处 `.claude/kb/checks-owed.md:423-423`（整段抄，未转述）**

```markdown
| C699 | 取号跳过改认出生代号 | 取号跳过只认实例表行时，洞右边那条根（R）所在实例自己更早的版本没有行证据盖到，读坏一个根槽的镜像上被认成取号跳过（见 C698（读坏一个根槽的镜像上的误红））。用户 2026-10-04 定改认「行 + R 的实例表指针出生代号」：R 所在实例自己更早的版本不算跳过；出生代号不住在 journal 里，短 journal 环盖掉记录那一形也修得到（推的，没量） | 改后：合法历史（短环 ss 432 个盘面、合法的取号跳过 skr 45 步）上分类全对，读坏一个根槽的镜像（nrb、rs1）上不把存在过的版本认成跳过；判别力自证：现行只认行的判法在 nrb、rs1 上必须认错，改完 0 | 先量前提「写行之后每一版照抄同一张实例表」：I-3.1（已分配统计对得上）那一行里「隔一条也是同一张实例表」按「一个实例写行之后每一版照抄同一张实例表」推的那一句写着「没量」（`crates/singlefs-checker/invariants/05-空间记账.md`）；量过再进下一轮实现；这个改法被攻过零轮；去向：里程碑三二·5（取号跳过改认出生代号：先量前提，随 C698（读坏一个根槽的镜像上的误红））（五·11，2026-10-12 判）。 | `research/prompts/m3-candidate-hole-r1-main-verification.md` 第二节「「取号跳过」认什么证据」表最后一行（行 + R 的实例表指针出生代号）与第六节「「取号跳过」认什么证据」行（先量前提，量过再进下一轮实现） |
```

**出处 `crates/singlefs-checker/invariants/05-空间记账.md:8-8`（整段抄，未转述）**

```markdown
| I-3.1 | 已分配统计对得上 | 已分配空间统计 == 实际遍历所有引用得到的和 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） ⚠️ **引用按映射位置记**（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3 与第七节用户定案「重构」，D19（块指针的结构与宽度预算） 已定项 5 / 8）：豁免三类（映射树自己的节点、树表单元、实例表单元）之外的码 1 / 2 / 3，走读按那一版的中央映射认位置，指针里的位置条目只当提示——引用集合记映射位置，提示过期（单元被搬走、源槽已释放）不进引用；提示与映射条目在同一序号上带的整单元校验和不同的那条指针不当过期的提示，照它自己的位置条目记（`walk::CentralMappingLocationsOfAVersion::locate`，实现员 2026-10-15 推的读法，交主 agent 认）；合法搬迁过的镜像在 `crates/singlefs-harness/tests/checker_relocated_images_judged_by_the_mapping_position.rs`。⚠️ 「源记录还在册、目的槽没有记录」那一形（映射换位、分配记录一条不动）按槽数加总对得上，这一条看不见，由 G3 的新判 N2 判红：I-3.15（未释放记录罩住的单元由映射指在这一槽），2026-10-15 实现。 ⚠️ **checker 读法（2026-09-14 用户收尾弹窗定甲）**：「实际遍历所有引用」按根环里全部有效根的引用取并集——新池新建文件之后第 0 版树表单元只被更早的根引用、仍占着空间（已释放、在 defer 队列里），只按最新根算会少 1 槽。**「有效根」= 回退候选集里的根**：按最新根指着的实例表有效（无那个实例的行，或有行 (i, Ti, Wi) 且 T ≤ Ti）且 txg ≥ F_生效（D16（发布语义） 已定项 1「生效」）。遍历的是最新根与不被抛弃、不低于 F 的有效根（RGall：候选集不再滤一圈窗口，环上有洞时洞那一槽里留着的上一圈的根只要不被抛弃、不低于 F 也走；代码 `crates/singlefs-checker/src/walk.rs` 的 `check_pool_image_given_the_container_capacity_knobs` 里 `candidate_indexes` 那一段）。F_生效 由 checker 自己算：取各块盘上最新的有效根所带 F 的最大值，再与池里自证过、fsid 与本池相同的系统配置槽带的 F 取大（`walk::check_pool_image` 里的 `effective_rollback_floor`，C556（checker 与崩溃点重放不读系统配置里的 F） 已还清）——被抛弃时间线的根引用的单元由影子账隔离、F 之下的根引用的已释放单元已可再分配，都不在当前账里（里程碑「覆盖写、释放、回退与复用」步 4 / 步 5，`crates/singlefs-checker/src/walk.rs`）。⚠️ **最新根带的 F 低于 F_生效（两者不等）时这一格不判、报「不适用」（实二的读法，主 agent 2026-09-26 暂认，最终代码三方专攻这一格）**：先把新 F 写进系统配置之后、带新 F 的根落盘之前，或带新 F 的根全读不出，最新根的「已分配」是按它自己那个较低的 F 记的，还算着 F_生效 之下那些根引用的槽，与按 F_生效 取的候选集并集不可比，照判会在合法状态上红；另一种读法（并集按最新根自己的 F 取、别的几条按 F_生效 取）条款没选。坏镜像 `checker_known_bad_images.rs` 的 `every_root_carrying_the_raised_floor_unreadable_keeps_the_candidate_invariants_green_because_the_checker_reads_the_system_configuration_floor` 钉住这一格报不适用，`crates/mutations.tsv` 里把这道例外拿掉的那一行（实二）让它红。F_生效 之下仍在 defer 的槽：F_生效 之下、没被抛弃的根引用着，候选集那一遍没走到，而最新根的账里罩住它的记录是已释放、释放代高于回收门槛的（还在 defer 里，分配器还不许回收；回收门槛 = max(F_生效, 环里最旧有效根)，D16（发布语义） 已定项 1「可再分配」那一行），并进遍历得到的和（收口表第 43 行取丁-defer，用户 2026-09-28 定，被攻过一轮，判决 `research/prompts/abandoned-floor-r1-main-verification.md` M1；`walk::deferred_slots_referenced_only_below_the_floor_per_device`，用例 `raising_the_floor_into_the_gap_an_abandoned_instance_left_keeps_the_allocated_statistic.rs`）；⚠️ 它会不会盖住「该回收而没回收」那一类错，没拿变异试过。⚠️ **双故障形是认下的例外（C684（双故障形下已分配统计该不该并被抛弃根的槽），用户 2026-10-20 定读法甲「不并，记成已认的代价」，`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 323 行）**：一个槽在最新根的账里是已释放、还在 defer，原来引用它的那一版的根在环里而这一刻读不出、候选集走不到它，剩下引用它的只有被抛弃的根——丁-defer 不并它（被抛弃的根不并），I-3.1（已分配统计对得上） 在这一形只许红在记账多的一边；与「钉最旧 txg 的代价」同一类，到那一槽被这个进程写掉为止。判定不因这一例外改，池级 checker 与 GPU 判器照旧。隔离的记录：已分配而没有任何根引用的记录按单元（起点槽、跨度）归组，**只认记号**——组里每块盘上那条记录都带隔离记号（跨度段次高位 0x4000）的整组豁免，不看副本；有一条不带记号的照判违例，不退回按副本判（D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」，用户 2026-10-22 定落点甲、checker 只认记号，条款原文是跑前登记 `research/prompts/e187-preregistration.md` 第五节「丙的 checker」一段、变体 `mark_only`；`walk::quarantined_slots_exempted_per_device`）。之前那一版按副本判（用户 2026-09-25 定「核出坏的才豁免」，没有位置项可比时比 journal 点名项、再退回自证加代号核对，判决 `research/prompts/m4-i315-r3-main-verification.md` 第三节 C1、C4）随这一定案退役。记号与引用不符由 I-3.17（隔离记号与引用相符） 判。 中央映射条目指的物理范围算进遍历得到的引用，与树里指向同一单元的引用同 (设备, 起点槽, 跨度) 时算一个（2026-09-27，`m2-rev-b1-implementer-report.md` 第六节第 2 条）。由记录施加出来的那一版照 D23（journal 的角色与格式） 已定项 15，走那条记录新根段里的实例表指针与分配记录树根（`walk::walk_version_applied_only_by_records`；用户 2026-10-02 定新根段回到三字段，实例表指针与分配记录树根随新根段换成那次发布的）。⚠️ **没量的前提**：「隔一条也是同一张实例表」是按「一个实例写行之后每一版照抄同一张实例表」推的，没量（`m2-rev-b2-implementer-report.md` 第七节第 2 条）；量它是 C699（取号跳过改认出生代号） 的第一步。 ⚠️ **钉最旧 txg 的代价（用户认的）**：读坏槽与读回全零的槽把回收门槛钉在它可能住过的最旧 txg（D16（发布语义） 已定项 1「根槽这一次读坏」），这样的合法盘面上 I-3.1（已分配统计对得上） 红在记账多的一边，到钉解开为止（这个进程写掉那一槽，或进程内记根时重读读出有效根、读出被抛弃的根，`crates/singlefs-core/src/allocator.rs` 的 `RootRingOccupant` 文档注释）；`crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs` 三条慢档用例在第 10h 批改成只许红在记账多的一边（用户 2026-10-04 定钉最旧 txg、2026-10-05 定全零槽钉门槛而不并进上限，`records/2026-09-30-里程碑三开工第一轮.md` 第 10b、10c 批）。 ⚠️ **钉落盘之后的并法（C773（读坏与全零根槽的回收门槛钉只在内存里） 定案，用户 2026-10-20 认；C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批已实现：`walk::released_slots_the_pin_kept_in_defer`（判法那一半 `walk::slots_the_pin_kept_in_defer_among`），被抛弃根引用着的槽由 `walk::slots_referenced_by_abandoned_roots` 各走一遍收；GPU 判器扫描内核 `c_deferred_below_the_floor` 同判，要排除的槽由走读内核在被抛弃根那一阶段记进草稿区末尾那一张；回归形（根环三方第三轮攻方 `zero.out` 的 `pinned` 组）在 `crates/singlefs-harness/tests/root_record_carries_the_reclaim_threshold_and_the_pinned_ring_slot.rs`）**：根记录带回收门槛 t 与钉槽 p 之后（I-7.18（根记录的回收门槛等于现算的门槛）、I-7.19（钉着的槽不住本实例写的根）），这一格只把「钉带出来的那一截」不看引用并进遍历得到的和，被抛弃根引用着的槽不并（C684（双故障形下已分配统计该不该并被抛弃根的槽） 保持「不并」，那一形照旧红在记账多的一边）；最新根下面没有记账树时照旧报不适用（`walk.rs` 今天已是）。「钉带出来的那一截」取释放代落在 (t, 不带钉现算的门槛] 的已释放槽，t 取最新根带的那一个，不带钉现算的门槛就是丁-defer 那个 max(F_生效, 环里最旧有效根)；只在 t 低于它时走被抛弃根（两腿的算例照这样取，推的）。被抛弃根引用着的不并，别的不看引用（F 之下没被抛弃的根引用着的也并，「不看引用并进」照这一行原句取，实现员的读法，交主 agent 认）；已经并进去的（候选版本走到的、隔离豁免的、F 之下仍在 defer 的）不重复数。出处：根环三方第二轮判决 `research/prompts/m3-close-rootring-r2-main-verification.md` 第二节 Y4 那一行，第三轮判决 `research/prompts/m3-close-rootring-r3-main-verification.md` 第二节 C773（读坏与全零根槽的回收门槛钉只在内存里） 那一行之后的格式件补漏段。 |
```

**出处 `crates/singlefs-checker/invariants/05-空间记账.md:11-11`（整段抄，未转述）**

```markdown
| I-3.5 | 引用区间的精确性 | **引用区间的精确性**。**内容**：对任一块 b 与任一根 R（快照根或活头根），**「b 从 R 可达」这个可达性谓词，必须等于记账所用的那个谓词**——两者是同一个定义，不是两份。**当前形态下该谓词的表达式**是 `birth(b) ≤ R.txg < death(b)`（[decisions.md](../../../.claude/kb/decisions.md) D5（快照 / 空间记账机制） 形态）；**换快照模型时表达式要重写，内容不变**。可判定形式（与 I-3.1（已分配统计对得上） 共用同一趟全盘遍历）：按 txg 升序枚举全部快照根与活头得 `Reach(b)`，则 ①② 合成一条**出现规则**：取回退候选集（I-7.4（近 K 代块未被复用） 用的那一份，加由记录施加出来、根槽从没落盘的那几版）按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次），相邻两条 (R⁻, R) 为一对，逐对判。R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，`Reach(b)` 在 txg 序上是否连续不再单判）满足 (a) `birth(b) > R⁻.txg`；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent 树、inode 树，`deleted_inodes` 树根非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号都不大于那个出生 txg（读不出头的单元不计入）。`birth(b)` 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1（块头自述逻辑地址） 与 I-2.1（校验和与内容匹配）。**缺口**：R.txg ≠ R⁻.txg + 1 时这一对之间有洞，分两种处置——最新根的实例表里有一行 (R⁻ 的实例, T = R⁻.txg)、R 的实例比 R⁻ 的大、两者之间每个实例的行都是 (j, 0, 0)，即实例表行证明 R 所在的实例直接接在 R⁻ 那一版上的，这一对照常判（gap-row）；不是的，用户可见树子树里的落点不判，别的落点（树表、记账树、分配记录树、中央映射树的单元与实例表单元）仍照 (a) 判 `birth(b) > R⁻.txg`。R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时整对报不适用；没有一对判到任何落点时整条报不适用并带理由（同 I-4.9（非最新候选根引用的单元不晚于它） 候选集只剩一条根的写法）；③ 若 b 不被活头引用，则 b **恰好**出现在一个 deadlist 中，且该 deadlist 的归属区间 = `(Reach(b) 中最晚者.txg, 其下一个根.txg]`；④ 若 b 被活头引用，则 b 不出现在任何 deadlist 中；③ 的逆向（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：deadlist 中每一条 b 都不被活头引用，且 `Reach(b)` 非空（某个活快照引用它）或 b 在未完成销毁意图的待放集里（意图 (头, lo = S.txg, hi, P = prev(S).txg) 下纪元落在 [lo, hi) 里、birth > P 的条目，D5（快照 / 空间记账机制） 已定项 12；hi 在下一侧是活快照时取它的 S.txg，下一侧是活头时取被判那一版根记录的 checkpoint txg，判决 `research/prompts/m4-c145-destruction-bound-r1-main-verification.md` 第二节 B2，与 I-3.6（deadlist 紧界） 的豁免句共用这一个谓词）。③ ④ 与逆向里的「快照」只算活快照：带销毁中记号（码 14 条目 flags 位 0，同一判决 S4）的快照从写意图那一刻起不再钉住块（D5（快照 / 空间记账机制） 已定项 20 的例外句），不进 `Reach`；「归属区间」按纪元认：快照 X 那一份 = 纪元落在 [prev(X).txg, X.txg)、活头那一份 = [prev(头).txg, ∞)，b 该在的那一份是引用它的最晚那个活快照之后的那一侧（纪元落在 [那个快照的 S.txg, 链上下一个活快照的 S.txg 或 ∞)）。b 按单元认、写成中央映射 key（活快照与活头两棵用户可见树下面每个单元的 key 从指针现算，deadlist 条目里就是 key）。⚠️ **射程**：缺口的来源是新实例第一条根的 txg 取根环与 journal 里见过的最大 txg 加一（`crates/singlefs-core/src/mount.rs` 取号那一步），所以写行那次发布的记录已落而根槽没落、多记录发布只落一部分之后再可写挂载，必出现缺口；另有恢复丢掉尾巴记录、根槽写失败重发跳一格（今天的代码走不到，随失败处置在后面的里程碑做）、根槽读不出、根槽被环盖掉而记录也被盖掉（短 journal 环）。**盲区**：缺口右边那一对里，落在用户可见树子树里的新写单元（新叶）的诞生代号被改小到 R⁻.txg 以下，判不出；整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像同样判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18（块里携带什么信息） 已定项 24），`crates/` 今天没有。**谱系记录落地之后**，(b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形与缺口条款并存（`research/prompts/m3-birth-floor-r2-main-verification.md` 第二节第 9 行）。**判别力**：诞生代号改小到恰好等于 R⁻.txg 的新写单元必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像不许红。**推翻条件**：任一合法镜像（池级 checker 与崩溃点重放其余判定都不红）上红。 | 部分实现：出现规则（TF、β、gap、gap-row、n3）已实现（相邻两版有一版走读记过失败、引用没收全的那一对，与后一版新引用而头读不出、诞生代号不知道的单元，报部分没判并逐条列出，不静默跳过（C701（别的判法静默跳过读不出的部分），用户 2026-10-20 定「一律报部分没判」；GPU 判器那几处记射程里判不了、不置判到位）；折叠表怎么折（C748（最终折叠表漏判引用区间的落点），2026-10-29 实现）：走读跟着一条指针记下的每个落点，第一次记下时记它是哪个单元——豁免三类（映射树自己的节点、树表单元、实例表单元）是那条指针第 0 条位置条目的落点，豁免三类之外的按中央映射 key 折成第一次记下这把 key 时那条指针认的第 0 条位置；读回的是第 1 条那一份、那个落点此刻还折在第 0 条落点上时改记成它自己；全部版本走完之后的那一张就是最终折叠表。「R 引用而 R⁻ 不引用」按落点认：豁免三类的落点按它自己认、不折，豁免三类之外的落点按最终折叠表折成的单元认（搬迁前后两个落点是同一个单元，不算后一版新出现的，判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3）；折叠表只用来取 birth(b)（取 b 折成的那个单元头里的诞生代号）与判 (b) 里子树的诞生代号。改之前比的是按最终折叠表折成的单元：读回第 1 条那一份把 b 改记成它自己之后，后一版指 (b, c) 的 c 折进 b、前一版也引用 b，c 不判（坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `a_placement_the_final_fold_table_folds_into_a_unit_of_the_previous_version_still_reddens_the_appearance_rule`，只在 I-2.1 已红的镜像上出现）；`walk::Walk::note_references_of_a_pointer_followed_by_the_walk`、`walk::Walk::versions_folded_into_units_by_the_final_fold_table`，GPU 走读内核同一个取法（落点表一行第 7 字最高位记按中央映射 key 折的落点）），池级 checker `crates/singlefs-checker/src/walk.rs` 的 `judge_the_appearance_rule_of_adjacent_candidate_versions`（`IMPLEMENTED_INVARIANTS` 下标 65；坏镜像在 `crates/singlefs-harness/tests/checker_judges_reference_intervals_and_every_new_root_segment_field.rs`；变异在 `crates/mutations.tsv` 里「第8批 I-3.5（引用区间的精确性）」那几行）；③ ④（deadlist 那两条）连同逆向已实现（2026-10-15，里程碑四只读快照 checker 侧，判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ①）：并在下标 65 这一条下，池级 checker 的 `walk::judge_snapshots_and_deadlists_of_the_versions` 在回退候选集每一版（加由记录施加出来的那几版，树表单元相同的只判一次）上判：④ 每一条钉的单元活头不引用，逆向每一条钉的单元有活快照引用或在待放集里，③ 活快照引用而活头不引用的每个单元恰好一条、头是活头、纪元落在它该在的那一侧；一版里一个快照都没有、deadlist 一条都没有时这三格没有对象、不判；树表、活头两棵树、deadlist 树或活快照的两棵树读不全时报射程里判不了。纯判法的单测与必红世界在 `crates/singlefs-checker/src/walk.rs` 的测试模块（④、逆向、③ 缺一条与挂错一侧）与 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`（写路径建两个快照之后往 deadlist 里放一个活头还引用的块、放一个谁都不钉的块）；变异在 `crates/mutations.tsv` 里「只读快照 checker 侧」那几行。GPU 判器内核那一位判出现规则，也判 ③ ④ 与逆向：扫描内核的引用与快照那一截（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 的 `judge_snapshots_and_deadlists_of_the_versions`）照 `singlefs_checker::walk` 的同名函数判、置位，对拍逐位比，下标 65 不在对拍不比的表里（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 的 `POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS` 是空表）。出现规则取按树继承下界加 β 出现规则（用户 2026-10-02 定 ① 取 β）加缺口处置；缺口处置用户 2026-10-03 定 gap 加 n3（判决里的局部标号，只放过用户可见树里的单元）加 gap-row 两样都加。**被攻过几轮**：按树继承下界加 β 加整对不适用的缺口，在合法镜像上两轮没红（各一次抽样）；gap 加 n3 加 gap-row 的合成形、同 txg 几条并成一条、`deleted_inodes` 非全零才算继承、读不出头的单元不计入这三处措辞、谱系记录落地之后的区间形，都是零轮，终轮不再开轮，实现时再攻。实现挂在池级 checker 的主走读上（每版落点集合快照、用户可见树标签、压得住的判、新判定函数、位置键折叠、`IMPLEMENTED_INVARIANTS` 加一条、用例与变异），I-3.1（已分配统计对得上） 在短环上洞里的误红归下一批调查，与这一条共用同一个「候选集有洞」判法（`research/prompts/m3-birth-floor-r3-main-verification.md` 第二节第 4、5 行与第四节）。② 的上界半（`birth(b) ≤ R_first.txg`）另立 I-4.9（非最新候选根引用的单元不晚于它）（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 1 行）。⚠️ **射程注（2026-09-13）**：跨头共享的块（稀疏旁表计数 > 1，D5（快照 / 空间记账机制） 已定项 6）不由 deadlist 单独管，判据 ③ / ④ 对它们不适用；它们的生死由旁表计数与各头的可达性合起来判，那条不变量还没立。 |
```

**出处 `crates/singlefs-checker/invariants/05-空间记账.md:16-16`（整段抄，未转述）**

```markdown
| I-3.9 | 释放代落在停止引用它的那一格区间里 | 任一带已释放标志的分配记录，它的释放代必须落在区间 `(最后一条仍引用这个落点的有效根的 checkpoint_txg, 最早不再引用它的有效根的 checkpoint_txg]` 内；根环没有洞时这个区间只有一个值，等同于「等于最早不再引用它的那条根的 txg」。若每一条有效根都还引用它，它不许带已释放标志（D3（空间分配） 已定项 7（释放时条目不删、改写 value：跨度段最高位置 1、后 8 字节从分配代换成释放代）；C374（释放代与树表诞生 txg 只有验收断言盯着））。环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条崩溃点重放流的 12 个状态）按区间判，按「等于」判会在合法镜像上判红；已经回收、还没被复用的已释放记录跳过不判（见证它释放的根已不在候选集里，定位不了那个区间），全是这一类时整条报不适用并带理由。判别力：把发布 B 那次的释放代从 4 写成 3 必须红（最早不再引用那十个落点的有效根是 B、txg 4）。 | 已实现（2026-09-18，`crates/singlefs-checker` 的 `walk::judge_release_generation_and_tree_table_birth`；一条根引用的落点按它自己的中央映射认（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G3：搬走之后源槽在搬迁那一版起不再被引用，释放代落在搬迁那一格；`walk::RootReferences::note_located_by_the_central_mapping`）；从 deadlist 放掉的块的必红世界（释放代改成写意图那次的 txg）在 `crates/singlefs-harness/tests/checker_snapshot_space_accounting_must_red_worlds.rs`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 的 `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`；崩溃点重放每个崩溃状态都判）。**判法收窄在实现途中**：今天任一候选根走不全就整条报不适用；收窄成只在比最后引用它的那条根晚的候选根都走得全时才判这一条已释放记录，攻方造出的对照格镜像（抬 F 的空发布之后再覆盖写，把最后一次空发布那条根的树表翻坏）钉进用例（`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 10 行）。**从 deadlist 放掉的块**（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S5 ③，攻方 H5-3）：释放代落在（写意图 txg, 最后一批 txg]。落法（2026-10-15）：「一条根引用的落点」算进它活快照的两棵树（码 14 照 inode 树、码 15 照 extent 树往下数）与它中央映射树叶里每条映射条目指的单元（被钉单元的映射条目留在映射里，放掉它的那一批同一次发布删条目，同一判决 S3、S4），带销毁中记号的快照的两棵树不算（D5（快照 / 空间记账机制） 已定项 20 的例外句）；于是一个从 deadlist 放掉的块，最后还引用它的是放它那一批之前的那条根、最早不再引用它的是那一批的根，取到的区间落在（写意图 txg, 最后一批 txg] 里，比判决那一句更紧。照数销毁中快照的树，那几个块在最后一批之前的每条根上都还算引用，合法态红；不数映射条目，写意图那条根之后就不再算引用、释放代晚于区间，合法态红。变异在 `crates/mutations.tsv`「只读快照 checker 侧」那几行（单测钉「销毁中快照的两棵树不记引用」）；盘上的必红世界（从 deadlist 放掉的块释放代写成写意图那一次的 txg）要等 deadlist 写者与删快照落地（里程碑四丙 2）之后补。 |
```

**出处 `research/prompts/m3-close-b19-design-r2-main-verification.md:22-22`（整段抄，未转述）**

```markdown
| Q6 C699 | **采第四条件**（辩方：只缩小认作取号跳过的集合、不新增误红）；量前提的装置判据 P1 排除实例 1（第一次可写挂载不写行，攻方）；攻方指清扫那一形不要读故障也能把存在过的版本认成取号跳过——第四条件正是挡它，量前提的实验要把这一形放进去。C699 先做量前提的实验，再实现 | 两腿 Q6 |
```

## 三、实现今天的样子

读的是主工作区此刻的 `crates/`（2026-10-29），行号用 `grep -n` 现取。

**池级 checker 的 I-3.5 缺口判法（被改的那一处）**

- `crates/singlefs-checker/src/walk.rs:846-856` `enum HoleInTheCandidateSet`：`NoHole`（R.txg = R⁻.txg + 1）、`HoleBridgedByTheInstanceTableRows`（gap-row，照常判）、`HoleNotBridgedByTheInstanceTableRows`（用户可见树子树里的落点不判，N3）。
- `walk.rs:858-900` `hole_in_the_candidate_set_between`：只读三样——R⁻ 那一组的 txg 与实例集合、R 的 txg 与实例、**最新根**实例表的行；接得上 ⟺ 对 R⁻ 的某个实例 i：有行 (i, T = R⁻.txg)，R 的实例 v > i，i 与 v 之间每个实例都有一行且都是 (j, 0, 0)。R 的实例表指针、它的出生 txg 一样都不读。`crates/` 里没有第四条件：`grep -n 'instance_table.*birth\|pointer_birth' crates/singlefs-checker/src/walk.rs` 在 846-1046 一带只命中 974、982 两行（用户可见树根指针的出生 txg，判 (b) 用），858-900 这个函数里零命中。
- `walk.rs:909-1046` `judge_the_appearance_rule_of_adjacent_candidate_versions`：候选集按 txg 分组、`groups.windows(2)` 取相邻两组、R 侧逐条调 `hole_in_the_candidate_set_between`；`HoleNotBridgedByTheInstanceTableRows` 时用户可见树里的落点 `continue`（不判），别的落点照 (a) 判。洞的分类不交出函数，也不进判定说明（只在判红的那一句里带 `{hole:?}`）。
- 候选集：`walk.rs:19635-19639` 按最新根的实例表判抛弃（有行 (i, Ti) 且 t > Ti）；`walk.rs:19695-19714` `candidate_indexes`：最新根恒收，别的根不被抛弃且 txg ≥ F_生效（RGall：洞那一槽里上一圈留下的根照样收）。由记录施加出来的版本：`walk.rs:11410-11432` 的文档注释 ①–④ 与 `walk.rs:11433` `versions_applied_only_by_records`；这类版本的实例表指针取那条记录新根段里的（`walk.rs:2573`）。
- 公开入口里**没有**能交出候选集或洞分类的函数：`grep -n '^pub fn' crates/singlefs-checker/src/walk.rs` 列出的 8 个 `pub fn` 里，与这条实验有关的只有 `check_pool_image`（`walk.rs:19183`，交回各不变量的判定）、`root_ring_the_ring_judgements_read`（`walk.rs:19252` 起，交回每条自证过的根住哪个区域哪一槽、它的实例与 txg，C700 的跨装置检查用的就是它）与 `journal_records_as_the_recovery_reads_them`（`walk.rs:10607`）。
- I-3.9：`walk.rs:7619-7700` `judge_release_generations` 只读候选根的引用集合与 txg、分配记录的释放代，不读 `HoleInTheCandidateSet`（`grep -n 'HoleInTheCandidateSet\|hole_in_the_candidate_set_between' crates/singlefs-checker/src/walk.rs` 的命中全在 846-1000 一带）。

**写者：实例表指针什么时候换**

- 指针怎么取：`crates/singlefs-core/src/transaction.rs:14988-15008`：`InstanceTablePlan::Rewrite` ⇒ 新建整条链、指针取新链第 0 片（出生 txg 是这次发布的 txg）；`InstanceTablePlan::Carry(pointer)` ⇒ 照抄上一版的指针。
- `Rewrite` 的构造只有两处：`crates/singlefs-core/src/mount.rs:5267`（`row_publish_plan_on_file_version`，上一版带文件的写行发布）与 `crates/singlefs-core/src/instance_table_sweep.rs:721`（`SweepPublishPlan::as_publish_plan`，清扫发布）。另有一处不经 `InstanceTablePlan` 而重写实例表：`transaction.rs:1829` `publish_instance_table_on_version_without_file`（上一版树表 0 条而要写行时，`mount.rs` 预演那一支 `PreviousVersion::WithoutFile` 走 `prepare_the_row_publish_on_a_version_without_file`）。
- `Carry` 的构造：`mount.rs:5163`（抬 F 的空发布）、`mount.rs:5307`（暖机）、`mount.rs:9941`（回退发布 `roll_back_by_a_forward_publish`，`mount.rs:9697`）、`transaction.rs:9540`、`9593`、`9657`、`9729`、`9896`、`10034`、`10182`、`10694`、`11868`（用户改动的各条发布路径）。
- 第一次可写挂载（只做过 mkfs 的池）：`mount.rs:8069-8070` 的文档注释「取号 1、不写行、零单元的发布推到本实例的根覆盖每块盘」；零单元发布的计划 `ZeroUnitPublishPlan`（`transaction.rs:1397`）没有实例表字段，指针照抄 mkfs 那一份（出生 txg 是 `GENESIS_CHECKPOINT_TXG` = 0，`crates/singlefs-core/src/make_filesystem.rs:56`）。
- 写行：`mount.rs:5841` 起 `instance_rows_to_write` 给 [max(上一个实例, 1), 新号) 里每个实例一行，上一个实例 (i, T, W)、中间实例 (i, 0, 0)；暖机次数 `mount.rs:5809-5837` `warm_up_publish_txgs`：从写行那一版起加一，直到两块盘都落过一条（根落哪块盘按 `crates/singlefs-core/src/root_ring.rs:167-175` `target_for_publish`：区域 = txg mod 3，槽 = (txg ÷ 3) mod S）。
- 清扫：`instance_table_sweep.rs:1-20` 文件头（可写挂载末尾，在写行、暖机、抬 F、删除续做之后判一次）；`mount.rs:7088-7098` 调 `sweep_the_instance_table_at_the_end_of_the_mount`；`instance_table_sweep.rs:782-813`：开关关着、行数 > 阈值才判；`instance_table_sweep.rs:326-370` `judge_the_instance_table_sweep`：根环每槽都要自证过，候选行 = 实例在环里一条根都没有的行，没有候选行就不发（`NoCandidateRow`）；阈值 `instance_table_sweep.rs:71` `PLACEHOLDER_UNTIL_C690` = `INSTANCE_TABLE_PAGE_RECORDS - 1`（`crates/singlefs-format/src/lib.rs:142` 是 370，故 369）、`instance_table_sweep.rs:73` `ANY_ROW` = 0、`instance_table_sweep.rs:96` `PRODUCT`；带配置的挂载入口 `mount.rs:8595` `mount_writable_with_instance_table_sweep_configuration`。
- 根记录里实例表指针的位置：`crates/singlefs-core/src/root_record.rs:272`（`NodePointer::spec_decode_at(bytes, 170)`）；指针里出生 txg 的位置：`crates/singlefs-checker/src/image.rs:1806`（`birth_txg: read_u64(bytes, 42)`）。

**造镜像要用的入口（执行员按今天的签名调）**：`mount.rs:8088` `mount_writable`、`mount.rs:8595`（带清扫配置）、`mount.rs:9697` `roll_back_by_a_forward_publish`；`transaction.rs` 的 `publish_first_file`、`publish_overwrite`、`publish_sequential_write`、`publish_new_inodes`、`publish_version`；`crates/singlefs-harness/src/memory_pool.rs` 的 `writes_and_segments` 与 `crates/singlefs-harness/src/segments.rs` 的 `StepKind::RootRecordFua`（崩溃截断点）。

## 四、跑之前已经存在的数

照实列；每条写它对判据的影响。标「推的」的是读码或算术推出来的，没在今天的写者上量过。

1. **条款里的数（C698、C699 两行，第二节）**：nrb 6 / 6、rs1 285 / 285 张上今天只认行的判法把存在过的版本认成取号跳过；I-3.5 在攻方 21072 行判定里红 0 行；I-3.9 在 rs1 285 张里红 150 张；ss 432 个盘面、skr 45 步。出处是第一轮攻方腿在它自己的副本上量的（`m3-candidate-hole-r1`，一次抽样，副本里的 core 打过补丁），不是入库装置上的数。**影响**：432、45、6、285 进第七节「出自条款」一类锚点；6 / 6、285 / 285 是 H1-A 那条臂阳性对照的期望方向（第五节）；150 / 285 让第 2 行 I-3.9 一半在跑之前就偏乙（见第 6 条）。
2. **取样域的构造从线索模型的源码读来**：`git show 59ce5a865^:research/prompts/m3-candidate-hole-r1-opus-model/proto/src/lib.rs`、`git show 04ceaa904^:…/proto/src/bin/skip_row.rs`、`git show 31128273b^:…/proto/src/bin/{short_ring,newest_row_bad,rollback_hole,slot_bad}.rs`、`git show 04ceaa904^:…/scripts/{rerun-all,short-ring-sweep,rollback-sweep,run,build}.sh`（只读构造与参数格，`results/` 下的输出与汇总一份没读）。按参数格现算张数（第七节锚点 B1–B4 的命令）：ss = 9 ×（13 + 16 + 19）= 432、skr = 3 × 15 = 45、nrb = 2 + 4 = 6、rs1 = 5 ×（1 + 4 × 2 + 16 × 3）= 285，与条款里的数一致。**影响**：组的定义照这份构造写死（第五节），坏写者 m4a 那一半只在副本 core 里有，不进取样域。
3. **sw-read 上甲必中（推的，读码）**：R 是清扫那一版 s，指针由 `Rewrite` 新建、出生 txg = s（`transaction.rs:15000-15008`），第四条件按字面成立；行 (2, T = b) 在、实例 3 > 2、中间没有实例，三条件成立；而 b 与 s 之间实例 3 的写行与暖机存在过。故每张 sw-read 镜像甲与 H1-A 各认错一对：E1(甲) 推的 = 315（`anchors.py` 按 s = b + 2 + w₃ 算出的张数，每张一对；s 若因挂载里另有发布而后挪，张数跟着变，所以不进第七节锚点）。这正是正推腿 K7 写的「已知口子」。**影响**：它当甲的阳性对照（第五节）；按问题单字面它也让第 1 行翻乙（第一节待认项 2）。
4. **sw-crash 上甲在 k = 0 必中（推的，读码 + 算术）**：触发条件逐条代登记的几何算过（`python3 -I anchors.py`，脚本在草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-design/anchors.py`，第十三节）：
   - 清扫在实例 3 的挂载末尾发：阈值 0、行 {(1, a), (2, b)} 共 2 行 > 0；b = a + 3S + d ≥ a + 3S，写行之前环里最近 3S 个 txg 全是实例 2 的根，实例 1 一条根都不在环里 ⇒ (1, a) 是候选行；环里每槽都写过（txg 从 0 起连续到 b + 1 + w₃）⇒ 自证那一关过（读故障这一支不注）。
   - 暖机次数 w = 写行那一版 txg ≡ 2 (mod 3) 时 2，否则 1（`mount.rs:5809-5837` 加区域→盘 [0, 1, 0]）；s = b + 2 + w₃（假定挂载里写行、暖机与清扫之间没有别的发布；有的话 s 往后挪，实例 4 那一边同样多发，k = 0 时 s 那一槽仍不被盖，两边对称）。
   - 实例 3 发到 b + 3S − 1，b + 3S 记录落、根槽没落 ⇒ b 那一槽（同余）留着；实例 4 写行 b + 3S + 1 与暖机 w₄ = w₃ 次（同余同盘）只盖 b + 1 … b + 1 + w₄，s 那一槽在 k = 0 留着：27 段历史逐段算过，`s_survives_k0`、`b_survives_k0` 全为 True。
   - 结论：k = 0 那一张上 (b, s) 是洞对，四条件全成立，真值是中间有存在过的版本 ⇒ E1(甲) 推的 = 27（每段一张，k ≥ 1 时 s 那一槽被实例 4 的第一次覆盖写盖掉、R 换成 s 之后实例 3 的版本、指针出生 txg = s < R.txg，甲不再认错）；H1-A 推的 E1 = 369（每段 k = 0 … k_b，k_b = 3S − 1 − w₄ 是 b 那一槽被盖之前的最后一步）。这是第二轮攻方腿 Q6「打中二」推出的那一形（攻方报告 `m3-close-b19-design-r2-opus-output.md:129-141`，读过）。**影响**：第 1 行跑之前已按读码偏乙；这条实验把它从「推的」变成「量的」。若量出 E1(甲) = 0，先查清扫发没发（第六节 Q12 的触发计数），不直接判甲。
5. **前提 P1–P3（推的，读码）**：写过行的实例的第一版是写行发布，走 `Rewrite` 或 `publish_instance_table_on_version_without_file`，指针出生 txg = 自己的 txg；之后只有清扫发布换指针（第三节两处 `Rewrite`、九处 `Carry`）。故 P2 的非清扫违例推的 = 0、P1、P3 违例推的 = 0；实例 1 的第一版指针出生 txg = 0 ≠ 它的 txg（攻方 Q6「打中一」与 `mount.rs:8069-8070`）。**影响**：P1 在实例 1 上的「必不成立」当作 P1 检查会红的阳性对照（第五节），不进判定。
6. **第 2 行 I-3.9 一半与甲无关（推的，读码）**：`judge_release_generations`（`walk.rs:7619-7700`）不读洞的分类，甲只改 `hole_in_the_candidate_set_between`。故第一节写死的读法下，I-3.9 一半在甲与 H1-A 下逐张同值，量的是今天的 checker；条款里的 150 / 285 让它跑之前偏乙。**影响**：判据照登记（第六节 Q16、Q17）；第一节待认项 1 交主 agent。
7. **nrb、rs1 上甲不认错（推的，读码）**：两组的 R 是实例 2 写行之后的某一版（nrb：覆盖写；rs1：回退发布与之后的动作，回退发布走 `Carry`，`mount.rs:9941`），指针出生 txg = 写行那一版的 txg < R.txg ⇒ 第四条件不成立 ⇒ E1(甲) 推的 = 0 / 6、0 / 285。
8. **ss、skr（推的）**：skr 的洞对 (last, last + 2) 真值直接接上（实例 2 那次没被施加），R 是写行发布，四条件全成立 ⇒ 两判法都认对；ss 的洞对 (last, last + 2) 行是 (1, T = last + 1) ≠ R⁻.txg ⇒ 两判法都不认作取号跳过，真值中间有 last + 1（被施加过）⇒ 认对。推的 E1、E2 在两组上都是 0；last + 1 那条记录还读得出、实例 2 的前几版已被环盖掉的那一形（H1-A 会错）在 1 MiB 环上推不出会出现，4 MiB 那一点（第八节）可能出现。
9. **挂钟的线索**：线索模型 `rerun-all.sh` 文件头「ss5 约 3 分钟」（54 段 × 16 张 × 12 组开关、8 件并行），推一次 `check_pool_image` 约 0.14 秒（180 秒 × 8 ÷ 10 368，推的，是别的树上的数）。只用于第十四节的估算。
10. **辩方 Q6**：「已知口子只在 R.instance = R⁻.instance + 1 时开」（`m3-close-b19-design-r2-sonnet-output.md:152-166`，读过）。sw 三组按构造都是 R⁻ 实例 2、R 实例 3，与它一致；登记不另设中间隔实例的组（问题单没点名）。
11. **全仓搜词时读到的一行**：`records/2026-09-30-里程碑三开工第一轮.md:544`（搜 `skr` 命中）写着那份线索模型复跑十段、「腿留存的 461 份输出逐份 `cmp` 全部逐字节相同」。**影响**：无，它说的是那份模型的可复现，不是这条实验的任何量。

## 五、臂、阳性对照、真实基线

**装置写在哪（写死）**：入库装置 `crates/singlefs-checker-tier/src/bin/e198_instance_table_pointer_birth_skip_premise.rs`。理由：第 1 行的前提问的就是 `crates/singlefs-core` 这份写者的性质（哪几次发布换实例表指针），第 2 行问的是 `crates/singlefs-checker` 今天的 I-3.9 在重造的镜像上报什么；另写一份独立模型答不了。装置只调 `crates/` 的公开入口造历史与读盘，**两条判法（H1-A、甲）、候选集、洞对、真值账本在装置里另写**，不调、不抄 `walk.rs` 的私有函数（第三节：它们没有公开入口）。要读 `crates/` 的常量一律改成装置里的本地常量，加一条回比断言（与 `crates/` 那一处逐个比等，不等就停，第十一节 S4）：

| 本地常量 | 值 | 回比的 `crates/` 那一处 |
|---|---|---|
| 根记录里实例表指针的偏移 | 170 | `crates/singlefs-core/src/root_record.rs:272`；运行时另用写者交回的 `RootRecord::instance_table.head.birth_txg` 逐条比（S2） |
| 指针里出生 txg 的偏移 | 42 | `crates/singlefs-checker/src/image.rs:1806` |
| 根环区域数 | 3 | `singlefs_format::ROOT_RING_REGIONS`（`crates/singlefs-format/src/lib.rs:291`）；落点公式另与 `singlefs_core::root_ring::target_for_publish` 逐 txg 比 |
| 清扫阈值占位 | 369 | `InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690`（`instance_table_sweep.rs:71`） |
| 清扫阈值「有一行就扫」 | 0 | `InstanceTableRowCountThreshold::ANY_ROW`（`instance_table_sweep.rs:73`） |
| 一片实例表的记录数 | 370 | `singlefs_format::INSTANCE_TABLE_PAGE_RECORDS`（`lib.rs:142`） |
| mkfs 那一版的 txg | 0 | `GENESIS_CHECKPOINT_TXG`（`make_filesystem.rs:56`） |
| 读坏一个根槽时翻的字节 | 槽内第 40 字节取反 | 无 `crates/` 常量；翻完用 `root_ring_the_ring_judgements_read` 核那一槽不再出现（S1） |

**几何（各组共用，照线索模型 `lib.rs` 的 `parameters`）**：两块盘、区域到盘 [0, 1, 0]、物理块 512 字节、最小 I/O 512、固定结构槽距 4096、每盘 256 MiB、fsid 自取 16 字节、写时刻固定；S（每区域根槽数）与 journal 环长按组给。内容：默认 3000 字节覆盖写（`publish_overwrite`）；标「顺序写」的组走 `publish_sequential_write`、内容长 c × 524 288 字节。

**臂**（判法，第一节写死）：

- **H1-A（真实基线，今天的 `crates/`）**：`walk.rs:858-900` 的三条件，原样照抄成装置里的一份。
- **甲（被测）**：正推腿 K7 的推荐条文，按支持它的人认的样子写（`research/prompts/m3-close-b19-checker-r1-sonnet-output.md:285` 整句抄，判决 Q6 采它）：「在 gap-row 的三个条件后加第四个：「且 R 的根记录里实例表指针（偏移 170）的出生 txg 等于 R 的 txg」。第四个条件不成立（出生 txg 小于 R.txg）⇒ 按「不是 gap-row」处理（用户可见树子树里的落点不判，别的落点照判 birth > R⁻.txg）。指针全零或读不出出生 txg ⇒ 这一对按「不是 gap-row」处理并记部分没判。」装置只取它对洞对的分类，不判落点。
- 「怎么做」推得出「做完会怎样」：两条臂只决定一对洞对认不认作取号跳过，E1、E2 按第一节的真值数；不另写「做完会怎样」。

**取样域（每组的构造；「崩」= 那次发布的写流截在第一个 `RootRecordFua` 之前，记录已落、根槽没落）**

| 组 | 历史 | 参数格 | 镜像 |
|---|---|---|---|
| ss | mkfs → 实例 1（产品挂载）第一个文件 → 覆盖写到 last → last + 1 崩 → 实例 2 产品挂载（恢复施加 last + 1）→ 顺序写 k 次 | S ∈ {4, 5, 6}、环 1 MiB、last ∈ {4, 6, 8}、c ∈ {2, 3, 4}、k = 0 … 3S | 每个 k 一张：432 |
| skr | mkfs → 实例 1 到 last → 实例 2 产品挂载崩在写行那次的 `RootRecordFua` 之前 → 实例 3 产品挂载（不施加实例 2 那条、取号越过）→ 覆盖写 k 次 | S = 4、环 1 MiB、last ∈ {6, 8, 10}、k = 0 … 14 | 45 |
| sw-crash（清扫那一形，主取样） | 每次挂载都走 `mount_writable_with_instance_table_sweep_configuration`（开关关、阈值 0、落点取挂载的分配器）：实例 1 到 a → 实例 2 挂载、覆盖写到 b = a + 3S + d → 实例 3 挂载（末尾清扫删 (1, a)，清扫那一版记 s）→ 覆盖写到 b + 3S − 1 → b + 3S 崩 → 实例 4 挂载 → 覆盖写 k 次 | S ∈ {4, 5, 6}、环 1 MiB、a ∈ {4, 6, 8}、d ∈ {0, 1, 2}、k = 0 … 3S | 432 |
| sw-read（读故障加清扫，甲的阳性对照） | 同 sw-crash 到实例 3 挂载为止，不崩；实例 3 在 s 之后接着覆盖写到 b + 3S − 1；每一版之后取镜像，把实例 3 在 s 之前的每一版（写行、暖机与其间别的发布）的根槽读坏 | 同 sw-crash 的 27 段 | s … b + 3S − 1 每版一张：推的 315（第四节第 3 条） |
| sw-none（附带，够判后不跑） | 同 sw-crash，b + 3S 不崩、照常落 | 同上 | 432 |
| nrb | mkfs → 实例 1 到 last → 实例 2 产品挂载（写行 r）→ 覆盖写 k 次 → 读坏 r … r + bad − 1 的根槽 | S = 4、环 1 MiB；（last 10、k 3、bad ∈ {1, 2}、3000 字节）与（last 4、顺序写 2 MiB、k ∈ {3, 4, 5, 6}、bad 1） | 6 |
| rs1 | mkfs → 实例 1 到 10 → 实例 2 产品挂载（写行 r）→ `roll_back_by_a_forward_publish` 到 (实例 1, target)（那一版记 v）→ 动作串 → 读坏 r … R − 1 的根槽 | S = 4、环 1 MiB；target ∈ {5 … 9}；动作串取 {不做} ∪ {一个} ∪ {两个}，每个动作 ∈ {覆盖写, 接长顺序写, 建一个 inode, 空发布}；R ∈ {v, …, v + 动作数} | 5 ×（1 + 4 × 2 + 16 × 3）= 285 |

动作的写法照线索模型 `rollback_hole.rs`：接长顺序写 = 内容取 target 那一盐与这一版盐两段接起来、`publish_sequential_write`；建一个 inode = `publish_new_inodes(…, 1, …)`；空发布 = `publish_version` 带 `InstanceTablePlan::Carry`、不带文件。今天的签名与它不同的照今天的签名调，语义不变。写者在某一格返回 `Err` 的，照实记「合法结局、无镜像」，不补别的格（第一节待认项 4）。

**阳性对照（每条臂、每个检查各一处）**

| 对照 | 对象 | 期望 | 不中怎么办 |
|---|---|---|---|
| PC-H1A | H1-A 在 nrb 上 | 每张 E1 ≥ 1（第四节第 1 条条款数 6 / 6；第七节 C1 钉 1 张的绝对值） | 作废（Z1） |
| PC-甲 | 甲在 sw-read 上 | 清扫发了的那几段里 E1 ≥ 1 | 清扫发了而 0：作废（Z2）；清扫没发：失败条款 F6 |
| PC-P1 | 前提 P1 检查放到实例 1 上 | 实例 1 的第一版报一处不成立（指针出生 txg = 0） | 作废（Z3） |
| PC-P2 | 前提 P2 检查在 sw 三组上 | 「清扫换指针」计数 ≥ 1（每段清扫发了的历史一次） | 作废（Z3） |
| PC-真值 | 真值账本 | skr 里「直接接上」的洞对 ≥ 1，ss 里「中间有存在过的版本」的洞对 ≥ 1 | 作废（Z4） |
| PC-单测 | 两条判法各一张已知答案表（装置里的 `#[test]`） | 每条臂各喂四类输入：E1 那一类、E2 那一类、两类认对的；判出与答案逐格相同 | 单测红就不跑产物 |

**真实基线**：H1-A，即今天 `crates/` 的判法。它在 ss、skr、sw 三组上的 E1 / E2 与甲同一遍算出（附带，不另跑）；在 nrb、rs1 上的 E1 是第 2 行判别力自证要的数（Q13）。

**分段（按岔路排序；每段交回后主 agent 对着问题单判续不续）**

- **第一段（补第 2 行）**：几何与本地常量回比、历史账本、候选集、两条判法与单测、停机条款 S1–S5 的核对、锚点 B3、B4、B7、B8、C1 与 B6 里 G4 那两个数；造 nrb、rs1 并逐张判甲 / H1-A 的分类、跑 `check_pool_image` 取 I-3.9 与 I-3.5 的判定；几何敏感性 G4。补完第 2 行够判（两组都重造并逐张判完）。
- **第二段（补第 1 行）**：造 ss、skr、sw-crash、sw-read，算前提 P1–P3、E1 / E2、清扫触发计数与洞对覆盖；锚点 B1、B2、B5、C2 与 B6 里 G1–G3 那三个数；几何敏感性 G1–G3。补完第 1 行够判。
- **第三段（附带）**：sw-none。第 1 行、第 2 行都够判时不跑，标「够判后未跑」。

## 六、报哪些量与各自的判据

每个量一行、各报各的判定，不合成一个真假。门槛都是 0（翻面取值 ≥ 1）；两条臂的定义里没有一句推得出「错分为 0」，甲的定义只说加一个合取项，所以门槛不是臂定义的同义反复。规模（第 3a 步 ①）：尝试造 1810 张镜像（第五节、第八节各组之和，sw-none 432 张附带不算；第七节 B1–B6）；每张的活是读一遍根环、最新根的实例表、journal 记录与几条根记录的 466 字节，按根数（≤ 3S = 18 条）与洞对数（每张个位数）推是毫秒级；`check_pool_image` 跑 411 次（nrb、rs1 与 G4 的张数）；造历史约 7000 次发布、1600 次挂载，其中 G1 约 375 次挂载（推的：rs1 285 段每段约 20 次发布，别的组合计约 1300 次）；峰值内存按每段一份稀疏镜像（`SparseBlockDevice`，只存写过的块）推每段不到 2 GiB。分辨率（第 3a 步 ②）：全部是整数计数，粒度 1；要分的差是 0 与 ≥ 1，第四节推的各臂基线是 H1-A 在 nrb 上 6、甲 0，sw-crash 上甲 27、H1-A 369，差都 ≥ 1，分得出。

| 量 | 怎么算 | 门槛与判定 | 问题单哪一行、翻面取值 |
|---|---|---|---|
| Q1 前提 P2 的非清扫违例数 | 全部组（ss、skr、sw-crash、sw-read、nrb、rs1 与第八节各点）的历史账本里，写过行的实例（≥ 2）同实例相邻两个存在过的版本，后一版指针出生 txg 既不等于前一版的、又不是「后一版是清扫发布且等于它自己的 txg」的次数；另报「清扫换指针」次数（PC-P2） | 0 ⇒ 前提在这几组上成立；≥ 1 ⇒ 不成立，报那一版是哪条发布路径 | 第 1 行：≥ 1 时第四条件的依据塌了，判乙并给出那一形 |
| Q2 前提 P1 违例数 | 写过行的实例的第一版指针出生 txg ≠ 它的 txg 的次数；实例 1 另报（PC-P1，不计入） | 0 / ≥ 1 | 第 1 行：≥ 1 判乙 |
| Q3 前提 P3 违例数 | 指针出生 txg > 自己 txg 的版本数（实例 1 也算） | 0 / ≥ 1 | 第 1 行：≥ 1 判乙 |
| Q4 甲在 ss 上的 E1 | 432 张逐张逐洞对 | 0 ⇒ 这一组分对；≥ 1 ⇒ 乙，报那一形 | 第 1 行 |
| Q5 甲在 skr 上的 E1 | 45 张 | 同上 | 第 1 行 |
| Q6 甲在 sw-crash 上的 E1 | 432 张 | 同上（第四节第 4 条推的 27） | 第 1 行 |
| Q7 甲在 ss 上的 E2 | 432 张 | 同上 | 第 1 行 |
| Q8 甲在 skr 上的 E2 | 45 张 | 同上 | 第 1 行 |
| Q9 甲在 sw-crash 上的 E2 | 432 张 | 同上 | 第 1 行 |
| Q10 甲在 sw-read 上的 E1 | sw-read 全部张 | 清扫发了的段里 0 ⇒ 作废（Z2）；≥ 1 ⇒ PC-甲中了，照问题单字面也让第 1 行判乙（第一节待认项 2） | 第 1 行（与阳性对照共用） |
| Q11 甲在 sw-read 上的 E2 | 同上 | 0 / ≥ 1 | 第 1 行 |
| Q12 清扫触发计数 | sw-crash、sw-read 各 27 段里，实例 3 挂载交回的 `MountOutput::instance_table_sweep` 是「这次挂载里判、抹、发」且发成的段数；另报清扫那一版的 txg s 与第四节第 4 条推的 b + 2 + w 是否相等 | 不判岔路；< 27 的段那一形没造出来，照 F6 处置 | 第 1 行的有效性闸：清扫那一形要造出来，Q6、Q10 才算量过 |
| Q13 H1-A 在 nrb、rs1 上的 E1 | 6 张、285 张各报张数（「这一张至少一对认错」）与对数 | 每组 ≥ 1 才算这组镜像分得出两判法（C699 的判别力自证）；nrb 为 0 ⇒ 作废（Z1）；rs1 为 0 ⇒ F7 | 第 2 行的有效性闸 |
| Q14 甲在 nrb 上的 E1 | 6 张，报张数与对数 | 0 ⇒ I-3.5 一半在 nrb 上不误；≥ 1 ⇒ 乙 | 第 2 行 |
| Q15 甲在 rs1 上的 E1 | 285 张 | 同上 | 第 2 行 |
| Q16 rs1 上 I-3.9 判违例的张数 | 285 张逐张跑今天的 `singlefs_checker::walk::check_pool_image`，取 `I-3.9` 那一格是 `Violated` 的张数；另报 `NotApplicable`、`Holds` 与「部分没判」各几张 | 0 ⇒ I-3.9 一半不红；≥ 1 ⇒ 乙 | 第 2 行（第一节写死的读法：甲不碰 I-3.9） |
| Q17 nrb 上 I-3.9 判违例的张数 | 6 张，同上 | 同上 | 第 2 行 |
| Q18 洞对覆盖 | 每组按真值三类（直接接上 / 中间有存在过的版本 / R⁻ 不在 R 的祖先链上）各几对；另报指针读不出几对 | 不判岔路；PC-真值（skr 直接接上 ≥ 1、ss 中间有存在过 ≥ 1）不中 ⇒ 作废（Z4）；某组洞对总数为 0 时那一组的 E1 = 0、E2 = 0 记「没有对象」，不记「分对」 | 第 1、2 行的有效性闸（阴性结果要能和「没跑到」分开） |
| Q19 附带：H1-A 在 ss、skr、sw 三组的 E1 / E2；nrb、rs1 上今天 I-3.5 的判定（`check_pool_image` 那一格）；sw-none 全部 | 与上面同一遍算出（sw-none 另造） | 不判 | 附带，够判后不跑（sw-none）；其余与甲同一遍算出，不另跑 |

判定写法：每组每个量各报一行（组名、张数、对数、E1 或 E2 的值、判定），被谓词消费的量另报第八节的轨迹三样。读不到（某张镜像上候选集、行或指针读不出而那一对没判）不当 0：照第十一节 Z6 记，那一张单列。

## 七、钉绝对值的断言

**出自被测条款本身的**（C698、C699 两行；不符时走第十节 F5 或 F8「条款可能错」，不作废）

| 锚 | 值 | 比什么 |
|---|---|---|
| A1 | ss 432 个盘面 | 造出来的镜像张数 |
| A2 | skr 45 步 | 同上 |
| A3 | nrb 6 张 | 同上 |
| A4 | rs1 285 张 | 同上 |
| A5 | nrb 上只认行的判法认错 6 / 6 | Q13 nrb 的张数 |
| A6 | rs1 上只认行的判法认错 285 / 285 | Q13 rs1 的张数 |
| A7 | rs1 上 I-3.9 红 150 / 285 | Q16（条款那一数出自攻方副本；今天的 checker 不同于它时如实并列） |

**独立算出、用命令核过的**（不符才作废，Z5；命令 `python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-design/anchors.py`，sha256 `a9cc365f62f6958060ac4a283325230d1ef1b96996ea5f4d6c9c44d9d7da82c4`，输出原样贴在第十三节）

| 锚 | 值 | 比什么 |
|---|---|---|
| B1 | 9 ×（13 + 16 + 19）= 432 | ss 参数格 × 步数，装置**尝试**造的张数（含写者返回 `Err` 的格） |
| B2 | 3 × 15 = 45 | skr 尝试张数 |
| B3 | 2 + 4 = 6 | nrb 尝试张数 |
| B4 | 5 ×（1 + 4 × 2 + 16 × 3）= 285 | rs1 尝试张数 |
| B5 | 9 ×（13 + 16 + 19）= 432 | sw-crash 尝试张数（sw-none 同） |
| B6 | rs1-S6 114、ss-4MiB 117、skr-S6 45、nrb-S6 6、G1 13 | 第八节各点尝试张数 |
| B7 | 根记录第 [212, 220) 字节 = 实例表指针出生 txg（170 + 42） | 每条写出的根：镜像上这 8 字节（小端）与写者交回的 `instance_table.head.birth_txg` 相等（同时是停机条款 S2） |
| B8 | 369 = 370 − 1 | 本地常量与 `PLACEHOLDER_UNTIL_C690` |
| C1 | nrb（last 10、k 3、bad 1）那一张：洞对恰好 1 对，(R⁻.txg, R.txg) = (10, 12)，R 的指针出生 txg = 11；真值「中间有存在过的版本」；H1-A 认作取号跳过（E1 = 1），甲不认（E1 = 0） | 这一张的洞对表与两条臂的分类（互比「甲 < H1-A」旁边钉的绝对值） |
| C2 | skr（last 6、k 0）那一张：洞对恰好 1 对，(6, 8)，R 的指针出生 txg = 8；真值「直接接上」；两条臂都认作取号跳过（E2 = 0） | 同上 |

C1、C2 的算术只依赖三样读码：取号 = 环与 journal 里见过的最大 txg 加一、暖机次数规则（`mount.rs:5809-5837`）、落点公式（`root_ring.rs:167-175`）；与 `anchors.py` 输出不符而装置的历史账本与 `root_ring_the_ring_judgements_read` 一致时，是读码错，照 Z5 作废后改锚点再跑（改锚点要主 agent 认，记进第十二节）。

## 八、轨迹与几何敏感性

**轨迹**（Q4–Q11、Q14、Q15 被问题单当翻面谓词的输入）：每段历史的 E1、E2 按步 k（sw-read 按版）各报三样——峰值、为正的步数、期末值；每组再报：峰值最大的那一段（参数与值）、为正步数的合计、期末值为正的段数。第四节推的形状：sw-crash 甲峰值 1、为正 1 步（k = 0）、期末 0；H1-A 为正 k_b + 1 步。只报期末值的地方正文不许写「从来」「恒」。

**几何敏感性**（`.claude/rules/mutation-sampling.md` 第六类；每点重跑它对应的那几个量，判定与主取样点同向记两点、反向记「不稳定」）

| 点 | 改哪个旋钮、往哪边 | 为什么方向相反 | 重跑的量 |
|---|---|---|---|
| G1 | sw-crash 的清扫阈值从 0 改成产品占位 369（S = 4、a = 4，挂载 371 次以上攒到 370 行；实例 2 … 实例 n−1 每次挂载只写行与暖机、不写用户数据，实例 n−1 的最后一版记 b，之后照 sw-crash 的步子），k = 0 … 12 | 主点「一行就扫」，这一点「攒满一片才扫」：清扫删一大批行、实例表长出第二片，写行那次重写的是两片链 | Q1、Q6、Q9、Q12 |
| G2 | ss 的 journal 环从 1 MiB 改成 4 MiB（S = 4、last 与 c 同主点），k = 0 … 12 | 短环让 last + 1 那条记录很快被盖；长环里它留得久，「记录还在、实例 2 前几版已被环盖」那一形（第四节第 8 条，H1-A 会错的形）只可能出现在这一边 | Q4、Q7（另报 H1-A） |
| G3 | skr 的 S 从 4 改成 6 | 环大一倍半，last 那一槽留得更久，洞对 (last, last + 2) 活更多步 | Q5、Q8 |
| G4 | nrb 的 S 从 4 改成 6（同 6 个参数格）；rs1 的 S 从 4 改成 6、target 只取 {5, 9}（2 × 57 = 114 张） | 环大，读坏之外别的根留得更多，候选集里同实例的旧版更多 | Q13–Q17 |

判别力自证：每个量的翻面门槛挪到主点与敏感点量出的两个值之间（两值相等时挪到 0.5），判定必须一边绿一边红；两值都是 0 的那几个量这一格记「两点都 0、门槛挪不进去」，不算自证过。

## 九、变异

装置里每条判法与账本的会红分支各配一条；执行员把它们写进 `crates/mutations.tsv` 末尾（入库装置的变异行），锚点照今天的源码逐字只命中一次。每条写明在哪个取样点上改变输出、哪条断言红。

| 变异 | 改什么 | 在哪个取样点改变输出 | 该红的断言 |
|---|---|---|---|
| V1 | 甲的第四条件恒真 | nrb（last 10、k 3、bad 1）：甲的 E1 从 0 变 1 | C1 |
| V2 | 第四条件拿 R 的指针出生 txg 去比 R⁻.txg + 1（不是 R.txg） | skr（last 6、k 0）：R.txg = 8 ≠ 7，甲不再认作取号跳过，E2 从 0 变 1 | C2 |
| V3 | 第四条件读 R⁻ 的指针（不是 R 的） | skr（last 6、k 0）：R⁻ = 6 是实例 1 的版本，指针出生 txg 0 ≠ 8，E2 从 0 变 1 | C2 |
| V4 | 真值账本把「由记录施加出来」的版本当没存在过 | ss 里 (last, last + 2) 成洞对的那几张（执行员从产物里点名第一张；Q18 的 ss「中间有存在过」那一类 ≥ 1 才有这个点）：真值翻成直接接上，两臂 E2 从 0 变 ≥ 1 | PC-真值 与 Q7 那一行的钉值（执行员补成那一张的绝对值断言） |
| V5 | 真值账本把「记录落了而没被施加」的版本当存在过 | skr（last 6、k 0）：真值翻成中间有存在过，两臂 E1 从 0 变 1 | C2 |
| V6 | 读指针出生 txg 的偏移从 42 改成 34（读成出生树号那一段） | 每一张：镜像读到的与写者交回的不等 | B7 / S2 |
| V7 | 候选集漏收由记录施加出来的版本 | sw-crash（任一段）k = 0：b + 3S 那一版若记录还读得出，洞对多出 (b + 3S − 1, b + 3S + 1) 一对；执行员先在不变异的产物里确认这一张有这条记录，确认不了就换 ss 里记录还读得出的一张，都没有就记「取样点不敏感」、照 `.claude/rules/mutation-sampling.md` 第三类补点 | 执行员补的那一张洞对表的绝对值断言 |
| V8 | 前提 P2 检查不数「清扫换指针」（清扫那一支直接跳过、不计数） | sw-crash 任一段：「清扫换指针」次数从 ≥ 1 变 0 | PC-P2 |
| V9 | 前提 P1 检查不排除实例 1 | 每一组：Q2 从 0 变 ≥ 1（实例 1 的第一版） | Q2 那一行的钉值（执行员补） |

H1-A 的「中间实例都是 (j, 0, 0)」与「R 的实例 > R⁻ 的实例」两条：域里没有「中间实例发过东西」的洞对（skr 的中间实例是 (2, 0, 0)，别的组没有中间实例），也没有「行 (i, T) 的 T 恰等于 R⁻.txg 而 R 仍是实例 i」的洞对（有行的实例在写行之后不再发版本），这两条分支没有敏感取样点，不列变异，在第十二节之前若补得出一个合法历史（中间实例写行之后再被跳过）由执行员补组、补变异，补不出写进报告。

## 十、失败条款

| 条款 | 内容 | 什么观测会让它触发 |
|---|---|---|
| F1 | 第 1 行判乙：甲在合法历史上把取号跳过与存在过的版本分不全 | Q4–Q9 任一 ≥ 1（报组、参数、k、R⁻、R、真值链）；按问题单字面 Q10 ≥ 1 同样触发（第一节待认项 2） |
| F2 | 第 1 行判乙：前提「写行之后同实例每一版照抄同一张实例表」除清扫之外还有例外 | Q1、Q2、Q3 任一 ≥ 1，并报那一版是哪条发布路径 |
| F3 | 第 2 行判乙（I-3.5 一半）：甲在重造的 C698 镜像上仍把存在过的版本认成取号跳过 | Q14 或 Q15 ≥ 1 |
| F4 | 第 2 行判乙（I-3.9 一半）：重造的镜像上今天的 I-3.9 仍红 | Q16 或 Q17 ≥ 1 |
| F5 | 条款点名的组今天的写者造不全 | 造出的张数与 A1–A4 不等（B1–B4 尝试张数相等）：报少了哪几格、写者交回的 `Err`；两行按实到的张数判，够不够判交主 agent |
| F6 | 清扫那一形在登记的几何上造不出来 | Q12 < 27（某段实例 3 的挂载没发清扫，或发了但被拒）：那几段报 `InstanceTableSweepAtTheEndOfTheMount` 的成员；Q6、Q10 只按发了的段判，没发的段不算「分对」 |
| F7 | rs1 这一组分不出两判法（C699 的判别力自证不成立） | Q13 的 rs1 张数 = 0 |
| F8 | 条款里的数在今天的写者上不复现（C698、C699 要改数） | A5、A6、A7 与 Q13、Q16 不等；如实并列条款数与量的数，不回头改判据 |

每一条触发后照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」问四句再定去向；F1 打中时先核那一形是不是 sw-read 那一类（R 正是清扫那一版、它前面同实例的版本读坏或被盖），写明是不是第四节第 3、4 条推过的形。

## 十一、作废条款与停机条款

**作废**（整段作废，不出结论，修好重跑）

| 条款 | 什么观测会让它触发 |
|---|---|
| Z1 | PC-H1A 不中：H1-A 在 nrb 上 Q13 的张数 = 0，或 C1 那一张 H1-A 的 E1 ≠ 1 |
| Z2 | PC-甲 不中：sw-read 里实例 3 的清扫发了的段（Q12 计数里的那几段）上 Q10 = 0 |
| Z3 | PC-P1 / PC-P2 不中：实例 1 的第一版没报 P1 不成立；或 sw 三组里清扫发了而「清扫换指针」次数 = 0 |
| Z4 | PC-真值 不中：skr 里「直接接上」的洞对 = 0，或 ss 里「中间有存在过的版本」的洞对 = 0 |
| Z5 | 「独立算出」那一类锚点（B1–B8、C1、C2）任一不符 |
| Z6 | 读不到当成 0：某张镜像上候选集、最新根的实例表或某条根记录读不出，而那一张被计进「分对」；照 `.claude/singlefs-ai-sop/rules/test-discipline.md`「读不到 ≠ 读到 0」，那一段作废重跑，读不出的张单列、不进 E1 / E2 |
| Z7 | 单测（PC-单测）红而产物照跑了 |

**停机**（装置与 `crates/` 对不上：两边都查，既不作废也不当结果，交主 agent；`.claude/rules/implementation-first.md` 第 4 条）

| 条款 | 比什么 | 什么观测会让它触发 |
|---|---|---|
| S1 | 历史账本推出的每个根槽住着哪条根（写过的、没被读坏的）对 `singlefs_checker::walk::root_ring_the_ring_judgements_read` 逐槽交回的（实例、txg） | 任一张任一槽不等（含读坏的那一槽仍被读出） |
| S2 | 镜像上根记录 [212, 220) 字节对写者交回的 `RootRecord::instance_table.head.birth_txg` | 任一条写出的根不等 |
| S3 | 最新根那张实例表：账本按每次挂载写的行与清扫删的行推出的行集，对 `singlefs_core::recovery::instance_table_of_root` 在镜像上读出的 | 任一张不等 |
| S4 | 第五节本地常量表逐个对 `crates/` 那一处 | 任一个不等 |
| S5 | 由记录施加出来的版本：装置按 ①–④ 认出的集合，对账本里「被某次挂载施加过、根槽没落」的集合（再减去 journal 里已读不出记录的那几版，用 `singlefs_checker::walk::journal_records_as_the_recovery_reads_them` 读） | 装置多认或少认一版 |

停机时报：哪一张、哪一条、两边各读出什么、各自依据的源码行；不改装置去凑 `crates/`，也不改 `crates/`。

**候选集本身对不上 `walk.rs` 的那一份核不到**：`walk.rs` 不交出候选集与洞分类（第三节），S1、S3、S5 只核到它的输入；装置的候选集与 `walk.rs:19695-19714` 的取法逐句对抄由执行员在报告里列对照表（哪一句对哪一行），这一项靠读，记进「没做什么」。

**够判停机**：每段交回附一张表，逐行写问题单两行「已够判 / 还差什么 / 剩下的量能不能让它翻面」；第 2 行够判的条件是 nrb、rs1 两组重造完、Q13–Q17 都有数；第 1 行够判的条件是 ss、skr、sw-crash（与 sw-read）造完、Q1–Q12 都有数、第八节 G1–G3 跑完。两行都够判就停，没跑的量（sw-none、第三段）逐条标「够判后未跑」。

这不是重跑登记，没有比对对象表。

## 十二、修订

（装置写之后、产物之前由执行员写，只许收严或补臂。原判据原样保留在第一至第十一节。）

**修订 1（2026-10-29，装置写完、单测与产物都还没跑；依据是派发提示，不是单测读数）**：主 agent 在派发提示「关的是哪几条」一节对第一节「待主 agent 认的项」逐条定案，原句「① 第 2 行只改 I-3.5 的缺口判法、I-3.9 照今天的判；② sw-read 不剔；③ 清扫阈值主取样点取 0，产品默认此刻已改成 365（C690 的补丁在主工作区排着没打，以 365 为准），几何敏感性 G1 那一档取 365；④ 造出来的张数与条款里的 432、45、6、285 对不上时照实报，以装置自己的枚举判，不以对上旧数为够判条件」。照它：

- 待认项 1、2：照第一节写死的读法做，第 2 行的 I-3.9 一半是今天的 `check_pool_image`（Q16、Q17），sw-read 的 E1（Q10）照问题单字面进第 1 行的翻面（F1）。
- 待认项 3：主取样点阈值取 `ANY_ROW`（0）不变；第五节本地常量表第 4 行「清扫阈值占位 369」改成「清扫阈值的产品默认 365」，回比的一边由 `InstanceTableRowCountThreshold::PLACEHOLDER_UNTIL_C690` 换成 `InstanceTableSweepConfiguration::PRODUCT` 的阈值（C690 的补丁把占位常量改名为 `DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE` = 365，两个版本的源码里都有 `PRODUCT`）；第七节 B8 随之读成「本地 365 等于 `PRODUCT` 的阈值」，不等即 S4 停机（C690 的补丁打上之前编出来的装置在这一格停机）。第八节 G1 的阈值取 365：要 366 行才扫，装置一路挂载到某次挂载交回清扫发成为止（上限挂到实例 400），k = 0 … 12 不变（B6 的 13 不变）。G1 那一行「为什么方向相反」写的「实例表长出第二片」在 365 下不成立（一片装 369 行，推的），G1 剩下的反向是「清扫一次删一大批行」；这一点照实写进实验页，G1 照跑。
- 待认项 4：A1–A4 照实报（F5），够判看装置自己的枚举（B1–B6 尝试张数），不以对上 432、45、6、285 为条件。

**修订 2（2026-10-29，同一时点）**：第九节 V4、V7 留给执行员补的取样点与断言写定如下（没有改判据）。V4：ss（S 4、last 4、c 4）里 (4, 6) 成洞对的那几张，单测 `short_ring_hole_left_by_an_overwritten_record_is_an_existed_version_in_between` 钉「真值中间有存在过的版本、两臂都不认」，另有纯函数单测 `truth_reads_the_parent_chain_over_versions_that_existed`。V7：sw-crash k = 0 那一张没有产物可先确认，照第九节那一行的退路换成 ss（S 4、last 4、c 2）挂载之后那一张（k = 0，1@5 的记录读得出、根槽从没落盘），单测 `short_ring_first_image_pins_the_applied_version_and_has_no_hole_pair` 钉「由记录施加出来的只有 1@5、没有洞对」。真值按第一节「祖先链」只走存在过的版本（合法历史里父版本都存在过，读数与只走父版本相同；V4 靠这一句才分得出）。

**修订 3（2026-11-02，第二段产物还没跑；第一段产物 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one.out` 已经跑过，这一条只管第二段、第三段，不回头动第一段的任何判定；依据是主 agent 的定案，不是单测读数）**：S5 的账本一边改字面。依据两处：`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 1909 行主 agent 定的「② S5 字面：根槽落过、后来被盖、记录还读得出而由记录施加出来的那几版是存在过的版本，账本收进去（登记修订 3，跑第二段之前写）」，和这一段派发提示「跑之前要写登记修订 3（S5 的字面）」。起因是上一段执行员的推的（没量）：sw-crash 里实例 4 的挂载盖掉实例 3 写行那一版起的几个根槽，这几版记录还读得出、b 那一槽还住着实例 2 的 b，`walk.rs` 的 ①–④ 四条都成立，装置认它们由记录施加出来，而原字面的账本一边只收「被某次挂载施加过、根槽没落」，这几版根槽落过，原字面下第二段必在 S5 停（报告 `~/.local/share/singlefs/work/m3-close-b19-e198-run/result.md`「结论」第三条）。

- **改什么**：第十一节停机条款 S5「比什么」那一格里账本的一边，原来只有「被某次挂载施加过、根槽没落」的版本（再减去 journal 里已读不出记录的），改成两样的并集（同样再减去记录已读不出的）：(一) 原来那样的版本；(二) **根槽落过、没被某次挂载施加过、后来被环盖掉或根槽读坏，且按账本推的行与根环占位 ① ④ 成立**的版本。① 最新根的实例表（账本推的行集，S3 核过它与镜像相同）里有这个实例的行 (i, Ti)，这一版 txg ≤ Ti；② 账本推的根环占位（减去读坏的槽）里没有这一版；④ 同一份占位里有一条没被那张实例表判抛弃的根，txg 比这一版小。③（不低于回退下界）账本不判：ss、skr、sw 各组不抬 F，F 生效值恒为 0，③ 恒成立。触发条件不变：装置多认或少认一版仍然停机，装置多认的里账本两样都不收的那几版（含根槽落过的）如实列出。
- **不是收严**：这一条把 S5 的账本一边放宽了，原字面在这几版上必停；它只有主 agent 的定案作依据，没有单测读数。它没动 S5 的触发条件、没动第六、七、九节已有的任何一条判据和报告方式，也不动第一至第五、八、十、十四节。第一段的 S5 读数（产物第 12 行，三个计数全 0）是原字面下量的；新字面下第一段的 S5 要照命令再核一遍同值（报告「要主 agent 跑的命令」），不同值则那一张按停机交主 agent，不回改第一段的判定。
- **补一条变异 V10**：第九节第一句「装置里每条判法与账本的会红分支各配一条」，新加的这一支是新的会红分支，补一条（没有去掉或改动 V1–V9）：V10 把这一支里的 ④ 改成恒真，在 sw-crash 取样点（S 4、a 4、d 0）上改变输出——实例 2 在 b 之前、被实例 3 盖掉的几版，记录读得出、① 成立，装置按 ④ 不认（环里最旧的有效根是 b），账本收下它们，两边对不上；必红断言是新单测 `records_applied_comparison_takes_in_the_covered_versions_whose_records_are_still_readable`（它同时钉「装置按 ①–④ 认出的版本里有根槽落过的 ≥ 1」，这是上一段的推的；量出 0 则这一条修订的起因不成立，单测红，交主 agent）。V5 的必红断言不变：V5 把崩掉的那一版记成根槽落过，它仍是「被某次挂载施加过」，不进新这一支（新这一支只收没被施加过的），两边仍对不上。
- **单测数**：25 条加这一条，共 26 条。

**修订 4（2026-11-02，第二段产物还没跑；依据是单测读数加派发提示的一句）**：修订 3 的账本一边里，(一) 那一支也要过 ① ② ④。依据两处：(a) 单测读数，主 agent 经任务 5010 跑装置单测，退 101、`25 passed; 1 failed`，红的是 `records_applied_comparison_takes_in_the_covered_versions_whose_records_are_still_readable`，断言消息原样「S5：k=9 装置认的与账本推的相同：RecordsAppliedComparison { apparatus_extra_with_a_root_that_landed_once: [], apparatus_extra_otherwise: [], apparatus_missing: [VersionKey { instance: 3, txg: 28 }] }  left: 1  right: 0」（红在第一个断言，第二个断言没跑到）；(b) 这一段派发提示「改的若是修订 3 的字面，在登记第十二节再记一条修订并写明依据」。

- **原因（读码推的，没有单独量）**：(3, 28) 是 sw-crash 里实例 3 崩掉的那一版（根槽没落、被实例 4 的挂载施加，账本里 `applied_by_a_mount` 为真、`root_landed` 为假），记录在 k = 9 那一张上还读得出。账本一边的 (一) 那一支只看这两个标志加「记录读得出」，不判 ① ② ④；装置按 `walk.rs` 的 ①–④ 认，④ 要「根环里有一条没被抛弃的根 txg 比它小」。k = 9 那一张上实例 4 的发布已经把环里最旧的有效根盖到 (3, 28) 的 txg 之后，装置按 ④ 不认、账本仍收，于是装置少认一版。k = 0 到 8 那几张上环里还留着比它老的根，两边都收，这几张读数一致，与单测先在 k = 9 红相符。装置的 ④ 抄的是 `walk.rs` 的 `versions_applied_only_by_records`（`oldest_valid_root_txg < record.checkpoint_txg`），没有错；错的是账本一边把 (一) 当成不受 ④ 管。
- **改什么**：第十一节停机条款 S5「比什么」那一格里账本的一边，(一)（被某次挂载施加过、根槽没落）与 (二)（根槽落过、没被某次挂载施加过、后来被环盖掉或根槽读坏）两支都要账本推的行与根环占位让 ① ② ④ 成立，再减去记录已读不出的。①②④ 的写法照修订 3 写的那三条；(一) 那一支的 ② 恒成立（根槽没落过，不在占位里）。触发条件不变：装置多认或少认一版仍然停机。
- **性质**：这一条把账本一边收窄了（原字面在环转过去之后收了装置按 ④ 必不认的版本），不是放宽；它没动 S5 的触发条件、没动第六、七、九节已有的任何一条的去留与报告方式（V10 的说明与原文换成覆盖两支，见下一条）。依据是单测读数，不是产物。第一段的 S5 读数（产物第 12 行，三个计数全 0）是原字面下量的：装置里不在第一段那两组上出现崩掉的版本（nrb、rs1 没有挂载施加出来、根槽没落的版本，推的），第一段 S5 要用新装置再核一遍同值（命令 6），不同值则那一张按停机交主 agent，不回改第一段的判定。
- **V10 的说明改写**（V10 还没追加进 `crates/mutations.tsv`，改的是还没落地的那一行）：V10 把这个共用判断里的 ④ 改成恒真，覆盖两支；在 sw-crash 取样点（S 4、a 4、d 0）的 k = 9 起，崩掉的那一版账本收、装置不认，两边对不上；必红断言仍是 `records_applied_comparison_takes_in_the_covered_versions_whose_records_are_still_readable`。该单测另加一段断言：崩掉的那一版在装置认的张数 ≥ 1、不认的张数 ≥ 1（环转过去前后各有）。单测数不变，仍是 26 条。

**修订 5（2026-11-02，同一时点；命名，不改任何判据与读数）**：门禁 data-names 点了装置里 22 处字符串字面量里的名字（单字母、单字母加数字、缩写 `seq`）；键名与动作名改成说出含义的全词，产物行里的键名一起改。只改名字，判定逻辑、取样点、数值都不动。改名对照（旧 → 新）：

| 旧 | 新 | 在哪 |
|---|---|---|
| `i35_verdicts=` | `reference_interval_verdicts=` | 第一段判决行 Q19 |
| `pc_p1_histories_missed=` | `instance_one_control_histories_missed=` | 失败与作废条款 Z3 的读数（两段） |
| `q14_pairs=` `q15_pairs=` | `nrb_with_pointer_birth_pairs=` `rs1_with_pointer_birth_pairs=` | 条款 F3 的读数 |
| `q16_images=` `q17_images=` | `rs1_release_generation_violated_images=` `nrb_release_generation_violated_images=` | 条款 F4 的读数 |
| `q13_rs1_images=` | `rs1_today_e1_images=` | 条款 F7 的读数 |
| `p1=` `p2=` `p3=` | `first_version_violations=` `adjacent_version_violations=` `birth_after_the_txg_violations=` | 第二段条款 F2 的读数 |
| `b=` | `last_txg_before_the_sweep_instance=` | 第二段清扫历史的 `name=history` 行 |
| 镜像标签 `k=<n>` | `publishes_after_the_mount=<n>` | 第二段 ss、skr、sw-crash、sw-none 的镜像标签 |
| 镜像标签 `v=<txg>` | `version_txg=<txg>` | 第二段 sw-read 镜像标签 |
| 回退动作名 `seq` | `sequential` | 第一段历史名与镜像行里的动作串（`actions` 后面，如 `actionsseq+over` 变 `actionssequential+over`） |

正文里把 k、v、b 当数学符号用的地方（第四节 `k = 0`、`b` 等）不是产物键名，不动。第一段产物要用新装置重出：旧文件 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-one.out` 一个字节不动、留着；新文件名 `…-segment-one-2026-11-02.out`。新旧两份按上表逐行比，除 `name=config` 行（线程数那两格）外只许这几处文字不同、判定相同；`replay.sh` 的第一段登记行改指新文件。

**修订 6（2026-11-02，第二段产物已经跑完之后补写；依据是主 agent 的派发提示，不是单测读数，也不是产物）**：补记修订 4 的射程与一笔输。派发提示的认定：修订 4 把账本两支都过 ① ② ④，实际放宽了停机条款 S5 的账本一边（账本不再收装置按 ④ 必不认的版本，这样的版本不再让 S5 触发）；这只在把 ④ 当成「装置与账本只比环窗口里的版本」这一层射程时成立。窗口外那一版照实记一次输：单测任务 5010 里 k = 9 那一张的实例 3、txg 28，账本收、装置按 ④ 不认，S5 对它不再触发；装置看不见被环盖过的那几版，写进实验页「它答不了的」。

- 第一至第十一节原文与修订 1–5 一字不动；修订 4 性质一条里「收窄、不是放宽」按这一条读：收窄的是账本的版本集合，S5 这条停机条款因此变松。
- 产物读数不因这条改动：第二段产物 `research/results/e198-instance-table-pointer-birth-skip-premise-segment-two.out` 是修订 4 之后的装置跑的，S5 在 1399 张上 `triggered_images=0`，是在这一射程下量的。

## 十三、读过的文件与跑过的命令

**读过的文件**（行号区间；grep 命中行也列）

- `.claude/agent-common.md:1-114`；`.claude/singlefs-ai-sop/rules/test-discipline.md:1-122`；`.claude/rules/three-way-inference.md:1-166`；`.claude/singlefs-ai-sop/rules/evidence-discipline.md:1-179`；`.claude/rules/mutation-sampling.md:1-93`；`.claude/rules/implementation-first.md:1-19`；`.claude/rules/test-script-discipline.md:1-106`
- `research/prompts/m3-close-b19-c699-forks.md:1-8`；`research/prompts/m3-close-b19-design-r2-main-verification.md:1-43`；`research/prompts/_m3-close-b19-design-r2-body.md` 标题行与 `:33-35`
- `.claude/kb/checks-owed.md:422-423`；`crates/singlefs-checker/invariants/05-空间记账.md` 标题行、`:1-8`、`:9`、`:11`、`:12`、`:16`、`:17`、`:27`（grep 命中）
- 线索：`research/prompts/m3-close-b19-checker-r1-sonnet-output.md:267-298`，及 grep 命中 `:46`、`:68`、`:99`、`:167`、`:285`、`:289`、`:315`、`:389`、`:398`；`research/prompts/m3-close-b19-design-r2-opus-output.md:123-141`，及标题行与 `:25`、`:258-259`；`research/prompts/m3-close-b19-design-r2-sonnet-output.md:152-166`，及标题行与 `:36`、`:288`
- 线索模型（已归档，`git show` 读出放草稿目录 `clue-model/`）：`59ce5a865^:research/prompts/m3-candidate-hole-r1-opus-model/proto/{Cargo.toml,src/lib.rs}`、`…/scripts/run.sh`；`04ceaa904^:…/proto/src/bin/skip_row.rs`、`…/scripts/{build,rerun-all,rollback-sweep,short-ring-sweep}.sh`；`31128273b^:…/proto/src/bin/{short_ring,newest_row_bad,rollback_hole,slot_bad}.rs`；`git ls-tree` 列过 `results/` 的文件名，内容没读；`walk-patch/hole_proto.rs` 与 `scripts/pair-modes.py` 只 grep 过函数名与开关名
- `crates/singlefs-checker/src/walk.rs:820-1120`、`:2560-2580`、`:7600-7700`、`:8630-8660`、`:11410-11433`、`:19215-19300`、`:19630-19730`，grep 命中 `:863`、`:913`、`:974`、`:982`、`:7619`、`:7924-7960`、`:8311-8321`、`:8574`、`:8641`、`:10607`、`:19183-19221`、`:20204`、`:20425-20428`
- `crates/singlefs-checker/src/lib.rs:456-496`；`crates/singlefs-checker/src/image.rs` grep 命中 `:1714-1715`、`:1805-1806`、`:1822-1823`
- `crates/singlefs-core/src/mount.rs:1015-1040`、`:5088-5320`、`:5395-5480`、`:5809-5850`、`:7080-7100`、`:8060-8100`、`:9925-9945`，grep 命中 `:34-36`、`:1379-1380`、`:3675-3683`、`:8088-8595` 各入口行、`:9697`、`:10005`
- `crates/singlefs-core/src/instance_table_sweep.rs:1-120`、`:300-420`、`:680-830`；`crates/singlefs-core/src/transaction.rs:14975-15020`、`:1397-1419`（grep）、`:1817-1829`，grep 命中 `:8588`、`:9540`、`:9593`、`:9657`、`:9729`、`:9896`、`:10034`、`:10182`、`:10694`、`:11868`、`:12754-12767`、`:16008-16022`
- `crates/singlefs-core/src/root_ring.rs:161-192`；`crates/singlefs-core/src/root_record.rs` grep 命中 `:259-340`；`crates/singlefs-core/src/make_filesystem.rs:56`；`crates/singlefs-format/src/lib.rs:142`、`:291`
- `crates/singlefs-harness/src/history.rs:1-80` 与 `pub` 项 grep；`crates/singlefs-harness/src/memory_pool.rs:148-156`、`:244-248`；`crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs:660-690`；`crates/singlefs-checker-tier/Cargo.toml` 的 `[dependencies]`
- `research/prompts/e197-preregistration.md:1-58` 与标题行、`:334-338`（只看格式）；`scripts/kb/claim-experiment.sh` 文件头；`scripts/experiment-apparatus/replay.sh:1668-1669`、`:3082`（grep）；`.claude/heavy-thread-default`；`.claude/hooks/lib_long_run.py:11`、`:19`、`:138`；`.claude/hooks/agent-write-scope.tsv:1-27`；`records/2026-09-30-里程碑三开工第一轮.md:544`（grep 命中）

**跑过的命令**（原样，只读与草稿目录里的写）

```
bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-design --purpose '批 19：C699 量前提实验的跑前登记'
grep -rn '\bskr\b\|432 个盘面\|432 盘面\|\bnrb\b\|\brs1\b' --exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude-dir=target --exclude=experiments.md --exclude=experiments-history.md --exclude='*.en.md' --exclude='*.ja.md' .
grep -rn '照抄同一张实例表' --exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude-dir=target --exclude=experiments.md --exclude=experiments-history.md --exclude='*.en.md' --exclude='*.ja.md' .
git log --all --diff-filter=D --name-only --format='COMMIT %h %ad' --date=short -- 'research/prompts/m3-candidate-hole-r1*'
git log --all --format='C %h %ad' --date=short --name-status -- 'research/prompts/m3-candidate-hole-r1-opus-model/proto/src/bin/*' 'research/prompts/m3-root-ring-code-r2-opus-model/proto/src/bin/*'
git show <提交>^:<路径> > <草稿目录>/clue-model/<名>（上面列的各份）
grep -c '^pub fn' crates/singlefs-checker/src/walk.rs   # 8
grep -n 'HoleInTheCandidateSet\|hole_in_the_candidate_set_between' crates/singlefs-checker/src/walk.rs   # 846 858 864 866 896 898 907 963 992
grep -rn 'InstanceTablePlan::Rewrite(\|InstanceTablePlan::Carry(' crates/singlefs-core/src/*.rs
bash scripts/kb/claim-experiment.sh --next   # E198
bash scripts/kb/claim-experiment.sh E198 取号跳过认出生代号的前提
python3 scripts/kb/quote-kb.py <草稿目录>/quoted-clause.md …（第二节那一条原样）
python3 -I /home/fy5090/.local/share/singlefs/work/m3-close-b19-c699-design/anchors.py
```

`anchors.py`（sha256 `a9cc365f62f6958060ac4a283325230d1ef1b96996ea5f4d6c9c44d9d7da82c4`）的输出原样：

```
ss=432 skr=45 nrb=6 rs1=285
sw_crash=432 sw_none=432 sw_read=315
S a d b r3 w3 s w4 s_survives_k0 b_survives_k0 k_last_with_b images_sw_read
4 4 0 16 17 2 20 2 True True 9 8
4 4 1 17 18 1 20 1 True True 10 9
4 4 2 18 19 1 21 1 True True 10 9
4 6 0 18 19 1 21 1 True True 10 9
4 6 1 19 20 2 23 2 True True 9 8
4 6 2 20 21 1 23 1 True True 10 9
4 8 0 20 21 1 23 1 True True 10 9
4 8 1 21 22 1 24 1 True True 10 9
4 8 2 22 23 2 26 2 True True 9 8
5 4 0 19 20 2 23 2 True True 12 11
5 4 1 20 21 1 23 1 True True 13 12
5 4 2 21 22 1 24 1 True True 13 12
5 6 0 21 22 1 24 1 True True 13 12
5 6 1 22 23 2 26 2 True True 12 11
5 6 2 23 24 1 26 1 True True 13 12
5 8 0 23 24 1 26 1 True True 13 12
5 8 1 24 25 1 27 1 True True 13 12
5 8 2 25 26 2 29 2 True True 12 11
6 4 0 22 23 2 26 2 True True 15 14
6 4 1 23 24 1 26 1 True True 16 15
6 4 2 24 25 1 27 1 True True 16 15
6 6 0 24 25 1 27 1 True True 16 15
6 6 1 25 26 2 29 2 True True 15 14
6 6 2 26 27 1 29 1 True True 16 15
6 8 0 26 27 1 29 1 True True 16 15
6 8 1 27 28 1 30 1 True True 16 15
6 8 2 28 29 2 32 2 True True 15 14
rs1_s6=114 nrb_s6=6 ss_4mib=117 skr_s6=45 g1=13
sw_crash 甲 E1 推的 = 27（每段历史 k=0 一张）；H1-A E1 推的 = 369（每段历史 k=0..k_b）
sw_read 甲 E1 推的 = 315（每张一对）
C1 nrb: row=11 w=2 ring=[5, 6, 7, 8, 9, 10, 12, 13, 14, 15, 16] holes=[(10, 12)] R_pointer_birth=11
C2 skr: row=8 w=2 ring=[0, 1, 2, 3, 4, 5, 6, 8, 9, 10] holes=[(6, 8)]
B7 pointer_birth_bytes=[212,220)  B8 placeholder=369
```

## 十四、并行与挂钟

整段估在 2 小时内（推的），各段的数在下面那一行。

独立单位：每段历史（一个组的一个参数格，如 ss 的 (S, last, c)、rs1 的 (target, 动作串, R)）从 mkfs 起自己造、自己判，不读盘、不共享状态，段与段之间没有先后依赖；同一段里每一步的镜像依赖前一步，段内不拆。G1 是一段（371 次以上挂载串着做）。

线程：编译装置经 `bash scripts/hosts/capped.sh 31 …`（登记值 31，`.claude/heavy-thread-default`）；跑的时候按段并行，最多 31 段同时跑（进程内线程池或 31 个单线程进程，每段内存上限按线索模型的 2G 记）；合并按段的次序，计数按段号相加。

串行：段内的步必须串行（后一步在前一步的镜像上发布，次序就是被测对象）；G1 那一段整段串行（行要一次次挂载攒起来）；`check_pool_image` 只在 nrb、rs1 与 G4 的镜像上跑，跟着各自那一段串在段里。没有计时类的量，并行不影响任何判定。

预计挂钟：编译段约 10–25 分钟（推的，照 E197 跑前登记第十四节对同一个包的估计）；分类 1810 张 × 5 毫秒 ≈ 9 秒（推的）；判 I-3.9 共 411 次 × 0.14 秒 ≈ 58 秒（第四节第 9 条，推的）；造历史 7000 次发布 × 30 毫秒 ≈ 210 秒（推的）；挂载 1600 次挂载 × 0.3 秒 ≈ 480 秒（推的）；单线程合计约 13 分钟，31 段并行之后被串着的 G1 拖住：375 次挂载 × 0.3 秒 ≈ 113 秒，跑段约 3–10 分钟；第一段、第二段合计约 15–35 分钟，2 小时内

预计在用核数：编译段 31 个；跑段前几分钟 31 个（31 段并行），尾巴上只剩 G1 那一段时 1 个；出表 1 个。
