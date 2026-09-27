# m2-rollback-forward-r2 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

挑 `crates/singlefs-core/src/allocator.rs:38` 这条引用（原文整行 `    /// 仍分配时是分配代；已释放时是释放代。`），在草稿目录 `/tmp/claude-1000/m2-rollback-forward-r2-verifier/selftest/claim.txt` 里把行号人为 +1 改成 `:39`，按同一比对方法核：

```
=== 声称第 39 行是 ===
    /// 仍分配时是分配代；已释放时是释放代。
=== 快照里第 39 行实际是 ===
    pub generation: CheckpointTxg,
```

两者不同 ⇒ 判 ✗。核查方法能分辨错误行号，往下正式核。

## 一、输入核验

- 云端攻方报告 sha256：现算 `00e82bb40e0449ceab67b6b3eadddc18db66a57026b578ff92072a7f6675a217`，与交回时给的一致。
- 云端辩方报告 sha256：现算 `cc304e67c85ee03f1ad44569fe6cd8a82d019be76020f1c6105a52ffbea714f0`，与交回时给的一致。
- 快照清单：`crates-sha256.txt`（128 行，含 2 行注释、125 条文件）、`kb-sha256.txt`（9 行，含 2 行注释、7 条文件）；两份清单里点名的文件在 `/tmp/claude-1000/m2-rollback-forward-r2/tree/`、`/tmp/claude-1000/m2-rollback-forward-r2/kb-snapshot/` 下都能找到（本报告下文全部对这两份快照核，不对主树核）。
- Opus 模型目录 `research/prompts/m2-rollback-forward-r2-opus-model/SHA256SUMS`：`sha256sum -c` 6 个文件全部 `OK`。

## 二、云端攻方（Opus）报告核对

对象：`research/prompts/m2-rollback-forward-r2-opus-output.md`。全部引用对快照 `/tmp/claude-1000/m2-rollback-forward-r2/tree/`、`/tmp/claude-1000/m2-rollback-forward-r2/kb-snapshot/` 核。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `16-发布语义.md:33` 整行「可再分配」表格行 | ✓ 快照第 33 行字符级相同 | `awk 'NR==33' .../kb-snapshot/.claude/kb/decisions/16-发布语义.md` |
| `16-发布语义.md:29`（转述：保留最近4个可退到的不同状态…） | ✓ 第 29 行定案句里含这句转述的全部要素 | 同上，`NR==29` |
| `16-发布语义.md:36`（转述：抬F按今天的上限） | ✓ 第 36 行正是「抬 F 的上限」表格行 | 同上，`NR==36` |
| `16-发布语义.md:37` 整行「生效」表格行 | ✓ 字符级相同 | 同上，`NR==37` |
| `16-发布语义.md:38` 整行「回退候选集」表格行 | ✓ 字符级相同 | 同上，`NR==38` |
| `16-发布语义.md:44`（转述：树表里 inode/extent 根指针…） | ✓ 第 44 行「非空」定义句包含全部转述要素 | 同上，`NR==44` |
| `invariants.md:54` I-7.4（转述+定位） | ✓ 第 54 行确是 I-7.4 那一行，内容与转述一致 | 同上，`NR==54` |
| `invariants.md:59` I-7.9（转述+定位） | ✓ 第 59 行确是 I-7.9 那一行，内容与转述一致 | 同上，`NR==59` |
| `crates/singlefs-core/src/allocator.rs:38` 整行 | ✓ 字符级相同（即判别力自证用的那条） | `awk 'NR==38' .../tree/crates/singlefs-core/src/allocator.rs` |
| `crates/singlefs-core/src/allocator.rs:1318`（`record_root_written_by_this_process` 函数名定位） | ✓ 第 1318 行正是该函数签名 | `awk 'NR==1318' .../allocator.rs` |
| `crates/singlefs-core/src/mount.rs:1012`（转述：起算每块盘最新有效根、各盘取小） | ✓ 第 1012 行是 `.min()` 那一句，正是跨盘取小；上文 1010 行是各盘内取 max | `awk 'NR==1005,NR==1016' .../mount.rs` |
| `crates/singlefs-core/src/recovery.rs:1165` 整行 | ✓ 字符级相同 | `awk 'NR==1165' .../recovery.rs` |
| `23-journal的角色与格式.md:404`（转述：第五条与回退行的落点被打回重议那一条边角） | ✓ 第 404 行标题正是这句的出处（原文多一个「用户」限定词，转述省去，不算误引，见下方说明） | `awk 'NR==404' .../23-journal的角色与格式.md` |
| `23-journal的角色与格式.md:380`（「起」那一段回退见证） | ✓ 第 380 行正是「回退见证」段落的开头 | `awk 'NR==375,NR==380' .../23-journal的角色与格式.md` |
| `rbf2_attack.rs` 里 15 个测试函数名（X1/X7/X2/X3/X4/X5/X6/X8 与没打中各条引用的函数） | ✓ 15 个 `grep -c "fn <name>"` 均命中 1 次 | 逐个 `grep -c "fn $name" .../m2-rollback-forward-r2-opus-model/rbf2_attack.rs` |
| `rerun-full-crash.out`、`rerun-sweep.out` 里报告正文引用的原样行（第 129、146-147 行两处） | ✓ 与模型目录里的 `.out` 文件逐字节相同 | `cat .../rerun-full-crash.out`、`cat .../rerun-sweep.out` 与报告正文比对 |

