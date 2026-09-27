# kb 写回规格（第五批：坏盘输入的 panic 面实审 A3a 收口）

每条：文件、旧串、新串、依据。同一文件各条旧串互不相交，按次序施加。

## 条 1

文件：`.claude/kb/decisions/22-单元原子性怎么合成.md`

旧串：
```
- 根环的降级语义写下来了、在真设备上量过，答的是 [pitfalls.md](../pitfalls.md) 第 9 条（degraded 挂载会写出单副本、越救越糟，根因是降级行为没有设计）。
- 根环只承担「本代根写坏了还能读到上一代」，不是「可以回滚 K 代」（已定项 25）；zoned 上这套参数不适用（未定项 6）。
```

新串：
```
- 根环的降级语义写下来了、在真设备上量过，答的是 [pitfalls.md](../pitfalls.md) 第 9 条（degraded 挂载会写出单副本、越救越糟，根因是降级行为没有设计）。
- **读者择系统配置时判根槽宽与固定结构槽距**：固定结构槽距 ∈ [4096, 1 MiB − 4096]（上界是槽 1 整槽落在根环基址之前，`root_ring::region_start(0)` 减系统配置槽宽 4096）；根槽宽（`physical_block_size`）∈ [根记录宽 457 字节（已定项 7）, 槽距]；越界整池拒绝挂载，与每区槽数 S 越界同一个处置（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts`，`SystemConfigurationValueOutsideWhatThisReaderAccepts::FixedStructureSlotSpacingOutsideTheFormatRange` / `PhysicalBlockSizeOutsideTheRootSlotBounds`）。
- 根环只承担「本代根写坏了还能读到上一代」，不是「可以回滚 K 代」（已定项 25）；zoned 上这套参数不适用（未定项 6）。
```

依据：research/prompts/m2-rev-a3a-implementer-report.md 第四节「条款要补的原句」第 1 条

## 条 2

文件：`.claude/kb/decisions/23-journal的角色与格式.md`

旧串：
```
**射程**：③ 的「参数 + 上界」让大盘上 768 MiB 只占容量的百分之几、小镜像按参数缩，`环 ≤ 容量 ÷ 4` 挡住「环吃掉整个盘」；⚠️ 连带：`.claude/kb/layout/01-first-txn.md` 的预想镜像是 **4 GiB**（1 GiB 的镜像在这条约束下装不下 768 MiB 的环）。实现：`crates/singlefs-core/src/make_filesystem.rs` 收环长参数、按最小那块盘的容量 ÷ 4 拒绝越界，默认值是 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_RING_DEFAULT_BYTES`。
```

新串：
```
**射程**：③ 的「参数 + 上界」让大盘上 768 MiB 只占容量的百分之几、小镜像按参数缩，`环 ≤ 容量 ÷ 4` 挡住「环吃掉整个盘」；⚠️ 连带：`.claude/kb/layout/01-first-txn.md` 的预想镜像是 **4 GiB**（1 GiB 的镜像在这条约束下装不下 768 MiB 的环）。实现：`crates/singlefs-core/src/make_filesystem.rs` 收环长参数、按最小那块盘的容量 ÷ 4 拒绝越界，默认值是 `crates/singlefs-format/src/lib.rs` 的 `JOURNAL_RING_DEFAULT_BYTES`。 挂载读者一侧另判环长：在飞上限（环槽数 ÷ F，已定项 18）≥ 1，且环末端不越过编译期单元区起点（768 MiB，`crates/singlefs-format/src/lib.rs` 的 `UNIT_AREA_START_SLOT` 与 `JOURNAL_RING_START_SLOT`）；越界整池拒绝挂载（实审 A3a 落地：`crates/singlefs-core/src/recovery.rs` 的 `SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange`）。
```

依据：research/prompts/m2-rev-a3a-implementer-report.md 第四节「条款要补的原句」第 2 条

## 条 3

文件：`.claude/kb/decisions/19-块指针的结构与宽度预算.md`

旧串：
```
「恢复期扫一个部分损坏的节点时能不能跳过坏掉的那一条继续读」是推论，未实测。
```

