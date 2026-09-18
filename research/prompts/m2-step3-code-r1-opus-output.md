# m2-step3-code-r1 云端攻方腿（Opus）报告

攻击面：X1、X2、X4、X6、X8。原仓只读，实验全在副本 `/tmp/claude-1000/m2-step3-code-r1-opus/copy/`（`rsync -a --exclude target --exclude .git`）上跑，下面凡标「副本」的数都出自那份副本，不进 kb。

## 复跑命令与 sha256

```
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ /tmp/m2-opus-recheck/copy/
cp research/prompts/m2-step3-code-r1-opus-model/attack_opus_r1.rs \
   /tmp/m2-opus-recheck/copy/crates/singlefs-harness/tests/attack_opus_r1.rs
cd /tmp/m2-opus-recheck/copy && nice -n 19 cargo test -p singlefs-harness --test attack_opus_r1 -- --nocapture
# X8 那两段按 step0-layer0-added-tests.rs.fragment 文件头的说明插进
# crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs 再跑：
cd /tmp/m2-opus-recheck/copy && nice -n 19 cargo test -p singlefs-harness \
   --test second_transaction_step_zero_layer0 -- --nocapture opus_attack
```

| 文件 | sha256 |
|---|---|
| `research/prompts/m2-step3-code-r1-opus-model/attack_opus_r1.rs` | `df37124f16589b021c5b40eb64a9eb95ce95557cde3ac07c392e9d16275d1a9e` |
| `research/prompts/m2-step3-code-r1-opus-model/step0-layer0-added-tests.rs.fragment` | `c6e413c84dd40c4b1f063a556796a8d08c6174ee61135b2857ce7a7b74f46bf5` |

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X1 前缀五条（原文是六条） | **打中** | 所选根自己那条记录读不出时 `expected_next = None`，代码把水位之上最小的那条无条件接上——断号即止在这一格整个失效；副本上造出一段历史，跳过 jsn 5 直接施加 jsn 6，读回的是第四版 |
| X2 重开重建 | **打中（弱，三条都不致命）** | ① 只 mkfs、没发过文件版本的池可写挂载报 `NoPublishedVersion`（副本实测）；② 释放判定路径拿从盘上读回来的指针做 `assert_eq!` 而不是报错；③ `own_record` 的回退可能取到别的实例的记录，今天不承重 |
| X4 写行的 T 与 W | **打中** | 与 X1 同一段历史：写出的行是 `(1, 6, 4)`，而被施加的只有 jsn 6 一条，事务 3 从没被验过、也没被施加——`W 是精确的前缀` 不成立 |
| X6 跨实例的计数 | **没打中** | 试了五种形状（见「没打中的形状」）；另有一条判别力观测：jsn 全池接着数这条规则只被步 3 用例里一句 `assert_eq!(…counter, 5)` 钉着，到 C 的层 0 对它全绿 |
| X8 层 0 到 C | **没打中** | 五个对照全部搬到到 C 的脚本上、逐个在同一格红（副本实测）；根槽偏移 24 / 28 读对了；另有两条判别力观测 |

## X1　前缀口径：链首锚点丢了就不再断号即止

### 打中的历史（副本可复跑）

`research/prompts/m2-step3-code-r1-opus-model/attack_opus_r1.rs` 的
`x1_gap_is_skipped_when_the_chosen_roots_own_record_is_also_unreadable`：

1. `build_pool` 之后同一个实例连发三版：txg 4（jsn 4，事务 2）、txg 5（jsn 5，事务 3）、txg 6（jsn 6，事务 4）。
2. 改坏 txg 5 与 txg 6 的根槽各一字节 ⇒ 所选根退到 (1, 4)。
3. 改坏 **jsn 4 与 jsn 5 两条记录、每条两份镜像都改**。

副本原样输出：

```
X1-A effective_root = Some((InstanceGeneration(1), CheckpointTxg(6))) prefix_applied = 1 above_water = 1
X1-A 读回的是 root = (InstanceGeneration(1), CheckpointTxg(4))，内容长度 3300（second=4100 third=2500 fourth=3300）
```

`JournalScanReport { valid_records: 4, above_water: 1, prefix_applied: 1, verification_passed: 1, verification_failed: 0, maximum_applied_transaction: 4 }`。
jsn 5 一条没有被施加、也没有被读到，jsn 6 被直接接在所选根后面。

### 每一步指到许可它的那一句

