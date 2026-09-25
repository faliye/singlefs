# m2-safety-r1 云端正推腿（Sonnet）输出：S4、S5

材料：`research/prompts/_m2-safety-r1-background.md`；代码冻结副本 `/tmp/claude-1000/m2-safety-r1/tree/crates/`；kb 冻结副本 `/tmp/claude-1000/m2-safety-r1/kb-snapshot/`。
以下代码行号一律指冻结副本自己的行号（现查过，不从背景材料数）；kb 行号指快照文件自己的行号。
本报告分段追加写；每段末尾写明这一段做了什么。

---

## S4：发布准入 vs 挂载准入，D28 已定项 1 九项逐项对齐

### 0. 结论先说

**实测结论与任务前提不同**：直接在冻结副本上加打点实测（见下）证明，
`transaction::prepare_the_version_publish`（`crates/singlefs-core/src/transaction.rs:4572`）
与 `mount::establish_instance`（`crates/singlefs-core/src/mount.rs:1866`）调用的是**同一个函数**
`admission::admission_reading_before_a_publish`（`crates/singlefs-core/src/admission.rs:581`），
对同一块盘、同一个磁盘状态，算出来的九项与 `available(d)` **逐字节相同**——两条式子本身没有一项口径不一样。
Z19-B 里「这一次会话每次覆盖写都被放行、下一次可写挂载被拒」的真正原因是**时间点**，不是**公式**：
每一次准入检查读到的是「这次操作发生之前」的分配器状态；一次覆盖写自己换下的旧物理块进了 defer 队列之后，
既算进「已分配」又被「defer 待释放」再扣一次（这是 `admission.rs:224` 自己写明的已知开账，C375／alloc-basis 岔路 2，还开着）。
这次覆盖写被放行时看不到自己制造的这笔双重扣减；**下一次检查**——不管它是同一会话里的第 6 次覆盖写，
还是关闭会话之后的下一次可写挂载——才会看到这笔双重扣减，从而翻脸拒绝。第一节详列九项各自的读法与出处，
第二节给实测数据，第三节给候选修法与在 Z19-B 历史上的实测量，第四节回答「发布准入与挂载准入结论相反」的另两个问句。

### 1. D28 已定项 1 九项逐项：发布准入取什么、挂载准入取什么

公式原文（`.claude/kb/decisions/28-挂载期承诺量.md:19`，kb 快照自己的行号，整行抄）：

```
可用(d) = 容量(d) − 已分配(d) − 不可回收(d) − defer 待释放(d) − 挂载期承诺量(d) − 被抛弃根独占量(d) − 待删占用 ÷ 副本数 − 已承诺预留 ÷ 副本数 − checkpoint 保留池 ÷ 副本数
```

九项在代码里都由同一个函数 `AdmissionReading::of_allocator`（`admission.rs:312`）与
`admission_reading_before_a_publish`（`admission.rs:581`）产出，两个调用点（`transaction.rs:4572`、`mount.rs:1866`）
传的参数形状相同（`allocator: &PoolAllocator`、`version_to_build_on: Option<&TransactionOutput>`）：

