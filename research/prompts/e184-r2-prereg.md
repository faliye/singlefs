# E184 第 2 次重跑登记：常规层与大负载层的代价计数（新段形下的锚与作废条款取值、第一段 18 件重跑后判定词翻不翻）

写于 2026-10-24，这一趟一件都还没跑，装置不改。重跑已有实验，不占号。下文「原登记」指 `research/prompts/e184-preregistration.md`（读的那一刻 sha256 `3a7839b863ec659534884424bbcdc7519b1199632c436527f3e174357882be4a`，745 行）；「改前产物」指 `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out`（sha256 `7eacaffaddd44d34bf4a67afe29214ccbbf7ffbd63663d61ef7489124e858c06`，950 行）。原登记第五至十一节的臂、阈值、作废与停机条款这一趟一个字不改；下面只写这一趟新加的量、它们的判据与比法，其余各节写「同原登记」并点名那一节。

<!-- doc-lint:not-numbers J1 J2 J3 J4 G1 G2 G3 G4 T0 T1 T2 T3 T4 T5 T6 T7 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 B1 B2 B3 B4 F1 F2 F3 F4 F5 F6 F7 X1 X2 X3 X4 X5 X6 X7 V1 V2 V3 V4 V5 V6 V7 V8 E184 E185 C409 C577 C755 C895 S1 S2 S3 S4 XR1 XR2 XR3 XR4 XR5 FR1 FR2 FR3 M1 M2 M3 M4 M5 M6 M7 M8 M9 P1 P2 P3 P4 P5 -->

**装置写在哪（写死）**：入库装置原样，不改：`crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs`（读的那一刻 sha256 `8f529e121a08f0c33c2957fa209eaf38e1af7d00e68a3b3fde05512f846ab4f5`，7349 行，批 16 只修钉值之后的现文）与 `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts_write_dump.rs`（sha256 `320822c4a897abd44f406b7b6fac0e3a9d082165ef4768dd4408c83ab57b8f0c`）；长跑脚本 `scripts/experiment-apparatus/e184-segment-one-long-run.sh`（sha256 `8f9617a306d1ac30fa11fdca1f1eb710173140c975dfe2dffaa4e11db6fa5cfd`）原样。这一趟新写的只有逐格比的脚本，走 `scripts/experiment-apparatus/`（独立于 `crates/`，只读两份产物的文本，照先例 `scripts/experiment-apparatus/e185_r1_compare.py` 与它的 `_test.py`、`.mutations.tsv` 三件写）。这一趟不新读 `crates/` 的常量，原登记文件头那张本地常量表不变。

**不是只写三样的差分重跑登记**：问题单第二行是「输入变了、同臂重跑」，第一行却要改原登记写死的锚与作废条款的数，所以照固定节名写全十四节，不变的节指回原登记。

## 一、问题

英文名：regular_layer_cost_counts

主 agent 给的问题（派发提示「关的是哪几条」逐字）：

1. 问题单第一行：登记第七节锚 A1、A2、A5、A7 与第十一节作废条款 X3（`SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST` 16842807）在新段形（新池 12 段、建快照 16 段、多记录 20 段）下各取什么。读数来源：`~/.local/share/singlefs/work/m3-close-b16-e184-e183-pins/result.md`（第一、三、五节，新旧 selfcheck 差的 18 行与「新段形现算」那一段）与装置 `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` 现文的钉值表 `PINNED_BY_THE_TESTS`、`voided_cold_runs`、`run_selfcheck_command`。验收：每个锚与 X3 新旧两数并列、各写来源（产物行或装置行号）与独立现算的式子；登记原数原样保留，修订写明改了什么、依据哪份读数、时点在第一段重跑之前。X3 的新值要独立现算，现算不出就写「现算不出」并说明缺什么，不抄装置。
2. 问题单第二行：第一段 18 件在新段形上重跑、逐格比判定词（T1–T4、A1–A6、B1 触发 F6）的跑法、产物名、逐格比法、作废条款与「翻面」的判据，跑前写死。验收：登记里每个量对上问题单的一行，判据写成跑前可判的一句。

问题单原文（`cat research/prompts/m3-close-e184-r2-questions.md`，逐字；8 行，sha256 `02c5473f5fda26f96a91cb4aa6527ebbecaaaa3a9b3563751a29a054c00cdc18`）：

# E184（常规层成本计数） 第 2 次跑前登记修订问题单（2026-10-24）

上游写序变化（取号写两轮、C755 每次可写挂载问一次 F2）之后，装置 `crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` 的钉值表已照新段形重钉（新池 11 → 12 段、建快照 15 → 16 段、多记录 19 → 20 段，`~/.local/share/singlefs/work/m3-close-b16-e184-e183-pins/result.md` 第一、三节），而跑前登记 `research/prompts/e184-preregistration.md` 第七节锚 A1、A2、A5、A7 与第十一节作废条款 X3（`SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST` 16842807）写的还是改前的数；第一段整份产物（18 件长跑）没在新段形上重跑。

| 问题 | 候选 | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|
| 登记第七节锚 A1、A2、A5、A7 与第十一节 X3 的数在新段形下各取什么 | 甲：照新段形现算的数改登记（修订记录写改了什么、依据哪份读数、时点在第一段重跑之前，原数原样保留）；乙：登记原数不动，另立重跑登记、只换数 | 现算的新数与装置 selfcheck 现录值逐个相同（不同则装置或现算有一边错） | 每个锚与 X3 都有新旧两数、来源与现算式子，装置 selfcheck 退 0 | 开着 |
| 第一段整份产物在新段形上判定词翻不翻（T1–T4、A1–A6、B1 触发 F6） | 不翻：重跑之后判定词逐格同改前；翻：某一格换判定词 | 重跑产物里任一格判定词与改前产物不同 | 第一段 18 件在新段形上重跑完、逐格比过 | 开着 |

### 1.1 读法写死

- **新段形**：今天主工作区 `crates/` 录出的段形，即装置现文三个段形常量：建快照流 `[2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2]`（16 段、64 写，`e184_regular_layer_cost_counts.rs:4486`–`:4487`）、新池新建文件流取它前 12 段（43 写，`:4489`）、多记录流 20 段（117 写，`:4491`–`:4493`）。
- **「在新段形上重跑」**：在今天主工作区的 `crates/` 与上面那两份装置现文上，用长跑脚本把第一段 18 件从一个新建的空工作目录整份重跑一遍。2026-10-19 以来 `crates/` 里别的改动（取号写两轮、C755 之外的）同样进了这一趟；翻了的格不归因到段形，只报「今天的树上翻没翻」。
- **18 件**：长跑脚本 `load_segment_table`（`e184-segment-one-long-run.sh:180`–`:189`）排的件：`repo-reads`、`selfcheck`、`record-fixed-chains`、`keys-fixed`，六档随机历史各一件 `dump-<档>` 与一件 `keys-<档>`（档表 `:94`–`:101`），`share`、`library`。
- **旧数 / 新数**：旧数 = 原登记第七节、第十一节写死的数（原登记行号写在第七节表里）；新数 = 新段形下的数。两边并列，原登记一个字不改（写范围只有这一份，原登记第十二节追不追一条指回这里，见本节末「未定项」）。
- **来源**：装置行号（上面那一版 sha256 的现文）；产物行写成 `name=anchor identifier=<名>` 这类可 grep 的键，不写行号（我没读产物正文，第十三节）。
- **独立现算**：不读装置的常量，只用测试文件注释写的每段的角色（哪几段是系统配置槽段、哪几段是记录段、根槽段、单元段）与 D13（验证路线） 已定项 4 的口径（原地覆写取三态，其余两态）算两态闭式、全量、甲二。两条路：**整式路**（照角色从头算）与**增量路**（原登记写死的旧数 + 新段形多出的系统配置槽 2 写段的增量：两态每段 +3、全量与甲二每段 +8）。两条路共用的前提只有一样：多出来的那一段是系统配置槽段、两次写都取三态；整式路另多一个前提：这条流上的记录写都不取三态。脚本与输出在第七节。
- **判定词**：产物里下面这几类行的判定字段；一格 = 一类行里的一把键，键与字段：
  - `name=stop_check`：键 `item=`，判定词 `result=`（pass / fail）。停机 T1–T4 在 `selfcheck` 件的 28 格里（T1 三格 `local_*_matches_crates`；T2 五格 `copied_text_verbatim_*` 与 `copied_segments_found`；T3 八格 `recorded_values_match_the_test_pins_*`、`pins_still_in_the_test_file_*`；T4 四格 `two_counting_routes_agree_*`；另有锚 B2 四格 `two_state_not_above_full_*`、锚 A3 四格 `blocks_per_point_follow_the_local_block_length_*`），`repo-reads` 件另有一格（停机 T6）。
  - `name=anchor`：键 `identifier=`，判定词 `match=`（true / false）；10 格：A1 一格、A2 两格、A3 一格、A4 一格、A5 与 A6 的全量四格、B1 一格。
  - `name=selfcheck_done`：一格，判定词 `clause_anchor_matches=`（B1 对不上即触发 F6 第一条）。
  - `name=overlap`：键 `scope=`，判定词 `verdict=`（J1-乙）。
  - `name=share_group`：键 `scope=`，判定词 `verdict=`（J1-丙，含 `not_planned`）。
  - 由上面几格推出的失败条款：F1、F2、F3 与 F6 第二条（式子在第六节）。
  - 不比的：`detail=`、`expected=`、`actual=` 这类数值字段（新段形下本来就变）、挂钟字段、`name=segment` 行。
- **对齐**：每行取第一个以 `name=` 开头的字段当行名，行首有没有 `E7RESULT ` 都认；同一份产物里同一类行同一把键出现两次，那一份不比（停机 S4）。

**问题单第二行的读法**（主 agent 定宽读，窄读那一段已删；窄读 40 格是宽读的子集，比较脚本照旧两行都报）：

- **宽读**：「整份产物」「任一格」指第一段产物里全部判定词：窄读那 40 格，加 J1-乙 的 `name=overlap` 与 J1-丙 的 `name=share_group` 各 11 格、F1–F3 与 F6 第二条四格推出的判定。回答它的是第六节「停机与锚判定词」「负载判定词」两行，任一行记「翻」即记「翻」。
- 两种读法下每个量自己的判据相同（同一格换没换词），只差哪几行算进问题单第二行的回答；两行都量，不加跑任何东西（比的是同一对产物）。

### 1.2 问题单各行由这一份的哪几个量回答

| 问题单行 | 这一份的量（第六节） | 补完够不够判 |
|---|---|---|
| 第一行（锚 A1、A2、A5、A7 与 X3 在新段形下取什么） | 「新旧锚并列」（每个锚与 X3 的旧数、新数、装置行号、两条现算路的数）、「现算与自检现录逐个比」、「自检退出码」 | 前一个量这一份写登记时已经算完（第七节）；后两个量在重跑产物的 `selfcheck` 件里读，读完就够判 |
| 第二行（第一段判定词翻不翻） | 「停机与锚判定词」（窄读与宽读都用）、「负载判定词」（宽读用）；「离门槛的距离」附带，够判后不跑 | 重跑产物完整、逐格比的脚本自证过、两份产物逐格比完，就够判 |

候选甲、乙（第一行）是记法的选择，不由量翻面：这一份就是乙说的「另立重跑登记、只换数」，第七节那张表同时备好了甲要往原登记第十二节追加的文字（改了什么、依据、时点都在表里）。追不追由主 agent 定。

未定项：

