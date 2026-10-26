# scripts/gate

提交前的门禁编排、门禁阶段共用的判法、准入判定、补丁合入与合入后验证。门禁阶段本身在 `.claude/gate.d/`。

审核回归：`python3 -B -m unittest discover -s scripts/gate/tests -v`。用临时 Git 仓和替身检查暂存树索引隔离、补丁删除检查、大文件历史、失败日志解析、测试目标选择与函数单元拒算边界，不编译真实项目。

| 文件 | 做什么 |
|---|---|
| `gate-staged.sh`、`gate-staged-selftest.sh` | 跑 `gate.sh --staged`：被判的暂存树经 `GATE_STAGED_TREE` 交给上游，编译目录固定在 `CARGO_TARGET_DIR`；整轮全绿、调用方又没给 `GATE_BASE` 才前移 `refs/sop/staged-green`（那一条 ref 只作记录，复用一处都不读它） |
| `changed-paths.sh` | 这次改动碰了哪些路径、新增了哪些行（只给静态门禁「只判新加行」这类判据用，不给任何判定当复用依据） |
| `baseline-diff.sh` | 判「这一格的红是不是这一轮带的」：`baseline-diff.sh <门禁脚本> [--check <格>] --base <提交>`，用主仓里的那一份判法分别判基准提交的树（`git worktree add --detach` 建在临时目录、跑完删；基准树里没有的判法脚本从主工作区补，不报 127）与主工作区，逐行比红行（抹掉行号这类会漂的段），只报工作区新增的；只差数字的红行（汇总行里的计数变了）单列「计数变了的红行 N 条」、两版各打一行、不算新红；门禁阶段与共享 doc-lint（点名 `.claude/singlefs-ai-sop/scripts/doc-lint.sh`，不是项目里写死根的包装）都能比；`--selftest` 在临时仓里造「基准就红」的格与「这一次新红」的格，只报后一个 |
| `admission.py` | 门禁阶段与实验开跑之前的准入判定；单元键、判定库（`<git common-dir>/singlefs-unit-verdicts/`）、门禁的复用答法、复跑表每一行的行键、实验键都在它里面 |
| `lib_function_units.py` | 函数单元：Rust、WGSL、Python、shell 切成函数一级的单元，从入口顺调用图收闭包、算一把只随闭包里单元内容变的键（认不出的拒算）；Rust 那一份与 `crates/singlefs-unit-graph` 不共用代码，`dump` 与闭包输出的写法以那一份为准（种类名 `macro_rules`、`item_macro`、`mod`、`mod_head`、`wgsl_compute` 这一套，限定名带 crate、目标与模块路径前缀，数据单元按读它的那一处列 `data` 行，写法在文件头），门禁 code-tooling 的 function-units-two-implementations-agree 格逐单元、逐入口对拍；`dump` / `closure` / `key` / `controls` |
| `stage-mine.py` | 几个会话共写一批文件时只暂存这一轮的块；`--by-row` 在整表重排、块里夹着别人的行时按整行挑；`--write-index 文件 内容来源文件 --foreign 正则` 直接写索引，写之前核新增行里没有别人的行 |
| `apply-writer-patch.py`、`merge-and-verify.sh`、`rerun-failed-tests.py`、`lib_libtest_result_lines.py` | 打实现员的补丁、合入后验证、只重跑红的、libtest 结果行的解析 |
| `test-targets-parallel.py` | 先编测试目标，再按编译产物逐目标并行跑；元数据跟随指定的清单，执行时去掉会扩大目标范围的选择参数；仅默认目标选择补跑所选包的文档测试 |
| `lib_workspace_lock.py`、`lib_rebuilt_packages.py` | 主工作区合入锁（打补丁与合入后验证持锁、排队、报持有者、接管死锁）；拿副本编时核改过的包都有 `Compiling` 行（合入后验证与证红共用） |
| `gate-structure-check.py`、`crate-sources-reachable.py`、`light-selftest-guard.py`、`hook-swap.py`、`closeout-status.py` | 门禁结构、crate 源文件可达、轻型自证的守卫、钩子先验后换、收尾进度表 |
| `shared-index-stale-check.py` | 提交之后查共用索引有没有停在旧 HEAD 上（C420：用 `GIT_INDEX_FILE` 指私有索引提交、共用索引没跟上，`git diff --cached` 把刚进仓的文件全列成已暂存的改动）；`.claude/hooks/shared-index-stale-reminder.sh`（PostToolUse:Bash）在 `git commit` 之后自动跑它，只报告、不拦 |
| `verus-proof-check.py` | 门禁 checker-tier-formal-verification 的 Verus 判法；整格逐项后完整验证，`coverage <根>` 检查函数与不变式负例覆盖，空真筛查不替代构造见证 |
| `lkmm-selftest.sh` | `.claude/scripts/lkmm.sh` 自己的自证（判它的两趟复用），门禁 checker-tier-lkmm 的样本档另在那一道 |
| `lib_reuse_checks.py` | 带判定库的门禁与入口共用的三样判法：复用三问（自证里 `reuse-three-questions` 一格）、复用哨兵、上次红的清单；门禁 code-tooling 的 reuse-selftests 格按登记表核那一格 |
| `directory-layout-check.py` | 门禁 code-tooling 的 directory-layout 格的判法：判 `scripts/` 与 `research/` 的目录形态（`.claude/rules/directory-layout.md`） |
| `large-file-check.py` | 门禁 code-tooling 的 committed-file-size 格的判法：被提交的东西里没有超过 100 MiB（GitHub 单文件推送上限，常量 `SINGLE_FILE_SIZE_LIMIT_BYTES` 全仓只在这里定义）的文件；甲判 HEAD 加暂存区里跟踪的每个文件，乙判还没推的提交里新增或改过的 blob，没有上游时乙本次未跑；超限的 `gzip -n -9` 压成 `.gz` 入库 |
| `data-name-lint.py` | 门禁 code-source-discipline 的 data-names 格的判法：字符串字面量（元组首列、=> 右侧、键= 的键名）与 kb 表格里名字形态的串也按命名纪律判，缩写词表读上游 naming-lint.sh，存量登记在 `.claude/data-name-lint-exclude`（只缩不涨） |
| `wallclock-assert-lint.py` | 门禁 checker-tier-research-build-and-replay 的 wallclock-asserts-registered 格的判法（C465）：`scripts/experiment-apparatus/*.rs` 的 `#[test]` 里按挂钟判的断言要带 `// wallclock-assert: <量的是什么>` 登记位（一行注释，装置不加代码；门禁单测格按 `--list` 的登记名给 `cargo test` 传 `--skip`），或带 `// wallclock-assert-load-safe: <为什么>`；`--list` 逐行列登记了的「文件、函数名」；立判法之前就在仓里还没复核的在 `.claude/gate.d/wallclock-assert-lag.tsv`（只缩不涨） |
| `checker-direction-overlap.py` | 门禁 checker-independence-and-sync 的 checker-direction-overlap 格的判法（C410）：池级 checker 遍历式与扫描式两个方向各从登记的入口收函数单元闭包（`lib_function_units.py` 的 Rust 闭包），交集只许落在格式解析（`singlefs-format` 整个包与登记的 `crates/singlefs-checker/src/lib.rs` 单元）与常量里，越界、取不到调用图都判红；入口、切点、比对点、局部绑定与允许共享的单元登记在 `.claude/gate.d/checker-direction-entries.tsv` |
