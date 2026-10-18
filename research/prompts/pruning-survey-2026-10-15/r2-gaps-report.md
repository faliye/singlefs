# 第二轮复查（路子 gaps）：全仓反查底稿漏了哪些键与标记

日期 2026-10-15。被读的树：主工作区，HEAD c1e9b324；`git status --short -- crates scripts .claude/gate.d .claude/hooks .claude/scripts litmus` 只列出 doc-text 一族、write-guard.sh 与 scripts/kb 下几份改动，都不碰复用键。
只读：没改仓里任何文件，没起门禁、测试二进制与 cargo，没动判定库；探针（python）经 `scripts/hosts/run-with-memory-cap.sh 8G` 起，跑在草稿目录与草稿目录里的小副本上（副本已删，见末节）。
每条结论标「量的」（指到行号或命令原样输出）或「推的」。底稿 = `~/.local/share/singlefs/work/survey-pruning-ledger/ledger-r1.md`（L1–L29）。

## 一、结论

1. 新开 3 行（表一，N1–N3）：E178 第四段项目录（键 = 项名带序号 + 装置整份源码 sha256 + 几何，库在工作根草稿目录）、E174 父进程续跑（按产物里已有的格与轮号补跑，键里没有代码与工具链）、`rerun-failed-tests.py` 只重跑上一份日志里红的（绿的按测试名沿用，键里没有代码）。量的（行号见表一）。
2. 已罩住的机制底稿没列全的落点与调用方 11 处（表二上半）：L3/L5 的两份给定状态调用方测试、L17 的标记读者 `apply-writer-patch.py` 与被整份收进键的四份多机脚本、L20 的内核树取法 `fetch-deps.sh`、L23 的 54 份实验变异表、L4 的 7 份 Cargo.toml 等。量的。
3. 命中文件 6714 份逐份有去向，全表 `hits-disposition.tsv`（6715 行含表头，sha256 f604c0d2…）：A 已罩住 199、N 新开 3、B 射程边或射程外 33、C 无按键跳过（含文字、登记表、判轻重）686、D 冻结证据与产物 4984、E 样本 809。其中 C 里 490 份是「推的」（两道筛没筛出、没逐行通读），见第二节。
4. 第 6 问：`admission.py manifest` 在当前树里没有调用方（全仓含 `.claude/skills/`、`research/`、`records/` 搜，量的）；只剩它自己自证里的两格（`admission.py:4347`、`:4892`）与一份对照实现（`:6423`），另有 `lib_source_digest.py:8` 一句说明与 `records/2026-09-16-subagent拆分提案.md:950` 一句历史。删它要连那两格自证一起改成走 `crash-case-manifest`（两格测的是 `manifest_of_files` 与构建环境指纹，crash-case-manifest 用同一个函数）。
5. 第 7 问：钉住旧键形态的共 78 行（文件 × 形态），375 处（量的，`q7-detail.tsv`）：自证段 19 行、单独的自证脚本 4 行、`selftest-registry.tsv` 1 行、门禁样本 54 行；另有 34 份重型阶段样本的 expect 钉了复用汇总句（第七节）。
6. 校验：换一条不共用词表的路子（全仓搜「关掉复用」的开关 `--no-reuse|--full|--start-over|--force|--resume|SINGLEFS_GATE_FULL…`）搜出 84 份，逐份对去向表：凡有真复用的都已在 A/N/B 里；落在 C 的那几份命中的是装置把 `--force` 转给上游 preflight（准入一律 always，量的）与文字。

## 二、列全：命令与原样输出

命令（派发提示给的词表，十个目录）：

    grep -rlIE 'sha256|digest|fingerprint|manifest|marker|版本|version|key_of|reuse|复用|沿用|carried|skip|跳过|cache|缓存|resume|续跑|green|全绿' crates scripts .claude/gate.d .claude/hooks .claude/scripts .claude/singlefs-ai-sop/scripts research litmus --exclude-dir=target --exclude-dir=.git | sort -u | wc -l
    6714

（`scripts/devices`、`scripts/experiments/*/` 在 `scripts` 之下，已含。）按目录：research/prompts 4528、.claude/gate.d 768、research/results 456、scripts/experiments 302、crates/singlefs-harness 223、.claude/singlefs-ai-sop 116、crates/singlefs-checker-tier 91、scripts/mutation 60、其余各几十份以下。代码类（.rs/.py/.sh/.wgsl/.awk）1838 份，去掉三方轮快照与腿的模型副本、门禁样本后 821 份。

怎么判「有没有一处按键决定跳不跳过」（每份都落进 `hits-disposition.tsv` 的一行，第三列写依据）：

- 第一轮已逐份判过的照抄：crates 路附录 `destinations.md` 的 292 份（它的 C 号按底稿「路子编号」列映到 L 号）；底稿第一节「落点」列点名的文件名映到它的 L 号。
- 821 份代码各过两道筛，筛出来的逐份读命中行（本轮逐份判的 147 份写进 `manual-dispositions.tsv`，sha256 aca4656c…）：
  ① 读写持久位置（`git-common-dir|\.git/singlefs|local/share/singlefs|XDG_DATA_HOME|XDG_CACHE|\.cache/|GATE_CROSS_RUN_TMPDIR|refs/sop/`）的 78 份；
  ② 续跑与读回（`续跑|resume|read_back|复用数|reused|no-reuse|load_product|pending_jobs`）在实验装置与 bin 里的命中。
- 两道都没筛出、第一轮也没判过的代码归「C 无按键跳过（推的）」，共 490 份：scripts/experiments 274（e170 五个目录 58 份与 e171 其余 48 份第一轮 scripts 路已判「不是复用键」）、crates/singlefs-harness 38、.claude/singlefs-ai-sop 29、scripts/kb 20、其余各十几份以下。
- 三方轮快照与腿的模型副本（`research/prompts/*snapshot*/`、`*model*/`）3395 份、其余 research/prompts 1133 份、research/results 456 份：冻结证据与产物，不运行、不改（`evidence-discipline.md`「原样保存的证据不许事后改」）。读产物的代码另判，例 E142 的 `research/results/e142-r2*-crates-sha256-*.txt` 全仓没有代码读（`grep -rnIE 'crates-sha256-' scripts .claude/gate.d` 无输出，量的）。
- 门禁样本 729 份、上游样本 80 份：归 E，钉旧形态的另列第七节。

