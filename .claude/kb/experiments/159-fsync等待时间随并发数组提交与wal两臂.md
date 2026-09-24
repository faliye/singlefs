## E159 fsync 等待时间随并发数：组提交与 WAL 两臂 —— 装置已写、计时未跑（2026-09-24，接续会话；纯计数部分确定性、54 单测 / 14 条变异全抓；正式计时的主网格、PC1–PC3、AA、G8、G1、G6 五轮尚未跑）

**问题**：跑前登记 `research/prompts/e159-preregistration.md`——岔路单 `research/prompts/m2-fsync-wait-forks.md` 第 1 行「甲（今天的做法：每次 fsync 写脏叶加全部祖先，按 D16（发布语义） 已定项 10 攒批、组提交）的等待时间可不可以接受」，量「一次 fsync 从发出到返回等了多久」在甲与两条 WAL 臂（wal_full-K10、乙-M-K10，两条都留、各判各的）上随并发数 k 的走法。装置写在 `research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs`（独立手写模型，不是入库装置——`crates/` 里没有组提交，第三节 2.3 现查零命中）。量延迟用真跑（真机、真 O_DIRECT 设备、真线程），不用计数模型换算，理由见跑前登记「五、5.1」。

**这一段（接续一个撞了会话限额而中断的执行员）只做到「装置与冒烟」**：正式计时（本机口径下的 5 轮主网格、PC1–PC3、AA、G8、G1、G6）没有跑——机器上这会儿有并发编译，计时会被污染（`.claude/kb/tooling.md`「仓里的性能数都只在本机成立」同一类顾虑），等机器空闲另派。

### 这一段做了什么

1. **装置**：从 `research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs` 第 36–2058 行原样拷来常量、K1–K4、5.3 组公式与三条臂的单发/批量函数（跑前登记停机条款 S2 要求这段与源文件逐个 diff 为空，已核，见「复跑」）；新写组提交队列（`SharedQueue`、`drain_batch_from_queue`）、WAL checkpoint 线程（`checkpoint_loop`、`should_trigger_checkpoint`）、闭环调用方（`caller_loop`）、O_DIRECT I/O（`RealBlockDevice`）、结构性单测用的假设备（`RecordingBlockDevice`、`SleepingBlockDevice`）、统计（分位数、Mann–Whitney A、规则 R、J1/J2a/J4 判定）、PC1 忙等注入、`cell`/`probe-device`/`anchors` 三个子命令。
2. **停机条款 S1（锚点格与 `crates/` 逐项对拍）**：单测钉死甲在 F1、P=1、k=1 的单盘总字节 172544、写调用 11 次（`jia_single_fsync_batch_matches_crates_test_table`，与第三节 `crates/` 测试表逐项加总一致），发布次序单元→屏障→记录→屏障→根槽 FUA→系统配置槽（先读两次、再写一次）逐步核对（`jia_single_fsync_batch_step_sequence_matches_crates_order`），根槽持久时刻不早于根槽 FUA（`jia_durable_instant_is_not_earlier_than_the_root_slot_fua`）。S1 不触发。
3. **停机条款 S2（拷贝块 diff 为空）**：`diff <(sed -n '39,2061p' e159_...) <(sed -n '36,2058p' e155_...)` 退出码 0。
4. **停机条款 S3（来宾 `logical_block_size`）**：真机 VM 探针 `probe-device` 子命令读回 `logical_block_size=512`，与主几何 `physical_block_size_bytes` 常量一致，不触发。
5. **单测与变异**：54 个单测全绿（`cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit`）；变异表 `research/mutations/e159_fsync_wait_group_commit.tsv` 14 条（M1–M14，覆盖第九节全部目标：甲发布去掉第二道屏障、第一道屏障后就放行、WAL 臂写混规划函数、组提交开也把批封顶为 1、`t_issue` 漏排队段、PC1 忙等挪到放行之后、随机取整改成向下取整、`T_time` 常量读错、系统配置槽写前不读、根槽宽度改错、J1 边界比较符改错、五轮合并改成多数轮说了算、开环 `issued_at` 改取实际发出时刻、`release_to_wake` 漏唤醒段），`mutate.sh` 跑两遍（源码改动前后各一遍）都 14/14 全抓、收尾「已还原，基线仍全绿」。M3–M6（组提交队列与真并发时序相关的四条）原本没有单测覆盖真并发引擎，这一段新增 6 条单测补齐：`plan_batch_for_arm_routes_each_arm_to_its_own_solver_not_a_different_one`（M3）、`drain_batch_from_queue_collects_everything_when_group_commit_is_on` / `_collects_only_one_when_group_commit_is_off`（M4）、`closed_loop_queueing_delay_is_counted_when_group_commit_is_off`（M5，用 `SleepingBlockDevice` 固定屏障延迟逼出排队，不需要真磁盘）、`pc1_injected_wait_lands_before_release_not_after`（M6，同样用 `SleepingBlockDevice`）、`checkpoint_time_threshold_constant_is_five_seconds`（M8，把 `T_time` 常量提成 `CHECKPOINT_TIME_THRESHOLD` 具名常量，`run_cell_subcommand` 与单测共用同一个落点，不再各写一份字面量）。
6. **真机冒烟撞见一个真 bug，已修复**：`RealBlockDevice::read_at` 原实现把数据读进一块普通 `vec![0u8; length]`（不对齐）再传给 O_DIRECT 的 `read_exact_at`，第一次真跑（`--arm jia --k 1024`）当场 `panicked ... EINVAL`；这条路径此前 0 个单测覆盖到（全部旧单测只用 `RecordingBlockDevice`/`SleepingBlockDevice`）。改法：直接把数据读进按 `ALIGNMENT_BYTES` 对齐的 `AlignedBuffer`（新增 `AlignedBuffer::as_mut`），读完再拷出返回值；修复后同一条命令换成 k=4 不再 panic。产物见「结果整行抄自产物」。
7. **真机冒烟：三条臂各跑通一格**（`VM_DISK_PREALLOCATE=1`，短 duration，只为验证装置真跑不崩、不为出正式数）：甲 k=1、wal_full-K10 k=2（duration 4s，未到 `T_time`=5s 与 `T_dirty`=256MiB，checkpoint_count=0，符合预期）、乙-M-K10 k=2（duration 6s，触发一次 checkpoint，checkpoint 机制真跑通过）。三条臂、组提交引擎、WAL checkpoint 线程都验证过端到端能跑通，没有再撞到新的 panic。
8. **`print_anchors` 重构**：改成经 `Emitter` 发 `E7RESULT` 行、带 `name=done emitted=N` 收尾（此前是裸 `println!`，不满足 `research/scripts/replay.sh` 的完整性闸），并把甲那一行的单盘字节算法从 `outcome.total_bytes() / DEVICE_COUNT`（对根槽字段是错的近似，会把单盘专属的 512 字节根槽也腰斩）改成与单测共用的 `jia_outcome_single_device_bytes`（挪成模块顶层函数），输出从 172288 改回正确的 172544，与 B4 锚点一致。
9. **命名纪律**：`naming-lint.sh` 对这个文件全程通过（唯一一条是 `naming-lint:external` 标注过的 `O_DIRECT` 外部标志名，不算违规）。

