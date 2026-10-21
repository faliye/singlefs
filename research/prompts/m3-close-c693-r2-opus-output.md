# m3-close-c693-r2 攻方腿（Opus）报告

日期 2026-10-20。立场：假设第一轮的修补（E186 第二次跑判 `unique_carry`、㊁ 的对象级 EIO 改法、墓碑缩题与删头的候选）是错的，造读法、盘面与历史去打。只攻正文第三节 V1–V4，重点 V1、V2、V4；V3 只补一处第一轮没扫的变量。不读这一轮正推腿的产物（禁读清单）。不重复第一轮攻过的角度：本报告不再论 B1 与 B2 哪个对、不再论「U5 空真少算」、不再按节点数重读第 7 位、不再论 `w_gens`、E71 宽读、树高在 t × d 上的取样、前端逼出墓碑。

## 复跑命令与文件 sha256

```bash
# 仓根；模型三件：E186 两次跑的入库产物重判（纯 Python），与两条 harness 测试目标（仓副本里编、跑；只加两份测试文件，crates 一字不改）
RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600 bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-c693-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen research/prompts/m3-close-c693-r2-snapshot --work <新的草稿目录>
# 末尾三行 name=rerun_compare … identical=1 即逐字节复出 out/ 下三份（测试结果行的耗时已抹掉）
```

本腿照这一条跑过一次（2G 包装，排队 111 秒起跑，退 0；草稿目录 `~/.local/share/singlefs/work/m3-close-c693-r2-opus/rerun2`，仓副本从主工作区现拷、现编），原样末四行（`name=rerun_roots` 那一行截掉了末尾 work= 的路径）：

```
name=rerun_compare file=e186-second-run-reread.out identical=1 lines=14
name=rerun_compare file=future-fields-probe.out identical=1 lines=82
name=rerun_compare file=planner-carry-cost.out identical=1 lines=34
name=rerun_roots repo=/home/fy5090/code/singlefs frozen=/home/fy5090/code/singlefs/research/prompts/m3-close-c693-r2-snapshot model=/home/fy5090/code/singlefs/research/prompts/m3-close-c693-r2-opus-model
```

```
aa778b65e5f60c2ce8ad589614950bbb8c258c32da4a3399663aee35213bd7e0  research/prompts/m3-close-c693-r2-opus-model/e186_second_run_reread.py
ad33ef0a35161c56e9d8ff778f96e5f499f04625710274623634b99edacb5fea  research/prompts/m3-close-c693-r2-opus-model/accounting_planner_carry_cost_with_long_step_histories.rs
f95db198966c7e1cc6d18f93ae343b3229b27ef94270906d95c5c2a7da68c666  research/prompts/m3-close-c693-r2-opus-model/tree_table_entry_future_fields_in_the_newest_version.rs
126fd96e00d4cff592fb0f7307fed701d0bf3ac4596cf2669486a4d371f9a642  research/prompts/m3-close-c693-r2-opus-model/tools/rerun.sh
7ca5c71456c5f6bb6e06f7a04aa887740b6fb901f48986eaa02685b623a38993  research/prompts/m3-close-c693-r2-opus-model/out/e186-second-run-reread.out
db3d9c9fd6a92deb0b7e50d2d94bcecf905479fcf7e8f22f550b4e204a00c260  research/prompts/m3-close-c693-r2-opus-model/out/future-fields-probe.out
9f77ffa6c6019f57d692fd94a67cb759bfb8c4f54cca0cabf2b7eb2f4a0e8140  research/prompts/m3-close-c693-r2-opus-model/out/planner-carry-cost.out
```

读的两份入库产物（开工时 sha256，与实验页记的相同）：

```
f1dfd78bc92b7ed919af4484e334b379b615f3914b71a5cf36f513843439a86a  research/results/e186-unknown-tree-accounting-rederive.out.gz
ef2bd29e33654aa9d3dab846a98bd297d9fa74c14fbd096648da68316197705e  research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz
```

三件各带会红的自证：`e186_second_run_reread.py` 先在「U5 不计」读法下把第二次跑的 172800 格向量逐格比第一次跑（应当逐格相等，因为第一次跑的 U5 从不为假），原样 `E7RESULT name=selftest check=u5_excluded_equals_first_run cells=172800 mismatches=0`；弄坏开关 `E186_REREAD_BREAK=keep_u5`（不减 U5）下原样 `E7RESULT name=selftest check=u5_excluded_equals_first_run cells=172800 mismatches=38400`、退 3；它照第二次跑登记原样重判，复出产物第 186276–186278 行（下面 V1-甲 贴）。两条 harness 目标各带对照：未来字段那一条的对照臂（同一套重写只改读者不看的 `previous_snapshot_txg`）三种盘面三处都落在 R；规划器那一条断言容量等于 crates 的叶 477、内部 150。副本数注明是副本：两条 harness 目标是在我草稿目录的仓副本（`rsync` 自主工作区 `crates/`，未改一字，只加两份测试文件）里编、经 1G 内存包装跑的。

## 各格判定一览

「量过」= 贴得出原样输出（入库产物上的重判、或副本里的 harness 目标）；「推的」= 读码或读条款推的，没跑。

| 格 | 攻的那一步 | 判定 | 量过 / 推的 |
|---|---|---|---|
| V1-甲 | 第二次跑 `unique_carry` 撑不撑得住 | **打中**：新 U5 记的正是 B1 排除理由所指的那一类（点名 ㊀ 的句子，非现任臂每条 +1）。按 B1 自己的理由把 U5 也排掉、其余照第二次跑登记（被判点集、形态表）不动，`legal_pooled` 形态是 `depends_on_point_mixed`、后果「交用户」；`r_tree_dim = 1` 组 `depends_on_point_carry_nowhere` | 量过（入库产物格行重判） |
| V1-乙 | 第二次跑的臂集够不够 | **打中**：把第一轮攻方的「按统计量」臂（零轮）照格行派生进来，照第二次跑判据原样判，`r_tree_dim = 2` 组 `PerStatistic` 3200 点独胜，`legal_pooled` 混合、交用户 | 量过（派生臂，不是装置的一次跑） |
| V1-丙 | ㊀ 射程里写「搬运省下的行在设备上是 0 个节点」 | **打中**（射程）：只在不认识的头台阶行短时成立；台阶行 500 级起 `Carry` 每次发布比 `NoCarry` 多重写 h_new 个记账节点（2 头 +2、4 头 +4） | 量过（副本，crates 规划器） |
| V1-丁 | 第二次跑有没有别的判据该改没改 | U1–U13 按新 U5 的「悬空即要改」通扫：除 U5 与 U12（B2）外没有一条在三条臂下悬空不同。漏改的不是别的条，是 B1 的射程句（并入 V1-甲） | 推的（读条款） |
| V2-甲 | 改法只改「不认识的 flags 位」 | **打中**：同一个 `None` 里还有预留 76 字节非零（D8 认购表要写的字段）与条目长度字段（D8 ④ 为兼容设的）；两样在三种盘面三处的结局与 flags 位逐项相同（没见证就静默退到上一版）。只改 flags 那一支的改法对它们不起作用 | 量过（副本 harness，12 条用例） |
| V2-乙 | 对象级 EIO、不进 F2 | **打中**：被 EIO 的那棵树的单元出了三处的读集（C755 的并集），R 没见证且那棵树有单元读不出（F2 本来要接的那一形）时三处不退、可写挂载提交在 R 上 | 推的（读码与条款） |
| V2-丙 | 对象级 EIO 对哪些树兑现得了 | **打中**：每次发布都重写的树（记账树整棵重规划；树表本身）带不认识的位时，可写挂载之后第一次发布没有合规的写法；调查的盘面是 inode 树，「对象级」就是整个命名空间 | 推的 |
| V2-丁 | 与 C755、C751 甲的交集 | **交集在 C755**：并集收不收「单元读得出、条目解不开」决定 flags 改法落在哪一处；C751 甲只看实例表指针，与 flags 不相交 | 推的 |
| V4-甲 | 缩进 D8 已定项 13、检查换形 | **打中**（检查的方向）：13 的定案已逐字含「不许把墓碑当数值…取最大」，并进去不改字；而 C383 与 D8 已定项 12 的两条检查都是「叶上的改动 ⊆ 前端条目」这一方向，朴素取最大吞掉墓碑时叶上没有改动，两条都绿 | 推的（读条款；对象是第一轮模型里 `naive_max` 留下的那一形） |
| V4-乙 | 删头时台阶行候选「删头那次不照抄」 | **打中**：与 D18 已定项 24 第 5 条「谱系只增不删、按头与台阶行按等于互核」冲突，被删的头有谱系记录时 I-9.30 按头判红 | 推的 |
| V4-丙 | 与 `crates/singlefs-core/src/transaction.rs:3682` 的 TODO 是不是同一件事 | **不是**：TODO 只点名一个函数；同族不分树的还有三处（水位取值、恢复条数判、checker），删头要的「哪几行属于被删的头」以分树为前提 | 量过（grep 的码事实）+ 推的 |
| V3 | 树高句的改后条文 | **打中**（漏变量）：台阶行条数是树高的又一个变量（D5 已定项 2 射程自己给了 462 的门槛）；只写 t × d 与 key 宽的条文在台阶行多的池上错 | 量过（副本规划器的 `height_in_r` 列） |

