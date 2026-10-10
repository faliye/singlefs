# 第 37 批代码轮第三轮：云端辩方（Sonnet）报告（2026-10-09）

轮名 m3-formal-code-b37-r3；立场：复核第二轮判决（`research/prompts/m3-formal-code-b37-r2-main-verification.md`），判正文第三节 D1、D2、D4。
代码引文读冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r3/tree/`，路径写 `crates/…`，行号是这棵树里的行号（`sha256sum` 与主工作区同一份，a62469c7…）；脚本、登记、产物读仓根。
没读禁读清单里的攻方报告与模型。没跑任何重型测试，没编 Rust；模型是纯 Python（读源码、`replay.sh` 与入库产物），在 `research/prompts/m3-formal-code-b37-r3-sonnet-model/`，复跑 `bash research/prompts/m3-formal-code-b37-r3-sonnet-model/tools/rerun.sh`（四个 `PYTHONHASHSEED` 下输出同一个 sha256）。

## 一、判定一览

| 条目 | 辩方判定 | 证据 | 推翻它的现象 |
|---|---|---|---|
| D1-a 14 个单元恰好是 `replay.sh` 里直接起装置的全部（`root_choice` 两族与杀伤界驱动除外） | 站得住 | E1 | `replay.sh` 里再找到一条起装置、产物带 `name=control` 行的驱动 |
| D1-b 14 份集合对点名的产物 | 站得住：14 / 14 标签集合相等、条数 11、7、4、19、9、6、19、6、8、16、8、4、16、8 | E2 | `tools/check_sets.py` 任一行不是 OK |
| D1-c 产物与今天的装置同一份判法（只到「判据集合」一级） | 站得住：从今天源码的两个谓词算出的集合与 10 个不同的（族，档，建表法）产物逐个相等；盘次序、`--only`、`--bounds` 不进对照 | E3 | 对照那一段（第 7751 行起）或 `run_profile_of_a_control` 之后被改而没重编产物 |
| D1-d 单元之外还有没有正式跑 | **打中一处**：登记的正式跑里有 4 个没有复跑行（r2 直接建表的合法、对抗，各偶、奇数一份），单测没钉；其中 r2 直接建表合法一族那一个，让 5 个一词之差的改坏（J7 五条的谓词换成「两份表都取」）正式跑少对照而单测不红 | E4 | `replay.sh` 或登记里这 4 份被认定不是正式跑；或补一个 r2 直接建表合法一族单元之后这 5 个仍不红 |
| D1-e `root_choice`、`root_choice_faults`、杀伤界驱动不是单元 | 站得住 | E5 | 杀伤界产物里出现 `name=control` 行 |
| D1-f 703 与 706 同一单元；750、753 的 `--only`、盘次序不进对照 | 站得住（对照 `name=control` 行逐字相同）；这两个单元只在「过滤那一层」有用 | E3 | 盘次序旋钮在第 6300 行之后又被读 |
| D1-g r1 三个单元注释点的 2026-10-04 产物不是 `replay.sh` 登记的那三份 | 小瑕疵，不碍：`replay.sh` 登记的是 2026-10-01 产物，两边 `name=control` 行逐字相同 | E2 | 两份 `name=control` 行不同 |
| D2-a 族这一维的打中与改法 | 站得住：70 个挪族改坏里新单测抓 66，余 4 个等价（T 两条走 `is_judged_in_every_family`，`family()` 对它们不起作用）；第二轮单测只抓 18 | E6 | 找到第 67 个放过的挪族改坏 |
| D2-b 改法是不是同样碍着别的 | 不碍别的：新单测红 ⇔ 14 个单元之一的对照集合变；`is_judged_under` 谓词互换 210 个里 197 个改变这 14 个单元、197 个全红，0 个误红 | E7 | 找到一个 14 个单元集合不变而新单测红的改坏 |
| D2-c C1-乙（r3 档同档无对照）不归这一批 | 站得住；补一笔：同一形状在 r2、r5 的对抗跑上同样存在（判 J6Qn、J7 或 J8 而对照只在合法一族），只是 r2、r5 有同档的合法一族正式跑，r3 没有 | E8 | 新单测的目的被定为「每个正式跑自带它判的每条判据的对照」 |
| D2-d C3-丙（读数一级）不归这一批 | 站得住；另有一道现成的守卫：`replay.sh` 的 exact 行逐字节比整份产物（含 `name=control` 读数行），E165 不在 `SLOW` | E9 | 读数少一部分而 `replay.sh` 对应行仍绿 |
| D2-e 那 9 个「正式跑集合不变却红」判成红得对 | **站不住一半**：这句话说的是第二轮改之前的五档单测；采下来的逐单元单测在这 9 个上 0 红。9 个里 5 个（J7 五条换成「两份表都取」）其实改变了一个有产物的正式跑（r2 直接建表合法一族，D1-d），4 个（J8 四条换成「拆行」）只改变没人跑的 r3 合法一族 | E7、E4 | 逐单元单测在这 9 个上有红 |
| D4-a 两行锚点各恰好一次 | 站得住 | E10 | 对冻结副本数得的不是 1 |
| D4-b 改后单测下红在集合断言上、第二轮单测下放过 | 站得住：第一行第二轮单测放过由主 agent 现跑坐实；第二行读码 + 模型 | E10 | 两行任一行在第二轮单测下红，或在改后单测下红在别处 |
| D4-c 挂钟 | 站得住：主 agent 现跑 10.95 s（五档版 6.39 s）；5 个对抗一族单元各约 4 至 5 s 的对照（release、7 线程） | E11 | 同机再跑超过 1 分钟 |

交回前判决里的「待主 agent 取舍」只有 D1-d 与 D2-e 两条（同一件事的两面），见第四节。

## 二、D1 的证据

### E1　D1-a：`replay.sh` 里起装置的驱动

`grep -c 'bin e165_replay_bounded_enumeration -- ' research/scripts/replay.sh` 得 18 行。去掉三条 `root_choice` / `root_choice_faults` 驱动（第 766、791、870 行，对照走各自的读故障对照，判据不进 `Criterion`），剩 15 行；第 703、706 行只差 `--bounds`，是同一单元，所以 14 个：
- 「--bounds segment1 --family legal」（`research/scripts/replay.sh:703`）、「--bounds light --family legal」（`research/scripts/replay.sh:706`）、「--bounds segment1 --family adversarial」（`research/scripts/replay.sh:710`）；
- 「--family multi_disk --profile r2」（`research/scripts/replay.sh:738`）、「--family legal --profile r2 --table both」（`research/scripts/replay.sh:741`）、「--family adversarial --profile r2 --table both」（`research/scripts/replay.sh:744`）、「--family multi_disk --profile r2 --table both」（`research/scripts/replay.sh:747`）；
- 「--only R --profile r2 --table both --disk-order reversed」（`research/scripts/replay.sh:750`）、「--only M,MC --profile r2 --table both --disk-order reversed」（`research/scripts/replay.sh:753`）；
- 「--family adversarial --profile r3」（`research/scripts/replay.sh:763`）；
- 「--family legal --profile r5」（`research/scripts/replay.sh:855`）、「--family adversarial --profile r5」（`research/scripts/replay.sh:858`）、「--family multi_disk --profile r5」（`research/scripts/replay.sh:861`）、「--marker-parity odd」（`research/scripts/replay.sh:864`）、「--marker-parity odd」（`research/scripts/replay.sh:867`）。
单测里 14 个单元的注释行号与这 15 行一一对上：「let formal_run_units: [(&str, Family, RunProfile, &[Criterion]); 14] = [」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18286`）。

