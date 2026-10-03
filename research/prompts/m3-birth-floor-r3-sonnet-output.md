# m3-birth-floor-r3 辩方腿（Sonnet）报告：复核第二轮判决；替 gap 之外的处置辩护；K3、K4、K5（2026-10-02）

<!-- doc-lint:not-numbers V1 V2s V3s TF K1 K2 K3 K4 K5 J1 J2 J3 J4 J5 N1 N2 N3 N4 CS1 CS2 BI L1 R2 R3 -->

批次：records/2026-09-30-里程碑三开工第一轮.md#5。立场：辩方（终轮），复核 `research/prompts/m3-birth-floor-r2-main-verification.md`，分工见 `research/prompts/_m3-birth-floor-r3-body.md` 第四节辩方那一行。读冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/`：对 `research/prompts/m3-birth-floor-r3-snapshot/tree-sha256.txt` 跑 `sha256sum -c`，OK 3054 行、不是 OK 的 0 行。kb 与代码行号都是冻结副本里的；攻方日志（第二轮攻方模型目录的 `logs/`）与判决在主仓。没读禁读清单里的文件（这一轮攻方的提示与产出、同批 m3-ring-r3 的产出）。没编译 Rust，没跑 checker 档任何东西。

我的模型在 `research/prompts/m3-birth-floor-r3-sonnet-model/`，纯 Python、不读 `crates/`。关键点：第二轮辩方模型（`birth_floor_v2.py`，我原样拷来）里 inode 树是「一个根、一片叶」，覆盖写总是整棵新写；真实现里 inode 树有两片以上叶，覆盖写只重写其中一片、别的叶照抄上一版（第二轮攻方 J1-a 那段历史里红的单元就是照抄回来的旧叶 L2₄）。这个形状在旧模型里表示不出来，所以旧模型上「TF 0 红」的数对 J1-a 那一形说不了话。我另写 `birth_floor_v3.py`（两片叶：覆盖写重写低号叶、建 inode 重写高号叶，其余同 v2）重量。v3 是零轮的抽象模型，数不进 kb。

## 先说结论

