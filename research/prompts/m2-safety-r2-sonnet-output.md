# m2-safety-r2 云端正推（Sonnet）——S4：A1–A4 从条款正推、在冻结副本拷贝上实现、量数据

分工：`research/prompts/_m2-safety-r2-body.md` 第五节「云端正推」一行——A1–A4 从条款正推各自的定义，
在冻结副本的拷贝上实现，在 Z19-B 历史与 D3（空间分配） 已定项 9 那两句的历史上量；挂载路径真卸载再挂载量。

工作树：冻结副本 `/tmp/claude-1000/m2-safety-r2/tree/crates/` 整份拷进
`/tmp/claude-1000/m2-safety-r2-sonnet/tree/crates/`（`cp -r`，逐字节相同，只加了根 `Cargo.toml` / `Cargo.lock`
让它能单独编译——两份文件与主仓根目录同名文件逐字节相同，`diff` 见「没做什么」一节）。
以下代码引用的行号，是这份工作树（不是主仓 `crates/`）里改完之后的行号，`grep -n` 现查。

## 一、S4 是什么（正推的出发点，不重复判决，只摘要）

正文第 11 行：小盘上每次覆盖写都被发布准入放行，下一次可写挂载却被挂载准入拒，只有回退丢掉一版才能再写。
第一轮正推腿的归因：两处调同一个函数（`admission::admission_reading_before_a_publish`）——卡住在判的时机加 defer
重复扣：这次发布换下的旧块要到下一次判才看得见，而且「已分配」与「defer 待释放」各扣一次。

**这条归因在今天的实现里逐字可查，不是推测**（`crates/singlefs-core/src/admission.rs`，行号是本腿改动前、也是改动后都没变的那一段）：

`DeviceAdmissionTerms::allocated` 字段的文档注释（第 222-225 行，本腿一个字没改）：

```
式子里的「已分配」（记账第 1 项「已分配字节」）：分配器的占着槽数——仍分配的加上已释放、还在 defer 窗口里的
（I-3.1（已分配统计对得上） 读法甲，2026-09-14 用户定）。
⚠️ 它含 defer，而式子另扣一次「defer 待释放」：按读法甲 defer 扣两次。删不删那一项是增补 2 收口表第 ② 行
alloc-basis 岔路 2，还开着；这里照式子逐字扣，不替它定。
```

`allocator.rs` 第 298 行 `allocated_slots()` 与第 312 行 `mark_released()`：`mark_released` 只把槽计进
`deferred_slots`（`self.deferred_slots += span`），**不**清位图、**不**减 `allocated_slots`——这是故意的，
`allocator.rs` 第 1601 行的既有测试断言「占着的槽数不变」（`releasing_a_placement_keeps_the_slots_occupied_and_moves_them_into_the_defer_queue`）。
所以 `available_on_each_device`（改动前的 `admission.rs`）里 `own_terms = [allocated, unreclaimable, deferred, mount_time_commitment, abandoned_root_exclusive]`
一次扣了两遍同一批 defer 槽——这正是 alloc-basis 岔路 2（附录 `research/prompts/alloc-basis-forks.md` 第 10 行）
「D28（挂载期承诺量） 已定项 1 里的『− defer 待释放』（第一轮双扣）」，**已经在 E156 量过「无运行时代价」、状态是「可以交用户定」**
（附录同一行「状态」列）——这一半不是本腿新发现，是本腿在实现里把它接上、连同 A1 一起量。

「这次发布换下的旧块要到下一次判才看得见」：`transaction.rs` 里 `prepare_the_version_publish`（本腿改动前）的准入调用
在 `settle_the_allocation_record_tree` 算出 `settled.release`（这次发布要释放的落点）**之后**、但 `publish_admitted`
（真正调 `mark_released`）**之前**调用 `admission_reading_before_a_publish(allocator, previous)`——它读的是
`allocator` 此刻的计数，这次自己要释放的那批槽还没被标记，读数里看不到它们。

## 二、A1–A4 的正推定义（写成对 `crates/` 的改动）

改法接线图（新增/改动的函数与它们各自的射程）：

