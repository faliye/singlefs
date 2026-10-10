# 9-6 池级 checker 簇实现员报告（2026-10-04）

批次：records/2026-09-30-里程碑三开工第一轮.md#9（9-6）。交补丁，主工作区一个字没动。两个补丁目录：

- `/tmp/claude-1000/m3-r1/batch9-checker/patch/`（本报告所在）：第 2、3、4、5 条。直接可打。
- `/tmp/claude-1000/m3-r1/batch9-checker/patch-item1/`：第 1 条（I-7.9 一圈窗口）。**第 1 条停下交主 agent**（第六节 Q1）；这一目录是「定了加镜像窗口之后」才打的现成补丁，要在本目录打完之后打（它的 `crates.patch` 以本目录打完的树为底）。

## 一、结论

| 条 | 结局 | 一句话 |
|---|---|---|
| 1 I-7.9 窗口 | **停下交主 agent** | 照条款字面（窗口锚在被判那条抬 F 的根上）实现，验收要的三形合法镜像照样红（草稿那一形红的那一句与改之前逐字相同）；随机历史 230 段 21 段红、150 段 9 段红。再加一个锚在镜像最新有效根上的窗口（候选过滤那一个）：三形全绿，随机历史 230 段、150 段各剩 1 段红，是带卸载记号那一支（种子 31045、1037，条款写明那一支照旧）。这一改法的补丁、用例、变异行在 `patch-item1/` |
| 2 I-7.8 认领 | 做完 | `scanned_tree_identifiers` 按认领规则跳（码 1、码 3 两槽，码 2 一槽）；fh-high、fh-foreign 两形转绿，fh-control、fh-inode 照旧绿，真孤儿坏镜像照旧只红 I-7.8 |
| 3 I-9.28 | 做完（GPU 一侧不归我） | 新判法 `judge_tree_identifier_birth_txg_across_every_readable_root`，`IMPLEMENTED_INVARIANTS` 69 条；不适用条件取与 I-9.14 同一种（Q2）；三段合法历史成立、C342 那一形红；第 3619 行那份镜像改成 I-9.28 红、I-9.14 照旧成立 |
| 4 RS4 钉 | bb、bc、bd、be、bf、bg 做完；同 txg 并组没造（Q4） | 五份镜像各自：对照红、条件缺一个时不判 |
| 5 改钉计数 | 做完 | 注释里的计数按 69 条改成现值，五张不适用表各加 I-9.28；`writable_mount_of_a_formatted_pool` 15 条全绿 |

推翻条件：主工作区打上之后，第四节那几个测试二进制有一条红；或 `prove-red.sh` 跑第五节没证的那几行有一行没红。

## 二、写过的文件

本目录 `crates.patch`（`git apply --stat` 原样）：

```text
 crates/singlefs-checker/src/image.rs               |   10 
 crates/singlefs-checker/src/walk.rs                |  146 +++++
 .../tests/checker_known_bad_images.rs              |  127 +++-
 ...ow_conditions_and_incomplete_reference_pairs.rs |  598 ++++++++++++++++++++
 ..._scan_of_tree_identifiers_claims_whole_units.rs |  355 ++++++++++++
 ..._birth_txg_agrees_across_every_readable_root.rs |  473 ++++++++++++++++
 .../tests/writable_mount_of_a_formatted_pool.rs    |   51 +-
 7 files changed, 1705 insertions(+), 55 deletions(-)
```

- `crates/singlefs-checker/src/walk.rs`（行号是打完之后的）：`slots_claimed_by_a_unit_header`（第 3163 行）与 `scanned_tree_identifiers`（第 3190 行）认领；`judge_tree_identifier_birth_txg_across_every_readable_root`（第 4469 行，调用在第 8106 行）；`judge_tree_table_birth_txg`（第 4401 行）的注释照 kb 改（判别力第 ③ 条那一段换成指向 I-9.28）。I-7.9 那一段本目录**没动**。
- `crates/singlefs-checker/src/image.rs`：`TREE_IDENTIFIER_BIRTH_TXG_AGREES_ACROSS_EVERY_READABLE_ROOT = "I-9.28"`，末尾追加（下标 68），表长 68 → 69。
- `crates/singlefs-harness/tests/checker_known_bad_images.rs`：见第三节第 3 条。
- `crates/singlefs-harness/tests/writable_mount_of_a_formatted_pool.rs`：第 5 条。
- 新建三份（都在 `crates/singlefs-harness/tests/`，都在派发提示的清单里）：`pool_checker_scan_of_tree_identifiers_claims_whole_units.rs`、`pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs`、`pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs`。
- 清单里没碰的：`crates/singlefs-checker/src/lib.rs`；`crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`（跑过、20 条全绿，不用改）。
- `mutations-append.tsv` 20 行（名字见第五节）；`mutations-replacements.tsv` 2 行：C511 那两行（主表第 272、273 行）点名的测试改了名，第六段换成新名，别的五段一字不动（第五段的筛选词是新名的前缀，照旧筛得到）。
- 主工作区 `git diff --stat -- crates litmus` 原样：输出为空（主工作区的改动都在暂存区）。
- 核过：`git -C /home/fy5090/code/singlefs apply --check …/patch/crates.patch` 退 0（输出 `APPLY_CHECK_OK`）。主工作区在我拷副本之后被别的会话改过 `walk.rs`（I-5.5、I-9.27 那几段）、`checker_known_bad_images.rs`（I-9.27 三份坏镜像）与 `crates/mutations.tsv`（末尾多 12 行 C673 ⑤ ⑥），与本补丁不在同一处；合在一起没跑过（第七节第 1 条）。