去向汇总（探针 `probe/disposition.py` 原样输出）：

    199 A 已罩住
    22 B 射程外
    4 B 射程边
    7 B 射程边·上游
    2 C 判轻重
    14 C 文字
    668 C 无按键跳过
    2 C 登记表
    456 D 产物与留存
    3395 D 冻结副本
    1133 D 原样证据
    80 E 上游样本
    729 E 门禁样本
    3 N 新开
    total 6714

A 行按底稿号数（一份文件可属几号）：L1 12、L2 11、L3 56、L4 9、L5 6、L6 42、L7 8、L8 2、L9 6、L10 5、L11 4、L12 7、L13 1、L14 3、L15 7、L16 1、L17 6、L18 5、L19 2、L20 4、L21 4、L22 3、L23 55、L24 11、L25 6、L26 6、L27 17、L29 1。L28 没有单列的文件（随各机制的文件头）。

## 三、表一：新开的行（底稿第一节 29 条都没罩住）

| 号 | 位置 | 管哪种判定 | 键的组成（逐项） | 违反 | 重做方向 |
|---|---|---|---|---|---|
| N1 | `scripts/experiments/e178_leaf_and_record_death_counts.rs:6191`（`const DEVICE_SOURCE: &str = include_str!("e178_leaf_and_record_death_counts.rs")`）、`:6202`（`segment_four_item_name`：`format!("{position:04}-{geometry}")`）、`:6208`（`segment_four_item_key`：`key item={name} device_source_sha256={…} geometry={…}`）、`:6213`（`read_back_the_item`：首行键相同、末行 `done lines=` 对得上就读回）、`:6243`（`compute_or_reuse_the_items`）、`:6848`（库目录 `E178_SEGMENT_FOUR_ITEMS`，默认 `~/.local/share/singlefs/work/e178-segment-four/items`）；文件头 `:15-18`；设计登记在 `research/prompts/e178-preregistration.md:888` | E178 第四段 997 个工作项各自的产物行（同一条命令重起，键相同的项读回不重算；`--no-reuse` 只写不读） | 项名（工作项在清单里的序号四位 + 几何标签）、装置整份源码（含注释）的 sha256、几何标签；没有工具链 | ②③（整份源码含注释：本轮在副本里只加一行注释，`device_source_sha256` 由 8d13f12d… 变 178226f3…，量的）⑤（序号进键：清单里插一项，后面的项全换键，读码推的）⑦（工具链与依赖版本不进键，读码量的）⑧（库在工作根的草稿目录里，不随仓、清草稿就丢；盘上现在没有这个目录，`ls` 量的） | 键 = `segment_four_item_lines` 起的函数单元闭包各自去注释的指纹多重集 + 这一项的参数值（几何的字段值，不用标签串）+ 工具链；序号与项名只当文件名索引；库挪到 git common-dir 下固定一处。装置在 `scripts/experiments/`，闭包算法与 L25 的复跑行键共用一份（底稿第二节第 8 条）。这一段续跑写在跑前登记里，改法要走登记修订（`research/prompts/e178-preregistration.md:888`） |
| N2 | `scripts/experiments/e174_lease_drop_wait_memory_return.rs:312`（`struct ChildJob`：格 + 轮号）、`:1495`（`type ProductRounds = BTreeMap<ChildJob, RoundData>`）、`:1503`（`parse_product`）、`:2792`（`pending_jobs`：产物里已有的（格，轮）不再跑，只补缺的轮与补轮）、`:2940`（`load_product` 读 `--output` 指的产物）、`:2987`（有 `name=done` 行且没有要补的就整趟不跑）、`:2995`（config 行只记 segment、seed、driver_version、skip_light、cells、rounds） | E174 两段（借据丢掉时的等待与显存回还）每格每轮的读数：父进程按产物续跑 | 格的标签（组、大小、存活、方法、臂）与轮号；产物文件就是库 | ⑦（装置代码、工具链、依赖版本一样都不进：改了子进程的测法，旧轮照算数；`grep -nE 'sha256|digest|DEVICE_SOURCE|include_str|rustc|toolchain'` 在这份文件里无输出，量的）⑤（格名进键）⑥（没有单独的库，产物就是标记） | 每轮读数的键 = 子进程 `child` 入口起的函数单元闭包指纹 + 这一格的参数值 + 工具链 + 驱动版本（它是测量对象，判的是显存行为，驱动进键）；键变了的旧轮不算进这一格；续跑仍按产物里的行，但先比行里记的键 |
| N3 | `scripts/gate/rerun-failed-tests.py`（文件头 `:4-8`：从一份 cargo test 日志里取出失败的用例、按测试目标各打一条只跑它们的命令）；规则 `.claude/rules/verification.md:25`「红了只重跑红的那几条」、`.claude/agents/implementation-writer.md:34`（1c 步）、`scripts/gate/merge-and-verify.sh:295` | 改完之后要重跑哪些测试 | 上一份日志里结局为 FAILED 的测试名；绿的按名字沿用 | ⑦（修红的那次改动可能碰到别的绿用例执行过的函数，键里没有代码，绿的照沿用；本轮在副本里喂一份一绿一红的日志，只打出红的那一条命令，量的，见第八节）⑤（只认测试名） | 判不了，交主 agent：它是「修完先快验」的流程工具，实现员 1d 步与合入后验证末尾照样整跑动到的测试二进制，绿的不会被永久沿用（读规则推的）。若要照单元口径改：上一趟绿的那几条，只有它们执行过的函数单元指纹都没变才不重跑（要每条测试的执行单元集合，与 L18 按测试函数入库同一套） |

照写派发提示要的材料：