| 项 | 发布路径怎么读（`prepare_the_version_publish`） | 挂载路径怎么读（`establish_instance`） | 是否同一读法 |
|---|---|---|---|
| 容量 | `device_map.unit_area_slots()`（`admission.rs:322`），几何常量，挂载期间不变 | 同一函数、同一份 `allocator` | 相同 |
| 已分配 | `device_map.allocated_slots()`（`admission.rs:323`） | 同一函数 | 相同（但读的是各自那一刻的 `allocator`，见第 2 节） |
| 不可回收 | 常量 `UNRECLAIMABLE_ON_A_DEVICE_OUTSIDE_ZONED = 0`（`admission.rs:138`） | 同一常量 | 相同 |
| defer 待释放 | `device_map.deferred_slots()`（`admission.rs:325`） | 同一函数 | 相同读法，不同时点（见第 2 节） |
| 挂载期承诺量（切换预留部分） | `instance_switch_reserve_on_one_device(allocator.instance_rows_after_this_mounts_row_publish(), checkpoint_cost)`（`admission.rs:589-592`）；`rows0` 是**这次挂载开始时算好、挂载期间不变的常量**（`mount.rs:1851-1853` 的 `record_instance_rows_after_this_mounts_row_publish`） | 同一函数、同一公式，但 `rows0` 在**这一次** `establish_instance` 里刚刚被**重新算过**（`mount.rs:1841` `instance_rows_to_write`→`mount.rs:1851` 写回），已经计入这次要新写的行 | 公式相同，输入的时点不同：发布路径读的是「本次挂载开始时」的行数，挂载路径读的是「下一次挂载开始时」的行数（多了本次要写的新行）|
| 被抛弃根独占量 | `device_map.isolated_slots()`（`admission.rs:327`），这个值只由 `isolate_slots_referenced_only_by_abandoned_roots`（`mount.rs:483`）写入 | 同一读法，但 `isolate_slots_referenced_only_by_abandoned_roots` **只在 `rebuilt_allocator`（`mount.rs:614-728`，调用点在 `mount.rs:674`）里调用**，也就是只有挂载/恢复路径会调用它；`grep -n "isolate\|abandoned" crates/singlefs-core/src/transaction.rs` 零命中，发布路径全程不重算这一项 | **口径不同**：发布路径整个会话期间用的是本次挂载开始时算好的旧值，挂载路径每次都用当前根环状态现算一遍 |
| 待删占用 | 常量 `PENDING_DELETE_OF_THE_FIRST_VERSION = 0`（`admission.rs:142`） | 同一常量 | 相同 |
| 已承诺预留 | 常量 `COMMITTED_RESERVATION_OF_THE_FIRST_VERSION = 0`（`admission.rs:147`） | 同一常量 | 相同 |
| checkpoint 保留池 | `checkpoint_reserve_pool(checkpoint_cost_of_the_version_to_build_on(previous, allocator), replicas)`（`admission.rs:593-596`），`previous` 是这次发布要接的上一版 | 同一函数，`version_to_build_on` 传的是 `start.previous.file_version()`——上一次挂载收尾时的那一版 | 公式相同，输入（上一版是谁）随各自的调用时点走，跟着状态推进，不是两套口径 |

`grep -n "fn checkpoint_cost_of_the_version_to_build_on\|fn instance_switch_reserve_on_one_device\|fn admission_reading_before_a_publish" crates/singlefs-core/src/admission.rs` 命中
`admission.rs:487`、`admission.rs:185`、`admission.rs:581`——三个函数只有一份实现，两个调用点共享，不存在「两条式子」。

### 2. 实测：同一块盘在两处读到的九项是不是真的一样

草稿目录：`/tmp/claude-1000/m2-safety-r1-sonnet/tree/`（冻结副本的可写拷贝）。在 `admission_reading_before_a_publish`
返回之前加一段 `SONNET_S4_TRACE` 环境变量门控的 `eprintln!`（不改变行为，只在设了这个变量时多打印），
在两个调用点各加一行 `S4TRACE ==PUBLISH_CHECK==` / `S4TRACE ==MOUNT_CHECK==` 做区分（写法见
`/tmp/claude-1000/m2-safety-r1-sonnet/tree/crates/singlefs-core/src/admission.rs:585-614`、
`mount.rs:1857`、`transaction.rs:4571`）。跑一段与 Z19-B 同形的历史（240 槽小盘，`AfterFirstFile` 起，
`CloseAndMountWritable` 一次、`InsideOneDataUnit` 覆盖写 5 次），命令：

```
cd /tmp/claude-1000/m2-safety-r1-sonnet/tree
CARGO_TARGET_DIR=/tmp/claude-1000/m2-safety-r1-sonnet/target \
  cargo test -q -p singlefs-harness --test s4_trace_probe -- --nocapture
```

第 5 次覆盖写（最后一次被放行的写，`PUBLISH_CHECK`）与紧接着的下一次挂载（`MOUNT_CHECK`）原样输出：