- `crates/singlefs-core/src/recovery.rs:630-637`：`root_own_record_counter` 用 `find` 找同实例同 txg 那条；找不到时 `expected_next` 是 `None`。
- `crates/singlefs-core/src/recovery.rs:638-643`：`if let Some(expected_key) = expected_next { if (record.instance, record.counter) != expected_key { break; } }`——`expected_next` 为 `None` 时这一段整个不执行，第一条候选无条件进入。
- 代码自己给的理由在 `crates/singlefs-core/src/recovery.rs:628-629`：「那条记录读不出（两份都撕了）时不知道它的 jsn，就从水位之上同实例最小的那条接——水位之下的记录都已经在根里，读不读得出不影响链」。这句话只覆盖「丢的是水位**之下**那条」，我打的是「水位**之上**也丢了一条」。
- 被违反的条款，`.claude/kb/decisions/23-journal的角色与格式.md:1221-1222` 整行抄：

  > **前缀判定的完整口径是六条，缺一不可**（引用 I-8.3（重放前缀严格连续）时连这句一起引；第六条 2026-09-13 随 D16（发布语义） 已定项 4 用户定案加）：
  > jsn 严格连续（断号即止）、**`(实例代号, checkpoint_txg)` 大于根的水位**、

  以及同文件 `:1236` 整行抄：

  > 1. **前缀规则不跨实例边界**：链从所选根覆盖的最后一条记录之后接，下一条的实例代号与所选根不同即停（六条口径第一条原样）。所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加。代价写在 D16（发布语义） 已定项 7 的注，要不要让新实例先暖机是 D16（发布语义） 已定项 8。

- ⚠️ 背景材料正文与判据表把它写成「前缀五条」，kb 原文逐字是「六条，缺一不可」（第六条：施加的单位是一次发布）。第六条今天空转——一次发布只写一条记录——但材料把条数写少了一条。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分。仓里已经有一条变异「步 3：链首不接在所选根自己那条记录之后」，把 `expected_next` 强行改成 `None`，由 `one_missing_record_right_after_the_chosen_root_stops_the_prefix_even_when_later_records_are_intact` 判红（`crates/mutations.tsv` 倒数第 4 行）。我打的是**同一个行为由数据触发而不是由代码触发**：不改一个字节的源码，只让锚点那条记录读不出，代码就自己走进了那条变异的行为。所以它分辨的是「有锚点」与「没锚点」两条实现路径，不是两条设计臂 |
| 被判的系统当时看不看得到判别它的东西 | **看得到，但看到的不是 jsn**。系统看得见「所选根自己那条记录找不到」这件事（`root_own_record_counter` 就是 `None`），保守读法（没有锚点就一条都不施加）当场可判。它看不见的是「jsn 5 到底存不存在过」——环里那个槽解不开，「从没写过」与「撕了」逐字节相同。所以这一格不是 `evidence-discipline.md`「判别子观测不到」那一类：**要判的那个谓词（有没有锚点）是可观测的，代码选了乐观的一边** |
| 满足的是判据字面的哪一个分句 | 背景材料第三节 X1 那一行的触发观测逐字是「一段历史让恢复施加了不该施加的记录，或漏施加该施加的」——中的是前半句 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 反向接受条款给的是「改代码、补一条会红的用例与一条变异行、再攻一轮」。**最直接的那个改法（没锚点就一条都不施加）在这一格不中了，但它把仓里一条已有的验收用例打红**：副本上把 `recovery.rs:636-637`（`expected_next` 那两行）改成「`root_own_record_counter` 为 `None` 就直接返回」，`x1_gap` 变成 `prefix_applied = 0` / 读回第二版（对），而 `crates/singlefs-harness/tests/first_transaction_step_six_recovery.rs:257` 的 `torn_journal_record_truncates_the_prefix_but_recovery_still_completes` 当场 FAILED（副本，`cargo test` 原样：`test result: FAILED. 3 passed; 1 failed`）。所以这不是一处笔误，是一个要定的取法 |

### 那条已有用例为什么会被打红——两种情形的分界线在哪

`first_transaction_step_six_recovery.rs:255-292`（用例本体从 :257 起）那条用例干的事：改坏 jsn 2 两份 + 改坏最新根槽 ⇒ 所选根退到 (1, 2)，它自己那条记录正是 jsn 2、读不出；水位之上最小的是 jsn 3，**而 jsn 3 正好就是紧接着的那一条**，于是「从最小的那条接」这次是对的。我的历史里，水位之上最小的是 jsn 6，中间少了 jsn 5。

两种情形系统当时能不能分开：**能**，但要多用一个量。
两边都有「锚点读不出」，差别是最后一条「读得出、且 (实例, txg) ≤ 水位」的记录与所选根之间差几代：

| | 最后一条 ≤ 水位的可读记录 | 所选根 txg | 水位之上最小的那条 jsn |
|---|---|---|---|
| 仓里那条用例 | jsn 1（txg 1） | 2 | 3 |
| 我这一段历史 | jsn 3（txg 3） | 4 | 6 |

