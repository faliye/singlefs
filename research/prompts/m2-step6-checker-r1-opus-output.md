# m2-step6-checker-r1 云端攻方腿（Opus）报告：Y1 / Y2 / Y5

轮名 m2-step6-checker-r1，立场：造可达的历史或镜像打穿 U1。分到 Y1、Y2、Y5；Y3 没碰。
副本 `/tmp/claude-1000/m2-step6-checker-r1-opus/copy/`（`rsync -a --exclude target --exclude .git`），
原仓一个已有文件都没改，没做任何 git 写操作。**下面每个数都是在副本上跑出来的，不进 kb。**

## 复跑命令与产物 sha256

```bash
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ /tmp/<你的目录>/copy/
cd /tmp/<你的目录>/copy
cp /home/fy5090/code/singlefs/research/prompts/m2-step6-checker-r1-opus-model/zz_attack_floor.rs crates/singlefs-harness/tests/
cp /home/fy5090/code/singlefs/research/prompts/m2-step6-checker-r1-opus-model/zz_attack_probe.rs crates/singlefs-harness/tests/
nice -n 19 cargo test -p singlefs-harness --test zz_attack_floor -- --nocapture
nice -n 19 cargo test -p singlefs-harness --test zz_attack_probe -- --ignored --nocapture
# 改法 P1 的对照：把 p1-effective-floor.patch 里的替换施加到 crates/singlefs-checker/src/walk.rs:793-797 再跑上面第一条
```

```
253a98a64a4b4f4c22d97e61abe131c9d8eacb4ce575971ced9ffee602b1cdb2  research/prompts/m2-step6-checker-r1-opus-model/p1-effective-floor.patch
a8e9a31cc6a15366f433a906ccfa7516436ebd5515161204cdbba0b5e6ac8ee5  research/prompts/m2-step6-checker-r1-opus-model/zz_attack_floor.rs
7970100915cba51db0e55fb97f6d39513bb5a9c2afdf631d30f13464e5ba0f02  research/prompts/m2-step6-checker-r1-opus-model/zz_attack_probe.rs
```

开跑前 `ps aux --sort=-%cpu | head -12` 看到 `second_transaction_step_zero_layer0-7921cf1fc5f129b1 --include-ignored`
占 100% CPU（层 0 全量在 release 下跑）与两个 `ray::RayWorkerProc.run` 各 27.8%；按派发提示所有 cargo 都加了 `nice -n 19`。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y1 复用漏判 | **打中** | checker 的候选集下界取「最新根自己带的 F」，而 I-7.4 / I-4.8 的定义句取 **F_生效**（各幸存盘所带 F 最大值的最小值）。只有一块盘落上带新 F 的根时两者不同：一段可达的历史里 mkfs 树表那一槽被新数据单元盖掉、退到那条根当场失败，而 I-7.4 / I-4.8 / I-2.1 **三条全判成立** |
| Y2 归错根 | **打中一半** | 报出来的只有**第一条**判违例的候选根，后面每条候选根各自那一格的红绿在 `into_report` 之后不可见——U1 声称的「按每条根各判一格」在产物里读不出来。另外 `walk_root` 在树表已访问时 `return`（walk.rs:232），那条根的中央映射树一格都不走。「I-4.8 最新根那一格与 I-7.2 永远同红同绿」**不成立**（仓里现成的 I-2.1 坏镜像形态就分得开），这一问没打中 |
| Y5 层 0 | **没打中误红，打中一格阴性结果** | 快用例 108 个状态上 I-7.4 与 I-4.8 各评估 108 个状态、违例 0，没有误红；但抬 F 那两次空发布之间的崩溃状态正是 Y1 那一格，checker 在那些状态上把 F 之下的根整批排除，所以「层 0 全绿」买不到对那批根的保护 |

## Y1 复用漏判：候选集下界取错了量

### 条款要的那个量