| 函数 | 文件:行 | 干什么 |
|---|---|---|
| `S4Candidate`（枚举，`Baseline`/`A1`/`A2`/`A3`/`A4`） | `admission.rs:623-644` | 候选开关，只供三方论证；产品路径恒 `Baseline` |
| `AdmissionReading::available_on_each_device_for_candidate` | `admission.rs:390-448` | A1/A2 的公式；A3/A4 落这里时按 Baseline 算（它们的差异在调用处） |
| `admit_on_every_device_for_candidate` | `admission.rs:494-541` | A3 的净释放豁免判据 |
| `mount_admission_a4_switch_reserve_only` | `admission.rs:562-582` | A4 专用，只在挂载入口调 |
| `PoolAllocator::s4_candidate` / `set_s4_candidate` | `allocator.rs`（`grep -n s4_candidate allocator.rs`） | 候选装在分配器上，同 `space_admission` 的装法 |
| 发布路径调用点 | `transaction.rs:4560-4595`（`grep -n released_this_publish_slots transaction.rs` 定位） | 算 `settled.release` 的字节、传给 `admit_on_every_device_for_candidate` |
| 挂载路径调用点 | `mount.rs:2089-2120`（`grep -n allocator.set_s4_candidate mount.rs` 定位） | A4 走专用函数，其余走 `admit_on_every_device_for_candidate` |


**逐条候选的定义（正推出来的，不是抄条款原文——条款没有把它们写成算法，这是本腿从条款推出的可执行形式）：**

- **A1**（第一轮候选 B）：`available_on_each_device_for_candidate` 的 `S4Candidate::A1` 分支
  （`admission.rs:420-425`）——`terms.allocated.0 - released_here`，`released_here` 是调用方传入的
  「这次操作自己要释放、还没进 `deferred` 字段的字节」。发布路径（`transaction.rs`）在算准入之前，
  从 `settled.release`（这次发布的释放判定结果，`release_checksum_check` 那一步用的同一份）现算
  `released_this_publish_slots`，同样的字节数填进每一块盘（一个单元两份落两块盘、每块盘同一个跨度）。
  **依据**：D3（空间分配） 已定项 17「释放空间这个操作本身不需要申请空间」——这次发布自己换下的块，
  它的「不需要申请空间」这一半今天没有兑现（要等下一次判才看得见），A1 把它现算进来。
  **只在发布路径生效**：挂载路径需求恒 0，不释放任何东西，`released_here` 传空切片，A1 与 Baseline 在挂载入口相同。

- **A2**（A1 + 删掉「− defer 待释放」重复扣）：`available_on_each_device_for_candidate` 的 `S4Candidate::A2` 分支
  （`admission.rs:427-433`）——`own_terms` 只有四项（不含 `terms.deferred`），因为「已分配」已经把 defer 那批槽
  算在内（前面一节引的 `DeviceAdmissionTerms::allocated` 文档注释）。**这正是 alloc-basis 岔路 2 的「甲-T1 下删掉这一项」**，
  E156 已经量过「无运行时代价」；本腿在同一处实现里把它与 A1 合用（正文第 39 行「A1 加删掉『− defer 待释放』的重复扣」）。

- **A3**（删除类发布豁免）：`admit_on_every_device_for_candidate`（`admission.rs:494-541`，净释放豁免判据在第 513-527 行）里，
  候选是 `A3` 且这次操作有非零需求时，先判「每一块盘上这次自己的释放字节 ≥ 这次自己的需求字节」，
  成立就直接 `Ok(())`、不算可用（`admission.rs` 第 513-527 行一带，`grep -n every_device_is_a_net_release admission.rs` 定位）。
  **依据**：`.claude/rules/fs-design.md:23`「释放空间这个操作本身不需要申请空间」——净释放的发布，
  它要新分配的那部分已经被自己腾出的空间抵掉，不必再过准入；由 checkpoint 保留池兜底它自己的固定点开销
  （已定项 4「它保证 checkpoint 自己的固定点写得出去」，与准入无关的另一条防线）。
  **只在需求非全零时可能触发**：挂载路径需求恒 0，`has_any_demand` 为假，A3 在挂载入口与 Baseline 相同
  （见下面「量出来的数」一节，A3 的挂载仍然会被拒，只是被拒的 k 更大）。

