## E175 产品路径上 inode 号水位退回与重发号 —— 已跑（够判）

<!-- doc-lint:not-numbers W1 W2 W3 P1 P2 Q1 Q2 Q3 Q3b Q4 Q5 Q6 Q7 Q9 Q10 Q11 S1 S2 S3 V1 V2 V3 V4 V5 V717 A1 A2 A3 A4 A5 A6 A7 F1 F2 F3 F4 F5 F6 F7 F8 F9 F3a F3b F4 D1 D2 D3 G0 -->

跑前登记 `research/prompts/e175-preregistration.md`（照它跑两段时 sha256 `d68c9d2229bcf3800c5efd36c9194e6af9684b193a172258b58846474afae805`，问法定成读法 B：「重发号」只算同一条时间线上的；第十二节「修订」有两次：修订 1 是执行员写的 8 条读法钉死与装置写法，修订 2 是设计员写的「两段都在一份冻结副本上跑」与几何四点，判据、门槛、臂、量的去留都没动）。岔路单 `research/prompts/m3-3-5-forks.md` 第 11 行 W：C479（inode 号复用已发过的号今天造不出来） 剩下的「活记录被原地换」那一形，翻面观测是「产品路径上（不装只供测试的水位开关），有没有一段历史能让 inode 号水位退回去、造出重发号」，够判条件是在随机历史与带回退的历史上跑够登记的历史数、数到重发号与 I-9.29（同号只许一个出生代） 红的次数，有一次就够判。

**这一页的数全部出自仓副本装置，跑在一份冻结副本上**：装置照登记第五节「装置写在哪」写死，放在冻结副本 `~/.local/share/singlefs/work/e175-frozen-crates-2/repo/` 的 `crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs`，主工作区 `crates/` 不动（形式化线冻结期）；装置源码原样留在产物目录的 `device-source.rs`。冻结副本 `code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`，拷于 2026-10-08，根环线第 10z 批第十、十一块合入、合后验证全绿之后；HEAD `31e47115471ba5502a4531d25dffcc921be36899`（`baseline-frozen-crates.out` 第一行）。kb 别处引这些数要写明「仓副本装置、冻结副本」；装置要不要进 `crates/`、要不要在入库装置上重做一次，由主 agent 定。

**现状**：两段都在这一份冻结副本上跑完：第一段（D 族钉值、F3a 定向穷举、F2 带回退的随机历史）与第二段（F1 四组现有随机历史、第八节几何四点 G-long、G-small、G-draw、G-long-F1），P、V1、V2 照登记各跑各的规模。第三段（F3b、F4）是附带，W 行够判之后没跑（够判后未跑）。

- **旧代码上的数**：产物目录下两个子目录都是旧代码上的产物，不进 W 行判决、不进任何量的合计。`superseded-b000407a/` 是上一份冻结副本（`code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`，根环线第 10z 批第九块合入之后，清单是其中的 `baseline-frozen-crates.sha`）上两段的 37 份，照登记修订 2 第 4 条那一形原样挪进去（挪之前、挪之后 37 份的 sha256 逐行相同）；`superseded-old-crates/` 是更早那份 `crates/`（清单 `superseded-old-crates/old-crates.sha`，336 行）上第一段的 19 份。两份冻结副本的清单相差 10 个文件，增删 0 个，其中有 `crates/singlefs-core/src/mount.rs`（改在 `placements_in_the_accounts_of_newer_readable_roots`、`rebuilt_allocator` 两个函数与模块内单测，`singlefs-core` 编进这个装置）。新的二十六份结果文件的 E7RESULT 行（去掉 `elapsed_ns`、排序后）与 `superseded-b000407a/` 里同名的逐份相同，两份汇总除 `code_baseline` 外逐行相同（「读数」一节的比对命令）。

- **产品臂 P**：登记的 17 105 段历史（D 8、F3a 16 105、F2 256、F1 四组各 128、几何四点 64 + 64 + 64 + 32）里，同一条时间线上的重发号（Q1）0 次，I-9.29（同号只许一个出生代） 红 0 段（判不了 0 步；F1 四组与 G-long-F1 有不适用的步，F3a、F2、G-long、G-small、G-draw 每一步都判成立），水位退回（Q3）0 步，跨被抛弃时间线的重发号（Q3b）0 次；任何一条不变量在任何一步都没判红。
- **路径跑到了**：跨过已发号的回退之后同一条时间线上又建了 inode 的窗口（Q5）F3a 7 870 个、F2 1 062 个，合计 8 932，过了登记的 50；第二段另有 F1 四组 572 个、几何四点 1 196 个。F1 四组抽到 `PublishFirstFile`（1 号重发唯一的路）：F1-broad 197 次、F1-reuse 68 次、F1-deleting 12 次、F1-newest 197 次，G-long-F1 203 次。
- **阳性对照**：V1（回退取目标那一版的水位）与 V2（发布不加新建个数）在跑到的每一族都造出重发号、I-9.29（同号只许一个出生代） 红，R-V1 在 F3a、F2、F1 四组、G-long 上全过，R-V2 在 F3a、F2、F1 四组上全过（几何点照登记只跑 G-long 的 V1），登记第十节 F1、F2 没触发。
- **停机条款**：S3 两条路判抛弃不一致 0 次、S1 与 S2 对不上（F3 两形）0 次、Q11（发的号不等于上一版水位）0 次、条款锚点 A1–A3 与独立锚点 A4–A7 全对上，都没触发。
- **W 行**：照登记第十一节「第二段跑完、F1 F2 F7 F8 都没触发，W 行够判」：登记的 17 105 段产品臂历史里没造出同一条时间线上的重发号，也没有一段 I-9.29（同号只许一个出生代） 红。这是「登记的历史里没造出」，不是「造不出」。

### 做了什么