按「一次发布一条记录、txg 每次加一」算，上界 = 那条记录的 jsn + (根 txg − 那条记录的 txg) + 1：用例里是 1 + 1 + 1 = 3，3 ≤ 3 放行；我这里是 3 + 1 + 1 = 5，6 > 5 拦下。
⚠️ **这个改法只在我自己的模型上量过、被攻过零轮**，而且它的前提（jsn 的增量不小于 txg 的增量）只有在「一次发布一条记录」时成立——`.claude/kb/decisions/23-journal的角色与格式.md:1221` 第六条明写前缀可以停在「一次发布的两个事务之间」，那就是一次发布多条记录的形态，到那时这个上界不再是上界。所以我不主张采纳它，只用它证明**这两种情形是可分的**，「没锚点就只能乐观」这个说法不成立。

第二条改法（同样被攻过零轮）：把所选根自己那条记录的 jsn 写进根记录。那是格式改动，`.claude/rules/format-evolution.md` 那一套要走。

### 这一格今天没有任何东西盯着

`.claude/kb/invariants.md:75` 的 I-8.3（重放前缀严格连续） 状态列逐字是「未实现」——checker 不判前缀连续性。到 C 的层 0 也判不了：整写子集模型下「根在而它的记录不在」不可达（记录那一段在根那一段之前，前面的段整段持久）。仓里唯一碰这一格的，就是被我这段历史绕过去的那条用例。

## X4　写行的 T 与 W：W 报了一个不是前缀的集合

### 打中的历史

同一段历史上做可写挂载（`attack_opus_r1.rs` 的 `x1b_mount_after_the_gap_writes_a_row_that_is_not_a_prefix`）。副本原样输出：

```
X1-B chosen=(InstanceGeneration(1), CheckpointTxg(4)) effective=(InstanceGeneration(1), CheckpointTxg(6)) prefix_applied=1 W=4 rows=[InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(6), applied_transaction_high_water: 4, is_rollback: false }]
```

写出的行是 (1, 6, 4)。被施加的记录只有 jsn 6 一条（事务 4）。**事务 3（jsn 5、txg 5）既没被读到、也没被验证、更没被施加**，而 W = 4 把它罩在里面了。

### 指到条款

`.claude/kb/decisions/18-块里携带什么信息.md:882`（那一行很长，下面是其中两句，逐字抄自该行）：

> W 是精确的前缀：事务号在实例内单调、记录按事务号顺序追加（第一版串行提交下恒成立），一个事务在发出单元写之后失败即实例切换、号更大的记录一条都不追加（D23（journal 的角色与格式） 已定项 7 的注）。

> **已发布谓词（全局）**：i_now = 挂载根的实例，单元写序 (i, n)、诞生代号 b——i < i_now 且无行 ⇒ 已发布；有行 (i, T_pub, W)：码 1 ⇒ b ≤ T_pub ∨ n ≤ W

代码把 W 定义成「这次施加的记录里最大的事务号」（`crates/singlefs-core/src/recovery.rs:669-670` 的 `report.maximum_applied_transaction = report.maximum_applied_transaction.max(record.transaction)`，`crates/singlefs-core/src/mount.rs:253` 的 `applied_transaction_high_water: journal.maximum_applied_transaction`）。**被施加的集合是前缀时这两个定义等价；不是前缀时 max 会把没施加的那几个号一起罩进去**，而已发布谓词读的是 `n ≤ W`。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分。W 的两个候选定义（「施加过的最大事务号」与「属于该实例的、被施加的那个前缀的上界」）在 X1 那一格之外处处同值，正是这一格把它们劈开 |
| 被判的系统当时看不看得到判别它的东西 | 看得到：`JournalScanReport` 里已经有 `prefix_applied` 与 `above_water`，代码自己知道这次只施加了 1 条而候选 1 条、链首没有锚。要判「这个集合是不是前缀」不需要新输入 |
| 满足的是判据字面的哪一个分句 | X4 那一行的触发观测逐字是「一段历史让下一次恢复按这一行做错（施加过头或停早了）」——这一格是**施加过头的那一半的下游**：行把没施加的事务 3 报成已施加，下一次恢复（或扫描重建）按 `n ≤ W` 会把事务 3 的码 1 单元判成已发布 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 反向接受条款的第一条（改代码 + 会红的用例 + 变异行）把 X1 修好之后，这一格自动不中（施加集合恢复成前缀，max 与前缀上界重新同值）。⇒ **这一格不用单独改法，它是 X1 的下游**；如果 X1 取的是「格式里带上锚点 jsn」那条路，同样自动不中。反过来，只改 W 的算法而不修 X1（例如 W 取「连续施加段的最后一个事务号」），在这一格上**还中**：施加集合仍然不是前缀，只是 W 变成 4 之外的另一个数，行仍然在说一件假话 |

