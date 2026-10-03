# m3-verus-land-code-r3 辩方腿的模型目录（2026-10-01）

读冻结副本 `/tmp/claude-1000/m3-verus-land-code-r3/tree/` 的拷贝；改前取第二轮模型目录 `research/prompts/m3-verus-land-code-r2-sonnet-model/` 的 `recovery_fcc4d340.rs` 与 `gen_old.sh`。

- 差分（J5）：`zz_diff.rs` 是第二轮 `zz_diff.rs` 加三栏（`value_differs`、`log_only`、`log_only_new_has_extra_reads`）、两处 `assert_eq!(t.diverged, 0)` 换成只打印的版本。用法：冻结副本拷一份，`gen_old.sh <recovery_fcc4d340.rs> <树>/crates/singlefs-core/src/recovery` 生成 `zz_old*.rs`，把 `zz_diff.rs` 放同一目录，`recovery.rs` 末尾加 `#[cfg(test)] mod zz_old; mod zz_old_m1; … mod zz_diff;`（各带 `#[cfg(test)]`），`bash batch3.sh <树目录名> <日志前缀>`（脚本里的 `D=` 路径要改成你的草稿目录）。`h52-restore.diff` 是把 H5-2 删掉的那半句加回去的 2 行改动（对照树）。`logs/diff-results.txt` 是四份日志与两份旧分栏日志里的 RESULT、CONTROL 行加各份全文的 sha256。
- 22 个单点改动对差分的阳性对照：`mutate_new_core_r3.py`（第二轮 `mutate_new_core.py` 原样，只改了说明文字），结果 `mutate_new_core_r3_out.txt`；要在撤回 H5-2 的树上跑（`tree_m` = 对照树）。
- J2：`j2/j2-rows.tsv`（S03–S12）、`j2/run-j2.sh`（逐行套、只跑新用例、拷回）、`j2/apply-row.py`（第二轮攻方的）、`j2/summary.tsv`、`j2/baseline.log`（冻结副本上原样跑那一条绿）。
- H3：`h3/mutate_glue_r3.py`（胶水单点改动，18 个，含一个等价对照）与 `h3/glue-r3-summary.txt`。
- H4：`h4/harness-rows-r3.sh`（`crates/mutations.tsv` 行逐行套、跑点名的 harness 目标）与两份 summary。
- Verus：`verus/verus-one.sh`（单次）、`verus/mustred.py`（必红表逐行判）、`verus/v1-pristine.log`、`verus/mustred-summary-93-rows.tsv`、`verus/mustred-summary-13-rlimit-rows-second-run.tsv`。
- H5-3 一处定义的原型：`h53/h53-prototype.diff`（对冻结副本 `crates/`），`h53/` 下有 Verus、`cargo check --workspace --all-targets`、core `--lib`、核 `--lib`、五个 harness 目标的输出。
- H6：`h6/compare_today.py`（用今天的脚本重算第二轮 168 行，需要第二轮模型目录的 `rows.tsv`、`rust_out.tsv` 放在当前目录）与两份输出。