`.claude/kb/invariants.md:50`（I-7.4 那一行，整行抄的开头）：

> | I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。

`.claude/kb/invariants.md:153`（I-4.8 那一行，整行抄的开头）：

> | I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集（D23（journal 的角色与格式） 已定项 14：按实例表判仍然有效 ∧ txg ≥ F_生效）里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致

F_生效 的定义在 `.claude/kb/decisions/16-发布语义.md:375`（整行抄）：

> | 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |

### 代码取的那个量

`crates/singlefs-checker/src/walk.rs:793-797`：

```rust
    let newest_rollback_floor = u64::from_le_bytes(
        roots[newest_index].2.record_bytes[130..138]
            .try_into()
            .expect("8 字节"),
    );
```

`walk.rs:803`：`let below_floor = root.checkpoint_txg < newest_rollback_floor;`，
`walk.rs:804`：`if index != newest_index && !abandoned && !below_floor {`——只有过了这一关的根才进 I-7.4 / I-4.8 的每根一格。

实现自己另有一个算 F_生效 的函数，`crates/singlefs-core/src/recovery.rs:351` `pub fn effective_rollback_floor`：
它按 `target_for_publish(root.checkpoint_txg).region` 把每条可读根归到盘上、每盘取 F 的最大值、再取各盘的最小值。
checker 没有用这个口径，用的是最新那一条根记录里的 8 字节。

**两者的方向是单向的**：最新根住在某一块盘上，它的 txg 最大、F 也是那块盘上最大的，所以
`F_生效 = min_盘(max F) ≤ 最新根的 F`。⇒ checker 的候选集**恒是条款候选集的子集**，只会漏判、不会误红。

### 可达的历史（副本上跑出来的）

历史照 `crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs` 的固定脚本：
A、B、重开取号 2、写行、暖机两次、C、重开回退到 (1, 3)、D、暖机一次、四次覆盖写（txg 11–14）、抬 F 到 11。
抬 F 要两次空发布（txg 15、16）才让两块盘各有一条带 F = 11 的根。
接着**打掉第二条载体根那一槽**——`bytes[100] ^= 0xff`，与仓里
`second_transaction_step_five_reuse.rs` 的
`roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates` 用的是同一手，
对应层 0 枚举里「根槽写到一半 / 只落了一块盘」那一格。然后再发布一次 E。

跑出来的原样输出（副本，`cargo test -p singlefs-harness --test zz_attack_floor -- --nocapture`）：

```
F_生效 = CheckpointTxg(0)；最新根 txg CheckpointTxg(17) 自己带的 F = CheckpointTxg(11)
txg 低于最新根 F 的根：[0, 3, 6, 9, 1, 4, 7, 10, 2, 5, 8]（按 F_生效 它们都还是回退候选）
判违例的：["I-3.1"]
  I-2.1: Holds
  I-3.1: Violated("盘 0：记账的已分配 Some(983040)，遍历全部有效根得到 917504")
  I-4.8: Holds
  I-7.2: Holds
  I-7.4: Holds
退到 (1, 2) 的结果：Err(Recovery(UnitUnreadable { slot: SlotNumber(50178) }))
```

逐条对上：

- E 的数据单元落回槽 50178（测试里 `assert_eq!(reuse.data_pointer.locations[0].slot, SlotNumber(50178))` 过了），
  而 50178 正是根 txg 0 / 1 / 2 的树表单元（探针原样输出：三条根的树表 loc0 都是 `(盘 0, 槽 50178, 校验和 0xdf15b658)`）。
- 实现自己的 `effective_rollback_floor` 报 F_生效 = 0 ⇒ 按 `invariants.md:50` 的字面，txg 0 / 1 / 2 都在候选集里，
  它们引用的块「未被重新分配给其他对象」必须成立。它已经不成立了。