- **A4**（挂载准入只判挂载期承诺量本身）：`mount_admission_a4_switch_reserve_only`（`admission.rs:562-582`），
  只在 `mount.rs` 的 `establish_instance` 里、候选是 `A4` 时调用（`mount.rs` 第 2100-2102 行一带），
  判「容量(d) − 挂载期承诺量(d) ≥ 0」，不算已分配、不可回收、defer 待释放、被抛弃根独占量、也不算三项全池承诺量
  （待删占用、已承诺预留、checkpoint 保留池）。**这是本腿最窄的读法**：正文只写「不因已分配多而拒」，
  没说要不要连三项全池承诺量一起丢；本腿选了最窄的（连 checkpoint 保留池也不判），
  理由写在下面「A4 的另一种读法与为什么没测」。**不经共用的 `available_on_each_device_for_candidate`**：
  这条读法只对「需求恒 0」的挂载安全，若被同一个全局候选开关带进发布路径（需求非零）会放行几乎一切，
  单独成一个函数、只在挂载入口调，避免这个泄漏面（`S4Candidate` 的文档注释 `admission.rs:605-621` 写了这条理由）。


**A4 的另一种读法与为什么没测**：正文只写「不因已分配多而拒」，字面上可以只丢「已分配」一项，
保留不可回收、defer 待释放、被抛弃根独占量与三项全池承诺量。本腿选了最窄的读法（连三项全池承诺量也丢），
理由：（1）「保留池够不够这次挂载自己的写」这句话里的「保留池」在 D28（挂载期承诺量） 已定项 3 的语境下指的就是
挂载期承诺量自己（切换预留），已经把暖机的 c_max 算在内（已定项 3「暖机……(N_switch + 1) × R × c_max」），
不必再重复扣一次式子第八项的 checkpoint 保留池；（2）两种读法哪个更安全没有先验答案，窄读法的失败面更大，
先测最坏情况更符合「反推最容易被跳过」的纪律。**没有测中间读法**（只丢已分配、留其余七项）——这是本腿的欠账，
写进「没做什么」。

## 三、真跑出来的数（Z19-B 历史，`CloseAndMountWritable` 是 `establish_instance` 的真实重跑，不是复用同一个分配器）

**基线（Baseline，未改代码，冻结副本原样）**：跑 `research/prompts/m2-final-code-r4-opus-model/tests/r4_opus_z19.rs` 的
`z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount`（拷进这份工作树、逐字节不改），
命令与全部原样输出落在 `research/prompts/m2-safety-r2-sonnet-model/baseline-z19b.log`。摘要（width=240，selector=2999，
selector=99 的数逐字相同——两个 selector 只是不同的填充选择器，不影响准入判定的结构）：

| k（会话里覆盖写次数） | next_writable_mount | again（再关再挂一次） |
|---|---|---|
| 0-3 | Applied | Applied |
| 4 | Applied | **SpaceAdmissionRefusedBeforeAcquisition** |
| 5（= 这一次会话准入放行的上限） | **SpaceAdmissionRefusedBeforeAcquisition** | SpaceAdmissionRefusedBeforeAcquisition |

width=256 上限 6、width=384 上限 10，同一种形状：**到达会话内准入放行上限之后，下一次可写挂载被拒；
再挂一次也被拒**。24 次逐根回退里多数也被拒（`rollback_members` 里 `SpaceAdmissionRefusedBeforeAcquisition` 占大头），
但**回退 + 冷启动恢复（`ColdStartRecover`）之后能重新拿到可写挂载**（`tail=["Applied","Applied"]`，
`research/prompts/m2-safety-r2-sonnet-model/baseline-z19b.log` 逐行可查）——这条路径本腿没有深挖为什么
`ColdStartRecover` 与普通 `CloseAndMountWritable`结局不同，列进「没做什么」。


**五个候选在同一段历史上的对拍**（width=240/256 见 `research/prompts/m2-safety-r2-sonnet-model/s4-r2-candidates-run1.log`
全文——这是第一次跑，`cargo test` 默认并行两个测试，width=384 那一段中途被别的 agent 自测发的一轮 TERM 打断；
width=384 见重跑的 `research/prompts/m2-safety-r2-sonnet-model/s4-r2-candidates-run2.log`——同一份测试源码、加
`--test-threads 1` 串行重跑，240/256 两段数据与 run1.log 逐字一致（`diff` 已核，唯一差异是 cargo 进度行与第一行
println 输出挤在同一行，不是数据差异），A3 的 k 循环因超过 40 分钟预算被本腿主动停在 k=66（详情见下与「没做什么」）；
A4 见补测 `research/prompts/m2-safety-r2-sonnet-model/s4-r2-width384-a4-run1.log`（独立小测试
`tests/s4_r2_width384_a4_supplement.rs`，只测 width=384、candidate=A4 这一格，跑完整不截断）。
下表的「会话内准入上限」= 一次挂载里连续覆盖写、准入放行的最大次数；「mount 首次拒绝的 k」= 到这个覆盖写次数、
关掉再挂载就会被拒；空表示在测过的 k 范围内一次都没被拒）：

