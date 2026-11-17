---
name: queue-health
description: 按任务号现算内存时用它——要知道某条或全部在跑任务此刻到底占多少内存与显存、占上限多少、两台现算合计占在跑上限之和多少时，或任务还在不在时，调 scripts/hosts/queue-health.py，不手敲 ps、free 现量，也不拿 task status 的峰值或日志里的内存行当占用。
---

## 怎么调

主 agent 在 Bash 里直接跑，不经 `task-server/task submit`（排队就量不到此刻）；子 agent 不跑，写进报告交主 agent。

| 要看什么 | 命令 |
|---|---|
| 全部在跑的任务 | `python3 scripts/hosts/queue-health.py` |
| 点名的任务 | `python3 scripts/hosts/queue-health.py <任务号>…` |
| 一段时间里的尖峰 | `python3 scripts/hosts/queue-health.py <任务号>… --watch <秒> [--interval <秒>]`（两台各自按间隔重算，留最大的那一轮） |

- 合计的口径：逐进程读 `/proc/<进程号>/smaps_rollup` 的 `Pss_Anon` + `Pss_Shmem`（写时复制页与共享内存按映射它的进程数分摊，整棵树加起来就是这棵树真占的）加 `VmPTE`。读不到 `smaps_rollup` 的进程退回 `RssAnon` + `RssShmem`（共享页各算一份，可能偏大），行里写「其中 N 个读不到 smaps_rollup」，结论里有一条「读不到 smaps_rollup」。
- 每条任务每台一行：号、第几次、哪台、合计、进程数、`memory.max` 与占比、cgroup anon + shmem、显存，下一行是按进程名分的前几项。派到第二台的任务在本机另有一行外壳（尝试单元里的进程），那一行没有上限，看第二台那一行。
- 退出码：0 都对得上；1 有要看的结论；2 用法错；3 有读不到的；78 两台都读不到。
- 末行以「结论：」起头，逐条判据在「── 结论」一段。
- 一次现算会漏尖峰，看尖峰用 `--watch`。排队原因看 `task-server/task status <号>` 的「在等：…」，这个脚本不判。

## 每个结论怎么处置

| 结论 | 处置 |
|---|---|
| 此刻找不到进程 | 不拿别的数顶。核号；`task-server/task status <号>` 看它还在排还是已有结局 |
| 现算与 cgroup 对不上 | 先看判据里有没有「其中 N 个进程读不到 smaps_rollup」：有，进程合计偏大是共享页各算一份，以 cgroup 那个数为准。进程合计偏小：tmpfs 文件、已退出进程留下的页，或与 cgroup 外进程共享的页有一部分记在外面，看那个 cgroup 的 `cgroup.procs`。偏大又没有走退路的，多半是读几百个进程的这段时间里内存在涨，再跑一次 |
| 占用贴近上限 | 不用手动处置，撞了由服务放大重排；连着撞的核代码或收小范围 |
| 现算合计超过在跑上限之和 | 先看有没有「读不到 smaps_rollup」的进程（共享页各算一份），再对着 slice anon + shmem 看 |
| 现算合计占在跑上限之和 N%，没有超过 | 只是比例，不用处置，也不据此判上限记得对不对；每条占上限的比例看各自那一行 |
| 读不到 smaps_rollup | 只提醒：这几个进程按 `RssAnon` + `RssShmem` 算，合计可能偏大；合计偏大时以 cgroup anon + shmem 为准 |
| 占用都在上限以内、现算与 cgroup 对得上 | 不用处置 |
| 读不到 | 第二台那一行写着原因，修好再跑；读不到的那一台不算没有进程 |
| 此刻没有在跑任务的进程 | 不用处置 |
