# E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 第一段（H4）重跑登记：清扫阈值的本地常量 369 → 365

写于 2026-10-30，装置改之前、产物之前。

<!-- doc-lint:not-numbers H2 H3 H4 H5 T1 T2 T3 T4 V37 G8 K30 K32 A30 A31 A33 -->

判据、门槛、作废与停机条款在这里写死；跑出数之后要改，照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」三步走，不回改这里。

**这是差分重跑**（`.claude/agents/experiment-designer.md`「做什么」第 4 步：输入变了、同臂重跑）：臂、H4 的判据、阳性对照与上一次登记相同；这一份只写三样——变了哪些输入、哪几行判据要重判、这一次新加的比对判据与钉值换法——其余各节写「同上一次登记」。

- 上一次登记：`research/prompts/e183-preregistration.md`，已在提交 `59ce5a865` 移出工作区，取法 `git show 59ce5a865^:research/prompts/e183-preregistration.md`（819 行，`sha256sum` 为 `c4609ca35a24704e5d4e52b2fe92503cda2d73efe18c39d503a3d5cd4e1a7f21`）。下文「上一次登记第 N 节」都指这一份。
- 问题单：`research/prompts/m3-close-b19-e183-threshold-365-questions.md`（17 行，`sha256sum` 为 `5237187230084de0a238b5b585daa42c6f31c93f871769b80cec0ab3b10aef89`）。
- 第几次：实验页 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md` 与 `.claude/kb/experiments-history.md` 的 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 一节都没有标题写着「第 n 次重跑」的（2026-10-23 那一次是「只修钉值」，没写重跑登记），这一份按 **E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 第 1 次重跑**（只跑第一段）写；主 agent 认。
- 文件名：照派发提示落在问题单旁边；定义里重跑登记的名字形态是 `research/prompts/e183-r1-prereg.md`，名字不同带来的后果写在第十一节末「文件名」一条。
- 只跑第一段：第三段（H5）的产物、第二段（H2）与第四段（H3）这一次一概不碰，不重跑、不重钉。

**装置写在哪（写死）：入库装置，同上一次登记。** 改的是 `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs` 这一份（执行员照 `experiment-runner` 定义用 `scripts/hosts/compile-then-swap.py` 换进主工作区）。依据同上一次登记表头那一句：问的是 `crates/` 今天这份代码在第一段上给出的输出。本地常量的规矩照旧：`SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 仍是装置里的本地常量，值抄自 kb 登记位，再与 `crates/` 那一份回比，**不许改成直接引** `InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE`。这一份要读 `crates/` 的常量只有这一个（第五节 5.1 表），其余 20 个本地常量照上一次登记表头那张表与第十二节第二条，不动。

**起跑前提（写死，第十一节「起跑前提不成立」那一条）**：执行员开工第一步核四样，任一不成立就不开工、交回：

1. `crates/singlefs-core/src/instance_table_sweep.rs` 里 `pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);` 恰好一处（今天在第 74 行）。
2. 装置里 `const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 369;` 恰好一处（今天在第 78 行），且 `grep -n SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 只命中两行（今天第 78、1879 行）。已经是 365 的（别的会话先改了），停下交回：那就不是这一份登记的起点。
3. `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` 与 `…-h4-2026-10-23-anchor-check.out` 两份在，`sha256sum` 与第十一节比对对象表里记的相同。
4. `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py` 的 sha256 仍是 `d44145025eda06d8a72683f64774d2914fcdd5e1bd929f03d94fd1308b3c265d`（上一次登记第十三节命令一的那一份），`small_disk_anchor_check.py` 的 sha256 仍是 `9004491d7639a08f7231e4b54bc6077ec1e52cbf3b0334f5e89726c2d273baee`（这一份第十三节现查的）。

另记不停的一样：开跑那一刻主工作区 `git diff HEAD -- crates/ | sha256sum` 与 `git diff HEAD --stat -- crates/ | tail -1` 写进第十二节（被测的是跑的那一刻的树；这一份写的那一刻的值在第三节）。

## 一、问题

英文名：capacity_read_cache_reserve_and_sweep_costs

（照抄上一次登记第一节第一行；装置文件名 `e183_capacity_read_cache_reserve_and_sweep_costs.rs` 与它相同。）

主 agent 给的问题逐字（2026-10-30 派发「要回答的问题」一节）：

> 问题单 `research/prompts/m3-close-b19-e183-threshold-365-questions.md` 两行（第一段装置常量 `SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 369 → 365 之后产物变哪几行、两条快档用例与 A31 独立核对对不对得上新产物）。被测条款：`crates/singlefs-core/src/instance_table_sweep.rs:74` 的默认阈值 365（C690（清扫触发阈值的默认值没定），`.claude/kb/checks-owed.md` 那一行现查）。E183 实验页 `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md`，第一段跑前登记 `research/prompts/e183-preregistration.md`。
>
> 照定义写重跑登记（这一次不新取实验号，是 E183 第一段的重跑）：写死要改的那一个常量、重出哪份产物（新产物文件名带 2026-10-30）、判据（「只变第 6 行」与「别的行也变」各怎么记）、作废与失败条款、复跑登记与钉值怎么换；落在问题单旁边 `research/prompts/m3-close-b19-e183-threshold-365-rerun-registration.md`，排他新建。不改装置、不跑产物。

问题单（不是岔路单：这一份不为岔路建，是常量镜像跟不上实现之后重出登记产物）的「要回答的问题」一节逐字（问题单第 12–17 行，`sed -n '12,17p'` 原样追加，放进围栏，免得它的二级标题打断这一份的节次）：

```markdown
## 要回答的问题

| # | 问题 | 怎么算答完 |
|---|---|---|
| 1 | 常量改成 365 之后，第一段 `all` 产物除第 6 行那一行比对之外还变不变 | 重出产物，与 2026-10-23 那一份逐行 diff；只变第 6 行就答「只变那一行」，别的行变了逐行列出、并判 H4 的判决（实验页「判决」一节整行抄的那几行）变没变 |
| 2 | 改后两条快档用例与 A31 独立核对（`driver_e183_anchor_check`）是不是都对得上新产物 | 新产物登记进复跑表，产物 sha 的钉值照新产物改，两条用例与独立核对的命令写进交回报告「要主 agent 跑的命令」 |
```

### 1.1 读法写死

