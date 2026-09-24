## E156 alloc-basis 四条岔路的代价数 —— 部分已跑，⚠️ `crates/` 有大范围并发改动仍在进行、`replay.sh E156` 现在报「对不上」，本页数字待重验（见「结果」开头警告）（2026-09-22 第一、二段 + 2026-09-23 R2 第一段 + 2026-09-24 R2 第二、三段 + 2026-09-24 R2 续派两段（s4），入库装置，确定性，6 单测 / 4 条变异全抓；覆盖 S1 对拍（含 S1(c) 完整重放）、锚点 K1/锚1/锚2/锚3/K8、H0/HR/HK/HK-F0/HF/Hh/HX/HY 八族历史、岔路 2 全套（Q2a/Q2b/Q2c）、岔路 7 全套（Q7a–Q7f，9 个基底）、岔路 3 一个几何格（Q3a–Q3d 都有值）+ Q3e（X8-A/HY，真实公开入口构造出来了）、岔路 1 两个 S 取样点（S=8/S=4）× 三个洞数取样点（Q1a–Q1d）+ 步数对齐对照（分开「随洞长」与「随步数长」）；岔路 1、3 的 S=16/ρ=1/4/位置「前」/k=4 与 HF 18 格全扫仍未做）

<!-- doc-lint:not-numbers K1 K8 S1 S2 S3 S4 S5 S9 V1 V2 V3 V6 V7 V8 V10 V11 V13 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 F11 F12 F13 F14 F16 Q1 Q2 Q3 Q7 R1 R2 R3 R4 R5 R6 R7 R8 G7 G8 G12 G26 G27 G28 H0 HF K9 U5 U7 U8 -->
本文里 K1、锚1/锚2/锚3（对应登记「七」第二类锚点表逐行的本地简写，不写成编号形状——kb 别的登记表已经用同一形状登记过别的含义，这里避开撞号）、K8、K9、S1–S5、S9、V1–V8、V10、V11、V13、F1–F14、F16、R1–R8、Q1a–Q1f、Q2a/Q2b/Q2c、Q3a–Q3e、Q7a–Q7f、U5、U7、U8、G7、G27、G12、甲-T1、F-扣、H0、HR、HK、HK-F0、HF、Hh(k)、HX、HY、β0/β1/β2/βK-w/r/f/l2/β_syn/β_F0、Bd(k)、Bk1/Bk2/Bk3
这些编号与代号都是 `e156-preregistration.md`／`e156-r2-prereg.md` 跑前登记自己的锚点 / 判据 / 历史族 / 臂名，登记位在那两份文件里，不是 kb 的 D / E / C / I 编号。

**备料**：给 alloc-basis 四条岔路（`research/prompts/alloc-basis-forks.md`）交用户之前配代价数——判决收口表第 ② 行因此挡着第 5、60、54、43 行；用户 2026-09-21 定「先跑计数实验，量完一次全交」。第二、七行判决标着「无运行时代价」，第二行量的是「删掉 D28（挂载期承诺量） 已定项 1『− defer 待释放』这一项要同步改的清单能不能被一条命令核」，第七行量的是「G27（给 defer 账加的一条检查）分不分得出今天两条检查判不出的差别」。等 D28（挂载期承诺量） 已定项 1 与 D16（发布语义） 已定项 1 按岔路定案时引用本实验。

**问题**：跑前登记 `e156-preregistration.md`（637 行，岔路 2 全套由它交用户定）；岔路 1、3、7 还开着，重跑登记 `research/prompts/e156-r2-prereg.md` 只覆盖这三行（岔路单第 9、11、12 行）。R2 第一段（2026-09-23）只做岔路 7；R2 第二、三段（2026-09-24）各做岔路 3、岔路 1，都是**缩小范围版**：只在 S = 8（mkfs 默认）、ρ = 1 一个几何取样点上各跑一条代表性历史。R2 续派（2026-09-24，同日，s4 执行员，两段）在此基础上补：第一段查清「零个洞时 Δ = 20」的来源（装置诊断代码自己的一处过滤漏洞，已修，不改变任何已报的 Δ 值）、给岔路 1 补 S = 4 第二个几何取样点、给岔路 3 的 K9 前提补现核；第二段（主 agent 续派消息「岔路表里还开着两行」）给岔路 1 补步数对齐对照（把「Δ 随洞数长」与「Δ 随步数长」分开）、给岔路 3 构造出 Q3e（X8-A/HY，真实公开入口）并补 Q3d。装置放在**入库装置**上：`crates/singlefs-harness` 的只读 bin `e156_allocation_basis_counts`，驱动真实 mkfs / 暖机 / 第一个事务 / 覆盖写 / 空发布 / 可写挂载 / 管理员回退 / 抬回退下界，不改 `crates/singlefs-core`、`crates/singlefs-checker` 的生产代码；G27 另写一份独立实现（R2 起改成只读被判镜像的记账行，做法在「这一段做了什么」第 7 条）。R2 第一段做登记「五、5.7」第一段：①「十一」S1 与 `crates/` 的逐项对拍（含 S1(c) 完整重放）；②锚点 K1、锚1、锚2、锚3；③R3（G27 改读镜像）；④岔路 7 全套 Q7a–Q7f，9 个基底（β0/β1/β2/βK-w/r/f/l2/β_syn/β_F0）。R2 第二段做岔路 3：一个 HF 历史上 G7（公开入口重组）与 F-扣（真实 `raise_rollback_floor`）各自的 D_rel、K9。R2 第三段做岔路 1：三个 Hh(k) 历史（k = 0、1、2）上甲-T1 与 G12 的多扣、Δ。缩小范围与续派各段还差什么见「它答不了的」与本页历史版本 2026-09-24 条目。

### 这一段做了什么

