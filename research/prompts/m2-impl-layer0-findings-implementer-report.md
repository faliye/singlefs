# 实现员报告：层 0 放量的发现日志（m2 收尾批「层 0 发现日志」，第二轮，交补丁）

报告时刻：2026-09-27 JST 10:58 起写（本机 UTC 01:58）。第一节先落盘，供主 agent 转给同时在改 54 号与 `layer0-shard-run.sh` 的工具实现员；后面各节做完再追加。

## 一、环境变量名与发现日志的行格式（接口，工具实现员照这一节）

### 环境变量

- `SINGLEFS_LAYER0_FINDINGS_FILE=<路径>`：发现日志写到这个文件（没有就建，有就往末尾追加）。没设：不写文件（标准输出那几行照打）。设了却是空串：枚举开跑前 panic（配错了就停，与别的 `SINGLEFS_LAYER0_*` 同一个脾气）。
- 读它的是 `crates/singlefs-harness/src/crash.rs` 的 `enumerate_layer0_in_state_slices_or_one_shard`（经它的那几条入口都认：层 0 快档、全量、续跑、分片跑一片、merge，以及不留进度文件的 `enumerate_layer0_selecting_versions` 一族）。代码里另有显式收路径的入口给用例用，不读环境变量。
- 一个进程里同一时刻只许一趟枚举写发现日志（进程内有一把锁，别的带发现日志的枚举排队）；两个进程写同一个路径不支持——54 号给每条崩溃枚举用例（与分片的每一片、merge 那一趟）各给一个路径，或者同一台机器上分片那一片与 merge 用不同路径。

### 文件：一节一趟枚举

- 文件由若干「节」组成，一趟枚举（整条流跑一遍、分片跑一片、merge 一次）追加一节；同一个进程先后跑几条流（例如树分裂那条用例七条流）就是几节，前面的节原样留着。
- 节的第一行是 `layer0_findings_begin`；跑的过程中，调用线程每按片号次序并进一片，就把这一片带来的「新签名」「跨台阶」各追加一行并 `fsync`（中途 `cat` / `tail` 读得到）。
- **这一趟跑完**：把这一节就地换成定稿（截到这一节的起点，重写 begin 行 + 每个签名一行 `layer0_finding` + 一行 `layer0_findings_summary`，`fsync`）。定稿与线程数、切法、续跑与否、单机还是 merge 无关，逐字节相同（签名按「最先那个状态的序号」编号）。
- 一节里没有 `layer0_findings_summary` 行 = 这一趟没跑完（还在跑、被杀、或 panic 判红）；它前面那些 `layer0_finding_new` / `layer0_finding_threshold` 行就是死之前找到的。续跑那一趟另起一节，读回的片按片号并进来时照样报一遍。
- `tail -f` 读的注意：跑完那一下文件被截短再写（`tail -f` 会报 file truncated 后接着读）。

### 行格式

