# mutate.sh 读 required-features：报告（tooling-writer，会话 singlefs-e1 交接的工具缺口）

## 结论

- `research/scripts/mutate.sh` 现在从 Cargo.toml 里对应 bin 的那一节 `[[bin]]` 现读 `required-features`（用 python3 的 tomllib 解析 `$_manifest`，也就是 mutate.sh 原来核「源文件↔bin」时用的那份清单），基线、每条变异、还原之后那一次 cargo test 都带上 `--features <包名>/<feature>,…`。feature 不在命令行上手敲，也没写死在脚本里。
- 没挂 required-features 的 bin，cargo 参数与改前逐字相同（`test --release --bin <名>`）：自证里专门有一格核这一条。stderr 头一行照旧是「开几个工作进程」；挂了 feature 的 bin 在它后面多一行，报带了哪些 feature。
- E162 那张表改前跑到 rc=2「基线就是红的」，一条变异都没跑；改后同一张表 rc=0：抓到 13、无效 0、没红 0，超时与内存撞顶都是 0，末尾是「已还原，基线仍全绿」。
- 两张变异表都没动（不需要加表头声明）。
- 推翻条件：某个 bin 的 required-features 写成 tomllib 解析不了的形式，或者 bin 不在 mutate.sh 算出来的 `$_manifest` 里（`<源文件目录>/../../Cargo.toml`），这时候读出来是空的，会退回改前的跑法；另外，要是 `cargo test --features 包名/feature` 在虚拟 workspace 根上被 cargo 拒掉，也会推翻上面的结论（本机 cargo 1.98.0 上是接受的，E162 实测跑通了）。

## 改的地方（行号是改后 research/scripts/mutate.sh 的）

- 第 12–21 行：文件头说明（`--selftest` 多了什么，required-features 那一段）；第 49 行：弄坏开关 `MUTATE_BREAK=nofeatures`（不读 required-features，一律不带 `--features`，也就是改前的跑法）。
- 第 316 行 `read_required_features()`，第 341–351 行：读出 feature、组成 `cargo_feature_arguments`；清单解析不了就退 2，并给出路。
- 第 495、559 行：两处 `cargo test` 后面加上 `${cargo_feature_arguments[@]+"${cargo_feature_arguments[@]}"}`（数组为空时什么都不展开）。
- 头一行之后打带了哪几个 feature 的那一行，在 `worker_count_and_head_line` 调用之后。
- 自证：假 cargo 学真 cargo 的行为，bin 挂了 required-features 而 `--features` 没给全（写 `feature` 或 `包名/feature` 都认）就退 101。新加一格从第 232 行开始，核两项：① 没挂的 bin 每次参数逐字是 `test --release --bin fake-bench`；② 清单追加 `required-features = ["fake-feature", "fake-other"]` 之后，每次参数逐字是 `test --release --bin fake-bench --features bench/fake-feature,bench/fake-other`，抓到那一条，退 0。

改法：先在草稿里改 `/tmp/claude-1000/mutate-features/mutate.sh.new`，再拷成同目录临时文件 `research/scripts/.mutate.sh.tmp`，然后 `mv` 盖过去（换了 inode，没有就地改）。改前那份存在 `/tmp/claude-1000/mutate-features/mutate.sh.before`。

```
$ git diff --stat -- research/scripts/mutate.sh
 research/scripts/mutate.sh | 98 +++++++++++++++++++++++++++++++++++++++++++---
 1 file changed, 92 insertions(+), 6 deletions(-)
```
改后 `sha256sum research/scripts/mutate.sh`：`cdd4e21dde0601c6e435f58406406fc77060933d7c1ff8d946846f0794a0ac6b`

## 判红输入：改前对 E162 表跑（今天的结局）