说明：`23-journal的角色与格式.md:404` 原文标题是「⚠️ **第五条与回退行的落点被用户打回重议（设计问题，不是代码问题）**」，报告转述成「第五条与回退行的落点被打回重议那一条边角」，省去「用户」二字——这不是加限定词，是去限定词，且用在括注定位而非声称整行引用，判 ✓，仅在此列出供主 agent 参考。

### 复跑（fast / sweep 独立重跑；full 估时不跑）

- 命令（草稿目录）：`bash research/prompts/m2-rollback-forward-r2-opus-model/rerun.sh /tmp/claude-1000/m2-rollback-forward-r2/tree /tmp/claude-1000/m2-rollback-forward-r2-verifier/rerun-work sweep`（脚本内部已带 `nice -n 19` 与 `run-with-memory-cap.sh 16G` + `capped.sh 14`）。
- 结果：`diff <(sed -E 's/finished in [0-9.]+s/X/' 模型目录/rerun-fast.out) <(sed -E 's/finished in [0-9.]+s/X/' 草稿/rerun-fast.out)` 退出码 0（逐行相同，仅挂钟不同）；`rerun-sweep.out` 同样 diff 退出码 0。**判 ✓：独立重跑复现了模型目录里的 `.out`。**
- 起跑前 `ps -eo pid,ppid,pcpu,pmem,etime,args` 现查：命中大量 `checker_known_bad_images`、`second_transaction_*` 等 mutation-target 二进制（gate-triage 在跑变异表），未见 `qemu-system` / `vm-bench.sh` / `e152-file-system-benchmark` / `fio`，按规则可以 `nice -n 19` 照跑，报告里记下已看到的负载。
- `full`：模型目录 `rerun-full-crash.out` 记录该档单独耗时 `finished in 1768.06s`（约 29.5 分钟，无系统竞争时）。本次核查时系统在跑变异表，实测同一套 `fast`（脚本注释估 4 分钟）在本机跑到约 25 分钟、`sweep`（脚本注释估 10 分钟）跑到约 10 分钟——负载下明显变慢；若再加 `full` 档，估计总时长（`fast`+`sweep`+`full`，脚本 `full` 模式是累加着跑）会显著超过 40 分钟。**核不动：full 未复跑，理由是估时超过 40 分钟阈值**，不是判定它错。
- 交回前已 `rm -rf` 草稿目录 `rerun-work/repo/target/`（清理前 481M）。

### Opus 自述的「被攻过零轮」条目（主 agent 点名要核的三条）

| 条目 | 报告里出现的位置 | 是否标了「被攻过零轮」 |
|---|---|---|
| 「同槽同分配代」那道核（原型第 4 步的核） | 第 46 行小节标题（「原型的最小规格……被攻过零轮」）覆盖全部原型条款，含这道核；第 335 行另说明它「是读法，不是条款」且被 X2 打中一处 | ✓ 是（section 级标注，X2 打中不改变它未经第二条腿复核这一事实） |
| 释放集合只取用户可见单元（原型第 4 步「释放」定义） | 同上第 46 行小节标题覆盖；第 335 行说明被 X1 打中一处 | ✓ 是（同上） |
| `max+hold`（这条腿自己提的改法） | 第 66、223、247、335 行四处均逐字写明「只在我的模型上量过、被攻过零轮」 | ✓ 是，且比前两条标得更直接、更多次 |