推翻：`replay.sh` 里再有一条起装置、产物带 `name=control` 的驱动；或有人给第 766 行那一族加了 `Criterion` 的对照。

### E2　D1-b、D1-g：14 份集合对产物

`tools/check_sets.py` 把单测里每个单元的 `Criterion::…` 数组按文件里「判据 → 标签」那张表（第 6120 行起）翻成标签，与注释点名的产物里 `name=control criterion=…` 的标签集合比，再比条数。原样输出（`rerun-output.txt` 第 1 至 16 行）：

```
703、706 egment1-r1-legal-family.out 11 11 OK 
710 egment1-r1-adversarial-family.out 7 7 OK 
738 egment1-r2-multi-disk.out 4 4 OK 
741 egment1-r2-table-both-legal-family.out 19 19 OK 
744 egment1-r2-table-both-adversarial-family.out 9 9 OK 
747 egment1-r2-table-both-multi-disk.out 6 6 OK 
750 egment1-r2-reversed-disk-order-legal-family.out 19 19 OK 
753 egment1-r2-reversed-disk-order-multi-disk.out 6 6 OK 
763 egment1-r3-adversarial-family.out 8 8 OK 
855 egment3-r5-legal-family.out 16 16 OK 
858 egment3-r5-adversarial-family.out 8 8 OK 
861 egment3-r5-multi-disk.out 4 4 OK 
864 egment3-r5-odd-marker-legal-family.out 16 16 OK 
867 egment3-r5-odd-marker-adversarial-family.out 8 8 OK 
units 14 bad 0
```