- **问法**：已定，宽读（1.1 节）。
- **X3 新值落进装置的时点**：X3 只在 `compose`（第三段 J2 合算）里用（`e184_regular_layer_cost_counts.rs:3035`），这一趟只重跑第一段、不跑 `compose`，装置常量 `SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST`（`:2913`，现值 16842807）这一趟不改。第三段开跑之前要把它改成第七节的新值，连同批 16 报告第三节点名的两条手造读数的用例（`cold_run_that_is_not_cold_or_not_monotone_or_not_the_pinned_item_is_void`、`the_compose_lines_give_the_pinned_cells_for_a_hand_made_load`）；不改就跑第三段，新段形下的小项 `states=` 会被 X3 判作废。谁改、什么时候改缺主 agent 定。另：实验页标题写「第二到四段未跑」，批 16 报告第三节写「第一段的 J2 冷跑按它判过」，两处说法我没核，交主 agent。
- **准入**：`python3 scripts/gate/admission.py experiment <仓根> E184` 退 2（`.claude/gate.d/stage-inputs.tsv` 没有 E184 这把键，原样输出在第十三节），准入判不了；补不补键缺主 agent 定。
- **比对对象表的机读登记**：E184 没有单元表，第十一节比对对象表的两行都要在 `.claude/gate.d/replay-comparison-units.tsv` 文件头写「# 不登记的比对对象：research/prompts/e184-r2-prereg.md #<号>：<理由>」，理由里要有一个开着的欠账号；E184 没有现成的号（E185 用的是 C895，只管 E185），缺主 agent 定立一笔新账还是另有出路。

## 二、被测条款与它引的定义

被测的是实验页 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md`「影响的决策」表点名的四条分项：D13（验证路线） 已定项 4、9、16、17。这一趟用到它们的地方：已定项 4 是独立现算的口径（原地覆写取三态、系统配置槽写属于这一类）；已定项 9 射程里的 12230590578 是锚 B1；已定项 16、17 只经原登记的判据进来（`beyond_u64`、「名字不进键」），这一趟不改。原登记第二节另抄的已定项 10、11、15 与 C409 那一行，这一趟没有量碰到，不重抄（原登记第 86–294 行）。

`scripts/kb/quote-kb.py` 用 `@标题` 整段抄（出口 `~/.local/share/singlefs/work/m3-close-b17-e184-r2-rereg/quoted-clauses.md`，108 行，脚本回读逐字节一致：「✓ 4 段整抄进 …，回读逐字节一致」），原样追加在下面。

**出处 `.claude/kb/decisions/13-验证路线.md:138-153`（整段抄，未转述）**

```markdown
#### 已定项 4：崩溃点重放只枚举整写子集

**定案**：崩溃点重放要枚举的崩溃状态集合定义为：录下来的写请求流**按设备**切成段——一次写只被它自己那块盘上之后的屏障（或它那块盘上的 FUA 写）排在之后的写前面；当前段里每块有写的盘都被自己的屏障或 FUA 放行时这一段才关上，有一块盘还没放行，这道屏障（或 FUA）不关段、前后的写同段。一个崩溃状态是「前若干段全部持久，当前段每次写各取它的几态、任意组合，之后各段全部没持久」；枚举的单位是**一次整写**。带 FUA 的写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写同段，别的盘上的写不因它持久。一次写取两态（持久 / 没持久）；**原地覆写**（不是单元写、长于一个扇区、罩住的范围里原来有东西的写，系统配置槽写与覆盖旧记录的 journal 写属于这一类）多取第三态「新旧都读不出」，全量与快档都枚举；第三态的镜像罩那次写的整个范围，新旧不同的那一截前一半是新字节、后一半是旧字节（按字节撕，不按扇区）；其余写的撕裂态一律并进「这次写没持久」。「没持久」的位置上，镜像里放的是**崩溃前那个位置的旧字节**，不是零，而且这些镜像（第三态的在内）要真的生成出来喂给 checker 与记录核对器，不许只当计数。镜像双写（D2（RAID 条带策略） 已定项 6 的 w ≥ 2、D22（单元原子性怎么合成） 已定项 8 的 journal 镜像）按**设备**各算一次写，harness 的状态数按自己的写流另算闭式。**同一块盘上连着的屏障并成一道**：录制流里一道屏障与这块盘上一道屏障之间没有任何一次写（别的盘的屏障不算写）时，这一道不记（`crates/singlefs-harness/src/lib.rs` 的 `push`），独立模型照同一条规矩记；中间没有写，这一段不产生崩溃状态，合并与否不改变崩溃状态集合，只改变操作数与种类串。

**射程**：它定义的是模型层的枚举域。真实设备的 FLUSH / FUA 是否如宣称生效归 C6（块层语义假设写错），要在 QEMU 里用真设备验。按设备切段多出实际走不到的状态（一块盘没放行时，别的盘上已放行的写也留在段里取任意组合），不漏走得到的。撕裂分两类：原地覆写之外的写（单元写、不长于一个扇区的写、罩住的范围里原来没东西的写）撕裂态与「没持久」的差别只有 CRC32C 的碰撞概率（32 位，D19（块指针的结构与宽度预算） 已定项 2、D23（journal 的角色与格式） 已定项 11），**知情接受**，撕裂粒度不进模型，与 D20（承重面：单元的原子性与自包含）「有父指针的单元不依赖任何宽度」一致；原地覆写撕裂时那一处的旧内容也没了，「没持久」却留着完整的旧内容，两者不是同一个状态，所以单独枚举。崩溃点重放的扇区取 512 字节；两条崩溃点重放流上取第三态的只有系统配置槽写。已知边角：第三态的旧字节取「这次写之前的写全落了」那一版，同段更早、与它重叠的写没落的组合里旧的那一半多带那次写的字节（两条崩溃点重放流上这种组合一次都没有）；根槽写长于一个扇区又罩住旧内容时第一版不支持第三态（harness 断言；崩溃点重放的池都按 512 字节物理块建，走不到）。设备把一次写撕成恰好撞上校验和碰撞的形态不在模型内。D13（验证路线） 已定项 5 与它正交：一个定枚举域、一个定 crate 边界。

**依据**：
- E77（发布的持久顺序）：段模型在四种屏障摆法下的状态数逐臂等于闭式——这套枚举域数得对。
- E142（新池新建文件的干跑）：第十七次跑第一段的几何敏感性取样点 G7（阳性对照同一条写流、只把 FUA 当边界）实测 4097 个状态、2046 个违例——没有屏障时单元、journal 记录与根槽合成一段 `[12]`；装置的切法与 `crates/singlefs-harness/src/segments.rs:91`「FUA 写关掉自己所在的那一段（FUA 不替前面的普通写做持久，所以它们同段、任意子集）」一致，「FUA 自成一段」那一读法预言的 2050 / 0 对不上——FUA 那一句的字面照实现的切法写。
- 三方对抗与判决：[verification-build.md](../verification-build.md)「三方对抗（2026-09-03）」判决第 3 条——每一类单元都有整单元校验（有父指针的靠父指针里的校验和，D4（校验和位置） 已定项 1、D16（发布语义） 已定项 7、D22（单元原子性怎么合成） 已定项 19；自证单元靠整单元校验和加被实际检查的代号，D22（单元原子性怎么合成） 已定项 21；journal 记录靠头校验和与头内的载荷校验和，D23（journal 的角色与格式） 已定项 13），撕裂态与「没持久」在校验和眼里是同一件事。
- 用户定案（按 E77（发布的持久顺序） 与三方对抗定；FUA 写算段边界那一句是 C313（FUA 写算不算崩溃段的边界） 收口时的用户定案，2026-09-26 用户定照实现改成「关掉它所在的那一段，与前面没被屏障隔开的写同段」），记在变更史。
- 按设备切段（有一块盘没放行就不关段）、FUA 只放行它那块盘、原地覆写多取「新旧都读不出」：用户定案（代码审阅第 1、3、4 条；2026-09-27 弹窗选「A 合并，按设备记屏障」「照补，全量也跑」与「原地覆写补第三态」），第三态按字节撕是主 agent 同日定的、被攻过零轮，记在变更史；实现与判别力（只吞盘 1 一道屏障，段尾 `26,2,1,2` → `26,5`，崩溃点重放小流里记录核对器报出来）在 `m2-rev-b3a2-implementer-report.md` 第一、三节。
- 同一块盘上连着的屏障并成一道：用户 2026-09-28 定「并，跟今天的录制器」（E142（新池新建文件的干跑） 第十九次跑撞上登记的停机 S19-crates：独立模型两道都记、录制器并掉第二道，暖机与整条流四格的操作数与种类串不同、崩溃状态数相同，报告 `e142-r19-runner3-report.md` 结论段，登记 `e142-r19-prereg.md` 的 R40），原话在变更史。

**欠**：C6（块层语义假设写错）：拿虚机的设备侧日志重建崩溃状态、崩溃测试必须变红的那一半。

```

**出处 `.claude/kb/decisions/13-验证路线.md:245-278`（整段抄，未转述）**

```markdown
#### 已定项 9：崩溃点重放要分三层，否则挂钟必炸

**定案**：

| 层 | 范围 | 触发 |
|---|---|---|
| 冒烟层 | 固定种子、固定脚本的负载与登记的崩溃枚举用例，不按写请求数限规模（规模见射程），精确全量 | checker 档的全量（已定项 15）：用户要求时跑；提交时默认只跑快档 |
| 常规层 | 中等规模负载（写请求数上千），精确全量，枚举不完靠精确剪枝，一时跑不了的节点标 ignore，覆盖报告与门禁逐个列「本次未跑」 | 每次提交 |
| 大负载层 | 常规层 负载 + 大规模负载 + 多设备档位交叉，N≥5 轮，精确全量 | 用户要求时 / 发布前 |

**剪枝加 ignore 之后敢说什么**：敢说「开着的节点在全部崩溃状态上通过」；
**不敢说**「ignore 的节点通过」。

**全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快档全绿（必要条件）。**
**里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑。**
「全量」指两样：checker-tier-crash-point-replay `--full`（门禁 checker-tier-crash-point-replay，已定项 15 的全量）与崩溃放量全量（`scripts/crash/crash-amplification.sh` 不带参数）。
「动了崩溃点重放或判器」按崩溃判定库现算：这一批改动让库里有判定块的单元键对不上、对不上的那几把键里有判器单元（判器入口 fn 的单元闭包里的函数单元）变了，就算；阶段把作废块数与变了的判器单元清单打出来，主 agent 照它弹窗，不按路径清单判。
「上一次全量」由主 agent 写进当前里程碑记录一行：日期、两样各自的结论。
**全量档必须优先保证「失败可复现」而不是「覆盖率最大」**——修一个全量档 bug 的挂钟 ≈ `log2(N) × 一趟全量的挂钟`，一个不可复现的全量档失败价值接近 0，
甚至为负，它训练人忽略红灯。

**射程**：只有冒烟层（门禁 checker-tier-crash-point-replay）；冒烟层分两档：平时快档每段原地写取任意子集、COW 写只取全落或全不落（用户 2026-09-26 定，叫「甲二」，在 C561（记录核对器的复用豁免在复用只落一半时假红） 修好之后用），它看不见只在 COW 部分落盘时才显出来的那一类，替不了全量；全量带断点续跑（门禁 checker-tier-crash-point-replay `--full`），登记在 `.claude/gate.d/stage-inputs.tsv` 的崩溃枚举用例已经长到并行线一那条流 12230590578 个状态，不再是「写请求数两位数」的冒烟体量，所以提交时默认只跑快档，全量按已定项 15 由用户要求时跑；常规层、大负载层在门禁里没有阶段。每个崩溃状态怎么定义归已定项 4；节点怎么精确剪枝、ignore 节点什么时候开归已定项 10、11。

