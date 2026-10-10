# m3-batch57-code-r1 正推腿（Sonnet）报告

轮：m3-batch57-code-r1；立场：正推（核代码做的是不是条款说的）；格：K1–K4。
冻结副本：`~/.local/share/singlefs/work/m3-batch57-code-r1/repo/`（sha256 与 `research/prompts/m3-batch57-code-r1-snapshot/crates-sha256.txt` 逐份核过，8 份全 OK）。我的拷贝与编译目录：`~/.local/share/singlefs/work/m3-batch57-code-r1-sonnet/{repo,target}`。
命令一律 `nice -n 19`、经 `research/scripts/run-with-memory-cap.sh 10G`、`capped.sh 10`、一次一条 cargo。开跑前 `ps` 看到主会话有一条 `cargo test --offline --release -p singlefs-checker-tier --features verdict-store,gpu`（pid 2390492）在跑，用的是主工作区 `target/`，不是性能测量，我用自己的编译目录照跑。

## 〇、条款（kb 文件自己的行号，均 `grep -nF` 现查）

- `.claude/kb/decisions/18-块里携带什么信息.md` 第 578 行：`3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。`
- 同文件第 576 行起 `1. **墓碑树**……容器号从 1 起，下一个号 = 树里最大号 + 1，零根时 1`（容器号从 1 起的条款；第 580 行第 5 条谱系同）。
- `.claude/kb/invariants.md` 第 309 行 I-9.21、第 310 行 I-9.22。I-9.22 今天的原文只写「每个打包记录类型 1 容器记录数 ≤ 旋钮值」，没有「非最右那片 == 旋钮值」：代码比 kb 多一句，kb 一行待随写回改（判决第三节 H1 已记）。
- I-9.36、I-9.37：`grep -n 'I-9.36\|I-9.37' .claude/kb/*.md` 零命中，kb 里没有这两条，条款依据只能落到 D18 已定项 24 第 3、5 条。
- I-1.10（`invariants.md` 第 35 行）只管「条目宽先于按字段表解条目」；`grep -rn 'E142 走读\|先判得了自己\|分配记录代与跨度\|分配记录逐条' .claude/kb/*.md .claude/kb/*/*.md` 零命中：冷走读里「分配记录代与跨度」排在两个数之前，kb 里没有条款，次序来自 `m3-batch52-54-code-r1` 判决 G1 与 `m3-batch56-code-r1` 判决 H4（`research/prompts/`），只在 `recovery.rs` 注释里。

## 一、K1　旋钮支装满（甲 H1）

### K1-1 合法镜像上误报：判定「一致」，另有一条前提「规则没说」