1. **锚1/锚2/锚3**：单元区每盘 211968 槽（`4 GiB ÷ 16384 − 50176`，与实现侧 `UNIT_AREA_START_SLOT` 回比）、根环 S=8 时 24 槽（与 `target_for_publish` 逐点核对映射一致，0 处不符）、分配记录节点容量 812（`⌊(16384−135)÷20⌋`）——锚1、锚2 在第一、二段用 `python3 -c` 独立复算过；锚3（节点容量）与两个几何锚点这一段改成装置自己在 `main()` 开头现算并 `assert_eq!`，不再是外部脚本核一遍。
2. **K1（S1(a)）**：装置真跑 mkfs→取号→暖机→第一个事务，盘 0 记账第 1 项 13 槽、第 2 项（空闲）211955 槽、第 5 项 1 槽，逐槽落点 `50180:2,50240:1,50242:2,50244:1,50245:1,50246:1,50247:1,50248:1`，与登记逐字相符（`matches_registered=true`）。
3. **S1(c) 完整重放（2026-09-23 R2 第一段，取代第一、二段的「轻量核」）**：装置自己用公开入口重放 `second_transaction_step_four_rollback.rs` 的 `rolling_back_keeps_the_abandoned_records_in_the_ring_and_a_plain_remount_keeps_the_isolation` 场景（A → B(4100B) → 重开取号 2、写行、暖机 → C(2500B) → 回退到 A → 普通重开），逐项现算并 `assert_eq!`：D 的 jsn 接在 C 的 8 之后（`c_counter=8 d_counter=9`）、5 条记录 `(1,4)(2,5)(2,8)(3,9)(3,10)` 原样在环里、普通重开取到实例 4、`isolated_slots_per_device == [(0,34),(1,34)]`、`abandoned_roots_unreadable == 0`——全部通过，是装置自己重新走一遍这个场景、不是确认同名测试还绿。**S1(b)（`second_transaction_step_five_reuse.rs` 两个用例）仍只是现跑测试文件本身，本段没做装置侧重放**，见「它答不了的」。
4. **H0/HR 两个合法状态**：H0（ρ=1，连续覆盖写，无抬 F、无回退）在第 50 步真实撞上 `AllocationRecordsExceedOneNode { records: 820, capacity: 812 }`（`crates/singlefs-core` 今天单节点分配记录树的硬容量——回收只在挂载/回退重建与抬 F 两处发生，登记「三」I3，H0 定义里两样都没有，分配记录只增不减）；β1 取 H0 实际跑到的第 49 步（txg 52）。HR 前缀同理止步于 49 步，回退到候选集里 txg 最小的根（txg 29，实例 1，候选集 24 条），尾段止步于 47 步（txg 102，实例 2）；β2 取 HR 实际跑到的最后一步。两个状态都是真实合法状态（`check_pool_image` 在 β1 / β2 两个端点上跑过；逐步核 checker 判定那一格没做，写在「它答不了的」那一节）。
5. **锚6（S1(e)）**：一次覆盖写（O）释放 10 槽、分配 10 槽，H0 全程 49 步逐步精确（`503 = 13 + 49×10`、`491 = 1 + 49×10`），与登记「四」P6 引 E153（账本形态与环上有洞的代价） 那半「10 槽」相符；空发布（E）本段未测（ρ=1 时 H0 不含 E）。
6. **岔路 2（Q2a/Q2b/Q2c）**：只读 D28（挂载期承诺量） 已定项 1 的公式、D5（快照 / 空间记账机制） 已定项 4 的对照表、`.claude/gate.d/51-admission-terms-covered.sh` 写出清单 L（4 处），再跑登记指定的命令 `grep -rn 'defer 待释放' .claude/ crates/ research/scripts/`（命中 45 处，去重 file:line 后 45 处）；`|M∖L|=41 > 0`，`|L∖M|=0`。门禁 51 号的判别力自证在草稿目录仓副本上做：①只删公式那一项 ⇒ 红；②只删对照表那一行（不改别处）⇒ **仍然红**（`03-空间分配.md:82` 的公式副本没跟着改）；③连同公式副本一起改 ⇒ 绿，且门禁 51 号脚本本身一个字节都没动。
7. **R3（2026-09-23 R2 第一段）：G27 改成只读被判镜像的记账行**，不再读内存里的 `PoolAllocator`（第一、二段那一版两侧都不读被判镜像的缺陷，「历史版本」记着撤回的经过，到这里改掉）：新函数 `allocated_minus_deferred_mismatches_referenced(pool, accounting_slot, referenced)` 直接解析记账树叶字节；旧的内存读法 `allocated_minus_deferred_matches_referenced` 保留成变异反面专用（U8）。
8. **HK / HK-F0 两族新历史（2026-09-23 R2 第一段，登记「五、5.2」）**：HK 是真实历史——β0 → O → E → 无崩溃重开（写行、暖机）→ O×4 → 抬 F 到上限 → O → 管理员回退到候选集里 txg 最小的根 → O → c2 崩溃（记录已持久、根槽没持久）并恢复 → O，覆盖 骨架发布 列的全部发布种类；HK-F0 是同一串操作但分配器强制 `ReuseWindow::ForcedToZero`（不可达阳性对照，R7）。两族各自的四个观测点（无崩溃重开、回退、抬 F、c2 恢复之后）连同 β0/β1/β2、β_syn（β0 镜像做 (第 1 项 −1、第 5 项 −1、第 2 项 +1)）、β_F0（HK-F0 第一个事务之后）凑成 9 个基底，逐个跑 Q7a（Bd(+1)/(−1)/(+8)）、Q7c①②、PC-检查-Bk1/Bk2/Bk3。
9. **发现并修了两处装置自己的实现缺口（写在 `e156-r2-prereg.md`「十二」修订，产物之前）**：① HK 的回退目标选取原样抄了 HR 那段代码（`readable_roots` 里挑 `checkpoint_txg` 最小的一条），没按 `effective_rollback_floor` 过滤候选集——HK 抬过 F 之后 genesis 已经不是候选，直接选最小 txg 会选到一条已经被挡在外面的根，补一条按登记 R7 早就写死、只是先前没写出来的过滤。② `mount_writable`/`mount_rollback` 内部按 `rebuilt_allocator` 重建分配器，`ReuseWindow` 恒回落产品默认，每次挂载/回退之后要重新 `set_reuse_window`，只在最外层设一次不够。
10. **岔路 7 全套（Q7a–Q7f，2026-09-23 R2 第一段，取代第一、二段那版 Q7a/Q7b/Q7c）**：见「结果」。
11. 命名纪律：`naming-lint.sh` 对文件全部通过（0 处违规；R2 这一段又新增并改名了 40 余处单字母/字母加数字的标识符，比如 `g8_prime_is_red` → `allocated_minus_deferred_mismatches_referenced`、`beta_w/r/f/l2` → `beta_after_reopen/rollback/raise_floor/crash_recovery`）。变异表 `crates/mutations.tsv` 对这个 bin 累计 3 条（第一段那条 `referenced_slots` 丢实例表兜底 + R2 新增两条：R8 数分配不数释放、G27 退回读内存分配器的行），三条都手工验证红→绿、已还原、基线仍全绿；门禁 59 号整张表未跑（归 `gate-triage`）。