`patch-item1/crates.patch`（以本目录打完的树为底，`git apply --stat` 原样）：

```text
 crates/singlefs-checker/src/walk.rs                |   43 ++
 ..._only_roots_inside_the_one_revolution_window.rs |  417 ++++++++++++++++++++
 2 files changed, 449 insertions(+), 11 deletions(-)
```

它在本目录打完的树上 `git apply --check` 退 0（`ITEM1_APPLY_CHECK_OK`）；变异表 4 行追加、2 行替换（主表「收口表第 26 行 I-7.9：有效根按抬之后的 F 取」与「一圈窗口：checker 的窗口算错一位」两行的原文跟着改，不改锚点就断）。

## 三、逐条做了什么

### 第 2 条（F4）I-7.8 扫描方向认领

- 条款：`.claude/kb/decisions/18-块里携带什么信息.md@已定项 7：第一版单元头字段表，按单元类分` 里「认领规则」一条（认领之后按整单元跳：码 1、码 3 两槽，码 2 一槽）。
- 判法：每块盘的候选槽排序去重后升序走，跨轮带「下一个没被认领的槽」；起点槽上 magic 对、类标签 1 / 2 / 3、头校验和过（码 1 头末 105、码 2 86 + 2k、码 3 107，校验和在 10）就认领，码 1、码 3 跳 `DATA_UNIT_BYTES / SLOT_BYTES` 槽，码 2 跳 `NODE_BYTES / SLOT_BYTES` 槽；被认领单元后几槽不再读头。码 2 才往下照原来两道排除数树 ID。
- 我取的两处（条款没写，报告列出，不当成设计判断：都只关系到「解出一个单元头」怎么认）：认领**不看 fsid**（别的池的单元同样占着那几槽）；**头读不出的槽不认领**、下一槽照扫。
- 用例 `pool_checker_scan_of_tree_identifiers_claims_whole_units.rs` 五条：攻方四臂照原型另写（`high`、`foreign`、`control`、`inode`，各断言 I-7.8 成立且一条违例都没有），坏镜像一条（没被认领的槽 = 单元区起点 + 100000 槽上写一个树 ID 2⁴⁰ 的真孤儿码 2 节点，断言只红 I-7.8、说明里有 1099511627776）。
- 认领规则留的缝（释放了的两槽单元第一槽被一槽的码 2 节点拿走、第二槽旧载荷还在）：**没造出用例**。合法历史上要分配器把一个码 2 节点放进被释放数据单元的第一槽、第二槽又一直空着；攻方原型 `fake_freed` 臂覆盖写 60 次没撞上（`research/prompts/m3-sweep-and-tree-id-r1-opus-model/results/out/th-fake_freed.out` 末行 `STEP 60 txg=63 fakes_on_disk=0 visible_under_claim_rule=0 fakes_in_slots_referenced_by_a_ring_root=0`：伪头在第 5 步前就被盖掉了）。手改镜像（把数据单元起点那一槽换成一个码 2 节点头）能让它红，但那不是合法历史，钉它就是钉一个误红，没写。另一个方向的缝（推的，没量）：被释放的两槽数据单元头还在第一槽、第二槽被一个真码 2 节点拿走时，旧头认领两槽，那个真节点不被数——少数，只会漏红、不会误红。

### 第 3 条 I-9.28