- **三样读法**（登记第六节）：S1 账：每一步之后用 `singlefs_core::recovery::readable_roots` 列可读根，同一根键只用 `rebuild_version` 重建一次，读那一版记账里的水位与 inode 树全部 (号, 出生代)，按号记「见过的出生代 → 持有过它的根」；S2：每一步对镜像调 `singlefs_checker::walk::check_pool_image`，判红不停，I-9.29（同号只许一个出生代）、I-9.6（水位大于两处最大号） 按成立 / 不适用 / 部分没判 / 违例四栏数；S3：被崩溃恢复抛弃的根两条路，路 1 用最新那条根的实例表按 `publish_is_abandoned_by_the_instance_table` 判，路 2 是步账（每次挂载做成，新实例接的是那一步之前模型根环里最新的那条根，同一旧实例里 txg 更大的根记被抛弃）。
- **臂**：P（冻结副本 `crates/` 原样，与拷它那一刻的主工作区逐字节相同：`baseline-frozen-crates.out` 的 `main_before_copy_equal=true main_after_copy_equal=true`）、V1（`crates/singlefs-core/src/mount.rs` 第 7917 行回退那次的水位换成目标那一版的）、V2（`crates/singlefs-core/src/transaction.rs` 第 8035 行 `} + new_inodes_of_this_publish;` 换成 `} + 0;`）；V717（`crates/mutations.tsv` 第 717 行，名字「实三（D23 已定项 14「水位」）：回退那次的 inode 号水位只取环里读得出的根，不与内存里现行那一版的取 max」）登记写明 Q6 > 0 才跑，两段各族 Q6 都是 0，记「与 P 同输出（由 Q6 = 0 推出），未跑」。三处锚点的行号与命中数在 `baseline-frozen-crates.out` 的 `name=mutation_anchor` 三行；补丁原样在 `patch-V1.diff`、`patch-V2.diff`。每臂施加前与还原后在副本根对 `baseline-frozen-crates.sha` 核一次（`name=arm_check` 九行，`listed_mismatch=0 extra_files=0`）；三臂二进制各编一次，两段共用（`name=arm_binary` 三行）。
- **执行**：`HistoryExecution { per_step_checker: Skipped, device_width: FourGibibytes（G-small 换 UnitAreaOf384Slots）, space_admission: JudgedByTheFormula }`，不调 `set_inode_number_watermark_source`、不加 `SharedFaultPlan`；5 个工作线程（派发提示的线程上限）按序号递增领活，合并按序号次序，与单线程逐字相同（快档 `one_thread_and_three_threads_give_the_same_lines`）；每段历史的挂钟在装置进程里自己计（结果行 `elapsed_ns`），被打断再跑同一条命令就续跑，跳过已有结果行的序号。每份结果文件第一行是 `# code_baseline=…` 文件头，续写时文件头不同就拒，汇总要求全部输入同一个 `code_baseline`、每一行汇总行带它（登记修订 2 第 2 条）。
- **试跑**（登记修订 2 第 5 条）：F2 P 前 8 个种子 488 步，平均 281.86 毫秒 / 步 / 线程，外推第一段约 54 分钟；G-long P 前 4 个种子 604 步，272.74 毫秒 / 步，外推第二段约 45 分钟；都不到 2 小时，不分片（`trial-f2-P-first-8-seeds.out`、`trial-g-long-P-first-4-seeds.out`，按结果行 `elapsed_ns` 之和除以 `steps_observed` 之和）。

### 复跑

```
# 冻结副本（拷 Cargo.toml、Cargo.lock、.cargo、crates 四样；主工作区 crates/ 不动），核清单再放装置
F=~/.local/share/singlefs/work/e175-frozen-crates-2/repo; R=$PWD/research/results/e175-product-path-inode-watermark-regression-2026-10-08
(cd $F && sha256sum -c --quiet $R/baseline-frozen-crates.sha)
cp $R/device-source.rs $F/crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs
# 每一臂：施加补丁（P 不施加；V1、V2 照 $R/patch-V1.diff、$R/patch-V2.diff）、编、拷出二进制、还原，再核一次清单
ROOT=$PWD; (cd $F && CARGO_TARGET_DIR=<编译目录> bash $ROOT/scripts/hosts/capped.sh 5 cargo test --release -p singlefs-harness --test e175_product_path_inode_watermark_regression --no-run)
E175_ARM=P bash scripts/hosts/capped.sh 5 bash scripts/hosts/run-with-memory-cap.sh 8G <测试二进制> --test-threads 5
E175_ARM=P E175_FAMILY=F2 E175_RESULT_FILE=$R/f2-P.out E175_CODE_BASELINE=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb E175_WORKER_THREADS=5 E175_TIME_BUDGET_SECONDS=170 \
  bash scripts/hosts/capped.sh 5 bash scripts/hosts/run-with-memory-cap.sh 8G <测试二进制> --ignored --exact run_one_family_on_one_arm_into_the_result_file
# 族名：D、F3a、F2、F1-broad、F1-reuse-after-raising-the-floor、F1-deleting-files-of-several-lower-leaves、
#       F1-newest-system-configuration-slot-reading-zeros、G-long、G-small、G-draw、G-long-F1
# 汇总（两段各一份）
E175_SUMMARY_INPUTS=<逗号分隔的结果文件> E175_SUMMARY_FILE=<新文件> E175_CODE_BASELINE=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb <测试二进制> --ignored --exact summarize_the_result_files
# 登记的复跑（scripts/experiments/replay.sh 一行 E175|@driver_e175：拷今天的 crates/ 进副本、放进 device-source.rs、跑 D 族 P 臂）
bash scripts/experiments/replay.sh E175
```

### 产物清单

