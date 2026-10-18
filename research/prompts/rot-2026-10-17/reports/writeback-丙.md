# 写回丙（实验页与其余 kb）报告

草稿目录：`/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-wb-丙`（认领过）。没建副本、没起编译。没提交，没跑 doc-lint、gate、`doc-decisions.sh --write`（并行写回，按派发提示留给主 agent）。

## 先说三件要主 agent 动手的事

1. **`crates/singlefs-checker/invariants/` 四个文件没写**：写范围闸（`write-guard.sh`）对 `crates/` 下的路径退出码 2（「kb-scribe 的写范围不含 …/01-块自描述.md」等四条），按定义不按规格硬写。其余路径闸全过，所以把规格拆成两份：在范围内的写了，不在范围内的 6 处（4 个文件）备好，`--dry-run` 已核全部恰好命中一次，kb-spec-check 退 0。主 agent 套用：
   `python3 scripts/kb/replace-batch.py /home/fy5090/.local/share/singlefs/work/rot-2026-10-17-wb-丙/rb-B-invariants.json`
   （规格原文：`specB-invariants.json` / `specB-invariants.md`；内容是 `01-块自描述.md:6` 墓碑容器由墓碑树码 2 节点指认；`02-根环.md:16` I-7.9 的 C556 已还清；`03-journal.md:15` I-8.8 三处；`08-加密不变量.md:41` C68 括注。）
2. **31 个被改正文的实验页，「影响的决策」回看格没动**（派发第 8 条），回看日期现在早于今天的页内改动；门禁 doc-experiments 的 decision-links 格若按「页内改了正文就要回看」判，会逐页红，要主 agent 另处理。
3. **`experiments.md` 的 E168 行、E164 行、E156 行、E167 行已改（见下）；E142 页 `:189`、`:315` 的「整条流…还没枚举过」按派发点名没改**，等跑全量那个会话的回话。

## 规格与检查

| 项 | 路径 | sha256 |
|---|---|---|
| 规格 A（范围内，128 条） | `…-wb-丙/specA.json`（同内容 `specA.md`） | `a127301798f459cd59e9ce036835f4c2779255ae2bc9e81ca90fda3492bf2e16` |
| 规格 A2（4 条，措辞修正：去掉「下面」等方位词） | `specA2.json` | `5ca51aa410e79b5066772857d78f0b77c81fe7d31cfecfd39495413df6957ff8` |
| 规格 A3（2 条，E159、E165 变更史项补「改前」说明） | `specA3.json` | `91ac1ac283dbef20bc75059f99420224a29372ed8e35896afff34fdae9317d36` |
| 规格 B（范围外，6 条，未写） | `specB-invariants.json` | `fa72b008afa905e30c1646eb1abd80a57cefd78dd10847f00cc0084c5b2676b7` |

`kb-spec-check.py` 原样末行（A、A2、A3、B 各一次，退 0）：

```
  ✓ 规格 128 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 128 条）
  ✓ 规格 4 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 4 条）
  ✓ 规格 2 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 2 条）
  ✓ 规格 6 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 6 条）
```

（第一次起草的 134 条合并规格红了 40 处，全是「编号（简称）」写法，改规格原文后过；上面是改后的拆分版。）

`replace-batch.py` 原样输出（A 先 `--dry-run` 得「✓ 128 处都命中得对（40 个文件），--dry-run 没写」）：

```
✓ 128 处替换，写了 40 个文件（改名换上新 inode），回读一致      （A）
✓ 4 处替换，写了 2 个文件（改名换上新 inode），回读一致         （A2）
✓ 2 处替换，写了 1 个文件（改名换上新 inode），回读一致         （A3）
```

共 134 处替换、40 个文件（31 个实验页、`experiments.md`、`experiments-history.md`、`layout/01`、`layout/03`、`prior-art.md`、`term-renames.md`、`tooling.md`、`verification-build.md`、`vm-harness.md`）。`relabel-item.py`、`doc-decisions.sh --write` 这一轮不涉及，没跑。

