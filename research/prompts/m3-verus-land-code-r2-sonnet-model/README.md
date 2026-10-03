# m3-verus-land-code-r2 正推腿的模型目录（2026-10-01）

H1：改前（fcc4d340）与改后 `replay_journal` 对同一批输入逐个比结局；H3：核失败到 `RecoveryFailure` 的映射；H6：E165 `--pinned-cases` 与钉表、Python 脚本的读法。

- `recovery_fcc4d340.rs`：`git show fcc4d340:crates/singlefs-core/src/recovery.rs` 的原样拷贝（报告引改前行号都指它）。
- `gen_old.sh`：从它取改前的 `record_ends_its_publish`、`readable_record_of_the_same_publish_follows`、`counter_of_the_last_record_the_root_covers`、`replay_journal` 生成 `zz_old*.rs`（改前原样与四个改前变异体 m1–m4）。
- `zz_diff.rs`：差分装置（放进冻结副本拷贝的 `crates/singlefs-core/src/recovery/`，在 `recovery.rs` 末尾加 `#[cfg(test)] mod zz_old; mod zz_old_m1; … mod zz_diff;`）。比返回值、读盘日志、是否 panic；Tally 分「只有改后 panic」「只有改前 panic」「两边都不 panic 而结局不同」，报错分种类。
- `run.sh`：经 `run-with-memory-cap.sh 8G` 与 `capped.sh 6` 起 `cargo test --release --offline -p singlefs-core --lib`。`batch1.sh`–`batch4.sh`：这一轮实际跑的几批（路径写死在 `/tmp/claude-1000/m3-verus-land-code-r2-sonnet/`，草稿里 `tree` 是冻结副本拷贝加上面的模块）。
- `mutate_new_core.py`：对改后核 `crates/singlefs-replay-prefix/src/lib.rs` 的 22 个单点改动，各用差分量红不红（`logs/mutate_new_core_out.txt`，每个改动一份日志在 `logs/mut_core/`）。
- `mutate_glue.py`、`mutate_glue2.py`：对胶水映射、报告拼装、新根来源与两条断言的 16 个单点改动，只跑五个 harness 目标（`logs/mutate_glue_out.txt` 与 `logs/mutate_glue2_out.txt`；前一份后 10 条是包装退 252 没跑成、不算结果，由 `mutate_glue2.py` 重跑；每个改动一份日志在 `logs/glue/`）。
- `h6_table_check.py`：钉表 151 行对装置读法的静态核（输出 `logs/h6_table_check.txt`）。`zz_h6_fingerprint.rs`、`mkrows.py`、`rows.tsv`、`rust_out.tsv`、`compare.py`、`e165-pin-red-examples.py.read`：H6 指纹对拍（装置那个纯函数的原文编进 harness 测试 vs Python 脚本的原文逐字执行）。
- 复跑：把冻结副本拷到草稿、放好 `zz_*.rs` 与模块声明，`DIFF_THREADS=6 DIFF_N=3000000 bash run.sh <树> <日志> -- --test-threads=1 --nocapture random_family` 之类；`DIFF_SEED=2` 换种子。
