# 挂载时比对设备容量：容量记在哪、怎么判（m4-capacity-r1）

日期：2026-11-17。

## 问题

用户定案（弹窗选「记容量、动格式」，`records/2026-11-17-里程碑四收口续做.md`）：C1040（挂载不比对探测到的物理块大小与两盘容量） 容量那一半不退化成各盘互比，要在盘上记下容量，挂载时拿探测值与记录值比。这一轮要定：

1. 容量记在哪：系统配置里逐盘记「本盘容量」（照「本盘设备号」那样每块盘的那一份只记自己）、系统配置里每一份都记全池每盘容量、还是提前写设备描述符表（D12（目标介质） 已定项 5 写「加第二套布局前必须写」）。
2. 记什么量、多宽：字节数还是槽数，几个字节，取 mkfs 时探测的值还是 mkfs 实际用到的那一段。
3. 挂载时怎么判：变小拒（可写与只读都拒，照 D12（目标介质） 已定项 4 比对表那一行与 2026-11-17 改成「拒绝挂载」的 `physical_block_size` 同一处置，还是只拒可写）、变大怎么「放行并记」（记要不要在挂载时写盘、写进哪一份、算不算一次发布）、记录值为 0 或越界怎么办。
4. 与 D2（RAID 条带策略） 已定项 2「各盘不必等大」、D22（单元原子性怎么合成） 已定项 15「三条比对在第一版退化成各盘互比」怎么收：哪几句要改、改成什么。
5. 池级 checker 怎么判、要不要新立不变量；GPU 判器要不要同判。

## 实现今天的样子

主 agent 的观测，方案按 `crates/` 今天的实现来谈：

- 系统配置每盘两槽、池里每盘一份（`crates/singlefs-core/src/system_configuration.rs` 第 2 行文件头「每盘两槽轮换」）；mkfs 只从参数拿五个量，「本盘设备号由 mkfs 逐盘现填，设备数由池里的盘数现数」（同一文件第 247 行）。
- 系统配置字段的偏移常量排到 `TAIL_OFFSET: u64 = 469`（第 80 行），回退下界 F 住槽尾（第 85 行 `ROLLBACK_FLOOR_OFFSET = SYSTEM_CONFIGURATION_BYTES - ROLLBACK_FLOOR_FIELD_BYTES`）；469 与回退下界之间有没有别的认购，以 `.claude/kb/layout/01-first-txn.md` 系统配置那一节为准，腿去现查。
- 挂载只比调用方交进来的 mkfs 参数（`crates/singlefs-core/src/mount.rs:7640` 的 `MakeFilesystemParameterField::PhysicalBlockSize`）；容量今天的护栏只有 `every_device_reaches_the_unit_area_start` 与分配记录越过单元区末尾那一判（`crates/singlefs-core/src/recovery.rs` 的 `placement_lies_in_the_unit_area_of_its_device`）。
- 用例 `crates/singlefs-verification/tests/unequal_devices.rs:183` 钉着两盘不等大照可写挂载（`mount_writable(&parameters(), &mut remounted_devices).expect("盘不等大的可写挂载今天接受")`）。
- 只读挂载入口 `crates/singlefs-core/src/mounted_read.rs:651` 的 `mount_read_only(reader: &dyn PoolReader)` 今天没有探测；另一个会话的实现员正在给它加 `physical_block_size` 的探测比对（同一份调度记录批 2），容量的比对与它同一处。
- `crates/` 里没有「设备描述符表」的代码：`grep -rn '设备描述符表\|device_descriptor' crates/singlefs-core/src | wc -l` 是 0，`grep -rn 'struct.*DeviceDescriptor\|fn .*device_descriptor' crates/` 也没有命中（主 agent 写正文时现查）。

## 压着这件事的条款

- `.claude/kb/decisions/12-目标介质.md`「#### 已定项 4：挂载时比对哪几条」「#### 已定项 5：第一版只实现普通 SSD 那一套布局」「#### 已定项 3：设备级几何量住系统配置里的设备描述符表」
- `.claude/kb/decisions/22-单元原子性怎么合成.md`「#### 已定项 15：系统配置字段表的四格连带」「#### 已定项 9：系统配置的字段表」「#### 已定项 2：根环的参数」
- `.claude/kb/decisions/02-RAID条带策略.md`「#### 已定项 2：各盘不必等大」「#### 已定项 3：设备集合变化走显式「几何再求值」事务」
- `.claude/kb/decisions/15-格式冻结政策.md`「#### 已定项 7：冻结分四层，四层之外另有三个独立冻结组件」（动格式落哪一层）
- `.claude/kb/checks-owed.md` 的 C1040、C218、C180 三行
- `.claude/kb/layout/01-first-txn.md` 系统配置那一节

## 判据格

每格各答「选哪个候选、依据（条款或实现的 文件:行）、它买到什么失败模式改善」：

1. 落点：逐盘记本盘 / 每份记全池 / 设备描述符表。判据照 `.claude/rules/fs-design.md`「格式分支的判据」：能不能把「用错了」变成「挂不上」。
2. 宽度与量纲：字节或槽，几个字节，取哪个值。
3. 变小的处置与射程（可写、只读）。
4. 变大的处置：「放行并记」怎么落地，写不写盘、写哪一份、算不算发布、崩溃在中间会怎样。
5. 记录值读不出、为 0、越界的处置。
6. 条款怎么改：D12 已定项 4 与 5、D22 已定项 15、D2 已定项 2 各改哪一句，或说明不用改。
7. checker 与 GPU 判器：判什么、要不要新立不变量。
8. 已有用例 `unequal_devices.rs:183` 在新判法下是绿是红。

## 分工

正推腿逐格推一个完整方案；攻方腿攻：第 1 格三个候选各自哪里会错、第 4 格「放行并记」在崩溃与回退下的漏洞、第 6 格改条款会不会与别的已定项打架。

## 交用户的

只交候选之间真分不开、要拍板的岔路；能由条款或实现判出的，判出不交。
