# e171-conclusion-r2 辩方腿报告（Sonnet，辩方）

轮名 e171-conclusion-r2（第二轮），格 H1–H3，立场辩方：替被第一轮改判出局的一方找它站得住的理由，判第一轮的改判够不够得着、同一份 kb 里有没有另一句把它翻回去、是不是同样打中了没被改判的行。日期 2026-10-03。
引 kb 条款写 kb 文件自己的行号（逐条 `grep -nF` 过，命中 0 次的不写成引文）；代码引文写冻结副本 `/tmp/claude-1000/e171-conclusion-r1/tree/` 下的路径与行号（cite-check 用 `--root` 指它）。开工快照 `sha256sum -c --quiet research/prompts/e171-conclusion-r2-snapshot/kb-sha256.txt` 退出码 0。
脚本与产物在 `research/prompts/e171-conclusion-r2-sonnet-model/`（`recovery_side.py`、`older_vs_red.py` 与各自的 `.out`，只读导出目录）。被复核的判决是 `research/prompts/e171-conclusion-r1-main-verification.md`，下文「判决:行号」指它的行。

## 0. 判定一览（照攻击面 R1–R5、R7 与格 H2）

判定三种：站住（第一轮改判没被辩回）/ 够不着（改判依的条款句被同一份 kb 的别的句子翻回、或读窄了）/ 同样打中别的臂（同一读法落到没被改判的行上，结论相反或自相矛盾）。一格可以同时有两种。

| 判决条目 | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| R1 第 242、270 行：「规约允许的集合」取已定项 12 而不取已定项 7 的写序（判决:26、:28） | 够不着；同样打中别的臂 | 已定项 12 的原文只管「根 + journal 前缀」，不含系统配置槽；D13 自己把合法崩溃状态定义成「合法写侧写出的历史的任一落盘子集」；invariants 的 I-7.12 用写序定义「合法状态」，按第一轮读法 O2 在第 102、265 行上的红成了误红。第 2 节 |
| R2 第 23 行归「取样点（历史）不敏感」（判决:23） | 标签够不着，读数作注解站住 | 页内三分定义（E171 页:82）里第 23 行取到的态就含条款不许的态，是真盲区候选；多建 233 个 inode 的数在副本上量，没在入库装置上重做。第 3 节 |
| R2 第 242、269 行同归「取样点（历史）不敏感」（判决:26、:27） | 站不住；同样打中别的臂 | 分得开的是「实现交回结局与基线不同」，没有任何现有 oracle 对它判红，与第 42 行（病根在 O1 缺答，第一轮留作真盲区）同形却分别对待。第 3 节 |
| R3 第 15 行由「对这一批等价」改真盲区候选（判决:36） | 站住；kb 内部两句相反 | D3 已定项 10 ⑤ 的落点表与 layout 写清单把 t1 钉在 50180；但 D3 已定项 8 射程说政策违反写出的镜像完全合法。第 4 节 |
| R3 第 34 行同改（判决:36） | 够不着 | D3 已定项 8 射程（03-空间分配.md:171）：分配器政策不是格式位、违反它的实现写出的镜像完全合法、只能靠运行时读数；第一轮材料里没有这一项。第 4 节 |
| R3 与 R1 的合起来看 | 同样打中别的臂 | 第一轮把写序义务降为「义务不是状态」、又把分配政策升为「状态非法」，kb 的写法恰好相反（I-7.12 对 D3:171）。第 4 节 |
| R4 第 27、277 行归「实现恢复违反 M4 字面」（判决:37） | 站住，条款比第一轮引的更硬 | M4 等号不带「已确认」，E165 把已确认放在 KJ1、最远一版放在 J4；D23 已定项 15 与已定项 14 直接管「记录在、根槽不在」。第 5 节 |
| R4 第 112 行归「违反 D23 注 1」（判决:38） | 站住 | 注 1 逐字点名所选根是 mkfs 第 0 代根的这一态。第 5 节 |
| R4 对没改判的行 | 未打中其他行 | 我数了 281 行里全部恢复侧有差的行：全放行的只有 27、112、277；只靠一条的 245、281 的「恢复比基线旧」的态各自都有列红（24 / 24、48 / 48），不是碰巧。第 5 节 |
| R5 第 92 行里程碑验收句不算条款（判决:25） | 理由读窄了，目的地部分站住 | 口径句的冒号后面限定的是预想的细节；验收标准一条就写「写入时间留成 A 的 ⇒ 判红」；layout 字节表已定的一行写「写入时刻」。决策正文零命中仍成立。第 6 节 |
| H2 写序义务今天归谁判 | 第一轮「无人判」不准 | 两条流的段序列由 checker 档用例钉死、登记表由门禁 doc-registries 比对；三条 oracle 与 RefFS 按设计不判。第 7 节 |
| H2 「只让 O3 红」那一形 | RefFS 判不到 | RefFS 的输入是恢复读出的内容，暖机不是 RefFS 的操作。第 7 节 |
| H3 四条文字改法 | 第 1 条写窄、第 2 条写宽、第 3 条一半放错格、第 4 条证据选错 | 第 8 节 |
| H3 C407 两种改写 | 钉四列矩阵与现有 replay 重叠、字面更合的是只钉全放行集合 | 第 9 节 |

## 1. 我跑的东西（只读导出目录，没起装置、没编译）

`recovery_side.py` 对 281 行逐行、逐流，把实现「看 journal 那一遍」恢复到的根与基线在同一个落地向量下比（按 states.tsv 的落地向量对号，写流变了而对不上号的标 stream_changed），数比基线旧 / 新 / 同根不同内容 / 走读失败的态数；`older_vs_red.py` 对比基线旧的态数它们之中 O1-崩溃判官红、O2 红、两者都不红各多少。命令与原样输出：

