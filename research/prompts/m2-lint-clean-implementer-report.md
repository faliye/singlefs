# 收尾批：harness clippy 与 fmt 清红

## 结论

红的根因不是 e156 或 checker_narrow 测试文件本身的代码坏了，是 `cargo clippy -p singlefs-harness --all-targets` 在默认（不带 `--keep-going`）情况下，第一个撞见的编译错误（在 `e158_root_choice_repair.rs`）会让 cargo 提前打印
「build failed, waiting for other jobs to finish...」并停止调度新任务，导致后面几个独立编译单元（e156 的 bin、e156 的单测、checker_narrow_invariants_and_abandoned_roots 这个测试目标）到底有没有查过、查出了什么，肉眼从终端输出上看不出来——
三份实现员报告各自只查自己那个目标，谁都没有意识到还有别的目标同一次没被跑到。加 `--keep-going` 之后才能一次看全：e156 与 checker_narrow 各有自己的红，与 e158 的红互不相关。

现在的状态：`crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs` 与
`crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs` 的 clippy 与 fmt 都已清红（命令与输出见下）。
`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 仍有 5 处 `shadow_unrelated` 与 67 处 fmt 差异——按派发指令列出、不改，E158 执行员在改这份文件。

## 要动的文件（2 份，未超过 8 份上限）

- `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`
- `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`

`crates/mutations.tsv` 一行没改：22 条锚定在 e156 文件上的变异「原文」逐条核过，改动前后都恰好命中一次（下面「变异表核对」一节附命令与输出），没有一处落在我改的那几行上；checker_narrow 文件上没有任何变异行。因此没有 `mutations-replacements.tsv`、`mutations-append.tsv`、`mutations-delete.txt`，`patch/` 目录只有 `crates.patch` 与本报告。

## 没做什么

- 没碰 A4d（`admission.rs`、`transaction.rs`、`allocator.rs` 及其准入测试）、C554 乙（`recovery.rs`、`mount.rs`、`lib.rs`、`second_transaction_supplement_two_unreadable_abandoned_root_slot.rs`、`second_transaction_step_three_formatted_pool.rs`）、E158（`src/bin/e158_root_choice_repair.rs`）——按派发指令，红了也只列不改。
- 没改任何行为：两处改动都是纯改名（变量名、闭包参数名）与把隐式通配臂改写成显式列出剩余变体，逐条测试跑过、结果与改动前一致（见下）。
- 没提交、没走三方对抗、没跑层 0 / QEMU / herd7 / 变异整表——这些归 crash-verifier 与提交流程，不归这一批。
- `.claude/gate.d/89-closeout-row27-preconditions.sh`、`74-model-differential.sh` 两道本次都退 77（本次未跑，不是通过）：89 号是它自己判定的收口表缺口（与这次改动无关，详情见下）；74 号判「与 HEAD 相比这次改动没有一个路径落在 crates/ 底下」——这是因为我没有把补丁应用进主工作区（按指令在副本里改、交补丁），所以 HEAD 视角下确实还没有 crates/ 改动。等主 agent 用 `apply-writer-patch.py` 打上补丁之后，74 号需要重跑。

## 改法（纯格式/命名/穷举分支，不改行为）

### e156_allocation_basis_counts.rs

1. `run_forward_rollback_scenario` 里 `mount_writable` 重开之后重新拿到的分配器 `let mut allocator = mounted.allocator;`（原第 3312 行）与函数开头 `allocator_after_make_filesystem` 拿到的分配器（原第 3271 行）是两个不相关的绑定，只是重用了同一个名字，clippy 判 `shadow_unrelated`（`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs` 修好之后能一次看全的那三条错误之一）。
   改法：把重开之后那个绑定改名为 `reopened_allocator`（与同一函数里已有的 `reopened_instance`、`reopened` 命名风格一致），随后两处使用（`publish_overwrite` 与 `roll_back_by_a_forward_publish` 的 `&mut allocator` 实参）跟着改名。声明与两处使用都在同一个函数体内，改名前后指向同一个值、同一段生命周期，行为不变。
2. `judge_group` 里两处 `.map_or_else(|| "none".to_string(), |key| key.to_string())`（原第 3880、3882 行）的闭包参数 `key` 与外层函数参数 `key: &GroupKey`（第 3843 行）同名但类型不同（闭包参数是 `smallest_key_above_threshold` 返回的 `u64`），clippy 判 `shadow_unrelated`。
   改法：把两处闭包参数改名为 `smallest_key`，闭包体内 `key.to_string()` 跟着改成 `smallest_key.to_string()`。闭包只在这一条表达式里用这个参数，改名不影响求值结果。

### checker_narrow_invariants_and_abandoned_roots.rs

`InvariantVerdict` 只有三个变体：`Holds`、`Violated(String)`、`NotApplicable(&'static str)`（`crates/singlefs-checker/src/image.rs:30-34`）。两处 `other => panic!(...)` 用通配臂兜底剩下的变体，clippy 判 `wildcard_enum_match_arm`（这条规则要求穷举写全，见 `code-discipline.md`「分支」一节）。

1. 第 188 行（`detail_of_the_violation`）：已经显式处理 `Violated(detail)`，通配臂原本兜的是 `Holds` 与 `NotApplicable(_)`。改成 `other @ (InvariantVerdict::Holds | InvariantVerdict::NotApplicable(_)) => { panic!(...) }`，逐字列出剩下的两个变体，panic 的消息与参数一字未改。
2. 第 618 行（读不出的根环槽那条用例的收尾 `match`）：已经显式处理 `NotApplicable(reason)`，通配臂原本兜的是 `Holds` 与 `Violated(_)`。改成 `other @ (InvariantVerdict::Holds | InvariantVerdict::Violated(_)) => { panic!(...) }`，同样只改分支写法，不改消息。

## 验证：改动前的红（在草稿仓副本 `/tmp/claude-1000/impl-lint-clean/repo/` 里跑，`bash research/scripts/capped.sh 4 …`）

```
$ cargo clippy -p singlefs-harness --all-targets --all-features --keep-going -- -D warnings \
    -D clippy::wildcard_enum_match_arm -D clippy::allow_attributes_without_reason \
    -D clippy::cast_possible_truncation -D clippy::cast_sign_loss -D clippy::cast_possible_wrap \
    -D clippy::undocumented_unsafe_blocks -D clippy::shadow_unrelated
exit=101
```
命中的三个编译单元与错误数（原样节选，`/tmp/claude-1000/impl-lint-clean/clippy-harness-keepgoing.txt`）：
```
error: `mut allocator` shadows a previous, unrelated binding
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:3312:13
error: `key` shadows a previous, unrelated binding
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:3880:54
error: `key` shadows a previous, unrelated binding
    --> crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:3882:54
error: could not compile `singlefs-harness` (bin "e156_allocation_basis_counts" test) due to 3 previous errors
...
error: wildcard match will also match any future added variants
   --> crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs:188:9
error: wildcard match will also match any future added variants
   --> crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs:618:9
error: could not compile `singlefs-harness` (test "checker_narrow_invariants_and_abandoned_roots") due to 2 previous errors
```
（不带 `--keep-going` 时只看得到 e158 那一份的 3 条 `shadow_unrelated`，e156 与 checker_narrow 的红被同一次 `cargo clippy` 调用挡住看不见——这正是三份实现员报告互相没认领到的原因，见证据文件 `/tmp/claude-1000/impl-lint-clean/clippy-harness-full.txt`。）

## 验证：改动之后（同一副本，改完再跑）

```
$ cargo clippy -p singlefs-harness --all-targets --all-features --keep-going -- -D warnings <同上七条> 2>&1 | tail -5
error: `ranges` shadows a previous, unrelated binding
    --> crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:7244:17
...
error: could not compile `singlefs-harness` (bin "e158_root_choice_repair" test) due to 5 previous errors
exit=101
```
`grep -oE '\-\-> crates/[^:]+' clippy-harness-keepgoing-2.txt | sort -u` 只剩一个文件：`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`。e156 与 checker_narrow 两份文件不再出现在错误列表里。

```
$ cargo clippy --workspace --all-targets --all-features --keep-going -- -D warnings <同上七条>
exit=101；错误数 7；命中的文件只有 e158_root_choice_repair.rs（`clippy-workspace-keepgoing.txt`）
```
整仓（不止 harness）扫过一遍，确认 A4d、C554 乙点名的那几份文件（`admission.rs`、`transaction.rs`、`allocator.rs`、`recovery.rs`、`mount.rs`、`lib.rs` 等）此刻在这七条 lint 下都不红，没有需要列出的第二批。

```
$ cargo fmt --all -- --check
exit=1；`grep '^Diff in' fmt-check-2.txt | sed -E 's/^Diff in (.*):[0-9]+:$/\1/' | sort -u`
只剩 /tmp/claude-1000/impl-lint-clean/repo/crates/singlefs-harness/src/bin/e158_root_choice_repair.rs
```
（改动之前的 `fmt-check-full.txt` 同样只有这一份文件、67 处差异，说明 e156 与 checker_narrow 本来就没有 fmt 问题，这次没有格式改动。）

```
$ cargo build --offline --all-targets -p singlefs-harness
exit=0（`build-harness.txt`）
$ cargo build --offline --all-targets   # 整个 workspace
exit=0（`build-workspace.txt`）
```

## 动到的测试二进制：整个跑一遍（经 `research/scripts/run-with-memory-cap.sh 8G`，线程上限 4）

```
$ bash research/scripts/capped.sh 4 bash research/scripts/run-with-memory-cap.sh 8G \
    cargo test --offline -p singlefs-harness --bin e156_allocation_basis_counts
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.69s
exit=0
```
含 `rolling_back_by_a_forward_publish_keeps_the_instance_and_isolates_nothing_after_a_plain_remount`
与 `rolling_back_to_the_oldest_candidate_after_seventy_two_overwrites_is_one_forward_publish`——正是用到重开后那个分配器（改名成 `reopened_allocator`）与 `roll_back_by_a_forward_publish` 的两条用例，全绿，证明改名没有改行为。

```
$ bash research/scripts/capped.sh 4 bash research/scripts/run-with-memory-cap.sh 8G \
    cargo test --offline -p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots
test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.38s
exit=0
```
含 `an_unreadable_root_ring_slot_leaves_the_abandoned_half_of_the_reuse_invariant_not_judged`（走到第 618 行那个 `match`）与另外多条会经过第 188 行 `detail_of_the_violation` 的用例（`erasing_a_unit_only_an_abandoned_root_references_reddens_only_the_reuse_invariant` 等），全绿。

未新增测试，因此没有「先证明会红」的新用例要跑；这两处改的是既有测试目标里的既有代码（生产代码里的一处改名、测试代码里的通配臂改写），验证方式是把改动前后的整份测试跑一遍、结果逐条比对（都是 ok，用例名与数目也没变）。

## 变异表核对：22 条锚定在 e156 文件上的行，改动前后各命中一次

```
$ python3 -c '...'（对 crates/mutations.tsv 里 22 行 e156 的「原文」字段逐条在改动后的文件里 str.count()）
line 197 ... OK
...（22 行全部 OK，无一行 BAD）
total checked: 22
```
`crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs` 在 `crates/mutations.tsv` 里没有任何行（`grep -c` 结果 0），不用核。
22 条锚点里离我改动最近的一条是第 826 行（`E156 M35`）：原文 `allocator_after_make_filesystem(parameters, devices, genesis)`，锚定在第 1458 行的另一处调用（辅助函数内部），不是我改名的第 3271/3312 行那两处调用（那两处原文都带 `&parameters, &devices, &genesis` 的引用形式，与锚点的字面不同，本来就不会被这条变异命中）；改动前后都恰好命中一次。
按派发指令：这一批不改行为、不加新变异；既有锚点没有被格式化挪动，`patch/mutations-replacements.tsv` 不建。

## 补丁与 diff --stat

没有写主工作区；改动在草稿仓副本里（`rsync -a --exclude target --exclude .git` 拷自主工作区），补丁在
`/tmp/claude-1000/impl-lint-clean/patch/crates.patch`。对当时的主工作区验过：

```
$ git apply --check /tmp/claude-1000/impl-lint-clean/patch/crates.patch
APPLY_CHECK_OK（无输出即通过）
```

`git diff --stat -- crates litmus` 因为我没有改主工作区、而主工作区此刻有别的会话留下的 376 个未提交改动（与这次改动无关），直接跑分不出是谁改的；改用 `git apply --stat` 读我自己这份补丁，等价于只看这次改动的 diffstat：

```
$ git apply --stat /tmp/claude-1000/impl-lint-clean/patch/crates.patch
 .../src/bin/e156_allocation_basis_counts.rs        |   10 +++++-----
 ...hecker_narrow_invariants_and_abandoned_roots.rs |    8 ++++++--
 2 files changed, 11 insertions(+), 7 deletions(-)
```
litmus/ 没有改动，不出现在这份 diffstat 里。

## e158_root_choice_repair.rs：列出、不改（E158 执行员在改）

fmt：`cargo fmt --all -- --check` 改动前后都只在这一份文件上报差异，67 处（行号 7719–11017，`fmt-check-full.txt`）。
clippy `shadow_unrelated`（5 处，两个编译单元各报一部分）：
```
crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:7244  `ranges` 与 7231 的 `ranges` 无关重名
crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:7245  `run` 与 7232 的 `run` 无关重名
crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:9950  闭包参数 `geometry` 与 9895 的函数参数 `geometry` 无关重名
crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:12973 `run` 与 12960 的 `run` 无关重名（只在 #[cfg(test)] 编译单元出现）
crates/singlefs-harness/src/bin/e158_root_choice_repair.rs:13126 `run` 与 13111 的 `run` 无关重名（只在 #[cfg(test)] 编译单元出现）
```

## 阶段归属表登记给 implementation-writer 的门禁阶段（在主工作区跑，只读仓里文本与 git 记录，未改任何文件）

```
$ awk -F'\t' -v me="implementation-writer" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv
33-mutation-tables.sh
53-format-const-placeholders.sh
74-model-differential.sh
92-layout-checker-sync.sh
94-checker-implementation-disjoint.sh
93-feature-bits.sh
89-closeout-row27-preconditions.sh
```

- `33-mutation-tables.sh` exit=0：`✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1121 条的原文各命中源码一次；…`
- `53-format-const-placeholders.sh` exit=0：`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
- `92-layout-checker-sync.sh` exit=0：`✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 D15（格式冻结政策） 已定项 4 的登记表；这次改动比 faf255e235300d129ede6d6f85af31d686a88519，106 个格式常量里变了 0 个（checker 在同一次改动里跟了 0 个，按滞后表放行 0 个），都不欠 checker 跟进`
- `94-checker-implementation-disjoint.sh` exit=0：`✓ checker 与实现只共享常量模块 singlefs-format（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；checker 的 4 份源码零处引 singlefs_core（别名引进来的 0 个）；共享模块 1 份源码的正文 286 行里没有分支与循环（#[cfg(test)] 标着的项 313 行不扫）`
- `93-feature-bits.sh` exit=0：`✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；没判位号的 1 处：SUPPORTED_INCOMPAT_BITS（crates/singlefs-core/src/system_configuration.rs:31，常量声明没写在一行里，解不出值））`
- `89-closeout-row27-preconditions.sh` exit=77（本次未跑）：`没做成探针的 2 笔：…第 27 行现算 5 笔；探针与没做成探针的清单两边都没有的 1 笔：被抛弃根的根槽读不出时既不隔离也不计数`「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐——这条与本次改动无关（本次没有碰第 27 行涉及的任何一处），照实登记，不归这一批处理。
- `74-model-differential.sh` exit=77（本次未跑）：`! 本阶段跳过（这次改动没碰它判的东西）：没碰：与基 HEAD 之间共 14 个改动路径，没有一个落在 crates/ 底下`——因为我在副本里改、没有把补丁应用进主工作区，此刻 HEAD 视角下确实没有 crates/ 改动；补丁打上之后需要重跑这一道。

## 收尾：删掉自己建的仓副本

`/tmp/claude-1000/impl-lint-clean/repo`（14G，其中 `target/` 13G）是这一轮自己建的 rsync 副本，交回前已删除。保留：`patch/`（`crates.patch`）与本报告、几份验证日志（`fmt-check-*.txt`、`clippy-*.txt`、`test-*.txt`、`build-*.txt`、`gate-*.txt`），均在 `/tmp/claude-1000/impl-lint-clean/` 下，供主 agent 复核。
