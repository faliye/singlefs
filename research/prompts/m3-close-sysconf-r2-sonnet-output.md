# m3-close-sysconf-r2 辩方腿（Sonnet）报告（2026-10-19）

立场：替第一轮被判出局或被攻的四条辩护——S5「不罩」、S5「并进 journal 环」、S2「改法丁照字面」、S1「接受永远只读」——逐条看它们的打中是不是只成立在攻方那一个起点、一个切点上，第一轮判决有没有越位，是不是同样打中所有替代方案（含挂起的形态甲、乙、丙）。T6 为主，T1 到 T5 兼看。
代码只读树副本 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree`，下面引代码写那份副本的绝对路径；kb 行号是 kb 文件自己的。没有读禁读清单里的文件。

## 复跑命令与模型文件

派发提示没给模型目录，按惯例取 `research/prompts/m3-close-sysconf-r2-sonnet-model/`。模型是纯 Python 的取号崩溃点扫描（不碰 crates、不编译）。复跑（整条经内存包装）：

```
bash scripts/hosts/run-with-memory-cap.sh 700M bash research/prompts/m3-close-sysconf-r2-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen /home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree \
  --work <新的草稿目录>
```

交回前我跑过一次，末两行原样：`name=sweep_vs_model_out result=same` 与 `name=rerun_roots repo=/home/fy5090/code/singlefs frozen=/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree model=/home/fy5090/code/singlefs/research/prompts/m3-close-sysconf-r2-sonnet-model work=/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r2-sonnet/rerun1`。`sha256sum` 在模型目录里跑：

```
55fa87fe8b16b4d750190e6cb12b4d241d2e0e282f358bdadcf6a1e77e35cd8e  ./acquisition_cut_sweep.py
f9fff5b0608ca7cc447ae71c214b1963535d75499096a22f9ef5f2908935873f  ./out/sweep.txt
30d20fe6b3776980d04d9ea11823bd3f10eababf8e0e2c005cdc0153a2edcbe0  ./tools/rerun.sh
```

模型是我按 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs` 的写序读出来的抽象，不是入库装置，数不进 kb。写序与臂的定义写在 `acquisition_cut_sweep.py` 文件头：取号写、屏障、单元写、屏障、记录写、屏障、根槽 FUA、轮换、屏障；崩溃状态 = 某一段正在飞，段内每个写可能没落、落了（系统配置槽的写还可能撕裂），之前各段全落；故障 = 崩溃之后每盘最多一槽出问题（翻坏或持续读错），共 25 种。它复现攻方两个观测：单元写那一段上「两盘最新槽都翻坏」取到同一个号（攻方 `probe-lines-tail.txt` 第 2 行），「两盘最新槽都读错」被 R2 拒（同文件第 4 行）。

## 判定一览

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| S5「不罩」被「字面相撞」打中（判决第二节 S5 行） | 字面词句对得上，够不着「出局」：同一个读法也打中今天的 R2 与形态甲里保留的 R2；承诺句在同一段里绑的是丢写的界；该行的结论应是「要改射程句、用户定」，不是「被打中」 | 第二节 T6-1 |
| S5「并进 journal 环」被「只写单元就崩」打中 | 打中成立，但那是 C555 行自己早就写着的残留，且不分出局：模型里它把撞号的组合从 6 个减到 3 个；攻方「checker 0 红」量在撞号之后，撞号之前 I-7.7 ① 红 | 第二节 T6-2、T6-6 |
| S2「改法丁照字面两半都换，重开 C585」 | 对 `record_both` 臂成立；「字面」二字不成立：丁的原 diff 只有一个 `if`，等于 `record_units` 臂（41 条全绿）。不影响候选清单 | 第二节 T6-3 |
| S1「接受永远只读」与射程相撞 | 够不着 δ 独有：相撞的对象是 R2 本身（C596 行记着用户选甲时知情），同一读法也打中形态甲保留的 R2 和「持续读错槽写不进」那一格 | 第二节 T6-4 |
| S1「α 单独站不住」「S1 与 S5 不能分开定」 | 对 α（不扫单元写序）成立；对 α 加单元写序扫描不成立（攻方自己的「没打中的形状」表里有，判决没提）。S1 与 S5 在这一点上可以分开定 | 第二节 T6-5 |
| S5「池级 checker 在这一形上 0 红，当不了判据」 | 观测点的事：撞号之后 0 红，撞号之前（注入之后、第二次挂载之前）I-7.7 ① 红，调查报告量过 | 第二节 T6-6 |
| S6「收」的判别子观测不到 | 打的是「按计数器与实例表认第一条」那个写法；岔路单里的「收」是「与恢复同键」，没被打到；它的真代价是正推写的「失去哨兵」 | 第二节 T6-7 |
| S5「两条腿各自独立推到同一个第三种」 | 提示正文写了「再找第三种」；趋同是真的，但算不上独立的两份证据 | 第二节 T6-8 |
| S3 落「自证不过」；S4 带闸形态没被攻到 | 站得住；我这边没有找到够得着的反驳 | 第四节 T5、第三节 T4 |
| 挂起形态甲「牵连面」 | 判决只列了正推数出的面；模型补两项：崩溃状态与拒挂载状态随取号窗口加倍；R2 路径上持续读错槽写不进时形态甲不比 δ 好 | 第三节 T1、T2 |

## 二、T6：第一轮判决逐条复核

### T6-1　S5「不罩」：判决说「与 D22 已定项 8 射程字面相撞（引文核过）」

判决原句：「「不罩」与 D22 已定项 8 射程字面相撞（引文核过）」（`research/prompts/m3-close-sysconf-r1-main-verification.md:31`）。

够得着的部分：那句话在，词句对得上。射程原文：「容错承诺是单故障：每盘一槽系统配置坏在承诺里；每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里」（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。C555 的故障（两盘最新槽各坏一个）不带「最新根读不出」，按词句落在第一半里。我不否认这一点；正推腿也写了这是一处空白：「条款没写「承诺」是否含可写挂载」（`research/prompts/m3-close-sysconf-r1-sonnet-output.md:17`）。