D1-g：第 331、333 行登记的 r1 产物是 2026-10-01 的（「E165|@driver_e165||e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out|exact」，`research/scripts/replay.sh:331`；「E165|@driver_e165_adversarial||e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out|exact」，`research/scripts/replay.sh:333`），单测注释点的是 2026-10-04 那三份。我对两边的 `name=control` 行做 `cmp`：legal、adversarial、light 三对都输出 `legal-same`、`adv-same`、`light-same`（逐字节相同），所以集合不受影响；只是注释点的文件不是 `replay.sh` 登记的那份，读的人顺着登记找不到。

推翻：三对里任一对 `cmp` 报不同。

### E3　D1-c、D1-f：产物与今天装置的判法

- 对照那段循环只经两道筛：「if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7758`）、「if !criterion.is_judged_under(run_profile) {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7761`）；`main` 起对照用命令行的族：「positive_control_readings(&selection.families, run_profile, thread_count);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17769`）。`--bounds`、`--only`、`--max-n` 不进这一调。
- 盘次序、建表法在对照里被摘掉：「let without_the_record_table_knobs = run_profile」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6301`），只有两条扫盘表对照把建表法补回 `Both`。所以 750、753 两个反盘次序单元的对照行与 741、747 逐字相同：`cmp` 输出 `reversed-legal-controls-eq-both-legal`、`reversed-md-eq`。这两个单元在单测里的用处只在过滤那一层（`is_judged_under(run_profile)` 收到的是没摘旋钮的档）。
- `tools/model.py` 从源码把「判据 → 族」「判据 → 谓词」解析出来，按档算出集合，与产物比：`r1 legal 11 model==product True` 起十行全 True（`rerun-output.txt` 第 19 至 28 行）。这是今天源码对 2026-10-04 产物的一致，不是今天装置复跑；复跑是重型，交主 agent（第五节）。主 agent 现跑的 `1 passed`（`~/.local/share/singlefs/work/m3-formal-37c-run/e165.log`）是同一件事的现跑旁证。
- 一致只到判据集合一级。读数（量在哪、值多少）我没比，那是 D2-d 的事。

推翻：对照那一段或第 6300 行起的函数在产物之后被改过而产物没重出；或主 agent 在改后的装置上复跑 `--bounds light --family legal --profile r5` 之类的组合，`name=control` 标签集合与单测写死的不同。

### E4　D1-d：登记的正式跑比 `replay.sh` 多 4 个