目录 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/` 顶层 37 份（`ls -1 | grep -v superseded | wc -l` 输出 37），全部是冻结副本 `code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`（拷于 2026-10-08，根环线第 10z 批第十、十一块合入、合后验证全绿之后）上的；另有子目录 `superseded-b000407a/` 37 份与 `superseded-old-crates/` 20 份（旧代码上的数，不进判决）：

| 文件 | 是什么 |
|---|---|
| `e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs` | 装置源码原样（sha256 `310c60a51a7a0c5cc015b070006d7d48ae0eb09d8a581ec8acba9d6bfc185fca`，两段同一份） |
| `e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.sha` | 冻结副本放装置之前的清单（340 行，路径相对副本根；sha256 `48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`，就是 `code_baseline`） |
| `e175-product-path-inode-watermark-regression-2026-10-08/baseline-frozen-crates.out` | `name=frozen_copy`（拷的日期、批次、HEAD、两行 diff / status 的 sha256、文件数、主工作区拷前拷后清单都与副本相同）、`name=mutation_anchor` 三行、`name=arm_check` 九行、`name=arm_binary` 三行、`name=main_drift_after_copy` 一行 |
| `e175-product-path-inode-watermark-regression-2026-10-08/patch-V1.diff`、`e175-product-path-inode-watermark-regression-2026-10-08/patch-V2.diff` | V1、V2 两臂施加的补丁 |
| `e175-product-path-inode-watermark-regression-2026-10-08/quick-tests.out` | 三臂快档与三个弄坏开关的输出 |
| `e175-product-path-inode-watermark-regression-2026-10-08/trial-f2-P-first-8-seeds.out`、`e175-product-path-inode-watermark-regression-2026-10-08/trial-g-long-P-first-4-seeds.out` | 外推挂钟的两次试跑 |
| `e175-product-path-inode-watermark-regression-2026-10-08/d-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/d-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/d-V2.out` | D 族钉值，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` | `driver_e175` 形态的输出（冻结副本上 D 族 P 臂 4 线程再跑一次，E7RESULT 行加收尾行），`replay.sh` 比它；去掉 `elapsed_ns` 与 `d-P.out` 的 E7RESULT 行逐行相同 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f3a-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f3a-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f3a-V2.out` | F3a，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f2-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f2-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f2-V2.out` | F2，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-one.out` | 第一段汇总：九份族 × 臂结果文件的合计、重复与冲突、事件计数 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-broad-V2.out` | F1 `BROAD`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-reuse-after-raising-the-floor-V2.out` | F1 `REUSE_AFTER_RAISING_THE_FLOOR`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-deleting-files-of-several-lower-leaves-V2.out` | F1 `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-V1.out`、`e175-product-path-inode-watermark-regression-2026-10-08/f1-newest-system-configuration-slot-reading-zeros-V2.out` | F1 `NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS`，三臂 |
| `e175-product-path-inode-watermark-regression-2026-10-08/g-long-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-long-V1.out` | 几何点 G-long（F2 比重，150 步），P 与 V1 |
| `e175-product-path-inode-watermark-regression-2026-10-08/g-small-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-draw-P.out`、`e175-product-path-inode-watermark-regression-2026-10-08/g-long-f1-P.out` | 几何点 G-small（`UnitAreaOf384Slots`）、G-draw（`RecentUniformOrBeyond`）、G-long-F1（`BROAD`，150 步），P |
| `e175-product-path-inode-watermark-regression-2026-10-08/summary-segment-two.out` | 第二段汇总：十七份族 × 臂结果文件 |
| `e175-product-path-inode-watermark-regression-2026-10-08/superseded-b000407a/` | 上一份冻结副本（`code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`，根环线第 10z 批第九块合入之后）上两段的 37 份，原名原样，与顶层 37 份一一同名；那份副本的清单是其中的 `superseded-b000407a/baseline-frozen-crates.sha`；挪之前、挪之后 37 份的 sha256 逐行相同 |
| `e175-product-path-inode-watermark-regression-2026-10-08/superseded-old-crates/` | 旧代码上的第一段 19 份（原名原样，与顶层同名的 16 份之外是 `superseded-old-crates/baseline.out`、`superseded-old-crates/baseline-segment-two.out`、`superseded-old-crates/trial-f3a-P-8-long-sequences.out`）与那份 `crates/` 的清单 `superseded-old-crates/old-crates.sha`；挪之前、挪之后 19 份的 sha256 逐行相同 |

### 读数（命令与原样输出）

在产物目录里跑，从两份汇总取每个族 × 臂的量（`sum_` 是各段相加，`segments_q2_one_birth_violated` 是 I-9.29（同号只许一个出生代） 红过的段数，`sum_i9_29_holds`、`sum_i9_29_not_applicable`、`sum_i9_29_partially_judged` 是 I-9.29 判成立、不适用、判不了的步数）：

```
for s in one two; do echo "== $s"; head -1 summary-segment-$s.out; awk '/name=summary/{o=""; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]~/^(family|arm|histories|complete|duplicates|conflicts|ending_completed|sum_q1|sum_q3b|segments_q2_one_birth_violated|sum_i9_29_holds|sum_i9_29_not_applicable|sum_i9_29_partially_judged|sum_q9_still|sum_q9_gone|sum_q3_steps|max_q3_peak|sum_q4|sum_q5|sum_q6|sum_q7|q10_segments_issuing_at_least_two|sum_q11|sum_s1_unreadable|sum_s3_disagreements|sum_f3_still_but_holds|sum_f3_violated_without_two_births)$/) o=o" "$i}; print substr(o,2)}' summary-segment-$s.out; done
```