- N1 要动的文件：`scripts/experiments/e178_leaf_and_record_death_counts.rs`（`:6190-6270`、`:6827-6860` 参数与库目录）；跑前登记修订一份；单测 `:7519` 那一格（`--no-reuse` 的解析）照留。必须作废的改动样本：`segment_four_item_lines` 调到的函数里改一个常量、换工具链、几何字段改一个值；必须不作废的：只改注释、只改与第四段无关的段（例 `segment-one` 的函数）、清单前插一项（后面各项内容不变）。
- N2 要动的文件：`scripts/experiments/e174_lease_drop_wait_memory_return.rs`（`:2792-2812`、`:2975-3010`、产物行格式与 `parse_product`）；E174 的四行复跑登记在 `scripts/experiments/replay.sh:918-921`（只登记了 judge、checks 那几种确定性读法，跑卡那两段不在复跑表里，量的）。必须作废：`child` 入口调到的计时函数改一行；必须不作废：只改 `judge` 子命令、只改注释。
- N3：若主 agent 定要改，要动 `scripts/gate/rerun-failed-tests.py` 与它读的 `scripts/gate/lib_libtest_result_lines.py`，规则句 `verification.md:25` 与 `implementation-writer.md:34` 随改。

## 四、表二：已罩住的（属于底稿哪一条）

### 4.1 底稿落点没列、本轮补上的调用方与使用方（改那一条时要一起改）

| 属于 | 文件:行 | 是什么 | 依据 |
|---|---|---|---|
| L3、L5 | `crates/singlefs-checker-tier/tests/crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library.rs:61`（`THIS_FILE`）、`:489`（表名 `random-history/{}/seed={}`）、`:745`（`setting_file: THIS_FILE`） | 给定状态表的调用方：设点文件路径、项名、带种子的表名进块键 | 量的；第一轮 crates 路附录 292 份里没有这份（`comm -23` 量的），C35 名单只列了 bin 与 `crash_injection.rs` |
| L3、L5 | `crates/singlefs-checker-tier/tests/record_checker_breaks_the_history_at_the_power_loss_seam.rs:43`（`THIS_FILE`）、`:51`（`item_name`）、`:52`（`setting_file`）、`:53`（`seeds_and_scale`） | 同上 | 量的；同上漏列 |
| L17 | `scripts/gate/apply-writer-patch.py` 文件头（「自上次崩溃验证全绿（git common-dir 里最新的 singlefs-crash-case-green.* 标记）以来打过的补丁到 5 个就打一行提示」） | 用例级全绿标记的读者（只提示、不跳过）；另写 `<git common-dir>/singlefs-applied-writer-patches.tsv` | 量的（文件头原文）；L17 删用例级标记时这里的「上次全绿」要换一个来源，否则提示永远从零数 |
| L17 | `scripts/crash/crash-amplification-two-hosts.sh`、`scripts/hosts/peer-host-lib.sh`、`scripts/crash/gpu-quiesce-run.sh`、`scripts/hosts/multi-host-run.sh` | 被 `admission.py` 的 `SHARD_DRIVER_FILES` 整份收进崩溃枚举用例的键 | 第一轮 scripts 路 K7；底稿 L17 只写「双机驱动脚本整份」没点文件名 |
| L20 | `.claude/scripts/fetch-deps.sh:16`、`:20-26`、`:28-29`（`kernel_tree_path`：`$SINGLEFS_KERNEL_TREE`、`~/linux`、`~/linux-bug-fix/linux`、缓存目录，谁先有 `tools/memory-model` 用谁；`fetch_kernel` 已有就跳过） | LKMM 判定读的 `.cat/.bell` 来自哪一棵树：不认版本、取哪一棵看本机有什么 | 量的；底稿 L20 写了「`tools/memory-model` 不进」，没写它是这样取的。重做时键里要的 `.cat/.bell` 内容摘要得从这里取到的那一棵算 |
| L23 | `scripts/mutation/tables/*.tsv`（54 份命中） | 实验变异表本身 | 量的（命中数）；L23 只写了 `mutate.sh` |
| L25 | `scripts/experiments/Cargo.toml`、`Cargo.lock`、`lib.rs` | 复跑行键「公共部分」整份进（`admission.py` 文件头 `:75`） | 量的 |
| L26 | `scripts/experiments/lib.rs:133`（`admit_experiment_run_or_exit`）、`scripts/experiments/e142_new_pool_file_creation_dry_run.rs`（全仓唯一调它的装置，`grep -rlE admit_experiment_run_or_exit` 量的）、`.claude/hooks/runner-dispatch-guard.sh:56`、`:860-875`（⑨）、`.claude/gate.d/doc-experiments.sh:422`（evidence-in-repo 经 `stored_product_status`） | 实验准入键的四个使用方 | 后两处第一轮已有（D86、G61）；前两处底稿没点名 |
| L21 | `crates/mutations.tsv` | 行键六段原样 | 量的 |
| L24 | `crates/singlefs-replay-prefix/verus-must-red-worlds.tsv`、`.claude/gate.d/verus-pins.tsv`、`e165-*` 三张账 | 形式化与钉表的登记行原文进键 | 量的 |
| L4 | `crates/*/Cargo.toml` 7 份 | 整份原文进版本 | 量的 |

L3 改块键时要一起改的给定状态调用方全集（`grep -rlE 'GivenStatesItem\b' crates` 23 份，量的）：`src/bin/` 下 e156、e158、e168、e171 两份；`src/crash_amplification.rs`、`src/crash_injection.rs`；`tests/` 下 common_given_states_of_harness_scenario_images/mod.rs、crash_amplification_pipeline_records_checks_and_reuses、crash_amplification_planned_import_names_the_gpu_conclusions、crash_enumeration_new_pool_file_creation_stream、crash_given_states_of_a_rollback_to_the_root_at_the_effective_floor、crash_given_states_of_random_histories_and_fault_injection_judged_through_the_library、crash_given_states_of_the_images_harness_scenarios_hand_out、crash_injection_campaign、crash_injection_read_faulted_states_never_hold_the_witness_criterion、crash_injection_record_checker_sees_every_publish_across_a_second_crash、crash_injection_writable_mount_after_the_crash、crash_points_of_the_rollback_publish、crash_points_tree_identifier_watermark_orphans、record_checker_breaks_the_history_at_the_power_loss_seam、record_checker_judges_absence_by_the_persisted_set、rollback_floor_written_into_the_system_configuration_first_and_normal_unmount。带 `setting_file|SETTING_FILE` 的共 49 份（量的），第一轮 C36 写「50 份」，差一份是数法不同（它的 grep 另带 `file!()`）。

