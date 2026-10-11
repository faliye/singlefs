# m3-formal-code-b31-r1 云端攻方（Opus）报告（2026-10-05）

轮：第 31 批代码轮第一轮。被攻对象：冻结副本 `/tmp/claude-1000/m3-formal-code-b31-r1/tree/` 的 `crates/`（下文代码引文的 `crates/…` 都指那棵树）。
攻击面：正文第三节 N1–N5，重点 N1、N3。副本上量出的数都是攻方副本上的，不算入库装置上的数。

## 复跑

模型目录 `research/prompts/m3-formal-code-b31-r1-opus-model/`。主 agent 在自己的草稿目录 `<草稿>` 里按次序跑（每步一条前台命令）：

```
bash research/prompts/m3-formal-code-b31-r1-opus-model/tools/rerun.sh <草稿> copy
bash research/prompts/m3-formal-code-b31-r1-opus-model/tools/rerun.sh <草稿> build      # 第一次从零编超过前台 240 秒，再起一次接着编
bash research/prompts/m3-formal-code-b31-r1-opus-model/tools/rerun.sh <草稿> mutants    # 8 趟，每趟约 80 秒，整条约 11 分钟，超过前台时限，放后台或拆开跑
bash research/prompts/m3-formal-code-b31-r1-opus-model/tools/rerun.sh <草稿> sweep      # 三批，约 1、2、3 分钟
bash research/prompts/m3-formal-code-b31-r1-opus-model/tools/rerun.sh <草稿> on-device  # 编约 30 秒、跑约 6 秒
bash research/prompts/m3-formal-code-b31-r1-opus-model/tools/rerun.sh <草稿> ignored    # 62 条慢档用例逐条，整条约 10 分钟，超过前台时限
```

全部是 harness 档（`singlefs-harness` 的测试二进制与一个依赖 `singlefs-checker-tier` 库的独立原型二进制），不跑 checker 档的测试、装置、QEMU 与变异整表。
线程 5，cargo 经 `run-with-memory-cap.sh 8G` 与 `capped.sh 5`。腿跑时的原始输出摘要在模型目录 `outputs/`（48 个文件）。

| 文件 | sha256 |
|---|---|
| `harness-tests/b31_opus_n3_reclaim_predicate_and_ring_table.rs` | `e848cbf52b2d296528bced3e31b78230194d6444a47fcfd2d2171da2fc32d94b` |
| `on-device-ring-table/Cargo.toml.template` | `be9e96792cb075d935e69aa8451705a381c22e288cec5e90498272ddea985638` |
| `on-device-ring-table/src/main.rs` | `a488932fdcf710f21d226cb96b93e12bf19737dfa181fe4b7ca5cef3b92c9f8e` |
| `tools/mutant-run.sh` | `5fe92d34353a97a029a01ef935f2dd8e8c33691f67c1a1165a33663514552e3c` |
| `tools/mutants.py` | `9bd727bbeb415c5fa78a83264d2dcf62c66bd1cd1e338f11ee80a6ec588fcd30` |
| `tools/rerun.sh` | `8d37c79e357f09889aec4168edc6407038848f4a433c3988e44205b51696e984` |
| `tools/sweep.sh` | `c5ab7921d7a17183b329c442c9c6a04c35393a3d53a43c9865a8a033717aa0e1` |
| `outputs/` 48 个文件按路径排序拼接 | `f318d3a8d6a3baf21ca0b4ddfd6a599043ba21bc362842fa26ace11723e29b81` |

原型三件：
- 甲（`scenario_a…`）：照 `build_pool` 搭的会话（盘换成内存稀疏盘），同一份 4100 字节覆盖写 200 次（txg 3→203），每一步核四样并跑池级 checker：① 分配器根环表里的有效根与盘上根环逐槽相同；② 31-3 新判法模拟的「环里最旧的根」与盘上根环最小 txg 相同；③ 新判法的「已回收」与 core 空闲图上真空着逐条相同；④ 每盘已分配 = 不是已回收的记录跨度之和。
- 乙（`scenario_b…`）：前缀固定在 build_pool 会话，之后按种子抽用户动作：覆盖写（0、1、1000、4100、9000、16000 字节）、抬 F（随机目标）、挂着时回退（随机挑盘上环里一条根）；每步同样核四样加 checker。
- 丙（`scenario_c…`）：每次覆盖写取一个崩溃态「这次发布第一道屏障之前的写全落、之后一个不落」，跑 checker 数违例（只打数）。
- 另一件独立原型 `on-device-ring-table`：`on_device_modes` 内存里拼的根环表与 core 两种来路逐字比，再各自接着覆盖写到 txg 80。