| 候选 | width=240 会话上限 | 240 mount 首拒 k（again / next） | width=256 会话上限 | 256 mount 首拒 k（again / next） | width=384 会话上限 | 384 mount 首拒 k（again / next） |
|---|---|---|---|---|---|---|
| Baseline | 5 | 4 / 5 | 6 | 5 / 6 | 10 | 9 / 10 |
| A1 | 6 | 4 / 5 | 6 | 5 / 6 | 11 | 9 / 10 |
| A2 | 12 | 10 / 11 | 12 | 11 / 12 | 22 | 20 / 21 |
| A3 | 16 | 4 / 5 | 17 | 5 / 6 | 80（探针 0..80 全部放行，`next_step=None`） | 9 / 10（k=10 起到本腿测过的 k=66 全部保持拒绝，未再反转；k=67..80 没跑到，见「没做什么」） |
| A4 | 5 | 一次都没拒（k=0..5 全放行） | 6 | 一次都没拒（k=0..6 全放行） | 10 | 一次都没拒（k=0..10 全放行） |

**读出来的结构**（三个宽度合起来看）：

1. **A1 单独几乎不解决 S4**：会话上限只多 1（5→6），mount 首拒的 k 与 Baseline 逐字相同——A1 只补了
   「这次自己换下的块」，而 defer 双扣（A2 那一半）才是把可用压低的主因。
2. **A2（A1 + 删双扣）把会话上限翻倍以上**（5→12，6→12），**但 mount 首拒的 k 也跟着往后挪**（4→10，5→11）——
   **A2 没有让「挂载还能不能写」这件事本身消失，只是把撞上它的点往后推**：式子本身没变，只是分子（可用）变大了，
   迟早还是会被同一条式子（未改的挂载准入，取 Baseline/A2 时挂载路径的公式与发布路径同一份）判负。
3. **A3 让发布侧的会话上限暴涨（5→16），但 mount 首拒的 k 与 Baseline 分毫不差（4/5）**：
   A3 的净释放豁免只接在发布路径（`admit_on_every_device_for_candidate` 里 `has_any_demand` 要求需求非零），
   挂载准入需求恒 0，从来不触发豁免，挂载入口对 A3 和 Baseline 用的是同一段代码（`available_on_each_device_for_candidate`
   的 `S4Candidate::A3` 分支就是 Baseline 分支，见 `admission.rs:411-419`）——**A3 单独对 S4 描述的症状（挂载被拒）零改善**，
   它改的是另一件事（发布本身更晚才被拒）。
4. **A4 是目前唯一让 mount 首拒完全消失的候选**（在测过的 k 范围内）：因为它把挂载准入与「已分配」彻底断开，
   会话侧还是照 Baseline 的公式在 k=5/6 就把发布拒掉（A4 不改发布路径），但**发布被拒之后，池仍然「可写挂载」**——
   S4 原话「下一次可写挂载却被挂载准入拒」的那一半，在 A4 下不再发生（本腿测到的范围内）。
5. **width=384 让「A3 对 S4 挂载零改善」这条结论更扎实**：会话上限从 16/17（240/256）暴涨到 80（384，探针 0..80
   全部放行），mount 首拒的 k 却仍然钉在 9/10——与 Baseline、A1 逐字相同。差距被拉大到「会话侧能写 80 次，
   但挂载侧第 10 次就已经会被拒」，这不是采样巧合：k=10 到本腿测过的 k=66，57 个数据点全部保持拒绝、
   一次都没有反转回 `Applied`。**A4 在 width=384 上同样是唯一的「一次都没拒」**（k=0..10 全放行，补测数据），
   与 240/256 的形状一致——A4 对 S4 挂载那一半的改善不随宽度变化，A3 的零改善也不随宽度变化，两条结论都在
   第三个取样点上被重复验证，不是只在两个点上碰巧成立。


## 四、D3（空间分配） 已定项 9 第 2 条的历史（删了再写同样大小，能不能在有界步数内成功）

**这条测的窄了，先说清窄在哪**：D3（空间分配） 已定项 9 第 2 条要求「删掉 s 字节之后同样大小的写在**有界步数（3 次
改变用户可见状态的发布）内**成功」，不要求立刻成功。本腿的 `s4_r2_d3_item9_delete_then_write_same_size_across_candidates`
只测了「到达会话内准入上限之后，紧接着再来一次同样大小的覆盖写，当场（0 次界内）成不成功」——**没有测「再等 1-3 次
别的发布之后是不是就成功了」**，那需要在拒绝之后接着推空发布再重试，这一段本腿没做（列进「没做什么」）。
数据能回答的是「窄义：立刻能不能」，回答不了条款要求的「宽义：3 步之内能不能」。