```
== one
# code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb
family=D arm=P histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=45 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=4 sum_q5=3 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=D arm=V1 histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=42 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=3 sum_q11=0 segments_q2_one_birth_violated=3 sum_q3_steps=7 sum_q3b=0 sum_q4=4 sum_q5=3 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=3 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=2
family=D arm=V2 histories=8 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=7 ending_completed=8 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=23 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=13 sum_q11=0 segments_q2_one_birth_violated=7 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=13 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F2 arm=P histories=256 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=231 ending_completed=256 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=15616 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=1748 sum_q5=1062 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F2 arm=V1 histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1769 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=336 sum_q11=0 segments_q2_one_birth_violated=53 sum_q3_steps=1476 sum_q3b=0 sum_q4=311 sum_q5=195 sum_q6=0 sum_q7=280 sum_q9_gone=5 sum_q9_still=331 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=7
family=F2 arm=V2 histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1265 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=696 sum_q11=0 segments_q2_one_birth_violated=53 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=696 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F3a arm=P histories=16105 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=16105 ending_completed=16105 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=159440 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=11641 sum_q5=7870 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F3a arm=V1 histories=1464 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=1464 ending_completed=1464 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=11788 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=975 sum_q11=0 segments_q2_one_birth_violated=567 sum_q3_steps=2243 sum_q3b=0 sum_q4=798 sum_q5=577 sum_q6=0 sum_q7=168 sum_q9_gone=0 sum_q9_still=975 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=4
family=F3a arm=V2 histories=1464 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=1464 ending_completed=1464 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=4392 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=3990 sum_q11=0 segments_q2_one_birth_violated=1464 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=3990 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
== two
# code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb
family=F1-broad arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=49 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3638 sum_i9_29_not_applicable=330 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=39 sum_q5=13 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-broad arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=821 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=12 sum_q11=0 segments_q2_one_birth_violated=4 sum_q3_steps=43 sum_q3b=0 sum_q4=11 sum_q5=4 sum_q6=0 sum_q7=9 sum_q9_gone=0 sum_q9_still=12 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=5
family=F1-broad arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=637 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=90 sum_q11=0 segments_q2_one_birth_violated=15 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=90 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-deleting-files-of-several-lower-leaves arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=110 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3910 sum_i9_29_not_applicable=58 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=615 sum_q5=526 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-deleting-files-of-several-lower-leaves arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=27 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=556 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=103 sum_q11=0 segments_q2_one_birth_violated=26 sum_q3_steps=327 sum_q3b=0 sum_q4=97 sum_q5=86 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=103 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=12
family=F1-deleting-files-of-several-lower-leaves arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=27 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=433 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=244 sum_q11=0 segments_q2_one_birth_violated=27 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=244 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-newest-system-configuration-slot-reading-zeros arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=57 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3638 sum_i9_29_not_applicable=330 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=52 sum_q5=18 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-newest-system-configuration-slot-reading-zeros arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=17 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=806 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=16 sum_q11=0 segments_q2_one_birth_violated=6 sum_q3_steps=52 sum_q3b=0 sum_q4=14 sum_q5=6 sum_q6=0 sum_q7=14 sum_q9_gone=0 sum_q9_still=16 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=5
family=F1-newest-system-configuration-slot-reading-zeros arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=17 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=613 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=110 sum_q11=0 segments_q2_one_birth_violated=17 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=110 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-reuse-after-raising-the-floor arm=P histories=128 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=67 ending_completed=128 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3917 sum_i9_29_not_applicable=51 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=32 sum_q5=15 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=F1-reuse-after-raising-the-floor arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=896 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=7 sum_q11=0 segments_q2_one_birth_violated=6 sum_q3_steps=18 sum_q3b=0 sum_q4=7 sum_q5=6 sum_q6=0 sum_q7=3 sum_q9_gone=0 sum_q9_still=7 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=7
family=F1-reuse-after-raising-the-floor arm=V2 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=15 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=806 sum_i9_29_not_applicable=10 sum_i9_29_partially_judged=0 sum_q1=82 sum_q11=0 segments_q2_one_birth_violated=15 sum_q3_steps=0 sum_q3b=0 sum_q4=0 sum_q5=0 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=82 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-draw arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=56 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3904 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=334 sum_q5=191 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-long arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=9664 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=1045 sum_q5=661 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-long arm=V1 histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=29 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=1411 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=476 sum_q11=0 segments_q2_one_birth_violated=29 sum_q3_steps=1943 sum_q3b=0 sum_q4=420 sum_q5=275 sum_q6=0 sum_q7=345 sum_q9_gone=6 sum_q9_still=470 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=9
family=G-long-F1 arm=P histories=32 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=25 ending_completed=32 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=4708 sum_i9_29_not_applicable=124 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=188 sum_q5=109 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
family=G-small arm=P histories=64 complete=yes duplicates=0 conflicts=0 q10_segments_issuing_at_least_two=53 ending_completed=64 sum_f3_still_but_holds=0 sum_f3_violated_without_two_births=0 sum_i9_29_holds=3904 sum_i9_29_not_applicable=0 sum_i9_29_partially_judged=0 sum_q1=0 sum_q11=0 segments_q2_one_birth_violated=0 sum_q3_steps=0 sum_q3b=0 sum_q4=397 sum_q5=235 sum_q6=0 sum_q7=0 sum_q9_gone=0 sum_q9_still=0 sum_s1_unreadable=0 sum_s3_disagreements=0 max_q3_peak=0
```

与上一份冻结副本（`superseded-b000407a/`）的逐族比对（新旧二十六份结果文件的 E7RESULT 行去掉 `elapsed_ns`、排序后取 sha256 前 16 位；两份汇总去掉 `code_baseline` 后比）：

```
for f in $(ls -1 d-P.out d-V1.out d-V2.out f1-*.out f2-*.out f3a-*.out g-*.out | sed 's/\.out$//'); do a=$(grep '^E7RESULT' superseded-b000407a/$f.out | sed 's/ elapsed_ns=[0-9]*//' | sort | sha256sum | cut -c1-16); b=$(grep '^E7RESULT' $f.out | sed 's/ elapsed_ns=[0-9]*//' | sort | sha256sum | cut -c1-16); echo "$f old=$a new=$b $([ $a = $b ] && echo same || echo DIFFERENT)"; done | awk '{print $NF}' | sort | uniq -c
for s in one two; do diff <(grep -v '^#' superseded-b000407a/summary-segment-$s.out | sed 's/ code_baseline=[0-9a-f]*//') <(grep -v '^#' summary-segment-$s.out | sed 's/ code_baseline=[0-9a-f]*//') && echo summary-$s-same-apart-from-code-baseline; done
```

输出 `     26 same`（例 `f3a-P old=40278af11b04d80e new=40278af11b04d80e same`）、`summary-one-same-apart-from-code-baseline`、`summary-two-same-apart-from-code-baseline`：P、V1、V2 每一族每一臂的每个量都与上一份冻结副本上相同，没有一族变了。上一份冻结副本上第一段九份与更早那份 `crates/`（`superseded-old-crates/`）同样逐份相同（例 `f3a-P` 三处都是 `40278af11b04d80e`）。

