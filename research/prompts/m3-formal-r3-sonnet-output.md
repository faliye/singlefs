# m3-formal-r3 辩方腿报告（Sonnet，2026-10-01）

<!-- doc-lint:not-numbers M1 M2 M3 M4 K3 K4 K5 K6 K7 K8 K9 K10 K11 J0 J1 J2 J3 J4 J5 J5a J5b J6 Z1 Z2 Z3 Z4 F4 F5 F8 F9 F14 R1 R2 R3 R4 R5 R6 R7 R8 S1 S2 S3 S4 S5 S6 S7 S8 S9 S10 T1 T2 T3 T4 W1 W2 P Qn Qw G2 E1 E2 L6 L9 C6 A6 AC IA1 IA2 Q7 Q14 -->

轮名 m3-formal-r3（第三轮、最终轮），立场辩方：复核第二轮判决 `research/prompts/m3-formal-r2-main-verification.md` 的十格，答 T1–T3、重点 T2。读冻结副本 `/tmp/claude-1000/m3-formal-r3/tree/`（不读主仓 `crates/`、不读禁读清单）；kb、代码与产物的引文都写那份文件自己的行号，这一次现查过。

记号：**[量]** 我这一次真跑出来的（命令与原样输出见第七节，脚本与原始数在 `research/prompts/m3-formal-r3-sonnet-model/`）；**[读]** 读代码、kb、产物；**[推]** 推的，没量；**[译]** 在我写的译本上量的，不是 Rust 上量的。

**环境上的一件事**：同一会话里别的任务一直占着包装的「日常一条」槽，我的第一次起跑退 252（命令没跑，输出不算）。照包装自己打印的出路，之后每次起跑我自己设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（理由「三方复核辩方腿与同会话别的任务各跑各的」），内存上限仍是 6G 或 1G、仍经包装。派发提示没写放行这一句，是我设的；主 agent 若认为这一步不该由我设，第七节的数按「没经批准的并行」处理，命令都在模型目录里，可在副本上重跑。

**一句话**：① S5 版本序：域内三种写法分不开，域外两种各瞎一边，第三种 (txg, 实例代号)（就是 D22 已定项 7 的择新序）三个选根器全分得开（零轮）；② S6 顶格：写成前提与「报 no_successor 且一条没施加算相等」在 E165 Q7 那个域上判得一样，**写成前提会把一个真缺陷（删掉顶格那一判，第二次 panic，70 个态）判成不判**，建议前提加一条独立的顶格断言（零轮）；③ S1 的「其余字段等于所选根」字面为假：卸载记号恒 0，卸载那一串的合法崩溃状态会撞上（1 867 682 个态）；④ S2 补强写得出来、证得过（14 verified, 0 errors），删 I-8.3 与删无锚点链首转红，但另有 6 道判删了仍绿；⑤ 七·5 的判绿四条件挡不住「前置条件恒假」（14 verified, 0 errors），要有真调用的见证函数。

## 一、判定一览（先给结论）

判定三档：**站得住**（原句字面被打中）、**够不着**（原句的另一个分句、括注或射程没被打到）、**同样打中别的臂**（这一格的依据换个地方照样成立，被判决留下的形态也挨了）。

| 判决条目（第二轮第几格 / 这一轮的形态） | 辩方判定 | 证据（指到后文） | 什么现象会推翻它 |
|---|---|---|---|
| 1 / S1：M2「结果」取投影加盘，「整条根」撤 | 「整条根」撤：**站得住**。投影六字段与「盘」三样：**够不着**。「其余字段等于所选根」：**同样打中别的臂** | 第四节 1。E165 J1 比的六字段就是投影，J2 / J3 / J4 就是盘，light 合法一族里除 U 验证关与 R 域外都是 0；而撤整条根的那条依据——实现按设计写的字段不等于条款读法——原样打在「其余字段」上：重建出来的根卸载记号恒写 0，所选根带记号时每个施加了记录的态都不等（L 域 1 007 514 / 1 007 514）[量] | 找到卸载那一串之外有哪一条合法路径让所选根带记号而之后同实例还有记录——没有，则这一格只是「写明除卸载记号」；找到，则要在条款里选一边 |
| 2 / S2：M2 拆句法与语义两层，句法层补连续与链首 | 补强本身：**站得住**。「删判核照样验过」：**同样打中别的臂** | 第四节 2。我把补强写出来并证过（14 verified, 0 errors），删 I-8.3 那一判与删无锚点链首那一判都转红（13 verified, 1 errors）[量]；但冻结核 13 道 break 判里有 8 道删了仍是 14 verified，补强之后仍有 6 道是绿的，它们不在 S2 的文字里 | 补强写进之后，第 4 批的核仍判得过「删 I-8.3」——那就是补强没落地 |
| 2 / S2：表按 (实例代号, 计数器) 升序写成 `requires` | **够不着**（链与链首的证明不用它）；只有读者规则那一判要它，而它进的是胶水义务 | 第四节 2 [量][读]：我的原型没有任何升序前提也证得过 | 原型里拿掉的升序前提其实被某条证明暗用了——Verus 报错 |
| 3 / S3：唯一前提只挂语义层，射程句绑原样重发 | **站得住**；**够不着**它的射程句 | 第四节 3。D23 已定项 14 两处原文逐字在；真路径上「唯一」靠 `scan_journal` 的先到先得，没有变异行、E165 不调它（欠账，第六节） | 一条合法写侧历史让同一 (实例代号, 计数器) 落两份不同内容 |
| 4 / S4：另立 M4、Z1 取「多出违例」 | M4、Z1 读法：**站得住**。「M4 加择根合起来就排除丢失」：**同样打中别的臂** | 第四节 4。从入库的第二段产物重数：legal 一族只被 J4 一族抓到的变体 7 个 [量]；而「已落盘的完整发布不丢」要 D22 择根、D23 注 1 的同实例、D16 已定项 8 的暖机三处合起来，择根序在平局与倒挂世界里分得出缺陷 [译] | 第二段产物里这 7 个的任何一个在 legal 一族上被 J1–J3 抓到 |
| 5 / S5：M3 单调写成推论（域内） | **站得住** | 第二节 1：域内三种版本序逐态重合，用到 `crates/singlefs-core/src/recovery.rs:1161`、`crates/singlefs-core/src/mount.rs:1114`、D23 注 3 [读][推] | 域内找到一对状态三种序判得不同 |
| 5 / S5：版本序（零轮） | 域内**分不开**；域外**两种写法各瞎一边** | 第二节 2 [译]：只比 txg 看不见「平局取低」，(实例代号, txg) 看不见「实例代号为主」还把今天的选根器判红；(txg, 实例代号) 三个选根器全分得开 | 在真 `choose_root` 上量 W1、W2 与译本不同 |
| 6 / S6：M3 幂等两读都证 | **站得住** | 第三节、第五节 [读]：记录级靠水位过滤，不动点靠 E165 J6 | J6 在界内合法一族某一格非 0 |
| 6 / S6：txg 顶格（零轮） | 写成前提 P、报错算相等 Qn 在 E165 Q7 那个域上判得一样；**P 会把一个真缺陷判成相等**（删掉 u64 最大值那一判：第二次调用 panic，70 个态 P 不判、Qn 判）；Qw（任何报错都算相等）会把对抗一族的第二次调用报歧义错也判成相等（464 / 2 904） | 第三节 [量]：8 个变异版 × light 两族 | 在别的变异或域里 Qn 也把真缺陷判成相等 |
| 7 / S7：主语是三处 `verify_named_units = true` | **站得住** | 第五节 [读]：三处调用的第五个参数现查；欠账：没有一条变异行改那几个 `true` | 有一处生产调用传 false |
| 8 / S8：第四格「之后」按计数器、缺号不停、挂 K10 | **站得住**，并补一条手证 | 第五节 [读][推]：K10 成立时核里那一判与全表看逐态相等，三行可证，不靠 n ≤ 3 的量 | K10 的写法不是「全表里同实例逐计数器不减」 |
| 9 / S9：同源性计数器加「条款欠一句」 | **站得住**；机器判不了 | 第五节：它是记账规则，没有代码与计数器 | 有一处代码在记这类事件 |
| 10 / S10：七·5 加「spec 太弱」样本；判绿四条件 | 样本：**站得住**。判绿四条件挡不住空真：**同样打中别的臂** | 第四节 2、第五节 [量]：今天的核对七·2 必红世界 14 verified, 0 errors（会判绿），补强后 13 verified, 1 errors；`replay_core` 的 `requires` 加 `false` 仍是 14 verified, 0 errors，加一个真调 `replay_core` 的见证函数才红（14 verified, 1 errors） | 补强前这份样本判红；或空真版被「判定行原样」截下 |
| T3 新增欠账 | 第六节 5 笔 | 卸载记号、七 S7 的 `true`、`scan_journal` 先到先得、Verus 一侧只有 S2 的几句、择根序 | — |

