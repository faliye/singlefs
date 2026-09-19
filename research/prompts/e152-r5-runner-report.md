# E152 第五次正式跑：执行员报告（2026-09-19）

时区：命令时刻均为 UTC；文中「JST」按 UTC+9 换算。worktree：/tmp/claude-1000/e152-r5-wt（主 agent 已建，HEAD 5efaf79，crates/ 与 HEAD 无差异）。

## 单测、变异

- `cd /tmp/claude-1000/e152-r5-wt/research && nice -n 19 cargo test --release --bin e152-file-system-benchmark`：**26 passed; 0 failed**。
- `nice -n 19 bash scripts/mutate.sh e152-file-system-benchmark e7-index-bench/src/bin/e152_file_system_benchmark.rs mutations/e152_file_system_benchmark.tsv`：**26 条全部标 ✅ 红，收尾行「已还原，基线仍全绿」**，0 无效、0 未抓。
- `python3 research/scripts/e152-tables.py --selftest`：`selftest: 通过（9 行渲染与期望逐字相同，不稳定的格子带了 ⚠）`。

三份装置文件 sha256 从始至终等于登记第三节（本轮对 bin 源文件与 `e152-run.sh` 做过的两处临时改动均已改回并核对，见下）。

## 产物（第一段，5 轮，不加 nice）

`E152_CONFIGURATIONS=singlefs bash research/scripts/e152-run.sh research/results/e152-file-system-benchmark-second-transaction-relay-timing-2026-09-19.out`：5 轮 `vm_exit=0` 一次过，宿主负载 6.88→5.98→5.21→4.64→4.15（单调降）。产物 142 行，末行 `E7RESULT name=summary_done emitted=26`。已拷回主仓 `research/results/`，两处 `sha256sum` 相等（`7f5b83c1bf214aead860e3b167cef0cbbbbe242086b3229f77c39c6fc35ddc26`）。

## 头号发现：装置改动之后仍有第二个截断来源（不在原登记预期之内）

5 轮里第 1–3 轮的 `singlefs_timing` 行本身被虚机关机时内核打印的一行 dmesg 从中间截断，例如第 1 轮：

```
E7RESULT name=singlefs_timing configuration=singlefs round=1 write_path_nanoseconds=19018861 second_transaction_nanoseconds=6424304 recovery_nanoseconds=7797794145 second_transaction_writes_both_devices=21 second_transaction_written_bytes_both_devices=344576 second_transaction_barriers_both_devices=4 second_transaction_[    8.423276] reboot: Power down
```

`second_transaction_outer_contains_inner`、`second_transaction_inner_nanoseconds`、`second_transaction_force_unit_access_writes_both_devices` 等字段落在截断点之后，读不到（第 2、3 轮截断点相同，内容略有出入）；第 4、5 轮该行完整。