**依据**：
- 无实验：分层是门禁挂钟的设计约束；常规层、大负载层在门禁里的负载还没有定，E184（常规层与大负载层的代价计数） 第一段已量候选负载的规模与态数区间重叠，只备料；第二到四段（作废面、冷吞吐、两种落法的固定开销）未跑。
- 三方论证：D13（验证路线） 新立那一轮（本地 LLM / Sonnet / Opus 各一条腿，立场不重复），材料没有留存，经过在变更史。
- 平时快档用甲二：用户定案，原话在变更史；崩溃点重放规模三轮判决 `research/prompts/m2-layer0-scale-r3-main-verification.md`。
- 用户定案：「我们不是抽样 我们是剪枝 加 初期ignore」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定案：不抽样，剪枝加初期 ignore」那一行）、「枚举可以设置 ignore 标记 然后慢慢开启」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节「用户定：ignore 标记慢慢开」那一行）、「1 该枚举该设计的设计完 设计全」「2 枚举不完 正是剪枝的用处」（`records/2026-09-28-里程碑三崩溃放量剪枝与GPU调度.md` 第三节里材料员被门禁 doc-process-records 的 implementation-premise 格挡下、用户定『设计全、枚举不完靠剪枝、粒度合适』的那一行）；第一轮判决 `research/prompts/m3-prune-gpu-r1-main-verification.md`「回看决策」一节：已定项 9 推翻。
- 用户定案（2026-10-01）：全量档不限时刻，用户要求时随时跑，删掉「夜间」，原话在变更史。
- 用户定案（2026-10-01）：全量档全绿的时点是里程碑收尾与动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑，原话在变更史；交用户那一格是三方判决 `research/prompts/test-script-align-r1-main-verification.md` 第二节 t1-3。
- 三方判决 `research/prompts/test-script-align-r2-main-verification.md` 第二节 U3（「U3 t1-3 写回照字面做不了」那一行）：「全量」的两个所指、「动了崩溃点重放或判器」的路径判法、「上一次全量」的记法，「准入条件」与「异步」合成一句。

**欠**：C409（常规层、大负载层在门禁里没有阶段）。

```

**出处 `.claude/kb/decisions/13-验证路线.md:413-428`（整段抄，未转述）**

```markdown
#### 已定项 16：按段登记的段整域装不进 u64 就分片判，态数用 1280 位定宽整数

**定案**：
- 分片：崩溃放量里按段登记的段（`.claude/gate.d/crash-point-replay-segments.tsv`，一行一段五列：项名、流名、段号、段写数、段写表摘要），整域态数装不进 u64 时按固定大小的片依次判；片大小由环境变量 `SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES` 定，没设取 2^32 态；片起点是判定库里的续点（续点按段写表摘要与判器单元键寻址，项名、流名、段号只作显示）。整域装得进 u64 的登记段不分片，整域一次判。代码落点：`crates/singlefs-checker-tier/src/crash_amplification.rs:14800` `REGISTERED_SEGMENT_SLICE_STATES_ENVIRONMENT_VARIABLE`、`crates/singlefs-checker-tier/src/crash_amplification.rs:14947` `fn segment_slices_of_the_recording_from_the_environment_and_the_library`。
- 位宽：登记段的总态数、片起点、片尾与进度一律用 1280 位（10 个 u128）定宽无符号整数 `StateCount1280Bits`（`crates/singlefs-checker-tier/src/crash.rs:1640`，肢数是同一文件的 `STATE_COUNT_LIMBS`），算术全部 checked，不饱和、不截断；驱动行 `registered_segment_progress=` 的已判态数与总态数都写精确十进制，不写幂的形态。
- 登记表怎么交给引擎：引擎只经环境变量 `SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY` 读按段登记表（`crates/singlefs-checker-tier/src/crash_amplification.rs:14764`，在 `fn registered_segments_of_the_flow_from_the_environment` 里），没设就当一段都没登，没有别的退路。144 个单元的文件顺序写成 145 个、extent 下段长成两层的那条流（引擎流名 `position_addressed_ExtentLowerSegmentGrowsToTwoLevels`）的单元写段（322 次单元写）与记录段（290 次 journal 记录原地写）两段都登。

**射程**：管崩溃放量引擎对按段登记的段怎么分片、态数用什么整数记、登记表从哪里读；不管哪几段该登（由用户点头，写在登记表文件头）、不管登记段的域本身怎么展开（判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10）、不管每个崩溃状态怎么定义（已定项 4）。没登记的段的态数、与整条流合起来的态数仍要装得进 u64，装不下在写库之前拒跑；登记段整域越过 1280 位同样在写库之前拒跑。片大小只许 1 到 2^32：两个 GPU 内核把片内序号当 u32 加到片起点上（`crates/singlefs-checker-tier/src/crash_amplification.rs:14807` `LARGEST_REGISTERED_SEGMENT_SLICE_STATES`）；轻型（`scripts/crash/crash-point-replay.sh --quick`）调用方没设片大小时设成 `LIGHT_REGISTERED_SEGMENT_SLICE_STATES`。展开上限盖不住又没登记的段，放量入口 `scripts/crash/crash-amplification.sh` 判红；崩溃点重放入口、放量入口与分项驱动开跑前核这份表再送给引擎。

**依据**：
- E161（崩溃放量的去重与分段耗时）「第 2 次重跑：取样域按 11 段的流重定（2026-10-05）」一节：段的全量态数与闭式、装置逐个数、crates 计划逐段相等，态数随段写数按幂长，撑的只是「长段整域装不进 u64」这个前提；片大小、位宽取值与登记表的读法是工程参数，用户定，它不撑。
- 用户定案，原话在变更史：分片与每片 2^32 态（`records/2026-10-11-崩溃放量两段与脚本审计.md` 第 96 行）、位宽 1280 位（同一份记录第 98、99 行）、两段都登（同一份记录第 92 行）。
- 分片与位宽是对判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 的 J10（按段登记）的补充，没经三方攻过。

**欠**：无。

```

**出处 `.claude/kb/decisions/13-验证路线.md:429-451`（整段抄，未转述）**

```markdown
#### 已定项 17：判定复用的单元是函数，键只由函数单元、工具链、依赖版本组成

**定案**：
- 重型门禁、崩溃判定库与实验准入的每一份判定（崩溃状态块、快档与模型对拍的每条测试函数、crates 变异每行、广谱变异每条、形式化每条、QEMU 每档、litmus 每份、研究复跑每行、实验变异表每条、实验准入）沿不沿用，只看它的单元键：它依赖的函数单元各自去注释后的指纹，加闭包里各单元（含 mod 头）的 `#[cfg]`、`#[cfg_attr]`、`cfg!` 真引到的 cfg 原子、各带这一次目标下的真值（闭包没引到的原子不进键：带不带 `gpu` 特性编的两趟，闭包不提 `gpu` 的判定键相同、共用一份库），加工具链（含 cargo 解析后的生效构建环境与外部工具版本）、从 `Cargo.lock` 解析出的闭包真引到的依赖版本，加这份判定自己的内容输入（段写表与起步镜像摘要、litmus 内容、变异原文与替换文这类）。函数单元从这份判定的入口沿调用与引用收，连同引用的常量、类型、静态项与宏展开一起算；wgsl 按 fn 切。
- 一律不进键、也不许当复用依据：按提交、HEAD 或树算的改动范围；整文件原文；文件级、包级、整表级的指纹或闭包；手写的版本常量；路径（仓根也算）、名字、种子串、行号、资源上限、限时；只有整道或整格的标记而没有按判定的库。键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。
- 收不全（动态分派、宏、`include_str!(concat!…)` 认不出）的那一份拒算键、照跑，不静默少收；每把键连同组成它的单元清单落盘，作废时报得出是哪几个单元变了。
- 两种排除不进键，理由不同、分开记。文件级：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与单元算法 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键。边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条，第二台那一路只出提示，按这条边排出键。边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。另有一格「键不变而退出码变」必红。python 模块有单元进闭包时，它在 import 时执行的模块级语句一起收进来：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。
- 每类判定一份库，固定在持久目录，不随一趟、不随判法换目录；仓根也不是库的输入：崩溃判定库的目录由环境变量 `SINGLEFS_CRASH_AMPLIFICATION_LIBRARY` 给时只收绝对路径，相对路径拒绝；全量开关只写不读、现判的照键写回同一份库；第二台判出的并进本机那一份。
- 改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。

**射程**：管重型门禁、崩溃判定库与实验准入的复用键；不管静态门禁「只判新加行」的改动范围（`scripts/gate/changed-paths.sh`，那是判据语义）与 preflight 的 `admission: inputs-changed`；不管枚举域（已定项 4）、分层与什么时候请用户跑全量（已定项 9）。镜像判定与给定状态判定的版本是判定定义那一组（CPU 参照判器 `crash::judge_crash_image`、`crash_given_states::judge_the_given_states_on_the_cpu` 的闭包）的键，不论这一块在 CPU 还是 GPU 上判的：GPU 判器是加速路径，它的准入靠与定义那一组的对拍（快档的 GPU 与 CPU 对拍用例），它的单元只进块属性当出处，不进键、不分种类；显卡驱动、卡型、几张卡怎么分活同样留作块的属性或跑的参数，不进键。

**依据**：
- 用户定案（2026-10-15），原话在变更史与 `records/2026-10-13-复用键与全仓改名.md`「2026-10-15 用户定：提交与重跑无关，重跑只跟代码变化走」一节、「2026-10-15 批3」那几段。
- 用户定案（2026-10-17）：判定不与硬件、不与 CPU 还是 GPU 绑定，复用与剪枝不看设备；原话在变更史与 `records/2026-10-13-复用键与全仓改名.md` 2026-10-17 叫停全量那几段。
- 主 agent 定案（2026-10-15）：仓根不进键与库、库目录只收绝对路径、cfg 成分只取闭包真引到的原子，出处同一份记录「2026-10-15 验证段第一步判红、两条定案、4-3 四批派出」定案第 1、2 条。
- 反证：E164（崩溃放量新设计的小范围功能测试） F4 量到只按函数取指纹、不连引用的常量一起收，漏判 V5 与 V11（V11 只改常量 `DATA_POINTER_BYTES`）：函数单元要连同引用的常量、类型、静态项与宏一起算。
- 盘上的数：`ledger.md`「一之二、盘上量到的」一节——只差名字的块各存各判 4100 块与 9679 块；整库一把的判法摘要一换累计多判 3 935 242 934 态，那次闭包里改动的 29 份文件有 24 份去注释后逐字相同：键带名字、按文件或整版取指纹，同一份判定会重判。
- 三方判决 `research/prompts/m3-close-reuse-closure-r1-main-verification.md`、`research/prompts/m3-close-reuse-closure-r2-main-verification.md`、`research/prompts/m3-close-reuse-closure-r3-main-verification.md`（两种排除并列、模块级语句随模块进闭包、第二台那一路只出提示按边排出键；三轮定，零轮改法交用户定）；用户定案 2026-10-22「丙加甲 要走三方」、2026-10-23 弹窗四题，原话在变更史。
- 没有腿况记录：除复用键闭包三轮（两种排除、模块级语句、第二台那一路）攻过的几处外，未经三方攻过；用户定改完代码开三轮攻方，只攻「剪枝是不是原子化的、键是不是只由函数单元组成」。

**欠**：C8（门禁范围判不出来）、C489（singlefs-core 的广谱变异对今天的代码没有结论）、C528（两条门禁路判的改动窗口不同）、C564（另外 6 条崩溃点重放全量没人跑）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C630（提交时形式化那一道第 1 步停下的频率没量）、C645（崩溃放量全量没有人能派）、C777（五个实验键在 stage-inputs.tsv 没有登记行）、C836（staged-green 挂的树不一定是被判的树）、C890（判法单元闭包少收 import 时执行的模块级语句）。

