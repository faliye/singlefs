# task-server 的 GPU 与 CPU 调度溢出审核（2026-11-11）

读现行源码，没有改调度，也没有在两台上复现一轮真实排队。行号对的是这一天的 `task-server/task_server.py` 与 `scripts/hosts/host-dispatch.py`。早前记录（`records/2026-11-01-task-server深入审核.md`、`records/2026-11-07-task-server代码审核.md`）里的取消、分片、写回旧尝试不在这一次的范围里。

结论：一张卡不会分给两条在跑的任务。溢出在两处账上。CPU 闸只把刚起 5 秒的尝试按线程数记上，之后只看此刻跑得动的进程数，清场和等 IO 的任务不占这个数，后面的 31 线程任务还能继续起。声明 `all` 的分卡把内存放不下的台上的空卡算进分子，一条任务可以从唯一放得下的那台上拿走超过公平份额的卡。显存预测固定 8192 MiB，比的是整台各张卡加总，不按这一条实际分到的卡扣。

| 编号 | 问题 | 位置 |
| --- | --- | --- |
| CPU-1 | 满 5 秒后不再记已放进的 31 线程，清场或 D 状态期间闸继续放行 | `task_server.py` 的 `cpu_problem`（8667–8685）、`CPU_SETTLE_SECONDS`（826） |
| CPU-2 | 核数或负载读不到时 CPU 闸放行 | 同上，8675–8676 |
| GPU-1 | `all` 的分子含内存放不下的台的空卡 | `all_cards_group`（8690–8703） |
| GPU-2 | 显存按整台合计比，同轮已记的 `gpu_booked` 不扣 | `judged_on_host`（7885–7892）、`host-dispatch.py` 的 `judge_host`（372–375） |

## 守住的部分

卡的互斥是按卡的键记账的。`busy_cards`（7043–7052）收在跑尝试记下的卡，以及按需领卡持着和正在清场的卡。`place_queued_into` 起完一条就 `busy.update(card_keys(placement.card))`（7357），同一轮后面的挑卡看见这份集合。排队分卡和领卡分卡共拿 `card_allocation_lock`。按需领卡现在一回一张（`card_lease_choice`，6951–6955）。

内存没有对应的洞：在跑的尝试按上限记账，这一轮起了的经 `PassView.book` 加进后面几条的待涨量。

## CPU-1：5 秒之后线程额度不算

`cpu_problem` 放行的条件是：

```text
max(1 分钟负载, procs_running) + 起了不到 5 秒的尝试的线程之和 + 这一条的线程
    ≤ 核数 × CPU_OVERSUBSCRIPTION（3）
```

每条尝试的线程数是登记的重活线程数，`.claude/heavy-thread-default` 现在是 31。5 秒内这条加法成立：`start_attempt` 把 `threads` 和 `started_at` 写进库，下一条读 `running_attempts` 能加进去。核数 32 时上限是 96，负载接近 0 时一轮里大约再放进 3 条（3×31=93，再加第 4 条就过 96）。

满 `CPU_SETTLE_SECONDS`（5 秒）之后，这 31 不再加。闸只看 `/proc/loadavg` 的 1 分钟负载和 `/proc/stat` 的 `procs_running`（`memory_peaks_lib.py` 的 `cpu_report`）。下面两类尝试此刻几乎不增加 `procs_running`，闸把机器当成还空着：

- 要卡的任务最外层套 `scripts/crash/gpu-quiesce-run.sh`。清场锁默认可等 900 秒（`GPU_QUIESCE_LOCK_SECONDS`），这段时间 31 个工作线程还没起来。
- 进程在 D 状态等 NFS 或盘。卡住判法对「每一轮都有进程在 D 状态」不停任务，这些尝试一直留在 `running`。

它们醒来时，在跑的线程是条数乘 31，可以超过核数 × 3。注释自己写了把刚起的线程记久了会把短任务挡死，所以窗口停在 5 秒。窗口之外没有「这台在跑尝试的线程之和」这一笔。

## CPU-2：读不到就放行

`cpus` 为空，或 `load1` 和 `procs_running` 都没有，`cpu_problem` 交空串，这台不再设 CPU 上限。`os.cpu_count()` 交空、两份 `/proc` 都读失败，都走这里。内存那一侧量不到余量是这一轮不派。

## GPU-1：`all` 的分子把放不下的台算进去

`all_cards_group` 的张数是：

```text
min(这台空卡数, ⌈各台空卡总数 ÷ (1 + 后面还要卡的条数)⌉)
```

分子是 `free_by_host` 里每一台的空卡（8702），包含内存判决 `fits` 为假的台。分母只数队列里排在后面、起跑就要卡的条数（7279–7280）。放得下的台才进入 `fitting`，切片只切选中的那一台。

例：本机 4 张空卡、内存放得下；第二台 4 张空卡、内存放不下；后面还有 1 条要卡、也只能用本机。公平份额是 ⌈4/2⌉ = 2。分子现在是 8，⌈8/2⌉ = 4，这一条拿走本机 4 张，后面那条没有卡。两台都放得下时，分子里的卡后面的任务还拿得到，偏的是「这一台分出去的张数」，不是把同一张卡分两次。

## GPU-2：显存不按分到的卡扣

`judge_host` 把这台报告里每张卡的总量加在一起（清场登记里有活着的持有者时改用每张的空闲），减去 `GPU_MARGIN_MEBIBYTES`（2560），再减去 `pending_gpu`。`judged_on_host` 在弄坏开关 `card-pending-counts-busy-cards` 没开时把 `pending_gpu` 传 0。同轮 `view.book` 加进去的 `gpu_booked` 不参与这次比较。

预测在 `computed` 那一支写成 `DEFAULT_GPU_PREDICTION_MEBIBYTES`（8192），不随张数变，也不随实测显存峰值变。两张 16GB 卡的合计远大于 8192，这条显存比较不会因为「这一条只能拿到剩下的一张」而拒绝。跨任务叠到同一张卡上，靠的是上面的卡键互斥。一条任务在自己分到的卡上用满配置额度，这笔 8192 的账不拦。