| # | 项 | 写死成 | 为什么 |
|---|---|---|---|
| Y1 | 「第一段 `all` 产物」 | 装置 `-- all` 的**标准输出**整份；标准错误（每项耗时、退出码）不算，另存 | 同上一次与 2026-10-23 那一份的取法（实验页「复跑」一节「每项耗时与退出码打到标准错误，不进产物」） |
| Y2 | 新产物 | `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`；A31 核对对它的输出 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`。旧的两份（2026-10-23）留着不动，不覆盖 | 问题单第 1 行「与 2026-10-23 那一份逐行 diff」；派发「新产物文件名带 2026-10-30」 |
| Y3 | 「逐行 diff」 | `diff <旧产物> <新产物>`（GNU diff 默认格式），原样存进执行员草稿目录并贴进实验页；行数不同照 diff 的 hunk 原样列 | 一条命令出、不手数 |
| Y4 | 「只变第 6 行」 | 三样同时成立：① diff 的输出恰好一个 hunk，头是 `6c6`；② 那个 hunk 的旧行逐字等于问题单背景第 3 条引的旧第 6 行；③ 新行逐字等于第七节 7.1 钉的那一行。三样任一不成立都不叫「只变那一行」 | 问题单第 1 行「只变第 6 行就答「只变那一行」」；②③ 防「第 6 行变了、但变成别的样子」 |
| Y5 | 「别的行也变」 | diff 里有第 6 行以外的任何一行（含行数变了、多出或少了的行） | 问题单第 1 行「别的行变了逐行列出」 |
| Y6 | 「H4 的判决」 | 旧产物第 23、26、49、120、197、289、369、395、421、447、458 行（实验页「判决」一节标题点名的 11 行）；新产物里按同一个 `name=verdict` 加同一个 `item=` 取对应的那一行（行数没变时就是同号的行）。「变没变」逐行、逐字段比 | 问题单第 1 行「判 H4 的判决（实验页「判决」一节整行抄的那几行）变没变」 |
| Y7 | 「两条快档用例对得上」 | 改完装置（第五节 5.1 那三处）之后，`local_constants_equal_the_implementation_constants` 与 `segment_one_all_output_equals_the_registered_product_byte_for_byte` 两条用例在主工作区那一份装置上都绿 | 问题单第 2 行 |
| Y8 | 「A31 独立核对对得上」 | `small_disk_anchor_check.py` 照原样（第十一节起跑前提第 4 样的 sha256）对新产物跑，退 0，输出末行 `name=verdict item=anchor_check` 的 `mismatches=0` | 问题单第 2 行；`driver_e183_anchor_check` 的判法 |

问题单两行在这一份里由谁罩：

| 问题单行 | 罩它的量（第六节） | 「怎么算答完」怎么对上 |
|---|---|---|
| 1 | 逐行差、第 6 行新值、判决行、行数与字节数、两遍确定性 | 「重出产物，与 2026-10-23 那一份逐行 diff」⇒ 逐行差；「只变第 6 行就答「只变那一行」，别的行变了逐行列出、并判 H4 的判决变没变」⇒ Y4、Y5 与判决行 |
| 2 | 回比用例、逐字节用例、A31 核对、A31 输出差 | 「新产物登记进复跑表，产物 sha 的钉值照新产物改，两条用例与独立核对的命令写进交回报告「要主 agent 跑的命令」」⇒ 第五节 5.2、5.3 的改法与第六节四个量 |

## 二、被测条款与它引的定义

被测的是 C690（清扫触发阈值的默认值没定） 那一行今天写的默认值。装置里这个本地常量的 kb 登记位，上一次登记表头那张表写的是「C690 那一行「阈值是占位值 369」」；那一句今天已不在，C690 那一行现写默认值 365（用户 2026-10-29 弹窗定）。这一次本地常量的值改抄自下面这一行。D28（挂载期承诺量） 已定项 1、3 与 D18（块里携带什么信息） 已定项 11 同上一次登记第二节，不重抄（第一段的判据只用到 D28（挂载期承诺量） 那两项，那两项的字面这一次不变；变的只是本地常量的登记位）。

由 `scripts/kb/quote-kb.py` 抄进草稿目录的出口文件 `quote-c690.md`、回读逐字节一致（rc=0，第十三节命令二）之后原样追加在下面：

**出处 `.claude/kb/checks-owed.md:360-360`（整段抄，未转述）**

```markdown
| C690 | 清扫触发阈值的默认值没定 | D18（块里携带什么信息） 已定项 11 的整轮清扫在开关关着时按实例表行数超过阈值触发，阈值是配置项，默认值没定（用户定：开关关着时看行数，超过阈值立刻清扫（大意）；默认值另定）；岔路单给的翻面观测是阈值版在某段历史上让行数越过 D28（挂载期承诺量） 已定项 3 切换预留式子里行数项的上界 | 默认值定为 365 之后：连挂到行数越过 365 的录制流里断言越过的那次可写挂载发出清扫发布、没越过的不发，且任一合法历史上行数不越过切换预留式子里行数项的上界；判别力自证：把阈值判断拿掉（每次都扫，或永不扫）必须红 | 清扫器已写（`crates/singlefs-core/src/instance_table_sweep.rs`，阈值默认 365：`DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE`，用户 2026-10-29 弹窗定「365」，出处 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 第三段 H5 的读数：θ = 365 不越过一片的上界、θ = 369 越过）；一片仍装 369 行，行数每多 369 行切换预留多一片（C597（实例表行只增不删，删行的条件没有条款））；越过 365 而不清扫有三形，后果都只是下次预留按两片留（保守）：现行那一版树表 0 条（`NoRowDeletedThisRound::SweepOnAVersionWithoutFileNotSupportedInTheFirstVersion`）、清扫这一轮一行不删（`NoRowDeletedThisRound` 的其余成员）、开关开着（交给挂着时的后台清扫）；去向：里程碑三二·4（与 C597（实例表行只增不删，删行的条件没有条款） 同属实例表清扫，默认值要先量）（五·11，2026-10-12 判）。 | 用户 2026-10-03 定，判决 `research/prompts/m3-sweep-and-tree-id-r1-main-verification.md` 第六节 Q1 行；岔路单 `m3-sweep-and-tree-id-r1-forks.md` Q1「多久跑一次」行 |
```

## 三、实现今天的样子

主工作区现状（HEAD `59ce5a865`，另有没提交的改动：`git diff HEAD --stat -- crates/ | tail -1` 末行「182 files changed, 52682 insertions(+), 5340 deletions(-)」，`git diff HEAD -- crates/ | sha256sum` = `97a9244396740ad4e022964108b874b7ed50cd8359adbe990407b519fbc28275`，第十三节命令三）。行号是 2026-10-30 现查的，执行员开工时照内容重核。

| # | 事实 | 位置 |
|---|---|---|
| J1 | 产品路径的清扫阈值：`pub const DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE: Self = Self(365);`；HEAD 那一版是 `pub const PLACEHOLDER_UNTIL_C690: Self = Self(INSTANCE_TABLE_PAGE_RECORDS - 1);`（第 71 行），改名与改值都在没提交的改动里 | `crates/singlefs-core/src/instance_table_sweep.rs:74`；HEAD 那一版 `git show HEAD:crates/singlefs-core/src/instance_table_sweep.rs` 第 71 行 |
| J2 | 产品路径的清扫配置 `InstanceTableSweepConfiguration::PRODUCT` 取 J1 的阈值；`mount::mount_writable` 用它 | `crates/singlefs-core/src/instance_table_sweep.rs:98-105`；`crates/singlefs-core/src/mount.rs:8407`（`pub fn mount_writable`） |
| J3 | 装置的本地常量与它的文档注释：第 77 行注释「清扫阈值的占位：`.claude/kb/checks-owed.md` C690 那一行「阈值是占位值 369」。」，第 78 行 `const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 369;` | `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs:77-78` |
| J4 | 回比：`constants_against_the_implementation` 的第 5 项 `"sweep_threshold_placeholder_rows"` 拿 J3 与 `InstanceTableRowCountThreshold::DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE.0` 比（HEAD 那一版比的是 `PLACEHOLDER_UNTIL_C690.0`，改名已在没提交的改动里跟上）；`--item constants` 每项打一行 `name=constant_compare constant=… local=… implementation=… equal=…`，末尾一行 `name=verdict item=constants constants_compared=… constants_mismatched=…` | 同上 `:1850-1881`（回比表）、`:2178`、`:2187`（两种结果行） |
| J5 | 第一段 `all` 跑的十项：`ITEM_NAMES`；`["all"] => run_items(&ITEM_NAMES, true)`；`"all"` 在全文件作字符串字面量恰好出现 1 次 | 同上 `:6124`、`:6290` |
| J6 | 第一段的可写挂载走产品路径 `mount_writable`（装置第 1257、1330、3032、3083 行），清扫配置取 J2；H4 的历史至多 mkfs、空池挂载、首个文件、正常卸载、再挂载，实例表行数到不了 365（推的，第四节 P6） | 同上 `:1257`、`:1330`、`:3032`、`:3083` |
| J7 | 第 4260 行另用 J1（`SweepArm::HandedToTheBackground` 的配置，开关交给后台、阈值不起作用），只在第三段（H5）的对照格里，第一段不走 | 同上 `:4245-4263` |
| J8 | 两条快档用例：`local_constants_equal_the_implementation_constants` 对 J4 每一项 `assert!(is_equal)`；`segment_one_all_output_equals_the_registered_product_byte_for_byte` 起子进程跑第一段 `all`，把每行从 `E7RESULT ` 起截下、各加一个换行连起来求 SHA-256，与 `SEGMENT_ONE_PINNED_SHA256`（`b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`）比，行数与 `SEGMENT_ONE_PINNED_LINE_COUNT`（459）比；两个钉值上面的文档注释写来源与历次重钉 | 同上 `:6853-6863`、`:6899-6909`、`:7010` 起 |
| J9 | 复跑登记：`E183|@driver_e183||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out|exact` 与 `E183|@driver_e183_anchor_check||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23-anchor-check.out|exact`；`driver_e183_anchor_check` 里写死读 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` | `scripts/experiment-apparatus/replay.sh:1811`、`:1812`、`:3273-3276` |
| J10 | A31 核对脚本：输出头一行 `name=anchor_check_script` 带锚点脚本与**产物**的 sha256（产物一换，这一行必变）；每份读数一行 `name=anchor_check`；末行 `name=verdict item=anchor_check readings_checked=… mismatches=…`；不读 `name=constant_compare` 行（`grep -n 'constant_compare\|sweep_threshold'` 在这个目录下零命中） | `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py:31-33`、`:56`、`:60` |
| J11 | 变异表里没有一行的原文含 J3 那一行或 J8 那个 SHA 值：`grep -n 'b4ff64f1\|SEGMENT_ONE_PINNED\|const SWEEP_THRESHOLD' crates/mutations.tsv scripts/mutation/tables/*` 零命中 | 第十三节命令四 |

