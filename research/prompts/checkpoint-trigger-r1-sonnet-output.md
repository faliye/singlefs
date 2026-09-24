# checkpoint-trigger-r1 云端正推腿（Sonnet）—— K1「第三支量得出来吗」

角色：三方论证的正推腿，只攻 K1。不碰 K2、K3、K4（`.claude/rules/three-way-inference.md`「各条腿必须互不重复」）。

开工核对：`sha256sum -c research/prompts/checkpoint-trigger-r1-start-snapshot.sha256` 全部 `OK`——8 个背景文件与开工快照一致，下面引用的都是这一份。

## 判据先引全（`.claude/rules/fs-design.md:17-38`，整段抄）

```
## 记账是事务的副产品，不是事后的遍历

**线划在「谁消费这个数」，不划在「用什么机制算出来」。**

| 消费者 | 允不允许遍历 | 为什么 |
|---|---|---|
| **运行时决策路径**（分配、ENOSPC 准入、defer 窗口、生命周期判定） | **不许**，且代价不许随盘容量增长 | 不是慢，是**在被问到的那一刻没有答案**。准入控制要「进门前先算最坏情况」，而「释放空间这个操作本身不需要申请空间」也压在同一个数上 |
| **checker / 审计** | **必须**遍历 | 若运行时也用遍历算，checker 的遍历与运行时就是**同一次计算**，对照关系当场归零 |
| **后台、可续做、非决策路径**（销毁快照、整理、scrub） | **无戒律** | 它们走「写意图 → 分批 → 可续做」，不参与任何即时判定 |

**空间统计与配额落在第一格**：必须在事务提交时增量维护出来。

⚠️ **一个量落不落在第一格，按这一格自己的判据判：「被问到那一刻有没有答案」，不按名词清单判。**
「快照用量」就是这样判出第一格的：没有任何运行时判定读「每树独占 / 每树共享字节」，
那两个统计量因此撤回、第一版不提供每快照用量——普查、依据与将来做 per-snapshot 配额时怎么办，
权威记录在 `.claude/kb/decisions/05-快照-空间记账机制.md` D5（快照 / 空间记账机制） 已定项索引表第 8 行。

⚠️ **被禁止的不是「遍历」，是两件事**：
「在需要答案的那一刻还没有答案」，以及「审计与被审计用同一段代码」。
第二条不依赖任何关于性能的假设——它要求的恰恰是**运行时与 checker 必须用不同的算法**。

⚠️ **别拿第一格的禁令去否第三格的设计。** 本工程已经在第二、三格豁免过三次：
checker 的全盘遍历、无界删除的意图分批、代际增量 scrub。
**一条被三次豁免的规则，它的判据就不在它字面写的那个量上。**
```

## 判定一览（先给结论，逐条论证见下文各节）

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1：今天有没有实现 | **一致**（与主 agent 的观测一致） | `grep -rn "T_time\|T_dirty\|checkpoint_trigger" crates/ --include=*.rs` 只命中系统配置的字段声明与注释，触发逻辑零实现 |
| Q2：量本身要不要遍历才算得出来 | **算得出来，不需要遍历** | `crates/singlefs-core/src/write_accounting.rs` 已经实现了同构的「按结构种类累计 + `.since()` 求两次快照之差」，专门对 `JournalRecord` 计过字节；`journal.rs` 的 `counter: u64` 是纯算术量，`current − marker` 是 O(1) 减法 |
| Q3：落在 fs-design 三格的哪一格 | **第一格（运行时决策路径），按这一格自己的判据「被问到那一刻有没有答案」判，答案是「有」** | checkpoint 触发判据是运行时决策，量本身满足「事务提交时增量维护」这句原话——不是被禁止的那种「没有答案」的遍历 |
| Q4：增量维护要付什么、放哪、崩溃后怎么办 | **一个新的标量 marker，内存即可，代价与盘容量无关；镜像计数会重复计数，要用 counter 算术而非写字节计数** | 见「Q3 之后：怎么维护」一节 |
| Q5（反过来想）：算不出来会怎样 | **不适用——它没有撞上第一格的禁令，候选乙不因这一面出局** | 但候选乙自己那句「这个 checkpoint 间隔」在今天的决策文本里指代不清，这是另一件事（见「留给 K4 的一个观测」） |