**2026-09-24 先做的一步：在今天的 `crates/` 上重跑 R2 第一段，与 2026-09-23 的产物逐字节比**——`crates/singlefs-core` 的 `mount.rs`、`transaction.rs`、`recovery.rs`、`walk.rs`、`crash.rs`（`git hash-object` 与「十二」修订 5 记的哈希比对，5 个文件的哈希都变了）已经被另一条线改动。重跑之后只有 3 处不同，全部集中在基底 `HK_l2`（c2 崩溃恢复之后那个观测点）：`base_i31_red` 由 `true` 变 `false`（该基底的 base 状态 I-3.1 由红转绿）、连带 3 行 `q7a` 与 `pc_check_bk2`/`pc_check_bk3_is_bd_plus_one` 的 `i31_red` 字段同向翻转、`q7a_summary` 的 `all_red_count` 由 12 变 15；`q7c1_flip_seen`/`q7c2_flip_seen`、全部 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least` 行逐字节不变。**这条差异不改变岔路 7 的任何已报判定**：Q7c①在可达基底上仍从未转色（V3 仍触发），本轮的差异只是 `walk.rs`（checker I-3.1 实现，另有实现员在改）让 `HK_l2` 的 base 状态从「本来就已经红」变成「真的是绿的合法状态」，使 `HK_l2` 那一格新纳入 `q7a_all_red_count` 的合取判定——这一条不归执行员判断对错，只如实记录读数（派发原话）。
12. **岔路 3（第二段，2026-09-24，`run_hf_single_cell`）：一个 HF 历史（S = 8、ρ = 1）上 G7 与 F-扣 的 D_rel、K9**。历史：β0 → 无崩溃重开（给「现行版本里有重写过的实例表单元」）→ 8 次非空覆盖写 → 从同一份快照分别重建三条分支（探上限、F-扣、G7），先核 V13（抬 F 之前三条分支设备字节与快照逐字节相同）再各自抬 F 到同一个上限（`raise_rollback_floor` 探测得到的 `ceiling`）。G7 用公开入口重组：`raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect` 先推带新 F 的空发布到每块盘都盖到，再 `reclaim_released_up_to(..., ReclaimedReuse::Immediately)`；F-扣 用真实 `raise_rollback_floor`。两条分支的推空发布次数都是 3（与 A9 在 φ 靠近 2 的取样点预测的次数量级吻合），最后一次推空发布的 txg 相同（16），因此两个口径在这一格上 D_rel 完全相同（见「结果」）。**只跑这一个几何格**：登记「五、5.2」HF 的 S ∈ {4,8,16} × ρ ∈ {1,1/4} × φ ∈ {0,1,2} 共 18 格没有全扫；X8-A（Q3e，需要 P5 那个 128 槽两段小池几何 HX/HY）这一轮没有构造，产物里那一行标 `status=not_done`。
13. **岔路 1（第三段，2026-09-24，`run_hh_cell`）：三个 Hh(k) 历史（k = 0、1、2，S = 8、ρ = 1，洞的位置固定「后」）上甲-T1 与 G12 的多扣、Δ**。每格：β0 → 工作负载直到环转过一圈（txg ≥ 3S + 3 = 27）→ 造 k 个洞（c2 崩溃 + 一次可写挂载恢复，每个洞之间隔 2 次工作负载发布）→ 最后无崩溃重开一次（甲-T1「实」读法的回收点，也是这一格的测量点）。甲-T1 的多扣（Q1a）= 这一刻记账第 1 项 − 走读引用（健康状态下等于第 5 项/`deferred`）；G12 的多扣（Q1b）= 遍历 D0 上「已释放」的记录，按 G12 的区间谓词（`环里没有一条根的txg落进[分配代,释放代)`，Hh 没有回退、没有抬 F ⇒`被抛弃∨txg≥F_生效`恒真）逐条判断是否还该扣住，累加扣住的跨度；分配代由装置自己维护的影子账本给（真实记录的 `generation` 字段释放那一刻被改写成释放代，登记「三」N2；账本实现 `snapshot_allocated_generations`/`record_release_generations`，单测 `allocation_generation_ledger_recovers_the_pre_release_generation` 覆盖，变异 `crates/mutations.tsv` 新增一条并手工验证红→绿）。三个洞数取样点上 Δ 单调增（20 → 70 → 120，每加一个洞 +50），h_缺 也单调增（0 → 8 → 14）；见「结果」与「它答不了的」（h = 0 时 Δ 已经是 20，越过 2 槽 / 1 槽两个门槛，这一条的机制没有查透）。**只跑这一个几何格**：S ∈ {4,8,16} × ρ ∈ {1,1/4} 的其余组合、洞的位置「前」、k = 4 都没有扫。

**R2 续派第一段（2026-09-24，同日，s4 执行员，产物 `…-stage3.out`）：**

14. **查清 `holes=0` 时 Δ = 20 的来源：是装置诊断代码自己的一处过滤漏洞，不是 G12 谓词的真实行为**。`run_hh_cell` 给 Q1b（与逐槽列举 `q1_delta_record`）过 `allocator.records()` 时只按 `is_released` 过滤，没核这条记录是不是已经被真实分配器 `reclaim_released_up_to` 回收过——回收之后 `is_released` 仍是 `true`（不删记录，等下一次同起点槽的 `record()` 才覆盖），但槽的位图已清空（`DeviceFreeMap::is_free` 为真）。`holes=0` 那格唯一一次回收（最后无崩溃重开）回收了 9 条记录（11 槽），全部满足「不再扣住」，被诊断循环误当成「G12 判不再扣住、甲-T1 仍算」的证据。修法：过滤条件加 `!device_map_d0.is_free(record.slot)`。**这条修法不改变任何已报的 Δ 值**——`q1a_over_withheld`/`q1b_over_withheld` 都不受影响，`holes=0/1/2` 三格的 `delta`（20/70/120）修法前后逐字节相同，只是逐槽列举更准确了。新增回归测试 `reclaimed_records_are_not_removed_but_their_slots_become_free` 钉住这条修法的前提（第 6 个单测）。
15. **岔路 1 补 S = 4 第二个几何取样点（方向与 mkfs 默认 8 相反）**：ρ = 1、回收时点「实」、洞位置「后」不变，k ∈ {0, 1, 2} 同 S = 8。新增 K8 锚点（核「本地打算跑的 S 与喂给 mkfs 的系统配置字段一致」）、`q1_geometry_sensitivity_s`（S = 8 与 S = 4 上「Δ 越过 2 槽」这条判定是否一致——结果：一致，两点都在 `holes=0` 就已越过）。
16. **岔路 3 的 K9 前提改成现核**：`run_hf_single_cell` 在抬 F 之前新增 `k9_precondition`，用「first_file 那一次 + 8 次覆盖写各一次」这个已知的非空 txg 集合，现读一次 `readable_roots` 数还留在环上的有几条——量出 `non_empty_valid_root_count=9 ≥ 4`，与代码注释的预估一致，但现在是量出来的。
17. **在今天的主工作区上重跑 R2 已有产物，与并发补丁核对**：`crates/` 的 `allocator.rs`/`mount.rs`/`transaction.rs`/`recovery.rs`/`walk.rs`/`format/lib.rs`/`crash.rs`（S3 跟踪的 11 个文件里 7 个）被另一条并发会话改过；把 `stage2.out`（生成于并发补丁之前）里格式没变的 329 行与今天重新跑出来的同名行逐行比对，**逐字节相同**——这批并发补丁没有改到 E156 这几条历史实际触达的代码路径。

**R2 续派第二段（2026-09-24，同日，主 agent 续派「岔路表里还开着两行」，产物 `…-stage4.out`）：**

18. **岔路 1：步数对齐对照，把「Δ 随洞数长」与「Δ 随步数长」分开**。`run_hh_cell` 新增 `matched_to_holes` 参数——`holes=0` 时若非零，把「环转过一圈」的门槛多加 `matched_to_holes × 6` 步工作负载（6 是实测出的「一次造洞固定消耗 6 个 txg」常量，与 S 无关），让总发布步数与 `holes=matched_to_holes` 那格对齐，但不造任何洞。结果：S = 8、S = 4 各两组（`matched_to_holes ∈ {1,2}`）里，终点 txg 与对应真实 `holes=k` 格逐一相同，而 `delta` **全部还是 20**——不随步数增长。据此 `Δ(k) − Δ(0, 步数对齐)` = 50（k=1，两个 S）、100/90（k=2，S=8/S=4）——**这才是「洞上多扣的槽数」，之前报的 20→70→120 曲线，增长部分不是步数带来的，是洞本身带来的**。另外从 `q1_delta_record` 逐槽拆解出 `holes=0` 那 20 槽的构成：两类各 8 条、各 10 槽（分配代 4→释放代 5、分配代 5→释放代 6），机制是「一次挂载里的回收只做一次、且做在重开的最开头，而重开自己还要再推进 2 个 txg」（I3、I13）与洞无关，S=8/S=4 逐槽一致。
19. **岔路 3：Q3e（X8-A/HY）用真实公开入口构造出来了，Q3d 也做了**。新建 P5 的 128 槽两段小池（`x8a_parameters`/`build_x8a_prefix`/`run_x8a_cell`），走 mkfs → 无崩溃重开 → 反复覆盖写到装不下为止。**HX**（填到真耗尽）：F-扣（真实 `raise_rollback_floor`）与 G7（新增可失败版重组）**都**在第一次推空发布上失败、**都**卡在同一个单元（`AccountingTree`，`NoFreeSlotOnAnyDevice`）——两者在这具体历史上卡的是同一步、同一个原因，不是 P5 原始简化模型里「扣住失败、不扣住成功」那种更尖锐的对比（G7 的真实定义是「推空全部结束才回收」，在完全耗尽的池上反而更早没有可用空间）；单独重放「回收并扣住」这一步，实测回收且扣住 66 条记录、81 槽，`free_slots` 从 5 涨到 86（P5「空闲计数报着那些槽」逐字应验），但 `lowest_empty_segment` 仍是 `none`（与 P5 那个「单独一段全空」的简化模型不完全一样，真实历史里两个聚簇段都混着仍分配与被扣住的槽）。**HY**（阳性对照，提前停手，`free=75`）：两个口径都成功，确认 HX 的失败是真耗尽导致，不是装置或探测本身的问题。Q3d（`D_acct`/`D_alloc`）是从代码已读的调用顺序推出来的，不是逐次持久根现测的（F-扣 回收在第一次推空之前 ⇒ `D_acct=0`；两个口径「能发」都在推空全部结束之后 ⇒ `D_alloc(F-扣)=D_acct(G7)=D_alloc(G7)=pump_publishes−1=2`），据实标注方法的限度。

### 结果整行抄自产物

⚠️ **承重的产物是 `research/results/e156-alloc-basis-counts-2026-09-24-stage4.out`（776 行，R2 第一、二、三段 + 续派两段累计，`replay.sh` 已改指向它）**。它在 `stage2.out`（338 行，2026-09-24 01:02，R2 第一、二、三段累计）的基础上，续派第一段追加 Q1 诊断溯源修法、S = 4 几何点、K9 现核（产物 `stage3.out`，682 行，与 `stage2.out` 共有的 329 行「格式没被这两段改动」的行逐字节相同，命令见「复跑」），续派第二段再追加步数对齐对照、20 槽拆解、X8-A/HY、Q3d（`stage4.out`，与 `stage3.out` 共有的 323 行逐字节相同）。`e156-alloc-basis-counts-2026-09-23-stage1.out`（324 行）、2026-09-22 的三份旧产物（31/35/35 行）原样留着当各自阶段的证据，不再承重；岔路 2 的结论（第 5–8 行）不受这几段改写影响，原样保留在下面。

⚠️⚠️ **`crates/` 在续派第二段收尾时又被改动，这次是大范围、还没做完的改动，实测数值大面积地变了，`replay.sh E156` 现在报「对不上」（354 行不同，占全文 776 行近一半）——`git status --short crates/` 显示 `singlefs-core`/`singlefs-checker` 几乎每个源文件都标着 `M`。这不是像修订 10 那次「改了但没碰到 E156 触达的路径」，这次连岔路 7（第一段）的核心读数都变了，下面整份「结果」与「这一段做了什么」的数字要按「生成产物那一刻的 `crates/`」来读，不能当「现在的 crates/ 会给出的数」**，例如：**HR 家族不再在第 47 步撞墙**（旧读数 `hr_lengths actual_tail=47`，新读数 `actual_tail=72`，跑满了没截断；`legal_state family=HR` 的 `item1/item2/item5` 从 txg=78 起不再单调增长、锁死在 `item1=242 item2=211726 item5=230` 不动，直到 txg=127——这与「一次挂载之内记录只增不减」的既有前提（登记「三」I3、I8）明显不一致，是不是分配记录树的容量/回收机制被改了，这一份不判断，交主 agent）；**`Q7d-2` 的 min_item5 从 1（β0）变成 0（`family=HR kind=rollback_row txg=53`）**——这直接冲击岔路 7 这一整段唯一的正式结论「可达合法状态第 5 项从来不是 0」，如果这份新读数站得住，V3/F16 的前提本身就不成立了；`q7b`（total_l1 167→192）、`q7a_summary`（all_red_count 15→18）、`integrity`（legal_state_rows 168→193）这些汇总数也全部跟着变。**岔路 1（Q1a–Q1d、步数对齐、20 槽分类）与岔路 3（Q3a–Q3d、X8-A/HY）续派两段量出的全部数字都在受影响之列**（`holes=0` 的 `delta` 从 20 变成 0，`q3e_f_kou` 的失败方式从 `PlacementRefused` 变成一个新增的 `MountError::RaiseFloorSequencePublishFailed` 包装）。**没有任何一段的读数可以确认「不受这次改动影响」**——上面那句「已经量出并核对过」说的是与更早一版 `crates/`（修订 5/10 记的哈希）逐字节核对过，不是与现在这版核对过。这一份不重新生成产物、不改判定，只如实记这条警告；下一步是等这条并发改动落定再重新跑一轮，还是先按哪个版本的读数交用户，交主 agent 定。

R2 全部段产物收尾行（`grep -n 'name=integrity \|name=done\|name=integrity_r2_segments' research/results/e156-alloc-basis-counts-2026-09-24-stage4.out`）：

```text
E7RESULT name=integrity families=3 legal_state_rows=168 hk_forced_to_zero_rows=9 basis_count=9
E7RESULT name=integrity_r2_segments hf_cells=1 hh_cells=6 hh_holes_sampled=s8h0,s8h1,s8h2,s4h0,s4h1,s4h2 matched_cells=4 matched_sampled=s8m1,s8m2,s4m1,s4m2
E7RESULT name=done emitted=776
```

分类计数（`grep -o 'name=[a-z0-9_]*' research/results/e156-alloc-basis-counts-2026-09-24-stage4.out | sort | uniq -c`）：392 条 `q1_delta_record`（S=8/S=4 各 holes=0/1/2，续派第一段新加的逐槽列举）、177 条 `legal_state`（beta0 1、H0 49、HR 100、HK 18、HK-F0 9）、30 条 `s1d_step`、27 条 `q7a`、10 条 `q1_hh`（S=8/S=4 各 holes=0/1/2）、10 条 `q1_delta_debug`、各 9 条 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least`/`pc_check_bk1`/`pc_check_bk2`/`pc_check_bk3_is_bd_plus_one`/`basis_snapshot`、4 条 `q1_step_matched_diff`（续派第二段）、4 条 `q1d_adjacent_diff`、2 条 `x8a_prefix`/`x8a_held_measure`/`q3e_x8a`/`q3e_g7_remount`（HX、HY 各一条），其余各 1 条或 2 条（`q7a_summary`/`q7b` 等首段量、`q3a_d_rel_g7` 等岔路 3 单格量、`q3d_d_acct_d_alloc`、`anchor_k8`、`k9_precondition`、`q1_geometry_sensitivity_s`）；总计 66 种量名、776 行。