产物齐不齐（逐份结果文件数结果行、带 `line_end=ok` 的、带臂字段的、事件行、`first_violation` 不是 `none` 的段、完成标记，每份一条 `awk '/^E7RESULT name=history /{n++; if($0 ~ / line_end=ok$/)ok++; if($0 ~ / arm=/)arm++; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="first_violation" && a[2]!="none") fv++}} /^E7RESULT name=event /{ev++} /name=family_complete/{c++} NR==1{head=$0} END{print head, n+0, ok+0, arm+0, ev+0, fv+0, c+0}' <文件>`）：二十六份结果文件第一行都是 `# code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`，各自的结果行数等于登记的段数（D 8、F3a P 16 105、F3a V 1 464、F2 P 256、F2 V 64、F1 P 每组 128、F1 V 每组 32、G-long P 64、G-long V1 32、G-small P 64、G-draw P 64、G-long-F1 P 32），每一行都带 `arm=` 与 `line_end=ok`，都有 `name=family_complete`；P 臂十二份的事件行 0 条、`first_violation` 不是 `none` 的段 0 段。两份汇总的 `unfinished_lines count=0`。F3a P 的操作步：`awk '/^E7RESULT name=history /{n++; for(i=1;i<=NF;i++){split($i,a,"="); if(a[1]=="steps_observed")s+=a[2]}} END{print "f3a-P histories=" n " operation_steps=" s-n}' f3a-P.out` 输出 `f3a-P histories=16105 operation_steps=143335`。

`PublishFirstFile`（1 号重发唯一的路）抽到几次：`for f in f1-broad-P f1-reuse-after-raising-the-floor-P f1-deleting-files-of-several-lower-leaves-P f1-newest-system-configuration-slot-reading-zeros-P g-long-f1-P f2-P; do echo "$f first=$(grep -h '^E7RESULT name=history' $f.out | grep -o '[[,]first(' | wc -l)"; done` 输出 `f1-broad-P first=197`、`f1-reuse-after-raising-the-floor-P first=68`、`f1-deleting-files-of-several-lower-leaves-P first=12`、`f1-newest-system-configuration-slot-reading-zeros-P first=197`、`g-long-f1-P first=203`、`f2-P first=0`（F2 这一组比重从 `AfterFirstFile` 起、unlink 之后那张表不抽它）。

### 判据逐条（登记第六节，每个量各报各的）

F1 四组按 `BROAD` / `REUSE_AFTER_RAISING_THE_FLOOR` / `DELETING_FILES_OF_SEVERAL_LOWER_LEAVES` / `NEWEST_SYSTEM_CONFIGURATION_SLOT_OF_EACH_DEVICE_READING_ZEROS` 的次序列，几何四点按 G-long / G-small / G-draw / G-long-F1 的次序列；数都出自「读数」一节两份汇总。

| 量 | F3a | F2 | F1 四组 | 几何四点 | 判定 |
|---|---|---|---|---|---|
| Q1 同一时间线重发号（P） | 0 | 0 | 0 / 0 / 0 / 0 | 0 / 0 / 0 / 0 | 没有 |
| Q2 I-9.29（同号只许一个出生代） 红（P，段数） | 0（成立 159 440 步，不适用、判不了 0 步） | 0（成立 15 616 步） | 0（成立 3 638 / 3 917 / 3 910 / 3 638 步，不适用 330 / 51 / 58 / 330 步，判不了 0 步） | 0（成立 9 664 / 3 904 / 3 904 / 4 708 步，G-long-F1 不适用 124 步，判不了 0 步） | 没有 |
| Q9（P 有 Q1 事件才有） | 无事件 | 无事件 | 无事件 | 无事件 | 不适用 |
| Q3 水位退回（P，步数；峰值与期末） | 0，峰值 0、期末 0 | 0，峰值 0、期末 0 | 0，峰值 0、期末 0 | 0，峰值 0、期末 0 | 没有 |
| Q3b 跨被抛弃时间线重发号（P） | 0 | 0 | 0 | 0 | 没有（附带） |
| Q4 回退跨过发号（P） | 11 641 | 1 748 | 39 / 32 / 615 / 52 | 1 045 / 397 / 334 / 188 | 报数 |
| Q5 跨过发号的回退之后又建 inode 的窗口（P） | 7 870 | 1 062 | 13 / 15 / 526 / 18 | 661 / 235 / 191 / 109 | F2 + F3a 合计 8 932 ≥ 50，够 |
| Q6 回退时环里可读根的水位最大值 < 现行（P） | 0（判了 17 954 次） | 0（判了 2 951 次） | 0（判了 185 / 74 / 733 / 224 次） | 0（判了 1 984 / 714 / 612 / 407 次） | V717 不跑 |
| Q7 挂载 / 恢复落到水位更低的根（P） | 0 | 0 | 0 | 0 | 报数 |
| Q10 一条时间线发过 ≥ 2 个号的段（各臂） | P 16 105、V1 1 464、V2 1 464 | P 231、V1 53、V2 53 | P 49 / 67 / 110 / 57，V1、V2 各 15 / 15 / 27 / 17 | P 53 / 53 / 56 / 25，G-long V1 29 | V2 跑到的每一族都必须红 |
| Q11 发号与上一版水位不等（各臂） | 0 | 0 | 0 | 0 | 一致 |
| R-V1（V1：Q5 ≥ 1 时 Q1 ≥ 1 且 Q2 ≥ 1 段） | Q5 577、Q1 975、Q2 567 段 | Q5 195、Q1 336、Q2 53 段 | Q5 4 / 6 / 86 / 6，Q1 12 / 7 / 103 / 16，Q2 4 / 6 / 26 / 6 段 | G-long：Q5 275、Q1 476、Q2 29 段 | 跑到的每一族都过 |
| R-V2（V2：Q10 ≥ 1 时 Q1 ≥ 1） | Q1 3 990 | Q1 696 | Q1 90 / 82 / 244 / 110 | 登记不跑 | 跑到的每一族都过 |
| R-V717 | 未跑 | 未跑 | 未跑 | 未跑 | 与 P 同输出（由 Q6 = 0 推出），未跑 |

几何敏感性（登记第八节）：G-long、G-small、G-draw、G-long-F1 四点各与 G0（F2，第一段）比，P 臂 Q1、Q2、Q9 的判定都是「没有」，不翻面；两点同值 0，门槛挪不进两点之间，判别力由 R-V1、R-V2 担（G-long 的 V1 臂 Q1 476、Q2 29 段）。

