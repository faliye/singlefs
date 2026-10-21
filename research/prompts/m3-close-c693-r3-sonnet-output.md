# m3-close-c693-r3 云端辩方腿报告（Sonnet，2026-10-20）

立场：辩方。复核前两轮判决（`research/prompts/m3-close-c693-r1-main-verification.md`、`research/prompts/m3-close-c693-r2-main-verification.md`）在 T1（兼 T3）上够不够得着、是不是同样打中所有替代方案；替第一轮被判写错的「U5 空真」读法与第二轮被打中的「按统计量」臂找站得住的理由，站不住的照写。方案按 `crates/` 今天的实现谈。每格标「量过」（本腿跑出的输出，进模型目录 `out/`）或「推的」（读条款、读码推的）。T2、T4 不归本腿。这是最终轮：本轮新冒出的形态标「零轮」。没读禁读清单里的文件。

## 复跑命令与 sha256

```
# 仓根；整条经内存包装；共用内存额度占满时前面加 RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-c693-r3-sonnet-model/tools/rerun.sh \
  --repo . --frozen research/prompts/m3-close-c693-r3-snapshot --work <新的草稿目录>
# 末尾 name=rerun_compare 四行都 identical=1，即复出 out/ 下四份；线程数取 .claude/heavy-thread-default（31）
```

读的入库产物（开工后现算）：

```
f1dfd78bc92b7ed919af4484e334b379b615f3914b71a5cf36f513843439a86a  research/results/e186-unknown-tree-accounting-rederive.out.gz
ef2bd29e33654aa9d3dab846a98bd297d9fa74c14fbd096648da68316197705e  research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz
29cafdd37626b06fed52b4c3b8c66ce302be74a7e516cb0227e92e5e51610e58  research/results/e97-entry-encoding-single-generation-2026-10-20.out
885c238a93c4519e58a4dd1f1251ebd0669f25bdbca7629ee21acfe94caa9c07  research/results/e71-accounting-keys-single-generation-2026-10-20.out
```

模型目录 `research/prompts/m3-close-c693-r3-sonnet-model/` 每个文件的 sha256：

```
303bd348ff4ac8fcc2db0845e9d1e3daebe640e59bdefcf5d33f31f995512a19  accounting_key_width_leaf_capacity.rs
7ae4ee37203bb92f415504f569b5c46ec2af24cc96f8514c92a5dd18a17ea78e  accounting_planner_per_statistic_arm_node_cost.rs
cbbb86b502a038b98ea95a558783d5a379d7e5bdee0a8c892295229e7607cf67  planner_claims.py
c708622c8baaa04c63f806b6ec5e159ef841a4b043c6256d12bbd8ca81ceca43  reading_matrix_wide.py
d13bc9e84caea27b58edcccfcd56476b6900b7a951d1a3e02e4fbbcf6bf7e52d  tools/rerun.sh
71758ce070cd3a61a554d5a1758f98c21beea06380167b4f044b6d3a9aa477b1  out/capacity.out
715720512664e0144ad7ef803943f9365a35c469101eacb2d836dd372e3d0e80  out/planner-claims.out
9344e9302e7a68db85c8d62bbe8c6687240e1eb15db23b9bc69871ba6a86e871  out/planner.out
76c590e40499fdccde8e3603962bd5c666f8e1f150ba0180829e9fac41f3e22d  out/reading-matrix-wide.out
```

两条 harness 测试目标是我加进草稿目录仓副本的测试文件（`crates/` 一字不改），不进主仓；规划器是 `crates/` 自己的 `plan_the_tree_after_this_publish`，容量取 `of_the_node_format(22, 34)`，断言等于 477 / 150。三份脚本各带会红的自证与弄坏开关（`MATRIX_BREAK=swap_acct`、`PLANNER_CLAIMS_BREAK=shift_carry`），见各脚本文件头。

## 判定一览

| 判决条目 | 辩方判定 | 证据（细节见后文各节） |
|---|---|---|
| 第一轮：U5 空真是「判据写错」 | 第二轮的更正站得住：空真是 B1 理由句的一致用法，不是写错 | 量过：第二次跑每格去掉 U5 后逐格等于第一次跑（172800 格 0 处不等）；见第 1.1 节 |
| 第二轮：㊀ 翻不翻面由读法定，不由量出的数定 | 站得住，且够得着；读法矩阵的四维够用 | 量过：128 种读法（多罩 W1/W2、㊁ 计不计、位次、逐 flags 臂层），第 6 位在前时 Carry 不处处在胜者里的只有「空真、㊀ 不计、按行」一类，W1/W2、㊁、Q2、逐 flags 层都不改；第 1.2、1.3 节 |
| 第二轮：B1 的射程该不该写到 D8:383 那半句 | 不是辩方能定：按理由句要罩、按字面射程不罩、按第 6 位的用途又不该罩；三种读法落在同一张四格表里，没有第五种结果 | 推的（读登记与条款）加量过（矩阵）；第 1.4 节 |
| 第二轮：NoCarry 每次发布省 h_new 行 | 站得住 | 量过：第二次跑产物点 6572、6573 的 Carry 与 NoCarry 行数；第 2 节 N1 |
| 第二轮：台阶行长时省 h_new 个节点 | 站得住但门槛要写：记账树出一片叶（超过 477 行）才有；之前四条臂节点数全等 | 量过：副本规划器 216 格，单叶 42 格全等，多叶 58 格里 Carry 比 NoCarry 多 0 到 4 个；第 2 节 N3 |
| 第二轮：Carry 只修 261、282、287 三处措辞 | 部分站不住：282、287 里是 C693 指针，各臂同删；261 要补台阶行一句；383 不动；恢复里 3 + 6 × 盘数那一判不是 Carry 独有 | 推的；第 2 节 C4 至 C6 |
| 第二轮：「按统计量」臂（零轮）在 r_tree_dim=2 独胜 | 同样打中别的臂：它与 NoCarry 死于同样两处（按节点、㊀ 计），r_tree_dim=1 与 NoCarry 逐字段相同；在「空真、㊀ 不计、按行」读法下它是唯一每点都在胜者里的臂 | 量过（派生臂，不是装置的一次跑）；第 3 节 |
| r_tree_dim=2 是假设取样点 | 站得住，且承重：Carry 在那一档赢，靠的是「第二个统计量没有台阶行」这条造法；有台阶行时 NoCarry 不丢元组，同一读法下独胜 | 量过（派生，零轮）；第 4 节 |
| T3：D5 五句改后条文 | 骨架站得住；三处取整或口径与产物不一致，一处漏变量（评审已采） | 量过：叶容量 491 / 477、79 到 141 块盘、E97 与 E71 产物行；第 5 节 |