- 条款：`.claude/kb/invariants.md` 的 I-9.28 一行；与 I-9.14 分开写（D13（验证路线） 已定项 5：判法 checker 自己写，不与 core 共用；也不与 I-9.14 的 `judge_tree_table_birth_txg` 共用，I-9.14 的判法一行没动）。
- 判法：根环里每条自证过的根（`check_pool_image` 的 `roots`，不按实例表、F、窗口滤）各按 `references_of_root(…, ReferenceScanDepth::TreeTableOnly, …)` 读树表，按树 ID（条目偏移 0）归并，诞生 txg 不一就红，说明里带每条根的（实例、txg）与记的值。
- 不适用（Q2）：取与 I-9.14 同一种——没有一棵树的条目出现在两个不同的树表单元里时整条报不适用。树表指针全零、读不出、条目比字段表短的根只少一份证据，不进比较、不另报「部分没判」。
- `checker_known_bad_images.rs`：
  - 第 3619 行那条改名为 `birth_txg_recorded_differently_only_on_the_timeline_cut_off_by_a_recovery_holds_the_same_line_invariant_and_reddens_only_the_every_readable_root_invariant`：先核干净镜像上每条照 `expected_verdict_with_one_record_per_transaction`（I-9.28 成立），再施加新列表 `known_bad_images_of_the_birth_txg_across_every_readable_root` 里那份坏镜像（只改被抛弃的 C 那片树表里 extent 树的诞生 txg）：I-9.14 成立、I-9.28 红在「树 65536」与「txg C）记 C」上、别的每条与干净镜像相同。
  - `birth_txg_that_the_line_after_a_rollback_…` 改名为 `…_reddens_only_the_same_line_and_the_every_readable_root_birth_invariants`，断言红的是 [I-9.14, I-9.28]（同线不一必然全部可读根里不一）。
  - `each_release_generation_and_tree_table_birth_bad_image_reddens_only_its_own_invariant_after_the_overwrite`：I-9.14 那份坏镜像的期望改成 [I-9.14, I-9.28]，名字没改（这个名字点着主表 10 行变异，改名要换 10 行；名字里「only its own」对 I-9.14 那一份不再字面成立，交主 agent 定改不改）。
  - `the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target`：干净镜像上 I-9.28 登记为不适用；「每条不变量至少一份坏镜像」那张并集里并进新列表（为此在这条里多建一次「崩溃恢复抛弃了 C」那份镜像）。
- 用例 `pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs` 四条，照攻方 `tree_id_histories` 另写：`orphan_ff`（断言 I-9.28 成立）、`applied_ff`（挂载之后不红；我多加一次覆盖写，之后断言成立——攻方那一臂没有这一步，加上才一定有两片带条目的树表可比）、`rollback`（成立）、`c342` 带系统配置退回（断言实例 2 重发的水位是 65548、放回被藏的根之后 I-9.28 红在树 65536 上）。
- GPU：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 第 78 行起那条编译期断言（覆盖表 + 「内核还没判」表的长度 = `IMPLEMENTED_INVARIANTS` 长度）在带 `gpu` 特性时编不过，要把下标 68 补进内核或补进「内核还没判」那张表；那几份文件 9-3 在改，我没碰，排在 9-3 交回之后。

### 第 4 条（RS4）I-3.5 gap-row 与引用不全

用例 `pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs` 五条；每份镜像都把洞右边（或这一对后一版）一个新写的单元诞生代号改小到 R⁻ 的 txg（按 (a) 它该红），条件缺一个时这一对不该判到它：

| 用例 | 镜像 | 钉的探针 |
|---|---|---|
| `a_hole_whose_row_for_the_previous_instance_names_another_txg_…` | A (1,3) → 重开实例 2（写行、暖机）→ D 覆盖写；实例 2 在 D 之前的根槽改坏、D 的数据单元诞生代号 → 3。行 (1,3) 不改：红（对照）；行改成 (1,4)：成立 | bb、be |
| `a_hole_with_an_instance_between_that_has_no_row_…` | A → 实例 2 → 实例 3 覆盖写 D，A 与 D 之间的根槽都改坏；D 的实例表里实例 2 那一行的实例号改成 9：成立 | bc |
| `a_hole_whose_instance_between_has_a_row_that_is_not_zero_…` | 同一段历史；实例 2 那一行的 T 改成 1（行在、不是 (0,0)；T 改小是为了让实例 2 那几条记录按行判成被抛弃，否则实例 2 最后那一版由记录施加出来、它与 D 那一对接得上，判的就不是 A 与 D 之间的洞——第一版没改 T 时这一条就是这样红的）：成立 | bd |
| `a_pair_whose_previous_version_has_an_incomplete_reference_set_is_not_judged` | A → B，B 的数据单元 → 3：红（对照）；再把只有 A 引用的 A 的 extent 树根两盘各翻一个字节：不红 | bf |
| `a_pair_whose_later_version_has_an_incomplete_reference_set_is_not_judged` | A → B → C，B 的树表 → 3：红（对照）；再把只有 B 引用的 B 的 extent 树根翻坏：不红 | bg |