- 这不是登记第十四节要修的那个来源（转打阻塞造成的打印积压）：装置改成先读到 EOF 再转打之后，`singlefs_inner` 行之间没有再出现几十毫秒级的积压（见下面 Q4）。这是另一个来源——虚机关机（`reboot()`）那一刻，guest 内核自己往同一个串口写 dmesg，与用户态最后一次 `println!` 竞争同一个 tty，字节被交错打断。装置读法怎么改都管不到这一步。
- 独立复算（用同一轮 `inner=second_transaction` 行的 `at_nanoseconds` 与它自报的 `nanoseconds=` 相减，不依赖被截断的 `singlefs_timing` 行）确认：5 轮外层减里层分别是 52 540、65 020、65 211、57 621、48 330 ns，全部为正——**物理上包含关系 5 轮都成立，只是被截断的 3 轮那一个字段读不到，不是关系不成立**。这一条不能当 Q1 的正式答案用（登记的读法只认 `singlefs_timing` 行的原样字段），只作区分记录。
- 3/5 出现在负载较高的前三轮、2/5 未出现在负载较低的后两轮，同一次跑负载单调降，样本太小说不清与负载的关系，不下因果结论。
- `.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」目前只记录了转打阻塞这一个来源，没有记这一个；已在 kb 实验页里写明，是否要另开一条 kb 记录交主 agent 定（写范围不含 `vm-harness.md`）。

## Q1（问题单第一行）：够判，走「过半不确定」路径

逐轮 `second_transaction_outer_contains_inner`：第 1–3 轮字段不存在（截断）、第 4、5 轮 `true`。5 轮里 0 假、3 不确定、2 真；不确定 ≥ 3（过半）⇒ 按登记第十四节，外层「第二个事务 ms」这一格整格不报。汇总行原样：

```
E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_milliseconds rounds=2 median=6.629 minimum=6.409 maximum=6.850 spread_percent=6.7 stability=stable values=4:6.850,5:6.409
```

A3 独立复算（外层=同轮 `second_transaction` 行 `at_nanoseconds` 减 `transaction` 行 `at_nanoseconds`；里层=`second_transaction` 行 `nanoseconds=`）：5 轮外层分别 6424304/6371843/6527894/6849546/6408654 ns，与可读到的第 4、5 轮 `singlefs_timing.second_transaction_nanoseconds` 逐位相等（6849546、6408654），S5 不触发。

## Q2（问题单第二行）：不够判

够判条件是 `singlefs_second_transaction_inner_milliseconds` 汇总行 `rounds=5`；实际因同一处截断只收 2 轮：

```
E7RESULT name=summary configuration=singlefs metric=singlefs_second_transaction_inner_milliseconds rounds=2 median=6.576 minimum=6.360 maximum=6.792 spread_percent=6.6 stability=stable values=4:6.792,5:6.360
```

B3 独立复算（第 4、5 轮 `second_transaction_inner_nanoseconds` ÷ 1e6 取中位/最小/最大/离散）：6.791925、6.360324 → 中位 6.576、最小 6.360、最大 6.792、离散 6.6%，与上面汇总行三位小数/一位小数逐字相等，S5 不触发。**只能报这 2 轮的数，不能代表 5 轮。**

## Q3（问题单第三行）：够判，5 轮与第三、四次正式跑逐字相同；C2 有一处与登记锚点不符（非本轮回归）

- C1（段序列）、C7（根）、C8（`content_matches`）、C9（逐盘 8 字段）：5 轮与旧两份产物（`e152-file-system-benchmark-second-transaction-2026-09-17.out`、`…-final-2026-09-17.out`）共 10 轮逐字相同，均为 `16+2+1+2`、`1:4`、`true`、device_0(writes=10,bytes=172032,fua=0,barriers=2)/device_1(writes=11,bytes=172544,fua=1,barriers=2)。这些字段全部来自 `inner=second_transaction`/`inner=recover_cold` 原始行，不受截断影响，5 轮齐。
- C3（写请求 21）、C4（写字节 344576）、C5（屏障 4）：来自 `singlefs_timing` 行但排在截断点之前，5 轮均可读、均与旧产物一致。
- C6（FUA 写=1）：仅第 4、5 轮可读（截断点之后），与旧产物一致；第 1–3 轮因截断缺字段，不能判等/不等，只能标「缺」。
- **C2（操作数）5 轮均为 23，不是登记引的锚点 21**（登记第十二节引文「操作数 21」、第六节 A-2、第七节同）。核实：第三、四次正式跑的旧产物里 `inner=second_transaction` 行的 `operations=` 同样是 23（两份产物 10 轮全部 23，非本轮新出现）；kb 实验页「这几个数说明什么」一节已有解释（「操作数 23（21 次写加 2 次屏障计数的口径见二进制）」）。`git log --since='2026-09-17 06:40' -- crates/singlefs-core/src/transaction.rs crates/singlefs-core/src/allocator.rs` 有一条改动（`cae5092`），但操作数 23 在这条改动**之前**的第四次跑产物（时间戳 2026-09-17 06:40）里已经存在，不是它带来的回归——按 F4 处置条款，判定为「条款的锚点本身错」，不是「crates/ 改了发布 B 的路」，不作废这 5 轮。**登记锚点 21 与实现 23 的差异是否要改登记，交主 agent 定**（按定义我不许自己改判据）。

## Q4（问题单第四行）：够判，5 轮全部满足

D1（第一条 `inner=segments` 到 `inner=transaction` 行跨度）：81 240、78 121、86 351、86 810、80 301 ns，**全部 ≤ 1 000 000 ns**。D2（行数）5 轮均为 13，与登记 B-3 新锚点一致（B-3 算术：`5+2+2+1+2+1=13`；旧产物是 8 行，登记第三节已预判两版行数不同并接受今天的 13 为准，不触发 S4）。D3（相邻最大间隔）：22 950、22 630、23 540、22 480、21 200 ns，与登记第十四节冒烟跑「24 µs」量级一致。D4 不适用（D1 无触发轮）。