够不着的部分：引文核过只证词句在，不证读法。判决要的读法是「在承诺里 = 不许出现撞号」（或「= 还能可写挂载」）。这个读法有三处过头：

1. 同一段里的 R2 句过不了这个读法。同一行（`.claude/kb/decisions/22-单元原子性怎么合成.md:178`）接着写：「判不出 ⟺ 有槽设备报读错，且没有一块盘两槽都自证过（R2，用户定，被攻过零轮，原话在变更史）」。R2 拒可写的盘面里，就有「一盘一槽读错、另一盘一槽读回全零」（每盘一槽坏、没有任何崩溃、没有根读不出）。模型里取号之前的原样盘面（没有崩溃）、每盘最多坏一槽的 25 种故障中，R2 拒的有 12 种（一读错加一翻坏 8 种，两盘各一读错 4 种，`out/sweep.txt` 末尾「其中盘面就是取号之前的原样」那一行）。按判决的读法，R2 本身就违反承诺。读法能同时判掉用户已定的 R2，就不能单独拿来判「不罩」出局。
2. 同一行把承诺的度量写成丢写：「丢写的界按类写：全池读回一个值的纯形丢写 120 格，一盘报读错、另一盘读回全 0 的混合形丢写 0 格」，又写「丢写格都落在每盘各坏一槽系统配置、同时最新根一时读不出的双故障形里」（均 `.claude/kb/decisions/22-单元原子性怎么合成.md:178`）。依据一节还写着这笔交换：「混合形多拒 116 格」（`.claude/kb/decisions/22-单元原子性怎么合成.md:187`）——用户为了丢写 0 格，认了多拒可写。承诺在这一段里是「不丢确认过的写」，没写成「一定可写」。C555 量到的后果是撞号，不是丢写：攻方量的格子 `checker_red=[]`（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:2`），没有一格量出丢了确认过的写。
3. C555 立账那一行自己把「不罩」列为出口：「要先定条款有洞，或者故障模型本来不收这个故障（两块盘最新的系统配置槽在之后的写落盘之后都持续坏）」（`.claude/kb/checks-owed.md:362`），并记着用户当时的态度：「用户 2026-09-25 原话「S6 撞号 先不要管 记成欠账」」（同行）。同类先例是 C583：「将来要罩住它，先改那条射程」（`.claude/kb/checks-owed.md:717`），即射程句本来就是靠改句子收窄的。调查报告也把两种读法并排交给主 agent：「算不算条款有洞，取决于故障模型收不收这个故障，查过的条款没写」（`research/prompts/seed540096-investigation/report.md:103`），并记着装置自己的说法：「s23 装置自己的注释称之为「两个故障」」（同行；那个装置不在树副本里，我没核，只转述）。

结论：「不罩」要改射程那一句，这是代价，不是出局。判决第四节已经把它交用户：「S5：承诺含不含「每盘最新一槽在取号之后持续坏」。」（`research/prompts/m3-close-sysconf-r1-main-verification.md:51`）。第二节表里「「不罩」与「并进环」都被打中」那半句（第 31 行）比第四节写得重，应降成「要改射程句」。

同样打中别的臂吗：会。形态甲保留 R2，R2 拒的那 12 种盘面照拒（模型两种取号写下该行相同，`out/sweep.txt` 末尾）；判决读法下形态甲也在承诺之外，除非把「人手工修（α）」算作承诺满足。

什么现象会推翻我这条：用户或 kb 补一句把承诺写成「还能可写」；或量出 C555 的撞号在某一格丢了确认过的写（那时「不罩」要付的就不是一句话）。

### T6-2　S5「并进 journal 环」：判决说「只写了单元就崩的切点上照样撞（切点量过）」

判决原句在 `research/prompts/m3-close-sysconf-r1-main-verification.md:31`。攻方的数是真的：切点在写流第 26 步（共 62 步，之前 18 次单元写），翻坏臂取到 2：「probe=c555_shape fault=flip second_mount=mounted instance=2 checker_red=[]」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:2`）。

够不着的是「所以并进环被打中、与不罩同列」：

1. 这是 C555 行自己写的残留，不是新发现：「取号式子多读 journal 环一路的补丁在调查副本上 225 格 0 红，只写了单元就崩的那几格照样撞号」（`.claude/kb/checks-owed.md:362`）。调查报告量到：补丁让红消失：「三种翻法共 225 格，M2 之后 0 红（主树副本同样 225 格里红 3 格）」（`research/prompts/seed540096-investigation/report.md:113`），红的三格是记录已落、根未落的格：「红的恰好是「journal 环里已有实例 2 的记录、根环里还没有实例 2 的根」三格」（同文件第 42 行）。攻方只量了单元写那一个切点，没量记录写那一段；并进环救的恰是那一段。
2. 模型把整条写序的各段都扫一遍（每段每个子集，25 种故障）。撞号的组合数（总计 600 个，`out/sweep.txt`）：

| 取号写 | 取号式子 | 撞号组合 | 落在哪几段 | 拒（R2 / 盘全废） |
|---|---|---|---|---|
| 一轮（今天） | 系统配置 + 根环（今天，等于「不罩」） | 6 | 单元写 3，记录写 3 | 312 / 42 |
| 一轮 | 再并进 journal 环 | 3 | 单元写 3 | 312 / 42 |
| 一轮 | 再并进单元写序 | 0 | 无 | 312 / 42 |
| 两轮（形态甲，总计 800 个） | 任一 | 0 | 无 | 420 / 63 |

   同一个「照样撞」，只是在数值上把「不罩」的 6 个减到 3 个，没有哪一格比「不罩」更差。判决把两者并列「都被打中」，信息丢了：并进环严格好于「不罩」，差于单元写序与两轮。