## V1　D8 已定项 8 ㊀（E186 第二次跑）

被攻的那一步：第二次跑「`legal_pooled` 6400 点 `unique_carry` ⇒ ㊀ 不翻面」。第二次跑的判决行（`research/results/e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz` 第 186276–186278 行）原样，我的重判在「照登记原样」读法下逐字复出同一判定（`out/e186-second-run-reread.out` 第 2–4 行）：

```
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_r_tree_dim_1 judged=yes judged_points=3200 excluded_points_h_new_0=1600 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_r_tree_dim_2 judged=yes judged_points=3200 excluded_points_h_new_0=1600 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=verdict check=q1_carry_acceptance_form judgement=dangling-u5 reading=b1_w2_q2_counted group=legal_pooled judged=yes judged_points=6400 excluded_points_h_new_0=3200 form=unique_carry consequence=carry_stands winner_sets=Carry:6400
```

```
E7RESULT name=q1_form reading=as_registered arms=three_arms group=legal_r_tree_dim_1 judged_points=3200 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=q1_form reading=as_registered arms=three_arms group=legal_r_tree_dim_2 judged_points=3200 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=q1_form reading=as_registered arms=three_arms group=legal_pooled judged_points=6400 form=unique_carry consequence=carry_stands winner_sets=Carry:6400
```

### V1-甲　新 U5 就是 B1 要排掉的那一类条

B1 排掉 ㊀ 的理由，`research/prompts/e186-preregistration.md:29`：「理由是每条非现任臂都要改它，计进去等于先给现任臂一票」。B1 的字面射程只到「D8 已定项 8 的 ㊀（连同射程里说 ㊀ 的句子）」，同一行里写的。

新 U5 数的是 D8 已定项 12 定案里点名 ㊀ 的半句，`.claude/kb/decisions/08-核心索引结构.md:383`：「这三棵树上的写**一律进前端**——含已定项 8 ㊀ 那类旧写者搬运的记账行」。新谓词 `research/prompts/e186-preregistration.md:801`：「N > 0 且 W_g 为空 ⇒ 假（那半句所说的行在这条臂下不存在，条款成了空指称，要改）」。登记自己推出它记几，`research/prompts/e186-preregistration.md:753`：「新 U5 在一格上记 1，当且仅当：记账臂是 `NoCarry` 或随根留着」；也自己承认，`research/prompts/e186-preregistration.md:760`：「新 U5 的值由臂的定义直接给出（不写行的两条臂记 1）」。

也就是说，新 U5 是「每条非现任臂都要改它」的又一句：这半句只因为 ㊀ 取搬运才存在，换掉 ㊀ 的每一条臂都要删它，删它不需要对前端那一条作任何别的改动。B1 排掉 ㊀ 本身的理由逐字套得上它，只是它住在 D8 已定项 12 而不住在 ㊀ 的射程里，B1 的字面没罩到。第一轮判决改 U5 时没有拿它对 B1 的理由核一遍。这一位又正是 `r_tree_dim = 1` 被判点上 `Carry` 胜过 `NoCarry` 的唯一一位（第 1–5 位全平，第 7 位 `NoCarry` 更小；第二次跑的实验页逐点七位：`NoCarry` 由 `…,1,18` 变 `…,2,18`、`Carry` 仍 `…,1,20`）。规则侧还有一句：`.claude/singlefs-ai-sop/rules/test-discipline.md@判据不写成合取，门槛不能是臂定义的同义反复`：「门槛不能是被测臂的定义」。

照 B1 自己的理由把 U5 也排掉（实现：第二次跑的格行里 `clauses_false` 含 U5 的，第 6 位减 1），被判点集、形态表、读法 W2、Q2 计、十五格总胜者一律照第二次跑登记不动。自证：这样得到的 172800 格向量与第一次跑产物逐格相等（`mismatches=0`，见报告开头）。原样（`out/e186-second-run-reread.out` 第 8–10 行）：

```
E7RESULT name=q1_form reading=u5_excluded arms=three_arms group=legal_r_tree_dim_1 judged_points=3200 form=depends_on_point_carry_nowhere consequence=flips_choice_to_user winner_sets=NoCarry+KeepWithRoot:1920|NoCarry:1280
E7RESULT name=q1_form reading=u5_excluded arms=three_arms group=legal_r_tree_dim_2 judged_points=3200 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=q1_form reading=u5_excluded arms=three_arms group=legal_pooled judged_points=6400 form=depends_on_point_mixed consequence=left_to_user winner_sets=Carry:3200|NoCarry+KeepWithRoot:1920|NoCarry:1280
```

`legal_pooled` 落在第二次跑登记表里的「依赖取样点·混合」，`research/prompts/e186-preregistration.md:916` 那一行写的后果是「不判翻面也不判站住，交用户」。所以「㊀ 不翻面」只在「B1 排 ㊀ 本身、却计点名 ㊀ 的另一句」这一种组合下成立；两种自洽的读法里，一种（B2，把 ㊀ 与点名它的句子都计）给 `Carry`（第一次跑产物第 186067 行是 B2 加空真 U5，`h_new > 0` 的 3200 点已是 `Carry` 独胜；再计悬空的 U5 只给另两条各加 1，不改这个胜者），一种（B1 照它的理由排掉两句）给「混合、交用户」。第二次跑的判决押在两者之间那个不自洽的点上。