## 1 T1：读法矩阵够不够罩全

### 1.1 做了什么（量过）

`reading_matrix_wide.py` 只读两份 E186 入库产物的 cell 与 point_verdict 行（172800 格），不读装置源码，不 import 别的腿的脚本。比第二轮正推的 16 格矩阵多罩四样：㊀ 与 ㊁ 拆开计（第二轮把两样绑成一个 B）；第 7 位的行数口径 W1（代段等于本版 txg 的行）与 W2（记账树全部行）；位次（第 6 位在前是用户定的，第 7 位在前只报）；判胜者的层（十五格总胜者与逐 flags 臂各一层）。轴：U5 两种 × ㊀ 计与不计 × ㊁ 计与不计 × Q2 计与不计 × 第 7 位四种（W2、W1、W2 按叶数、W1 按叶数）× 位次两种 = 128 种读法，每种在被判点（合法点里 `h_new > 0`，6400 点）上报记账胜者集合。三条自证都对，原样：

- 「E7RESULT name=selftest_second_minus_u5_equals_first cells=172800 mismatches=0」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:1`)
- 「E7RESULT name=selftest_b2_minus_b1_equals_u12_plus_u13 cells=172800 mismatches=0」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:2`)
- 「E7RESULT name=selftest_second_run_pooled mine=Carry:6400 product=Carry:6400 equal=1」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:3`)

第三条之后还有第一次跑产物的两个分组对账（同一文件第 4、5 行，`equal=1`）。弄坏开关 `MATRIX_BREAK=swap_acct` 把 NoCarry 与随根留着对调：第 4 行 `equal=0`、末行 `selftest_ok=0`、退出码 3（本腿跑过）。

### 1.2 第 6 位在前时的四类读法（量过）

| 读法 | r_tree_dim=1 的胜者 | r_tree_dim=2 的胜者 | Carry 处处在胜者里 |
|---|---|---|---|
| U5 空真、㊀ 不计、第 7 位按行（W2 或 W1 同） | NoCarry（lpc=1 时与随根留着并列） | Carry | 否 |
| U5 空真、㊀ 不计、第 7 位按叶数 | Carry 与 NoCarry 并列 | Carry | 是 |
| U5 要改（第二次跑）、㊀ 不计，行或叶数 | Carry | Carry | 是 |
| ㊀ 计（U5 任一种、第 7 位任一种） | Carry | Carry | 是 |

逐行出处（同一文件）：

- 空真、㊀ 不计、W2 按行：「u5=vac fst=skip snd=skip q2=counted seventh=w2 order=67 r1=NoCarry:1280|NoCarry+KeepWithRoot:1920 r2=Carry:3200 in_every_judged_point=none」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:6`)
- 同上 W1 按行：「u5=vac fst=skip snd=skip q2=counted seventh=w1 order=67 r1=NoCarry:1280|NoCarry+KeepWithRoot:1920 r2=Carry:3200 in_every_judged_point=none」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:8`)
- 空真、㊀ 不计、按叶数：「u5=vac fst=skip snd=skip q2=counted seventh=w2n order=67 r1=Carry+NoCarry:1280|Carry+NoCarry+KeepWithRoot:1920 r2=Carry:3200 in_every_judged_point=Carry」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:10`)
- 第二次跑的读法：「u5=dang fst=skip snd=skip q2=counted seventh=w2 order=67 r1=Carry:3200 r2=Carry:3200 in_every_judged_point=Carry」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:70`)
- ㊀ 计：「u5=vac fst=count snd=skip q2=counted seventh=w2 order=67 r1=Carry:3200 r2=Carry:3200 in_every_judged_point=Carry」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:38`)

### 1.3 没改结论的轴（量过）