## 四、跑之前已经存在的数

这一份是重跑，答案一部分已经写在问题单与实验页里；下面照实列，标明对判据的影响。`research/results/` 下两份旧产物只算过 `sha256sum`、`wc -l`、`ls -l` 字节数、`grep -vc '^E7RESULT '` 与末字节，内容没读。

| # | 已经存在的数 / 事实 | 从哪读到 | 对判据的影响 |
|---|---|---|---|
| P1 | 旧产物第 6 行整行「E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=369 implementation=369 equal=true」 | 问题单背景第 3 条 | 进 Y4 ② 当旧行的期望；不是新产物的答案 |
| P2 | 两条快档用例 2026-10-30 红：回比用例报「sweep_threshold_placeholder_rows：本地 369 实现 365」；逐字节用例红 | 问题单背景第 3 条 | 改前的红就是两条用例「分得出这一处」的读数（第九节）；改后要绿 |
| P3 | 旧产物 sha256 `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`、459 行、152434 字节、每行以 `E7RESULT ` 起头、末字节是换行；旧 A31 输出 sha256 `da09f8264058bfac8b39e83185141ade227b1ec3ee760d33837a423a4a453cf7`、58 行 | 第十三节命令五（量的）；J8 的钉值与实验页「做了什么」第 7 条同值 | 进第十一节比对对象表与第七节 7.2 |
| P4 | 实验页「复跑」一节的结果行条数表（21 行 `name=constant_compare`、11 行 `name=verdict`、56 行 `name=h4_reading_device` 等）与「做了什么」第 1、7 条（`constants_compared=21 constants_mismatched=0`；A31「56 份读数 0 处不符」） | 实验页第 27、33、53–75 行 | 新产物的 `constants_mismatched` 期望 0（第七节 7.1）；A31 的 `readings_checked` 期望 56（7.2）。这几个是产物的结构数，不是 H4 判决的值 |
| P5 | C690（清扫触发阈值的默认值没定） 那一行与 `instance_table_sweep.rs` 第 69–73 行注释里写着第三段（H5）的结论「θ = 365 不越过一片的上界、θ = 369 越过」 | 第二节抄的 C690 行；J1 | 无：第一段的判据不用 H5 的任何数，这一次不碰第三段 |
| P6 | **推的**：常量改成 365 之后只变第 6 行，新第 6 行是 7.1 钉的那一行。依据：① `SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 只在回比里用（J3、J4，问题单背景第 2 条的 `grep`）；② 第一段的挂载走产品路径，阈值从 369 变 365 只在挂载末尾行数 > 365 时改行为，第一段的历史行数至多个位数（J6）；③ "369" 与 "365" 等长，只变第 6 行时新产物仍是 459 行、152434 字节。**推不到的一块**：2026-10-23 之后 `crates/` 另有别的改动（第三节的树 sha 与那一次不同），它们会不会改第一段别的行，这一份没核 | 第三节 J3–J7 | 这是答案的预估，写在这里是为了不藏。判据（第六节）不用它：Y4、Y5 只看 diff，不看推的；推的与量出的不符就如实并列（第十节「别的行也变」） |
| P7 | 旧产物第 23 行推的是 `name=verdict item=constants` 那一行（第 1 行 `name=config`，第 2–22 行是 J4 回比表次序的 21 行，第 6 行是其中第 5 项，与问题单说的「第 6 行」对得上） | 从 J4 次序与 P4 条数推的，没读产物 | 判决行 Y6 里这一行的 `constants_mismatched` 改后期望仍是 0 |

## 五、臂、阳性对照、真实基线

臂、真实基线、阳性对照（PC-堆、PC-准入、PC-准入-文件）同上一次登记第五节 5.1、5.3 与第十二节执行员第一段两条修订；第一段 `all` 的十项照旧全跑，阳性对照照旧在 `all` 里每一项各判，不另起。这一次不加臂、不删臂。变的输入只有下面 5.1 那一处常量（连同它的注释）与 5.3 的钉值；别的输入（`crates/` 自 2026-10-23 以来的其余改动）不由这一份改，跑的那一刻是什么就是什么，树的 sha 记进第十二节。

### 5.1 装置改法（写死；只许这几处，别处一个字节不动）

| # | 位置（今天的行号，照内容重核） | 改前 | 改后 | 什么时候改 |
|---|---|---|---|---|
| U1 | 装置第 78 行 | `const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 369;` | `const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 365;` | 跑产物之前 |
| U2 | 装置第 77 行（U1 的文档注释） | `/// 清扫阈值的占位：`.claude/kb/checks-owed.md` C690 那一行「阈值是占位值 369」。` | `/// 清扫阈值：`.claude/kb/checks-owed.md` C690（清扫触发阈值的默认值没定） 那一行「阈值默认 365」（用户 2026-10-29 定）；常量名与产物里的键名 `sweep_threshold_placeholder_rows` 是 2026-10-18 起的旧名，不改（改了产物第 6 行的键跟着变）。` | 跑产物之前 |
| U3 | 装置 `SEGMENT_ONE_PINNED_SHA256`、`SEGMENT_ONE_PINNED_LINE_COUNT` 两个钉值（今天第 6907–6909 行）与它们上面的文档注释（第 6899–6906 行） | `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`、`459` | 新产物（第一节 Y2）的 `sha256sum` 与 `wc -l`；注释末尾补一句：C690（清扫触发阈值的默认值没定） 默认值定 365 之后重钉、变了哪几行（照第六节「逐行差」的判定写）、对的是 2026-10-30 重出的第一段 `all` 产物 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out` | 产物跑完、第十节放行之后（第十节「判决行变了」打中时不改） |

**不许改的**：常量名 `SWEEP_THRESHOLD_PLACEHOLDER_ROWS` 与回比表里的键 `"sweep_threshold_placeholder_rows"`（改名会让第 6 行除数值外再变一个字段，「只变第 6 行」判不清是哪一样引起的）；第 4209 行 `SWEEP_THRESHOLD_CANDIDATE_ROWS`（第三段的四个 θ）；其余 20 个本地常量；任何判法与输出格式。

**钉值为什么只能在产物之后改**：U3 的值取自新产物，逐字节用例因此在构造上就会对上它自己那一份装置在 release 下的输出；这条用例能分辨的是「测试构建里起的子进程」与「release 二进制」两次独立跑的输出是不是逐字节相同（第六节「逐字节用例」一行），不是「新产物对不对」——对不对靠「逐行差」「第 6 行新值」「A31 核对」三行。

### 5.2 产物跑法（照 2026-10-23 只修钉值那一次，实验页「做了什么」第 7 条）

1. 起跑前提四样（表头）核过。
2. 仓副本（主工作区当天的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，不带 `target/`）里只打 U1、U2，`git diff --no-index` 或 `diff` 核副本里装置与主工作区那一份只差 U1、U2 两行；release 编出装置（`opt-level = 3`、`overflow-checks = true`，照 checker 档包的配置），记源文件与二进制的 sha256。
3. `all` 跑两遍（内存上限 8G 同上一次；线程上限照派发提示；装置单线程），标准输出各存一份、标准错误另存；两遍都退 0、`cmp` 逐字节相同，才把其中一份落成 Y2 的新产物（新文件名，不覆盖旧的）。
4. `find crates -name '*.rs' -newer <二进制>` 在副本与主工作区各跑一次：`singlefs-core`、`singlefs-format`、`singlefs-harness` 下有比二进制新的源文件，照第十一节作废条款办。
5. 跑第六节各行（`diff`、`sha256sum`、`wc -l`、`wc -c`、`grep -vc '^E7RESULT '`、判决行逐字段比、A31 核对），原样输出存草稿目录。
6. 第十节放行之后再打 U3：副本里改钉值，`scripts/hosts/compile-then-swap.py` 把装置换进主工作区（换进去的那一份与副本逐字节相同，只比主工作区原来那一份多 U1、U2、U3）。

### 5.3 复跑登记与钉值怎么换（写死；第十节「判决行变了」打中时一处都不改）

| # | 文件 | 改什么 |
|---|---|---|
| W1 | `scripts/experiment-apparatus/replay.sh` 第 1811 行 | 期望产物 `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` → `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`；驱动、判法 exact 不变 |
| W2 | 同上第 1812 行 | 期望产物 `…-h4-2026-10-23-anchor-check.out` → `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out`；判法 exact 不变 |
| W3 | 同上 `driver_e183_anchor_check`（第 3275 行） | 读的产物路径 `research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` → `…-h4-2026-10-30.out`；第 3271–3272 行注释照旧 |
| W4 | 装置 U3 | 见 5.1 |
| W5 | `.claude/gate.d/replay-comparison-units.tsv` | 第十一节比对对象表两行各登记一行（实验号 E183（容量斜率读缓存小盘准入与清扫阈值的代价数）、登记出处这一份的路径、单元名 `all`、单元表出处装置源文件、留存产物名为表里的「重跑产物名」、复跑 `driver_e183` 与 `driver_e183_anchor_check`）：`all` 在装置里作字符串字面量恰好 1 次（J5），一个单元对两个复跑行各登一行，照那张表文件头的写法 |
| W6 | 旧的两份产物 | 留着，不改名、不删（实验页引它们；归档照 `.claude/agent-common.md`「规则怎么读」那一条由后面的提交做） |

实验页与历史条目由执行员照它定义写：实验页加「第 1 次重跑」一节（diff 原样、判定、两条用例与 A31 的结果），「复跑」一节里指旧产物名的三处（第 46、49、51 行一带）照现状改指新产物；改前原文进 `.claude/kb/experiments-history.md`。

### 5.4 分段与实现量

一段做完：改两行装置、编一次、跑两遍 `all`、一次 A31、改钉值、改复跑登记三行与比对对象登记两行。不分段。两条快档用例是 checker 档（重型），执行员不跑，命令写进交回「要主 agent 跑的命令」（第六节对应两行写了命令形态）。

## 六、报哪些量与各自的判据

**同上一次登记第六节的那几行（R4a–R4d）这一次不重判**：第一段的判据原样，新产物里它们的数若与旧产物不同，就是下表「判决行」与「逐行差」那两行打中，按第十节办，不在这一份里重新下 H4 的结论。下表是这一次新加的量；每个量各占一行、各报各的判定，不做合取。

| 量 | 怎么算 | 门槛 / 判定 | 对应问题单哪一行 · 什么取值让它翻面 |
|---|---|---|---|
| **逐行差** | `diff research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`，原样输出；另报 hunk 数（`grep -c '^[0-9]'`） | Y4 三样都成立记「只变那一行」；否则记「别的行也变」，逐个 hunk 列出旧行与新行（整行抄） | 问题单第 1 行。diff 里出现第 6 行以外的任何一行 ⇒ 从「只变那一行」翻到「别的行也变」 |
| **第 6 行新值** | 新产物 `awk 'NR==6'` | 逐字等于第七节 7.1 钉的那一行记「对」，否则「不对」并按字段列（`local=`、`implementation=`、`equal=` 各自是什么） | 问题单第 1 行（Y4 ③）。`implementation=` 不是 365 ⇒ 第十一节停机「实现又变了」；`local=` 不是 365 ⇒ 第十一节作废「装置改错」 |
| **行数与字节数** | 新产物 `wc -l`、`wc -c`、`grep -vc '^E7RESULT '`、末字节 | 照报；行数 ≠ 459 或字节数 ≠ 152434 记「结构变了」（必然伴随「别的行也变」）；`grep -vc` ≠ 0 照第十一节作废 | 问题单第 1 行的旁证（「369」与「365」等长）；问题单第 2 行：U3 的行数钉值取这里的 `wc -l` |
| **判决行** | Y6 那 11 行，旧行与新行逐字段比（按空格切成 `键=值`，键集合与每个值各比） | 每行各记「没变」或「变了：<键> <旧值> → <新值>」；只要有一行「变了」，这一格总记「H4 的判决行变了」 | 问题单第 1 行「并判 H4 的判决变没变」。任一判决行有一个字段不同 ⇒ 第十节「判决行变了」 |
| **两遍确定性** | 5.2 第 3 步两遍标准输出 `cmp` | 相同记「两遍逐字节相同」，否则第十一节停机（同上一次 T4） | 不对应问题单的翻面；是新产物能当登记产物的前提 |
| **回比用例** | `local_constants_equal_the_implementation_constants`，在 U1–U3 都换进主工作区之后跑（checker 档，主 agent 跑：`cargo test --release -p singlefs-checker-tier --bin e183_capacity_read_cache_reserve_and_sweep_costs -- local_constants_equal_the_implementation_constants`，范围声明、内存包装与 `run-build-directory.sh` 照 `.claude/agent-common.md`「跑编译出来的代码经内存包装」那一条拼，`SINGLEFS_HEAVY_TESTS` 由主 agent 定） | 绿记「对得上」；红记「对不上」并抄失败信息 | 问题单第 2 行。红 ⇒ 第十一节停机（同上一次 T2） |
| **逐字节用例** | `segment_one_all_output_equals_the_registered_product_byte_for_byte`，同上一行的跑法（同一条命令里两个过滤名都写上即可） | 绿记「对得上」；红记「对不上」并抄失败信息（SHA 不同还是行数不同） | 问题单第 2 行。红 ⇒ 测试构建与 release 两次跑的输出不同，第十一节停机（同上一次 T4，有隐藏状态或构建档位影响输出） |
| **A31 核对** | `python3 -B -I scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out`（纯 Python，执行员跑），标准输出落成 Y2 的核对产物 | 退 0 且末行 `mismatches=0` 记「对得上」，否则「对不上」并列不符的那几行 | 问题单第 2 行。`mismatches` ≠ 0 或退非 0 ⇒ 第十节「A31 对不上」 |
| **A31 输出差** | `diff` 旧、新两份 A31 输出 | 恰好一个 hunk `1c1`、旧行与新行只差 `product_sha256=` 的值（新值等于新产物的 `sha256sum`）记「只变产物 sha 那一行」；否则「别的行也变」并逐行列出 | 问题单第 2 行的旁证：A31 读的是读数行，「逐行差」只变第 6 行时它推的只变头一行（J10）。两行判定不一致（逐行差只变第 6 行而这里别的行也变，或反过来）⇒ 第十一节停机「两边不一致」 |

**没有登记的量**：H2、H3、H5 的任何量（这一次不跑那三段）；第一段的耗时（标准错误，不进产物，不判）。

## 七、钉绝对值的断言

互比（旧产物与新产物、release 与测试构建）旁边要有钉绝对值的：两份一起错时互比照样成立，下面这些不会。

### 7.1 出自被测条款本身的（不符时两边都查；都核过仍不符，记「条款的这一格可能错」，同上一次 G8）

| # | 断言 | 出处 |
|---|---|---|
| **N1** | 新产物第 6 行逐字是 `E7RESULT name=constant_compare constant=sweep_threshold_placeholder_rows local=365 implementation=365 equal=true` | `implementation=365`：C690（清扫触发阈值的默认值没定） 那一行「阈值默认 365」（第二节已整段抄）与 J1；`local=365`：U1；行的形态：J4 第 2178 行的格式串与问题单引的旧第 6 行（P1）只差两个数 |
| **N2** | 新产物里 `name=verdict item=constants` 那一行 `constants_compared=21 constants_mismatched=0` | 回比表 21 项（上一次登记第十二节第二条）；改后每一项本地与实现都该相等（T2 的反面） |

### 7.2 独立算出、用命令核过的（第十三节；不符照第十一节判）

| # | 断言 | 值 |
|---|---|---|
| **N3** | 旧产物 | sha256 `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe`、459 行、152434 字节、`grep -vc '^E7RESULT '` = 0（第十三节命令五）；与 J8 钉值相同 |
| **N4** | 旧 A31 输出 | sha256 `da09f8264058bfac8b39e83185141ade227b1ec3ee760d33837a423a4a453cf7`、58 行（同上） |
| **N5** | 锚点脚本与核对脚本 | `anchors_e183.py` sha256 `d44145025eda06d8a72683f64774d2914fcdd5e1bd929f03d94fd1308b3c265d`（与上一次登记第十三节命令一相同），`small_disk_anchor_check.py` sha256 `9004491d7639a08f7231e4b54bc6077ec1e52cbf3b0334f5e89726c2d273baee`（第十三节命令五） |
| **N6** | 新 A31 输出末行 | `readings_checked=56`（第一段每份读数一行，56 份：上一次登记的 7 项 × 2 状态 × 2 入口 × 2 盘，实验页第 77 行同数）、`mismatches=0` |
| **N7** | 上一次登记第七节 A30、A31、A33 与 7.1 的 K30–K32 | 同上一次登记，照旧在 `all` 的 `anchors` 项与各读数的 `reading_checks` 里判；这一次不改，值不重抄 |

## 八、轨迹与几何敏感性

**轨迹**：同上一次登记第八节 8.1（第一段被谓词消费的量是 H4 搜索里每次尝试的可用(d)，照旧逐次进产物；它们的新旧值若不同，落在「逐行差」里逐行列出）。这一次新加的量都是逐字节比较与常量，不被任何谓词消费，不报轨迹。

**几何敏感性**：不适用于这一次新加的判定——「只变那一行」是两份产物的逐字节比较，不是在某个几何取样点上写死的门槛判决，没有阈值可挪。第一段原有判决的几何取样（S ∈ {4, 8, 16}、L ∈ {1 MiB, 4 MiB}、默认盘）同上一次登记第八节 8.2，`all` 十项照旧全跑，比较覆盖全部取样点。

**射程**：阈值从 369 改到 365 只在挂载末尾实例表行数落在 366–369 时改变产品路径的行为；第一段的历史到不了这个行数（J6），所以这一次的比较对阈值本身不敏感，它答的只是「改一个本地常量之后第一段别的输出动没动」。阈值在 366–369 行上的行为由第三段（H5）的格与 `crates/singlefs-harness/tests/instance_table_sweep_at_the_end_of_a_writable_mount.rs` 那一组用例管，不归这一份。

## 九、变异

同上一次登记第九节与它第十二节各条修订列的变异行，不动。这一次**不新加变异行**，理由逐条：

- U1 改的常量只被回比用例读（J3、J4）。「把它改回 369」这条变异在改之前的树上已经真跑过一次：2026-10-30 的快档里回比用例红在「sweep_threshold_placeholder_rows：本地 369 实现 365」、逐字节用例同时红（问题单背景第 3 条，P2）。执行员在实验页「第 1 次重跑」一节把这两条红记成「改前的红」，当这两条用例分得出这一处的读数；它在唯一的取样点（回比表第 5 项、产物第 6 行）上改变输出。
- U3 的钉值只被逐字节用例读；它能抓什么由上一次登记的变异行与 2026-10-23 那次重钉时的红（实验页「做了什么」第 7 条）说明，这一次只换值。
- 改完之后变异表里每一行的原文仍要在装置里恰好命中一次（`mutation-sampling.md` 第七类）：J11 那条 `grep` 零命中，说明没有一行的原文落在 U1–U3 改的那几行上；执行员改完照门禁 code-source-discipline 的 mutation-tables 格的读法再数一遍装置那几行变异原文的命中数，写进报告。

主 agent 要一条留档的变异时（待认项），形态写死成：原文 `const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 365;`、替换文 `const SWEEP_THRESHOLD_PLACEHOLDER_ROWS: u64 = 369;`、点名 `local_constants_equal_the_implementation_constants`；它在回比表第 5 项那一个取样点上改变输出，其余 20 项不敏感（那 20 项各有自己的回比，不由这一条罩）。

## 十、失败条款

| # | 条款 | 什么观测会让它触发 | 怎么记、往下怎么走 |
|---|---|---|---|
| **F1** 别的行也变 | 常量之外，自 2026-10-23 以来别的输入改了第一段的输出（P6 推的那一句不成立） | 「逐行差」记「别的行也变」：diff 里有第 6 行以外的 hunk，或行数、字节数与 N3 不同 | 问题单第 1 行答「别的行也变」，逐个 hunk 整行列出；接着看 F2。F2 没打中（判决行全「没变」）：照 5.1 U3、5.3 W1–W5 换钉值与复跑登记，交回里把变了的行按 `name=` 分类列给主 agent。F2 打中：见 F2 |
| **F2** 判决行变了 | H4 的判决（实验页「判决」一节整行抄的 11 行）在新产物上不同 | 「判决行」那一格记「H4 的判决行变了」：11 行里任一行有一个字段不同 | 新产物照样落盘（Y2），A31 照跑；**U3 与 W1–W5 一处都不改**（钉值与复跑表仍指旧产物，两条快档用例里逐字节那一条照旧红），实验页不改结论；交回主 agent：判决行变了是 H4 结论的事，改不改实验页的判决与依它的决策，由主 agent 定（要动 `.claude/kb/decisions.md` 的照 `.claude/rules/three-way-inference.md` 走三方） |
| **F3** 第 6 行新值不对 | 新第 6 行不是 N1 | 「第 6 行新值」记「不对」 | 按字段分：`implementation=` 不是 365 走第十一节停机 S2；`local=` 不是 365 走第十一节作废 V1；两个都是 365 而别的字段不同（键名、`equal=`、行的形态）记 F1 并交回 |
| **F4** A31 对不上 | 新产物上的读数与锚点脚本现算的单元区容量或环窗口字节不符 | 「A31 核对」退非 0 或 `mismatches` ≠ 0 | 停在这一步，不换钉值与复跑登记；不符的行原样交回，两边查（锚点脚本对条款的读法、装置、`crates/`），同上一次 T3 的处置 |

失败条款只记这一次的判定，不回改上一次登记的任何判据。

## 十一、作废条款与停机条款

### 作废（装置或跑法有问题，这一次的产物不出结果，修好重跑）

| # | 条件 | 触发的观测 |
|---|---|---|
| V1 | 装置改错：换进去、或拿去跑产物的那一份装置与主工作区原来那一份差的不只是 U1、U2（产物之后再加 U3） | 5.2 第 2 步的 `diff` 多出 U1、U2 以外的行；或新第 6 行 `local=` 不是 365 |
| V2 | 二进制不是这一次的源码编的，或二进制编出之后 `singlefs-core`、`singlefs-format`、`singlefs-harness` 下有源文件改过 | 二进制 sha256 记不出来源；5.2 第 4 步 `find … -newer` 列出这三个包里的文件 |
| V3 | 跑没跑完 | 两遍里任一遍退非 0；或新产物 `grep -vc '^E7RESULT '` ≠ 0、末字节不是换行、没有 `name=done` 那一行 |
| V4 | 读不到 ≠ 读到 0（同上一次 V37） | 第六节任一格的输入读出来是空的（产物缺行、字段空、A31 输出空），那一格这一次作废、重跑，不当 0、不当「没变」 |
| V5 | 覆盖了旧产物 | 两份 2026-10-23 的旧产物 sha256 在跑完之后与 N3、N4 不同 |

### 停机（两边都查，既不作废也不当结果；`.claude/rules/implementation-first.md` 第 4 条）

| # | 条件 | 触发的观测 | 处置 |
|---|---|---|---|
| **S1** | 起跑前提不成立（同上一次 T1） | 表头四样任一 | 不开工，交回 |
| **S2** | 实现又变了：跑的那一刻 `crates/` 的默认阈值不是 365 | 新第 6 行 `implementation=` 不是 365；或开跑时 J1 那一行不在 | 停，交回主 agent 两边查（C690（清扫触发阈值的默认值没定） 那一行与 `instance_table_sweep.rs` 哪一边变了）；不改 U1 去凑 |
| **S3** | 本地常量与 `crates/` 回比不等（同上一次 T2） | N2 不成立（`constants_mismatched` ≠ 0）；或「回比用例」红 | 两边查：kb 登记位与 `crates/` 哪一边变了；不改本地常量去凑，交主 agent |
| **S4** | 确定性的量两次不同（同上一次 T4） | 「两遍确定性」不同；或「逐字节用例」在 U3 换上之后红 | 两份输出原样交回（release 两遍不同，或测试构建与 release 不同）；不重钉、不改复跑登记 |
| **S5** | 两边不一致 | 「逐行差」只变第 6 行而「A31 输出差」别的行也变，或「逐行差」里 `name=h4_reading_device` 行没变而 A31 的 `name=anchor_check` 行变了 | 停，两份 diff 原样交回；A31 读的就是那几行，两边对不上是核对脚本或产物的事，两边查 |

### 够判停机

问题单两行各自的够判：第 1 行 = 「逐行差」出了判定（「只变那一行」或「别的行也变」并逐行列出）且「判决行」出了判定；第 2 行 = 「回比用例」「逐字节用例」「A31 核对」三格都出了判定（两条用例由主 agent 跑，执行员交回时这两格写「待主 agent 跑」，命令写进「要主 agent 跑的命令」一节）。两行都够判就停；H2、H3、H5 与第一段以外的任何量不跑，也不记「够判后未跑」（不在这一份的登记里）。

### 比对对象（这一次要与之比的上一次产物；一行一个正式跑）

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out` | `b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe` | 459 | `replay.sh` 第 1811 行，驱动 `@driver_e183`，判法 exact；跑法装置 `-- all`（5.2） | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out` | 「逐行差」「第 6 行新值」「行数与字节数」「判决行」比的是这一份；换登记照 5.3 W1 |
| 2 | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23-anchor-check.out` | `da09f8264058bfac8b39e83185141ade227b1ec3ee760d33837a423a4a453cf7` | 58 | `replay.sh` 第 1812 行，驱动 `@driver_e183_anchor_check`，判法 exact；跑法第六节「A31 核对」那一行的命令 | `e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out` | 「A31 输出差」比的是这一份；换登记照 5.3 W2、W3 |