其他核对：`sweep-term.py --check` 改前改后都是「全仓还有 1 份文件、3 处旧名」，那 1 份是 `records/2026-10-17-全仓知识腐烂治理.md`（不是我的文件），我的 `term-renames.md` 改动没带来新红。实验页里 358 条 `git show <提交>:<路径> 第 N 行（name=…）` 指针（改前 360 条，E165 两条改成了文件名）我写了脚本逐条 `git show` 核：路径存在、行号在范围内、`name=` 与那一行对得上，0 条不符（改前唯一不符的是 E159 :42，已改）；`git show dbcc0d2f:…/probe_crash_states.rs` 取得到。

## 变更史

`experiments-history.md` 在 31 个 `## E<n>（简称）` 节顶各加一块 `### 2026-10-17`、一个 `- ` 项（摘要加 `**改前**：git show c1e9b324:<页>`；只增行，`git diff` 里 0 行删除）：E16、E28、E34、E41、E49、E78、E79、E88、E94、E95、E100、E113、E128、E142、E151、E152、E156、E158、E159、E161、E164、E165、E168、E170、E171、E172、E173、E174、E176、E177、E178。E159、E165 两项的「改前」另注：被改的那几行指针写法是同批 2026-10-15 条目新写的、尚无提交，`c1e9b324` 里是整行抄的产物行（其余页的旧串都在 `c1e9b324` 里，脚本核过）。

## fixes-丙.tsv 逐行去向

行号是 `fixes-丙.tsv` 的文件行号（第 1 行是表头）。