## Q1：今天 `crates/` 里有没有实现（观测，非推论）

```
$ grep -rn "T_time\|T_dirty\|checkpoint_trigger" crates/ --include=*.rs
crates/singlefs-core/src/system_configuration.rs:139:/// 字段表里归这一档的 3 行：可调值段的「T_time 4」「T_dirty 8」「整理低 / 高 / 停三水位 8 × 3」。
crates/singlefs-core/src/system_configuration.rs:141:/// 也没有触发路径——T_time 与 T_dirty 的发布触发没实现，整理三水位恒 0 = 内置默认
crates/singlefs-core/src/system_configuration.rs:151:    /// T_time（D16（发布语义） 已定项 5）。
crates/singlefs-core/src/system_configuration.rs:157:    /// T_dirty（D16（发布语义） 已定项 5）；挂载时的有效值另按环长夹取，那不改盘上这 8 字节。
crates/singlefs-core/src/system_configuration.rs:554:            "系统运行配置：T_time 4 + T_dirty 8 + 整理三水位 24"
```

零命中 `checkpoint_trigger`。`system_configuration.rs:141` 那一行是代码自己写的注释，逐字确认了背景材料 39 行「主 agent 的观测」：`T_time` 与 `T_dirty` 只有盘上字段位，没有触发路径。**什么现象会推翻它**：以后任何一次 `crates/` 改动让这条 grep 出现除字段声明外的命中，就要重新判 Q1。

## Q2：这个量本身算不算得出来（正推：从代码推到结论）

先分两句问，对应规则里「被问到那一刻有没有答案」这句判据：

1. **需要答案的那一刻是什么时候**：每次要不要触发 checkpoint 的判定点（无论是写路径里顺手查一下，还是一个独立的定时/触发轮询）。
2. **答案是不是已经在那一刻之前就被算好、只等取用**：这是「记账」；反面是「那一刻才现算」，也就是遍历。

`crates/singlefs-core/src/write_accounting.rs` 已经实现了一个结构完全对应的量，可以直接拿来做正推的证据——它不是候选乙要的那个量本身，但它证明了「journal 记录按字节增量记账」这件事在这个代码库里**已经有实现、已经有测试**，不是要凭空造一种新机制：

`write_accounting.rs:1-6`（整段抄）：
```
//! 每次发布按结构种类计的写（里程碑「第二个事务」增补 1 第 1 件）：写调用数与写字节。
//!
//! 种类由发布路径在构造提交步骤时给出：单元写带着它的角色，journal 记录、根槽、系统配置槽由步骤成员本身定。写入口把一次写交给设备、
//! 设备报成功之后记一笔。记账不发写、不改写的次序、屏障与段序列，也不加提交步骤成员或块设备动作（D17（实现分层与第三方管道）
//! 已定项 2 / 已定项 5）。一次写调用 = 交给一块盘的一次 `write_at`：镜像的两份各算一次，与设备一层数的口径相同，
//! 所以一次发布按种类的合计要与设备一层数的逐次相等。
```

它按 `WrittenStructureKind::JournalRecord` 这一种，在**写入口写成功那一刻**记一笔（`transaction.rs:142-148`，整段抄）：

```
            CommitStep::WriteJournalRecordToEveryDevice { counter, record } => {
                let offset = record_offset(counter, self.parameters.geometry.journal_ring_bytes);
                for (_, device) in self.devices.iter_mut() {
                    device.write_at(offset, record, WriteDurability::Plain)?;
                    self.writes_by_structure_kind
                        .count_write_call(WrittenStructureKind::JournalRecord, record);
                }
            }
```

`count_write_call`（`write_accounting.rs:117-124`，整段抄）只做一次 `BTreeMap` 查找 + 两个整数相加：