- E165 第 5 次重跑第一段的比对对象表里，第 4 至 7 行是 r2 直接建表的合法、对抗，各偶、奇数一份，写的是「无复跑行（`config` 行）」：「| 4 | `e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family-2026-10-02.out`」（`research/prompts/e165-r5-prereg.md:669`）、第 5、6、7 行同形（`research/prompts/e165-r5-prereg.md:670`、`:671`、`:672`）；表后一句「第 1–16 行的重跑在第一段」（`research/prompts/e165-r5-prereg.md:686`）。重跑产物在 `research/results/`：`ls research/results | grep` 得 `…-2026-10-04-r5-segment1-r2-legal-family.out`、`r2-adversarial-family.out`、`r2-odd-marker-legal-family.out`、`r2-odd-marker-adversarial-family.out`，`name=config` 行是 `profile=r2`、没有 `table=`（直接建表）；`name=control` 标签数 17、7、17、7（我数的：58 / 10 / 60 / 10 行，17 / 7 / 17 / 7 种）。
- 单测的单元表取自 `replay.sh` 起装置的驱动（转述；`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18286` 起的 14 项），这 4 个没有驱动行，不在里面。
- 这 4 个里有一个让集合变的区别：r2 直接建表合法一族是 J7 判、T 不判（17 条）；14 个单元里 r2 的合法一族只有「两份表都取」那两个（19 条，T 判）。J7 五条在 `is_judged_under` 的同一个臂：「| Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.judges_j7(),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6221`）；`main` 判 J7 走另一个谓词调用：「if run_profile.judges_j7() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8218`）。把对照这一侧的谓词换成 `compares_the_two_record_tables()`（定义见「matches!(self.table_source, RecordTableSource::Both)」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:387`）：r2 两份表都取那两个单元里 J7 照判（`Both` 为真），r1 不判（两边都假），r3 合法一族不是单元；14 个单元的集合一个不变，r2 直接建表合法一族的正式跑却丢掉 J7 五条的对照而 `main` 仍判 J7。
- 模型把 `is_judged_under` 的每条判据各换成另外 6 个谓词（35 × 6 = 210 个），分别对「14 个单元」与「14 个加 r2 直接建表合法、对抗两个」算集合变化，输出（`rerun-output.txt` 第 46、47 行）：「mutants changing some of 18 registered formal runs 202 ; missed by the 14-unit test 5 [('J7a_instance_table', 'compares'), ('J7a_allocation_record_tree_root', 'compares'), ('J7a_filesystem_identifier', 'compares'), ('J7b', 'compares'), ('J7c', 'compares')]」。挪族那一类加上这 2 个单元也没有新增放过：「family-move mutants missed by 14 but changing 18: []」。
- 补法（我没写）：单元表加一项 r2 直接建表合法一族（`RunProfile::r2(even)`、`Family::LegalHistory`、17 条＝基础 11 条＋J6Qn＋J7 五条），注释写「`research/scripts/replay.sh` 无驱动行；登记 `research/prompts/e165-r5-prereg.md` 比对对象表第 4 行」。对照只多花几十毫秒（合法一族，产物 `-timing.out` 里同类单元 0.02 至 0.07 s）。对抗一族那个不用补：集合同 r1 对抗的 7 条，模型里加它不抓新东西。奇数两份集合同偶数，只在读数上不同，读数一级不在这一格。

推翻：把 4 份产物认定为「不是正式跑」（例如登记里写明它们只作比对对象、不算这一次的跑）；或补上 r2 直接建表合法一族之后，那 5 个互换仍不红。

### E5　D1-e：杀伤界驱动与 `root_choice` 两族

- 杀伤界驱动（第 334、346 至 348、355 行）调装置的界是「return ["--bounds", "segment2"]」（`research/scripts/e165-replay-mutation-kill-bounds.sh:557`），族不给，取默认的合法、对抗两族：「"all" => {}」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17419`）；变体各编各跑，产物不留 `name=control`。命令与原样输出：

