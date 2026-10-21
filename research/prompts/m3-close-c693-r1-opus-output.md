# m3-close-c693-r1 攻方腿（Opus）报告

日期 2026-10-20。立场：假设「三份实验的读数推得出 ㊀ 翻面、㊁ 重开、D5 依据句与树高句要改、C344 与 D5 已定项 3 第 3 行可销账」是错的，造取样点与历史去打。只攻正文第三节 W1–W4；不判正推那一侧。

## 复跑命令与文件 sha256

```bash
# 仓根；三份脚本都是纯 Python 算术，只读 research/results 下三份入库产物，不编译、不读 crates/
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-c693-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen research/prompts/m3-close-c693-r1-snapshot --work <新的草稿目录>
# 末尾三行 name=rerun_compare ... identical=1 即逐字节复出 out/ 下三份
```

本腿跑过一次（草稿目录 `~/.local/share/singlefs/work/m3-close-c693-r1-opus/rerun1`），原样输出末四行：

```
name=rerun_compare file=e186-reanalysis.out identical=1
name=rerun_compare file=w3.out identical=1
name=rerun_compare file=w4.out identical=1
name=rerun_roots repo=/home/fy5090/code/singlefs frozen=/home/fy5090/code/singlefs/research/prompts/m3-close-c693-r1-snapshot model=/home/fy5090/code/singlefs/research/prompts/m3-close-c693-r1-opus-model work=/home/fy5090/.local/share/singlefs/work/m3-close-c693-r1-opus/rerun1
```

```
028d68710b4169c0eb401d890a65616640b8539967e2e0e581b2585f7c18fc17  research/prompts/m3-close-c693-r1-opus-model/e186_reanalysis.py
88204f30c67d6030baa143539d49a115a6f6bde41fd680b206ed929921637f07  research/prompts/m3-close-c693-r1-opus-model/w3_geometry.py
927e1b513976de8840ff5ad991dcd8590bd40437dafac169b790d54695bf5696  research/prompts/m3-close-c693-r1-opus-model/w4_frontend_fold.py
cb14681c4a578d7c6c97151d7c53694d51eaeeb7fa0964254d240854d5ffea54  research/prompts/m3-close-c693-r1-opus-model/tools/rerun.sh
2de6fd9d6c66330d8fbb2f91b6178110f45cc2c25658b49efcc90541d3ace537  research/prompts/m3-close-c693-r1-opus-model/out/e186-reanalysis.out
3e6037726b1d4e575335d4debe9fc07bc2e05f53a183dd474ac08758ead4c615  research/prompts/m3-close-c693-r1-opus-model/out/w3.out
0edd9bc8008926d27235286d186afd065d7ce68e8d15acc7e591207935aab393  research/prompts/m3-close-c693-r1-opus-model/out/w4.out
```

读的三份入库产物（未进 git，开工时 sha256）：

```
f1dfd78bc92b7ed919af4484e334b379b615f3914b71a5cf36f513843439a86a  research/results/e186-unknown-tree-accounting-rederive.out.gz
29cafdd37626b06fed52b4c3b8c66ce302be74a7e516cb0227e92e5e51610e58  research/results/e97-entry-encoding-single-generation-2026-10-20.out
885c238a93c4519e58a4dd1f1251ebd0669f25bdbca7629ee21acfe94caa9c07  research/results/e71-accounting-keys-single-generation-2026-10-20.out
```

三份脚本各带会红的自证：`e186_reanalysis.py` 用基线读法重算产物里 112 行胜者判决行、逐字相等（`selftest verdict_lines_checked=112 mismatches=0`），弄坏开关 `E186_REANALYSIS_BREAK=drop_reject_plus_one` 下 `mismatches=8`、退 3；`w3_geometry.py` 四档叶扇出对上 E97 产物、crates 几何对上 477/150，弄坏开关 `W3_GEOMETRY_BREAK=entry_without_seq` 下 `leaf=540 … equal=0`、退 3；`w4_frontend_fold.py` 两条路（整棵重建、前端点删 + 到达序折叠）逐版逐行相同，弄坏开关 `W4_FOLD_BREAK=arrival_keeps_tomb_target` 下 `differing_versions=20`、退 3。内存包装上限 2G，排队 4 秒起跑，没有撞顶。

## 各格判定一览

「量过」= 贴得出原样输出（入库产物或我的模型 out/）；「推的」= 读码或读条款推的，没跑。

