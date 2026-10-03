# 正推腿的模型与脚本（m3-unmount-after-short-continuation-r1）

- `probe_unmount.rs`：副本里的临时 harness 测试（`crates/singlefs-harness/tests/probe_unmount.rs`），一格 = 盘型 + 先覆盖写次数 O + 文件单元数 U + 崩在意图后 B 批 + 删除批大小 K；
  造盘（补足的小盘 `Pad`、不补足的窄盘 `Narrow`、准入判着的第一个大文件 `First`）、直发意图与 B 批、丢会话、重新可写挂载、看续做结局，
  再在盘面拷贝上试：正常卸载、卸载后再挂、只写系统配置里新 F 再挂（候选乙新）、各自循环到续做走空。
- `transaction-rs-reserve-prototype.patch`：候选甲的原型（删除类发布的落点那一判另留 `PROBE_UNMOUNT_RESERVE` 个槽；取 `ckpt3` 时 = 3 × ckpt_cost），
  对冻结副本 `crates/singlefs-core/src/transaction.rs` 的 `diff -u`，只改副本。
- `run-probe.sh` 起一个测试进程（内存包装 + 线程上限 + nice）；`run-par.sh <批号> <环境变量前缀> <行文件>` 每行起一个进程；`lines-*.txt` 是用过的行文件。
- `summarize.py`、`tabulate.py`：把 `outputs/*.out` 的 CELL 行压成表。
- `outputs/`：原样输出，只把 `grow-refused` 那种带整段 Debug 的行截成 400 字符；`base1-*` 是原代码（甲的预留取 0），`r12/r18/r24/r24c/c3-*` 是甲取 12 / 18 / 24 / 24 / 3×ckpt_cost，
  `chk-*`、`jchk-*` 带池级 checker 判，`first-*` 是准入判着的第一个大文件（全部被拒在 `ContentExceedsDataUnit`），`jreg-*` 是甲的回归格。

复跑：把冻结副本拷一份（`rsync -a <冻结副本>/ <草稿>/tree/`），把 `probe_unmount.rs` 放进 `tree/crates/singlefs-harness/tests/`，`transaction-rs-reserve-prototype.patch` 用 `patch -p0` 打到拷贝的
`crates/singlefs-core/src/transaction.rs`（补丁头里是绝对路径，要改成拷贝里的路径）；`cargo test --offline -p singlefs-harness --test probe_unmount --no-run` 编，`run-probe.sh` 里的 `cd` 与 `BIN` 要改成你的草稿目录；
格由环境变量 `PROBE_GRID=custom PROBE_DISKS=… PROBE_O=… PROBE_U=… PROBE_B=… PROBE_K=…` 给（盘名 `pad384` / `pad256` / `pad240` / `n<槽数>` / `f<槽数>-<单元数>`），候选甲的预留由 `PROBE_UNMOUNT_RESERVE`（数字或 `ckpt3`）给。
内存包装要求 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 与 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON`（run-probe.sh 里写了我用的理由，换会话请改）。