打中之后四句：① 分辨臂：分辨（`r_tree_dim = 1` 的 3200 个被判点上，这 1 是 `Carry` 胜过 `NoCarry` 的唯一一位）。② 系统看不看得到：不适用（这一格判的是判据，不是被判系统）。③ 满足的分句：B1 的理由句（`e186-preregistration.md` 第 29 行）对新 U5 字面成立；B1 的射程句没罩到它。④ 跑前条款给的改法：第二次跑的形态表本身（第十节）在打中的格上照常工作，只是它给出「交用户」；能把结论拉回 `unique_carry` 的只有放弃 B1（改 B2），那是主 agent 跑前定的读法，要动它得交用户。

推翻条件：主 agent 判 D8 已定项 12 那半句不是「说 ㊀ 的句子」，即换掉 ㊀ 之后它除了删这半句还要改别的（例如前端那一条因此要改形态）。我按 D8:383 原文读，删掉这半句之后前端那一条的其余字一字不动。

### V1-乙　臂集缺「按统计量」那一条，第二次跑判据原样下它在 `r_tree_dim = 2` 赢

第一轮攻方 W1-戊（零轮）提的「按统计量」规则：有台阶行例外的统计量（今天只有 inode 号水位）不带这一代那一行，其余统计量照 `Carry` 带。它没进 E186 的臂集。我在被判点上照格行派生它的七位（规则写在脚本头）：记账臂起作用的 flags 格里，`r_tree_dim = 1` 时等于 `NoCarry` 那一格；`r_tree_dim = 2` 时等于 `Carry` 那一格、第 7 位减 h_new；其余 flags 格等于 `Carry` 那一格。一个点核一遍：点 6573（实现两盘几何、`h_new = 4`、`s = 3`、`lpc = 0`、`r_tree_dim = 2`、`w_gens = 3`、`G = 10`），第二次跑产物第 105176、105177 行：

```
E7RESULT name=cell point=6573 arm=RejectEntry_Carry writable_gens=10 rdonly_violations=0 dead_revived=0 vector_q2_counted=0,0,0,0,0,1,37
E7RESULT name=cell point=6573 arm=RejectEntry_NoCarry writable_gens=10 rdonly_violations=0 dead_revived=0 vector_q2_counted=0,0,4,1,0,2,29
```

派生的 `RejectEntry_PerStatistic` 是 `0,0,0,0,0,1,33`：第 3 位 0（水位元组靠 3 级台阶读得出，第二个统计量照带）；第 6 位与 `Carry` 同（U5 下它这次为不认识的树写了第二个统计量那几行、代段都是这一版，原谓词为真；其余条与 `Carry` 同）；第 7 位 = 自己那些行 16 + 认识的头一级台阶 1 + 4 × (3 + 0 + 2 − 1) = 33，与登记乙类闭式同一个式子少一项。照第二次跑登记原样判（`out/e186-second-run-reread.out` 第 5–7 行）：

```
E7RESULT name=q1_form reading=as_registered arms=three_arms_plus_per_statistic group=legal_r_tree_dim_1 judged_points=3200 form=unique_carry consequence=carry_stands winner_sets=Carry:3200
E7RESULT name=q1_form reading=as_registered arms=three_arms_plus_per_statistic group=legal_r_tree_dim_2 judged_points=3200 form=unique_other_arm consequence=flips winner_sets=PerStatistic:3200
E7RESULT name=q1_form reading=as_registered arms=three_arms_plus_per_statistic group=legal_pooled judged_points=6400 form=depends_on_point_mixed consequence=left_to_user winner_sets=Carry:3200|PerStatistic:3200
```

再叠上 V1-甲 的读法（第 11–13 行），`PerStatistic` 在每个被判点的胜者集合里：

```
E7RESULT name=q1_form reading=u5_excluded arms=three_arms_plus_per_statistic group=legal_pooled judged_points=6400 form=depends_on_point_carry_nowhere consequence=flips_choice_to_user winner_sets=PerStatistic:3200|NoCarry+KeepWithRoot+PerStatistic:1920|NoCarry+PerStatistic:1280
```

也就是说：`Carry` 在 `r_tree_dim = 2` 赢的是 `NoCarry` 那条稻草（`NoCarry` 连没有台阶行的统计量也丢），不是「不带有台阶行的那一行」这条能力；在 `r_tree_dim = 1` 赢靠的是 V1-甲 那一位。正文问「㊀ 站住之后 W1-戊 还要不要」：要。它是第二次跑判据原样之下唯一能在 `r_tree_dim = 2` 打败 `Carry` 的形态，第二次跑的「唯一 Carry」是臂集缺它的产物。

打中之后四句：① 分辨臂：分辨（`r_tree_dim = 2` 上第 7 位差 h_new）。② 系统看不看得到：看得到——写者认识统计量标签（不认识的标签照 C71 拒可写挂载），知道哪一个统计量有台阶行例外。③ 满足的分句：第二次跑登记第十节「唯一别的臂 ⇒ ㊀ 翻面成 X」（`r_tree_dim = 2` 组）与「依赖取样点·混合 ⇒ 交用户」（合起来那一行）。④ 改法：第二次跑没有给「臂集外的形态」写任何条款，只在「混合」那一行写了「按翻转变量分支的规则不在臂集里，标臂集外，未量」；按统计量的规则不是按 `r_tree_dim` 分支（同一个写者版本里一条规则对每个统计量各判一次），那句话罩不到它。

限度：`PerStatistic` 的向量是我按臂的定义从格行派生的，不是装置的一次跑；要引得在入库装置上加这一条臂另跑（照 `.claude/rules/three-way-inference.md` 判决那一条）。派生只在合法点上成立：(`s`, `lpc`) = (0, 0) 时水位那一元组没有台阶行，这条规则会丢行，我没派生那一组。

### V1-丙　「搬运省下的行在设备上是 0 个节点」只在台阶行短时成立

第一轮攻方 W1-丁 的数：每格每代最多 47 行，一片记账叶 477 条，三条臂每次发布都写 1 个记账节点。正文要把它当代价信息写进 ㊀ 的射程。47 这个数来自 E186 的取样点，那里台阶行至多 3 级（登记第一节第 8 条 (`s`, `lpc`) 的六组）。而台阶行在每次建 inode 的发布都留一行、随根一直留着，`.claude/kb/decisions/05-快照-空间记账机制.md:55`：「代价：一行 34 字节；每次发布多写的单元，台阶行到第 462 行为止是 0」。一个用过一阵的可写头（今天的主头，将来的克隆头）台阶行是几百、几千级。

我在副本里用 crates 的规划器（`plan_the_tree_after_this_publish`，记账 key 22 字节、叶 477、内部 150）量三条记账臂每次发布重写的记账节点数，扫盘数、认识的头的台阶数、不认识的头数与每头台阶数；行形照 `accounting_entries_of_this_publish`，旧写者 10 次发布不建 inode（`out/planner-carry-cost.out`，`steady_max` 是第 2–10 次发布里的最大值）。抽几行原样：