门禁 doc-experiments 的 replay-comparison-units 格读 `.claude/gate.d/replay-comparison-units.tsv` 点名的登记出处，加 `research/prompts/` 顶层以 `e<号>` 起头、有上面这个固定表头的登记。这一份的文件名不以 `e183` 起头，执行员照 5.3 W5 登记两行之前，那一格看不到这张表（不红也不判）；登记之后，它按登记出处读这一份，表里两行都要对上登记行，对不上判红。

### 文件名

派发提示点名了这一份的路径（问题单旁边）。与定义的形态 `research/prompts/e183-r1-prereg.md` 不同，有三处后果，交主 agent 定改不改名（改名要在执行员开工之前，内容不变）：

1. 执行员的写范围（`.claude/hooks/agent-write-scope.tsv`）只认 `research/prompts/e*-preregistration.md` 与 `research/prompts/e*-r*-prereg.md`，第十二节「修订」由执行员用 Edit 写进这一份时会被写闸拒。
2. 派发闸认续做时按「第 n 次重跑的登记看实验页有没有一行标题写着「第 n 次重跑」」认登记，认的是 `e<号>-r<n>` 的名字形态；这一份的名字它可能认不出是 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 的重跑。
3. replay-comparison-units 格只在 W5 登记之后才看得到这张表（上一段）。