失败与停机条款（登记第十、十一节）：F1、F2 没触发（上表 R-V1、R-V2）；F3 两形 0（`sum_f3_still_but_holds`、`sum_f3_violated_without_two_births` 二十六份全 0）；F4、F5 没触发（下一节）；F6、F9 只在 P 有 Q1 事件时做，没有事件、没做；F7 没触发（F2 + F3a 的 Q5 合计 8 932）；F8 没触发（S1 重建报错 0 次，P 臂看到根的次数 F3a 1 431 682、F2 304 853、F1 四组 54 585 / 67 690 / 57 613 / 60 489、几何四点 213 378 / 75 155 / 75 554 / 104 381，出自两份汇总的 `sum_s1_root_observations`）；S3 两路不一致 0 次、读不出实例表的步 0 步；合并不确定没触发（二十六份 `conflicts=0`、`duplicates=0`，快档多线程与单线程对拍过）。

V1、V2 上全部 Q1 事件按 Q9 分类与那一步 I-9.29（同号只许一个出生代） 的判定交叉数：`grep -h '^E7RESULT name=event ' d-*.out f3a-*.out f2-*.out f1-*.out g-*.out | grep -o 'arm=[A-Z0-9]* .*kind=[a-z0-9_]* .*q9=[A-Za-z]* i9_29_at_step=[a-z_]*' | sed -E 's/index=[0-9]+ //; s/position=[^ ]* inode=[^ ]* new_birth=[^ ]* older_births=[^ ]* //' | awk '{print $1, $2, $3, $4}' | sort | uniq -c` 输出四行：`2 arm=V1 kind=q1_same_timeline q9=Gone i9_29_at_step=holds`、`9 arm=V1 kind=q1_same_timeline q9=Gone i9_29_at_step=violated`、`1917 arm=V1 kind=q1_same_timeline q9=Still i9_29_at_step=violated`、`5225 arm=V2 kind=q1_same_timeline q9=Still i9_29_at_step=violated`。判「还在」的事件那一步 I-9.29 全是违例（F3 第一形 0）。

阳性对照臂上「活记录被原地换」那一形（Q9「已转出」）出现 11 次：`grep -n 'q9=Gone' f2-V1.out g-long-V1.out` 命中 `f2-V1.out` 第 67、283、325、326、385 行与 `g-long-V1.out` 第 40、71、155、358、389、390 行。其中 `g-long-V1.out` 第 389、390 行（G-long V1 序号 24，第 92、95 步发号 7、8）那两步 I-9.29（同号只许一个出生代） 判成立：旧出生代已转出候选集、同一步候选集里也没有别的号有两个出生代，I-9.29 看不见那次重发——C479（inode 号复用已发过的号今天造不出来） 那一行写的「被换之前那一版转出候选集之后看不见」在 V1 臂上出现了；其余 9 次那一步另有号的两个出生代、I-9.29 判违例。P 臂上这一形 0 次。

### 钉值（登记第七节，快档用例，三臂各跑一次）

- A1、A2（D1，起点）：`first_file_alone_reads_inode_one_born_at_three_and_watermark_two` 三臂都过，现行那一版 inode 树 {(1, 3)}、水位 2，与 D8（核心索引结构） 已定项 6「新池新建文件写」相同。
- A3、A4（D2(k)、D3，P）：用例 `rollback_one_step_then_create_matches_the_arm` 等六条与 `creations_across_mounts_and_a_cold_start_match_the_arm` 在 P 臂上过（`quick-tests.out` 的 `===== q-P exit=0` 那一段，`test result: ok. 26 passed; 0 failed; 2 ignored`），断言 D2(1)–D2(3) 依次发 2、3、4，D3 发 2、3、4。`d-P.out` 的结果行与之相符：D2(1)、D2(2)、D2(3) 各 `issuances=4 watermark_final=5 rollbacks_applied=1`（起点 1 号加三次建）；D2(4) `issuances=3 watermark_final=4 rollbacks_applied=1`（回退到第一个文件还活着的那一版，之后建 inode 不调）；D2(5)、D2(6) `rollbacks_applied=0`，A3 那三格没有样本（修订 1 第 5 条）；D3 `issuances=4 watermark_final=5`。这一次快档没带 `--nocapture`，用例里的打印行没进 `quick-tests.out`。
- A5（D2(1)，V1）：发 3、Q1 恰好 1、Q9「还在」、那一步 I-9.29（同号只许一个出生代） 违例，过；k = 2、3 上发的号 = S1 读到的目标那一版水位 2，过。
- A6（D2(1)，V2）：前两次都发 2、第一次建之后那一步 I-9.6（水位大于两处最大号） 违例、第二次建那一步 Q1 恰好 1，过。
- A7：`f3a_scale_matches_the_registered_counts` 过（16 105、1 464、143 335），产物里数出来的段数与步数也是这三个数；第二段规模 `second_segment_scales_match_the_registered_counts` 过（F1 P 15 360 步、F1 V 3 840 步、几何四点 P 22 080 步、G-long V1 4 800 步），产物里各族段数与登记相同（「读数」一节的 `histories=`）。

### 判决行的点名

装置不写 `name=verdict` 行：在产物目录顶层 `grep -n 'name=verdict' $(ls -1 | grep -v superseded) | wc -l` 输出 0。每个量各报各的数（「读数」一节），判定按登记第六节逐条写在「判据逐条」一节；不为 0 的「没过」类计数只在阳性对照臂上，都是登记预期内的：V1、V2 两臂 Q1、Q2 不为 0（阳性对照本该红），V1 臂 Q3、Q7 不为 0（回退取目标那一版的水位，水位就会退回去）。P 臂上 Q1、Q2、Q3、Q3b、Q7、Q11、S3 不一致、F3 两形、S1 重建报错都是 0。

### 单测与变异

单测 28 条：`grep -c '^#\[test\]' device-source.rs` 输出 28，其中 2 条带 `#[ignore]`（大规模入口与汇总）；`quick-tests.out` 里三臂各 `test result: ok. 26 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out`。比第一段那一版多的 11 条：第二段规模与比重 2 条、每个新族序号 0 走一遍 8 条、汇总拒收两份代码混用或缺文件头的结果文件 1 条（`result_files_from_two_code_baselines_or_without_a_header_are_refused_by_the_summary`）。