新串：
```
「恢复期扫一个部分损坏的节点时能不能跳过坏掉的那一条继续读」是推论，未实测。 加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法（实审 A3a 落地在两处读者——多层码 2 树内部条目、journal 新根段指针：`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent 偏移那 2 字节读到非 0 怎么处置仍没有条款，记进欠账 C581（extent 偏移非 0 没有判据）。
```

依据：research/prompts/m2-rev-a3a-implementer-report.md 第四节「条款要补的原句」第 3 条

## 条 4

文件：`.claude/kb/decisions/09-加密.md`

旧串：
```
写路径把这些位写 0（`PointerHead::write_to`、`RootRecord` 与 `JournalRecord` 的写出、`SystemConfiguration::to_slot`、单元头的 29 字节预留），读路径跳过它们，池级 checker 不判它们。
```

新串：
```
写路径把这些位写 0（`PointerHead::write_to`、`RootRecord` 与 `JournalRecord` 的写出、`SystemConfiguration::to_slot`、单元头的 29 字节预留）；core 读路径今天判单元头 29 字节全 0（I-2.4（头校验和覆盖范围））、系统配置加密类型恒 0、两处读者（多层码 2 树内部条目、journal 新根段指针）的指针头部 MAC / nonce 恒 0，非 0 一律判损坏（实审 A3a）；这两处读者之外还没换上的指针读者、系统配置其余结构常量、journal 记录自己的算法类型 / nonce / MAC 预留，读路径仍跳过。池级 checker 仍不判这些字段，checker 那一半实现在做。
```

依据：research/prompts/m2-rev-a3a-implementer-report.md 第四节第 4 点；「三、checker 那一侧」

## 条 5

文件：`.claude/kb/checks-owed.md`

旧串：
```
，够不着只说明坏法还不够，不说明它们不用修。
```

新串：
```
，够不着只说明坏法还不够，不说明它们不用修。 **2026-09-27 实审 A3a 判全接上真实路径**：①分配器族 R6–R9 那 9 处的判据（`allocation_records_fit_the_pool_geometry`）此前只挂在 `walk_to_file`、挂载走的 `rebuild_version` 一次不调（2026-09-22 已现查坐实）；A3a 第 33 / 24 条把 `rebuild_version` 改成与 `walk_to_file` 调同一组判定函数，这 9 处由此接上可写挂载路径。缺口表 `pointer.rs:108/149` 升序断言坐实走得到（副本探针：树表条目根指针与第 0 代根实例表指针两条位置条目对调，都让可写挂载 panic 在 `pointer.rs:222`），已按 I-2.5（位置条目按设备身份升序） 判（重建时对根记录四条指针、树表条目根指针、extent 树、inode 叶容器、分配记录树、记账树、映射树每个节点的指针判升序，逆序或相同报 `InvariantViolated`，`invariant` 字段填这条不变量的名字）。**A3a 清单外仍开着**：②条目宽度族 8 处、R11–R13（checker `image.rs` / `walk.rs`）；缺口表 `lib.rs:268`、`walk.rs`、`inode_tree.rs:88`（普查行号，没按今天的文件重查）；指针头部判的读法在 A3a 清单外还没换上的读者——`root_record.rs:131/135/136/137`、`records.rs:328/357/369`、`instance_table.rs:206`、`extent_tree.rs:322/327/688`、`allocation_record_tree.rs:774/869`（主工作区行号，实审 A3a 报告第三节）。**判全带来一处不在原 21 处普查里的新发现**：`transaction.rs:6339` 树表条目按树 ID 升序的断言仍走得到——I-7.8（根记录树 ID 水位不低于全池最大树 ID） 只挡「号越过水位」这一形，水位之下换次序照样走到这句断言（推的，没造镜像实测）；拦不拦、拦在哪，交主 agent 定（实审 A3a 报告第七节末、第十一节）。
```

依据：research/prompts/m2-rev-a3a-implementer-report.md 第三节 C476 行、第七节、第十一节

## 条 6

文件：`.claude/kb/checks-owed.md`

旧串：
```
| C580 | 两处读不出无声放过没动 | `recovery::effective_rollback_floor` 自己再择一次根、读实例表，读不出就「不按表滤」；挂着时抬 F 重算影子账用 `readable_roots`，根槽读不出的被抛弃根照旧不隔离、不计数（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 前半段） | 造这两格镜像，断言按定下的判法处置；判别力自证：把「不按表滤」/「不隔离」换成拒绝，今天的实现必须由绿转红 | 不在这一件的射程里，没改 | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q6） |
```

新串：
```
| C580 | 两处读不出无声放过没动 | `recovery::effective_rollback_floor` 自己再择一次根、读实例表，读不出就「不按表滤」；挂着时抬 F 重算影子账用 `readable_roots`，根槽读不出的被抛弃根照旧不隔离、不计数（C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 前半段） | 造这两格镜像，断言按定下的判法处置；判别力自证：把「不按表滤」/「不隔离」换成拒绝，今天的实现必须由绿转红 | 不在这一件的射程里，没改 | 2026-09-27 实 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙报告（`research/prompts/m2-impl-c554-yi-implementer-report.md`「六、停下交主 agent 的设计问题」Q6） |
| C581 | extent 偏移非 0 没有判据 | 指针头部 extent 偏移那 2 字节第一版写 0（D19（块指针的结构与宽度预算） 已定项 3 只写了 MAC / nonce 恒 0 的判法，没写这两字节）；读者今天 `reader.skip(2)`，不判、不拒，读到非 0 悄悄丢掉（`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadAsRead::read_from`） | 造一份指针 extent 偏移非 0 的镜像，按定下的条款判（同 MAC / nonce 一样判该指针所在的结构损坏，或者别的处置）；条款定之前照旧按今天的读法（跳过、不拒），盘上不变；判别力自证：条款接上判定之后，同一份镜像必须由「悄悄丢掉」变成按新条款拒绝或标记 | 条款没定，交主 agent | 2026-09-27 实审 A3a 报告（`research/prompts/m2-rev-a3a-implementer-report.md` 第四节第 4 点「条款要补的原句」段） |
| C582 | 挂载盘容量下界没条款该取哪个 | 挂载一侧盘容量下界没有条款写取哪个：`DeviceFreeMap` 的前置条件要「完整槽数 ≥ 单元区起始槽号」，mkfs 的下界更严（要装下 mkfs 写的实例表与树表第 0 版）；今天挂载一侧的实现取的是前者，条款没写该取哪个 | 造一块盘，容量落在『装得下单元区起点、装不下 mkfs 第 0 版写的东西』这个区间里，挂载与 mkfs 分别跑；按定下的条款判该拒的是哪一步；判别力自证：把挂载一侧的下界改回不检查，同一份镜像必须由拒绝变成走到更深处的坏结果 | 条款没定，交主 agent；今天实现取「完整槽数 ≥ 单元区起始槽号」 | 2026-09-27 实审 A3a 报告（`research/prompts/m2-rev-a3a-implementer-report.md` 第四节第 5 点） |
```

依据：research/prompts/m2-rev-a3a-implementer-report.md 第四节第 4、5 点

## 条 7

文件：`.claude/kb/decisions-history/2026-09.md`

旧串：
```
## 历史版本