**岔路 2**（第一段，未受这次改写影响）：`|M∖L|=41`（清单漏了 41 处，逐条列在跑前登记「修订」引用的 `/tmp/claude-1000/e156-q2/list-M.txt` 与本报告里，含 `decisions-history` 归档、`milestone/`、`layout/`、多份实验页、`crates/singlefs-core/src/transaction.rs:1488`、`crates/singlefs-harness/src/model.rs:57`、两个测试文件与一份 research 脚本 fixture）；`|L∖M|=0`。门禁 51 号的判别力自证：①红 ②仍红（`03-空间分配.md:82` 的公式副本不跟着改就过不了）③绿；**门禁 51 号脚本本身不需要改**。

**岔路 7（R2 第一段之后：仍不够判，续跑翻不了面，交主 agent 定改判据还是认下）**。R3 改成只读被判镜像之后，逐项判定：

- **Q7b（登记「六」，判据 0）**：`name=q7b red_l1=0 total_l1=167 red_l2=0 total_l2=1`——H0、HR、HK 全部 168 个可达合法状态（L1 167 个、c2 崩溃恢复之后的 L2 1 个）上 G27 零误报。**F5 不触发。**
- **Q7d-1/Q7d-2（新，判据「至少两个第 5 项不同的取样点」「min 值」）**：`name=q7d1_by_kind kind=O min=10 max=10 count=153`、`kind=E min=4 max=4 count=1`——O、E 两种发布自己的释放槽数在全部样本上都是常数（10、4），与 S1(e) 的 assert 一致。`name=q7d2_min_item5 min_item5=1 family=beta0 kind=first_file txg=3`——**168 个可达合法状态里没有一个第 5 项为 0**，最小值 1 出在最早的那个（β0，第一个事务之后），印证 P21(g) 的预估：只要发布过一次带记账行的事务，第 5 项就不可能是 0。
- **Q7d-3 / PC-可达（判据必须 = 0）**：`name=q7d3_pc_reachable hk_forced_to_zero_smallest_item5_item5=0 beta_syn_item5=0 holds=true`——**度量方法本身看得见 0**（HK-F0、β_syn 都测到 0），证明 Q7d-2 的「min=1，从未见过 0」不是量法的盲区，是可达状态集合本身的性质。**V11 不触发。**
- **Q7c①②③（登记「十一」V3 原字面）**：①（不减第 5 项）在全部 **7 个可达基底**（β0、β1、β2、βK-w/r/f/l2）上都是 `flips_red_to_green=false`——比第一、二段（只测过 β0/β1/β2 三个）多测了 HK 的四个观测点，结论没有变、证据强了一倍多；只在 **2 个不可达基底**（β_syn、β_F0，第 5 项人为造成 0）上转色。②（`==`→`>=`）在全部 7 个可达基底上都 `flips_red_to_green=true`（`name=q7a_summary all_red_count=15 q7c1_flip_seen=true q7c2_flip_seen=true`，2026-09-24 在今天的 `crates/` 上重跑：`all_red_count` 由昨天的 12 变成 15——差值全部来自 `HK_l2` 那一格的 `base_i31_red` 由 `true` 转 `false`（另有实现员在改 `walk.rs` 的 I-3.1 实现，见上方「先做的一步」），`q7c1_flip_seen`/`q7c2_flip_seen` 本身与逐条 `q7c1_not_subtracting_defer`/`q7c2_threshold_at_least` 行都逐字节不变）；在 β_syn/β_F0 上**无从执行**（`not_applicable=true reason=deferred_is_zero_cannot_subtract_one`：那两个基底第 5 项已经是 0，Bd(−1) 会把它减成负数，`e156-r2-prereg.md`「十二」修订 3 记了这条发现）。⇒ 按 V3 字面，**Q7a 整张表仍然作废**——①在可达基底上从来看不到转色，根子还是「可达合法状态第 5 项从不为 0」（Q7d-2 已经把这句从「两个基底」的观察扩成「168 个状态、7 个覆盖八种发布种类的基底」全部同一个结论）。
- **Q7e（新，把「判据在可达状态上没有判别力」与「装置或 G27 写坏了」分开）**：`name=q7e beta_forced_to_zero_flip1=true beta_forced_to_zero_flip2=false beta_syn_flip1=true beta_syn_flip2=false beta_forced_to_zero_i31_red=true beta_forced_to_zero_i52_red=false beta_syn_i31_red=true beta_syn_i52_red=false`。①在两个不可达基底上都转色（**V10 不触发**）；②这一行的 `flip2=false` **读不出「转色失败」还是「不适用」**——本行没有带 `not_applicable` 字段，要去同一份产物里对应基底的 `q7c2_threshold_at_least` 行核对，那两行才写着 `not_applicable=true`。**这是产物本身的一处可读性缺口，交主 agent（下面单列一条）**。`beta_*_i31_red=true` 说明这两个基底为什么不是合法状态：造它们时人为改动了记账字节而没有同步改走读引用能看到的东西，`I-3.1（已分配统计对得上）` 天然判红。
- **Q7f（附带，不解除 V3）**：`name=q7f_count count=7`——**全部 7 个可达基底**上，真实 G27 判绿而「不减第 5 项」的变体判红：G27 比这个特定的坏变体更不容易漏报，但这条不进 V3 的判定，只是留给主 agent 的候选数据（登记「六」Q7f 原话）。
- **S9（停机，不当结果、不作废）**：`name=s9_raise_floor_refused family=HK-F0 error=RollbackFloorCeilingNeedsUnreadableValidRootTreeTable { root: RollbackTarget { instance: InstanceGeneration(0), checkpoint_txg: CheckpointTxg(0) }, failure: UnitUnreadable { slot: SlotNumber(50178) } }`——HK-F0 的抬 F 探测上限撞上这个错误：`ForcedToZero` 下 `release()` 会把释放代 ≤ 这次 txg 的全部已释放记录一起回收（`allocator.rs`），genesis 的树表单元在 HK-F0 早期就被这样收走挪作他用，`rollback_floor_ceiling` 判非空要读候选根的树表，读不到就报错——这正是 I-7.4（近 K 代块未被复用） 要挡的那类复用，HK-F0 故意不可达，撞上了停这一格（只影响 HK-F0 后半段：`beta_after_raise_floor`/`beta_after_rollback`/`beta_after_crash_recovery` 三个字段在 main() 里从没被 HK-F0 引用过，S9 不影响任何已报的判定）。