3. 残留那一段的后果没人量。调查报告写明：「k=4–22 这几格 M2 也撞号 2，但 M2 之后全绿」，推测是第二次挂载按同一落点重写盖掉了孤儿单元，但「没有逐个核对 M1 的单元是不是全被盖掉」（`research/prompts/seed540096-investigation/report.md:118`）。条款侧的担心在 D18：「重发同一个号一发布，诞生代号等于它的孤儿全部判已发布」（`.claude/kb/decisions/18-块里携带什么信息.md:318`）。并进环与形态甲的取舍，卡在「残留那一段到底有没有后果」这一个没量的数上，判决没有把它列成够判条件。
4. 判决的候选表是互斥的三选一；「并进环，再把残留的单元那一窗口写成射程内的已知代价」这个组合没有被评估。这是零轮形态，我只指出它没被看过，不推荐。

同样打中别的臂吗：「照样撞」同样打中「不罩」（6 个里含这 3 个）；不打中两轮、单元写序。所以它是区分臂的信息，不是淘汰并进环的理由。代价侧：并进环的代价是把读阶段已读的环最大实例代号传进取号式子（正推腿的写法在 `research/prompts/m3-close-sysconf-r1-sonnet-output.md` 的 S5 候选二），不改盘上格式；两轮的牵连面（9 个文件写死世代号、layout 字节表重钉）见判决第二节 S5 行。

什么现象会推翻我这条：量出残留那一段在某一格留下了被已发布谓词当成已发布的孤儿单元，且后果是丢写或读错数据；那时并进环救不了的就是要害，两轮或单元写序才是对的。

### T6-3　S2「改法丁照字面两半都换，重开 C585」

判决原句：「丁照字面两半都换重开 C585，17 条用例转红，会话发布之后 checker 红 I-3.1（量过）」（`research/prompts/m3-close-sysconf-r1-main-verification.md:28`）。

够得着：`record_both` 臂确实 17 条转红、I-3.1 红，主 agent 已复跑（判决附：补跑一节）。这 17 条是仓里现有用例文件自己的格，不是攻方那一个起点上的东西，所以「只在攻方那一个起点成立」这条不用在这里。

够不着：「照字面」。证据三条：

1. 丁的原 diff（c245-r3 攻方原型）：`git show 31128273b^:research/prompts/c245-r3-opus-model/yi-j-mount.diff` 共 30 行，一个 hunk，只换一处 `if`（旧行是 `newest_system_configuration_journal_tail >= version_journal_position` 那一句）。这一句今天在 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5802`，在缺单元那一半的函数里：「if newest_system_configuration_journal_tail >= version_journal_position {」。换这一处等于 `record_units` 臂（41 条全绿，`research/prompts/m3-close-sysconf-r1-opus-model/out/c585-tests-record_units.txt:1`）。
2. 根那一半的 `is_behind_the_version` 是后来才有的：`git log -S"is_behind_the_version" --format='%h %ad %s' --date=short -- crates/singlefs-core/src/mount.rs` 最早一条是 `5d6374691 2026-10-03`，丁的 diff 写于 2026-09-27 至 09-28。diff 作者没见过这一半；今天它的作用写在代码注释里：「旧快照取在一次发布途中（单元与记录已写、根槽 FUA 还没写）、那次发布的根又写在这块盘上的，单元都在，靠根那一半拒」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:5933`）。C585 的「改法 C」是 2026-09-28 用户定的（`.claude/kb/checks-owed.md:722`）。
3. `record_both` 是攻方为「落后与否」换两处写的移植（`research/prompts/m3-close-sysconf-r1-opus-model/tools/patch_behind_arms.py`），不是 diff 的字面。公平地说，D18 的条款句里「落后」是两半共用的一个前提：「世代号最大那份系统配置的 (实例代号, journal tail) 小于所选那一版那次发布末条记录的 jsn，**并且**所选那一版有单元在这块盘上它的落点读不出或字节不同」（`.claude/kb/decisions/18-块里携带什么信息.md:320`）；按条款句的结构读，两半都换也有依据。所以我的判定是：「字面」二字有歧义，按 diff 是 `record_units`，按条款句是 `record_both`；判决把歧义里更坏的一种写成了「字面」。

结果上的影响：没有。两条腿和判决都已同意只换缺单元那一半（`research/prompts/m3-close-sysconf-r1-main-verification.md:28` 末句），岔路单的丁一与丁二照旧并列。只提醒：岔路单里的「丁二」不是 c245-r3 判过零轮的那个丁。

同样打中别的臂吗：会。任何拿掉根那一半的候选（丁二）都重开 C585；丙（缺的份数不到 2）不碰根那一半，不受这 17 条影响，但会不会让「恰好缺一份单元的旧快照」放行，见第三节 T3。

什么现象会推翻我这条：在 c245-r3 的原始提交里找到丁的第二处改动（改根那一半）；或 D18 条款的修订史写明「落后」共用是有意的。

### T6-4　S1「接受永远只读」：判决说「与 D22 已定项 8 射程「每盘一槽系统配置坏在承诺里」相撞」

判决原句在 `research/prompts/m3-close-sysconf-r1-main-verification.md:27`。攻方的盘面是：一盘一槽持续读错加另一盘一次普通断电撕裂轮换（单故障加崩溃），之后每次可写挂载都拒。