一行一条；字段之间用**制表符**分；第一个字段是行的种类（不带 `=`），其余是 `key=value`（按第一个 `=` 切）。值里不会有制表符与换行：违例原文里的 `\` 写成 `\\`、制表符写成 `\t`、换行写成 `\n`、回车写成 `\r`，其余原样（中文照写）。

| 种类 | 字段（按次序） |
|---|---|
| `layer0_findings_begin` | `format=1`、`stream=<流名>`（续跑 / 分片 / merge 用的流名；不留进度文件的枚举写 `unnamed`）、`states=<整条流的状态数>`；分片跑一片时另加 `shard=<i>/<n>`、`shard_states=<这一片负责的状态数>` |
| `layer0_finding_new` | `finding=<号>`、`pass=…`、`violated=…`、`segment=…`、`publish=…`、`first_state=<最先那个状态的序号>`、`first_violation=<那一条违例原文>` |
| `layer0_finding_threshold` | `finding=<号>`、`pass=…`、`violated=…`、`segment=…`、`publish=…`、`states_at_least=<10、100、1000…>` |
| `layer0_finding`（定稿） | `finding=<号>`、`pass=…`、`violated=…`、`segment=…`、`publish=…`、`states=<这个签名的状态数>`、`sample_states=<最先至多 3 个状态的序号，逗号分>`、`sample_violation_1=<原文>`，有第二、三个样本时再跟 `sample_violation_2=`、`sample_violation_3=` |
| `layer0_findings_summary` | `signatures=<签名数>`、`red_states=<至少一遍判红的状态数，一个状态只算一次>`、`states=<这一节评过的状态数（分片跑一片时是这一片的）>`、`states_by_finding=<号>:<状态数>,…`（没有签名写 `none`） |

签名 = `pass` + `violated` + `segment` + `publish` 四样：

- `pass`（判红的是哪一遍）：`journal_consulted_oracle`（看 journal 那一遍恢复过 oracle，计进 `violations`）、`journal_ignored_oracle`（不看 journal 那一遍，计进 `ignored_violations`）、`pool_checker`（池级 checker）、`record_checker`（记录核对器）。一个状态几遍都红，就各进各的签名。
- `violated`（违了哪几条，逗号分）：两遍 oracle 各一个类名——`no_root_chosen`、`recovered_to_an_older_root`、`read_failed`、`no_file_where_an_older_root_has_one`、`no_file_where_this_root_has_one`、`file_read_where_this_root_has_none`、`wrong_content`；`pool_checker` 是判违例的不变量号（`I-3.1,I-7.1`，次序照 checker 的清单）；`record_checker` 是 `root_without_record`、`claimed_state_missing_unit` 里成立的那几个。
- `segment`：状态所在的段号（从 0 起，枚举那张段表的下标），最后那个「每一段都整段持久」的状态写 `all_persisted`（与 `LAYER0_PROGRESS` 的 `segments=` 同一个写法）。
- `publish`：状态归哪次发布——`instance<i>_txg<t>_root_write<写表下标>`、`after_the_last_root`、`every_write_persisted`。
- 号（`finding=`）：从 1 起，按签名最先那个状态的序号排（同一个状态上几个签名，按 pass、violated、segment、publish 排）；跑的过程中 `layer0_finding_new` 给的号与定稿里的号相同。
- 台阶：签名第一次出现打 `layer0_finding_new`（算过了 1），之后状态数跨过 10、100、1000……各打一行 `layer0_finding_threshold`；并进一片一下跨过几级就打几行。

### 标准输出（整段进 54 号的全量日志）

- `LAYER0_FINDING event=new finding=<号> pass=… violated=… segment=… publish=… first_state=<序号> first_violation=<原文，转义同上，这个字段一直到行尾、里面可以有空格>`
- `LAYER0_FINDING event=threshold finding=<号> pass=… violated=… segment=… publish=… states_at_least=<台阶>`
- 每趟枚举跑完一行：`LAYER0_FINDINGS signatures=<签名数> red_states=<红的状态数> states=<这一趟评过的状态数>`；分片跑一片时末尾另带与 `LAYER0_PARALLEL_*` 同一串 ` shard=<i>/<n> all_states=<…> all_slices=<…>`。
- 这几行与 `LAYER0_PROGRESS` 一样只在并片时打，设没设 `SINGLEFS_LAYER0_FINDINGS_FILE` 都打；`LAYER0_FINDING` 的前后次序随切法变（只有文件里的定稿逐字节相同）。`LAYER0_FINDINGS ` 不与任何已登记的 `count-line=` 前缀相撞（`admission.py` 按「前缀 + 空格」认行）。

### 续跑与分片

- 发现表随片进进度文件与分片账本（片行多一个 `findings=` 字段）；进度文件格式号 1 → 2、账本格式号 1 → 2，旧文件整份作废、从头跑（输入指纹本来也跟着 crates 变）。
- merge 那一趟读 n 份账本、按切片序号并发现表，在它自己的发现日志里写一节定稿——与单机跑同一条流的那一节逐字节相同（begin 行不带 shard 字段）。两片各自的发现日志是各自那一片的（begin 行带 `shard=`），merge 不读它们。

## 二、结论

- 做完规格表第 1–4 行：发现表按签名去重（`crash.rs`）、发现日志边跑边落盘与跑完定稿（`layer0_progress.rs`）、随进度文件与分片账本续跑与 merge、先红后改的新测试与 18 条变异。交补丁：`/tmp/claude-1000/impl-layer0-findings/patch/`。
- **补丁外另有一份 `golden-repin.patch`，动的是清单外的文件**（`crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs` 的两个钉值与文档，13 行增 10 行删）：规格要求的 `LAYER0_FINDINGS` 行与进度文件格式号 1 → 2 必然改掉它钉的「不分片时打印的行与进度文件的字节」。没放进 `crates.patch`（派发的文件清单里没有它），由主 agent 定打不打；不打，`unsharded_enumeration_prints_and_writes_byte_for_byte_what_it_did_before_sharding` 红（`crates/mutations.tsv` 里「实分一 默认不分片：不设分片开关也在 LAYER0_PARALLEL_* 行末打 shard=…」那一条点名的就是它，基线也跟着红）。
- **那条钉值在我开副本那一刻的树上已经是红的**（基线副本，我这三份文件原样）：打印行 `(65, 8e1ca03b…)` 对钉值 `(65, e4d8027a…)`，是别的会话没提交的改动带来的，不是这一件。我的重取值是在那棵树上算的：打印行 67 行 `15867e36…`、进度文件 `6a153e1e…`。与基线逐行对过：打印行只多出两行 `LAYER0_FINDINGS signatures=0 red_states=0 states=54`（第 6 行、第 65 行），进度文件去掉每行校验和、`findings=none` 与格式号之后逐字节相同，进度文件名（计划哈希 `5026593803…`）不变。别的会话若先重取了它那一份，这份补丁的上下文会对不上，照「只多两行 `LAYER0_FINDINGS`、进度文件每行多 `findings=none`、格式号 2」在它那一版上重取即可。
- 新测试文件名与派发清单不同：清单写的是 `tests/layer0_findings_log_dedupes_by_signature_and_survives_resume.rs`，规格「约束」一节又写「新测试文件名不含 layer0」，重型测试闸也按名字里的 layer0 拒跑（拒了就证不了红）。照规格起名 `tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs`（与既有的 `crash_enumeration_resumes_from_its_progress_file.rs`、`crash_enumeration_sharded_across_processes.rs` 同一族）。
- `lib.rs` 没动（`crash`、`layer0_progress` 两个模块本来就是 `pub mod`）。
- 什么现象会推翻「发现日志的定稿与线程数、切法、续跑、merge 无关」：任一条层 0 流上，同一批状态换线程数或切法（或续跑、分片 merge）跑出的发现日志定稿不逐字节相同；或 `Layer0FindingsReporter::finish` 的断言（跑的过程中报的号 = 定稿按最先那个状态排的号）在任一趟里打响。

## 三、改了什么（行号是补丁打上之后的文件）

`crates/singlefs-harness/src/crash.rs`：
- 发现表的类型：`Layer0OracleViolationKind`（1188 行，oracle 违例的七类）、`Layer0RedPass`（1249 行，判红的是哪一遍 + 违了哪几条）、`Layer0SegmentOfState`（1306 行）、`Layer0FindingSignature`（1325 行）、`LAYER0_FINDING_SAMPLES_KEPT = 3`（1332 行）、`Layer0Finding` / `Layer0Findings`（1343、1366 行）。`Layer0Findings::absorb_following_slice`（1418 行）：状态数相加、样本接在后面只留最先 3 个，后面那一片的样本序号不大于已留的就断言（片要按序号从小到大并）；`signatures_in_first_state_order`（1455 行）给号；`events_of_absorbing`（1471 行）算一片并进来会报的新签名与跨台阶。
- `Layer0Tally` 多一个字段 `findings`（1554 行），`absorb_following_slice` 拆字段时一并并。别的字段与它们的并法一处没动。
- `classified_oracle_violation_for_versions`（1754 行）：原 `oracle_violation_for_versions` 的本体，另交回违例是哪一类；`oracle_violation_for_versions` 改成调它、只取原因（原文逐字不变，三条 oracle 变异的锚点照旧恰好一次）。
- `evaluate_state_recording_findings`（2052 行）：原 `evaluate_state_for_versions` 的本体，多收一个「这个状态在按段枚举里的位置」（序号、段、发布）；有位置时把判红的每一遍按签名记进发现表，至少一遍红的状态记一个。`evaluate_state_for_versions` 调它、不给位置（手摆的状态发现表是空的，与 `states_by_publish` 同一个口径）。`evaluate_state_slice` 给位置。计数、`first_violation`、`first_ignored_violation`、`checker_first_violation` 的值与原来逐字相同。
- `RecordCheck` 多 derive `PartialOrd, Ord, Hash`（当签名的一段）；`Layer0PublishOfState::name_with_the_root_write_index`（发现日志里 `publish=` 的写法，比 `name()` 多根槽写下标：两块盘同身份的根不并成一格）。
- `Layer0FindingsReporter`（3288 行）：每并进一片之前报这一片带来的新签名与跨台阶（`LAYER0_FINDING` 行 + 发现日志追加一行），跑完打 `LAYER0_FINDINGS`、把发现日志里这一节换成定稿，并断言跑的过程中报的号与定稿的号相同。
- `enumerate_layer0_in_state_slices_or_one_shard`（3504 行）签名不变，改成读 `SINGLEFS_LAYER0_FINDINGS_FILE` 再调新入口 `enumerate_layer0_in_state_slices_or_one_shard_with_findings_log`（3542 行，多收一个 `&Layer0FindingsLog`）；并片那两处（读回的片、这一趟跑的片）各先报再并，跑完 `finish`（3901 行）。`merge_the_shard_ledgers`（3953 行）同样报、并、`finish`（4033 行）。
- 单元测试：三处 `Layer0Tally` 字面量补 `findings` 字段；新增 `absorbing_a_following_slice_adds_the_states_of_each_signature_and_keeps_its_earliest_three_samples`、`absorbing_a_slice_reports_new_signatures_by_their_first_state_and_every_threshold_crossed`。

`crates/singlefs-harness/src/layer0_progress.rs`：
- `LAYER0_FINDINGS_FILE_ENVIRONMENT_VARIABLE = "SINGLEFS_LAYER0_FINDINGS_FILE"`（54 行）；`PROGRESS_FILE_FORMAT` 1 → 2（57 行）、`SHARD_LEDGER_FORMAT` 1 → 2（1240 行）。
- 片行多一个字段 `findings`（`SLICE_LINE_KEYS` 25 项，501 行）：`findings_field`（766 行，空表 `none`，否则 `hex:` 加十六进制）、`findings_from_field`（835 行，解回来并核形状：签名的状态数 1..=判红的状态数、样本个数 = min(状态数, 3) 且序号递增、判红的状态数为 0 当且仅当没有签名、同一签名不出现两次；不对整份作废、从头跑）。
- 发现日志：`Layer0FindingsLog`（1693 行，`NotWritten` / `AppendedTo(路径)`，环境变量设了空串就停）、`Layer0FindingsLogBegin`（1731 行）、进程内一把锁（1740 行）、`Layer0FindingsLogSection`（1744 行：开节时记文件长度当起点、逐行追加并 `sync_data`、跑完 `set_len` 截回起点写定稿并 `sync_all`）、转义 `escaped_finding_text`（1824 行）与反转义、各行的拼法，定稿 `findings_log_final_lines`（2010 行）。
- 单元测试：`tally_with_every_field_set` 带上四遍各一个签名的发现表（往返测试因此覆盖新字段）；新增 `a_findings_table_whose_shape_does_not_hold_voids_the_whole_file`、`the_environment_decides_whether_the_findings_log_is_written`、`escaped_finding_text_fits_one_tab_separated_field_and_reads_back`。

新文件 `crates/singlefs-harness/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs`：三条用例，都在第一条流（mkfs → 取号 → 暖机 → A，甲二 54 个状态）上把版本表写错（暖机第二次发布 txg 2 说有文件、A 的内容说成别的），两遍恢复各红两类、分在好几段，共 13 个签名、33 个红状态（跑出来的 `LAYER0_FINDINGS signatures=13 red_states=33 states=54`）。

## 四、新测试与证红

新测试（8 条）：

| 测试 | 在哪 | 核什么 |
|---|---|---|
| `red_states_are_deduplicated_by_signature_and_the_findings_log_is_readable_while_running` | 新测试文件 | 先红后改那一条：一个与枚举器分开写的观察者逐状态重判（两遍恢复各过 `classified_oracle_violation_for_versions`、池级 checker、记录核对器；段由逐段数甲二状态数、发布由 `publish_of_each_segment` 独立算），核发现表每个签名的状态数、最先 3 个状态与原文开头、判红的状态数；核今天的计数只留一条 `first_violation`；核发现日志定稿逐行（begin、每签名一行、汇总行各字段照第一节的接口现拼，不借实现的名字函数）；核跑的过程中：观察者在下一片并进来之前读文件，最先那个状态并进来之前没有 `layer0_finding_new`、之后恰好一行，第 10 个状态并进来之后恰好一行 `states_at_least=10` |
| `the_findings_log_is_byte_identical_across_worker_threads_and_slicings` | 新测试文件 | 线程数 1、4、3、2、4 × 切法每片 1 个、1 个、7 个、整流 1 片、按线程数定：五份发现日志逐字节相同、计数（连发现表）逐项相同；同一文件跑两趟是两节定稿、前一节原样 |
| `findings_survive_a_resume_from_the_progress_file_and_a_merge_of_shard_ledgers` | 新测试文件 | 留进度文件跑完、截成前 30 片 + 半行、换 3 个线程续跑：计数（连发现表）逐项相同、发现日志逐字节相同；读回的 30 片里就有签名第一次出现；两片各写自己那一节（begin 带 `shard=i/2`、有汇总行），merge 的发现表与单机相同、定稿与单机逐字节相同 |
| `absorbing_a_following_slice_adds_the_states_of_each_signature_and_keeps_its_earliest_three_samples` | `crash.rs` 单元 | 并片：状态数相加、样本只留最先 3 个、只在一片里的签名原样、判红的状态数相加 |
| `absorbing_a_slice_reports_new_signatures_by_their_first_state_and_every_threshold_crossed` | `crash.rs` 单元 | 并片前报的事：新签名按最先状态排（与签名大小次序相反的一对）、一下跨两级报两件、早就跨过的不再报 |
| `a_findings_table_whose_shape_does_not_hold_voids_the_whole_file` | `layer0_progress.rs` 单元 | 片行里发现表形状不对（样本多于状态数、样本倒序、签名状态数多于判红数、有签名而判红数为 0）整份作废 |
| `the_environment_decides_whether_the_findings_log_is_written` | `layer0_progress.rs` 单元 | 没设不写、设了写那个路径、空串停下且点名环境变量 |
| `escaped_finding_text_fits_one_tab_separated_field_and_reads_back` | `layer0_progress.rs` 单元 | 转义之后没有制表符、换行、回车，反转义读回原样，`\` 后跟别的与末尾 `\` 解不开 |

另：`a_slice_line_reads_back_to_the_same_tally_field_by_field` 等既有往返用例经 `tally_with_every_field_set` 带上了四遍各一个签名的发现表（原文带制表符、换行、回车、反斜杠）。

「先红」：改之前这些测试用的 API（`Layer0Findings`、`enumerate_layer0_in_state_slices_or_one_shard_with_findings_log`、`Layer0FindingsLog`……）都不存在，编不过；今天的 `Layer0Tally` 只有 `first_violation` 一条（第一条用例里 `tally.violations > 1 && first_violation.is_some()` 那一句钉的就是这个现状）。真正的证红是下面的变异。

证红：`bash research/scripts/prove-red.sh --copy <副本> --memory 8G singlefs-harness <变异名…>`，18 条都证过（每条测试至少一条），都是「抓到」。每条只红点名的那一条测试（日志里 FAILED 的只有它）：

| 变异（改坏哪一行） | 红的断言 |
|---|---|
| 签名不分段：`segment: self.segment,` → `AllPersisted` | 用例 1「每个签名的状态数与最先 3 个状态，与逐状态重判的逐项相同」 |
| 判红的状态数不记：`red_states += 1` → `+= 0` | 用例 1「至少一遍判红的状态数」 |
| 不看 journal 那一遍记成看 journal 那一遍 | 用例 1「每个签名的状态数与最先 3 个状态…」 |
| oracle 读错内容归成「有文件却报没有」 | 用例 1「这份版本表让两遍恢复各红两类…」 |
| 发布不带根槽写下标（`_root_write{}` 写死 0） | 用例 1「第 1 个签名那一行」 |
| 跑的过程中不追加进发现日志 | 用例 1「最先那个状态并进来之后报了一次」 |
| 定稿不截掉跑的过程中那些行（`set_len` 取文件当前长度） | 用例 1「begin 行 + 每个签名一行 + 汇总行」 |
| 一片里的样本不限 3 个 | 用例 2「第 2 趟的发现日志与 1 个线程、每片 1 个状态那一趟逐字节相同」 |
| 定稿的号按签名排、不按最先状态排 | 用例 2 里 `crash.rs` 的断言「跑的过程中报的号就是定稿里按最先那个状态排的次序」 |
| merge 不写定稿（`finish` 换成 `drop`） | 用例 3「merge 那一趟写的定稿与单机跑同一条流的逐字节相同」 |
| 进度文件不带发现表（写 `Layer0Findings::default()`） | 用例 3「续跑之后的计数（连发现表）与一口气跑完的逐项相同」 |
| 并片时样本不截到 3 个 | 单元「状态数 2 + 4、样本 1、4 之后只接上 10…」 |
| 并片时签名的状态数不相加 | 同上 |
| 新签名不按最先那个状态排 | 单元「记录核对器最先在 25、池级 checker 在 40…」 |
| 早就跨过的台阶再报一遍（`> states_before` → `> 0`） | 同上 |
| 进度文件里样本倒序也照收（`|| !samples_ascend` → `|| false`） | 单元「发现表形状不对整份作废」 |
| 环境变量设了空串不停（`!path.is_empty()` → `true`） | 单元「设了空串要停下」 |
| 制表符、换行、回车不转义而丢掉（`text.chars()` 滤掉控制字符） | 单元「解得回原样」 |

最后一条替换了第一版的「制表符不转义」（原文、替换文里有 `\t`，门禁 33 号不收 `\n` 以外的反斜杠；那一版也证过红，换掉之后单独再证一次）。18 行全部追加在 `patch/mutations-append.tsv`，名字都以「层 0 发现日志：」起头；没有留给 59 号才证的行。

改之前编不编得过（基线副本 = 开副本那一刻的树、我这几份文件原样，把新测试文件拷进去）：
`cargo build --offline -p singlefs-harness --test crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume` 退 101，`grep -E '^error' | sort | uniq -c` 原样：

```
      1 error: could not compile `singlefs-harness` (test "crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume") due to 7 previous errors
      1 error[E0432]: unresolved imports `singlefs_harness::crash::classified_oracle_violation_for_versions`, `singlefs_harness::crash::enumerate_layer0_in_state_slices_or_one_shard_with_findings_log`, `singlefs_harness::crash::Layer0FindingSignature`, `singlefs_harness::crash::Layer0Findings`, `singlefs_harness::crash::Layer0OracleViolationKind`, `singlefs_harness::crash::Layer0RedPass`, `singlefs_harness::crash::Layer0SegmentOfState`
      1 error[E0432]: unresolved imports `singlefs_harness::layer0_progress::unescaped_finding_text`, `singlefs_harness::layer0_progress::Layer0FindingsLog`
      1 error[E0609]: no field `findings` on type `&Layer0Tally`
      4 error[E0609]: no field `findings` on type `Layer0Tally`