```
S4TRACE ==PUBLISH_CHECK==
S4TRACE rows0=1 checkpoint_cost=MetadataBlocks(5)
S4TRACE device=DeviceIdentity(0) capacity=BytesOnOneDevice(3932160) allocated=BytesOnOneDevice(1327104) deferred=BytesOnOneDevice(1064960) mount_time_commitment=BytesOnOneDevice(1114112) abandoned_root_exclusive=BytesOnOneDevice(0)
S4TRACE pool_wide=PoolWideCommitments { pending_delete: BytesSummedOverAllReplicas(0), committed_reservation: BytesSummedOverAllReplicas(0), checkpoint_reserve_pool: BytesSummedOverAllReplicas(163840) }
S4TRACE available device=DeviceIdentity(0) value=AvailableBytesOnOneDevice(344064)
S4TRACE ==MOUNT_CHECK==
S4TRACE rows0=2 checkpoint_cost=MetadataBlocks(5)
S4TRACE device=DeviceIdentity(0) capacity=BytesOnOneDevice(3932160) allocated=BytesOnOneDevice(1556480) deferred=BytesOnOneDevice(1294336) mount_time_commitment=BytesOnOneDevice(1114112) abandoned_root_exclusive=BytesOnOneDevice(0)
S4TRACE pool_wide=PoolWideCommitments { pending_delete: BytesSummedOverAllReplicas(0), committed_reservation: BytesSummedOverAllReplicas(0), checkpoint_reserve_pool: BytesSummedOverAllReplicas(163840) }
S4TRACE available device=DeviceIdentity(0) value=AvailableBytesOnOneDevice(-114688)
```

`checkpoint_cost`（5）、`mount_time_commitment`（1114112）、`abandoned_root_exclusive`（0）、`pool_wide` 三项**逐字节相同**——
这四项在这一段历史上没有分叉。真正变化的只有 `allocated`（1327104→1556480，+229376）与
`deferred`（1064960→1294336，同样 +229376）：第 5 次写自己换下的旧物理块进了 defer 队列，
这批字节**同时**被计入「已分配」（读法甲：已分配含未回收的已释放块，`admission.rs:220-221`）
**又**被「defer 待释放」再扣一次（`admission.rs:224` 原话，整行抄）：

```
    /// ⚠️ 它含 defer，而式子另扣一次「defer 待释放」：按读法甲 defer 扣两次。删不删那一项是增补 2 收口表第 ② 行
```

两次扣减合计 458752 字节，与 `available` 从 344064 掉到 −114688 的差值（458752）完全对上。

**这不是「挂载准入」特有的效应**：把上面历史的最后一步从「关闭再挂载」换成「再做第 6 次覆盖写」（同一会话内，不关闭），
`PUBLISH_CHECK` 读到的九项与上面 `MOUNT_CHECK` 逐字节相同（`allocated=1556480`、`deferred=1294336`、
`mount_time_commitment=1114112`、`available=-114688`），第 6 次写同样被拒（`PublishError::SpaceAdmissionRefused`）。
即：**下一次检查，不管形态是「同会话内再写一次」还是「关闭会话后再挂载」，看到的都是同一个已经越界的 `available`。**
两处调用点没有一项公式不一样；「结论相反」的真正原因是**准入检查发生在这次操作的释放效果落盘之前**，
而释放效果一旦落盘就会被 defer 双扣吃掉，被哪一种「下一次检查」先撞见是偶然的。

### 3. 候选修法（都不动盘上格式；候选 A 在 Z19-B 历史上量过，候选 B 未测量）

**候选 A：去掉 defer 双扣**（与攻方 `m2-final-code-r4-opus-model` 里 `R4_OPUS_ADMISSION=no-defer-term`
臂同一处、同一个改法——alloc-basis 岔路单第 2 行「删掉 − defer 待释放」；这里不是抄它的旧数，是在**今天这份**
`/tmp/claude-1000/m2-safety-r1/tree/`（含实二五、实二六）上重新打了同一个开关、重新跑了 Z19-B）：

- 改 `admission.rs` 的 `available_on_each_device`（原文件 `admission.rs:366-373` 那段 `own_terms` 数组），
  `deferred` 那一项按环境变量置零（改法入库在 `research/prompts/m2-safety-r1-sonnet-model/s4/admission.rs.patch` 第一段 hunk，
  `SONNET_S4_NO_DEFER_TERM`）。
- **量了什么**：`cd /tmp/claude-1000/m2-safety-r1-sonnet/tree && CARGO_TARGET_DIR=.../target SONNET_S4_NO_DEFER_TERM=1
  cargo test -q -p singlefs-harness --test s4_z19b_rerun -- --nocapture --exact z19_b_every_admitted_session_leaves_the_pool_writable_at_the_next_mount`
  （完整拷贝 `research/prompts/m2-final-code-r4-opus-model/tests/r4_opus_z19.rs`，一字不改）。
