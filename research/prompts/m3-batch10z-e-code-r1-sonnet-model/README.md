# 第 10z 批第五块代码轮第一轮：正推腿的模型

都只在草稿目录 `~/.local/share/singlefs/work/m3-batch10z-e-code-r1-sonnet/` 的冻结副本拷贝上跑；入库的是源码、脚本与输出，编译目录与副本不入库。
冻结副本被判的源文件与 `research/prompts/m3-batch10z-e-code-r1-snapshot/crates-sha256.txt` 逐项对上（`sha256sum -c`：334 项 OK、0 项不是 OK）。

- `probes/rounds_new.rs`：上一轮攻方 `attack_10zc_rounds.rs` 与正推 `rounds_sim4.rs` 改到新接口（`RoundsOfTheCards`、`StopOfTheCards`）：外层循环、卡线程、起下一张卡按冻结副本 `crash_amplification.rs` 的次序，用 harness 档的纯函数与类型搭；叫停是第 n 个事件出现的单调标志（`UNSTOP_AFTER=d` 时第 n+d 个事件又消失，模拟叫停文件被删）。放 `crates/singlefs-harness/tests/`。
- `probes/endings_new.rs`：攻方 `attack_10zc_endings.rs` 与正推 `explain_demo.rs` 改到新接口（`record_the_device_lost(why, cause)`、`device_when_the_work_returns`、`end_the_work_and_explain_its_failure` 多一个参数），加真显存不够的两个窗口与函数真值表。同一目录。
- `probes/killers_new.rs`：对 harness 档 `--lib` 里照绿的变异补的会红用例（真代码上绿，变异上红）。同一目录。
- `mut/mutants.py`：自造变异 40 条（原文在 `gpu_memory_ledger.rs` 测试段之前恰好命中一次）；`mut/run_mut.py <起> <止>` 逐条改、跑 `cargo test -p singlefs-harness --lib -- gpu_memory_ledger`（`FILT=` 空是整个 `--lib`）、还原；`mut/run_killers.py` 把 `killers_new.rs` 对点名的变异跑一遍；`mut/results.tsv` 是结果（前 40 行带过滤，后 9 行整个 `--lib`）。脚本里的路径是草稿目录的绝对路径，复跑要改。
- `mut/prove-red-unproven.log`：实现员报告里没证的 7 条 harness 档变异行（追加第 3、7、9、14、15、21、22 行）经 `prove-red.sh` 跑的原样输出。
- `wgpu-strings/`：独立小工程（`wgpu =30.0.1`，不起设备），打 wgpu 几个错误的原文；输出在 `logs/wgpu-strings.out`。
- `logs/`：各条的原样输出。

命令（工作目录是副本树根）：
`CARGO_TARGET_DIR=<编译目录> nice -n 19 bash scripts/run-with-memory-cap.sh 12G bash scripts/capped.sh 6 cargo test --offline -p singlefs-harness --test <名> -- --nocapture --test-threads 1`