```

## 五、验证（副本 `/tmp/claude-1000/impl-layer0-findings/copy`，自带 target；线程上限 4，跑测试经 `run-with-memory-cap.sh 8G`）

开跑前 `ps`（2026-09-27 UTC 01:57）：没有 qemu-system / vm-bench / e152 / fio；有别的实现员副本里的 cargo build（`/tmp/claude-1000/impl-rev-a4e/…`），各用各的 target，没等锁。

基线红集（基线副本，同一批命令）：`--lib` 里 `model_comparison::tests::every_published_version_is_compared_by_content_instance_table_and_every_role_both_ways`（`test result: FAILED. 89 passed; 1 failed`，签名 `ModelDisagreement`「模型说该成、实现拒了」，`MountError::NewerStateStillUnreadableAfterOneReread`）；`crash_enumeration_sharded_across_processes` 里的钉值用例（打印行 `(65, 8e1ca03b…)` 对钉值 `(65, e4d8027a…)`）；门禁 74 号随机历史三条（见下）。都是开副本那一刻别的会话没提交的改动带来的，不在我的改动里，没修。

| 这一趟跑的 | 末尾原样 |
|---|---|
| `cargo fmt --all -- --check` | 退 0，无输出 |
| `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `check.sh` 的 `CODE_DISCIPLINE_LINTS` 七条 `-D` | 退 0；`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.43s` |
| `cargo build --offline --all-targets` | 退 0；`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 54.53s` |
| `--test crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume` | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 68.81s` |
| `--test crash_enumeration_sharded_across_processes`（打了 golden-repin 那一份） | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 37.55s` |
| `--test crash_enumeration_resumes_from_its_progress_file` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 59.69s` |
| `--lib` | `test result: FAILED. 94 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 332.14s`（红的那 1 条就是基线红集里那条；基线 89 条过，多出的 5 条是新单元测试） |
| `prove-red.sh`（18 条 + 换掉的那 1 条） | `✓ 点名 18 条：跑了 18 条，跳过 0 条，跑的都抓到了`；`✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了` |
| `git apply --check`（主工作区，交回前） | `crates.patch` 退 0；`golden-repin.patch` 退 0；`apply-writer-patch.py <patch> --dry-run`：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1198 行` |