- **结果**（今天这份 crates/ 上重新量到）：240 槽小盘一次会话放行的覆盖写次数从 5 次涨到 11 次，256 槽从 6 涨到 11，
  384 槽从 10 涨到 21（原样输出见下方「模型与产物」一节）；**但同一个「放行到头、下一次检查被拒」的形态原样复现**——
  240 槽在 k=11 时 `next_writable_mount=MountError::SpaceAdmissionRefusedBeforeAcquisition`，
  256 槽在 k=11 时 `again=MountError::SpaceAdmissionRefusedBeforeAcquisition`（第二次重挂才撞上，
  第一次 `next_writable_mount=Applied`）。**池能不能写**：候选 A 只是把撞墙的 k 往后推，
  没有改变「撞墙之后只有回退能重新可写」这个结局（`rollback_members` 里仍然是
  `{"Applied": N, "RollbackTargetNotACandidate(OnAbandonedTimeline)": M, "SpaceAdmissionRefusedBeforeAcquisition": …}`
  这种混合分布，需要回退到足够旧的根才能重新可写，不是每一次回退都能成功）。**回退能不能重试**：候选 A 不碰回退路径，不变。**384 槽在更深的 k 上（k=20、21）还看到一种更差的形态**：
  `rollbacks_applied=0/24`，24 次回退尝试**全部**被 `MountError::SpaceAdmissionRefusedBeforeAcquisition` 拒绝
  （原样输出：`Z19-B width=384 selector=2999 k=21 session_all_admitted=true next_writable_mount=MountError::SpaceAdmissionRefusedBeforeAcquisition again=MountError::SpaceAdmissionRefusedBeforeAcquisition rollbacks_applied=0/24 rollback_members={"MountError::SpaceAdmissionRefusedBeforeAcquisition": 24} tail=["Applied", "MountError::SpaceAdmissionRefusedBeforeAcquisition"] ending=Completed checker_runs=23`）——
  同一段历史里 k 更小时回退从来不会全军覆没（总有几次 `Applied`）。这说明候选 A 不只是把撞墙点往后推，
  **在撞得更晚的那些格子上，回退本身也可能被同一条准入检查拦住，比原来更彻底地卡死**——
  候选 A 不能笼统算作「改善」，需要连回退路径一起量。
- **新开的失败面**：这是纯粹的减项（去掉一次重复扣减），不新增判据分支，没有新的拒绝路径；
  唯一的代价是 kb 里凡是钉死了「小盘上第几次写起被拒」这类绝对值的条目（转述：`checks-owed.md` C283 那一行的「代价」列记着「2026-09-25 准入接进产品路径之后，
  384 槽的小盘…第 11 次覆盖写起、256 槽的小盘第 5 次起一直被拒」这类具体阈值，
  `.claude/kb/checks-owed.md:248`）都要跟着重新量、重新写。

**候选 B：准入检查改成「计入本次操作自己即将产生的释放」再判**（未在本轮实现，未测量，只是设计候选）：

- 改法：`prepare_the_version_publish` 在算出 `settled.release`（这次发布要换下的旧物理块清单）之后、
  判准入之前，把这批块算成「即将转入 defer」的量，一并计入这次检查的 `available`，而不是等到下一次检查才看到它们。
  这样第 5 次覆盖写自己就会因为「写完之后 available 会变负」被拒，而不是被放行、留到下一次检查才发现。
- **在 Z19-B 历史上会怎样（推的，没有测）**：会让「一次会话里能放行的覆盖写次数」再少 1（因为原来最后一次放行的写
  正是让 `available` 越界的那一下），但**换来的是**：越界发生在「写」这一步、报 `PublishError::SpaceAdmissionRefused`，
  池全程保持可挂载——不会出现「会话关闭时一切正常、下一次挂载却发现回不去」的落差，用户也不需要回退就能继续写别的、
  更小的对象（前提是分配器还没被这次将被拒的写实际动过，`crates/` 今天本来就保证「拒了在任何写之前返回」，
  `transaction.rs:4557` 附近注释）。