### 4.2 按底稿号的归属（全表在 `hits-disposition.tsv`，这里只列底稿号与落在它下面的文件数）

L1 12、L2 11、L3 56、L4 9、L5 6、L6 42、L7 8、L8 2、L9 6、L10 5、L11 4、L12 7、L13 1、L14 3、L15 7、L16 1、L17 6、L18 5、L19 2、L20 4、L21 4、L22 3、L23 55、L24 11、L25 6、L26 6、L27 17、L29 1。底稿每一条都至少落了一份文件，没有「底稿有、仓里找不到」的条（量的，按这张表）。

## 五、表三：射程边与射程外（存了读回、但不是沿用测试或判定；交主 agent 定的标「判不了」）

| 文件:行 | 是什么 | 归哪 |
|---|---|---|
| `.claude/singlefs-ai-sop/scripts/preflight.py:335`（`input_fingerprint` 逐文件 sha256 连路径）、`:367`（`stamp_path` 记在 git common-dir）、`:379`（`read_stamp`） | 上游 `inputs-changed`：输入没变就不起 | 射程边·上游。项目里没有一份脚本声明它：`grep -rlI 'admission: *inputs-changed' scripts .claude/gate.d .claude/hooks .claude/scripts` 只命中上游两份（量的）；装置头一律 `// admission: always`（18 份经 preflight 判，量的）。底稿第二节第 5 条已划出 |
| `.claude/singlefs-ai-sop/scripts/selftest.sh:2` | 上游包自证，`inputs-changed` 收整个包目录 | 射程边·上游；项目不起它（`grep` 无调用方，量的） |
| `.claude/singlefs-ai-sop/scripts/gate.sh:119-123`、`:781`；`lib.sh:175` | `refs/sop/gate-ok` 定 diff 窗口 | 底稿第二节第 5 条 |
| `.claude/singlefs-ai-sop/scripts/claude-hook-lib.sh`（`stop_hook_already_shown`）、`claude-hooks/gate-reuse-check.sh` | 同一份红不连拦两次；名字里的 reuse 指「门禁查重」 | 射程外：判定照跑，只少拦一次 |
| `.claude/gate.d/doc-process-records.sh`；`.claude/crates-review-exempt`、`.claude/agent-def-review-exempt` | 人的「这次不走三方」豁免按「路径 + 整份 sha256」认 | 射程边，判不了：形状同 ②⑤，但它是人的豁免，不是测试判定；整份哈希让只改注释也要重新豁免（偏严，不漏判） |
| `scripts/gate/wallclock-assert-lint.py --list` | 登记了挂钟断言的测试，门禁单测格按名 `--skip` | 射程边：有意排除，不是沿用旧判定 |
| `scripts/gate/changed-paths.sh` | 只判新增行的改动范围 | 底稿第二节第 5 条 |
| `.claude/gate.d/crash-point-replay-segments.tsv` 第五列 | 段写表摘要钉身份 | 射程边（第一轮 gates 路已列） |
| `.claude/gate.d/crates-mutation-light-rows.tsv` | 轻型 crates 变异 | 射程外（轻型） |
| `crates/singlefs-checker-tier/src/gpu_unit_checks.rs:630-660` | 驱动着色器盘缓存 | 射程外（编译缓存，第一轮 C40） |
| `scripts/devices/build-patched-qemu.sh:16`、`:61`；`scripts/hosts/source-stamp.py`、`minimal-copy.py`、`compile-then-swap.py`、`target-prune.py`、`target-prune.conf`；`scripts/gate/merge-and-verify.sh`、`lib_rebuilt_packages.py` | 构建戳、编译目录复用、核真重编了 | 射程外（构建缓存，判定每次现跑） |
| `scripts/kb/verify-citations.sh`、`scripts/agents/three-way-materials.py` | 文献文本缓存、三方材料 `--resume` | 射程外（第一轮 scripts 路已列） |
| `scripts/hosts/host-dispatch.py`、`detached-run.sh`、`.claude/hooks/lib_long_run.py` | 长活豁免表按整份 sha256；显存峰值表 | 射程外 |
| `scripts/hosts/run-with-memory-cap.sh:103`、`memory_peaks_lib.py:70`、`host-resources.py` | 内存峰值表只管排先后 | 射程外 |
| `.claude/gate.d/harness-test-environment.sh:31`、`:136`；`scripts/hosts/test-environment-check.py` | 门禁耗时 current/previous、跨轮缓存目录白名单 | 射程外 |
| `scripts/agents/agent-watch.py:520`、`:2656-2670` | 看门狗报过的不再报 | 射程外（运维状态） |
| `scripts/devices/snapshot-check.sh` | 三方开工快照核对，对不上比 HEAD | 射程外（第一轮 scripts 路已列） |
| `.claude/hooks/heavy-test-guard.sh:139-140`、`lib_heavy_tests.py:1317` | 同一次判定里外层复用里层、`lru_cache` | 不落盘，不算 |

## 六、第 6 问：`admission.py manifest` 有没有调用方

答：当前树里没有（量的）。

命令与原样输出（三条不同写法，全仓连 `.claude/skills/`、`research/`、`records/`）：

    grep -rnIE 'admission(\.py)?["'"'"' }]*[ ,]+["'"'"']?manifest\b' --exclude-dir=target --exclude-dir=.git . | wc -l
    2
    （records/2026-09-16-subagent拆分提案.md:950 与 scripts/gate/admission.py:2996 的用法句）

    grep -rnIE '(admission|ADMISSION)[A-Za-z_]*\}?"?[ ]+manifest\b' …；grep -rnIE '"manifest"' …；grep -rnIE 'admission\.py[` ]+manifest|admission manifest|manifest 子命令' …（scripts .claude research records litmus crates）合起来去重 6 行：
    records/2026-09-16-subagent拆分提案.md:950、scripts/gate/admission.py:2996、:4347、:4892、:6423、:6446