- **I-7.4、I-4.8、I-2.1 三条全判 Holds。**
- 退到 (1, 2) 直接 `Err(Recovery(UnitUnreadable { slot: SlotNumber(50178) }))`——这正是 `invariants.md:153` 判别力句说的那件事：
  「那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**」。假回退候选真的出现了，三条判据一条都没说话。

### 对照组：同一段历史不打掉第二条载体根

```
对照：F_生效 = CheckpointTxg(11)
对照跑完
```

「对照跑完」之前一行 non-Holds 都没打出来 ⇒ 26 条全成立。这一格是**正当的绿**：F_生效 = 11，
txg 0 / 1 / 2 按条款也已经不在候选集里。所以打中的那一格与对照组的差别**只有一个量**：F 在不在两块盘上落齐。

### 打中之后的四句

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分辨。U1 那条臂（按最新根的 F 取候选集）在打中的格上判绿，改法 P1（按 F_生效 取）在同一格上判红、在对照格上仍判绿。两格方向相反，不是一起中 |
| 被判的系统当时看不看得到判别它的东西 | **看得到**。`check_pool_image` 已经把根环全部槽读进 `roots`（`walk.rs:744` 的 `valid_roots`），每条 `RootView` 自带 `rollback_floor`（`crates/singlefs-checker/src/lib.rs:203`），区域到盘的映射在 `geometry.region_devices` 里。算 F_生效 不要任何新输入 |
| 满足的是判据字面的哪一个分句 | Y1 那一行的触发观测原文：「一段历史让候选根的单元被盖而 `check_pool_image` 的 I-7.4 判成立」。逐字对上：候选根 = txg 0 / 1 / 2（按 F_生效 判），单元 = 槽 50178，被盖 = E 的数据单元写进去，I-7.4 判成立。**不是**「`visited_units` 让第二次走跳过校验」那个分句，也不是「校验和碰巧相同」那个分句——这两个分句我没打中，见「没打中的形状」 |
| 跑前条款给的每个改法在打中的那几格上还中不中 | 反向接受条款写「Y1 / Y2 / Y5 打中 ⇒ 改代码、补一条会红的坏镜像与一条变异行、再攻一轮」，没有给具体改法，所以没有「改法在这一格上不起作用」的问题。要提醒的是**补坏镜像那一步**：这一格的坏镜像**不能**建在 `checker_known_bad_images.rs` 的干净镜像上（那份镜像 F 恒 0，两个口径同值、这条变异抓不到——`mutation-sampling.md` 第三类「取样点不敏感」）。要用的是抬过 F、只有一块盘带新 F 的镜像 |

### 改法 P1（只在我的副本上量过、被攻过零轮）

`research/prompts/m2-step6-checker-r1-opus-model/p1-effective-floor.patch`：把 `walk.rs:793-797` 那五行换成
按 `geometry.region_devices` 分盘取 F 最大值、再取各盘最小值。施加之后同一份镜像的原样输出：

```
判违例的：["I-2.1", "I-3.1", "I-4.8", "I-5.1", "I-7.4"]
  I-2.1: Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")
  I-4.8: Violated("候选根 txg 0 出发的遍历有单元对不上或读不出")
  I-7.4: Violated("候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）")
```

对照格施加 P1 之后仍然 26 条全成立（同一次跑，「对照跑完」之前无 non-Holds 行）。
⚠️ 我只量了这两格，没量层 0 的 108 个状态，也没量 `checker_known_bad_images` 那 26 份坏镜像
（P1 会把 F 之下的根重新拉回遍历，`I-3.1` 的并集与 `I-5.1` 的重叠判定都跟着变——上面那次 I-5.1 就多红了一条）。
**这个改法被攻过零轮。** 主 agent 要采纳的话，I-3.1 / I-2.1 那两行 `invariants.md` 里
「按 I-3.1 那个候选集里的根算」的口径要一起重判：`invariants.md:107` I-2.1 那一行现在逐字写着
「2026-09-17 起「被引用」按 I-3.1（已分配统计对得上） 那个候选集里的根算」，而 I-3.1 的并集口径与
`crates/singlefs-core/src/mount.rs:437` 那条注释（「checker 的 I-3.1 按那条根自己的 F 取候选集的并集」）是绑在一起的——
**记账那一侧按「那条根自己的 F」是实现有意选的，回退保护这一侧按 F_生效 是条款要的，两者是两个量，不能共用一个 `newest_rollback_floor` 变量。**
这一点我只读到 `mount.rs:437` 的注释，没有跑实验证明分开之后 I-3.1 还对得上。