⚠️ **交主 agent 的两点**：① Q7e 输出行本身应当带 `not_applicable` 标记而不是把「不适用」压成 `false`，同「基底已红」的 PC-检查行一样（登记 5.5 footnote 已经预见了「基底已红不算失败也不算通过」这条，这里同一类问题换了个地方冒出来）；② V3/F16 已经确认到「7 个可达基底、168 个可达状态，Q7c① 从没转过色」，续跑不会再翻面——改判据（不再要求红→绿的严格翻转）还是认下（「G27 的判别力由 Q7b 加上坏镜像那一格干净对照立住」），交主 agent 判。

**岔路 3（R2 第二段，2026-09-24，一个几何格：S = 8、ρ = 1，`run_hf_single_cell`）**：

```text
E7RESULT name=v13_hf_prefix_identical holds=true
E7RESULT name=q3a_d_rel_g7 release_generation=13 dispensable_txg=16 d_rel_non_empty=0 d_rel_all=2 pump_publishes=3
E7RESULT name=q3b_d_rel_f_kou release_generation=13 dispensable_txg=16 d_rel_non_empty=0 d_rel_all=2 pump_publishes=3
E7RESULT name=q3c_diff max_diff=0 threshold=3 verdict=difference_not_large
E7RESULT name=k9_newest_batch_d_rel_non_empty g7=0 f_kou=0 expected=3 note=see_p21e_prediction
E7RESULT name=q3e_x8a status=not_done reason=hx_hy_small_pool_geometry_not_built_this_round
```

- **V13 holds=true**：抬 F 之前，探上限 / F-扣 / G7 三条分支各自从同一份快照重建，逐字节相同——两条口径的历史前缀确实一样，Q3a/Q3b/Q3c 不因为「起点就不同」而失去可比性。
- **Q3a（G7）与 Q3b（F-扣）**：同一批被抬 F 处置放开的槽（释放代 = 13，抬 F 之前 D0 上最大的已释放 generation）在两个口径上「分配器真的能把它发出去」的时刻**完全相同**（`dispensable_txg=16`，两条口径推空发布次数都是 3）——`d_rel_非空 = 0`（16 与 13 之间没有非空发布，只有推空的空发布）、`d_rel_全 = 2`。**这不是巧合**：G7 与 F-扣 在这一格里推空发布的次数只由「哪块盘还没盖到带新 F 的根」决定，与谁先回收无关，两条口径因此在同一段历史上推出同样长的空发布序列，最后一次推空发布的 txg 自然相同。
- **Q3c（两个口径的差）**：`max_diff=0 ≤ 3` ⇒ **差别不大**——两个口径各一个数、差是 0，第 11 行「够判条件」的第一半满足。⚠️ **只在这一个几何格上量到**，没有跑登记「五、5.2」HF 的 18 格全扫（S × ρ × φ），也没有验证「差别不大」在别的 φ（抬 F 起点相位）或别的 S 上是否仍然一致——不许外推成「两个口径在所有几何上都差别不大」。
- **K9（新，released 记录唯一那一批的 D_rel_非空）**：`g7=0 f_kou=0`，**都不是登记预期的 3**。K9 的前提（抬 F 那一刻环里 ≥ 4 条非空有效根）**续派第一段已经现核**（不再是没查过）：`name=k9_precondition non_empty_valid_root_count=9 threshold=4 holds=true`——前提成立，「= 3」不成立不是因为前提没满足，而是这一批（release_generation=13, dispensable_txg=16）之间的 3 次持久发布恰好都是抬 F 自己的推空发布，本来就不算「非空」，与 P21(e)「越早释放等得越久」是同一方向的现象。K9 只报数，不作为判据，交主 agent 决定是否需要专门构造别的几何来验它的方向性。
- **Q3e（X8-A）与 Q3d：续派第二段做出来了**，见下面「R2 续派第二段」小节，不再是 `status=not_done`。

**R2 续派第一段（2026-09-24，同日，s4 执行员，产物 `…-stage3.out`）新增**：

```text
E7RESULT name=anchor_k8 configured=4 local=4 holds=true
E7RESULT name=q1_hh s=4 holes=0 txg=17 allocated=132 deferred=120 referenced=12 q1a_t1_over_withheld=120 q1b_g12_over_withheld=100 delta=20 h_missing=0
E7RESULT name=q1_hh s=4 holes=1 txg=23 allocated=176 deferred=164 referenced=12 q1a_t1_over_withheld=164 q1b_g12_over_withheld=94 delta=70 h_missing=8
E7RESULT name=q1_hh s=4 holes=2 txg=29 allocated=220 deferred=208 referenced=12 q1a_t1_over_withheld=208 q1b_g12_over_withheld=98 delta=110 h_missing=8
E7RESULT name=q1_geometry_sensitivity_s crossing_2_by_s=s8=true,s4=true stable=true
```

S = 4（方向与 mkfs 默认 8 相反）上 Δ 同样单调不减（20 → 70 → 110），「越过 2 槽」这条判定与 S = 8 一致（`stable=true`）——S = 4 的 `h_missing` 在 holes=1/2 都停在 8（不像 S = 8 那样 8 → 14），`delta` 在 holes=2 也是 110 不是 120：环更浅时洞的计数方式与多扣的绝对量确实随几何变，但方向性判定没有变。

**R2 续派第二段（2026-09-24，同日，主 agent 续派，产物 `…-stage4.out`）新增**：