登记给实现员的门禁阶段（在副本上跑、仓根给副本）：

| 阶段 | 退出码 | 末行原样 |
|---|---|---|
| 33 | 0（第一趟 1：我那条带 `\t` 的变异，换掉后重跑） | `  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1171 条的原文各命中源码一次；…` |
| 53 | 0 | `  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）` |
| 74 | 1 | `签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。` —— 红的三条（`crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43`、`random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`、`rolling_back_to_the_root_abandoned_by_the_crash_recovery_step_is_refused_on_the_abandoned_timeline`）在基线副本上同样三条红（`test result: FAILED. 21 passed; 3 failed; 2 ignored`），不是这一件 |
| 92 | 77（本次未跑） | `  ! /tmp/claude-1000/impl-layer0-findings/copy 不是 git 仓，本阶段跳过`（它管 kb 布局表与 checker，这一件没碰 checker） |
| 93 | 0 | `  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（…扫了 56 个 .rs…）` |
| 94 | 0 | `  ✓ checker 与实现只共享常量模块 \`singlefs-format\`（…）`（末行是它的「判不了的」说明） |
| 89 | 77（本次未跑） | `  ⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）` |

锚点：副本里 `crates/mutations.tsv` 落在 `crash.rs`、`layer0_progress.rs`、`lib.rs` 上的 129 行（原有 111 + 新增 18）原文各恰好命中一次（自写的核对脚本 `check-anchors.py`：`checked=129 bad=0`）；原有 111 行的原文我一行没改。