## 二、T2 之一：S5 版本序（零轮；替挂起的一方——(实例代号, checkpoint_txg)——辩护，并给分辨观测）

**先说结论**：S5 自己划的域里（串行提交、D16 已定项 7 的持久顺序、不带故障、所选根取最新有效根）三种写法逐态重合，分辨不了；分得开的只在域外（失而复得的平局、倒挂），而且**两种写法各瞎一边**：只比 checkpoint_txg 的看不见「平局取低」的选根器，(实例代号, txg) 的看不见「实例代号为主」的选根器、还把今天的选根器判红。第三种写法（txg 为主、实例代号破平局，就是 D22 已定项 7 的择新序）三个选根器全分得开、今天的选根器上 0 红。这第三种是零轮，不是这一轮攻过的。

**1　域内为什么重合（[读][推]；步骤用到的条款与代码行都在下面逐条现查过）**

1. 择根序是 (txg, 实例代号)：「let candidate_key = (candidate.checkpoint_txg, candidate.instance);」（`crates/singlefs-core/src/recovery.rs:1161`）；文档注释「三个区域全部槽逐个验自证校验和，取 `(checkpoint_txg, 实例代号)` 最大的」（`crates/singlefs-core/src/recovery.rs:1148`）；条款侧「择新在 checkpoint_txg 平局时按实例代号高者赢」（`.claude/kb/decisions/16-发布语义.md:160`）。
2. 取前缀不跨实例：「record.instance == root_instance && record.checkpoint_txg > root_checkpoint_txg」（`crates/singlefs-replay-prefix/src/lib.rs:57`）。所以 `Recover` 落到的版本的实例代号恒等于所选根的实例代号；跨实例的比较只发生在「所选根」上。
3. 新实例第一次发布的 txg 取全环最大再加一：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1114`），条款是「checkpoint_txg 也一样」那一句（`.claude/kb/decisions/23-journal的角色与格式.md:369`）。实例代号取盘上最高再加一：「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」（`crates/singlefs-core/src/transaction.rs:705`），加一溢出报 `InstanceGenerationPastTheTopOfItsRange`。
4. 推：同一崩溃状态里读得出的两条根 R、R'，实例代号 i < i' ⇒ txg t < t'（i' 那个实例开张时 i 的每一条读得出的根都在环里，3 把 t' 抬到它们之上；i 实例此后不再写，串行提交）。所以 txg 序、(实例代号, txg) 序、(txg, 实例代号) 序在域内给出同一个次序，也没有跨实例的平局。

这一步用的前提是「开张那一刻环里的根都读得出」。读不出（暂时）或整盘不在，4 就断：D16 已定项 6 骑手 1 写「丢盘回退 + 盘失而复得会造出两条同 txg 的自证合法根」（`.claude/kb/decisions/16-发布语义.md:160`，同形的欠账 `.claude/kb/checks-owed.md:88` 的 C88），C331 写「较旧实例的全部根暂时读不出时，新实例的第一次发布从更旧的根往上数 txg」（`.claude/kb/checks-owed.md:277`）。两处都在 S5 自己写的「域放宽到带故障时重开」之外，所以 S5 那句本身不受这里影响，受影响的是「两种写法选哪个」这件事到重开那天才有分别。

**2　分辨观测（[译]：选根器是我译的一行比较，不是 Rust 上量的；数在 `research/prompts/m3-formal-r3-sonnet-model/s5_version_order_model_v2.py` 的输出里）**

根 = (实例代号, txg)。三个世界：注 3 世界（新实例的 txg 都在旧实例之上）、只有平局（旧实例 1：100、101，新实例 2：101、102）、倒挂（旧实例 1：100–102，新实例 2：101、102）。每个世界取根集合的全部非空子集，数全部 s ⊆ u 的对（211、65、211 对）里「land(s) 晚于 land(u)」的对数。选根器三个：真的（txg 为主、实例代号取大，译自 `crates/singlefs-core/src/recovery.rs:1161`）、变异 A（平局时实例代号低者赢）、变异 B（实例代号为主）。

| 世界 | 选根器 | 只比 txg | (实例代号, txg) | (txg, 实例代号) |
|---|---|---|---|---|
| 注 3 | 真 / 变异 A / 变异 B | 0 / 0 / 0 | 0 / 0 / 0 | 0 / 0 / 0 |
| 只有平局 | 真 | 0 | 0 | 0 |
| 只有平局 | 变异 A（平局取低） | **0（看不见）** | 3 | 3 |
| 只有平局 | 变异 B | 0 | 0 | 0 |
| 倒挂 | 真 | 0 | **9（把今天的选根器判红）** | 0 |
| 倒挂 | 变异 A | 0 | 42 | 30 |
| 倒挂 | 变异 B（实例代号为主） | 9 | **0（看不见）** | 9 |

两个最小见证（根集合是读得出的根）：W1（只比 txg 看不见）s = {(2,101)}、u = {(1,101),(2,101)}，变异 A 在 u 上落到 (1,101)，比 s 的 (2,101) 早，txg 都是 101 ⇒ 只比 txg 判「不晚于」成立；W2（(实例代号, txg) 判红今天的选根器）s = {(2,101)}、u = {(1,102),(2,101)}，真选根器 land(s) = (2,101)、land(u) = (1,102)：txg 上不倒退，(实例代号, txg) 上倒退。W2 就是 C331：实例 2 已确认的写被旧实例更高 txg 的根压过去。

**3　替 (实例代号, txg) 辩护**

- 站得住的理由：① S1 里「版本」的身份已经是这一对（M2「结果」取 (实例代号, checkpoint_txg) 等于时间线上第 j 版），M3 用同一个身份最省一条词；② 取前缀自己的水位序就是实例代号为主（「记录水位 `(实例代号, checkpoint_txg)` 按实例代号为主比」，`.claude/kb/decisions/23-journal的角色与格式.md:368`），核里 `above_the_chosen_root` 也是先看实例相等；③ 看得见「平局取低」这一类选根器缺陷（W1），只比 txg 的看不见。
- 它挨的打（不替它遮）：④ 域外它把今天的选根器判红（W2，9 / 211 对），所以 M3 若按它写、域重开那天要么同时修 C331 的择根倒挂，要么在域里写明排除；⑤ 看不见「实例代号为主」的选根器（变异 B），那一类只比 txg 的看得见；⑥ 它不是 D22 已定项 7 择新用的序，M3 推论的第一步「所选根单调」要多用 3 才站得住（只比 txg 或 (txg, 实例代号) 时第一步就是 `choose_root` 对超集取最大，不用 3）。
- 所以我辩的是「域内它站得住、而且不比攻方那一写法多丢东西」，不是「它比攻方的好」。真要挑，第三种 (txg, 实例代号)：三个选根器全分得开，今天的选根器上 0 红，且不加条款——它就是 `crates/singlefs-core/src/recovery.rs:1148` 与 D16 已定项 6 已经写的择新序。零轮，进交用户的表。

**推翻条件**：① 域内找到一个根集合对（不带故障、串行提交）使三种序判得不同——4 的前提被推翻，要看是不是 3 没做到（`crates/singlefs-core/src/mount.rs:1114` 取的是不是全环）；② 用真 `choose_root` 在两实例的内存池上量 W1、W2（我只量了译本），选根器的比较不是 (txg, 实例代号)；③ 写侧有一条路让旧实例在新实例开张之后还落根（4 的「此后不再写」断）。

## 三、T2 之二：S6 txg 顶格（零轮；替「写成前提」一方辩护，给分辨观测）

**三种写法**（本节用这三个名字）：**P** = 写成前提：所选根与施加后的根的 txg 小于 u64 最大值，顶格的态不判；**Qn** = 第二次因所选根 txg 顶格报 `chosen_root_txg_has_no_successor` 且一条没施加，算相等；**Qw** = 第二次报任何错（panic 不算错）且一条没施加，算相等。Qn 是攻方第二轮候选句的原话：「第二次因所选根 txg 顶格报错且一条没施加的，算相等」（`research/prompts/m3-formal-r2-opus-output.md:246`），Qw 是派发提示里的缩写读法。

**怎么量**：把冻结副本的 E165 装置拷进草稿目录，只加计数不改判据（`research/prompts/m3-formal-r3-sonnet-model/patch_e165.py`）：J6 的第二次调用之后按 (前提真假, 第一次是不是落在 u64 最大值, 第二次结局种类) 数。打了补丁的基线跑 `--bounds light --family legal`，876 行 `E7RESULT` 与入库的 `research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out` 逐行相同 [量]；每个 (运行, 域) 我数的「当前判据下违例数」与装置自己的 `j6_premise_holds_violations` 在全部 126 行上相等 [量]。变异版是把 `crates/mutations.tsv` 的行套到副本上重编：第 24、36、406（同 414）、512、513、518、1003、1004 行，每个版本各跑 light 的合法一族与对抗一族（第二段的完整杀伤界没重跑）。

**E165 那个域上各判什么**（实验页「逐行判定」Q7：「F8 触发在 E1：不动点读法带前提也为假」，`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:168`）：E1 域 398 个态（产物 `research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out:806`「domain=E1 ring_slots=64 smallest_n=1 largest_n=4 states=398」），所选根 txg = u64 最大值减一。

| 运行 / 域 | 判了几个态 | 当前判据违例 | P | Qn | Qw | 违例是哪一种第二次结局 |
|---|---|---|---|---|---|---|
| 基线 E1 | 398（P 下 328） | 70 | 0 | 0 | 0 | 70 个全是（顶格、报 no_successor） |
| 第 1003 行（删掉顶格那一判）E1 | 398 | 70 | **0** | **70** | **70** | 70 个全是（顶格、第二次 panic） |
| 第 406 行 E1 | 398 | 114 | 0 | 0 | 0 | 顶格、报 no_successor |
| 第 24、512、36 行 E1 | 398 | 35、52、70 | 0 | 0 | 0 | 同上 |
| 第 406 行 L 域（别的合法域同形） | 4 031 078 | 563 488 | 563 488 | 563 488 | 563 488 | 第二次返回的根不等，非顶格 |
| 基线对抗 A / AC | 660 278 / 5 129 830 | 464 / 2 904 | 464 / 2 904 | 464 / 2 904 | **0 / 0** | 第二次报 `more_than_one_last_record_flag`，非顶格 |
| 第 518 行对抗 A / AC | 534 790 / 4 144 646 | 1 664 / 10 944 | 同 | 同 | **0 / 0** | 同上 |
| 第 406 行对抗 A / AC | 660 278 / 5 129 830 | 4 944 / 20 040 | 同 | 同 | 3 808 / 13 376 | 一部分返回的根不等、一部分报歧义错 |

（完整的 126 行在 `research/prompts/m3-formal-r3-sonnet-model/s6_table.tsv`。第 513 行 E1 同基线 70；第 1004 行 E1 第一次调用就报错、J6 不判；其余行见那张表。）

**读数**：
1. **基线的 70 个违例三种写法判得一样**：70 个全是「第一次落到 u64 最大值、第二次报 no_successor」，P 把这 70 个态移出（判的态数 398 → 328），Qn 与 Qw 把它们判相等，违例都是 0。所以在 Q7 那个域上两种写法**不可分**。
2. **P 会把一个真缺陷判成相等**：第 1003 行是「所选根的 txg 改回裸加一」（`crates/mutations.tsv:1003`），E1 里第二次调用从 u64 最大值的根起，加一溢出 panic（装置副本是 `overflow-checks = true` 的 release）；这 70 个态 P 不判（前提假），Qn、Qw 判违例。同一条缺陷在第一次调用上由 E2 抓：红例账里 E2 首红例的差是「+IA1,+J1,+J2,+J3,+J4,+panicked」（`research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv:121`）。所以 P 丢的是「J6 对它的第二条独立线索」，不是杀伤界：E2 与 IA1 判同一类输入（所选根 txg = u64 最大值）。
3. **Qw 会把真缺陷判成相等的格在对抗一族**：基线 464 / 2 904、第 518 行 1 664 / 10 944 的违例全是第二次报歧义错（不是顶格），Qw 全吞；合法一族的 8 个变异版里没有 Qw 吞掉的格。这些态都在同实例 txg 随计数器回退的表上（K10 不成立），本来也挂 K10 前提。
4. **Qn 在这 8 个变异版里没把任何「第二次不是 no_successor」的格判成相等**：它只吞「顶格且报 no_successor」（第 406 行 E1 的 114 个里比基线多的 44 个也在其中，那是半次发布被施加到顶格的后果，J2 与 J4 另判），而这一类恰是被条款写明的行为——核里 `replay_core` 的这一支（`crates/singlefs-replay-prefix/src/lib.rs:353`「if root_checkpoint_txg == u64::MAX {」）是证明逼出来的：我在冻结核上删掉这一判，Verus 报「error: possible arithmetic underflow/overflow」（`research/prompts/m3-formal-r3-sonnet-model/verus/d12_top_txg_check.verus.out:1`），结果行 13 verified, 1 errors。

**替 P 辩护**：① P 的前提就是核里那一支的分支条件，写成前提是把「顶格走报错那条臂」交给核的证明，不另造一条相等；② P 丢掉的 1003 那一格由 E2 与 IA1 判，同一个函数、同一类输入；③ 相等关系保持原样（返回根且全字段相等），不把「第一次成功、第二次失败」塞进「相等」里；④ 顶格要 2^64 次发布才走到 [推]；挂载一侧的取号函数只在加一溢出时拒：「version.0.checked_add(1).map(CheckpointTxg).ok_or(」（`crates/singlefs-core/src/mount.rs:1122`），所以写到 u64 最大值那一版没被拒 [推：发布路径没逐条读]，而读侧从那一版起报错、整池拒挂载，P 在这里划的是一条写读不对称的边界，不是「走不到」。
**它挨的打**：2 里的格——P 单独用，J6 在顶格那一类态上对「第二次 panic / 返回不等 / 报别的错」全瞎。
**合成**：P 加一条独立的顶格断言（所选根 txg = u64 最大值 ⇒ 报 no_successor 且一条没施加，就是 IA1 的写法），不把它折进「相等」。这样 1003 那一格仍由断言判，相等关系保持纯，也不吞 Qw 吞的那一类。这是零轮，进交用户的表。

**推翻条件**：① 合法一族别的变异（本轮只量了 8 个版本、light 档）里有一个让 P 判不到而 E2 与 IA1 也判不到的顶格缺陷；② Qn 在某个变异版里把真缺陷判成相等；③ 第 1003 行在 E2 上不红（红例账第 121 行是入库产物，我没重跑第二段）。

## 四、T2 之三与 T1：十格的依据够不够得着，「改」「撤」的依据是不是同样打中被留下的形态

第二轮判决改、撤各格用的依据有四种形状：**A**「过字面的实现在合法历史上违例，而那不是实现缺陷」（第 1 格撤整条根用的，也是这一轮跑前判据第二条）；**B**「把一道判删掉，核照样绿」（第 2、10 格用的）；**C**「合法历史取窄，必红世界造不出反例」（第 2 格）；**D**「共用生成器的真相」（R8，第 1、4 格）。把每种形状放到被留下的形态上试：

| 形状 | 试的是哪一句 | 结果 |
|---|---|---|
| A | S1「其余字段等于所选根」 | **打中**：卸载记号（下面 1） |
| A | S6 的 txg 顶格 | 打中，两种写法各吸收一部分（第三节） |
| A | S4 的 M4（落到的恰是最远一版） | **没中**：light 合法一族除 U 验证关（测试专用臂）与 R 域（违反唯一前提）外，各域 J4 都是 0（`research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out` 第 161、234、343、404、570、675、757、806、837 行的 `j4_violations=0`）；X 域里下一个实例的记录不算（同实例） |
| B | S2 补强 | **打中别的判**（下面 2） |
| B | S7 的三处 `true`、S8 的读者规则判 | 打中：Verus 一侧删判仍绿（S8 的 follows 判，d09），S7 的 `true` 没有变异行（第六节） |
| C | S5 的域、S2 / S4 语义层前提取 E165 一族 | S5 的域内重合、域外分叉（第二节）；S2 / S4 取一族没中 |
| D | S4 的 J4、S1 的「盘」 | 仍是 E165 页自己写的「三条路径共用生成器与同一批条款」（`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:223`），本轮没有新的打击 |

### 1　S1：投影六字段与「盘」站得住，「其余字段等于所选根」被形状 A 打中

**够不着的部分**：E165 的 J1 比六个字段（实例、txg 与新根段四个，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:256` 的 `VersionFields`），J2 / J3 / J4 判「盘」三样，在上面那九行里都是 0（U 验证关的 J2、J4 是 352 608，那是只供测试的关点名验证的臂；R 域 956，违反唯一前提）。