变异（登记第九节）6 条：V1、V2 是两条阳性对照臂，跑到的每一族都分得出（「判据逐条」一节 R-V1、R-V2 过）；V3、V4、V5 是装置里的弄坏开关，`quick-tests.out` 里各红在点名的那一条（三次都带 `--skip first_history_of`，跳过 8 条每族序号 0 的取样用例，`17 passed; 1 failed; 2 ignored; 0 measured; 8 filtered out`）：V3（账按号去重）与 V4（发号事件分类时每条根都判被抛弃）在 V1 臂上都红在 `rollback_one_step_then_create_matches_the_arm` 的「A5：Q1 恰好 1」（`left: 0`、`right: 1`；V4 那一次同一段打印 `q1=0 q3b=1`），V5（Q9 一律判「还在」）在 P 臂上红在 `q9_classification_says_gone_when_no_holder_is_readable`；V717 照登记有条件，Q6 = 0，没跑。抓到 5、无效 0、没红 0、未跑 1。装置不在 `research/` 下、也不进 `crates/mutations.tsv`，没有变异表，`mutate.sh` 不适用。装置文件头弄坏开关那张表给 V3 写的用例名 `rollback_one_step_then_create_on_v1_reissues_three_on_the_same_timeline` 与实际红的 `rollback_one_step_then_create_matches_the_arm` 不同（文件头的字没跟用例改名一起改），开关红在的是后者。

### 岔路表

| 岔路行 | 状态 | 依据 | 还差什么，剩下的量能不能让它翻面 |
|---|---|---|---|
| W：C479（inode 号复用已发过的号今天造不出来） 剩下的「活记录被原地换」那一形 | 已够判（冻结副本 `code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`；上一份 `b000407a…` 上同样够判、读数逐族相同，不进判决） | 「读数」一节两份汇总：P 臂带回退的历史（F3a、F2）与随机历史（F1 四组）、几何四点 Q1 全 0、`segments_q2_one_birth_violated` 全 0；F2 + F3a 的 Q5 合计 8 932 ≥ 50；R-V1、R-V2 在跑到的每一族都过；F1、F2、F7、F8 没触发 | 不差。登记的 17 105 段产品臂历史里没造出同一条时间线上的重发号，I-9.29（同号只许一个出生代） 红 0 段，水位退回 0 步。剩下的量是第三段附带的 F3b（崩溃恢复抛弃根之后再建）、F4（写 / 刷盘故障注入之后再建），够判后未跑；它们不在 W 行够判条件点名的两类历史里，但能让「水位退回」那半句翻面：崩溃恢复落到较旧的根之后水位随所选根退回，那是条款允许的跨被抛弃时间线那一形（Q3b），按读法 B 不翻 W 行；若在那两族里量出同一条时间线上的 Q1 ≥ 1，W 行翻面 |

### 它答不了的

- 「登记的历史里没造出」，不是「造不出」：P 臂 17 105 段是登记的取样（F3a 中段长到 4 的穷举、F2 / F1 / 几何点按种子随机），长度更长、回退目标或操作次序没被抽到的历史没有数。
- 崩溃恢复抛弃根（双故障形，F3b）与写 / 刷盘故障注入后建 inode（F4）是登记第三段附带，够判后未跑；历史里抽到的 `CrashRecoveryAbandoningTheNewestRoot` 在 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 乙之后挂载被拒、不造被抛弃根，Q3b 的 0 不说明跨被抛弃时间线的重发不存在。装置没有数「判成被抛弃的根」的次数，两段里 S3 有没有判出过一条被抛弃根，页上不下结论。
- Q6 的读法有一处局限：现行那一版取自这一步可读根里最新的那条，回退之前它自己就在环里读得出，Q6 > 0 只可能在现行那一版自己重建不出时出现；无读故障的历史里 Q6 = 0 由这个读法推得出，不是独立的观测。V717 因此在每一族都没跑，「回退取 max」那条护栏在这些历史里有没有被走到，这一页答不了。
- 「活记录被原地换」那一形 I-9.29（同号只许一个出生代） 看不见：阳性对照 V1 臂 G-long 序号 24 的两次重发（`g-long-V1.out` 第 389、390 行）那两步 I-9.29 判成立。产品臂上没造出这一形，所以这一页答不了「产品路径上出现了 I-9.29 能不能抓」，只说 P 臂上它 0 次。
- 「仓副本装置、冻结副本」：数出自 2026-10-08 拷的第二份冻结副本（`baseline-frozen-crates.out`），没在入库装置上重做过。拷完之后主工作区 `crates/` 又变了 6 个文件（`baseline-frozen-crates.out` 的 `name=main_drift_after_copy`），其中 `crates/singlefs-core/src/mount.rs` 与 `crates/singlefs-harness/src/gpu_memory_ledger.rs` 编进这个装置；F3a、F2、F1、几何点这些族没在漂移后的代码上重跑，只有 `scripts/experiments/replay.sh` 的 `driver_e175` 在漂移后的主工作区上编、跑 D 族 P 臂一次，输出去掉 `elapsed_ns` 与 `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` 相同。
- 什么现象会推翻页上的读数：同一份冻结副本、同一装置重跑出任何一段 P 臂结果行与产物不同（续跑与多线程合并在快档对拍过）；或第三段 F3b / F4、或登记之外的历史里出现同一条时间线上 Q1 ≥ 1、I-9.29（同号只许一个出生代） 红。

### 路径与结论登记