命令与全部原样输出：`research/prompts/m2-safety-r2-sonnet-model/s4-r2-candidates-run1.log`（同一次跑产出两个测试的输出）。

| 候选 | width=240 会话上限（此前已放行的覆盖写次数） | 再来一次同样大小的覆盖写 | width=256 会话上限 | 再来一次 |
|---|---|---|---|---|
| Baseline | 5 | `SpaceAdmissionRefused` | 6 | `SpaceAdmissionRefused` |
| A1 | 6 | `SpaceAdmissionRefused` | 6 | `SpaceAdmissionRefused` |
| A2 | 12 | `SpaceAdmissionRefused` | 12 | `SpaceAdmissionRefused` |
| A3 | 16 | `PlacementRefused(NoFreeSlotOnAnyDevice)` | 17 | `PlacementRefused(NoFreeSlotOnAnyDevice)` |
| A4 | 5 | `SpaceAdmissionRefused` | 6 | `SpaceAdmissionRefused` |

**读出来的结构**：

1. **窄义上，五个候选没有一个当场成功**——D3（空间分配） 已定项 9 第 2 条允许 3 步之内成功，这条数据不能拿来说
   「谁违反了已定项 9」，只能说「谁都没有在 0 步内做到」，这本来就在条款允许的范围里。
2. **A3 把失败面从「准入拒绝」换成了「落点拒绝」**：会话内的这次覆盖写不再被准入公式拦（净释放 ≥ 需求），
   走到了真正取落点那一步，落点取不到才拒——**这更贴近 D3（空间分配） 已定项 9 第 1 条「只有盘真的写满才许报
   ENOSPC」的字面**：`PlacementRefused(NoFreeSlotOnAnyDevice)` 是分配器真的找不到空槽，
   `SpaceAdmissionRefused` 是准入公式算出「负」但槽可能其实还在。**这不构成「A3 满足已定项 9」的结论**——
   落点取不到之后能不能在 3 步内腾出空间，本腿没有继续跟踪（同样的「没测到底」欠账）。
3. **A4 在这条历史上与 Baseline 完全一样**：A4 只改挂载准入，这条历史里的失败发生在覆盖写（发布路径），
   A4 对它没有任何影响——**A4 与「D3 已定项 9 第 2 条」这条症状是正交的**，它解决的是另一个症状（S4 的挂载那一半）。


## 五、每条候选的失败面与「什么现象会推翻它」