```

## 三、实现今天的样子

主工作区现状（含没提交的改动），2026-10-24 现读：

- 态数的口径：`crates/singlefs-checker-tier/src/crash.rs:2184` `TearableInPlaceOverwrites`、`:2261` `is_tearable_in_place_overwrite`（`:2275` 系统配置槽写不算 COW；`:2278` 长于一个扇区；`:2298` 罩住的范围里基镜像有非零字节或更早有带字节的写重叠，才取三态）；`:2047` `CrashPointReplaySegmentExpansion::state_count`（全量：段内各写态数相乘减 1；甲二：COW 写只取全不落或全落，单元段只剩 1）；`:2469` `crash_state_count_with_torn_in_place_overwrites`、`:2501` `quick_tier_expansion`、`:2510` `full_expansion`；`crates/singlefs-harness/src/memory_pool.rs:659` `closed_form_state_count`（1 + Σ(2^|段| − 1)）。
- 测试文件写的段的角色与钉的数（独立现算的输入，都在 `crates/singlefs-checker-tier/tests/` 下）：
  - `crash_enumeration_snapshot_creation_stream.rs:46`–`:48`：段形与每段的角色（取号两轮各 2、暖机两次各「记录 2、根 1、轮换 2」、A 的单元 24 与它的记录、根、轮换，S 的单元 16 与它的记录、根、轮换）；`:13`–`:15` 文件头写全量 16842815、两个单元段占 16842750、两态 16842785、系统配置槽写取三态；`:244`–`:247` 全量的断言写成 `1 + 6 * (3 * 3 - 1) + 4 * 3 + 4 + ((1 << 24) - 1) + ((1 << 16) - 1)` 与 `16_842_815`。
  - `crash_enumeration_new_pool_file_creation_stream.rs:772`–`:781`：两态 16777243（取号写一轮时 16777240）、全量 16777268（五个系统配置槽段；取号写一轮时四个、16777260）、甲二 54（取号写一轮时 46）。
  - `crash_enumeration_fixed_script_stream.rs:99`–`:107`：两次取号各多一个 2 写段，78 段 1678377089 → 80 段 1678377095；系统配置槽段 25 段（取号写一轮时 23），全量 1678377095 + 25 × 5 = 1678377220（取号写一轮时 1678377204）。
  - `crash_enumeration_multi_record_publish_stream.rs:21`–`:22`：全量 1358954642、两态 1358954607、B、C 两个单元段占 1342177278；C577 之前 12230590578。
- 装置现文（`e184_regular_layer_cost_counts.rs`，sha256 见文件头）：`:2913` `SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST = 16_842_807`（注释 `:2912` 写测试文件头已是 16842815、改不改由登记定）；`:3016` `voided_cold_runs`（`:3035` X3 的小项态数那一条）；`:3055`–`:3069` `compose` 里的停机 T5（小项 `states=` 与 J1 输出里重录的建快照流全量比）；`:2918` 中项常量 167837792（锚 A8，这一趟不碰）；`:4485`–`:4493` 三个段形常量；`:4508`–`:4551` `PINNED_BY_THE_TESTS`（注释 `:4508`–`:4509` 列了取号写一轮时的旧数）；`:4553`–`:4554` 锚 B1 的 12230590578；`:4837` `run_selfcheck_command`（`:4844`–`:4885` A1、A2 两态、新池写数、A3、A4 五个锚的期望；`:4887`–`:4924` 四条冒烟流重录与全量锚；`:4925`–`:4929` B1；`:4941`–`:4949` 收尾行与退出码：停机检查有一条没过或 A 开头的锚对不上退 3，B1 不在其内）。
- 长跑脚本 `scripts/experiment-apparatus/e184-segment-one-long-run.sh`：`:9`–`:14` 件的次序与长活三样声明；`:45` 产物名；`:147`–`:157` `next_free_product_path`（带当天日期，同一天已有加 `-r2`）；`:180`–`:189` 18 件；`:94`–`:101` 随机历史六档；`:696`–`:702` 产物头尾的 `name=segment_one_begin`、`name=segment` 行。默认二进制目录 `:79` 指到 2026-10-19 那一趟的编译目录（旧钉值那一版），默认工作目录 `:84` 是上一趟的 `~/.local/share/singlefs/work/e184-segment-one`（现在还在）。
- 复跑表：`scripts/experiment-apparatus/replay.sh:1747` `E184|@driver_e184||e184-regular-layer-cost-counts-selfcheck-2026-10-23.out|exact`，驱动 `:3203` 只跑 `selfcheck`；整份第一段产物不登记逐字节复跑（`:3198`–`:3200` 注释）。
- 比两份 E184 第一段产物判定词的脚本仓里没有：`grep -rln 'e184.*compar\|e184_r2' crates scripts` 只命中 `crates/mutations.tsv`（装置 `share`、`compare` 子命令的变异行，比的是块与写流，不是产物判定词）；`grep -rln 'verdict_compar\|verdict-compar' crates scripts` 命中 `scripts/experiment-apparatus/e28_map_rebuild.rs`、`scripts/experiment-apparatus/e142_new_pool_file_creation_dry_run.rs` 各一条单测名（第 191、11047 行），只管那两个实验自己的判定（第十三节）。

## 四、跑之前已经存在的数

写判据之前我读到、算到的数，照实列。判据（第六节）没有照其中任何一个数调：第一行的判据是「逐个相同」，第二行的判据是「换没换词」，都没有可调的门槛。

- 批 16 执行员报告 `~/.local/share/singlefs/work/m3-close-b16-e184-e183-pins/result.md`（sha256 `785b98014ab5cfce5b09ebf8a195e4b993fda4437d890f9d45fcb28cb199f5a9`）：
  - 第一节：改钉值之前的装置 `selfcheck` 退 3；改后退 0，停机检查 28 条全过，锚 10 行里 9 行 `match=true`，B1 `match=false`；「e184 的 T1–T4、A1–A6 全过，B1 仍触发 F6」；「e184 第一段整份产物没重跑，判定词翻不翻面是推的」。
  - 第二节：新产物 `e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` 40 行、sha256 `24ed4a59…`，收尾 `selfcheck_done stop_checks=28 failed=0 anchors=10 anchors_mismatching=0 clause_anchor_matches=false emitted=39`；新旧 selfcheck 差 18 行（两行是照抄段的起止行号，其余是四条流的两态、全量、块数与 A1、A2、写数锚、四条全量锚、B1 的现录值）。
  - 第三节：钉值改动表（四条流的两态、全量、甲二、段数、写数，三个段形常量，中项 167837784 → 167837792，A1、A2、写数锚 16842785、16777243、43，单测 12 处）；`SNAPSHOT_CREATION_FULL_STATES_PINNED_BY_THE_TEST` 没改；原登记 A1、A2、A5、A7、X3 的旧数 16842782、16777240、41 写、16777260、46、1678377089、1678377204、278、16842807。
  - 第四节「新段形现算」：`snapshot_creation 16 64 16842785`、`new_pool [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2] 12 43 16777243`、`multi_record 20 117 1358954607`、`blocks new pool full 257`；改前收尾 `stop_checks=28 failed=6 anchors=10 anchors_mismatching=4`；拒绝消息上界 271。
  - 第五节：B1 现录 1358954642；「区间 S 下界 16777240 → 现钉 16777243；123 条随机历史与覆盖写链里离区间两端最近的一条也差 370870 … **推的**是判定词不翻；上界那条钉住的流没现量」。
- 实验页（只读了标题行、「做了什么」「复跑」「影响的决策」三节，没读读数、判决、岔路表各节）：标题「部分已跑（第一段「负载规模与重叠」已跑，G1 够判；第二到四段未跑）」；各件累计 5462 秒；改前产物各类行的条数（`stop_check` 29、`anchor` 10、`selfcheck_done` 1、`overlap` 11、`share_group` 11、`control` 174、`determinism` 12、`dump_done` 6、`pinned_smoke` 10、`flow` 127、`segment` 18）；覆盖写链 u = 1、4、16 的覆盖写次数 30、23、11，写数 1003、1042、1050；u = 16 的全量态数 2954361402816462969；「影响的决策」表的回看句：常规层「在写数轴上分得开冒烟层 14 条已钉流（最大 824 写），在态数轴上随步数档从 overlap 走到 separate」、「随机历史与冒烟层四条重录流共用块占比最大 0.001985」、「12 条 `name_not_in_key` 全部 `passed=true`」、「13 条 `family_summary` 的 `flows_beyond_u64` 都是 0」。
- 装置与测试文件注释里的旧数（第三节）：取号写一轮时 16777240、16777260、46、41 写；1678377089、1678377204、278、78 段；1358954604、1358954634、115 写；16842782、16842807。
- 我自己算的（第七节脚本，第十三节命令）：建快照流新段形 16 段、64 写、两态 16842785、全量 16842815、甲二 67（甲二没有哪里钉，不当锚）；旧段形 15 段、62 写、16842782、16842807；新池 12 段、43 写、16777243、16777268、54，旧 11 段、41 写、16777240、16777260、46；多记录（角色是推的）20 段、117 写、1358954607、1358954642，旧 1358954604、1358954634；增量路与整式路逐个相同；固定脚本只有增量路：1678377095、1678377220、294。
- `admission.py` 退 2：「准入登记里没有实验键 E184」。

对判据的影响：
1. 第一行：批 16 已经量到改后 `selfcheck` 退 0、锚 9 行 `match=true`，我的现算与装置常量逐个相同，预计「现算与自检现录逐个比」全部相同。接受反过来的结果：重跑产物里有一个锚的 `actual=` 与第七节的新数不同，就照停机 S3 两边查，不改第七节的数去凑。
2. 第二行：批 16 推的是不翻（只核了区间下界一端）。接受反过来的结果：任一格换了词就记「翻」，失败条款 FR1 照写，不回头改格的定义或键。
3. 原登记 F6 第一条（B1）在新段形下仍触发：B1 的条款数 12230590578 不变，现录 1358954642 与之不等。

## 五、臂、阳性对照、真实基线

### 5.1 臂与真实基线

- 臂 R（固定种子随机历史）、臂 F（覆盖写链）、真实基线冒烟层 R4 与区间 S：同原登记 5.1 与修订 6、10，一个字不改。这一趟的「改前」一侧就是它们在 2026-10-19 那份树上的产物（改前产物），「改后」一侧是同一套臂在今天的树上的重跑产物。
- 问题单第二行的两个候选：「不翻」= 今天的树上第一段的判定词逐格同改前；「翻」= 至少一格换了词。支持「翻」的一方会说：取号写两轮与 C755 给每条冒烟流和每段带可写挂载的随机历史都多加了系统配置槽段，区间 S 两端与每条候选流的态数都跟着动，靠近门槛的格可能跨过去；重跑用的是同一套门槛、同一套臂，不是为翻而换的取样点。

### 5.2 阳性对照（每一类判定行都跑）

- 原登记 5.2 的五个对照照旧在长跑的 `share`、`keys` 件里跑（自比、基镜像改一扇区、同流两种展开、名字不进键、转储往返；每条臂每一档，修订 10 记下的 u = 16 缺三类照旧）。任一没过，长跑退 4，原登记 X1 照办。
- 第一行（锚）的现算路自己先过一个对照：整式路在旧段形上必须算回原登记写死的旧数（建快照流 16842782、16842807；新池 16777240、16777260、46），多记录流的推的角色在旧段形上必须算回装置注释 `:4508`–`:4509` 列的旧数 1358954604、1358954634。第七节脚本输出里这几样都算回来了。
- 逐格比脚本的对照（执行员写进脚本的 `_test.py`，一条对照一个用例；「每一类」= `stop_check`、`anchor`、`selfcheck_done`、`overlap`、`share_group` 五类判定行与四条推出的失败条款）：

| 对照 | 怎么造 | 必须得到 |
|---|---|---|
| 自比 | 改前产物对它自己 | 窄读 40 格、宽读 22 格与四条推出的条款全部相同，判「不翻」 |
| 每类换一个词 | 改前产物的副本，五类里各挑一格把判定词换成同一类的另一个合法词（`pass`↔`fail`、`true`↔`false`、`overlap`→`separate`、`renamed`→`partial` 这类），一类一份副本 | 恰好那一格不同、判「翻」并点名那一格，其余格全部相同 |
| 只改数值字段 | 副本里改一格 `stop_check` 的 `detail=` 与一格 `anchor` 的 `actual=`，判定词不动 | 判「不翻」 |
| 少一格、多一格 | 副本删掉一行 `name=anchor`；另一份副本把一行 `name=overlap` 复制一遍 | 都报键不齐（停机 S4），不给「翻 / 不翻」 |
| 推出的条款 | 副本把 `scope=family:random_history_main` 的 `share_group` 判定词换掉（改前是 `renamed` 就换成 `independent`，否则换成 `renamed`；脚本按改前产物现读，不预设） | F2 推出值换边、判「翻」 |
| F6 第二条的边界 | 手造两行 `name=overlap scope=family:random_history_main`，`at_most_smoke_interval_maximum_fraction=0.500000` 与 `0.499999` | 前一行推出 F6 第二条为真，后一行为假 |
| 行首前缀 | 改前产物的副本把每行行首的 `E7RESULT ` 去掉 | 与原样比判「不翻」，格数不变 |

### 5.3 跑法

起跑前核五样（主 agent 起长跑之前，命令原样；任一不成立照停机 S1 办）：

```bash
# 仓根
sha256sum crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts_write_dump.rs scripts/experiment-apparatus/e184-segment-one-long-run.sh
#   P1 三个值依次等于文件头写的 8f529e12…、320822c4…、8f9617a3…；变了的在第十二节记一行新值与 git diff 的摘要，主 agent 认了再起
sha256sum research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out
#   P2 等于 7eacaffaddd44d34bf4a67afe29214ccbbf7ffbd63663d61ef7489124e858c06
test ! -e "$HOME/.local/share/singlefs/work/e184-segment-one-r2" && echo fresh
#   P3 打出 fresh（工作目录全新，不沿用上一趟任何一件）
E184_WORK_DIRECTORY="$HOME/.local/share/singlefs/work/e184-segment-one-r2" bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --list-segments
#   P4 列出 18 件、每件都没做完（不起装置、不编译）
grep -rn "CrashFlow" crates/singlefs-checker-tier/src/crash_injection.rs crates/singlefs-harness/src
#   P5 零命中（原登记停机 T6；长跑的 repo-reads 件另核一次）
```

起长跑（主 agent；预计超过 2 小时，照实验页「复跑」一节的起法，只多一个新工作目录）：

```bash
bash scripts/hosts/detached-run.sh start --wall-clock over-2h --label e184-segment-one-r2 -- env E184_WORK_DIRECTORY="$HOME/.local/share/singlefs/work/e184-segment-one-r2" bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --build-here
```

- `--build-here` 用这棵树编出的两份 bin；主 agent 改用 `E184_BINARY` 指到 `compile-then-swap.py` 编出的二进制的，在第十二节记一行（哪一份、sha256），判据不变。不许用脚本的默认二进制目录（`:79`，旧钉值那一版）。
- 多机照脚本文件头「多机的读法」（`:15`–`:28`）；第二台的块键并进来时装置 sha256 要与本机相同，不同的件被脚本报作废、本机现跑。
- 重跑产物名：脚本自己取（`:147`–`:157`）：2026-10-24 拼出的是 `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out`；别的日期拼出的，执行员在第十二节记实际的名字，比对对象表与机读登记照实际的名字写。
- 长跑退 3（停机）或 4（阳性对照没过）：不重跑，原样交主 agent（停机 S2、作废 X1）。

逐格比（执行员；长跑做完之后）：

```bash
python3 scripts/experiment-apparatus/e184_r2_verdict_compare_test.py
python3 scripts/experiment-apparatus/e184_r2_verdict_compare.py research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out research/results/e184-regular-layer-cost-counts-segment-one-2026-10-24.out > research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-2026-10-24.out
```

脚本只读两份产物，不起装置。输出一行一格，写法写死：

- `name=cell family=<stop_check|anchor|selfcheck_done|overlap|share_group> key=<键> before=<词> after=<词> same=<true|false>`
- `name=derived clause=<F1|F2|F3|F6_second> before=<true|false> after=<true|false> same=<true|false>`
- `name=anchor_recompute identifier=<名> recomputed=<第七节的新数> expected=<产物里的 expected=> actual=<产物里的 actual=> same=<true|false>`（10 格，第一行用；B1 那一格的 `recomputed=` 是多记录流全量的现算值）
- `name=void_check family=<control|determinism|dump_done|pinned_smoke> cells=<格数> failing=<个数>`（重跑产物上 `passed=false`、`equal=false`、`base_mismatches` 不为 0、`match=false` 的个数）
- `name=verdict_comparison reading=<narrow|wide> cells=<格数> differing=<个数> verdict=<flipped|not_flipped|keys_differ>`（窄读一行、宽读一行）
- 末行 `name=done emitted=<行数>`。

### 5.4 分段

只有一段（第一段重跑）加它之后的逐格比，没有分段的必要：问题单两行都在这一段里够判。

| 步 | 谁 | 做什么 | 补问题单哪一行 | 产物 |
|---|---|---|---|---|
| 1 | 执行员 | 写逐格比脚本三件（`e184_r2_verdict_compare.py`、`_test.py`、`.mutations.tsv`，照 `e185_r1_compare*` 的写法），跑 `_test.py`（5.2 对照），在改前产物上跑一次自比 | — | 三份脚本；自比输出进草稿目录 |
| 2 | 主 agent | 核 P1–P5，起长跑（可与第 1 步同时） | — | `research/results/e184-regular-layer-cost-counts-segment-one-<日期>.out` |
| 3 | 执行员（续派） | 核产物完整（末行 `name=done emitted=` 等于行数）；读 `selfcheck` 件；跑逐格比；变异表跑法照 `e185_r1_compare.mutations.tsv` 的先例 | 第一、二行 | `research/results/e184-regular-layer-cost-counts-segment-one-r2-verdict-comparison-<日期>.out` |

## 六、报哪些量与各自的判据

每个量各占一行、各报各的判定。这一趟没有可调的门槛：第一行的判据是逐个相等，第二行的判据是同一格的词换没换；臂 R、F 与 R4 的定义只说负载怎么造，推不出哪一格在今天的树上取什么词。原登记第六节的 J1-甲、J1-乙、J1-丙 照原判据在重跑产物里重判（长跑的 `share` 件），J1-库 与 J2、J3、J4 各量这一趟不跑。

| 量 | 怎么算 | 判据与判定 | 问题单行 | 翻面取值 |
|---|---|---|---|---|
| 新旧锚并列 | 第七节表：A1、A2、A5（三条流）、A7、X3 各一行，旧数（原登记行号）、新数、装置现文行号、整式路、增量路两条现算的数；A6 多记录流附带一行 | 两条现算路的数相同、且等于装置现文的钉值记「新数定」；两条路不同或与装置不同记「新数未定」，照停机 S3 两边查。写这一份时已算完：全部「新数定」，固定脚本只有增量路（整式路「现算不出」：80 段的段形测试文件与装置都没钉，缺每段的写数与角色） | 第一行 | — （这是第一行要的数本身） |
| 现算与自检现录逐个比 | 逐格比脚本的 `name=anchor_recompute`：重跑产物 `selfcheck` 件 10 行 `name=anchor` 的 `actual=` 与第七节新数逐格比；另比 `expected=`（装置钉值）与新数 | 10 格都 `same=true` 记「逐个相同」；有一格不同记「有一边错」 | 第一行 | 有一格不同：第一行的翻面观测成立（装置或现算有一边错），第一行不够判，照停机 S3 两边查 |
| 自检退出码 | 重跑产物里 `selfcheck` 件：`name=segment segment=selfcheck state=done`，收尾 `name=selfcheck_done … failed=<n> anchors_mismatching=<m>` | `failed=0` 且 `anchors_mismatching=0` 记「退 0」；否则记「退 3」（长跑在这一件停下，停机 S2） | 第一行 | 「退 3」：第一行的够判条件「装置 selfcheck 退 0」不成立 |
| 停机与锚判定词 | 逐格比脚本：`stop_check` 29 格（`result=`）、`anchor` 10 格（`match=`）、`selfcheck_done` 1 格（`clause_anchor_matches=`），共 40 格；`name=verdict_comparison reading=narrow` | 40 格的键两边一样、每格的词都相同记「不翻」；键一样而有一格以上换词记「翻」并逐格列；键不一样记「键不齐」（停机 S4，不判翻不翻） | 第二行（窄读、宽读都用） | 「翻」：第二行翻向「翻」；其中 `clause_anchor_matches` 由 `false` 变 `true` 即 B1 不再触发 F6 第一条 |
| 负载判定词 | 逐格比脚本：`overlap` 11 格（J1-乙 `verdict=`）、`share_group` 11 格（J1-丙 `verdict=`），共 22 格；四条推出的条款：F1 =（`scope=family:random_history_main` 与 `scope=family:overwrite_chain_main` 两行 `overlap` 都是 `overlap`，且两行 `above_fraction` 都 < 0.5）；F2 =（`share_group scope=family:random_history_main` 是 `renamed`）；F3 =（`share_group scope=family:overwrite_chain_main` 是 `renamed`）；F6 第二条 =（`overlap scope=family:random_history_main` 的 `at_most_smoke_interval_maximum_fraction` ≥ 0.5）；`name=verdict_comparison reading=wide` 把这 22 格、四条条款与上一行的 40 格合起来判 | 同上一行：键一样、词全同记「不翻」；有一格或一条条款换边记「翻」并逐格列；键不一样记「键不齐」 | 第二行（宽读） | 「翻」：第二行翻向「翻」；F1、F2、F3、F6 第二条有一条换边，原登记第十节那一条失败条款的判定跟着换 |
| 离门槛的距离 | 每格 `overlap`：`inside_fraction` 到 0.1 与 0.5 的差（改前、改后各一）；每格 `share_group`：`median_share_fraction` 到 0.5 的差、`maximum_share_fraction` 到 0.05 的差 | 附带，够判后不跑；只在「负载判定词」记「翻」、主 agent 要看翻在哪一边时跑 | （第二行旁证） | — |

第一行的候选甲、乙不由这几行的数翻面（1.2 节）；这几行只回答「数是多少、和装置现录对不对得上」。

## 七、钉绝对值的断言

### 7.1 出自被测条款本身的（不符时两边都查；都核过仍不符，记「条款这一格可能错」，败原登记 F6）

- B1：D13（验证路线） 已定项 9 射程的 12230590578，这一趟不改（条款没改）。重跑产物里多记录流全量的现录值预计 1358954642（7.2 附带那一行），与 12230590578 不等，F6 第一条照旧触发；这是原登记第四节就写下的预计，不作废。
- B2、B3、B4：同原登记 7.1，不改。

### 7.2 独立算出、用命令核过的（不符先查现算，再查装置，两边都查完仍不符按停机 S3 办）

新旧两数并列。旧数原样抄自原登记（行号是原登记的），原登记一个字不改；新数是这一趟现算的，两条现算路与装置现文都列出来。整式路：1 + Σ 各段的态数，系统配置槽 2 写段（下表记 C）全量与甲二 3² − 1 = 8、两态 2² − 1 = 3；记录 2 写段 3；根槽 1 写段 1；单元段全量与两态 2^n − 1、甲二 1。增量路：旧数 + 新段形多出来的系统配置槽 2 写段数 × 增量（两态 3、全量 8、甲二 8）。

| 锚 | 原登记原文 | 旧数 | 新数 | 装置现文 | 整式路（式子） | 增量路 | 重跑产物里核哪一格 |
|---|---|---|---|---|---|---|---|
| A1 建快照流两态 | 原登记:473 | 16842782（15 段、62 写） | **16842785**（16 段、64 写） | 段形 `:4486`–`:4487`；钉值 `:4544`；锚期望 `:4847` | 段角色照测试文件 `:46`–`:47`：C、C、R2、T1、C、R2、T1、C、U24、R2、T1、C、U16、R2、T1、C；1 + 10 × 3 + 4 × 1 + (2^24 − 1) + (2^16 − 1) = 16842785 | 16842782 + 1 × 3 = 16842785 | `name=anchor identifier=snapshot_creation_two_state` |
| A2 新池两态与写数 | 原登记:474 | 16777240，建快照流前 11 段，41 写 | **16777243**，建快照流前 12 段，**43** 写 | 段形 `:4489`；钉值 `:4514`、`:4519`；锚期望 `:4854`、`:4861` | 前 12 段：1 + 8 × 3 + 3 × 1 + (2^24 − 1) = 16777243；写数 2 × 8 + 1 × 3 + 24 = 43 | 16777240 + 3；41 + 2 | `identifier=new_pool_two_state`、`identifier=new_pool_writes` |
| A5 新池全量、甲二 | 原登记:477 | 全量 16777260、甲二 46 | 全量 **16777268**、甲二 **54** | `:4515`–`:4516` | 全量 1 + 5 × 8 + 3 × 3 + 3 × 1 + (2^24 − 1) = 16777268；甲二 1 + 5 × 8 + 3 × 3 + 3 × 1 + 1 = 54 | 16777260 + 8；46 + 8 | `identifier=full_states_smoke-new_pool_file_creation_stream`；两态、甲二在 `stop_check item=recorded_values_match_the_test_pins_smoke-new_pool_file_creation_stream` |
| A5 固定脚本到卸载 | 原登记:477 | 两态 1678377089、全量 1678377204、甲二 278（装置另钉段数 78，原登记 A5 没写） | 两态 **1678377095**、全量 **1678377220**、甲二 **294**、段数 **80** | `:4524`–`:4528` | 现算不出：80 段的段形与每段的角色测试文件与装置都没钉（`segment_sizes: None`），缺每段写数与角色；另一条路只算全量：两态 + 25 个系统配置槽段 × 5 = 1678377095 + 125 = 1678377220（段数 25 抄自测试文件 `crash_enumeration_fixed_script_stream.rs:102`–`:104`） | 两次取号各多一个 C 段（测试文件 `:99`–`:100`）：两态 +6、全量 +16、甲二 +16、段数 +2 | `identifier=full_states_smoke-overwrite_fixed_script_stream`；两态、甲二、段数在 `stop_check item=recorded_values_match_the_test_pins_smoke-overwrite_fixed_script_stream` |
| A5 建快照流 | 原登记:477 | 段形同旧 A1、两态 16842782、全量 16842807 | 段形同新 A1、两态 16842785、全量 **16842815** | `:4544`–`:4547` | 全量 1 + 6 × 8 + 4 × 3 + 4 × 1 + (2^24 − 1) + (2^16 − 1) = 16842815 | 16842807 + 8 | `identifier=full_states_smoke-snapshot_creation_stream`；段形在 `stop_check item=recorded_values_match_the_test_pins_smoke-snapshot_creation_stream` |
| A7 | 原登记:479 | J2 小项 `states=` = 16842807 = 重录建快照流全量 | J2 小项 `states=` = **16842815** = 重录建快照流全量 | 装置里没有 A7 自己的常量：前半由 `compose` 的停机 T5 比（`:3055`–`:3069`），后半是 A5 建快照流全量的钉值 `:4545` | 同 A5 建快照流全量 | 同上 | 这一趟只核后半（同 A5 建快照流那一格）；前半要第三段的 J2 冷跑，这一趟不跑 |
| X3（小项那一条） | 原登记:547 | 小项 `states` ≠ 16842807 即作废 | 小项 `states` ≠ **16842815** 即作废；X3 其余三条（`reused`、包含自检、单调）不变 | `:2913` 常量仍是 16842807（没改，第一节未定项），用在 `:3035` | 同 A5 建快照流全量：1 + 6 × 8 + 4 × 3 + 4 × 1 + (2^24 − 1) + (2^16 − 1) = 16842815 | 16842807 + 8 = 16842815 | 这一趟不跑 `compose`，X3 用不到 |
| A6 多记录流（附带；原登记 A6 没写数） | 原登记:478 | 19 段、115 写、两态 1358954604、全量 1358954634（装置注释 `:4508`–`:4509`） | 20 段、117 写、两态 1358954607、全量 1358954642 | `:4491`–`:4493`、`:4534`–`:4539` | 段角色是推的（照建快照流类推：取号两轮，暖机两次与 A、B、C 各「记录、根槽、轮换」，B、C 的记录段 4、6 写）：两态 1 + 10 × 3 + 5 × 1 + (2^24 − 1) + (2^28 − 1) + 15 + (2^30 − 1) + 63 = 1358954607；全量多 7 个 C 段 × 5 = 1358954642 | +3、+8 | `identifier=full_states_smoke-multi_record_publish_stream`、`identifier=multi_record_full_named_by_the_clause` 的 `actual=` |
| 比对的格数 | — | 改前产物：窄读 40 格（`stop_check` 29、`anchor` 10、`selfcheck_done` 1）、宽读另 22 格（`overlap` 11、`share_group` 11）；条数抄自实验页「复跑」一节的 `uniq -c` 原样输出 | 重跑产物同样 40 与 22 | — | — | — | 逐格比脚本的 `cells=`；两份不同照停机 S4 |

X3 的新值现算得出（两条路都是 16842815），不抄装置（装置那个常量这一刻仍是 16842807）。两条路共用一个前提：取号写两轮多出来的那一段是系统配置槽段、两次写都取三态（测试文件 `crash_enumeration_snapshot_creation_stream.rs:15`、`:46`；D13（验证路线） 已定项 4「系统配置槽写与覆盖旧记录的 journal 写属于这一类」；`crates/singlefs-checker-tier/src/crash.rs:2275`、`:2278`、`:2298`）。整式路多一个前提：这条流上 4 个记录段的写都不取三态（环里没有被覆盖的旧记录）；这一条我不录流就核不了，由重跑产物的 `full_states_smoke-snapshot_creation_stream` 那一格核。整式路在旧段形上算回原登记的旧数，是这套角色读法的对照（5.2）。

**修订记录（候选甲要往原登记第十二节追加的就是这一段与上表；这一份里它就是新数的出处）**：写于 2026-10-24，第一段重跑之前（这一趟一件都没跑）。改了什么：原登记第七节 A1、A2、A5、A7 与第十一节 X3 小项那一条的数，换成上表「新数」一列；判据、门槛、锚的意思一个字不改；原登记原数原样保留。依据：装置 `e184_regular_layer_cost_counts.rs` 现文（sha256 `8f529e12…`）的行号；批 16 执行员报告（sha256 `785b9801…`）第一、三、四、五节；四份冒烟测试文件的注释与钉值（第三节）；下面这份现算脚本与它的输出。

现算脚本（草稿目录 `recompute.py`，sha256 `a1d3c50a2d032d29a8d92693ebc52130f4c6354a076f219f124b0f203ab3eca4`，原样）：

```python
# E184 r2 登记的独立现算：段形与段的角色抄自测试文件的注释，按 D13 已定项 4 的口径算两态、全量、甲二；不读装置。
# 角色：C = 系统配置槽 2 写段（两次写都是原地覆写，三态）；R = journal 记录段（两态）；T = 根槽 1 写段（两态）；U = 单元段（COW，两态）。
def two_state(seg):  # 每次写两态
    return 2 ** seg[1] - 1