同 txg 并组（Q4）没造。改镜像的那几个辅助函数（`reseal_unit`、`units_on_disk`、`propagate`、`rewrite_unit_and_propagate` 等）照 `checker_judges_reference_intervals_and_every_new_root_segment_field.rs` 另写了一份（那份文件不在我的清单里，抽公共模块要新建清单外的文件）。

### 第 5 条 改钉计数

`writable_mount_of_a_formatted_pool.rs`：「21 条 / 25 条」→ 31 / 38；「多报两条」→ 多报四条（I-8.6、I-8.9、I-8.10、I-8.12，按表实数）；「少 … 四条」→ 少七条（补上 I-8.11、I-3.12、I-3.5 三条的理由）；用例文档注释的「23 / 25 / 42 条成立」→ 34 / 38 / 62（69 − 35、69 − 31、69 − 7）。五张表都加 I-9.28（树表条目只出现在一片树表里，与 I-9.14 同一个理由），表长 30→31、34→35、5→6、6→7、23→24。条数都是按今天的 69 条与表长现算的。

## 四、验证（第 4 步那几样，末尾原样）

本目录那棵树（打完本补丁、没打 `patch-item1` 的样子），一条脚本 `/tmp/claude-1000/m3-r1/batch9-checker/final.sh` 跑完，各段退出码各写一个文件再逐段读：

```text
fmt rc=0
checker-lib rc=0
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.33s
harness rc=0
     Running tests/checker_known_bad_images.rs (target/debug/deps/checker_known_bad_images-fa5092303c98133d)
test result: ok. 69 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.06s
     Running tests/checker_narrow_invariants_and_abandoned_roots.rs (target/debug/deps/checker_narrow_invariants_and_abandoned_roots-416add8e921bedb7)
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.28s
     Running tests/pool_checker_gap_row_conditions_and_incomplete_reference_pairs.rs (target/debug/deps/pool_checker_gap_row_conditions_and_incomplete_reference_pairs-fea6059af961e3a9)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.40s
     Running tests/pool_checker_scan_of_tree_identifiers_claims_whole_units.rs (target/debug/deps/pool_checker_scan_of_tree_identifiers_claims_whole_units-a872931051d946d8)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.40s
     Running tests/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root.rs (target/debug/deps/pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root-39d6c04416f740cb)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.16s
     Running tests/writable_mount_of_a_formatted_pool.rs (target/debug/deps/writable_mount_of_a_formatted_pool-d604020127c6ce48)
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.11s
build rc=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 05s
```

（fmt 是 `cargo fmt --all -- --check`；build 是 `cargo build --offline --all-targets`，整个工作区、默认特性；测试都经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`。）

clippy：在打了两个补丁的那棵树上跑 `cargo clippy --offline --keep-going -p singlefs-checker -p singlefs-harness --all-targets -- -D warnings` 加 `check.sh` 里 `CODE_DISCIPLINE_LINTS` 那七条 `-D`（照那份脚本第 72–80 行现抄），`grep -E '^error' | sort | uniq -c` 原样：

```text
      1 error: could not compile `singlefs-harness` (test "an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping") due to 2 previous errors
      2 error: `unit` shadows a previous, unrelated binding