按目录数「同一行里有 admission 又有 manifest、去掉 crash-case-manifest / manifest.sh / manifest_of / input_manifest / rules-manifest」：.claude/skills 0、research 102、records 1、scripts 3、.claude/gate.d 1、.claude/hooks 0、.claude/rules 0、.claude/agents 0、.claude/kb 0、上游 0、crates 0、litmus 0。research 的 102 行逐份看了前两份的上下文，都是 C613 那一行里的 `cargo build --manifest-path`；.claude/gate.d 那 1 行是广谱变异样本里的 `manifest.tsv` 字样；scripts 的 3 行是 `admission.py` 自己两行与 `lib_source_digest.py:8` 的说明句。

逐处：

| 位置 | 是什么 | 删 `manifest` 子命令时怎么办 |
|---|---|---|
| `scripts/gate/admission.py:2993-3014`（`command_manifest`）、`:6446`（`COMMANDS` 表）、`:6467`（用法句） | 子命令本体 | 删 |
| `scripts/gate/admission.py:4343-4350`（自证 ⑨「manifest 的清单写法：拿 sha256sum、sort -zu 与 cargo -V 另算一遍，逐字节比（crash-case-manifest 写清单用的是同一个 manifest_of_files）」） | 自证拿它当载体测 `manifest_of_files` | 改成走 `crash-case-manifest`，或直接调函数；它测的那个函数还在用 |
| `scripts/gate/admission.py:4889-4894`（构建环境那一组自证的 `fingerprint()`） | 自证拿它测构建环境进指纹 | 同上 |
| `scripts/gate/admission.py:6423`（`INDEPENDENT_MANIFEST_SCRIPT`，「manifest 子命令（不带 --build-environment）同一套步骤的另一份实现，只给自证当对照」） | 自证的独立对照 | 随 ⑨ 改 |
| `scripts/gate/lib_source_digest.py:8`（「实验的输入指纹（admission.py experiment / manifest）不走这里」） | 说明句 | 删掉「/ manifest」 |
| `records/2026-09-16-subagent拆分提案.md:950` | 历史：2026-09-26 门禁 54 号输入清单改用 `admission.py manifest` | 记录，说的是那一次，不改 |

历史上的调用方：`git log --oneline -G'admission[^ ]*"? manifest ' -- .claude/gate.d scripts`（只查了这两个目录）只命中一次搬目录的提交 503b80e2，它的 diff 里 `manifest` 一词的调用全是 `crash-case-manifest`（量的）；门禁 54 号当年那一处什么时候换走的，本轮没再往前追（没做）。

## 七、第 7 问：自证、样本、钩子用例与登记表里钉住旧键形态的逐处清单

判法（探针 `probe/q7d.py`，原样输出存 `q7-detail.tsv`，78 行、375 处，sha256 c0545c0c…）：六种形态各一条正则——全绿标记文件名（`singlefs-stage-green|singlefs-crash-case-green|singlefs-gate-environment\.|singlefs-layer0-full-green|STAGE_MARKER_PREFIX|CRASH_CASE_MARKER_PREFIX|GATE_ENVIRONMENT_RECORD_PREFIX|singlefs-checker-tier-qemu-device-streams\.|stage-marker-(check|write)|crash-case-marker-(check|path)|全绿标记`）、`--changed-since`（`changed[-_]since`）、`--sync-tree`（`sync[-_]tree`）、整道键（`gate-reuse\b|stage-fingerprint|stage-must-run|GATE_REUSE|change-touches-crates|gate-record-environment`）、judge-version（`judge[-_]version|JUDGE_RULES_VERSION|judging[-_]digest|JUDGE_VERSION`）、`refs/sop/staged-green`（`refs/sop/staged-green|staged-green|SINGLEFS_STAGED_TREE`）。
范围：`.claude/hooks`、`.claude/gate.d`（含 fixtures）、`scripts`、`.claude/scripts`、`crates`、上游 `scripts/`。「自证段」= Python 里名字带 selftest、以 `_cells` 结尾或参数里有 `selftest` 的顶层函数与类，shell 里名字带 selftest 的函数到下一个行首 `}`；文件名带 selftest 的整份算；样本与登记表整份算。自证段的边界是按这条规则切的，不是逐个读出来的（推的边界）；`scripts/agents/cite-check.py` 命中的是它自己的 `--unchanged-since`，与这里无关，剔掉。

### 7.1 自证段、自证脚本与登记表（「处数」= 落在自证段里的 / 全文）