## 十二、修订

（装置写之后、产物之前由执行员写，只许收严或补臂。）

## 十三、读过的文件与跑过的命令

### 13.1 规则、定义、问题单、上一次登记

- `.claude/agents/experiment-designer.md`（派发带的定义正文）；`.claude/agent-common.md` 整份；`.claude/singlefs-ai-sop/rules/test-discipline.md`、`.claude/rules/three-way-inference.md`、`.claude/singlefs-ai-sop/rules/evidence-discipline.md`、`.claude/rules/mutation-sampling.md`、`.claude/rules/implementation-first.md`、`.claude/rules/test-script-discipline.md` 整份。
- `research/prompts/m3-close-b19-e183-threshold-365-questions.md` 第 1–17 行。
- 上一次登记 `git show 59ce5a865^:research/prompts/e183-preregistration.md`（存到草稿目录 `e183-preregistration-archived.md`）：第 1–100、342–660 行。
- `.claude/hooks/agent-write-scope.tsv` 第 1–30 行与 `grep -n designer`（零命中）；`.claude/gate.d/replay-comparison-units.tsv` 第 1–20 行注释与前 3 行数据、`grep -n 'e183\|E183'`（零命中）；`.claude/gate.d/doc-experiments.sh` 第 2549–2569 行一带与 `cell_replay_comparison_units` 上方的说明（第 2865–2905 行）。