### 另外两条，都不是打中，是核完的结果

1. **T 取「重放之后那个根的 checkpoint_txg」是对的。** `crates/singlefs-core/src/mount.rs:252` 写 `selected_root_txg: effective_root.checkpoint_txg`，`.claude/kb/decisions/18-块里携带什么信息.md:882` 逐字是「所选根那个实例写 (i, **重放之后那个根的 checkpoint_txg**, **属于实例 i 的、被这次重放施加的最大事务号**)」。⚠️ 同一行里 kind 0 行记录的字段表逐字是「所选根的 checkpoint_txg 8」——**字段名与写行规则说的不是同一个量**，条款自己里外不一，代码跟的是更具体的那一句。这是条款的问题，不是代码的问题，但字段名照字面读会让下一个实现者写成 `chosen_root.checkpoint_txg`。
2. **中间实例 (i, 0, 0) 与「实例 0 不写」都对得上。** `crates/singlefs-core/src/mount.rs:247` 的 `effective_root.instance.0.max(1)`；重放不跨实例 ⇒ `effective_root.instance == chosen_root.instance`，与条款里的「max(所选根的实例, 1)」同值。写行那次发布事务号 0 与 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 19 ① 的「事务号 0 保留给不承载事务的记录」相符，且 `max(x, 0) = x` 让它不进 W 的 max（`crates/singlefs-core/src/recovery.rs:669-670`）。

## X2　重开重建：三条，都不致命

### ① 只 mkfs、没发过文件版本的池，可写挂载开不了（副本实测）

`attack_opus_r1.rs` 的 `x2_fresh_pool_cannot_be_mounted_writable`，副本原样输出：`X2-A NoPublishedVersion`。
路径：`crates/singlefs-core/src/recovery.rs:347-352`（树表 0 条 ⇒ `InvariantViolated { invariant: "挂载" }`）→ `crates/singlefs-core/src/mount.rs:196-200` 翻成 `MountError::NoPublishedVersion`。

**与 D16（发布语义） 已定项 8 冲不冲突：不冲突，但留了一个今天走不通的岔口。** `.claude/kb/decisions/16-发布语义.md:192` 逐字写着「第一次可写挂载的暖机时树表 0 条 ⇒ 零单元」——条款明确认得「树表 0 条」这种池，而 `mount_writable` 对它一律拒开。仓里真正走这条路的是另一个函数 `crates/singlefs-core/src/transaction.rs:321` 的 `warm_up`，它自己拼 `JournalRecord`、自己调 `pool.perform(CommitStep::…)`，**不经过 `publish_version`**。于是「空发布」在这个二进制里有两份实现：mkfs 那次零单元，挂载那次 c_max 块。两份按 D16（发布语义） 已定项 9 各自都对，所以不是条款违反；但它是 `.claude/rules/fs-design.md`「一个事务层，所有结构共用」那条要问的形态（「绕开状态机的第二条发布路径不允许」）。
⚠️ 这一条踩到 S6 / X5 的对象（暖机），而 X5 是本地攻方的格。我只报「有两条发布路径」这个事实，不判 X5 的三个问题（次数、上限、覆盖怎么算）。

今天走不通的是这一格：**第一次可写挂载做完、一个文件都没发就关掉，再开就只能走 `warm_up` 那条路**，而 `warm_up` 的入参是 `genesis: &RootRecord`（mkfs 的创世根），拿不到「上一次挂载暖机之后那个根」。`crates/` 里没有第二个入口（`grep -rn "fn mount" crates/singlefs-core/src/` 只有 `mount_writable`）。里程碑步 3 的现状段说「刚 mkfs 的池走第一次可写挂载那条路」，那条路在代码里是 `build_pool` 测试辅助拼出来的，不是一个函数。

### ② 释放判定路径拿盘上读回来的指针做断言，不是报错

`crates/singlefs-core/src/transaction.rs:607-610`：

```rust
        assert_eq!(
            locations[0].slot, locations[1].slot,
            "两盘同槽（D2（RAID 条带策略） 已定项 10）"
        );
```

`previous` 在重开这条路上**整个是从盘上读回来的**（`rebuild_version`），它的两条位置条目来自树表 / 映射节点里的指针。`read_unit_via_locations`（`crates/singlefs-core/src/recovery.rs:199-210`）逐条试、任一条校验和对上就算读到——所以一个「位置条目 0 指错、位置条目 1 指对」的单元走读能过，而 `placements_to_release_via_mapping` 会在这一行 panic。同一函数里另外三种同类错（不在册、已释放、跨度不符）都是 `Result`，只有这一条是断言。
**没跑成**：我没造出这个镜像。改一条位置条目要连带重算宿主单元的整单元校验和、再重算它父指针里的那一份，副本上没做完。所以这一条是读代码读出来的，不是实测，reachability 只到「蓄意改字节」这一档——不是崩溃状态。按 `code-discipline.md`「边界上验一次，换成带证明的类型」，重建出来的上一版是不可信输入，这一处该是 `Result`；但它今天不会被崩溃点重放碰到。