```
    /// 记一次写调用：这一种的写调用加 1、写字节加这次写的长度。
    pub fn count_write_call(&mut self, kind: WrittenStructureKind, bytes: &[u8]) {
        let entry = self.counted.entry(kind).or_insert(WriteCallsAndBytes::NONE);
        *entry = entry.plus(WriteCallsAndBytes {
            write_calls: 1,
            written_bytes: u64::try_from(bytes.len()).expect("一次写的长度装得进 u64"),
        });
    }
```

取两次快照之差同样是 O(1)（`write_accounting.rs:150-175`，`since` 方法整段抄，节选核心两行）：

```
    pub fn since(&self, earlier: &WritesByStructureKind) -> WritesByStructureKind {
        ...
                        write_calls: later_writes.write_calls - earlier_writes.write_calls,
                        written_bytes: later_writes.written_bytes - earlier_writes.written_bytes,
```

这套机制今天已经在跑（不是设计稿）：`transaction.rs:247-253` 的 `count_failed_publish` 就是拿它给「落盘阶段中途失败的发布」记账，同一份代码里已经在使用「取一次快照、之后再 `.since()` 减一次」这个模式。**这就是候选乙第三支要的量在结构上需要的全部东西**：一个在提交时递增的计数器，加一次减法。它不读盘、不扫环、代价是常数，与盘有多大、环里已经写了多少条记录**无关**——不满足 fs-design.md 第一格「代价不许随盘容量增长」这条禁令的任何一个反面条件。

**独立于 `write_accounting.rs` 的第二条证据**：journal 记录自己带一个纯算术量，`journal.rs:83`：
```
pub struct JournalRecord {
    ...
    pub counter: u64,
    ...
```
`counter` 是全实例范围内单调递增的记录序号，`record_offset`（`journal.rs:107-114`，整段抄）把它换算成环内物理偏移：
```
/// 记录 n 落在环内偏移 `(计数器 − 1) mod 槽数 × 4096`（D23（journal 的角色与格式） 已定项 18）。
#[must_use]
pub fn record_offset(counter: u64, ring_bytes: u64) -> DeviceOffsetInBytes {
    let ring_slots = ring_bytes / JOURNAL_RECORD_BYTES;
    DeviceOffsetInBytes(
        JOURNAL_RING_START_SLOT * SLOT_BYTES + ((counter - 1) % ring_slots) * JOURNAL_RECORD_BYTES,
    )
}
```
`counter` 本身不取模——取模只发生在算物理偏移的那一步。所以「从某个标记点 `marker_counter` 到当前 `counter` 之间写了多少字节」= `(counter − marker_counter) × JOURNAL_RECORD_BYTES`，同样是一次减法一次乘法，不涉及环的物理布局、不用管绕环。这条路径比借道 `write_accounting.rs` 更干净，理由见 Q4 的「镜像重复计数」那条坑。

**反推**：假设这个量算不出来（即真的需要遍历），我本该在 `crates/` 里看到什么？本该看到——`I-8.1（环几何够大）` 的「最坏占用」是 mkfs 时静态声明的量（背景材料 43 行的观测），或者看到某处运行时代码为了拿到「当前占用」去读盘、扫记录。我去查了：`crates/singlefs-core/` 里唯一会遍历 journal 环的代码是 checker 那一侧（见 Q3），核心层的提交路径（`transaction.rs`）从头到尾只在写成功那一刻做 O(1) 记账，没有任何一处为了「查询当前占用」而读盘。这个反推没有被推翻。

**什么现象会推翻 Q2 这条结论**：如果日后发现 `counter`（或 `write_accounting` 的累计值）本身在崩溃后无法可靠恢复到「continue from here」的状态、必须靠扫环才能重建（不是「要不要额外记一笔账」的问题，而是「记的账本身靠不靠得住」的问题），那就要把这一条改判——那时候增量记账退化成了「记了也没用，恢复时还是要扫」，见 Q4 的讨论。

## Q3：落在三格的哪一格

`.claude/rules/fs-design.md:29`（整段抄）先把判据钉死：

```
⚠️ **一个量落不落在第一格，按这一格自己的判据判：「被问到那一刻有没有答案」，不按名词清单判。**
```

不按名词清单判，所以不能因为「checkpoint 触发」这个名字没有逐字出现在表格的举例列（`分配、ENOSPC 准入、defer 窗口、生命周期判定`）就归到别的格。按判据本身走：