```text
E7RESULT name=q1_step_matched_diff s=8 k=1 txg=35 delta_k=70 delta_0_matched=20 diff=50 h_missing_k=8 h_missing_0_matched=0
E7RESULT name=q1_step_matched_diff s=8 k=2 txg=41 delta_k=120 delta_0_matched=20 diff=100 h_missing_k=14 h_missing_0_matched=0
E7RESULT name=q1_step_matched_diff s=4 k=1 txg=23 delta_k=70 delta_0_matched=20 diff=50 h_missing_k=8 h_missing_0_matched=0
E7RESULT name=q1_step_matched_diff s=4 k=2 txg=29 delta_k=110 delta_0_matched=20 diff=90 h_missing_k=8 h_missing_0_matched=0
E7RESULT name=x8a_prefix pool=hx overwrite_count=10 stop_reason=PlacementRefused { unit: InodeRoot, refusal: NoFreeSlotOnAnyDevice } allocated=123 free=5 deferred=111
E7RESULT name=x8a_held_measure pool=hx ceiling=12 reclaimed_and_held_records=66 reclaimed_and_held_span=81 allocated_after_hold=42 free_after_hold=86 deferred_after_hold=30 lowest_empty_segment_after_hold=none
E7RESULT name=q3e_f_kou pool=hx first_push_result=failed_placement_refused:PlacementRefused { unit: AccountingTree, refusal: NoFreeSlotOnAnyDevice } release_reclaim_holds_reached=false
E7RESULT name=q3e_g7 pool=hx first_push_result=failed_placement_refused:PlacementRefused { unit: AccountingTree, refusal: NoFreeSlotOnAnyDevice } release_reclaim_holds_reached=not_applicable_g7_never_holds
E7RESULT name=q3e_x8a pool=hx status=done f_kou_stuck=true g7_stuck=true both_stuck_at_same_step=true note=see_x8a_held_measure_and_f_kou_g7_lines
E7RESULT name=x8a_prefix pool=hy overwrite_count=3 stop_reason=stopped_at_cap_3_by_request allocated=53 free=75 deferred=41
E7RESULT name=q3e_f_kou pool=hy first_push_result=succeeded release_reclaim_holds_reached=true
E7RESULT name=q3e_g7 pool=hy first_push_result=succeeded release_reclaim_holds_reached=not_applicable_g7_never_holds
E7RESULT name=q3e_x8a pool=hy status=done f_kou_stuck=false g7_stuck=false both_stuck_at_same_step=true note=see_x8a_held_measure_and_f_kou_g7_lines
E7RESULT name=q3d_d_acct_d_alloc d_acct_f_kou=0 d_alloc_f_kou=2 d_acct_g7=2 d_alloc_g7=2 method=derived_from_documented_call_order_not_per_push_measured
```

- **步数对齐对照（岔路 1 够判条件里「随洞的个数怎么长」这句现在有了干净的对照）**：`Δ(0, 步数对齐到 holes=k)` 在全部四组（S=8/S=4 × k=1/2）里**都还是 20**，与 `holes=0`（未对齐）完全一样——同样的总发布步数、只是不造洞，Δ 不涨。`Δ(k) − Δ(0, 步数对齐)` = 50（k=1，两个 S）、100/90（k=2，S=8/S=4）：**这才是洞本身贡献的多扣，之前的 20→70→120 曲线，增长部分不是步数带来的**。
- **`holes=0` 那 20 槽的分类（上一段「没有查透」这条现在有答案了）**：`q1_delta_record`（S=8/S=4 两个几何点逐槽一致）显示只有两类、各 8 条各 10 槽——分配代 4→释放代 5、分配代 5→释放代 6。机制：ring-fill 循环停在某个 txg 时环里最旧有效根还没推进到那两个释放代，紧接着的「最后一次无崩溃重开」用当时的 floor 回收，但重开自己的写行 + 暖机又把环推进 2 个 txg，使这两批释放留在「已回收但比当时 floor 新」的状态——与洞完全无关，S=8/S=4 逐槽相同（读代码得出的机制，不是新读出的独立事实，详见 `e156-r2-prereg.md`「十二」修订 12）。
- **Q3e（X8-A/HY）真实构造出来了**：HX（填到真耗尽）上 F-扣 与 G7 **都**卡在同一步（`AccountingTree` 分配不到落点）、同一个原因；HY（阳性对照，提前停手）上两个口径**都**成功——确认 HX 的失败是耗尽导致，不是装置的问题。单独测过「回收并扣住」这一步：`free_slots` 从 5 涨到 86 但 `lowest_empty_segment` 仍是 `none`（P5「空闲计数报着那些槽」逐字应验，但这段真实历史比 P5 的简化模型更混乱——两个聚簇段都混着仍分配与被扣住的槽，不是干净的「一段活一段空」）。**第 11 行「X8-A 那一格两者各卡在哪」现在有答案**：两者卡在同一步，不是 P5 简化模型暗示的「扣住失败、不扣住成功」那种更尖锐的差别。
- **Q3d**：`D_acct(F-扣)=0`、`D_alloc(F-扣)=D_acct(G7)=D_alloc(G7)=2`——从代码已读的调用顺序推出来的（F-扣 的回收在第一次推空之前、两个口径「能发」都在推空全部结束之后），不是逐次持久根现测的，据实标注方法限度。

**岔路 1（R2 第三段，2026-09-24，一个几何格：S = 8、ρ = 1，位置固定「后」，`run_hh_cell`）**：

```text
E7RESULT name=q1_hh holes=0 txg=29 allocated=252 deferred=240 referenced=12 q1a_t1_over_withheld=240 q1b_g12_over_withheld=220 delta=20 h_missing=0
E7RESULT name=q1_hh holes=1 txg=35 allocated=296 deferred=284 referenced=12 q1a_t1_over_withheld=284 q1b_g12_over_withheld=214 delta=70 h_missing=8
E7RESULT name=q1_hh holes=2 txg=41 allocated=340 deferred=328 referenced=12 q1a_t1_over_withheld=328 q1b_g12_over_withheld=208 delta=120 h_missing=14
E7RESULT name=q1d_adjacent_diff holes_from=0 holes_to=1 q1a_from=240 q1a_to=284 q1b_from=220 q1b_to=214 delta_from=20 delta_to=70 diff=50 h_missing_from=0 h_missing_to=8
E7RESULT name=q1d_adjacent_diff holes_from=1 holes_to=2 q1a_from=284 q1a_to=328 q1b_from=214 q1b_to=208 delta_from=70 delta_to=120 diff=50 h_missing_from=8 h_missing_to=14
E7RESULT name=q1d_first_crossing threshold_2_slots=holes=0 txg=29 delta=20 threshold_1_slot=holes=0 txg=29 delta=20
E7RESULT name=q1d_monotonic holds=true
```

- **Q1a / Q1b / Δ 随洞数怎么长**：三个取样点（k = 0、1、2）上 Δ 分别是 20、70、120，**单调增、每加一个洞恰好 +50**（`q1d_monotonic holds=true`）；h_缺 同步单调增（0 → 8 → 14）。**第 9 行「够判条件」逐字满足**：至少两个洞数不同的取样点各有 Δ，且报得出越过 2 槽 / 1 槽两个门槛的规模——**在这三个取样点上，Δ 从 h = 0 起就已经越过两个门槛**（`q1d_first_crossing` 两栏都指向 `holes=0 txg=29 delta=20`）。
- **h = 0 时 Δ = 20 的来源，已在续派两段查清（不再是「没有查透」）**：一是诊断代码的一处过滤漏洞（不改变 Δ 的数值，见「这一段做了什么」第 14 条）；二是这 20 槽本身是「一次挂载只回收一次、重开自己还要再推进 2 个 txg」这条实现事实造成的、与洞无关的基准量（见续派第二段的 20 槽分类）；三是步数对齐对照证明「+50/洞」的边际效应确实是洞本身的贡献、不是步数的贡献。**读这三个数时**：20 是与洞无关的基准（S=8/S=4 逐槽一致），50/洞（k=1）与 100(S=8)/90(S=4)/洞（k=2）才是「洞上多扣的槽数」——第 9 行「洞上多扣的槽数」这句问的正是后者，不是包含基准值的聚合曲线。
- **续派第一段补了 S = 4，其余几何维仍未扫**：S = 16、ρ = 1/4、回收时点「每」、洞位置「前」、k = 4 都没有跑，第三段 a/b 的完整 24+14 条历史仍未做；登记「五、5.6」第六类要求的其余几维（回收时点、洞位置）没有取到方向相反的第二个点。

### 复跑

```
bash research/scripts/replay.sh E156
```

