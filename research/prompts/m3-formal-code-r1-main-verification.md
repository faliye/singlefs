# 形式化验证第 7–10 批合入的代码：代码轮第一轮判决（2026-10-02）

<!-- doc-lint:not-numbers M1 M2 M3 M4 O3 R3 H08 H09 C1 C2 C3 C4 C5 C6 C7 K1 K2 K3 K4 XE1 XE2 XE3 J1 J6 J7 Qn G1 P0 P4 P5 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 改动全文 | `research/prompts/_m3-formal-code-r1-body.md`、`_m3-formal-code-r1-background.md`、`_m3-formal-code-r1-diff.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-r1-snapshot/`：`crates-sha256.txt`（冻结副本 `/tmp/claude-1000/m3-formal-code-r1/tree/`）、`kb-sha256.txt` | 腿交齐之后冻结副本 `sha256sum -c` 全 OK；kb 快照全 OK |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-r1-sonnet-output.md`（359 行，sha256 9b3d72e6…），模型 `…-sonnet-model/`（只读检查脚本） | 只读码，没编译 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-r1-opus-output.md`（296 行，sha256 353a9a3f…），模型 `…-opus-model/`（66 个文件） | 主 agent 在 `/tmp/claude-1000/m3-formal-code-r1-main-check/` 照报告「复跑」一节复现两条：K1-b——不打补丁时它补的用例 `attack_root_choice_txg_beats_instance_through_the_glue`「test result: ok. 1 passed」，打 `probes/g1-glue-swap.patch` 后「test result: FAILED. 0 passed; 1 failed」；K2-a——`ATTACK_CASES=3000000` 跑出「name=fixed-point-brute cases=3000000 first_ok=2631573 premise_cases=1790678 premise_cases_without_i812=881122 violated_with_i812=0 violated_without_i812=4492」，与报告同。「G1 下 harness 档 742 条、core 库 141 条全绿」没复现（整包 harness 在 40–130 的负载下跑，主 agent 没跑），只当攻方的数引 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本> --root . --background <背景材料>`：「✓ 核了 119 处引文，对上 118 处，没判 1 处」；没判的那处是正推第 263 行引 `e165_replay_bounded_enumeration.rs:1366`（只写了文件名），主 agent 现查冻结副本那一行是「/// 多盘域任一份在即在（登记 r2 5.3，…）」，与它引的原文同 |

正文有两处写错，两腿都点出：「journal 环第一版 1024 槽」（1024 是 journal 环起点槽号，`crates/singlefs-format/src/lib.rs` 第 216 行；默认环每盘 196 608 个记录槽）；「I-8.12 还没登进 `invariants.md`」（`.claude/kb/invariants.md` 第 101 行已有，状态未实现）。

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1-a 胶水与改前同义 | 对上：读槽次序与次数、自证失败的跳法、比较键、严格大于都同义；`scan_journal` 扫环循环没动，先查后插与 `or_insert` 同义 | 没打中：随机根环 5 万态、随机扫环 5 万态，结局与读盘序列 0 处不同 | **站住** |
| K1-b 胶水摊择新键这一步谁罩 | 胶水的摊入次序与下标没有变异行；「胶水拿所选根调重放」那一步也没有（`&chosen_root,` 在 `crates/mutations.tsv` 0 命中） | **打中**：胶水把择新键两项摊反（实例代号为主），只有它补的用例红（主 agent 复现）；按读码推，合法历史里两种择新序不分叉，层 0 也罩不住 | **打中，复现坐实**：攻方那条用例（经 `choose_root` 钉「txg 为主」）进 `crates/`，`crates/mutations.tsv` 加「胶水摊反择新键」一行（今天第 1827 行只管「不带实例代号」，管不到对调）与「胶水拿所选槽之外的根调重放」一行，下一批实现 |
| K1-c Vec 收齐再建表的内存 | 取舍不是条款问题；峰值约为盘数倍（推的） | 量出形状：两盘 2.01 倍、四盘 3.93 倍（8192 槽）；外推默认环改前约 0.92 GB、两盘约 1.86 GB、四盘约 3.63 GB（线性外推，推的） | **记欠账**：功能不错，代价随盘数涨；改法（核逐盘交、先到先得只证单步，或胶水逐盘合并）要在「证明强度」与「内存」之间取舍，这一轮不改，登记新欠账、写明量过的形状与外推 |
| K1-d 键完全相同的槽取哪个 | 核一侧对上，由严格大于、spec 第三句、单测、两条变异行钉住 | 半中：核取前一个；`newest_root_among` 文档写「与 choose_root 同一个择法」，实现 `max_by_key` 取后一个（量过）；合法历史里没造出 | **打中一处自相矛盾**：`newest_root_among` 改成与核同一择法（或改文档），下一批实现；条款没写平局取哪个，写回时补进 D22 已定项 24（零轮） |
| K2-a M3 不动点前件 | 对不上：spec 多 I-8.12 一条前件，D13 已定项 1 M3 一行没有 | **打中，已知形状独立复现**：只满足条款字面前提的 881 122 态里 4 492 态不成立，全在 I-8.12 不成立的表上；加 I-8.12 的 909 556 态 0 反例（主 agent 复现） | **打中条款，不打代码**：D13 已定项 1 M3 一行的不动点前提补「I-8.12（同一实例里 journal 记录的 txg 随计数器不减）」，写回；与 E165 r2 对抗一族的结论（10-5 对账，见调度记录）同向 |
| K2-b 合判推论会不会空真 | 没发现空真；推论新证出来的只有序的传递，内容薄；缺一行「去掉 J1 前提」的必红世界 | 没打中：真调两个核、交回值原样交给推论，57 verified 0 errors | **站住**；必红表补「去掉 J1 前提」一行，下一批 |
| K2-c 必红表 99 行 2 → 3 | 理由只是实现员推测；这一行期望两批里反向改过 | 理由站得住，但 M 记的是求解上下文：10-3 之前的核上只加一个没人用的 spec fn，99 行就 2 → 3，多出的就是在飞上限那条循环不变式；今天换线程、rlimit、无关改动 M 都稳定在 3 | **站住，记欠账**：「期望只数证明错」（用户定案 U4）下，条数会随无关的 spec 改动跳；登记新欠账（必红世界的期望条数随求解上下文漂），这一轮不改判法 |
| K2-d 必红表 125 行 2 → 3 | 站得住（择根与重放共用 S5 序） | 站得住：多出的一条落在 `choose_root_slot` 的循环不变式上 | **站住** |
| K3 变异行与用例 | 静态全过；R3-1 投影之外四个字段只有卸载记号有变异行（第 1650 行），另三处拷贝没有；S03 / S07 补的是敏感取样点（`.claude/rules/mutation-sampling.md` 第三类），依赖「读缓存不收全零块」；第 173、174、177 行今天各命中 1 次；第 1830、1831 行（E168）锚点 0 命中 | 这一轮相关 48 行锚点都恰好命中一次；第 173、174、177 行同 | **站住**；R3-1 另三处拷贝（`instance_table`、`filesystem_identifier`、`allocation_record_tree_root`）补变异行，下一批；第 1830、1831 行不归这一轮，记进收拢表、告诉持有 E168 的会话 |
| K4 装置 | E165 r1 档 `SELF_CHECKS` 里 `check_the_fixed_point_control` 取真结局，仍可能让变体启动时倒（r1 第二段变体都跑完了，没有反证）；多盘一族代表实现的是「照 `device_identities()` 次序」那一栏；E170 Q3-f 的 28 / 66 是上界（补丁不进任何发布，`root_without_record` 76 行里 0 次置位） | 没攻；读一处，同正推多盘那一条 | **站住，补三句**：E165 页写明代表实现的那一栏；E170 页 Q3-f 写明是上界；r1 档那一条自检记进 E165 第 3 次重跑的注意（它的装置也走 r1 档的启动路径） |
| C7 必红表与 verified 数 | 对上：153 行锚点全命中 1 次、登记数 56、主 agent 的 `rows.out` 153 行 match | — | **站住** |

被点名的 `crates/*/src/*.rs`：`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-replay-prefix/src/lib.rs`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`、`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`。

## 三、打中的改法（零轮，下一批实现，改完开代码轮第二轮）

1. K1-b：攻方用例进 `crates/singlefs-harness/tests/`；`crates/mutations.tsv` 加「胶水摊反择新键」「胶水拿所选槽之外的根调重放」两行证红。
2. K1-d：`newest_root_among` 与核同一择法（键完全相同取前一个），或改文档；选哪一种在实现员报告里写依据。
3. K2-b：`crates/singlefs-replay-prefix/verus-mutations.tsv` 加「合判推论去掉 J1 前提」一行，逐行核。
4. K3：R3-1 另三处拷贝各一行变异证红。
5. 写回：D13 已定项 1 M3 一行补 I-8.12 前提；D13 已定项 1 第 31 行一带「择根与扫环不在 Verus 里」改成现状（`choose_root_slot`、`first_copy_of_each_record_key`、合判推论）；D22 已定项 24 补「键完全相同取前一个」；E165 页、E170 页各补一句。
6. 新欠账两笔：扫环胶水内存随盘数涨（K1-c）；必红世界期望条数随求解上下文漂（K2-c）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 不影响 | 2026-10-02 不受影响：M3 前提补句与落点现状句等写回 |
| D22（单元原子性怎么合成） 已定项 7 | 不影响 | 2026-10-02 不受影响：择新序与核一致（K1-a） |
| D22（单元原子性怎么合成） 已定项 24 | 不影响 | 2026-10-02 不受影响：平局句等写回 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-02 不受影响：扫环先到先得与条款一致（K1-a） |

## 改法落地（2026-10-02，实现员 11-10）

第三节第 1、2、4 条做完（报告 `/tmp/claude-1000/m3-formal-11-10/report.md`）；第 3 条（合判推论去掉 J1 前提的必红行）写不成：去掉之后 Verus「56 verified, 0 errors」，探针证出只凭 M2 就推得出 J1（`requires` 只写 M2、`ensures` 写 J1 的空体 proof fn「57 verified, 0 errors」）。主 agent 取第一条路：不加这一行，J1 在推论里是由 M2 推得出的多余前提，写进这一节；正推腿「去掉 J1 前提几乎一定红」不成立。另：「胶水拿所选槽之外的根调重放」只在 `recover` 一处加了变异，`mounted_read.rs:567`、`mount.rs:4553` 两处同样的 `&chosen_root,` 还没有，进下一批。