| # | 路径 | 怎么算 | 源码落点 | 读了哪些共用项 |
|---|---|---|---|---|
| 1 | S3 路 1（实例表） | 最新那条可读根的实例表，按 (实例, txg) 判每条根抛不抛弃；对两段每一步的每一条可读根（扫遍） | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:713 abandoned_by_the_table`，判法 `crates/singlefs-core/src/recovery.rs:1553 publish_is_abandoned_by_the_instance_table` | `readable_roots` 列出的根、盘上实例表、`crates/` 的抛弃判法 |
| 2 | S3 路 2（步账） | 每次挂载做成，模型根环里那一步之前最新的那条根当所选根，同一旧实例里 txg 更大的根判抛弃 | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:723 abandoned_by_the_step_ledger`，所选根取自 `crates/singlefs-harness/src/model.rs:2235 answer_mount_writable` 的择法 | `readable_roots` 列出的根、理想模型的根环、执行器交回的挂载结局 |
| 3 | S1 账的重发号（Q1） | 按号记出生代与持有它的根，一次发号碰上同号另一个出生代就记事件 | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:1028 HistoryAccount::observe`，重建 `crates/singlefs-core/src/recovery.rs:5405 rebuild_version` | `readable_roots`、`rebuild_version`、路 1 的抛弃判法 |
| 4 | S2 checker 的 I-9.29（同号只许一个出生代） | 池级 checker 对每一步的镜像判 | `research/results/e175-product-path-inode-watermark-regression-2026-10-08/device-source.rs:1158 HistoryAccount::judge_with_the_checker`，`crates/singlefs-checker/src/walk.rs:10947 check_pool_image` | 只有镜像本身；与 1–3 不共用解析代码 |

路 1 与路 2 判同一个量，逐根比、不一致计数（`device-source.rs` 第 865 行 `s3_path_disagreements`），两段二十六份结果文件都是 0 次；两路共用 `readable_roots` 列出的那一组根，它列错了两路会一起错。路 3 与路 4 不是同一个量，登记第十节 F3 把两者对着核（第 1188、1199 行），两段都是 0 次。跨路径的比较是计数、不是 `assert_eq!`，欠一条把两路连起来的断言。

### 影响的决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D8（核心索引结构） 已定项 6 | 备料 | 2026-10-08 不受影响：W3 的 Opt2 定案（用户 Y1）只让合法的跨抛弃重发多 11.6%，被崩溃恢复抛弃那段里发过的号本来可以重发，射程「同一条时间线上 inode 号不复用」不变（`research/prompts/m3-w3-r3-main-verification.md` 第二节「Opt2 的代价」那一行与「回看决策」表）；两段在第二份冻结副本（`code_baseline=48e2809c…`）上的读数（登记的 17 105 段产品臂历史上 0 次重发、0 段 I-9.29（同号只许一个出生代） 红，W 行够判）是改之前的取法上量的；决策依据段没引 E175（产品路径上 inode 号水位退回与重发号），这一格该不该升成支撑交主 agent 定 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-08 改了：W3 的 Opt2 定案（用户 Y1），「水位」那一格的 inode 号水位改成 max(cur 那一版内存里的, 环里读得出、按实例表没被崩溃恢复抛弃的根的)，树 ID 水位照旧（`research/prompts/m3-w3-r3-main-verification.md` 第六节）；两段在第二份冻结副本（`code_baseline=48e2809c…`）上的读数（回退那次的水位取 max 这一格，D2(k) 上发 4（A3）、F3a / F2 / F1 / 几何点的 Q4 窗口里 0 次重发）是改之前取环里全部根的取法上量的；决策依据段没引 E175（产品路径上 inode 号水位退回与重发号），升不升成支撑交主 agent 定 |

## 历史版本

### 2026-10-08：两段在第二份冻结副本上重跑，读数逐族不变

根环线第 10z 批第十、十一块合入之后 `crates/singlefs-core/src/mount.rs` 又改了（抬 F 那一串），主 agent 拷了第二份冻结副本（`code_baseline=48e2809c75c44b79e317baa3fc128ccdbc9fb294b16b99997c59f2eb57ae9beb`，与上一份差 10 个文件），两段（D、F3a、F2、F1 四组、几何四点）的 P、V1、V2 照登记在它上面整段重跑，装置源码不变（`310c60a5…`）。上一份冻结副本上的 37 份照登记修订 2 第 4 条那一形挪进 `superseded-b000407a/`，不进判决。新二十六份结果文件的 E7RESULT 行去掉 `elapsed_ns` 与上一份逐份相同，两份汇总除 `code_baseline` 外逐行相同；状态仍是「已跑（够判）」，页上的数只换了 `code_baseline`、拷的批次、V1 锚点行号（7758 → 7917）、线程数（7 → 5）、试跑挂钟与 `f2-V1.out` 里「已转出」那几行的行号。

### 2026-10-08：两段在冻结副本上重跑，W 行够判

照用户定的「新代码上两段都重跑」与登记修订 2：主 agent 在根环线第 10z 批第九块合入、验完之后拷了一份冻结副本（`code_baseline=b000407ab5f4afdffe9083ddaac343fe8470438ea17891c4637b1684d5052b44`），第一段整段重跑、第二段（F1 四组、几何四点）首次跑，都在这一份上。旧代码上第一段的 19 份挪进 `superseded-old-crates/`，不进判决；新第一段九份结果文件的 E7RESULT 行去掉 `elapsed_ns` 与旧的逐份相同。装置加了 `code_baseline` 文件头与汇总列（单测 28 条），`e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out` 换成冻结副本上的那一份。两段 P 臂 Q1、Q2 都是 0，F1 F2 F7 F8 没触发，状态由「部分已跑（第一段；第二段停跑：crates/ 与第一段不同）」改成「已跑（够判）」，第三段够判后未跑。

### 2026-10-08：第二段派发，crates/ 与第一段不同，停跑

第二段（F1 四组、第八节几何点）开跑前核主工作区 `crates/`，6 个文件与第一段基线不同（`baseline-segment-two.out`），照派发提示停跑，一份产物都没跑；状态由「部分已跑（第一段；第二段待点名）」改成「部分已跑（第一段；第二段停跑：crates/ 与第一段不同）」。补上 F1 族与几何旋钮的装置只在执行员草稿目录里，没进产物目录。

### 2026-10-08：建页，第一段已跑

仓副本装置写好（单测 17 条，快档三臂各 15 过、2 条大规模入口忽略），跑前登记第十二节写了修订 1–8；第一段（D、F3a、F2 三族在 P、V1、V2 三臂上）跑完、汇总，状态「部分已跑（第一段；第二段待点名）」。`scripts/experiments/replay.sh` 登记一行 `E175|@driver_e175`，抽出驱动试跑一次，输出与 `d-P.out` 的 E7RESULT 行逐行相同、落成 `e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out`。