- **什么现象会推翻候选 B**：如果在 Z19-B 同一段历史上量出「计入自己释放之后判准入」反而让**更早的**、
  本该放行的写也被拒（比如某次覆盖写换下的旧块恰好被同一次发布里别的分配复用，两边重复计费），
  候选 B 就不成立，得先把「这次发布换下的块」与「这次发布新分配的块」分开算，不能简单相加。
- **新开的失败面**：候选 B 没有实现，risk 在于要新增一条「预估这次发布自己的释放量」的计算路径，
  这条路径本身要不要重复 `settled.release` 已经做过的推导、算错了会不会让准入变得**过严**（把不该拒的写也拒了），
  这一条本轮没有观测，交下一轮量。

### 4. S4「另要回答」的两问

1. **发布准入与挂载准入对同一块盘为什么结论相反**：见第 2 节实测——**不是两条式子哪一项口径不一样**，
   两处调用的是同一个函数、同一份公式，九项里七项（容量、已分配的读法、不可回收、defer 的读法、
   checkpoint 保留池、待删占用、已承诺预留）逐字节一致；`checkpoint_cost`（挂载期承诺量与保留池的共同输入）
   在这一段历史上也逐字节一致。差的是**检查发生的时间点**：每次检查都读「这次操作之前」的状态，
   而一次覆盖写自己换下的旧块要等到**下一次**检查才会被看见（并且被 `admission.rs:224` 记录在案的 defer 双扣放大）。
2. **改哪一边、改成什么**：因为两处共享同一个函数，不存在「把一边改成另一边」这件事；
   要改就是改这个共享函数本身——候选 A（去掉双扣，第 3 节）或候选 B（预先计入本次操作自己的释放，第 3 节，未测量）。

第 1 节里另外指出了两处**结构上确实不同**、但在这一段 Z19-B 历史上没有驱动数值差异的口径缺口（供下一轮排查用，
不建议现在就并进这一轮改法，因为没有观测支撑）：

- 挂载期承诺量的 `rows0`：发布路径读的是**这次挂载开始时**冻结的常量，挂载路径读的是**下一次挂载开始时**重新算出、
  已经计入新行的数（`mount.rs:1841`、`:1851`）。这一次历史里两者换算出的页数相同（`⌈(1+3)/369⌉ = ⌈(2+3)/369⌉ = 1`），
  没有露头；一旦行数逼近 369 的整数倍边界，两处会给出不同的 `mount_time_commitment`，值得单独立一笔账排查。
- 被抛弃根独占量：只有 `rebuilt_allocator`（挂载/恢复路径）会调用 `isolate_slots_referenced_only_by_abandoned_roots`
  （`mount.rs:483`、调用点 `mount.rs:674`），发布路径全程不重算；这一次历史没有发生过回退，两处都读到 0，
  没有露头。一旦同一会话里发生过回退、又有根被抛弃，发布路径会继续沿用挂载时算好的旧值，挂载路径会重新现算，
  这里就是一个真实的、按代码结构就能推出来的口径分叉，但今天没有实测数据。

---

## S5：E158 第十段 S4 几何上 239 格「成功但违例」变「挂载报错」——报的是哪个成员、该不该拒

### 0. 出处与结论先说

`research/prompts/e158-s10-runner-report.md:124-125`（整行抄）：

```
从 1355 变成 0，`cold_recover_with_fault_failed`/`mount_writable_with_fault_failed` 还从 0 变成
239——同一批构造里，239 个从「成功但违例」变成「调用直接报错」，失效形态本身变了。
```

**结论**：在冻结副本上重新造出这批格子里的一个子集（34 格，取样条件见第 1 节），全部 34 格报的是同一个成员
`MountError::Recovery(RecoveryFailure::NoValidRoot)`。对着 D23（journal 的角色与格式） 已定项 14「回退见证」
逐行核代码，`choose_root`（`crates/singlefs-core/src/recovery.rs:669-692`）确实按「跳过被见证抛弃的根」实现，
这 34 格全部落在「跳过之后一条候选根都不剩」这一支——**该拒，而且这正是回退见证要做的事**：
把「静默选中一条被回退抛弃的根、之后被 checker 判违例」变成「挂载这一步就直接报错」。

### 1. 怎么造出来的、报的是哪个成员