- 第 6 位在前的 64 种读法里 Carry 不处处在胜者里的只有 8 种（`grep -c` 数出 64，`grep -vc` 数出 8），8 种全是 `u5=vac fst=skip` 且第 7 位按行，㊁ 计不计、Q2 计不计、W1 对 W2 各占一倍。所以 ㊁、Q2、W 口径都不是承重轴，第二轮的四维（U5、㊀、第 7 位按行或节点、Q2）加用户定的位次已经罩全；其中 Q2 在这张表上也不改任何一格。
- 逐 flags 臂那一层与十五格总胜者同形：「E7RESULT name=per_flags_level u5=vac fst=0 snd=0 q2=True seventh=w2 order=67 r_tree_dim=1 flags=Skip winners=NoCarry:1280|NoCarry+KeepWithRoot:1920」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:535`)，RejectEntry 一行同数（第 536 行）。
- 取样变量里，r_tree_dim=1 上能把 NoCarry 与 NoCarry 加随根留着分开的只有 `lpc`，几何、`g_gens`、`s`、`w_gens`、`h_new`、`registered`、`k_keep` 都不分：「variable=lpc 0->NoCarry:1280 1->KeepWithRoot+NoCarry:1920」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:577`)。
- 位次取第 7 位在前（用户没定，只报）：64 种里 32 种 Carry 不处处在，r_tree_dim=2 上随根留着会独胜，如「u5=vac fst=skip snd=skip q2=counted seventh=w1 order=76 r1=NoCarry:1280|NoCarry+KeepWithRoot:1920 r2=KeepWithRoot:3200 in_every_judged_point=none」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:9`)。这一格是用户已定位次之外的读法，判决不依赖它；但它说明第 6 位在前这个定案本身就在替 Carry 挡掉一类结果，交用户时一句话带上。

### 1.4 为 U5 空真辩护：站得住的与站不住的

站得住（量过或读原文）：

1. 第一轮判「U5 判据写错」不成立，第二轮更正站得住。空真与 B1 的理由句同向：「6. **读法 B（被判的那一项计不计）**：取 B1：D8 已定项 8 的 ㊀（连同射程里说 ㊀ 的句子）不计进记账维三条臂」(`research/prompts/e186-preregistration.md:29`) 后面接的理由是每条非现任臂都要改它，计进去等于先给现任臂一票。D8:383 那半句「含已定项 8 ㊀ 那类旧写者搬运的记账行」(`.claude/kb/decisions/08-核心索引结构.md:383`) 只因 ㊀ 取搬运才存在，取 NoCarry 或随根留着都要删它，理由句逐字套得上。第一次跑的谓词原文：「写者这次为不认识的树写进去的行，代段 = 这次的 txg；没写就空真」(`research/prompts/e186-preregistration.md:433`)。
2. 第二次跑与第一次跑只差 U5 这一处：1.1 的第一条自证（去掉 U5 后 172800 格逐格相等）。所以「空真读法」不是另一个装置的产物，是同一张格表上的一种取法，两次的数可以直接并列。
3. 谓词对 Carry 与对 NoCarry 是同一把尺：按内容判、不按「要不要动笔」判。Carry 在点 6572 上 `clauses_false=none`，它自己要做的措辞改动（第 2 节 C4）任何读法下都记 0；NoCarry 的 383 在空真读法下也记 0，两边都不记「动笔」。这是对称的。

站不住或只能半站：

4. 空真读法的前提「现任臂不用改 ㊀」在文字上不完全成立：Carry 也要补 261 一句台阶行（第 2 节 C4，推的）。但那一句是台阶行例外带来的，三条臂都有这个例外，补法各臂同做，不构成差别。差别只剩 383 这一处。
5. 第 6 位的名字是「要改几条已定条款」：「用户 2026-10-20 定：在 E113…的字典序」(`research/prompts/e186-preregistration.md:13`)。383 是已定项 12 的一句，不是 ㊀ 自己；按这一位的用途（给改别的已定项记账），它该记。这是对空真读法最重的一击，辩方答不掉，只能指出它与 B1 理由句是两条原则，选哪条不是数能定的，归用户（推的）。
6. B1 字面射程只到「射程里说 ㊀ 的句子」，D8:383 在已定项 12 不在已定项 8 的射程里，字面不罩；理由句罩。所以「B1 的射程该不该写到那半句」不是补一个漏洞，是在字面与理由之间选一个，写进登记就是改了读法。

### 1.5 一句话给用户（建议写法）

「㊀ 取哪一臂，取决于一件事：删 D8 已定项 12 里『含已定项 8 ㊀ 那类旧写者搬运的记账行』这半句，算不算『要改一条已定条款』。算（或把 ㊀ 自己也计进去，或第 7 位按叶数数）⇒ Carry 在今天唯一的取样点上不输；不算且第 7 位按行数 ⇒ NoCarry 每次发布少写 h_new 行，在 E186 的取样区里这是 0 个节点。」其余的轴（㊁、Q2、行数口径、逐 flags 层、取样变量）都不改这个分岔（1.3）。

## 2 T1：各臂代价（交用户用，逐条标出处）

出处约定：「第二次跑产物」= `research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz`，「第一次跑产物」= `research/results/e186-unknown-tree-accounting-rederive.out.gz`；两份产物同一点的行号相同。「副本规划器」= 本腿模型目录的 `accounting_planner_per_statistic_arm_node_cost.rs`，在草稿目录的仓副本里编、跑，不是入库装置的一次跑（交用户前要么主 agent 复跑，要么做成入库装置的附带量，照 `.claude/rules/three-way-inference.md` 判决那一条）。

### 2.1 NoCarry

- N1 每次发布少写 `h_new × r_tree_dim` 行（量过）。点 6572（实现两盘几何、`h_new` = 4、`r_tree_dim` = 1）：「E7RESULT name=cell point=6572 arm=RejectEntry_Carry writable_gens=10 … rows_written_w2=32」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:105160`)，「E7RESULT name=cell point=6572 arm=RejectEntry_NoCarry writable_gens=10 … rows_written_w2=28」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:105161`)，差 4。点 6573（`r_tree_dim` = 2）：37 对 29，差 8（同文件第 105176、105177 行）。
- N2 在 E186 的取样区里这笔省下的行是 0 个节点（量过）。两份产物全部 cell 行的 `rows_written_w2` 最大值都是 47（`grep ' name=cell ' <产物> | grep -o 'rows_written_w2=[0-9]*' | cut -d= -f2 | sort -n | tail -1`，本腿跑出 47、47），小于叶容量 477：「E7RESULT name=claim_a_single_leaf lines=42 all_four_arms_equal=42」(`research/prompts/m3-close-c693-r3-sonnet-model/out/planner-claims.out:2`)，即记账树装得进一片叶的 42 格，四条臂每次发布重写的节点数全等。
- N3 台阶行长时才省节点，门槛是记账树出一片叶（量过，副本规划器）。216 格里树高 1 当且仅当行数不超过 477（0 处例外；扫到的最大树高 1 行数 453、最小树高 2 行数 512）。`r_tree_dim` = 1 的多叶 58 格里，Carry 比 NoCarry 多重写的节点数分布是 0 个 18 格、1 个 8 格、2 个 15 格、3 个 6 格、4 个 11 格，最大值等于 `h_new`：「E7RESULT name=claim_b_multi_leaf r_tree_dim=1 second=0 lines=58 …」(`research/prompts/m3-close-c693-r3-sonnet-model/out/planner-claims.out:3`)。出一片叶不是充分条件（那 18 格多叶仍为 0）：尾部几行与别的被改写的行共一片叶时不多写。第二轮攻方的 500 级、2000 级点与本腿的 96 个重叠（格 × 臂）逐个相等：「E7RESULT name=selftest_overlap_with_r2_opus cells_x_arms=96 mismatches=0」(`research/prompts/m3-close-c693-r3-sonnet-model/out/planner-claims.out:1`)。扫到的台阶行条数里，最小的「Carry 多写节点」出现在每头 100 级（64 盘、4 个头）到 300 级（2 盘、2 个头）之间（同文件第 6 至 13 行）。所以写进 ㊀ 射程的代价句应是：「每个不认识的头的台阶行与别的行共一片叶时 0 个节点；记账树超过一片叶（477 行）之后，每次发布最多多重写 `h_new` 个节点」，变量是记账树总行数（盘数、台阶行条数都进去），不是 `r_tree_dim`。
- N4 第 6 位（要改几条已定条款）：空真读法记 0；第二次跑的读法记 1（383 那半句）；㊀ 计时另加 U12。同一点同一臂的第一次跑与第二次跑向量：「E7RESULT name=cell point=6572 arm=RejectEntry_NoCarry writable_gens=10 … vector_q2_counted=0,0,0,0,0,1,28」(`research/results/e186-unknown-tree-accounting-rederive.out.gz:105161`)，「E7RESULT name=cell point=6572 arm=RejectEntry_NoCarry writable_gens=10 … vector_q2_counted=0,0,0,0,0,2,28」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:105161`)；第 6 位的 1 与 2 里都含 Q2 对 RejectEntry 加的 1。量过。
- N5 要改 D8 的行（推的，读条款）：261 整句换成不搬；282 里「不搬的代价」一句换成不搬的依据；287 的取舍结论改写；383 删半句。282、287 里的「K 代说法已撤回…C693」是指针，不论取哪一臂，C693 销账时都要删。
- N6 水位元组靠台阶行留着，台阶行靠一条没实现的条款（量过加推的）。台阶行为 0 的点（`s` = 0、`lpc` = 0）：「E7RESULT name=cell point=6564 arm=RejectEntry_NoCarry writable_gens=10 … vector_q2_counted=0,0,4,0,0,3,16」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:105033`)，第 3 位（丢元组）= 4，同一行 `clauses_false` 含 U8（I-9.11：每个可写头至少一行水位行）。合法点上不丢，靠的是「克隆头的台阶行按 (统计量, 树) 分，建克隆那次给新头留一行」(`.claude/kb/decisions/05-快照-空间记账机制.md:55`)；这一条在 `crates/` 是 TODO：「`crates/` 今天没有克隆路径」(`crates/singlefs-core/src/transaction.rs:3682`)。所以 NoCarry 在 `r_tree_dim` = 1 上「不丢元组」要等克隆路径落地才成立；Carry 不靠它。
- N7 `r_tree_dim` = 2 时丢第二个统计量的元组（量过）：「E7RESULT name=cell point=6573 arm=RejectEntry_NoCarry writable_gens=10 … vector_q2_counted=0,0,4,1,0,2,29」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:105177`)，第 3 位 4（丢元组）、第 4 位 1（没有不变量盖）。这是造出来的取样点（第 4 节）。
- N8 改的是用户 2026-09-06 的定案：「用户定案 2026-09-06：四问…㊀ ㊁」(`.claude/kb/decisions/08-核心索引结构.md:288`)。K 代撤回让 ㊀ 的取舍要重推（C693），重推的结论改 ㊀ 要用户认；取 Carry 不改定案。