| 行 | 去向 |
|---|---|
| 2 | 改了：`experiments.md` E156 行「哪一份承重」换成「主 agent 已认定第 5 次产物」并指到 `e156-r5-prereg.md` 末尾认定第 2 条（现查第 655 行起属实） |
| 3、4 | 改了：E167 索引行 `L11` 改 `l11`（页内 `\bL11\b` 0 处、`l11` 22 处） |
| 5、6、7 | 改了：E168 索引行写明 segment1 产物 115 MB、被 `.gitignore` 第 32 行忽略（`git check-ignore -v` 命中）、未入库，「九份」里工作区入库的只有 positive-controls 与 s1-history 两份，另六份在 git 历史（`31128273`）里 |
| 8、9 | 改了：E100 :81，C78（根记录没有字段表） 补「2026-10-11 已并入 C66（格式级空白没有分项号)，原文在 git 历史」 |
| 10 | 改了：E113 :124 落点改 C693（K 代撤回之后要重推的论证）（原 C160 并入） |
| 11、12 | 改了：E128 :120 落点改 C148（树表条目变宽把间接层门槛压到二层）（原 C157 并入） |
| 13 | 改了：E142 页首段（第 16、20、21、22 行）按第二十五次主产物核换，见下「E142」 |
| 14 | 改了：E142 :32 按派发点名 4 写成「以第二十五次跑的冻结版为准：11081 行，sha256 前缀 352ac898…；现行源码已有后续改动，行数用 `wc -l` 取」（现查 `wc -l` 11083，HEAD 同） |
| 15 | 改了：E142 :89 标题加括注 |
| 16 | 没改，交主 agent：E142 :189 的「整条流…还没枚举过」，等跑全量的会话回话 |
| 17 | 改了：E142 :199，装置常量 `JOURNAL_NEW_ROOT_SEGMENT_BYTES` 说法改成「第十一次跑时」，现为字面量 `= 364` 加编译期等式断言（`e142_….rs` 第 107、108 行现查）。**不写「2026-09-27 已改」**：日期在仓里查不到，规格里那个日期没抄 |
| 18、19、20、21、22 | 改了：G1（C305 2026-10-13）、G9（C311 2026-10-11）、G13、G15（C306 2026-10-11）、G14（C304 2026-10-11）落点加「已还清」，日期取自 `checks-owed.md` 已还清表。落点里不重复写编号，以免 `kb-spec-check` 的「编号（简称）」判红，写成「同一行（2026-10-11 已还清，见欠账表已还清表）」 |
| 23 | 改了：G16 的 C291 改成「条款那一半已还（D19 已定项 9），会红的检查那一半仍欠（仍欠三项）」 |
| 24、25、26 | 改了：G19 的 mkfs 模型改成第二十五次起五段、`operations=25 segments=14+1+1+1+4 closed_form=16402`，与登记行逐字相同；第二十四次及之前每盘四段 |
| 27、28、29、30 | 改了：E142 局限第 9 条（:309）：名字写 `journal_header_reader_branch_counts`（`header311_…` 是旧名）、加「第十四次跑时」、补今天的流（点名 12 项、闭式 16777240、补第三态全量 16777260，本页装置 `skipped=true` 不枚举）；**F929（行 30）「要人看」按派发点名 4 采纳「加『第十四次跑时』并补现数」** |
| 31 | 没改，交主 agent：E142 :315 同 :189 |
| 32 | 改了：E151 :7 |
| 33–37 | 改了：E152 五处括注 |
| 38 | 改了：E156 页标题「承重」；另把「17 单测」标成第 4 次重跑口径（见「E156 三种口径」） |
| 39–48 | 改了：E156 :189、E158 六处、E159 :26、E161 :14、E164 :13 标题括注 |
| 49 | 改了：E168 :33 产物表 segment1 行补未入库说明。规格里的「见下方判决行一节」含「见下方」（doc-lint 位置指代）没抄，写成「说明在「第一段的读数」一节的「原始输出未入库」那一行」（该节第 48 行现查有这句） |
| 50 | 改了：E170 :39 括注 |
| 51、52、53、55 | 改了：E173、E176、E177、E178 复跑块，见下「`cd research`」 |
| 54 | 改了：E178 第四段命令，见下 |
| 56 | 改了：E28 :3 |
| 57 | 改了：E41 :58（现查 I-7.5 在 `02-根环.md` 第 12 行，已实现） |
| 58 | 改了：E78 :40 C85 → C88（根环的时间线判别未实现） |
| 59–62 | 改了：E79 :36、:43 |
| 63 | 改了：E88 :84（`pool_checker_scan_of_tree_identifiers_claims_whole_units.rs` 在 `crates/singlefs-harness/tests/`，ls 过） |
| 64、65、66 | 没改，写判断：E92 :24、:75、:140 的「C88 未实现、需求未知／消费者未实现」。C88 的登记简称仍是「根环的时间线判别未实现」，三处与登记位一致；这三格说的是「需求值空白」（页内「它答不了的那一半」第 1 条：两格只判了单位、没判值），择新平局按实例代号已有只让 C88 只欠 checker 一侧，不让需求值变得已知。如果主 agent 认为要写成「择新一侧已有」，需要对着 E92 意图再判 |
| 67、68 | 改了：E94 :7、:100；E94 :58 的旧文件名 `invariants.md` 另改（派发点名 3） |
| 69 | 改了：E95 :52 |
| 70 | 改了：layout/01 :19，「文末历史版本记一行」改「直接改正文，经过靠 git」 |
| 71–77 | 改了：layout/01 :36 换第二十五次主产物，已承诺预留 196 760 |
| 78–85 | 改了：layout/01 :404 换第二十五次主产物，mkfs 行与用例逐字相同 |
| 86、87 | 改了：layout/01 :408「第二十四次及之前」 |
| 88、89、90、91 | 没改，交主 agent：layout/03 :54、:55、:56 以及 :50 的「推的，待干跑核」。E172 页自己写这些读数在退役树加进之前、E172 装置没有退役项（欠账 C775），撤标记要现跑装置核；派发提示（pending-main S24）说记进欠账、不归这一轮，欠账表归别的写回员 |
| 92 | 改了：layout/03 :59 挂载续做一行。**注意**：我照规格写了「不再标推的」，但 E172 对这一行的读数也是退役树加进之前的（同上），主 agent 若采取「要现跑装置再撤」的口径，这一格要回头改；`checks-owed.md:475`（C714）引这一行现文，由写回员甲同改 |
| 93 | 改了：`prior-art.md` :169 |
| 94、95 | 改了：`term-renames.md` :81、:82（`sweep-term.py --check` 改前改后相同，见上） |
| 96 | 改了：`tooling.md` :165 |
| 97 | 改了：`tooling.md` :199 |
| 98 | 改了：`tooling.md` :200 |
| 99、100、101 | 改了：`verification-build.md` :4（八个 crate、补 `singlefs-unit-graph`、崩溃枚举用例 14 条）。CLAUDE.md:8 的「七个 crate」不在我的范围，`fixes-主.tsv` 已有 |
| 102 | 改了：`verification-build.md` :168 |
| 103 | 改了：`verification-build.md` :295（十行） |
| 104 | 改了：`vm-harness.md` :10 |
| 105–108 | **没写**（范围闸拒），规格已备好，见开头第 1 条 |

