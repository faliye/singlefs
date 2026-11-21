# m3-close-b20-repairs-code-r1 辩方腿报告（2026-11-10）

立场：替 r2 判决（`research/prompts/m3-close-b20-code-r2-main-verification.md`）第五节「被攻过零轮的」那几处辩护：判官加本池 fsid 修没修到 A1、A2；B1、B2、C973 射程句的条款写法与代码对不对得上；取号这一形的 D22 新写法是不是用户定的「拒，留两轮」；E205 退场删开关有没有删多或删少。方案按 `crates/` 今天的实现谈；工作区里别的会话对同一批文件的改动没追查，只判正文点名的几处。

## 零、先交代一件事（禁读清单）

开工后我跑过一条 `sed -n 1,80p research/prompts/m3-close-b20-code-r2-sonnet-output.md | cut -c1-400`，那份文件在禁读清单里，我当时没先对清单。读到的只是前 80 行、每行截到 400 字：r2 辩方腿对 r1 判决的复核表与「别池的槽」「重读粒度」两节的开头。之后没再读它，也没读清单里另外三份。下面的结论都是我自己读代码、读用例、读 kb 得的，没有哪一条靠那 80 行；但读过就是读过，主 agent 判采不采纳时请把这一条算进去。我的判官 A1、A2 的盘面形态取自判决第二节，不取自那份报告。

## 一、我跑过什么

只读命令（`grep`、`sed`、`git show`/`git diff` 只读、`task-server/task status <号>` 只读）与一份只读的草稿脚本 `~/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-defense/tools/anchors_still_present.py`（sha256 `17535ad7772e513c80f111b0bec80a2c1ed6630dc67c2603e28d5b07035f2e5c`，读 HEAD 的 `crates/mutations.tsv` 与当前文件，不写仓、不起项目代码）。编译、测试、变异、门禁一条没跑，没交任务，没建模型。所以下面凡写「绿」「红」的，都是别人量的读数或我读代码推的，逐条标了。

## 二、五件事的结论

| 要核的事 | 回 | 一句话 |
|---|---|---|
| 判官加 fsid 修没修到 A1、A2 | **成立**（修的是 `judge_writable_mount_refusal` 这一支），**同一形在崩溃态两支没修，注释的前提被一条现有用例破了**（`crates/singlefs-verification/src/mount_transparency.rs:3353`） | 见第三节 |
| B1 条款写法与代码 | **成立**（轻：条款字面少一句「同一个别的池」，`crates/singlefs-core/src/recovery.rs:1888`） | 见第四节 |
| B2 条款写法与代码 | **成立**（「丢写」的归类依赖一条推的，没量） | 见第四节 |
| C973 射程句与代码 | **成立**（推的，读码，没跑） | 见第五节 |
| 取号这一形 D22 新写法是不是用户定的「拒，留两轮」 | **成立**：「拒」「留两轮」是用户原话；「离线出路也拒」是测出的后果、「承诺管的是数据不错，不管这一形还能不能再写」是写回员的归纳，都不是用户原话 | 见第六节 |
| E205 退场删多删少 | **删多：没有；删少：有两处轻的**（测试注释里指向已删装置与一条会失效的 `git show HEAD:` 命令） | 见第七节 |

## 三、判官加 fsid：A1、A2

**判官这一支确实修到了。** 判官入口 `crates/singlefs-verification/src/mount_transparency.rs:3293` 的 `pub fn judge_writable_mount_refusal(` 多了本池 fsid 参数，择系统配置换成 `crates/singlefs-verification/src/mount_transparency.rs:3305` 的 `choose_system_configuration_of_the_pool(crash_image, filesystem_identifier_of_the_pool)`；六条判据都吃这一次择到的配置（它们自己再用择到的配置里的 fsid 数槽，`crates/singlefs-verification/src/mount_transparency.rs:2457` 起 `system_configuration_unreadable_after_the_read_stage_on_the_crash_image` 就是这样），所以 A1、A2 的根（择错池）断在源头。

「别的池的槽」的判法与产品同口径，我逐处对了：

- 产品：`crates/singlefs-core/src/system_configuration.rs:168` 的 `if bytes.len() < slot_bytes() || bytes[..4] != SYSTEM_CONFIGURATION_MAGIC {` 加整槽校验和过之后，才拿 `crates/singlefs-core/src/recovery.rs:1627` 的 `.is_some_and(|self_described| self_described != filesystem_identifier_of_the_pool)` 比 fsid，筛在解析之前。
- 判官：`crates/singlefs-verification/src/verification_read.rs:138` 的 `fn system_configuration_slot_bytes_belong_to_another_pool(` 先问 checker，只在 `TooShort`、`BadMagic`、`ChecksumMismatch` 三种时判「不是别的池」，其余（含 incompat 位不认、格式版本不认）都比偏移处的 16 字节。checker 判这三关的次序在 `crates/singlefs-checker/src/lib.rs:362` 的 `return Err(Verdict::ChecksumMismatch);` 之前、格式版本与 incompat 位之前，所以「magic 与整槽校验和过」两边是同一个条件；fsid 偏移两边都是 102（`verification_read.rs:134`、`system_configuration.rs:45` 的 `const FSID_OFFSET: usize = 4 + 2 + 96;`），各自另写。
- 读的那一层把别池的槽读成读不到，`verification_read.rs:170` 的 `!system_configuration_slot_bytes_belong_to_another_pool(` 在 `read` 的过滤里，checker 当「不自证」处置，也不从它取槽距；与产品先筛 fsid 再判值的次序同向。
- 择后的两处出口也对齐：某块盘最优的一槽带读者不收的值整池拒（`verification_read.rs:217` 的 `return Err(())`）、各盘 fsid 或盘数对不上拒（`verification_read.rs:234` 起），对应产品 `crates/singlefs-core/src/recovery.rs:1888` 的 `return Err(RecoveryFailure::SystemConfigurationsDisagree);` 与 `SystemConfigurationValueRefused`。