### 结果整行抄自产物

`research/results/e159-fsync-wait-group-commit-2026-09-24-anchors.out`（纯算术、逐字节可复跑，见「复跑」）：

```text
E7RESULT name=anchor label=jia_p1_f1_k1_single_device_bytes value=172544
E7RESULT name=anchor label=write_ahead_log_full_k10_p1_f1_k1_single_device_bytes value=102400
E7RESULT name=anchor label=yi_m_k10_p1_f1_k1_single_device_bytes value=86016
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k1_single_device_expected_bytes value=365346.000
E7RESULT name=anchor label=write_ahead_log_full_k10_f8a_p10000_k16_single_device_expected_bytes value=4313362.000
E7RESULT name=done emitted=6
```

`research/results/e159-fsync-wait-group-commit-2026-09-24-smoke.out`（真机冒烟存证，不参与 `replay.sh` 的逐字节比对——真延迟每次跑都不同——只证明装置真跑得通；完整上下文见该文件）：修复之后三条臂各一格的 `E159RESULT` 行，例如乙-M-K10 那一格：

```text
E7RESULT E159RESULT arm=yi-M-K10 k=2 group_commit=true samples=37 median_ns=193801295 p99_ns=854997334 containment_violations=0 checkpoint_count=1 checkpoint_bytes=434688 bytes_written_estimate=12751360
```

`containment_violations=0`（V5 包含自检没有违反）、`checkpoint_count=1`（WAL checkpoint 机制真触发过一次）。

### 复跑

```
bash research/scripts/replay.sh E159
```

只登记了 `anchors` 子命令这一行（纯算术、不碰真设备、逐字节可复跑）：`E159|e159-fsync-wait-group-commit|anchors|e159-fsync-wait-group-commit-2026-09-24-anchors.out|exact`，本次交回前跑过、判定「字节一致」。真机冒烟（S3 探针与三条臂各一格）不登记进 `replay.sh`——延迟数每次跑都不同，没有能钉住的逐字节判据；复跑命令原样存进 `research/results/e159-fsync-wait-group-commit-2026-09-24-smoke.out` 文件头，需要重跑时手动执行，例如：

```
cargo build --release --target x86_64-unknown-linux-musl --bin e159-fsync-wait-group-commit -p e7-index-bench
bash research/scripts/vm-bench.sh <musl 二进制路径> probe-device --device /dev/vda
VM_DISK_PREALLOCATE=1 bash research/scripts/vm-bench.sh <musl 二进制路径> cell --device /dev/vda \
  --arm yi-m --k 2 --group-commit on --warmup-seconds 0.3 --duration-seconds 6 --seed 5 \
  --file-count 10000 --family f8a
```