## Y2 归错根

### 打中：每根一格判了，但产物里只剩一格

`Judgements::judge`（`crates/singlefs-checker/src/image.rs:53-66`）对每条不变量只留**第一处**违例
（`self.first_violation.entry(invariant).or_insert_with(detail)`），`into_report`（`image.rs:79`）把一条不变量
折成一个 `InvariantVerdict`。⇒ I-7.4 在一份镜像上评估 N 次（N = 候选集里更早的根数），
产物里只有一行，而且那一行的 txg 是**环序里第一条判红的根**，不是「真坏的那条」，更读不出另外 N−1 格是红是绿。

副本上加了一句 env 门控的打印（`ZZ_PER_ROOT=1`，打在 `walk.rs:816` 的 `.judge("I-7.4", ...)` 之前），拿仓里现成的两种坏法跑：

```
[zz] 候选根 txg 0 实例 0：I-7.4 = 违例
[zz] 候选根 txg 1 实例 1：I-7.4 = 违例
[zz] 候选根 txg 2 实例 1：I-7.4 = 违例
乙 I-2.1: Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")
乙 I-4.8: Violated("候选根 txg 0 出发的遍历有单元对不上或读不出")
乙 I-7.2: Holds
乙 I-7.4: Violated("候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）")
```

三条根共用槽 50178 那一个坏单元，三格都红（`read_referenced_unit` 在 `visited_units` 那一关**之前**调，
`walk.rs:158-168` 先读先判 I-2.1，`walk.rs:169-174` 才查已访问）——所以「后走的看不到增量」在这份镜像上不成立，
辩方在这一点上站得住。打中的是另一半：**产物里看不见有三格**。I-4.8 报的是「候选根 txg 0」，
而最新根那一格（`walk.rs:781-785`）是先判的、判成立，于是 I-4.8 这一行既不说最新根，也不说另外两条候选根。

**这一格为什么要紧**：`invariants.md:50` 的状态列逐字写着「按每条回退候选集里更早的根各判一格」。
一格判了、结果被折叠掉，等于这条状态列描述的东西在产物里不可核。步 6 的验收标准（`.claude/kb/milestone/02-second-txn.md:230`，整行抄）
写着「阴性结果与「代码没跑到」分开：新接的每条不变量至少一个崩溃状态上真被评估过，报出评估次数」——
报评估次数这一步在层 0 的 `Layer0Tally` 里做了（按状态数），在 `check_pool_image` 的产物里没做（按根数）。

### 打中：树表已访问时那条根的中央映射树一格都不走

`walk_root`（`walk.rs:229-233`）：

```rust
        let Some(tree_table) =
            self.read_index_node(&record[36..122], 0, KEY_SCHEMA_TREE_TABLE, "树表单元")
        else {
            return;
        };
```

`read_index_node` 在树表单元已被更早一次走读访问过时 `return None`（`walk.rs:169-174`），**不推 `walk_failures`、不判任何一条**。
于是 `walk_root` 在这里 `return`，而中央映射树的根住在**根记录自己**里（`walk.rs:238-243` 读的是 `record[256..342]`，
不是树表里的条目）——共用树表的两条根**可以有不同的映射树根**，后走的那条根的映射树因此一个单元都不读，
它那一格 I-7.4 / I-4.8 就只由「实例表单元 + 树表单元」两次校验和撑着。