def full(seg):
    kind, n = seg
    return (3 ** n - 1) if kind == 'C' else (2 ** n - 1)
def quick(seg):  # 甲二：COW（单元）段只取全不落 1；其余段同全量
    kind, n = seg
    return 1 if kind == 'U' else full(seg)
def total(f, segs):
    return 1 + sum(f(s) for s in segs)
C2, R2, T1 = ('C', 2), ('R', 2), ('T', 1)
# 建快照流，取号写两轮（测试文件第 46–47 行：取号两轮各 2、暖机两次（记录 2、根 1、轮换 2）、A 24；A 的记录 2、根 1、轮换 2；S 16、记录 2、根 1、轮换 2）
snap_new = [C2, C2, R2, T1, C2, R2, T1, C2, ('U', 24), R2, T1, C2, ('U', 16), R2, T1, C2]
# 取号写一轮：开头只有一个取号段
snap_old = snap_new[1:]
new_pool_new = snap_new[:12]
new_pool_old = snap_old[:11]
for name, segs in [('snapshot_new', snap_new), ('snapshot_old', snap_old), ('new_pool_new', new_pool_new), ('new_pool_old', new_pool_old)]:
    print(name, 'segments', len(segs), 'sizes', [n for _, n in segs], 'writes', sum(n for _, n in segs),
          'two_state', total(two_state, segs), 'full', total(full, segs), 'quick_tier', total(quick, segs))