```

红在 `crates/singlefs-harness/tests/an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs` 第 181、182 行，不是这一轮的文件（第 8 批改钉报告第七节第 1 点已报过）；`--keep-going` 之下别的目标一条告警都没有。我自己文件里第一趟查出的两处 `shadow_unrelated`（I-9.28 用例 C342 那一条里的 `devices`、`image`）已改名。**和定义的偏离**：没带 `--all-features`、没跑整个工作区——`--all-features` 会开 `singlefs-checker-tier` 的 `gpu` 特性，`crash_judge_gpu.rs` 那条编译期断言在 69 条下必编不过（第三节第 3 条末），再加 RocksDB 现编 C++，前台 240 秒跑不完；带特性的那一遍归 9-3 交回之后。

`patch-item1` 那棵树（两个补丁都打）：第 1 条那份用例文件 4 条全绿，在 `run3`（日志 `/tmp/claude-1000/m3-r1/batch9-checker/logs/run3-harness.log`，`test result: ok. 4 passed; 0 failed`；同一趟里第二节那 6 个二进制也全绿）；那之后这棵树只改了注释、`cargo fmt` 与 C342 用例里两个改名，没再跑测试。

## 五、证红

`bash research/scripts/prove-red.sh --copy <副本> --memory 8G singlefs-harness <名…>` 跑了 4 行，原样末尾：

```text
第9批 I-7.8 认领（F4）：认领之后一律只跳一槽（关掉按整单元跳，数据单元第二槽上的伪头又被数进出现过的树 ID）	抓到	a_node_header_forged_on_the_second_slot_of_a_file_unit_is_claimed_by_that_unit_and_the_watermark_invariant_holds 红了（日志 /tmp/claude-1000/m3-r1/batch9-checker/logs/prove-red/001.log）
第9批 I-9.28：恒判成立（C342 那一形整卷回滚之后放回被藏的根不再红）	抓到	tree_identifiers_reissued_after_a_whole_volume_rollback_redden_the_every_readable_root_birth_invariant_once_the_hidden_roots_come_back 红了（日志 /tmp/claude-1000/m3-r1/batch9-checker/logs/prove-red/002.log）
第9批 I-3.5 gap-row（RS4 探针 bb）：去掉「有行 (R⁻ 的实例, T = R⁻.txg)」整个条件	抓到	a_hole_whose_row_for_the_previous_instance_names_another_txg_is_not_bridged_and_a_user_visible_unit_born_before_it_is_not_judged 红了（日志 /tmp/claude-1000/m3-r1/batch9-checker/logs/prove-red/003.log）
第9批 I-3.5 引用不全（RS4 探针 bf）：前一版引用不全不再跳过这一对	抓到	a_pair_whose_previous_version_has_an_incomplete_reference_set_is_not_judged 红了（日志 /tmp/claude-1000/m3-r1/batch9-checker/logs/prove-red/004.log）
✓ 点名 4 条：跑了 4 条，跳过 0 条，跑的都抓到了
```

改坏哪一行 → 哪条断言红（日志里的原样）：

| 变异 | 红的断言 |
|---|---|
| `first_slot_not_claimed = slot + claimed_slots;` → `slot + 1;` | 认领用例第 257 行 `assert_eq!` 的 I-7.8：`left: Violated("根环水位最大 65548，盘上出现过的最大树 ID 1099511627776")`、`right: Holds` |
| `birth_txg_is_the_same_in_every_readable_root,` → `true,` | I-9.28 用例第 470 行 panic：`老线与新线上同一树 ID 65536 诞生 txg 不同，I-9.28 要红，得到 Holds` |
| `row_of_the_previous_version` → `(row_of_the_previous_version \|\| true)` | gap-row 用例第 473 行：`left: Violated("相邻两版 (实例 {1}、txg 3) → (实例 2、txg 6)（HoleBridgedByTheInstanceTableRows）：…单元盘 0 槽 50182…诞生代号 3 不大于前一版的 txg…")`、`right: Holds` |
| `previous_references_are_complete = …all(…)` → `= true;` | 第 556 行：`A 的引用集合数不全，(A, B) 那一对不判：Violated("相邻两版 (实例 {1}、txg 3) → (实例 1、txg 4)（NoHole）：…槽 50182…")` |

基线：prove-red 每组参数先跑的基线都过了（它判的；`logs/prove-red/baseline.log`）。另有两笔不经 prove-red 的现场证据：第 1 条只取条款字面那个窗口时三条用例红（`logs/run2-harness.log`，红的说明见第六节 Q1）；bd 那一条第一版没改 T 时红在 (实例 2、txg 5) → (实例 3、txg 8) 那一对上（同一份日志）。

**没证的**：本目录 20 行里另 16 行、`patch-item1` 的 4 行，原因是一次派发 10 次编译 / 测试的额度用完了（第九节）。整表复跑归提交时的 checker-tier-crates-mutation-replay；要先单证的命令在第七节第 2 条。没证的行里，每条新测试都有一行点着它（第 2 条五条、第 3 条四条加 `checker_known_bad_images.rs` 两条改名的与两条改了期望的、第 4 条五条、第 5 条一条），已证的 4 行罩住其中 4 条。

## 六、停下交主 agent 的设计问题

### Q1（第 1 条，停下）：I-7.9「之前的根」的窗口锚在被判那条根上，挡不住抬 F 之后才出现的洞

条款原文（`.claude/kb/invariants.md` I-7.9 一行）：「「之前的根」取一圈窗口里 txg 比它小的那些：窗口的锚取被判那条根的 txg（N = 那条根的 txg，窗口 = txg > N − 3S，照 D16（发布语义） 已定项 1「一圈窗口里最旧有效根」那一行）」。

- 为什么不够（推的，下一条量过）：洞是 txg t 的根槽没写、那一槽留着 t − 3S 的根。t 晚于抬 F 那条根 r 时 t − 3S > r − 3S，那条旧根落在锚在 r 上的窗口里；而比它新、当初撑着上限的几条根已被 t 之后的发布盖掉，`rollback_floor_ceiling_from` 对计入的集合单调，上限变低。
- 量的（甲 = 照条款字面；乙 = 甲再并一个锚在这张镜像根环里最新有效根上的窗口，即候选过滤那一个 `OneRevolutionWindow`）：

| 形 | 今天 | 甲 | 乙 |
|---|---|---|---|
| 删除与并发那条线的草稿（卸载那一串，再挂） | 红 | 红：`实例 2 txg 13 那条根把回退下界 F 从 0 抬到 9，高于它之前的根算出的抬 F 上限 7：每块盘上最新的有效根 {0: 12, 1: 7}，非空有效根 [7, 11, 12]，空不空判不了的有效根 []，最旧有效根 txg 7` | 不红 |
| inject-6 种子 1（实例 5 txg 25 扣下，由记录施加） | 红 | 红：`…上限 1：…空不空判不了的有效根 [1, 6]，最旧有效根 txg 1` | 不红 |
| K1 类（实例 6 txg 26 写行那次扣下） | 红 | 红：`…上限 2：…空不空判不了的有效根 [2, 6]，最旧有效根 txg 2` | 不红 |
| 随机历史 230 段（code-r1 那五件 q11、q12、r10–r12） | 24（8-3 报告） | 21 | 1（种子 31045） |
| 随机历史 150 段（m3-ring-r3 种子 1000–1149） | 14（8-3 报告） | 9 | 1（种子 1037） |

  甲那三行原样出自 `logs/run2-harness.log`；乙在 `logs/run3-harness.log`（4 条全绿）。随机历史用 `research/prompts/m3-root-ring-code-r1-opus-model/rr-attack-proto/src/bin/window_fuzz.rs` 原样（窗口臂），在副本里分别编甲、乙两版的池级 checker，汇总用同目录 `scripts/fz-summary.py`；输出在 `/tmp/claude-1000/m3-r1/batch9-checker/fz/out-{A,B}{230,150}-*.out`，230 段用 1 MiB journal、256 MiB 盘，150 段用原型默认。每件 `checker_red_seeds` 里红的都只有 I-7.9。
- 乙剩下的两段：种子 31045、1037，8-3 报告第四节把这两颗种子归在带卸载记号那一支（我没再查那两张镜像的说明，归类是照它的，推的）。条款写明那一支「根环里 txg 比它小的最新那条根」照旧，所以乙也到不了验收要的 0。
- 乙为什么不会误红（推的）：洞里留下的根都 ≤ N_镜像 − 3S，被锚在镜像上的窗口挡掉；窗口里 txg 比 r 小的根在抬 F 那一刻就在环里、之后不会被盖（盖它要一条 txg ≥ 它 + 3S > N_镜像 的根），所以留下的是抬 F 那一刻那批根去掉最旧的一段前缀，重算只会更宽。代价：镜像比 r 新出一圈以上时 r 之前一条根都不剩、这条根不判（抓得到的过头抬 F 更少）。
- 要定的：① 条款改成乙（「之前的根」取两个窗口的交，或直接写锚在镜像根环里读得出的最新有效根上）还是别的；② 带卸载记号那一支要不要也取窗口（不取，乙在这两颗种子上照样误红）。定乙就打 `patch-item1/`（它的报告写了改了什么）；定别的，那一目录作废。
- 本目录 I-7.9 一行没改：照条款字面的甲没有一条能红的用例（甲比今天只少误红，三形验收照样红），不单独交。

### Q2（第 3 条，我取了一种）：I-9.28 什么时候报不适用

条款没写。我取：根环里读得出的根中没有一棵树的条目出现在两个不同的树表单元里时整条报不适用（与 I-9.14 同一种，C374 那一类恒真判定不报成立）；某条根的树表读不出不算「部分没判」。推翻它的会是：有一份合法镜像上某条根树表读不出、而读得出的那几条恰好都一致，主 agent 认为该报「部分没判」。

### Q3（第 2 条，记一笔）：认领不看 fsid；反向的缝

认领只看 magic、类标签、头校验和。反向的缝（推的，没量）：被释放的两槽单元头还在第一槽、第二槽被一个真码 2 节点拿走时，旧头认领两槽，真节点不被数——只会漏红 I-7.8，不会误红。条款「这条规则要做成 checker 里能判红的形式」那半句（认领本身判红）我没做：条款没写判什么红。

### Q4（第 4 条）：同 txg 并组造不出坏镜像

候选集里两版同 txg 只有两条路：根环里两条同 txg 的根（根槽位置由 txg 定，I-7.17（根槽位置等于 txg 算出的位置）要求一个 txg 只有一个槽，两条要共存只能有一条住错槽，那张镜像 I-7.17 先红），或一条根与一版由记录施加出来的版本同 txg（新实例取号取环与 journal 里见过的最大 txg 加一，合法历史上不重号；C342 那种整卷回滚才可能）。手造一张「一条根抄进别的槽、实例号改掉」的镜像能让不并组的放松红，但那一形本身已违 I-7.17，钉它钉的是一张非法镜像上的判法细节；没写，交主 agent 定要不要。

## 七、要主 agent 跑的命令

1. 打上之后（与别的会话那几处合在一起）先跑动到的 harness 档二进制，工作目录主仓，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`：
   `nice -n 19 bash research/scripts/run-with-memory-cap.sh 8G bash research/scripts/capped.sh 10 cargo test --offline --no-fail-fast -p singlefs-harness --test checker_known_bad_images --test writable_mount_of_a_formatted_pool --test checker_narrow_invariants_and_abandoned_roots --test pool_checker_scan_of_tree_identifiers_claims_whole_units --test pool_checker_tree_identifier_birth_txg_agrees_across_every_readable_root --test pool_checker_gap_row_conditions_and_incomplete_reference_pairs`，再加 `cargo test -p singlefs-checker --lib`；看每个 `test result:` 行都是 `0 failed`。打了 `patch-item1` 的再加 `--test pool_checker_floor_raise_ceiling_takes_only_roots_inside_the_one_revolution_window`（4 条）。