**可达性我没证到**：探针在固定脚本上量了两次，共用树表的只有 txg 0 / 1 / 2 三条根，而它们共用的 mkfs 树表
条目数为 0（探针原样输出：`50178：树 ID 0 层级 0 条目 0`），映射指针也全零（`映射槽 0 全零 true`）。
回退 + 四次覆盖写之后的 15 条根里，除这三条外**每条根各有自己的树表槽**
（50248 / 50256 / 50309 / 50313 / 50317 / 50326 / 50373 / 50377 / 50386 / 50394 / 50402 / 50410）。
⇒ 今天的 `crates/` 实现里，空发布也 COW 树表，所以这条路我造不出一个真的假绿。
**这一条按「结构上的洞、今天不可达」记，不按打中记。**
什么现象会推翻「不可达」：出现一次只改中央映射树、不改树表的发布（或者一次让两条根共用树表而映射根不同的发布）。

### 没打中：I-4.8 最新根那一格与 I-7.2 不是同一个判定

`walk.rs:781-785` 判 I-4.8 用 `newest_failures.is_empty() && walk.judgements.violation_count("I-2.1") == 0`；
`walk.rs:832` 判 I-7.2 只用 `newest_failures.is_empty()`。两者在「一份对不上、另一份对得上」时分开——
`read_referenced_unit`（`image.rs:301-324`）对**每条位置条目**都判 I-2.1，只要有一份对得上就返回、不推 `walk_failures`。
副本上拿仓里 I-2.1 那份坏镜像的形态（只改盘 0 的那一份数据单元，`checker_known_bad_images.rs:307-313`）跑：

```
甲 I-2.1: Violated("数据单元 在盘 0 槽 50180 的那一份与位置条目里的校验和对不上")
甲 I-4.8: Violated("最新根（txg 3）出发的遍历有单元对不上或读不出")
甲 I-7.2: Holds
甲 I-7.4: Holds
```

⇒ 「两条不变量在每个镜像上永远同红同绿」为假，这一问没打中。

### 没打中：`walk_failures` 收不收「头用不了 / 条目区解不开」

收。`judge_unit_header` 在头校验和不过或类标签不对时推 `walk.rs:118` 的「`{what}` 的头用不了」，
`index_node_view` 解不开时推 `walk.rs:179` 的「`{what}` 的条目区解不开」，两处都在 `walk_root` 的调用链里，
所以 `walk.walk_failures.len() > failures_before` 罩得住这两类。这一问没打中。

## Y5 层 0

### 快用例上的评估次数与违例数（副本）

在副本的 `second_transaction_step_zero_layer0.rs` 里往
`every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims` 末尾加了一段打印
（只打印 `Layer0Tally` 已有的两张表，没改枚举、没改判定），原样输出的相关行：

```
[zz] I-2.1：评估 108 个状态、违例 0 个状态
[zz] I-4.8：评估 108 个状态、违例 0 个状态
[zz] I-7.2：评估 108 个状态、违例 0 个状态
[zz] I-7.4：评估 108 个状态、违例 0 个状态
[zz] 状态总数 108
```

⇒ **没打中误红**：108 个崩溃状态上新的两条一次都没红，而且每个状态都真被评估过（不是「不适用」）。
Y5 那行的触发观测是「一个崩溃状态让 I-7.4 或 I-4.8 红而那个状态按恢复语义是合法的」，我没造出来。

试过的方向与为什么造不出：根槽写到一半 ⇒ 那一槽自证不过、不进 `valid_roots`，候选集里少一条根，不会误红；
单元写到一半 ⇒ 单元的两份写在根槽之前的段里、两道屏障隔着，能让根成为最新根的状态里那些单元都已落齐；
记录写到一半 ⇒ 不进遍历方向。更早的候选根引用的单元在这三类里恒不被碰（COW：新版本写新落点），
只有**复用**能碰它们，而复用被 F 与影子账挡着。