## E142（行 13、14、17–31）

- 页首（第 16–22 行）按 `research/results/e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out`（`scripts/experiments/replay.sh` 第 993 行 `@driver_e142` 登记）核换：`name=gap` 33 行（G1–G32 与 F2；`grep -c '^E7RESULT name=gap '` 数出），其中「已收口」开头仍 13 行；判决行第 917 行 `width_mismatches=0`、`write_list_ok=true`、`recover_full_ok=true`、`control_*_ok=true`，`layer0_states_ok=layer0_violations=not_run`；`name=width` 30 行；`name=write_list writes=29 barriers=6 fua=1 named=12`；`name=recover_full root=1:3`；阳性对照 `name=layer0_control` 8192 态违例 4092；`name=layer0 … states=not_run closed_form=16777240`；`journal_differing_states=not_run`。
- 现行常量（`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs`）：`FULL_STATES` = 16_777_260、两态闭式 16_777_240、`JOURNAL_DIFFERING_STATES` = 6。页首写「第八格」时保留第十七次跑第二段的 3，同时点出今天 crates 一侧钉 6。
- 派发点名里 mkfs `operations=25 segments=14+1+1+1+4`（r25 第 35 行）、记录头 487、新根段 364（装置第 107 行）、点名项 64 容量：这几个数在 G19、:199、局限第 9 条里用到的已核；页首段没有再写记录头与新根段。
- 页首第 18–19 行（「第七次跑收口了三处」，G24/G25 是第七次跑新记）是带日期的事件句，没动。
- **G7 的 C284**：查 `checks-owed.md` 第 218 行，C284 仍在未还表，行内写「施加语义已定，剩会红的检查」；已还清表里没有它，变更史里只有立账与改口径、没有销账记录。所以不是「已还清」，我改成「条款那一半已还、检查那一半仍欠」（与 F511 对 G16 的判法一致）。这一条是我做的判断，不是规格给的；如主 agent 查到别处另有销账记录，请改回。

## E156 三种口径（行 38）

`17 单测` 是第 4 次重跑的口径（页内第 187 行 `17 passed`，22 行变异）；第 5 次 18 条（页内第 35 行「第 5 次 18」；第 169 行「整份装置的 18 条单测」没跑）；第 6 次 19 条（第 35 行、第 177 行复跑看 `19 passed`，没跑）。`experiments.md` 与 E156 页标题里的「17 单测」都改成「第 4 次重跑时 17 单测」；第 6 次的「单测 19 条…没跑，归提交时」本来就带次数，没动。

## `cd research`（行 51–55，派发点名 3）