命令（在 research/ 下跑，时间 2026-09-27）：`BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include MUTATE_JOBS=1 MUTATE_TARGET_DIR=/tmp/claude-1000/mutate-features/target nice -n 19 bash scripts/capped.sh 4 bash scripts/mutate.sh e162-crash-verdict-block-store e7-index-bench/src/bin/e162_crash_verdict_block_store.rs mutations/e162_crash_verdict_block_store.tsv`
```
mutate: 这一轮开 1 个工作进程（各项取最小、至少 1）：核数 32 的一半、至多 16，给 16 个；MUTATE_JOBS 给 1 个；表里 13 条。卡在「MUTATE_JOBS」这一项；每个 cargo 编译并行度 4（调用方的 CARGO_BUILD_JOBS）
mutate: 基线就是红的，先修好再来
rc=2
```
基线的输出被丢进了 /dev/null，所以原因另外用 `cargo test --release --bin e162-crash-verdict-block-store --no-run` 看了一次（这条只编不跑，编之前就失败了）：
```
error: target `e162-crash-verdict-block-store` in package `e7-index-bench` requires the features: `e162-block-stores`
Consider enabling them by passing, e.g., `--features="e162-block-stores"`
rc=101
```

## 自证：弄坏开关 → 判红，关掉开关 → 转绿

`MUTATE_BREAK=nofeatures bash research/scripts/mutate.sh --selftest`：
```
  ✗ 自检：挂了 required-features 的 bin 应当每次带「--features bench/fake-feature,bench/fake-other」跑、抓到那一条、退 0，实际退 2、对不上的参数：test --release --bin fake-bench：mutate: 这一轮开 1 个工作进程（各项取最小、至少 1）：核数 32 的一半、至多 16，给 16 个；表里 1 条。卡在「表里的条数」这一项；每个 cargo 编译并行度 32（核数 ÷ 工作进程数）
mutate: 基线就是红的，先修好再来
  ✗ mutate.sh 自检 1 处不对（查了 24 项）
rc=1
```
`bash research/scripts/mutate.sh --selftest`（不设开关）：
```
  ✓ mutate.sh 自检通过（查了 24 项：……；required-features：没挂的 bin 的 cargo 参数逐字不变，挂了两个的带「包名/feature」跑得起来、照常判抓到）
rc=0
```
（中间的「……」是我省掉的原句中段，全文在 `/tmp/claude-1000/mutate-features/selftest-green.out`。）

## 改后对 E162 表跑（原样）

命令与判红那次相同（换成改后的 mutate.sh），时间 2026-09-27：
```
mutate: 这一轮开 1 个工作进程（各项取最小、至少 1）：核数 32 的一半、至多 16，给 16 个；MUTATE_JOBS 给 1 个；表里 13 条。卡在「MUTATE_JOBS」这一项；每个 cargo 编译并行度 4（调用方的 CARGO_BUILD_JOBS）
mutate: e162-crash-verdict-block-store 在 e7-index-bench/src/bin/../../Cargo.toml 里挂了 required-features，每次 cargo test 带 --features e7-index-bench/e162-block-stores
基线：全绿
✅ [M1_F写路径换成朴素写法直接写终名] 红：tests::file_registered_write_path_leaves_no_partial_file_at_any_abandoned_step
✅ [M2_redb提交用DurabilityNone] 红：tests::every_registered_arm_commit_waits_for_the_device,tests::redb_registered_write_path_keeps_every_confirmed_block_across_sigkill
✅ [M3_K的同步写关掉WAL仍开] 红：tests::every_registered_arm_commit_waits_for_the_device
✅ [M4_K关掉WAL] 红：tests::every_registered_arm_commit_waits_for_the_device,tests::rocksdb_registered_write_path_keeps_every_confirmed_block_across_sigkill
✅ [M5_F读路径不查长度魔数与CRC] 红：tests::file_read_path_reports_a_half_written_file_as_a_read_error
✅ [M6_核对只比块值前4096字节] 红：tests::silent_corruption_control_finds_exactly_block_seven_on_every_arm
✅ [M7_核对把在途而读不到记成丢块] 红：tests::block_classification_follows_the_registered_table,tests::file_registered_write_path_keeps_every_confirmed_block_across_sigkill,tests::file_registered_write_path_leaves_no_partial_file_at_any_abandoned_step,tests::in_flight_block_that_never_landed_is_not_a_lost_block,tests::redb_registered_write_path_keeps_every_confirmed_block_across_sigkill,tests::rocksdb_registered_write_path_keeps_every_confirmed_block_across_sigkill
✅ [M8_核对不枚举键] 红：tests::phantom_key_control_finds_exactly_one_key_on_every_arm,tests::silent_corruption_control_finds_exactly_block_seven_on_every_arm
✅ [M9_打开失败时新建空库接着跑] 红：tests::open_breaking_control_counts_exactly_one_open_failure_on_every_arm
✅ [M10_杀点总在第1个C之后] 红：tests::permutation_and_kill_plan_match_registered_anchor_values
✅ [M11_提交计时两次Instant都在提交调用之前] 红：tests::commit_timing_includes_the_injected_five_millisecond_sleep
✅ [M12_P-rand块值全0] 红：tests::random_block_values_match_registered_anchor_values
✅ [M13_块键整数按小端编码] 红：tests::fingerprint_and_interleaved_keys_match_registered_anchor_values
计数：内存撞顶 0 条（上限 16G）、超时 0 条
已还原，基线仍全绿
rc=0
2026-09-27
```
汇总（用 `grep -c` 数的）：抓到 ✅ 13、没红 ❌ 0、无效 ⏭ 0、超时 ⏱ 0、内存撞顶 🧱 0、没跑完 💥 0、抓名字有盲区 ⚠️ 0。rc=0。
这份产物没有放进 `research/results/`：那个目录不在我的写范围里。原件在 `research/results/e162-crash-verdict-block-store-mutate-2026-09-27-features.log`，要入库的话请主 agent 拷过去（门禁 69 号判「变异表改了、results 里却没有不比它旧的产物」，这张表是这一轮新建的，没进 git）。

