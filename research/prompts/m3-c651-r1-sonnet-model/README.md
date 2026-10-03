# m3-c651-r1 正推腿的模型与探针（只读导入，不进门禁）

- `c651_probe.rs`：放进冻结副本 `/tmp/claude-1000/m3-c651-r1/tree/crates/singlefs-harness/tests/` 里的一份探针用例（我在副本的拷贝
  `/tmp/claude-1000/m3-c651-r1-forward/tree/` 里编译、跑过）。造 io_min 512 / 8192 的两盘池，把槽 0 弄坏，再各放一份旧池残留的槽，
  同一份镜像交给 core 的 `choose_system_configuration` 与 checker 的 `verified_system_configuration_slots`、`check_pool_image`（I-8.5），打印两边择到什么。
- `c651_probe.out`：上面那一份的原样输出（`--nocapture --test-threads=1`）。
- `count_slot0_b.py`：在 `research/e7-index-bench/src/bin/e170_o3_spec_executor/` 目录下跑（读该目录的 `crash_dump.py`、`crash_image.py`、
  `executor_core.py`、`system_configuration.py` 与 `research/target/e170-dumps`），数 E170 两条流每个取样态里「槽 0 每盘自证过几块、4096 处槽 1 每盘自证过几块」。
- `count_slot0_b.out`：上面那一份的原样输出。

跑法（不跑 checker 档）：
    cp -r <冻结副本> <草稿目录>/tree; cp c651_probe.rs <草稿目录>/tree/crates/singlefs-harness/tests/
    cd <草稿目录>/tree && bash research/scripts/capped.sh 8 cargo test --offline -p singlefs-harness --test c651_probe --no-run
    bash research/scripts/run-with-memory-cap.sh <上限> bash research/scripts/capped.sh 8 target/debug/deps/c651_probe-<哈希> --nocapture --test-threads=1
    cd research/e7-index-bench/src/bin/e170_o3_spec_executor && bash <仓根>/research/scripts/run-with-memory-cap.sh <上限> bash <仓根>/research/scripts/capped.sh 8 python3 <模型目录>/count_slot0_b.py

- `c651_probe_b.rs` / `c651_probe_b.out`：同一份探针再加一行 `recover(image, JournalPolicy::Consult).outcome`（择到旧池残留之后恢复走到哪），
  输出的上限用 `run-with-memory-cap.sh 256M`（前一份的峰值表条目是 11M；第一次写 8G 被「内存不够排不上」退 252，没跑，换小上限重跑，输出是重跑那一次的）。
- `c651_probe_c.rs` / `c651_probe_c.out`：只一格——同一个 fsid 重做 mkfs（槽距 4096 → 8192）、两盘槽 0 都坏、4096 处残留旧槽 1。同样 `run-with-memory-cap.sh 256M`。