单测：`cargo test --release -p e7-index-bench --bin e159-fsync-wait-group-commit` → `54 passed; 0 failed`。变异：`cd research && bash scripts/mutate.sh e159-fsync-wait-group-commit e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs mutations/e159_fsync_wait_group_commit.tsv` → 14/14 全抓，收尾「已还原，基线仍全绿」（跑了两遍，源码改动前后各一遍，结论不变）。

### 它答不了的

- **岔路单第 1 行的翻面观测还没有答案**：正式计时（5 轮主网格 × 3 臂 × 8 个 k、PC1–PC3、AA、G8、G1、G6）一格都没跑，J1/J2a/J4 三句判据都还没有值。这一段只证明了装置能真跑通、字节口径与 `crates/` 对得上、变异表全抓。
- **S4 写量预算（跑前登记 5.7）没有算清**：只有零散的冒烟读数（例如甲 k=4、duration 3s 约写 58.5 MB，折合约 17.7 MB/s，但这是在稀疏盘首次触碰扇区、没做「测量之前整盘写一遍」那一步的读数下量到的，真实预热之后的吞吐可能明显更高、真实字节总量因此可能被这个数低估）；没有按第一段全部格子（主网格 24 格 + PC1/PC2/PC3/AA/锚点 + G8 24 格 + G1 12 格 + G6 8 格，每格 11s 或 2s，5 轮）逐格累计算出精确的预算总量，也就没有确认 1 TiB 那道停机线会不会触发。下一段跑正式计时之前要先把这笔账算清、或者先做一次预热盘上的代表格实测再推算。
- **「测量之前整盘写一遍」（跑前登记 5.3）这一步还没有实现进装置**：本段冒烟只用 `VM_DISK_PREALLOCATE=1`（`fallocate` 预分配，不是整盘写满），第一次不带这个参数的冒烟因此撞见 samples=0（不是 bug，是稀疏盘首次触碰扇区的宿主 ext4 元数据开销），修复 read_at 之后没有验证过「整盘写满一遍」这一步本身。
- **G8 开环泊松到达、G1 互不共享、G6 `T_time=200ms` 三个反向取样点都还没跑**，第八节的「方向相反的取样点」判别力自证因此也没做。
- **PC1/PC2/PC3/AA 四个阳性对照与噪声地板都没有正式跑过**（只有非正式的冒烟：乙-M-K10 那一格触发过一次真 checkpoint，证明 checkpoint 机制通，不是 PC 系列的正式判定）。

### 影响的决策

这一段只交装置与冒烟，没有计时结论；下面每条分项的「依据」段目前都没有引用 E159（逐条 `grep -n "E159（" .claude/kb/decisions/<文件>` 零命中），按格式演进纪律都记「备料」，等正式计时跑完再回看要不要升级成支撑或推翻。

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 1 | 备料 | 2026-09-24 不受影响：`grep -n "E159（" .claude/kb/decisions/23-journal的角色与格式.md` 零命中；这一段只验证装置的批与字节口径与该分项定案的「甲」形态一致，没有新的代价数 |
| D16（发布语义） 已定项 2 | 备料 | 2026-09-24 不受影响：`grep -n "E159（" .claude/kb/decisions/16-发布语义.md` 零命中；checkpoint 触发判据 `should_trigger_checkpoint` 照抄该分项的 `∨` 式子实现，冒烟只验证机制真触发，没有新的可量的量 |
| D16（发布语义） 已定项 5 | 备料 | 同上文件零命中；`T_time`=5s、有效 `T_dirty`=256MiB 两个常量在装置里对拍过（`checkpoint_time_threshold_constant_is_five_seconds`），没有产生新的代价数据 |
| D16（发布语义） 已定项 7 | 备料 | 同上文件零命中；S1 核对了发布持久顺序与 fsync 返回条件，没有改变这条已定项 |
| D16（发布语义） 已定项 10 | 备料 | 同上文件零命中；这是这个实验要量代价的那条政策本身（攒批组提交），量代价的正式计时还没跑，因此还谈不上支撑或推翻 |
| D16（发布语义） 已定项 11 | 备料 | 同上文件零命中；装置的两条 WAL 臂与甲共用同一个 journal/checkpoint 节奏，没有另开一条日志格式，符合这条已定项的判据，未产生新代价数 |
| D25（目标负载优先级） 已定项 7 | 备料 | 2026-09-24 不受影响：`grep -n "E159（" .claude/kb/decisions/25-目标负载优先级.md` 零命中；这条已定项「欠」的那笔账（C488）正是这个实验要还的，装置已写、冒烟已通，但代价数（并发 fsync 下等待多久）还没量出来 |