开跑前看的负载：没有性能测量在跑（qemu-system、vm-bench、e152、fio 都没有）。当时有别的会话在跑两样：pid 3889423 `cargo test --offline -p singlefs-harness --test second_transaction_supplem…`，以及 pid 4057253 `run-with-memory-cap.sh 8G cargo run --release -p e7-index-bench --features e162-block-stores --bin e162-crash-verdict-block-store`。我这边加了 nice -n 19 照常跑，编译目录用自己的 `MUTATE_TARGET_DIR`，没有跟它们抢锁。

## E163 那张表：没跑

- mutate.sh 里没有一栏叫「跑不了」（「跑不了」是 `research/scripts/replay.sh` 的栏，在第 813、824、837、843 行）。mutate.sh 里对应「跑不了」的现成出口是：基线不绿就退 2（「基线就是红的」），一条变异都不跑，不会报成抓到。
  E163 的三条 GPU 单测在枚举不到卡时会 panic（`research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs` 第 828 行的 `assert!`），所以在没有卡的机器上会走这个出口。这一点是推出来的，我没有造一台没卡的机器去验。
- 本机有两张卡（`nvidia-smi -L`：RTX 5090、RTX 5060 Ti），但两张卡上都常驻着本地模型服务：`ray::RayWorkerProc` pid 2014846 占 15870 MiB，pid 2014845 占 15338 MiB。E163 的测试要在第 0 张卡上建设备、派计算，出口也没要求跑这张表，我怕扰到模型服务，就没跑。
  要跑的话，命令照 E162 那条换 bin、源文件和表，加 `MUTATE_JOBS=1`（测试里的 GPU 锁只管同一个进程；几个工作进程同时撞上同一张卡，会像源码第 816–820 行注释写的那样挂住）。

## 收尾检查（末行原样与退出码）