条款侧：`.claude/kb/decisions/18-块里携带什么信息.md:578`：「3. **开放容器**：有新记录的那次发布才重写，不删除的发布照抄；满了才关；关闭路径加只供测试的容量旋钮，旋钮不落盘。」
代码侧（冻结副本）：
- 写侧只有一个追加函数：开放容器「还装得下」才追加，装满才开新片，`crates/singlefs-core/src/container_tree.rs:537`：「.is_some_and(|open| open.records.len() < records_per_container);」。三棵树各一个包装调它：`crates/singlefs-core/src/tombstone_tree.rs:126`、`crates/singlefs-core/src/lineage_tree.rs:109`、`crates/singlefs-core/src/retirement_tree.rs:132`。`recovery.rs:4257` 的 `RecordContainer {` 是读盘解出来的，不是写。所以一个实例内旋钮不变时，非最右那片恒 == 旋钮值，新判不会在写侧自己写出的镜像上红。
- 新判把最右那片排除：墓碑 `crates/singlefs-checker/src/walk.rs:12154`：「if is_the_rightmost_tombstone_container_under_the_knob {」，谱系与退役 `crates/singlefs-checker/src/walk.rs:12055`：「if is_the_rightmost_under_the_knob {」；GPU 循环上界不含最右，`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3356`：「for (var container_index = 0u; container_index + 1u < d_container_count; container_index = container_index + 1u) {」。
- 根长高：容器按容器头读叶序、不沿根条目读（`walk.rs` 12074 起的注释），判的是叶序里的 (容器号, 记录数) 表，与根几层无关。

我复跑的（原样输出在 `research/prompts/m3-batch57-code-r1-sonnet-model/base-slow.log`）：
`cargo test -p singlefs-harness --test container_trees_grow_a_level_when_the_root_is_full -- --ignored --nocapture`（经内存包装 10G、capped 10）：
```
CONTAINER_TREES_RANDOM_HISTORY seed=7 records_per_container=2 containers(tombstone, lineage, retirement)=(19, 5, 2) levels(tombstone, lineage, retirement)=(5, 3, 1)
test a_random_history_under_every_container_knob_of_two_records_reddens_no_invariant_at_any_step ... ok
CONTAINER_TREES_RANDOM_HISTORY seed=7 records_per_container=1 containers(tombstone, lineage, retirement)=(37, 9, 4) levels(tombstone, lineage, retirement)=(6, 4, 2)
test a_random_history_under_every_container_knob_reddens_no_invariant_at_any_step ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 226.84s
```
即 60 步建号、unlink、回退的随机历史，旋钮 1、旋钮 2 各一遍，每步 0 红，历史走完根已长到 5、6 层（根长高）、回退在内（回退照抄）。「只给一棵树装旋钮」：`checker_known_bad_images.rs` 的 `checker_knobs` 只给自己那一棵装、另两棵当没装（`crates/singlefs-harness/tests/checker_known_bad_images.rs:16417` 起），基线 253 条全绿：
```
test result: ok. 253 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 100.32s
```
（这是冻结副本原样、未加我的探针时的基线。）

「规则没说」那一格：旋钮在一个镜像的历史中途变了（换实例重挂时旋钮各异）。D18 已定项 24 第 3 条只说旋钮不落盘，没说一份镜像的历史里旋钮能不能变；池级 checker 却只收一个旋钮值判整份镜像。我写了探针（`research/prompts/m3-batch57-code-r1-sonnet-model/probe_k1_mixed_knob_tests.rs`，追加进拷贝的 `checker_known_bad_images.rs` 末尾）：墓碑树先在旋钮 2 下写 3 条（[2, 1]），旋钮改 1 写 1 条（[2, 1, 1]），旋钮改回 2 写 1 条（[2, 1, 2]），每步写入口都是写侧自己合法的追加。原样输出：
```
PROBE_K1B after knob 2,2,2 then 1: [2, 1, 1]
PROBE_K1B after knob 2 again: [2, 1, 2]
PROBE_K1B judged_with_knob=1 violated=["I-9.22"]
PROBE_K1B judged_with_knob=2 violated=["I-9.22"]
PROBE_K1B judged_with_knob=3 violated=["I-9.22"]
```
三个旋钮值都判红；改之前（只判 ≤ 旋钮值）旋钮 2 下 [2, 1, 2] 是绿的。另一条：旋钮 2 写 [2, 1]、改 3 再写一条成 [2, 2]，旋钮 2 判绿、旋钮 3（写入口最后用的）判红（`PROBE_K1 stage=[2,2](knob 2 then knob 3 write) judged_with_knob=3 violated=["I-9.22"]`）。
- 含义：新判多了一个隐含前提「整份镜像的历史用同一个旋钮值写」。仓里今天没有测试这样写（主 agent 跑的 1999 条全绿；我跑的 `-p singlefs-harness` 三个目标也全绿），所以不是现有用例的误报；但以后有人写「换实例改旋钮」的历史用例并交新入口，会在合法历史上红。归「规则没说」，建议记入 I-9.22 / I-9.37 写回那一行（前提：同一份镜像的历史旋钮不变）。
- 推翻条件：若有测试 / 装置今天就在一份镜像的历史里改旋钮并交新入口判，而它绿——则此格不成立。我 `grep -rln 'set_deletion_switches\|CappedForTests'` 看过 harness 与 checker-tier 的测试：压容器容量的目标里，没有哪份在一条历史中途换容量（checker-tier 里两个带 `CappedForTests` 的流压的是节点容量与批大小，不是容器容量）；没有逐个读全 194 个测试二进制，那部分是「没穷举」。

### K1-2 坏镜像漏报：判定「一致」（CPU 读码加复跑）；GPU 一半「复核不了」

三棵树各漏不漏，逐形：
- 旋钮下早关（旋钮 1 写、旋钮 2 / 3 判）：新增坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs:16940` 起三条（墓碑 / 谱系 / 退役各一）在基线全绿；断言是「只红自己那一棵的 I-9.22 / I-9.37」。变异表行 3828、3837、3846（`crates/mutations.tsv`，把 `record_count == …` 换成 `<=`）我用自己的 `run_row.py` 复跑，三行「必须红的测试红了」、每行 FAILED 共 1 条（日志 `row-3828.log`、`row-3837.log`、`row-3846.log`）。
- 多一条：旧判法还在，墓碑 `crates/singlefs-checker/src/walk.rs:12146`：「judgements.judge(invariant, record_count <= knob_records, || {」，谱系与退役 `crates/singlefs-checker/src/walk.rs:12048`：「judgements.judge(invariant, record_count <= records_the_knob_allows, || {」；这一句这一批没动。
- 最右之外有一片少一条且不是第一片（中间片）：真实代码抓得到——探针 [2, 1, 2] 在旋钮 2 下判红（上节输出，第 2 片是中间片，只装 1 条）。但**用例没钉**，见 K4 的 MB2 / MC 两种改坏。
- 三棵树：墓碑与「谱系 + 退役」是两份文本一致的循环（`walk.rs:12154`、`walk.rs:12055`），谱系与退役共用一个函数，逐形无差别。
- CPU 与 GPU 判得一样吗：读码一致。CPU：`crates/singlefs-checker/src/walk.rs:12158`：「record_count == knob_records;」，GPU：`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3357`：「if (d_container_record_count(container_index) != knob_records) {」，两边都是「最右之外 == 旋钮值」且与「≤」那一句分开判；零根、`len < 1` 两边都不判（CPU `walk.rs:11995` 的 `None => 2,` / `Some(_) => 1`，GPU 同函数开头 `d_container_count == 0u` 返回）。GPU 的新对拍三条在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 末尾，**属 checker 档，按派发要求我没跑**：「GPU 与 CPU 同红」我复核不了，只读码。
- 推翻条件：(a) 用 `walk.rs` 里任一棵、任一形造出一份镜像（容器表 (号, 条数) 沿叶序，第 1 片号 1、逐片加一、都不超旋钮值、除最右外某片 < 旋钮值）交新入口而它绿；(b) GPU 对同一状态与 CPU 的红位不同（checker 档对拍里 `mismatches` 非空）。

## 二、K2　产品支三判与首片 0（甲 H2、H3）

### K2-1 单测与整镜像是不是同一段代码：判定「一致」

- 单测直接调的就是整镜像路径调的那两个函数：整镜像 `crates/singlefs-checker/src/walk.rs:11403`、`crates/singlefs-checker/src/walk.rs:11650`（谱系、退役）调 `judge_lineage_or_retirement_container_capacity_of_a_version(`（定义 `walk.rs:11984`），`crates/singlefs-checker/src/walk.rs:11496` 调 `judge_tombstone_container_capacity_of_a_version(`（定义 `walk.rs:12079`）；单测 `walk.rs:14042`、`walk.rs:14064` 调同样两个函数，参数里的 (容器号, 记录数) 表与 `Option<旋钮>` 与整镜像路径同型。
- 整镜像路径「走没走到产品支」不靠单测钉，靠非慢档的整镜像用例：我造了一个变异——三个调用点在没装旋钮时不调判法（脚本 `mutate_k2_callsites_skip_product.py`，只改我拷贝的 `walk.rs`）。
  三处同时跳过（脚本跳了墓碑、谱系、退役三个调用点），`-p singlefs-harness --test checker_known_bad_images -- product` 过滤到的用例红了谱系与退役各两条（过滤词 `product` 没选到墓碑的用例）：
```
test two_rollbacks_one_record_per_lineage_container_judged_as_a_product_image_reddens_only_the_full_container_invariant ... FAILED
test the_old_tombstone_knob_entry_judges_the_lineage_tree_by_its_product_capacity ... FAILED
test two_leaves_retired_one_record_per_retirement_container_judged_as_a_product_image_reddens_only_the_full_container_invariant ... FAILED
test the_old_tombstone_knob_entry_judges_the_retirement_tree_by_its_product_capacity ... FAILED
test result: FAILED. 0 passed; 4 failed; 2 ignored; 0 measured; 251 filtered out; finished in 0.63s
```
  墓碑那一处单独跳过，在 `container_trees_grow_a_level_when_the_root_is_full` 目标里红 `a_capped_tombstone_tree_judged_as_a_product_image_reddens_only_the_full_container_invariant`（`mut-K2-callsite-skip-tombstone-product.log`），`checker_known_bad_images` 255 条与 `-p singlefs-checker --lib` 32 条在那个变异下仍绿。
  所以「单测绿、整镜像那条路没判」这个洞没有：三个调用点都有非慢档整镜像用例看着。慢档里「产品容量下满片 + 1」那两条（`checker_known_bad_images` 的 2 条 `#[ignore]`，名 `nine_hundred_sixty_retirements_at_product_capacity…`、`one_thousand_three_hundred_sixty_rollbacks_at_product_capacity…`）不在这条结论里。
- 推翻条件：存在一种调用点改坏（传错树、传错旋钮、产品支吞掉返回），使三个目标全绿而整镜像上某句不判。我只试了「产品支整个不调」。

### K2-2 产品支三判每一句 + 首片 0 的改坏能不能拦：判定「一致」（CPU 单测）；GPU 谱系 / 退役「规则没说」

- 新用例 `crates/singlefs-checker/src/walk.rs:14207` 起的 `assert_two_containers_of_one_record_judged_with_the_knob_two_violate_only_the_full_sentence` 与同一模块里的产品支四形（第一片号 2、3 / 号 1、3 / 最右 0 条 / 号 0、1），断言写成「只红一句」：`crates/singlefs-checker/src/walk.rs:14100`：「judgements.violation_count(invariant),」后跟 `1`，再核违例说明里的片段。因此改坏一句，既让该句不红、又不让别的句红，用例才红。
- 我把变异表第 57 批 CPU 的 24 行逐行用自己的脚本复跑（锚点在冻结副本里命中恰好一次、点名的测试红）：行 3821–3828、3830–3837、3839–3846（3829、3838、3847、3849 是 GPU 行，3848 是慢档写侧行，这几行没跑）：每行输出一行 `必须红的测试红了；FAILED 共 1 条`，24 行全红；3850、3851（H4）另算，也红。例（原样）：
```
ROW 3821 rc=101 必须红的测试红了；FAILED 共 1 条；第57批 H2 I-9.21：产品支不判第一片容器号是 1（墓碑树那一形，正推 c8 / 攻方 c-product-sk
ROW 3825 rc=101 必须红的测试红了；FAILED 共 1 条；第57批 H3 I-9.21：第一片容器号判成 ≤ 1（墓碑树产品支那一形，正推 c3）
ROW 3843 rc=101 必须红的测试红了；FAILED 共 1 条；第57批 H3 I-9.36：第一片容器号判成 ≤ 1（退役树产品支那一形，正推 c3）
```
  逐行日志 `research/prompts/m3-batch57-code-r1-sonnet-model/row-<行号>.log`（24 行加 3850、3851，共 26 个）。
- 首片判 ≤ 1：产品支靠 (0, 满), (1, 1) 一形（`product_*_containers_numbered_zero_and_one…` 三条），旋钮支靠 `checker_known_bad_images.rs:16897` 起的整镜像三条；变异表行 3825 / 3834 / 3843（产品）与 3826 / 3835 / 3844（旋钮）全红。
- GPU 一半：GPU 产品支只补了墓碑那一个调用点（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3374`：「d_judge_container_numbers_from_one_by_one_and_no_empty_container(INVARIANT_I_9_21);」，用例 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13794` 起）。谱系调用点在 `wgsl:3213`、退役调用点在 `wgsl:4127`，没有会红的对拍。**这一半我只读码，没跑（checker 档 + GPU）。**
- 规则没说一条：正文把谱系与退役一并归到 `D_MAX_LINEAGE_RECORDS` 256 条的上限，这条理由只对谱系成立（`wgsl:3217`：「if (d_lineage_record_count >= D_MAX_LINEAGE_RECORDS) {」）。退役树这一路上，谱系的 256 条上限不适用；调用点 `wgsl:4127` 在读退役记录、叶身份之前就判了容量。我 `grep -n 'const D_MAX_' crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 看到的别的表上限有 `D_MAX_CONTAINERS_OF_A_CONTAINER_TREE` 256 片（退役产品容量 959 条一片，两片就够判）、`D_MAX_LEAF_IDENTITIES` 256（读不全只把 I-9.34 / I-9.35 记成射程里有一部分没判，容量那句已经判过）。CPU 侧同形的 960 次退役用例是慢档，`crates/singlefs-harness/tests/checker_known_bad_images.rs:17047`：「#[ignore = "harness 慢档用例：产品容量下退役 960 次（约 2900 次发布）"]」；墓碑那条 GPU 用例（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:13729` 起的录流函数）靠「只展开最后一次 unlink 的段」绕过长流。所以「谱系 256 条」这个理由对退役调用点不成立；退役 GPU 产品支补不上若另有卡点（我没试着造这条对拍，复核不了），正文没写。这条在正文第二节「已知」里，**我不当打中报**，只记理由只对谱系成立。
- 推翻条件：能在冻结副本里造一份产品支三判某一句改坏、而第 57 批新用例与旧用例全绿的变异（我试了表里 24 行 + 下面 K4 的几种，没有；K4 里列的那几种是有的）。

## 三、K3　冷走读次序（乙 H4）

条款侧：kb 里没有这一条次序。`grep -rn 'E142 走读\|先判得了自己\|分配记录代与跨度\|分配记录逐条' .claude/kb/*.md .claude/kb/*/*.md` 零命中；I-1.10（`.claude/kb/invariants.md:35`）只管条目宽先于按字段表解条目。次序的出处只在判决 `m3-batch52-54-code-r1`（G1）与 `m3-batch56-code-r1`（H4）和 `recovery.rs` 的注释里。所以本节「代码做的是不是条款说的」判不了条款，只判「代码做的是不是 H4 说的」。

### K3-1 那张坏镜像是不是只由次序分得出：判定「一致」

- 次序（冻结副本）：冷走读分配记录逐条判在前，`crates/singlefs-core/src/recovery.rs:5395`：「allocation_record_generations_and_spans_are_judged(allocation_records, root)?;」，数记账条目在后，`crates/singlefs-core/src/recovery.rs:5402`：「accounting_entry_count_is_three_plus_six_per_device(」，数映射条目更后，`crates/singlefs-core/src/recovery.rs:5407`：「mapping_entry_count_is_one_per_mapped_unit(」。重建同序：`crates/singlefs-core/src/recovery.rs:2894`、`crates/singlefs-core/src/recovery.rs:2928`、`crates/singlefs-core/src/recovery.rs:2944`。可写挂载走重建：`crates/singlefs-core/src/mount.rs:1236`：「let rebuilt = rebuild_version_in_the_unit_area_starting_at(」。
- 用例不会在错的理由上绿：用例比的是整个 `RecoveryFailure` 值，其中 `detail` 是「分配记录跨度为 0，或分配代 / 释放代晚于根」，而这句字面在 `crates/` 的非测试码里只有一处发出：`crates/singlefs-core/src/recovery.rs:4852`：「detail: "分配记录跨度为 0，或分配代 / 释放代晚于根",」（我 `grep -rn` 其余命中是测试断言与注释）。别的判先拒了这张镜像只会换成别的 `detail`，用例当场红。
- 前提被钉死：`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1879`：「"前提：只坏记账那一处时，只读恢复在 txg 4 那条根上报记账条目数"」——只改台阶行标签时冷走读给的是条目数不对，所以改完分配记录之后用例要的结果是次序决定的。
- 我复跑变异表 3850（冷走读挪到两个数之后）、3851（重建挪到数记账条目之后）：各自只红自己那条（`ROW 3850 … FAILED 共 1 条`、`ROW 3851 … FAILED 共 1 条`），说明两条用例各钉一条路，互不顶替。

### K3-2 用没登记的标签 13 会不会钉住标签而不是次序：判定「一致」（有一条风险写明）

- 数条目只把「统计量是 inode 号水位且代早于根」的行排除，`crates/singlefs-core/src/recovery.rs:4795`：「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg」；换成 13 就不被排除，算进 3 + 6 × 盘数。用例里 13 的定义是 `crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs:1695`：「const STATISTIC_TAG_AFTER_THE_LAST_REGISTERED_ONE: u16 = STATISTIC_INODE_WATERMARK + 1;」。登记表到 12：`crates/singlefs-core/src/records.rs:29`：「pub const STATISTIC_INODE_WATERMARK: u16 = 12;」。
- 钉的是次序不是标签：用例断言的结果与标签无关（是分配记录那句），标签只负责造出「数条目会红」这个前提；前提另有断言（上一条 `:1879`）。
- 风险（规则没说）：13 不是随便一个号——D18 已定项 24 第 4 条写它是留给将来的，`.claude/kb/decisions/18-块里携带什么信息.md:579`：「4. **回收**：第一版不回收墓碑，统计量 13、14（D5（快照 / 空间记账机制） 已定项 4）第一版不登记；10⁶ 次 unlink 的墓碑约 76.7 MiB（两份）」。将来真登记 13（并让数条目认它）时，这条前提断言会先红（响，不是静默绿），需要把造数换一个号；用例这一点今天没写注释。
- 推翻条件：读者今天已判标签（未登记就拒）——则 `:1879` 的前提断言会红，我看到的是绿（基线 37 条全绿，见 `base-corrupt_on_disk_content_is_refused_instead_of_panicking.log`）。

### K3-3 重建那一条钉的是哪一个次序

钉的是「分配记录逐条判代与跨度」排在「数记账条目」之前（行 3851 挪到它之后红）。没钉的次序（我跑出来的，照绿）：
- 冷走读把「数映射条目」整句挪到「分配记录逐条判」之前（脚本 `mutate_k3_mapping_first.py`）：`-p singlefs-harness --test corrupt_on_disk_content_is_refused_instead_of_panicking` 输出 `test result: ok. 37 passed; 0 failed`，`-p singlefs-core --lib` 输出 `test result: ok. 248 passed; 0 failed`。原因：那张镜像里映射条目数是对的（两处改动都不动节点数），用例分不出两个数谁先。H4 判决写的是「排在两个数之前」，用例只钉了其中一个数。
- 冷走读把「分配记录逐条判」从条目宽、逐条判代与 seq 之前挪到数记账条目的紧前面（脚本 `mutate_k3_gen_before_count_only.py`，仍在两个数之前）：37 条全绿。这一格 H4 没要求，记在这里。
- 重建路径里分配记录逐条判在读记账树之前（`recovery.rs:2894` 在 `recovery.rs:2900` 之前），映射条目数在 `:2944`：「挪到读树之后、数记账条目之前」这一种读码推出仍在数之前、用例照绿（推的，没跑）。
推翻条件：另造一张记账条目数对、映射条目数不对且分配记录代晚于根的镜像，它在「映射先」变异下红。我没造。

## 四、K4　判别力

### K4-1 第 57 批新加的变异行能不能拦住改回去：判定「一致」（CPU 与 H4 行），GPU 与慢档行「复核不了」

- 31 行（`grep -c '^第57批' crates/mutations.tsv` 输出 31）的原文锚点我用脚本在冻结副本里逐行数命中次数，31 行每行都是 1 次（输出 `3821 1 walk.rs` … `3851 1 recovery.rs`）。
- 我跑了其中 26 行（3821–3828、3830–3837、3839–3846 共 24 行，加 H4 的 3850、3851），每行：把原文换成替换文、跑该行点名的 cargo test 参数、看「必须红的测试」是不是出现在 FAILED 里：26 行全红，每行 FAILED 恰 1 条。
- 没跑的 5 行：3829、3838、3847、3849 是 GPU 行（checker 档，不归我）；3848 写侧早关一片要 `--release --include-ignored` 的慢档（约两到四分钟一次，且是我这一腿没必要的重复；读码：它改的 `crates/singlefs-core/src/container_tree.rs:537` 是三棵树共用的那一个追加函数，慢档 `container_trees_grow_a_level_when_the_root_is_full.rs` 两条我在基线下跑过绿，行 3848 的红我没复核）。
- 推翻条件：任一行换成替换文后点名的测试不红。

### K4-2 还有哪几种改坏照绿（我造出来跑过的）

1. MB2：旋钮支只判最左那片、其余非最右片不判。改法：把 `crates/singlefs-checker/src/walk.rs:12152` 起那句 `is_the_rightmost_tombstone_container_under_the_knob` 的右边改成 `position_in_leaf_order != 0 || containers.len() == 1`（脚本 `run_mutant.sh`，参数见 `mut-MB2-harness.log` 同名）。结果：`-p singlefs-checker --lib` 32 条绿；`-p singlefs-harness` 的 `checker_known_bad_images`、`container_trees_grow_a_level_when_the_root_is_full`、`corrupt_on_disk_content_is_refused_instead_of_panicking` 三个目标原样输出：
```
test result: ok. 255 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 110.66s
test result: ok. 6 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 1.68s
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.86s
```
   （255 里含我加的 2 条探针，基线 253。）
2. MC：只判倒数第二片。把同一句右边改成 `position_in_leaf_order + 2 != containers.len()`：`-p singlefs-checker --lib` 32 条绿，三个目标 `255 passed / 6 passed / 37 passed` 全绿（`mut-MC-unit.log`、`mut-MC-harness.log`）。
3. 谱系 + 退役共用那个函数同样的改法（`crates/singlefs-checker/src/walk.rs:12053` 那句右边改成 `position_in_leaf_order != 0 || container_numbers_and_record_counts.len() == 1`）：lib 32 条、`checker_known_bad_images` 255 条、`container_trees_grow_a_level_when_the_root_is_full` 6 条全绿（`mut-MB2-lineage-retirement.log`）。
   原因：新单测的表都是两片 `&[(1, 1), (2, 1)]`（`crates/singlefs-checker/src/walk.rs:14210`：「assert_the_capacity_judge_holds(tree, &[(1, 1), (2, 1)], Some(1));」），整镜像用例也只造两片（`crates/singlefs-harness/tests/checker_known_bad_images.rs:16944`：「let pool = pool_with_records_in_one_container_tree_under_its_knob(tree, tag, 1, 2);」）；两片时「最右之外」只有第 0 片，三种写法等价。三片以上、只有中间片少一条的坏镜像没有用例（真实代码抓得到，探针 [2, 1, 2] 红，但没钉）。随机历史慢档只造合法镜像，抓不到「少判」。
   GPU 侧同构（读码，没跑）：`wgsl:3356` 的循环上界是 `container_index + 1u < d_container_count`，GPU 新对拍的流只有两片容器（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 末尾三条「两片各一条」），把上界改成 `+ 2u` 在读码上同样不红。
4. 上面 K3-3 的两种次序：冷走读「映射先」、冷走读「分配记录判紧贴数条目」：照绿。
5. 等价的（不算漏）：`record_count == knob` 改成 `>=` 或 `<=`——旁边 `<= 旋钮值` 那句（`walk.rs:12146`、`walk.rs:12048`）已经单判，合取之后 `==` 与 `>=` 等价，所以变异表 3827/3836/3845 把 `==` 换成 `<=` 才是有效的那一种；`checked_add(1)` 换成 `wrapping_add(1)`：要让两者在某一对上结果不同，左片号得是 `u64::MAX`，而要走到它前面那一对必须已经违例（第一片号不是 1、或相邻不加一），整条 invariant 的判定同样是红，只有违例条数不同，对判定无影响（推的，没造镜像）。
- 推翻条件：1、2、3 任一在你们的环境里（含 GPU 对拍）红了，则「照绿」不成立；我跑的只有 CPU 与 harness 档。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1-1 合法镜像误报 | 一致；另「规则没说」一条 | 旋钮不变的合法历史（含随机历史旋钮 1、2、根长高、回退）0 红；旋钮在一份镜像的历史中途改变（[2,1,2]）三个旋钮值都红，D18 没说能不能变 |
| K1-2 坏镜像漏报 | 一致（CPU）；GPU 复核不了 | 早关、多一条、中间片少一条真实代码都红；GPU 对拍属 checker 档没跑，读码与 CPU 同 |
| K2-1 单测与整镜像同一段 | 一致 | 同两个函数；三个调用点的产品支被跳过时非慢档整镜像用例红 |
| K2-2 产品支三判 + 首片 0 | 一致；GPU 谱系 / 退役「规则没说」 | CPU 24 行变异全红；GPU 只补墓碑调用点，「256 条」理由只对谱系成立，对退役未说全 |
| K3-1 坏镜像只由次序分得出 | 一致 | detail 句只有一处发出，前提断言钉死；3850、3851 各红一条 |
| K3-2 标签 13 | 一致，附风险 | 钉的是次序；13 是 D5 留给将来的号，登记那天前提断言先红 |
| K3-3 重建钉哪个次序 | 一致；范围窄于 H4 题面 | 只钉「分配记录判在数记账条目之前」，没钉在数映射条目之前 |
| K4-1 新变异行 | 一致（跑了 26 行全红）；5 行没跑 | 31 行锚点各命中 1 次；GPU 行与慢档写侧行复核不了 |
| K4-2 照绿的改坏 | 有 | 旋钮支只判最左片 / 只判倒数第二片（3 棵树）；冷走读映射先；分配记录判紧贴数条目 |

## 没做什么

- GPU 对拍（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 的新增 4 条）与变异表 GPU 行 3829、3838、3847、3849：checker 档，按派发不跑；GPU 的结论全是读码。
- 变异表行 3848（写侧早关一片）：要 `--release --include-ignored` 慢档，没复跑。
- 没读全 194 个测试二进制；「没有测试在历史中途改旋钮」只 grep 了 `set_deletion_switches`、`CappedForTests` 的出处。
- 没跑任何门禁阶段与 checker 档测试；没有重型测试；没动冻结副本（改动都在我拷贝 `~/.local/share/singlefs/work/m3-batch57-code-r1-sonnet/repo` 里，每次变异后用冻结副本还原并 `cmp` 过）。
- 不替主 agent 判采纳；上面「建议」只是读码后的补用例方向，没有落成代码。
- 我自己的编译目录 `~/.local/share/singlefs/work/m3-batch57-code-r1-sonnet/target` 与仓拷贝 `~/.local/share/singlefs/work/m3-batch57-code-r1-sonnet/repo` 交回前已删（拷贝只是冻结副本的复制，编译目录可重建）；复跑时照 `research/prompts/m3-batch57-code-r1-sonnet-model/README.md` 重新拷冻结副本、另起编译目录，探针追加进拷贝的 `checker_known_bad_images.rs` 末尾。
