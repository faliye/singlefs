# m3-verus-land-code-r1 正推腿的模型目录

差分装置：改前 `replay_journal`（`git show fcc4d340:crates/singlefs-core/src/recovery.rs` 第 2941–2953、3057–3229 行）与改后（冻结副本）对同一批输入逐个比结局。

- `gen_old.sh`：从改前文件生成 `zz_old*.rs`（改前原样，以及四个改前变异体 m1–m4 作阳性对照）。
- `zz_diff.rs`：差分装置本体（放进冻结副本拷贝的 `crates/singlefs-core/src/recovery/`，在 `recovery.rs` 末尾加 `#[cfg(test)] mod …;`）。比的是：返回值（报告五个字段、重建的根、报错种类与内容）、读盘日志（盘号、偏移、长度的次序）、是否 panic。
- `run.sh` / `run_m.sh`：经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10` 起 `cargo test --release --offline -p singlefs-core --lib`。
- `mutate_core.py`：对核的可执行代码做单点改动，用差分量每个改动红不红；每条带「Verus 预测」（读 spec 与不变式推的，没跑 Verus）。
- `digest-probe/`：用 `crash_identity.rs` 里非测试的 `judging_code_of_the_judges` 在几份仓树上算判法摘要（工具链字段取固定串，只比相对变化）。
- 输出日志：`logs/`。
- `mutate_unit.py`：同一批核改动只跑新 crate 自带的两条单测，看红不红（`logs/unit_out.txt`）。