### ③ `own_record` 的回退可能取到别的实例的记录

`crates/singlefs-core/src/mount.rs:185-193`：找不到生效根自己那条记录时，`.or_else(|| records.values().max_by_key(|record| record.counter))` 取**全环 jsn 最大**的那条，不管实例、不管 txg，塞进 `previous.record` 与 `previous.record_bytes`。
今天不承重：读这两个字段的只有 `publish_overwrite`（`crates/singlefs-core/src/transaction.rs:866-869` 的 `counter` / `transaction` / `back_chain`），而 `mount_writable` 交出去的 `current` 是一次真发布的输出，调用方拿的是它。⇒ latent，一旦有人把 `rebuild_version` 的产物直接喂给 `publish_overwrite`（比如只读挂载之后转可写、或者切换路径），事务号与反向链会接到别的实例那条记录上。我没造出今天可达的历史。

### X2 的四句（按 ① 那一条答，②③ 没跑成不算打中）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 不分辨任何设计臂。它分辨的是「第一版可写挂载支持哪些池」这个射程，而射程今天没有条款写死 |
| 被判的系统当时看不看得到判别它的东西 | 看得到（树表条数就在手里） |
| 满足的是判据字面的哪一个分句 | X2 那一行的触发观测逐字是「一份走读能过的镜像重开时 panic 或写出错的东西」——**这一条没满足**：它既不 panic 也不写错东西，是干净地报错。所以我把它记成「弱」，不是打中那一档。②③ 同样没满足触发观测 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 反向接受条款里适用的是「打中的格若落在没有条款的地方 ⇒ 记进 `02-second-txn.md` 步 3 的决策点、标预想交用户，代码按最保守的读法改」。最保守的读法就是现在这样（拒开），所以**代码不用动**，要动的是里程碑那份文件上的一行现状：写明第一版可写挂载只接在「已经发过文件版本」的池后面，没发过的池今天只有 `build_pool` 那条测试路径 |

## X6　跨实例的计数：没打中

`crates/` 今天的取法（现读）：`crates/singlefs-core/src/mount.rs:212-217` 的 `next_counter` = 全环全部可读记录的 counter 最大值 + 1（不分实例）；`crates/singlefs-core/src/mount.rs:273-274` 写行那次 `transaction: 0`、`back_chain: 0`；暖机 `crates/singlefs-core/src/mount.rs:308` 用 `back_chain_of(&current.record_bytes)`；发布 C 用 `previous.record.transaction + 1`（`crates/singlefs-core/src/transaction.rs:867`）。

试过的形状与取样范围（都没打中）：

1. **残留 jsn 冒充**：让实例 1 留一条读得出的记录，看实例 2 能不能算出与它相同的 jsn。算不出——`next_counter` 取的是全环最大值 + 1，只要那条记录读得出它就在 max 里。
2. **残留 jsn 撕了之后被复用**：让实例 1 的 jsn 5 两份都读不出 ⇒ 实例 2 算出 5、写到同一个环槽上。两条记录的键是 (实例, counter)，撕掉那条根本不在 `scan_journal` 的表里（`crates/singlefs-core/src/recovery.rs:588-592`），冒充不成立。
3. **一份镜像留旧记录**：实例 2 的新记录只在盘 0 落盘，盘 1 那个槽还是实例 1 的旧记录。两条键不同、同时在表里，但旧那条的 checkpoint_txg 一定不大于水位（同实例内 jsn 与 txg 同向），进不了 `above`。
4. **环绕圈**：环槽数取小（环长是 mkfs 参数）让 jsn J 与 J + 槽数 落同一槽。绕圈留下的旧记录 txg 更小，同样进不了 `above`；所选根自己那条被绕圈盖掉时走的是 X1 那条路，已在 X1 里报。
5. **反向链互相接上**：`crates/singlefs-core/src/journal.rs:170` 注释逐字「不查反向链（那是前缀取法的事）」，而 `replay_journal` 也不查——所以链接错了没有任何判定会说话。这与 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 19 ② 的依据句一致（「那条链没有任何判定在用」），不构成打中。

### 判别力观测（不是打中）