| 候选 | 它新开的失败面 | 什么现象会推翻「这条候选解决了它声称解决的那一半」 |
|---|---|---|
| A1 | 无新失败面（只是把已经算出来的释放字节提前一步看见，不改变最终会不会放行——它不改变「稳态」下的可用量，只补一次性的滞后） | 若在某段历史上 A1 反而**放行**了一个 Baseline 会拒的、事后被证明确实超卖的请求（即 A1 把「这次自己换下」算重了，比如同一个落点在 `settled.release` 里被算了两次），量出可用 > 真实剩余空间就推翻 |
| A2 | **可能重新打开 alloc-basis 岔路 1 提到的「环上有洞」那一格**（附录 `alloc-basis-forks.md` 第 9 行）：A2 删掉的是「defer 待释放」那一项，若将来有一种历史让「已分配」统计**不**包含某些该算的 defer 槽（今天的实现里不存在，但这是式子层面的假设，不是代码层面已证的不变量），A2 会少扣一次、放行本不该放的请求 | 造一段历史，让 `allocated_slots()` 与 `deferred_slots()` 出现「有槽只在 deferred 里、不在 allocated 位图里」的状态（今天的 `mark_released` 保证不会，见 `allocator.rs:312-320`），A2 在那一格放行、池级 checker（I-3.1（已分配统计对得上）／I-5.2（空闲统计对得上）） 判红就推翻 |
| A3 | **豁免可能被滥用**：任何「净释放 ≥ 新分配」的发布都跳过准入，包括**恶意或异常构造的发布**（例如反复对同一个已经很大的文件做「删一半、写等量新内容」，每次都精确卡在净释放刚好等于需求）——这与攻方腿分工里点名的「把普通写伪装成删除类发布」是同一类；本腿的净释放豁免判据是**逐盘的字面比较**（released ≥ demand），没有再检查「这次释放的槽，是不是这次发布自己真的能拿到、而不是要等 defer 窗口才能拿到」——**这正是本腿自己没有解决的旧问题**：A3 相信 `released_by_this_operation_on_each_device` 里的数，而这批槽进的是 defer 队列、有窗口，不是立刻可用；A3 跳过准入不等于这些槽立刻空出来给这次发布用，实际空间够不够仍由后面的落点分配步骤兜底（`PlacementRefused` 那条真正的安全网），但准入本该做的「进门前算最坏情况」这一步被跳过了 | 造一个净释放 ≥ 需求、但落点分配阶段也放行（因为分配器还有别的空闲槽，不是靠这次的释放）的历史，量出「这次发布实际让池的物理占用净增长」——若能找到一格 A3 放行、Baseline 会拒、且拒的理由不是「defer 双扣的假象」而是真实空间不够，就推翻 A3「只豁免无害的删除类发布」这句话 |
| A4 | **本腿测到的范围内看不出安全后果**（因为下游的取落点/预演步骤仍在「任何写之前」拦一次，接线条款「拒了在任何写之前返回」对 A4 同样成立），但**没有测到「A4 放行一个挂载、随后暖机 / 写行那几次发布因为真的没有物理空间而被 `PlacementRefusedBeforeAcquisitionMountAdmissionUndecided` 拒绝」这一格**——这正是 A4 把清楚的错误信息（`SpaceAdmissionRefusedBeforeAcquisition`）换成含糊的错误信息（挂载先成功一半、再在预演里失败）的那个代价，附录 C283 与接线条款都没有覆盖这一种新组合 | 造一个池：已分配 + 不可回收 + defer + 被抛弃根独占量已经把盘塞满、只剩下刚好够「挂载期承诺量」那么大的空间，A4 会放行取号，但取号之后暖机与写行的预演（`dry_run_of_the_publishes_after_acquisition`）应该会因为真的没有位置而拒——量到这一格出现（`MountError::WarmUpAdmissionRefusedBeforeAcquisition` 或 `RowPublishAdmissionRefusedBeforeAcquisition`）就证明 A4 只是把拒绝往后挪了一步，不是消除了它；本腿没有构造过这段历史（列进「没做什么」） |


## 六、共用问句逐条答（正文第 17-20 行）

- **「这条合法历史走完之后，池还能不能写、删文件能不能腾出空间再写」，答案要量，不许推**：
  第三、四节的数已经回答——Baseline / A1 / A3 在测过的历史上都会走到「不能写」（挂载被拒或落点被拒）；
  A2 把能写的窗口显著拉长但没有消除「最终会撞上」这一格；A4 让「挂载」这一半的「不能写」在测过的范围内消失，
  但没有测「删文件」这一半（本腿的历史都是覆盖写，没有真正的 unlink——第一版分配器 `PENDING_DELETE_OF_THE_FIRST_VERSION`
  恒 0，`admission.rs:142-143`，这是「没有删除路径」这句话在代码里的样子，所以「删文件腾出空间」这条问句在
  第一版实现上**没有对象可测**，列进「没做什么」）。
- **不许用『拒绝』换『安全』而把池永远卡在只读，除非说得出用户手里还有哪一步能让它重新可写**：
  Baseline 的答案是「回退 + 冷启动恢复」（第三节，`tail=["Applied","Applied"]`）；A2 的答案是「多等几次覆盖写窗口
  更宽，但同一步骤仍然管用」；A4 让这一步在测过的范围内不需要——但 A4 没有回答「真的没有物理空间时，用户手里还有
  哪一步」，因为它把这一问推到了更下游（预演/落点拒绝），那一层原有的答案（回退）本腿没有重新验证在 A4 下是否照样管用。
- **`.claude/rules/fs-design.md`「释放空间这个操作本身不需要申请空间」与 D3（空间分配） 已定项 9 逐句核**：
  见第一、二节——这句话与已定项 9 第 2 条正是 A3 的依据来源；本腿核过 `fs-design.md:23` 与
  `.claude/kb/decisions/03-空间分配.md:180-197`（kb 快照行号）两处原文逐字与正文/附录引用一致（`grep -nF` 已核，见第一节）。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| A1 的定义能不能从条款正推出来 | 一致 | 读数额外加回这次操作自己的释放，接线在 `admission.rs:420-425`；只对发布路径生效 |