```
R2PLANNER devices=2 known_steps=1 unknown_heads=2 unknown_steps=3 rows_in_r=24 height_in_r=1 Carry:first=1,steady_max=1 NoCarry:first=1,steady_max=1 KeepWithRoot:first=1,steady_max=1
R2PLANNER devices=2 known_steps=1 unknown_heads=2 unknown_steps=500 rows_in_r=1018 height_in_r=2 Carry:first=4,steady_max=4 NoCarry:first=4,steady_max=2 KeepWithRoot:first=2,steady_max=2
R2PLANNER devices=2 known_steps=1 unknown_heads=4 unknown_steps=2000 rows_in_r=8020 height_in_r=2 Carry:first=6,steady_max=6 NoCarry:first=6,steady_max=2 KeepWithRoot:first=2,steady_max=2
R2PLANNER devices=64 known_steps=1000 unknown_heads=4 unknown_steps=500 rows_in_r=3391 height_in_r=2 Carry:first=8,steady_max=8 NoCarry:first=8,steady_max=4 KeepWithRoot:first=4,steady_max=4
R2PLANNER devices=2 known_steps=1000 unknown_heads=4 unknown_steps=3 rows_in_r=1031 height_in_r=2 Carry:first=3,steady_max=3 NoCarry:first=3,steady_max=3 KeepWithRoot:first=3,steady_max=3
```

32 格里：不认识的头每头台阶 1 或 3 级的 16 格，三条臂每次发布重写的节点数相同（W1-丁 的结论在这一段成立）；每头 500 或 2000 级的 16 格，`Carry` 稳态比 `NoCarry` 多重写恰好 h_new 个节点（2 头 +2、4 头 +4），随根留着与 `NoCarry` 同数。机制：记账 key 按（统计量, 树, 设备, 代）排，每个头的水位台阶连成一段、「这一代的行」在段尾；台阶长到一片叶装不下时，每个不认识的头的段尾各在一片叶里，`Carry` 每次发布删上一代那一行、插这一代那一行，就把这几片叶连同路径重写一遍。

所以写进 ㊀ 射程的代价句不能是「0 个节点」：只能写成「不认识的头的台阶行与别的行共一片叶时 0 个节点；每个不认识的头的台阶行自成一片叶以上时，每次发布多重写 h_new 片叶」，变量是「每头台阶行条数」，不是盘数也不是 `r_tree_dim`。这一条不改 V1-甲、乙的判（第 7 位是用户定的按行数），只改正文要写进射程的那句代价信息。

打中之后四句：① 分辨臂：分辨（`Carry` 对 `NoCarry`、随根留着，长台阶那 16 格）。② 系统看不看得到：看得到（台阶行就在记账树里）。③ 满足的分句：W1-丁 的「三条臂每次发布都写 1 个记账节点」是全称句，取样只到 3 级台阶。④ 改法：正文打算写的「省的行在设备上是 0 个节点」在长台阶那 16 格上照样错；上面那句带条件的写法在 32 格上都对（量过）。

### V1-丁　第二次跑有没有别的判据该改没改

把新 U5 的规则（「条款点名的对象在这条臂下不存在 ⇒ 要改」）通扫 U1–U13：U11（`research/prompts/e186-preregistration.md:439`，D23「不点删旧代」，模型里整棵重建「没有点删这一步就空真」）在三条臂下一样空真、不分辨；U8–U10 的对象（每个头、每级台阶）三条臂下都在；U12、U13 只在 B2 下计。设计员另搜点名 ㊀ 那类行的条款（`research/prompts/e186-preregistration.md:815` 那一段）只得 D8 已定项 8 自己与 D8:383 两处。我按反方向（在 `Carry` 下悬空的条）读了清单 grep 命中的 24 行，没有一句点名「不带过去」这类只在 `NoCarry` 下存在的东西，除了 ㊀ 自己射程里那句「不搬的代价」（`.claude/kb/decisions/08-核心索引结构.md:282`，B1 排掉）。所以第二次跑漏改的不是别的条，是 B1 的射程句——见 V1-甲。推的（读条款，没量）。

## V2　㊁ 与实现的出入：改法「不认识的 flags 位 ⇒ 对象级 EIO、不进 F2」

被攻的那一步：正文 V2 的改法「不认识的 flags 位做成 ㊁ 的「该条目 EIO、位置还在」（对象级），不进整表拒、不进 F2 退回」。㊁ 原文 `.claude/kb/decisions/08-核心索引结构.md:262`：「未知位一律拒收——该条目 EIO」。

### V2-甲　「不认识的 flags 位」只是同一个 `None` 里的一样