# 增量路：多一个系统配置槽 2 写段，两态 +3、全量 +8、甲二 +8（原登记写死的旧数 + 增量）
for name, old, added in [('snapshot', (16842782, 16842807, None), 1), ('new_pool', (16777240, 16777260, 46), 1),
                          ('fixed_script', (1678377089, 1678377204, 278), 2), ('multi_record', (1358954604, 1358954634, None), 1)]:
    print('delta', name, 'added_system_configuration_segments', added, 'two_state', old[0] + 3 * added, 'full', old[1] + 8 * added,
          'quick_tier', (old[2] + 8 * added) if old[2] is not None else 'not_pinned')
# 固定脚本另一条路：全量 = 两态 + 系统配置槽段数 × (3² − 1 − (2² − 1))，段数取测试文件第 102–104 行写的 25（取号写一轮 23）
print('fixed_script_full_by_segment_count new', 1678377095 + 25 * 5, 'old', 1678377089 + 23 * 5)
# 多记录流（锚 A6，附带）：段形抄自装置以外的测试文件 prepare 钉的写法不在这里读；角色照建快照流类推（推的）：
# 取号两轮、暖机两次与 A、B、C 三次发布各「记录段、根槽段、轮换段」，B、C 的记录段 4、6 写（两盘各一份、每单元一条）。
multi_new = [C2, C2, R2, T1, C2, R2, T1, C2, ('U', 24), R2, T1, C2, ('U', 28), ('R', 4), T1, C2, ('U', 30), ('R', 6), T1, C2]
multi_old = multi_new[1:]
for name, segs in [('multi_record_new', multi_new), ('multi_record_old', multi_old)]:
    print(name, 'segments', len(segs), 'sizes', [n for _, n in segs], 'writes', sum(n for _, n in segs),
          'two_state', total(two_state, segs), 'full', total(full, segs))