2. 没证的变异行（在主仓的一份副本上，打完补丁之后；每行约一分钟，前台 240 秒放得下四五行，要分几次或放后台）：
   `nice -n 19 bash research/scripts/capped.sh 10 bash research/scripts/prove-red.sh --copy <副本> --memory 8G singlefs-harness <名…>`，名字是本目录 `mutations-append.tsv` 第 2–8、10–14、16–18、20 行的第一段（16 个），与 `patch-item1/mutations-append.tsv` 的 4 行（打了它的话）；看末行 `✓ 点名 N 条：跑了 N 条，跳过 0 条，跑的都抓到了`。
3. GPU 一侧（9-3 交回之后）：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 里把 I-9.28 的下标 68 补进内核覆盖表或「内核还没判」那张表（`POOL_CHECKER_INVARIANT_INDEXES_THE_KERNEL_DOES_NOT_JUDGE_YET`），否则 `--features gpu` 编不过；之后 `cargo clippy --all-targets --all-features -- -D warnings`（加 `CODE_DISCIPLINE_LINTS`）整仓那一遍。
4. checker 档（提交时，`SINGLEFS_HEAVY_TESTS=commit`）：门禁 `checker-tier-layer0-replay` 快档与第八节点名的用例，`checker-tier-crates-mutation-replay` 整表。