## 各格判定一览

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| N1 断言条件 | 没打中 | 产品路径上建分配器的只有两处，都装表；core 之外 23 + 4 处测试与装置的建法逐处对过；harness 档 165 个二进制之外再跑 62 条慢档：红的 4 条在撤掉 31-1 的对照副本上同样红、同样的数 | 量过（harness 档）；checker 档测试与装置推的 |
| N1 表与盘上的环不符而不被拦 | 没打中 | build_pool 前缀之后 64 个种子 × 60–150 步（覆盖写、抬 F、回退），表里的有效根与盘上根环逐槽相同，0 处不同 | 量过 |
| N1 `expect` 与 `# Panics` | 小打中（文字） | 字段注释说没装表的分配器做「准入读数」，`admission_reading_before_a_publish` 没装表就 panic | 读码 |
| N2 两处手拼的表 | 没打中 | `accounting_tree_node_full` 不做 mkfs，盘上环空、表空，对得上；`checker_known_bad_images` 那张表比盘上少几条根，但那个分配器上没有一条可回收的记录，回收结果不变 | 读码 |
| N2 C551 新用例罩不罩得住别的回收错法 | 打中（覆盖缺口） | 回收门槛高一代、低一代、释放代少记一、根环公式少一槽四种坏实现它都红；「带单元的发布在取落点之前记根」（R4）它与 31-3 判法、随机历史那条都照绿，只在崩溃态上红（60 次发布里 38 个崩溃态红，基线 0） | 量过 |
| N3 on_device_modes 的表与 core 的 | 没打中 | 4 种 S × 2 种块大小，txg 3 时三张表逐字相同；之后 77 次覆盖写每一步表都相同（分配记录从 txg 4 起不同，来源是提交内生块的 bump 起点，与表无关） | 量过 |
| N3 31-3 的模拟与 core 的谓词 | 不同一个来源，但在被测会话上逐条相同 | 甲 201 步、乙 6504 步：模拟的最旧根 = 盘上最小 txg，「已回收」= core 空闲图，0 处不同 | 量过 |
| N3 模拟与实现一起错 | 打中（判法不独立） | 31-3 判法从实现自己的分配记录读释放代、用 core 的 `target_for_publish` 推环：释放代少记一（R3）与根环公式少一槽（B3）下那条用例照绿；同档别的用例（31-1 新用例、随机历史那条）与 checker 抓到 | 量过 |
| N3 三个装置「输出不变」 | 没打中 | 在三个装置用到的会话长度（txg ≤ 4、≤ 15）上成立；回收在这一形会话里最早从 txg 26 起，更长的会话照变 | 量过（原型）；装置本身推的 |
| N4 注入入口 | 没打中 | 跑法里唯一一处造注入的地方交三格空，其余非空的都在 `mod tests` 里 | 读码 |
| N4 p6 补丁 | 没打中 | 从 Q5 树拿 `dbef78ed…` 现打 `p6-on-dbef78ed.patch`（fuzz 0）得 `2c933e21…`，与仓里那份 `cc736772…` 之差恰好 4 行 | 量过 |
| N4 U13、U14 期望数的前提 | 没攻 | checker 档内联用例，跑不了 | — |
| N5 新加与替换的 16 行 | 静态全过；「build_pool 不装根环表」那一行副本上证红 | 锚点各恰好 1 次、点名的测试各有 1 个定义且都没标 ignore | 量过 |
| N5 互相掩护 | 打中（推的） | 带 31-1 断言的 core 下，V32（U11 那一侧）与 E156 M35 那两行红在 core 的 panic 上，点名用例自己那条判据走不到；U11 的 I-3.1 判据在表里没有一行证红 | 读码，checker 档没跑 |
| 附带 | 与第 31 批无关 | 冻结副本上 4 条 harness 慢档用例红，`crates/mutations.tsv` 里 12 行点名它们，整表复跑时这 12 行基线就红 | 量过（对照副本） |

## N1　core 断言

### 断言的条件是不是恰好「没装根环表而记了一条根」