树表条目的读者在四处交回同一个 `None`：`crates/singlefs-core/src/records.rs:736`：「|| bytes[bytes.len() - 76..].iter().any(|byte| *byte != 0)」（预留 76 字节非零）、`crates/singlefs-core/src/records.rs:742`：「if u64::from(reader.get_u16()) != TREE_TABLE_ENTRY_BYTES {」（条目长度不是 200）、`crates/singlefs-core/src/records.rs:753`：「_flags_this_reader_refuses => return None,」（flags 位），以及根指针头部那一判（同一函数的文档注释列了 MAC / nonce、算法、压缩码、extent 偏移）。它们由 `crates/singlefs-core/src/recovery.rs:2804`：「TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;」一起变成整表拒。

其中两样是格式条款明写要给将来的写者用的。D8 已定项 8 ④ 取条目自带长度，`.claude/kb/decisions/08-核心索引结构.md:260`：「前者让旧读者**跳过**一棵不认识的树」，㊁ 的射程还说 `.claude/kb/decisions/08-核心索引结构.md:282`：「㊁ 拒收不丢数据依赖 ④ 的条目自带长度」——可今天的读者遇到长度不是 200 的条目整表拒。预留 76 字节有认购表，第一行 `.claude/kb/decisions/08-核心索引结构.md:272`：「| livelist 每头计数器 | 头条目 | 8 |」——将来的写者照认购表写进去，今天的读者就判这条条目坏。

我在副本里沿用调查的搭建（同一个 R、同一套校验和链重算、同三种盘面），把「改 inode 条目」换成三样：flags 位 1（调查量过的那一形，作对照）、预留 76 字节第一字节写 1、条目长度字段写 208；另带调查的对照臂（只改 `previous_snapshot_txg`）。12 条用例全过（`test result: ok. 12 passed`）。按用例把三处结局并成一行（命令：对 `out/future-fields-probe.out` 跑一段 awk、再用 sed 去掉 `WithChecksumsRecomputed` 后缀，取 recovery 落在哪一版、两种挂载的 `chosen_root`、挂载之后恢复读出谁的内容；awk 原文在草稿目录 `probe-summary.awk`，没进模型目录，下面的行逐行对着原始输出核过）：

```
change=HarmlessField face=RConfirmedButItsRotationSlotsReadBackZero recovery=r read_only=txg9 writable=txg9 after=content_of_r
change=HarmlessField face=CrashBetweenTheRootOfRAndItsRotation recovery=r read_only=txg9 writable=txg9 after=content_of_r
change=HarmlessField face=SystemConfigurationWitnessedR recovery=r read_only=txg9 writable=txg9 after=content_of_r
change=ReservedBytesSubscribed face=RConfirmedButItsRotationSlotsReadBackZero recovery=version_before_r read_only=txg8 writable=txg8 after=content_of_the_version_before_r
change=ReservedBytesSubscribed face=CrashBetweenTheRootOfRAndItsRotation recovery=version_before_r read_only=txg8 writable=txg8 after=content_of_the_version_before_r
change=ReservedBytesSubscribed face=SystemConfigurationWitnessedR recovery=refused read_only=refused writable=refused after=none
change=UnknownFlagsBit face=RConfirmedButItsRotationSlotsReadBackZero recovery=version_before_r read_only=txg8 writable=txg8 after=content_of_the_version_before_r
change=UnknownFlagsBit face=CrashBetweenTheRootOfRAndItsRotation recovery=version_before_r read_only=txg8 writable=txg8 after=content_of_the_version_before_r
change=UnknownFlagsBit face=SystemConfigurationWitnessedR recovery=refused read_only=refused writable=refused after=none
change=EntryLengthFieldWider face=RConfirmedButItsRotationSlotsReadBackZero recovery=version_before_r read_only=txg8 writable=txg8 after=content_of_the_version_before_r
change=EntryLengthFieldWider face=CrashBetweenTheRootOfRAndItsRotation recovery=version_before_r read_only=txg8 writable=txg8 after=content_of_the_version_before_r
change=EntryLengthFieldWider face=SystemConfigurationWitnessedR recovery=refused read_only=refused writable=refused after=none
```

一组原始输出（预留字节、R 已确认而轮换槽读回全零；`out/future-fields-probe.out` 第 22–28 行，可写挂载那一行截掉了末尾的 `journal=…` 一段）：

```
R2PROBE change=ReservedBytesSubscribedWithChecksumsRecomputed face=RConfirmedButItsRotationSlotsReadBackZero r=(InstanceGeneration(1), CheckpointTxg(9)) version_before_r=(InstanceGeneration(1), CheckpointTxg(8))
R2PROBE   recovery: file_read root=(InstanceGeneration(1), CheckpointTxg(8))(version_before_r) content_of_the_version_before_r effective_root=Some((InstanceGeneration(1), CheckpointTxg(8)))
R2PROBE   read_only_mount: ok chosen_root=(InstanceGeneration(1), CheckpointTxg(8)) effective_root=(InstanceGeneration(1), CheckpointTxg(8))
R2PROBE   writable_mount: ok chosen_root=(InstanceGeneration(1), CheckpointTxg(8)) effective_root=(InstanceGeneration(1), CheckpointTxg(8)) new_instance=InstanceGeneration(2) rows_written=1 abandoned_roots_unreadable=0 isolated_slots_per_device=[(DeviceIdentity(0), 12), (DeviceIdentity(1), 12)]
R2PROBE   recovery_after_the_writable_mount: file_read root=(InstanceGeneration(2), CheckpointTxg(11))(other_version) content_of_the_version_before_r effective_root=Some((InstanceGeneration(2), CheckpointTxg(11))) root_of_r_still_self_verified_in_the_ring=true
```

三样在三种盘面三处的结局逐项相同：没见证 R 就静默落到 R 之前一版、R 的内容没了、挂载返回 Ok；见证了 R 就三处都拒。改法若只在 `records.rs:753` 那一支把 flags 分出来，另两样（还有根指针头部那几样）照样走整表拒与 F2 退回。将来的写者照 ④ 加宽条目、照认购表写预留，是格式条款预告了的升级，比塞一个不认识的 flags 位更早会发生。

另一处同形、没量：`crates/singlefs-core/src/recovery.rs:2812`：「return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);」——同一种类（码 14、15 除外）出现两条就整表拒；将来的写者建克隆、给新头再挂一棵同种类的 inode 树时（种类码怎么取没定），今天的读者走的也是这一条。推的。

打中之后四句：① 分辨：分辨两种改法（只改 flags 对比「凡校验和过而条目解不开的未来字段」）——在预留与长度那 6 格上前者照旧静默退、后者不退。② 系统看不看得到：看得到，四样都是单元自证通过之后在条目字节里读到的。③ 满足的分句：正文的改法字面只说「不认识的 flags 位」。④ 改法：只改 flags 那一支在预留、长度这 6 格上不起作用（量过：今天的结局就是改了 flags 之后它们的结局，那一支碰不到它们）。

### V2-乙　对象级 EIO 的那棵树出了 F2 的读集

F2 要接住的那一形，用户认下的代价写在 `crates/singlefs-core/src/recovery.rs:6891`：「代价（判决同一条，用户认下）：已确认的一版在较新的系统配置槽与它的单元同时读回全零时，c_见证 落回它之前，读回的是上一版。」。C755 定的是三处用走读、打开、建上一版三样读的单元并集判退不退（`research/prompts/_m3-close-rootring-r3-body.md:15` 那一行的第二列，原文里「退不退」带引号，这里不整句抄）。

历史（每一步指到许可它的那一句）：
1. 将来的写者 v2 在发布 R 里给树 X 的条目置一个 v1 不认识的 flags 位，并改写 X 的几个单元（㊁ 许可：flags 只放「不必被旧读者看见」的改动，D8:262 同一行「必须被旧读者看见的改动走系统配置 incompat 位」）。
2. R 的根槽 FUA 落了，轮换的系统配置槽与 X 的一个新单元两份都读回全零（`crates/singlefs-core/src/recovery.rs:6891` 那一形，用户认下它在 F2 的射程里）。
3. v1 可写挂载。照改法，X 的条目 EIO、不进整表拒；v1 不认那一位，就不知道 X 的节点怎么解（位改了语义，`D8:282`「一棵旧读者自以为认识的树被加了位，它的语义变了」），三处都不走 X，X 那个读不出的单元不在任何一处的读集里，C755 的并集里没有它。
4. 所选那一版走读做成 ⇒ 不退；v1 在 R 上取号、发布，R 之前一版被新实例抛弃。
5. v2 回来挂载：X 最新一版的那个单元读不出，而 R 之前一版里 X 的旧单元已随被抛弃的版本可以回收。

今天（整表拒）这条历史落在 R 之前一版：X 是旧的、完整的。改法之后落在 R 上：X 缺一个单元。改法把用户认下的「读回的是上一版」变成了「X 那棵树坏了」。推的：没在副本里实现对象级 EIO（要动恢复、两种挂载、走读四处），没造盘。

打中之后四句：① 分辨：分辨改法前后（今天落上一版、改后落 R）。② 系统看不看得到：看不到——v1 解不开 X，就判不出 X 的单元读不读得出；这正是改法要承认的盲区。③ 满足的分句：改法的「不进 F2 退回」把 X 的单元从三处的读集里拿掉了。④ 改法：要么「带不认识的位、又没见证的那一版，三处不落它也不静默退：照报失败（只读挂载除外）」，要么认下这一格是改法的代价、写进射程。两样都是我提的、被攻过零轮。

### V2-丙　对象级 EIO 对哪些树兑现得了