### 13.2 kb

- `.claude/kb/experiments/183-容量斜率读缓存小盘准入与清扫阈值的代价数.md`：`grep -n '^#'` 的节标题（第 79 行标题里带判决行的 11 个行号）、第 25–78 行（「做了什么」「复跑」）、第 613–615 行。没读「判决」「读数」「作废、失败、停机条款逐条」「单测与变异」「岔路表」「第三段」各节的正文。
- `.claude/kb/experiments-history.md` 第 5821–5900 行里的标题行与含「重跑」的行（只看 E183（容量斜率读缓存小盘准入与清扫阈值的代价数） 有没有「第 n 次重跑」）。
- `.claude/kb/checks-owed.md` `grep -n 'C690'` 命中的三行：第 359 行（C597（实例表行只增不删，删行的条件没有条款），命中在正文里）、第 360 行（C690（清扫触发阈值的默认值没定），第二节整抄）、第 785 行（C704（挂载透明不认清扫发布））。第 359、785 行里有「阈值占位 369」的旧说法，第 360 行里有第三段（H5）的结论句（P5）。

### 13.3 `crates/` 与脚本

- `crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs`：第 70–82、1840–1900、4245–4268、6850–6915、7005–7045 行；`grep -n` 常量名、钉值、`"all"`、`mount_writable(`、`name=constant_compare`；`git show HEAD:` 那一版的 `grep -n`。
- `crates/singlefs-core/src/instance_table_sweep.rs` 第 60–110 行；`git show HEAD:` 那一版 `grep -n 'pub const .*Self('`。
- `crates/singlefs-core/src/mount.rs` `grep -n 'pub fn mount_writable\b\|InstanceTableSweepConfiguration::PRODUCT'`；`crates/singlefs-core/src/admission.rs` `grep -n` 阈值（零命中）。
- `grep -rn 'DEFAULT_KEEPING_THE_ROWS_WITHIN_ONE_PAGE\|PLACEHOLDER_UNTIL_C690' crates/` 的命中行（含 `crates/mutations.tsv` 第 2266、6130、6231 行）。
- `scripts/experiment-apparatus/replay.sh` `grep -n 'e183\|E183'` 命中行（第 1811、1812、1814、3265–3294 行一带）。
- `scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py` `grep -n 'print\|argv\|sha256\|name='` 命中行。