副本上把 `crates/singlefs-core/src/mount.rs:212-217` 的 `next_counter` 改成写死 `1`（新实例的 jsn 从 1 重新数，正对着 `.claude/kb/decisions/23-journal的角色与格式.md` 已定项 14 第 3 条「计数器全池接着走」），跑 `cargo test -p singlefs-harness` 的结果：只有 `second_transaction_step_three_second_instance` 的两条用例红（`:132` 与 `:404`，都是 `assert_eq!(…counter, …)` 那种等值断言）；**到 C 的层 0（`second_transaction_step_zero_layer0`）全绿**，池级 checker 与记录核对器一句话都没有。
⇒ jsn 全池接着数这条规则今天只被两句等值断言钉着。这个变异下实例 2 的记录会落回实例 1 记录占着的环槽上，而崩溃状态那一层看不见。

## X8　层 0 到 C：没打中

### ① 五个对照全部搬到到 C 的脚本上，逐格在同一格红（副本实测）

`step0-layer0-added-tests.rs.fragment` 的 `opus_attack_controls_moved_to_the_third_publish_script`，副本原样输出：

```
C 的单元写 16 条、记录写 2 条
① 全持久：effective=Some((InstanceGeneration(2), CheckpointTxg(8))) violations=0 outcome_is_file=true
② 根持久 + C 的单元一份都不在：violations=1 ignored=1 claimed_missing_unit=1 first=Some("走读失败：UnitUnreadable { slot: SlotNumber(50326) }（持久的写：…）")
③ C 的根槽不在、记录与单元都在：effective=Some((InstanceGeneration(2), CheckpointTxg(8))) prefix_applied=1 journal_differing=1 violations=0
④ 根持久 + C 的两份记录都不在：violations=0 root_without_record=1
⑤ C 一个字节都不在：effective=Some((InstanceGeneration(2), CheckpointTxg(7))) violations=0 outcome=true
```

与到 B 那份对照（`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:322-395`）逐格同形：② 红在 oracle 与记录核对器、③ journal 承重、④ 只有记录核对器红、①⑤ 零违例。**「对照仍在到 B 的脚本上」这一条我没打中**：搬过去照样红。

### ② 根槽字节偏移 24 / 28 读对了

`crates/singlefs-core/src/root_record.rs:12-13` 的字段表逐字：「magic 4 + fsid 16 + flags 4 + 实例代号 4 + checkpoint_txg 8 + 树表指针 86 + 树 ID 水位 8 + 回退下界 F 8 = 138」⇒ 实例代号在 [24, 28)、checkpoint_txg 在 [28, 36)，与 `crates/singlefs-harness/src/crash.rs:639-649` 的 `write.bytes[24..28]` / `write.bytes[28..36]` 一致，字节序都是小端（`to_slot` 用 `put_u32` / `put_u64`，读用 `from_le_bytes`）。
判别力：副本上把 `24..28` 改成 `20..24`（flags 恒 0 ⇒ 每次发布的实例都读成 0），`first_transaction_step_seven_layer0` 三条用例红；**`second_transaction_step_zero_layer0` 的两条非 ignored 用例全绿**（副本原样：`test result: ok. 2 passed; 0 failed; 1 ignored`）。⇒ 这个偏移是被第一个事务那套（单实例）的用例钉着的，到 C 这条**两实例**的流对它一个字都不说——而分辨 (1,4) / (2,5) / (2,6) / (2,7) 正是它存在的理由。变异表 `crates/mutations.tsv` 里也没有这一条（那里的三条 oracle 变异打的是比较式与找版本的谓词，不是读字节的偏移）。

### ③ oracle 本身的射程：写行那次发布五个角色里有三个它看不见

`step0-layer0-added-tests.rs.fragment` 的 `opus_attack_instance_table_unit_missing_but_every_root_persisted`：把写行那次发布（txg 5）的 10 个单元写按角色分成五对，逐对让两份都不落盘、别的全落盘。副本原样输出：

```
写行发布的单元写下标 [56, 57, 58, 59, 60, 61, 62, 63, 64, 65]
X8-B 第 0 对单元写缺席：outcome_is_file=false violations=1 ignored=1 claimed_missing=1 checker=[("I-2.1", 1), ("I-7.2", 1)]
X8-B 第 1 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1)]
X8-B 第 2 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1)]
X8-B 第 3 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1)]
X8-B 第 4 对单元写缺席：outcome_is_file=true violations=0 ignored=0 claimed_missing=1 checker=[("I-2.1", 1), ("I-3.1", 1)]
```