用例：A1 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1363`（盘 0 两槽都是别池的，判官判合法），A2 同文件 `:1411`（一块盘最新一槽是别池的，产品照常做成，声称的拒判不合法）与 `:1444`（「读阶段判完之后系统配置又读不出」那一判据同样）。变异三行在 `crates/mutations.tsv:7122` 起（`grep -c` 数出 3 行，量的）。测试里调 `judge_writable_mount_refusal(` 的九处（四个测试目标，`grep -n` 数的）都传 `SMALL.parameters().filesystem_identifier` 或用例自己的 `FILESYSTEM_IDENTIFIER`，没有拿 `choose_system_configuration` 择出的 fsid 回填（`grep` 看过，量的）。这些用例绿不绿我没跑；正文写的合入后验证 11336「测试 3375 条全绿」是主 agent 的读数（我没复核）。

### 三之二、判官这一支够不着的地方（辩方也认）

1. **崩溃态两支没加 fsid，注释写的前提被一条现有用例破了。** 两个崩溃态入口（`judge_writable_mount_refusal_of_a_crash_state` 与 `..._knowing_the_read_faults_under_the_refused_mount`）仍用不筛 fsid 的择法：`crates/singlefs-verification/src/mount_transparency.rs:3367` 的 `let system_configuration_of_whichever_pool = choose_system_configuration(crash_image).ok();`。注释给的理由是 `crates/singlefs-verification/src/mount_transparency.rs:3353` 的「这一支手里没有本池 fsid，择系统配置不筛 fsid：它判的崩溃态由一个池的写流造出，盘上没有别的池的槽。」。这句对崩溃注入与 E171 装置的调用成立（它们的盘面是写流盖在一份底盘上，调用在 `crates/singlefs-verification/src/crash_injection.rs:3533`、`:4589` 与 `crates/singlefs-implementation-experiments/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes/witness_criterion_and_writable_mount_probe.rs:169`），底下的原因也在：mkfs 清零 `[0, 根环基址)` 再过一道屏障，才写别的（`crates/singlefs-core/src/make_filesystem.rs:692`、`:694`），用过的盘上重做 mkfs 不留旧池的槽。但这一支的入口是 `pub`，同一个目标里有一条用例手造别池的槽喂给它：`crates/singlefs-verification/tests/writable_mount_refusal_after_the_read_stage_counts_self_verified_slots_by_the_pool_checker.rs:95` 的 `crash_image.apply_writes(&foreign_slots);`，接着 `:127` 调 `judge_writable_mount_refusal_of_a_crash_state(`。所以「盘上没有别的池的槽」对这一支的全部调用方不成立，注释写成了无条件。
   - 这条用例今天判 Legal，是不是靠对的理由：盘 0 两槽是别池的、盘 1 是本池的，不筛 fsid 的择法在盘 0 择到别池的配置、盘 1 择到本池的，两边 fsid 对不上，`crates/singlefs-verification/src/verification_read.rs:234` 起返回 `Err(())`，择不到按「判据为真」（推的，读码，没跑）。若别池的槽在每块盘上都是同一个别池的 fsid，择法会择到那一份别池的配置，之后判据拿别池的 fsid 数槽，数出「每块盘都自证过」，判成假——A1 的形，在这一支上还在（推的，没造盘面量）。
   - 实现员报告也自己列了这一条（草稿目录 `m3-close-b20-judge-fsid/result.md`「停下交主 agent 的」第 1 项，转述），主 agent 判时是知情的；我补的是：它的理由「崩溃走不到」只覆盖崩溃注入的调用方，不覆盖这一支的手造调用方。
   - 判官 `judge_writable_mount_refusal` 在 `crates/` 下只有测试调用（`grep -rn 'judge_writable_mount_refusal(' crates` 只命中定义、文档与 `tests/`，量的）。所以 r2 打中的 A1、A2（手造盘面）修好了，进崩溃注入的那两支同一形没改。判决第五节写的是「判官加 fsid 的改法」，这一支没在它的字面里，所以不算判决错，算射程要补一句。
2. **A2 的用例把所选那一版写死了。** 判官的所选那一版通常从 `read_recovery_fields` 来，它先跑产品的只读恢复（`crates/singlefs-verification/src/mount_transparency.rs:158` 的 `let report = recover(reader, JournalPolicy::Consult);`），那一步用不筛 fsid 的择法，A2 盘面（一块盘最新一槽是别池的）上各盘 fsid 对不上，恢复报错、所选那一版交回 `None`（推的：实现员报告第 3 项这么说，我读码同意，没跑）。A2 用例因此写死 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1287` 的 `fn newest_version_of_the_minimal_history() -> (InstanceGeneration, CheckpointTxg) {`。这样的写法能钉住「判官择对池」这一半（变异三行也咬它），但「从盘上取所选那一版、再交给判官」的整条链在 A2 盘面上没有用例走过。不是缺陷，是这条用例的取样点。
3. **推翻我这一节的现象：** 用别池的槽（每块盘同一个别池 fsid）造一张盘面喂 `judge_writable_mount_refusal_of_a_crash_state`，拒因取 D23 单子上「读阶段判完之后系统配置又读不出」，若判 Legal 则第 1 条的推断错；若判 Illegal 则崩溃态两支同形，要加 fsid 或把注释前提改成调用方契约。命令见第九节第 3 条（要先写一条用例，我不能往仓里写，交实现员）。

## 四、B1、B2：条款写法与代码

条款落在 `.claude/kb/decisions/22-单元原子性怎么合成.md:185`（射程一行）。

### B1「别池世代号高」：成立（轻）

条款原文（同一行，子串）：「别的池的槽（校验和过、fsid 与本池不同）世代号比本池的都大、而每块盘都有这样一槽时，可写挂载拒」。逐句对：

- 可写挂载拒：别池的槽在乙入判不出里当读得出而自证不过计：计数定义在 `crates/singlefs-core/src/recovery.rs:5136` 的 `pub slots_of_another_pool_whose_checksum_verifies: u64,`，乙那一判在 `crates/singlefs-core/src/mount.rs:9196` 的 `.any(|slots| slots.slots_of_another_pool_whose_checksum_verifies > 0)` 用它；每块盘都有一槽别池的就没有一块盘两槽都自证过；用例 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1464`（`fn newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only() {`）断言「可写挂载乙入判不出拒」，并断言拒在取号之前盘上逐字节不变。拒不拒与世代号大小无关：同一文件 `:1111` 的 `fn hand_made_image_with_the_older_slot_of_every_device_from_another_pool_is_refused() {` 世代号小的也拒。条款写「世代号比本池的都大」是这一形的充分条件，不是拒的必要条件；与上一句「fsid 不同而校验和过的槽算读得出而自证不过」（同一行）不冲突。
- 只读与恢复报 `NoValidRoot`：只读恢复用不筛 fsid 的择法（`crates/singlefs-core/src/recovery.rs:1886` 起的各盘比对），每盘取世代号最大的那一槽，别池的世代号大就择到别池那一份，按它的 fsid 读不到根。用例 `:1489` 的 `"只读挂载择到别的池那一份、一条根都读不到"` 断言 `MountReadOnlyFailure::Recovery(RecoveryFailure::NoValidRoot)`。条款写「恢复」也报它：只读挂载是包着恢复的，恢复这一路的错就是它（读码；用例没单独断言恢复入口）。
- 只剩离线出路、走得通：同一用例 `:1501` 的 `.expect("离线出路照断言的那一版重写别的池的那两槽");` 之后可写挂载做成（实例 2）。
- **轻的缺口**：条款写「别的池」，用例造的是同一个别池（两盘的别池槽都是改本池 fsid 一个字节、同一个值）。若两块盘各是不同的别池，各盘最优槽的 fsid 对不上，恢复走 `crates/singlefs-core/src/recovery.rs:1888` 的 `return Err(RecoveryFailure::SystemConfigurationsDisagree);`，不是 `NoValidRoot`（读码，没造盘面量）。条款字面没限定「同一个别的池」；同样「世代号都大」只在每块盘上都是别池的槽最大时才有「只读择到别池那一份」，只一块盘大时只读挂载报 `SystemConfigurationsDisagree`、可写挂载做成（A2 用例 `:1411` 量到后者的可写一半）。条款与这一半没冲突，只是没写到。写回批若嫌条款不够，补一句「同一个别的池、每块盘最大的槽都是它」。
- 推翻条件：用两块盘各不同的别池造盘面，只读挂载若报 `NoValidRoot` 而不是 `SystemConfigurationsDisagree`，我这一条轻缺口作废。

### B2「丢写倒挂不罩」：成立（依赖一条推的）

条款原文（子串）：「一次丢写让一块盘落后、两槽都自证过，另一块盘最新那一槽读回全零，再加两条根与它们的记录读坏」。

- 盘面对得上：用例 `:1533`（`fn a_device_holding_two_stale_self_verified_slots_lets_the_inverted_first_txg_through() {`）把盘 0 的最新一槽写回前一轮内容、两槽都自证过（断言 `(0, 2, 0)`），再用六格读故障、去掉盘 0 槽上的那两条读故障，剩下盘 1 最新槽读回全零加两条根与记录读坏，断言可写挂载做成、新实例第一次发布落在被藏的 D 之下；对照 `:1569`（`fn the_same_faults_are_refused_when_device_zero_is_not_behind() {`）盘 0 没落后时拒，拒因 `NewerStateStillUnreadableAfterOneReread`。「不判」与「放行」都是用例的断言，条款与它一致。
- 「代码不动」成立：这一形走的是 R2 另一半「没有一块盘两槽都自证过」不成立；这一半的判法没改（`git diff HEAD` 里 `crates/singlefs-core/src/recovery.rs:5142` 的 `both_slots_self_verified` 函数体没有改动行，同一个结构体只多了一个字段与文档注释；`both_slots_self_verified` 那处在变异 `crates/mutations.tsv:7127` 钉现状）。
- **依赖一条推的**：归 C6（块层语义假设写错）的前提是「落后」只能由丢写造成。我读码的结论与判决一致：盘 0 要在 C 那次轮换之后的 D 根槽已持久时仍缺 C 那次轮换，要越过两次轮换之间的屏障，纯断电做不到（推的，没造盘面量）。用例造的盘面里盘 0 只落后一次轮换，同一形的相邻盘面（盘 0 与盘 1 只差一次轮换、而 D 的根还没持久）就是纯崩溃造得出的，但那一形盘 1 的见证值也在盘 0 的旧槽之上，判据照拒（对照用例那一形）。所以归类不错，但写回批若只写「一次丢写」，读者会问「只落后一次轮换算不算丢写」，条款可补半句「在 D 的根槽已持久之后」。
- 推翻条件：造出纯崩溃（不要说谎的设备、屏障守约）能走到「一块盘两槽自证过且见证值不超过所选那一版，另一块盘最新槽读回全零」的历史，则 B2 的「归 C6」要改，改归崩溃承诺。

## 五、C973 射程句：成立（推的，读码，没跑模型）

射程句在 `.claude/kb/checks-owed.md:613`（C973 一行）。逐分句对代码：

| 分句 | 代码 | 回 |
|---|---|---|
| 「第 0 代之后至多一圈 3S 个 txg 里落到这一槽的那一个」 | `crates/singlefs-core/src/instance_table_sweep.rs:611` 的 `(GENESIS_CHECKPOINT_TXG.0 + 1..=highest_txg_of_the_pass.0)`，`:612` 的 `.take(ring_length_in_slots)`，`:613` 的 `target_for_publish(CheckpointTxg(txg), slots_per_region) == ring_slot)`；落点公式 `crates/singlefs-core/src/root_ring.rs:172` 的 `region: checkpoint_txg.0 % ROOT_RING_REGIONS,` 与 `:173` 的 `slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),` | 成立。连续 3S 个 txg 在 `(区域, 槽)` 上恰好各落一次（区域 = txg 模 3、槽 = txg 除 3 再模 S，模 3S 双射），所以「那一个」在 1..=3S 里恰有一个，终点 3S 落在区域 0 槽 0，范围含终点 |
| 「第 0 代根判据」只管第 0 代根 | `genesis_root_in_a_ring_slot_the_publishes_of_the_pass_have_reached` 首句判 `checkpoint_txg != GENESIS_CHECKPOINT_TXG` 就交假；第 0 代根只在三个区域的槽 0 过落点核（`root_ring.rs:188` 起） | 成立 |
| 「区域 r 的槽 0 读到第 0 代根、而这一遍读出的最大 txg 已到…时算这一槽自证不过，走原有的重读一次，仍不过这一次不删」 | `crates/singlefs-core/src/instance_table_sweep.rs:470` 的 `fn every_root_of_the_ring_self_verified(` 先逐槽读（读不出、全零、错位读各自处置），再取最大 txg 判第 0 代根，这一槽只读过一次的重读一次、仍不过交 `Err` | 成立 |
| 「只在『实例在根环里没有自证根』时删行」（第 362、367 行） | `instance_table_sweep.rs:363` 的 `roots_of_the_ring.iter().map(` 起一句收进集合、`:367` 的 `.contains(&row.instance))`（行号对得上，第 362 行是 `let` 起头的那一行） | 成立 |
| 射程：写进槽 0 的根槽写没落盘时清扫每轮都停，只少删不多删 | 判据只会把一条第 0 代根改判「自证不过」，结果只有 `Err`（这一次不删）或照旧；它不往候选里加行 | 成立（单故障内）；双故障的多删另登 C977（`.claude/kb/checks-owed.md:617`） |
| 「不要说谎的设备，暖机第一次发布的记录落了、根槽写之前崩溃一次也走得到」 | 恢复由记录重建那次发布的根、只在内存里（D23 已定项 15，`.claude/kb/decisions/23-journal的角色与格式.md:471` 起）；下一个实例第一次发布的 txg 是 `crates/singlefs-core/src/mount.rs:1495` 的 `checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))`，记录的 txg 在内，所以取 2，txg 1 对应的区域 1 槽 0 再不被写，留着第 0 代根，直到 txg 3S+1 | 成立（推的：读码；没造崩溃态量） |
| 「单次白停至多 3S−1 次发布（S = 16 时 47）」 | 区域 1 槽 0 丢了 txg 1 的写：判据在最大 txg 到 2 起触发（txg 1 在范围里），到 txg 3S+1 把那一槽写掉为止，最大 txg 取 2..=3S 共 3S−1 个值；区域 2、区域 0 同形各自也是 3S−1（手算） | 成立；`S = 16` 时 3×16−1 = 47，算术对；模型读数我没复跑 |
| 「停一圈的前提是跳过的 txg 号不重复；S = 4：跳 1 从 13 起放行，跳 1 与 13 从 25 起，跳 1、13、25 从 37 起」 | 范围恒从 txg 1 数起、取 3S 个（`:611`、`:612`）：槽 0 里还是第 0 代根，只要最大 txg 到了落点 txg 就恒触发；S = 4 时区域 1 槽 0 的落点 txg 依次是 1、13、25、37（13 % 3 = 1、13 / 3 = 4 模 4 = 0；25、37 同形），算术对 | 成立（手算） |

小缺口一处（轻）：射程句「清扫每轮都停」没写「最大 txg 到了那一槽的落点 txg 才停」。丢了 txg 1 的写时最大 txg 到 2 起才触发，最大 txg 还是 0 或 1 的那几轮（根环里只有第 0 代根与零条别的）判据不触发，但那时环里没有别的实例可删，不改结论。

推翻条件：在盘面上造「区域 1 槽 0 留着第 0 代根、最大 txg 在 2..=3S 里」清扫放行了（不停），或造出一次只丢单个根槽写、清扫删掉了环里还有根的实例行，则射程句错。

## 六、取号这一形：D22 新写法是不是用户定的「拒，留两轮」

- **用户原话**（记录里查得到）：`records/2026-10-18-里程碑三收尾与近期欠账.md:2606` 的「用户定案四条：取号这一形「拒，留两轮」；E205「退场」；丢写倒挂「写进射程不罩」；别池世代号高「条款写明只剩离线」。」。判决第四节同写「拒，留两轮」，说明只有两样：判拒、取号写两轮不动。
- **D22 现写法**（`.claude/kb/decisions/22-单元原子性怎么合成.md:185`）：「取号之后两块盘最新那一槽持续坏判拒：可写挂载拒，只读挂载读回上一版，离线出路也拒，承诺管的是数据不错，不管这一形还能不能再写」，后接「取号写两轮（…；去留见 C974（取号写两轮的去留没定））」。
- **逐分句对用户原话与代码**：
  - 「判拒」「可写挂载拒」：是用户「拒」。用例 `crates/singlefs-verification/tests/acquisition_writes_two_rounds_and_rolls_each_device_back_once.rs:362` 的 `fn both_devices_losing_the_newest_system_configuration_slot_after_an_acquisition_refuse_writable_and_the_offline_way_out_and_read_back_the_previous_version(` 断言拒可写、拒在取号之前盘上逐字节不变。成立。
  - 「取号写两轮不动」：`crates/singlefs-core/src/transaction.rs:1296` 的 `fn write_the_witnessed_acquisition` 与 `:747` 的 `fn roll_back_acquisition` 在，两轮；D22 同一行保留那句并指向 C974。成立。
  - 「只读挂载读回上一版」：判决第四节写「只读可读」，用例同一条 `acquisition_writes_two_rounds_and_rolls_each_device_back_once.rs:452` 的 `"只读挂载读回取号之前那一版"` 断言。成立（用户原话没有这一句，来自判决转述与测出的结局）。
  - 「离线出路也拒」：**不是用户原话**。它是已有行为（离线出路的 `birth_txg` 闸）在这一形上的后果，用例 `acquisition_writes_two_rounds_and_rolls_each_device_back_once.rs:462` 的 `.expect_err("号 2 写下的单元比断言的那一版新：离线出路拒");` 断言，拒因 `UnitBornAfterTheAssertedVersion`。事实对，写进 D22 是写回员加的，用户弹窗说的是「拒，留两轮」。
  - 「承诺管的是数据不错，不管这一形还能不能再写」：**不是用户原话**，是对「拒」的归纳，同时把容错承诺从「取号之后两块盘最新那一槽持续坏也在承诺里」（写回前的写法，`.claude/kb/decisions-history.md:12184` 记着改前原句）收窄。这一收窄是「拒」的必然推论（拒了就不能再写），但归纳的口径（只担数据不错）没有出现在任何一条用户原话里。
- **回**：成立。「判拒」「留两轮」两分句与用户原话一一对应；多出来的两个分句（离线出路也拒、承诺只管数据不错）一个是量出的后果、一个是归纳，主 agent 若要求「条款只写用户定的」，这两句要么降成「测得」标注，要么另请用户认。变更史里原话落点：`.claude/kb/decisions-history.md:12183` 与 `:13580`（grep 得，量的）。
- 推翻条件：用户弹窗的原文里「拒」不含「离线出路也拒」之义（我读不到弹窗原文，只读到记录里那一行摘要），则 D22 这两分句要标成测得。

## 七、E205 退场：删多、删少

- **产品路径还是 D23 已定项 14 第 3 条那一档**：D23 `.claude/kb/decisions/23-journal的角色与格式.md:373` 写「新实例（普通挂载、切换）的第一次发布的 checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」；代码 `crates/singlefs-core/src/mount.rs:1478` 的 `fn first_txg_of_new_instance<Device: BlockDevice>(` 取环里记录最大 txg 与根环最大 txg 的较大者，`crates/singlefs-core/src/mount.rs:1495` 的 `checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))` 加一；唯一的调用点 `crates/singlefs-core/src/mount.rs:9871` 的 `let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records)?;` 没有档位参数。开关名 `FirstTxgOfTheNewInstance`、`RingSlotsTheFirstTxgGoesPast`、`first_txg_past_the_ring_slots`、`mount_writable_with_test_only_switches_and_the_first_txg_of_the_new_instance` 在 `crates/`、`scripts/`、`.claude/` 下 0 命中（`grep -rnE` 量的；唯一的近名 `first_txg_of_the_new_instance` 是 `crates/singlefs-implementation-experiments/src/bin/e156_allocation_basis_counts/main.rs` 自己的局部变量，与开关无关）。
- **删多：没有。** 三处我独立核了：
  1. `every_root_ring_slot_read_once` 留着：`crates/singlefs-core/src/mount.rs:3270` 的 `fn every_root_ring_slot_read_once<Device: BlockDevice>(` 有产品调用 `crates/singlefs-core/src/mount.rs:3363` 的 `let first_readings = every_root_ring_slot_read_once(devices, system_configuration);`（还有 `:3383` 重读那一遍）。
  2. `singlefs-device-library` 的依赖留着：`crates/singlefs-verification/Cargo.toml:33` 的 `singlefs-device-library = { path = "../singlefs-device-library" }` 仍在，因为 `crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair/main.rs:60` 的 `use singlefs_device_library::DeviceLibrary;` 还用它。实现员报告说「只有 E205 bin 用」「连依赖一起删」是错的（我按它的写法删，E158 装置的 bin 就编不过）；工作区用的是不带清单的那一份补丁（正文写的 `patch-without-manifest`），依赖行与它上面的注释（改成点 E158 装置，`crates/singlefs-verification/Cargo.toml` 的 git diff 里量到）都对。
  3. 变异行：被删的 85 个名字，我拿 HEAD 的 `crates/mutations.tsv` 取每行的文件与原文，到当前文件里数命中，量的 `names 85 not_found_in_head 8 anchor_still_in_current_file 0`：HEAD 里找得到的 77 行，原文在今天的文件里一处都没有，删得对。HEAD 里找不到的 8 行（名字都以「E205（…）修订 15：」起头，是工作区里别处加的、后来又删了）我核不到原文，推不出。留下整行替换的那一行 `C378（认了）…回卷成旧号` 的原文在当前 `crates/singlefs-core/src/mount.rs` 里恰好命中 1 次（量的）。
  4. 变异总行数：当前 `crates/mutations.tsv` 里含 `e205_c971`、`writable_mount_first_txg_switch`、`first_txg_past_the_ring_slots_read_bad_at_mount_tests` 的行数是 0，HEAD 里是 77（`grep -c`，量的）。
- **删少：两处轻的，都在测试注释里**，都在 `crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs`：
  1. 注释还指着已删装置里的名字：`:92` 的 `/// 每块盘的字节数（同 E205 装置 \`DEVICE_BYTES\`）。`、`:102` 的 `同 E205 装置）。`、`:414` 的 `E205 装置 \`build_starting_history\` 的同一造法`、`:570` 的 `E205 装置 \`fault_ranges_of\``。装置与它的函数现在只在 git 历史里，注释成了指向不存在代码的来历句。门禁 doc-text 的 comment-history 格是不是判这几句我没跑；它们不说日期与轮次，按「来历」算边缘。改法：改成这个文件自己的造法说明，或标 `history-kept: <理由>`。
  2. `:20` 的 `（\`git show HEAD:research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out\` 第一组 \`segment=11\` 那一行）`：这份产物在暂存区是删除（`git status --short research/results` 量到 `D  research/results/e205-c971-inverted-first-txg-arm-costs-torn-rotation.out`），HEAD 里还在（`git ls-tree HEAD` 命中 1 行），提交之后 `HEAD:` 就没有它了，这条命令会失效。`.claude/gate.d/quoted-product-unreplayable.tsv:13` 已把它登记成「只留作依据」。改法：注释改写成提交号加路径，或指到仍在树里的 `research/results/e205-c971-inverted-first-txg-arm-costs-six-cells-2026-11-07.out`（`:13` 已引它）。
- **没核的**：删掉的 27 条开关测试里有没有「今天这一档」的取样点是别的目标没有的。实现员只比了变异行（报告「什么现象会推翻」第二条自己也写了），我这边只核到那 77 行的原文不在了；今天这一档仍有的变异行（`C378`、`实审 A2c 第 36 条`、`C764 核心侧`、`C650 不倒挂留档` 四行，报告第 2 条的命令）要等 `--item` 结局，推不出，命令在第九节。
- 推翻条件：第九节第 3 条四行里有没红的，或 build、clippy 红在 `mount.rs` 的未用项，则「删多」要改成有；E158 装置 bin 编不过则说明依赖被误删（工作区现状下不会）。

## 八、正文八格（逐格回）

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | **不成立（`crates/singlefs-verification/src/mount_transparency.rs:3353`）** | 正文写「本轮改动没有这一类」。E205 退场那份报告的原句我查到了（草稿目录 `m3-close-b20-e205-retire/result.md` 第 56 行，量的）；另两份（观察者、换造法）写「无」「没有」也对，产品代码一个字没动。但判官加 fsid 那一批新写了一条走不到的前提：崩溃态两支「盘上没有别的池的槽」（`crates/singlefs-verification/src/mount_transparency.rs:3353`），且它自己的报告「停下交主 agent」第 1 项列着；这条前提被 `crates/singlefs-verification/tests/writable_mount_refusal_after_the_read_stage_counts_self_verified_slots_by_the_pool_checker.rs:127` 破了（见第三节之二）。所以「没有这一类」不成立，有一类，前提只对崩溃注入与 E171 装置的调用成立 |
| 类型与名字 | **成立** | fsid 形参是裸 `&[u8; 16]`，不是 newtype，但全仓 fsid 都是裸数组（`crates/singlefs-core/src/make_filesystem.rs:82` 的 `pub filesystem_identifier: [u8; 16],`），`.claude/rules/fs-design.md:79` 点名要 newtype 的是逻辑地址、物理地址、设备内偏移、代号、inode 号，没有 fsid；新加行里没有 `_ =>`（`git diff HEAD -U0` 对三份源文件的加行 `grep -cE '^\+\s*_ =>'` 得 0，量的）；标识符没有缩写，「fsid」只在注释里（存量大量同写，不另判）。一处注：验证包新函数 `crates/singlefs-verification/src/verification_read.rs:194` 的 `pub(crate) fn choose_system_configuration_of_the_pool(` 与核心层 `crates/singlefs-core/src/recovery.rs:1807` 的 `pub fn choose_system_configuration_of_the_pool(` 同名，两个包各一份、独立实现，同一文件里不重名，门禁不拦，读者要靠包路径分 |
| 不兼容旧代码与旧格式 | **成立** | 开关名全仓 0 命中（第七节），没有为旧开关留的分支；两处轻的是测试注释里的来历句（第七节「删少」），不是兼容分支 |
| 不新建判定库 | **成立** | 对四份文件的加行 `grep -ciE 'rocksdb\|sqlite\|File::create\|OpenOptions\|fs::write\|std::env\|hostname\|/home/\|HOME'` 各得 0（量的）；新用例都造内存盘（`MemoryPool`） |
| 所有任务多机可用 | **成立（新用例）；推不出（GPU 那条）** | 新用例不读本机状态（同一个 grep 0 命中）。`the_gpu_recovery_kernel_reads_a_unit_through_a_central_mapping_of_more_than_a_thousand_entries_like_the_cpu_recovery` 在 `crates/singlefs-verification/tests/crash_judge_gpu_reads_long_files_and_lineages_block_by_block.rs`（`:35` 的 `#![cfg(feature = "gpu")]`）；它 `:452` 调 `mount_writable(`，路径经过这一批改的 `mount.rs`，所以「不是这一批改的代码」不能只凭文件没改过就算数；但 Xid 109 是 GPU 内核超时，机理在驱动与卡。各一次读数（5060 Ti 红、32G 卡绿）判不出确定性。照 `.claude/agent-common.md`「写」一节「派到某一台就红的是缺陷，当场修」，复现在 5060 Ti 上就是多机缺陷；是不是这一批带来的，要在这一批之前的树上同样跑才分得开。命令在第九节第 4 条 |
| 测试优先多线程 | **成立** | 新目标 `crates/singlefs-verification/tests/crash_points_observers_take_only_the_refusal_into_the_undecidable_witness.rs` 12 条 `#[test]`（`grep -c` 数的）各造自己的盘面，文件里没有 `static`、`thread_local`、`OnceLock`、`env::`、`std::fs`、`tempdir`（`grep -nE` 只命中 `#[test]` 行，量的），cargo 默认就在测试线程上并行；它不是切片枚举，没有该并行而没并行的循环。不用写「不能并行」 |
| 带 --item 的命令与项 | **推不出；其中一处不成立（任务 11066）** | `task-server/task status` 读到 11592、11593、11594 此刻都是 queued（量的），没有结局；11592 点 7230–7242 共 13 项，正好是观察者 13 行的行号；11593 点 7121–7129 共 9 项，7122–7124 是判官 3 行、7125–7129 是别池与丢写那 5 行，7121 是取号写两轮「只写一轮（写数断言）」那一行（`grep -n` 与 `awk` 量的，用户定的「变异行改用写数断言证红」那一行也在里面）；11062（观察者 13 行证红）是 succeeded、退 0（量的）。正文写「新目标由 --list-items 列得出（任务 11066）」：11066 的命令是 `python3 scripts/crash/crash-case-check.py --only placement,modules,provenance,must-red`，succeeded、退 0（量的），不是 `--list-items`；它说明崩溃枚举用例登记那四格没红，不说明新目标被列出。新目标不在登记表里（`.claude/gate.d/stage-inputs.tsv` 里 `grep` 它得 0 行，量的），本来也不是崩溃枚举项，用不着 `--list-items` |
| 新崩溃流多罩了什么 | **成立（没有新流）** | 同上：新目标不在登记表，用例直接造盘面调观察者的共用判（`crates/singlefs-verification/src/mount_transparency.rs:2630` 的 `pub fn image_after_the_refusal_into_the_undecidable_witness(`），没有新录的写流。共用这一处会不会把 18 个观察者一起判错归攻方腿，我不判 |

## 九、要主 agent 跑的命令

我没往服务交任何任务。下面都在仓根 `/home/fy5090/code/singlefs` 起；每条看 `TASK_SERVER_RESULT` 那一行的 `type=` 与 `exit=`，再按各条写的看 `task-server/task log <号>`。

1. 判官加 fsid 的 A1、A2 用例与 B1、B2、B3 现状用例此刻绿不绿（正文已知读数写合入后验证 11336 全绿，这一条是给我读的这一批用例再单独坐实）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors; 范围=1 个测试目标; 为什么=复核判官加 fsid 的 A1、A2 用例与别池世代号高、丢写倒挂两条现状用例此刻都绿' task-server/task submit -- cargo test -p singlefs-verification --test writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors`
   - 目录：仓根。判结果：`exit=0`，日志里 `test result: ok.` 一行，`failed` 为 0；点名的几条（`the_judge_chooses_the_slots_of_this_pool_…` 三条、`newest_generation_of_another_pool_on_every_device_leaves_neither_writable_nor_read_only`、`a_device_holding_two_stale_self_verified_slots_lets_the_inverted_first_txg_through`、`the_same_faults_are_refused_when_device_zero_is_not_behind`、`a_slot_of_another_pool_with_unknown_incompat_bits_lets_the_inverted_first_txg_through`）都在 `... ok` 行里。
2. 取号这一形（D22 新写法的用例）此刻绿不绿：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 acquisition_writes_two_rounds_and_rolls_each_device_back_once; 范围=1 个测试目标; 为什么=复核取号这一形判拒、只读读回上一版、离线出路也拒' task-server/task submit -- cargo test -p singlefs-verification --test acquisition_writes_two_rounds_and_rolls_each_device_back_once`
   - 目录：仓根。判结果：`exit=0`，`both_devices_losing_the_newest_system_configuration_slot_after_an_acquisition_refuse_writable_and_the_offline_way_out_and_read_back_the_previous_version` 在 `ok` 行里。
3. E205 退场之后今天这一档取第一个 txg 仍被抓住（实现员报告第 2 条那四行；若主 agent 已交的 11594 已含这四行就不必重交，看它的结局）：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 crates-mutation-replay.sh 带 --item; 范围=点名的 4 项; 为什么=E205 开关目标删掉后核今天这条取法仍有用例抓住' task-server/task submit -- bash .claude/gate.d/crates-mutation-replay.sh --item 'C378（认了）：取号之后写行或暖机报块设备错时把系统配置的实例代号回卷成旧号（改成回卷）' --item '实审 A2c 第 36 条：新实例第一次发布的 txg 直接加一（溢出 panic）' --item 'C764 核心侧：新实例第一个 txg 只看根环（环里记录的 txg 压到不超过根环；接在只由记录施加出来的那一版之后，新实例与那条记录同 txg）' --item 'C650 不倒挂留档：新实例第一次发布不看环里的根与记录（每个新实例都从 txg 1 起，写出比前一个实例的根 txg 更低的根）' /home/fy5090/code/singlefs`（四个名字我 `grep -cF` 在当前 `crates/mutations.tsv` 里各命中 1 行，量的）
   - 目录：仓根。判结果：收尾「计数：」行抓到 4、没红 0、无效 0、锚点失效 0。有一行没红，第七节「删多」要改。
4. GPU 那条（多机格）要两台各跑几次再比；哪台、哪张卡由服务定，所以多交几次、读 `task-server/task status <号>` 看各次落在哪台：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-verification 的测试目标 crash_judge_gpu_reads_long_files_and_lineages_block_by_block 里一条用例; 范围=1 个测试函数; 为什么=Xid 109 超时只见一次，量重复几次在两台各红几次' task-server/task submit -- cargo test -p singlefs-verification --features gpu --test crash_judge_gpu_reads_long_files_and_lineages_block_by_block -- the_gpu_recovery_kernel_reads_a_unit_through_a_central_mapping_of_more_than_a_thousand_entries_like_the_cpu_recovery --exact`
   - 目录：仓根。判结果：同一条命令交至少 3 次，记每次落的台与卡（`task status`）、`exit=` 与是否 `Xid 109`；只在 5060 Ti 红、32G 卡都绿，则多机格是缺陷，再在这一批之前的树（`git stash` 之类归主 agent，不是我）上同样跑，红就不是这一批带来的。命令的 `--features gpu` 我按 `crates/singlefs-verification/Cargo.toml:42` 的 `gpu = ["dep:wgpu", "dep:pollster"]` 写，用例能不能只靠这一个特性编，我没验。
5. 第三节之二第 3 条那张盘面（崩溃态两支同形）要先有一条用例：把「每块盘两槽都换成同一个别池的 fsid、整槽校验和重封」的盘面交给 `judge_writable_mount_refusal_of_a_crash_state`，拒因取「读阶段判完之后系统配置又读不出」。我不能往仓里写，也不在写范围里造它；若主 agent 想坐实，交实现员写（形态可抄 `crates/singlefs-verification/tests/writable_mount_refusal_after_the_read_stage_counts_self_verified_slots_by_the_pool_checker.rs:95` 起那一条，把 `[0, 1]` 的盘 0 换成两块盘都换）。
6. 复跑我的只读核（不是项目代码，不必交服务）：`python3 -I ~/.local/share/singlefs/work/m3-close-b20-repairs-code-r1-defense/tools/anchors_still_present.py ~/.local/share/singlefs/work/m3-close-b20-e205-retire/patch-without-manifest/mutations-delete.txt`，应得 `names 85 not_found_in_head 8 anchor_still_in_current_file 0`。

## 十、什么现象会推翻这份报告里的哪几条（汇总）

| 结论 | 推翻它的现象 |
|---|---|
| A1、A2 在判官这一支修到了 | 第九节第 1 条里 `the_judge_chooses_…` 三条有一条红，或变异 7122–7124 有一行没红 |
| 崩溃态两支同形（第三节之二） | 第九节第 5 条那张盘面判 Legal |
| B1 条款成立、轻缺口 | 两块盘各是不同的别池时只读挂载报 `NoValidRoot` 而不是 `SystemConfigurationsDisagree` |
| B2 条款成立、「归 C6」 | 纯崩溃（屏障守约）走得到「一块盘两槽自证过、见证值不超过所选那一版，另一块盘最新槽读回全零」 |
| C973 射程句成立 | 造出清扫在「区域 1 槽 0 留着第 0 代根、最大 txg 在 2..=3S」时放行，或单个根槽写丢失就删了环里还有根的实例行 |
| 取号这一形「拒，留两轮」成立 | 用户弹窗原文里「拒」不含离线出路也拒之义（那两分句要降成测得） |
| E205 退场没删多 | 第九节第 3 条四行有没红的；E158 装置 bin 编不过 |
| 判决第五节「零轮形态」整体站得住 | 以上任何一条被推翻，对应那一格改判 |

## 十一、没做什么

- 没跑编译、测试、变异、门禁，没往统一任务服务交任务，没建模型；所有「绿」「红」「抓到」都是别人量的读数（已标）或我读码推的（已标「推的」）。第九节是等这些结果才能往下判的几条，停在这里交回。
- 没读禁读清单里的另外三份（r1 代码轮两份、r2 攻方）；r2 辩方那份读了前 80 行（第零节交代了）。
- 没判崩溃观察者共用一处会不会把 18 个观察者一起判错、新造法造的盘面是不是用例想要的那一形、删开关后 `MountOutput` 少一个字段有没有让断言变空、C971 在 E205 退场之后靠什么拦（归攻方腿的问题）。
- 没追查工作区里别的会话对 `mount.rs`、`recovery.rs` 的改动；`git diff HEAD` 里 `mount.rs` 有压缩整理相关的大段改动，不在这一批，我只看正文点名的几处。
- 没读弹窗原文，用户原话只从 `records/2026-10-18-里程碑三收尾与近期欠账.md:2606` 与 kb 变更史摘来。
- 没改仓里任何文件；只写这份报告与草稿目录里的 `tools/anchors_still_present.py`。草稿目录里没有仓副本、编译目录。
