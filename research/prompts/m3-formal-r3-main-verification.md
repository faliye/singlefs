# 形式化验证的题面：第三轮判决（2026-10-01）

<!-- doc-lint:not-numbers M1 M2 M3 M4 K10 J1 J2 J4 J5 J6 Z1 Z2 S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 T1 T2 T3 T4 G2 G4 G5 -->

第三轮是最终轮（`.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」）：这一轮里新冒出来的写法记零轮、进第六节交用户的表，不再开轮。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 | `research/prompts/_m3-formal-r3-body.md`、`_m3-formal-r3-background.md`（8 份 kb、清单抄 136 行） | — |
| 开工快照 | `research/prompts/m3-formal-r3-snapshot/`：`crates-sha256.txt`（212 份，冻结副本 `/tmp/claude-1000/m3-formal-r3/tree/`）、`kb-sha256.txt` | 腿交齐之后冻结副本 `sha256sum -c` 全 OK；`kb-sha256.txt` 只有 `.claude/kb/checks-owed.md` 对不上：相对 HEAD 的改动在第 45、108、130、182、214、269、282–284、353 等行，腿引的第 88、277 行不在改动处 |
| 云端攻方（Opus） | `research/prompts/m3-formal-r3-opus-output.md`（305 行，sha256 60544b2b…85a1）；模型 `research/prompts/m3-formal-r3-opus-model/` | 第五节 |
| 云端辩方（Sonnet） | `research/prompts/m3-formal-r3-sonnet-output.md`（271 行，sha256 ca7b1c6d…2057）；模型 `research/prompts/m3-formal-r3-sonnet-model/` | 第五节 |
| 引文 | 两份腿报告 | `python3 scripts/cite-check.py <两份报告> --root /tmp/claude-1000/m3-formal-r3/tree --root . --background research/prompts/_m3-formal-r3-background.md`：「✓ 核了 103 处引文，对上 103 处，没判 0 处」 |
| 主 agent 现查的代码 | 冻结副本 `crates/singlefs-core/src/recovery.rs` 第 3137–3156 行 | 施加记录重建出的根 `unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence`（第 3147 行），注释引 D22（单元原子性怎么合成） 已定项 7「其余根写 0」 |

两条腿的数都是冻结副本的拷贝或译本上量的；进 kb 之前在入库装置上重做。

## 二、逐格判决（第三轮正文第三节 S1–S10）

| # | 攻方 | 辩方 | 判决 |
|---|---|---|---|
| S1 前半（根的投影加盘） | 站住，第二轮的数复现 | 站住，「整条根」撤站得住 | **站住，两条腿同向** |
| S1 后半（其余字段等于所选根） | 打中：正常卸载那一串里第二次空发布的记录落了、根槽没落（不带故障的崩溃状态），施加出的根卸载记号 0、所选根 1（池上原型 `runs/unmount-marker.txt`：`chosen_marker=WrittenByTheUnmountSequence rebuilt_marker=NotWrittenByTheUnmountSequence`） | 打中同一处：副本上让时间线根带记号，施加了记录的 1 867 682 态记号全不等，fsid、实例表指针、分配记录树根不等 0 个 | **打中，两条腿独立同向，主 agent 现查代码坐实**（第一节最后一行）。实现照 D22（单元原子性怎么合成） 已定项 7 写 0，形态的字面照 D23（journal 的角色与格式） 已定项 15 之外「其余等于所选根」，两条条款各指一边，是条款欠一句。收严成「其余字段等于所选根，卸载记号除外（由记录重建的根取「不是卸载那一串写下的」）」——**零轮**，交用户 |
| S2 句法层补强 | 改（收严）：只补连续与链首时 Z2、表第 515、516 行仍 14 verified；再补在飞上限与提交标记两句才红；`requires` 升序对补强的两句不承重 | 补强写得出、证得过；13 道 break 判里补强后仍有 6 道删了照样验过（反向链、事务边界、提交标记、读者规则、点名单元、在飞上限），由 E165 判；升序 `requires` 不承重 | **收严**：spec 除连续与链首外，在飞上限与提交标记也写进去（攻方量到加了才红；辩方原型没写这两句，两条腿不矛盾）。反向链与点名单元是回调的结果（G4、G5），spec 里只能写成「被施加的每条记录，回调交回了真」，其余靠 E165。升序 `requires` 两条腿都说对证明不承重：留着，它管的是公开 API 的次序前提（代码轮 C1 那一格），不当证明的一部分。第 4 批 4-1 正在照这一条改核 |
| S3 唯一前提只挂语义层 | 没打中 | 站住；真路径上「唯一」靠 `scan_journal` 先到先得，没有变异行 | **站住**；「先到先得没有变异行」记欠账 |
| S4 M4 不少施加 | M4 站住；射程句「连同择根才排除丢已落盘的完整发布」只在不带故障的状态上成立（在飞上限截掉记录完整的发布：L6 354、L9 51 998、C6 215 态，甲切法合法态 0） | M4 与 Z1 读法站得住；射程句要 D22 择根、D23 已定项 14 注 1 同实例、D16 已定项 8 暖机三处合起来，且写「已确认的」 | **M4 站住，射程句收严**：两条腿各给一半前提，合起来写——**零轮**，交用户 |
| S5 M3 单调写成推论 | 推论站住（第 4 步读代码核过） | 域内推论站得住 | **站住，两条腿同向**。版本序见第三节 |
| S6 M3 幂等两读都证 | 站住 | 站住 | **站住**。txg 顶格见第三节 |
| S7 主语 | 没打中 | 站住；三处 `true` 现查了，没有变异行改它们 | **站住**；「三处 `true` 没有变异行」记欠账 |
| S8 第四格补句 | 没打中；K10 下 0 / 0 在 Rust 上量到 n = 4 | 站住，补一条三行手证 | **站住，两条腿同向** |
| S9 同源性计数器 | 没打中 | 站住 | **站住** |
| S10 七·5 判别力样本与判绿条件 | 打中两处：(a) verified 数是函数个数，spec 变弱它不动；(b) 必红按「verified, M errors」字样判，rlimit 用尽也算 | 打中：判绿四条件挡不住空真，`requires` 加 `false` 仍 14 verified, 0 errors | **打中三处，主 agent 现跑 Verus 坐实**（第五节）。收严：判绿条件加「空真格判过」（门禁 verus-vacuity 格已有判法，要写进条件）；必红世界的 errors 不许是 rlimit 用尽（判定行之外要核报错不是 `Resource limit (rlimit) exceeded`，rlimit 写死）；verified 数之外另立 spec 强度样本（Z2、表第 515、516 行这类改坏代码必须转红的世界）。门禁那一半进下一批工具簇，`verus-mutations.tsv` 那一半在 4-1 合入时逐行核 |

## 三、两格零轮（T2）

| 格 | 攻方 | 辩方 | 判决 |
|---|---|---|---|
| S5 版本序 | 在 S5 的域上两种写法不可分（按 D23 已定项 14 注 3、已定项 16、管理员回退推的） | 域内三种写法逐态重合；域外 checkpoint_txg 看不见「平局取低」的选根器，(实例代号, txg) 看不见「实例代号为主」的选根器、还把今天的选根器判红；第三种 (txg, 实例代号) 就是 D22（单元原子性怎么合成） 已定项 7 的择新序，三个选根器全分得开、今天的 0 红（译本上量的，没在真 `choose_root` 上量） | **两条腿在域内同向「不可分」**。主 agent 现查：冻结副本 `recovery.rs` 第 1161 行择根键是 `(candidate.checkpoint_txg, candidate.instance)`，第 1148 行文档注释同；D16（发布语义） 第 160 行「择新在 checkpoint_txg 平局时按实例代号高者赢」。第三种写法与择根同一个序，辩方的分辨观测是译本上的。**零轮**，三个候选交用户；主 agent 的倾向是 (txg, 实例代号)，理由只有「与择根同序」这一条，域外的分辨力是译本上量的 |
| S6 txg 顶格 | 分得开：表第 1003 行（删掉顶格那一判）写法 A（前提）判 328 态 0 违例，写法 B（报错且一条没施加算相等）判 398 态 70 违例；今天的实现两种都 0 | 同一结论：P（前提）把那个真缺陷（第二次调用 panic，70 个态）判成不判，Qn 判；Qw（任何报错都算相等）在对抗一族把第二次报歧义错也判成相等（464 / 2 904）；建议 P 加一条独立的顶格断言 | **两条腿独立同向：写成前提会漏判一个真缺陷**。主 agent 复跑攻方模型，`runs/txg-cap-summary.txt` 里 `core8 E1 … atk_j6_writing_a_judged=328 atk_j6_writing_a_violations=0 atk_j6_writing_b_judged=398 atk_j6_writing_b_violations=70` 原样在。候选：只认「第二次因所选根 txg 顶格报 `CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR` 且一条没施加」算相等（Qn），不放宽到任何报错（Qw）。**零轮**，交用户 |

## 四、欠账（T3）

第二轮判决第五节第 2 条那四笔照旧欠着（`crates/mutations.tsv` 里 `expected_key` 仍零命中；Verus 一侧「删 I-8.3」在第 4 批 4-1 里补）。这一轮新记：

| # | 要拦什么 | 出处 |
|---|---|---|
| 1 | 由记录重建的根的卸载记号：条款欠一句（D22 已定项 7 与 D23 已定项 15 之外的「其余字段」），S1 收严写回之前没人判它 | 两条腿 |
| 2 | spec 判不到的两样回调结果（反向链、点名单元）只由 E165 判，Verus 一侧只能写成后置条件 | 两条腿 |
| 3 | 必红世界的 errors 是 rlimit 用尽也会被当成红 | 攻方，主 agent 现跑坐实 |
| 4 | verified 数判不出 spec 变弱：要 spec 强度样本 | 攻方 |
| 5 | 空真：判绿条件里没写空真格 | 辩方，主 agent 现跑坐实 |
| 6 | C 域合法崩溃状态的口径：甲切法（D16 已定项 7 的屏障位置）与乙切法混用，判决与 kb 引数时写明切法 | 攻方 |
| 7 | 择根与重放没有合判（S4 的射程句要两者合起来才成立） | 攻方、辩方 |
| 8 | E165 的 J6 不照选定的顶格写法判（用户定 S6 之后改装置） | 攻方 |
| 9 | 三处以 `verify_named_units = true` 调 `replay_journal` 的 `true` 没有变异行 | 辩方 |
| 10 | 唯一前提在真路径上靠 `scan_journal` 先到先得，没有变异行 | 辩方 |

## 五、复跑（T4 与主 agent 自己跑的）

- 攻方模型：照 `research/prompts/m3-formal-r3-opus-model/README.md` 在 `/tmp/claude-1000/m3-formal-r3-main-check/opus/` 从冻结副本拷一份、`patch_attack.py`（recovery.rs 4 处、E165 装置 13 处）、编一次、`run-all.sh` 与 `summarize.py`，经 `scripts/run-with-memory-cap.sh 3G`：`summary-all.txt` 的 sha256 与入库那份相同（98c764c4…011e）。攻方报的第二轮数全部复现：1 007 514、264 499、352 608、K10 下 0 / 0、57 585 320 都在冻结副本上逐数对上；口径改一处：第二轮引的 15 343 是乙切法的合法态（C 14 737、C6 606），D16（发布语义） 已定项 7 的屏障位置下是甲切法 C 8 006、C6 383、共 8 389，开关 1、2 在两种口径上 J1–J4 都是 0，结论不变。
- 卸载记号的池上原型：没复跑（要编 harness 档），机理主 agent 读代码坐实（第一节），辩方另一条副本独立量到同一处，两条路径不共用代码。
- Verus（0.2026.09.20.aef82ed，`--no-cheating`，`/tmp/claude-1000/m3-formal-r3-main-check/`，`rlimit-check.txt`、`v08-1000.log`、`vac.log`）：攻方补强加两句之后的核（`s2_strengthen.py` 再 `s2_plus.py`）原样 rlimit 200 与 1000 都是「14 verified, 0 errors」；关掉反向链的变体 rlimit 200「error: while loop: Resource limit (rlimit) exceeded … 13 verified, 1 errors」、rlimit 1000「14 verified, 0 errors」；Z2（不看在飞上限）rlimit 200 与 1000 都是「13 verified, 1 errors」；删 I-8.3 那一判 rlimit 200「13 verified, 1 errors」。冻结核的 `replay_core` 的 `requires` 头一条加 `false,`：「14 verified, 0 errors」。
- 辩方模型（Python 译本与 E165 副本加计数）没有复跑：它与攻方是两条互不共享代码的路径，判决引它的地方（S1 的 1 867 682、S5 域外、S6 的 464 / 2 904）都标了出处，S1、S6 的结论有攻方那一条独立路径同向。
- 两条腿在内存包装上各自设了同会话并行的放行（辩方「三方复核辩方腿与同会话别的任务各跑各的」），照包装文件头的出路做的，认。

## 六、交用户的与下一步

1. **零轮、交用户认的四格**：S1 收严成「卸载记号除外」；S4 射程句的前提写全（不带故障、D22 择根、D23 已定项 14 注 1、D16 已定项 8 暖机、「已确认的」）；S5 版本序三选一；S6 txg 顶格取 Qn。照规则第三轮之后不再开轮，用户认了就写回，不认的留开。
2. **写回**（用户认、K10 在 m3-inv-b-r1 那一轮判完之后）：D13（验证路线） 已定项 1、已定项 13，D23（journal 的角色与格式） 已定项 4，`.claude/kb/milestone/03-third-txn.md` 七·4、七·5；E165 实验页 C 域合法态数写明切法。
3. **并进代码与门禁**：S2 的 spec 收严在 4-1 里；S10 三处——4-1 合入时逐行核 `verus-mutations.tsv` 的红不是 rlimit 用尽，门禁 verus-must-red-worlds 格写死 rlimit 并拒 rlimit 报错、判绿条件加空真格、另立 spec 强度样本，进下一批工具簇；第四节第 9、10 两笔变异行并进下一批实现员。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 备料 | 2026-10-01 不受影响：M1–M4 候选形态三轮攻完，四格零轮交用户，认之前不写回 |
| D13（验证路线） 已定项 13 | 备料 | 2026-10-01 不受影响：同源计数器那一句三轮站住，随已定项 1 一起写回 |
| D16（发布语义） 已定项 7 | 不影响 | 2026-10-01 不受影响：只用它的屏障位置划甲切法的口径 |
| D22（单元原子性怎么合成） 已定项 7 | 不影响 | 2026-10-01 不受影响：被引作卸载记号「其余根写 0」与择新序的条款，正文不动 |
| D23（journal 的角色与格式） 已定项 4 | 备料 | 2026-10-01 不受影响：第四格补句三轮站住，挂 K10 前提，等 m3-inv-b-r1 |
| D23（journal 的角色与格式） 已定项 15 | 不影响 | 2026-10-01 不受影响：新根段四个字段的口径不动，S1 的收严只在 M2 的字面里写「卸载记号除外」 |