第 1 / 2 / 3 对：**oracle 零违例、文件照读**，只有池级 checker（I-2.1）与记录核对器出声。这不算 oracle 判错——最终根是 C 的 (2, 8)，它早把那几个角色重写过了，读路径确实不碰那几个单元；oracle 的合同本来就只管「实际走的根该读出哪一版」。我把它记成**射程**：到 C 的脚本上，写行那次发布五个角色里有三个完全靠 checker 与记录核对器兜着，oracle 不是它们的第二道。
⚠️ 反过来的一面：这五个状态里记录核对器**每一格都判 `claimed_missing = 1`**，包括那三个终态完全正常的。`crates/singlefs-harness/src/crash.rs:484` 的判据是 `effective_root.txg >= publish.checkpoint_txg` 就要求那次发布的每个单元原样在盘上，没有问「最终那个根还引不引用它」。这五个状态在今天的枚举里都不可达（单元那一段在记录、根之前，前面的段整段持久），所以 `assert_eq!((tally.record_root_without_record, tally.record_claimed_state_missing_unit), (0, 0))`（`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:167-173`）照样绿。**步 0 一旦把枚举放宽（比如允许两次发布各缺一部分），这条断言会先在这几格上红，而红的不是实现。** 这条我没打中 X8 的触发观测（「一个状态该判违例而代码判绿」），记在这里是给步 0 往下走留一笔。

### ④ 重开接缝合成 4 写一段：核过，没打中

`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs:128-137` 的段序列 `[2,2,1,2,2,1,18,2,1,18,2,1,4,10,2,1,10,2,1,10,2,1,18,2,1,2]` 里那个 `4`，是 B 的两个超级块槽写加取号的两个超级块槽写。把进程退出当成**不是**屏障 ⇒ 这四写落一段 ⇒ 枚举给它们任意子集（15 个状态），比「退出算屏障」多出来一批状态。多判不是假阴性，方向是保守的那一边，所以没打中。取号自己那道屏障在 `crates/singlefs-core/src/transaction.rs:305` （两次超级块写之后），与段序列对得上。

## 没打中的形状（除上面 X6 五种、X8 四条之外）

| 试过什么 | 取样范围 | 结果 |
|---|---|---|
| 施加一条记录会不会丢掉实例表指针 | 读 `crates/singlefs-core/src/journal.rs:80-94` 的 `JournalRecord` 字段表（没有实例表那一项）与 `crates/singlefs-core/src/recovery.rs:678` 的 `instance_table: rebuilt.instance_table` | 今天不可达：会重写实例表的只有每个实例的第一次发布（`InstanceTablePlan::Rewrite` 只在 `crates/singlefs-core/src/mount.rs:277` 出现一处），而那条记录的实例代号严格大于任何已有的根 ⇒ 永远不是重放的候选。回退（步 4）一进来这一条要重看 |
| 实例表写出重复行 | 手推四段历史：连续两次可写挂载、取号后崩、实例 2 的根全坏退回实例 1、实例 2 早期根坏而晚期根在 | 都推不出重复行。`crates/singlefs-core/src/mount.rs:265` 的 `table_after.rows.push(row)` 不去重、不覆盖，而条款 `.claude/kb/decisions/18-块里携带什么信息.md:882` 逐字是「按实例代号唯一、后写覆盖」——今天推不出可达历史是因为「行 (i, …) 只写进实例 i+1 及以后的根引用的表版本里」。回退那条路（同一行逐字：回退行「**不许被后来的恢复覆盖**」）一进来就可达，而 I-3.8 会判红 |
| 空发布中途插进来让事务号重号 | 读 `crates/singlefs-core/src/transaction.rs:867` 的 `previous.record.transaction + 1` | 今天不可达（空发布只出现在一个实例的开头）。真实例事务号从 `FIRST_TRANSACTION_NUMBER = 1` 起（`crates/singlefs-core/src/transaction.rs:59`），与「事务号 0 保留给不承载事务的记录」不撞 |
| 前缀第六条（施加的单位是一次发布） | 读 `.claude/kb/decisions/23-journal的角色与格式.md:1221` 与 `crates/singlefs-core/src/transaction.rs` 的发布路径 | 今天空转：一次发布恰好写一条记录，前缀不可能停在一次发布中间。条款在，检查没有对象 |
| 在飞上限按实例还是全局 | 读 `crates/singlefs-core/src/recovery.rs:626-627`、`:638` 的 `above.into_iter().take(in_flight_limit)` | 候选表在 `take` 之前已经按实例滤过（`:621`），所以是按实例计；`take` 只会让施加得更少，不会让施加得更多，方向保守 ⇒ 没打中 |
| 所选根是 mkfs 第 0 代根时一条都不施加 | 直接读 `crates/singlefs-core/src/recovery.rs:618-623` | 成立：实例 0 与任何记录的实例都不相等，`above` 为空 |
| 回退行那第五条今天怎么表达「没有输入」 | `grep -rn "is_rollback" crates/` | 只有 `crates/singlefs-core/src/mount.rs:32/47/70-77`（解析与写 0）与步 3 用例三处写 `is_rollback: false`；`replay_journal` 一处都不读，**checker 也一处都不读**（`crates/singlefs-checker/src/walk.rs:415-430` 只看 kind 与实例代号，flags 那一字节根本不取） ⇒ 代码把「没有回退行」表达成「根本不查实例表」，与条款第五条（该实例的记录只施加到回退行的 W 为止）在今天同值（表里恒无回退行）。步 4 写出第一条回退行的那一刻，`replay_journal` 就少一条口径 —— 这是已经登记的 C124（回退行与重放下界没有会红的检查），不是新发现 |
| 实例表行的 flags 字节有没有人核 | `grep -rn "is_rollback" crates/`（5 处，全在 `mount.rs` 与步 3 用例）＋读 `crates/singlefs-checker/src/walk.rs:410-441` | 写者侧 `InstanceRow::parse`（`crates/singlefs-core/src/mount.rs:70-72`）对 bit0 之外的位非 0 拒收；checker 的行解析器根本不取那一字节 ⇒ 一份 flags 带杂位的镜像 checker 判绿、下一次可写挂载的 `InstanceTableRecords::parse` 返回 `None` ⇒ `MountError::InstanceTableMalformed`（开不了）。**这一格是 X7（I-3.8 射程）的，归本地攻方**，我只记事实、不判 |