## 八、受影响的层 0 流与崩溃枚举用例

（推的，一条都没跑：checker 档不归我。）

- **I-9.28 新立**（本目录）：层 0 每个崩溃状态多判一条（`IMPLEMENTED_INVARIANTS` 下标 68）。登记了每条不变量期望判定的流要补 I-9.28 那一格：`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs`、`crash_enumeration_new_pool_file_creation_stream.rs`（两份里有 I-9.14 的登记，I-9.28 多半与它同报不适用或同成立）；逐条遍历 `IMPLEMENTED_INVARIANTS` 判「不红」的 `crash_enumeration_record_spill_over_stream.rs`、`crash_enumeration_multi_record_publish_stream.rs`、`crash_enumeration_writable_mount_of_a_formatted_pool.rs`、`crash_enumeration_writable_mount_of_an_io_min_8192_pool.rs`、`crash_segments_per_device_and_torn_in_place_overwrites.rs`、`crash_enumeration_position_addressed_trees.rs`、`crash_enumeration_tree_split_streams.rs`（合法状态上 I-9.28 红只在 C342 那种整卷回滚之后，推的不会红，计数类钉值可能变）；GPU 对拍 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 与 `gpu_judge_*_like_the_cpu_checker.rs`（CPU 多一位，GPU 那一侧没补之前逐位比会差）；`src/layer0_progress.rs`、`src/crash_amplification.rs`、`src/crash.rs`、`src/crash_injection.rs` 按表遍历或按下标编位，判定版本 / 复用键若带不变量条数，崩溃放量库里存的旧结论要按新版本重判。
- **I-7.8 认领**（本目录）：扫描方向少数一类节点（两槽单元起点之后那一槽里的码 2 头），只会让 I-7.8 少红。`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs` 两条崩溃孤儿流的钉值若数到「落在一个仍有头的两槽单元第二槽里的孤儿节点」会变（推的）；`gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs` 与 GPU 判器里 I-7.8 的扫描要同一条认领规则，否则 GPU 对拍在这一形上分叉。
- **I-3.5**（本目录）：判法没动，只加用例。
- **I-7.9 两个窗口**（只在打了 `patch-item1` 时）：`crash_enumeration_fixed_script_stream.rs` 里「抬过 F 的三条流要求真被评估过」那几格、`crash_enumeration_floor_raise_pushed_by_admission.rs`、`rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`：镜像比抬 F 那条根新出一圈以上时那条根改成不判，「真被评估过」的计数可能变少；原先在洞上误红的状态转绿。GPU 判器的 I-7.9 也要取同样两个窗口。