| 格 | 攻的那一步 | 判定 | 量过 / 推的 |
|---|---|---|---|
| W1-甲 | 反向接受条款字面推得出「㊀ 翻面」 | **打中**：判定是「依赖取样点」，条款两支（含别的臂 ⇒ 翻面；并列 ⇒ 交用户）在 1920 点上重叠、对「依赖取样点」没写；h_new = 0 的 1600 点（没有作用对象）在任何读法下都让触发句成立 | 量过（入库产物第 186059、186067 行） |
| W1-乙 | 「Carry 在 r_tree_dim = 1 一次都不是胜者」是读数 | **打中**：它只在读法 B1 下成立；B2（把 ㊀ 本身计进「要改几条已定条款」）下同一组点 Carry 3200 点独胜、1600 点三条并列 | 量过（入库产物第 186067 行） |
| W1-丙 | B1 下第 6 位数全了非现任臂要改的条款 | **打中**：U5（D8 已定项 12 定案「含已定项 8 ㊀ 那类旧写者搬运的记账行」）对 NoCarry、随根留着空真记 0，而正文自己把 D8:383 列为翻面要动的条款；按要改计 1，B1 下 r_tree_dim = 1 合法点也是 Carry 3200 独胜 | 量过（我的重算，只读入库产物） |
| W1-丁 | 第 7 位「每代写行」分得出 NoCarry 与 Carry | **打中**：11520 点里每代写行最大 47，记账树叶容量 477，三条臂每次发布都写 1 个记账节点；第 7 位按节点数读，r_tree_dim = 1 合法点 Carry 处处在胜者里 | 量过（我的重算） |
| W1-戊 | 按 r_tree_dim 分支 | **限定**：分支变量是写者版本里的常量（不认识的统计量标签照 C71 拒可写挂载），不违反分支约束 1；但它真正的变量是「这个统计量有没有台阶行」，写成按统计量的规则就不必分支 | 推的 |
| W2-甲 | Q2「换档」推得出 ㊁ 重开 | **打中**（判别子观测不到）：换档只在 w_gens = 0 的点上，w_gens 是写者做 flags 处置那一刻看不到的未来窗口；每格在四档 w_gens 上取最坏，四种读法下 2400 组全是 RejectEntry | 量过（我的重算） |
| W2-乙 | 「计」那一位是「要改的条款」 | **打中**（归错了判据）：「EIO 但位置还在」已写在 ㊁ 射程里；它欠的是实现，不是条款 | 推的（读条款） |
| W2-丙 | crates 整表拒（recovery.rs 第 2804 行）=「拒整个池」 | **打中**：整表拒在可写挂载里走 F2 退回：系统配置没见证最新一版时退到上一条根、不重放，最新一次发布静默丢掉；㊁ 的对象级 EIO 没有这一支 | 推的（读码，没造盘） |
| W3-甲 | E71 第一次重跑让 D5:145 那句「失去支撑」 | **打中**：那句写的是宽读臂，而 E71 宽读臂在 1024 × 64 旧口径下 22 / 30 / 34 字节也不过界（只 50 字节过）；它在 E71 这一侧从来没有支撑过，FC1 触发不改变什么 | 量过（入库产物第 91 行） |
| W3-乙 | 「树高恒为 2」改成「窄读 1、宽读 2，四档相同」 | **打中**：E97 一代口径的树高只在 t = 64、d = 8 一个点量；crates 今天的行集 3 + 6d + 1 在 d = 79..141 上 7 字节与 22 字节 key 树高不同 | 量过（我的算术，自证对上产物扇出） |
| W3-丙 | 「多带一维的段宽不构成代价 / 代价曲线是平的」只要改字 | **打中**：一代口径整批重写时代价是叶数，随条目宽走；E97 自己的新口径产物里 22 字节对 7 字节的节点字节在宽读格是 2.0 倍 | 量过（入库产物第 69、93 行） |
| W3-丁 | 已定项 4「哪几项按树分」的取舍会塌 | **没打穿**：取舍立在消费者普查上；今天十二项表全按树分在 1024 × 64 也不过界（13 762 560），8192 × 64 过界 | 量过一格（我的算术）+ 推的 |
| W4-Q1 | 「crates 从不点删 ⇒ 第 3 行与 C344 没有对象」 | **打中**：crates 在 btree 层每次发布删掉上一版全部非台阶 key；开前端后 D8 已定项 12「写一律进前端」逼出墓碑；朴素折叠在我的模型上 20 次发布行数 16 → 35，I-3.13 语法判、条数判、I-9.30 ① 三道 0 红 | 量过（我的模型）+ 读码 |
| W4-Q2 | 删头时台阶行怎么离开 | **打中**：没有条款；按 crates TODO 字面「按树段分开取」而不看头还在不在，删掉的头 3 行台阶行永留，I-3.13 语法判与条数判 0 红 | 量过（我的模型）+ 读码 |

## W1　D8 已定项 8 ㊀（记账维）

被攻的那一步：正文第三节 W1 把 E186 Q1 读成「㊀ 要不要改、改成哪一条或按 r_tree_dim 分支」，要动的条款「改了要动的条款（D8:261、D8:383、D5 已定项 2 与 I-3.13 的例外）」（`research/prompts/_m3-close-c693-r1-body.md:28`）。㊀ 原文：「写者必须把这类行原样带到新代」（`.claude/kb/decisions/08-核心索引结构.md:261`）。

### W1-甲　反向接受条款的字面推不出「㊀ 翻面」

E186 跑前登记的反向接受条款：「Q1 胜者含别的臂 ⇒ ㊀ 翻面，要改的条款按第六节那一格的明细列；并列 ⇒ 交用户；Q2 换档 ⇒ ㊁ 重开」（`research/prompts/e186-preregistration.md:554`）。同一登记的判定规则：「判定：合法点上处处同一个集合 ⇒ 记这个集合…并列出翻转点」（`research/prompts/e186-preregistration.md:414`），中间省掉的是「随点变 ⇒ 记依赖取样点」。入库产物第 186059 行（判的读法）原样：

```
E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=Carry+NoCarry+KeepWithRoot:1600|NoCarry:1280|NoCarry+KeepWithRoot:1920
```

三处字面的出入：

1. 判定是 `depends_on_point`。条款写了「含别的臂」与「并列」两支，没写「依赖取样点」时怎么判；把 1280 点的 `NoCarry` 当成全体的判决，是越过条款的推论。
2. 1920 点的胜者是 `NoCarry+KeepWithRoot`：它同时是「含别的臂」（⇒ 翻面）与「并列」（⇒ 交用户），条款没写哪一支先。
3. 1600 点是 `h_new = 0`（没有不认识的头，三条记账臂按登记的阴性对照乙必须全等）。触发句「Q1 合法点上的记账胜者不是 {Carry}」在这 1600 点上任何读法都成立——下面 B2 那一行同样有 `Carry+NoCarry+KeepWithRoot:1600`。登记第十节「打中不分辨臂」只排十五格全等的点，这 1600 点 flags 维还分得开，没被排掉。条款在没有作用对象的点上照样触发。

四句：① 分不分辨臂——1280 点分辨（NoCarry 对 Carry 第 7 位）；1600 点不分辨（定义上全等）。② 系统看得到吗——不适用（这一格判的是判读条款，不是被判系统）。③ 满足的分句——触发句的「不是 {Carry}」，后果句里没有对应「依赖取样点」的那一支。④ 跑前给的改法「㊀ 翻面」在打中的那几格（r_tree_dim = 1、lpc = 0、h_new > 0）上消掉了第 7 位的差，但在 r_tree_dim = 2 的 3200 点上让 `NoCarry` 第 3 位丢行（产物第 186060 行 r_tree_dim = 2 胜者 `Carry:3200`）——这个改法在没打中的那一半上造出新的错。

### W1-乙　「Carry 在 r_tree_dim = 1 一次都不是胜者」只在读法 B1 下成立