**Opus 报告计数**：核了 21 处（14 条文件:行引用 + 1 批 15 个测试函数名 + 1 批产物原样行 2 处 + fast 复跑 + sweep 复跑 + full 估时）。✓ 20，✗ 0，核不动 1（full，估时超阈值未跑）。

## 三、云端辩方（Sonnet）报告核对

对象：`research/prompts/m2-rollback-forward-r2-sonnet-output.md`。该腿没有模型目录（未写代码、未跑测试），全部引用是对冻结副本源码与第一轮三腿报告/判决原文的现读，逐条核如下。

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `mount.rs:576`（`abandoned_by_table` 函数定义） | ✓ 第 576 行正是该函数签名 | `awk 'NR==576' .../mount.rs` |
| `mount.rs:790`、`:991`、`:1155`、`:1171`（`abandoned_by_table(` 的四处调用） | ✓ `grep -n "abandoned_by_table("` 全仓（不含定义）恰好命中这 4 行、不多不少 | `grep -n "abandoned_by_table(" .../mount.rs` |
| `mount.rs:785-791`（`is_abandoned` 闭包范围） | ✓ 785 行正是 `let is_abandoned = ...` 开头，791 行正是闭包的 `};` | `awk 'NR==783,NR==793' .../mount.rs` |
| `mount.rs:787`（闭包内 `rollback_witness.abandons(...)` 那一支） | ✓ 字符级相同 | 同上 |
| `mount.rs:801-811`（`ShadowLedger::On`/`Off` 两支） | ✓＊范围内容对（On 在 802 行、Off 在 810-811 行都在范围内），但整个 `match` 块实际收尾在第 814 行的 `};`，引用范围比实际收尾短 3 行——不影响所指内容，标注供参考，不判 ✗ | `awk 'NR==799,NR==817' .../mount.rs` |
| `mount.rs:742`（`rebuilt_allocator` 函数定义） | ✓ | `awk 'NR==742' .../mount.rs` |
| `mount.rs:2537-2540`、`:2537-2548`（候选集校验块范围） | ✓ 范围边界精确（2537 行 `if newest_table` 起、2548 行该 `if` 块的 `}` 止） | `awk 'NR==2530,NR==2550' .../mount.rs` |
| `mount.rs:2491-2648`、`:2491`（`mount_rollback_with_space_admission` 函数体） | ✓ 2491 行是函数签名；下一个顶层 `fn` 在 2655 行，与 2648 行 `}` 一致 | `grep -n '^fn \|^pub fn ' .../mount.rs`（限定行号范围） |
| `mount.rs:2546`（`OnAbandonedTimeline` 「唯一调用点」） | ✓ 全仓 `grep -rn "OnAbandonedTimeline"` 在 `crates/singlefs-core/src/` 下只有两处：`mount.rs:257`（枚举定义）与 `mount.rs:2546`（唯一构造/调用点）；harness 测试与 model.rs 的同名/相关标识符不算生产代码调用点 | `grep -rn "OnAbandonedTimeline" .../tree/crates/` |
| `mount.rs:257`（`OnAbandonedTimeline` 枚举定义） | ✓ | `awk 'NR==257' .../mount.rs` |
| `mount.rs:604`、`:606`（两处注释片段，非整行声称） | ✓ 引用片段是这两行注释的精确子串 | `awk 'NR==604' .../mount.rs`；`awk 'NR==606' .../mount.rs` |
| `mount.rs:1896`（`rollback_witness_tables_of_this_mount` 定义）、`:1925`（`this_rollback` 赋值） | ✓、✓ | `awk 'NR==1896' .../mount.rs`；`awk 'NR==1925' .../mount.rs` |
| `mount.rs:2055`（`establish_instance` 定义）、`:2163`（调用点） | ✓、✓ | `awk 'NR==2055' .../mount.rs`；`awk 'NR==2163' .../mount.rs` |
| `mount.rs:2367`（`mount_writable_with_space_admission` 定义） | ✓ | `awk 'NR==2367' .../mount.rs` |
| `mount.rs:2620`、`:2624`、`:2625`（`PreviousInstanceRow{ is_rollback: true, .. }` 唯一置位处） | ✓ 三行内容与引用逐字相同 | `awk 'NR==2618,NR==2626' .../mount.rs` |
| `recovery.rs:669`（`choose_root` 定义） | ✓ | `awk 'NR==669' .../recovery.rs` |
| `recovery.rs:681`（`if rollback_witness.abandons(...) {` 整行） | ✓ 字符级相同 | `awk 'NR==681' .../recovery.rs` |
| `invariants.md:54`（I-7.4，判据文字「按实例表判仍然有效」焊进判定对象集合） | ✓ | 同第二节 |