## 九、额度、负载、草稿目录

- 编译 / 测试的 cargo 调用按 Bash 调用数：run1（checker 库单测 + harness 首编，编不过一处）、run2、run3（乙的现场证据）、两次编攻方原型（乙、甲各一次；另有一次被内存包装以「超峰值理由不到 8 个字」退 251、命令没跑）、两次 clippy、一次合成的末尾验证（fmt + 两段测试 + build）、一次 prove-red，合计 10，到额度停下，所以第五节只证了 4 行。
- 开跑前 `ps` 看到别的会话在跑看门狗 `research/scripts/watch.sh`、几条 cargo / rustc（我数到 7 个进程），没有 `qemu-system`、`fio`、`vm-bench.sh`、`e152-file-system-benchmark`；没等锁。
- 草稿目录 `/tmp/claude-1000/m3-r1/batch9-checker/` 留着：`patch/`、`patch-item1/`、`logs/`（全部原样日志：`run1-*`、`run2-*`、`run3-*`、`final-*`、`clippy1.log`、`clippy2.log`、`protoA-build.log`、`protoB-build.log`、`prove-red-1.log`、`prove-red/`）、`fz/`（随机历史 22 份输出与跑法 `fz/batch.sh`、件说明 `specs.txt`、`specs-r3.txt`）、`walk.rs.literal-clause`（甲那一版 walk.rs，量随机历史用）、`walk.rs.variant-b`（乙那一版、没加注释之前）、`anchorcheck.py`（逐行核变异表原文在文件里恰好命中一次的小脚本）、`mut-merged*.tsv`（两棵树各自用的合并变异表）、`run1.sh`–`run3.sh`、`final.sh`。
- 随机历史的产物没进 `research/results/`：我的写范围只到 `crates/`、`litmus/` 与草稿目录；要留就由主 agent 拷，路径在上一行。
- 已删：三份仓副本（`tree`、`tree1`、`treeP`）连同各自的 `target`、两份做 diff 用的拷贝（`diffroot`、`diff2`）、一份原型二进制拷贝（`binB`）。

## 十、没做什么

- 没走三方对抗；checker 档（`checker-tier-layer0-replay` 快档与全量、QEMU、herd7、crates 变异整表、Verus 形式化验证）一样没跑，归提交时的主 agent；没提交。
- 第 1 条停在设计问题上（Q1），本目录不带 I-7.9 的改动；带卸载记号那一支没动。
- GPU 那一侧的 I-9.28、认领规则、两个窗口都没做（9-3 的文件）。
- 同 txg 并组的钉没造（Q4）；认领规则那条缝没造出合法历史上的用例（第三节第 2 条）。
- clippy 没带 `--all-features`、没跑整仓（第四节）；第二节说的与别的会话改动合在一起之后没跑过。
- 第五节 20 行变异没证。
- kb 一处没写（`.claude/kb/invariants.md` 里 I-9.28 的实现状态、I-7.8 认领、I-3.5 钉的登记都要 kb-scribe 跟着改；`invariants.md` 第 12 行那个「判 N 条」的数也要按 69 改，推的，没核那一行今天写的是几）。
