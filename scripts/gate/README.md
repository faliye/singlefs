# scripts/gate

提交前的门禁编排、门禁阶段共用的判法、准入判定、补丁合入与合入后验证。门禁阶段本身在 `.claude/gate.d/`。

| 文件 | 做什么 |
|---|---|
| `gate-staged.sh`、`gate-staged-selftest.sh` | 跑 `gate.sh --staged`，整轮全绿才前移 `refs/sop/staged-green` |
| `stage-must-run.sh`、`stage-must-run-selftest.sh`、`stage-run-or-skip.sh`、`changed-paths.sh`、`change-touches-crates.sh` | 一道阶段要不要跑、这次改动碰了哪些路径 |
| `admission.py` | 门禁阶段与实验开跑之前的准入判定 |
| `lib_source_digest.py` | 源码摘要：给一份文件算不随注释与空白变的摘要；`admission.py` 的门禁指纹（全绿标记、崩溃枚举用例）与 `stage-must-run.sh` 两棵树的比对共用 |
| `stage-mine.py` | 几个会话共写一批文件时只暂存这一轮的块；`--by-row` 在整表重排、块里夹着别人的行时按整行挑；`--write-index 文件 内容来源文件 --foreign 正则` 直接写索引，写之前核新增行里没有别人的行 |
| `apply-writer-patch.py`、`merge-and-verify.sh`、`rerun-failed-tests.py`、`lib_libtest_result_lines.py` | 打实现员的补丁、合入后验证、只重跑红的、libtest 结果行的解析 |
| `lib_workspace_lock.py`、`lib_rebuilt_packages.py` | 主工作区合入锁（打补丁与合入后验证持锁、排队、报持有者、接管死锁）；拿副本编时核改过的包都有 `Compiling` 行（合入后验证与证红共用） |
| `gate-structure-check.py`、`crate-sources-reachable.py`、`light-selftest-guard.py`、`hook-swap.py`、`closeout-status.py` | 门禁结构、crate 源文件可达、轻型自证的守卫、钩子先验后换、收尾进度表 |
| `verus-proof-check.py` | 门禁 checker-tier-formal-verification 的 Verus 判法 |
| `directory-layout-check.py` | 门禁 code-tooling 的 directory-layout 格的判法：判 `scripts/` 与 `research/` 的目录形态（`.claude/rules/directory-layout.md`） |
| `data-name-lint.py` | 门禁 code-source-discipline 的 data-names 格的判法：字符串字面量（元组首列、=> 右侧、键= 的键名）与 kb 表格里名字形态的串也按命名纪律判，缩写词表读上游 naming-lint.sh，存量登记在 `.claude/data-name-lint-exclude`（只缩不涨） |
| `wallclock-assert-lint.py` | 门禁 checker-tier-research-build-and-replay 的 wallclock-asserts-registered 格的判法（C465）：`scripts/experiments/*.rs` 的 `#[test]` 里按挂钟判的断言要带 `// wallclock-assert: <量的是什么>` 登记位（一行注释，装置不加代码；门禁单测格按 `--list` 的登记名给 `cargo test` 传 `--skip`），或带 `// wallclock-assert-load-safe: <为什么>`；`--list` 逐行列登记了的「文件、函数名」；立判法之前就在仓里还没复核的在 `.claude/gate.d/wallclock-assert-lag.tsv`（只缩不涨） |