断言就是 `root_ring` 为 `None` 时 `expect`：「        let occupancy = self.root_ring.as_mut().expect(」（`crates/singlefs-core/src/allocator.rs:1919`）。要误 panic，只能是某条合法路径拿一个没装表的分配器记根。逐个来路数：

- 产品路径上建分配器的只有两处，两处都装：mkfs 同进程那一份「    allocator.install_root_ring_occupancy(root_ring_occupancy_after_make_filesystem(」（`crates/singlefs-core/src/make_filesystem.rs:731`），挂载重建那一份「    allocator.install_root_ring_occupancy(RootRingOccupancy::read_from_the_ring(」（`crates/singlefs-core/src/mount.rs:2105`）。core 里 `PoolAllocator::new` / `rebuild_from_records` 的其余调用点都在 `rebuilt_allocator` 那一串里或在单测里（`grep` 结果与 31-1 报告第 5、6 节的表一致）。取号之前的预演、抬 F 的预演都在分配器的拷贝上走（「    let mut copy = allocator.clone();」（`crates/singlefs-core/src/mount.rs:3641`），拷贝带着表。
- 回退（`roll_back_by_a_forward_publish`）、抬 F（`raise_rollback_floor`）、卸载、挂载内的零单元发布都只拿调用方交进来的分配器，不另建；回退那次发布与抬 F 那两次空发布都经分配器记根。没有一处把已装的表拿掉（core 里 `root_ring` 只在 `new` 里写 `None`、在 `install_root_ring_occupancy` 里写 `Some`）。
- 取号回卷、多实例：新实例只能经可写挂载开出来，挂载重建的分配器装着表。
- 测试与装置：冻结树里 `PoolAllocator::new(` 在 core 之外 23 处、`rebuild_from_records(` 在 core 之外 4 处。31-1 表外多出来的两处都在 31-2 的新装置里：第 10775 行只取文本、不发布；第 12070 行是 U12 故意造的不装表的分配器。

量过的（副本，harness 档）：
- 合入验证那一趟只跑了不带 `--ignored` 的 165 个二进制。我把 harness 包里标了 `#[ignore]` 的 65 条里能跑的 62 条逐条跑了（三条读环境变量的大档不跑），59 条绿、4 条红（日志 `outputs/ignored/`）：
  - `reuse_after_raising_the_floor` 的 `raising_the_floor_to_the_first_release_generation_reclaims_the_first_data_slot_and_the_next_publish_reuses_it`：「assertion `left == right` failed: I-5.5 在 E 之后的镜像上要成立」；
  - `rollback_by_a_forward_publish` 里三条：`plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_…`、`the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_zeroed_…`、`…_torn_after_the_first_half_…`，都是「崩溃之后重开：池级 checker 一条违例都没有」那一句 I-3.1 红。
- 对照：同一份副本撤掉 31-1 的两处（`expect` 换回 `let … else { return; }`、`build_pool` 不装表）重编，这 4 条照样红，数逐字相同（例：「盘 0：记账的已分配 Some(6455296)，遍历全部有效根得到 5537792」两边都是这个数；`outputs/control/`）。所以这 4 条与第 31 批无关。这 4 条在 build_pool 会话里都只写到 txg ≤ 10 就可写挂载、换成挂载建的分配器，31-1 改的那两处碰不到它们，与对照一致。
- 那 4 条之外，没有一条红在新断言（消息开头「分配器没装根环表却记了一条根」）或跳号断言上。

推的（checker 档，没跑）：checker 档测试与装置里会记根的建法，31-1 / 31-3 / 31-2 各补了装表；E156 W7 那一格阳性对照与 M35 那一行变异今天第一次记根就 panic，已在背景材料第二节「已知、已记的」里。

### 装的表与盘上的环不符而不被拦

`root_ring_occupancy_after_make_filesystem` 只记三个区域槽 0 的第 0 代根，mkfs 清成全零的别的槽不在表里。这与「mkfs 之后先挂载」装的表不同：挂载会把读回全零的槽记成「挂载时读回全零」、钉回收门槛。进程内这一份不钉，因为这个进程自己写的零，知道那里没有根。
只要这个进程之后写的每一条根都经分配器记，表与盘就一致。31-1 把「记根」从「没装表就静默跳过」变成「必记」，跳号断言随之在 build_pool 会话里第一次生效。

量过：原型乙把 build_pool 会话之后的用户动作放开扫（只固定前缀，没有故障），每一步核表里的有效根与盘上读得出的根（`readable_roots`，按 `target_for_publish` 归槽）逐槽相同：
- 种子 0–3 各 60 步，`outputs/sweep/s0-3-60steps.log` 原样：「name=scenario_b steps=244 steps_with_reclaimed_records=167 released_records_seen=141850 reclaimed_records_seen=58804 checker_runs=244 highest_txg=69 applied={"overwrite": 143, "raise_floor": 29, "roll_back": 34} refused={"raise_floor:RollbackFloorAboveCeiling": 22, "roll_back:TargetNotACandidate": 12} findings=0」
- 种子 100–139 各 80 步，4 件 × 810 步，每件 `findings=0`、最高 txg 90–97；
- 种子 200–219 各 150 步，4 件 × 755 步，每件 `findings=0`、最高 txg 163–169。
合计 64 个种子、6504 步，表与盘 0 处不同，checker 0 条违例，新断言与跳号断言 0 次 panic。

没扫的动作：unlink、建 inode、顺序写跨多单元（原型没接 `mounted_session`）；故障注入一种没加。

### `expect` 消息与 `# Panics`

消息写了依赖的不变量（I-3.1）与出路，`# Panics` 两条都在，没发现漏写。一处字段注释与代码不符（小，读码）：
- 字段注释说没装表的分配器「只做不发布的事（分配、准入读数）」：「    /// 都不装）。没装的分配器不许记根：`record_root_written_by_this_process` 断言（C551，用户 2026-10-05 定），只做不发布的事（分配、准入读数）。」（`crates/singlefs-core/src/allocator.rs:1296`）；
- 而准入读数的那个入口没装表就 panic：「        .expect("分配器上装着根环：挂载与 mkfs 建的分配器都装（`install_root_ring_occupancy`）")」（`crates/singlefs-core/src/admission.rs:1387`）。
没装表的只能走 `admission_reading_before_a_publish_on_a_ring_of`。这不是第 31 批引入的行为，只是这一批新写的注释说宽了。

## N2　`build_pool` 与装根环表

### 两处手拼的表

- `accounting_tree_node_full.rs`：79 / 80 块盘上不做 mkfs，直接发 txg 3。盘上根环全空，表也空（`latest` 2），对得上。记了 txg 3 之后表里最旧有效根就是 3，门槛升到 3，但这次发布不释放东西，回收不到。
- `checker_known_bad_images.rs`：表里只记实例 1 最新那条根。盘上环里还有实例 1 更早的根（含 txg 0 那一槽），所以这张表的「最旧有效根」比盘上的高，门槛偏高、偏不安全的方向。
  它不出事，靠的是那个分配器是 `new` 加 mkfs 两个单元现拼的，上面除了写行那次自己释放的旧链（释放代比门槛新）一条已释放记录都没有。这是这一处现在正确、改了搭法就会出事的前提，注释里写了「账与没装根环表时一样」，没写这个前提。读码，没量。

### C551 新用例罩不罩得住「记根时不回收」以外的回收错法（量过，副本）

在副本上打 5 种回收错法（`tools/mutants.py`），每种重编之后并行跑 5 条：
- ① 31-1 新用例 `root_ring_turn_in_the_make_filesystem_session`；
- ② 31-3 改过的 `allocated_statistic_equals_the_span_sum_of_the_unreclaimed_allocation_records_past_six_hundred_records`；
- ③ 随机历史里 mkfs 同进程转环那条（`turning_the_root_ring_with_overwrites_in_the_make_filesystem_process_…`，慢档）；
- ④ 原型甲；
- ⑤ 原型丙（崩溃态）。

| 坏实现 | 改了什么 | ① | ② | ③ | ④ | ⑤ |
|---|---|---|---|---|---|---|
| 基线 | 不改 | 绿 | 绿 | 绿 | 绿，`findings=0` | `violated_states=0` |
| R1 | 回收门槛 + 1 | 红（defer 账 346 ≠ 360） | 红 | 红（I-3.1） | 红 | 没跑（⑤ 是后加的） |
| R2 | 回收门槛 − 1 | 红（374） | 红 | 红（I-3.1） | 红 | 37 个态红，都是 I-3.1 |
| R3 | 释放代少记一（`record.generation = release_generation` 改成减一） | 红（I-3.1） | **绿** | 红（I-3.9） | 红（checker I-3.9） | 没跑 |
| R4 | 带单元的发布在取落点之前记根（`publish_admitted` 里那一次记根挪到「落点先于内容」那一段之前） | **绿** | **绿** | **绿** | **绿**，`findings=0` | **38 个态红**：I-2.1、I-4.8、I-7.4 各 38，I-3.10 37 |
| B3 | 根环公式每区少用一槽 | 红（318） | **绿** | 红（I-7.17） | 红（I-7.17） | 42 个态红，都是 I-7.17 |
| 变异表第 2736 行 | `build_pool` 不装根环表 | 红，panic 在新断言上 | 红，同 | 绿（这条不走 build_pool） | 绿（原型自己装表） | 0 |

原样摘几处：
- R4 的 ⑤（`outputs/mutants/R4-record-the-root-before-taking-placements/5-p3-scenario-c.log`）：「name=scenario_c steps=60 last_txg=63 violated_states=38 by_invariant={"I-2.1": 38, "I-3.10": 37, "I-4.8": 38, "I-7.4": 38} smallest_closed_form_state_count_of_one_publish=16777223」，第一处「first_violation=txg 26（第 23 次覆盖写，切在第 28 步）I-2.1：树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上」。
- 基线的 ⑤：「name=scenario_c steps=60 last_txg=63 violated_states=0 by_invariant={} smallest_closed_form_state_count_of_one_publish=16777223」。
- 变异表第 2736 行的 ①：「分配器没装根环表却记了一条根：写根的分配器都由建它的一方装上根环表…」，panic 位置 `crates/singlefs-core/src/allocator.rs:1919:49`。31-1 把这一行留给门禁证红，这里在副本上证了。

R4 的机理（读码 + ⑤ 量过）：
- 记根的位置是调用方的前置条件：「    // 这次的根要盖掉根环里的一个槽：落点都取完了、这次不再分配，在装记账行之前记下它——盖掉的是被抛弃的根就清它的隔离位，」（`crates/singlefs-core/src/transaction.rs:7844`）。
- 挪到「    // 落点先于内容：分配记录树要装自己那条（D3（空间分配） 已定项 5），所以这次重写的落点在装任何单元之前全部取定。」（`crates/singlefs-core/src/transaction.rs:7838`）之前，这次记根回收出来的槽当场被这次发布取走。被盖掉的那条根（txg 26 时是 txg 2 那条，它的树表就在槽 50178）在这次的根落盘之前还在盘上、还引用着这些槽。
- 发布做完之后那条根已被盖掉，所以 ①–④ 这些只看发布之后镜像的判据都看不见。

### 四句（R4 这一格）

- 分不分辨臂：这一格不判「装表 / 不装表」两臂，判的是 C551 新用例（与 31-3 判法）罩回收错法的范围。R4 在两臂上同样漏，所以不是装表引入的。31-1 改法把 build_pool 一族会话第一次带进「转环之后回收」，这一族流上的崩溃枚举从此能碰到这种错法。
- 系统看不看得到：①–④ 只看发布做完之后的镜像，那一刻被盖掉的根已不在盘上，看不到；判别它的东西只在发布中途的崩溃态里。
- 满足的是判据哪一句：正文 N2 最后一句「C551 的新用例罩不罩得住「记根时不回收」以外的回收错法」，罩不住 R4 这一种。
- 改法在这一格上还中不中：跑前条款没给改法。我提的改法见文末「自己提的改法」，只在我的模型上量过、被攻过零轮。

推翻条件：checker 档里已经有一条用例对「build_pool 一族、mkfs 同进程里转过根环」的流做层 0 崩溃枚举，并且在 R4 下红。我没跑 checker 档，没核到有没有这样一条。

### 装了之后哪些读数会变

- 会变的边界是能量出来的：这一形会话里第一次回收在 txg 26（盖掉 txg 2 那条暖机根，第一个文件在 txg 3 换下的 mkfs 树表这时可再分配）。原型甲 201 步里带已回收记录的 178 步（「steps_with_reclaimed_records=178」），与 txg 26–203 的步数相同（只对上了条数，没逐步打出来）。
- 随机历史那条慢档的注释写的也是这个数：「txg 26 那次覆盖写（第 22 步）盖掉 txg 2 的暖机根」。
- checker 档 30 份引 `build_pool` 一族的文件（29 份测试加 E161），我只做了静态扫描：数覆盖写调用点、找大的循环字面量，判不出哪几份在 mkfs 同进程里写过 txg 25。没逐份数，这一点同 31-1 报告第 9 节。

## N3　31-3 的装置与用例

### `on_device_modes` 拼的表与 core 给的是不是逐项相同（量过，没打中）

`allocator_rebuilt_from_the_records_with_the_root_ring_of_this_process` 自己拼第 0 代根：txg 取暖机第一条根减一（「            .checked_sub(1)」（`crates/singlefs-checker-tier/src/on_device_modes.rs:208`），F 取常量（「        ROLLBACK_FLOOR_AT_MKFS,」（`crates/singlefs-checker-tier/src/on_device_modes.rs:223`），再补记 txg 1–3（「        allocator.record_root_written_by_this_process(root.checkpoint_txg);」（`crates/singlefs-checker-tier/src/on_device_modes.rs:232`）。

原型 `on-device-ring-table` 比三张表的 `{:?}`：
- 甲：装置那条路，`run_new_pool_file_creation` 之后调上面那个函数；
- 乙：同一个进程里真分配器那条路，`allocator_after_make_filesystem` 起，取号、暖机、第一个文件；
- 丙：`root_ring_occupancy_after_make_filesystem` 装上之后记 txg 1–3。

几何扫 S ∈ {4, 5, 8, 16} × 块大小 ∈ {512, 4096}，共 8 格。原样（`outputs/on-device/run.out`）：
- 每格都是「genesis_txg=0 first_file_txg=3 same_first_file_output=true on_device_equals_in_process=true on_device_equals_core_table_plus_records=true」；
- 甲乙两边接着各用自己的分配器在同字节的盘上覆盖写 77 次（到 txg 80），每格「steps_with_different_ring_tables=0」。

同一份输出里每格还有「first_divergence=txg 4: 表同 true 记录同 false defer 同 true」。第一处不同是一条提交内生块的落点：甲 `SlotNumber(50304)`，乙 `SlotNumber(50253)`。
这来自 `rebuild_from_records` 重建的分配器不带进程内的 bump 起点，第 31 批之前的 `allocator_rebuilt_from_the_records_of` 就是这样，与根环表无关。summary 行的 `findings=8` 数的就是这 8 格的记录不同。

推翻条件：mkfs 改了第 0 代根写在哪几槽或 txg 取几，而 `on_device_modes` 那份没跟着改（31-3 报告第 9 节第 1 条已写这个代价）。今天的 mkfs 上量不出差别。

### 31-3 新判法的「已回收」与 core 的可再分配谓词是不是同一个

- 不是同一份代码：用例自己从第 0 代根起按 `target_for_publish` 推环（「            target_for_publish(CheckpointTxg(written_txg), slots_per_region),」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:686`）；
- 用例判「已回收」读的是实现交回的分配记录（「            |record: &&AllocationRecord| record.is_released && record.generation <= reclaim_floor;」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:728`）。
- core 的回收按表里最旧有效根与表里的 F；条款是 `.claude/kb/decisions/16-发布语义.md@已定项 1：defer 窗口取按盘回退下界的收严形态，按状态数读` 的「可再分配」那一行，与 `.claude/kb/decisions/03-空间分配.md@已定项 7：分配记录条目的盘上编码，与分配准入合取的登记表` 的「条目留到该落点被重新分配时覆盖」。

在被测会话上逐条相同（量过）：
- 原型甲照抄那条用例的会话（同一份 4100 字节、200 次），每一步核：模拟的最旧根 = 盘上根环最小 txg；新判法说已回收 ⇔ core 空闲图上那几槽空着；每盘已分配 = 不是已回收的记录跨度之和；checker 全跑。
- 原样（`outputs/mutants/baseline/4-p3-scenario-a.log`）：「name=scenario_a steps=201 steps_with_reclaimed_records=178 released_records_seen=156984 reclaimed_records_seen=39098 checker_runs=201 highest_txg=203 applied={"overwrite": 200} refused={} findings=0」。
- 原型乙在抬 F、回退混进来的 6504 步上同样 0 处不同（N1 一节）。抬 F 之后用例取的是根上的 F，core 的表里 F 不跟着抬，两边照样相同，因为抬 F 那一串自己按新 F 回收过。

这一条附带回答了 C551 的推翻条件（只指路：`.claude/kb/checks-owed.md` 第 425 行 C551 那一行的第三栏）：同形会话 201 个版本上全部不变量都没红。量的是副本上的同形会话，没改那条用例本身。

### 造一个模拟与实现一起错的坏实现（量过，打中）

新判法与 core 共用两样输入：释放代（从实现自己的分配记录读）、`target_for_publish`（core 的函数）。各造一个只改这一样的坏实现，见 N2 那张表的 R3、B3 两行：
- R3 释放代少记一：「            record.generation = release_generation;」（`crates/singlefs-core/src/allocator.rs:1575`）改成减一。新判法那条用例（②）**绿**。实现提早一代回收，环里最旧的那条根还引用着被回收的槽。31-1 新用例在 I-3.1 上红，checker 在第一个文件那一版就判 I-3.9 红（「盘 1 槽 50178 的分配记录释放代 2 不在 (2, 3] 里」）。
- B3 根环公式每区少一槽：「        slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:115`）改成 `count() - 1`。②**绿**。31-1 新用例在 defer 账上红（318 ≠ 360），checker 判 I-7.17 红。

四句：
- 分不分辨臂：这一格不判臂，判的是新判法够不够独立。R3、B3 在 build_pool 装不装表两种搭法下都改坏同一处，同一条用例都绿，所以与装表无关，病在判法从实现取了两样输入。
- 系统看不看得到：那条用例只看分配记录与记账行，看不到盘上的根环与根引用的单元，R3、B3 的判别子不在它手里；同一会话上跑 checker 就看得到（原型甲 ④ 列红）。
- 满足的是判据哪一句：正文 N3「模拟写错会不会让用例与实现一起错」。严格说一起错的不是模拟（模拟本身在 6705 步上与盘相同），是模拟的两样输入取自被测实现。
- 改法还中不中：跑前没给改法。我提的改法（那条用例每一版或至少末一版跑 checker）在 R3、B3 上都红，见文末，只在我的模型上量过、零轮。

### E170、旧 E171、E156 内联单测「输出不变」的推论在更长会话上还成立吗

- 推论的前提是这几处装了表的分配器最多记到 txg 4（E170、旧 E171，流二随后换成挂载建的分配器）、txg 15（E156 内联单测）。
- 这一形会话里第一次回收在 txg 26（N2 末节），所以在这几个长度上成立。会话更长就不成立：E171 新装置流三写到 txg 30 以上，常量已按修订 7 重取，背景材料与 31-2 报告都写了。
- 推的（没跑装置）：旧 E171 的 s4 坏实现表，我按锚点与替换文里带 txg 加减、暖机、补记零单元根这几样扫了两张表（117 行、164 行），只扫出 117 表第 95 行一行会在 mkfs 会话里改变结局，与 31-3 报告第 4 节一致。挂载层那几行在挂载之后才作用，那时用的是挂载建的分配器。

## N4　31-2 的 E171 新装置

- 注入入口（读码，没打中）：跑法里造 `ConfirmedLossShapes` 的只有 `run_confirmed_loss_form` 一处，三格一律交空：「            // 跑法里三格一律空、命令行不开放（登记修订 6 第 6.2 节第 3 条；修订 5 第 5.1 节…注入只经装置内部的开关进…）。」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8037`）。其余造非空注入的都在 `mod tests` 里，`PiecePanicInjection` 在测试之外只有阳性对照内置的那一份，那是原有设计。没找到命令行或环境变量能把它们打开的路。
- p6 补丁（量过，没打中）：
  - 从 `scripts/experiments/target/e171-r1-frozen-tree-revision-5-q5/` 拷出装置（sha256 前缀 `dbef78ed3eede487`），`patch --fuzz=0 -o` 打 `/tmp/claude-1000/m3-formal-31-2/patch/p6-on-dbef78ed.patch`，得 `2c933e2136f991a6…`；
  - 它与冻结树里那份（`cc73677273fd4a4f…`）`diff` 只多 4 行（`RefusalConditionNotMet::SystemConfigurationUnreadableAfterTheReadStageCriterionTrueOnACrashImageWithoutReadFaults` 那一个分支），这个名字在 `research/prompts/_m3-formal-code-b29-r1-diff.md` 里出现 12 次，是第 29 批加的。
  - 另：`scripts/experiments/target/e171-r1-frozen-tree-revision-6/` 里那份装置的 sha256 前缀是 `67e7a5498425ec3f`，与 `2c933e21…` 之差是流三的段长表、写数、全量状态数与闭式那几个常量加两行注释（修订 6 第 6.2 节第 2 条重取），不是 p6 本身。
  - 那棵树的 core 不带第 31 批断言，U12 的注释写明「修订 6 树的 core 不带、那棵树不编用例」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:12281` 那一行起）。
- U13、U14 的期望数：checker 档内联用例，没跑，没攻。

## N5　变异行

- 静态（量过，冻结树）：第 31 批新加与替换的 16 行（第 570 行 m4b 换点名那一行，第 2734–2748 行），锚点在被改文件里各恰好 1 次，点名的测试各恰好 1 个定义、都没标 `#[ignore]`。
- 证红（量过，副本）：第 2736 行「C551：build_pool 不装根环表」31-1 留给门禁，我在副本上打了，点名的新用例红在新断言上（N2 那张表最后一行）。31-1 / 31-3 已证红的几行没重证。checker 档那 11 行没跑。
- 第 2734 与第 2748 行、第 2742 与第 2743 行改法逐字相同、点名不同的测试，门禁的重复判据明写这样合法（`.claude/gate.d/code-source-discipline.sh` 的 `CRATES_DUP` 只拒四样都相同的）。

互相掩护（推的，读码；checker 档没跑）：
- 第 2743 行（V32，U11 那一侧）：在带第 31 批断言的 core 上，删掉装表那一句之后第一个文件那次发布就在 core 里 panic，U11 红在「每次调用都成功」上。U11 自己的注释也这样写：「    /// U11（登记修订 6 第 6.5 节；V32 下两种 core 都红：带第 31 批断言的 core 在第一个文件那次发布 panic，…每次调用都成功…红；」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:12204`）。U11 的本意是转环之后 I-3.1 成立，那一句判据在表里没有一行能让它成为红的原因：点名 U11 的只有这一行（`grep -c` 为 1），这一行又先撞 core 断言。一个让 U11 的 I-3.1 那一句失效的坏用例（比如把判 I-3.1 那一段删掉）在整表复跑里不会被发现。
- 第 803 行（E156 M35，「岔路 1 的族的分配器退回不装根环表」）：改法是把产品路径换成 `allocator_without_the_root_ring`，第 31 批之后这个分配器第一次记根就 panic。这一行照旧会红，但红因从「点名用例判出不回收」换成了「core 断言」。31-1 报告第 9 节已点到这一行，没核。

## 附带看到的，与第 31 批无关（量过，对照副本）

- 冻结副本上 harness 慢档 4 条红（N1 一节），撤掉 31-1 的对照副本上照样红、数相同。
- `crates/mutations.tsv` 点名这 4 条的有 12 行（第 32、33、34、40、44、45、47、191、193、739、2018、2019 行，前 11 行点名 `raising_the_floor_to_the_first_release_generation_…`，第 739 行点名 `plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_…`），整表复跑时基线就红。
- 归哪条线、什么时候红起来的，我没查（工作区里别的线有未提交改动，背景材料第二节）。

## 自己提的改法（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | R3 | R4 | B3 | 依据 |
|---|---|---|---|---|---|
| 甲：`overwrite_in_one_instance` 那条用例每一版（或至少最后一版）跑池级 checker、要求 0 违例 | N3 判法不独立 | 量过：红（I-3.9） | 量过：不红 | 量过：红（I-7.17） | 原型甲 ④ 列，`outputs/mutants/*/4-p3-scenario-a.log` |
| 乙：加一条 harness 档用例，在 build_pool 会话转环之后，对每次发布取「第一道屏障之前的写全落」那一个崩溃态跑 checker（原型丙的形） | N2 R4 那一格 | 推的：会红（checker 在发布之后的镜像上已红） | 量过：38 个态红，基线 0 | 量过：42 个态红 | 原型丙，`outputs/mutants/*/5-p3-scenario-c.log`；只取一个崩溃态，不是层 0 全量 |
| 丙：加一行变异「`publish_admitted` 记根挪到取落点之前」，点名乙那条用例 | N2 R4 那一格的证红 | — | 推的：乙红它就红 | — | 由乙推 |
| 丁：再加一行变异让 U11 红在 I-3.1 上（例如照第 2735 行的改法「记根时环转了也不回收」点名 U11） | N5 第 2743 行掩护 | — | — | — | 推的：U11 注释说不带断言的 core 不回收时 txg 26 起 I-3.1 红；第 2735 行的改法不碰断言、也不回收 |

## 没打中的形状与取样范围

| 形状 | 取样 | 结果 |
|---|---|---|
| 合法路径在装表之前记根（N1） | core 产品路径两处建法、回退、抬 F、卸载、挂载内零单元发布逐个读；core 之外 23 + 4 处建法逐处对；harness 档 62 条慢档（对照副本 4 条） | 没有一条红在新断言或跳号断言上 |
| build_pool 前缀之后用户动作放开扫（N1、N3） | 64 个种子：0–3 × 60 步、100–139 × 80 步、200–219 × 150 步；动作抽覆盖写 6 种长度、抬 F、回退（各件 `applied` / `refused` 相加：做成 5562 次、被拒 878 次，加 64 次起点核对共 6504 步）；最高 txg 169 | 表与盘、模拟与盘、谓词与 core、已分配与跨度和、checker：0 处不同 |
| on_device_modes 的表（N3） | S 4 种 × 块 2 种；txg 3 时比三张表，之后各覆盖写 77 次 | 表 0 处不同 |
| 31-3 判法在原会话上（N3） | 200 次覆盖写，txg 3→203，每步 checker | 0 处不同 |
| 回收错法（N2） | R1、R2、R3、R4、B3、第 2736 行 | 只有 R4 漏过全部发布之后的判据；R3、B3 漏过 31-3 判法一条 |
| 注入入口（N4） | 全文读 `ConfirmedLossPiecePanicInjections` 与 `PiecePanicInjection {` 的每个造处 | 测试之外只有交空与阳性对照内置的一份 |
| p6（N4） | 现打补丁、比 sha256、`diff` | 差恰好 4 行 |
| 变异行静态（N5） | 16 行 | 0 处不合 |

取样上没做的：
- 原型乙没接 unlink、建 inode、跨多单元顺序写、故障注入；
- 原型丙每次发布只取一个崩溃态，不是层 0 全量：一次覆盖写的闭式状态数最小 16 777 223（「smallest_closed_form_state_count_of_one_publish=16777223」），超过定义给的约 10⁶ 一段，全量不跑；
- 场景乙 64 个种子 × 每段 60–150 步合计约 6.5 千步，单件估时按先跑 4 个种子 26 秒（约 0.11 秒一步）定，每批 4 件并行、每件 755–810 步。

## 这条腿自己的限度

- 我没跑 checker 档（测试、装置、内联用例、变异整表、QEMU）。凡是涉及它们的判定都写了「推的」：N1 里 checker 档的建法，N2 里 checker 档有没有一条能抓 R4，N5 的两处掩护，U13 / U14。
- 副本上量的数是攻方副本上的，不是入库装置上的。原型丙的「38 个态红」是一个人工取的崩溃态族，不是层 0 枚举域。
- 对照副本只撤了 31-1 的两处（core 断言、build_pool 装表），31-3 / 31-2 改的文件没撤。那 4 条红的用例不经那些文件，所以够判「与第 31 批无关」；推翻条件是那 4 条里有一条经过了 31-3 / 31-2 改的文件。
- 违了两处规矩，照实记：
  - 一次在循环里不经内存包装直接执行了 harness 测试二进制的 `--list --ignored`（只列名字、不跑用例），之后都经包装；
  - `on-device-ring-table/src/main.rs` 第一段一次写了 157 行，超过一次 150 行。
- 中途踩过一次 cargo 复用旧产物：坏实现撤回时我保留了原文件的修改时间，下一趟「基线」用的仍是 R4 编出来的 core。发现之后改成撤回时刷新修改时间（`tools/mutants.py` 里写了），并把那一趟日志改名 `baseline-was-actually-R4` 留在草稿目录，不当基线引。表里的基线行与 R4 行都是之后重跑的。

## 没做什么

- 不判正推那几格，不替主 agent 采纳。
- 没读本轮正推腿的报告、别轮的腿报告与禁读清单里的产物。
- 没改仓里任何文件。写过的只有本报告与模型目录。
- 没跑 checker 档、QEMU、变异整表、实验复跑。

## 草稿目录处置

`/tmp/claude-1000/m3-formal-code-b31-r1-opus/`：
- 删掉：仓副本 `copy/`、编译目录 `target/` 与 `target-proto/`、原型工程 `proto/`、p6 核对用的 `p6/`。撤回 31-1 的对照副本 `copy-ctl/` 用完就删了。
- 留着（只有文本）：日志 `ign/`、`ign-ctl/`、`mutants/`、`p1/`、`p3/`、`rc/` 与几份辅助脚本。

## cite-check

命令：`python3 scripts/cite-check.py research/prompts/m3-formal-code-b31-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b31-r1/tree --root . --background research/prompts/_m3-formal-code-b31-r1-background.md`，末行原样：

```
  ✓ 核了 18 处引文，对上 18 处，没判 0 处（逐处列在上面）
```