报「字节一致」（`replay.sh` 的 `driver_e156` 直接 `cargo run -q -p singlefs-harness --bin e156_allocation_basis_counts`，先例同 E142（第一个事务的干跑），两个 cargo workspace 互相看不到对方，不能合并成一次调用；确定性——同一个二进制跑两遍逐字节一致，V8 未触发，续派两段各自现跑过；产物换到 `…-stage4.out`，sha256 `5311e01a0cec4797bf848ffacbbf3f624b6a01627412491c76a8c92f2cea0f02`）。单测：`cargo test -p singlefs-harness --bin e156_allocation_basis_counts` → `6 passed`（续派第一段新增 1 条：`reclaimed_records_are_not_removed_but_their_slots_become_free`，钉住 Q1 诊断修法的前提；此前 5 条不变）。变异：`crates/mutations.tsv` 对这个 bin 累计仍是 4 条（续派两段没有新增变异行——新加的诊断/构造代码要么不改变任何已判的量、要么是新的产品行为组合，不是候选/判据分歧，加变异的收益与工作量都没到位，逐条理由见「它答不了的」），逐条手工改坏、`cargo test` 判红、还原后再判绿（未跑门禁 59 号整张表，归 `gate-triage`）。**手工验证过一条没红的变异**（G7 重组里 `ReclaimedReuse::Immediately` 改成 `HeldUntilFloorTakesEffect`）：当前的 Q3a/Q3b/Q3c/K9 输出逐字节不变，按 `mutation-sampling.md`「三类」判为**第一类：真盲区**——`run_hf_single_cell` 那条历史在回收之后没有再做一次分配尝试；**X8A（续派第二段）从另一个角度补了这条盲区的证据**：`x8a_held_measure` 直接量出「回收但扣住」之后 `free_slots` 涨了、`lowest_empty_segment` 却仍是 `none`，`q3e_f_kou`/`q3e_g7` 也确实在这两种回收方式上表现不同（HX 上二者都失败但是不同代码路径导致），但这仍不是给 `run_hf_single_cell` 那条历史加的变异，两处是两个独立的观测点，不进 `crates/mutations.tsv`。

### 它答不了的

- **S1(b)（`second_transaction_step_five_reuse.rs` 两个用例）仍没有装置侧重放**：只现跑了测试文件本身，不像 S1(c) 那样在装置里重新走一遍那个场景再逐项 `assert_eq!`。
- **S1(d) 只核了 G27 与 `check_pool_image` 两条检查在 H0 前 30 步（3S+6）逐步的判定，没有核第三条独立算法**：`s1d_step` 那 30 行只报 `item1/item2/item5`（镜像读法）与今天两条检查的判定，三者之间没有第三条完全独立的路径互证——`item1/item2/item5` 与 G27 用的是同一次 `mirror_accounting_row_slots` 读法。
- **岔路 7 的判别力自证①在可达状态上做不出转色，Q7d-2/Q7e 已经把「量法本身盲区」这一种解释排除掉**（Q7d-3 证明量法看得见 0），剩下的只有「可达合法状态第 5 项从来不是 0」这一条，且已扩到 7 个基底、168 个状态——**这不是本段答不了的，是这一段唯一的正式结论**：续跑这套自证设计翻不了面，要么改判据（不再要求红→绿的严格翻转），要么认下（G27 的判别力由 Q7b + 坏镜像干净对照立住），这两条路怎么选，本段不定，交主 agent。
- **Q7e 那一行的 `flip2=false` 读不出「转色失败」还是「不适用」**：字段本身没带 `not_applicable` 标记，要跨行去对应基底的 `q7c2_threshold_at_least` 输出核对——这是产物的一处可读性缺口，不是判定错，见「结果」末尾那条给主 agent 的提醒。
- **H0/HR 的实际长度被 `AllocationRecordsExceedOneNode` 收窄到 49/47 步，不是登记的 144/72+72**：这是装置在今天这份 `crates/` 上量到的真实边界，不是我选的参数。Hh(k) 族避开了这堵墙（每格只跑到 txg 29/35/41 或更小，远没到 812 条记录的墙），但**没有跑登记「五、5.2」要的那种连续 6N 步长历史**，撞不撞墙、撞墙之后的规模没有另外验证。
- **HK-F0 撞上 S9（抬 F 探测上限时候选根树表读不出）之后停在那一格**：`beta_after_raise_floor`/`beta_after_rollback`/`beta_after_crash_recovery`（HK-F0 那三个观测点）没有真正跑出来，main() 里也从未引用它们，不影响任何已报的判定，但 HK-F0 家族「每一种发布至少出现一次」这条覆盖到抬 F 之后就停了。
- **旁路评估的射程**（登记「十一」）：G12 只在真实历史旁边做谓词评估，不改分配器行为，答不了「按它跑出来的历史会长成什么样」；G7 这一份已经改成用公开入口重组真实历史（R4），不再是纯旁路评估，但仍不改真实分配器的产品路径。
- **岔路 1：S = 16、ρ = 1/4、回收时点「每」、洞位置「前」、k = 4 都没跑**：第三段 a/b 的完整 24+14 条历史仍未做；`Q1c-洞`（把 h_缺 = 0 与 h_缺 ≥ 1 分开报）、`Q1e`（附带）、PC-多扣、PC-洞（阳性对照）都没做；M9 的周期重开表没有实现（这一段的 Hh 族只在最后做一次无崩溃重开，与登记「每 S 次工作负载发布重开一次」不同，见「这一段做了什么」第 13 条与本页「历史版本」）。
- **岔路 3：HF 的 S ∈ {4,8,16} × ρ ∈ {1,1/4} × φ ∈ {0,1,2} 共 18 格里 17 格没跑**：Q3a–Q3d、K9 只在一个几何格（S=8、ρ=1）上量出；X8-A/HY 用的是另一个独立小池（P5 几何），只跑了一个「填到耗尽」与一个「提前停手」的取样点，没有扫过介于两者之间的中间态（比如「刚好差 1 槽放不下」那种边界）。
- **Q3d 是推出来的，不是逐次持久根现测的**：`q3d_derived` 的方法限度见「这一段做了什么」第 19 条，更强的验证方法（逐次持久根现读记账行）没有做。
- **X8-A 在 P5 的简化模型与这一轮的真实历史构造上不完全对应**：P5 原本的对比是「扣住失败、不扣住成功」，这一轮两个真实口径（F-扣 扣住、G7 从不扣住）在完全耗尽的池上都失败、卡在同一步——「不扣住就成功」这个 P5 式的更尖锐对照，这两个真实口径都没有走到（G7 是「从不扣住」但也「从不提前回收」，不是 P5 那种「回收但不扣住」）；有没有必要另建一个「回收但不扣住」的第三口径去对齐 P5 的原始模型，这一轮没有做，交主 agent。
- **G7 重组「立刻可发」与「回收但扣住」的差别，作为一条变异仍是真盲区**（`mutation-sampling.md` 第一类）：`run_hf_single_cell` 的测法在回收之后没有再做一次分配尝试，看不出这个差别；X8-A 的 `x8a_held_measure` 从另一个角度量到了这个差别的效应（`free_slots` 涨但 `lowest_empty_segment` 仍是 `none`），但两处是独立的观测点，没有把这条盲区补进 `crates/mutations.tsv`。
- **门禁 59 号（`crates/mutations.tsv` 整张表复跑）未跑**：这个 bin 累计的 4 条已逐条手工验证红→绿，全表复跑归 `gate-triage` 统一跑。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-09-24 不受影响：`grep -n 'E156（' .claude/kb/decisions/28-挂载期承诺量.md` 零命中，这一分项的依据段仍没有引用 E156（alloc-basis 四条岔路的代价数）；R2 第二、三段新加的岔路 3、岔路 1 读数（Q3a–Q3c、Q1a–Q1d）都不是这一分项的对象（它管准入式子，不是回收时点或多扣），没有改变这条判断。是否该升成支撑/推翻交主 agent |
| D5（快照 / 空间记账机制） 已定项 4 | 备料 | 2026-09-24 不受影响：`grep -n 'E156（' .claude/kb/decisions/05-快照-空间记账机制.md` 零命中，依据段没有引用 E156；R2 第二、三段的新结论没有改变这条判断。是否该升成支撑/推翻交主 agent |
| D16（发布语义） 已定项 1 | 备料 | 2026-09-24 不受影响：`grep -n 'E156（' .claude/kb/decisions/16-发布语义.md` 零命中，依据段仍没有引用 E156；**Q3a–Q3d（两个 F 口径 D_rel 相同、差别不大）、K9（前提已现核）、Q3e/X8-A（两个真实口径在耗尽的池上卡在同一步）、Q1a–Q1d（洞数与多扣差 Δ 单调，步数对齐对照证明增量确实来自洞）都是这一分项判据「用户知情接受的代价三条」「回退候选集/回退下界」直接相关的量，证据面比之前宽（S=8/S=4 两个几何点、HX/HY 两个小池取样点），但仍不是登记「五、5.2」的全量扫描——是否该把这一分项从「备料」升成「支撑」（或触发重议），交主 agent 判，本页不自己升级 |
| D3（空间分配） 已定项 7 | 备料 | 2026-09-24 不受影响：`grep -n 'E156（' .claude/kb/decisions/03-空间分配.md` 零命中，依据段没有引用 E156；岔路 1 读数（甲-T1 与 G12 的记账第 1 项形态）与这一分项的记录格式相关，续派两段把取样点从一个几何格扩到两个（S=8/S=4）并补了步数对齐对照，但仍不是全量扫描，是否该升成支撑交主 agent |