### 2026-09-27（其八）：D23（journal 的角色与格式） 已定项 14、D16（发布语义） 已定项 1：C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续——系统配置见证过的更新状态读不出，重读一次仍读不出拒可写
```

新串：
```
## 历史版本

### 2026-09-27（其十三）：D9（加密） 已定项 10：读路径判单元头、系统配置加密类型与两处指针头部，checker 那一半在做

> 快查·改前：D9（加密） 已定项 10 射程写「读路径跳过它们，池级 checker 不判它们」。
>
> 快查·改后：改成现状：core 读路径今天判单元头 29 字节、系统配置加密类型、两处指针头部的 MAC / nonce，非 0 判损坏；清单外的读者与系统配置其余结构常量仍跳过；池级 checker 仍不判，checker 那一半实现在做。

- 改前：D9（加密） 已定项 10 射程写「`crates/` 里今天没有加密代码：写路径把这些位写 0……读路径跳过它们，池级 checker 不判它们」。
- 改后：改成现状：core 读路径今天判单元头 29 字节全 0（I-2.4（头校验和覆盖范围））、系统配置加密类型恒 0、实审 A3a 清单内两处读者（多层码 2 树内部条目、journal 新根段指针）的指针头部 MAC / nonce 恒 0，非 0 一律判损坏；A3a 清单外还没换上的读者、系统配置其余结构常量、journal 记录自己的算法类型 / nonce / MAC 预留仍跳过；池级 checker 仍不判这些字段，checker 那一半实现在做。
- 依据：实审 A3a 报告（`research/prompts/m2-rev-a3a-implementer-report.md` 第四节第 4 点「D9（加密） 已定项 10 射程那句……随这一件过时」与「三、checker 那一侧」）。

### 2026-09-27（其十二）：D19（块指针的结构与宽度预算） 已定项 3：MAC / nonce 非 0 判损坏补齐，extent 偏移记欠账

> 快查·改前：D19（块指针的结构与宽度预算） 已定项 3 只写「不加密的卷把 MAC / nonce 留成空位」，没写读者读到非 0 怎么处置；extent 偏移那 2 字节同样没有条款。
>
> 快查·改后：射程补一句：MAC / nonce 恒 0，非 0 判该指针所在结构损坏（同 I-2.4（头校验和覆盖范围） 单元头 29 字节的读法），实审 A3a 已落地在两处读者；extent 偏移那 2 字节读到非 0 怎么处置仍没有条款，记进欠账 C581（extent 偏移非 0 没有判据）。