### 2.2 Carry

- C1 每次发布多写 `h_new × r_tree_dim` 行（N1 的反面）；节点数 N3 的反面：单叶时与 NoCarry 相同，多叶时最多多 `h_new` 个（`r_tree_dim` = 2 时副本里量到最多 5，边界 `h_new × r_tree_dim` = 8）。
- C2 第 6 位：U5、U12 的内容都满足，任何读法记 0：「E7RESULT name=cell point=6572 arm=RejectEntry_Carry writable_gens=10 … vector_q2_counted=0,0,0,0,0,1,32」(`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz:105160`)，`clauses_false=none`。
- C3 取 Carry 不动 383；动 261、282、287 的措辞（推的，第二轮正推 V1.3 的改后条文）。
- C4 261 要补一句：台阶行不换代段，上一版那次发布水位变了时同一行既是现行行又是台阶行。台阶认法在码里只取第一条：「let row_of_this_version = self.accounting_entries.iter().find(」(`crates/singlefs-core/src/transaction.rs:3696`)。282、287 只是删 C693 指针（各臂同删）。
- C5 实现：`accounting_entries_of_this_publish`（`crates/singlefs-core/src/transaction.rs:6931`）加一路「上一版里写者不认识的头的行」输入（推的，第二轮正推 V1.5 第 3 条）。
- C6 恢复里「记账条目数是 3 + 6 × 盘数」那一判：「if entries_other_than_the_inode_number_watermark_steps != 3 + 6 * device_count {」(`crates/singlefs-core/src/recovery.rs:5724`) 会被 Carry 搬来的行顶红，第二轮正推把它记在 Carry 名下。我的读法是它不是 Carry 独有（推的）：新写者带克隆头发布时，R 自己就带了克隆头的「这一代的行」，同一判在 R 上先红，三臂同要改。
- C7 ㊀ 在 `crates/` 里到不了（推的，第二轮正推 V1.5 第 1 条）：未登记的树种类码被当损坏，旧写者先死在这里：「what: "树的种类没登记",」(`crates/singlefs-core/src/recovery.rs:4124`)。三臂同受，先做 D15 compat。

### 2.3 「按统计量」臂（零轮，派生，不是装置的一次跑）

规则：有台阶行例外的统计量（今天只有 inode 号水位）不带这一代那一行，其余统计量照 Carry 带。派生法见 `reading_matrix_wide.py` 文件头。

- S1 `r_tree_dim` = 1 时与 NoCarry 逐字段相同（因为今天唯一带树维的统计量就是有台阶行的那一个，没有「其余统计量」可带）。N1 至 N8 在今天的取样点上全部适用于它。
- S2 `r_tree_dim` = 2 时行数比 Carry 少 `h_new`；节点数：第二个统计量码排在水位之后（码 13）时与 NoCarry 全等，排在水位之前（码 7）时比 NoCarry 多 0 或 1 个，Carry 比它多 0 到 4 个（副本规划器；「E7RESULT name=claim_b_multi_leaf r_tree_dim=2 second=7 lines=58 …」(`research/prompts/m3-close-c693-r3-sonnet-model/out/planner-claims.out:5`)）。key 在树里的位置会左右节点数，这一点 `r_tree_dim` = 2 的数不可外推。
- S3 第 6 位有两种记法（推的）：照第二次跑的机械谓词（N > 0 且这一版没写不认识的行 ⇒ 记 1），它在 `r_tree_dim` = 1 上记 1，与 NoCarry 相同；照条款含义（这半句描述的规则对「没有台阶行的统计量」仍有对象，不是空指称）记 0。两种记法的矩阵行见第 3 节。
- S4 条文：要多一条通则「有台阶行例外的统计量不搬」。「有台阶行例外」今天按名字写在 D5 已定项 4 第 12 项上：「第 12 项只为可写头维护，水位变了的那次发布留一行台阶行，随根 COW 一直留着」(`.claude/kb/decisions/05-快照-空间记账机制.md:138`)；通则要么点名第 12 项，要么给统计量登记表加一列（第二轮正推 V1.6 说要加列，是冻结组件；我读法是点名第 12 项就够，推的，两种写法的第 6 位后果没量）。
- S5 没有任何入库装置跑过这条臂；它的向量由 Carry 与 NoCarry 的格逐字段派生，节点数来自副本规划器。

## 3 「按统计量」臂：同样打中别的臂，以及它自己的限度（量过加推的，零轮）

臂集加上这一条（A4，派生臂）后，第 6 位在前的几行（同一文件 `research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out`）：

- 空真、㊀ 不计、按行：「u5=vac fst=skip snd=skip q2=counted seventh=w2 order=67 r1=NoCarry+KeepWithRoot+PerStatistic:1920|NoCarry+PerStatistic:1280 r2=PerStatistic:3200 in_every_judged_point=PerStatistic」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:134`)。这一读法下它是唯一每个被判点都在胜者里的臂；NoCarry 只赢 `r_tree_dim` = 1，Carry 只赢 `r_tree_dim` = 2。
- 空真、㊀ 不计、按叶数：同文件第 138 行，Carry 与它并列，Carry 处处在。
- 第二次跑的读法、按行：「u5=dang fst=skip snd=skip q2=counted seventh=w2 order=67 r1=Carry:3200 r2=PerStatistic:3200 in_every_judged_point=none」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:198`)。第二次跑的「唯一 Carry」在加这一条臂后不成立，与第二轮攻方的重判同数：「E7RESULT name=q1_form reading=as_registered arms=three_arms_plus_per_statistic group=legal_r_tree_dim_2 judged_points=3200 form=unique_other_arm consequence=flips winner_sets=PerStatistic:3200」(`research/prompts/m3-close-c693-r2-opus-model/out/e186-second-run-reread.out:6`)。
- ㊀ 计：同文件第 166 行，Carry 处处在。

判定：

1. 同样打中。它的赢面和 NoCarry 在 `r_tree_dim` = 1 的赢面是同一种东西——每次发布少写几行、且这几行不兑现成节点。第 7 位按叶数、或 ㊀ 自己计进去，两者同时消失（第 138、166 行对 NoCarry 的 `r_tree_dim` = 1 一栏同样是并列或 Carry 独胜）。所以用「少写的行不兑现成节点」「㊀ 自己该计」去驳它，也驳掉 NoCarry 的 `r_tree_dim` = 1；留着 NoCarry 的赢面就不能在这两条上单驳它。第二轮判决把 NoCarry 的 `r_tree_dim` = 1 当「量过的数」、把它当「派生零轮」，在这一点上两者是同一张格表上的同一类算术（它的向量由 Carry 与 NoCarry 的格逐字段派生，自证见第 1.1 节），区别只在它没有进过装置的臂集。
2. 它不是「按 `r_tree_dim` 分支的规则」。第二次跑登记对混合形态写的是：「依赖取样点·混合（`depends_on_point_mixed`） | 有的点含 `Carry`、有的点不含 | 不判翻面也不判站住，交用户：列出含与不含 `Carry` 的两组点数与翻转变量；「按翻转变量分支」的规则不在臂集里，标「臂集外，未量」，不当候选推」(`research/prompts/e186-preregistration.md:916`)，按翻转变量分支的规则被排在候选之外；这一条臂按统计量各判一次，写者在写时看得到统计量标签，不看 `r_tree_dim`，不在那句话罩的范围里（推的）。
3. 限度，不替它藏：今天的 `crates/` 里它与 NoCarry 的行为逐字段相同（2.3 的 S1），差别只在条文多一条通则；它赢 `r_tree_dim` = 2 靠的是那一档的造法（第 4 节）。第 6 位的记法有两种（2.3 的 S3）：机械谓词下它在第二次跑读法的 `r_tree_dim` = 1 上输给 Carry（第 198 行）；按条款含义记 0 时它每点都在：「u5=dang fst=skip snd=skip q2=counted seventh=w2 order=67 r1=PerStatisticMeaning:3200 r2=PerStatistic+PerStatisticMeaning:3200 in_every_judged_point=PerStatisticMeaning」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:454`)。后一种记法是我加的派生（推的，零轮），谁的记法对要主 agent 与用户判，我不替它定。
4. 它没被攻过：腿提的收严只在自己的模型上量过，算没被攻过（规则文件 `.claude/rules/three-way-inference.md` 的「判决由主 agent 做，不由投票做」一节）。交用户时写「零轮」。

## 4 `r_tree_dim` = 2 怎么写进交用户的话

事实（读原文）：登记把这一档写成造出来的：「`r_tree_dim = 2` 是假设取样点（第一版只有一个带树维的统计量，第二个按」(`research/prompts/e186-preregistration.md:32`)，接着写的造法是第二个统计量只有这一代的行、没有台阶行。今天只有一个带树维的统计量：「第 12 项只为可写头维护，水位变了的那次发布留一行台阶行，随根 COW 一直留着」(`.claude/kb/decisions/05-快照-空间记账机制.md:138`)。

量过（派生，零轮）：Carry 在 `r_tree_dim` = 2 赢，靠的是造法里「第二个统计量没有台阶行」，NoCarry 因此丢它的元组（N7）。若将来第二个带树维的统计量也有台阶行例外，NoCarry 在那一档不丢元组（第 3、4 位记 0），其余照 NoCarry：空真、㊀ 不计、按行读法下它每点都在、且独胜 `r_tree_dim` = 2：「u5=vac fst=skip snd=skip q2=counted seventh=w2 order=67 r1=NoCarry+KeepWithRoot+PerStatistic+NoCarryStepsBoth:1920|NoCarry+PerStatistic+NoCarryStepsBoth:1280 r2=NoCarryStepsBoth:3200 in_every_judged_point=NoCarryStepsBoth」(`research/prompts/m3-close-c693-r3-sonnet-model/out/reading-matrix-wide.out:262`)。按叶数与 ㊀ 计的读法下它不改 Carry 的处处在（同文件第 266、294 行）。

所以建议交用户的话分开写，不把两档并成一张「混合」的表：

「E186 的取样点分两档。`r_tree_dim` = 1 是今天的实情（只有 inode 号水位一个带树维的统计量），第 6、7 位的读法决定 Carry 还是 NoCarry（第 1.2 节的四类读法）。`r_tree_dim` = 2 是造出来的假设点，造法是『第二个统计量没有台阶行』；它只能说『将来再加带树维的统计量时，不搬的丢失量按树数长』的方向，不是与 `r_tree_dim` = 1 平权的第二个取样点；它给出的『Carry 赢』在第二个统计量也有台阶行时不成立。」

另外，第二轮判决把两档并成 `legal_pooled`、定形态为「依赖取样点·混合」交用户。并池本身不改任何读法下的胜者（第 1.2 节每格 r1 与 r2 分开列），但把真取样点与造出来的取样点按点数各占一半，会让「混合」看起来比实情更均衡。这是写法建议，不是对第二轮判决的翻案。

## 5 T3：D5 五句的改后条文（兼，量过）

范围：第二轮正推报告 V3.2 的五条改后条文对 `crates/` 自己的函数与 E71、E97 入库产物。不重做攻方该做的全称检查。

5.1 叶容量与树高的算术，在 `crates/` 的容量函数上复算（`accounting_key_width_leaf_capacity.rs`，条目宽 = key 宽 + 12，22 字节对应 `of_the_node_format(22, 34)`）：

- 「R3CAPACITY key_width=22 entry_width=34 leaf=477 internal=150」(`research/prompts/m3-close-c693-r3-sonnet-model/out/capacity.out:5`)，key 21 字节叶 491（同文件第 4 行），key 7 字节叶 855（第 1 行）。条文 ④ 的「491 条掉到 477 条」对。
- 条文 ① 的「79 到 141 块盘上 7 字节与 22 字节 key 的树高不同」：行数 3 + 6 × 盘数 + 一级台阶行 = 6d + 4；22 字节在 d = 79 时 478 > 477 起两层，7 字节要到 6d + 4 > 855 即 d = 142 才两层，所以 d = 79 到 141 共 63 个值，对。D8 总表邻句「80 块盘起才分裂」：「两块盘的池上除台阶行外常态 15 行，80 块盘起才分裂」(`.claude/kb/decisions/08-核心索引结构.md:34`)，不数台阶行 3 + 6 × 80 = 483 > 477，数一级台阶行 79 块盘起，V3.4 的建议对。
- 副本规划器：216 格里树高 1 当且仅当记账树总行数 ≤ 477（N3）。台阶行条数、盘数、不认识的头的行数都只通过总行数进树高。D5 已定项 2 射程自己的数：「代价：一行 34 字节；每次发布多写的单元，台阶行到第 462 行为止是 0」(`.claude/kb/decisions/05-快照-空间记账机制.md:55`)，462 = 477 − 15（两块盘常态 15 行）。①④⑤ 里写「树高」的句子要带的变量是「记账树总行数（盘数 × 6 + 3 + 台阶行条数）是否超过一片叶」，评审已采「台阶行条数」，这里给出完整的变量。

5.2 取整与产物不一致的三处（量过）。条文 ③ 写「22 字节臂 47%」「22 字节臂 376%」「507 904 字节占预算 2.4%」，E97 产物自己的 `new_percent` 是截断的，永远不是这三个数：

- `1024 × 64` 宽读 22 字节：「geometry=acct_e73_22B header=58 key_bytes=22 entry=30 leaf_fanout=542 trees=1024 devs=64 reading=wide … new_node_bytes=10092544 new_over=0 new_percent=46 new_permille=469」(`research/results/e97-entry-encoding-single-generation-2026-10-20.out:69`)：产物 46，条文 47。
- `8192 × 64` 宽读 22 字节：「geometry=acct_e73_22B header=58 key_bytes=22 entry=30 leaf_fanout=542 trees=8192 devs=64 reading=wide … new_node_bytes=80740352 new_over=1 new_percent=375 new_permille=3759」(`research/results/e97-entry-encoding-single-generation-2026-10-20.out:73`)：产物 375，条文 376。
- `8192 × 64` 窄读 22 字节：「geometry=acct_e73_22B header=58 key_bytes=22 entry=30 leaf_fanout=542 trees=8192 devs=64 reading=narrow … new_node_bytes=507904 new_over=0 new_percent=2 new_permille=23」(`research/results/e97-entry-encoding-single-generation-2026-10-20.out:72`)：产物千分之 23，条文 2.4%（507904 / 21474836 = 2.37%）。
- 19 字节臂的 42%、47%、337%、2.1% 都和产物的 `new_percent`、`new_permille` 对得上（同文件第 77、79、81、78 行）。建议：要么照产物的截断写 46 / 375 / 2.3，要么条文里明写「四舍五入」；条文里数字的取整口径不能和同一句引的产物行不一致。

5.3 ④ 里「少 2.8%」：(491 − 477) / 491 = 2.851%，四舍五入是 2.9%（本腿算术：`python3 -c "print((491-477)/491)"` 输出 `0.028513238289205704`）。

5.4 E71 的几处能在产物里直接对上：「E7RESULT name=over_budget_count arm=upper_bound widths=registered_widths cells=45 old_over=9 new_over=5 new_over_cells=narrow:8192:64,middle:8192:64,wide:1024:64,wide:8192:8,wide:8192:64」(`research/results/e71-accounting-keys-single-generation-2026-10-20.out:289`)，宽读臂 45 格新口径 3、旧口径 5 在第 293 行，窄读臂 0 格在第 291 行，今天 34 字节条目宽 `8192 × 64` 仍过在第 294 行。条文 ② 里的 14.53 / 28.27 / 128.47 / 283.52 是第二轮正推脚本里的比值，不是产物里的字面数，本腿没复算（见「没做什么」）。

## 6 本轮新冒出的形态（零轮，不再为它们开一轮）

- Z1 「两个带树维的统计量都有台阶行例外」时的 NoCarry（第 4 节）：向量由 NoCarry 的格派生（第 3、4 位记 0），不是装置的一次跑。
- Z2 「按统计量」臂的第 6 位按条款含义记 0（第 3 节第 3 条）：派生记法，不是登记里的谓词。
- Z3 「记账树超过一片叶才有节点差」的门槛，以及第二个统计量码排在水位之前或之后对节点数的影响（N3、S2）：副本规划器的数。
- Z4 第 7 位排在第 6 位之前时随根留着独胜 `r_tree_dim` = 2（第 1.3 节）：用户没定的位次，只报。
- Z5 恢复里「3 + 6 × 盘数」那一判不是 Carry 独有（C6）：读码推的。
- Z6 「按统计量」臂的条文只需点名第 12 项、不必给统计量登记表加列（S4）：推的。
- Z7 交用户的话把 `r_tree_dim` 两档分开写（第 4 节）：写法建议。

以上都没被攻过。

## 7 什么现象会推翻这些结论

- 「四维够用」（第 1.3 节）：入库装置新一次跑出来的格行，在 W1 / W2、㊁ 计与不计、逐 flags 层上给出与本腿 `out/reading-matrix-wide.out` 不同的胜者集合；或把 U1 到 U13 里别的条款按「悬空即要改」重判后，别的条款在三条臂下悬空不同（第二轮攻方 V1-丁 推了没有，本腿没重判）。
- 「U5 空真是 B1 的一致用法」（第 1.4 节第 1 至 3 条）：主 agent 或用户判定 383 的半句属于「要改一条已定条款」所指的对象（第 1.4 节第 5 条）——那时空真读法成了漏计，第二次跑的读法才是对的读法。
- 「台阶行长时省节点」的门槛（N3）：入库装置或真 checker 上，行数不超过 477 时三条臂的重写节点数不等；或行数超过 477 时 Carry 比 NoCarry 多出 `h_new × r_tree_dim` 个以上。
- 「按统计量」臂同样打中（第 3 节）：有人给出一条只落在它、不落在 NoCarry 的理由，例如统计量登记表必须加列且这一列在第 6 位记 1 而 NoCarry 不用——那时它在「空真、㊀ 不计、按行」下不再每点都在。
- `r_tree_dim` = 2 的承重假设（第 4 节）：将来的第二个带树维的统计量没有台阶行例外——造法成真，Carry 在那一档赢成立。

## 没做什么

- T2（F2 白名单）、T4（墓碑缩题与删头台阶行）不归本腿，没碰。没读禁读清单里的文件。
- 派生臂（按统计量、两个统计量都有台阶行的 NoCarry、按含义记 U5 的按统计量）都不是入库装置的一次跑；副本规划器的数也不是。要引进 kb，主 agent 把它们做成 E186 装置的新臂或附带量，或复跑本腿模型目录（命令在文件头），照 `.claude/rules/three-way-inference.md` 判决那一条。
- 没复算第二轮正推 V3.2 条文 ② 里的 14.53 / 28.27 / 128.47 / 283.52 四个比值，也没复算 524 288 个格点里的 17 440 与 92 663（第二轮正推脚本的数，攻方 `w3.out` 同数，本腿只核了叶容量、79 到 141 块盘、产物百分数）。条文 ④「多叶的池上最多多写同比例的叶」没量。
- 没造 checker 盘面，没跑 `crates/` 的测试，没跑崩溃点、变异、形式化这类重型；harness 档只跑了两个自加的测试目标，经 4G 内存包装，线程 31。
- 没重判 U1 到 U13 逐条（第二轮攻方 V1-丁 的推，本腿没动）；没核 `checks-owed.md` 与里程碑落点。
- 不替主 agent 撤销或维持判决，只交复核结果。

## 复跑结果与自证

模型目录的 `tools/rerun.sh` 在新草稿目录跑了两次，两次的输出与 `out/` 比：第一次 `out/` 还空，四行 `identical=0` 之后把输出放进 `out/`；第二次（经 4G 包装，用了 5 分 32 秒，机器上有别的编译在跑）：

```
name=rerun_compare file=reading-matrix-wide.out identical=1 lines=582
name=rerun_compare file=planner.out identical=1 lines=218
name=rerun_compare file=capacity.out identical=1 lines=6
name=rerun_compare file=planner-claims.out identical=1 lines=15
```

弄坏开关：`MATRIX_BREAK=swap_acct` 退 3（第 1.1 节）；`PLANNER_CLAIMS_BREAK=shift_carry` 退 3（本腿跑过）。

## cite-check 末行

```
✓ 核了 47 处引文、2 处裸行号，对上 47 处，没判 2 处（其中裸行号没有引文可比 2 处；逐处列在上面）
```

命令：`python3 scripts/agents/cite-check.py research/prompts/m3-close-c693-r3-sonnet-output.md --root . --background research/prompts/_m3-close-c693-r3-background.md`。

草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-c693-r3-sonnet` 里的仓副本与两次复跑的工作树（各约 1.4G，含编译目录）已删，原因：数都已进模型目录 `out/`，复跑命令能重建。