补丁核对：把 `crates.patch` 与 `golden-repin.patch` 打在 base 快照（与主工作区这几份文件逐字节相同，`git status` 这几份无改动）上，打出来的四份文件与副本逐字节相同。

## 六、条款没写、我照规格往下定了的（主 agent 看要不要改）

没有碰到要停下的分支（这一件不落盘、不改格式，没有「走得到的未定分支」）。下面几处规格没写死，我定了、写在这里：

1. **跑完那一节是「就地换成定稿」，不是只追加一行汇总。** 规格第 2 行写「跑完追加一行汇总」，第 4 行要「换线程数与切法，发现日志逐字节相同」。两条同时要，跑的过程中那些「跨台阶」行做不到与切法无关：一片并进来时跨过 10 的那个状态是这一片里第几个红状态，要这一片全部红状态的序号才算得出，片里只留了最先 3 个。所以跑的过程中照并片次序追加（给中途读的人看），跑完截回这一节起点、写定稿（begin + 每签名一行 + 汇总），逐字节相同的是定稿；merge 的「重写成合并后的样子」也就是写同一份定稿。`tail -f` 会看到一次截短。
2. **判红的那一遍分四遍，不是三遍。** 规格列的是「池级 checker / 不看 journal 那一遍 / 记录核对器」，没列看 journal 那一遍的 oracle（计进 `violations`、层 0 各流断言它为 0 的那一个）；我加成第四遍 `journal_consulted_oracle`。
3. **两遍 oracle 的「违例的不变量集合」取 oracle 的类。** oracle 没有不变量号；按 `oracle_violation_for_versions` 的七个分支各起一个类名（`Layer0OracleViolationKind`）。记录核对器取两条判据的名字。
4. **一个文件多节、进程内一把锁。** 一个进程里先后枚举几条流（树分裂那条用例七条流）各追加一节；同一进程里同时开两节会被截掉，所以带发现日志的枚举在进程内排队（锁中毒照开）。两个进程写同一个路径不支持，第一节接口里写了。
5. **进度文件与账本的片行里，每个签名最多带 3 个样本原文。** 全红的流上片行会变长（签名数 × 3 条原文 × 十六进制两倍）；干净的流每行只多 `findings=none`。没做「已凑满 3 个的签名不再往进度文件里写样本」那一步省空间。
6. **`LAYER0_FINDINGS` 每趟枚举都打，没红也打**（`signatures=0 red_states=0`），平时 `cargo test` 里每次枚举多一行标准输出；这就是钉值那条用例变的原因。
7. **续跑的发现日志另起一节**：被杀那一趟留下的半节（没有汇总行）不删，续跑那一节把读回的片照样报一遍、跑完写定稿。
8. 发现日志的环境变量由 `enumerate_layer0_in_state_slices_or_one_shard` 读（经它的入口都认，包括不留进度文件、没有流名的 `enumerate_layer0_selecting_versions` 一族，begin 行写 `stream=unnamed`）；另开 `..._with_findings_log` 给用例显式传，老入口的签名一个没改（名字带 layer0 的既有测试因此一行不用动）。