```
$ nice -n 19 python3 research/prompts/e171-conclusion-r2-sonnet-model/recovery_side.py research/target/e171-dumps research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out
row category red_cols(batch1) write_changed recovery_report_differs | per-stream older/newer/other/failed/diffstates | stream_changed
27 counted - false true | fixed:51/0/0/0 new_p:6/0/0/0
108 counted 1 true false | fixed:stream_changed(87/87unmatched) new_p:0/0/0/12
112 counted - false true | fixed:0/3/0/0 new_p:0/3/0/0
115 counted 1C true false | fixed:stream_changed(87/87unmatched) new_p:0/0/0/39
233 counted C false true | fixed:0/0/0/48 new_p:0/0/0/3
237 counted 1C true false | fixed:stream_changed(87/87unmatched) new_p:0/0/0/60
244 counted 12 true false | fixed:stream_changed(87/87unmatched) new_p:0/0/0/9
245 counted 2 true false | fixed:24/0/0/0 new_p:0/0/0/0
277 counted - false true | fixed:51/0/0/0 new_p:6/0/0/0
281 counted 1C true false | fixed:stream_changed(87/87unmatched) new_p:48/0/0/0
$ nice -n 19 python3 research/prompts/e171-conclusion-r2-sonnet-model/older_vs_red.py research/target/e171-dumps 27 277 245 281
row=27 stream=fixed older=51 of_which_crash_judge_red=0 o2_red=0 neither=51
row=27 stream=new_p older=6 of_which_crash_judge_red=0 o2_red=0 neither=6
row=277 stream=fixed older=51 of_which_crash_judge_red=0 o2_red=0 neither=51
row=277 stream=new_p older=6 of_which_crash_judge_red=0 o2_red=0 neither=6
row=245 stream=fixed older=24 of_which_crash_judge_red=0 o2_red=24 neither=0
row=245 stream=new_p older=0 of_which_crash_judge_red=0 o2_red=0 neither=0
row=281 stream=fixed older=0 of_which_crash_judge_red=0 o2_red=0 neither=0
row=281 stream=new_p older=48 of_which_crash_judge_red=48 o2_red=0 neither=0
```

读法：`fixed` 是流二（固定脚本，15 次调用）、`new_p` 是流一（新池新建文件）；`red_cols` 里 1 是 O1-对拍、C 是 O1-崩溃判官、2 是 O2、3 是 O3 甲，只取批 1。第 27、277 行 `new_p` 6、`fixed` 51 与实验页 E171 页:88、:172 写的 6 / 51 同数，算校准。
限度：①只覆盖导出目录里批 1 的态；②流二在第 4 次调用（可写挂载）报错而停的行（108、115、237、244、281）流二与基线的落地向量对不上号，`stream_changed(87/87unmatched)`，流二没比；③写流变了的行（245、281 等）里「同一落地向量下基线的恢复」不一定就是该行自己的 j*，只当比较的参照；④态数是我从导出里数的，没起入库装置。

## 2. R1：第 242、270 行取已定项 12、不取已定项 7（判决:26、:28）

**判决怎么说**：判决:26 判「分歧在规约允许的集合指哪一条」取攻方的读法，判决:28 写「写序义务今天三条都不判」后归「不算三条的共同盲区，算写序义务无人判」。

**辩方判定：够不着。** 三处。

① **已定项 12 的原文管不到这一态。** 它的定案是把 D8 的中间状态句改写成「根 + journal 任意前缀，replay 之后合法」（`.claude/kb/decisions/16-发布语义.md:264`），整节只有定案、射程、依据三段；射程只写「定的是合法性怎么判，不定前缀本身怎么截」（`.claude/kb/decisions/16-发布语义.md:266`）。公式里的对象是根与 journal 前缀两样，系统配置槽不在里面。第 242 行被写序挪动的正是系统配置槽（页:168 写「系统配置先于它见证的根落盘的态，照条款的写序造不出来，三条都放行」，`.claude/kb/experiments/171-三oracle合谋测试.md:168`）。公式没有提到的东西，不能反过来把一个态判成合法；这个槽的先后只有已定项 7 说了：「系统配置槽在根槽持久之后再更新，每个 checkpoint 一次」（`.claude/kb/decisions/16-发布语义.md:181`）。

② **同一份 kb 里另有四句把合法崩溃状态定义在写序上**：
- D13 把 M1–M4 的前提域定成「合法写侧写出的历史的任一落盘子集」（`.claude/kb/decisions/13-验证路线.md:55`），M2 语义层的前提是「盘上每条记录出自这一版的合法写侧」（`.claude/kb/decisions/13-验证路线.md:32`）。写序不对的写侧写出的态落在这个域之外，不是域里的合法态。
- 已定项 7 自己写「一次发布的段序列因此唯一」（`.claude/kb/decisions/16-发布语义.md:181`）：合法崩溃状态的集合由那一条唯一的段序列定。
- invariants 的 I-7.12 正文：「所以任何合法状态上一条根带的 F 都已经在它所在那块盘的系统配置里」（`.claude/kb/invariants.md:66`），同一行写「层 0 每个崩溃状态都跑池级 checker」（`.claude/kb/invariants.md:66`）。这是 kb 自己用写序来定义「合法状态」，对象还是同一对关系（系统配置槽先于根）的另一个字段；O2 在单镜像上判它。
- D13 给 O3 写的判什么是「崩溃点截断后的每个磁盘状态是否落在规约允许的集合里」（`.claude/kb/decisions/13-验证路线.md:163`）：被判的对象是磁盘状态，不是 replay 之后落到哪一版。第一轮把集合取成 replay 结果，描述的是 E170 执行器今天的做法，不是这一句的字面。

③ **E171 页内的三分没有伤害判据。** 真盲区的定义是「取到的态里有条款不许的，三条都放行」（`.claude/kb/experiments/171-三oracle合谋测试.md:82`）；页:484 记着第 242、270、265 行同样以已定项 7 的持久顺序当状态合不合法的判据（`.claude/kb/experiments/171-三oracle合谋测试.md:484`）。第一轮的读法等于给三分加了一条「伤害要落在登记域内」，那是另一个维度（伤害域），不是「态合不合法」。第 270 行另有 C577 把性质钉成崩溃状态上的谓词：「发布返回后每个崩溃点见证值 ≥ 末条记录计数器」（`.claude/kb/checks-owed.md:582`），D16 写用户加屏障的起因是那条用例在更早红（「那条用例在抬 F 之前更早红：发布路径根槽 FUA 之后轮换系统配置就返回，没有屏障」，`.claude/kb/decisions/16-发布语义.md:186`）：用户加屏障时认的就是这个崩溃状态谓词。