- **谁在问这个量**：D16（发布语义） 已定项 2 的触发式子——`触发 ⟺ (now − 上次 checkpoint 的时刻 ≥ T_time) ∨ (脏字节数 ≥ T_dirty) ∨ (候选乙的第三支)`，这个 `∨` 式子要在**运行时**被求值，用来决定要不要**现在**发起一次 checkpoint（一次发布，见 D16（发布语义） 已定项 6「fsync 触发的发布也是发布」）。这是一次决定「现在做不做一个动作」的判断，不是审计、也不是可以拖到后台慢慢做的整理，三格里只有第一格（运行时决策路径）符合。
- **被问到那一刻有没有答案**：按 Q2 的证据，如果维护一个 marker（`counter` 或 `written_bytes` 的快照）作为 checkpoint 完成那一刻的副产品，那么任何时刻问「这个间隔占用了多少」，答案是 `current − marker` 现成摆在那里，不需要现场去读盘、扫环——**有答案**。

⇒ 落在**第一格**，而且规则原句「空间统计与配额落在第一格：必须在事务提交时增量维护出来」（`fs-design.md:27`）里的动词「增量维护」正是 Q2 证明可行的那件事。

**与二、三格划清界限**：
- 不是第二格（checker / 审计）。checker 侧确实有一段会遍历整个环——`crates/singlefs-checker/src/walk.rs:1705-1736` 的 `scanned_journal_records_of_device`（整段抄的关键行）：
  ```
      let slots = reader
          .candidate_journal_slots(device)
          .unwrap_or_else(|| (0..geometry.journal_ring_bytes / JOURNAL_RECORD_BYTES).collect());
      ...
      for slot in slots {
  ```
  这是 `0..ring_bytes/RECORD_BYTES` 的整环遍历，专门喂给 `I-8.6（反向链算法）`（`walk.rs:1762-1771` 附近的 `judge_journal_back_chain`）与 `I-8.7`。它证明「遍历整环」这件事在这个代码库里是存在的、但**只存在于 checker 这一条完全独立的代码路径**，与核心层的提交路径（`transaction.rs`）没有共享一行代码——满足 `fs-design.md:34-36` 那条「运行时与 checker 必须用不同的算法」。候选乙的第三支如果去复用 `scanned_journal_records_of_device` 这类扫描逻辑，就会撞上这一条禁令；但 Q2 已经证明它不需要复用，走 `counter` 算术即可。
  - 不是第三格（后台、可续做、非决策）。第三格「无戒律」是给「销毁快照、整理、scrub」这类不参与任何即时判定的动作用的（`fs-design.md:25`）；checkpoint 触发恰恰**就是**一次即时判定（要不要现在发布），把它塞进第三格等于否认它是决策路径，这与题面「触发判据」四个字本身矛盾。

**什么现象会推翻这一条**：如果候选乙最终的实现形态是「checkpoint 触发只在后台巡检线程里、允许滞后任意长时间才响应」，那就要重新论证它是不是真的「运行时决策路径」；但即便滞后，只要它最终仍然是「决定现在要不要发布」而不是可无限期推迟的整理动作，判定不变。

## Q4：增量维护要付什么代价，放哪，崩溃后怎么恢复

**代价**：一个 `u64`（或与 `journal.rs` 的 `counter` 同类型），一次比较、一次减法、一次乘法（`× JOURNAL_RECORD_BYTES`）。不随盘容量、环长增长——盘越大环越大，这次减法还是这次减法。满足 `fs-design.md:23`「代价不许随盘容量增长」这一列。

**放哪**：

- **最小必要形态：只放内存。** 这个 marker 只是「上一次 checkpoint 完成时 `counter` 的值」，在同一次挂载会话里，任何一次成功发布（`transaction.rs:172-178` 的 `RotateSystemConfigurationSlots` 那一步之后）把它更新成这次发布的 `plan.counter`——这与今天已经存在的写法完全同构，`transaction.rs:581-584`（零单元发布）与 `transaction.rs:2165-2168`（普通发布）两处都已经在每次发布时写：
  ```
              writer.perform(CommitStep::RotateSystemConfigurationSlots {
                  journal_tail: plan.counter,
                  journal_instance: plan.instance,
              })
  ```
  即：「在发布完成的那一刻，把某个标量设成 `plan.counter`」这件事**今天已经在做**（`journal_tail` 字段），候选乙的 marker 只是同一时机、同一动作模式下再多记一个标量，不是新引入一种机制。