**打中的部分**：实现重建出来的根有四个字段不在新根段里：fsid、实例表指针、分配记录树根照所选根抄（「instance_table: root.instance_table,」`crates/singlefs-core/src/recovery.rs:3153`；「allocation_record_tree_root: root.allocation_record_tree_root,」`crates/singlefs-core/src/recovery.rs:3157`），卸载记号却不抄：「unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence,」（`crates/singlefs-core/src/recovery.rs:3147`）。条款两处：D23 已定项 15「施加一条记录 = 把所选根的这四个字段换成记录里的」（`.claude/kb/decisions/23-journal的角色与格式.md:443`）没写记号；D22 已定项 7 的 flags 一行写「其余根写 0」（`.claude/kb/decisions/22-单元原子性怎么合成.md:137`），恢复那一行的注释引的就是它。所以写进 S1 的句子要带「除卸载记号外（恒 0）」，否则字面为假。

**这不是只有构造得出来的表**（[读][推]，没在池上量）：正常卸载「推了一串带卸载记号的空发布」（`crates/singlefs-core/src/mount.rs:1963`），第一版几何「下推 2 或 3 次」（`crates/singlefs-core/src/mount.rs:2064`），每次是一条完整发布（循环里调「let published = publish_version_with_unmount_marker(」，`crates/singlefs-core/src/mount.rs:2540`），空发布也写记录：「事务号 0 保留给不承载事务的记录，提交标记写 1」（`.claude/kb/decisions/23-journal的角色与格式.md:509`）。D16 已定项 7 的持久顺序是「journal 记录 → 屏障 → 根槽（FUA）」（`.claude/kb/decisions/16-发布语义.md:179`）。于是卸载那一串的第一条根落了（带记号）、第二条记录落了而它的根槽没落，是 D13 已定项 4 里的合法崩溃状态：所选根带记号，之后同实例有读得出的记录，恢复施加它，返回的根记号为 0，既不等于所选根也不等于第二次发布真发布出去的那一条根。

**E165 为什么看不见**：生成器把这四个字段全历史定死不变（「不在新根段里的字段（实例表指针、分配记录树根、卸载记号、fsid）全历史相同，不进判据。」`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:289`），`to_root` 里记号写死 0（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:293`）。我在副本上让时间线每一版的根带记号（`S1_MARKER=1`，J1–J4 的 `E7RESULT` 行与基线逐行相同），数「施加了至少一条」的态里返回的根与所选根在这四个字段上的差 [量]：合法一族十个域合计 1 867 682 个施加了记录的态，记号不等 1 867 682 个，fsid、实例表指针、分配记录树根不等 0 个（L 域 1 007 514 / 1 007 514，与第二轮攻方的「整条根不等 1 007 514」是同一批态）。

**判**：投影与盘够不着；「其余字段」同样打中，收严写法零轮：「其余字段（fsid、实例表指针、分配记录树根）等于所选根，卸载记号恒 0」。**推翻**：D13 已定项 4 的状态里找不到所选根带记号而之后同实例还有记录的——则只剩「写明记号」，不是字面为假。

### 2　S2：补强写得出来、证得过；「删判核照样验过」不止 I-8.3 一道

**我把补强写出来并证过**（[量]；原型 `research/prompts/m3-formal-r3-sonnet-model/verus/proto2-lib.rs`，相对冻结核加 151 行、改 6 行）：`m2_holds` 末尾加一条「存在锚点 a，使 a 是所选根那次发布读得出的记录里带末条标志的那一条的计数器（一条都没有则为 None），且施加的下标序列从链首起逐一连续」（spec 三个函数 `root_publish_last_flagged`、`anchor_is`、`chain_from_the_head`，`research/prompts/m3-formal-r3-sonnet-model/verus/proto2-lib.rs:94`、`:107`、`:125`；`m2_holds` 里的那一条在 `:166`）；锚点函数多一条 ensures 与两条循环不变量；主循环多两条不变量（已施加加攒着的合起来是链；期望键与链尾对应）。`verus` 直调 `--no-cheating`：「verification results:: 14 verified, 0 errors」（`research/prompts/m3-formal-r3-sonnet-model/verus/proto2.verus.out:2`），没有 `external_body` 与 `assume`。

**删判表（13 道 break 判各删一次：条件前加 `false &&`，Verus 0.2026.09.20.aef82ed）**：

| 删哪一判（冻结核 `lib.rs` 行） | 冻结核 | 补强原型 |
|---|---|---|
| I-8.3 连续（434） | **14 verified, 0 errors** | 13 verified, 1 errors |
| 无锚点链首（445） | **14 verified, 0 errors** | 13 verified, 1 errors |
| 反向链回调（452） | 14 verified, 0 errors | 14 verified, 0 errors |
| 一次发布首条序号 1（456） | 13 verified, 1 errors | 13 verified, 1 errors |
| 发布内换 txg（463）、跳号（467） | 13 verified, 1 errors | 13 verified, 1 errors |
| 事务边界（472） | 14 verified, 0 errors | 14 verified, 0 errors |
| 末条要带提交标记（480） | 14 verified, 0 errors | 14 verified, 0 errors |
| 读者规则 follows（484） | 14 verified, 0 errors | 14 verified, 0 errors |
| 点名单元验过（493） | 14 verified, 0 errors | 14 verified, 0 errors |
| 在飞上限（427） | 14 verified, 0 errors | 14 verified, 0 errors |
| 所选根 txg 顶格（353）、水位之上筛选（422） | 13 verified, 1 errors | 13 verified, 1 errors |

（原始行：冻结核 `research/prompts/m3-formal-r3-sonnet-model/verus/summary-variants.txt`，原型 `.../summary-variants2.txt`。）冻结核上 I-8.3 那一判删掉仍是 14 verified, 0 errors：第二轮判决主 agent 现跑的是 13 verified（该轮的核），冻结核现在是 14，核在两轮之间改过；「删了仍绿」这一结论我这次重量对上。冒烟与两行必红世界在原型上（`.../summary-variants3.txt`）：链条里「+1」改「+2」12 verified, 2 errors，首条「+1」改「+2」13 / 1，`expected_next` 改「+2」13 / 1，表第 36 行（无锚点时不看 txg）13 / 1；表第 448 行（等价变异）在原型上也红 13 / 1，是我的证明把序号为 1 直接取自那一判而不是取自后面冗余的那一判，补一条 assert 可以去掉，没做。

**判**：S2 补强站得住（写得出、证得过）；但形状 B 同样打中它没写的 6 道判（反向链、事务边界、提交标记、读者规则、点名单元、在飞上限）：补强之后它们删了仍绿。它们不是 S2 的文字，也不是 M2「不切发布」的字面（M2 的字面只要整次发布，序号与换 txg 三道已经红），E165 判它们（与它们对应的第二段变异：第 515、516、614、920–922、354 行与 Z2 都在第二段红，`research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out`），所以不是「没有任何东西判」，是「只有有界穷举判」——写回时 M2 的「证到了」要写明是哪几道。
**「表按 (实例代号, 计数器) 升序写成 `requires`」够不着**：我的原型没有任何升序前提，链条证明只用循环里的计数器相等那一判；升序只给读者规则那一判与它的 BTreeMap 原形同义（核注释自己写「这一条没有写成前置条件，也没有证」，`crates/singlefs-replay-prefix/src/lib.rs:149`），而它没有 ensures，写成 `requires` 进的是胶水义务（调用方 `records.values()` 的次序，`InstanceGeneration` 派生 `Ord`：「#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]」`crates/singlefs-core/src/address.rs:29`）。**S10**：今天的核对七·2 必红世界（删 I-8.3）报 14 verified, 0 errors，「spec 太弱」那份样本今天会判绿；补强之后判红，所以样本要排在补强之后，与第二轮判决写的次序相同。
**推翻**：第 4 批落地的补强让「删 I-8.3」或「删无锚点链首」仍判绿——说明他们写的 spec 比我这份弱；我这份在冻结核副本上证过，不等于他们的核。

### 3　S3：站得住，射程句够不着

原句两处 D23 已定项 14 原文逐字在：「根槽写失败重发时 checkpoint_txg 推进一格再发」（`.claude/kb/decisions/23-journal的角色与格式.md:396`）与「合法历史里同一 (实例代号, checkpoint_txg) 不会有两条带末条标志的记录」（`.claude/kb/decisions/23-journal的角色与格式.md:399`）。「唯一」在真路径上靠 `scan_journal` 把记录放进以 (实例代号, 计数器) 为键的表时第一份赢：「.or_insert(record);」（`crates/singlefs-core/src/recovery.rs:2924`），设备次序在前的那块盘赢。没有变异行改它，E165 不调 `scan_journal`（第二轮判决第五节第 2 条已记）。**判**：站得住；欠账补一笔（第六节）。**推翻**：合法写侧让同一 jsn 的两份内容不同。

### 4　S4：M4 与 Z1 读法站得住；「M4 加择根合起来才排除丢失」同样打中别的臂

**M4 站得住**：从入库的第二段产物重数（我写的 Python 读 `research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-kill-bounds.out` 的 `first_clause_red`，只数合法一族各域）[量]：首红例里只有 J4（带 J5a、J6 前提真，那几项在红例账里是变体少了违例）的变体 7 个：第 24、512、517、921、922、1004 行与 Z1；与第二轮攻方的「7 个」相同。**Z1 取「多出违例」站得住**：Z1 的 E1 反例是「+J4,-J6_premise_holds,-applied_ge1」（`research/results/e165-replay-bounded-enumeration-2026-10-01-segment2-red-examples.tsv:144`），J6 那一项是基线那 70 个顶格违例被拿掉，不是多出违例。

**「合起来」打中**：M4 说的是所选根同一实例时间线上的最远一版；「已落盘的完整发布不丢」还要三处合起来才成立，而其中择根那一处在 S5 的两个域外世界里分得出缺陷：① D22 已定项 7 的择根（txg 为主），② D23 注 1 的「下一条的实例代号与所选根不同即停」（`.claude/kb/decisions/23-journal的角色与格式.md:367`）：新实例第一次发布只有记录、根槽没落时，所选根还是旧实例的，恢复一条都不施加；③ D16 已定项 8 的暖机：「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回」（`.claude/kb/decisions/16-发布语义.md:196`），所以那一版没被确认。所以 S4 要写成「已**确认**的完整发布不丢」才站得住，而且域要写明（S5 的域）；在平局或倒挂世界（C88、C331）里它不成立 [译]。**推翻**：D16 已定项 8 的暖机不覆盖某一类新实例（则 ② 里的丢失是已确认的）。

## 五、T1 续：S5 推论、S6 两读、S7–S10

**S5 推论（第二轮第 5 格；站得住，域要写全）**：攻方四步推导每一步现查过依据（[读][推]）：①「所选根单调」靠 D13 已定项 4 的「整写子集」（标题「崩溃点重放只枚举整写子集」，`.claude/kb/decisions/13-验证路线.md:70`：写一个根槽要么整个落要么不落，没有撕裂的半个根）与 `choose_root` 对超集取最大；②「所选根之上至多一次发布有记录」靠串行提交（「事务层先承诺串行提交」，`.claude/kb/decisions/13-验证路线.md:45`）与持久顺序「journal 记录 → 屏障 → 根槽（FUA）」（`.claude/kb/decisions/16-发布语义.md:179`），下一次发布的记录要等上一次发布返回之后才写；③④ 用 J1（落点不早于所选根）与 J2（完整在盘的记录全施加）。三种版本序在这个域里重合（第二节），所以推论的结论与序无关。这个域的边是「不带故障、串行、持久顺序」，S5 自己已写「并行提交或域放宽到带故障时重开」。

**S6 幂等两读都证（第二轮第 6 格；站得住）**：记录级就是水位过滤——「record.instance == root_instance && record.checkpoint_txg > root_checkpoint_txg」（`crates/singlefs-replay-prefix/src/lib.rs:57`）：已施加的那条记录的 txg 等于重建根的 txg，第二次进不了候选集；I-8.4 的字面是「任一 journal 记录被重放两次与重放一次，产生的盘上状态相同」（`.claude/kb/invariants.md:87`），主语是记录，所以「七·4 的『就是 I-8.4』改成两条性质」站得住（七·4 原句在 `.claude/kb/milestone/03-third-txn.md:170`）。不动点：我这次在 light 合法一族上重数，J6 前提真的违例只在 E1（顶格，70 个），L、L6、L9、U、C、C6、X、R 全 0 [量]（`research/prompts/m3-formal-r3-sonnet-model/s6_table.tsv`）；L6、L9、C6 的前提假那几格（354、51 998、215）是在飞上限咬到的小环，本来就挂「不截断前提」。顶格那一格的写法见第三节。

**S7 主语（第二轮第 7 格；站得住）**：`replay_journal` 的生产调用三处，第五个参数现查：可写挂载读阶段「true,」（`crates/singlefs-core/src/mount.rs:4366`）、只读挂载「true,」（`crates/singlefs-core/src/mounted_read.rs:564`）、`recover`「policy == JournalPolicy::Consult,」（`crates/singlefs-core/src/recovery.rs:4191`，`Consult` 臂传真）。另有两处 `e158_root_choice_repair.rs` 的装置调用与一处 harness 用例，不是产品路径（用例 `crates/singlefs-harness/tests/row_publish_record_without_its_root.rs` 也传 `true`）。**欠账**：`crates/mutations.tsv` 里没有任何一行改这三处的实参（在冻结副本上 `grep -c 'records,\\n        true' crates/mutations.tsv` 输出 0，`grep -c 'policy == JournalPolicy::Consult' crates/mutations.tsv` 输出 0；碰 `mount.rs` 里 `replay_journal` 那一处的只有表第 358 行，它改的是调用之前的扫描，不动实参）[量]；E165 直接调函数，不经这三处。有没有现成用例在某一处改成 false 时红，我没跑（第七节「没做什么」）。**推翻**：有一处生产调用传 false，或 `ConsultWithoutNamedVerification` 被产品路径用到。

**S8 读者规则第四格（第二轮第 8 格；站得住，并补一条手证）**：S8 写「K10 下与全表看逐态相同」，第二轮的依据是 n ≤ 3 / 4 的量。它可以手证，三行：设 K10 =「同一实例里读得出的记录按计数器升序、checkpoint_txg 不减」。核里那一判从这一条之后第一条同实例记录起往后扫：遇到 txg 更大的返回 false，遇到 txg 相等的返回 true，别的继续（「if !(later.checkpoint_txg <= record.checkpoint_txg) {」`crates/singlefs-replay-prefix/src/lib.rs:173`，「if later.checkpoint_txg == record.checkpoint_txg {」`crates/singlefs-replay-prefix/src/lib.rs:177`）。K10 下后面没有 txg 更小的，而 txg 相等的都排在 txg 更大的之前，所以「表里有后面的同 txg 记录」⇔「扫描先遇到相等、没先遇到更大」，即全表看。这一步用到「切片按 (实例代号, 计数器) 升序」，它是胶水义务（第四节 2）。[推，没跑；K10 要写成「全表里同实例的读得出的记录」，不是「链上」。]

**S9（第二轮第 9 格；站得住，机器判不了）**：D13 已定项 13 的计数规则（「同源性要被观测」`.claude/kb/decisions/13-验证路线.md:286`、「累计 ≥ 5 次不一致而判定「spec 错」的次数为 0」`.claude/kb/decisions/13-验证路线.md:287`）加一格「条款欠一句」是记账规则：仓里没有代码在记 spec 与 model 的不一致事件，所以它只有「补句之后再判一次」这种人工核法，没有会红的东西（七·5 里写它的那一格 C191 是欠账）。

**S10 七·5 的判绿与样本（第二轮第 10 格；站得住，判绿四条件挡不住空真，同样打中）**：判绿 =「判定行原样、errors 0、verified 数等于登记、缓存命中不出判定行不算绿」；我在冻结核上给 `replay_core` 的 `requires` 加一条 `false`，Verus 报「verification results:: 14 verified, 0 errors」——四条件全过 [量]（`research/prompts/m3-formal-r3-sonnet-model/verus/summary-variants4.txt`）。这份空真样本因此要有额外的闸：我在核里加一个真调 `replay_core`（空记录表）的见证函数，基线 15 verified, 0 errors，空真版 14 verified, 1 errors（`error: precondition not satisfied`）[量]（`.../summary-variants5.txt`）；它不是照抄一遍前置条件，而是真调用，前置条件改了它自己跟着红；代价是登记的 verified 数跟着多一。七·2 的「spec 太弱」样本今天会判绿、补强之后判红（第四节 2）。**推翻**：Verus 对 `requires false` 的函数的核另有告警行、被判绿条件里的「判定行原样」截下。

## 六、T3：未定项清单还缺哪几笔

第二轮判决第五节第 2 条记了四笔（投影之外的根字段、E165 只看 0 号盘、E165 不调 `scan_journal`、`crates/mutations.tsv` 补 I-8.3 那一行）。按 `.claude/kb/checks-owed.md` 的写法（要拦什么输入、现状）补五笔，每笔的现状都在冻结副本上现查过：

| # | 哪一句没人判 | 要拦的输入 | 现状（[量] = 在冻结副本上数的） |
|---|---|---|---|
| 1 | S1「其余字段等于所选根」里的卸载记号（第一笔「投影之外的根字段」没写到它：那一笔说的是没人判，这里补的是**今天实现与这句字面相反**） | 所选根带 `WrittenByTheUnmountSequence`、之后同实例还有读得出的记录（卸载那一串第 2 条记录落了、它的根槽没落） | 重建根记号恒 0（`crates/singlefs-core/src/recovery.rs:3147`）；E165 生成器造不出记号根（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:293`）；`crates/mutations.tsv` 里改 `unmount_marker: UnmountMarker::NotWrittenByTheUnmountSequence`、`instance_table: root.instance_table`、`allocation_record_tree_root: root.allocation_record_tree_root` 这三行的变异各 0 行 [量]；副本上让根带记号，施加了记录的 1 867 682 个态记号全不等，J1–J4 的行与基线逐行相同 [量] |
| 2 | S7 三处生产调用传 `true`（`crates/singlefs-core/src/mount.rs:4366`、`crates/singlefs-core/src/mounted_read.rs:564`）与 `Consult` 臂传 `policy == JournalPolicy::Consult`（`crates/singlefs-core/src/recovery.rs:4191`） | 把任一处实参改成 false（只读挂载或可写挂载不验点名单元） | 变异行 0（第五节 S7）；E165 直接调函数、不经这三处；有没有现成用例会红没跑 |
| 3 | S3 的「唯一」在真路径上由 `scan_journal` 的第一份赢担保（`crates/singlefs-core/src/recovery.rs:2924`） | 两块盘上同一 (实例代号, 计数器) 的内容不同，取哪份 | `or_insert(record)` 的变异行 0 [量]；E165 自己建 `records` 表、不调 `scan_journal`（第二轮已记）；设备次序在前的那块盘赢，没有条款写这一格 |
| 4 | M2 的「证到了」只到 S2 文字写的那几句：补强之后 Verus 一侧删判仍绿的 6 道（反向链、事务边界、提交标记、读者规则、点名单元、在飞上限） | 删掉其中一道判的变体 | 只有有界穷举判（对应的第二段变异红）；Verus 一侧哪几道能写成 spec 没试：提交标记、事务边界、在飞上限三道的 spec 是对施加序列的逐条谓词、与 S2 同形 [推]，反向链与点名单元是回调、只能写成「回调交回真」，读者规则要升序前提 |
| 5 | 择根序（S4「合起来」、S5 版本序所依赖的 `choose_root`） | 同 txg 的两条根（高实例代号应赢）；旧实例更高 txg 的根与新实例的根并存（C331） | 改 `candidate_key` 一行的变异 0 行 [量]；E165 把所选根直接交给 `replay_journal`、不经择根；C88、C331 已记欠账（`.claude/kb/checks-owed.md:88`、`.claude/kb/checks-owed.md:277`），本轮补的是「M3 / M4 的写回依赖它而它没有会红的东西」 |

另有一笔是写法上的：七·5 的空真样本要有「真调一次 `replay_core`」的见证函数（第五节 S10），否则判绿四条件放过它；这一笔归第 4 批里写门禁的人，不另立欠账号。

## 七、量到的：命令与原样输出

模型与脚本都在 `research/prompts/m3-formal-r3-sonnet-model/`（`README.md` 写了复跑步骤）。线程一律 `capped.sh 4`，内存 6G（Verus 批 1G），均经 `research/scripts/run-with-memory-cap.sh`；起跑前 `ps` 看过，没有 qemu、vm-bench、e152、fio，别的会话在跑 `cargo test` 与 `broad-mutation`。

**冻结副本**：`cd /tmp/claude-1000/m3-formal-r3/tree && sha256sum -c research/prompts/m3-formal-r3-snapshot/crates-sha256.txt`，不是 OK 的 0 行、OK 的 212 行。

**装置副本的编译**（只加计数的补丁 `patch_e165.py` 与 `e165/patch_e165_precontrol.py`）：`capped.sh 4 cargo build --release --locked --offline -p singlefs-checker-tier --bin e165_replay_bounded_enumeration`，末两行 `Finished \`release\` profile [optimized + debuginfo] target(s) in 2m 40s`、`exit=0`；第 406、513、518、1003、1004、24、512、36 行的变异版各重编一次，退出码全 0（`build-variants.log`：`row 513 build exit=0` … `row 36 build exit=0`；第 406 行版 `exit=0`）。

**包装的 252**：第一次起跑 `run-with-memory-cap.sh 6G bash capped.sh 4 ./bin-baseline-patched --bounds light --family legal` 退 252：「这个会话（…）已经有一条日常的在跑，同一个会话同时只许一条；命令没跑」，在跑的是别的任务的 `cargo test` 与 `cargo build`。重试十几次都是 252（别的任务一个接一个占着槽），之后设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON` 才起得来（头部已写明）。252 那些次的输出不算结果，没有用。

**装置没动判据**：`diff <(grep '^E7RESULT' base-legal.out) <(grep '^E7RESULT' research/results/e165-replay-bounded-enumeration-2026-10-01-light-legal-family.out)` 无输出，`diff exit=0`；打了补丁的基线 876 行 `E7RESULT`，入库产物同为 876 行。

**八个变异版 × light 两族**（`batch-e165.sh`，一次包装一条脚本）：退出码 `base-legal exit=0;base-adversarial exit=0;base-legal-marker exit=0;v406-legal exit=0;v406-adversarial exit=0;v513-legal exit=0;v513-adversarial exit=0;v518-legal exit=1;v518-adversarial exit=0;v1003-legal exit=0;v1003-adversarial exit=0;v1004-legal exit=1;v1004-adversarial exit=0;v24-legal exit=0;v24-adversarial exit=0;v512-legal exit=0;v512-adversarial exit=0;v36-legal exit=0;v36-adversarial exit=0;batch-done`。第 518、1004 行版退 1 是装置自己的停机条款（`halt_conditions=6` 之类：变异让第一次调用就报错，装置按登记停机并照常打完整份输出），输出完整（第 518 行版合法一族 987 行、`name=done emitted=882`）；`extract/sha256-full-outputs.txt` 是 19 份完整输出的 sha256。
**汇总核对**：`python3 s6_tabulate_v2.py` 出 126 行，其中「我数的当前判据违例」与装置自己的 `j6_premise_holds_violations` 不等的 0 行（`awk -F'\t' 'NR>1 && $8!="-" && $4!=$8' s6_table.tsv | wc -l` 输出 0，`awk 'END{print NR-1}' s6_table.tsv` 输出 126）。

**Verus**（`verus --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating src/lib.rs`，Verus 0.2026.09.20.aef82ed，每个变体一次，经包装 1G 或 auto）：判定行汇总逐字如下。

冻结核删判（`verus/summary-variants.txt`）：
```
base	verification results:: 14 verified, 0 errors	exit=0
d01_continuity_I83	verification results:: 14 verified, 0 errors	exit=0
d02_chain_head_without_anchor	verification results:: 14 verified, 0 errors	exit=0
d03_back_chain	verification results:: 14 verified, 0 errors	exit=0
d04_first_ordinal_is_1	verification results:: 13 verified, 1 errors	exit=1
d05_txg_change_in_open_publish	verification results:: 13 verified, 1 errors	exit=1
d06_ordinal_skip	verification results:: 13 verified, 1 errors	exit=1
d07_transaction_boundary	verification results:: 14 verified, 0 errors	exit=0
d08_commit_mark_of_last_record	verification results:: 14 verified, 0 errors	exit=0
d09_reader_rule_follows	verification results:: 14 verified, 0 errors	exit=0
d10_named_units_not_verified	verification results:: 14 verified, 0 errors	exit=0
d11_in_flight_limit	verification results:: 14 verified, 0 errors	exit=0
d12_top_txg_check	verification results:: 13 verified, 1 errors	exit=1
d13_above_the_root_filter	verification results:: 13 verified, 1 errors	exit=1
```
补强原型上同一张表（`verus/summary-variants2.txt`）：
```
base	verification results:: 14 verified, 0 errors	exit=0
d01_continuity_I83	verification results:: 13 verified, 1 errors	exit=1
d02_chain_head_without_anchor	verification results:: 13 verified, 1 errors	exit=1
d03_back_chain	verification results:: 14 verified, 0 errors	exit=0
d04_first_ordinal_is_1	verification results:: 13 verified, 1 errors	exit=1
d05_txg_change_in_open_publish	verification results:: 13 verified, 1 errors	exit=1
d06_ordinal_skip	verification results:: 13 verified, 1 errors	exit=1
d07_transaction_boundary	verification results:: 14 verified, 0 errors	exit=0
d08_commit_mark_of_last_record	verification results:: 14 verified, 0 errors	exit=0
d09_reader_rule_follows	verification results:: 14 verified, 0 errors	exit=0
d10_named_units_not_verified	verification results:: 14 verified, 0 errors	exit=0
d11_in_flight_limit	verification results:: 14 verified, 0 errors	exit=0
d12_top_txg_check	verification results:: 13 verified, 1 errors	exit=1
d13_above_the_root_filter	verification results:: 13 verified, 1 errors	exit=1
```
原型上的冒烟与必红世界（`verus/summary-variants3.txt`），空真样本（`verus/summary-variants4.txt`）与带见证函数的两份（`verus/summary-variants5.txt`）：
```
base	verification results:: 14 verified, 0 errors	exit=0
code_smoke_expected_plus2	verification results:: 13 verified, 1 errors	exit=1
spec_smoke_chain_plus2	verification results:: 12 verified, 2 errors	exit=1
spec_smoke_head_plus2	verification results:: 13 verified, 1 errors	exit=1
tsv36	verification results:: 13 verified, 1 errors	exit=1
tsv448	verification results:: 13 verified, 1 errors	exit=1
vacuous_requires_false	verification results:: 14 verified, 0 errors	exit=0
base_plus_witness	verification results:: 15 verified, 0 errors	exit=0
vacuous_requires_false_plus_witness	verification results:: 14 verified, 1 errors	exit=1
```
**版本序译本**：`python3 s5_version_order_model_v2.py` 的输出就是第二节的表（`s5_out_v2.txt`，md5 与重跑一致：`4209d081b1246cc6f6178507a47afc22`）。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；两格零轮我各给了分辨观测，选哪一种写法交主 agent 与用户。
- 没在入库装置上重做第二轮攻方的数（T4 归攻方）。我的数在副本上量：E165 装置副本只加计数，打了补丁的基线 876 行 `E7RESULT` 与入库逐行相同；Verus 在草稿目录的核副本上跑。第二轮的 1 007 514 我只侧面对上一次（S1 的卸载记号那一格里 L 域施加了记录的态数）。
- 没跑第二段杀伤界整表，没跑 `--bounds segment1` / `segment2`：只 light 档，变异版 8 个（表第 24、36、406、512、513、518、1003、1004 行），不是 26 个；第 1003 行在 E2 上的红引的是入库的红例账第 121 行，没重跑。
- S5 版本序的数是译本（选根器一行比较，译自 `crates/singlefs-core/src/recovery.rs:1161`），不是真 `choose_root`、真池上量的；S1 卸载记号的「合法崩溃状态」是读 `mount.rs` 的卸载那一串与 D16 已定项 7 推的，没在池上量；E165 副本上的 1 867 682 是让时间线根带记号后的态数，不是卸载那一串的录制流。
- Verus 原型只补了 S2 文字写的那一条（连续与链首）；提交标记、事务边界、在飞上限三道能不能写成 spec 没试，反向链与点名单元是回调类、读者规则要升序前提，都是推的；原型没并进 `crates/`、不替第 4 批做，它的数不等于第 4 批的核。
- 没跑 harness 档用例（没验 S7 的 `true` 改成 false 时哪些用例红）、没跑 checker 档测试、没跑 `cargo test`、没跑门禁、没做 git 写、没改 kb 与 `crates/`。
- 没读禁读清单里的文件（`research/prompts/m3-formal-r3-opus-output.md`、`research/prompts/m3-formal-r3-opus-model/`、第 4 批四个草稿目录）。
- 自己设了 `RUN_WITH_MEMORY_CAP_PARALLEL_REASON`（头部已写）。
- 草稿目录 `/tmp/claude-1000/m3-formal-r3-sonnet/`：交回前删掉了我自己建的编译目录与仓副本（`work/e165`，含 `work/e165/target`；`work/base`；`verus/base-crate`；九个重编出来的二进制 `work/bin-*`），都能按模型目录里的脚本重建；留着的是运行输出 `work/runs/`、`verus/variants*` 里每个变体的 `verus.out`、`seg/` 报告草稿、脚本与汇总表，都不是编译目录。删前 `du -sh` 的数写在交回里。

交回前的引文核对（`python3 research/scripts/cite-check.py research/prompts/m3-formal-r3-sonnet-output.md --root /tmp/claude-1000/m3-formal-r3/tree --root .` 的末行，原样）：`✓ 核了 36 处引文，对上 36 处，没判 0 处（逐处列在上面）`