`research/results/e158-root-choice-repair-2026-09-25-q3-1-s4.out:57`（整行抄，这是登记跑产出的原文件，
不是本轮重新跑的）：

```
E7RESULT name=fault_set_violation_summary geometry=S4(S=4,ring=3MiB) overwrites_before_sigma=[1, 2, 3] sigma_length_limit=4 nodes_visited=14252 pairs=1116897 pairs_with_any_violation=0 cold_recover_with_fault_failed=239 mount_writable_with_fault_failed=239 skipped_by_the_reduction_rule=932402
```

驱动这批产物的二进制是 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`，`mode == "q3-1-s4"` 那一支
（原文件 `:5129-5131`）调用 `run_fault_set_violation_family(&GEOMETRY_SMALLER_ROOT_RING, &initial_overwrite_counts, 4)`，
`initial_overwrite_counts = [1, 2, 3]`。这个函数对每一个模拟出来的历史节点（`SimNode`，带 `rollback_events`
字段，`:345-350`）、每一个 ≤ `sigma_length_limit` 个根环槽的故障集合，都做一次 `mount_writable`
（`evaluate_violation_checkpoints`，`:1217-1286`）；`mount_writable` 报错时该函数原样丢弃错误、只记一个布尔
（原文件 `:1270` `Err(_error) => (None, true)`）——**这就是为什么 e158 的登记产物里看不出报的是哪个成员**，
只能自己造。

**本轮在冻结副本上做的事**（草稿 `/tmp/claude-1000/m2-safety-r1-sonnet/tree-s5/`，与主线程 S4 用的
`/tmp/claude-1000/m2-safety-r1-sonnet/tree/` 是分开的可写拷贝，避免两边并发编译同一个 target）：

1. 把 `Err(_error) => (None, true)` 改成 `Err(error) => { 在 SONNET_S5_TRACE 环境变量下打印 error 的 Debug；(None, true) }`
   （不改变行为，只加打印；见 `tree-s5/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 同一处）。
2. 加一个更浅的模式 `q3-1-s4-timing-probe-4`：`run_fault_set_violation_family(&GEOMETRY_SMALLER_ROOT_RING, &[1], 4)`——
   与登记跑同一个 `sigma_length_limit=4`、同一个 `GEOMETRY_SMALLER_ROOT_RING`，只是把 `overwrites_before_sigma`
   从 `[1,2,3]` 收窄成 `[1]`，把「造几格」缩到几十秒内能跑完，不是登记跑的 1,116,897 对全量。
3. 跑（release，线程上限 8）：

```
cd /tmp/claude-1000/m2-safety-r1-sonnet/tree-s5
CARGO_TARGET_DIR=/tmp/claude-1000/m2-safety-r1-sonnet/target-s5 SONNET_S5_TRACE=1 \
  cargo run -q -p singlefs-harness --bin e158_root_choice_repair --release -- q3-1-s4-timing-probe-4
```

原样输出（汇总行 + 全部 34 行 trace，用 `sort | uniq -c` 数重复，不是手数）：

```
E7RESULT name=fault_set_violation_summary geometry=S4(S=4,ring=3MiB) overwrites_before_sigma=[1] sigma_length_limit=4 nodes_visited=4203 pairs=327727 pairs_with_any_violation=0 cold_recover_with_fault_failed=34 mount_writable_with_fault_failed=34 skipped_by_the_reduction_rule=273463
     34 S5TRACE mount_writable_with_fault error=Recovery(NoValidRoot)
```

`overwrites_before_sigma=[1]` 是收窄后的登记（比登记跑的 `[1,2,3]` 窄），造出的 34 格是登记跑 239 格的一个
真子集，不是同一份数；但两边 `cold_recover_with_fault_failed == mount_writable_with_fault_failed`
这个等式在两份产物里都成立，说明触发条件相同（一次故障集合同时让 (a) 冷启动 recover 与 (b) 可写挂载失败），
本轮抽到的这 34 格可以代表登记跑那 239 格的失效形态。

### 2. 对着 D23 已定项 14「回退见证」判该不该拒

条款原文（`.claude/kb/decisions/23-journal的角色与格式.md:380`，kb 快照自己的行号，整行抄，「回退见证」那一段）：