| A2 的定义（= A1 + 删双扣）能不能从条款正推出来 | 一致 | 与 alloc-basis 岔路 2「甲-T1 下删掉这一项」同一处，E156 已量「无运行时代价」；本腿在 `admission.rs:427-433` 实现并与 A1 合用 |
| A3 的定义能不能从条款正推出来 | 一致 | `.claude/rules/fs-design.md:23` 与 D3（空间分配） 已定项 9 支持「净释放不必申请空间」这条豁免；本腿把它做成逐盘净释放比较（`admission.rs:513-527`） |
| A4 的定义能不能从条款正推出来 | 冲突（本腿的窄读法 vs 正文可能允许的宽读法） | 正文「不因已分配多而拒」没写清要不要连三项全池承诺量一起丢；本腿测了最窄的（`admission.rs:562-582`），中间读法没测，见「没做什么」 |
| A1 对 S4（挂载被拒）的效果 | 一致（数已量） | 会话上限只多 1，mount 首拒的 k 与 Baseline 相同——单独几乎不解决 S4 |
| A2 对 S4 的效果 | 一致（数已量） | 会话上限翻倍以上，但 mount 首拒的 k 同比例后移——延后，不消除 |
| A3 对 S4 的效果 | 一致（数已量，含 width=384） | mount 首拒的 k 与 Baseline 在三个宽度上逐字相同（4/5、5/6、9/10）——对 S4 挂载那一半零改善；width=384 上会话上限已涨到 80，首拒仍钉在 9/10，缺口反而更明显 |
| A4 对 S4 的效果 | 一致（数已量，含 width=384，范围有限） | 三个宽度（240/256/384）测过的 k 范围内 mount 首拒从未出现——目前唯一让 S4 的挂载那一半消失的候选，但真正的物理耗尽场景没测（见第五节的推翻条件） |
| 四候选对 D3（空间分配） 已定项 9 第 2 条（窄义：立刻成功） | 一致（数已量） | 五个候选在测过的历史上都不能当场成功；A3 把失败面换成落点拒绝而不是准入拒绝，其余不变 |
| D3（空间分配） 已定项 9 第 2 条（宽义：3 步之内） | 规则没说（本腿没测） | 需要在拒绝之后接着推空发布再重试，本腿受时间预算没做 |
| 挂载路径是不是真卸载再挂载量的 | 一致 | `CloseAndMountWritable` 走 `mount_writable_with_space_admission_and_s4_candidate` → `establish_instance` 完整重跑（`rebuilt_allocator` 从盘上重建），不是复用同一个 `PoolAllocator`；`history.rs` 的 `apply_mount_writable` 逐行可查 |


## 八、没做什么

- **接手记录**：本腿由前一个会话中断后接手（原因是同一台机器上别的 agent 自测发的一轮 TERM，不是这条腿自己的问题）。
  接手时核过现场：报告已写到第 248 行、模型目录已有 diff/tests/baseline 日志，但候选对拍的后台任务
  （`s4-r2-candidates-run1.log`）在 width=384、candidate=a3 的 k 循环中途被同一轮 TERM 打断（k 停在 58，
  没有 `test result` 收尾行），且这份日志当时从未被拷进模型目录——报告引用的路径在仓里其实不存在，
  是本腿在核现场时发现并修的一处证据缺口。按「不重做已落盘的、接着做完」的原则，本腿：
  ① 把 run1.log 原样拷进模型目录留档（240/256 的完整数据与 D3I9 全部数据都在这份里，没有失真）；
  ② 用同一份测试源码加 `--test-threads 1` 重新串行跑一遍（`s4-r2-candidates-run2.log`），240/256 与 run1.log
  逐字一致（`diff` 已核）；③ width=384 的 A3 因 k 循环耗时随 k 增长（checker 成本随历史变长），本腿看着
  实测速率推算跑满 0..=80 会超过 40 分钟的时长预算，在 k=66（57 个数据点全部保持拒绝、无一次反转）时用
  `proc.py stop` 主动停止，这是「超过 40 分钟先缩取样并在报告里写明」的落实；④ A3 停跑之后 A4 的 width=384
  数据没有着落，另写一份独立小测试 `tests/s4_r2_width384_a4_supplement.rs`（只测这一格，26 秒内跑完整，
  不受 a3 那条大循环拖累）补上。本腿还在核现场时顺手核对了报告里的代码行号引用，改正了三处：
  `admission.rs:562-580`→实测是 562-582；`admission.rs:605-620`→实测是 605-621；
  `alloc-basis-forks.md` 第 8/7 行→实测「甲-T1 下删掉这一项」在第 10 行、岔路 1 的行在第 9 行
  （连带同步改了 `admission.rs` 代码注释与它的 diff patch，`grep -nF` 逐条现查见本节与第二节正文）。
