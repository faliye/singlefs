# m3-batch50-code-r1 云端正推腿报告（Sonnet）

轮名 m3-batch50-code-r1（代码轮第一轮，删除线 W3 + 第 50 批合入）。立场：正文第四节「云端正推」一行——核「代码做的是不是条款说的」，对得上 / 对不上，不替主 agent 判决。
格：正文第三节 K1–K7 全部。冻结副本根 `~/.local/share/singlefs/work/m3-batch50-code-r1/tree/`，下文代码路径都是相对它的 `crates/…` 加那份文件自己的行号；kb 条款写 kb 文件自己的行号（用 `grep -nF` 现查）。
判定词只有三个：一致 / 冲突 / 规则没说。「推翻条件」写在每条结论后面。

## 零、做法与环境

- 开工前 `ps -o pid,args -u "$(id -u)"` 看负载：没有 qemu-system、vm-bench.sh、e152-file-system-benchmark、fio；有别人的 `cargo test`（pid 1286970、1317959 等，checker 档与 GPU 那一趟）与 `gate.sh`/`gpu-quiesce-run.sh` 在跑——按共用约束加 `nice -n 19` 照常跑，自己同一时刻只起一条 cargo。
- 编译目录：`~/.local/share/singlefs/work/m3-batch50-code-r1-sonnet/target` 与 `target-rel`（草稿目录，交回时按 session-wrapup 第 5 条处置，见末尾「没做什么」）。命令一律 `run-with-memory-cap.sh 10G bash capped.sh 8 cargo …`。没跑门禁阶段、checker 档测试、GPU。
- 我自己写的探针只在草稿目录的仓副本 `probe-tree/` 里（冻结副本没动）：`crates/singlefs-core/tests/probe_m3_oracle.rs`，把 `crates/` 的 `inode_tree` 逐步对拍 E176 模型的 M3 臂，结果见 K4。
- 冻结副本的 sha256 清单是主 agent 给的（`research/prompts/m3-batch50-code-r1-snapshot/crates-sha256.txt`），我没重算；我读的是 `tree/` 下的文件，`mutations.tsv` 在冻结副本里与主工作区 sha256 相同（`089fb615ae5c…`，命令见 K7）。

## 一、K1　W3 回退水位（Opt2）

### K1.1　「按实例表没被崩溃恢复抛弃」与 D23 已定项 14 是不是同一件事

条款：`.claude/kb/decisions/23-journal的角色与格式.md:385`「inode 号水位 = max(cur 那一版内存里的水位, 环里读得出、按实例表没被崩溃恢复抛弃的根的水位)」；候选集那一格 `.claude/kb/decisions/23-journal的角色与格式.md:382`「(i, T) 可选 ⟺ cur 的实例表无 i 的行，或有行 (i, Ti, Wi) 且 T ≤ Ti」。