### gate-33-mutation-tables
```
  ✓ 150 个实验二进制都有成形的变异表，1710 条变异的原文各命中源码一次；crates/mutations.tsv 1302 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 7 张：research/mutations/e125_zoned_wp.tsv research/mutations/e129_tear_injector.tsv research/mutations/e129_thin_neighbour.tsv research/mutations/e129_thin_rmw.tsv research/mutations/e158_arms.tsv research/mutations/e158_r3_arm_mutations.tsv research/mutations/e158_r4_arm_mutations.tsv （本阶段不跑变异，只验装置在、锚点对得上、两段里没有 \n 以外的反斜杠转义、crates 那张表里没有两行重复——重复按「文件 + 原文 + 替换文 + 点名的测试」四项认，变异名另判）
rc=0
```
### gate-47
```
           再回头检查 perform_check 里对应字段的比较逻辑。
  → 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。
rc=1
```
### gate-62-stage-owners
```
  ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）
rc=0
```
### gate-63-agent-write-scope
```
  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸、交回闸与弹窗断言闸注册着、自证通过，定义的 model 取值认得，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着、自证通过，书记官写入后的核对 hook 注册着、自证通过，表与定义一致（4 个有 Write 或 Edit 的定义、28 条路径模式），共用切词模块的 14 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 13 个文件），共用重型测试判定模块的 36 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 13 个文件；selftest、load_sibling_module 不算，见 NOT_SHARED_JUDGMENT）
rc=0
```
### gate-73-research-gate-lint
```
  ✓ 研究脚本、hook 与 .claude/scripts 的门禁纪律：扫了 3 个目录（research/scripts .claude/hooks .claude/scripts），拒绝都带出路、shell 纪律守住；进程安全连 .claude/gate.d/ 共扫 4 个目录；执行位：判过（2 个目录）
    没扫的目录 0 个（不在）：（没有）
rc=0
```
### lint-doc-lint
```
  !   「按本决策办」这种嵌在句中的漏得掉。绿不代表这两条穷举过了。
  ✓ 文档铁律检查通过（检查 524，跳过 0；DOC_LINT_VERBOSE=1 看全部）
rc=0
```
### lint-rules-lint
```
  ✓ 规则只写怎么做（语言 zh；扫了 29 份文件 1967 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 8 行的日期只在引号或反引号里）、解释性段落 0、解释性半句 0、没带劝阻句的链接 0；词法说明判了 1844 行（围栏与表格行不判），命中 0；使用者名字这一条无对象可判：被扫的仓没有 I18N 或没登记 consumers=）
rc=0
```
### lint-gate-lint
```

  ✗ 门禁自检失败：45 处（共 667 个脚本、1093 条拒绝）
rc=1
```
### lint-shell-lint
```

  ✗ shell 纪律检查失败：61 处（共检查 532 个脚本）
rc=1
```
### lint-preflight-lint
```
  ✗ 准入与运行条件：判了 141 个脚本（登记目录 53、脚本 3、钩子 9、门禁阶段 76），1 处不合格；排除 192 个：.claude/gate.d/lib-format-const.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-history-brief.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-index-vs-body.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-item-ref-status.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-manifest.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-open-item-review.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-owed.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/gate.d/lib-prime-marks.py（被门禁阶段 import 的函数库，没有自己的入口，不单独调）；.claude/hooks/lib_heavy_tests.py（被钩子 import 的函数库（--selftest 由 hooks-registered 与 63 号经钩子跑），不单独调）；.claude/hooks/lib_selftest_scratch.py（被钩子 import 的函数库（--selftest 由 hooks-registered 与 63 号经钩子跑），不单独调）；.claude/hooks/lib_shell_words.py（被钩子 import 的函数库（--selftest 由 hooks-registered 与 63 号经钩子跑），不单独调）；research/scripts/changed-paths.sh（被门禁阶段 source 的函数库（gate_diff_base、gate_changed_paths），不单独调）；research/scripts/stage-run-or-skip.sh（被门禁阶段 source 的函数库（stage_run_or_skip）；它的 --selftest 由 47 号跑）；research/scripts/lib_atomic_replace.py（被 replace-once.py、insert-row.py import 的函数库，不单独调）；research/scripts/e125-zoned-wp-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e129-tear-injector.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e129-thin-neighbour.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e129-thin-rmw.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e152-run.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e152-stage-root.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e152-tables.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e21-compute.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e21-prove-gpu.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e21-transfer.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e34-iomin-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e56-report.py（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e56-sweep.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e6-units.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/e72-devtable-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/local-repro-tuned.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/local-repro.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/stripe-map-probe.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/vm-bench.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/scripts/vm-geom.sh（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e100_system_configuration_slot.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e101_node_tag_reserve.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e102_unit_class_registry.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e103_inode_update_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e104_current_version.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e105_extent_leaf_packed.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e106_stripe_member_table.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e107_stripe_table_wa.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e108_plaintext_layer_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e109_position_authority.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e110_stripe_table_steady.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e111_stripe_table_key.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e112_old_writer_unknown_tree.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e113_unknown_tree_full_arms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e114_pack_ledger.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e115_system_configuration_completeness.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e116_pack_settle.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e117_reserved_header.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e118_single_disk_recovery.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e119_slot_tiers.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e120_tier_ratio.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e121_capacity_tiers.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e122_directory_locality.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e123_reuse_window_versus_rollback_depth.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e124_system_configuration_recompute.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e126_system_configuration_slot_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e127_group_identity_under_split_merge.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e128_pointer_birth_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e12_lifecycle.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e130_livelist_bounded_destroy.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e131_livelist_carrier.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e132_livelist_carrier_recount.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e133_map_key_format_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e134_map_key_slot_baselines.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e135_rollback_floor.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e136_fork_cost_rows.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e137_map_key_performance.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e138_per_disk_floor.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e139_tightened_floor.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e140_header_alignment.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e141_switch_reserve_mount_admission.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e142_region_diff_independent.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e143_one_unit_per_transaction_journal.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e144_header_checksum_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e145_self_describing_node_header.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e146_livelist_entry_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e147_system_configuration_recompute_from_layout.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e148_commit_fixpoint_two_record_trees.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e149_pack_container_repair_options.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e14_discrimination.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e150_rollback_reuse_of_abandoned_roots.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e151_arrival_and_container_arms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e152_file_system_benchmark.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e153_ledger_shape_and_ring_holes.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e154_two_gates_serial_rejudge_and_reclaim_timing.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_fourth_run_group_commit_concurrency.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_fsync_write_volume.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_second_run_fsync_write_volume.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e155_third_run_release_cascade.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e157_parallel_line_one_clauses.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e159_fsync_wait_group_commit.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e160_random_small_read_share.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e16_journal.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e17_merge.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e18_branch.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e19_defer.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e20_fanout.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e21_cpu.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e23_journal_geom.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e24_recovery.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e25_journal_reserve.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e26_accounting.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e27_snapshot_accounting_risk_paths.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e28_map_rebuild.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e29_blast_radius.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e30_range_rebuild.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e31_aad_snapshot.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e32_journal_timeline.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e33_pin_rules.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e34_ring_iomin.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e35_head_forms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e36_slot_mapping.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e37_log_epoch.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e38_accounting_copy_on_write.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e39_back_chain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e40_checksum_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e41_root_ring_geom.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e42_transaction_records.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e43_extension_point_budget.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e44_jsn_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e45_span_ring_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e46_region_spacing.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e47_ring_loss.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e48_ring_placement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e49_chain_width.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e50_ring_slots.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e51_chain_chances.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e52_head_mechanisms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e53_ring_failure.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e54_accounting_generations.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e56_epsilon.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e57_field_authority.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e58_csum_grain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e59_message_recompute.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e60_rebalance.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e61_chain_hash.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e62_ring_home.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e63_width_rule.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e65_write_grain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e66_small_files.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e67_device_subset.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e68_inline_threshold.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e69_backref_cost.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e6_multicore.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e6_units.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e70_ckpt_thresholds.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e71_accounting_keys.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e73_key_range.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e74_allocation_records.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e75_record_size.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e76_payload_checksum.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e77_publish_order.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e78_replay_start.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e79_root_record.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e7_index.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e80_partial_stripe.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e81_commit_fixpoint.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e82_admission_overlay.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e83_tombstone_grain.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e84_tombstone_pinning.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e85_unit_header.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e86_scan_step.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e87_fixed_placement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e88_impostor_orphan.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e89_interval_frontier.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e8_split.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e90_tree_aad.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e91_ring_admission.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e92_reuse_requirement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e93_aging_placement.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e94_move_touchset.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e95_node_layout_arms.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e96_hybrid_consistency.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e97_entry_encoding.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e98_inode_record.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e99_writebuffer_sequence.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；research/e7-index-bench/src/bin/e9_keylayout.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补）；crates/singlefs-harness/src/bin/e142_first_transaction_write_dump.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/e142_first_transaction_write_dump_one_device.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/e158_root_choice_repair.rs（还没改完：实验要写 inputs-changed，登记它的结果取决于的全部输入（代码、决策、跑前登记），逐个对着跑前登记列完再补；它在 crates/ 里，改动还要走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/first_transaction_device_log_check.rs（还没改完：它在 crates/ 里，加 preflight 调用是 crates 代码改动，要带测试并走代码三方（门禁 56 号））；crates/singlefs-harness/src/bin/first_transaction_on_device.rs（在虚机的 busybox initramfs 里跑的静态二进制，那里没有 python3，preflight 判不了条件）；crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs（还没改完：它在 crates/ 里，加 preflight 调用是 crates 代码改动，要带测试并走代码三方（门禁 56 号））；.claude/scripts/preflight.sh（被项目脚本 source 的函数库：找到规范副本里的 preflight.sh（本树或主仓）就 source 它，找不到就定义一个当场判红的 preflight；不单独调）；.claude/scripts/project_preflight.py（被项目的 python 脚本 import 的函数库：找到规范副本里的 preflight.py（本树或主仓）再转调，找不到判红；不单独调）；.claude/gate.d/lib-governance-refs.py（被门禁 10 号调的函数库（治理文档里的门禁号、路径与「小节」指不指得到），不单独调）
     → 怎么办： 按上面每一处的出路改；规矩见 rules/preflight-discipline.md
rc=1
```
### lint-gate-overlap
```
  ✓ 相对 262c02d3e47a：新加的门禁与钩子 0 个（无）都写明了比过谁，改过的 6 份脚本对照已有的 134 份没有整段相同
    没判的：装进来的 SOP 副本 37 份只当对照（.claude/singlefs-ai-sop/scripts）
rc=0
```

