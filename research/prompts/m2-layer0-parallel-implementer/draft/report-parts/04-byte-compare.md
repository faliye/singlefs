## 四、逐字比对（验收第 2 条）

跑法：`draft/run-layer0.sh <线程数> <测试二进制> <用例名> <日志>`，在副本里 `SINGLEFS_LAYER0_THREADS=<线程数> nice -n 19 cargo test --offline --release -p singlefs-harness --test <二进制> -- --include-ignored --exact <全量用例> --nocapture`，日志首尾记 UTC 时刻、`uptime`、挂钟秒数、退出码。

第一条流（`first_transaction_step_seven_layer0::layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations`）：

| 日志（`logs/`） | 线程数 | 切法 | `LAYER0` 行 | `CHECKER` 行 | `CHECKER_FIRST` 行 |
|---|---|---|---|---|---|
| `first-stream-threads32-loaded.log` | 32 | 512 片 × 513 | 同 | 同 | 同 |
| `timing-first-stream-threads1.log` | 1 | 64 片 × 4097 | 同 | 同 | 同 |
| `timing-first-stream-threads32.log` | 32 | 512 片 × 513 | 同 | 同 | 同 |
| `timing-first-stream-threads1-second-try.log` | 1 | 64 片 × 4097 | 同 | 同 | 同 |
| `timing-first-stream-threads32-second-try.log` | 32 | 512 片 × 513 | 同 | 同 | 同 |
| 对照 `research/prompts/m2-wave2-crash-verifier/54-supplement-first-txn-checker-line.log`（单进程） | 1（并行之前的代码） | 不切 | 同 | 同（全量那一行，不是 22 个状态那一行） | 同 |

「同」是 bash 字符串逐字相等（`[[ "$a" == "$b" ]]`）；三行的 sha256 前 16 位：`LAYER0` b77948b460c752a4、`CHECKER` 107a95c7d9c0c295、`CHECKER_FIRST` 1049609028f8b4d9。原样：

```
LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
CHECKER_FIRST {}
```

第二条流（`second_transaction_step_zero_layer0::full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean`）：`LAYER0B` 行去掉行首 `LAYER0B `，与 `research/prompts/m2-wave2-crash-verifier/54-layer0-replay.log` 里「✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle）：」之后的文字逐字相等，879 个字符，两边 `printf '%s' | sha256sum` 都是 `2bde7d65a2c283e72364ac1b9fbdb9fae90b2b8715562d975698f42fbdd66c3d`。比过的日志：`second-stream-threads32-loaded.log`（32 线程、负载 74→78，最高 130 上下）、`timing-second-stream-threads32.log`（32 线程），机器空闲时的第二次见第五节。

对照日志跑的是 HEAD 502ba80 加当时工作区的改动；今天主工作区的 crates 已经是 cae5092 加未提交改动，两份对照读数与今天代码的并行读数逐字相同，说明中间的代码变化没动这两条流的读数，也说明并行没动。