```

输出（`nice -n 19 python3 recompute.py`，退 0，原样）：

```text
snapshot_new segments 16 sizes [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2] writes 64 two_state 16842785 full 16842815 quick_tier 67
snapshot_old segments 15 sizes [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 16, 2, 1, 2] writes 62 two_state 16842782 full 16842807 quick_tier 59
new_pool_new segments 12 sizes [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2] writes 43 two_state 16777243 full 16777268 quick_tier 54
new_pool_old segments 11 sizes [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2] writes 41 two_state 16777240 full 16777260 quick_tier 46
delta snapshot added_system_configuration_segments 1 two_state 16842785 full 16842815 quick_tier not_pinned
delta new_pool added_system_configuration_segments 1 two_state 16777243 full 16777268 quick_tier 54
delta fixed_script added_system_configuration_segments 2 two_state 1678377095 full 1678377220 quick_tier 294
delta multi_record added_system_configuration_segments 1 two_state 1358954607 full 1358954642 quick_tier not_pinned
fixed_script_full_by_segment_count new 1678377220 old 1678377204
multi_record_new segments 20 sizes [2, 2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2] writes 117 two_state 1358954607 full 1358954642
multi_record_old segments 19 sizes [2, 2, 1, 2, 2, 1, 2, 24, 2, 1, 2, 28, 4, 1, 2, 30, 6, 1, 2] writes 115 two_state 1358954604 full 1358954634
```

## 八、轨迹与几何敏感性

### 8.1 轨迹

这一趟没有新的被谓词消费的量：第一行是固定流上的定数，第二行比的是两个端点（改前、改后）的判定词，不说轨迹，正文不写「从来」「恒」。原登记 8.1 的三样（J3-乙、J2-乙 丁、J1-甲 按步数档）里只有 J1-甲 在第一段产物里，照原登记在重跑产物里按步数档报，这一趟不另判。

### 8.2 几何敏感性

- 第一行：纯算术，没有几何旋钮，不适用（段形是今天的树录出来的定数，不是取样点）。
- 第二行：逐格比本身就罩住原登记 8.2 的全部取样点——`overlap`、`share_group` 的 11 个范围是主取样（`family:random_history_main`、`family:overwrite_chain_main`）加每个取样组各一个范围（盘宽 `UnitAreaOf240Slots`、比重 `BROAD_WITH_COMPACTION_WHILE_MOUNTED`、步数 50 与 200、400，覆盖写链 u = 1、16 这些方向相反的点）。每个范围各报各的换没换词，不合并；「翻」只发生在一部分范围上时，逐个范围点名，不写成「第一段翻了」一句了事。
- 「翻 / 不翻」这条判据没有数值门槛可挪，原登记第六类「把门槛挪到两点之间」的判别力自证在这里换成 5.2 的「每类换一个词」对照：同一套比法在一格换词的副本上必须由「不翻」转「翻」。

## 九、变异

原登记第九节与修订 4、9、10 的变异（装置的 V1–V27）这一趟不动、不重跑。下面是逐格比脚本的变异，执行员照 `scripts/experiment-apparatus/e185_r1_compare.mutations.tsv` 的写法写进 `e184_r2_verdict_compare.mutations.tsv`（原文在脚本里恰好命中一次，替换之后 `_test.py` 里点名的那条用例红）：

| 变异 | 改什么 | 在哪个取样点上改变输出（哪条对照红） |
|---|---|---|
| M1 键不带范围 | `overlap`、`share_group` 的键只取行名、不取 `scope=` | 自比：宽读格数从 22 掉到 2（同名覆盖），格数断言红；「每类换一个词」的 `share_group` 副本点名错格 |
| M2 比数值字段 | `stop_check` 比 `detail=` 而不是 `result=` | 「只改数值字段」副本判成「翻」 |
| M3 缺格当相同 | 一边缺的键当 `same=true` | 「少一格」副本不报键不齐 |
| M4 重复键后写盖前写 | 同一份产物里同一把键出现两次时不拒、取后一行 | 「多一格」副本不报键不齐 |
| M5 F6 第二条用开区间 | `>= 0.5` 写成 `> 0.5` | 「F6 第二条的边界」0.500000 那一行推出为假 |
| M6 F1 丢掉 above 那半句 | F1 只看两行都是 `overlap` | 手造两行 `overlap`、其中一行 `above_fraction=0.600000`：F1 应为假，变异后为真 |
| M7 窄读带上负载格 | `reading=narrow` 的格里混进 `overlap` | 自比：窄读 `cells=` 从 40 变 51 |
| M8 前缀不剥 | 行首 `E7RESULT ` 不去掉就取行名 | 「行首前缀」副本：带前缀那一份一格都认不出，键不齐 |
| M9 新数抄成旧数 | `anchor_recompute` 里建快照流全量的现算值写成 16842807 | 手造一行 `name=anchor identifier=full_states_smoke-snapshot_creation_stream expected=16842815 actual=16842815 match=true`：`same=` 应为真，变异后为假 |

等价变异照 `.claude/rules/mutation-sampling.md` 办：没红先换取样点，确认所有输入上同值才写成等价性用例留档。

## 十、失败条款

这几条是正当结果，如实记下，不作废。

- **FR1（推的「不翻」被推翻）**：「停机与锚判定词」或（宽读下）「负载判定词」记「翻」。触发它的观测：逐格比输出里 `name=verdict_comparison` 那一行（窄读或宽读）`verdict=flipped`、`differing=` 不为 0。后果：实验页「影响的决策」表 2026-10-23 那几句回看（「第一段整份产物没重跑、判定词推的不翻」）不再成立，由主 agent 回看四条决策分项；翻在哪几格逐格列出，不归因到段形（1.1 节）。
- **FR2（B1 不再触发 F6）**：`selfcheck_done` 的 `clause_anchor_matches=` 由 `false` 变 `true`。触发它的观测：重跑产物里 `name=anchor identifier=multi_record_full_named_by_the_clause` 的 `match=true`，即现录多记录流全量等于 12230590578。
- **FR3（原登记 F1、F2、F3、F6 第二条换边）**：四条推出的条款里有一条 `same=false`。触发它的观测：逐格比输出 `name=derived clause=<名> … same=false`。后果照原登记第十节那一条的字面判，新判定写进实验页，旧判定留在历史。
- 原登记 F1–F7 其余字面不改；F4、F5、F7 这一趟碰不到（J2、J3、J4 不跑）。

## 十一、作废条款与停机条款

### 作废（装置、跑法或测量有问题，那一格或整段不出结果）

- **XR1**（原登记 X1）：长跑退 4，或逐格比的 `name=void_check family=control` 报 `failing=` 不为 0。整份重跑产物作废，修装置后照 5.3 换新工作目录重跑。
- **XR2**（原登记 X2）：`name=void_check family=determinism` 的 `failing=` 不为 0（种子 0、1 两遍的写表摘要不同）。随机历史那几格（`overlap`、`share_group` 里随机历史的范围）作废。
- **XR3**：`name=void_check family=dump_done` 的 `failing=` 不为 0（有转储重建的基镜像与录制流对不上，修订 3）。那一档作废。
- **XR4**：重跑产物不完整：末行不是 `name=done emitted=<行数>`，或 18 件里有一件的 `name=segment` 行不是 `state=done`（`library` 写 `state=not_run reason=no_family_share_verdict_partial` 算完整）。不比，同一条命令重起续跑。
- **XR5**：逐格比脚本的 `_test.py` 有一条没过（5.2 的对照）。逐格比输出作废，重跑产物不作废。
- **X6、X7**：同原登记（读不到不当 0；内存包装退 245 或 250–254 的那一次不算结果）。

### 停机（两边都查，既不作废也不当结果；`.claude/rules/implementation-first.md` 第 4 条）

- **S1**：起跑前五样（5.3 的 P1–P5）有一样不成立：装置或长跑脚本的 sha256 变了而第十二节没有主 agent 认过的记录；改前产物的 sha256 不是 `7eacaff…`；工作目录不是新建的；`--list-segments` 不是 18 件、或有件已做完；停机 T6 有命中。
- **S2**：长跑退 3：`selfcheck` 件里停机 T1–T4 有一格 `fail`、A 开头的锚有一行 `match=false`，或 `repo-reads` 件报钉住的常量对不上、T6 有命中。照原登记 T1–T4、T6 的字面两边查。这时第一行的「自检退出码」记「退 3」。
- **S3**：`name=anchor_recompute` 有一格 `same=false`：第七节的现算值、装置钉值（`expected=`）、现录值（`actual=`）三个里有一个与另两个不同。先查现算（角色读法、增量路的段数），再查装置与 `crates/`；查完之前第七节的数不改、第一行不判。
- **S4**：两份产物的格对不齐：窄读或宽读的键集合不同（`verdict=keys_differ`），或同一份产物里同一把键出现两次。说明产物的行形变了（装置或长跑脚本与改前那一版的输出不是同一套键），两边查哪一边改了行形，不判翻不翻。

### 够判停机

- 第一行够判：「新旧锚并列」全部「新数定」（写这一份时已成立），「现算与自检现录逐个比」10 格全部相同，「自检退出码」记「退 0」。
- 第二行够判：XR1–XR5 都没触发，「停机与锚判定词」（宽读另加「负载判定词」）记出「翻」或「不翻」。
- 两行都够判就停；「离门槛的距离」标「够判后未跑」，除非 FR1 触发、主 agent 点名要看。原登记第二到四段这一趟一律不跑，标「这一趟不罩」。

### 射程：这一份答不了的

- 翻了的格是取号写两轮、C755 引起的，还是 2026-10-19 以来 `crates/` 别的改动引起的，这一趟分不开（1.1 节）。
- J2 的锚 A7 前半、X3 在新段形下是否真按 16842815 判，要等第三段；这一趟只把数定下来。
- 原登记第四节第 1 条说的「J1-乙 很可能判重叠」之类的预计，这一趟不重新预计；只比改前、改后两份产物。

### 比对对象（这一次要与之比的上一次产物）

| # | 比对对象 | sha256 | 行数 | `replay.sh` 行 / 跑法 | 重跑产物名 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `e184-regular-layer-cost-counts-segment-one-2026-10-19.out` | `7eacaffaddd44d34bf4a67afe29214ccbbf7ffbd63663d61ef7489124e858c06` | 950 | `replay.sh` 里没有整份第一段的行（`:3198`–`:3200` 注释：带挂钟、不登记逐字节复跑）；跑法 5.3「起长跑」 | `e184-regular-layer-cost-counts-segment-one-2026-10-24.out` | 问题单第二行：窄读 40 格、宽读另 22 格与四条推出的条款逐格比判定词（第六节） |
| 2 | `e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` | `24ed4a59db6a04601178451d72c0c80131acbac889cdb39d589b4d8cdbce229c` | 40 | `replay.sh:1747` 那一行（驱动 `driver_e184`，只跑 `selfcheck`，期望产物就是这一份，逐字节比）；这一趟走 5.3 长跑的 `selfcheck` 件 | `e184-regular-layer-cost-counts-segment-one-2026-10-24.out` | 问题单第一行：重跑产物 `selfcheck` 件的 10 行 `name=anchor` 与这一份逐行比 `actual=`、`match=`，只报；判据是第六节「现算与自检现录逐个比」 |

重跑产物名里的日期是写登记这一天；长跑拼产物那一天不同的，执行员在第十二节记新名，`.claude/gate.d/replay-comparison-units.tsv` 照新名登记。E184 没有单元表，两行都要在那份表的文件头写「# 不登记的比对对象：research/prompts/e184-r2-prereg.md #<号>：<理由>」，理由里要有一个开着的欠账号（第一节未定项）；写之前门禁 doc-experiments 的 replay-comparison-units 格判这份登记红。

## 十二、修订

- 2026-10-24（第一段重跑之前、比较脚本单测之后，依据 `~/.local/share/singlefs/work/m3-close-b17-e184-r2-compare/result.md` 第四节与改前产物自比 `self-compare-2026-10-19-vs-itself.out`；跑前判据不改，只把脚本的四处读法写死）：
  1. 宽读 `verdict_comparison` 的 `differing=` 是换词的格数加换边的推出条款数，`cells=` 只数格（62）。
  2. `anchor_recompute` 的 `same=` 在条款锚 `multi_record_full_named_by_the_clause` 上只比 `actual=` 与第七节新数；它的 `expected=` 是条款写的 12230590578，按登记预计与现算不等。
  3. 行的认法：去掉可有可无的 `E7RESULT ` 前缀后，第一个字段必须是 `name=`（比 1.1「第一个以 `name=` 开头的字段」窄；改前产物 950 行全是这个形状，两种认法结果相同）。
  4. XR4 的产物完整性检查进了脚本：不完整退 2、不比（收严）。
- 2026-10-24（第一段重跑起跑之后、第一件 repo-reads 停机之后，产物一件都没出）：长跑第一趟 repo-reads 退 3，钉值冒烟 `pinned=12 missing=21 mismatches=1`，对不上的是 `large_file_overwritten_across_lower_leaves_stream`（2026-10-21 进仓，钉的是展开段的另一个量 `CLOSED_FORM_OF_THE_EXPANDED_SEGMENTS_WITH_TWO_STATES_PER_WRITE`，脚本两种钉法都认不出，记成 `pinned_value=none … match=false`）；停机输出存 `~/.local/share/singlefs/work/m3-close-b17-e184-r2-smoke-fix/stopped-run-evidence/`。改脚本：段形在、两种钉法都认不出的流记缺钉值（`reason=segment_shape_present_but_no_pinned_two_state_closed_form`），不算 mismatch、不进钉表；认出钉值而与现算不等的照旧停机（收严不变，只补一类认法）。P1 第三个值由 8f9617a3… 改为 c1f9176e2e4db634bf13f1e046afaecc61a3adead18cd33fe77a1aa73d1bb2d2（补丁 `~/.local/share/singlefs/work/m3-close-b17-e184-r2-smoke-fix/patch/e184-long-run-smoke.patch`，379935dd…；自证 53 格全绿，开关 `missing-pinned-as-mismatch` 恰好红新加的两格）。旧工作目录（503M，没有做完的件）删掉，照 P3 用全新的同名目录重起。

## 十三、读过的文件与跑过的命令

### 13.1 读过的文件（行号区间；grep 命中行也算）

- 规则与共用约束：`.claude/agent-common.md` 1–112；`.claude/singlefs-ai-sop/rules/test-discipline.md` 1–122；`.claude/rules/three-way-inference.md` 1–166；`.claude/singlefs-ai-sop/rules/evidence-discipline.md` 1–179；`.claude/rules/mutation-sampling.md` 1–93；`.claude/rules/implementation-first.md` 1–19；`.claude/rules/test-script-discipline.md` 1–106；`.claude/singlefs-ai-sop/rules/writing-discipline.md` 1–133；`.claude/singlefs-ai-sop/rules/verify-before-claiming.md` 1–53；`.claude/singlefs-ai-sop/rules/command-safety.md` 1–162。
- 问题单与原登记：`research/prompts/m3-close-e184-r2-questions.md` 1–8；`research/prompts/e184-preregistration.md` 1–745。
- 读数来源：`~/.local/share/singlefs/work/m3-close-b16-e184-e183-pins/result.md` 节标题（grep）、1–92（第一至五节）；同目录 `.deliverables`、`progress.md` 全文。没读同目录 `e184-before.out` 与补丁。
- 实验页 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md`：节标题（grep，含第 1 行标题）、23–79（「做了什么」「复跑」）、390–398（「影响的决策」）。读数、判决、岔路表、它答不了的、路径与结论登记各节没读。
- 装置：`crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` 2900–3109、4460–4979，grep 命中行（2913、2918、3016、3035、3037、3095、4510、4837、4888–4891、6712、6799、7051、7071、7087、7346；560、929、1547–1549、1576–1578、1804、1895、2008、2020、2420、2654、3171、3180、5469–5501、5555、5720、6123、6129、6490；1402、1419、1940、1948、7324），`git diff HEAD` 对这份文件的全部差（57 增 54 删）；`e184_regular_layer_cost_counts_write_dump.rs` grep 命中行 15、55、450、1078、1085、1316。
- 测试文件（`crates/singlefs-checker-tier/tests/`）：`crash_enumeration_snapshot_creation_stream.rs` 1–60、236–250 与 grep 命中行（13、14、48、180、226、247、283）；`crash_enumeration_new_pool_file_creation_stream.rs` 750–800 与 grep 命中行（772–781、842–998 里 15 行）；`crash_enumeration_fixed_script_stream.rs` 95–110 与 grep 命中行（16、21、98–107、686–785 里 8 行）；`crash_enumeration_multi_record_publish_stream.rs` grep 命中行（21、22、51、54、100、173–175、213、287–480 里 13 行）。
- `crates/`：`crates/singlefs-checker-tier/src/crash.rs` 2040–2080、2175–2300、2460–2520 与 grep 命中行（305、2184、2261、2275、2278、2298、2469、2483、2501、2510、2740、2980–3336 里 6 行）；`crates/singlefs-harness/src/memory_pool.rs` 650–690；`crates/mutations.tsv` 第 4744、4749、4765 行（grep 命中）。
- 脚本与登记表：`scripts/experiment-apparatus/e184-segment-one-long-run.sh` 1–158、175–232 与 grep 命中行（201–318、436–456、560、575、637–660、696–702、779、822、947）；`scripts/experiment-apparatus/replay.sh` 第 1747、3198–3204 行；`.claude/gate.d/replay-comparison-units.tsv` 1–51；`.claude/gate.d/doc-experiments.sh` 第 15–27 行（gate-cell 行）与 34–84 里 grep 命中行；`scripts/experiment-apparatus/` 目录清单；`scripts/experiment-apparatus/e185_r1_compare.mutations.tsv` 1–3；`research/prompts/e185-r3-prereg.md` 节标题（grep）、1–4、455–470、674–700（只看登记的写法，没读它的结果）；`.claude/hooks/lib_long_run.py` 第 10、18、103 行；`.claude/heavy-thread-default` 全文；`scripts/gate/admission.py` 1–40 里 grep 命中行；`scripts/kb/quote-kb.py --help`、`scripts/kb/replace-once.py --help` 的输出；`scripts/experiment-apparatus/e28_map_rebuild.rs` 第 191 行与 `e142_new_pool_file_creation_dry_run.rs` 第 11047 行（grep 命中）。
- `.claude/kb/decisions/13-验证路线.md` 138–153、245–278、413–428、429–451（`quote-kb.py` 抄进第二节的四段）。
- `research/results/` 下只取了两份产物的 sha256 与行数（`sha256sum`、`wc -l`）和文件清单，没读正文。