## 历史版本

### 2026-09-23

- 撤回「独立于 Q7a 的一格干净信号」那一段。原文：**独立于 Q7a 的一格干净信号**（`stage3.out` 的 `name=q7a basis=beta0_k1` 那一行，字段语义修正之后才读得对）：基底上 G27 成立、`I-3.1（已分配统计对得上）` 与 `I-5.2（空闲统计对得上）` 都判绿；坏镜像（Bd(+1)）上 **G27 不成立（判红）而那两条仍判绿**。⇒ G27 在这一格分得出差别、而今天那两条分不出，且 G27 在基底上不误报。这一格不进 Q7a 的聚合（那张表已按 V3 作废），单独可读。
- 撤回依据：重跑登记的设计员现查、主 agent 复核 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 的 `run_q7a_cell`——`corrupted_check_holds = allocated == deferred + referenced`，其中 `(allocated, _free, deferred)` 取自调用方传入的 `corrupted_allocator_row`，不读 `corrupted_pool`；基底那一侧的 `base_check_holds` 读的是内存分配器。G27 在第一段里一次都没读过盘上的记账行。

**（同日，R2 第一段，执行员）** 装置改写：R3（G27 改读镜像）、新增 HK/HK-F0 两族历史、9 个基底、Q7d/Q7e/Q7f、S1(c) 完整重放、锚1/锚2/锚3 锚点。结论：岔路 7（岔路单第 12 行）仍不够判，且这一次的证据面比第一、二段宽得多（7 个可达基底、168 个可达状态，横跨 H0/HR/HK 三族、八种发布种类）——Q7c①在可达状态上从来看不到转色，Q7d-2/Q7d-3 排除了「量法本身看不见 0」这条解释，剩下的只有「可达合法状态第 5 项从来不是 0」。续跑这套自证设计翻不了面，改判据还是认下，交主 agent（见「结果」与本页顶的岔路表）。产物换成 `research/results/e156-alloc-basis-counts-2026-09-23-stage1.out`（324 行），`replay.sh` 已改指向它；旧的 `stage1/2/3.out`（第一、二段）原样保留，不再承重。

### 2026-09-24

**（R2 第二、三段，执行员）** 先做的一步：`crates/` 在 R2 第一段之后被另一条线改动（`mount.rs`、`transaction.rs`、`recovery.rs`、`walk.rs`、`crash.rs` 五个文件的 `git hash-object` 都变了），在今天的代码上重跑 R2 第一段并与昨天的产物逐字节比对——只有 `HK_l2` 基底的 `I-3.1` 判定由红转绿（另有实现员在改 `walk.rs` 的 checker 实现）这一处差异，Q7c 的读数与岔路 7 的判定不变，详见「这一段做了什么」与「结果」。

装置改写：新增 `run_hf_single_cell`（岔路 3）、`run_hh_cell`（岔路 1）两个函数，`raise_rollback_floor_by_reconstructing_reclaim_after_floor_takes_effect`（G7 用公开入口重组）、`snapshot_allocated_generations`/`record_release_generations`（岔路 1 的分配代账本，G12 需要的那 8 字节真实记录里没有，装置自己维护）。**两条岔路都只跑了一个几何格**（S = 8、ρ = 1），不是登记「五、5.2」的全量取样——理由是这一份执行员的时间预算：岔路单第 3 行要「两个口径各一个数 + X8-A 两者各卡在哪」，这一格已经给出前一半（D_rel 相同、差别不大）；岔路单第 1 行要「至少两个洞数不同的取样点」，三个取样点（k = 0、1、2）已经给出、且越过了两个门槛。结论：

- **岔路 3（第 11 行）**：一个几何格上 G7 与 F-扣 的 D_rel 完全相同（都是 0，非空发布计），Q3c 判「差别不大」；K9 没有验到预期的 3（前提「环里 ≥ 4 条非空有效根」没有专门核过，也可能是 P21(e) 已预估的方向不成立）；Q3e（X8-A）完全没做。**第 11 行部分够判**：两个口径各一个数拿到了，X8-A 那一半没有。
- **岔路 1（第 9 行）**：三个洞数取样点上 Δ 单调 +50/洞，从 h = 0 起就越过 2 槽 / 1 槽两个门槛；但 h = 0 时 Δ 已经是 20，这个基线值本身没有查透（只做到聚合数级别，没有逐槽拆解）。**第 9 行的两个够判条件字面满足**（至少两个取样点、报得出越过门槛的规模），但只在一个几何格上、基线机制不明，交主 agent 判是否已经够判到可以交用户，还是要先查清 h=0 的 20 从哪来。

产物换成 `research/results/e156-alloc-basis-counts-2026-09-24-stage2.out`（338 行，累计第一、二、三段），`replay.sh` 已改指向它；2026-09-23 的 `stage1.out`（324 行）原样保留（今天在最新 `crates/` 上重跑一份新的、与它只差 HK_l2 那 3 处，未另存文件，差异写进「这一段做了什么」）。单测由 4 条增到 5 条，变异表由 3 条增到 4 条，另手工验证了一条不会红的变异（G7 的「立刻可发」标记改成「回收但扣住」，见「复跑」），按 `mutation-sampling.md` 判为第一类真盲区，不进 `crates/mutations.tsv`。

**（同日，R2 续派第一段，s4 执行员，接续一个撞了会话限额而中断的执行员）** 开工先核现场：交接摘要 `/tmp/claude-1000/handover/ab85369a718bfa7f4.md` 记的后台任务已经跑完（留在 `/tmp` 的 `probe1.out` 与产物同值，未落盘）。先按派发要求在今天的主工作区上重跑已有产物、与并发补丁核对：`crates/` 的 7 个 S3 跟踪文件被另一条并发会话改过，把 `stage2.out` 里格式没变的 329 行与今天重跑的同名行逐行比对，逐字节相同——并发补丁没有改到 E156 这几条历史触达的代码路径（结论详见「这一段做了什么」第 17 条）。查清 `holes=0` 时 Δ = 20 的来源是装置诊断代码自己的一处过滤漏洞（已修，不改变任何已报的 Δ 值），给岔路 1 补 S = 4 第二个几何取样点，给岔路 3 的 K9 补前提现核。产物换成 `research/results/e156-alloc-basis-counts-2026-09-24-stage3.out`（682 行），`replay.sh` 已改指向它；`stage2.out` 原样保留。单测由 5 条增到 6 条。

**（同日，R2 续派第二段，执行员，主 agent 消息「接着做 E156 这一段，岔路表里还开着两行」）** 给岔路 1 补步数对齐对照（`matched_to_holes` 参数）：k=0 但工作负载步数补到与 holes=k 相同、不造洞，测出 `Δ(0, 步数对齐)` 恒为 20——证明之前报的 20→70→120 曲线，增长部分是洞本身贡献的，不是步数带来的；顺带从 `q1_delta_record` 拆解出 `holes=0` 那 20 槽的构成（两类各 8 条各 10 槽）与机制。给岔路 3 构造出 Q3e（X8-A/HY）：新建 P5 的 128 槽两段小池，两个真实口径（F-扣、新增可失败版 G7 重组）在填到耗尽的 HX 上都卡在同一步（`AccountingTree` 分配不到落点），在提前停手的 HY 上都成功——确认 HX 的失败是耗尽导致；一并做出 Q3d（推导法，见方法限度）。**结论**：岔路 1（第 9 行）与岔路 3（第 11 行）现在都有了比上一段更完整的证据（岔路 1 补了几何点与步数对照，岔路 3 补齐了 Q3a–Q3e 与 Q3d），但都还不是登记「五、5.2」的全量扫描，够不够判交主 agent 看「岔路表」定。产物换成 `research/results/e156-alloc-basis-counts-2026-09-24-stage4.out`（776 行），`replay.sh` 已改指向它；`stage3.out` 原样保留。单测仍是 6 条，变异表仍是 4 条（新增代码没有配套新变异行，理由见「复跑」）。