代码：
- 回退取水位只把按 cur 的实例表没被抛弃的根交给取最大那个函数：「&ring_roots_not_abandoned_by_crash_recovery(&ring_roots, &current_table),」（`crates/singlefs-core/src/mount.rs:8120`）；再与 cur 内存里的取大：「.map_or(current.inode_number_watermark(), |ring| {」（`crates/singlefs-core/src/mount.rs:8123`）、「ring.max(current.inode_number_watermark())」（`crates/singlefs-core/src/mount.rs:8124`）。
- 「被抛弃」的判法只有一处定义，候选集那一判与水位那一判调同一个函数：「if abandoned_by_table(&target_root, current_table) {」（`crates/singlefs-core/src/mount.rs:7647`）；函数体 「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:1565`）——表里有该根实例的行且 txg 大于该行的 Ti 才算抛弃，没有行不算抛弃。与条款 382 行「无 i 的行，或 T ≤ Ti」逐字反过来相等。
- 树 ID 水位仍取环里全部读得出的根，再与 cur 取大：「let tree_identifier_watermark = ring_roots」（`crates/singlefs-core/src/mount.rs:8111`）、「.map(|root| root.tree_identifier_watermark)」（`crates/singlefs-core/src/mount.rs:8113`）、「.fold(current.root.tree_identifier_watermark, u64::max);」（`crates/singlefs-core/src/mount.rs:8114`）；这一支不经 `ring_roots_not_abandoned_by_crash_recovery`。与条款 385 行「树 ID 水位照旧 = max(cur 那一版内存里的水位, 环里读得出的全部根的水位)」一致。
- 读不出的根不算：「Ok(RebuiltVersion::WithoutFile) | Err(_) => None,」（`crates/singlefs-core/src/mount.rs:7820`）（从盘上重建那一版失败或是无文件的一版，都不进最大值）。cur 的实例表读不出先拒整次回退：「.map_err(|_unreadable_or_malformed| RollbackError::CurrentInstanceTableMalformed)?」（`crates/singlefs-core/src/mount.rs:8001`），与条款 385 行「cur 的实例表读不出时今天的代码先拒整次回退」一致。

判定：**一致**。我专找了两个方向的形：
- 被抛弃的根没被滤掉（重发号）：滤用的表是 cur 根指着的那张（`instance_table_chain_of_root`，mount.rs:8000），与候选集同表同判；被抛弃的根（同实例、txg 大于 Ti）一定被滤，没有第二条路径把环里的根算进 inode 号水位。
- 没被抛弃的根被滤掉（水位低于某个还会被选的根）：能被滤掉的只有「读不出」与「无文件」两类（mount.rs:7820），它们本来就不进候选集或没有水位；而没被抛弃的读得出的根，按 D8 `inode 号在每一条时间线上单调`，水位不超过 cur 内存里那一项（`max` 的另一边兜底）。我没造出「没被抛弃的读得出的根，水位高于 cur 内存里的」这一形——只有只供测试的压回开关（transaction.rs 的 `inode_number_watermark_source`）能造，那是被 mutations.tsv 第 54 行罩着的另一件事。

推翻条件：造出一段合法历史（不用压回开关），环里有一条按 cur 的实例表没被抛弃、读得出、带文件的根，它那一版记账里的水位大于回退那一刻 cur 内存里的水位——那样 Opt2 的 `max` 里 cur 那一项就不是下界，我的「一致」不成立；或造出一条按表判被抛弃的根进了 `ring_roots_not_abandoned_by_crash_recovery` 的输出。

### K1.2　环里读不出的根、实例表行不全、回退目标就是 cur、连续两次回退、回退之后崩溃

| 形 | 取到的水位 | 对不对 | 依据 |
|---|---|---|---|
| 环里有读不出的根 | 它不进最大值，水位 = cur 内存里的（或其余读得出的根里最大的） | 对：条款 385 行「根读不出本来就不算」 | 「Ok(RebuiltVersion::WithoutFile) | Err(_) => None,」（`crates/singlefs-core/src/mount.rs:7820`） |
| 实例表行不全（表里没有某根的实例） | 那条根不算被抛弃，进最大值 | 对：条款 382 行「cur 的实例表无 i 的行」可选 | 「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`crates/singlefs-core/src/recovery.rs:1565`） |
| 回退目标就是 cur | 没有特判，目标过候选集（不被抛弃、有文件），水位 = max(环, cur 内存) = cur 的 | 对：回退到自己，不放水位 | mount.rs:7647 起的候选集判，无「目标 ≠ cur」分支（`grep -n 'target == ' mount.rs` 零命中，见下） |
| 连续两次回退 | 第二次的 cur 是第一次发出的版本（`*current = published`），环里多了第一次回退的根，它的水位 = 第一次取到的 | 对 | 「*current = published;」（`crates/singlefs-core/src/mount.rs:8186`） |
| 回退之后崩溃 | 恢复择根，回退那条根自带一行代段 = 它的 txg 的水位行（见 K2.2），从盘上重建读得回 | 对 | recovery.rs:2932-2936 要的就是这一行 |

命令（现查）：`grep -n 'target == \|target_root == \|== current.root' crates/singlefs-core/src/mount.rs | sed -n 1,5p` 在冻结副本里的输出见下一段「命令输出」。

（K1.2 末尾说的命令输出：`grep -n 'target == \|target_root == \|== current.root' crates/singlefs-core/src/mount.rs | sed -n 1,5p` 在冻结副本里零行输出——回退没有「目标 = cur」的特判分支，走的就是一般路径。）

更正一处：K1.1「推翻条件」里写的「被 mutations.tsv 第 54 行罩着」，意思是变异表里名为「第49批 W3 Y1 Opt2：管理员回退取 inode 号水位时环里读得出的根全都算……」的那一行（`crates/mutations.tsv` 的行号我没数，按名字认），不是任何「第 54 行」。

## 二、K2　W3 台阶行与谱系

### K2.1　读者有没有把台阶行当成这一版那一行、或反过来

条款：`.claude/kb/decisions/05-快照-空间记账机制.md:53`「记账在 inode 号水位变了的每次发布给这一统计量留一行台阶行 (g, H)」、同一行「I-3.13（记账行代段等于这一版的 txg） 对台阶行同样不适用」。

台阶行与「这一版那一行」同 key 维度（统计量 12、树、设备 0xFFFF_FFFF），只差代段；写侧不另存标记，读侧按行认：代段 < 这一版 txg 的是台阶，代段 = txg 的那一行值与最后一级台阶不同才也算台阶。逐个读者：

| 读者 | 怎么认 | 对不对 |
|---|---|---|
| 核心 `inode_number_watermark()`（下一次发号起点） | 取代段最大的那一行：「.max_by_key(|entry| entry.generation)」（`crates/singlefs-core/src/transaction.rs:3059`） | 对：这一版自己那一行代段 = txg 最大，台阶都比它小 |
| 核心 `inode_number_watermark_steps()`（下一次发布照抄） | 「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < version_txg」（`crates/singlefs-core/src/transaction.rs:3082`）；这一版自己那一行值不同于最后一级才补进：「.is_none_or(|latest_step| latest_step.value != row.value)」（`crates/singlefs-core/src/transaction.rs:3093`） | 对：水位没变的发布不留台阶 |
| 写记账行 | 照抄的台阶接在这次的 15 行后面按 key 排序：「entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);」（`crates/singlefs-core/src/transaction.rs:5090`） | 对 |
| 恢复重建一版 | 要「代段 = 根 txg 的那一行」在：「if !accounting_entries.iter().any(|entry| {」（`crates/singlefs-core/src/recovery.rs:2932`）、「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation == root.checkpoint_txg」（`crates/singlefs-core/src/recovery.rs:2933`）；数行时代段 < 根 txg 的水位行不算：「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg」（`crates/singlefs-core/src/recovery.rs:4777`），其余必须恰好 3 + 6 × 盘数：「if entries_other_than_the_inode_number_watermark_steps != 3 + 6 * device_count {」（`crates/singlefs-core/src/recovery.rs:4781`） | 对（只剩台阶行的一版重建不出，是有意的，变异表有一行专罩） |
| 池级 checker I-3.13 | 同统计量、不带设备维、代段 < 版本 txg 的行跳过：「&& row_generation < accounting_version_txg;」（`crates/singlefs-checker/src/walk.rs:2264`） | 对，与 `.claude/kb/invariants.md:155`「inode 号水位那一统计量的台阶行不判」一致 |
| 池级 checker I-9.6 取「现行值」 | 行按 key 序入 map，后者盖前者，同键最后一行是代段最大的：「.insert((read_u16(row, 0), read_u32(row, 10)), read_u64(row, 22));」（`crates/singlefs-checker/src/walk.rs:2282`） | 单头镜像上对；依赖「行按 key 序到达、同 (统计量, 设备) 只有一棵树」（见下「潜在」） |
| 池级 checker I-9.30 | 另用自己的认台阶函数（按字段表另写，不用核心的）：见 walk.rs:10784-10805；水位取代段最大行：walk.rs:10831-10833 | 对 |
| ckpt_cost 记账那一项 | 下一次发布的 key 集 = 这一版代段等于 txg 的 15 行换成 txg + 1 代，加这一版的台阶：「.filter(|entry| entry.generation == version_txg)」（`crates/singlefs-core/src/admission.rs:1177`）、「.chain(version_to_build_on.inode_number_watermark_steps())」（`crates/singlefs-core/src/admission.rs:1187`） | 对 |
| GPU 判器 | 走读内核 I-3.13 跳过同一类行（crash_judge_checker.wgsl:2983 附近）；扫描内核按「代段早于最新根 txg」认台阶（crash_judge_checker_scan.wgsl:2811-2850） | 读码一致，没跑（GPU 不在我这一档） |

判定：**一致**。潜在（不是今天的缺陷，写出来给主 agent）：
- I-9.6 的「后者盖前者」只在单头（一棵 inode 树）上成立；有克隆头时同 (统计量 12, 设备 0xFFFF_FFFF) 会有两棵树的行交错，map 键里没有树 ID。代码里已有 TODO 注释点名（`crates/singlefs-checker/src/walk.rs:10819` 起），今天 `crates/` 没有克隆路径。
- GPU 扫描内核最多收 256 行水位行（`crash_judge_checker_common.wgsl:862`）；台阶行随根一直留着、从不删，一条时间线上建过 inode 的发布超过 256 次，最新根的 I-9.30 在 GPU 上就走 `UNDECIDABLE_TOO_MANY_ENTRIES`（scan.wgsl:2886-2889）交 CPU，不会误判；只是长历史上 GPU 那一判退化。

推翻条件：造出一个读者拿台阶行当这一版那一行的镜像——例如恢复在「只剩台阶行」时没报错（`recovery.rs:2932` 那句被去掉仍绿），或 I-9.6 在两个同号水位行里取到不是代段最大的那一行。

### K2.2　`recovery.rs` 要求「代段 = 根 txg 的那一行」在：合法镜像有没有被判坏

每次带树的发布都经 `accounting_entries_of_this_publish` 写池级三行（含水位行）、代段 = 这次 txg（transaction.rs:5067-5077）；`mount.rs` 回退那次发布的 `PublishPlan.txg = rollback_txg`（mount.rs:8153），走同一个记账行构造。逐形：
- 空发布 / 写行 / 暖机 / 抬 F：走 `prepare_the_row_publish…` 与 `publish_version`，同一条记账行构造，水位行每次都写。
- 回退那次：水位行代段 = 回退 txg（K1 取到的水位），台阶照抄 cur，在。
- 零根 / mkfs 之后第一版 / 树表 0 条的版：根本不进这条重建（`RebuiltVersion::WithoutFile`，mount.rs:7820 那一支），不要求这一行。
- 只剩台阶行：不是合法镜像（没有哪条发布路径写出它），判坏是对的。
- 崩溃恢复抛弃过的段：重建读的是所选根自己的树表与记账树，不看被抛弃根；抛弃根的台阶不在所选根的记账里。
判定：**一致**（没找到被判坏的合法形）。推翻条件：任一发布路径（含 `crates/singlefs-core/src/mounted_session.rs`、journal 重放施加出来的版本）写出一版记账树而其中没有代段 = 它根 txg 的水位行——`grep -n 'accounting_entries:' crates/singlefs-core/src/*.rs` 在冻结副本里只有三处（recovery.rs:2921 读、mount.rs:7914 回退照抄 cur、transaction.rs:2860 结构定义），没有第二个构造点。

### K2.3　谱系记录 24 字节与 I-9.30 ①②④

条款：`.claude/kb/decisions/18-块里携带什么信息.md:580`「记录 = T_rb 8 + T_old 8 + H_cur 8 = 24 字节」；同一行「H_cur 是 cur（回退之前的现行那一版）的 inode 号水位，从 cur 那一版读，不从发布路径交来的」；同一行「每条谱系记录上，台阶行里代段 ≤ T_rb − 1 的那几行的最大水位等于 H_cur，H_cur 为 0 也判红」。

代码：
- 记录字节次序 T_rb | T_old | H_cur：「writer.put_u64(self.inode_number_watermark_of_the_version_rolled_back_from);」（`crates/singlefs-core/src/records.rs:197`），宽与容量 「pub const LINEAGE_RECORD_BYTES: u64 = 24;」（`crates/singlefs-format/src/lib.rs:119`）、「pub const LINEAGE_CONTAINER_RECORDS: u64 = 1359;」（`crates/singlefs-format/src/lib.rs:122`），与条款的 24 与 ⌊(32768 − 136) ÷ 24⌋ = 1359 一致。
- H_cur 取自 cur：「inode_number_watermark_of_the_version_rolled_back_from: current.inode_number_watermark(),」（`crates/singlefs-core/src/mount.rs:8172`）。`current` 是调用方的现行版本；交给发布路径的 `previous` 的记账行也是 `current.accounting_entries.clone()`（mount.rs:7914），两处读到的是同一个数。没有取到别的版本水位的一形：目标版本的水位只用于 `target_version`（用户可见树），不进谱系记录。
- ④ 的实现：台阶里代段 < T_rb 的最大水位必须等于 H_cur：「.filter(|(step_txg, _)| *step_txg < record.rollback_txg)」（`crates/singlefs-checker/src/walk.rs:10905`）、「largest_step_before_the_rollback == Some(watermark_rolled_back_from),」（`crates/singlefs-checker/src/walk.rs:10910`）；`largest_step_before_the_rollback == Some(…)` 对 H_cur = 0 自然判红（最大台阶至少是 1，walk.rs:10813 的注释说明了为什么不另写一判）。

合法镜像上误不误报：我推了四种历史（连续两次回退、回退后又建 inode 再回退、崩溃恢复抛弃一段后再回退、回退到比上一条谱系记录更早的根），在每种里 `cur` 的台阶都是随根一路带来的，T_rb − 1 以前最大的那级台阶的 H 就是 cur 的水位（水位单调；cur 自己那一行要么本身是最后一级台阶，要么值等于最后一级），④ 都成立。真跑的：在冻结副本的拷贝里跑了 W3 相关的 harness 测试，原样输出（命令：`cargo test --offline -p singlefs-harness --test checker_known_bad_images --test rollback_lineage_record_and_its_checkpoint_cost_term --test inode_number_watermark_pressed_back_reissues_a_number --test accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf -- watermark lineage step`，经 `run-with-memory-cap.sh 10G` 与 `capped.sh 8`）：
```
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 24.31s     （accounting_tree_grows_a_level…）
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 197 filtered out; finished in 16.79s   （checker_known_bad_images）
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.51s      （inode_number_watermark_pressed_back…）
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.49s      （rollback_lineage_record…）
```
（完整日志 `~/.local/share/singlefs/work/m3-batch50-code-r1-sonnet/harness-w3.log`；这是拷贝上的复跑，不替主 agent 在主工作区上的那一遍。）

坏镜像抓不抓得到：①（台阶抬到水位之上）、④（H_cur 偏小 / 写 0 / 台阶少一级）各有只红 I-9.30 的坏镜像，上面 27 个里含这几条（名字见 `crates/mutations.tsv` 里「第49批 W3 I-9.30 …」几行的第六段）。

③ 恒成立（C769）这一点：读码结论是对的——台阶认法把「这一版自己那一行」要么算成最后一级台阶（H = 水位），要么它的值等于最后一级的 H，所以 `水位 > 初值 ⇒ 有一级 H ≥ 水位 ∧ g ≤ 最新根 txg` 在单张镜像上由认法保证（walk.rs:10815-10816 的注释同此）。它**不能**抓的是「历史里台阶写错但与水位、记录、谱系都自洽」那一类：例如把某一级中间台阶 (g, H) 的 g 改晚、H 不变，只要没有对象夹在 (旧 g, 新 g) 之间出生、且没有谱系记录的 T_rb 落在两者之间，①②④ 全绿。代价：这种镜像上台阶只是用来互核的证据，核不出就放过；这是 C769 自认的缺口，我没有造镜像去量，是读码的推理。推翻条件：造出一张 ① ② ④ 全绿而 I-9.6 / I-9.29 也全绿、但台阶行与真实历史对不上的镜像，且它在回退之后会让 `ring_roots_not_abandoned_by_crash_recovery` 之外的路径重发 inode 号——这样 ③ 的缺口就有实害。

判定：**一致**（④ 实现与条款逐字相符；①②④ 没找到误报的合法形；③ 缺口是已记欠账）。

### K2.4　ckpt_cost 记账那一项按这次之后的行数算：台阶跨叶、记账树长层那一次有没有少算

条款：`.claude/kb/decisions/28-挂载期承诺量.md:115`「记账树不进 Σ，单列，按每发布的节点数；」。

代码：下一次发布的 key 集 = 这一版代段等于 txg 的那组行换成 txg + 1 代、加这一版的台阶：「.filter(|entry| entry.generation == version_txg)」（`crates/singlefs-core/src/admission.rs:1177`）、「.chain(version_to_build_on.inode_number_watermark_steps())」（`crates/singlefs-core/src/admission.rs:1187`）；形状由写路径同一个规划器 `plan_the_tree_after_this_publish` 现算（admission.rs:1196-1203）；规划器拒时才退回这一版的节点数。key 集只依赖键不依赖值，所以空发布、覆盖写、建 inode、回退下一次发布的 key 集相同，不需要按「做什么」分。断言「至多比这一版多一行」在 admission.rs:1190-1195。

台阶跨叶的那一次（产品容量两块盘第 463 级台阶起 +2）：由 `accounting_tree_nodes_after_the_publish_built_on` 按这次之后的 key 集规划，会数到；harness 的 `accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf.rs` 里两条用例（上面 K2.3 的第一行 2 passed）把 ckpt_cost 那一项与真发布写出的记账树节点数逐步对过。我没有另写独立算法再比一次，这一条的「独立性」只有那条用例本身（它读 `TransactionOutput` 的真节点数、不读 ckpt_cost 的中间量）。
判定：**一致**。推翻条件：一次发布写完之后的记账树节点数大于 ckpt_cost 那一项给出的数——例如把 admission.rs:1187 的 `.chain(…inode_number_watermark_steps())` 去掉后 463 级台阶那条用例仍绿（变异表里「第49批 W3 补做 ckpt_cost」两行正是盯这个，变异表第六段点名了这条用例）。

## 三、K3　退役树与退役记录

### K3.1　登记与新池新建文件的字节：有没有一处还按 11 条、65548 读

条款：`.claude/kb/decisions/08-核心索引结构.md:316`「新池新建文件写出的 **12** 条树表条目照此表（码 1..4、6..13 各一条；中央映射树的根住根记录、不进树表」；`.claude/kb/decisions/08-核心索引结构.md:343`「退役树 65548；mkfs 时水位 65536，第一个文件版本那次之后 65549」。

代码（功能代码都已是 12 条 / 13 棵 / 65548 / 65549）：
- 格式常量：「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 12;」（`crates/singlefs-format/src/lib.rs:197`）、「pub const TREE_IDENTIFIER_RETIREMENT: u64 = 65548;」（`crates/singlefs-format/src/lib.rs:216`）、「pub const TREE_IDENTIFIER_WATERMARK_AFTER_FIRST_PUBLISH: u64 = 65549;」（`crates/singlefs-format/src/lib.rs:219`），种类码 「pub const TREE_KIND_RETIREMENT: u16 = 13;」（`crates/singlefs-core/src/records.rs:432`）。
- 树表条目 12 条、头 ID 写 inode 树的树 ID（65537 是 mkfs 水位起的 inode 树号，代码里取 `trees.inode.0`，不写字面值）：transaction.rs:8734-8743，树表升序断言 「"树表条目按树 ID 升序（D8（核心索引结构） 已定项 8 排序契约）：十三个号连号发、中央映射树不进树表，剩下十二条仍按发号次序升序"」（`crates/singlefs-core/src/transaction.rs:8749`）；树表单元声明长度 = 条目数 × 200，由 `build_index_node` 按 `tree_table_entries.len()` 算（transaction.rs:8752-8767），2400 不是字面值，在 `tree_table_holds_twelve_entries_keyed_by_tree_identifier`（core lib，我跑过：240 passed 里含它）里钉。
- 读侧：core 的发号次序表 「const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [」（`crates/singlefs-core/src/recovery.rs:2377`）、checker 的 「const TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER: [u16; 12] = [」（`crates/singlefs-checker/src/walk.rs:103`）、GPU 的 「const TREE_KINDS_OF_THE_TREE_TABLE: u32 = 12u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:141`） 与树表容量 「const C_MAX_TREES_PER_TABLE: u32 = 12u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:425`）；重建一版逐种取号含退役树：「let retirement_root_pointer = pointer_of(TREE_KIND_RETIREMENT)?;」（`crates/singlefs-core/src/recovery.rs:2845`）。

命令（冻结副本，原样）：
```
grep -rnE '11 条|十一条|十一棵|\b11 entries|; 11\]' crates/singlefs-core/src crates/singlefs-checker/src crates/singlefs-format/src
crates/singlefs-format/src/lib.rs:721:            "D18 已定项 24 第 6 条树表 7 → 11 条，退役树（判决 m3-retire-r1 第三节第 1 条）11 → 12 条"
crates/singlefs-core/src/transaction.rs:3172:    /// 树表条目的诞生 txg（十一条同一个数：树建起来那次发布）。
crates/singlefs-core/src/transaction.rs:8639:    // t8 树表单元：十一条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录�
crates/singlefs-core/src/recovery.rs:2564:    // 这一版那十二棵树各自的号照盘上读回来：十一棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里
```
（第 1 行是断言消息里讲历史的话，不算残留；后 3 行是**注释里的旧数**：「/// 树表条目的诞生 txg（十一条同一个数：树建起来那次发布）。」（`crates/singlefs-core/src/transaction.rs:3172`）、「// t8 树表单元：十一条按树 ID 升序（D8（核心索引结构） 已定项 8）；映射树的根住根记录、不进树表（D19（块指针的结构与宽度预算） 已定项 11）；」（`crates/singlefs-core/src/transaction.rs:8639`）、「// 这一版那十二棵树各自的号照盘上读回来：十一棵按树表条目的种类取条目里的树 ID，中央映射树不进树表、取根记录里」（`crates/singlefs-core/src/recovery.rs:2564`）——现在是十二条 / 十三棵树（十二棵取自树表 + 中央映射树）。只是注释，不影响行为；改法落在这三行的数字。）

判定：**一致**（功能代码没有一处还按 11 条读）；另有 3 处注释里的旧数（文档残留）。推翻条件：造出一张新池镜像，第一个文件版本之后的树表单元声明长度不是 2400 / 条目数不是 12 / 树 ID 水位不是 65549——`new_pool_file_creation_publish.rs` 与 core lib 的 `tree_table_holds_twelve_entries…` 钉着，我没另量字节。

### K3.2　死亡写侧：删空摘掉、左吸收右各写一条，借不写；死亡 txg；只追加不去重

条款：`.claude/kb/decisions/08-核心索引结构.md:169`「死亡 txg = 身份消失那次发布的 checkpoint_txg」；`.claude/kb/decisions/08-核心索引结构.md:171`「删空摘掉的容器同一次发布给它的身份写一条退役记录」。

代码：
- 一次发布里只有最后一批会删 inode 记录，且不与别的记录写同一次发布：「DeletionStepOfThePublish::FinalBatch { .. }」（`crates/singlefs-core/src/transaction.rs:5311`）、「remove_record_from_leaf_containers(&containers_before, inode)?」（`crates/singlefs-core/src/transaction.rs:5320`）。所以「同一次发布里分裂、合并、再分裂」造不出：分裂只在写记录（建 inode / 覆盖写 / 意图 / 批）那几步，合并与借只在最后一批，两类步骤互斥。
- 退役记录点名的身份来自删除函数返回的 `retired`（删空摘掉与被吸收的右片，借不进去）：「&retirement_records_of_this_publish(&inode_tree.retired_identities(), self.txg),」（`crates/singlefs-core/src/transaction.rs:5434`）；死亡 txg 取这次发布的 txg：「death_txg: checkpoint_txg,」（`crates/singlefs-core/src/retirement_tree.rs:114`）；只追加（容器树 `push`，不去重）见 K3.4。
- 条数：同一次发布最多几条退役记录？结构上等于这次删除函数的 `retired.len()`。我把 crates 的 `remove_record_from_leaf_containers` / `write_records_into_leaf_containers` 与 E176 模型的 M3 臂（研究装置里手写的独立实现，`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs`，只读 std）并排跑：40 个种子 × 9000 步，增长 / 随机删 / 顺序删 / 删一端四种阶段轮换，每一步比形态、退役集（上一版叶身份集减这一版）、重写清单与模型的内容记法。命令（草稿目录 `probe-tree/`，冻结副本的拷贝加一个 `tests/probe_m3_oracle.rs`，文件在 `research/prompts/m3-batch50-code-r1-sonnet-model/`）与原样输出：
```
PROBE_SEEDS=40 PROBE_STEPS=9000 nice -n 19 run-with-memory-cap.sh 10G capped.sh 8 cargo test --release --offline -p singlefs-core --test probe_m3_oracle -- --nocapture --test-threads=3
PROBE deletes=209019 merges=466 borrows=3006 retired_total=9685 max_retired_per_delete=1
test m3_crates_vs_e176_model ... ok
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.00s
```
  20.9 万次删除里每次删除退役记录至多 1 条（总 9 685 条，合并 466 次、借 3 006 次）。理由是个归纳不变量：任何相邻两片的和 ≥ 234（插入只往最右片追加、满了才末尾分裂，新片 1 条加上满片 233 条 = 234；删除后合并 / 借把受影响的两对都查到不满足为止），所以删一条之后至多一对刚好 233 而被吸收，吸收后幸存片 233 条，继续查不出第二对。我的探针里的 `assert` 逐步核了「相邻两片之和 > 233」「分隔 key ≤ 本片最小 key 且 > 左邻最大 key」。
- 不满足这条不变量的树（旧 M1 时代的池：每片很少几条）上一次删除可以连锁吸收很多片：我手造 135 片 × 2 条的树删中间一条（`adversarial_cascade_on_a_tree_that_breaks_the_no_adjacent_fit_invariant`）：
```
CASCADE retired=116 rewritten=1 leaves_after=19 survivor_sizes_first_few=[2, 2, 2, 2, 2, 2]
CASCADE-LEFTMOST retired=116 rewritten=1
```
  一次发布写 116 条退役记录、只重写 1 片叶（R3）。116 × 34 B 仍在一两片退役容器里，W 里「至多跨两片」成立；但同一次发布还会释放 116 片 inode 叶（各 2 槽），分配记录树那一项 `2·(K + 26)` 按「一批 K = 144 个数据单元」推的，叠上 116 片叶是否还罩得住我没推也没量。这棵树不是 `crates/` 能走到的状态（上面 20.9 万步里不变量一直成立），只在旧池或坏镜像上出现；**没有任何 checker 不变量盯「相邻两片之和 > 233」**（I-9.x 里没有这一条），所以 W 的前提只靠写路径自己守着。列入「没验」。

判定：**一致**（退役身份、条数、死亡 txg 与条款相符；借不写、吸收写、只追加）。推翻条件：造出 `crates/` 自己走得到的一串操作使一次最后一批退役 ≥ 2 片（我的探针 20.9 万次没有）；或一次发布的退役记录点名的身份与「上一版叶身份集减这一版」不相等。

### K3.3　回退照抄 cur、崩溃恢复随所选根

条款：`.claude/kb/decisions/23-journal的角色与格式.md:378`「退役树同墓碑树、谱系树，随现行那一版、不取 R_old」。

代码：回退取 cur 的退役树与索引节点：「retirement_containers: current.retirement_containers.clone(),」（`crates/singlefs-core/src/mount.rs:7929`）（与墓碑树 「tombstone_tree_index_nodes: current.tombstone_tree_index_nodes.clone(),」（`crates/singlefs-core/src/mount.rs:7923`） 同形）；退役树的三个角色不在「取回退目标」的集合里：「| TransactionUnit::RetirementNodeBelowTheRoot(_) => is_a_user_visible_unit(identity),」（`crates/singlefs-core/src/mount.rs:7565`）（经 `is_a_user_visible_unit`，退役角色返回 false，mount.rs:7530）；恢复按所选根的树表读它的退役树：「let retirement_root_pointer = pointer_of(TREE_KIND_RETIREMENT)?;」（`crates/singlefs-core/src/recovery.rs:2845`）。每个版本自带一棵退役树，被抛弃那一段里写的退役记录不在所选根那一棵里（树随根 COW），所以「被抛弃段里死掉、所选根里还活着的容器被判死」造不出；反过来「所选根里死了、而退役记录在被抛弃段」也造不出（记录与死亡在同一次发布里写）。
「回退越过一片叶的出生」「删 → 回退 → 再删」两种历史下记录与谱系的配合（死亡 txg 不在新根父链上的不算）由 checker 的 I-9.33 / I-9.34 按谱系滤，对应的 harness 用例我在拷贝上复跑过，输出：
```
test rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens ... ok
test unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens ... ok
test a_retirement_record_naming_the_live_leaf_reddens_only_the_no_live_leaf_invariant ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 213 filtered out; finished in 11.52s   （checker_known_bad_images，过滤词 rolling_back / unlinking_rolling / retirement / borrowed / lender）
```
判定：**一致**。推翻条件：造出一版 `retirement_containers` 不等于 cur（回退）或不等于所选根树表所指（恢复）的发布路径。

### K3.4　`container_tree.rs` 一次追加多条、祖先并集重写

代码：追加按给的次序逐条：装得下就 push，满了才开新片、形状随之挂上去（根满长高）：「if open_container_has_room {」（`crates/singlefs-core/src/container_tree.rs:538`）、「shape.after_a_container_is_opened_at_the_right_end(node_entries.entries_per_node());」（`crates/singlefs-core/src/container_tree.rs:559`）；重写清单每片只进一次、叶序升序：「if rewritten_containers.last() != Some(&rewritten_position) {」（`crates/singlefs-core/src/container_tree.rs:562`）；重写的码 2 节点 = 被碰到的每片容器的祖先并集（`ancestors_of_containers` 用 `BTreeSet` 去重）：「.ancestors_of_containers(&self.rewritten_containers)」（`crates/singlefs-core/src/container_tree.rs:442`）。墓碑树、谱系树仍走单条包装 `append_record_to_the_open_container`（container_tree.rs:476-496），它调同一个多条函数传一条；一条时 `records.len() = 1`，等价于旧的单条版本。
满了才关 / 根满长高 / 跨容器边界追加：一次追加多条正好跨边界的用例是 `a_full_open_container_is_closed_and_a_publish_with_two_records_can_span_two_containers` 与 `appending_several_records_rewrites_every_touched_container_and_all_their_ancestors`（变异表第 105–107 行罩它们）；core lib 240 个全过（K4 的变异跑里每次都含这一遍）。
判定：**一致**。推翻条件：墓碑树或谱系树的单条追加与改之前的字节不同——`git show HEAD:crates/singlefs-core/src/container_tree.rs` 与现在比对过 diff（diff 里 `append_record_to_the_open_container` 只是转调）；我没有对旧版逐字节跑差分，这一句靠读 diff 与 240 个 lib 用例。

### 更正（前一段 K3 里三处写错的说法，原文不改，在这里改）

1. K3.1 里「`tree_table_holds_twelve_entries_keyed_by_tree_identifier`（core lib，我跑过：240 passed 里含它）」错了：它是 harness 的用例（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1359`），不在 core lib 里。我在拷贝上补跑了这一个测试文件，原样输出：`test tree_table_holds_twelve_entries_keyed_by_tree_identifier ... ok`，`test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s`（日志 `harness-newpool.log`）。该用例钉的东西：12 条、树 ID 65536..65548 缺 65540、种类 1,2,3,4,6..13、头 ID `[65537, 65537, 0, 0, 0, 0, 0, 65537, 65537, 0, 0, 65537]`、声明长度 2400、树表头 key 区间 65536..65548、后五条根指针全零（文件第 1362–1406 行）。
2. K3.4 里「变异表第 105–107 行」是我自己给新增行排的序号，不是 `crates/mutations.tsv` 的行号；该按名字认：变异表里名字以「第 50 批 容器树多条追加…」「第 50 批 退役树：开放容器满了不关…」「第 50 批 退役树：根满不长高…」开头的几行。
3. K3.4 里「`git show HEAD:…` 比对过 diff」改成实情：我读了 `research/prompts/_m3-batch50-code-r1-diff.md` 里 container_tree.rs 的那一段（diff 第 3805–4159 行），旧的 `append_record_to_the_open_container` 是「有位置就 push、没位置就开新片、形状挂上去、重写清单 = 最右一片」，新的多条版对一条记录逐步等价，单条包装转调它；没有跑旧版对新版的字节差分。core lib 基线我补跑了（release，`cargo test --release --offline -p singlefs-core --lib`）：`test result: ok. 240 passed; 0 failed`，其中含 container_tree / retirement_tree 的用例 13 个（`lib-base.log`）。

## 四、K4　M3 合并、均分借、N1、R3

### K4.1　合并与借的判定

条款：`.claude/kb/decisions/08-核心索引结构.md:171`「左吸收右与左从右借两样都做」；同一行「左从右借时借的条数取均分（借完两片条数相差不超过 1），右片保留身份、不写退役记录，它的分隔 key 改成它这一版的最小 key」。≤ 233 / < 117 / 先右后左 / 借完从借出那片接着看这几个细节**条款（kb）没写**，来自 E176 的 M3 臂：「三个候选：M1 只删空即摘（`crates/` 今天）；M2 只做左吸收右（相邻两片合起来 ≤ 233 条就并）；M3 照 D8（核心索引结构） 已定项 6 原文，左吸收右加左从右借（主取样点下限 f = 117、均分借）。装置是独立手写模」（`.claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md:7`） 与实现员报告第 4 条 「4. **M3**：照 E176 M3 那一臂：删空摘掉；先查右邻、再查左邻，两片之和 ≤ 233 左吸收右（保留左半身份与分隔 key），并完接着查；一次没并、少于 117 条且有右邻就借 ⌊(右 − 左) ÷ 2⌋ 条最小的（右片保留身份，分隔 key 改成它这一版最小 key，N1），借完从借」（`research/prompts/m3-batch50-m3-implementer-report.md:63`）。

逐项对 `crates/singlefs-core/src/inode_tree.rs`（与 E176 模型 `scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs` 并排）：

| 细节 | 代码 | E176 模型 | 判 |
|---|---|---|---|
| 两片之和 ≤ 233 才并 | 「<= INODE_LEAF_RECORDS」（`crates/singlefs-core/src/inode_tree.rs:432`） | 模型 pair_fits_in_one_container：`<= LEAF_RECORD_CAPACITY` | 一致 |
| 下限 f = ⌈233 ÷ 2⌉ = 117，**少于**才借 | 「usize::try_from(INODE_LEAF_RECORDS.div_ceil(2)).expect("117")」（`crates/singlefs-core/src/inode_tree.rs:196`）；「if records_in_the_borrower >= records_below_which_a_leaf_borrows_from_its_right_neighbour()」（`crates/singlefs-core/src/inode_tree.rs:491`） | 「if records_in_the_borrower >= floor {」（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs:254`） | 一致 |
| 借 ⌊(右 − 左) ÷ 2⌋ | 「.map_or(0, |difference| difference / 2);」（`crates/singlefs-core/src/inode_tree.rs:498`） | 「BorrowAmount::HalfTheDifference => (records_in_the_lender - records_in_the_borrower) / 2,」（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs:259`） | 一致 |
| 先右后左 | 「if right_pair_fits {」（`crates/singlefs-core/src/inode_tree.rs:466`） | 「NeighbourCheckOrder::RightPairFirst if right_fits => Some(PairToAbsorb::WithRightNeighbour),」（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs:221`） | 一致；**规则没说**这一条，也没有任何现有测试钉它（见 K7） |
| 借完从借出那片接着看 | 「self.absorb_until_no_pair_fits(lender_position);」（`crates/singlefs-core/src/inode_tree.rs:520`） | 「self.merge_until_no_pair_fits(lender_container_number, order, changes);」（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs:273`） | 一致 |
| 这趟并过就不借 | 「if !absorbed_at_least_once {」（`crates/singlefs-core/src/inode_tree.rs:389`） | 「if !merged {」（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs:312`） | 一致（在没有「相邻两片之和 ≤ 233」的树上才有区别，见下） |
| 借时右片分隔 key 取它这一版的最小 key | 「lender.contents.separator_key_in_the_parent_entry = lender」（`crates/singlefs-core/src/inode_tree.rs:510`） 起 510–515 | （模型不存分隔 key） | 一致于 kb（D8 第 164 行） |

「有没有一形留下低于下限又没处理的叶」：有，而且是设计：借只在「删了一条的那一片、没并过、有右邻」时触发，最右那一片（没有右邻）不借，所以最右片可以一直低于 117；这与 E176 M3 臂一致，也是条款里「右吸收左或右从左借会破记录号 ≥ 容器号」的直接后果。其余各片：我的探针 20.9 万次删除里每步核「相邻两片之和 > 233」「分隔 key ≤ 本片最小 key 且 > 左邻最大 key」「容器号沿叶序严格增」「记录号 ≥ 容器号」，没有一步破。

真跑的（命令与输出见 K3.2，同一条探针）：crates 与 E176 M3 臂逐步对拍 40 种子 × 9000 步，每步 `shape`（容器号、记录数、最小号、最大号）逐片相同、退役数相同、重写清单相同。再加一条定向形（三片各 117 条、从中间片删一条，左右两对同时装得下）：
```
DIRECTED model(right-first)=[(1, 117), (234, 233)] crates=[(1, 117), (234, 233)]
test directed_three_117_both_pairs_fit_after_one_delete ... ok
```
判定：**一致**。推翻条件：造出一串 `crates/` 操作使某一步的形态与 E176 M3 臂不同（探针已覆盖随机与定向各一形），或使相邻两片之和 ≤ 233 的一对在一次删除之后没被并掉。

### K4.2　R3：挪了位置没变内容的片照抄；删空最左一片新写的槽数

代码：「if let InodeLeafContainerOrigin::CarriedFrom(previous_index) = origin {」（`crates/singlefs-core/src/transaction.rs:7568`） 起，照抄的片拿上一版叶序 `previous_index` 上的指针（transaction.rs:7569-7578），映射 key 同取上一版那个叶序（transaction.rs:5777-5781）；只有 `is_changed_this_publish` 的片进重写清单（inode_tree.rs:529-538）。

- 「内容其实变了却被照抄」：探针每步断言「重写清单 = 与上一版同身份那片相比记录或分隔 key 变了的那几片」，且每一片 `CarriedFrom(上一版叶序)` 的内容与上一版那一片逐项相等，20.9 万次删除全过。
- 「身份变了、映射 key 不对」：我在拷贝上把照抄取指针的叶序从 `previous_index` 改成当前叶序（`.get(position)`），以及把映射 key 的取法改成当前身份，各跑 harness `inode_leaf_absorb_and_borrow_on_the_unlink_path`，两次都恰有一条红：`the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty`（输出见 K7）。所以 R3 的两个取值点各被恰好一条用例盯着。
- 「删空最左一片那次新写的槽数是不是真的不随右边片数涨」：**在 `crates/` 走得到的状态上这一形不出现**。探针里 20.9 万次删除没有一次「删空一片、而它有右邻」（计数器 `emptied_with_right_neighbour` = 0，输出 `PROBE emptied_with_right_neighbour*1000+leftmost=0`）：有右邻的片一旦低于 117 就借，借完两片都 ≥ 约 117，删不空；删空只发生在没有右邻的最右片 / 唯一的片。实现员报告第五节第 2 条（`research/prompts/m3-batch50-m3b-implementer-report.md`）自己也说走不到，把验收改到「第一片吸收第二片」，右边每一片挪位置、新写槽数 20 片与 40 片的树都是 14：这条用例我在拷贝上跑过（4 passed，输出见 K7）。`remove_record_from_leaf_containers` 的删空分支（inode_tree.rs:381-386）因此只有手造的树（单测）走得到。
判定：**一致**（照抄的判定、指针、映射 key 都对；「删空最左片」那一形在可达状态上是空的，验收已移到吸收）。推翻条件：造出 `crates/` 自己走得到的一串操作使某次删除后某个 `CarriedFrom` 的片内容与上一版不同，或使吸收之后新写的槽数随右边片数涨（实现员的 20 / 40 片用例是固定的两点，不是扫描）。

### K4.3　N1 删 I-9.4 第三句之后，I-9.12 与别的判据还拦不拦得住分隔 key 写错

条款：`.claude/kb/decisions/08-核心索引结构.md:164`「分隔 key = 该孩子创建时的最小 key，左从右借的那次发布改成被借的右孩子这一版的最小 key」。I-9.12 的判法（walk.rs）：「self.judgements.judge("I-9.12", separator <= smallest, || {」（`crates/singlefs-checker/src/walk.rs:3298`）、「.judge("I-9.12", *separator > previous_separator, || {」（`crates/singlefs-checker/src/walk.rs:3305`）、「.judge("I-9.12", *separator > previous_largest, || {」（`crates/singlefs-checker/src/walk.rs:3309`）——分隔 key ≤ 孩子最小 key、严格递增、> 左邻孩子最大 key。

逐类写错：
- 比孩子最小 key 大：第一道抓。
- ≤ 左邻孩子最大 key（例如借完右片的分隔 key 没改、还是旧容器号，而它的最小几条已被借走）：第三道抓。变异表的「第 50 批 N1：借之后右片的分隔 key 不改成它这一版的最小 key」（用例 `a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing`）盯着写路径一侧。
- 落在 (左邻最大 key, 本片最小 key] 里的任意值：I-9.12 不抓，但这样的值对已有 key 的路由与现行（最右插入）没有区别——这一类是「写错而无害」。
- 「容器号」（身份那 8 字节）给小了、仍大于左邻号：I-9.12 不看容器号，I-9.4 第三句删了之后没有判据抓，只剩 `inode_tree` 单测——这正是用户 2026-10-08 接受的 N1 代价（判决第五节 X4 那一行），我没有造这类镜像去量。
一处 kb 内部的不一致（不影响代码）：`.claude/kb/invariants.md:300`（I-9.12 那一行）引 D8 写成「分隔 key = 该孩子这一版的最小 key（借之后右片的分隔 key 改成它这一版的最小 key，D8（核心索引结构） 已定项 6）」，而 D8 第 164 行是「创建时的最小 key」，只有借的那一次才改成「这一版的最小 key」。两句并排：invariants.md 把「创建时」引成了「这一版」。代码（inode_tree.rs:64-67 的注释与 :510-515）按 D8 做，checker 只判不等式，所以没有行为差异，只是 I-9.12 那一行的引文不是整行抄。
判定：**一致**（I-9.12 的三道仍拦得住所有会改变路由的写错；被删掉的只是容器号偏小那一类，已是用户认的代价）；kb 引文一处不一致（上）。

### K4.4　理想模型与胶水对 M3 的描述：两份独立的算法，还是抄了同一段

- harness 的理想模型 `crates/singlefs-harness/src/model.rs` 与对拍胶水 `model_comparison.rs` **根本没有 M3**：模型只罩「删第一个文件那次最后一批让唯一那片叶容器身份消失」这一形：「/// 退役树的开放容器（判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 5 条，照墓碑树）：模型罩的历史里只有删第一个文件」（`crates/singlefs-harness/src/model.rs:104`）、「/// 最后一批（D8（核心索引结构） 已定项 16 第 2 条）：剩下的单元不多于产品批大小 144 时意图之后（或上一批之后）的下一步——释放剩下的单元、」（`crates/singlefs-harness/src/model.rs:179`）；`grep -n 'borrow\|absorb\|M3\|merge' model.rs` 零命中，`model_comparison.rs` 只有三处提到退役树（"模型今天只罩退役树一片容器"）。所以合并 / 借 / R3 在 harness 的对拍里**没有对照**，它们只被 `inode_tree.rs` 的单测与那 4 条 harness 发布路径用例盯着，期望值是手算的。
- 仓里唯一把 `crates/` 的 inode 树对着独立模型逐步对拍的装置是 E176 的 `scripts/experiments/e176_crates_oracle/oracle.rs`，它**只对拍 M1**：「let mut model_tree = InodeLeafTree::empty(MergePolicy::DropEmptyOnly, LeafContainerLimit::OneRootNode);」（`scripts/experiments/e176_crates_oracle/oracle.rs:82`）；而且它读 `after.containers_dropped`（「} else if after.containers_dropped != model_rewrites.containers_dropped {」（`scripts/experiments/e176_crates_oracle/oracle.rs:124`）），这个字段在这次的补丁里没有了（`grep -rn containers_dropped` 在冻结副本 `crates/` 里零命中，取而代之的是 `retired`），它是 git 里 `A`（已暂存）的文件，没在 70 个补丁文件里——所以它**对这次改后的 `crates/` 编不过**（我没编，依据是字段零命中）。
- E176 模型的 M3 臂与 `crates/` 的 M3 实现**结构一一对应**（`merge_until_no_pair_fits` / `absorb_until_no_pair_fits`、`borrow_from_right_if_below_floor` / `borrow_from_the_right_neighbour_if_below_the_floor`，判断次序相同）：不是各写各的，是同一份策略文字翻成两份代码。两份在 20.9 万次随机删与一条定向形上逐步相同（K3.2 的输出），说明实现忠实于 E176 的 M3 臂，**不说明** M3 臂本身对不对。我另外断言的「相邻两片之和 > 233」「分隔 key 与区间」「退役集 = 上一版叶身份集 − 这一版」「照抄的片内容不变」是我自己写的独立性质，也全过。
判定：**冲突**（一半）：条款（正文第 4 节分工、kb 里「合并时机是写路径策略（单测 / 模型对拍）」 那一句括号）说「单测 / 模型对拍」，而 harness 模型没有 M3、E176 的对拍装置对这次的 `crates/` 编不过。改法落在：要么 `crates/singlefs-harness/src/model.rs` 加 M3 的描述（多叶、吸收、借），要么把 `oracle.rs` 改成读 `retired` 并对拍 M3 臂后入库。推翻条件：找到 harness 里另一处用独立算法算 M3 形态的地方（我 grep 的词是 borrow / absorb / M3 / merge / 退役树，若有别的名字我没搜到）。

### 更正（K4.4 判定行里缺掉的路径）

K4.4 最后一行「判定：**冲突**（一半）」里「kb 里「合并时机是写路径策略（单测 / 模型对拍）」 那一句括号」前后缺了出处（我写的时候反引号被 shell 吃了），也不该写「正文第 4 节分工」（第 4 节是各腿的分工，没有这句话）。正确的出处是：`.claude/kb/decisions/08-核心索引结构.md:171`「合并时机是写路径策略（单测 / 模型对拍），不进格式」。那一句说合并时机靠「单测 / 模型对拍」罩；harness 理想模型没有 M3（上面 K4.4 第一条），E176 对拍装置对改后的 `crates/` 编不过（第二条）。判定不变。

## 五、K5　准入（R2 两样都做）

### K5.1　各项取哪一版的 H_退役、n_记录；一次发布至多几条退役记录，W 罩不罩得住

条款：`.claude/kb/decisions/03-空间分配.md:52`「标签 6 = 副本数 × ((2 + H_墓碑 + 1 + 2 + H_退役 + 1) × 16384 + 41 × n_live + 35 × n_记录)」；`.claude/kb/decisions/28-挂载期承诺量.md:29`「W = 24 + (6 + H_退役) + Σ_ℓ min(N_ℓ, 2·(K + 26))」；`.claude/kb/decisions/28-挂载期承诺量.md:118`「退役容器单列一项，按一次退役容器 COW 连祖先算：容器 2 块 + 退役树根 1 块 + 退役树内部层数，零根与只有根的退役树 3 块」。

| 项 | 取哪一版 | 代码 | 对条款 |
|---|---|---|---|
| 标签 6 的 H_退役、n_记录 | **这次发布之后**（写入类发布按发布之后的标签判；记账行写的是同一个数） | 「retirement_tree_internal_levels:」（`crates/singlefs-core/src/transaction.rs:4408`）（H 从根节点码 2 头现读，这次重写根时读新根，否则读上一版 `RetirementRoot` 单元：「if resolved.retirement_tree.is_the_root_rewritten() {」（`crates/singlefs-core/src/transaction.rs:4426`））；「inode_records: inode_records_of_the_inode_tree(&resolved.inode_tree.containers),」（`crates/singlefs-core/src/transaction.rs:4410`）（n_记录 = 这次之后 inode 树全部记录，含 nlink = 0） | 一致（D3 第 52 行「写入类发布按发布之后的标签判」；n_记录不数 n_live 的理由在同一行） |
| W 里的 H_退役 | **要接着的那一版**（版本 to_build_on） | 「retirement_tree_internal_levels: version.retirement_tree_internal_levels(),」（`crates/singlefs-core/src/admission.rs:453`）、「retirement_tree_internal_levels: version_to_build_on」（`crates/singlefs-core/src/admission.rs:1568`）；公式体 「+ DELETION_PUBLISH_RETIREMENT_TREE_SLOTS_BEYOND_ITS_INTERNAL_LEVELS_AT_MOST,」（`crates/singlefs-core/src/admission.rs:417`）、「.and_then(|slots| slots.checked_add(retirement_tree_internal_levels))」（`crates/singlefs-core/src/admission.rs:420`） | 一致（D28 第 29 行「H_退役 是退役树的内部层数」，6 + H_退役 里的 +1 已留给根满长高） |
| ckpt_cost 退役项 | 要接着的那一版 | 「let retirement_tree_internal_levels = file_version.retirement_tree_internal_levels();」（`crates/singlefs-core/src/admission.rs:1081`）；容器 2 + 根 1 + H 块：admission.rs:208-216；中央映射树删插数加 1 + 1 + H 把：「nodes.checked_add(central_mapping_entries_of_one_retirement_container_cow(」（`crates/singlefs-core/src/admission.rs:1095`）；列进 Σ 名单外的单列项：「let retirement_container_cow =」（`crates/singlefs-core/src/admission.rs:1123`） | 一致（D28 第 118 行；「中央映射树删插数」在 D28 第 113 行那条里） |

单位核算（独立于代码，我手算对 core lib 里用例的期望值）：H_墓碑 = 0、H_退役 = 0、n_live = n_记录 = 1、两副本：2 × ((2 + 0 + 1 + 2 + 0 + 1) × 16384 + 41 + 35) = 2 × (98304 + 76) = 196760；H_退役 = 1、n_live = 0、n_记录 = 1：2 × ((2 + 0 + 1 + 2 + 1 + 1) × 16384 + 35) = 2 × (114688 + 35) = 229446（与 admission.rs 单测注释里的 229446 相同，用例 `the_retirement_share_of_tag_six_counts_every_inode_record_and_the_retirement_tree_levels` 在 core lib 的 240 个里，baseline 全过）。W：产品批 K = 144、S = 8、H_退役 = 0、两块 4 GiB 盘：24 + 6 + min(524, 340) + min(4, 340) = 374，环窗口 25 × 374 槽（用例 `ring_window_on_two_four_gibibyte_devices_is_twenty_five_times_three_hundred_seventy_four_slots` 过）。

一次发布至多几条退役记录：
- `crates/` 自己走得到的状态上至多 **1** 条（K3.2：20.9 万次删除，`max_retired_per_delete=1`）。
- 一批删掉多片、连锁吸收：只出现在不满足「相邻两片之和 ≥ 234」的树上（旧 M1 池 / 坏镜像）；手造 135 片 × 2 条的树删中间一条：一次退役 116 条（K3.2 的 CASCADE 输出）。116 × 34 B = 3 944 B，装在 1–2 片退役容器里，W 里「至多跨两片退役容器」（`crates/singlefs-core/src/admission.rs:386-389`，注释自己写的是「一次发布的退役记录 ≤ 134 条 < 959」）仍成立；这一格的 W 罩得住退役树那一项。W 的分配记录树那一项（`2·(K + 26)` 每层）是否罩得住「116 片叶一起释放」我没推也没量（见 K3.2 末）。
- 跨层长高：退役树根满（135 片容器 × 959 条 = 129 465 条之后）才长一层，那一次发布新写的退役单元 = 新容器 2 + 新根 1 + 旧根下沉节点 1 + 内部层 H（旧）= 4 + H + 1 ≤ 6 + H；这是 D28 第 29 行自己的拆法（2 × 2 + 1 + H + 1），代码侧在 `container_tree.rs` 的 `after_a_container_is_opened_at_the_right_end`（container_tree.rs:337-370）与它的单测 `opening_a_container_past_what_one_root_holds_grows_the_retirement_tree_one_level`。我没造一棵 129 465 条退役记录的池去跑那一发布（产品容量下跑不动；单测用压小的容量旋钮）。
判定：**一致**。推翻条件：造出一次发布写的退役单元超过 6 + H_退役 个槽（含根满长高那一次），或标签 6 / ckpt_cost 读了「这次之前」的 H_退役 / n_记录。

### K5.2　「贴边建满后 unlink 不超卖」：先给读法，再造

我的读法（两条，都不是实现员的甲乙丙原样）：
- **读法 R1（操作结局）**：判据是**操作是否做得成**，不读「可用」这个数——贴边盘面上，每一个删除类操作（unlink 整串：意图 + 各批 + 末批，以及释放空间的那一个）都要发成、不被「空间」拒；写入类操作可以被拒。理由：D3 已定项 17 逐字「释放空间这个操作本身不需要申请空间」，kb 对删除类发布的读法本来就是 `.claude/kb/decisions/03-空间分配.md:52`「删除类发布（意图、各批、挂载里的续做批）不判式子那一判，落点那一判照判」，判「可用 ≥ 0」会把环窗口的暂态当成违规——实现员量到的「意图之后可用 −14…−26」正是环窗口在兜的暂态，不是超卖。
- **读法 R2（稳态）**：删除走完、转过一圈根环、defer 放回之后「可用 ≥ 0」。这条在 `crates/` 里没有现成的读数入口（要自己转环），我没做；实现员量到的开出退役树那一次可用不变（`research/prompts/m3-batch50-m3b-implementer-report.md` 第五节第 3 条第 3 点）是对 R2 的间接支持，我没有复跑。

我造的（探针 `probe_k52.rs`，在拷贝的 harness 档里，经会话 `mounted_session` 发布；盘是随机历史那档 384 槽小盘 `HistoryDeviceWidth::UnitAreaOf384Slots`，两块）：mkfs、取号、暖机、第一个文件（inode 1）→ 建 234 个空 inode（叶 233 + 2）→ 把第一个文件顺序写成 du 个数据单元（du 从 60 扫到 220，**逐个 du 都跑**，其中 du=108…115、187…189、208…210 是各变体的边）→ 在写入的边上再逐个 `create_one_inode` 到被拒（600 次封顶）→ unlink 2 号（开出墓碑树）→ unlink 3 号（左片 231 + 右片 2 = 233，吸收，开出退役树）→ unlink 1 号（带 du 个数据单元的文件）。每步记「发成 / 被拒的变体名」。基线（冻结副本代码不动）原样输出，边附近：
```
K52 du=113 write=ok(raises=0,versions=1) then +0 inodes (… | unlink2=ok(raises=0,versions=2) unlink3=ok(raises=0,versions=2) unlink1=ok(raises=0,versions=2)
K52 du=114 write=ok(raises=0,versions=1) then +0 inodes (… | unlink2=ok(raises=0,versions=2) unlink3=ok(raises=0,versions=2) unlink1=ok(raises=0,versions=2)
K52 du=115 write=ERR:NoSpaceAfterRaisingTheFloor(NoSpaceAfterRaisingTheFloor { la | unlink2= unlink3= unlink1=
K52 max_ok_write_du=114
```
（du ≥ 115 的 36 个都是 `NoSpaceAfterRaisingTheFloor`；du ≤ 114 的每个都发成、之后 `create_one_inode` 一个也进不去（+0），三次 unlink 全发成。）**读法 R1 下基线造不出超卖**。

「去掉退役那几项会不会让它红」：我在拷贝上分别去掉标签 6 的退役常驻三项（容器 2 + 根 1 + H_退役）、35 × n_记录、W 的 6 + H_退役、ckpt_cost 的退役项，再四样一起去掉，各自扫到该变体写入的边，边上同样建满后 unlink：

| 变体 | 写入的边（最大 du） | 边上 unlink 2 / 3 / 1 |
|---|---|---|
| 基线 | 114 | 全发成 |
| 去标签 6 退役常驻三项 | 115 | 全发成 |
| 去 35 × n_记录 | 114 | 全发成 |
| 去 W 的 6 + H_退役 | 188 | 全发成 |
| 去 ckpt_cost 退役项 | 133 | 全发成 |
| 四样全去 | 209 | 全发成 |

（原样日志在草稿目录 `var-M-ra.log`、`var-M-rb.log`、`var-M-rc.log`、`var-M-rd.log`、`var-M-all.log`、`var-BASE.log`；变体的替换串在 `v/r?.old / .new`，都已入模型目录。）所以在读法 R1 下，这四项退役项**都不会因为去掉而让「贴边建满后 unlink」红**——边移动了（去 W 那项一下多收 74 个单元），可 unlink 照样做得成：环窗口 (3S+1) × W 这块缓冲在 384 槽小盘上远大于一次删除真要的几十槽。这与实现员停下的结论一致（他没写用例，原因同上），我的补充是把「可用的读法」换成「操作结局」之后也一样。

判定：**规则没说**（「可用」在删除类发布之后怎么读，条款没写；这是实现员的停下问题，我给了 R1 / R2 两种读法）；R1 下 **没造出超卖**，也没造出「释放空间的操作被准入拒」。推翻条件：造出一个盘面使 R1 下某次 unlink 的某一批被「落点那一判」拒——我的扫描只覆盖了「一个文件 + 234 个空 inode + 384 槽盘 + 两次 unlink 空 inode 再 unlink 文件」这一族；没扫多文件、多次开新退役容器（要 959 次叶死亡）、大盘（4 GiB 盘的 W = 374）、删 → 回退 → 再删（C617 已记）。这些我**没做**，不能当作没打中。

### K5.3　开出第一片退役容器之后，常驻 3 槽与实占同时在账上：换一种盘面会不会让可用变负

读码：标签 6 的常驻项是恒定的 (2 + H_墓碑 + 1 + 2 + H_退役 + 1) 个槽（admission.rs:347-354），不随「第一片容器已经开出来」而减；开出来之后那 2 + 1 个槽又进了 `已分配`：同一批槽在两处都扣。这是 D3 第 52 行式子**字面**的结果（常驻项没有「减去已实占」一项），墓碑树同形（已有）。所以：贴边盘面上，一次 unlink 开出退役树（或墓碑树）的第一片容器，稳态「可用」会少 3 槽（每个树）。我的探针里这一格没让操作失败：unlink 之后只是写入类操作被拒（基线 du ≤ 114 的每格 unlink 之后，写入仍按准入式子判）。「可用变负」会不会让可写挂载拒——读码：`mount.rs` 的可写挂载准入「可用(d) ≥ 0」不够时走 `push_floor_raises_after_the_row_publish`，推完抬 F 仍不够交回 `StillShortAfterTheFloorRaises`（mount.rs:6689-6695）而不是拒挂载，删除续做照常跑（删除队列非空时窗口算作可动用）。所以变负的后果是**写入类操作被拒、挂载仍可写、删除仍能做**，不会把池锁死。我没有造这个盘面去挂载（没做）。
判定：**规则没说**（条款没讲常驻项在容器开出后要不要减）；行为与字面一致；后果读码上无害。推翻条件：造出一个池，其可用在 unlink 之后变负，随后的可写挂载或删除续做因此失败（不是写入被拒）。

## 六、K6　checker 与 GPU 判器

### K6.1　I-9.31–I-9.35 的 CPU 判法在合法镜像上误不误报

条款：`.claude/kb/invariants.md:323`「被判的这一版与它的父版相比（这一版的 txg 恰是这一版谱系树里某条记录的 T_rb 时，父版是那条记录的 T_old，否则是 txg − 1；父版在回退候选集里读得出才判）」。

代码：父版取法 「.find(|record| record.rollback_txg == version.checkpoint_txg)」（`crates/singlefs-checker/src/walk.rs:11204`）（有 T_rb = 本版 txg 的记录取它的 T_old，否则 `checkpoint_txg.checked_sub(1)`，txg 0 没有父版）；父版 txg 在版本清单里没有就跳过（walk.rs:10699-10701 的 `[] => continue`），不止一份报判不了（walk.rs:10702-10708）；候选集滤不掉被抛弃根时整条部分没判：「judgements.partially_judged(」（`crates/singlefs-checker/src/walk.rs:10669`）。漏记与多记两个方向：「gone.difference(retired_at_this_txg).collect();」（`crates/singlefs-checker/src/walk.rs:10759`）、「let extra: Vec<&ContainerIdentityOfTheScan> =」（`crates/singlefs-checker/src/walk.rs:10760`）。I-9.32 逐条：「let holds = retired.record_type == PACKED_TYPE_INODE」（`crates/singlefs-checker/src/walk.rs:10587`）（类型 2、出生树 ≠ 0、容器出生代 < 死亡 txg ≤ 本版 txg）。

逐形（真跑的用例在拷贝上复跑，原样输出见 K3.3 与 K2.3）：
| 形 | 用例 / 推理 | 结果 |
|---|---|---|
| 回退越过一片叶的出生 | `rolling_back_past_the_birth_of_a_leaf_compares_with_the_rollback_target_and_no_invariant_reddens`：回退版的父版取 T_old，叶集合与 R_old 相同，漏记多记都空 | 过（ok） |
| 删 → 回退 → 再删 | `unlinking_rolling_back_and_unlinking_again_writes_a_second_record_of_the_same_leaf_and_no_invariant_reddens`：同一身份两条，第一条死亡 txg 不在新版父链上 | 过（ok） |
| 借后吸收 | `the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it`（absorb 目标文件 4 passed）：一次借 + 一次吸收，退役 1 条 | 过（ok） |
| 克隆头 | `crates/` 今天没有克隆路径（walk.rs:10819 的 TODO 与 transaction.rs:3073 的 TODO 都点名）；I-9.33 / I-9.34 / I-9.35 按头 ID 配树的判法在多头镜像上**没有输入** | 没法测（没有输入）|
| 候选集里有被抛弃根 | 被抛弃根在候选集里时 I-9.35 整条报部分没判（walk.rs:10666-10678）；Opt2 用例 `a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root` 过（在 `harness-k6b.log` 里 ok）| 过 |
| 空发布 / 回退那次 / 崩溃抛弃过的段 | 空发布：父版 = txg − 1，叶集合不变，退役集空，成立；回退那次：K2/K3 已述；崩溃抛弃段：父版 txg 在抛弃根上时候选集滤掉它 → `[] => continue` | 读码一致，无单独用例 |

另跑了更大的一批 `pool_checker_judges_the_deletion_state_of_every_walked_version`（45 passed）、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version`（5 passed）、`rollback_by_a_forward_publish`（3 passed、22 ignored——ignore 的是 harness 慢档，原因文字在日志里）。
判定：**一致**。推翻条件：造出一张合法镜像（含 `crates/` 走得到的历史）使 I-9.31–I-9.35 任一条判红；或使父版取法 txg − 1 在崩溃恢复之后指到一版**被选根的时间线上不是父亲的**候选根（崩溃后新实例的 txg 都大于环里每个 txg，D23 第 14 项注 3，所以 txg − 1 要么是真父亲要么不在候选集——这一句是推的）。

### K6.2　GPU 判器与 CPU 逐条同判

我没跑 GPU（checker 档，主 agent 手里）。读码对照（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`）：
- 码 8 那一支：`grep -rn '码 8' --include=*.rs --include=*.wgsl crates` 在冻结副本里零命中；`UNDECIDABLE_*` 常量只有 1…7（`crash_judge_common.wgsl:87-94`）——正文第二节说的「候选集里有一版退役树不是零根时扫描内核记码 8、交 CPU」在冻结副本里**已经没有**（实现员报告 `research/prompts/m3-batch50-m3b-implementer-report.md` 第 7 行「码 8 那一支删了」）；正文这一句是旧的。
- 退役树非零根：GPU 自己判 I-9.32–I-9.35。I-9.32 逐项与 CPU 同（GPU 的 `d_judge_every_retirement_record_is_well_formed`：类型、出生树、`!lt64(container_birth, death_txg)`、`!le64(death_txg, version_txg)`、记录数 ≤ 959）；I-9.35 的漏记 / 多记两个方向与 CPU 同：「&& !d_retired_at(gone, version_txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3732`）、「if (!d_leaf_identity_table_holds(D_LEAF_IDENTITIES_OF_THE_PARENT_VERSION, parent_leaf_identity_count, retired)」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3745`）；父版取法：「parent_txg = sub64(version_txg, vec2<u32>(1u, 0u));」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3640`）（无 T_rb 记录时 txg − 1）。
- 容量差异（GPU 有上限、CPU 没有）：水位行 ≤ 256（「const D_MAX_INODE_NUMBER_WATERMARK_ROWS: u32 = 256u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:862`），超了走 `UNDECIDABLE_TOO_MANY_ENTRIES`：「if (d_inode_number_watermark_row_count >= D_MAX_INODE_NUMBER_WATERMARK_ROWS) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2886`））；一棵 inode 树叶身份 ≤ 256（「const D_MAX_LEAF_IDENTITIES: u32 = 256u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:871`），合法树至多 135 片）；退役记录两两比 ≤ 4096（「const DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE: u32 = 4096u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3335`）、「if (record_total > DELETION_MAX_RETIREMENT_RECORDS_COMPARED_PAIRWISE) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3507`））；树表 ≤ 12 棵（「const C_MAX_TREES_PER_TABLE: u32 = 12u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:425`），第 13 棵判不了）。超了都是「判不了、交 CPU」，不会给出与 CPU 相反的判定；代价是长历史（台阶行与退役记录都是只增不删）上 GPU 的 I-9.30 / I-9.33 退化成 CPU。
- 一处结构性差异（读码，没有跑出不同结果）：CPU 的 I-9.34 用 `BTreeMap` 按树 ID 收 inode 树叶身份（同 ID 后一条盖前一条），GPU 用「同种类同 ID 的最后一条」（「inode_tree_entry = d_last_tree_table_entry_of_kind_keyed_by(tt, TREE_KIND_INODE, 0u, head_identifier);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3677`） 起 `d_last_tree_table_entry_of_kind_keyed_by`），两边都是「最后一条胜」，同 ID 的第二条 inode 树条目是坏镜像（I-9.16 ③ 红）。
判定：**一致**（读码）；GPU 的实跑对拍由主 agent 的 checker 档跑。推翻条件：GPU 对拍那一趟里任一状态 CPU 与 GPU 的违例集合不同。

### K6.3　I-9.16 ④ 十二种、I-9.27 加种类 13、I-1.7 退役记录宽 34：坏镜像各自只红它自己吗

| 不变量 | 坏镜像 | 红的 | 判 |
|---|---|---|---|
| I-9.16 ④（缺 deadlist / logged_ops 那一条） | `a_tree_table_without_its_deadlist_entry_reddens_only_the_tree_table_ordering_invariant`、`…logged_operations_entry…`（11 条的树表） | 只红 I-9.16 | 一致 |
| I-9.16 ④（缺退役树那一条） | `a_tree_table_without_its_retirement_entry_reddens_the_tree_table_ordering_invariant_besides_the_walk` | I-9.16 **加** I-1.1（树表头 key 区间末端还是 65548，去掉最后一条之后不合）：断言就写 `["I-1.1", "I-9.16"]`（checker_known_bad_images.rs:9653-9657） | **不是「只红它自己」**；用例标题与注释老实写了 besides the walk，原因是被去掉的恰是最后一条、头的 key 区间没跟着改 |
| I-9.27（种类 13 头 ID 配不上） | `a_retirement_tree_entry_whose_head_identifier_names_no_inode_tree_reddens_only_the_head_identifier_invariant` | 只红 I-9.27 | 一致 |
| I-1.7（退役容器记录宽 ≠ 34） | **没有这样的坏镜像**：`set_the_record_width_of_the_first_container_of_the_tree_of_kind` 的调用点只有 inode ×1、墓碑 ×2、谱系 ×2（`grep -rn -A3 …` 数出来），没有 `TREE_KIND_RETIREMENT`；GPU 对拍文件里对退役容器的改动是改记录内容 / 追加一条（宽仍 34），不改宽 | — | **缺**，见 K7 的幸存变异 |

另有一处 kb 与代码并排的**冲突**：
- kb：`.claude/kb/invariants.md:32`「类型 1（墓碑）是 40、类型 5（谱系）是 16，D18（块里携带什么信息） 已定项 24」（I-1.7 那一行，对打包记录类型登记的记录宽的列举；该行没有提类型 3 退役记录宽 34）。
- 代码：「PACKED_TYPE_LINEAGE => Some(24),」（`crates/singlefs-checker/src/walk.rs:208`）、「PACKED_TYPE_RETIREMENT => Some(34),」（`crates/singlefs-checker/src/walk.rs:210`）；格式常量 「pub const LINEAGE_RECORD_BYTES: u64 = 24;」（`crates/singlefs-format/src/lib.rs:119`）。
谱系记录宽是 24（W3 把 16 改成 24），kb 的 I-1.7 行还写 16，也没有 34；D18 第 580 行本身写的是 24。改法落在 `.claude/kb/invariants.md` 第 32 行那一句（把「类型 5（谱系）是 16」改成 24，加「类型 3（退役）是 34，D18 已定项 11」）。这是 kb 文字的陈旧，checker 代码本身对。
判定：I-9.16 ④（deadlist / logged_ops）、I-9.27 **一致**；I-9.16 ④（退役）**部分冲突**（另红 I-1.1，文档已写明）；I-1.7（退役宽）**缺会红检查**。

## 七、K7　判别力（变异表追加 71 行、整行替换 45 行、删 2 行）

先数：命令 `git diff -- crates/mutations.tsv > mut-unstaged.diff`（工作区对暂存区；暂存区那一份是这一批合入之前的）得到 `118 47`（`git diff --numstat -- crates/mutations.tsv` 原样输出 `118	47	crates/mutations.tsv`）。我按第一列名字分：加的 118 行里 45 行的名字在删的行里出现（整行替换：45 行里只改了原文 / 替换文两段的 21 行、只改原文一段的 8 行、只改点名的测试一段的 8 行、改了「cargo 参数 + 测试名」的 3 行，等等），73 行是新名字；删的 47 行里 2 行名字没再出现（`实审 B2 第 9 条（I-9.4 第三句）…` 两行，正是 N1 删的那两行）。净增 118 − 47 = 71，与题面一致。

### K7.1　这批的行能不能拦住改回去

我把 118 行里**非 checker 档**的 98 行逐行重放：把「原文 → 替换文」套在拷贝上（原文在文件里恰好命中一次），跑那一行点名的 cargo 参数，看点名的测试是不是红（脚本 `bin/rows.py`，入模型目录；经 `run-with-memory-cap.sh 10G` 与 `capped.sh 8`）。结果（`rows-results.tsv`，一行一行 `RED-NAMED` = 点名的那条测试红了）：
```
98 行：RED-NAMED 98，GREEN 0，RED-OTHER 0，COMPILE-ERR 0，TIMEOUT 0，ANCHOR（原文命中不是恰好一次）0
按文件：admission.rs 15、checker 的 walk.rs 25、inode_tree.rs 14、transaction.rs 9、model.rs 7、container_tree.rs 6、format 的 lib.rs 6、recovery.rs 5、mount.rs 3、records.rs 3、retirement_tree.rs 2，另 model_comparison.rs / mount_transparency.rs / scenario.rs 各 1
```
（`cut -f3 rows-results.tsv | sort | uniq -c` 输出 `98 RED-NAMED`。这是拷贝上的复跑，不替主 agent 在主工作区上的证红。）
没重放的 20 行是 checker 档（GPU 判器 .wgsl、`crash_judge_gpu.rs`、E165 的 bin、层 0 快档的两行 transaction.rs）——正文第一节说那一档在主 agent 手里，我不跑；我也就**没有**那 20 行的证据，列在「没做什么」。

### K7.2　哪几种改坏的写法照绿

正文点名的几类逐个对：
| 改坏的写法 | 拦它的 | 我的验证 |
|---|---|---|
| M3 阈值：并的条件少一格 / 多一格（233 ≤ → <） | 表里有行（「两片之和恰好 233 不并」）；core lib 的 `deleting_from_the_left_of_a_pair_borrows_until_the_pair_fits_in_one_and_then_absorbs` 也红 | 我自己另改：`<=` → `<` 红（lib 一条） |
| M3 阈值：借的下限 117 ± 1 | 表里的行是「低于下限也不借」（只拦方向，不拦 ±1）；lib 一条用例红 | 我自己改成 118、116：各红同一条 lib 用例，lib 里只有这一条盯住下限的取值 |
| M3 借的条数（⌊(右 − 左) ÷ 2⌋ 改成向上取整） | 表里的行是「多借一条」；lib 同一条用例红 | 我改向上取整：红同一条 |
| M3 方向：**先右后左** | **表里没有行，也没有用例** | **幸存**：把 `absorb_until_no_pair_fits` 里 `right_pair_fits` / `left_pair_fits` 两支对调（「if right_pair_fits {」（`crates/singlefs-core/src/inode_tree.rs:466`） 起），core lib 240 个全过、harness `inode_leaf_absorb_and_borrow_on_the_unlink_path` 4 个全过；我的探针里的随机流 20.9 万步也不红（相邻两对同时装得下要「三片各 117 条、从中间删一条」那种形，随机流碰不到）；只有我加的定向用例红：`DIRECTED model(right-first)=[(1, 117), (234, 233)] crates(变异后)=[(1, 233), (467, 117)]`。这是 E176 的 M3 臂（`scripts/experiments/e176_inode_leaf_merge_capacity_and_rewrites.rs` 第 221–222 行）取的先右后左，kb 没写这一格（K4.1 判「规则没说」）；是行为差、不是正确性差（两种结果都满足所有不变量），所以严重性低，但「先右后左」这个正文点名的方向没有任何会红的东西钉着 |
| R3 照抄的判定：照抄时取错叶序的指针 / 映射 key | 表里有「按位置重写」那一类的行；取指针叶序与取映射 key 这两处**没有专门的行** | 我改成取当前叶序的指针、取当前身份的映射 key：各红恰好一条 `the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty`（其余 3 条绿）——有拦，但每处只有一条用例拦 |
| R3 被借的片 / 借出的片不进重写清单 | 表里没有专门的行 | 借出的片（`lender`）不标「改过」：红两条 lib 用例；借的片（`borrower`）不标「改过」：**绿**，但它在这之前已经被标过（删记录那一步 inode_tree.rs:380），这一句是冗余的，等价变异 |
| 死亡 txg 的取法 | 表里有行（「取这次发布的前一个 txg」） | 我另改「+ 1」（死在本版之后）：红两条（absorb 目标里的 `the_left_leaf_absorbs_…` 与 `the_lender_absorbs_…`），因为 I-9.32 要求 death ≤ 本版 txg |
| Opt2 的滤法 | 表里有行（环里全算）、旧行（只取环不与 cur 取 max） | 我另改成取反（只留被抛弃的）：红恰好一条 `a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root`（rollback_by_a_forward_publish.rs 里不带 ignore 的那一条） |
| ckpt_cost 那一项用这次之前还是之后的行数 | 表里两行（退回上一版节点数 / 不照抄台阶行） | 98 行重放里这两行 RED-NAMED |
| I-1.7 对退役记录宽 34 的判 | **表里没有行，也没有坏镜像** | **幸存**：在 `judge_packed_container` 里让类型 3 跳过宽度比较（「&& registered_record_width(record_type) == Some(width)」（`crates/singlefs-checker/src/walk.rs:3523`） 那一句外加 `record_type == PACKED_TYPE_RETIREMENT ||`），`checker_known_bad_images` 全部 224 个与 absorb 目标 4 个全过（命令输出见下），GPU 一侧同样没有退役容器宽度的坏流（K6.3 的 grep） |
| 等价变异（不是缺口，写出来免得被当成幸存） | — | 「借完又没并过才借」那个保护（inode_tree.rs:389）去掉、删空之后对新邻居再并（「tree.absorb_until_no_pair_fits(holding_position - 1);」（`crates/singlefs-core/src/inode_tree.rs:385`））去掉：都绿。推：在「相邻两片之和 ≥ 234」的不变量下这两处是死代码（删一条之后至多一对刚好 233；删空的那片只有 1 条时两边邻居各 ≥ 233，和 466），`remove_record_from_leaf_containers` 的这两支只有手造的树能走到 |

两条幸存的原样输出（命令都在拷贝 `probe-tree/` 里，经 `run-with-memory-cap.sh 10G` 与 `capped.sh 8`；脚本 `bin/variant3.py`、`bin/variant4.py`）：
```
VARIANT V-order exit=101 compile_error=False inode_tree_lib_ok=20
  red=['directed_three_117_both_pairs_fit_after_one_delete']
  ['test result: ok. 240 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s', 'test result: FAILED. 23 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.56s']
VARIANT H-order exit=0 compile_error=False tests_ok=4
  red=[]
  ['test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.47s']
VARIANT C-retwidth exit=0 compile_error=False tests_ok=228
  red=[]
  ['test result: ok. 224 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 121.55s', 'test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.19s']
```
（V-order：lib 240 个全过，红的只有我自己加的定向用例；H-order：同一变异在 harness 的 4 条 absorb 用例上全绿；C-retwidth：I-1.7 对类型 3 不判宽，224 + 4 全绿。）

判定：**一致**（这批的 98 行非 checker 档变异行全红）；**两处缺口**（先右后左；I-1.7 对退役记录宽 34 没有会红的检查）。推翻条件：找到点名「先右后左」或「退役容器记录宽」的现有测试（我 grep 的是 `left_pair_fits` 与 `TREE_KIND_RETIREMENT`、`RETIREMENT_RECORD_BYTES` 在 tests 里的出现，找不到会红的）；或这两处变异在 checker 档 / 其它 harness 文件里有红的（我只在上面那几个目标上跑，没跑全部 188 个目标）。

## 附：K4.2 与 K7 引用的原样输出，以及 K5.2 的扫描范围更正

命令都是 `nice -n 19 … run-with-memory-cap.sh 10G bash capped.sh 8 cargo test --offline --target-dir <自己的编译目录> -p singlefs-harness --test <目标>`，在拷贝 `probe-tree/` 上，变体由 `scripts/variant4.py` 套上再还原。日志在 `research/prompts/m3-batch50-code-r1-sonnet-model/logs/`。
```
基线 inode_leaf_absorb_and_borrow_on_the_unlink_path（未改）：
test the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one ... ok
test a_leaf_below_the_floor_borrows_half_the_difference_from_its_right_neighbour_and_retires_nothing ... ok
test the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it ... ok
test the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 30.76s

T-carrypointer（照抄取指针的叶序改成当前叶序）：
test the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty ... FAILED
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.73s
T-carrykey（映射 key 改取当前身份）：
test the_first_leaf_absorbing_the_second_writes_as_many_slots_in_a_tree_of_twenty_leaves_as_in_a_tree_of_forty ... FAILED
test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.28s
D-death-plus-one（死亡 txg 取这次 + 1）：
test the_left_leaf_absorbs_its_right_neighbour_and_the_final_batch_retires_the_right_one ... FAILED
test the_lender_absorbs_its_right_neighbour_when_the_two_fit_after_lending_and_retires_it ... FAILED
test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.94s
O-opt2-inverted（Opt2 的滤取反，跑 rollback_by_a_forward_publish）：
test a_rollback_after_a_recovery_abandoned_a_root_that_created_inodes_keeps_the_inode_number_watermark_off_the_abandoned_root ... FAILED
test result: FAILED. 2 passed; 1 failed; 22 ignored; 0 measured; 0 filtered out; finished in 3.45s
```
K5.2 的扫描范围更正（上文 K5.2 写「du 从 60 扫到 220，逐个 du 都跑」说大了）：基线逐个 du 跑了 96–150（没有「建 inode 到被拒」那一步的第一版探针），加上带那一步的第二版探针跑了 113–115；四个变体与全去变体各自先在 108–220 内粗扫找到写入的边（每个变体的扫描区间：去标签 6 退役常驻 110–130，其余 108–220，全去 200–225），再用带「建 inode 到被拒」的第二版探针在边附近三格（边 − 1、边、边 + 1）上跑；表里「边上 unlink 全发成」指的是这三格里写得进去的那两格。这也就是说我**没有**对每个变体的每个 du 都跑过「建满再 unlink」，只跑了边附近三格。「只在 384 槽小盘一族上」这一限制保持。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1.1 回退水位的滤法与 D23 第 14 项 | 一致 | 候选集与水位用同一个 `abandoned_by_table`，树 ID 水位仍取环里全部根 |
| K1.2 读不出 / 行不全 / 目标 = cur / 连回退 / 回退后崩溃 | 一致 | 各形取到的水位都与条款一致；回退没有「目标 = cur」特判 |
| K2.1 台阶行与每次发布那一行的读者 | 一致 | 核心、恢复、checker、ckpt_cost 的读法都分得清；潜在：I-9.6 的后者盖前者与 GPU 256 行上限（今天不出错） |
| K2.2 恢复要求「代段 = 根 txg 的那一行」 | 一致 | 记账行只有 `publish_admitted` 一个构造点，合法镜像都有这一行 |
| K2.3 谱系 24 字节、H_cur、I-9.30 ①②④ | 一致 | H_cur 取自 cur；W3 相关 37 条 harness 用例在拷贝上全过；③ 的缺口是已记欠账（读码推理，没造镜像） |
| K2.4 ckpt_cost 记账那一项按这次之后的行数 | 一致 | 用写路径同一个规划器按下一次发布的 key 集现算，台阶跨叶那次有用例核 |
| K3.1 登记与新池字节 | 一致 | 12 条 / 65548 / 65549 / 2400 / 头 ID 都对，harness 用例过；3 处注释里残留旧数（十一条） |
| K3.2 死亡写侧 | 一致 | 退役身份 = 上一版叶集减这一版；死亡 txg 对；只追加；20.9 万次删除每次至多 1 条；不满足「相邻两片之和 ≥ 234」的树（旧池 / 坏镜像）上一次可退 116 条，没有 checker 不变量盯这个前提（列入没验） |
| K3.3 回退照抄 cur、崩溃随所选根 | 一致 | 退役角色不在「取回退目标」集合里，恢复按所选根的树表读 |
| K3.4 容器树一次追加多条 | 一致 | 逐条追加、祖先并集重写；墓碑 / 谱系走单条包装，读 diff 等价；lib 240 过 |
| K4.1 M3 判定 | 一致 | 阈值、方向、借量与 E176 M3 臂逐项相同，20.9 万步对拍逐步相同；先右后左、≤ 233 / < 117 是规则没写的细节 |
| K4.2 R3 | 一致 | 照抄判定、指针、key 对；「删空最左片新写槽数」在可达状态上是空形（计数 0），验收已移到吸收 |
| K4.3 N1 之后 I-9.12 | 一致 | I-9.12 三道仍拦所有会改路由的写错；少了的只是「容器号偏小」一类（用户认的代价）；kb I-9.12 行把 D8 的「创建时」引成「这一版」 |
| K4.4 模型与胶水 | 冲突 | harness 理想模型没有 M3；E176 对拍装置只对拍 M1、且读已删字段对改后 crates 编不过；crates 与 E176 M3 臂结构一一对应、非独立 |
| K5.1 各项取哪一版 / 一次发布几条 / W 罩不罩 | 一致 | 标签 6 取这次之后、W 与 ckpt_cost 取要接着的那一版；可达状态上一次发布至多 1 条；手造的连锁形 116 条仍在两片容器内；分配记录树那一项对「叶一起释放」没验 |
| K5.2 贴边建满后 unlink 不超卖 | 规则没说 | 「可用」的读法条款没写；操作结局读法下 384 槽小盘一族上造不出超卖，四项退役项分别或全去掉边都会移动、unlink 都照样发成 |
| K5.3 常驻 3 槽与实占同时在账上 | 规则没说 | 字面式子没有「减去已实占」一项，可用会多扣 3 槽 / 树；后果读码上是写入被拒、挂载与删除仍可用 |
| K6.1 I-9.31–35 CPU 合法镜像误报 | 一致 | 回退越过叶出生 / 删回退再删 / 借后吸收 / 被抛弃根的用例过；克隆头没有输入 |
| K6.2 GPU 与 CPU 同判 | 一致 | 读码一致，码 8 那一支已删（零命中）；GPU 有 256 / 4096 / 12 的容量上限，超了判不了交 CPU；没跑 GPU |
| K6.3 三类坏镜像各只红自己 | 部分冲突 | I-9.16 ④ 缺退役树另红 I-1.1（文档已写）；I-1.7 退役宽 34 没有坏镜像；kb invariants.md 第 32 行仍写谱系宽 16、没有 34 |
| K7.1 这批变异行能不能拦住改回去 | 一致 | 非 checker 档 98 行重放 98 行 RED-NAMED；20 行 checker 档没跑 |
| K7.2 照绿的写法 | 两处缺口 | 先右后左对调全绿（我加的定向用例才红）；I-1.7 对类型 3 跳过宽度比较全绿（224 + 4）；另三处等价变异 |

## 没做什么

- 没跑任何门禁阶段、checker 档测试、GPU、QEMU；没跑全量 `cargo test`。只在拷贝里跑了 harness 档与 core lib：`-p singlefs-core --lib`、`-p singlefs-core --test probe_m3_oracle`（我自己的探针）、`-p singlefs-harness` 的 `inode_leaf_absorb_and_borrow_on_the_unlink_path`、`checker_known_bad_images`、`rollback_lineage_record_and_its_checkpoint_cost_term`、`inode_number_watermark_pressed_back_reissues_a_number`、`accounting_tree_grows_a_level_when_inode_number_watermark_steps_cross_one_leaf`、`new_pool_file_creation_publish`、`pool_checker_judges_the_deletion_state_of_every_walked_version`、`unlink_idempotence_criteria_and_rollback_to_the_emptied_version`、`rollback_by_a_forward_publish`、`probe_k52`（我自己的探针）；188 个目标里其余的没跑。
- 变异表 20 行 checker 档（GPU 判器、`crash_judge_gpu.rs`、E165 bin、层 0 快档的两行 transaction.rs 行）没重放，那一档的证红在主 agent 手里；GPU 对拍没跑，K6.2 只是读码。
- 没造这些：克隆头镜像（`crates/` 没有克隆路径）；产品容量下退役树根满长高那一发布（要 12.9 万条退役记录）；「删 → 回退 → 再删」环下的超卖（C617 已记）；W 的分配记录树那一项对「116 片叶一起释放」够不够（没推没量）；容器号偏小（N1 代价）那类坏镜像；多文件 / 多次开新退役容器 / 4 GiB 盘上的贴边扫描（K5.2 只覆盖 384 槽小盘一族）；I-9.30 ③ 的缺口镜像（读码推理）。
- 没对旧版 `container_tree.rs` 做字节差分（读了 diff，等价是推的）；没有重算冻结副本的 sha256 清单。
- 不替主 agent 采纳或判决；本报告里「冲突」「缺」只是并排抄出两边。

## 要主 agent 跑的命令

无长活要交接。想复核我的探针：把 `research/prompts/m3-batch50-code-r1-sonnet-model/` 的 `probe_m3_oracle.rs` 与 `e176_model.rs` 放进冻结副本拷贝的 `crates/singlefs-core/tests/`，用 `README.md` 里那条命令跑，看末行 `PROBE deletes=209019 merges=466 borrows=3006 retired_total=9685 max_retired_per_delete=1` 与 `test result: ok. 25 passed`（用时约 42 秒）。想复核两处幸存变异：`variants/order.old|new` 套在 `crates/singlefs-core/src/inode_tree.rs`，`variants/retwidth.old|new` 套在 `crates/singlefs-checker/src/walk.rs`，各跑 `scripts/variant4.py` 里写的那几个目标。

## 草稿与编译目录的处置

见交回正文（删除记录与大小）。