读法 B1 的理由：「理由是每条非现任臂都要改它，计进去等于先给现任臂一票」（`research/prompts/e186-preregistration.md:29`）。可第 6 位的名字是「要改几条已定条款」，㊀ 本身就是一条已定条款；B1 把每条非现任臂都要付的那一条从账上拿掉，第 6 位就不再数它自称数的东西。入库产物第 186067 行（B2，附带量）原样：

```
E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b2_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=2 judgement=depends_on_point winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600
```

B2 下 r_tree_dim = 1 合法点：h_new > 0 的 3200 点 Carry 独胜，剩下 1600 点就是 W1-甲 第 3 条那批没有对象的点。也就是说，「㊀ 翻面」这一推论的全部分量押在 B1/B2 这个读法选择上，不在实验量出的数上。读法是主 agent 跑前定的取舍，不是读数；翻面要拿出来，得先说服用户 B1 是对的，而不是从产物推出来。

四句：① 分辨臂（Carry 对另两条）。② 不适用。③ 触发句在 B2 下只剩 h_new = 0 的 1600 点成立（见 W1-甲 第 3 条）。④ 改法「㊀ 翻面」在 B2 下没有一格要它。

### W1-丙　即使留在 B1，U5 的空真规则漏数了正文自己列的 D8:383

U5 的谓词：「写者这次为不认识的树写进去的行，代段 = 这次的 txg；没写就空真」（`research/prompts/e186-preregistration.md:433`）。被它代表的条款是 D8 已定项 12 定案，原文含「含已定项 8 ㊀ 那类旧写者搬运的记账行」（`.claude/kb/decisions/08-核心索引结构.md:383`）。NoCarry、随根留着都不写这类行，谓词空真，记 0；可正文 W1 把 D8:383 列为 ㊀ 一翻面就要动的条款（正文第三节 W1 那句，上面已引）。同一点 8824 的 `RejectEntry_NoCarry` 格（产物第 141193 行）`clause_units_b1=0`、`clauses_false=U12:1/10/1`——U5 没算。

我把这一条按「要改」计（NoCarry、随根留着在记账臂起作用且 h_new > 0 的格第 6 位各 +1，其余照产物），读法仍是 B1、W2、Q2 计。原样输出（`out/e186-reanalysis.out`）：

```
E7RESULT name=q1_overall_accounting_winners variant=u5_dangling reading=b1_w2_q2_counted group=legal_r_tree_dim_1 winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600
E7RESULT name=q1_overall_accounting_winners variant=u5_dangling reading=b1_w2_q2_counted group=legal_r_tree_dim_2 winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600
```

四句：① 分辨臂。② 不适用。③ 打中的是第 6 位的计数口径（谓词空真记 0），不是判据门槛。④ 改法「㊀ 翻面」在这一读法下没有一格要它。推翻条件：主 agent 判 D8:383 那半句在 NoCarry 下可以原样留着（空指称不算要改），这一条就不成立——但那样正文 W1 的「要动的条款」清单也要删掉 D8:383。

### W1-丁　第 7 位的差在设备上是 0 个节点

实验页自己写了第 7 位「数的是行，不是写的节点或单元」。crates 的记账树叶容量是 477 条：「格式算出来的节点容量：记账树叶 477、内部 150」（`crates/singlefs-core/src/transaction.rs:2561`）。我数了全部 172800 格的 `rows_written_w2`：

```
E7RESULT name=max_rows_written_w2 value=47 leaf_capacity=477 fits_one_leaf=1
```

每一格每一版都是一片叶，三条记账臂每次发布写的记账节点都是 1 个。把第 7 位换成节点数（ceil(行数 / 477)，没有新版本的格记 0），其余不动：

```
E7RESULT name=q1_overall_accounting_winners variant=nodes reading=b1_w2_q2_counted group=legal_r_tree_dim_1 winner_sets=Carry+NoCarry:1280|Carry+NoCarry+KeepWithRoot:3520
E7RESULT name=q1_overall_accounting_winners variant=nodes reading=b1_w2_q2_counted group=legal_r_tree_dim_2 winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600
```

r_tree_dim = 1 合法点上 Carry 处处在胜者里（lpc = 0 的 1280 点与 NoCarry 并列、随根留着输在第 6 位）。四句：① 1280 点上 Carry 与 NoCarry 不再分开——第 7 位按行数分开它们，按节点不分。② 不适用。③ 第 7 位是用户定的「每代写行」，按字面数行没错；打中的是「省 h_new 行 ⇒ 值得改 ㊀」这一步，它在设备写上兑现为 0。④ 改法「㊀ 翻面」买到的是 34 × h_new 字节、0 个节点。

### W1-戊　「按 r_tree_dim 分支」的分支变量（推的）

正文要求按 `.claude/rules/fs-design.md` 判分支。r_tree_dim 数的是不认识的树带的、写者认识标签的带树维统计量：㊀ 的射程只到「树 ID 不认识、统计量标签认识」那一半，标签不认识的一半按 C71 拒可写挂载（D8 已定项 8 射程同一段）。所以 r_tree_dim 不是池里会变的量，是写者版本里写死的常量——分支约束 1「只在操作期间不会变的输入上分支」过得去；可「每条分支必须能被测试强制进入」在单个版本里做不到，另一支在那个版本里没有代码。更准的写法不是分支，是按统计量的一条规则：有台阶行例外的统计量（今天只有 inode 号水位）不带这一代那一行，其余统计量照 Carry 带。这条规则在 r_tree_dim = 1 上等于 NoCarry、在 r_tree_dim = 2 上第二个统计量不丢行。它没进 E186 的臂集，我没量，**推的、被攻过零轮**；在第 7 位按节点读（W1-丁）时它与 Carry 一样多写 0 个节点，买不到东西。

### W1 小结

E186 的数没有错，错在「读数 ⇒ ㊀ 翻面」这一步：它押在三件跑前选择上（读法 B1、U5 的空真规则、第 7 位按行数），每一件换成正文或实验页自己承认的另一种读法，Carry 都回到 r_tree_dim = 1 合法点的胜者里；条款字面对「依赖取样点」也没给后果。能从产物直接读出、不依赖这三件选择的只有一句：r_tree_dim = 2（第二个带树维、没有台阶行的统计量）时 NoCarry 丢行、Carry 胜。推翻条件：主 agent 判 B1、U5 空真、按行数这三件各自都站得住，且给条款补上「依赖取样点 ⇒ 按 r_tree_dim = 1 那一半判」的后果句（这一句是新写的，等于跑后改判据，按 evidence-discipline 是新一轮）。