记账树每次发布整棵重规划（`crates/singlefs-core/src/code_two_tree.rs:655`：「for deleted in previous_keys.difference(keys_after_this_publish) {」），树表每次发布随根指针重写。这类树的条目带 v1 不认识的位时，v1 可写挂载之后的第一次发布（调查里挂载本身就写了一行、发布了一次，`rows_written=1`）只有三种写法：不重写它（违反「每次发布把这一代的行全写一遍」）、去掉那一位照自己的语义写（静默改掉 v2 的语义，即 ㊁ 不许的「忽略未知位」）、留着那一位照自己的语义写（同上）。所以对这几棵树，「对象级 EIO、不拒整个池」只能落成「只读挂载照开、那棵树 EIO；可写挂载拒」。调查造的盘面是 inode 树：它的「对象」就是整个命名空间，可写挂载做成之后用户能做的每一件事都碰它。正文要求「每个改法写明它在调查的三种盘面上各落哪一版」，下面那张表按我读码推的列（第一轮攻方 W2-丙 末句提过「核心树的条目 EIO ⇒ 拒可写挂载」，这里给的是它在三种盘面上的落点）。

| 改法 | 见证了 R | 崩在轮换之前（R 没确认） | R 已确认、轮换槽读回全零 |
|---|---|---|---|
| 今天（量过，调查与上面 V2-甲） | 三处拒 | 三处落 R 之前一版，静默 | 三处落 R 之前一版，静默 |
| 正文改法：flags ⇒ 对象级 EIO、不进 F2（推的） | 三处落 R，inode 树 EIO；可写挂载做成之后无合规写法（V2-丙） | 三处落 R | 三处落 R |
| 我提的：凡校验和过而条目解不开的未来字段 ⇒ 不进 F2；那棵树是每次发布都重写的树或活头的 inode 树 ⇒ 可写挂载拒、只读挂载照开（推的，零轮） | 恢复、只读落 R（那棵树 EIO）；可写拒 | 同左 | 同左 |

第三行仍有 V2-乙 那一格（X 不是核心树时可写挂载照开，X 的单元不在读集里）；要连它一起收，得加 V2-乙 ④ 的第一种写法。推的。

### V2-丁　与根环那一簇已定 F2 改法的交集

C755（并集只判退）：并集里收不收「单元读得出、条目解不开」没写。今天的三处退回分支接住任何失败（`crates/singlefs-core/src/recovery.rs:7116`：「Err(failure) => {」、`crates/singlefs-core/src/mounted_read.rs:655`：「Err(failure_of_the_selected_version) => {」、`crates/singlefs-core/src/mount.rs:8016`：「Err(failure_of_the_selected_version) => {」）。C755 的实现若把并集写成「三样读里任一处失败」，V2-甲 那四样照样触发退回，flags 改法要在同三处再改一次；若写成「三样读到的单元里有读不出的」，解不开的条目就不在并集里，没见证那两种盘面由「静默退」变成「照报失败」，这正是 V2-丙 表第三行要的落点，而正文的改法（对象级 EIO、照常落 R）就和它冲突。两件改的是同三处分支：要么同一批做，要么先定并集的成员。C751 甲（见证到的那一版与所选根共用实例表指针才收同实例的根）判的是退到哪一条根，不判触发，与这件不相交。推的（读码与条款）。

「退回分支不看失败种类要不要分种类」：要分，而且分的线不是「flags 对其余」，是「单元读不出」对「单元读得出、内容这个读者不认」。后一类不是 F2 要接的丢写（`crates/singlefs-core/src/recovery.rs:6891` 那句写的是单元读回全零），退回它只会把将来写者的合法发布当成丢写处理。

落点：第一轮正推说随 deadlist 合并，第一轮攻方说欠的是实现。今天的害处（静默丢一次已确认的发布）不靠 deadlist 合并才出现，它与 C755 的实现改同三处分支，所以随根环这一簇系统配置落地补丁之后的实现批做，与 C755 同批。要立的欠账一行：「树表条目里将来写者合法写下、今天的读者不认的字段（flags 未知位、预留非零、条目长度、根指针头部、同种类第二条）与单元读不出共用整表拒与 F2 退回」，检查是 V2-甲 那 6 格加 flags 3 格各一条用例，判别力自证用调查的两条 `minimal_` 用例。推的。

## V4　点删与墓碑

### V4-甲　并进 D8 已定项 13 不改字；要换的是检查的方向

D5 已定项 3 第 3 行，`.claude/kb/decisions/05-快照-空间记账机制.md:75`：「点删的墓碑不参加取最大——读一个 key 先看最新一条是不是墓碑」。D8 已定项 13 的定案已经逐字收了它的落叶那一半，`.claude/kb/decisions/08-核心索引结构.md:397`：「E 按同一次发布里的到达序取最后一次改动，不按值取」，同一行接着写「不许把墓碑当数值与 A–D 一起取最大」。所以「缩成并进 13」在条文上是删 D5 那一句、留一个指路，不改 13 的字。第 3 行的读那一半（「读一个 key 先看最新一条」）在 D8 已定项 12「ε 恒 0、发布前排空到叶」之下没有盘上对象：前端不落盘，盘上没有墓碑条目可读。推的（读条款）。

要换的是检查。今天写下的两条检查都是同一个方向：
- C383 的检查，`.claude/kb/checks-owed.md:277`：「运行时断言：一次发布里每棵树每个 key 进叶至多一次，进的是到达序最后那一次」。
- D8 已定项 12 的检查，`.claude/kb/decisions/08-核心索引结构.md:385`：「一个 checkpoint 里前端树叶上的每一处逻辑改动（key → 值）都对得上一条这个 checkpoint 进过叶的前端条目」。

两条都从叶上的改动出发去找前端条目（叶 ⊆ 前端）。第一轮攻方 `out/w4.out` 里朴素取最大那一形（`naive_max … rows_final=35 stale_rows_final=17`）坏在反方向：前端里有上一代水位那一行的墓碑，朴素取最大把它吞掉，叶上那一行原样不动——叶上没有逻辑改动，第二条无从对；那个 key 一次都没「进叶」，第一条的「至多一次」「进的是最后那一次」都空真。两条在这一形上都绿。C344 自己写的检查，`.claude/kb/checks-owed.md:252`：「读一个 key 时最新一条是墓碑就判死、被点删的那一代读不到」，是读路径的检查，在 ε 恒 0 的设计里没有读路径会同时看到墓碑与叶。

会红的形是反方向的一句：「排空之后，前端里每个 key 到达序最后那一条都在叶上兑现：是值就等于它，是墓碑就那个 key 不在叶上」。它与 13 的定案同义，只是从前端条目出发；判别力自证照 C344 原文：退回朴素取最大，同一批用例由绿转红。推的、被攻过零轮（我没把它写进模型）。第一轮攻方提的两条 checker 侧的形（按 D5 意义认台阶、相邻两级台阶 H 严格递增）只抓水位那一个统计量；这一句对三棵前端树的每一个 key 都成立。

打中之后四句：① 分辨：分辨（`naive_max` 对 `arrival`：17 行陈旧对 0 行，第一轮模型量过）。② 系统看不看得到：看得到——排空时墓碑就在前端里。③ 满足的分句：两条现有检查的方向（叶 → 前端）。④ 改法：「缩进 13、检查沿用 C383 那一句」在打中的那一形上不起作用；换成反方向那一句才起作用。

### V4-乙　删头时台阶行的候选与谱系记录

正文要写出删一个可写头时它的台阶行怎么离开的候选。第一轮攻方给了两句（零轮）：重建落法「删头那次不照抄它的台阶行」，点删落法「以墓碑离开、按到达序折叠」。两句都与 D18 已定项 24 第 5 条碰：谱系记录 `.claude/kb/decisions/18-块里携带什么信息.md:582`：「随现行那一版、只增不删」，并与台阶行互核：「每条谱系记录上，台阶行里代段 ≤ T_rb − 1 的那几行的最大水位**等于** H_cur，H_cur 为 0 也判红」，同一行「有克隆头时这一判与台阶行都按头（带树 ID）」。

历史：建克隆头 C → 在 C 里建 inode（C 长台阶）→ 对 C 做一次管理员回退（追加一条按头的谱系记录，H_cur 取 C 那一刻的水位）→ 删 C。两条候选在删 C 之后都让 C 的台阶行离开，谱系记录照「只增不删」留着；I-9.30 按头判那条记录时，C 的台阶行是空集，最大水位不等于 H_cur ⇒ 红。合法历史上误红。候选要连着写「被删的头的谱系记录怎么办」（随头删，等于改 D18 的「只增不删」；或 I-9.30 只判活头，等于改不变量的射程）。第一轮的 `per_tree_all`（台阶行永留）反而是三者里唯一不撞这一条的。推的：克隆头的管理员回退今天没有实现（`mount::roll_back_by_a_forward_publish` 遇到树 ID 集合不同就拒，TODO 原话），按头的谱系字节形态也没定（D18:582「按头之后谱系记录的字节形态没定」那半句）；这条历史要等那两样定了才可达，我写的是条款层的冲突。

### V4-丙　与 `crates/singlefs-core/src/transaction.rs:3682` 的 TODO 是不是同一件事

不是。TODO 原句 `crates/singlefs-core/src/transaction.rs:3683`：「做克隆时按树段分开取、建克隆那次给新头留一行。」——它管建克隆、只点名 `inode_number_watermark_steps` 这一个函数。删头要先能答「哪几行台阶属于被删的那个头」，这要分树；而今天不分树的不止这一处：
- 取水位，`crates/singlefs-core/src/transaction.rs:3668`：「.max_by_key(|entry| entry.generation)」——过滤只看统计量、不看树，两个头的「这一代的行」代段相同，取到哪一个头的值看次序。
- 恢复的条数判，`crates/singlefs-core/src/recovery.rs:5720`：「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < root_txg」——每多一个可写头，台阶之外就多一行「这一代的行」，`3 + 6 × 盘数` 那一判就红（同一函数）。
- checker 的 I-3.13 语法认法（`crates/singlefs-checker/src/walk.rs:2649` 起）与 `inode_number_watermark_steps_of_a_version`（`crates/singlefs-checker/src/walk.rs:13039`），E186 登记第三节点名过「同样不按树分」。

所以删头那一问是「分树」之后的下一件事：TODO 是它的前提之一，TODO 本身没罩到删头、也没罩到上面另三处。量过的只是这几处码事实（grep 现取的行）；「两个头时取到哪一个」「条数判红」是读码推的，crates 今天造不出两个头。

### V4-丁　落点

照欠账表的落点写法：
- 墓碑那一半（V4-甲的反方向检查）随里程碑八 一·1（write buffer 前端落地，`.claude/kb/milestone/08-eighth-txn.md:39`），它的验收「排空漏一条的变异判红」是同一族；C344 并进 C383 时，C383 的检查句换成反方向那一句。
- 删头那一半随里程碑十四 二·2（销毁头，`.claude/kb/milestone/14-fourteenth-txn.md:50`），前提是建克隆那一节的分树（V4-丙 那四处一起）；I-9.30 与谱系「只增不删」的射程同批定（V4-乙）。
推的（读里程碑文件，没核作业点的依赖图）。

## V3　树高与代价曲线的改后条文：漏了一个变量

第一轮判决让第二轮以第一轮攻方 `out/w3.out` 为准写树高与代价曲线的改后条文。`w3.out` 用的行集是 `rows=3+6d+1`（一级台阶），变量只有盘数 d 与 key 宽。台阶行条数是第三个变量：它随建过 inode 的发布数一直涨，D5 已定项 2 射程自己写了门槛（`.claude/kb/decisions/05-快照-空间记账机制.md:55`：「台阶行到第 462 行为止是 0」）。副本规划器（`out/planner-carry-cost.out` 的 `height_in_r` 列，crates 的 22 字节 key、叶 477）：两块盘、认识的头 1 级台阶、两个不认识的头各 1 级时 `rows_in_r=20 height_in_r=1`；同样两块盘、认识的头 1000 级台阶时 `rows_in_r=1019 height_in_r=2`；64 块盘、1 级台阶时 `rows_in_r=392 height_in_r=1`。所以「段宽不动树高」「窄读 1、宽读 2」「80 块盘起才分裂」这几类句子，只要不带「台阶行条数」，在一个用了一阵的两块盘池上就错。改后条文要么写成「记账树的行数 = 3 + 6d + 每个可写头的台阶行数之和（+ 各头这一代那一行），树高与每次发布重写的叶数随它走」，要么只说取样点（t、d、台阶数）上量到的数、不说全称。推的部分：只在 crates 的 22 字节 key 上量了，E97 那四档 key 宽没有在台阶数这一维上量过。

打中之后四句：① 不判臂。② 不适用。③ 满足的分句：依据句与射程里的全称（「恒」「起才」）。④ 改法：第一轮攻方给的「一页叶装得下时段宽 0 代价；超过一页叶，写出的叶数随条目宽走」在方向上对，但「一页叶装不装得下」由台阶行条数决定，句子要把它点出来。

## 几个改法各修哪一格

全部是我提的（V1-乙 的「按统计量」臂是第一轮攻方提的、这一轮我派生量过），**只在我的模型上量过、被攻过零轮**。「量过」= 模型 `out/` 下有原样输出；「推的」= 按代码与条款推、没实现没跑。

| 改法 | 修哪一格 | 在打中的格上 | 标 |
|---|---|---|---|
| 第二次跑的判据里 U5 照 B1 的理由排掉（或另立一次跑，B1 的射程句写明「连同点名 ㊀ 的别的条款」） | V1-甲 | `legal_pooled` 由 `unique_carry` 变 `depends_on_point_mixed`，按登记的形态表交用户 | 量过（`out/e186-second-run-reread.out` 第 8–10 行） |
| 改走 B2（㊀ 与点名它的句子都计） | V1-甲 | `Carry` 站住（第一次跑产物第 186067 行；与第二次跑同向） | 量过（第一轮），但改的是主 agent 跑前定的读法，要交用户 |
| 臂集补「按统计量」一条，在入库装置上另跑 | V1-乙 | 照第二次跑判据原样，`r_tree_dim = 2` 组 `PerStatistic` 独胜，`legal_pooled` 混合 | 量过（派生，第 5–7 行）；入库装置上没跑 |
| ㊀ 射程的代价句写成带条件的「台阶行与别的行共叶时 0 个节点；每个不认识的头台阶自成一片叶以上时每次发布多重写 h_new 片叶」 | V1-丙 | 32 格都对 | 量过（`out/planner-carry-cost.out`，副本） |
| 只改 `records.rs:753` 那一支 | V2-甲 | 预留、长度 6 格照旧静默退 | 量过（今天的结局，副本） |
| 凡「单元自证过、条目这个读者不认」的未来字段（flags、预留、长度、根指针头部）一律不进 F2 | V2-甲、V2-丁 | 6 + 3 格不再静默退 | 推的 |
| 上一条再加：那棵树是每次发布都重写的树或活头的 inode 树 ⇒ 可写挂载拒、只读挂载照开 | V2-丙 | 见证、没见证三种盘面都落 R，只读照开、可写拒 | 推的 |
| 带不认识的字段、又没见证的那一版：三处照报失败（只读挂载除外），不落它也不静默退 | V2-乙 | X 的单元读不出那一形不再提交在 R 上 | 推的 |
| C755 的并集写明只收「单元读不出」 | V2-丁 | 与上两条落在同三处分支，同批做 | 推的 |
| C383 的检查换成反方向：前端每个 key 的最后一条到达在叶上兑现，墓碑 ⇒ key 不在 | V4-甲 | 第一轮 `naive_max` 那 17 行陈旧要靠它红 | 推的（判别力自证照 C344 原文） |
| 删头候选连着写被删头的谱系记录去向（随头删，或 I-9.30 只判活头） | V4-乙 | 「克隆、回退、删克隆」那段历史不误红 | 推的 |
| 分树的改动面写全（水位取值、恢复条数判、checker 两处、台阶照抄） | V4-丙 | 删头的前提 | 码事实量过（grep），后果推的 |
| 树高句带上「台阶行条数」这一变量 | V3 | 两块盘、1000 级台阶树高 2 那一格 | 量过（副本规划器，一种 key 宽） |

## 没打中的形状

- **V1：别的 U 条在新 U5 规则下悬空不同。** U1–U13 逐条读过，只有 U5（与 B2 下的 U12）在三条臂下悬空不同（V1-丁）。
- **V1：`r_tree_dim = 1` 上不靠 U5 打败 `NoCarry` 的形态。** `PerStatistic` 在 `r_tree_dim = 1` 上与 `NoCarry` 全等（派生定义如此）；我没找到第二种在 `r_tree_dim = 1` 上第 7 位不输 `NoCarry`、第 6 位又不为假的写法。
- **V1：几何、`G`、`k_keep`、`w_gens`。** 第二次跑的敏感性行已量；我的重判在 11520 点全取，没另扫。
- **V2：C751 甲。** 它判的是 F2 收哪一条根（看实例表指针），与「什么触发 F2」不相交；我没造出两者交叉的盘面。
- **V2：见证了 R 那一形。** 三样未来字段在见证了 R 时都三处拒、盘不变，与 flags 位同；那一格改法前后都不丢数据，只差可用性。
- **V4：两条落法（整棵重建对点删 + 到达序折叠）的等价。** 第一轮量过 20 版逐行相同，我没再造历史去打它。
- **V4：相邻台阶 H 严格递增那条检查（第一轮零轮）。** 我找合法历史上相邻两级 H 相等的情形：水位只增、回退不让水位跳升（D5 已定项 2 射程「回退不让水位跳升，所以台阶行只在建 inode 与建克隆的发布写」），崩溃回退落到旧根时台阶随旧根一起回去，同一条根里的台阶序列仍严格增。没造出误红。推的。
- **崩溃点。** 这一轮的打中都不靠枚举崩溃状态：V1 是入库产物上的重判与规划器算术，V2 用的是调查的三种盘面（一个崩溃面、两个读回全零面），没调 `enumerate_crash_states`；V2-乙 那条历史需要「根槽落了、轮换与 X 的一个单元读回全零」，我没在副本里实现对象级 EIO，没造这一形。

## 这条腿自己的限度

- V1 的数是我在两份入库产物的格行上重判的（自证：「U5 不计」读法逐格等于第一次跑，弄坏开关下 38400 格不等、退 3），不是入库装置的一次跑；`PerStatistic` 是派生臂。要引得在入库装置上加一种判据、加一条臂另跑（照 `.claude/rules/three-way-inference.md` 判决那一条）。
- V1-甲 的打中是「判据不自洽」，不是「数错了」：它不说 B1 对还是 B2 对（第一轮攻过），只说第二次跑的组合两头不靠。
- V1-丙、V3 的数出自副本里 crates 的规划器，行形是我照 `accounting_entries_of_this_publish` 手搭的（池级三行、每盘六行的统计量标签照 `records.rs` 的常量取），不是从 crates 的发布路径录的；只量了 22 字节 key 一种宽度、旧写者不建 inode 一种历史。
- V2-甲 是副本里 harness 档的 12 条用例，沿用调查的搭建（只改「改哪个字节」）；只造了 inode 树条目，没造别的种类、别的位；条目长度那一格只改了长度字段、条目本身没加宽。
- V2-乙、V2-丙、V2-丁 与 V4 全部是读码与读条款推的，没实现对象级 EIO、没造「X 的单元读不出」的盘面、没有克隆头。
- 读了：正文、背景材料里 E186 实验页与 D8 已定项 8、12、13、D5 已定项 2 那几段，第一轮判决与两腿报告里攻方那一份，E186 跑前登记第 1–40、295–658、660–989 行，调查报告与它的测试文件，根环第三轮判决与正文第 15 行，`checks-owed.md` 的 C71、C344、C383、C693、C751、C755、C160 几行，里程碑八、十四的相关几行，crates 里引到的那几段。没读：第一轮正推报告正文（只看了第一轮判决对它的转述）、英文与日文版、这一轮正推腿的产物（禁读）。

## 要主 agent 跑的命令

无重型、无轻型。`tools/rerun.sh` 是本腿模型的复跑入口（报告开头那一条）；判决引 V1 的数时要在入库装置上另跑，引 V2-甲 的数时要把那一条测试目标做成入库用例，这两样由主 agent 定要不要立。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没碰 crates、kb、入库装置与入库产物。副本上的数不算入库装置上的数。
- 没跑任何重型测试；harness 档的两条测试目标只在我草稿目录的仓副本里跑（经 1G 内存包装，`RUN_WITH_MEMORY_CAP_WAIT_SECONDS=21600`），Python 重判经 1G 包装（先以 4G、2G 起过两次，都排在共用内存额度里，我停掉了等待中的准入进程，那两次的退出码 251 是被停，不是结果）；`tools/rerun.sh` 先以 4G 起过一次，同样排队被我停掉（没起跑），之后以 2G 跑通（上面报告开头）。
- 没调 `enumerate_crash_states`：这一轮的打中都不是从崩溃状态枚举里来的，没有缩崩溃状态。
- 草稿目录里两份仓副本（`repo/` 与 `rerun2/repo/`，连同编译目录各约 1.6G）交回前已删；留下的只有日志、重判输出与 `probe-summary.awk`，都没入库（数已进模型目录 `out/`）。

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m3-close-c693-r2-opus-output.md --root . --background research/prompts/_m3-close-c693-r2-background.md` 末行（这一节写进去之后跑的；这一节里那个 `records.rs` 行号也被数成一处裸行号）：

```
  ✓ 核了 32 处引文、15 处裸行号，对上 32 处，没判 15 处（其中裸行号没有引文可比 15 处；逐处列在上面）
```

没判的 15 处都是裸行号（产物与登记的行号、`walk.rs` 两处、里程碑两处、`records.rs:753` 复指），原文在报告别处带引文核过或在那一行旁边贴了原样输出。
