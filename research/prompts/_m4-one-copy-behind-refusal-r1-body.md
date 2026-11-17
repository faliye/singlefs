# m4-one-copy-behind-refusal-r1：只落一份的末条记录加早先的坏副本，可写挂载要不要放行（2026-11-09）

<!-- doc-lint:not-numbers P1 -->

## 一、题面

用户 2026-11-09 定：普通断电加一份坏盘之后池只剩只读，不认，开一轮三方定放行。这一轮问放行条件写成什么才不会放掉真缺单元的盘。主 agent 推荐的方向是候选「点名窗口」，不算结论。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 拒的地方：`crates/singlefs-core/src/mount.rs` 的 `units_of_the_version_missing_on_a_device_behind_it`（第 7049 行）。拒因 `WritableMountRefusedByDevicesWithoutTheSelectedVersion`，`BehindTheSelectedVersionAndMissingItsUnits`。
- 判法不认这个拒因：`crates/singlefs-verification/src/mount_transparency.rs` 第 2275、2330 行。
- 新用例在补丁里、还没进主工作区：`~/.local/share/singlefs/work/m4-close-r3-unit-read-writer/copy/crates/singlefs-verification/tests/crash_points_of_the_repair_of_the_units_exempt_from_the_mapping.rs` 的 `every_persisted_set_of_the_batches_on_the_disk_with_the_damage_still_on_it_mounts_writable_and_is_green_after_the_unmount`。那份副本是主工作区加这一批补丁。
- 量的（任务 11560，一条流、两块盘，确定性内存池，日志 `~/.local/state/singlefs/task-server/logs/task-b0f6b921-11560.attempt-1.log`，整行与判定在 `~/.local/share/singlefs/work/m4-close-partial-persist-investigator-2/result.md` 第 1 节）：
  - 坏盘流 153 个崩溃状态 3 个可写挂载被拒，都是这次发布的 journal 末条记录只落盘 1、盘 0 那一份没落；两份记录写在同一屏障段。
  - 盘 0 tail 比所选版本落后一条，缺的两份（映射树叶槽 50269、实例表槽 51136）是早先弄坏的，10 个记录写都没点名它们。
  - 三个状态二次可写挂载都拒，拒前后盘上不变；只读挂载与恢复读文件做成；只把坏字节放回就挂得上。放回字节的流 153 个、没弄坏的流 63 个都挂得上。
  - 其余 150 个状态挂得上之后，卸载判真盘这一段因为用例先 panic 没跑到，没有读数。
- 调查员的三处没证（同一份报告第 4 节）：tail 是跨盘取的见证值，会不会写到没收过单元的盘上；取号、暖机、抬 F、写行这几种发布的记录是否也点名全部单元；放行之后盘 0 没有这条记录，接着发布、卸载、判真盘的后果。
- 方案按 `crates/` 今天的实现来谈。

## 三、候选（被攻过零轮，都没编没跑）

1. 点名窗口：缺单元判只算 (tail, 所选] 那几条记录点名的单元；这些之外的单元缺一份按读到一份坏处理。
2. 紧邻一版的点名：同一处，只在 tail 紧挨所选发布时放宽。

判决 `research/prompts/m3-close-sysconf-r3-main-verification.md` 第 23 行的前提是末条记录是分得开「落盘之后坏了一份」与「从没落盘」的唯一盘上证据；候选改的就是这个前提。

## 四、要回答的问题

1. 候选 1、2 判得对不对：找一段历史，放行之后盘 0 真的缺这一版要的、而 tail 之后又没点名的单元；或者该放行的还拒。要造到盘面，能量的量。
2. 第二节三处没证的逐条答。tail 取哪块盘、记录的点名是不是全集（`crates/singlefs-core/src/transaction.rs` 第 1521、1528 行一带），写行、抬 F、暖机、取号发布。
3. 放行之后接着可写：盘 0 上那一版的记录缺着，下一次发布、下一次崩溃、卸载之后判池级 checker，各走到什么；坏的那一份由谁换（D4（校验和位置） 已定项 4 的搬迁能不能接上）。
4. 两块盘都落后、或坏的不止一份、或坏的一份正好是这次发布写的，判什么。
5. 改动面：`crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/mounted_session.rs` 里同一判的调用、GPU 与 CPU 判器、`crates/singlefs-verification/src/mount_transparency.rs`，各要不要改，代价多少步。

## 五、压着的条款

D18（块里携带什么信息） 已定项 11；D23（journal 的角色与格式） 已定项 14；D4（校验和位置） 已定项 4；C593（落后且缺一份单元的盘被永久拒可写）；C596（见证判不出的系统配置盘面只剩只读挂载）。