## W2　D8 已定项 8 ㊁（flags 维）

被攻的那一步：正文 W2「㊁ 要不要改」（`research/prompts/_m3-close-c693-r1-body.md:31`），同一句接着问 recovery.rs 第 2804 行整表拒与 ㊁「该条目 EIO」的出入谁改。㊁ 原文：「未知位一律拒收——该条目 EIO」（`.claude/kb/decisions/08-核心索引结构.md:262`）。

### W2-甲　Q2 换档是判别子观测不到

入库产物第 186275 行（判的读法 B1、计）：

```
E7RESULT name=verdict check=q2_overall_winner_flags_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=RejectEntry:3600|Skip:600|Skip+PerBit:600
```

Skip 独胜的 1200 点全是 w_gens = 0。w_gens 的意思在登记里：Skip「隐形窗口里出生的块误释放」（`research/prompts/e186-preregistration.md:356`），误释放 = 64 × w_gens。这是挂载之后、树隐形期间会不会有快照销毁并错邻居——写者在做 flags 处置那一刻（解析树表条目时）看不到它，只能对所有 w_gens 用同一条处置。我把每一格在四档 w_gens（0、1、3、8，其余参数相同的一组）上取字典序最坏的向量，再取胜者：

```
E7RESULT name=q2_flags_winners_w_gens_unobserved reading=b1_w2_q2_counted groups=2400 winner_sets=RejectEntry:2400
E7RESULT name=q2_flags_winners_w_gens_unobserved reading=b1_w2_q2_not_counted groups=2400 winner_sets=RejectEntry:2400
E7RESULT name=q2_flags_winners_w_gens_unobserved reading=b2_w2_q2_counted groups=2400 winner_sets=RejectEntry:2400
E7RESULT name=q2_flags_winners_w_gens_unobserved reading=b2_w2_q2_not_counted groups=2400 winner_sets=RejectEntry:2400
```

另：B2 + 不计时产物自己就是 RejectEntry 处处独胜（第 186287 行 `judgement=same_set_everywhere winner_sets=RejectEntry:4800`）。

四句：① 分辨臂（RejectEntry 对 Skip，第 6 位）。② **系统看不到**：分开它们的条件 w_gens = 0 在决定那一刻不可观测——evidence-discipline「判别子观测不到」那一形。③ 满足的分句是反向接受条款「Q2 两种读法下 flags 胜者在某点不同」，字面成立；它的后果「㊁ 重开」以「在某点」为单位，没问那一点写者认不认得出。④ 改法「㊁ 重开」能取的另一条处置只能是 Skip（或按位 PerBit），它在 w_gens > 0 的 3600 点上第 1 位误释放 64–512 块。

### W2-乙　「计」那一位数的不是「要改的已定条款」（推的）

Q2「计」给 RejectEntry 加的那 1，理由是「要一条「EIO 但位置还在」的语义」。这条语义已经在 ㊁ 的射程里写着：「条目 EIO 但位置还在」（`.claude/kb/decisions/08-核心索引结构.md:282`），连同它推出的「不做跨它的 deadlist 合并」。缺的是 crates 的实现（下面 W2-丙），不是条款。把实现欠账记进「要改几条已定条款」，就是登记第四节自己说的「第 6 位的值本身不算实验证据」（`research/prompts/e186-preregistration.md:335`）的那一类定义性常数，打中的格归错了判据：它该进实现欠账，不该进第 6 位。

### W2-丙　整表拒在 crates 里不是「拒整个池」，是退回上一版（推的，读码）

具体历史（两块盘的池，写者 v2 认识某个新的 flags 位、写者 v1 是今天的 crates）：