够得着：这盘面确实判不出，用例钉着拒（攻方引的用例在 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-harness/tests/system_configuration_witness_is_undecidable_only_when_a_slot_reports_a_read_error_and_no_device_verified_both.rs:139`）。模型里这一类成因（持续读错 1 槽、没有别的故障、崩溃自己撕裂了槽）在 R2 拒的 312 个组合里占 24 个（今天的取号写），形态甲下占 36 个（`out/sweep.txt`「R2拒 的成因分类」）。

够不着的是「所以 δ 不如别的」：

1. 这笔代价是用户选 C331 的甲时知情认下的，原文写在 C596 的成因格里：「一次普通断电撕裂系统配置轮换、或一槽系统配置持续读错，之后每次可写挂载都拒，只读挂载不写盘，那一槽没有机会被重写，池永远只剩只读」，同行记着「用户 2026-09-28 选甲时知情」（`.claude/kb/checks-owed.md:388`）。攻方的盘面一字不差就是这句话。δ 不加任何东西，就是今天。
2. 相撞的对象是 R2，不是 δ。T6-1 的第 1 点已证：判决的读法会判 R2 违反承诺。若这个读法成立，该动的是 R2（用户 2026-10-03 定的），不是在 α 与 δ 之间给 δ 记一笔。
3. 其他候选没有躲开这一类：
   - α（形态甲的出路）不动 R2，判不出的盘面要人到场并断言版本；
   - 正推腿自己写了推翻条件：「发现 `write_system_configuration_slot` 在持续读错的槽上写不成（设备拒写），则 α 在这一形上给不出出路」（`research/prompts/m3-close-sysconf-r1-sonnet-output.md:95`）。模型里这一格：持续读错的槽写不进时，重写之后仍 R2 拒的组合是 120 个（今天的取号写）、164 个（形态甲），与 δ 在这些组合上的结局相同；
   - 这一格在第一轮没被任何一方量过。harness 的读错包装盘「写、写零、屏障原样交给里面那块盘」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-harness/tests/common/mod.rs:1238`），表达不出「写不进」；攻方的重写模拟用不带包装的盘（`research/prompts/m3-close-sysconf-r1-opus-model/opus_sysconf_r1_probe.rs:288`）写、再用它挂载（同文件第 311 行），等于预设「重写之后读得好」，这个预设没量过。
4. 形态甲让拒挂载的状态更多而不是更少：取号写多一轮，多一个「正在写槽」的窗口，R2 拒的组合由 312/600 变成 420/800（比例 52.0% 对 52.5%），盘全废拒由 42 变成 63。多出来的是窗口数，不是每个窗口更坏。

结论：δ 的优点是不新增信任、不新增代码、与今天一致；缺点是没有出路。「与射程相撞」这一条是 R2 的账，应单列，不能记在 δ 名下；判决第四节把 S1 交用户是对的，但第二节 S1 行里这一句让 δ 看起来比 α 多背一笔，其实 α 背同一笔。

什么现象会推翻我这条：量出设备对持续读错的槽的写能成功（那时 α 的 120/164 个组合会消失，α 优于 δ 的幅度变大）；或用户把 R2 的拒可写改成别的取舍。

### T6-5　S1「α 单独站不住」与「S1 与 S5 不能分开定」

判决原句：「α 单独站不住：它不改取号，修完之后取号仍按 `transaction.rs:941` 的式子，攻方的撞号格照中。S1 与 S5 不能分开定」（`research/prompts/m3-close-sysconf-r1-main-verification.md:27`）。