## 七、写过的文件

补丁（`/tmp/claude-1000/impl-layer0-findings/patch/`，`git apply --stat` 原样）：

```
 crates/singlefs-harness/src/crash.rs               |  936 +++++++++++++++++++-
 crates/singlefs-harness/src/layer0_progress.rs     |  720 +++++++++++++++
 ...log_dedupes_by_signature_and_survives_resume.rs |  879 +++++++++++++++++++
 3 files changed, 2480 insertions(+), 55 deletions(-)
 .../crash_enumeration_sharded_across_processes.rs  |   23 +++++++++++---------
 1 file changed, 13 insertions(+), 10 deletions(-)
```

- `crates.patch`：`crates/singlefs-harness/src/crash.rs`、`crates/singlefs-harness/src/layer0_progress.rs`、新文件 `crates/singlefs-harness/tests/crash_enumeration_findings_log_dedupes_by_signature_and_survives_resume.rs`。
- `golden-repin.patch`（清单外，见第二节）：`crates/singlefs-harness/tests/crash_enumeration_sharded_across_processes.rs`。
- `mutations-append.tsv`：18 行，变异名依次是「层 0 发现日志：」加 签名不分段（几段的红并成一格）／判红的状态数不记／不看 journal 那一遍记成看 journal 那一遍（两遍的红并成一格）／oracle 读错内容归成「有文件却报没有」那一类／发布不带根槽写下标（两块盘同身份的根并成一格）／跑的过程中不追加进发现日志（跑完才读得到）／定稿不截掉跑的过程中那些行（跑完的一节不是定稿）／一片里的样本不限 3 个（发现表随切法变）／定稿里的号按签名排、不按最先那个状态排（与跑的过程中报的号对不上）／merge 不写定稿／进度文件不带发现表（续跑之后读回的片没有发现）／并片时样本不截到 3 个／并片时签名的状态数不相加（取后面那一片的）／新签名不按最先那个状态排（按签名的大小次序报）／早就跨过的台阶再报一遍／进度文件里样本倒序也照收／环境变量设了空串不停／制表符、换行、回车不转义而丢掉（原文读不回）。
- `report.md`：这份报告的拷贝。
- `crates/lib.rs` 没动；主工作区一个字没写（交补丁）。主工作区 `git diff --stat -- crates litmus` 里没有我的改动，不贴（全是别的会话的）。