| `r1-main-verification.md:43`（F3 表格行，判「还要」，引 H7） | ✓ 字符级相同（这正是主 agent 点名要核的第一句） | `awk 'NR==43' research/prompts/m2-rollback-forward-r1-main-verification.md` |
| `r1-main-verification.md:44`（H6 那一行判词） | ✓ 字符级相同 | `awk 'NR==44'` 同上 |
| `r1-main-verification.md:45`（I-7.4 判词） | ✓ 字符级相同 | `awk 'NR==45'` 同上 |
| `r1-main-verification.md:41`、`:42`（回退见证「可以删」、实例表回退行「可以删（推的）」） | ✓、✓ | `awk 'NR==41,NR==42'` 同上 |
| `r1-opus-output.md:27`（H7 自己的结论「未打穿到读错……两道都删的那一格没造」） | ✓ 字符级相同 | `awk 'NR==27' research/prompts/m2-rollback-forward-r1-opus-output.md` |
| `r1-opus-output.md:190`（`t8b` 历史描述 + 「新根引用一个账里空闲的槽……零故障」） | ✓ 两处引文字符级相同 | `awk 'NR==190'` 同上 |
| `r1-opus-output.md:175-182`（H6/`t7c` 历史范围） | ✓ 范围内容与报告转述一致 | `awk 'NR==175,NR==182'` 同上 |
| `r1-opus-output.md:181`（D 落 50184/50186 两个数） | ✓ 字符级相同 | `awk 'NR==181'` 同上 |
| `r1-opus-output.md:184`（50184 读失败） | ✓ 内容一致（`MappingStillUnreadable { slot: SlotNumber(50184) }`） | `awk 'NR==184'` 同上 |
| `r1-opus-output.md:192-194`（H12/`t7` 历史范围） | ✓ 范围与内容一致 | `awk 'NR==192,NR==194'` 同上 |
| `r1-opus-output.md:194`（「Opus 的原话是『影子账开 / 关两臂逐字相同，今天的冻结副本同样中』」） | **✗ 引文不是该行原样**：第 194 行原文是「影子账开 / 关两臂逐字相同，**冻结副本**同样中」，没有「今天的」三字；报告把「今天的」三字算进了「原话」引号里。「今天的」三字实际出自另一处被并列引用的 `r1-main-verification.md:54`（那一行确实写「今天的冻结副本同样中」）——报告把两个不同来源的说法拼成一句话，标成对 `r1-opus-output.md:194` 的逐字引用 | `awk 'NR==194' research/prompts/m2-rollback-forward-r1-opus-output.md`；`awk 'NR==54' research/prompts/m2-rollback-forward-r1-main-verification.md` |
| `r1-main-verification.md:54`（H12 越格段落，含「今天的冻结副本同样中」） | ✓ 字符级相同（这条本身没问题，问题出在上一条把它与 opus-output.md:194 混成一句） | `awk 'NR==52,NR==54'` 同上 |
| `abandoned_by_table` 「有四处调用，只有一处与管理员回退路径相关」，第四处点名 `mount.rs:2537-2540` | **✗ 归类不准**：`mount.rs:2537-2540` 那段代码（`newest_table.rows.iter().any(|row| row.instance == target.instance && target.checkpoint_txg > row.selected_root_txg)`）与 `abandoned_by_table` 函数体逻辑等价，但**不是**对 `abandoned_by_table(...)` 这个具名函数的字面调用——全仓 `grep` 该函数的字面调用只有 4 处（790/991/1155/1171），2537-2540 是手写的等价复刻，不在这 4 处调用之内。报告把它算作「有四处调用」里「与管理员回退相关的那一处」，与代码字面不符；**不改变报告下游的实质判断**（`OnAbandonedTimeline` 连同 `mount_rollback_with_space_admission` 一起被删、新函数清单缺一条等价排除这一结论仍然成立，因为不论 2537-2540 是字面调用还是等价复刻，它都随该函数一起被删） | `grep -n "abandoned_by_table(" .../mount.rs`；`awk 'NR==2537,NR==2540' .../mount.rs` |