```
**回退见证**（C332（回退实例两个根都读不出时回退被撤销） 的修法）：回退在系统配置槽里记一张见证表，放在系统配置字段表之后、越过 512 字节，靠系统配置的整槽校验和与两槽轮换认撕裂（越过 512 撕开的那一槽作废，等于这次写没持久）。一个条目记一次回退：新实例代号 4 字节、R_old 的实例代号 4 字节与 txg 8 字节。根 (i, T) 被条目 (N, r_old, T_old) 抛弃 ⟺ (r_old, T_old) < (i, T)（实例代号为主比）且 i < N；择根先跳过被任一条目抛弃的根，再照 D22（单元原子性怎么合成） 已定项 7 择新。见证随回退那一次挂载写行之后的第一次系统配置轮换写，之后每一次系统配置写都带着整张表。条数上限 = 根环槽数减 1（R × S − 1，只由根环几何定）。每次回退都崩在写行轮换之后、暖机之前，连着 R × S 次，根环全读得出也写得满（代码轮第二轮判决第三节）；写满时的处置见「回退见证的实现取法」②。见证表读不出就是系统配置槽读不出，挂载报错，不静默择根。字段落点随实现写进 D22（单元原子性怎么合成） 已定项 9 的字段表与 [layout/02-second-txn.md](../layout/02-second-txn.md)；条目什么时候删随实现，交代码三方。
```

这一段里管这一格判定的是两句：「根 (i, T) 被条目 (N, r_old, T_old) 抛弃 ⟺ (r_old, T_old) < (i, T)（实例代号为主比）且
i < N；择根先跳过被任一条目抛弃的根，再照 D22（单元原子性怎么合成） 已定项 7 择新」管「跳过」这一支；
「见证表读不出就是系统配置槽读不出，挂载报错，不静默择根」管见证表自己读不出的那一支——不是这一格命中的那一支，
但表达的原则一致：见证介入之后不许悄悄选中不该选的根。

代码逐行核对（都在冻结副本 `crates/singlefs-core/src/recovery.rs`）：

- `choose_root`（`:669-692`）遍历 `visit_valid_roots` 交出的每一个候选根，先判
  `rollback_witness.abandons(candidate.instance, candidate.checkpoint_txg)`（`:681`），命中就 `return`（跳过，
  不进 `best` 的候选），逐字对应条款「择根先跳过被任一条目抛弃的根」。
- 全部候选都被跳过时 `best` 保持 `None`，`choose_root` 交回 `None`（`:691`）。
- 调用方（`mount::mount_writable_with_space_admission`，`crates/singlefs-core/src/mount.rs:2117`）
  `choose_root(&*devices, &system_configuration).ok_or(RecoveryFailure::NoValidRoot)?`——`None` 就转成
  `RecoveryFailure::NoValidRoot`，包进 `MountError::Recovery(..)` 向上抛，任何写之前返回（同一函数在取号、
  写行之前）。

**这 34 格命中的正是「跳过之后一条候选都不剩」这一支，不是「见证表本身读不出」那一支**——两句判据管的是两种不同的坏，
这一格落在第一句管的范围里。第一句判据的字面就是「跳过」，没有写「跳过之后没有候选剩下时怎么办」，
但**从「不静默择根」这个总纪律**（第二句判据的措辞，虽然管的是另一种情形，但表达的原则一致：见证介入之后不能悄悄选中
不该选的根）**与已定项 14 的开篇定案**（`.claude/kb/decisions/23-journal的角色与格式.md:353` 整行抄）：

```
**定案**：**恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录；陈旧 tail 只是「从哪开始扫环」的优化，不再决定重放集合。**
```

能看出「跳过之后没有候选」等价于「这一份镜像上，按见证判定，没有一条根是这个时间线上还站得住的」——这时候
唯一站得住脚的选择就是**报错**，不是在「跳过」这条规则之外再去挑一条本该被跳过的根。**该拒。**

**与「成功但违例」那条基线对照**（数字已在第 0 节整行抄过 `e158-s10-runner-report.md:124-125`，这里不重复）：
旧行为（`pairs_with_any_violation` 在这批构造里非零）对应「`choose_root` 没有见证、静默选中被抛弃的根、
挂载成功、checker 之后判违例」；新行为对应「见证介入、候选清空、`NoValidRoot`、挂载在任何写之前就报错」。
后者把一次**已经发生**的违例，提前到**写之前**用一个明确的错误挡住，是收严不是收窄——没有把任何本该放行的历史拒掉，
只是把「放行了但错」变成了「不放行」。

