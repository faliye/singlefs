# 第 30 批代码轮第一轮判决（2026-10-05）

<!-- doc-lint:not-numbers M1 M2 M3 M4 E1 B1 B2 B3 B4 F1 F2 G1 KX1 KX2 -->

被判的文件（第 30 批三份补丁动到的全部 `crates/`，逐份见改动全文 `research/prompts/_m3-formal-code-b30-r1-diff.md`）：
- 30-1：`crates/singlefs-harness/tests/model_compares_the_third_and_fourth_inode_leaf_containers.rs`；
- 30-2：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_gpu.rs`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`；
- 30-3：`crates/singlefs-core/src/system_configuration.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`、`crates/singlefs-harness/tests/system_configuration_mutability_classes.rs`、`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs`；
- `crates/mutations.tsv` 三份追加与替换，另主 agent 改一行名字。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b30-r1-body.md`、`_m3-formal-code-b30-r1-diff.md`、`_m3-formal-code-b30-r1-background.md` | — |
| 冻结副本与快照 | `/tmp/claude-1000/m3-formal-code-b30-r1/tree/`（第 30 批合入后），清单同目录 309 行；kb 快照 `research/prompts/m3-formal-code-b30-r1-snapshot/kb-sha256.txt` | 腿交回后核过 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b30-r1-sonnet-output.md`（sha256 c2caeb39…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh all` 复跑 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b30-r1-opus-output.md`（sha256 34662d94…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh all` 复跑 |
| 引文 | 两份腿报告 | 主 agent 各跑一次 cite-check：24 / 24、29 / 29 |

## 二、逐格判决

### M1　30-1

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 三条用例对四个入库变体 | 一致：四行各被点名用例抓到 | 没打中：放开回退目标与回退后建数，20 段历史上凡分辨得了的取样点都红 | 站住 |
| 「第 k 片都对」再外一档 | 冲突：胶水封顶第 4 片、回退只取回第 2–3 片、回退只丢第 3–4 片在三条新用例与 133 个宽测试集上全过，5 片的格才抓得到——今天只罩到 k = 4 | 打中：同形三个变体，只有建到第 5 片的取样点抓得到 | **两腿同判，坐实**（主 agent 复跑：正推 `RESULT 100 SURVIVED passed=133`、`102`、`103` 同；攻方各段 45 / 48 份逐行同，另 3 份差在打印与规模、不在结局）。b29 L1 那一形往后挪了一片：按取样点逐片往外加用例追不完。改法 B3（胶水映射逐片单测，k = 0..134 互不相同，攻方副本上量过两个胶水变体都红）与 B4（模型回退的结构单测，推的），零轮，并进第 32 批 |
| 锚点互相掩护 | 一致 | 没打中 | 站住 |

### M2　30-2

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| E1 合法历史误红 | 没见：7 309 次检查（删除进行中、回退、可写挂载）新旧逐行同 | 没打中：四组比重 168 段历史 7 728 张镜像 I-2.4 违例 0，探针臂证 1 704 / 2 604 张走到了墓碑树 / 谱系树容器子指针 | **两次独立抽样都没打中**（复跑：攻方探针臂规模小一档，372 张里 275 张走到那种指针，结论同） |
| 走读跟随而不判 extent 偏移的指针 | 一致；另记 12 条旧树表根指针只被 I-9.28 读、不在走读里 | 没打中（逐调用点） | 站住；那 12 条指针不是走读跟随的，I-2.4 那一行没管到它们，归写回批时把 I-2.4 的指针清单写成实测清单（b29 判决的 E2）一并写明 |
| 池级 checker 不判指针头的算法类型、压缩码、压后长度 | 没提 | 打中：core 拒的三种坏盘上池级 checker `violated=[]`、I-2.4 成立（CPU 量过，GPU 走读读码） | **坐实为条款外的缺口**（主 agent 现查 `.claude/kb/invariants.md` 第 133 行 I-2.4 只写指针头部 MAC / nonce 与 extent 偏移；池级 checker `crates/singlefs-checker/src/image.rs:1311`、`:1355` 也只判这两段）。不是实现违反条款。改法 F2（池级 checker 把指针头部前 32 字节的加密 / 压缩几段一并按第一版取值判，攻方副本上量过：坏盘都红、入库坏镜像全过、合法批 0 误红），零轮；I-2.4 那一行要加这几段，归写回批；并进第 32 批，GPU 走读内核同改 |
| GPU 恢复内核种类 ≥ 32 与加密 / 压缩 8 处 | 一致（推的；28 671 个头谓词翻译 0 分歧） | 没打中（读码） | 站住到 GPU 对拍跑过为止 |
| I-5.4 按版本：合法历史 | 一致（新旧逐次同） | 没打中（0 次多出部分没判） | 站住 |
| I-5.4 按版本：坏盘 | 一致（清零 26 个单元时 10 个由 Holds 变 PartiallyJudged） | 打中：把一版的树表（B 的 50264、A 的 50252、第 0 代 50178）清零，I-5.4 仍报成立而那一版的分配记录一格没判——新旧判法共用的缺口 | **坐实**（攻方副本量过，主 agent 复跑 `i54-units-with-f1.log` 同）。改法 F1（树表读不出的候选根改报部分没判；攻方副本上量过：三张镜像改报部分没判、入库坏镜像 89 条全过、合法批 0 次），零轮，并进第 32 批 |

### M3　30-3

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 三处同单位同值 | 一致 | 没打中：T_time 取 5 种值时池级 checker 与恢复判定逐字不变 | 站住 |
| 全仓按秒解偏移 433 | `crates/` 零读者；E142 已改（第 23、24 次）；E170 合成池写 5、只解不判 | 没打中 | 站住；E170 合成池不动（执行器不判这一格） |
| 三个新钉值出处 | 一致（独立纯 Python CRC-32C 重算同值：整盘 CRC 旧 4048568891、新 1329175690） | 没打中：单位换回秒后两个槽摘要与整盘 CRC 都等于旧钉值 | 站住（复跑 `SLOTCHECK … only_433_437_and_155_159=True` 两行同） |
| checker 档那几个钉值 | 读码一致；`crash_enumeration_sharded_across_processes.rs` 三个常量要重取 | 打中重取法：整份取 `left` 时，T_time 带来的计划哈希变化与 30-2 可能带来的 `GOLDEN_TALLY` 计数变化分不开 | 采纳攻方的重取法：重取时逐行比新旧 58 行，只许 `LAYER0_RESUME` 那一行（计划哈希）不同；别的行也变，先查是不是 30-2 改了判定集合。转根环线（它点名跑这一条） |
| T_time u32 毫秒的两端 | 规则没说（约 49.7 天上限） | 打中一格格式空白（推的）：4 字节毫秒写不出 1 ms 以下与约 49.7 天以上，D16 已定项 5 写「不限制、不拒绝」 | 坐实为条款空白；今天没有用户设值的路（core 只写默认值），不挡这一批。记欠账，交用户定（第五节第 1 条） |

### M4　主 agent 两处

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 变异行改名 | 结局一致：行键含名字，三个行键互不相同，旧记录不复用也不错用；但正文那句「锚点与替换文不动」只相对 30-3 补丁成立——那一行的锚点早已由 30-3 从 SECONDS 换成 MILLISECONDS | 没打中：改名只触发重跑 | 站住；正文那句的射程更正为「相对 30-3 补丁合入后的那一行」 |
| 字段表第 149 行与 D16 | 一致；D16 正文没有盘上单位，字段表那一行是单位的唯一落点 | 打中两端（见 M3 最后一行） | 站住 |

## 三、去向

1. **第 32 批实现员**：B3（胶水映射逐片单测 k = 0..134）与 B4（模型回退结构单测）；F2（池级 checker 与 GPU 走读内核判指针头部加密 / 压缩几段按第一版取值）；F1（树表读不出的候选根 I-5.4 改报部分没判）；`overwrite_in_one_instance.rs` 812 用例的说明与判法（第 31 批记下的那一件）。
2. **写回**（并进写回批）：I-2.4 那一行加指针头部算法类型、压缩码、压后长度，并写实测的指针清单（含 E1 之后与那 12 条只被 I-9.28 读的旧树表根指针）；I-5.4 那一句照 F1 写；欠账一笔：T_time 毫秒字段的两端。
3. **转根环线**：`crash_enumeration_sharded_across_processes.rs` 三个钉值的重取法（逐行比 58 行，只许 `LAYER0_RESUME` 变）。
4. **下一轮代码轮**（第 32 批合入后）：攻 B3 / B4、F1、F2。

## 四、复跑核对

攻方：模型目录拷到会话草稿 `b30-opus-rerun/model/`，`CAP=6 bash model/tools/rerun.sh <草稿>/work all` 退 0。与模型目录 `out/` 同名文件按 `B30*`、`test result`、`SWEEP`、`test … ok/FAILED` 行比：48 份里 45 份相同；`baseline.log` / `baseline.txt` 差在复跑没带 `--nocapture`、SWEEP 行没打出（20 条用例 `test result: ok. 20 passed`，结局同）；`i54-units-and-time.log` 少 B30PIN 那一条（`rerun.sh` 的 m3-pins 一步把它单写进 `slot-digests.log`，复跑那一份里两行 `B30PIN … old_equals=true differing_offsets=[155, 156, 157, 158, 433, 434]` 与 `image-crc.log` 的 `B30CRC today=1329175690 reverted=4048568891` 都与报告同）；`e1-probe-arm-batch1` 复跑规模 372 张、腿那一趟 2 604 张（摘要文件只有腿那一份），「探针走到容器子指针」在两边都成立。

正推：`REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b30-r1/tree WORK=<会话草稿>/b30-sonnet-rerun RERUN_ALREADY_WRAPPED=1` 经 `run-with-memory-cap.sh 8G` 与 `capped.sh 6` 跑 `rerun.sh all`，退 0：setup `sha_not_ok=0`；变异结局与报告同（入库 0–6、16–18 `CAUGHT`；腿加的 100、102、103 在宽测试集上 `SURVIVED passed=133`、101 被 135 边界单测抓到，在三条新用例上 100–103 `SURVIVED passed=3`、104 `CAUGHT`）；`SLOTCHECK` 两行 `only_433_437_and_155_159=True`、新摘要 e5cc28ab / 144c1785；m2-scan 清零镜像的 `differing_invariants` 行与报告同。其余探针段（m2-sweep、m2-heads、m2-kind、m2-i54、m3-433、m3-greps、m4-keys）复跑退 0，汇总行的格式与腿报告逐数比没做，只核了退出码。

核不动：GPU 对拍与 GPU 变异、checker 档钉值、`crates/mutations.tsv` 整表（两腿都没跑）。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | T_time u32 毫秒写不出 1 ms 以下与约 49.7 天以上，D16 已定项 5 写「不限制、不拒绝」：这两端挂载时怎么处置 | 攻方，正推同记 | 推的（今天没有用户设值的路） | 记欠账，等实现「挂载时传 T_time」那一步交用户定 |
| 2 | B3 / B4 结构单测 | 攻方 | B3 副本量过、B4 推的 | 第 32 批 |
| 3 | F2 指针头部加密 / 压缩几段 | 攻方 | 副本量过 | 第 32 批 + 写回 I-2.4 |
| 4 | F1 树表读不出的候选根 I-5.4 报部分没判 | 攻方 | 副本量过 | 第 32 批 + 写回 I-5.4 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 5 | 备料 | 2026-10-05 不受影响：毫秒字段两端写不出的值怎么处置是条款空白，记欠账，定案句不改 |
| D19（块指针的结构与宽度预算） 已定项 3 | 备料 | 2026-10-05 不受影响：指针头部字段表不变，F2 只让池级 checker 照它判全；I-2.4 的措辞归写回批 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-10-05 不受影响：I-5.4 改法 F1 只改判不了时的报法 |