**主 agent 点名要核的两句**：
1. `r1-main-verification.md:43` 引 H7 引错了——**核实：报告对 43 行原文的引用逐字相同（✓），且 H7（`r1-opus-output.md:27`、`:190`）自己的场景确实是「向前回退到 C」这一次管理员发起的回退，与 43 行括注写的理由「与管理员回退无关」字面相反；43 行那句理由文字（「崩溃恢复落到旧根、实例表抛弃较新的根照样发生，与管理员回退无关」）逐字见于 `r1-main-verification.md:44`（H6 的判词），不是 H7 自己的判词。Sonnet 指出的错位属实。**
2. `OnAbandonedTimeline` 在冻结副本里唯一的调用点是 `mount.rs:2546`——**核实：属实**，全仓只有 `mount.rs:257`（枚举定义）与 `mount.rs:2546`（唯一构造点）两处，`:2546` 落在 `mount_rollback_with_space_admission`（`:2491`–`:2648`）内，该函数确实是正文第一节点名要删的对象。

**Sonnet 报告计数**：核了 38 处引用。✓ 36，✗ 2（`r1-opus-output.md:194` 引文多算入「今天的」三字；`abandoned_by_table` 「四处调用」把 `mount.rs:2537-2540` 的等价复刻代码算作字面调用）。

## 四、本地攻方核对

第一问（task one，16 格表）两次调用（原提示 s1、拆分后 part1 s1）均退出码 3、输出 0 字节，按派发指令已停，本报告不核第一问内容，只核下面两份核对表本身的出处行号与译文。第二问（task two，B1 卸载串空发布次数）两份干净样本核算术；两份核对表（`translation-audit.md`、`split-audit.md`）逐行核出处。

### 4.1 第二问算术复核

| 样本 | 结果 | 核法 |
|---|---|---|
| `-part2-output-s1.md` | ✓ 三种 `t mod 3` 情形（0/1/2）算出的区域、盘、总数（2/3/2）与题面公式（`region = txg mod 3`；区域 0/2→盘0，区域1→盘1）手算结果完全一致；词数 `wc -w` = 129，与 runlog 记的一致 | 用题面公式（Clause 1-4 给的区域数、轮转公式、区域-盘归属）独立重算三种情形 |
| `-part2-output-s2.md` | ✓ 同上，且与 s1 结果一致（两份干净样本互相印证，未见字词损坏） | 同上，`wc -w` = 122，与 runlog 一致 |

### 4.2 `translation-audit.md` 引用表（9 行）核对

| 行 | 引用 | 结果 |
|---|---|---|
| Clause 1 | `16-发布语义.md:52`「F 只住在根记录里……可以只有一条带新 F 的根」 | ✓ 快照第 52 行含此句，且表格如实标注「故意只译到分号为止」的截断点，截断处确实是原文分号 |
| Clause 2 | `16-发布语义.md:37`「每块幸存盘上都有……最大值的最小值」 | ✓ 与第二节已核的 37 行一致 |
| Clause 3 | `16-发布语义.md:38`「按实例表判仍然有效……F_生效」 | ✓ 与第二节已核的 38 行一致 |
| Clause 4 | `_m2-rollback-forward-r2-body.md:15`；`r1-main-verification.md:32` | ✓ 两处引文分别与 body.md 第 15 行、判决第 32 行字符级相同 |
| Clause 5 | `22-单元原子性怎么合成.md:55`「3（R = F + 1，F = 2）」 | ✓ 字符级相同（markdown 加粗符号已按惯例剥离，不算改字） |
| Clause 6 | `22-单元原子性怎么合成.md:57`「跨区轮转……已定项16」 | ✓ 字符级相同（同上剥离 `**`/反引号） |
| Clause 7 | `16-发布语义.md:189` 两处引文 | ✓ 两处引文都在第 189 行原文里逐字可查（表格如实说明截断点：删去「两次正好覆盖两块盘」一句、删去「写行那次发布的 txg 取…」一句） |
| Clause 8 | `r1-main-verification.md:29`「B1：卸载另立上限……两块盘各一份」 | ✓ 字符级相同（与第二节已核的 29 行一致） |
| State 设定 | `r1-main-verification.md:31`「盘 1 上带新 F 的根一坏」、`:32`「抬 F 那一串崩在盘 1 那一次之前」；`16-发布语义.md:52` | ✓ 三处子串均逐字见于对应行 |