```
for f in research/results/e165-replay-bounded-enumeration-2026-10-0[1-4]*kill-bounds.out; do echo -n "$(basename $f | cut -c36-) "; grep -c 'name=control' $f; done
6-10-01-r1-segment2-kill-bounds.out 0
6-10-01-segment2-kill-bounds.out 0
6-10-02-r2-segment2c-kill-bounds.out 0
6-10-02-r2-segment2-kill-bounds.out 0
6-10-03-r3-segment3-root-choice-kill-bounds.out 0
6-10-03-r4-segment2-root-choice-faults-kill-bounds.out 0
6-10-04-r5-segment2-r2-mode-kill-bounds.out 0
6-10-04-r5-segment2-r3-mode-kill-bounds.out 0
6-10-04-r5-segment2-r4-mode-kill-bounds.out 0
6-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out 0
```

所以它们不是「产物里点得到 `name=control`」的单元。
- `root_choice`、`root_choice_faults` 在 `main` 里先于对照那段分流：「if selection.families.contains(&Family::MultiDisk) && !run_profile.judges_j6qn() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17696`）之前就 `return`（`run_the_root_choice_family`、`run_the_root_choice_faults_family`，第 17652、17663 行），不走 `positive_control_readings`。

推翻：杀伤界产物里有 `name=control`；或 `root_choice` 路径在 `main` 里改成也调第 7751 行那一个函数。

## 三、D2 与 D4 的证据

### E6　D2-a：族这一维

- 第二轮单测是 5 次调用，每次给全族；新单测每个单元单族：「let readings = positive_control_readings(&[family], run_profile, thread_count);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18547`）。`Criterion::family` 只被第 7758 行那一处读。
- 模型（`tools/model.py`，`rerun-output.txt` 第 38 至 41 行）：「mutants 70 caught 66 missed [('T_table', 'Adversarial'), ('T_table', 'MultiDisk'), ('T_row', 'Adversarial'), ('T_row', 'MultiDisk')]」、「old 5-profile test catches 18 of 70」。放过的 4 个是 T 两条换族：T 走 `is_judged_in_every_family`（「const fn is_judged_in_every_family(self) -> bool {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6202`），第 7758 行的 `&&` 让它绕过族过滤，换族对任何一次跑都没有效果，是等价改坏。数字 18 与第二轮判决里辩方的「新单测抓到 18」同口径；66 与「改变某个正式跑的对照集合 66 个」相同。
- 变异行 3498 一行把那个八项或模式整臂换族：臂从 `Self::A0Defined` 起到「| Self::J6QnSplitByI812 => Family::Adversarial,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6193`）收尾，改一行等于七条对抗判据加拆行一起进合法一族，所以行名写「八条」，名实相符。
- 推翻：找到第 67 个挪族改坏——在 14 个单元的集合上放过、又改变 18 个登记的正式跑之一；或 `Criterion::family` 在第 7758 行之外又有人读（那时要加单元）。

### E7　D2-b、D2-e：改法碍着什么

- 新单测的断言只有三类：集合相等（「"{unit_name} 那个单元跑出对照的判据与那条正式跑产物里的那一份不同"」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18554`）、读数全红、跨单元罩全表（「controlled_in_some_unit.contains(criterion),」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18566`），外加标签互不相同（「assert_eq!(labels.len(), Criterion::ALL.len(), "判据的标签互不相同");」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18285`）。集合相等对 14 个写死集合，所以「新单测红」恰好就是「某个单元的集合变」，不会因为登记语义被改而误红。
- 模型 210 个互换（`rerun-output.txt` 第 43 行）：「{'changed': 197, 'unchanged': 13, 'unch_new_red': 0, 'unch_old5_red': 9, 'changed_new_red': 197}」。197 个改变 14 个单元之一的互换新单测全红；13 个不改变的互换新单测 0 红；第二轮的五档单测在这 13 个里红 9 个。
- 判决第二格说：「站住。那 9 个是登记语义被改的改坏，红是对的，不算过严」（`research/prompts/m3-formal-code-b37-r2-main-verification.md:28`）。这句对第二轮改前的五档单测成立；判决同一份第三节采的改法（逐单元）把它变成了 0 红，判决没有回头改这句。9 个的内容（`rerun-output.txt` 第 44 行）：J7 五条换成「两份表都取」＋J8 四条换成「拆行」。前 5 个是 E4 里放过的那 5 个，改变的是有产物的 r2 直接建表合法一族；后 4 个（J8 → 拆行）改变的只有 r3 档的合法一族，r3 没有合法一族的正式跑（C1-乙，E8），这 4 个与那一条同类。
- 所以「红得对」站不住的是这一半：9 个里 5 个是真缺口（正式跑少了对照），不是「过不过严」的取舍；单测对它们放过是单元表比登记的正式跑少一个。另 4 个按这一批的立场（单测只钉正式跑）放过是对的。

推翻：逐单元单测在这 9 个里有红（那时 `unch_new_red` 不是 0）；或 E4 的 4 份产物被认定不是正式跑（那 5 个也就回到「只改没人跑的格」）。

### E8　D2-c：C1-乙 该不该归这一批

- 判决的原话：「现查坐实，不归这一批」（`research/prompts/m3-formal-code-b37-r2-main-verification.md:27`）。事实：r3 的唯一正式跑只跑对抗一族（「--family adversarial --profile r3」，`research/scripts/replay.sh:763`），这个跑在 A 域判 J6Qn、J7：「j7_judged=45149528」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out:209`），它的对照集合只有 8 条（E2 的 `763` 行），J6Qn、J7 的对照在合法一族，r3 没有合法一族的跑。
- 同一形状不是 r3 独有：r5 对抗跑在 A 域判 J8：「j8_judged=45149528」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family.out:213`），对照 8 条里没有 J8；r2 两份表都取的对抗跑（第 744 行）同样判 J6Qn、J7、对照 9 条里没有。区别只在 r2、r5 各有同档的合法一族正式跑（第 741、855 行）可以担，r3 没有。这是「每个正式跑是否自带它判的每条判据的对照」的设计，不是这条单测引进来的：单测的文档注释自己说它钉的是「逐个核正式跑的单元」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18271`）。
- r3 登记把 r3 对抗跑的正确性押给与 r2 的逐字节对比：「去掉这几行之后与 `r2` 档同一条命令的输出逐字节相同」（`research/prompts/e165-r3-prereg.md:424`）。补合法一族的 r3 正式跑，或在登记里写明由 r2 档担，是登记与跑法的事，单测无从补；归不归进这一批，只取决于这一批要的是不是「单测钉住正式跑的现状」。正文第三节把读数一级与 C1-乙都划在这一轮不攻之外：「读数一级（第二轮判决第三节第 3 条记账）与 r3 档同档无对照（交用户）不再攻」（`research/prompts/_m3-formal-code-b37-r3-body.md:12`）。站得住。
- 有一处可以顺手：单测里「r3 对抗」那一项（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18438`）没有注释说明 r3 档没有合法一族的跑，将来有人给 r3 加合法一族驱动时单测不会提醒他加单元（集合相等只对表里的单元）。一句注释的事，不是缺陷。

推翻：这一批被定义成「保证每个正式跑自带它判的每条判据的对照」（那时 r3 缺口该在这一批里挡）。

### E9　D2-d：读数一级不归这一批

- 事实：单测只比判据集合（`controlled` 与 `registered` 同为 `BTreeSet<Criterion>`），读数的条数、量在哪、值多少不比。
- 现成的守卫：`replay.sh` 的 E165 各行是 exact，逐字节比整份产物，产物含 `name=control` 读数行：「E165|@driver_e165_r2_segment3_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk.out|exact」（`research/scripts/replay.sh:335`），第 336 至 341、349 至 353 行同形；门禁复跑那一格的慢单清单里没有 E165：「SLOW=(E9 E17 E20 E21)」（`.claude/gate.d/checker-tier-research-build-and-replay.sh:82`）。所以一个少掉部分读数的改坏，在提交时的复跑里会让对应行逐字节对不上。这一条我是读脚本推的，没有看 E165 各行在整轮门禁里实际跑没跑（准入判「输入没变」可能跳过，我没查 E165 行的准入键）。
- 判决的话：「不归这一批，记账」（`research/prompts/m3-formal-code-b37-r2-main-verification.md:30`）。站得住；比判决原有理由多一条（复跑的 exact 比对），不是它的替代。

推翻：构造一个读数少一部分、判据集合不变的改坏，在对应 E165 行的 exact 复跑里仍逐字节相同。

### E10　D4-a、D4-b：两行变异

- 锚点各恰好一次、每行六段：我对冻结副本数原文串（把 `\n` 还原成换行，与 `research/scripts/crates-mutation-rows.py` 的还原同法），原样输出：

```
3498 fields 6 anchor occurrences 1 test every_criterion_has_a_positive_control_and_every_control_is_red
3499 fields 6 anchor occurrences 1 test every_criterion_has_a_positive_control_and_every_control_is_red
```

两行：「E165 对抗一族八条挪进合法一族」（`crates/mutations.tsv:3498`）、「E165 T_table、T_row 只在合法一族跑对照」（`crates/mutations.tsv:3499`）。`crates-mutation-rows.py static-check` 整表我没跑（归主 agent，第二轮判决已记 `CHECKED 3494 322 10`）。
- 第 3498 行：改后单测下红在 `assert_eq!`（「controlled, registered,」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18553`）：整臂八条进合法一族，第一个单元「r1 合法」就多出 7 条对抗判据（18 对 11）。第二轮单测（5 次调用，每次含合法、对抗两族）不红：族并起来集合没变；主 agent 现跑 `1 passed`（第二轮判决第四节）；模型 `old 5-profile test catches 18 of 70` 里这 8 个同族整挪也不在那 18 个里（读码判，没有单独跑 8 个整挪）。
- 第 3499 行：`is_judged_in_every_family` 恒假，T 两条只剩合法一族自己带：「两份表都取的 r2 合法」仍 19 条，下一个单元「两份表都取的 r2 对抗」从 9 条降到 7 条，红在同一个 `assert_eq!`。第二轮单测里 T 随合法一族进来，全族调用不变：模型 `every_family= false new red True old red False`（`rerun-output.txt` 第 40 行）。读码 + 模型，没跑装置；第二轮判决第四节记的第五条证红「两份表都取的 r2 对抗」与此一致。

推翻：两行任一行在第二轮单测下红；或在改后单测下红在 `assert_eq!` 之外的位置（锚点一替换编不过也算）。

### E11　D4-c：挂钟

- 主 agent 现跑改后单测：「test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 81 filtered out; finished in 10.95s」（`~/.local/share/singlefs/work/m3-formal-37c-run/e165.log:10`），命令是 `cargo test --release` 经 `capped.sh 15`；五档版是 6.39 s（`~/.local/share/singlefs/work/m3-formal-37b-run/e165.log:30`）。两次负载不同，不是对照实验。
- 14 个单元里对抗一族 5 个（r1、两份表都取的 r2、r3、r5、奇数 r5），产物的对照耗时 4.172、4.475、5.110 s 等（release、7 线程）：「scope=positive_controls elapsed_seconds=4.172」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r1-adversarial-family-timing.out:1`）、「scope=positive_controls elapsed_seconds=4.475」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family-timing.out:1`）、「scope=positive_controls elapsed_seconds=5.110」（`research/results/e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family-timing.out:1`）；合法与多盘各单元 0.01 至 0.07 s。
- 每条变异行跑装置的整个测试模块（第五段 `-p … --bin …` 没有测试名筛，82 条），每行多花的是这 11 秒量级，不是新瓶颈。补 E4 的合法一族单元只加几十毫秒。

推翻：同机再跑改后单测超过 1 分钟。

## 四、交主 agent 取舍的两件事（同一个缺口的两面）

1. 单元表要不要多一项（D1-d）。补一项 r2 直接建表合法一族（17 条）就把 E4 的 5 个放过堵上，花几十毫秒；不补，就得把第二轮判决第二格那句「那 9 个……红是对的，不算过严」改成「对第二轮改前的单测成立，逐单元单测在这 9 个上 0 红；其中 5 个改变 r2 直接建表合法一族这一没有复跑行的正式跑」。两条路都是读码 + 模型的结论，没在入库装置上跑。
2. 那 4 个 J8 → 拆行的互换（D2-e 后半）与 C1-乙同类：r3 档没有合法一族的正式跑。已在交用户表里（C1-乙），不另开。
3. 零轮的东西：补 r2 直接建表合法一族单元这个做法是我提的，被攻过零轮；如果它还要一条变异行守（把 `judges_j7()` 换成 `compares_the_two_record_tables()`），那一行也是零轮，我没写。

## 五、没做什么

- 没跑任何 cargo、没跑 E165 装置、没跑 `crates-mutation-rows.py static-check` 或 `prove-red.sh`；D4 的「红在断言上」是读码 + 模型 + 主 agent 第二轮日志，不是我的现跑。
- 没复核读数一级（每条判据量了几个域、几个读数）与单元内部、单元之间的维度（那是攻方的 D3）；没评估 `--table scan`、r4 的 `root_choice_faults` 的对照。
- 没读禁读清单里的文件（攻方本轮报告与模型）；没替主 agent 撤销或维持任何判决。
- 模型的前提：「判据 → 族」「判据 → 谓词」是从冻结副本源码解析的（`tools/model.py`），登记规则我没有为这一轮重写一遍（第二轮报告 E2 对五档写过）；E4 的「正式跑」取自 r5 登记比对对象表与 `research/results/` 里 2026-10-04 的产物名，没有读那 4 份产物的内容细节，只数了 `name=control` 标签与 `name=config` 行。

## 六、要主 agent 跑的命令（可选）

没有必须跑的。想在入库装置上把 E4 的 5 个放过坐实，可选：在草稿副本里把 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 第 6221 行 `| Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.judges_j7(),` 换成 `| Self::J7cUnmountMarkerWhenNothingIsApplied => run_profile.compares_the_two_record_tables(),`（只换这一条臂的谓词；J7 另外四条在它上面的同一臂里一起换，是整臂），然后

`cd <副本根> && SINGLEFS_HEAVY_TESTS=user-request bash research/scripts/run-with-memory-cap.sh 24G bash research/scripts/capped.sh 15 cargo test --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- every_criterion_has_a_positive_control_and_every_control_is_red`

预期（模型推的，没跑过）：当前单测 `1 passed`（放过）；加上 r2 直接建表合法一族单元之后同一改坏红在「r2 直接建表合法 那个单元跑出对照的判据与那条正式跑产物里的那一份不同」。

## 七、模型与复跑

- 目录 `research/prompts/m3-formal-code-b37-r3-sonnet-model/`：`tools/check_sets.py`（sha256 `844b3ffb4a4f8a8e61bcfd62adcb571b3e71307f29b352ad2574f59c4ec3e202`）、`tools/model.py`（`f9146c4462a1dd3fafd09976346933ec178bfded7aa31a3836d8f56967ba8dc8`）、`tools/rerun.sh`（`d43f59f9bbaf51b2cc7ec864747728090c2c652e4173a3246b8b2fcefc59341f`）、`rerun-output.txt`（`dd65b851d817b9bdee9bbb0d895075c1cb25ddf0a02bd68ebcee48f6ba9e25a7`，47 行）。
- 复跑：`bash research/prompts/m3-formal-code-b37-r3-sonnet-model/tools/rerun.sh`，工作目录仓根，默认读冻结副本与仓根；末行「与 rerun-output.txt 逐字相同」。`PYTHONHASHSEED` 取 1 至 4 的输出同一个 sha256。纯 Python，没有要经内存包装的命令。它读仓里 `research/results/` 下 2026-10-04 的产物。