- 改前：D19（块指针的结构与宽度预算） 已定项 3 射程只写「不加密时留位」的取舍与代价换算，没有一句写读者读到 MAC / nonce 非 0、或 extent 偏移非 0 时怎么处置。
- 改后：射程补一句：加密关着时 MAC 16、nonce 12 恒 0，读者遇到非 0 判该指针所在的结构损坏，同 I-2.4（头校验和覆盖范围） 给单元头 29 字节的读法；实审 A3a 已落地在两处读者（多层码 2 树内部条目、journal 新根段指针，`crates/singlefs-core/src/pointer.rs` 的 `PointerHeadFieldOutsideTheFirstVersion::MacOrNonceNotZero`）；extent 偏移那 2 字节读到非 0 怎么处置仍没有条款，记进欠账 C581（extent 偏移非 0 没有判据）。
- 依据：实审 A3a 报告（`research/prompts/m2-rev-a3a-implementer-report.md` 第四节「条款要补的原句」第 3 条；主 agent 按先例定：同 I-2.4（头校验和覆盖范围） 单元头 29 字节的读法，被攻过零轮，代码第二轮三方攻）。

### 2026-09-27（其十一）：D23（journal 的角色与格式） 已定项 18 / 19：读者判环长的界补齐

> 快查·改前：D23（journal 的角色与格式） 已定项 18 只写在飞记录数上限的算法与语义，已定项 19 ③ 只写环长是 mkfs 参数、约束环 ≤ 设备容量 ÷ 4、越界拒绝 mkfs，两处都没写读者挂载时判环长的界。
>
> 快查·改后：已定项 19 射程补一句：读者择系统配置时判环长——在飞上限（环槽数 ÷ F，已定项 18）≥ 1，且环末端不越过编译期单元区起点（768 MiB）；越界整池拒绝挂载，实审 A3a 已落地。

- 改前：D23（journal 的角色与格式） 已定项 19 射程只写 mkfs 一侧的环长约束（环 ≤ 设备容量 ÷ 4，越界拒绝 mkfs），没有一句写挂载读者一侧判环长的界。
- 改后：射程补一句：挂载读者一侧另判环长——在飞上限（环槽数 ÷ F，已定项 18）≥ 1，且环末端不越过编译期单元区起点（768 MiB，`crates/singlefs-format/src/lib.rs` 的 `UNIT_AREA_START_SLOT` 与 `JOURNAL_RING_START_SLOT`）；越界整池拒绝挂载；实审 A3a 已实现（`crates/singlefs-core/src/recovery.rs` 的 `SystemConfigurationValueOutsideWhatThisReaderAccepts::JournalRingBytesOutsideTheSupportedRange`）。
- 依据：实审 A3a 报告（`research/prompts/m2-rev-a3a-implementer-report.md` 第四节「条款要补的原句」第 2 条；主 agent 按先例定，被攻过零轮，代码第二轮三方攻）。

### 2026-09-27（其十）：D22（单元原子性怎么合成） 已定项 2：读者判根槽宽与固定结构槽距的界补齐

> 快查·改前：D22（单元原子性怎么合成） 已定项 2 只定了根槽宽取值方式（等于探测到的 physical_block_size）与槽距的算法（4096 向上取整到 io_min 的整数倍），没写读者挂载时判它们的上下界。
>
> 快查·改后：射程补一句：读者择系统配置时判固定结构槽距 ∈ [4096, 1 MiB − 4096]、根槽宽 ∈ [根记录宽 457, 槽距]，越界整池拒绝挂载（与每区槽数 S 越界同一个处置），实审 A3a 已落地。

- 改前：D22（单元原子性怎么合成） 已定项 2 射程只写了根槽宽取值方式与几处已知边角，没有一句写读者挂载时判根槽宽、固定结构槽距的上下界。
- 改后：射程补一句：读者择系统配置时判根槽宽与固定结构槽距——槽距 ∈ [4096, 1 MiB − 4096]（槽 1 整槽落在根环基址之前）、根槽宽（physical_block_size）∈ [根记录宽 457 字节, 槽距]；越界整池拒绝挂载，与每区槽数 S 越界同一个处置；实审 A3a 已实现（`crates/singlefs-core/src/recovery.rs` 的 `system_configuration_values_this_reader_accepts`）。
- 依据：实审 A3a 报告（`research/prompts/m2-rev-a3a-implementer-report.md` 第四节「条款要补的原句」第 1 条；主 agent 按先例定：每区槽数 S 越界同一处置，该件被攻过零轮、代码第二轮三方攻）。

### 2026-09-27（其八）：D23（journal 的角色与格式） 已定项 14、D16（发布语义） 已定项 1：C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙-配置续——系统配置见证过的更新状态读不出，重读一次仍读不出拒可写
```

依据：research/prompts/m2-rev-a3a-implementer-report.md 第四节「条款要补的原句」全五条