### 13.2 跑过的命令（原样；只读命令与写这份登记的命令，没编译、没跑装置）

1. `bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m3-close-b17-e184-r2-rereg --purpose 'E184 第 2 次重跑登记的草稿'`（「认领了已有的目录」）
2. `git diff HEAD --stat -- crates/singlefs-checker-tier/src/bin/e184_regular_layer_cost_counts.rs` 与不带 `--stat` 的同一条；`git log --oneline -3 -- <装置>`；`git status --short` 对装置、转储 bin、长跑脚本与五份测试文件
3. `sha256sum research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out` → `7eacaffaddd44d34bf4a67afe29214ccbbf7ffbd63663d61ef7489124e858c06`；`wc -l` → 950
4. `sha256sum research/results/e184-regular-layer-cost-counts-selfcheck-2026-10-23.out` → `24ed4a59db6a04601178451d72c0c80131acbac889cdb39d589b4d8cdbce229c`；`wc -l` → 40
5. `sha256sum` 装置、转储 bin、长跑脚本、原登记、问题单、批 16 报告（值在文件头、第一、四节）
6. `timeout 200 nice -n 19 python3 scripts/gate/admission.py experiment /home/fy5090/code/singlefs E184` → 退 2，原样：「✗ 准入登记里没有实验键 E184（.claude/gate.d/stage-inputs.tsv）」「→ 怎么办：在 .claude/gate.d/stage-inputs.tsv 加一行「E184<制表符><装置入口，空格分隔>[<制表符><准入条件>]<制表符>#<为什么>」，照 E142 那一行写；装置入口传的键与登记的键逐字相同」
7. `nice -n 19 python3 recompute.py`（第七节脚本与输出）
8. `nice -n 19 python3 scripts/kb/quote-kb.py <草稿目录>/quoted-clauses.md` 加四个 `@#### 已定项 …` 取法 → 「✓ 4 段整抄进 …，回读逐字节一致」；`tail -n 108 <登记> | cmp - <草稿目录>/quoted-clauses.md` → 退 0
9. `grep -rln 'verdict_compar\|verdict-compar' crates scripts`（两处单测名）；`grep -rln 'e184.*compar\|e184_r2' crates scripts`（只有 `crates/mutations.tsv`）
10. `python3 -c "…print(l.CANONICAL_HOSTS_SENTENCE)"`（在 `.claude/hooks` 下 import `lib_long_run`）
11. `TZ=Asia/Tokyo date +%F` → 2026-10-24
12. `python3 scripts/kb/replace-once.py research/prompts/e184-r2-prereg.md <旧串> <新串>`（改第三节一句 grep 结论，「命中 1 次，已替换」）
13. 定位与核行号用的 `grep -n`、`awk 'NR==…'`、`sed -n`、`wc -l`、`ls`（命中行列在 13.1）

## 十四、并行与挂钟

- 独立单位：长跑 18 件里，`dump-<档>` 与 `keys-<档>` 按档两两成对、档与档之间独立；一件里每条流（随机历史一个种子一条、覆盖写链一档一条、冒烟一条）互相独立，装置按流切片并行；逐格比是一次读两份文本，不切。第一行的现算已在写登记时做完。
- 线程：每台取登记值 `.claude/heavy-thread-default`（今天 31）；长跑脚本档表（`:94`–`:101`）在内存吃紧的档上写得更小（`keys` 在 100 步档 8 条、200 与 400 步档 3 条），`E184_THREADS` 不设；逐格比单线程。
- 串行：件的前后次序由长跑脚本的前提表定（`:220`–`:231`：`keys-<档>` 等它的 `dump-<档>`，`share` 等全部 `keys`，`library` 等 `share`），因为后一件读前一件的产物；`keys_seconds` 是装置在进程里自己计的，不判、不要求独占；逐格比要等长跑产物拼完才起。
- 预计挂钟：长跑约 1.7–2.4 小时（推的：上一趟各件累计 5462 秒，实验页「复跑」一节；`--build-here` 编两份 bin 10–40 分钟，原登记 5.6 的估计；新段形每条流多一到几个 2 写段，转储与块键的工作量按段数涨不到百分之几）；逐格比与它的对照几分钟（推的：两份各约 950 行的文本）。合计可能超过 2 小时，长跑照「超过 2 小时」起。
- 预计在用核数：编译那一段约 31 核；转储各档 8–31 核（档表：50、100 步档 `all`，200 步档 16、400 步档 8）；块键各档 3–31 核（50 步档与固定流 `all`，100 步各档 8，200、400 步档 3）；`share`、`library` 1 核（推的：单线程逐流读块键文件）；逐格比 1 核。

分片：长跑脚本文件头 `long-run-shards:` 那一行（一条流一片，按档排成件，`--share k/n` 把每档 `keys` 的流按文件次序取余分给各台）。
续跑：长跑脚本文件头 `long-run-resume:` 那一行（同一条命令重起，做完的件按完成记录整件跳过，装置 sha256 变了的件作废重做）。
多机：台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量
第二台怎么起、块键怎么并回本机照长跑脚本文件头「多机的读法」（`:15`–`:28`）；块键文件大（100 步档 20 条流合计约 15G，脚本文件头第 20 行），只跑本机也行。