### 打中一格阴性结果：层 0 全绿买不到对 F 之下那批根的保护

固定脚本到 E 含抬 F 那两次空发布（txg 15、16）。**两条载体根之间的崩溃状态**里，
最新那条持久根已经带 F = 11，而 F_生效 还是 0（同一段历史，Y1 那一节的原样输出：
`F_生效 = CheckpointTxg(0)；最新根 txg CheckpointTxg(17) 自己带的 F = CheckpointTxg(11)`）。
在那些状态上 checker 按 `walk.rs:803` 的 `below_floor` 把 txg 0–10 的根整批排除 ⇒
I-7.4 / I-4.8 虽然「被评估过」，评估的却是一个**不含风险根**的候选集。

这正是 `crates/singlefs-core/src/mount.rs:437-439` 那条注释自己点名的窗口（整行抄第一句）：

```
    // 取候选集的并集，释放代 ≤ F 的落点不在并集里，记账的「已分配」也不能再算它们。但回收的槽在生效（每块盘都有带新 F 的根）之前
```

实现靠「回收的槽在生效之前不许发出去」挡住它，仓里也有一条用例钉着
（`second_transaction_step_five_reuse.rs` 的 `slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect`）。
⇒ **那条用例是这个窗口今天唯一的守卫，而 I-7.4 本来该是它的独立第二道。** 今天两道并成了一道：
守卫失效时 checker 在那个窗口里判绿（Y1 那一格就是把守卫失效造出来之后量的）。
`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 的「校验路径本身也要证明它会红」说的正是这件事。

什么现象会推翻这一条：给 checker 换上 F_生效 的口径之后，同一份镜像仍判绿。

### Y5 的取样范围

只跑了快用例这一档（108 个状态：1 + 二十个 2 写段各 3 + 两个 4 写段各 15 + 十七个 1 写段各 1，
数与 `second_transaction_step_zero_layer0.rs` 里 `assert_eq!(tally.states, 108, ...)` 那句一致）。
全量 2 104 413 个状态我**没跑**：开跑时本机已有一个 release 的全量在跑（`ps` 见报告开头），
debug 下这一份按仓里那条 `#[ignore]` 的说明要半小时以上，45 分钟的预算放不下。
所以「全量上新的两条会不会误红」这一问，我这条腿**没有观测**。

## 没打中的形状

