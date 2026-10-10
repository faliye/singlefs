# 第 37 批代码轮第二轮：云端辩方（Sonnet）报告（2026-10-08）

轮名 m3-formal-code-b37-r2；立场：复核第一轮判决（`research/prompts/m3-formal-code-b37-r1-main-verification.md`），替被改掉的写法（并集判、r3 直接建表那一档）找站得住的理由；判 C1、C2、C4。
代码引文读冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b37-r2/tree/`，路径写 `crates/…`，行号是这棵树里的行号（我核过：本报告引的六份 crates 文件与主工作区逐字节相同）。
没读禁读清单里的文件。没跑任何重型测试，没编 Rust；模型是纯 Python（读源码、`replay.sh` 与入库产物），在 `research/prompts/m3-formal-code-b37-r2-sonnet-model/`，复跑 `bash tools/rerun.sh`。

## 一、判定一览

| 条目 | 辩方判定 | 证据 | 推翻它的现象 |
|---|---|---|---|
| C1-a 五份写死集合 = 登记给这一档的集合 | 站得住 | E1、E2 | 登记原句里有一条我没读到的「某档判 / 不判某条」；或代码谓词改动后模型 `rerun.sh` 的「写死=登记」「写死=代码」任一格为 False |
| C1-b r1 只取合法、对抗两族；r2 至 r5 取三族 | 站得住，附一条够不着（见 C1-d） | E3 | 正式跑里出现一条 `--profile r1 --family multi_disk`（现在在 `main` 里被拒） |
| C1-c 正式跑的每档对照集合是写死集合的子集 | 站得住；r1、r2 两份表都取、r5 这三档的正式跑并起来等于写死集合；r2 直接建表与 r3 两档是「虚档」（正式跑只在一两族起），写死集合比正式跑大 | E3、E4 | 有一个正式跑的对照标签不在写死集合里（模型第 37–41 行任一格「并 ⊆ 写死」为 False） |
| C1-d 五档把「正式跑每一档起的族」钉住了 | 够不着 | E5 | 单测里加一段按 `replay.sh` 每个（档，族）各核一次、或单独核 `Criterion::family` 之后，下面的换族改坏被抓到 |
| C1-e J8「五行」与写死的四条 | 站得住（登记 C8a 至 C8d 四条对照，C8e 是单测断言不是判据） | E2 | 登记 r5 里 J8 另有一条对照没有对应的 `Criterion` |
| C2-a B1-甲「并集放得过某档少跑」 | 站得住（打中成立） | E6、E7 | 主 agent 复跑改前并集单测加坏法变成红（判决第四节记的是 `1 passed`） |
| C2-b B1-乙（r1 段读数为空也照过）、r2 段 | 站得住 | E7 | 改前 r1 段读数为空时旧单测红 |
| C2-c B1-丙「改成正式跑真用的五档」 | 站得住；r2 两份表都取取代 r3 两份表都取是对的 | E3、E8 | 正式跑里有一条 `--profile r3 --table both` |
| C2-d 第一轮采攻方的取舍有没有同样打中别的写法 | 同样打中一处：全族一次调用，`Criterion::family` 的过滤没人钉；这一处旧写法同样放过，不是新添的 | E5、E9 | 同 C1-d |
| C2-e 拿掉「第一档 r3 直接建表」与「r3 两份表都取」会不会让哪条判据失去唯一一份读数 | 旧写法里会（拿掉两档：J7a 三格、J7b、J7c、T_table、T_row 共 7 条）；新写法里不会（每一档都拿掉一遍，只有拿 r2 两份表都取会让 T 两条失去、拿 r5 会让 J8 四条失去） | E8 | 模型第 86–93 行任一格变化 |
| C2-f 替并集判辩护 | 并集判站不住「某档少跑」这一条；只剩两点好处：调用次数与新写法同为 5 次、不用手写集合，新写法保留了它的「每条判据至少有一条读数」那一检 | E9 | 并集写法在 `rerun.sh` 的改坏 A 里抓到的比例追平五档写法 |
| C4-a 四行锚点各恰好一次 | 站得住 | E11 | 门禁 59 号对某行报锚点不是恰好一次 |
| C4-b 点名的测试筛得到、在改后单测下红在断言上 | 站得住（读码判；主 agent 日志旁证） | E11 | 点名测试在 `cargo test` 里 0 条匹配 |
| C4-c 换掉的两行在改前单测下的行为 | 前两行（3383、3407）锚在被测的 `is_judged_under`，在改前并集单测下不红（名字里的「几档并起来判时被 r3 的读数补上」就是这个）；第三行（3496）同 | E7、E11 | 改前并集单测加这三个坏法有一个红 |
| C4-d 第 3497 行「只有 nff 那条杀得掉」 | 成立（限 cargo 测试；e158 产物行次序会变但没有测试罩） | E12 | 找到另一条断言报数次序的测试 |
| C4-e 三处注释 | 与代码对得上 | E13 | 见 E13 |

## 二、C1 的证据

### E1　代码这一侧：两条路，共用五个谓词

- 对照那段循环只经两道筛：「if !families.contains(&criterion.family()) && !criterion.is_judged_in_every_family() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7758`）与「if !criterion.is_judged_under(run_profile) {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:7761`）。
- 哪条判据挂哪个谓词，在「const fn is_judged_under(self, run_profile: RunProfile) -> bool {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6211`）。模型把它解析出来：「is_judged_under 起于第 6211 行；谓词分布： {'compares': 2, 'judges_j6qn': 5, 'judges_j7': 5, 'judges_j8': 4, 'splits_j6qn_by_i812': 1, 'true': 18}」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:8`）。合起来 35 条，与判据表行数相同（`rerun-output.txt:1`）。
- 正式跑判不判走 `main` 的判定行，不经 `is_judged_under`：「if run_profile.judges_j7() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8218`）、「if run_profile.judges_j8() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8237`）、「if run_profile.splits_j6qn_by_i812() && domain.family() == Family::Adversarial {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8257`）、「if run_profile.compares_the_two_record_tables() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8260`）；基础 18 条里对抗一族的 A_limit 不看档，每档都打：「a_limit_violations={}」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:8187`）。
- 所以两条路共用的是五个 `judges_*` / `splits_*` / `compares_*` 谓词（`…:340`、`…:350`、`…:358`、`…:368`、`…:387`），不共用的是「判据到谓词」那张映射。谓词改了，两条路一起变，单测抓不到也不必抓（`main` 跟着变）；映射在某档错了，只有对照这一侧错，`main` 照判——这正是 B1-甲 要防的一类。

### E2　登记逐档对（写死集合 = 登记给这一档的集合）

我先读登记原文、独立写成规则（模型 `registered()` 函数，不看代码），再与新单测里写死的数组、与解析出来的代码谓词各比一次：
- 「对照那段循环遍历判据表的每一行」（`research/prompts/e165-preregistration.md:508`）。表里 J0 与 A0、J1 至 J6、J5a 与 J5b 分得开、AS 三条、A-互、A-结构、A-限、IA1 与 IA2 共 18 条（转述，按标签数出来；新单测数组大小 18 见 `rerun-output.txt:9`），每档都判。
- r2：「多盘域 M、M6、MC 只在这一档有」（`research/prompts/e165-r2-prereg.md:409`）；J6Qn「每个域（两族、多盘）」（`research/prompts/e165-r2-prereg.md:439`）；「原登记 5.4 那张表的每一行在 r2 档照跑」（`research/prompts/e165-r2-prereg.md:450`）；J7a 三格各一条对照、J7b、J7c（r2 登记 5.5 同一张表，第 457 至 459 行）；T-表 与 T-行 只在 `--table both`：「`--table both` 下每个状态两张表逐键逐值比，不等即一态」（`research/prompts/e165-r2-prereg.md:443`）。
- r3：「`r3` 档 = `r2` 档全部原样，另加两样」（`research/prompts/e165-r3-prereg.md:422`），其中对抗一族的拆行：「对抗一族（`--family adversarial`）每个 (域, n) 多出 Q3 的几行（5.6）」（`research/prompts/e165-r3-prereg.md:424`）。
- r5：「`r5` 档 = `r3` 档原样，只改两样」（`research/prompts/e165-r5-prereg.md:420`）——所以 r5 保留拆行，去掉 J7 五条，换上 J8；多盘一族 r5 认（同一行，`--family multi_disk` 在 `replay.sh` 第 861 行起）。
- J8 的「五行」与写死的四条：登记对照表是 C8a「J8a 三格」、C8b「J8a 卸载记号」、C8c「J8b」、C8d「J8c」四条判据对照（`research/prompts/e165-r5-prereg.md:466`、`:467`、`:468`、`:469`），第五条 C8e「J8 无出处与 PH6 那道断言」（`research/prompts/e165-r5-prereg.md:470`）是单测断言、不是 `Criterion`。四个变体与产物标签 J8a、J8a_unmount_marker、J8b、J8c 一一对上（`rerun-output.txt:51` 的 r5 合法历史一族产物标签数 16 = 11 + J6Qn + 这四个）。
- 逐档结果：「r1       写死 18  登记规则 18  代码 18  写死=登记 True  写死=代码 True」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:12`），r2 至 r5 四档同形（`rerun-output.txt:13` 至 `:16`：28、30、29、28 条），五档并起来罩全表：「五档并起来罩全表： True （缺： [] ）」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:17`）。

推翻：登记 r2 至 r5 里有一句我没读到的、改变某档「判哪几条」的话；或有人改了谓词而没改写死的数组（那种改动恰好是单测要红的）。

### E3　正式跑（`replay.sh` 里起 E165 装置的）与族

- r1：合法一族「--family legal」（`research/scripts/replay.sh:703`，轻档同一族在 `:706`），对抗一族「--bounds segment1 --family adversarial」（`research/scripts/replay.sh:710`）。r1 档没有多盘：「if selection.families.contains(&Family::MultiDisk) && !run_profile.judges_j6qn() {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17696`）。
- r2 直接建表只有多盘一族：「--bounds segment1 --family multi_disk --profile r2」（`research/scripts/replay.sh:738`）。三族都在 r2 两份表都取下起：「--family legal --profile r2 --table both」（`research/scripts/replay.sh:741`）、「--family adversarial --profile r2 --table both」（`research/scripts/replay.sh:744`）、「--family multi_disk --profile r2 --table both」（`research/scripts/replay.sh:747`）；反盘次序两条（第 750、753 行）是同一档同一集合（`run_profile_of_a_control` 把表与盘次序旋钮摘掉再补 T，「fn run_profile_of_a_control(control: ControlKind, run_profile: RunProfile) -> RunProfile {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6300`）。
- r3 只有对抗一族：「--family adversarial --profile r3」（`research/scripts/replay.sh:763`）；择根一族 `--family root_choice` 与 r4 的 `root_choice_faults` 走各自的对照，不进 `Criterion`，模型不含。
- r5 三族：「--family legal --profile r5」（`research/scripts/replay.sh:855`）、「--family adversarial --profile r5」（`research/scripts/replay.sh:858`）、「--family multi_disk --profile r5」（`research/scripts/replay.sh:861`），奇偶另一种两条（第 864、867 行）。
- 对照取的族是命令行的族：「positive_control_readings(&selection.families, run_profile, thread_count);」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17769`）。
- 模型逐个正式跑算出对照条数（`rerun-output.txt:20` 至 `:34`），再按档并起来比写死集合：r1 18 = 18、r2 两份表都取 30 = 30、r5 28 = 28；r2 直接建表 4 对 28、r3 8 对 29，都是子集（`rerun-output.txt:37` 至 `:41`）。
- 判定：没有一档「正式跑判了而集合里没写」（并都 ⊆ 写死）。「写了而正式跑不起」有两档：r2 直接建表（24 条）与 r3（21 条）——写死集合是整档（全族）的集合，而正式跑只起一两族。这两档是虚档，不碍：对任何族子集，正式跑的对照集合 = 整档集合 ∩ 族过滤，整档集合对了、过滤也对，就对（过滤没人钉，见 E5）。

### E4　入库产物旁证（2026-10-04 的产物，旧数据，只作旁证）

十个正式跑各取一份产物的 `name=control` 标签，与谓词算出的这一跑的集合比，十格全相等（`rerun-output.txt:44` 至 `:53`，例：「r5       legal      产物 16 种标签，谓词算 16，相等：True」，`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:51`）。产物是旧代码跑的，今天没复跑（复跑是重型），所以只说「十份产物与今天的谓词没有矛盾」。

### E5　够不着的一处：五档用全族一次调用，`Criterion::family` 的过滤没人钉

- 新单测每一档都给全族：「let all_families = [Family::LegalHistory, Family::Adversarial, Family::MultiDisk];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18280`），r1 给合法、对抗两族：「let legal_and_adversarial = [Family::LegalHistory, Family::Adversarial];」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18279`）。全族时第 7758 行那道族过滤对每条判据都放行（T 两条另走 `is_judged_in_every_family`），单测没有读过一次「只选一族」时的集合。
- 正式跑一次只选一族（E3），所以一个正式跑的对照集合 = 整档集合 ∩ 族过滤。把某条判据在 `Criterion::family` 里换到别的族，整档集合不变、单测照过，正式跑却变：该族的正式跑多一条对照，原族的正式跑少一条（判了没对照，B1-甲 同形）。
- 模型把 35 条判据各换到另外两族（70 个改坏）：「换族 70 个，改变某个正式跑的对照集合 66 个；新单测抓到 18，旧单测抓到 0，按正式跑逐个核的写法抓到 66」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:96`）。新单测抓到的 18 条是把基础判据换去多盘一族（r1 没有多盘，r1 那一档集合变小）；其余 48 个放过新单测又改变正式跑：「放过新单测且改变正式跑的换族共 48 个」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:121`），例「J0 LegalHistory -> Adversarial：正式跑变了 [('r1', ['Adversarial']), ('r1', ['LegalHistory']), ('r2-both', ['Adversarial'])]」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:115`）。
- 这是旧写法同样放过的（旧单测抓到 0），所以第一轮的改法没有变坏，只是没补上。判决第三节第 1 条说「五档并起来罩 `Criterion::ALL`」「正式跑用的五档」，罩的是档，不是（档，族）。
- 补法（我没写、没跑，只量了个数）：按 `replay.sh` 的（档，族）逐个核，共 10 次调用、92 条判据的对照，比现在 5 次、133 条少：「按正式跑逐个核（每个 (档, 族) 一次） 10 次调用、 92 条判据的对照」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:126`）对「新单测 {'r1': 18, 'r2': 28, 'r2-both': 30, 'r3': 29, 'r5': 28} 共 5 次调用、 133 条判据的对照」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:125`）。这是判据个数不是挂钟，每条对照扫的域大小不同，没量。
- 推翻：找到另一条单测或门禁按（档，族）核了对照集合（那这一处就罩住了）；或 `family()` 是从别处派生的、不可能单独改。

### E6　C2-a：B1-甲 成立

- 坏法落在哪：变异行 3383 的原文是 `is_judged_under` 里基础 18 条那一整个 `=> true` 臂的末行「| Self::ALimit => true,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6248`；臂从第 6231 行的 `Self::J0Defined` 起，转述），换掉末行的 `true` 就是整臂 18 条换谓词。
- 正式跑确实判了：对抗一族的 A_limit 在每档都打（E1 的第 8187 行）；而对照只由 `positive_control_readings` 里的 `is_judged_under` 筛，那一档少跑它，`main` 不拦，V2 只数跑了的：「if !reading.is_red_as_required {」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:17779`）。
- 改前并集单测放得过：主 agent 的日志（仓外，`~/.local/share/singlefs/work/m3-formal-37b-run/old-union-test-with-alimit-mutation.log`，我读了末几行）是 `test result: ok. 1 passed; 0 failed`。独立旁证是模型的整组改坏：「3383 基础 18 条换成 !judges_j8（r5 档不判）：改变正式跑的对照集合 2 个丢、0 个多；旧并集单测红：False；新单测红：True；新单测红在 ['r5']」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:129`）。
- 量级（我自己的改坏空间：35 条判据各换成另外 6 个谓词之一，共 210 个）：「改坏 210 个：正式跑某个跑丢一条对照（真缺口）170 个，只多一条对照 27 个，正式跑集合不变 13 个」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:56`）；真缺口里旧并集单测抓到 36、新单测 170：「旧单测（并集）                          36/170           2/27              0/13」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:58`）、「新单测（五档各自核）                      170/170          27/27              9/13」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:59`）。这与攻方的 230 个、177 个不是同一个空间（它的数我没复核），方向一致，不拿来互证。
- 推翻：改前并集单测加坏法变红。

### E7　C2-b：B1-乙（r1、r2 两段）

- 改前 r1 段只断言读数全红，读数是空集就一条断言都不走：「for reading in &r1_readings {」（`research/prompts/_m3-formal-code-b37-r2-diff.md:27`）、「assert!(reading.is_red_as_required, "r1 档对照没红：{reading:?}");」（`research/prompts/_m3-formal-code-b37-r2-diff.md:28`）。r2 段只断言没有拆行那一条：「"r2 档不判 J6Qn 的 I-8.12 拆行"」（`research/prompts/_m3-formal-code-b37-r2-diff.md:40`），别的 27 条是否判不看。
- 变异行 3496 在改后单测下：r1 档读数为空，「r1 档跑出对照的判据与登记给这一档的那一份不同」，主 agent 日志里 `left: {}`（仓外 `prove-red-logs/003.log`，我读了）。模型同形：「3496 基础 18 条换成 judges_j6qn（r1 档不判）：改变正式跑的对照集合 2 个丢、0 个多；旧并集单测红：False；新单测红：True；新单测红在 ['r1']」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:131`）。
- 推翻：改前单测在 r1 档读数为空时红。

### E8　C2-c、C2-e：五档的去留

- 新五档在单测里的写法：「RunProfile::R1,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18332`）、「RunProfile::r2(parity),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18338`）、「RunProfile::r2(parity).with_table_source(RecordTableSource::Both),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18344`）、「RunProfile::r3(parity),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18350`）、「RunProfile::r5(parity),」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18356`）。
- 改前取 r3 两份表都取，登记里 `--table` 的意思是 r2 档的旋钮：「`--table direct|scan|both`（不给取 `direct`）」（`research/prompts/e165-r2-prereg.md:409`），`replay.sh` 里带 `--table both` 的正式跑全是 r2（E3），没有 `--profile r3 --table both`。所以 r3 两份表都取是装置里没人起的档，换成 r2 两份表都取是对的。
- 拿掉两档后谁失去唯一一份读数（模型，并集意义）：旧写法拿掉 r3 两份表都取，T_table、T_row 失去；两档都拿掉，J7a 三格、J7b、J7c、T 两条共 7 条失去：「旧单测（r1、r2 只核全红；并集取 r3、r5、r3-both） 去掉 r3+r3-both：并集里没有读数的判据 7 ['J7a_allocation_record_tree_root', 'J7a_filesystem_identifier', 'J7a_instance_table', 'J7b', 'J7c', 'T_row', 'T_table']」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:88`）。旧写法里 r2 档不进并集，J7 的唯一读数只有 r3 两档带着。
- 新写法每档拿掉一遍：「新单测（五档） 去掉 r2-both：并集里没有读数的判据 2 ['T_row', 'T_table']」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:91`）、「新单测（五档） 去掉 r5：并集里没有读数的判据 4 ['J8a', 'J8a_unmount_marker', 'J8b', 'J8c']」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:93`），拿掉 r1、r2、r3 都没有判据失去读数（`rerun-output.txt:89`、`:90`、`:92`）。
- 但「不失去读数」不等于「这一档没用」：逐档核集合时每一档管自己的格子。把 r3 拿掉，模型的真缺口只漏 1 个（J6Qn 拆行换成 judges_j8）；拿掉 r2 直接建表，漏 0 个——这两档只管单格翻转（正式跑里 r3 有 8 格、r2 直接建表有 4 格只在这两档判）：「r3                     判着的格 29，其中正式跑用到的  8；新单测抓到  8，旧单测抓到  0」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:79`）、「r2                     判着的格 28，其中正式跑用到的  4；新单测抓到  4，旧单测抓到  0」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:77`）。辩方观点：r2 直接建表这一档最薄（只罩正式跑 738 行多盘一族那 4 条，且它的 24 条别的格子是虚的），拿掉它损失最小；留着不碍（调用数与旧写法同为 5）。不替主 agent定。
- 推翻：正式跑里加了 `--profile r3 --table both` 或 `--profile r5 --table both` 的驱动（那时两份表都取要多一档）。

### E9　C2-d、C2-f：替并集判辩护，以及采攻方这一步碍着什么

辩方能找到的、并集写法站得住的理由，逐条列，各自的结局：
1. 「调用次数没变多」：成立，但新写法同样是 5 次调用，判据个数 134 对 133：「旧单测 {'r1': 18, 'r2': 28, 'r3': 29, 'r5': 28, 'r3-both': 31} 共 5 次调用、 134 条判据的对照」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:124`）、「新单测 {'r1': 18, 'r2': 28, 'r2-both': 30, 'r3': 29, 'r5': 28} 共 5 次调用、 133 条判据的对照」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:125`）。挂钟：主 agent 日志里改前并集单测加坏法跑 14.30 s、改后单测 6.39 s（仓外 `~/.local/share/singlefs/work/m3-formal-37b-run/` 的 `old-union-test-with-alimit-mutation.log`、`e165.log`，两次负载不同，不是对照实验，只说「没有明显变长」）。
2. 「不用手写集合，不会和 `is_judged_under` 双份」：成立，是新写法的代价；新单测自己说了为什么要写死：「（写死成字面，不从 `is_judged_under` 现算，那是被核的东西）」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18272`）。值是五份集合各一份、判据表加一行就要在单测里的某几份里加一行（`Criterion::ALL` 少了会红，见下一条）。
3. 「每条判据至少有一条读数」那一检：新写法保留了，换成跨五档：「"判据 {} 在正式跑用的五档里都没有对照的读数",」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18392`）。并集写法没有独有的检。
4. 「不会误红」：并集写法在 13 个「正式跑的对照集合不变」的改坏上 0 个红，新写法红 9 个：「旧单测（并集）                          36/170           2/27              0/13」（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:58`）与同表第 59 行。这 9 个是 J7 五条的谓词换成 compares、J8 四条换成 splits（`rerun-output.txt:100` 至 `:108`）：登记说 r2、r3 判 J7、r5 判 J8，改动把这些格子变了，但正式跑一族一族起，r2 直接建表只起多盘、r3 只起对抗，J7、J8 都在合法一族，所以正式跑的输出不变。辩方看法：这 9 个红是登记语义被改的红，不是误红；若主 agent 把单测定位成「只守正式跑的输出」，这 9 个算过严。另有 4 个（多盘四条换成 true）新写法也不红（`rerun-output.txt:109` 至 `:112`），因为 r1 没有多盘。
5. 「并集抓得到少跑」：不成立。170 个真缺口里旧并集抓到 36（E6），旧写法能抓到的只有「某条判据在并集里只剩这一档读数」的格子：J8 四条在 r5（T 两条只在 r3 两份表都取这一档，正式跑不起它；单格翻转的合计：「合计 正式跑用到的格 88：新单测抓到 88，旧单测抓到 4」，`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:82`；抓到的四格全是 r5 的 J8，`rerun-output.txt:83`）。

结论：并集判替自己能辩的只有 1、2 两点，都是代价层面；抓不抓得到「某档少跑」这一条上站不住，第一轮采攻方是对的。

### E10　采攻方这一步另外碍着的

- 这一单测只核偶数奇偶：「let parity = MarkerParity::EvenVersionsWrittenByTheUnmountSequence;」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18278`）。正式跑有 r5 奇偶另一种两条（`replay.sh` 第 864、867 行，E3）。集合相等不依赖奇偶（`is_judged_under` 不看奇偶，E1），但奇偶下的读数是否全红不归这一条管；r5 奇偶档在别的单测里被用到（「RunProfile::r5(MarkerParity::OddVersionsWrittenByTheUnmountSequence),」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:19592`），我没读它断言什么，不下结论。
- 单测现在断言「读数集合」而不再断言「读数多少条」：两种坏法都放得过——同一判据多跑一条对照（集合不变），或同一判据少跑一条（集合不变）。这一格是攻方 C3 的（只加不删），我只记一笔：集合比的是标签，不是条数。
- 一次断言失败只报第一个失败的档（`assert_eq!` 之后单测就停），后面的档不报：定位要一档一档修，不影响抓不抓得到。

## 三、C4 的证据

### E11　四行变异：锚点、测试名、红在哪

- 锚点各恰好一次。我在冻结副本里对三份源文件数了原文串（原样输出）：

  ```
  '            | Self::ALimit => true,' 1
  '                run_profile.compares_the_two_record_tables()' 1
  1
  ```

  第一行是第 3383、3496 行共用的原文（落在「| Self::ALimit => true,」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6248`），第二行是第 3407 行的原文（落在「run_profile.compares_the_two_record_tables()」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:6215`；另两处同名调用是 `if … {` 开头，不含这 16 个空格的前缀，不命中），第三行是第 3497 行的三行原文（`crates/singlefs-core/src/write_accounting.rs:45` 至 `:47`）。每行六段，我用 awk 数过：四行 NF 都是 6。主 agent 的 `crates-mutation-rows.py static-check` 整表结果我没复跑（门禁阶段归主 agent）。
- 四行名字（逐字）：「E165 基础 18 条在 r5 档判了不跑对照（几档并起来判时被 r3 的读数补上）」（`crates/mutations.tsv:3383`）、「E165 T_table、T_row 在两份表都取的 r2 档判了不跑对照（几档并起来判时被 r3 的读数补上）」（`crates/mutations.tsv:3407`）、「E165 基础 18 条在 r1 档不跑对照（r1 段读数为空也照过）」（`crates/mutations.tsv:3496`）、「报数次序里已删 inode 树与墓碑树对调（各种之和不变，core 的自检照绿）」（`crates/mutations.tsv:3497`）。
- 点名的测试筛得到：`every_criterion_has_a_positive_control_and_every_control_is_red` 在 e165 装置里恰好一条（「fn every_criterion_has_a_positive_control_and_every_control_is_red() {」，`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18276`，该函数名在这个文件里 `grep -c` 得 1）；第四行的「fn publish_writes_line_gives_the_total_then_every_kind_in_report_order_with_zeros() {」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1966`）一条。E165 三行第 5 段没有测试名筛，跑的是这个装置的整个测试模块（主 agent 日志里「81 passed; 1 failed」那类行，共 82 条），与表里已有的 E165 行同形：第 1462、1625 行第 5 段也是「-p singlefs-checker-tier --bin e165_replay_bounded_enumeration」，没有筛。代价是每行跑 82 条，不是错。
- 改后单测下红在断言上（读码判）：红的位置是五档循环里的集合相等断言，「controlled, registered,」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18378`）带的消息「"{profile_name} 档跑出对照的判据与登记给这一档的那一份不同"」（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:18379`）。三行各红在哪一档（模型）：第 3383 行 r5、第 3407 行 r2 两份表都取、第 3496 行 r1（`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:129` 至 `:131`）。主 agent 的 `prove-red-logs/001.log` 至 `003.log`（仓外，我读了）依次是 r5 档、两份表都取的 r2 档、r1 档，panic 在同一个 `assert_eq!`（第 18377 行起）；第四行红在 nff 的 `assert_eq!`（日志 `004.log`，第 1971 行）。读码的结论与日志一致。
- 换掉的两行在改前单测下：改前并集单测不红（第 3383 行主 agent 现跑 `1 passed`；三行的模型整组改坏：旧并集单测红都是 False，新单测红都是 True，见 `rerun-output.txt:129` 至 `:131`）。旧的两行原文（锚在旧单测里的 `readings.extend(positive_control_readings(` 那一块）在改后文件里 0 次命中，不换掉就是门禁 59 号的锚点腐化红，所以必须换；换成的新两行不再守「单测自己的形状」。
- 单测形状本身的变异现在没有行守：把 `profiles` 数组里的 r1、r2 直接建表、r3 任一项整项删掉，新单测不红（`Criterion::ALL` 仍被别的档罩住：模型「新单测（五档） 去掉 r1：并集里没有读数的判据 0 []」，`research/prompts/m3-formal-code-b37-r2-sonnet-model/rerun-output.txt:89`、去掉 r2 直接建表、去掉 r3 同形，`rerun-output.txt:90`、`:92`）。改前那两行守的正是这一类（删一档）。这是第一轮采攻方换行时放掉的东西：生产代码侧的行（本批四行）守「某档少跑」，没有行守「单测少核一档」。要不要补是主 agent 的取舍；补法是各加一行删数组项，原文锚在 `RunProfile::r3(parity),` 那一项（`…:18350`）上。

### E12　第 3497 行「只有 nff 那条杀得掉」

- 要换的是报数清单里相邻两项：「WrittenStructureKind::DeletedInodesTreeNode,」（`crates/singlefs-core/src/write_accounting.rs:45`）与「WrittenStructureKind::TombstoneTreeNode,」（`crates/singlefs-core/src/write_accounting.rs:46`）。
- `IN_REPORT_ORDER` 的全部使用方（定义与注释除外，`grep -rn` 得 7 个文件在读它）：core 自检逐种断言每种恰好一笔、按清单相加对合计、字段名去重（「for kind in WrittenStructureKind::IN_REPORT_ORDER {」，`crates/singlefs-core/src/write_accounting.rs:301`，`:330` 排序去重），换位置都不变；`mount.rs` 逐种相加（`crates/singlefs-core/src/mount.rs:6523`）、`crash_injection.rs` 的 `.any`（「WrittenStructureKind::IN_REPORT_ORDER.iter().any(|kind| {」，`crates/singlefs-checker-tier/src/crash_injection.rs:2335`）、harness 的 `deletion_scenario_writes.rs` 过滤求和与 `write_accounting_per_structure.rs` 只比长度，都不看次序；看次序的只有两处：nff 的 `describe_writes_by_kind`（「WrittenStructureKind::IN_REPORT_ORDER」，`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:180`）与 `e158_root_choice_repair.rs` 的 `emit_writes_by_kind`（「for kind in WrittenStructureKind::IN_REPORT_ORDER {」，`crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:4865`）。
- nff 那一条逐字断言整行，已删 inode 树在前、墓碑树在后：「deleted_inodes_tree_node_write_calls=0 deleted_inodes_tree_node_written_bytes=0」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1980`）；对调就红。e158 那处我在整个 crates 里没找到断言它输出次序的单测（`emit_writes_by_kind` 只在装置的跑法里调），所以「只有 nff 那条杀得掉」对 cargo 测试成立；e158 产物里 `kind=` 行的次序会变，复跑产物才看得见，不是测试。
- 推翻：另有一条测试断言报数次序（grep `tombstone_tree_node` 与 `deleted_inodes_tree_node` 只命中 nff 那条断言字符串与 core 的 `report_name` 两行，目前没有）。

### E13　三处注释与代码

- nff 第 1460 行：「warm_up_operation_count（该文件第 189、201 行）量的是同一个边界」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:1460`）。harness 文件第 189 行是声明「let warm_up_operation_count;」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:189`），第 201 行是赋值「warm_up_operation_count = stream.operations().len();」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:201`）。对得上。
- nff 第 2444 行同一句：「`warm_up_operation_count`（该文件第 189、201 行）量的是同一处」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2444`）。对得上。
- nff 第 2446 行：「钉住该文件那五处断言的操作数：25、2、20、35、57」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2446`）。nff 单测断言「vec![25, 2, 20, 35, 57],」（`crates/singlefs-checker-tier/src/bin/new_pool_file_creation_on_device.rs:2476`）；harness 文件五处：「(25, "14+1+1+1+4".to_string(), 16402)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:356`）、第 2 条取号段 `(2, "2".to_string(), 4)`（第 368 行，转述）、「(20, "2+1+2+2+1+2".to_string(), 15)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:380`）、「(35, "24+2+1+2".to_string(), 16_777_223)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:392`）、「(57, "2+2+1+2+2+1+2+24+2+1+2".to_string(), 16_777_240)」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:404`）。对得上。注释里「mkfs 23 → 25」的来由：harness 第 347 行「mkfs 清 [0, 根环基址)（C651 第一轮判决用户定案 W1）之后每块盘多一步整段清零、两块盘多 2 步」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:347`），kb 欠账行「mkfs 先把每盘 [0, 根环基址) 写 0」（`.claude/kb/checks-owed.md:510`）；注释里 C577 一段与欠账行「系统配置没见证到的最新根，乙罩不到」（`.claude/kb/checks-owed.md:666`）对得上题面。
- `image.rs` 第 212 行补的两种：「`replay_the_next_mount_would_do`），一条记录都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验证不过（起点含介质损坏）时报不适用。」（`crates/singlefs-checker/src/image.rs:212`）；代码四臂恰是这四种：「ReplayTheNextMountWouldDo::AnchorUndecided => judgements.not_applicable(」（`crates/singlefs-checker/src/walk.rs:9017`）、「ReplayTheNextMountWouldDo::ChosenRootTxgHasNoSuccessor => judgements.not_applicable(」（`crates/singlefs-checker/src/walk.rs:9021`）、第 9025 行的 `StoppedAtANamedUnitThatDoesNotVerify`、「ReplayTheNextMountWouldDo::NothingApplied => judgements.not_applicable(」（`crates/singlefs-checker/src/walk.rs:9033`）；`Applied` 臂（第 9037 行起）走 `judgements.judge`。walk 自己的文档注释同说四种：「一条都不施加、锚点认不定、所选根 txg 没有后继、前缀停在点名单元验不过（起点含介质损坏）都报不适用。」（`crates/singlefs-checker/src/walk.rs:9001`）。对得上。

## 四、模型与复跑

- 目录 `research/prompts/m3-formal-code-b37-r2-sonnet-model/`：`tools/model.py`（sha256 `514b9c29895c36e6a6828722fe0561123da602d75fac11d4a479071e065d166f`）、`tools/rerun.sh`（`fbb4bace23650d14d9d09bb4b14a525ba07d55ab0d8db08a6d25c1276b1a9d65`）、`rerun-output.txt`（`1e5ef3291a3712a503bbb1b6ddd52315c8b72265ff58e589798133596a4c95b0`，131 行）。
- 复跑：`bash research/prompts/m3-formal-code-b37-r2-sonnet-model/tools/rerun.sh`，工作目录仓根，默认读冻结副本和仓根；有 `rerun-output.txt` 就逐字比，末行「与 rerun-output.txt 逐字相同」。我跑了四遍，都相同。纯 Python，没有 cargo，所以没有要经内存包装的命令。它读仓里的 `research/scripts/replay.sh` 与 `research/results/` 下 2026-10-04 的产物；这两处别的会话改了输出会变（`replay.sh` 现在是 `MM` 状态）。
- 模型的前提（推翻它们就推翻对应数字）：判据到谓词的映射与 `family()` 是从源码解析出来的，不是我抄的；登记规则 `registered()` 是我读 r1、r2、r3、r5 登记写的，出处在 E2；「正式跑」取自 `replay.sh` 里起 E165 装置的行（不含 `root_choice`、`root_choice_faults`、r4）；改坏假设「多一条对照一定红」对旧并集单测是宽的（旧单测少抓不会多抓）。

## 五、没做什么

- 没跑任何 cargo、没跑 E165 装置、没复跑门禁 59 号或 `crates-mutation-rows.py static-check`（重型 / 归主 agent）；E6、E7、E11 里「红在断言上」是读码加读主 agent 日志（仓外 `~/.local/share/singlefs/work/m3-formal-37b-run/` 下，我读了 `old-union-test-with-alimit-mutation.log`、`e165.log`、`prove-red.log` 与 `prove-red-logs/001.log` 至 `004.log`），不是我自己的现跑。
- 2026-10-04 的产物是旧代码跑的，只作旁证（E4）；今天装置复跑出的 `name=control` 行没看过。
- 没判 C3（攻方的格）：只在 E5、E10 记了两笔相关的边（全族一次调用没钉族过滤、集合比标签不比条数），没往下挖；没评估 `--table scan`、反盘次序、r4、r5 加 `--table both` 这几个没进 `replay.sh` 的档。
- 没读 `_m3-formal-code-b37-r2-background.md` 与它的附录、清单（正文、改动全文、冻结副本与登记已够判这三格）；没读禁读清单里的攻方报告与模型。
- 没替主 agent 撤销或维持任何判决：E5 的补法、E11 末条的补行、E8 里 r2 直接建表的去留，都是写出来的选项，取舍归主 agent。

## 六、要主 agent 跑的命令

没有必须跑的。想把 E5 的换族坏法在入库装置上坐实（我只在模型里量过），可选：在草稿副本里把 `Criterion::family` 的 `Self::ALimit` 从对抗一族臂移到合法历史一族臂，然后

`cd <副本根> && SINGLEFS_HEAVY_TESTS=user-request bash research/scripts/run-with-memory-cap.sh 24G bash research/scripts/capped.sh 15 cargo test --release -p singlefs-checker-tier --bin e165_replay_bounded_enumeration -- every_criterion_has_a_positive_control_and_every_control_is_red`

预期（模型推的，没跑过）仍是 `1 passed`；同一份副本上 `--family adversarial --profile r5` 的 `name=control` 行里不再有 `criterion=A_limit`，合法一族多一条。看到单测红，就说明 E5 说的放过不成立。
