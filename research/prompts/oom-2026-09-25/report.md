# 2026-09-25 两次整机 OOM：原因（主 agent 现查）

## 结论

两次都是 E142 第十六次跑给独立比对程序 `research/e7-index-bench/src/bin/e142_region_diff_independent.rs` 跑变异表时，第 2 条变异 `M2_diff_segments_equality_flipped` 让单测无限分配内存；变异跑道 `research/scripts/mutate.sh` 只有 120 秒超时、没有内存上限，整机 60 GiB 十几秒被吃满，内核全局 OOM 先杀了本地模型服务 vllm-prod 的 ray 进程，Claude Code 会话在内存耗尽时断掉，最后才杀到这个单测进程。

## 证据

- 内核日志（`kernel-oom-0750-0810-utc.log`，`journalctl -k` 原样）：
  - `Sep 25 07:58:02 … Out of memory: Killed process 1691212 (e142_region_dif) total-vm:67315408kB, anon-rss:56156416kB`
  - `Sep 25 08:06:28 … Out of memory: Killed process 1902180 (e142_region_dif) total-vm:67315412kB, anon-rss:59381760kB`
  - 两次之前都先杀了一串 `ray::IDLE` / `ray::RayWorkerP`（`task_memcg=/system.slice/vllm-prod.service`，`oom_score_adj:1000`）；触发 OOM 的线程里有 `tests::comparin`、`tests::flipping`（这个比对程序的单测名前缀）与 `claude`。
- 第二次那一跑的变异日志（`mutate-independent-bin-second-run.log`，执行员草稿 `/tmp/claude-1000/e142-r16-s1/mutate-independent-bin.log` 原样，最后写入 08:06:31 UTC）：`💥 [M2_diff_segments_equality_flipped] 测试进程没跑完（signal 9）`。第一次那一跑的同名日志被第二次覆盖了；第一任执行员的变异表里同一条 M2 在第 2 行（交接摘要 `/tmp/claude-1000/handover/ac3a68b737fd2bf64.md` 第 398 行），被杀的是同一个二进制。
- 机理：M2 把 `diff_segments` 外层的 `if left.get(position) != right.get(position)` 翻成 `==`。遇到相等的字节就进「不同段」那一支，内层 `while` 条件立刻为假、位置不前进，推一个长度 0 的段，外层又回到同一位置——无限推段，`segments` 每次多 16 字节。
- 复现（`e142_region_diff_independent-with-M2.rs`，单文件 `rustc --edition 2021 --test -O` 编；主 agent 08:45 UTC 跑）：
  - 原程序在 `systemd-run --user --scope -p MemoryMax=1G -p MemorySwapMax=0` 下 3 条单测 0.00 秒过；
  - 打 M2 的程序只跑 `tests::flipping_one_hex_character_reports_exactly_that_region_and_segment`，0.21 秒吃满 1G、退出码 137，内核只杀它自己：`repro-cgroup-kill.log` 里 `Memory cgroup out of memory: Killed process 2864932 (m2_test) … anon-rss:1045504kB`。
  - 按这个速度约每秒 5 GB，60 GiB 十几秒满，120 秒超时来不及。

## 第二次是主 agent 造成的

第一次崩了之后主 agent 没查 `journalctl -k`，就派接手的执行员「照登记重跑步 ④」，它又跑了同一张变异表。

## 防再发

记在 `records/2026-09-16-subagent拆分提案.md` 第四十节第 28 行：变异跑道与门禁 59 号每条变异放进 cgroup 内存上限、撞顶单列一类；看门狗加进程常驻内存过线告警；会话启动 / 恢复时 hook 报近几小时内核 OOM 杀过的进程。