### 13.4 跑过的命令（原样；读文件的 `grep -n` / `awk` / Read 在上面按行列过）

**命令一**（认领草稿目录）：`bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-b19-e183-365-design --purpose 'E183 第一段清扫阈值占位常量跟上 365 的重跑登记'` ⇒ rc=0。重跑登记不跑 `claim-experiment.sh`。

**命令二**（第二节）：

```
$ nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/quote-c690.md '.claude/kb/checks-owed.md~^\| C690 \|'; echo rc=$?
rc=0
  ✓ 1 段整抄进 <草稿目录>/quote-c690.md，回读逐字节一致
```

追加进这一份之后回读：`grep -Fxc -- "$(awk 'NR==360' .claude/kb/checks-owed.md)" <这一份>` ⇒ `1`。

**命令三**（第三节）：`TZ=Asia/Tokyo date +%F` ⇒ `2026-10-30`；`git diff HEAD -- crates/ | sha256sum` ⇒ `97a9244396740ad4e022964108b874b7ed50cd8359adbe990407b519fbc28275  -`；`git diff HEAD --stat -- crates/ | tail -1` ⇒ ` 182 files changed, 52682 insertions(+), 5340 deletions(-)`；`git log -1 --format='%h %ad' --date=short` ⇒ `59ce5a865 2026-10-26`。