- **width=384 的 A3 没有跑满整个 k 循环**：探针显示 0..80 全部放行（`overwrites_admitted_in_one_session=80`），
  循环设计是 0..=80（81 次），本腿测到 k=66 就因超时间预算停止——k=67..80 这 14 个点没有观测，
  不能排除这段区间出现某种反转（例如某个更大的 k 反而重新被放行）。**什么现象会推翻「mount 首拒钉在
  9/10、之后再也不会变」这条结论**：把 `s4_r2_width384_a4_supplement.rs` 的写法照抄一份、只把 candidate
  换成 A3、k 的范围改成 67..=80，若跑出任何一个 k 让 `next_writable_mount` 或 `again` 变回 `Applied`，
  就推翻。
- **A4 的中间读法没测**：只丢「已分配」一项、留其余七项（不可回收、defer 待释放、被抛弃根独占量、三项全池承诺量），
  见第二节「A4 的另一种读法」。只测了最窄的读法。
- **D3（空间分配） 已定项 9 第 2 条的「3 步之内」没测到底**：只测了「立刻（0 步）成不成功」，没有在拒绝之后接着推
  空发布、数到第几次改变用户可见状态的发布才成功——那需要另造一段「拒绝之后手动推 N 次别的发布再重试」的历史，
  时间预算里没做完，是本腿最大的一块欠账。
- **「删文件腾出空间」这条问句在第一版实现上没有对象可测**：`PENDING_DELETE_OF_THE_FIRST_VERSION` 恒 0
  （`admission.rs:142-143`），第一版没有真正的 unlink / 删除路径，本腿的历史全部是覆盖写（COW 换掉同一个逻辑位置），
  不是删除。A3 的「净释放」在覆盖写场景下成立是因为 COW 释放旧版本、分配新版本，数量上接近相等——这**不等于**
  A3 在真正的「删除然后写别的东西」场景下也这样，那条历史本腿没造（列在这里，供攻方腿或下一轮接着造）。
- **回退与冷启动那一段没有跨候选对拍**：Z19-B 原版测了 24 次逐根回退 + 冷启动恢复，本腿在 S4-R2-B 里去掉了这一段
  （只留两次 `CloseAndMountWritable`）以控制跑的时长；Baseline 上「回退 + 冷启动能不能重新拿到可写挂载」这条现象
  （第三节）在 A1/A2/A3/A4 下是否依然成立，没有测。
- **A3 的「豁免被滥用」与「删除类发布自己把保留池吃光」这两条攻击面**只在第五节写了「什么现象会推翻」，
  没有实际去造历史验证——那是分工表里云端攻方（Opus）的分工，正文第 49 行点名由它攻，本腿不重复做。
- **另造『行数逼近 369 的整数倍』『同一会话里回退过、又有根被抛弃』两类历史**（正文第 42 行要求）：**没有做**。
  时间预算优先给了 Z19-B 与 D3（空间分配） 已定项 9 这两段点名的历史，这两类结构性分叉的历史本腿没有构造，
  是本腿在正文要求范围内最大的缺口，交给核查员或下一轮补。
- **本地攻方的算术事实表**（正文第 50 行）不归这条腿，没有做。
- **判决与岔路取舍**不归这条腿，只交数据，不采纳、不出判决。
- **没有跑全量测试、没有跑 `gate.sh`**：按定义与 `.claude/agent-common.md`（不是 `.claude/agents/agent-common.md`，
  实测这个路径不存在，见「没做什么」本条）的「不做」一节，只跑了自己动到的
  `singlefs-core --lib`、`singlefs-harness --test s4_r2_candidates` / `second_transaction_supplement_two_admission_formula`
  / `s4_r2_width384_a4_supplement` 这几个目标，没有跑 `--workspace`（被 `heavy-test-guard.sh` 拒绝过一次，见下）。
- **`--workspace` 一次被门禁拒绝**：`cargo test --workspace --no-run` 被 `heavy-test-guard.sh` 当场拒绝
  （「子 agent 一律不跑全量测试」），改用逐 crate `cargo build -p <crate> --tests` 核对编译，没有绕过。