- **要不要落盘**：这是一个可以另开的岔路，不是 K1 判据本身要求的。这个量是一个**软触发的启发式输入**，不是崩溃一致性要保护的不变量——`.claude/kb/invariants.md` 的 I 系列（`I-8.1`、`I-8.6` 等）都是「镜像合不合法」层面的东西，triggered-too-late 或 triggered-too-early 都不改变镜像合法性，只改变「丢失窗口有多大」「环会不会被撑爆」这类性能 / 鲁棒性指标。所以：
  - **崩溃后 marker 丢失（只在内存里）**：新实例挂载之后，最简单也最安全的取法是把 marker 初始化为「本次实例第一条记录的 `counter`」（即新实例暖机那一刻，见 `transaction.rs` 的 `warm_up_after_journal_counter`），等价于「新实例从 0 开始重新计一个 checkpoint 间隔的占用」。这是**保守方向**：不会低估占用导致该触发的时候没触发（因为新实例的环占用天然从 0 起算，不可能比真实值更大导致误报，但也不可能比真实值更小导致漏报——因为真实值本来就该从新实例的第一条记录开始算），不产生安全问题。
  - **要更精细（跨实例延续同一个「占用尚未 checkpoint」的计数）**：需要把 marker 落盘（例如塞进系统配置槽，与 `journal_tail` 同一批写），但这不是 K1 要回答的必答题——候选乙的题面本身只要求「量得出来」，没有要求「跨崩溃精确延续」。

**什么现象会推翻这一条**：如果日后有决策要求这个 marker 必须跨崩溃精确延续（不许退化成「新实例重新起算」），那就要另外论证「只放内存」是否还够；但即便如此，落法也只是多写 8 字节进系统配置槽（`transaction.rs` 已经在同一步写 `journal_tail`），代价形态不变，仍然是 O(1)，不改变 Q2/Q3 的判定。

**镜像重复计数的坑（Q2 里提过，这里给出处）**：如果实现时图省事直接复用 `write_accounting.rs` 的 `WrittenStructureKind::JournalRecord` 累计字节数，要注意它是**按设备计的**（`write_accounting.rs:5-6`「镜像的两份各算一次」+ `transaction.rs:142-148` 的 `for (_, device) in self.devices.iter_mut()` 循环），N 块镜像盘会把同一条逻辑记录的字节数算 N 次。用它直接判「环占用 ≥ 环长 ÷ F」会把占用夸大 N 倍，在多盘镜像池上提前触发。**避坑做法**：走 `journal.rs` 的 `counter`（每实例内单调递增、与设备数无关）做算术，不借道 `write_accounting.rs` 的字节计数。

## 留给 K4 那条腿的一个观测（不判、只记录——K4 不归我攻）

追查 Q3 的时候顺带读到一处与「checkpoint 间隔」这个词本身有关的事实，写下来但不下判断，因为它属于 K4「甲今天真的不成立吗」的射程（背景材料 52 行），不是我这条腿该判的格：