| # | 判决条目（第二轮判决第二节行号） | 辩方判定 | 证据 |
|---|---|---|---|
| 1 | 行 1 J1-a：短 journal 环上回退根槽被盖、记录被盖，候选集出现洞，TF 红；四句「不分辨臂」 | **站得住；同样打中别的臂，连第二轮判决第 9 行采的谱系区间形也打中** | 攻方日志：「tf-red-state:layer0-green evaluated=700 violations=696 states=696」（`research/prompts/m3-birth-floor-r2-opus-model/logs/crash-head.log:33`）；β 同数：「beta-red-state:layer0-green evaluated=700 violations=696 states=696」（`research/prompts/m3-birth-floor-r2-opus-model/logs/crash-head.log:19`）；I-3.1 不适用且层 0 无红：「tf-red-state:I-3.1=not-applicable evaluated=696 violations=696 states=696」（`research/prompts/m3-birth-floor-r2-opus-model/logs/crash-head.log:31`）。两片叶模型复现同形：深度 8、12 618 652 张，TF（scp）红 83 620、β（scp_all）红 173 848、V1 系（v1p）红 83 620、判决第 9 行的谱系区间形（lin_tree）红 83 620（`r3c_ROLL8.out`、`r3b_lin_ROLL8.out`）。四种写法同红 ⇒ 共用前提「候选集无洞」是病根，判决第 1 行的处置对 |
| 2 | 行 2 洞的处置：采 gap（R.txg ≠ R⁻.txg + 1 整对不适用，零轮）；理由「两条腿各自独立推出同一类改法」 | **gap 本身站得住；「两腿独立印证」够不着 TF** | 辩方上一轮那条「V1 系加丢槽不适用 0 红」我重跑对得上（12 618 652 + 16 307 544 = 28 926 196 张，v1p 红 41 810 / 12 973，skip 后 0；`deep_ROLL8.out`、`deep_ROLLKX8.out`），但同一批上 TF 本来就 0 红（scp 0），所以它没检验 TF + gap。TF + gap 的独立印证现在才有：两片叶模型上 TF 不处置红 83 620 / 25 946，gap 后 0，28 926 196 张全 0（`r3c_ROLL8.out`、`r3c_ROLLKX8.out`）。另：判决写「缺口今天只出现在根槽读坏、被抛弃的根、根环旧槽、可写挂载几处」不全，见本表下的补充 |
| 3 | 行 3 J1 其余形状没打中（根槽读坏、被抛弃的根与回退同在、放开动作） | **站得住（一次抽样）**；我的模型含 X、L、K 同在的 16 307 544 张，gap 后 0 红 | `r3c_ROLLKX8.out`：scp / scp_all 不处置 25 946 / 36 184 红，gap=all 与 gap=lost 都是 0 红。攻方 2 712 段数得对：八份 sweep 日志各一行 `R2_JOBS`，加起来 2 712（364 + 364 + 364 + 528 + 364 + 243 + 242 + 243，命令见 G1） |
| 4 | 行 4 J2 β 再次出现在合法镜像上没打中（洞两侧除外，由 gap 处置） | **站得住** | 两片叶模型：β 比 α 多红的 90 228 张（173 848 − 83 620）全在洞上，gap 后 0（`r3c_ROLL8.out`）；不带洞的合法镜像上 β 不比 α 多红 |
| 5 | 行 5 J3 坏镜像：J3-c（链改到 = R⁻.txg）只红 TF；J3-d（再次出现）只红 β；链改到 R⁻.txg 以下全漏 | **站得住**，原样行在日志里；J3-d 在入库装置上要主 agent 重做（K3） | J3-c：「BJ J3-c chain→5+count pool_checker_violated=[]」（`research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r2.log:61`）与「CS2:tf-lenient evaluated=96 violations=8」（`research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r2.log:65`）；J3-d：「BJ J3-d/step3-mapping pool_checker_violated=[]」（`research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r2.log:223`）、「CS2:beta-re-lenient evaluated=2 violations=2」（`research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r2.log:225`）、「CS2:tf-lenient evaluated=114 violations=0」（`research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r2.log:229`） |
| 6 | 行 6 复核第一轮：V1 出局站得住；引文更正（`walk.rs:5977` 改 `image.rs:994` 一带）；取 TF 不取 V2s / V3s，理由改写成「带 V1 条件在丢槽形状上红」 | V1 出局与引文更正**站得住**；取 TF 的**取法站得住、改写的理由够不着** | 引文：第一轮冻结副本里 `walk.rs:5977` 是候选集那一行，这一份冻结副本里同一句在 6766：「(!abandoned && !below_floor);」（`crates/singlefs-checker/src/walk.rs:6766`，那一行以 `let walked = *index == newest_index` 起头，竖线在表格里转义不了，只引后半）；读不出的根静默丢掉在：「pub fn valid_roots(reader: &dyn ImageReader, geometry: &PoolGeometry) -> Vec<(u64, u64, RootView)> {」（`crates/singlefs-checker/src/image.rs:994`）。理由：改写的理由靠辩方上一轮一片叶模型里「TF 0 红、V1 系红 41 810」；两片叶模型上同一批 12 618 652 张 TF 与 V1 系都红 83 620，加 gap 都是 0，丢槽形状不再分辨 TF 与 V1 系（`r3c_ROLL8.out`、`r3c_ROLLKX8.out`）。V1 系仍多抓「只有 extent 链改小」：两片叶模型 3 620 / 3 620 对 TF 0 / 3 620（k = 2、3，`r3e_bad.out`）；TF 多抓的在这套里没有（模型里两者都取严格小于；J3-c 只红 TF 不红第一轮攻方的 V3，那一格归「严格小于」那处收严，不归「V1 条件」，见 K3-3）。这不翻判决：V3s 是零轮、真实现里没跑过，只是 TF 与 V3s 的取舍在 gap 之后没有证据分辨，该写成「没分辨，取 TF 是因为措辞少一个跨树条件」，不写成「V3s 红合法镜像」 |
| 7 | 行 7 替 α 辩护被 J3-d 推翻，维持用户定的 β | **站得住** | J3-d 三步造法 0 红的日志行见第 5 行；I-3.9 的区间判法确实只在带已释放记录时红：「环上有洞时（被施加的 journal 记录做的释放写着一个根槽从没写过的 txg，实测 2026-09-18 残留记录那条层 0 流的 12 个状态）按区间判」（`.claude/kb/invariants.md:151`）。判决引的 `walk.rs` 3496、3519：「let Some(last_referencing_txg) = scanned」（`crates/singlefs-checker/src/walk.rs:3496`）、「let in_the_witnessed_interval = record.generation > last_referencing_txg」（`crates/singlefs-checker/src/walk.rs:3519`）对得上 |
| 8 | 行 8 J4 措辞三处 + 代价 | **站得住，要补四处**（K4 写进措辞） | ① 同 txg 并成一条：新实例首发取最大值加一：「根环全部自证过的根里最大的 checkpoint_txg：新实例的第一次发布取 max(它, 环里全部自证通过的记录的 checkpoint_txg) + 1」（`crates/singlefs-core/src/recovery.rs:1289`），合法下造不出同 txg 两条根，措辞里保留「R 侧逐条判、R⁻ 侧取并」无害；设备失而复得时能造出（推的，没造，归 K2）。② b 自己的头读不出时 birth(b) 没有值，措辞没写，补「这一个落点不判」。③ 诞生代号表按第 0 条位置键、落点集合按每份位置条目记：「birth_txg_of_walked_units: BTreeMap<(u32, u64), u64>,」（`crates/singlefs-checker/src/walk.rs:341`）、「placements_referenced_in_this_version: BTreeMap<(u32, u64), String>,」（`crates/singlefs-checker/src/walk.rs:304`），差集要先折成单元键。④ 实例表链页不进诞生代号表：「**打包记录类型 4 的实例表单元不进这个函数**」（`crates/singlefs-checker/src/walk.rs:542`），这些页的诞生代号要另取（取指针里的 birth_txg，推的） |
| 9 | 行 9 J5：谱系记录落地后 (b) 收成「R 的谱系树里有一条 R⁻.txg < T_rb ≤ R.txg 的记录、树根指针出生 txg ≤ T_old」，辩方模型 0 红、整条链改小 36 306 张全抓 | **够不着（一半）**：区间形在真实的多片叶结构上红合法镜像，必须与 gap 并存；它不是 gap 的替身 | 两片叶模型：区间形（lin_tree）不处置红 83 620 / 25 946，加 gap 后 0（`r3b_lin_ROLL8.out`、`r3b_lin_ROLLKX8.out`）。原因：洞之后那条覆盖写根的 inode 树根是新写的（出生代号 > T_old），树根指针条件不成立，而它底下照抄回来的旧叶诞生代号 ≤ T_old。辩方第二轮报告 C2 节「lin_fix 在 12 618 652 张 0 红」是一片叶模型的人造物。谱系树的字段够判不变：「T_rb 是回退那次发布的 txg，T_old 是 R_old 的 txg」（`.claude/kb/decisions/18-块里携带什么信息.md:569`）、「墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」（`.claude/kb/decisions/23-journal的角色与格式.md:377`）。格级形见下节 N4 |
| 10 | 行 10 短环终态镜像上 I-3.1 红，不挡这一轮，记下一批 | **站得住；与 I-3.5 的关系见 K5**：同一个洞的两个后果，不冲突、不互相掩盖 | K5 |
| 11 | 第三节改法的射程句「整条链改小到 R⁻.txg 以下判不出、缺口右边第一次出现的不判」与「实现与写回等用户定」 | **站得住**；「缺口右边第一次出现的不判」的量在窄形里可以缩（N3），写进 K4 措辞时按用户选的那条写 | N3、K4 |

### 行 2 的补充：gap 的触发源与已有的量

判决列的缺口来源不全。txg 不相邻还来自两处合法路径：
- 新实例第一次发布的 txg 取环里全部自证记录的最大值加一，不看那条记录有没有提交标记、有没有被施加：「let highest_record_txg = records」（`crates/singlefs-core/src/mount.rs:1153`）；调用点：「let first_txg = first_txg_of_new_instance(devices, &system_configuration, &records)?;」（`crates/singlefs-core/src/mount.rs:4685`）。`records` 是读阶段扫出的「环里全部自证过的记录」，丢掉的无提交标记的尾巴记录也在里面（没看提交标记有没有滤，推的）。这样一次恢复丢了一条尾巴记录，新实例的根就比最后一条根大 2，留一个 txg 的洞，没有任何根或可施加的记录在里面。
- 根槽写失败重发：「根槽写失败重发时 checkpoint_txg 推进一格再发」（`.claude/kb/decisions/23-journal的角色与格式.md:398`），推进的那一格没有根。

已有的量（都是攻方日志，副本上的数）：
- 不带故障的 2 712 张终态镜像：八份 sweep 日志里 CS2 的 TF 判定次数与加 gap 的逐份相等，gap 一对都没跳（例：「R2 none:CS2:tf-lenient evaluated=71624 violations=0 images=0 first=-」（`research/prompts/m3-birth-floor-r2-opus-model/logs/sweep-A0.log:940`）对「R2 none:CS2:tfgap-lenient evaluated=71624 violations=0 images=0 first=-」（`research/prompts/m3-birth-floor-r2-opus-model/logs/sweep-A0.log:942`）；八份的数在 G1）。这批里没有「恢复丢掉无提交标记的尾巴」那一形（没造，推的）。
- J1-a 那几段（短环）：1 MiB 环 a=1 一份：「R2 stale:CS2:tf-lenient evaluated=1646468 violations=344 images=172」（`research/prompts/m3-birth-floor-r2-opus-model/logs/jwrap1-a1.log:1412`）对「R2 stale:CS2:tfgap-lenient evaluated=1582396 violations=0 images=0」（`research/prompts/m3-birth-floor-r2-opus-model/logs/jwrap1-a1.log:1414`），gap 跳了 64 072 次（3.9%）；6 MiB 环：「R2 stale:CS2:tf-lenient evaluated=188416 violations=88 images=44」（`research/prompts/m3-birth-floor-r2-opus-model/logs/jwrap6-a2.log:388`）对 `jwrap6-a2.log:390` 的 180 760，跳 7 656 次（4.1%）；默认 768 MiB 环那批：「R2 stale:CS2:tf-lenient evaluated=2739932 violations=0 images=0 first=-」（`research/prompts/m3-birth-floor-r2-opus-model/logs/stale-ow-a2.log:36`）对「R2 stale:CS2:tfgap-lenient evaluated=2734972 violations=0 images=0 first=-」（`research/prompts/m3-birth-floor-r2-opus-model/logs/stale-ow-a2.log:38`），跳 4 960 次（0.18%）。
- 这几个数是「判定次数里被跳的占比」，不是「坏镜像漏红的比例」；坏单元落在被跳的对里才漏，见 N3 的盖小数。

## 替 gap 之外的处置辩护

被替的有五条：丙（I-3.5 不立，等谱系记录）、N1（gap 只在缺的 txg 不全由被抛弃的根占着时触发）、N2（正文说的窄形：只在根槽读不出处触发）、N3（缺口对里仍判用户可见树之外的单元）、N4（谱系记录落地之后用格级形取代 gap）。N1、N3、N4 是我提的，零轮，只在我的两片叶模型上量过；N2 与丙是正文点名要辩的。基准都是判决采的 gap。

### 丙：I-3.5 不立，状态列写明等谱系记录

站得住的理由：
- 判决第 1 行已经按最弱读法给 TF 记了一次输（跑前判据第一条字面打中），gap 是打中之后加的零轮改法，第二轮结束时没有人在真实现上从坏镜像一侧攻过它（第三轮攻方的结果我没读）；丙是唯一不吃第二轮那次打中的处置。
- 谱系记录落地之后 (b) 要重写（第 9 行），而重写成哪一形决定 gap 还留不留（见 N4）：先立 gap、再换形，等于同一行写两遍；丙一次写全。
- I-3.5 今天未实现，没有在跑的判定，丙不引入回归，也不引入新盲区。

站不住 / 够不着的地方：
- 丙放弃的覆盖是真的：第二轮攻方 `bad-r1six.log` 里 28 张坏镜像，池级 checker 0 红而 TF 红的有 15 张（BI-1 四张、BI-1b 四张、BI-3 七张，数法在 G1）；J3-c（链改到 = R⁻.txg）也只红 TF（第 5 行）。丙之下这些全靠别的条，而别的条（I-9.26 之类）已经红过一部分、剩下这 15 张没有。
- 丙与用户 2026-10-02 定的「I-3.5 ① 取 β」冲突：β 把 ① ② 合成一条出现规则，② 的下界半不立，β 就只剩「再次出现」一半；J3-d 那一形只有 β 抓，丙连它一起放弃。要走丙得先让用户改 β。
- 丙回答不了「谱系记录什么时候落地」：它是里程碑三三·3，今天没有日期。窗口里写路径若出诞生代号盖小的错，只有这 15 张一类的镜像才抓得到它。
- 辩方判断：丙是站得住的退路（任何一条之外的处置都得在它与 gap 之间比覆盖与盲区），但覆盖上被 gap 支配，只有在「gap 盲区或实现代价比这 15 张的覆盖更大」被量出来时才该取丙；今天两样都没量到。

### N2：只在根槽读不出处触发

**够不着 J1-a，所以过不了跑前判据第一条。** J1-a 的洞不是读不出：txg 9、10 的槽被 txg 33、34 的根盖掉，位置由「r == txg mod R 且 s == (txg div R) mod S」（`.claude/kb/invariants.md:71`）定，两个槽读得出、自证过，只是装着别的 txg 的根（第二轮攻方报告 `research/prompts/m3-birth-floor-r2-opus-output.md` J1-a 一节第 1 步，写 33、34 盖掉 9、10 的槽）。checker 里「读不出」就是 I/O 读失败：「let some_root_slot_is_unreadable = root_slot_positions(geometry)」（`crates/singlefs-checker/src/walk.rs:5895`）一带判的是 `reader.read(…).is_none()`；读到了但自证不过（撕裂、读坏）的槽另走静默丢弃：「.and_then(|bytes| check_root_slot(&bytes, &geometry.filesystem_identifier).ok())」（`crates/singlefs-checker/src/image.rs:1001`），连「读不出」这个条件都不触发。所以 N2 在 J1-a、在读坏的槽上都不触发，TF 照红合法镜像。要收窄只能按「缺的 txg 有没有解释」收，不能按「槽读不读得出」收。

### N1：缺的 txg 都由被抛弃的根占着时不整对跳过

- 机理：被抛弃的根在根环里读得出，checker 本来就认得它们：「let abandoned_by_the_newest_roots_table = |root: &crate::RootView| {」（`crates/singlefs-checker/src/walk.rs:6708`）。它们占的 txg 是被解释了的洞（新实例的根接在所选根上，树一样）；没人占、也没有记录的 txg（根槽读不出、环盖掉、尾巴记录被丢）才是洞。
- 模型（ROLLKX，含抛弃 X）：只对「之间有根槽读不出的 txg」跳（gap=lost）0 红；只对「之间有被抛弃的根」跳（gap=ab）仍红 25 834 / 36 048，跳过的对 17 424 641 → 12 272 407（−29.6%），被跳对里的新单元 90 440 082 → 69 993 239（−22.6%）（`r3c_ROLLKX8.out`）。ROLL 里没有 X，lost 与 all 一样。
- 够不够用：今天 harness 造不出被抛弃的根（可写挂载在取号之前就拒：「取号之前拒可写」（`crates/singlefs-harness/src/history.rs:394`）），所以今天没有镜像受益；这个字句要等真有被抛弃的根出现才值。辩方判断：站得住、没有代价，但今天不值得写进措辞，留作 K2 的注脚。

### N3：缺口对里仍判用户可见树之外的单元

做法：缺口那一对不整对跳过，只跳「落在 R 的用户可见树（extent、inode、`deleted_inodes`）子树里的单元」；固定点单元（树表、记账、分配记录树、中央映射根）、实例表单元照 (a) 判：birth(b) > R⁻.txg。
- 理由：回退只把 extent、inode 两条树表条目的根指针换成目标的：「root: if entry.kind == TREE_KIND_EXTENT || entry.kind == TREE_KIND_INODE {」（`crates/singlefs-core/src/mount.rs:5162`），别的树随现行（「墓碑树、谱系树随现行那一版，不取 R_old，回退那次发布写一条谱系记录 (T_rb, T_old)」（`.claude/kb/decisions/23-journal的角色与格式.md:377`），实例表照现行）。所以用户可见树之外、R 引用而 R⁻ 不引用的单元，只能是 R⁻ 与 R 之间某次发布写的，诞生代号 > R⁻.txg，洞不改变这一点（推的：读 mount.rs，没有逐个读完建根的路径）。洞只让「继承来的子树」失去来源，不让固定点单元失去来源。
- 模型（两片叶，`r3c_variants.py` 的 fxonly 列）：合法镜像 28 926 196 张 0 红（`r3c_ROLL8.out`、`r3c_ROLLKX8.out`，scp、scp_all、v1p、v1p_all 的 fxonly 都是 0）。
- 坏镜像（`r3d_bad.out`，3 000 段随机历史含 L、X，只在缺口对上盖小，每个新单元一张）：

```text
  类 | 张数 | 不处置抓到 | gap=all 抓到 | gap=fxonly 抓到
  固定点 fx | 2482 | 2482 | 0 | 2482
  用户树新叶 Ila/Ilb/Xl | 3155 | 3155 | 0 | 0
```

  gap 整对跳过时固定点单元盖小全漏（0 / 2 482），N3 全抓；用户可见树里的新叶盖小两样都漏（只有谱系记录能补，同第一轮）。第二轮攻方 BI-1、BI-1b 那一类盖法（盖回退根自己新写的固定点单元；它们在无缺口的对上，两种处置都抓）若落在缺口之后的那一对，gap 漏、N3 不漏。
- 代价：K4 本来就要给每个落点打「落在哪棵用户可见树之下」的标签，N3 多用一次，没有第二趟走读。
- 风险：零轮；v3 里的固定点单元是整个新写，没有记账树、中央映射树「叶部分共享」的结构，N3 在真实现上等于一个读 mount.rs 推出的断言。验证路径便宜：主 agent 在攻方原型（`proto.diff`）的 `tfgap-lenient` 旁加一个只跳用户可见树单元的版本，跑三份 jwrap 日志与层 0 头尾两段，应当在 TF 红的 44 / 172 / 256 张上 0 红、在 BI-1 / BI-1b 缺口版上红。
- 辩方判断：N3 是 gap 的严格收窄（判得更多、没有多漏），模型上 0 红；若主 agent 要在 gap 之外再给用户一个选项，它是最便宜的一个。

### N4：谱系记录落地之后，格级形取代 gap

- 做法：(b) 写成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且 b 的诞生代号 ≤ T_old，且 b 落在 R 的某棵用户可见树的子树里」，谱系树读不全时该对不适用。与判决第 9 行采的区间形比，不要求树根指针出生 txg ≤ T_old，改看 b 自己。
- 为什么能不要 gap：谱系树随现行那一版走（第 9 行引的 D23 那一行），洞里那条回退根丢了，R 的谱系树仍带着记录。
- 模型（两片叶，`r3b_lineage.py`）：深度 8，28 926 196 张，格级形不加 gap：

```text
[r3 谱系写法·合法镜像] 字母表 ROLL 深度 8：镜像 12618652
  lin_tree           红的镜像 83620
  lin_tree_gap       红的镜像 0
  lin_cell           红的镜像 0
  lin_cellin         红的镜像 0
[r3 谱系写法·合法镜像] 字母表 ROLLKX 深度 8：镜像 16307544
  lin_tree           红的镜像 25946
  lin_tree_gap       红的镜像 0
  lin_cell           红的镜像 0
  lin_cellin         红的镜像 0
```

  （去掉各行后面的「首例」列；`_all` 行同样，原样见 `r3b_lin_ROLL8.out`、`r3b_lin_ROLLKX8.out`。lin_tree = 判决第 9 行的区间形；lin_cell = 只看 b 的诞生代号 ≤ T_old；lin_cellin = lin_cell 再要求 b 在用户可见树之下，即上面的做法。）
- 坏镜像（`r3b_lin_bad.out`，1 500 段随机合法历史，只在 R.txg = R⁻.txg + 1 的对上盖）：回退根自己新写的固定点单元盖到 T_old：lin_tree 3 599 / 3 599、lin_cellin 3 599 / 3 599、纯格级 lin_cell 0 / 3 599（纯格级把固定点单元免掉了，所以 lin_cellin 要带「在用户可见树之下」）；非回退根整条链盖小（M2both、M2x、M2i，k = 1、2、3）三种逐格相同。
- 强弱：在洞对上比 gap 判得多（固定点单元与诞生代号 > T_old 的单元都判），比 lin_tree 判得少（用户可见树里诞生代号 ≤ T_old 的单元不看树根指针就免）；整条链改小到 T_old 以下仍判不出，同 lin_tree。依赖谱系树可读、谱系记录可信（同第二轮辩方 C2 的前提：连记录一起伪造没人核）。
- 对判决的影响：第 9 行写成「树根指针出生 txg ≤ T_old」，落地时照写就得永远留着 gap；想让谱系记录落地后撤 gap，要写这一形。零轮，交用户表标「零轮」，三·3 实现时再攻。

## K3：终轮复核前两轮站住的结论

### K3-1　V1 出局；取 TF 不取 V2s / V3s

- V1 出局：站得住，但它是在「没有 gap」的世界里判出局的。跑前判据没写「补了不适用之后」，字面成立；加 gap 之后 V1 系（V3s）两片叶模型上也 0 红（`r3c_ROLL8.out`、`r3c_ROLLKX8.out` 的 v1p 行 gap=all 列），所以 V1 出局不是 V1 内核错，这一点辩方上一轮已写，这一轮的新证据是它在真结构上与 TF 同红同绿。
- 取 TF：见第 6 行。两片叶模型上 V1 系与 TF 的合法镜像面逐数相同；坏镜像面，V1 系多抓 extent 链单改（原样，`r3e_bad.out`，1 500 段随机历史，种子 5）：

```text
  模式 k | 镜像数 | TF(scp_all) | V1系(v1p_all) | TF+gap | V1系+gap
  M2x k=2 | 3620 | 0 | 3620 | 0 | 3620
  M2x k=3 | 3620 | 0 | 3620 | 0 | 3620
  M2i k=2 | 4241 | 745 | 745 | 745 | 745
```

  其余各行（M2both、M2i；k = 1、2、3）两列相同。TF 没有 V1 系抓不到而它抓得到的一类。主 agent 若看重「写路径里只有 extent 树的诞生代号打戳出错」这一类，V3s + gap 是在 TF + gap 之外的备选，真实现里没人跑过（零轮）。

### K3-2　β 的「再次出现」那一半有只红它的坏镜像：在入库的池级 checker 上重做，有没有别的不变量跟着红

我不编译 Rust，这一格在入库装置上重做归主 agent（要跑的命令在 G1）。读码与日志能说的：
- 攻方日志里每一步的池级 checker 红项：第一步只换指针：「BJ J3-d/step1-pointer pool_checker_violated=[("I-3.9"」（`research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r2.log:196`）；第二步改两条分配记录后只剩：「BJ J3-d/step2-allocation pool_checker_violated=[("I-3.11"」（`research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r2.log:210`）；第三步换映射条目后 0 红（第 5 行引的 `bad-r2.log:223`）。红项是一项一项补掉的，不是巧合没红。
- 读 kb 里的不变量表，今天已实现与实现在途的几条（I-9.6 水位大于两处最大号、I-9.25、I-9.26、I-4.9、I-3.12、I-3.13）对 J3-d 都不红：J3-d 把 txg 7 的 inode 树里第二片叶指回旧叶，水位照旧，旧叶容器里每条记录的改动计数都不大于容器诞生代号 4；I-4.9 只判非最新候选根，J3-d 改的是最新根（txg 7）。三·3 的 I-9.5、I-9.18 到 I-9.20 判 `deleted_inodes` 与墓碑，与它无关。这是读表推的，没在入库装置上造。
- 这个坏镜像的语义：txg 7 静默丢掉 txg 6 建的那个 inode，而账、映射一起改平。真实的写路径错误很难恰好把三处一起改对，所以 β 那一半值多少是用户已经定了的事（判决第 7 行）；这里只回答「只红它」成不成立：日志与读表两路都成立，入库装置上的一路待主 agent。

### K3-3　「严格小于」那一处收严有只红它的坏镜像

站得住：J3-c 池级 checker 0 红（`bad-r2.log:61`）而 TF 红（`bad-r2.log:65`）。严格小于对合法镜像没有代价：R 引用而 R⁻ 不引用的树根，出生 txg 等于 R⁻.txg，意味着那次发布新写了一棵 R⁻ 自己不引用的用户可见树，造不出（推的；两片叶与一片叶模型上 `pb < P.txg` 都是严格小于，合法镜像 0 红）。

### K3-4　判决里的数能不能核

| 判决里的数 | 核法 | 结果 |
|---|---|---|
| 攻方 696 / 700 | `crash-head.log:33` | 对上：evaluated=700 violations=696；这是头 700 个状态（0 到 699），层 0 全量 196 665 个没跑完；「TF 红而层 0 全绿」成立靠抬 F 窗口里 I-3.1 不适用：「最新根带的 F 低于 F_生效（两者不等）时这一格不判」（`.claude/kb/invariants.md:143`），不是每个状态都这样（尾 700 个 TF 与 I-3.1 同红，见第二轮攻方报告） |
| 攻方 472 张 | 三份 jwrap 日志的 `images=` | 172（`jwrap1-a1.log:1412`，上面已引）+ 256：「R2 stale:CS2:tf-lenient evaluated=2204684 violations=512 images=256」（`research/prompts/m3-birth-floor-r2-opus-model/logs/jwrap1-a2.log:2086`）+ 44（`jwrap6-a2.log:388`，上面已引）= 472；对上 |
| 攻方 2 712 段 | 八份 sweep 日志的 `R2_JOBS` 相加 | 2 712，对上（G1 贴原样输出）；2 184 + 528 也对 |
| 辩方 2 892 万张 | 重跑 `run_r2_deep.py ROLL 8 7`、`ROLLKX 8 7`（内存包装外的纯 Python，7 线程，各约 105 秒） | 12 618 652 + 16 307 544 = 28 926 196；v1p 41 810 / 12 973、v1p_all 86 940 / 18 096、scp 与 skip 各列 0，与辩方上一轮原样相同；「2 892 万」是 2 892.6 万的截断，写 2 893 万也行。这个数检验的是 V1 系加不适用，对 TF + gap 没有检验力（scp 本来就 0） |

## K4：措辞写全，对着冻结副本 checker 写要改几处与代价

### 整段（可直接进 `.claude/kb/invariants.md` I-3.5 那一行的内容列，替换 ① ② 两句；③ ④ 与 ⚠️ 射程注（2026-09-13）照旧）

被替换的是现在这一句：「① `Reach(b)` 在 txg 序上连续无洞」（`.claude/kb/invariants.md:146`）与它后面的 ②。下面是 TF + β + gap + 第二轮判决第 8 行三处措辞，加上 K4 补的两处（标「补」）：

> **可判定形式**（与 I-3.1 共用同一趟全盘遍历）：取回退候选集——`check_pool_image` 判 I-3.1、I-7.4 用的那一份（根环里自证过、按最新根指着的实例表仍有效、txg ≥ F_生效 的根，加由记录施加出来、根槽从没落盘的那几版）——按 txg 升序，同 txg 的几条并成一条（R⁻ 侧引用集合取并，R 侧逐条各判一次）。相邻两条 (R⁻, R) 为一对，逐对判：
> ① **不适用**：R.txg ≠ R⁻.txg + 1 时整对报不适用（候选集在这一对之间有洞：根槽读不出、根槽被环盖掉而记录也被盖掉、那个 txg 从没发布过）；R 或 R⁻ 的引用集合数不全（走读读不出，或走到没有条款的形状）时同样整对报不适用；所有对都不适用时整条报不适用并带理由（补，同 I-4.9 候选集只剩一条根的写法）。
> ② **出现规则**：R 引用而 R⁻ 不引用的每个落点 b（第一次出现与再次出现都判，Reach 在 txg 序上是否连续不再单判）满足 (a) birth(b) > R⁻.txg；或 (b) b 落在 R 树表里某棵用户可见树（今天 extent、inode，`deleted_inodes` 非全零之后加它）的子树里，那棵树的根指针非全零、出生 txg 严格小于 R⁻.txg，且那棵子树里每个读得出的单元头的诞生代号不大于那个出生 txg（读不出头的单元不计入）。birth(b) 取 b 的单元头里的诞生代号；b 自己的头读不出时这一个落点不判，读不出归 I-1.1 与 I-2.1（补）。
> ⚠️ **射程**：整条用户可见链（树根指针出生 txg 与树里单元的诞生代号）一起改小到 R⁻.txg 以下的坏镜像判不出：盘上与一次合法回退逐项同形，判别子是回退那次发布写的谱系记录（D18 已定项 24），`crates/` 今天没有；缺口右边那一对里第一次出现的单元不判（推的盲区，用户选 N3 时缩成「只有用户可见树里的单元不判」）；② 的上界半（`birth(b) ≤ R_first.txg`）归 I-4.9。谱系记录落地之后 (b) 收成「R 的谱系树里有一条记录 (T_rb, T_old)，R⁻.txg < T_rb ≤ R.txg，且那棵用户可见树的根指针出生 txg ≤ T_old；谱系树读不全时该对不适用」，这一形要与 ① 的缺口条款并存（补；上文第 9 行、N4）。
> **判别力**：改小到恰好等于 R⁻.txg 的新写单元（J3-c）必须红；最新根的 inode 树指回已丢的旧叶、分配记录与映射条目一起改平（J3-d）必须红；回退根槽被环盖掉、记录也盖掉之后的合法镜像（J1-a）不许红。**推翻条件**：任一合法镜像（池级 checker 与层 0 其余判定都不红）上红。

状态列（主 agent 按第三轮判决写，这里只列该带的零轮项）：gap、第 8 行三处措辞、上面标「补」的几处、谱系区间形，都是零轮或被攻过一两轮；整条链改小判不出、缺口右边第一次出现不判是已知盲区。

### 要改哪几处（冻结副本 `crates/singlefs-checker/src/`）

第一轮判决第三节写的两处（`RootReferences` 按树分开落点、去掉只在最新根有已释放记录时才走到叶的条件）读码之后都不必改：`RootReferences` 是 I-3.9 与 I-9.14 的另一趟走读：「struct RootReferences {」（`crates/singlefs-checker/src/walk.rs:2931`），主走读每版已经维护好每版自己的落点集合，且共享子树靠合并补全（五处命中 `visited_units` 的地方都调用合并函数：「self.merge_placements_below_a_unit_walked_by_an_earlier_version(placement);」（`crates/singlefs-checker/src/walk.rs:956`），另四处在 1762、1868、2016、2183）。改法挂在主走读上：

| # | 位置 | 改什么 | 代价（推的，没量） |
|---|---|---|---|
| 1 | `Walk` 结构体：「struct Walk<'reader> {」（`crates/singlefs-checker/src/walk.rs:287`） | 加一张「每版结束时的快照」：(txg, 实例) → {落点集合、每个落点的用户可见树标签、走读失败数是否增加} | 每条候选根一份落点集合，默认环最多 24 条根 |
| 2 | 每版开头清空那一份集合：「fn begin_walking_a_version(&mut self) {」（`crates/singlefs-checker/src/walk.rs:702`） | 在 `walk_root` 与 `walk_version_applied_only_by_records` 的末尾（「fn walk_root(&mut self, record: &[u8], is_newest: bool) {」（`crates/singlefs-checker/src/walk.rs:991`）、「fn walk_version_applied_only_by_records(&mut self, version: &VersionAppliedOnlyByRecords) {」（`crates/singlefs-checker/src/walk.rs:1033`））把「placements_referenced_in_this_version」搬一份出来；两处都要，由记录施加的版本也是候选集的一员 | 一次集合拷贝 |
| 3 | 树标签：「fn walk_tree_table_entry(&mut self, entry: &[u8]) {」（`crates/singlefs-checker/src/walk.rs:1438`），extent 与 inode 两支：「if kind == TREE_KIND_EXTENT {」（`crates/singlefs-checker/src/walk.rs:1539`）、「if kind == TREE_KIND_INODE {」（`crates/singlefs-checker/src/walk.rs:1574`） | 走进用户可见树之前设「当前树 = (种类, 根指针出生 txg)」，往下记落点时带标签；`deleted_inodes` 有根之后加种类 9 | 每个落点多一个小标签 |
| 4 | 压得住：合并函数「fn merge_placements_below_a_unit_walked_by_an_earlier_version(」（`crates/singlefs-checker/src/walk.rs:739`）里已经按单元读诞生代号表 | 在那里与首次走读处各取「这棵树之下诞生代号的最大值」，与树根出生 txg 比；读不出头的单元表里没有，天然不计入 | 不多一趟读盘 |
| 5 | 新判定：放在所有候选根与由记录施加的版本都走完之后：「for version in &versions_applied_only_by_records {」（`crates/singlefs-checker/src/walk.rs:6812`）循环之后、「judge_blocks_referenced_by_abandoned_roots(」（`crates/singlefs-checker/src/walk.rs:6839`）之前 | 新函数 `judge_birth_floor_of_adjacent_versions`：按 (txg, 实例) 排序、同 txg 并成一条、逐对判；「数不全」用每版走读失败数的差，写法同 6783 与 6814 两处的 `failures_before` | 相邻差集 + 逐落点查表，每版 O(落点数) |
| 6 | 诞生代号表按第 0 条位置为键：「birth_txg_of_walked_units: BTreeMap<(u32, u64), u64>,」（`crates/singlefs-checker/src/walk.rs:341`），落点集合按每份位置条目记 | 差集前先把落点折成单元键（第 0 条位置），否则第二份副本的落点查不到诞生代号；实例表链页不进这张表（`walk.rs:542`），它们的诞生代号取指针里的 birth_txg | 小 |
| 7 | 注册：「pub const IMPLEMENTED_INVARIANTS: [&str; 65] = [」（`crates/singlefs-checker/src/image.rs:72`） | 加 "I-3.5"；kb 第 12 行写「判 49 条」、它括号里那条数法的命令（G1 贴了原样输出）数出 49，数组却是 65 项，两处对不上是今天就有的（不是这一题造成的），写回时顺手对 | 无 |
| 8 | 用例与变异 | `crates/singlefs-harness/tests/checker_known_bad_images.rs` 加：J3-c、J3-d、BI-1 / BI-1b 一类各一张只红 I-3.5 的坏镜像；J1-a 一张合法镜像不红；`crates/mutations.tsv` 加：去掉缺口条款、`<` 改 `≤`、去掉「再次出现」、去掉「压得住」 | 层 0 每个崩溃状态都跑它，单状态耗时没量 |

代价的量法交主 agent：在层 0 一个流上数「候选根不止一条」的状态占几成，再比有无新判定的单状态耗时（checker 档，我不跑）。内存是候选根数 × 每版落点数，默认环 24 条；两条以上才有对可判。

## K5：短环终态镜像上池级 checker 红 I-3.1 与 I-3.5 的关系

一句话：同一个洞的两个后果。gap 管不到 I-3.1；I-3.5 落地不与它冲突，也不被它掩盖；但 I-3.1 以后怎么修，决定 gap 的盲区今天看不看得见（第 4 点）。

1. **同一批历史，默认环 0 红、短环红，红的恰是 I-3.1。** 默认 768 MiB 环那批：「R2 stale:pool-checker evaluated=3240 violations=0 images=0 first=-」（`research/prompts/m3-birth-floor-r2-opus-model/logs/stale-ow-a2.log:39`）；同形历史换 1 MiB 环：「R2 stale:pool-checker evaluated=1944 violations=1560 images=1560」（`research/prompts/m3-birth-floor-r2-opus-model/logs/jwrap1-a2.log:2089`）；6 MiB 环：「R2 stale:pool-checker evaluated=54 violations=44 images=44」（`research/prompts/m3-birth-floor-r2-opus-model/logs/jwrap6-a2.log:391`），三行红项都是 `["I-3.1"]`。
2. **机理（读码）**：checker 已经为「洞里那一版的记录还在」补候选集：「/// ④ **它换下的单元还没到回收的时候**：根环里有一条没被实例表判抛弃的根，txg 比它小。」（`crates/singlefs-checker/src/walk.rs:5800`），判法是「let its_units_are_not_reclaimable_yet = oldest_valid_root_txg < record.checkpoint_txg;」（`crates/singlefs-checker/src/walk.rs:5844`）：环里最旧的有效根比这一版老，这一版换下的单元回收不了、仍算已分配，所以要把这一版并进遍历。短环把洞里那几版的记录也盖掉，补不回，这些单元已分配、没有任何走过的版本引用，I-3.1 少算。I-3.5 的洞与它是同一个：候选集在 (R⁻.txg, R.txg) 之间缺 txg 且补不回。
3. **不冲突、不掩盖，两个方向都核了**：I-3.5 只比相邻两版的落点集合，不读也不改 I-3.1 的并集，两条没有共享状态。终态镜像上 I-3.1 红，池级判定本来就红，I-3.5 在洞上不适用不改变总判；层 0 里抬 F 窗口的 696 个状态 I-3.1 因 F 不等而不适用（「最新根带的 F 低于 F_生效（两者不等）时这一格不判」（`.claude/kb/invariants.md:143`）），池级其余全绿，TF 单独红，这时挡住它的只有缺口条款。
4. **I-3.1 红是洞上的「碰巧兜底」，修 I-3.1 会把 gap 的盲区放出来。** 攻方量到的带洞镜像，TF 或 β 红的每一张池级 checker 都先红 I-3.1（J1-a 终态 472 张；根槽读坏那批八份 sweep 日志里 3 368 条 `R2_RED` 行，池级 0 红的一条都没有，命令与输出在 G1），唯一的例外是抬 F 窗口那 696 个状态。也就是说今天任何带未补回的洞的镜像整张就是红的，缺口右边藏的坏单元不会让一张本来绿的镜像漏成绿。下一批调查员若把 I-3.1 在洞上改成不适用（与 I-3.9 的「环上有洞时……按区间判」（`.claude/kb/invariants.md:151`）同一个思路），带洞镜像整张转绿，gap 的盲区就成了真盲区。所以：① I-3.1 与 I-3.5 的洞谓词要共用一个函数（K4 第 5 行的位置），不各写各的；② K1 问的「池级 checker 别的条有没有兜住」，答案要在 I-3.1 修了之后重答；③ 这一条是推的，没造「洞右边藏一个坏单元」的镜像去看池级 checker 红不红。
5. 不去修 I-3.1，按分工那一件下一批派调查员。

## G　复跑、文件、推翻条件、没做什么

### G1　复跑

都在仓根跑，纯 Python，线程上限 7：`bash research/scripts/capped.sh 7 nice -n 19 python3 research/prompts/m3-birth-floor-r3-sonnet-model/<脚本> <参数>`；各约 100 到 150 秒，计数逐次相同，「首例」列随进程到达序会变，比数不比 sha。

| 脚本与参数 | 输出 | 说明 |
|---|---|---|
| `run_r2_deep.py ROLL 8 7`、`run_r2_deep.py ROLLKX 8 7` | `deep_ROLL8.out`、`deep_ROLLKX8.out` | 辩方上一轮一片叶模型的重跑，数与上一轮逐个相同（K3-4） |
| `run_r3c_deep.py ROLL 8 7`、`run_r3c_deep.py ROLLKX 8 7` | `r3c_ROLL8.out`、`r3c_ROLLKX8.out` | 两片叶模型 × 各缺口处置（none / all / lost / ab / fxonly）；`r3b_ROLLKX8.out` 是同一批的前一版输出（少 fxonly 列） |
| `run_r3b_lineage.py ROLL 8 7`、`run_r3b_lineage.py ROLLKX 8 7`、`run_r3b_lineage.py bad 1500 14 5` | `r3b_lin_ROLL8.out`、`r3b_lin_ROLLKX8.out`、`r3b_lin_bad.out` | 谱系写法 lin_tree / lin_cell / lin_cellin × 是否加 gap；`r3_lin_ROLL8.out` 是前一版（少 lin_cellin） |
| `run_r3d_bad.py 3000 16 7` | `r3d_bad.out` | 缺口对上盖小：gap=all 对 gap=fxonly |
| `run_r3e_bad.py 1500 14 5` | `r3e_bad.out` | 两片叶模型上 TF 与 V1 系的整条链盖小 |
| `count_only_tf_red_bad_images.py <bad-r1six.log>` | 下面原样 | 15 张的数法 |

```text
$ python3 research/prompts/m3-birth-floor-r3-sonnet-model/count_only_tf_red_bad_images.py research/prompts/m3-birth-floor-r2-opus-model/logs/bad-r1six.log
坏镜像 28 张；池级 checker 红的 13 张；池级 0 红而 TF 红的 15 张
{'BI-1': 4, 'BI-1b': 4, 'BI-3': 7}
$ grep -h '^R2_JOBS' research/prompts/m3-birth-floor-r2-opus-model/logs/sweep-*.log | awk '{s+=$2} END{print s}'
2712
$ grep -c '^| I-.*已实现' /tmp/claude-1000/m3-batch5-freeze/tree/.claude/kb/invariants.md
49
$ cd research/prompts/m3-birth-floor-r2-opus-model/logs && grep -h '^R2_RED ' sweep-*.log | grep -c 'pool_red=\['
3368
$ cd research/prompts/m3-birth-floor-r2-opus-model/logs && grep -c 'pool_red=\[\]' sweep-*.log   # 每份日志里池级 checker 0 红的 R2_RED 行数
sweep-B1.log:0
sweep-A1.log:0
sweep-B0.log:0
sweep-C0.log:0
sweep-raise-A.log:0
sweep-A0.log:0
sweep-C2.log:0
sweep-C1.log:0
```

两片叶模型深度 8 的汇总（`r3c_ROLL8.out`、`r3c_ROLLKX8.out` 里的 `[r3` 起头行、「相邻对」行与 scp、scp_all、v1p、v1p_all 各行，行本身原样；「首例」行与 plain 各行没摘，原样见 `.out`）：

```text
[r3 深枚举] 字母表 ROLL（16 个操作）深度 8：前缀 28 个；镜像 12618652
  相邻对 72739224，gap=all 跳过的对 11834302（含缺口的镜像 9080972），新单元 342431969，其中在被跳过的对里 63866277；gap=lost 跳过的对 11834302，其中新单元 63866277
  scp|none       红的镜像 83620
  scp|all        红的镜像 0
  scp|lost       红的镜像 0
  scp|ab         红的镜像 83620
  scp|fxonly     红的镜像 0
  scp_all|none   红的镜像 173848
  scp_all|all    红的镜像 0
  scp_all|lost   红的镜像 0
  scp_all|ab     红的镜像 173848
  scp_all|fxonly 红的镜像 0
  v1p|none       红的镜像 83620
  v1p|all        红的镜像 0
  v1p|lost       红的镜像 0
  v1p|ab         红的镜像 83620
  v1p|fxonly     红的镜像 0
  v1p_all|none   红的镜像 173848
  v1p_all|all    红的镜像 0
  v1p_all|lost   红的镜像 0
  v1p_all|ab     红的镜像 173848
  v1p_all|fxonly 红的镜像 0
[r3 深枚举] 字母表 ROLLKX（14 个操作）深度 8：前缀 40 个；镜像 16307544
  相邻对 111869417，gap=all 跳过的对 17424641（含缺口的镜像 12475366），新单元 384595318，其中在被跳过的对里 90440082；gap=lost 跳过的对 12272407，其中新单元 69993239
  scp|none       红的镜像 25946
  scp|all        红的镜像 0
  scp|lost       红的镜像 0
  scp|ab         红的镜像 25834
  scp|fxonly     红的镜像 0
  scp_all|none   红的镜像 36184
  scp_all|all    红的镜像 0
  scp_all|lost   红的镜像 0
  scp_all|ab     红的镜像 36048
  scp_all|fxonly 红的镜像 0
  v1p|none       红的镜像 25946
  v1p|all        红的镜像 0
  v1p|lost       红的镜像 0
  v1p|ab         红的镜像 25834
  v1p|fxonly     红的镜像 0
  v1p_all|none   红的镜像 36184
  v1p_all|all    红的镜像 0
  v1p_all|lost   红的镜像 0
  v1p_all|ab     红的镜像 36048
  v1p_all|fxonly 红的镜像 0
```

首例（`r3c_ROLL8.out` 里 `scp|none` 那一行）：`W, L1, W, W, R(o,0), N, F4, L3`——回退根的根槽读不出之后又建 inode，旧叶在新 inode 树根之下重现。

### G2　什么现象会推翻这些结论

- 第 1 行「同样打中别的臂」：找到一种写法（TF、β、V1 系、谱系区间形之外）在 J1-a 上不红、又不靠看 txg 缺口。
- 第 2 行与 N3：入库装置（或攻方原型加一个只跳用户可见树单元的版本）上，三份 jwrap 日志里 TF 红的 44 / 172 / 256 张，N3 红了任何一张；或造出一张「缺口之后固定点 / 实例表单元诞生代号 ≤ R⁻.txg 而又合法」的镜像（例如恢复丢尾巴记录之后的哪一类建根路径把老的固定点单元带进新根）。
- 第 6 行与 K3-1：真实现上 V3s + gap 红了一张 TF + gap 不红的合法镜像，或 TF 抓到一张 V3s 抓不到的坏镜像。
- 第 9 行与 N4：两片叶以上、含多次回退叠加的合法镜像上 lin_cellin 红；或谱系记录落地后写出的 T_old 不是 R_old 的 txg。
- K3-2：J3-d 在入库池级 checker 上有任何一条红。
- K5 第 4 点：造出一张「洞右边藏坏单元而池级 checker 其余全绿」的镜像（说明 I-3.1 的兜底不成立）；或造出 I-3.1 红而没有洞的合法镜像（说明两条不是同一个洞）。

模型目录文件（sha256 前 12 位；`.out` 在复跑时「首例」列会变，比数不比 sha）：

```text
2bae0b6eb443…  birth_floor_v2.py
f7559bcf88a0…  birth_floor_v3.py
95f38fcff71a…  count_only_tf_red_bad_images.py
f5455a54ab88…  r2_variants.py
62430a6e7b00…  r3b_lineage.py
bc43462c5ca0…  r3b_variants.py
4dc6f7185b40…  r3c_variants.py
871a9f1cc651…  r3_lineage.py
342057ddde26…  r3_variants.py
1a453c37d4a1…  run_r2_deep.py
df2ba90d329c…  run_r3b_deep.py
428c7dafa778…  run_r3b_lineage.py
58eae2ecc9b5…  run_r3c_deep.py
83e55229fa2a…  run_r3d_bad.py
e6cd5df13c4a…  run_r3_deep.py
a2de89d4263b…  run_r3e_bad.py
3abce5be651e…  run_r3_lineage.py
9b3bb74fb3fe…  deep_ROLL8.out
928bc3fd02e3…  deep_ROLLKX8.out
574f26fcbd5b…  r3b_lin_bad.out
cef70dfed8c7…  r3b_lin_ROLL8.out
f143df8ff868…  r3b_lin_ROLLKX8.out
fce5f03f0df8…  r3b_ROLLKX8.out
a78167ef1a18…  r3c_ROLL8.out
279ecd237835…  r3c_ROLLKX8.out
499b56534c8a…  r3d_bad.out
bbddf8f77dfb…  r3e_bad.out
186c0a293d4d…  r3_lin_ROLL8.out
67ff5f061b8e…  r3_ROLL8.out
```

### G3　没做什么

- 没编译 Rust，没跑 checker 档任何东西；所有「红 / 0 红」是两片叶抽象模型或攻方日志上的数，抽象模型零轮、不进 kb。机理对代码是读码核的，没在入库镜像上造任何一张。
- 没在入库池级 checker 上重做 J3-d（K3-2）：要写成 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 里的一条用例再在入库 checker 上跑，是实现员与主 agent 的活；`bash research/prompts/m3-birth-floor-r2-opus-model/rerun.sh /tmp/claude-1000/m3-batch5-freeze/tree <新草稿目录> bad-r2` 复现的是攻方副本上的数，不算入库。
- 没验 N3 在真实现上（要在攻方原型的 `tfgap-lenient` 旁加一个只跳用户可见树单元的版本，约 35 分钟的 `rerun.sh`，归主 agent 派）；没造「洞右边藏坏单元而池级其余全绿」的镜像（K5 第 4 点）；没造「恢复丢尾巴记录」的合法历史（第 2 行补充）。
- 没判 K1、K2（攻方的格）；这一轮开头那些量（gap 在 J1-a 段上跳 3.9% 的判定次数，不带故障 2 712 张上跳 0）是读攻方日志得来的，不是我造的历史，也不是漏红比例。
- 没读禁读清单里的任何文件；没替主 agent 撤销或维持判决，交的是复核结果与几条零轮的备选（N1、N3、N4）。
- 一条前台命令（`run_r3_deep.py ROLL 8 7`，约 122 秒）超过默认 120 秒被系统转成后台、自己跑完、输出落了文件；之后的长命令都带 `timeout` 参数前台跑。没起过 `nohup`、没停过任何进程。
- 草稿目录 `/tmp/claude-1000/m3-birth-floor-r3-sonnet/` 里只有模型副本与各次输出（约 208K），没有编译目录、工作树或仓副本，没有要删的。

### G4　引文核对

kb 条款与代码行号都是冻结副本里的；`research/prompts/` 下的日志与判决在主仓。末行：

```text
$ python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r3-sonnet-output.md --root /tmp/claude-1000/m3-batch5-freeze/tree --root /home/fy5090/code/singlefs --background research/prompts/_m3-birth-floor-r3-background.md
  ✓ 核了 62 处引文，对上 62 处，没判 0 处（逐处列在上面）
$ python3 research/scripts/cite-check.py research/prompts/m3-birth-floor-r3-sonnet-output.md
  ✓ 核了 62 处引文，对上 62 处，没判 0 处（逐处列在上面）
```