更正：上文「第 816–820 行注释」应为第 817–822 行（`grep -n` 现查到第 818、821 行各是那段注释的一句）。

### 红的几项：点名的文件都不在这一轮的改动里，按定义没修
- 47 号 rc=1：唯一一条红的是 `python3 research/scripts/check-segment-registry.py --selftest`（段序列登记表与 E142 产物、`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs` 对不上）。这次 47 号跑在 mutate.sh 换上之后，它 runner 表第 56 行登记的 `bash research/scripts/mutate.sh --selftest` 不在红项里。
- gate-lint（45 处）、shell-lint（61 处）：这两份输出里 `grep -c mutate` 都是 0，点名最多的是 `gate.sh` 和 `research/prompts/…` 下的临时脚本。
- preflight-lint（1 处）：`research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs:726 fn main 的第一句不是 preflight(…)`，这个文件不在我的写范围里。
- 阶段归属表里没有登记给 tooling-writer 的阶段（awk 列出来是空的）。

## 新写的 `# gate-similar:` / `# hook-events:` 行

没有：这次没有新建门禁或钩子，只改了一个研究脚本。`gate-overlap.py --list` 里管变异表的是 33 号与 59 号，没有哪一道管 required-features。

## 改过的定义

没有（这次不是改定义那种活）。

## 删了什么

- `/tmp/claude-1000/mutate-features/target`（E162 的编译目录，删之前 `du -sh` 是 5.0G），已删。
- mutate.sh 自己建的 `singlefs-mutate-*` 副本，由它退出时的 trap 删掉。`/tmp/singlefs-mutate-target*` 那一批不是我建的（我设了自己的 `MUTATE_TARGET_DIR`），没碰。
- 留着的：草稿目录 `/tmp/claude-1000/mutate-features/`，里面是报告、progress.md、改前改后的运行日志、各检查的输出，还有 `mutate.sh.before` / `mutate.sh.new`，给主 agent 存档。

## 没做什么

- 没跑重型测试（54、55、57、59、87 号本身，`gate.sh`，全量 `cargo test`，`check.sh`），也没跑 `crates/mutations.tsv`。
- 没跑 E163 的变异表（原因见上文）。「没卡的机器上 E163 会停在基线就是红的、退 2」这一条只是推出来的，没验过。
- 没把 E162 的产物放进 `research/results/`（不在写范围里）；没走定义三方；没提交。
- 两张变异表的表头都没加声明：feature 从 Cargo.toml 现读，用不着。