`research/` 下已无 Cargo 工程（`scripts/experiments/Cargo.toml` 的 `name = "e7-index-bench"`）。E173、E176、E177、E178 复跑块里 `cd /home/fy5090/code/singlefs/research` 与 `cd research` 一并改成 `scripts/experiments`，块里 `bash scripts/hosts/…` 在该目录下写成 `../hosts/…`，变异行写成 `bash ../mutation/mutate.sh <bin> <源文件>.rs ../mutation/tables/<表>.tsv`（表在 `scripts/mutation/tables/`，ls 核过），`cd ..` 回仓根改成 `cd ../..`，E173 末尾的 `python3 research/results/…` 前补一行 `cd ../..`。E174 同样改，它的上卡命令按装置文件头改成 `python3 scripts/experiments/device-code-key.py run . --binary e174-… --feature e174-gpu --entry … --when-refused=--without-library -- run …`（在仓根起，产物路径写 `research/results/…`；`device-code-key.py` 起装置不带 `cwd=`，从仓根起则 `--output research/results/` 相对仓根成立），判的命令也改在仓根跑。E178 第四段两条命令改成经 `device-code-key.py run . --binary e178-… --entry e7_index_bench::segment_four_item_lines --when-refused=--no-reuse -- segment-four [--shard 0/2]` 起，注释改成项库默认路径与 `E178_SEGMENT_FOUR_ITEMS` 弃用、设了退 2（装置文件头第 24 行）、直接起装置只认 `--no-reuse`（`e178_….rs` 第 7121 行）。E95 :52 注释、`vm-harness.md` :10 一并改。

这些块我没有真跑（命令太重，也不属于写回），只核了：路径存在、`mutate.sh` 文件头写的就是 `cd scripts/experiments && bash ../mutation/mutate.sh <bin> <源文件> ../mutation/tables/<表>`、`device-code-key.py` 文件头用法与入口名。

**没改**：`../mutate.sh` 在 E108（:78–:83 六行）与 E92（:151–:155 五行）全在「影响的决策」回看格里，整句写的是「脚本路径搬迁只动路径（`../mutate.sh` 改到 `../mutation/mutate.sh`）」，同一行旧、新路径都在，属于说搬迁这件事本身的行（`.claude/rules/path-moves.md` 第 5 条），而且回看格这一轮不动（派发第 8 条）。

**发现但不归我（丁甲「21 个实验页复跑块」）**：E92 :130、E116 :186、E135 :151、E139 :160、E160 :187、E172 :39 和 :231 的 `../mutations/<表>.tsv`，E106 :139、E108 :70–71 的 `scripts/experiments/<…>.rs mutations/<表>.tsv` 目录不存在；E105 :56、E104 :132 等已是对的写法。我没碰，主 agent 派丁甲那一批时带上。

## 派发点名 8 件的去向