`.claude/kb/decisions/16-发布语义.md:147`（已定项 6，整段抄）：
```
**定案**：每次发布把 checkpoint_txg 加一——fsync 触发的发布也是发布，没有「小发布不记号」的例外；根槽的轮转键（D22（单元原子性怎么合成） 已定项 2 的「区域 = txg mod R」里那个 txg）、记账的代（D5（快照 / 空间记账机制） 已定项 2）、恢复重放的水位（D23（journal 的角色与格式） 已定项 14）都是这同一个计数。不另设「发布代号」字段——它与 checkpoint_txg 是同一个东西。**四个等号**：fsync = 提前发布、区域 = txg mod R、一个 txg 一个号、记账代 = checkpoint 号，四处读的是同一个数，哪个实现读岔一处就会撞根槽或读错账。
```
这条把「checkpoint」与「发布」等同（`.claude/kb/checks-owed.md` C245 那一行 2026-09-16 的回扫已经把这条用来收口「每个 checkpoint 一次」与「每次发布一次」是同一频率）。若真按字面读，「上次 checkpoint 的时刻」= 「上一次任意发布（含 fsync 触发的）的时刻」，那么候选乙的 marker（`counter` 差值）在每次 fsync 之后都会被重置为 0——这不影响 Q1–Q4 关于「算不算得出来」的判定（不管重置得多频繁，`current − marker` 都还是那一次减法），但会影响这个量**是否真的会长到 `环长 ÷ F`**这一件事——K4 判 K4 那一格。我只把这处出处交出去，不替 K4 下结论。

## 对题面第 4 点「最值钱的产出」的直接回答

题面要求的不是替候选乙找台阶，是给出确定答案。答案：**K1 这一面，第三支的量算得出来，不撞第一格的禁令。**

依据收在一句话里：这个代码库今天已经在用「事务提交时增量维护、`.since()` 求两次快照之差」的机制给 journal 记录按字节记账（`write_accounting.rs` + `transaction.rs:142-148`），并且 journal 记录自带一个与设备数无关的纯算术序号（`journal.rs` 的 `counter`）。候选乙第三支要的量在结构上与「一次发布的写字节数」是同一类量（`当前累计 − 某个更早的快照`），二者的**计算模式完全相同**，只是快照的取法时点不同（一个是「发布开始前」，一个是「上一次 checkpoint 完成时」）。这不是「乙可以怎么改造得能算」，是「乙要的这类量，这个代码库里今天已经有一个正在跑、有单元测试盯着的同构实现」——候选乙没有在这一面撞上任何需要新发明机制才能迈过去的门槛。

真正没有解决、需要交给别的腿或主 agent 的是**这个量的精确边界定义**（「间隔」从哪个事件算起到哪个事件为止），这属于 K4（甲今天成不成立、checkpoint 与发布是不是同一个事件）与 K3（三支会不会打架）的射程，K1 判的是「不管边界怎么划，这类量算不算得出来」，答案是「算得出来」。

## 判定一览（与开头一致，收尾复述）

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1：今天有没有实现 | 一致 | grep 零命中触发逻辑，只有字段声明 |
| Q2：量本身算不算得出来 | 算得出来，O(1)，不遍历 | `write_accounting.rs` 的按结构种类记账 + `since()`，以及 `journal.rs` 的 `counter` 算术，都是现成的同构证据 |
| Q3：落在三格哪一格 | 第一格（运行时决策路径） | 按「被问到那一刻有没有答案」判，答案「有」；checker 的整环遍历（`walk.rs:1705-1736`）是完全独立的另一条代码路径，不共用 |
| Q4：维护代价与放哪 | 一个标量 marker，内存即可；崩溃后按「新实例重新起算」即安全 | 与今天已经在做的 `journal_tail: plan.counter` 同一时机、同一写法；避免直接借用镜像计数（会被设备数放大） |
| 反过来想（算不出来会怎样） | 不适用 | 它没有被第一格的禁令挡住，候选乙不因 K1 这一面出局 |

## 没做什么

- 不判 K2（第三支自己造出的坏情形）、K3（三支打不打架）、K4（甲今天真的不成立吗）——按分工只攻 K1。
- 没有去论证「checkpoint 间隔」精确边界该怎么划（留给 K4 的观测那一节已交出处，不下结论），这不是 K1「量得出来吗」这个问题本身要回答的。
- 没有实现候选乙、没有改 `crates/` 任何一行——本轮是设计轮，写范围只到这份报告与草稿目录，没有写代码。
- 没有跑门禁、没有编译（`cargo build`/`cargo test`），这一轮不要求；`grep` 与 `Read` 都是只读观测，未改动工作区。
- `/tmp/claude-1000/checkpoint-trigger-r1-sonnet/` 草稿目录本轮未使用（空着）：全部证据都能直接从主工作区只读取得，未建副本、未做任何写操作。