### 4.3 `split-audit.md` 段落分配表（26 行）核对

对 `research/prompts/m2-rollback-forward-r2-local-attack.md`（原提示，74 行）逐行核，表中列出的 26 处行号（:3、:5、:7、:9-10、:12-13、:15-16、:18-19、:21-22、:24-25、:27-28、:30-31、:33、:35、:37-40、:42-52、:54、:56、:58、:60、:62、:64、:66、:68、:70、:72、:74）**全部现查**：每一处的行内容都与表格「原提示片段」列描述的主题一致（背景段、Clause 1-8、State 设定、Part 3/4/5/6 标题与正文、Question 1-3、Formatting rules），编号改写（Clause 5→1、6→2、7→3、8→4）与表格「处理」列描述的机械改写规则逐条吻合。**✓ 26/26**。

**本地攻方计数**：核了 2（算术）+ 9（translation-audit）+ 26（split-audit）= 37 处。✓ 37，✗ 0，核不动 0（第一问内容按指令不核，不计入本计数）。

## 五、总计

| 腿 | 核了 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| 云端攻方（Opus） | 21 | 20 | 0 | 1（full 复跑，估时超 40 分钟阈值未跑） |
| 云端辩方（Sonnet） | 38 | 36 | 2 | 0 |
| 本地攻方 | 37 | 37 | 0 | 0（第一问内容按指令不核，不计入） |
| 合计 | 96 | 93 | 2 | 1 |

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑。
- 不判 Sonnet 报告第一节「按挂载数隔离窄化挡不住 H6」这条论证本身对不对——只核它引的行号与转述准确。
- 不判 Opus 报告里 X1-X8 各条「打中」的判定是否成立——只核它引的条款、代码行、测试名与产物原样行。
- `full` 档没有复跑：估时（fast 在本机负载下实测约 25 分钟、sweep 约 10 分钟、加上模型目录已记录的 full 单档 1768 秒）合计会明显超过 40 分钟阈值，按规则不跑，照实写在第二节。
- 没有重新验证 Opus 报告第八节「这条腿自己的限度」、第九节「没做什么」里的自述内容是否完整（例如「几何只有两块 4 GiB 盘」「没有真并发」这类范围声明），只核了主 agent 点名的「被攻过零轮」三条标注是否存在。
- 没有重新验证 Sonnet 报告「没做什么」一节里「读 `_m2-rollback-forward-r2-background.md` 第 1-250 行、因单文件读取上限被截断」这一自述细节（无法从外部核实它当时读到了第几行）。
- 没有编译、没有跑门禁、没有跑层 0 / QEMU / 全量 `cargo test`；只用 `run-with-memory-cap.sh 16G` + `capped.sh 14`（脚本自带）跑了 `fast`、`sweep` 两档 `rbf2_attack` 测试的独立重跑。
- 没有读禁读清单（本轮无专门禁读清单给核查员，按各腿报告自述的禁读范围间接确认它们互相没有读对方的产出）。
- 交回前已删除草稿目录 `/tmp/claude-1000/m2-rollback-forward-r2-verifier/rerun-work/repo/target/`；发现一个自己起的、条件永假的后台等待循环（等 `run.log` 里出现从未会写入的 "EXIT=" 字样），已用 `proc.py stop` 逐层停掉，交回前 `ps` 复核无残留。