1. 「整行抄自产物」括注：改了 21 处，统一加「（产物已移出工作区的行写成一行指针）」，保留原字面。S11 的 17 处（E142 :89、E152 五处、E156 :189、E158 六处、E159 :26、E161 :14、E164 :13、E170 :39）；S08 比 S11 多的 E171 :291、E49 :177（引导句「（整行抄）」后接括注）；候选没罩到的 E172 :234、:251。**S08 的第 20 处 E142 :91 没改**：它是一段说明文字，只指到别的几节，自己下面没有指针行，:89 的标题括注已经罩住（S08 给的改法是在 :93 那句后补指针说明，写回时它与标题括注重复）。
2. E159 :42 指针括注由空的 `name=` 改成 `E7RESULT E159RESULT arm=yi-M-K10`（那一行 `git show 5d0b7e7c1667:…smoke.out` 第 175 行现查就是这样）；E165 :115、:116 改成指 `research/results/e165-…-segment2-kill-bounds.out` 第 30、31 行（`git ls-files` 在，32 行，第 30、31 行即 `name=restored`、`name=verdict`）。
3. `cd research` 六处、`invariants.md` 旧文件名（E170 两处、E171 三处、E94 一处、`prior-art.md`）、E174 `run` 命令：改了。`../mutate.sh`：没改，理由见上。
4. E142：页首换成 r25、:32、:309、G 表落点、G7：改了；:189、:315 没改（交主 agent，等跑全量的会话）；G7 的 C284 判法见上。
5. `verification-build.md` :4、:168、:295，`tooling.md` :165、:199、:200，`prior-art.md`，`term-renames.md` :81–82、:25：改了。**`INDEX.md` 没改**：逐行读过，没有判出要改的行（`reports/judge-S*.md` 里 INDEX.md 的行全判「不相干」，也没有 unit-graph 之类要补的登记）；如主 agent 另有所指，请点出行。`term-renames.md` :25 写成「不换的两类…（原有的第三类，分层表「层 0」档，随分层表三档改成冒烟层、常规层、大负载层已没有对象）」。
6. 不变量清单：规格备好、**没写**（范围闸拒）。I-8.8 三处已对着 `recovery.rs` 的 `replay_journal` 文档注释（前缀停在一次发布中间整体不施加、按记录标志位 0 认发布边界）、`crates/singlefs-replay-prefix/src/lib.rs` 提交标记分支、`transaction.rs:2043` 核过。第三处写的是「`crash_enumeration_record_spill_over_stream.rs` 要求 I-8.8 至少在一个状态上真被评估过、全部状态不判违例」（该文件第 344–365 行断言就是这样），没有写成规格原话的「每个状态必评估」。I-8.8 那一行里「它管的是条款允许的形态，不是第一版的形态…挡的是一事务跨多条记录落地那一天」没在三处之内，没动。
7. layout/01 :36、:404、:408：改了；layout/03 :59：改了；`experiments.md` 四行：E164 加限定（并给 E164 页 :158 W5 一行同加）、E167 `l11`、E168 改实数、E156 口径：改了。
8. 「影响的决策」回看格：一格没动。

## pending-main 里落在我文件上的其余条目

- 乙二乙「`git show` 指针没核」：核了，见开头，358 条 0 不符。
- 乙一「索引里把『今天』钉成具体日期的几处（E145、E156、E163、E165、E170）」：逐行读了这几行，带日期的都是「某天的读数」（例：E145「2026-09-13 的头逐宽试探解得出登记树」、E163「M2…在 2026-09-27 的显存条件下」），是事件句不是现状句，没改。
- S05「E171 页第 11 行 O3 用 E170 第 1 次重跑冻结的执行器」与 `13-验证路线.md:183`、E170 页第 460 行（E171 第 1 次重跑用 v2-487）矛盾：**没改**。:11 那一段写的是 s4 那棵冻住的树（2026-10-03，段尾「三分与读数一样只属于 s4 那一棵冻住的树」），那时 O3 就是 E170 第 1 次重跑冻结的执行器；v2-487 是 E171 第 1 次重跑（2026-10-07 起）用的。两处说的不是同一次，不矛盾。
- S14 E34 :209：改了（脚本已删，取回法见 `tooling.md`「本地 LLM（三方论证不派它当腿）」一节）。S16 E16 :307：改了（改指 C42）。
- S21 F600「实验页约 150 行仍写 `.claude/kb/invariants.md`」：在我的文件里现查只剩 `invariants.md` 裸名 6 处（上面已改）；带 `.claude/kb/` 前缀的写法 0 处。
- S02「E166 模型常量仍是退役树加进之前的 65548 / 12 个号，仓里没有欠账」：立账归欠账表的写回员，没动。S11 的 C308 G4 偏移列：改了。

## 没改的与理由（汇总）

E142 :91、:189、:315；E92 :24、:75、:140；E108/E92 回看格里的 `../mutate.sh`；E171 :11；layout/03 :50、:54、:55、:56；`INDEX.md`；`experiments.md` 带日期的读数句；各页「影响的决策」回看格。