## 八、没做什么

- 没走三方对抗；层 0 全量与快档（名字带 layer0 的测试二进制、54 号）、QEMU、herd7、crates 变异整表（59 号）都没跑，归 `crash-verifier` / 提交时的整轮门禁。名字带 layer0 的既有测试没改、没跑：它们的计数断言我逐个 grep 过，只有同一流两种切法的整份 `Layer0Tally` 相等（`second_transaction_step_zero_layer0.rs` 的 `one_state_slices_tally == one_slice_tally`），发现表的并法与切法无关、不会让它变；没有钉标准输出或进度文件字节的。
- 没提交。门禁 92、89 号 77（本次未跑，原因见第五节）；74 号红与基线同样三条，没修；`--lib` 的 model_comparison 那条基线红没修。
- 没加的（条款没写的不是分支）：发现日志 panic 时追加一行「中断」（没有汇总行就是没跑完）；`Layer0Findings::absorb_following_slice` 是 `pub`（`Layer0Tally` 那个是私有的），没收窄。
- 草稿与删除：副本 `/tmp/claude-1000/impl-layer0-findings/copy`、基线副本 `/tmp/claude-1000/impl-layer0-findings/baseline` 交回前删（大小写在交回里）；我跑钉值用例留下的四个 `/tmp/singlefs-crash-enumeration-sharded-<pid>-golden-progress` 目录（各 56K–60K）已删，`/tmp` 里另外几个同名目录不是这一趟的，没动。

删除（交回前，UTC 2026-09-27T03:00:18Z）：`/tmp/claude-1000/impl-layer0-findings/copy`（17G，含它的 target 与 prove-red-logs）、`/tmp/claude-1000/impl-layer0-findings/baseline`（3.3G）已 rm -rf。留下的 `base/` 是补丁的底（四份源文件快照）与各趟日志，共 1.8M。