| 形状 | 取样范围 | 为什么没打中 |
|---|---|---|
| `visited_units` 让第二次走跳过校验和（Y1 第一个分句） | 读 `walk.rs:158-174`、`walk.rs:203-227`、`walk.rs:349-381`、`walk.rs:484-501` 四处 `visited_units` 的全部调用点，再在副本上拿仓里 I-7.4 那份坏镜像（改 50178 再重封）跑一次每根打印 | **跳不过**。`read_referenced_unit` 在四处都排在 `visited_units.insert` **之前**，校验和是每次引用都判的；`visited_units` 只跳过「解容器 / 判头 / 往下走」。三条共用 50178 的候选根三格全红 |
| 复用之后新对象与旧位置条目校验和相同（Y1 第二个分句） | 没量 | 需要 CRC32 碰撞，1/2³²。这不是可达历史能造出来的，只能靠挑字节，我判它不属于「造可达的历史」 |
| 被复用的单元只被映射条目引用、不被树表引用（Y1 第三个分句） | 探针跑了两段历史（第一个事务之后 4 条根、回退 + 四次覆盖写之后 15 条根），逐条打印树表槽与映射槽 | 走得到。`walk_root`（`walk.rs:238-267`）对每条映射条目都调 `read_referenced_unit`，唯一走不到的情形是树表已访问那条 `return`（见 Y2 那一节），而今天每条根各有自己的树表，造不出来 |
| 被抛弃根引用的单元被盖 ⇒ I-7.4 该不该红 | 只读条款、没造镜像 | 这一问是 Y1 那行括注里的「被抛弃根不在候选集里但它引用的单元被盖——这一格 I-7.4 该不该红（条款字面是候选集）」。`invariants.md:50` 逐字写着「被抛弃时间线的根不在候选集里，这只管回退目标的选择；它们引用的块在离开根环之前同样不许重新分配、不许抹头」⇒ 条款字面**要求**护住它们，而 U1 的候选集把它们排除（`walk.rs:800-802` 的 `abandoned`）。这是条款与代码之间的一处差，但它落在 Y4「代码与条款」那一格（Sonnet 正推腿的），我不判 |
| Y5 误红 | 快用例 108 个状态全跑 | 见 Y5 那一节 |
| 内部节点的树（extent / 分配 / 记账层级 > 0）下面的单元 | 只读代码，没造历史 | `walk.rs:298-304` 在这三类树有内部节点时只调 `not_applicable("I-7.2", ...)` 就不往下走；而 `not_applicable` 只在那条不变量**一次都没被评估过**时生效（`image.rs:79-93` 的 `into_report`：先看 `first_violation`，再看 `evaluated > 0`），I-7.2 在 `walk.rs:832` 无条件判过 ⇒ **那句 `not_applicable` 是死码**，产物照样报 I-7.2 成立，而那棵树下面的单元一个都没读、候选根那几格也跟着空判。**可达性我没证到**：固定脚本的文件只有 2000–4100 字节，三类树都停在层级 0。要让它可达得先有一棵长到两层的 extent / 分配 / 记账树 |

## 这条腿自己的限度

1. **副本上的数不进 kb。** Y1 那一格、对照格、改法 P1 的三组读数都是
   `/tmp/claude-1000/m2-step6-checker-r1-opus/copy/` 上跑出来的。要引，按
   `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」重做。
2. **改法 P1 被攻过零轮**，而且只在两格上量过（打中格转红、对照格仍绿）。没量层 0 的 108 个状态、
   没量 `checker_known_bad_images` 的 26 份坏镜像、没量 `crates/mutations.tsv` 的 55 条。
   施加之后 I-5.1 在打中那一格多红一条，我没判那是对是错。
3. **I-3.1 那一侧我没判。** 打中那一格的镜像上 I-3.1 也红了
   （`盘 0：记账的已分配 Some(983040)，遍历全部有效根得到 917504`），对照格不红 ⇒ 是打掉载体根带来的。
   我没查这是记账口径的问题还是我那段历史的副作用。两个口径（记账按「那条根自己的 F」、
   回退保护按 F_生效）是不是真要分开，要 alloc-basis 那一轮判，不归这条腿。
4. **全量层 0 没跑**，理由与取样范围写在 Y5 末尾。
5. **Y3、Y4、Y6 一个字都没判**，按分工表。Y1 括注里「被抛弃根引用的单元」那一问我也交回去，它落在 Y4。
6. **「可达」的判据我用的是「今天的 `crates/` 跑得出来」**：Y1 那一格的历史全部由
   `build_pool` / `publish_overwrite` / `mount_rollback` / `raise_rollback_floor` 跑出来，
   只有「打掉第二条载体根那一槽」那一步是直接写字节，它对应层 0 枚举里根槽没落齐的那一格，
   而且是仓里 `roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates` 用的同一手。
   **那一步之后「再发布一次 E」用的是进程内已经回收过的分配器**——实现在真的重新挂载时会按
   F_生效 = 0 重算并扣住那些槽（`mount.rs:437-439`），所以这一段历史模拟的是
   「抬 F 生效判定失效」这一类实现缺陷，正是 I-7.4 存在的理由。**要求 checker 在实现没坏时也不红，
   与要求它在实现坏了时红，是两件事；这一格问的是后一件。**
