## E202 GPU与CPU两种模式GC对照组 —— 跑前登记已写、装置未跑

**中文** · [English](202-gpu-vs-cpu-gc-comparison.en.md) · [日本語](202-GPUとCPUの2方式GC対照群.ja.md)

跑前登记 `research/prompts/e202-preregistration.md`。回答问题单 `research/prompts/m4-gpugc-compare-questions.md`：比较同一负载、同一池下 `background_compute=cpu` 与 `background_compute=gpu` 两种模式的 GC 挂钟、占用的主机 CPU 时间与省下的时间、前台 fsync 延迟分布，以及省下的 CPU 时间随盘容量的变化。

装置 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs`（入库装置，住在产品 GPU 的测试包 background-gpu-tests 里）。**这一页只记跑前登记与装置骨架已写，产物一次没跑。**

### 现状

- 跑前登记已写（2026-11-03），装置骨架已立，产物一次没跑。
- 第一段的几何核起过一次、没跑完、没留产物：造起步池太慢，照量到的速度过不了任务服务的四小时上限。量到的：2288MiB 档顺序写那一半约 34 分钟，交错写那一半约 60 秒做 95 步、共 18304 步；8GiB 档 2100 秒做到顺序写 18292 步、共 65536 步；三档同时造，常驻内存 2100 秒时 9.4 GiB。用户定这一次不跑，等造池提速做完再跑。
- 盘容量采用定占用读法（容量涨，活数据占单元区比例固定 50%）。
- 装置约束：GPU 模式每张卡上限 1 GiB、按卡的顺序填满；一张卡与多张卡各跑；判定逐条与 CPU 参照相同才计一组数。
- 多张卡那一臂用几张卡由 `lockstep --cards <张数>` 定（给 0 或 1、或多于服务交来的张数退 2；不给就用服务交来的全部卡），产物里 `name=every_card_arm_cards` 那一行记 `cards_named` 与它是怎么定的（`named_by=cards_option` 或 `every_handed_over_card`）；续跑时张数与要续的产物不同，打 `name=stopped clause=cards_option_differs_from_the_product_being_resumed` 退 2。

### 复跑

```bash
# 入库装置编译与运行归主 agent；经 task-server 调度
task-server/task submit -- cargo run --release -p singlefs-background-gpu-tests --features tests-on-a-real-gpu --bin e202_gpu_cpu_gc_comparison
```

### 口径

- 原始输出未留存：实验未跑。
- 一次整理：一次跑里 GC 线程从第一次调后台入口到最后一次调用返回为止做的全部整理。
- GC 挂钟：GC 线程上每一次后台入口调用前后读 `std::time::Instant` 之差相加。
- GC 的主机 CPU 时间：每次 GC 调用前后读 `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` 之差减去同一窗口前台线程增量。
- 省下的 CPU 时间：同一档同一次重复里 CPU 臂的进程口径减 GPU 臂的进程口径。
- 重复次数：每档每臂 5 次，报中位、最小、最大与中位绝对偏差。

### 判据与失败条款

- 判据：每一档容量上「CPU 模式减 GPU 模式」的差有数、有正负、有离散度；阳性对照下省下的 CPU 时间必须少于正常对照。
- 失败条款：若 GPU 模式省下的主机 CPU 时间在各容量档位均恒 ≤ 0，或前台 fsync 延迟劣化超限，则触发失败条款。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 备料 | 2026-11-14 不受影响：这一轮改的是装置搬进测试包与起法、多张卡那一臂的张数定法，判据、臂与口径没改，产物一次没跑，仍是跑前登记已写、装置未跑 |

## 历史版本

### 2026-11-13

- 第一段的几何核起了一次（任务 16507），造起步池太慢、内存三次撞上限从头重来，主 agent 取消；用户定这一次不跑，等造池提速。没有产物。
- 装置从产品 GPU 包整份搬进它的测试包 `crates/singlefs-background-gpu-tests/src/bin/`，起法换成 `-p singlefs-background-gpu-tests --features tests-on-a-real-gpu`；多张卡那一臂的张数改由 `lockstep --cards <张数>` 定，产物多一行 `name=every_card_arm_cards`（用户定案原话在 `records/2026-11-11-GPU两段与按需领卡核查.md`）。判据、臂与口径没改，产物一次没跑。

### 2026-11-03

- 建档，登记 E202（GPU与CPU两种模式GC对照组）跑前登记与装置骨架，未正式跑。