| 文件 | 形态 | 所在 | 处数 | 自证段里的行号（全列） |
|---|---|---|---|---|
| `.claude/hooks/heavy-test-guard.sh` | --changed-since | 自证函数 | 2/3 | 1170,1173 |
| `.claude/hooks/heavy-test-guard.sh` | --sync-tree | 自证函数 | 1/3 | 1168 |
| `.claude/hooks/lib_heavy_tests.py` | judge-version | 自证函数 | 1/2 | 3505 |
| `.claude/hooks/lib_heavy_tests.py` | --sync-tree | 自证函数 | 2/8 | 3403,3404 |
| `.claude/hooks/lib_heavy_tests.py` | 整道键 | 自证函数 | 1/1 | 3641 |
| `.claude/hooks/selftest-registry.tsv` | 整道键 | 自证登记表 | 2/2 | 38,114 |
| `.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh` | 整道键 | 自证函数 | 5/15 | 124,125,162,163,166 |
| `.claude/singlefs-ai-sop/scripts/selftest.sh` | 整道键 | 自证脚本 | 9/9 | 708,710,711,712,721,725,747,919,920 |
| `scripts/crash/crash-point-replay.sh` | 全绿标记文件名 | 自证函数 | 6/17 | 866,872,877,1060,1066,1286 |
| `scripts/experiments/e165-pin-red-examples.py` | judge-version | 自证函数 | 4/15 | 2041,2045,2046,2250 |
| `scripts/gate/admission.py` | judge-version | 自证函数 | 19/38 | 4379,4416,4417,5885,5935,5940,5947,5956,5957,5958,5959,5962,5963,5964,5966,5969,5970,5971,6097 |
| `scripts/gate/admission.py` | refs/sop/staged-green | 自证函数 | 11/22 | 3990,3996,4000,4004,4007,4010,4359,4535,4546,4548,4591 |
| `scripts/gate/admission.py` | 全绿标记文件名 | 自证函数 | 61/140 | 3568,3569,3574,3579,3580,3590,3591,3596,3599,3600,3604,3606,3610,3611,3619,3632,3633,3641,3661,3668,3676,3678,3682,3694,3696,3702,3704,3707,3719,3722,3735,3737,3888,3911,3990,3997,4362,4375,4388,4532,4554,4555,4558,4560,4563,4564,4565,4567,5070,5356,5380,5381,5386,5389,5400,5401,5403,5405,6101,6264,6268 |
| `scripts/gate/admission.py` | 整道键 | 自证函数 | 35/75 | 3579,3586,3587,3592,3593,3609,3611,3617,3623,3629,3639,3640,3651,3672,3681,3683,3685,3687,3688,3697,3713,3820,3979,4357,4358,4359,4360,4422,4513,4537,4540,5144,5146,5153,6042 |
| `scripts/gate/gate-staged-selftest.sh` | refs/sop/staged-green | 自证脚本 | 14/14 | 4,10,25,27,44,48,65,96,98,103,109,113,120,123 |
| `scripts/gate/stage-must-run-selftest.sh` | refs/sop/staged-green | 自证脚本 | 17/17 | 41,46,47,51,55,63,67,70,72,75,79,85,87,99,101,104,108 |
| `scripts/gate/stage-must-run-selftest.sh` | 整道键 | 自证脚本 | 6/6 | 4,11,100,101,111,115 |
| `scripts/gate/verus-proof-check.py` | judge-version | 自证函数 | 3/11 | 1707,1785,1786 |
| `scripts/mutation/broad-mutation.py` | --changed-since | 自证函数 | 15/51 | 4417,4444,4460,4467,4469,4505,4506,4513,4519,4521,4523,5007,5019,5031,5083 |
| `scripts/mutation/broad-mutation.py` | judge-version | 自证函数 | 3/16 | 4138,5029,5033 |
| `scripts/mutation/broad-mutation.py` | --sync-tree | 自证函数 | 73/122 | 3800,3805,3811,3812,3848,3916,3976,3991,4009,4033,4044,4056,4073,4098,4099,4107,4113,4121,4123,4125,4126,4130,4141,4147,4157,4158,4169,4183,4190,4228,4253,4280,4324,4357,4375,4426,4447,4458,4487,4494,4508,4530,4554,4556,4564,4571,4576,4577,4601,4625,4641,4645,4647,4663,4671,4674,4680,4688,4720,4784,4788,4809,4849,4879,4942,4958,4970,5000,5005,5007,5011,5016,5034 |
| `scripts/mutation/crates-mutation-rows.py` | judge-version | 自证函数 | 4/19 | 3757,3797,3809,3956 |
| `scripts/mutation/crates-mutation-rows.py` | 全绿标记文件名 | 自证函数 | 1/8 | 3779 |
| `scripts/mutation/mutation-shard-run.sh` | judge-version | 自证函数 | 2/7 | 652,657 |

另有三份 checker 档测试把 `judge_version=` 打进汇总行（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:618`、`crash_amplification_fixed_script_through_rollback.rs:369`、`crash_amplification_fixed_script_through_unmount.rs:459`），崩溃枚举用例登记的 `count-line=` 把这一行原样记进全绿标记（`admission.py` 文件头 `:40`）：L7 删手写号、L17 删用例级标记时这三行的字段跟着换。不在自证段里，单列（量的）。

### 7.2 门禁样本里写进文件内容的旧形态（54 行；按阶段与形态并，列样本目录名）

| 阶段 | 形态 | 样本目录 |
|---|---|---|
| `checker-tier-broad-mutation.sh` | --changed-since | gate-verdict-no-conclusion-red |
| `checker-tier-broad-mutation.sh` | judge-version | library-version-is-staged-tree-export-range-red、library-version-is-staged-tree-file-differs-red、library-version-is-staged-tree-global-key-file-red、library-version-is-staged-tree-global-key-missing-red、library-version-is-staged-tree-global-key-red、library-version-is-staged-tree-green、library-version-is-staged-tree-marker-clean-written、library-version-is-staged-tree-marker-unstaged-skipped、library-version-is-staged-tree-merge-conflict-red、library-version-is-staged-tree-new-file-red、library-version-is-staged-tree-pending-red、library-version-is-staged-tree-source-fingerprint-red、library-version-is-staged-tree-toolchain-red、library-version-is-staged-tree-unstaged-change-green、library-version-is-staged-tree-version-key-red |
| `checker-tier-broad-mutation.sh` | --sync-tree | gate-verdict-swapped-between-cells-red、library-version-is-staged-tree-pending-red |
| `checker-tier-broad-mutation.sh` | 全绿标记文件名 | library-version-is-staged-tree-export-range-red、library-version-is-staged-tree-file-differs-red、library-version-is-staged-tree-global-key-file-red、library-version-is-staged-tree-global-key-missing-red、library-version-is-staged-tree-global-key-red、library-version-is-staged-tree-green、library-version-is-staged-tree-marker-check-clean-reused、library-version-is-staged-tree-marker-check-clean-unmarked、library-version-is-staged-tree-marker-check-staged-differs-red、library-version-is-staged-tree-marker-clean-written、library-version-is-staged-tree-marker-unstaged-skipped、library-version-is-staged-tree-merge-conflict-red、library-version-is-staged-tree-new-file-red、library-version-is-staged-tree-pending-red、library-version-is-staged-tree-source-fingerprint-red、library-version-is-staged-tree-toolchain-red、library-version-is-staged-tree-unstaged-change-green、library-version-is-staged-tree-version-key-red |
| `checker-tier-crash-point-replay.sh` | 全绿标记文件名 | green、quick-features-gpu-configured-green、quick-features-gpu-configured-judged-on-cpu、quick-features-gpu-not-run、quick-features-green、quick-features-interrupted、quick-features-no-quiesce-config、quick-features-quiesce-red、quick-features-red、quick-features-zero、quick-gpu-configured-plain-pass-judged-on-cpu、quick-gpu-configured-plain-pass-not-run、red |

`checker-tier-crash-point-replay.sh` 那 13 份是样本根里的 `.claude/gate.d/stage-inputs.tsv` 副本与 `setup.sh` 里写的崩溃枚举用例登记行（`crash-case:<名>	crates/ Cargo.toml	test=…	# 样本：快档只核它的全绿标记`）与「全绿标记与日志放 git common-dir」的现场说明：L14 改登记表语义、L17 删用例级标记时整批换。