1. v2 在发布 P（最新一次）给某条树表条目置了一个 v1 不认识的 flags 位（例如码 14 快照条目上位 0 以外的位），发布成功、fsync 返回；系统配置的见证计数还停在 P 之前（只要 P 那次没有轮换系统配置槽）。
2. v1 可写挂载，选到根 P。解树表条目：flags 不是 0、也不是码 14 上的位 0 ⇒「_flags_this_reader_refuses => return None,」（`crates/singlefs-core/src/records.rs:753`）。
3. 整张树表读不下去：「TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;」（`crates/singlefs-core/src/recovery.rs:2804`），映射成「TreeTableEntriesRefusal::EntryMalformed => RecoveryFailure::UnitMalformed {」（`crates/singlefs-core/src/recovery.rs:2687`）——与单元损坏同一个失败。
4. 可写挂载对所选那一版建不出来的处置：「所选那一版建不出来（走读失败）、系统配置没见证它：退到 F2 交回的那一版建上一版」（`crates/singlefs-core/src/mount.rs:7994`），「退过去不重放：所选根与所选那一版都换成它」（`crates/singlefs-core/src/mount.rs:7995`）；退到哪一条：「系统配置没见证过所选那一版时，退到 [见证到的那一版, 所选根) 里读得出、不被所选根的实例表抛弃的最新一条根」（`crates/singlefs-core/src/recovery.rs:6868`）。
5. 根 P−1 的树表还没有那个位，建得出来；v1 在 P−1 上接着发布。v2 在 P 写下、fsync 确认过的东西不见了，挂载没有报错。

㊁ 的「该条目 EIO（对象级，不拒整个池）」走不到第 4 步：那一版照样建得出、只有那一棵树 EIO。所以「谁改」的答案不对称：条款侧照 crates 改成整表拒（等于 E112 的 Refuse），在 crates 里实际得到的是静默退一版，不是拒；只有改 crates（把「不认识的 flags 位」与「单元坏了」分成两种失败，前者不进 F2 退回）才同时满足 ㊁ 与 F2 的本意。没核的前提：P 那次发布是否必然轮换或重写系统配置槽（若必然，第 1 步的「没见证」不成立，这条历史就只在 P 之后崩溃、系统配置槽没写成时可达）；我没造盘跑，**推的**。另，㊁ 的「对象级」对 day-1 那 12 棵树（分配记录树、记账树等）兑现不了：记账树 EIO 时准入不等式读不到值，按 D5 已定项 8 射程「空 = 从未维护」只能拒可写挂载——㊁ 的射程句要补一个「核心树的条目 EIO ⇒ 拒可写挂载」的限定（推的）。

四句：① 分辨 ㊁ 的两种落法（整表拒、对象级 EIO）。② 写者看得到：位是条目里的字段。③ 打中的是正文的「谁改」这一问，不是 E186 的判据。④ 「条款照实现改」这个改法在这条历史上照样丢 P。

## W3　D5 已定项 4 的依据与「树高恒为 2」

被攻的那一步：正文 W3 把两份重跑读成「D5 已定项 4 依据两句失去支撑」，并把「树高恒为 2」改成「窄读 1、宽读 2」。

### W3-甲　D5:145 那句在 E71 这一侧从来没有支撑过

D5 已定项 4 依据：「树数 × 设备数 到 1024 × 64 时宽读那条臂过 T_dirty 的 1%」（`.claude/kb/decisions/05-快照-空间记账机制.md:145`）。E71 第一次重跑产物第 91 行（宽读臂、30 字节、1024 × 64）原样：

```
E7RESULT name=publish_budget width=middle width_role=registered_width key_bytes=22 value_bytes=8 sequence_bytes=0 entry_bytes=30 arm=per_statistic_wide arm_role=incidental_wide_reading trees=1024 devices=64 entries_per_generation=267264 old_bytes=16035840 old_over=0 new_bytes=8017920 new_over=0 new_permille_of_budget=373 budget=21474836
```

四档宽度全看（`out/w3.out`）：

```
E7RESULT name=e71_wide_arm_1024x64 width=narrow entry_bytes=22 old_bytes=11759616 old_over=0 new_bytes=5879808 new_over=0
E7RESULT name=e71_wide_arm_1024x64 width=middle entry_bytes=30 old_bytes=16035840 old_over=0 new_bytes=8017920 new_over=0
E7RESULT name=e71_wide_arm_1024x64 width=wide entry_bytes=50 old_bytes=26726400 old_over=1 new_bytes=13363200 new_over=0
E7RESULT name=e71_wide_arm_1024x64 width=current_implementation entry_bytes=34 old_bytes=18173952 old_over=0 new_bytes=9086976 new_over=0
```

旧口径（一写一删）下宽读臂在 1024 × 64 只有 50 字节那档过界；22、30、34 字节都不过。E71 的 FC1 是对上界臂判的，「触发」不说明 D5:145 那句失去了 E71 的支撑——它在 E71 这里本来就没有。这句的数真正来自 E97 判据 10 的旧节点口径（E97 实验页「出处出入」一段已经写了）。所以「依据要不要改」的问题不该写成「重跑让它塌了」，而是「它一直挂错了名，现在连 E97 那一格也翻了」。四句：① 不分辨臂（不判臂）。② 不适用。③ 打中的是 FC1 的归属：FC1 字面问上界臂，D5:145 字面说宽读臂。④ 正文能给的改法「依据句删掉 E71 名下那半句」照样成立，这一条只改它的理由。

### W3-乙　「四档 key 宽的树高相同」在取样点之间与 crates 行集上不成立

D5 已定项 1 依据：「四档 key 宽的树高恒为 2，多带一维的段宽不构成代价」（`.claude/kb/decisions/05-快照-空间记账机制.md:42`）；已定项 5：「四档 key（7 / 14 / 19 / 22 字节）的树高恒为 2，代价曲线是平的」（`.claude/kb/decisions/05-快照-空间记账机制.md:169`）。E97 第五次重跑的树高 48 行（`name=acct_geom_new_caliber`）每行都是 `entries_narrow=172 … entries_wide=2944`，即只在 t = 64、d = 8 一个点上量（命令 `grep 'name=acct_geom_new_caliber' research/results/e97-entry-encoding-single-generation-2026-10-20.out | grep -o 'trees=[0-9]* devs=[0-9]*' | sort | uniq -c` 输出空：那几行没有 t、d 字段，条目数只有这两个值）。一代口径把条目数压低了 25 倍，树正落在 1 层与 2 层的交界附近，key 宽能把它推过界。我按两套几何算（自证对上 E97 产物的四档叶扇出 1087 / 740 / 603 / 542 与 crates 的 477 / 150），原样：

```
E7RESULT name=today_rows_height_differs_across_keys geometry=crates rows=3+6d+1 devices_1_to_4096_differing=63 ranges=79..141
E7RESULT name=today_rows_height_differs_across_keys geometry=e97_header58 rows=3+6d+1 devices_1_to_4096_differing=91 ranges=90..180
E7RESULT name=today_rows_point geometry=crates devices=79 rows=478 k7:h1/nodes1 k14:h1/nodes1 k19:h1/nodes1 k22:h2/nodes3
E7RESULT name=e97_narrow_height geometry=e97_header58 trees=256 devices=8 entries=556 k7:h1 k14:h1 k19:h1 k22:h2
E7RESULT name=e97_narrow_height geometry=e97_header58 trees=128 devices=64 entries=580 k7:h1 k14:h1 k19:h1 k22:h2
```

crates 今天的行集（池级 3 + 每盘 6 + 一级台阶）在 79–141 块盘上，7 字节 key 一层、22 字节 key 两层、写 3 个节点对 1 个；E97 那张十一项表的窄读在 t = 256、d = 8 这类网格之间的点上同样分开。所以把「恒为 2」改成「窄读 1、宽读 2，四档相同」会把一个只在 t = 64、d = 8 量过的点写成普遍句。四句：① 不判臂。② 不适用。③ 打中的是依据句的射程（「恒」）。④ 改法「换成窄读 1、宽读 2」在 d = 79..141 那几格上仍错。附带：D8 已定项 1 射程「两块盘的池上除台阶行外常态 15 行，80 块盘起才分裂」（`.claude/kb/decisions/08-核心索引结构.md:34`）——发布过一次之后至少有一级台阶行，79 块盘就是 478 行、分裂（上面 `devices=79` 那一行）；「80」只在不数台阶行时对。

### W3-丙　一代口径下段宽有代价：E97 自己的产物

K 代口径下一次发布只改每个统计量的一个 key，代价看树高（一条路径）；一代口径下每次发布把这一代的行全写一遍，crates 的规划器把上一版全部非台阶 key 删掉、这一版的插进来（W4-Q1 引的 `crates/singlefs-core/src/code_two_tree.rs:655`），含它们的叶全部重写，代价是叶数，随条目宽走。E97 第五次重跑自己报的新口径节点字节按 key 臂排开（`out/w3.out` 从产物 `cell_side_by_side` 行取）：

```
E7RESULT name=e97_new_node_bytes_by_key trees=1024 devs=64 reading=wide k7=5046272 k14=7405568 k19=9076736 k22=10092544 ratio_22_over_7=2.000
E7RESULT name=e97_new_node_bytes_by_key trees=1024 devs=64 reading=narrow k7=49152 k14=65536 k19=65536 k22=81920 ratio_22_over_7=1.667
E7RESULT name=e97_new_node_bytes_by_key trees=64 devs=8 reading=narrow k7=16384 k14=16384 k19=16384 k22=16384 ratio_22_over_7=1.000
```

（宽读 1024 × 64 那两格的原行是产物第 69 行 `geometry=acct_e73_22B … new_node_bytes=10092544` 与第 93 行 `geometry=acct_windowgen_7B … new_node_bytes=5046272`。）一页叶装得下的规模上（今天两块盘 16 行）段宽确实是 0 代价；超过一页叶，22 字节对 7 字节就是 1.7–2.0 倍的写。已定项 1「多带一维的段宽不构成代价」与已定项 5「代价曲线是平的」在一代口径下都要改，不只是「恒为 2」几个字；已定项 5 第一行「取 2 是留扩展余量（已定项 4 自陈清单不封闭、D12（目标介质） 不封顶介质套数），而四档 key 宽的树高恒为 2」（`.claude/kb/decisions/05-快照-空间记账机制.md:159`）的理由同样要换。四句：① 不判臂。② 不适用。③ 打中的是「代价 = 树高」这个代价口径，它是 K 代口径的产物。④ 改法「只换字」在宽读格上低估 2 倍。

### W3-丁　已定项 4「哪几项按树分」的取舍没塌（没打穿）

两份实验用的是 2026-09 的九项 / 十一项表，按树分的是第 8、9 项；今天这两项已撤回，撤回依据是两腿独立普查零命中（D5 已定项 4 统计量清单第 8 行），不是代价。今天唯一带树维的第 12 项带树维是功能要求（每个可写头一个水位）。依据句 D5:145、D5:146 是给「窄读 vs 宽读」配的代价侧证据，今天没人提宽读，它们塌不塌都碰不到定案。我按今天的表补算了一格（每树 6d + 4 行，4 是不带设备维的项按 v1 登记口径数，比 crates 写的池级 3 行多算一行，偏保守；34 字节、crates 几何）：

```
E7RESULT name=today_table_single_generation reading=all_items_by_tree heads_or_trees=1024 devices=64 entries=397312 height=3 nodes_rewritten=840 node_bytes=13762560 over_budget=0
E7RESULT name=today_table_single_generation reading=all_items_by_tree heads_or_trees=8192 devices=64 entries=3178496 height=3 nodes_rewritten=6710 node_bytes=109936640 over_budget=1
```

「全部按树分」在 1024 × 64 不过界、在 8192 × 64 过界：要给依据句换数，这是今天的表能给的那一格；它只当量级，没进任何入库装置。

## W4　① 块：点删与墓碑那一族

被攻的那一步：正文 W4「答案决定 D5 已定项 3 第 3 行与 C344 保留、缩题还是销账」（`research/prompts/_m3-close-c693-r1-body.md:37`），摸底报告的前提是「写路径每次发布写当前代、从不点删」（摸底报告 A1，依据 D5:60「写路径每次发布写当前代的 15 行、从不点删」（`.claude/kb/decisions/05-快照-空间记账机制.md:60`））。被判的两处：D5 已定项 3 第 3 行「点删的墓碑不参加取最大——读一个 key 先看最新一条是不是墓碑」（`.claude/kb/decisions/05-快照-空间记账机制.md:75`）；C344 自己写的射程「它在 write buffer 前端或节点缓冲让同一个 key 多条并存时才发生」（`.claude/kb/checks-owed.md:252`）。

### W4-Q1　开前端之后整批换代：在 crates 今天的实现上答

**「从不点删」只在出行那一层成立。** btree 那一层，crates 每次发布都删 key：规划器先删上一版有、这一版没有的 key——「for deleted in previous_keys.difference(keys_after_this_publish) {」（`crates/singlefs-core/src/code_two_tree.rs:655`）、「delete_at_the_root(&mut root, deleted)?;」（`crates/singlefs-core/src/code_two_tree.rs:656`）。记账行的代段每次发布都换，上一版全部非台阶 key（两块盘 15 个）每次都被删掉。今天没有前端，这些删直接改叶，所以没有墓碑。

**开前端之后，两种答案都不是「没有对象」。** D8 已定项 12 定案：「这三棵树上的写一律进前端」（`.claude/kb/decisions/08-核心索引结构.md:383`），检查句：「一个 checkpoint 里前端树叶上的每一处逻辑改动（key → 值）都对得上一条这个 checkpoint 进过叶的前端条目」（`.claude/kb/decisions/08-核心索引结构.md:385`）。

- 答「写新一代 + 点删上一代」：删上一版 key 是对记账树的写，按「一律进前端」就是前端里的墓碑；同一个 key 在叶上有值、在前端有墓碑，正是 C344 说的「同一个 key 多条并存」。对象只是从「K 代之外那一代」换成「上一代」。
- 答「重建整棵」：要么旧 key 的消失也经前端（又回到墓碑），要么不经前端，那么已定项 12 检查句的「每一处逻辑改动」要么把「key 消失」算进去（整批换代每次都对不上，要给它开第四个例外），要么不算（漏掉的旧 key 无人判）。这一半要改的是已定项 12 的检查句，不是销账。

**墓碑朴素折叠在今天的检查下是静默的**（我的模型，`w4_frontend_fold.py`，行形照 crates 今天：池级 3 行 + 每盘 6 行，两块盘；台阶照抄、I-3.13、条数判、I-9.30 ① 逐条照 crates 写）。一段 20 次发布、只有两次建 inode 的历史，原样（截了 rows_trajectory 之后的字段，全行在 `out/w4.out`）：

```
E7RESULT name=history history=single_head_two_inode_publishes step_rule=crates design=rebuild fold=none publishes=20 rows_final=18 stale_rows_final=0 deleted_head_rows_final=0 versions_with_any_check_red=0 i313_syntactic_red=0 count_check_red=0 i930_one_red=0
E7RESULT name=history history=single_head_two_inode_publishes step_rule=crates design=point_delete fold=arrival publishes=20 rows_final=18 stale_rows_final=0 deleted_head_rows_final=0 versions_with_any_check_red=0 i313_syntactic_red=0 count_check_red=0 i930_one_red=0
E7RESULT name=history history=single_head_two_inode_publishes step_rule=crates design=point_delete fold=naive_max publishes=20 rows_final=35 stale_rows_final=17 deleted_head_rows_final=0 versions_with_any_check_red=0 i313_syntactic_red=0 count_check_red=0 i930_one_red=0
```

机制：发布 g 的前端里有「上一代水位那一行」的墓碑；单调统计量按最大值折叠时它活下来，成了一条代段早于 txg 的水位行。crates 的三处语法认法都把它当台阶行：照抄规则「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < version_txg」（`crates/singlefs-core/src/transaction.rs:3691`），checker 的 I-3.13「&& row_generation < accounting_version_txg;」（`crates/singlefs-checker/src/walk.rs:2653`）跳过它，恢复的条数判「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg」（`crates/singlefs-core/src/recovery.rs:5720`）不数它；I-9.30 ① 比的是 H，假台阶的 H 等于真台阶的 H。于是每次发布多留一行、永远照抄，三道检查 20 版 0 红。两条路（整棵重建、前端点删 + 到达序折叠）逐版逐行相同（`two_paths … differing_versions=0`）：第 3 行的规矩（等价于 D8 已定项 13「E 按同一次发布里的到达序取最后一次改动，不按值取」（`.claude/kb/decisions/08-核心索引结构.md:397`））正是让点删等于重建的那一条。

结论：第 3 行与 C344 不能销账。可以讨论的是「第 3 行与 D8 已定项 13 重复、并进去」（缩题），而 C344 的「会红的检查」要换形：今天的语法认法认不出假台阶，判它要按 D5 意义认台阶（那一版水位真的变了），或判「同一 (统计量, 树) 下相邻两级台阶的 H 严格递增」——假台阶的 H 与前一级相等，按定义这一判会红；我没把它写进模型，**推的，没实现、被攻过零轮**。四句：① 分辨两种折叠（naive_max 17 行陈旧、arrival 0 行）。② 系统看得到：墓碑在前端条目里，排空时就在手边。③ 打中的是摸底报告的前提句「从不点删」，不是哪条判据。④ 改法「销账」在这几格上照样留着那 17 行。

### W4-Q2　删一个可写头时它的台阶行怎么离开

**今天的 crates 上答：没有路径，也没有条款。** crates 只有一个可写头、没有克隆路径，照抄台阶行不分树：「这里不按树段分；做克隆时按树段分开取、建克隆那次给新头留一行」（`crates/singlefs-core/src/transaction.rs:3683`）。D5 已定项 2 射程说台阶行「随根 COW、一直留着」，没有一句说头删掉之后怎么办（`grep -n '台阶行' .claude/kb/decisions/*.md | grep -v '\.en\.md\|\.ja\.md' | grep -E '删|销毁|离开'` 只命中 D18 已定项 24 谱系记录那一行，与删头无关）。

我的模型里一段「第 2 次发布建克隆、第 4、5 次在克隆里建 inode、第 10 次删克隆」的历史，三种照抄规则：

```
E7RESULT name=history history=clone_created_then_deleted step_rule=per_tree_all design=rebuild fold=none publishes=20 rows_final=19 stale_rows_final=3 deleted_head_rows_final=3 versions_with_any_check_red=16 i313_syntactic_red=0 count_check_red=0 i930_one_red=16
E7RESULT name=history history=clone_created_then_deleted step_rule=per_tree_live design=rebuild fold=none publishes=20 rows_final=16 stale_rows_final=0 deleted_head_rows_final=0 versions_with_any_check_red=5 i313_syntactic_red=0 count_check_red=0 i930_one_red=5
```

- 按 TODO 字面「按树段分开取」、不看头还在不在（`per_tree_all`）：删掉的头 3 行台阶行永远照抄。I-3.13 语法判、条数判 0 红；I-9.30 ① 红 16 版，但正确的那一形（`per_tree_live`）也红 5 版——它今天不按头分（walk.rs 那条 TODO），红得不分辨，不算抓到。
- 只照抄活头的台阶（`per_tree_live`）：重建落法下干净。可在点删落法下，这 3 行就要以墓碑离开——又是单调统计量上的墓碑，W4-Q1 的朴素折叠在同一段历史上留下 8 行（`out/w4.out` 的 `per_tree_live design=point_delete fold=naive_max … deleted_head_rows_final=8`）。
- crates 今天的规则（`step_rule=crates`）在两个头时只认 `find` 到的第一条这一版行，克隆头的台阶从建克隆那次起就没被照抄（模型里 `step_rule=crates design=rebuild … deleted_head_rows_final=0`，0 的原因是从来没留下，不是删干净）——这是那条 TODO 的已知缺口，不是删头的答案。

结论：删头那一问的答案决定 C344 的另一半对象，而它今天在条款里是空白：要补一句「删一个可写头那次不照抄它的台阶行」（重建落法），或「它的台阶行以墓碑离开、墓碑按到达序折叠」（点删落法）。两句都是**推的、被攻过零轮**。四句：① 分辨两种照抄规则（3 行永留对 0 行）。② 看得到：头删没删在树表里。③ 打中的是摸底报告第 2 问本身没有现成答案。④ 改法「销账」碰不到这一格。

## 几个改法各修哪一格

全部是我提的，**只在我的模型上量过、被攻过零轮**；「量过」= `out/` 下有原样输出，「推的」= 没实现没跑。

| 改法 | 修哪一格 | 在打中的格上 | 标 |
|---|---|---|---|
| E186 反向接受条款补一支「依赖取样点 ⇒ 交用户，并列出翻转变量」，触发句限到 h_new > 0 | W1-甲 | 1600 个 h_new = 0 点不再触发；1280 / 1920 点交用户而不是自动翻面 | 推的（改的是判据，按 evidence-discipline 是新一轮） |
| 第 6 位按 B2 读，或把 U5 的空真改成「条款里点名的对象在这条臂下不存在 ⇒ 记 1」 | W1-乙、W1-丙 | r_tree_dim = 1 合法点 Carry 3200 独胜 | 量过（`variant=u5_dangling`、入库产物第 186067 行） |
| 第 7 位按记账节点数读 | W1-丁 | r_tree_dim = 1 合法点 Carry 处处在胜者里 | 量过（`variant=nodes`） |
| ㊀ 改成按统计量的规则（有台阶行例外的统计量不带这一代那一行，其余照带） | W1-戊、r_tree_dim = 2 的丢行 | 没量 | 推的 |
| Q2 判胜者前按写者看不到的变量取最坏 | W2-甲 | 2400 组全 RejectEntry | 量过（`q2_flags_winners_w_gens_unobserved`） |
| crates 把「不认识的 flags 位」从 `EntryMalformed` 里分出来，不进 F2 退回 | W2-丙 | 那段历史不再丢 P | 推的 |
| ㊁ 射程补「day-1 核心树的条目 EIO ⇒ 拒可写挂载」 | W2-丙 末句 | — | 推的 |
| D5:145 删 E71 名下那半句，理由写「E71 宽读臂旧口径就不过界」 | W3-甲 | — | 量过（E71 产物第 91 行） |
| 已定项 1、5 的树高句改成「一页叶装得下时段宽 0 代价；超过一页叶，写出的叶数随条目宽走」，不写「恒」 | W3-乙、W3-丙 | 覆盖 d = 79..141 与宽读格 | 量过（`out/w3.out`） |
| 第 3 行并进 D8 已定项 13（缩题），C344 的检查改成按 D5 意义认台阶或判相邻台阶 H 严格递增 | W4-Q1 | naive_max 那 17 行要靠它红 | 前半量过（模型），检查推的 |
| 补「删头那次不照抄它的台阶行」或「以墓碑离开、按到达序折叠」 | W4-Q2 | 3 行永留 / 8 行永留两格 | 量过（模型），条文推的 |

## 没打中的形状

- **W1：r_tree_dim = 2 那一半。** 换 B2、U5、节点三种读法，r_tree_dim = 2 合法点都是 `Carry:3200|Carry+NoCarry+KeepWithRoot:1600`（`out/e186-reanalysis.out` 四种 variant 各两行），我找不到让 NoCarry 在那一半赢的读法。
- **W1：几何、G、k_keep。** 实验页的几何敏感性 19 行已经量过，换几何与 G 不翻；我没另扫。
- **W2：Q2 在 w_gens > 0 的 3600 点上。** 四种读法下胜者都是 RejectEntry，我没找到打它的读法。
- **W3：已定项 4 的定案本身**（见 W3-丁）。
- **W4：两条路的等价**——整棵重建与点删 + 到达序折叠在两段历史 × 20 版上逐行相同，没找到让到达序折叠出错的历史（试过单头两次建 inode、建克隆再删克隆两种，每种 20 次发布）。
- **崩溃点。** 这一轮的打中都不靠崩溃：W1–W3 是读法与算术，W4 是无崩溃的发布序列。我没调 `enumerate_crash_states`，没有崩溃状态被缩掉或漏掉；W4 的前端折叠在崩溃之后（排空做到一半）是什么样，没量。

## 这条腿自己的限度

- E186 那几格的数是我拿入库产物的格行重算的（自证 112 行判决行逐字对上），没改、没编入库装置；按 `.claude/rules/three-way-inference.md`，要引就得在入库装置上另做一次跑（例：给 `e186_unknown_tree_accounting_rederive.rs` 加节点读法与 U5 的另一读法当附带量）。
- W3 的算术沿用 E97 的「只数叶、不数内部节点」与 crates 的节点格式；内部节点我在 crates 几何下数了（`nodes_rewritten`），E97 几何下只数了树高。
- W4 是独立手写模型，行形与三条语法认法照 crates 今天写；crates 没有前端、没有克隆、没有删头，这三样的行为全是按条款推着写的，不是 crates 的路径。
- W2-丙 全靠读码，没造一个带未知 flags 位的盘去跑可写挂载；「P 那次不轮换系统配置槽」这个前提没核。
- 读了正文、背景材料里 E186 / E71 / E97 那几段、三份产物、E186 跑前登记、D5 / D8 中文版对应小节、C344 / C693 两行、摸底报告第 2 项 ① 节、crates 里引到的那几段；没读英文、日文版与别的 kb 页。

## 要主 agent 跑的命令

无重型、无轻型。要核 W2-丙，需要一个 harness 档的用例（造一个最新一版树表条目带未知 flags 位、系统配置没见证它的池，可写挂载后看挂到哪一版），这一条我没写，交主 agent 定要不要立。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没碰 crates、kb、三份入库装置。
- 没跑 cargo、没编译任何东西；三份脚本都是 Python 纯算术，经内存包装 2G 跑。
- 草稿目录 `~/.local/share/singlefs/work/m3-close-c693-r1-opus/` 里只有几份输出与复跑目录 `rerun1/`（里面是开工快照的拷贝，只有 `kb-sha256.txt`），没有仓副本与编译目录。


## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m3-close-c693-r1-opus-output.md` 末行（写这一节之前跑的；这一节本身不含引文）：

```
  ✓ 核了 37 处引文、1 处裸行号，对上 37 处，没判 1 处（其中裸行号没有引文可比 1 处；逐处列在上面）
```

没判的那 1 处是 W3-丙 里指回 W4-Q1 已引过的 `code_two_tree.rs` 第 655 行，原文在 W4-Q1 那一处带引文核过。