---

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| S4 D28 已定项 1 九项逐格对齐 | 七项一致，两项结构上可能不同（未在这段历史上露头） | 容量/已分配/不可回收/defer 读法/checkpoint 保留池/待删占用/已承诺预留逐字节一致；挂载期承诺量的 `rows0` 与被抛弃根独占量只在挂载路径现算，发布路径整会话冻结，本轮历史里两者数值没有分叉 |
| S4「结论相反」的真正原因 | 不是口径不一样 | 实测两处公式九项与 `available(d)` 逐字节相同；差的是检查时点——下一次检查（无论是同会话再写一次还是关闭后重挂）才看得见上一次操作自己造成的 defer 双扣 |
| S4 候选 A（去掉 defer 双扣） | 有效但不彻底，且可能让回退更难 | 240/256/384 槽小盘放行的覆盖写次数从 5/6/10 涨到 11/11/21（今天的 crates/ 上重新量到），撞墙形态原样复现；384 槽在更深的 k 上还看到 24 次回退全部被拒（基线没有这种全军覆没） |
| S4 候选 B（预先计入本次释放） | 未实现、未测量，只是设计候选 | 预期能把「关闭之后才发现回不去」变成「写的时候就报 ENOSPC、池仍可挂载」，但要先解决「这次发布换下的块」与「这次新分配的块」会不会被重复计费 |
| S5 报的成员 | `MountError::Recovery(RecoveryFailure::NoValidRoot)` | 在冻结副本上重新造出 34 格（S4 几何、`sigma_length_limit=4`，登记跑 239 格的真子集），全部 34 格同一个成员 |
| S5 该不该拒 | 该拒 | `choose_root`（`recovery.rs:669-692`）逐字实现 D23 已定项 14「回退见证」的「跳过被见证抛弃的根」；候选清空之后交回 `None`→`NoValidRoot`，是把「静默选中被抛弃的根、之后 checker 判违例」改成「挂载这一步直接报错」，是收严不是收窄 |

## 没做什么

- **候选 B（预先计入本次释放）没有实现、没有在 Z19-B 历史上量**：只写了设计与预测，属于「推的」，见第 3 节的推翻条件。
- **S4 里两处结构性口径缺口**（挂载期承诺量的 `rows0` 时点、被抛弃根独占量只在挂载路径现算）**没有找到能让它们露头的历史**：
  今天量到的 Z19-B 各段里，这两项在发布检查与挂载检查之间始终数值相同；要坐实它们真的会导致「结论相反」，
  需要专门构造「行数逼近 369 的整数倍」或「同一会话内发生过回退又有根被抛弃」这两类历史，本轮没有构造。
- **候选 A 对回退路径的影响没有系统扫描**：只观察到 384 槽在 k=20、21 两个格子上回退 0/24；有没有更早的 k
  也会出现类似的「回退全灭」，没有逐格扫。
- **S5 的 239 格没有全量复现**：只用 `overwrites_before_sigma=[1]`（登记跑是 `[1,2,3]`）造出真子集 34 格；
  没有跑满登记跑的 1,116,897 对全量（那是重型负载，按定义不该由子 agent 全量重跑）。34 格与 239 格
  是否在每一格上都对应同一个具体故障集合，没有逐格比对，只核了「触发条件相同」（`cold_recover_with_fault_failed
  == mount_writable_with_fault_failed` 这个等式两边都成立）与「报的成员一致」。
- **S1、S2、S3 不归这条腿**，没有判。
- **主 agent 与其余腿要不要采纳哪个候选、S4/S5 最终怎么定，不由这条腿出判决**。

## 产物与复核

- 报告：`research/prompts/m2-safety-r1-sonnet-output.md`（本文件）。
- 模型与用例：`research/prompts/m2-safety-r1-sonnet-model/`，含 `SHA256SUMS`、`rerun.sh`、S4/S5 两组 patch、
  新增测试文件、原样日志。
- 交回前已用 `ps -o pid,ppid,args -u "$(id -u)" | grep -i "cargo\|rustc"` 现查过，没有命中，没有后台任务在跑。
