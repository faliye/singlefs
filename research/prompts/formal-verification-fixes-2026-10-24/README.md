# 形式化验证修复证据（2026-10-24）

正式结论见 [records 修复报告](../../../records/2026-10-24-形式化验证问题修复.md)。此目录保存 `/tmp/singlefs-formal-fix-*` 的原始输出，文件名去掉共同前缀；没有改写日志内容。`source-sha256.json` 是归档时相关源码的 SHA-256，不代表整个工作区或 HEAD。

## 证据与复现入口

在仓库根目录运行 `python3 scripts/gate/verus-proof-check.py <检查名> .`。检查名 `proof`、`vacuity`、`coverage` 分别对应同名日志；不传 `--item` 或判定缓存。`proof` 包含逐项及最终无过滤整 crate 验证。工具固定 Verus 0.2026.09.20.aef82ed，`--no-cheating --rlimit 10 --num-threads 2`。

- `kernel.json` / `kernel.stderr`：原始整 crate 79 verified / 0 errors；生成在见证局部变量改下划线名前。随后 `proof.log` 验证了最终源码。
- `landings.log`：对原有 165 行执行 `must-red-landings . --write`，逐项真实验证后迁移第六列。
- `new-landings.log`：对新增 16 行执行同一迁移入口。已有第六列的行保留。两份日志合计 181 行；不等于迁移后重新执行过一次整表 `must-red-worlds`。
- `witness-negative.log`：见证变量改名后，以 `must-red-worlds` 的 `--item` 点名两个见证负例重新核对落点。证明错数量差异为信息项，落点必须一致。
- `omitted-regression.log`：原审计的最小未登记模块工程现在被拒绝，预期退出码 1。原工程构造见审计目录 `deep/gate_probe_false.py`。
- `coverage-regression.log`：内存中分别删除一个函数负例与一个 invariant 负例，覆盖判定都报告缺项；基线无缺口。
- `selftest.log`：`python3 scripts/gate/verus-proof-check.py --selftest`，128 格与 64 个破坏开关通过。
- `cargo.log`：`cargo test -p singlefs-replay-prefix --lib`，12 项通过。
- `clippy.log`：`cargo clippy -p singlefs-replay-prefix --all-targets -- -D warnings`，最终通过。
- `fmt.log`：`cargo fmt -p singlefs-replay-prefix -- --check`，退出码 0，无输出。

未运行整个工作区门禁、全部实验或崩溃流。空真筛查通过不构成一般可满足性证明；负例锚点覆盖也不构成语义穷尽性证明。