**命令四**（J11）：`grep -n 'b4ff64f1\|SEGMENT_ONE_PINNED\|const SWEEP_THRESHOLD' crates/mutations.tsv scripts/mutation/tables/*` ⇒ 零命中；`grep -o '"all"' <装置> | wc -l` ⇒ `1`。

**命令五**（N3–N5）：

```
$ sha256sum research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out; wc -l < 同上
b4ff64f11be3b2d07137458ca66ebbeca9607111e1f62b152ecc82b41a7de6fe  research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out
459
$ sha256sum research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23-anchor-check.out; wc -l < 同上
da09f8264058bfac8b39e83185141ade227b1ec3ee760d33837a423a4a453cf7  research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23-anchor-check.out
58
$ ls -la research/results/ | grep e183   # 字节数：旧产物 152434，旧 A31 输出 19100
$ grep -vc '^E7RESULT ' research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-23.out
0
$ tail -c1 同上 | od -An -c
  \n
$ sha256sum scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/anchors_e183.py scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs/small_disk_anchor_check.py
d44145025eda06d8a72683f64774d2914fcdd5e1bd929f03d94fd1308b3c265d  …/anchors_e183.py
9004491d7639a08f7231e4b54bc6077ec1e52cbf3b0334f5e89726c2d273baee  …/small_disk_anchor_check.py
$ sha256sum <草稿目录>/e183-preregistration-archived.md
c4609ca35a24704e5d4e52b2fe92503cda2d73efe18c39d503a3d5cd4e1a7f21
```

## 十四、并行与挂钟

独立单位：一次 release 编译（装置一个 bin），之后两遍 `all` 互相独立（各从 mkfs 起、不共享状态），A31 核对只依赖新产物；两条快档用例各自独立（主 agent 跑）。
线程：编译取派发提示的线程上限（登记值 `.claude/heavy-thread-default` 是 31）；`all` 装置单线程（实验页「做了什么」第 3 条），两遍并行起占 2 条线程；A31 单线程。
串行：编译 → 两遍 `all` → 比较与 A31 → 换钉值、换进主工作区 → 两条快档用例，必须依次走：后一步吃前一步的产物（钉值取自产物、用例比的是钉值）；两遍 `all` 之间不计时，可以并行。
预计挂钟：合计 2 小时内（推的）：编译 1 次 × 20–40 分钟（上一次登记第五节 5.5 第一段的依据：本机负载高时同包编译 20–40 分钟）+ `all` 2 遍并行 × 每遍至多 10 分钟（推的：203 次尝试，每次 mkfs 加至多三次挂载、盘 4–64 MiB 内存盘，另三项默认盘）+ A31 1 次 × 秒级 + 换装置再编 1 次 × 20–40 分钟（compile-then-swap）+ 快档用例由主 agent 另编另跑，不算进执行员这一段。
预计在用核数：编译时至多 31 个核（派发的线程上限）；跑 `all` 时 2 个核；A31 与比较时 1 个核。