### 7.3 expect 里钉了复用汇总句的重型阶段样本（34 份；`grep -lE '复用|沿用|快照|判法版本|指纹|全绿标记|staged' …/expect`）

| 阶段 | 份数 | 样本目录 |
|---|---|---|
| `checker-tier-broad-mutation.sh` | 22 | gate-verdict-escalated-threads-rerun-green、library-version-is-staged-tree-global-key-red、gate-verdict-escalated-green、library-version-is-staged-tree-global-key-file-red、library-version-is-staged-tree-file-differs-red、library-version-is-staged-tree-toolchain-red、gate-verdict-green、library-version-is-staged-tree-export-range-red、library-version-is-staged-tree-merge-conflict-red、library-version-is-staged-tree-new-file-red、library-version-is-staged-tree-marker-check-staged-differs-red、library-version-is-staged-tree-pending-red、library-version-is-staged-tree-marker-check-clean-unmarked、library-version-is-staged-tree-global-key-missing-red、library-version-is-staged-tree-source-fingerprint-red、library-version-is-staged-tree-green、library-version-is-staged-tree-version-key-red、library-version-is-staged-tree-marker-unstaged-skipped、library-version-is-staged-tree-marker-clean-written、library-version-is-staged-tree-unstaged-change-green、library-version-is-staged-tree-marker-check-clean-reused、library-version-is-staged-tree-literal-global-key-green |
| `checker-tier-crates-mutation-replay.sh` | 3 | green、not-run、red |
| `checker-tier-formal-verification.sh` | 3 | pinned-red-examples-replay-reuse-twice、verus-toolchain-pinned-fingerprint-red、verus-toolchain-pinned-green |
| `checker-tier-research-build-and-replay.sh` | 4 | experiment-replay-reused、mutation-tables-compile-red、mutation-tables-compile-green、mutation-tables-compile-deny-green |
| `harness-model-differential-and-scenarios.sh` | 2 | model-differential-green、model-reuse-sampling-red |

`harness-model-differential-and-scenarios.sh` 那两份的「复用」是文件系统的块复用（取样点名），不是判定复用，不用改（读 expect 原文，量的）。`checker-tier-crates-mutation-replay.sh/green/expect` 钉了「复用：现判 5 条（一律现跑 0、行新加或改了 0、没有覆盖记录 5、被变异的函数变了 0、执行过的函数变了 0、函数外有改动 0）」与「那份快照上已有、沿用 0 组」两句：L21 改成按单元键、去整树快照之后这两句的分项要跟着换。

### 7.4 登记表与弄坏开关

- `.claude/hooks/selftest-registry.tsv:38`（`scripts/gate/change-touches-crates.sh	light`）、`:114`（`scripts/gate/stage-must-run.sh	light`）：L15 删 `change-touches-crates` 与树对比路时这两行一起删或改。登记了 light 而自证里钉旧形态的还有 `:24` admission.py、`:30` broad-mutation.py、`:31` mutation-prune.py、`:48` crash-point-replay.sh、`:58` crates-mutation-rows.py、`:79` e165-pin-red-examples.py、`:81` gate-staged.sh、`:97` mutation-shard-run.sh、`:106` replay.sh、`:126` lib_source_digest.py、`:128` verus-proof-check.py（行号 `grep -n` 量的）；改了这些脚本，按钩子规则自证要在集成副本里由主 agent 跑。
- 钩子判轻重的用例按旧参数名写：`heavy-test-guard.sh:1168`（`--sync-tree --commit …`）、`:1170`、`:1173`（`--changed-since HEAD`）；`lib_heavy_tests.py:3403-3404`（`--sync-tree`）、`:3505`（`crates-mutation-rows.py judge-version`）、`:3641`（`stage-must-run.sh --selftest`）；判法正文 `lib_heavy_tests.py:579`（`BROAD_MUTATION_FIRST_ROUND_QUERY_ARGUMENTS … {"--sync-tree"}`）、`:2151`。L22 删 `--changed-since` / `--sync-tree`、L21 去掉 `judge-version` 子命令时，这几格的命令要换成新参数，判法里认参数的集合跟着缩。

## 八、正推、反推、校验，推翻条件与造它的那一次

- 正推：每一行新开的都指到算键与读回的那几行代码（表一）；每一份 A 行都指到第一轮那一路的去向或底稿落点列（`hits-disposition.tsv` 第三列）。
- 反推（若「只有这 3 行是漏的」是错的，本该看到什么）：会有一份代码在持久位置存了东西、下一趟读回就不做某件事，而不在 A/N/B 里。本轮拿两条去找：
  ① 持久位置读写的 78 份逐份看过（第二节），落在 C 的都是只读配置位置（例 `doc-text.sh:353` 取 git common-dir 只为找私有配置文件）；
  ② 实验装置与 bin 里「既写文件又读文件」的 20 份（`File::create|fs::write|OpenOptions` 与 `read_to_string|fs::read(` 同时出现）逐份看过读的是什么：除 E174、E178 与已罩住的 E164、E162 实验本身、E170/E171 外，读的都是 `/sys/block/*/stat`、`/proc/self/io`、`/proc/mdstat` 或比对步读产物（E158 `run_third_run_compare`、`run_fourth_run_compare`），不跳过任何一格（量的）。