够得着：对 α 的写法（正推腿定义：号取各盘自证槽、根环、journal 环的最大值，不扫单元）成立。取号式子在 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:941`：「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」，只看系统配置槽与根环。模型里 α 在今天的取号写下撞号的组合是 9 个（`out/sweep.txt`，`arm_write=one_round formula=today` 一节的「出路=alpha」行）。

够不着：「α 的变体」没被评。攻方自己的报告里写过这个变体并认为它不中：「读单元区之后打中一不中（推的）；代价是整盘扫单元头，没量」（`research/prompts/m3-close-sysconf-r1-opus-output.md:102`），还把它列进「没打中的形状」表：「管理员工具连单元区写序一起取 max」（同文件第 269 行）。判决第二节 S1 行没提这一条，就把「撞号」记成了 α 这一类的性质。模型里 α 加单元写序扫描（`alpha_scan`）：今天的取号写下撞号 0 个（`arm_write=one_round formula=today` 一节），形态甲下 α 与 α 加扫描都是 0 个。

后果：S1 的出路与 S5 的取号改法在这一点上可以分开定。离线工具读整个单元区的代价大，但它离线、不在挂载路径上，S5 要避免的是挂载路径上多读一路。若 S5 选了两轮，α 本身就够（模型里 α 在形态甲下 0 撞号），也不用扫描。所以「不能分开定」是 α 不带扫描的结果，不是 S1 与 S5 的结构性联系。

同样打中别的臂吗：这个撞号只打中「出路之后取号仍读不到单元写序」的臂：α、只读挂载下定点修复（β，同一个写）、放行类（γ）。δ 不打中，因为它不让池可写。它不打中带扫描的 α，也不打中形态甲之后的 α。

什么现象会推翻我这条：量出离线扫整个单元区的挂钟或内存大到不可用（例如要几天）；或单元头的写序实例在翻坏、撕裂的单元上读不出（扫描漏掉最要害的那几个）。

### T6-6　S5「池级 checker 在这一形上 0 红，当不了判据」

判决原句：「池级 checker 在这一形上 0 红，当不了判据（量过）」（`research/prompts/m3-close-sysconf-r1-main-verification.md:31`）。

够得着：攻方量到的是真的，且每个探针都是挂载之后才取的 checker：`let reds = red_invariants(&image_of(&devices));` 在 `mount_writable` 之后（`research/prompts/m3-close-sysconf-r1-opus-model/opus_sysconf_r1_probe.rs:225`）。

够不着「当不了判据」：撞号之前呢。调查报告在同一类盘面量过：「注入之后、第二次挂载之前，checker 已经判 I-7.7 ① 红，而 `mount_writable` 照样放行、照样取号」（`research/prompts/seed540096-investigation/report.md:103`），单元只落了一部分的格也一样：「k=4–22（只落了单元）：M2 也取 2。M2 之前 I-7.7 红，M2 之后全绿」（同文件第 43 行）。条款也是这么写的：「① fsid 与本池相同的每个根记录、journal 记录、单元写序的实例代号 ≤ 各盘系统配置实例代号的最大者」（`crates/singlefs-checker/invariants/02-根环.md:14`）。所以：撞号之后 checker 看不见（调查报告的推测是第二次挂载按同一落点重写盖掉了孤儿单元，没有逐个核对），撞号之前它红。所以 C555 的判据可以取两个观测点：挂载之前的镜像红（I-7.7 ①）；挂载之后取到的号不等于任何残留单元或记录的写序实例。后一个不依赖 checker。

后果：判决里「当不了判据」应改成「不能在第二次挂载之后取」。这条对三个臂一样（不罩、并进环、单元写序），不分辨臂。

什么现象会推翻我这条：挂载之前的 checker 红在攻方的 26 步切点上没有（我没在那个切点上跑过；模型不含 checker）。

### T6-7　S6「收」的判别子观测不到

判决原句：「攻方：「收」的判别子观测不到（实例表行里没有起始计数器）」（`research/prompts/m3-close-sysconf-r1-main-verification.md:32`）。

攻方打的写法是提示正文里的那个：「按计数器与实例表认本实例第一条」（`research/prompts/m3-close-sysconf-r1-opus-output.md:257`），提示原文对得上（`research/prompts/_m3-close-sysconf-r1-body.md:101`）。岔路单里列的「收」是另一个写法：「收（checker 找前一条的办法改成与恢复同键）」（`research/prompts/m3-close-sysconf-r1-forks.md:10`），也是正推腿的候选：「做法：checker 前一条的找法改成恢复的找法：按 (实例代号, 计数器 − 1) 在全池结果里找」（`research/prompts/m3-close-sysconf-r1-sonnet-output.md:262`）。同键的写法不需要实例表里的起始计数器，checker 里已有同键查找：`fn record_can_enter_a_replay_prefix(`（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker/src/walk.rs:9089`），按 (实例代号, 计数器 − 1) 在全池结果里找。所以攻方的打中够不着岔路单里的收。

同键收的真代价是正推腿写的：「代价：失去唯一的哨兵」（`research/prompts/m3-close-sysconf-r1-sonnet-output.md:266`），即计数器大于 1 的新实例第一条链写错非 0 时没有任何东西查（恢复本来就不查）。判决第二节 S6 行的理由栏只写「一致：不收」与攻方的持续红，没有对上这一条，也没有量这个哨兵实际抓过什么。「不收」没被推翻，但判决是靠两条腿各自的理由凑出来的一致：攻方的理由是对着另一个写法的，正推的理由是代价。

同样打中别的臂吗：不适用（只有收和不收两臂）。

什么现象会推翻我这条：同键查找在池级 checker 的跨盘合并路径上产生新的假绿（例如把 A 实例的记录当成 B 实例前一条）；这要造镜像才知道，我没造。

### T6-8　S5「两条腿各自独立推到同一个第三种」

判决原句在 `research/prompts/m3-close-sysconf-r1-main-verification.md:31`。提示正文在 S5 一节最后写了「再找第三种。」（`research/prompts/_m3-close-sysconf-r1-body.md:96`），两条腿被要求去找第三种；趋同的是「把新号也写进另一个槽」这个最直接的想法。趋同说明想法自然，不说明它在别的维度上也站得住（崩溃状态数、牵连面、新增的窗口）。模型里它有一项代价是两条腿都没量的：多一个槽写窗口，崩溃状态由 24 个变 32 个，R2 拒与盘全废拒各多出一个窗口的量（见第三节 T1）。这一句判决不影响结论，只请别当作两份独立的支持。

### T6-9　判决里我没有反驳的句子

| 判决句 | 辩方判定 | 依据 |
|---|---|---|
| S2 丁只换缺单元那一半，41 条全绿，C593 形挂得上，挂后 checker 与对照逐条相同 | 站得住 | `research/prompts/m3-close-sysconf-r1-opus-model/out/c585-tests-record_units.txt:1`；对照盘面挂载之前就红同样三条（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:8`），所以那三条红是注入的坏副本，不是臂造成的 |
| S3 两条腿一致落「自证不过」 | 站得住 | 攻方自己「找不到」丢写，我的复核见第四节 T5；回头看模型：翻坏臂的撞号只出现在单元写、记录写两段，别的段干净，所以 S3 的两臂差异是 S5 窗口的产物 |
| S4 带闸形态没被攻到 | 站得住 | 见第三节 T4 |
| 附加条件放行（γ）判别子观测不到 | 没核 | 判决标了「推的」，模型不含根环读错，我没有新的材料 |
| 判决第五节：攻方探针只走一个起点一个切点 | 站得住，并且我补了切点 | 模型把整条写序扫了一遍（第二节 T6-2 表） |

## 三、T1 到 T4：挂起的三个形态（辩方的看法）

本节不攻形态，只回答：挂起的形态有哪些面是第一轮判决没写的、哪些数是我从模型或读码里得到的。模型的数是组合数（崩溃状态 × 每盘最多坏一槽的 25 种故障），不是概率，不同故障的发生率没有量。

### T1　形态甲：取号写两轮

模型两种取号写下的总表（`out/sweep.txt`）：

| 取号写 | 崩溃状态数 | 撞号（取号式子今天的写法） | R2 拒 | 盘全废拒 |
|---|---|---|---|---|
| 一轮（今天） | 24（共 600 个组合） | 6 | 312 | 42 |
| 两轮（形态甲） | 32（共 800 个组合） | 0 | 420 | 63 |

逐项：

- 两轮之间崩：取号写第二轮那一段、什么都没落的状态就是这一格。单元、记录、根都还没写，没有任何带新号的产物，所以号被烧掉也不会撞（撞号的定义是新取到的号等于崩掉那一次留下的产物里的号）。0 个撞号。
- 第二轮写一半崩：这一段模型里每个槽写可以撕裂。撞号 0；但有一类是新增的窗口：撕裂一槽加同盘另一槽坏（崩溃加单故障）的组合，盘上这块盘一份自证的都没有，下一次可写挂载拒（C463 的格）。每个系统配置槽写窗口各贡献盘全废拒 21 个组合（取号写一轮、第二轮、发布末尾轮换，各 21），形态甲多一个窗口就多 21 个，R2 拒也多一个窗口的量（108）。这笔增量不是新类型：今天发布末尾的轮换已经是同形的窗口（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:529` 起的 `RotateSystemConfigurationSlots`）。但它让 S4 的盘面更常见，S4 是否给取号开例外因此更要紧。
- 回卷两轮时写失败：模型没覆盖。读码：回卷写覆写的是取号之前最新的那一槽：「回卷写之前一道池屏障（代码审阅第 19 条）：回卷写覆写的是这块盘上取号之前最新的那一槽」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:670`）。两轮之后两个槽都带新号，「回卷成取号之前的最大号」要把两个槽都覆写，次序与屏障没有人写。条款自己容忍回卷不成：「回卷期间盘上出现的「同一集合内代号不等」由 I-7.7（系统配置实例代号不低于根环） 的第 ② 句判成立」（`.claude/kb/decisions/18-块里携带什么信息.md:319`）。所以回卷失败的后果是留下更大的号（烧号），不是撞号；这一格我判不是要害，但回卷的写序要先写出来再谈。
- 取号之后第一次轮换之前每盘再坏一槽：模型里 25 种故障逐种扫，两轮 0 个撞号；今天 6 个。这是形态甲的实打实的收益。

世代号整体加 1 之后现有判法会不会把中间态判红（读条款，没跑）：
- I-7.16：「两个槽都可择时世代号相差不超过 1」（`crates/singlefs-checker/invariants/02-根环.md:23`）。取号写按「自证过的最大世代号 + 1」算：「世代号 = 这块盘两槽里自证过的最大世代号 + 1（两槽都读不出时从 1 起），槽 = 世代号 mod 2」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:554`）。两轮之后两槽世代号差仍为 1，奇偶对得上。
- I-7.7 第 ② 句：「较大者不出现在任何根记录、journal 记录、单元写序里」（`crates/singlefs-checker/invariants/02-根环.md:14`）。两轮之间崩，两盘的号可以不等，单元之前没有产物，② 判成立。
- I-7.15 比的是设备数，取号写不改它。
- 择系统配置按盘取世代号最大的可择槽、只比 fsid，对不上整池拒：「return Err(RecoveryFailure::SystemConfigurationsDisagree);」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/recovery.rs:1448`），不看世代号跨盘是否相等。

崩溃点重放状态数：模型抽象里 24 变 32（加 8，三分之一），是第二轮窗口的量；入库的 `enumerate_crash_states` 没跑，真实增量我没有数。

### T2　形态甲之上的离线出路

- 修完之后再可写挂载会不会撞号：形态甲之后 α 与 α 加扫描都是 0 个（模型，`arm_write=two_rounds` 各节的「出路」行）。今天的取号写下 α 不扫描是 9 个，见第二节 T6-5。
- 断言错一位时会不会一个字节不写：这是 α 的构造，不是模型能证的：正推腿写「与断言不等，一个字节不写就拒」（`research/prompts/m3-close-sysconf-r1-sonnet-output.md:58`）。要用例钉反向格，这是写进岔路单的够判条件，不是我量的。
- 持续读错、写不进去的槽：模型里 α 没有办法写进它，重写后仍 R2 拒的组合是 120 个（一轮）与 164 个（两轮），与 δ 的结局相同；见第二节 T6-4。形态甲没有改变这一类。
- R2 收掉的那一形（每盘一槽坏，同时最新根一时读不出）：α 的版本断言只与只读口径读出的所选版本比。盘面上最新根读不出时，所选版本是旧的，操作者若照只读报告断言它，α 照写，池重新可写，而最新根正读不出：这就是 R2 本来要挡的丢写。正推腿已写下这个边界：「罩不住的历史：断言者断言了一个不是最新的版本」（`research/prompts/m3-close-sysconf-r1-sonnet-output.md:68`）。意思是 α 不是自动出路，它把判断交给人。判决第四节 S1「出路许不许靠人断言」交用户是对的，这一点要写进给用户的话里。

### T3　形态乙

「本盘环槽里有那一版末条记录」这一判在三种盘面上（读码，没跑）：
- 环转过一圈：落点是 `(counter − 1)` 对环槽数取余（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/journal.rs:204` 起的 `record_offset`）。落后盘的那一槽被后来的记录盖过时，读到的 (实例, 计数器) 与所选那一版不等，判成不带那条记录，回到今天的单元核，方向保守，不会多放。
- 记录只在一块盘上：记录写到每块盘之间没有屏障（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:495`），有记录的盘按乙不落后、跳过单元核（C593 放行）；没有记录的那块盘单元仍都在（单元在记录之前的屏障里持久），单元核通过，只有这块盘再缺一份单元才拒。这一格乙不覆盖，正推腿推荐丁丙就是为了它。
- 旧快照在发布途中取下：根那一半不动，C585 的格靠它拒。`record_units` 臂 41 条全绿就是这个。

但这个「41 条全绿」缺最宽的一条。用例文件里扫描那一条被每条臂都跳过了：攻方报告写明「每条臂都跳过了」（`research/prompts/m3-close-sysconf-r1-opus-output.md:282`），结果行里那个数是「41 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out」（`research/prompts/m3-close-sysconf-r1-opus-model/out/c585-tests-record_units.txt:1`）。被滤掉的那条扫描：`for snapshot_after_overwrites in [0_usize, 1] {`（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:360`），旧快照取自 0 次或 1 次覆盖写之后，换哪块盘，之后会话发 1、2、3 次或卸载一次，共 16 格。第一轮判决第二节 S2 行用「41 条全绿」支持丁只换缺单元那一半，没提这条没跑。判决附录的补跑照的是同一条 cargo 命令，命令里写着「--skip stale_snapshot_substitution_is_refused_by_every_entry_over_the_swept_user_steps」（`research/prompts/m3-close-sysconf-r1-opus-model/tools/rerun.sh:87`），同样滤掉了它。这条扫描里「取自 1 次覆盖写之后」的旧快照，恰是只落后一次发布的盘，也是「缺的份数不到 2」会不会放行它的试金石。

「缺的份数不到 2」会不会让 C585 的旧快照重新放行：用例文件里点名的几格，旧快照取自第一个文件那一刻（txg 3）、现行 txg 5，落后两次覆盖写（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-harness/tests/entries_after_mount_refuse_swapped_or_behind_devices.rs:15` 的起点说明：「起点同攻方：两块 4 GiB 内存稀疏盘上 mkfs、取号、暖机、第一个文件（txg 3），会话里覆盖写两次（txg 4、5）。」），缺的单元不会只有一份，丙不放行它们。只落后一次发布的格在被滤掉的扫描里，我没数。

三条池级 checker 红（I-2.1、I-4.8、I-7.4）是不是另一笔账：是，并且不是乙的账。对照盘面（系统配置跟得上、只缺一份单元，今天就放行）在挂载之前就红同样三条：「probe=c593_control_before_mount arm=tail checker_red=[」（`research/prompts/m3-close-sysconf-r1-opus-model/out/probe-lines-tail.txt:8`）。红的是注入的那份坏副本，可写挂载没有补写它。该落在「放行『单份坏』却不补写」这一条款自己的欠账上：D18 已定项 11 写放行两种，「系统配置跟得上、只缺几份单元（单份坏，由 D19（块指针的结构与宽度预算） 已定项 5 硬规则 1 的读盘核隔离）」（`.claude/kb/decisions/18-块里携带什么信息.md:320`），所以归 D19 读盘核隔离那一侧，与 C593 的「落后加缺单元」不是同一条。

### T4　形态丙

闸是三件：本盘带所选那一版的全部单元（清单非空、逐个读到、字节与验得过的那一份相同），根槽那一半不缺，补写失败按取号失败回卷。读码与读测试名，没造盘面：

- 空盘：单元读不到，闸拒。清单为空（所选那一版没有单元）时「清单非空」也拒，所以刚 mkfs 的池不会被补写。
- 别池的盘：单元头记着 fsid，I-1.4 要它等于系统配置 fsid：「任一单元，其头记录的 fsid == 系统配置 fsid」（`crates/singlefs-checker/invariants/01-块自描述.md:9`）。别池的盘与本池同槽同字节几乎不可能，闸拒。
- 对调的盘：盘体对调今天靠自证槽里记的本盘设备号挡（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:709` 的 `OwnDeviceNumberDiffersFromTheIdentityHandedIn`，用例有对调的盘体经四个入口拒）。这个判需要对调的盘上有一槽自证。丙要补写的盘恰是两槽全废的，它没有设备号可比。第一版两盘，另一块盘的槽自证，它的设备号与盘表位置对不上就由那一块盘拒；两块盘都两槽全废则读阶段就判不出、拒，到不了补写。所以「对调的盘带着同一版全部单元」这一形，在两盘池里补写的是一份完整副本、对调后另一块盘自己报警，我推它漏不过；三盘池第一版不收。这条是推的。
- 先例：要防的坏处在代码里有一条旧的：空盘被入口当成成员接收的后果写在注释里：「卸载报成功、空盘收到 28 次写，之后真盘再挂可写被拒、池级 checker 红 4 条」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:717`）。丙的闸必须挡住的就是它；正推腿把这个测试名列进要翻的用例，没有核它们每格断言什么（它自己的「没做什么」写着）。
- 补写之后到第一次轮换之间池级 checker 红不红：补写的是世代号 1 的一槽（另一槽仍坏）。I-7.16 对世代号小于 2 不判奇偶，也只在两槽都可择时判差值（`crates/singlefs-checker/invariants/02-根环.md:23`）；I-7.15 比设备数，同；I-7.7 号等于其余盘，不触发 ②。推的，没跑。

判决说丙「被攻过零轮」（`research/prompts/m3-close-sysconf-r1-main-verification.md:30`）：这个说法是对的，我这一节也只是读码，仍是零轮的推。

## 四、T5：S3、S6 再抽一次（辩方立场的一次读法）

规则要求「没打中」再抽一次要用同立场的另一条腿（`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」）。我是辩方，不是那条同立场腿，下面不当作那一次抽样；只是把我读到的写出来，供主 agent 对照。

### S3「自证不过」那一边：单故障加一次断电，会不会丢确认过的写或撞号

读法（读码，加模型里的槽写序）：
- 发布末尾的轮换每次覆写这块盘较旧的那一槽，另一槽是上一次轮换：「取号写覆写的是这块盘上较旧的那一槽，另一槽可能是上一次发布末尾轮换刚写的」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/transaction.rs:1136`，同一个口径用在发布末尾的轮换上，`RotateSystemConfigurationSlots` 在同文件第 529 行）。轮换是逐盘写（同文件第 533 行的循环），发布在轮换之后那道屏障之后才返回。
- 单故障：别池的槽盖掉盘 0 的一槽（fsid 不符，算自证不过）；崩溃：撕裂盘 1 正在写的那一槽。设上一次确认过的发布的 tail 为 t，这次在飞的发布的 tail 为 t 加一。轮换前盘 0、盘 1 各有「上一次轮换的槽」带 t，「更老的槽」带 t 减一。在飞的轮换覆写的是更老的槽。盘 1 撕裂的是它更老的槽，盘 1 带 t 的槽完好；盘 0 的故障最多盖掉带 t 的槽（盘 0 在飞的写若已落，带 t 加一的槽也可能被盖）。要见证值落到 t 以下，必须两块盘带 t 的槽同时没了——单故障加崩溃做不到，需要第二个故障，那是 C583 的双故障形（承诺之外）。
- 所以「自证不过」这一边在单故障加一次断电下不丢确认过的写（推的）。攻方自己的报告也写了找不到：「「算自证不过」在单故障加一次崩溃下丢写」「推的，找不到」（`research/prompts/m3-close-sysconf-r1-opus-output.md:272`）。
- 撞号：别池的槽盖掉的是一块盘上的一槽；撞号需要两块盘上带新号的槽都没了（模型里撞号只出现在两盘最新槽都翻坏的组合，且集中在单元写、记录写两段）。单故障下另一块盘的新号槽还在，式子看得见，不撞。

「算读错」那一边的打中（攻方：错向写加断电永远只读）与我这边读到的一致，不重复。

此外有一条判决没提：S3 表里「整盘换成别池的盘、只换一槽而别池槽世代号更高」在 R2 之前就整池拒（正推腿 S3 一节的前置事实，代码就是第三节 T1 引过的 `SystemConfigurationsDisagree` 那一处），所以 fsid 不符真正走到 R2 的只有「别池的槽世代号不高于本盘自己的槽」这一小块，两边的差异盘面比判决表里看起来少。

### S6「不收」：没有读故障的崩溃状态里，这一格会不会出现

读法（读码）：
- 新实例第一条记录的计数器：「let next_counter = records」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-core/src/mount.rs:8074`），取扫环读得出的最大计数器加一。
- checker 这一判：「Some(_record_of_another_instance) => ExpectedBackChain::Value(0),」（`/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r1-tree/crates/singlefs-checker/src/walk.rs:9058`）。
- 这一格红，需要新实例的某条非第一条记录，其前一槽坐着别的实例的记录。新实例从读得出的最大计数器加一起写；上一个实例已经不会再写，它在每块盘上的残留，只要挂载那一刻读得出，计数器都小于新实例的第一条。新实例第一条之后的记录，前一槽就是新实例自己的记录或空槽。所以前一槽坐着别的实例的记录，只有「那条残留在新实例挂载扫环时读不出」才可能（读故障，或残留本身撕裂；撕裂的记录永远自证不过，不会在后来变得读得出）。没有读故障的崩溃状态里造不出来。这条与两条腿的结论一致，我得到的是同一个结论，不是第三份独立样本。
- 攻方的造法同样需要读故障：「一次多记录发布、挂载扫环时残留那一槽一次瞬时读错、发布途中崩溃三样」（`research/prompts/m3-close-sysconf-r1-opus-output.md:58`），并且产品路径要多条记录需要点名项很多：「产品路径要点名 65 项以上」（`research/prompts/m3-close-sysconf-r1-opus-output.md:245`）。

「持续红直到环转一圈」是攻方的推；对「不收」的牵连：红的只是 checker（恢复照常），要靠射程句写明。崩溃点重放里 E165 那一族会不会碰上，攻方写的是「碰上它也会报红，要另记已知红」，没有造出来，我也不能确认。

## 五、没做什么

- 没替主 agent 撤销或维持判决。上面每条「够得着 / 够不着」是辩方复核，不是改判。
- 没编译 Rust、没跑 cargo、没跑任何用例文件；没跑崩溃点重放、`enumerate_crash_states`、checker。模型是纯 Python 的抽象，不是入库装置，数不进 kb（`.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）。
- 模型没有的东西：根环读错与「最新根一时读不出」那一形；journal 记录的计数器与反向链；单元的真实落点与孤儿单元被已发布谓词判成已发布的后果；多于两块盘；故障的发生率。
- 没造盘面验证 T4 的补写闸，也没跑第二个起点或别的几何。
- 没读禁读清单里的文件。没读攻方这一轮的产出。
- 没写 kb、没动 `crates/`。模型目录 `research/prompts/m3-close-sysconf-r2-sonnet-model/` 是新建的（派发提示没给模型目录，按惯例取名）；草稿目录 `/home/fy5090/.local/share/singlefs/work/m3-close-sysconf-r2-sonnet/` 里只有 `rerun1/`（复跑输出），不是建的仓副本或编译目录，留着，主 agent 看过可删。

## 六、要主 agent 跑的命令

第一轮 41 条全绿缺最宽的那条扫描（见第三节 T3）。在攻方 `rerun.sh` 建好的树里，对 `tail` 与 `record_units` 两臂各跑一次整个用例文件、不带 `--skip`，看 `test result` 行与红的条数。它是 harness 档，不是重型测试：

```
cd <攻方 rerun.sh 的 --work 目录>/tree && \
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 entries_after_mount_refuse_swapped_or_behind_devices; 范围=1 个测试目标; 为什么=补第一轮判决里被 --skip 滤掉的 16 格扫描，看 tail 与 record_units 两臂' \
SINGLEFS_PROBE_BEHIND=record_units nice -n 19 bash scripts/hosts/capped.sh 31 cargo test --offline -p singlefs-harness --test entries_after_mount_refuse_swapped_or_behind_devices
```

`SINGLEFS_PROBE_BEHIND=tail` 同样各跑一次。线程上限取派发提示的 31，内存上限由 `scripts/hosts/run-with-memory-cap.sh` 的默认或主 agent 指定。判结果：两臂的 `test result` 行都应为 42 passed；`record_units` 若出现红，红的格名就是丁只换缺单元那一半在扫描里重开 C585 的格。

## 七、什么现象会推翻本报告的结论（汇总）

- T6-1：用户或 kb 补一句承诺含「还能可写」；或量出 C555 撞号在某格丢了确认过的写。
- T6-2：量出单元写段上留下的孤儿单元被已发布谓词当成已发布，且后果是丢写或读错数据。
- T6-3：在 c245-r3 原始提交里找到丁的第二处改动；或 D18 修订史写明「落后」共用是有意的。
- T6-4：量出设备对持续读错槽的写能成功（α 优势变大）；或用户改 R2 的取舍。
- T6-5：离线扫单元区的挂钟或内存大到不可用；或扫描在翻坏、撕裂的单元上漏掉要害。
- T6-6：挂载之前的 checker 在攻方 26 步切点上不红。
- T6-7：同键查找在池级 checker 的跨盘合并路径上造出新的假绿。
- T3：第六节的整文件跑法在 `record_units` 臂出现红。