## 交主 agent 的清单

1. 套用 `rb-B-invariants.json`（开头第 1 条）。
2. layout/03 :50、:54、:55、:56 的「推的，待干跑核」要不要撤；:59 我已写成「不再标推的」，口径要与前几行一致。
3. E142 :189、:315（等跑全量的会话）。
4. G7 的 C284 判法（上）；E92 三处 C88（上）。
5. 31 个实验页回看日期（开头第 2 条）；E142、E156、E164 等页的 `experiments.md` 索引行已同改。
6. 丁甲那批 `../mutations/` 路径（上）。
7. `records/2026-10-17-全仓知识腐烂治理.md` 的 3 处旧名使 `sweep-term.py --check` 红（不是我的文件，改前就红）。
8. `decisions-history.md` 开工后被别的会话改过（`sha256sum -c` 对不上），不是我改的，也不在我的规格里。

## 改过的文件与前后 sha256（开工时 → 收尾时；`sha-start.txt`、`sha-end.txt` 在草稿目录）

| 文件 | 开工 | 收尾 |
|---|---|---|
| `.claude/kb/experiments-history.md` | `12401818f654a1cd…` | `4901dce0ac1be02b…` |
| `.claude/kb/experiments.md` | `a12dd055e7275f12…` | `08db279114df090b…` |
| `.claude/kb/experiments/100-系统配置的三段几何.md` | `5e8b9b45d48ec561…` | `aa154d69db466094…` |
| `.claude/kb/experiments/113-不认识的树-补齐臂集.md` | `cbc499b620ab3352…` | `01c303958d311654…` |
| `.claude/kb/experiments/128-指针带出生树txg之后的点查代价.md` | `534f53493cb53e3c…` | `c6c19511e815e078…` |
| `.claude/kb/experiments/142-新池新建文件的干跑.md` | `117e92ea058d9064…` | `98f127a99b908224…` |
| `.claude/kb/experiments/151-用户数据落点的到达序与容器臂.md` | `04aa37690d2d177d…` | `b057ca38ba0734de…` |
| `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md` | `9d488da6420155fa…` | `6bfb916c87211505…` |
| `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md` | `80b0f16f71466ca8…` | `409e16072a284975…` |
| `.claude/kb/experiments/158-择根与修复四岔路.md` | `f1d238e84ef1bd49…` | `967bee6b3f75ceb2…` |
| `.claude/kb/experiments/159-fsync等待时间随并发数组提交与wal两臂.md` | `a2a40ff6b7867355…` | `1bf376462567b64e…` |
| `.claude/kb/experiments/16-journal的角色WALvs意图日志.md` | `e72499e806b5b7dc…` | `b6f795b42b697bae…` |
| `.claude/kb/experiments/161-崩溃放量的去重与分段耗时.md` | `b529611f88c8a53d…` | `fa48284ffb4dc3de…` |
| `.claude/kb/experiments/164-崩溃放量新设计的小范围功能测试.md` | `0ce9ae3596d8d0e8…` | `79f3015ffb2ca347…` |
| `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md` | `95d35530a8b41416…` | `00880ef3c1bba975…` |
| `.claude/kb/experiments/168-删除借保留量的环窗口.md` | `23b57f8f99603e16…` | `fa496176fb98ba2a…` |
| `.claude/kb/experiments/170-O3规约执行器第一版.md` | `43987d8b849e151e…` | `462179f2ae87a1db…` |
| `.claude/kb/experiments/171-三oracle合谋测试.md` | `5b8952ccd2b95d23…` | `121fd3d347f88262…` |
| `.claude/kb/experiments/172-删除写出的字节的干跑.md` | `596cea13ea05b9cd…` | `f679914fc5c14ca0…` |
| `.claude/kb/experiments/173-上限卡在读坏槽的三种处置.md` | `c0840a4957ce1cb8…` | `eb912d9e2366b5cc…` |
| `.claude/kb/experiments/174-借据丢掉时等待与显存回还.md` | `d5cc3f0978014e4a…` | `0bc991cccde09efe…` |
| `.claude/kb/experiments/176-inode树叶合并三候选的容量与写代价.md` | `4265e0f2b93769e6…` | `fcdd1396baa8face…` |
| `.claude/kb/experiments/177-逐树不写墓碑的复活数.md` | `8170479b1633867c…` | `e78f45a5f0647fcc…` |
| `.claude/kb/experiments/178-叶容器与记录死亡的计数.md` | `bcafaf533e605760…` | `3e50843af2c8e042…` |
| `.claude/kb/experiments/28-明文映射层能不能只靠单元自描述重建.md` | `71b767d4884792d9…` | `c3f1a9557bb5853e…` |
| `.claude/kb/experiments/34-根环槽几何.md` | `1766ecb65e093baf…` | `3825fa27034b752f…` |
| `.claude/kb/experiments/41-根环槽几何2x4x256.md` | `37ab3347d33f70f2…` | `32ad2dd17df6b727…` |
| `.claude/kb/experiments/49-反向链宽度32还是64.md` | `4aa4c937e85538a9…` | `9537dae8b37195df…` |
| `.claude/kb/experiments/78-重放的起点.md` | `89cc87f31606ab00…` | `2c80dc48116929a3…` |
| `.claude/kb/experiments/79-根记录的容量.md` | `9b6f30a3fb9eac81…` | `f6f0604b034d124e…` |
| `.claude/kb/experiments/88-冒名单元与孤儿单元的判别.md` | `82c5226b02ff413f…` | `b42e52755dee865e…` |
| `.claude/kb/experiments/94-搬共享落点的触碰面.md` | `f0a8f45eb373ed58…` | `7e1fcaaac8a0ac2b…` |
| `.claude/kb/experiments/95-老化的节点布局臂.md` | `4a73777f208b3b9a…` | `5aae5b926b477260…` |
| `.claude/kb/layout/01-first-txn.md` | `261ba929e2635526…` | `93079c952807ebf4…` |
| `.claude/kb/layout/03-third-txn.md` | `dab641b3957d02fa…` | `5911b012363fb866…` |
| `.claude/kb/prior-art.md` | `7f0f2db98486b75c…` | `13c6757944e2c7ac…` |
| `.claude/kb/term-renames.md` | `2835a3dbe40cc59a…` | `95e8dbb2ccc55ee8…` |
| `.claude/kb/tooling.md` | `0a4ff0491cbc824b…` | `ca2432f4481b55ae…` |
| `.claude/kb/verification-build.md` | `d60abc5e4eb6208c…` | `09b3c4d681386081…` |
| `.claude/kb/vm-harness.md` | `d197211f38bb347c…` | `36b013bcae817ed6…` |

共 40 个。`.claude/kb/decisions-history.md` 开工与收尾 sha 不同（`c8d81ebc8c1112b1…` → `62570728893035c8…`）是别的会话改的，不在我的规格里。完整 64 位见 `sha-start.txt`、`sha-end.txt`。

## 没做什么

- 没判定案对不对；没写规格外的句子（G7 的 C284 是我按 `checks-owed.md` 现文做的一处判断，已在报告里标出）；没修不是这一轮的红；没提交。
- 没跑 doc-lint、gate、`doc-decisions.sh --write`（`decisions.md` 状态列这一轮不动）；没跑重型测试；没建副本、没起编译。
- 复跑块里的命令只核了路径与文件头写法，没真跑。
- `crates/singlefs-checker/invariants/*.md` 四个文件没写（范围闸拒），规格已备好。
- 推翻条件：若有人 `git show c1e9b324:<页>` 之外另有这些页的更晚提交，我写进变更史的「改前」引用要跟着改；若跑全量的会话回话说 E142 流已整轮枚举过，:189、:315 与页首「本页装置不枚举」那句要回头对一遍。