- 校验（不共用词表的另一条路子）：搜「关掉复用」的开关，命令
  `grep -rlIE -- '--no-reuse|--start-over|--fresh\b|--rejudge|--ignore-library|--no-library|--full\b|SINGLEFS_GATE_FULL|--force\b|--milestone-full|--resume\b|--no-resume' crates scripts .claude/gate.d .claude/hooks .claude/scripts .claude/singlefs-ai-sop/scripts litmus --include=*.rs --include=*.py --include=*.sh | grep -v /fixtures/ | wc -l` → `84`。
  逐份对去向表（`check-flags-dispositions.tsv`：A 40、N 2、B 9、C 30、不在命中表 3）：有复用的全在 A（40 份，含 crates 侧 12 份崩溃枚举与给定状态测试）、N（N1、N2）、B（上游 preflight 一族、长活、看门狗）；落在 C 的 30 份里，实验装置 15 份命中的是把 `--force` 转给上游 `preflight.py check`（例 `e162_crash_verdict_block_store.rs:410-414`），而装置头一律 `// admission: always`（`grep -rhoE '^// admission: [a-z-]+'` → `18 // admission: always`，量的），所以那个 `--force` 不放过任何复用；其余是钩子与门禁里的文字或判轻重的参数表。另有 3 份不在命中表（`.claude/scripts/preflight.sh`、`project_preflight.py`、`scripts/crash/crash-point-replay-shard-configuration-check.sh`），读过：前两份只转上游 preflight，第三份是 K7 收进 L17 键的配置判法（归 L17）。
  这条路子自己会不会红：它确实搜出了 N1（`--no-reuse`）与 N2（`--force` 不在它里头，靠 `resume` 一词搜出），也搜出了底稿里有库的每一道重型阶段，说明它分得出「有复用」的文件（量的，按上表）；N3 它搜不出（`rerun-failed-tests.py` 没有关复用的开关），这一行只靠正推那一路，单路，交主 agent 判要不要算。

推翻条件与造一次：

| 结论 | 什么现象会推翻它 | 本轮在副本里造的那一次（原样输出） |
|---|---|---|
| N1 的键随注释变（②） | 只加一行注释后 `device_source_sha256` 不变 | 探针 `probe/e178_key.py` 在副本里给 `/// SHA-256（FIPS 180-4）` 那一行前插一行 `// probe: comment only`：`device_source_sha256 before=8d13f12d2c94a0b8 after_comment_only=178226f30fde8afe equal=False`；`key_format_has_item_name_and_whole_source_sha256=True item_name_has_position=True` |
| N3 绿的按名沿用（⑦） | 喂一份一绿一红的日志，它也打出绿的那一条 | 副本里的 `rerun-failed-tests.py` 喂 `probe/rerun/test.log`（`green_before_the_fix ... ok`、`red_before_the_fix ... FAILED`）：`✓ 1 条失败的用例，分在 1 个测试目标里` / `cargo test -p sample --test sample_target -- --include-ignored --exact red_before_the_fix`，退 0（只打了红的；没起 cargo） |
| N2 键里没有代码 | 产物或 `ChildJob` 里有装置源码或二进制的摘要 | 没造（读码量的：`grep -nE 'sha256|digest|DEVICE_SOURCE|include_str|rustc|toolchain'` 在整份文件里无输出） |
| 第 6 问「没有调用方」 | 有一处用变量拼出子命令名（例 `"$sub" …` 而 `sub=manifest`） | 没造；按 `"manifest"` 字面量另搜过一遍，只命中 `admission.py` 自己（第六节） |
| C 里 490 份「推的」都没有按键跳过 | 其中一份在仓内路径或 `/tmp` 下存结论、下一趟读回跳过 | 没造；两道筛与校验路子都没筛出，这一半是推的 |

## 九、排除掉的解释与凭的观测

- 「E178 那个库是底稿 L25（复跑行键）管的」：排除。E178 在复跑表里只登记了 `segment-one`（`replay.sh:925`），第四段项目录是装置自己的，复跑键不读它（量的）。
- 「E174 的续跑是 L25 的一部分」：排除。复跑表四行 E174 都是 `@driver_e174_*` 的 judge 与 checks（`replay.sh:918-921`），跑卡两段不在表里（量的）。
- 「`doc-experiments.sh` 与派发闸 ⑨ 是新机制」：排除，它们只用 L26 的同一个键，第一轮已记 G61、D86（量的，底稿全文 grep）。
- 「上游 preflight 的 inputs-changed 在项目里挡掉了某道重活」：排除，项目里没有一份声明它（第五节，量的）。
- 「crates 里还有别的落盘续跑」：排除到这一步——`crash_point_replay_progress.rs` 文件头写明进度文件与双机账本已没有（`:5-6`），第一轮判 C37 合规；给定状态与放量的调用方全集见 4.1（量的）。

## 十、没做什么

- 没跑任何门禁、测试二进制、cargo、轻型三条与自证；N3 的造法跑的是副本里那份 python 脚本，不起 cargo（它自己不起）。
- C 里 490 份（第二节）没逐行通读，只过了两道筛与校验路子，是推的。research/prompts 下 4528 份与 research/results 下 456 份按「冻结证据与产物」整批归类，没逐份读（只核了 E142 的整树 sha256 清单没有代码读）。
- 门禁 54 号当年用 `admission.py manifest` 那一处是哪次提交换走的，没追。
- 第七节的「自证段」边界按命名规则切，没逐个函数核；落在自证段之外的命中（例 `admission.py` 判法正文里的标记前缀）不在 7.1 的处数里，全文命中数照列在「处数」的分母。
- 第三节其余几问（Rust 调用图、Verus 按函数拆、QEMU 与 LKMM、libtest 逐条、盘上的库、E164 阳性对照）不归这一路，没答。
- 表一 N1、N2 的重做方向与 N3 要不要改，涉及跑前登记修订与流程规则，没定，交主 agent。

## 十一、草稿与副本

- 草稿目录 `~/.local/share/singlefs/work/survey-pruning-r2-gaps/`：报告、`hits-disposition.tsv`（6714 份逐份去向，sha256 f604c0d23c8f850ea7ff7c3b24af07cbc04cbccd39f3566a14499d3f18204f93）、`manual-dispositions.tsv`（本轮逐份判的 147 份，sha256 aca4656cab4303005b480af8a9ecf88bbb905f070e8eaa0dc0612b2c547d832f）、`q7-detail.tsv`（第七节全行号，sha256 c0545c0c42c3aad19296f9006080aeb8ef34ac25cc58a312212e8c5a3612bae2）、`check-flags-dispositions.tsv`（校验路子 84 份与各自去向）、探针 `probe/`。
- 仓副本 `copy/`（scripts/gate、E178 装置一份、.claude/scripts 与上游 scripts，不含 target 与 .git）：删前 `du -sh` 为 `3.0M`，已删（`ls` 报 No such file or directory）。没建编译目录，没写源码摘要缓存。