## 这条腿自己的限度

1. **X1 与 X4 那一段历史的故障模型是「两份镜像都改坏一字节」，不是整写子集。** `.claude/kb/decisions/13-验证路线.md` 已定项 4 的崩溃状态集合罩不到它：记录那一段在根那一段之前，前面的段整段持久 ⇒ 「根在而它的记录不在」在层 0 里不可达。我之所以仍然把它记成打中，理由是**同一个故障模型、同一个仓、同一张测试文件里已经有一条验收用例在用它**（`crates/singlefs-harness/tests/second_transaction_step_three_second_instance.rs:469-501` 与 `first_transaction_step_six_recovery.rs:257-292` 都是 `flip_byte` 两份镜像），而且被打中的那条代码（`recovery.rs:628-637`）的注释本身就写着「两份都撕了」——它是为这个模型写的。如果主 agent 认为这个模型不在这一轮的射程内，X1 / X4 两格应当整格作废，理由要写下来，并同时说明那两条已有用例凭什么还算数。
2. **X2 的 ② ③ 两条没跑成**，是读代码读出来的。② 要造一份「一条位置条目指错、另一条指对、宿主单元校验和还对得上」的镜像，要连带重算两层校验和，我在副本上没做完；③ 今天没有调用方，我造不出可达历史。两条都只到「结构上可以发生」这一档。
3. **X8 全量那条（789555 个状态）我没跑。** 它标 `#[ignore]`、debug 下十几分钟，归门禁 54 号在 release 下跑。我跑的是 57 个状态那条快的、五个搬过来的对照、以及五组手摆的持久集合。所以我说的「没打中」只覆盖这些形状，不覆盖全量枚举。
4. **副本上量出的数一个都不进 kb。** 上面每条原样输出都来自 `/tmp/claude-1000/m2-step3-code-r1-opus/copy/`；要引用得在入库装置上重做一次（`.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）。副本上做过的三个变异（`recovery.rs` 的保守改法、`crash.rs` 的偏移 24→20、`mount.rs` 的 `next_counter = 1`）在量完之后都已还原成原仓那一份。
5. **我自己提的两个改法（X1 的「按 txg 差算 jsn 上界」与「根记录里带锚点 jsn」）只在我的模型上量过、被攻过零轮。** 前者的前提（jsn 增量不小于 txg 增量）在「一次发布多条记录」下就不成立，后者是格式改动。两个都不该按我写的样子直接采纳。
6. **由用户决定的动作我一个都没写死。** X2 ① 交出去的是「第一版可写挂载的射程要不要写进条款」，不是「应该支持还是不支持」；X1 交出去的是「锚点丢了取乐观还是保守，保守就要同时判 `first_transaction_step_six_recovery.rs:255` 那条用例的去留」，两边我都没替主 agent 选。
7. **不判正推腿与辩方腿那几格**（X9、X10，以及 S1–S11 的逐条对文），也不判 X3、X5、X7、X11。X2 ① 的第二段与 X6 那条判别力观测各自蹭到 S6 / X5 与 X7 的对象，已在原地标出。

## 没做什么

- 没跑 `gate.sh`，也没跑 `.claude/gate.d/stage-owners.tsv` 里的任何阶段：`awk -F'\t' -v me=three-way-attack '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv` 输出为空，表里没有登记给这条腿的阶段。
- 没改原仓任何已有文件，没做任何 git 写操作。原仓新增的只有这份报告与 `research/prompts/m2-step3-code-r1-opus-model/` 两个文件。
- 没跑 herd7、QEMU、层 0 全量、门禁 54 / 59 号。
- 没读这一轮别的腿的输出，也没读 `m2-step3-code-r1-sonnet.md` 与 `-local-*.md`。