**同样打中别的臂**：
- 第 102、265 行（抬 F 先写系统配置那一步改了写序，态在 replay 之后同样合法）O2 在批 1 红，红的是 I-7.12：「name=row_matrix row=265 batch=1 O1=quiet O1-对拍=quiet O1-崩溃判官=quiet O2=red」（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:4178`）、第 102 行同（`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:1645`）。若「合法」只看 replay 结果，这两行上 O2 的红是误红；第一轮没有这样判（它们在 Q3 里算 O2 抓到）。第一轮的读法与 O2 在这两行的红不能同时成立。
- 执行员的用法本来分得清：恢复结果用已定项 12（页:485 与第 277 行），状态用已定项 7（页:484）。第一轮把 12 越界用到了状态上。
- 与 R3 方向相反：见第 4 节末。

**第一轮对的部分，留作注解**：第 270 行的伤害要读故障、纯崩溃域里没有可见害处（攻方副本上 87 态 0 态因见证值拒，我没重跑）；第 242 行在崩溃后可写挂载上 9 / 276 态拒（副本数，我没重跑）。这些是「伤害域」一列，写进判决的注释就够，不改「态合不合法」。
**什么现象会推翻我**：找到一句 kb 条款写明合法崩溃状态就是「replay 之后落到合法版」的任意盘面、与写序无关；或判决把 O2 在第 102、265 行的 I-7.12 红改判成误红（那样 I-7.12 条文要改）。

## 3. R2：「取样点（历史）不敏感」（判决:23、:26、:27）

**第 23 行：标签够不着，读数作注解站住。** 页内定义：「取样点不敏感＝两条流里有条款不许的态，取样规则没取到」、「真盲区＝取到的态里有条款不许的，三条都放行」（`.claude/kb/experiments/171-三oracle合谋测试.md:82`）。第 23 行批内就有条款不许的态（「择到第一个文件那一版（txg 3）及之后：流一批 1 12 态」，`.claude/kb/experiments/171-三oracle合谋测试.md:87`），判决:23 自己写「这一批的态非法且三条放行（Q1 计它）」。按页内定义它就是真盲区候选；第一轮加的「历史」是第二个维度（在同一个坏实现上再接一步用户动作），不在登记里也不在页内三分里。mutation-sampling 把「换一个取样点就同值不再」归第三类，处置是补一个敏感的取样点而不是改分类；判决:23 同一格也只写「补的是历史取样（跨一片叶），不是 oracle」。所以读数（多建 233 个 inode，O2 现有的 I-1.1、I-9.4、I-9.12 红）作注解可以收，不当三分的改判。
证据来源另有一条：233 个 inode 的数是攻方在副本里把第 23 行变异打进 core 之后量的，主 agent 复跑用的是攻方的脚本；规则写「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」（`.claude/rules/three-way-inference.md:115`），这一步没做。

**第 242、269 行：站不住。** 「分得开」在这两行上指的是「实现交回的结局（可写挂载拒或成）与基线不同」，没有任何现有 oracle 对这个结局判红：攻方自己的数是 O2 在这些态上 0 红，O1 没有「崩溃态上可写挂载」的答，判决:27 列的改法就是「补 O1 历史」，攻方限度第 3 条还写明基线在撕裂系统配置槽的态上就拒可写、模型的答要先与 C331 的取舍对齐。这与第 42 行同形：历史再走一步就看得到交回值不同，缺的是 O1 的答；第一轮对第 42 行写「病根在 O1 一列」（`research/prompts/e171-conclusion-r1-main-verification.md:24`）留作真盲区候选，对 242、269 却记「取样点（历史）不敏感」。两者唯一的区别是第 42 行的卸载调用在登记流里、崩溃后可写挂载不在（页:461 写「没有分裂、没有删除、没有空间紧的池」一类，`.claude/kb/experiments/171-三oracle合谋测试.md:461`）。这个区别成立，但说明的是登记域，不是取样点：补了取样点也没有 oracle 红，处置不是「补一个敏感的取样点」。更合的标签是「登记域外 + O1 缺答」。
第 269 行还有一层：判决:27 写「态在已定项 12 口径下 replay 后合法」，与 R1 同病；D23 写「mkfs 写 tail 0，mkfs 之后第一次取号读到 0、写 0：新池新建文件的字节不变」（`.claude/kb/decisions/23-journal的角色与格式.md:392`），页里这一行的行名也写「新池新建文件的字节变了」（`.claude/kb/experiments/171-三oracle合谋测试.md:153`）——是字节契约，态本身不合法，与 replay 之后落到哪无关。

**同样打中别的臂**：第 15、34 行页内也写了「会分开的历史」（空间只够复用的池，O1-对拍会红；页:89、页:461 写「推的、没跑」，`.claude/kb/experiments/171-三oracle合谋测试.md:461`）。第一轮没有把它们也挪过去，一致的理由只能是「量过才算」；这条标准对 242、269 的「模型会答该成」（推的）同样适用，而它们被挪了。
**什么现象会推翻我**：在入库装置上重做崩溃后可写挂载，并且有一条现有 oracle 在 242、269 的历史上红（那样它们是真正的取样点不敏感）。

## 4. R3：第 15、34 行落点（判决:36）

**判决怎么说**：判决:36 把两行由「对这一批等价」挪到真盲区候选，依 D3 已定项 10 ③。

**第 15 行：站住。** 条款：③「数据单元取最低的、起点 32768 对齐、两槽都空、整跨度都过合取的起点」（`.claude/kb/decisions/03-空间分配.md:204`）；⑤ 的落点表把 extent 树根钉在 50240：「| t2 | extent 树根兼叶（码 2，16 KiB） | 11 | 50240 | 0 |」（`.claude/kb/decisions/03-空间分配.md:210`）；layout 写清单把 t1 钉在 50180–50181（`.claude/kb/layout/01-first-txn.md:67`），落点政策那一行写「新池新建文件 t1@50180」（`.claude/kb/layout/01-first-txn.md:309`）。第 15 行的变异把 t1 落到 50240、后面七块整体后移 64 槽，与这两张表逐字不符，是字节表契约，不依赖分配器政策，所以下面的 D3:171 对它不是翻回。执行员页里的 grep 只找「起扫」「单元区起点」，漏了这些（页:86）。
一处要让主 agent 知道：D3 已定项 10 写「改新池新建文件的字节：是」（`.claude/kb/decisions/03-空间分配.md:220`），已定项 8 写「改新池新建文件的字节：否」并说明「新池新建文件写出的字段与宽度一个都不变，变的只是分配记录条目里那个落点的取值」（`.claude/kb/decisions/03-空间分配.md:171`）。两句管的东西不同（⑤ 钉七个提交内生块的落点，已定项 8 管用户数据的取值），不矛盾，但第 15 行同时动了两样。

**第 34 行：够不着。** 已定项 8 的射程：「与已定项 5 同一类——盘上没有字段声明它，违反它的实现写出的镜像完全合法」，「拦它只能靠运行时读数」（都在 `.claude/kb/decisions/03-空间分配.md:171`）。用户数据落点的政策正文在该项第 1 条：「起点槽号最小、起点 32768 对齐、跨度内两槽都空、不在任何开放的聚簇段里」（`.claude/kb/decisions/03-空间分配.md:163`），③ 是同一条政策在已定项 10 里的重复。第 34 行没有 ⑤ 那样的表可依（抬 F 之后的发布不在新池表里），条款层只有这条政策，加上 D16 对可再分配的许可条件「已释放 ∧ 释放代 ≤ max(F_生效, 一圈窗口里最旧有效根)」（`.claude/kb/decisions/16-发布语义.md:33`）——那是许可，不是镜像上的状态。
按 kb 自己的话，第 34 行的态本身合法，错在运行时：空间只够复用扣住那几槽的池报假性 ENOSPC（D3 已定项 9 的题目就是不许有假性 ENOSPC）。这正是执行员写的「落点不是条款定的值」加「会分开的历史」（`.claude/kb/experiments/171-三oracle合谋测试.md:89`）。第一轮两腿与判决都没引已定项 8：
```
$ grep -c '违反它的实现写出的镜像完全合法' research/prompts/_e171-conclusion-r1-background.md research/prompts/_e171-conclusion-r2-background.md
research/prompts/_e171-conclusion-r1-background.md:0
research/prompts/_e171-conclusion-r2-background.md:2
$ grep -c '用户数据块不进聚簇段' research/prompts/e171-conclusion-r1-main-verification.md research/prompts/e171-conclusion-r1-sonnet-output.md research/prompts/e171-conclusion-r1-opus-output.md
research/prompts/e171-conclusion-r1-main-verification.md:0
research/prompts/e171-conclusion-r1-sonnet-output.md:0
research/prompts/e171-conclusion-r1-opus-output.md:0
```
我量了一遍两份导出里盘 0 单元区开头的写（`data_unit_slots.py`，只读 `ops.tsv`）：基线在第 498 条 op（抬 F 之后那次发布）落 50176，第 34 行落 50196，其余各次相同（`research/prompts/e171-conclusion-r2-sonnet-model/data_unit_slots.out` 全文）。这与第一轮报的 50176 / 50196 同，说明变异确是不复用已可再分配的槽，不说明镜像非法。第一轮提的补法「O2 拿上一版的分配状态」（判决:36）与「拦它只能靠运行时读数」相冲：O2 是只判一个镜像的一元谓词。

**同样打中别的臂（R1 与 R3 方向相反）**：第一轮对写序（已定项 7，kb 在 I-7.12 里拿它定义合法状态）说「义务不是状态」，对分配政策（已定项 8，kb 自己写镜像完全合法）说「状态非法」——两处都与 kb 自己的写法相反。一致的读法只有两种：按 kb 的写法（写序 ⇒ 状态非法，政策 ⇒ 镜像合法），第 242、270 行留真盲区候选、第 34 行归「三条按设计都不判分配政策」；或按第一轮的「义务不是状态」一律读，第 34 行与 242、270 同归「义务无人判」。
**什么现象会推翻我**：已定项 8、10 之外有一句条款写明未复用已可再分配的槽是非法镜像；或第 34 行的变异在某段历史里让盘上字节违反 invariants 里的某一条。

## 5. R4：第 27、277、112 行（判决:37、:38）

**辩方判定：站住，我没找到翻回的句子，条款比第一轮引的更硬。**

**M4 的等号不带「已确认」。** 原文「返回的根恰是 j」（`.claude/kb/decisions/13-验证路线.md:34`）后面接的是「从所选根起每一版都完整在盘上……的最远一版」，没有已确认。D13:55 那句 M4 连同择根才排除已确认的完整发布被恢复丢掉，说的是 M4 加择根合起来的用途（前提写了不带故障的崩溃状态、链不跨实例边界、暖机，`.claude/kb/decisions/13-验证路线.md:55`）：较强的条款推出较弱的用途，用途句不收窄等号。落点那一边同样如此：E165 把已确认做成另一条判据 KJ1（「KJ1：落到的版本不早于崩溃前最后一次已确认的用户发布」，`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:430`），最远一版是 J4 单列在手写判据 J0–J4 里（「照条款文字手写的判据 J0–J4、J6」，`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:22`；「Q4 J4（最远一版）」，`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:167`）。M4 的落点自己也不带限定。

**更直接的条款（第一轮没引）**：D23 已定项 15 题为崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根：「施加一条记录 = 把所选根的这七个字段换成记录里的」（`.claude/kb/decisions/23-journal的角色与格式.md:450`）；已定项 14：「恢复只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录」（`.claude/kb/decisions/23-journal的角色与格式.md:357`）；依据句「忽略 journal 与查 journal 的恢复结果在一批状态上不同——journal 承重」（`.claude/kb/decisions/23-journal的角色与格式.md:456`）。第 27、277 行的 57 态「全是 call_returned=False its_root_slot_landed=False」（`.claude/kb/experiments/171-三oracle合谋测试.md:88`），就是「记录在、根槽不在」那一格；条款对它的要求与调用返没返回无关。所以即使 M4 被判成带「已确认」，也只收窄 M4，收不窄已定项 15。

**第 112 行**：D23 注 1 逐字点名这一态：「所选根是 mkfs 的第 0 代根时它一条记录都不覆盖，之后的记录全属于更新的实例 ⇒ 一条都不施加」（`.claude/kb/decisions/23-journal的角色与格式.md:369`）。页:92 判「暖机没返回，0,0 与 1,1 都在允许集合里」（`.claude/kb/experiments/171-三oracle合谋测试.md:92`）用的是 O1-崩溃判官那条弱判据：「③ 盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧」（`crates/singlefs-harness/src/model.rs:2614`）与 `if chosen < persisted {`（`crates/singlefs-harness/src/model.rs:2639`）。我能替执行员找到的唯一理由：RefFS 的允许集合含 j = lo（「j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi」，`.claude/kb/decisions/13-验证路线.md:104`），两版都读不到文件，POSIX 可见层等价。这条够不到注 1：注 1 是对恢复施加什么的条款。另：第 112 行是多施加（我的表 newer 3 + 3），不是 M4 的不少施加，依据只有注 1。

**三条按设计都不判**：同意判决:37 的归类。O1 行写「崩溃之后允许看到的集合归 RefFS」（`.claude/kb/decisions/13-验证路线.md:161`），RefFS 的集合含旧版 lo，所以 RefFS 按设计也不判 M4；落点是 E165 与 Verus。

**同样打中别的臂？没有打中。** 第 1 节的表是 281 行里全部恢复侧有差的行：全放行的只有 27、112、277（233 只在丙）。只靠一条的行里，第 245 行流二 24 个比基线旧的态全部 O2 红（24 / 24），第 281 行流一 48 个比基线旧的态全部 O1-崩溃判官红（48 / 48），没有「碰巧有一列红」的少施加态；108、115、233、237、244 的恢复差是走读失败（没有根），不是少施加；第 245 行的写侧反向链是错的，不在 M4 前提「盘上每条记录出自这一版的合法写侧」（`.claude/kb/decisions/13-验证路线.md:32`）之内。限度：流二在 108、115、237、244、281 上没比（第 1 节限度②）。
**什么现象会推翻我**：D13:34 加上已确认限定且 D23 已定项 15 同改；或 E165 的 J4 改成只在有已确认发布的态上判。

## 6. R5：第 92 行里程碑验收句（判决:25）

**辩方判定：理由读窄了，目的地部分站住。**
- 「它不要求正确」的整句是「规划文件，2026-09-16 建档，按当日的判断写。它不要求正确：预想的细节会随决策与实装逐步修正」（`.claude/kb/milestone/02-second-txn.md:7`）：冒号后面限定的是预想的细节。同页的读法句把预想的细节分成两类，「是仓里没定、写代码时先这么取的东西，定了就改」的才标预想（`.claude/kb/milestone/02-second-txn.md:25`），「验收标准」是「每条都要先证明会红」的（`.claude/kb/milestone/02-second-txn.md:26`）。「写入时间是发布参数」那一条没有标预想（`.claude/kb/milestone/02-second-txn.md:103`）；验收标准里一条直接写「inode 记录的写入时间留成 A 的 ⇒ 判红」（`.claude/kb/milestone/02-second-txn.md:113`），收口表把这条变异当欠账记下（`.claude/kb/milestone/02-second-txn.md:388`），后来落进变异表，就是第 92 行（行名是步 1 验收第 4 条）。把它读成「不是条款」是把口径句的范围读大了。
- 第一轮与两腿 grep 的是「写入时间」，decisions 零命中成立；但字节表有一行已定：`.claude/kb/layout/01-first-txn.md:269` 写「| inode 记录（偏移 96） | atime / mtime / ctime 纳秒 | 12 | 写入时刻 | D8（核心索引结构） 已定项 6 | 已定 |」，偏移 64 那一行（秒）同写「写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md:267`）。我 grep 决策正文里「写入时刻」只命中 D2 一处不相干的句子。限度：layout 这两行写的是新池新建文件的 mkfs 参数，覆盖写那次的取值只有里程碑页写成发布参数。所以条款级出处是 layout 字节表（已定）加里程碑验收（要求判红），不是决策正文；决策正文欠一句，第一轮的补法（D8 已定项 6 补句）与此一致。
- 目的地：「不可见」部分站住。O1 的 ObservedFile 只有内容（「O1：ObservedFile 只有内容」，`.claude/kb/experiments/171-三oracle合谋测试.md:91`），O1 判的是「POSIX 可见语义：内容、大小、目录结构」（`.claude/kb/decisions/13-验证路线.md:161`），O2 单镜像不知道发布参数，没有 API 交回它。所以第 92 行与 27、112、277 同类：三条按设计都不判；不是「条款没定」，是「条款在 layout 与里程碑页、决策正文欠一句、三条按设计看不见」。
**同样打中别的臂**：没有。同一条验收句里并列的另两条变异（extent 指针不换、反向链算成 A 之前）里，第 245 行的条款依据在 invariants 的反向链一条，不靠里程碑页；别的全放行行的条款依据都是决策正文或字节表，没有一行靠里程碑页。
**什么现象会推翻我**：在决策正文或 invariants 里找到写入时间取值的句子（那样第 92 行直接是真盲区候选）；或 layout 那两行被改。

## 7. H2：O3 与 RefFS（R1、R6）

**7.1 「规约允许的集合」取 replay 结果，对三方分工意味着什么。**
- 对 O3 的定义：D13 写的是「崩溃点截断后的每个磁盘状态是否落在规约允许的集合里」（`.claude/kb/decisions/13-验证路线.md:163`），被判对象是磁盘状态。执行器今天的做法是另一回事：它从崩溃镜像上自己恢复（「recovery = recover(image, context.dump.device_identities, context.dump.sector_bytes, cache, gaps)」，`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_core.py:124`），判恢复落到的版在不在 A(c) 里，A(c) 是「A(c) = 版本表里流序落在 [ack(c), begun(c)] 的各版」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/version_table.py:91`），不读实现的判定与恢复（「不读导出给的布局分类，不读实现判定」，`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/executor_core.py:1`；「这里一律不打开（登记 5.6：冻结之前写执行器的人不打开）」，`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/crash_dump.py:4`）。把集合取成 replay 结果，等于承认 O3 判的是版本而不是磁盘状态：这是把第一版执行器的做法升成定义。
- 对 O1-崩溃判官：它的判据是「② 择到的根必须是模型提交过的某一版」（`crates/singlefs-harness/src/model.rs:2612`）与「③ 盘上已持久的最新根槽是 (T, i) 时，择到的根按 (txg, 实例) 不许比它旧」（`crates/singlefs-harness/src/model.rs:2614`）。同样是版本层，下界是盘上已持久的最新根槽；O3 的下界是调用返回点 ack。两者在 replay 结果这一层重叠，O3 比 O1 多的只有 ack 下界与一份独立写的恢复。
- 对 RefFS：输入白名单只收「每次恢复读出的内容」等五样（`.claude/kb/decisions/13-验证路线.md:97`），允许集合是「j = lo，或 j 恰是某次显式发布的末尾；lo ≤ j ≤ hi」（`.claude/kb/decisions/13-验证路线.md:104`），是 POSIX 内容层；显式发布点「只取历史里的发布调用与 fsync」，「自动触发的发布不在 RefFS 射程」（都在 `.claude/kb/decisions/13-验证路线.md:110`）。RefFS 不看 (txg, 实例)，也不认暖机。所以版本层的下界（ack）在三方里只有 O3 有；C447 要的 O3 对 RefFS 逐态比，则要先把版本映射到内容，这件事 C447 自己写着没判（`.claude/kb/checks-owed.md:362` 的「要不要翻译层没判」）。
- 同一段 D13 写 O3 的职责不变（「仍是崩溃点集合的独立执行器，职责不变」，`.claude/kb/decisions/13-验证路线.md:110`）：第一轮的窄化与这一句要一起改，不然两处自相矛盾。

**7.2 写序义务今天归谁判：不是无人判，是三条与 RefFS 按设计不判。**
- D13 表里三条的判什么、验不到什么（`.claude/kb/decisions/13-验证路线.md:161` 到 :163）没有一格是写序：O1 不碰盘，O2 判单个镜像，O3 判恢复落点。RefFS 只吃内容。E165 判 journal 重放，也不判写序。
- 判它的是层 0 的段序列钉与点名测试：layout 登记表写「一条线的提交协议由它全部根槽写路径的录制流段序列定」（`.claude/kb/layout/01-first-txn.md:402`）、表与产物逐字比对由门禁 doc-registries 的 segment-registry 格做（「门禁 doc-registries 的 segment-registry 格逐字比对」，`.claude/kb/layout/01-first-txn.md:406`；「gate-cell: segment-registry 段序列登记表与 E142 产物逐字比对」，`.claude/gate.d/doc-registries.sh:21`），录制流与登记表由单测钉住（「单测 registered_segment_sequences_match_every_recorded_path 钉住它们」，`.claude/kb/layout/01-first-txn.md:402`）；固定脚本的段序列向量由 checker 档用例断言（「固定脚本到 D 为止的段序列」，`crates/singlefs-checker-tier/tests/crash_enumeration_fixed_script_stream.rs:428`）。第 242 行改写序后根槽 FUA 与系统配置写切进同一段（「基线里根槽 FUA 自成一段，变异把它与系统配置写切进同一段」，`.claude/kb/experiments/171-三oracle合谋测试.md:168`），段向量会变；第 270 行掉了末尾屏障，op 数 57 变 53（页:170）。这两个钉会不会红我没跑（checker 档，重型），是推的。
- 点名测试：第 270 行的在 checker 档（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/mutations.tsv:1307`：「C577：发布在系统配置轮换之后不过屏障就返回（树表 0 条的一版上返回之后见证值可以落后）」，测试名 every_crash_point_after_a_publish_on_a_version_without_file_returns_keeps_the_witness 同行）；第 242 行的是故障注入用例（`/tmp/claude-1000/e171-conclusion-r1/tree/crates/mutations.tsv:170`：publish_that_fails_on_the_system_configuration_slot_is_frozen_so_the_next_publish_cannot_overwrite_the_units_of_its_root），不是写序用例——写序本身对第 242 行只有段序列钉。
- 已知的洞：segment-registry 格对 mkfs 那一行不读活代码（C485，出处 `.claude/kb/layout/01-first-txn.md:408`，mkfs 行由用例钉住）；litmus 那一份 herd7 判得出判不出没跑过（`.claude/kb/checks-owed.md:582`，在提交时门禁）。
- 要不要进「验不到什么」：要。无论取哪一种 R1 读法，D13 表里三条的最后一列都没写写序；第一轮 G5 的文字改法只给 O3 加了两句、没给 O2、O3 加这一句，而判决:28 又把「写序义务无人判」当结论。建议写成「一次发布内各写的先后三条都不判；归层 0 段序列钉与点名测试」，不写「无人判」。

**7.3 攻方「只让 O3 红」那一形，RefFS 判不判得到：判不到（按输入推）。** 那一形是暖机第二次空发布不等持久就返回（写流层面改了屏障与 FUA，288 态、27 态 O3 红，副本、我没重跑）。RefFS 的输入是恢复读出的内容（`.claude/kb/decisions/13-验证路线.md:97`）：两版都读不到文件；暖机不是 RefFS 的操作（第一版的操作只有「第一个文件、覆盖写、发布调用、fsync、回退」，`.claude/kb/decisions/13-验证路线.md:98`）；自动触发的发布不在射程。记录核对器两条判据（「根在而记录一条都不在」，`crates/singlefs-checker-tier/src/crash.rs:86`；「恢复自称的 txg ≥ 某次发布、那次发布的某个单元全部副本都缺席」，`crates/singlefs-checker-tier/src/crash.rs:88`）也不涉及 ack。O1-崩溃判官的下界是盘上已持久的最新根槽，攻方按判据推它绿（没在模型上跑）。所以按各自的输入，这一形只有 O3 的 ack 下界判得到。
一个限度要写：这一形在纯崩溃域里 POSIX 不可见（两版内容同），伤害靠 D13 为暖机立的前提：M4 的前提写着「新实例本实例写成的根覆盖两块盘之前不让任何 fsync 返回」（`.claude/kb/decisions/13-验证路线.md:55`）。所以它是 M4 前提被违反的一形，M4 自己不罩它。它也不是 mutations.tsv 里的代码变异，是写流模拟；进 E171 表要另立表（页:124 写 E171 的表取自 mutations.tsv，`.claude/kb/experiments/171-三oracle合谋测试.md:124`）。
**什么现象会推翻我（7.2、7.3）**：段序列钉在第 242 或 270 行的变异上不红（那样写序真的无人判）；或 RefFS 在同一批态上被接上暖机的 ack。

## 8. H3 之一：G5 的四条文字改法（判决:59–:62）

| 改法 | 辩方判定 | 理由 |
|---|---|---|
| 1 关系：备料改支撑，不推翻「互相独立」（判决:59） | 写窄 | D13 写「若三者全部放行，说明存在共同盲区，回头补 oracle」（`.claude/kb/decisions/13-验证路线.md:166`），条件就是三者全部放行，没有「共享前提」限定；E171 的 Q1 正是这一支被触发的观测，页:478 自己把它留作推论（「是撑住「独立性要靠合谋测试验证」还是落到定案里」，`.claude/kb/experiments/171-三oracle合谋测试.md:478`）。判决:59 的文字没写「触发了回头补 oracle 那一支」，而这是读数直接撑得住的一句；「没有一行是三条共享同一个错前提」是逐行读条款的推论，且 E171 没量过 D13 表第 4 列写的源头独立。「不推翻互相独立」可以写，但写成「未测到」，不是「E171 支撑它不被推翻」 |
| 2 依据：引读数不引 N 行（判决:60） | Q1 存在性站住；Q3「23 / 25」写宽；「表里没有 O3 独有的那一类」写宽 | 判决:52 自己写这类比例「不当任何总体的比例」，:60 却要把 23 / 25 写进依据；分母 37 是补位之前的口径（`.claude/kb/experiments/171-三oracle合谋测试.md:460`），补位后会变。「表里没有 O3 独有的那一类」是攻方现数（拿掉屏障的 13 行只进了第 270 行），判决自己记着主 agent 没另核（判决:94）。依据里该写的只有存在性：有一行三者全部放行、有一行只靠一条 |
| 3a O3 行「验不到什么」加两句（判决:61） | 站住 | 实现写出的根的内容：第 42 行 O3 quiet（页:90）；实现自己的恢复与读者代码：页逐行写明读者侧的行（页:104、:105、:185、:188），加代码里「这里一律不打开」（`research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor/crash_dump.py:4`）。页:465 写这是读执行器代码读出来的、没另造反例（「这是读执行器代码读出来的，没有另造一行只让 O3 红的坏实现去核」，`.claude/kb/experiments/171-三oracle合谋测试.md:465`），读数加读码合起来撑得住这两句 |
| 3b O3 行「判什么」窄化成实现自己发布过的版的集合（判决:61） | 放错格 | 这是把 E170 第一版的做法写进设计列。D13:169 的「实现今天的样子」一段就是写现状的地方（`.claude/kb/decisions/13-验证路线.md:169`，已有「两边全放行，不当它判得对的证据」）；D13:110 写职责不变。窄化应写进射程，「判什么」留设计句，「验不到什么」加两句，三处合起来才不自相矛盾 |
| 4 「全部故事就是这三个 oracle」要改（判决:62） | 改的结论站住，证据选错 | 「每个放行行另有点名测试」是构造出来的：E171 的表取自 mutations.tsv（`.claude/kb/experiments/171-三oracle合谋测试.md:124`），那张表每行本来就带必须红的测试名（「每行六段，制表符分隔」，`/tmp/claude-1000/e171-conclusion-r1/tree/crates/mutations.tsv:1`），不是 E171 的读数。该句要改另有 D13 自己的内部证据：M4 落点是 E165 与 Verus（`.claude/kb/decisions/13-验证路线.md:34`）、RefFS 调用点（`.claude/kb/decisions/13-验证路线.md:99`）、被改的原句在 `.claude/kb/decisions/13-验证路线.md:167`。替换文里「每个坏实现另有点名测试罩着」写宽，应写「这批已知坏实现」 |

## 9. H3 之二：C407 的两种改写（判决:64）

先把两种说清（都按判决:64 与两腿 G5 的字面）：**甲 = 钉四列矩阵**（计入分母的行逐行钉 O1-对拍、O1-崩溃判官、O2、O3 的红 / 不红，任一格翻面或多出一行全放行就红，每个全放行行挂补 oracle 的欠账号，判别力自证沿用阳性对照）；**乙 = 只钉全放行集合**（门禁判「每行至少一条红」，今天有 12 行不满足，所以必须带登记：全放行行的集合等于登记集合、每行有欠账号；表加 O3 独有的一形；「关一条仍有别的腿红」降为测量）。攻方写「每行至少一条红」时没写这个登记，照字面今天就红，所以我按带登记的乙评。

**字面。** D13 写的判定量是：「独立性要被验证，不许自称」（`.claude/kb/decisions/13-验证路线.md:165`），「若三者全部放行，说明存在共同盲区，回头补 oracle」（`.claude/kb/decisions/13-验证路线.md:166`）。量的是「有没有坏实现三者全部放行」，那是乙钉的东西；甲钉的是整张读数，全放行集合只是其中一部分。「关一条仍有别的腿红」不在 D13 正文，只在 C407 的改法列（「把其中一条 oracle 的判定关掉，必须仍有别的腿红，不红就说明共同盲区正在那里」，`.claude/kb/checks-owed.md:327`），又在 25 个被抓行里只对两行成立（判决:64），降为测量（Q3 已经在量）是对的。

**这两种与已有的东西的关系（两腿与判决都没提）。** E171 的整表复跑已经以逐字比对的形式登记进 replay：「E171|@driver_e171_s4_f4_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out|exact」（`research/scripts/replay.sh:262`），驱动重跑全部行再跑比对器（`research/scripts/replay.sh:1014` 起；「row_driver.py --repo-root」在 `research/scripts/replay.sh:1018`，扩表那一张表在 `research/scripts/replay.sh:1020`）。比对器产物里每行每批的 `name=row_matrix` 行就是四列矩阵。所以甲在 experiment-replay 格里已经以整份产物逐字比的形式存在，它缺的是语义（哪些行是欠账、哪些是新红），不是钉。第一轮正推写「门禁里没有它」只 grep 了 gate.d。

**判别力与代价（数从我跑的命令来）：**
```
$ python3 research/prompts/e171-conclusion-r2-sonnet-model/rows_timing.py research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out
rows=117 wall=5569.6s per_row=47.60s
rows=164 wall=7607.9s per_row=46.39s
total_rows=281 total_wall=13177.5s (3.66 h) mean_per_row=46.90s 39_rows=1829s (30.5 min) 14_rows=657s (10.9 min)
```
（两行的 `table_wall_ns` 在 `research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-rows-timing.out:286`、`:290`，两条线道各 2 线程；39 = 37 加没触达却红的 2 行，14 = 12 个全放行加那 2 行。）

| | 甲：钉四列矩阵 | 乙：钉全放行集合（带登记）加 O3 独有一形 |
|---|---|---|
| 抓得到 | 任一格翻面，含第 11、244 行（两条红）丢掉一条 | 新出的全放行行；只靠一条的行丢了唯一的红（23 / 25 行就是这样的）。第 11、244 行丢一条抓不到，那两行本来也不属于 D13 字面 |
| 稳定性 | 差：第 102、233、265 行的列在批 1 与批 2 翻面（`name=row_matrix row=102 batch=1 O1=quiet O1-对拍=quiet O1-崩溃判官=quiet O2=red`，`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:1645`；批 2 同行 O2=quiet，`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:1646`；第 265 行批 1 O2=red，`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:4178`），钉矩阵就钉了批 1 的取样；换取样就红 | 好：判的是「任一列红」，那三行在批 1 都有红，只是它们靠的那一列（O2）不稳，登记要写 |
| O3 一列 | 钉住的是整列零（`name=q1_column_red_rows counted_rows=37 O1=12 O1-对拍=11 O1-崩溃判官=4 O2=15 O3甲=0 O3乙=0`，`research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out:4435`），一列零不测 O3 的判别力 | 加 O3 独有一形后 O3 列才有一行只靠它；但那一形是写流模拟、不在 mutations.tsv，进表要另立表与核对（页:124），且今天只在攻方副本上量过，算零轮 |
| 挂钟 | 自己新做一道要整表复跑：全表 3.66 h，只跑 39 行约 30.5 min；与 replay 重叠 | 做成读已入库产物的判官（读 row_matrix 行判「全放行集合 = 登记集合」「每个全放行行有欠账号」），近零挂钟；重跑仍归 experiment-replay |
| 先例 | 无 | 已知红例钉进门禁逐行复判：`.claude/gate.d/e165-pinned-red-examples.tsv:1` 写「钉住的红例：全量那一档每出一个红，按输入原样钉成这里的一行，快速档逐行复判」，门禁里一格 `gate-cell: pinned-red-examples-replay`（`.claude/gate.d/checker-tier-formal-verification.sh:18`） |

**哪一种照「独立性要被验证」的字面更合：乙。** 字面判定量是全放行集合，乙钉它，甲钉的是读数；甲的额外判别力只在 2 / 25 行，代价是对取样敏感、与 replay 重叠。**辩方替原改法留下的一条**：原改法的前半（至少一条必须红、三条全绿即判红并打印哪三条都放行，`.claude/kb/checks-owed.md:327`）加上全放行行的登记就是乙，没有判决:64 说的「当天就红」——「当天就红」只对不带登记的写法成立；所以 C407 的「没有代码」那一半有路径还清（E171 装置、驱动、比对器已写、整表已出），不必只改写；「回头补 oracle」留在各欠账号上。这条是我提的新改法，零轮，没被攻过。
**什么现象会推翻我**：补位那一趟（6 行编不过）让全放行集合变大变小而取样依赖更强；或 experiment-replay 格在提交时因耗时被整格跳过（那样甲、乙都靠不住 replay，要自己跑 30.5 min）。

## 10. 没做什么

- 没起装置、没编译、没跑任何 checker 档测试（重型不跑）；第 7.2 里段序列钉与点名测试会不会在第 242、270 行变异上红，是推的。
- 没在入库装置上重做攻方副本上的数（233 个 inode、崩溃后可写挂载 9 / 276 与 6 / 31 / 46 / 92、只让 O3 红的 27 态）：第 2、3 节与 7.3 里这些数都是引攻方报告，标了「副本」。
- 没造新坏实现；没读禁读清单里的文件（`research/prompts/e171-conclusion-r2-opus-output.md`、`research/prompts/e171-conclusion-r2-opus-model/`、`records/2026-10-01-里程碑三形式化接入.md`）；H4 不是我的格，没答。
- R6（O3 0 红不是独立性的证据）不在我的攻击面，只在 H2 里答了 RefFS 判不判得到那一问，没替 R6 辩。
- 不替主 agent 撤销或维持判决，上面只是复核结果。
- 没动仓内任何文件：写了本报告、`research/prompts/e171-conclusion-r2-sonnet-model/` 下的三个只读脚本（`recovery_side.py`、`older_vs_red.py`、`data_unit_slots.py`）与它们的输出，草稿在 `/tmp/claude-1000/e171-conclusion-r2-sonnet/`（`q.sh` 查引文用，没建编译目录与仓副本）。
- 报告写完后就地改了三处笔误（7.2 的 op 数、C485 出处行号、第 4 节一处空格）与第 9 节里一条命令的写法（原先写成缩写，现换成真跑的命令与原样输出）；引文行号没动。

## 11. cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/e171-conclusion-r2-sonnet-output.md --root /tmp/claude-1000/e171-conclusion-r1/tree --root /home/fy5090/code/singlefs --background research/prompts/_e171-conclusion-r2-background.md` 的末行（只给仓根、不给冻结副本时同样 54 对 54）：

```
  ✓ 核了 54 处引文，对上 54 处，没判 0 处（逐处列在上面）
```
