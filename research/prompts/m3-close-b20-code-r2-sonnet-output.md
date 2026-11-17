# m3-close-b20-code-r2 辩方腿报告（2026-11-09）

立场：替 r1 判决（`research/prompts/m3-close-b20-code-r1-main-verification.md`）辩护。复核它的三处打中够不够得着、改法修没修到打中的格，判「成立」的几格站不站得住，「重读粒度改条款不改代码」是不是同样打中别的读法；另对正文八格逐格回「成立 / 不成立 / 推不出」。方案按 `crates/` 今天的实现谈。没读禁读清单里的攻方报告。

读过：r1 判决、正文、小节清单；`.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 8 射程、`.claude/kb/decisions/23-journal的角色与格式.md` 第 397 行、`.claude/kb/checks-owed.md` 的 C707、C971、C973 三行、`research/prompts/e205-preregistration.md` 第 45–47 行；`crates/singlefs-core/src/` 的 `recovery.rs`、`mount.rs`、`instance_table_sweep.rs`、`root_ring.rs`、`make_filesystem.rs` 的相关段；验证包新用例、透明判官乙那一判、`crates/mutations.tsv` 第 7064–7069 行；实现员报告 `~/.local/share/singlefs/work/m3-close-b20-code-r1-fixes/result.md`。

## 一、模型与复跑

模型是我写的一份纯 python，不读仓、不调产品代码；它从落点公式与 mkfs 种根推 C973 第 0 代根判据，不抄判据的写法，参照另写（`reference()`）。**我没有跑它**（只做了 `python3 -m py_compile` 与 `bash -n`），下面所有模型读数都是「推的，待主 agent 跑」。模型与 r1 攻方模型共用的前提只有一个：落点公式与「mkfs 把第 0 代根种进三个区域的槽 0」，两样都在下面第五节现引。

| 文件 | sha256 |
|---|---|
| `~/.local/share/singlefs/work/m3-close-b20-code-r2-defense/model/genesis_criterion_model.py` | 9b86ecf7d103f1ff8a03df43032eeb4fc40a786c88d37ff8a3d7244d78046eca |
| `~/.local/share/singlefs/work/m3-close-b20-code-r2-defense/model/tools/rerun.sh` | b4dbf72305ba000f7c2c50a760d058e599a756555aa60a29e2972e654ada8c6d |

复跑命令在第七节「要主 agent 跑的命令」。判据：`violations=0`，两个弄坏开关（`never`、`take-short`）各自 `violations` 非 0，等价变异（`exclusive`）仍为 0。

## 二、复核表

| r1 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 乙入判不出认的两种坏槽、重读读的是系统配置槽、「没有一块盘两槽都自证过」与 R2 同一个量、拒在取号之前、取号与 F2 逐盘核只用 R2（r1 第二节第一行，判成立） | 站得住 | 新字段全仓只有一处读它（`crates/singlefs-core/src/mount.rs:9247`）；R2 与取号读见证值、F2 退回都只调 R2：「if system_configuration_witness_is_undecidable(&slots_of_every_device) {」（`crates/singlefs-core/src/transaction.rs:964`）、「if system_configuration_witness_is_undecidable(&slots_of_each_device) {」（`crates/singlefs-core/src/recovery.rs:8150`）；逐盘核解构时用 `..` 丢掉新字段：「} = system_configuration_slots_of_one_device_read_once(」（`crates/singlefs-core/src/mount.rs:6789`）。详见第三节 |
| 重读粒度（r1 打中：实现整个读阶段重读，D22 写「重读那一槽一次」） | 够得着；「改条款不改代码」站得住，但判决里「用户定的原句没说只重读那一槽」只对第 46 行成立，要补限定 | 第 45 行说了「那一槽」，第 46 行的「仍是」偏槽级；整个读阶段重读来自 N-配置那条用户定案（D23 第 397 行）与同文件 F1 乙的口径；E205 的取样域分不出两种读法。详见第四节 |
| 别池的槽（r1 打中：登记要求把 fsid 不符的槽算读得出而自证不过，实现略过） | 够得着；改法修到了打中的格，且没动 R2 | 字面满足的是第 47 行「两种读法共用」那一分句与 C707 已定条款，不是第 46 行；新用例钉住打中的那一形，五条变异行咬它（`crates/mutations.tsv` 第 7064–7068 行：核心层两条加计数一条，透明判官两条）。详见第三节 |
| 拒因成员注释 `mount.rs:399–401`、`rereads.read_stage` 记法（轻） | 站得住 | 注释现在写全判不出的两形与整个读阶段重读，见第三节第 3 条 |
| C973 调读者同一个落点核、错位读重读一次仍不过不删、第 0 代根判据走同一条重读路（判成立） | 站得住 | 第五节：落点公式推出判据只在窗口里起作用，窗口里判据都触发；模型待跑 |
| C973 条款写法改成上界式（判「打中」，改条款） | 站得住 | 「第 0 代之后至多一圈 3S 个 txg」恰好含每个槽的那一个，第五节第 2 条 |
| C973 丢了写进槽 0 的根槽写，只影响活性（判射程） | 站得住（单故障）；叠上错位读是另一回事 | 单次丢写白停 3S 次发布，与「最多停一圈」同数；丢写加错位读两处故障叠在一起有一格判据放过，见第五节第 5 条，零轮 |
| C973 判据「没打中」只有一次抽样（r1 第四节） | 第二份样本：单故障下没打中；叠加故障下找到一格 | 第五节，两条路径（手推 + 模型待跑），模型路径不共用攻方的手抄代码 |
| C973 上界 `..=` 与 `..` 等价 | 站得住 | 第五节第 4 条，模型 `exclusive` 变异待跑 |
| 检查单八格（判成立） | 成立 7 格，类型与名字 1 格带一处轻注 | 第六节 |

## 三、三处打中够不够得着、改法修没修到

### 1. 别池的槽

- 字面满足的分句：登记第 47 行「两种读法共用」那一句，不是第 46 行的「这两种」。第 47 行写「同口径（整槽校验和过、fsid 与本池相同）；布局号不认、每区槽数越界的槽照实现不算坏（不进乙）」（`research/prompts/e205-preregistration.md:47`）。括号里「照实现不算坏」只挂在布局号与每区槽数上，不挂 fsid；fsid 不符落哪边由 C707 定：「条款已定：fsid 不是本池的槽按自证不过处置」（`.claude/kb/checks-owed.md:424`）。「读得出而自证不过」的那一种，第 45 行已定义为乙直拒的触发之一，第 46 行的「这两种」指它。所以 r1 的打中归对了分句。
- 同一个形在别的臂上：R2 本身——C707 那一行写实现把这种槽「落在判得出那一边」（`.claude/kb/checks-owed.md:424`），条款认可，R2 不改；取号与 F2 退回——只调 R2，见第二节第一行；逐盘核——用 `..` 不读新字段（`crates/singlefs-core/src/mount.rs:6789`）；只有透明判官（不调实现的第二份乙）同形，实现员已改：「slots.slots_of_another_pool += 1;」（`crates/singlefs-verification/src/mount_transparency.rs:2378`）。
- 改法修没修到打中的格：打中的格是「没有一块盘两槽都自证过、没有撕裂 / 全零 / 读错槽、有别池槽」。新用例两块盘较旧那一槽都换成别池的，钉拒：「fn hand_made_image_with_the_older_slot_of_every_device_from_another_pool_is_refused() {」（`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1093`）；阳性对照只换盘 1，照常做成：「fn hand_made_image_with_the_older_slot_of_only_device_one_from_another_pool_mounts_writable() {」（同文件 `:1125`）。变异行咬核心层两处与透明判官两处：「C971 乙入判不出（别的池的槽）：读阶段见证读不认整槽校验和过而 fsid 不是本池的槽」（`crates/mutations.tsv:7064`）、「C971 乙入判不出（别的池的槽）：别的池那一判丢了 R2 的另一半」（`crates/mutations.tsv:7065`）。证红 10241 的日志我读了（主 agent 给了号）：点名 3 条、抓到 3 条，是第 7064、7065 行与第 7069 行重读粒度那一行；其余三行（第 7066–7068 行）等 `--item` 复判 10312，此刻 `task-server/task status 10312` 读到第 2 次尝试 running，没有结局，不引。
- 替代做法是不是同样打中：别池槽折进 R2 的读错计数，违反 C707 已定的「判得出那一边」，还会让取号与 F2 退回也拒（拒因成员 `AcquisitionReadsAnUndecidableWitness` 之类）；让读的一方把别池槽报成读坏，要同时破「读的一方不报读坏（不进读坏账）」（`crates/singlefs-core/src/recovery.rs:5127`）所说的，并改变续做预演剔槽那一本账。r1 取的单独计数只喂乙那一判，是满足第 47 行与 C707 又不动其余几臂的唯一一种；打中对这三种做法一视同仁，没有只打中实现的一种。
- 还得说的一格：登记第 47 行把布局号不认的槽挡在乙外。别池的槽若同时布局位不认，核心层 `parse_slot` 拒成 `IncompatBitsNotRecognized`，不进新计数（`crates/singlefs-core/src/recovery.rs:5179`）；判官走池级 checker 的解析，布局位不认识的进 `Err`、不计。几何字段不一致的槽（判官的 `journal_ring_geometry_field_that_disagrees` 那一拒）两边是否同拒我没逐字段核，推不出。实现员报告说「盘 1 一槽布局位不认识」那一形没造用例、交主 agent 定，我这一轮也没造。

### 2. 重读粒度

放到第四节：实现整个读阶段重读，打中的是 D22 那一句，不是实现。

### 3. 拒因成员注释与记法

- 注释：`StillUnreadableAfterOneReread::PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion` 现在写判不出两形、整个读阶段重读、系统配置槽直接从盘上重读、按重读那一遍的读数判（`crates/singlefs-core/src/mount.rs:399-405`）；与代码对得上：重读那一遍用的还是同一个 `read_stage`，系统配置槽读的是原始 `devices`：「let witness = newer_publish_witness(devices, system_configuration, &effective_root, &records);」（`crates/singlefs-core/src/mount.rs:9280`）。
- 记法：实现员核过，第一遍判真、重读那一遍判假放行时交回 `OnTheOneReread { first_read, reread }`；已有用例 `system_configuration_slots_failing_self_verification_only_on_the_first_read_let_the_mount_through_on_the_reread` 钉着（`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1009`）。站得住。
- 轻：r1 与 `.claude/kb/decisions-history.md` 第 12185 行引的实现位置是 `mount.rs` 第 9326–9340 行；今天重读那几句在第 9341–9353 行（「let reread = read_stage(&cache, devices, system_configuration)?;」`crates/singlefs-core/src/mount.rs:9351`）。行号漂了，不影响判决，kb 里引行号的地方下次顺手核。

## 四、重读粒度：「改条款不改代码」是不是同样打中别的读法

### 1. 各来源对粒度说了什么

| 来源 | 说了什么 | 偏向 |
|---|---|---|
| 用户弹窗原话 | 「乙族：重读仍坏就拒可写」（`records/2026-10-18-里程碑三收尾与近期欠账.md:2583`） | 不指粒度 |
| 登记乙直拒（用户没选这一行） | 「有一槽读回全零或读得出而自证不过就重读那一槽一次，仍是这两种之一就拒可写，不再看 R2 的两半」（`research/prompts/e205-preregistration.md:45`） | 槽级 |
| 登记乙入判不出（用户选的） | 「乙入判不出：重读之后仍是这两种之一的槽，并进 R2 的」（`research/prompts/e205-preregistration.md:46`） | 没写重读谁；「仍是」字面偏槽级 |
| N-配置（用户 2026-09-27 定，早于乙入判不出） | 「为真就在这次挂载内有效的读缓存」（`.claude/kb/decisions/23-journal的角色与格式.md:397`）上把读阶段重做一遍，「重做那一遍为假就用它往下走，仍为真就在取号之前拒可写挂载」（同一行）；乙入判不出并进的是 R2 判不出这一入口 | 整个读阶段 |
| 同一文件里的先例 | 「与读阶段那一次重读（整个读阶段重做，环在内）同一个口径」（`crates/singlefs-core/src/mount.rs:9390`） | 整个读阶段 |
| E205 的取样域 | 「重读一次读回同样的字节（读故障在一次挂载里不变，S5 核），所以按一遍读数判」（`crates/singlefs-verification/src/bin/e205_c971_inverted_first_txg_arm_costs.rs:884`） | 分不出两种读法 |

注：D23 第 397 行里「含重读仍坏的槽并进读错那一半（乙入判不出）」是乙入判不出定案之后写回的，不算独立证据；该行 2026-09-27 定的那一整句（读阶段重做一遍、按重做那一遍判）才是。

### 2. 两种读法差在哪

乙入判不出的判据对「坏槽集合」单调：坏槽越多，「有槽自证不过」与「没有一块盘两槽都自证过」只会更容易为真。整个读阶段重读按重读那一遍的坏槽集合 B2 判；槽级重读只重读第一遍坏的槽，坏槽集合是 B1 与 B2 的交集，不大于 B2。所以槽级的拒是整阶段的拒的子集，差只在「第一遍好、重读那一遍读坏」的槽：整阶段拒，槽级放行。这一形是单侧多拒，r1 判的「差异只在多拒一侧」成立（推的，手推；用例也是这样造的：「只重读第一遍坏的那几槽时那几槽重读读好、会放行」，`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:1154`）。

E205 的六格与撕裂轮换 513 态都在「故障一次挂载内不变」的取样里（同一份验证包用例头写着「故障在这次挂载里不变（重读读到同样的字节）」，`crates/singlefs-verification/tests/writable_mount_counts_system_configuration_slots_still_failing_self_verification_as_read_errors.rs:12`），在这个域里两种读法的答案逐格相同，所以用户定案所依的代价数不区分它们。多拒的那一域（一槽第一遍好、重读读坏）没有量过，多拒多少是未知数。

### 3. 「改条款不改代码」是不是同样打中别的读法

r1 的打中是 D22 第 185 行旧句（「重读那一槽一次」）与实现不符。对三种读法一视同仁地看：

| 读法 | 对得上 | 对不上 |
|---|---|---|
| 整个读阶段重读，按重读那一遍判（今天的实现） | N-配置那一整句（第 397 行）、同函数 F1 乙先例（`mount.rs:9390`）、用户原话（不指粒度）、E205 取样域 | 登记第 45 行（用户没选的一行）、第 46 行「仍是」的字面 |
| 槽级重读（D22 旧句的读法） | 第 45 行、第 46 行字面 | N-配置那一整句：同一判据下乙入判不出一半重读一个子集、另一半（R2、c_见证比较）整阶段重做；要把两遍的好槽合起来判，破「按重做那一遍判」；要新写代码；与 F1 乙先例不同口径 |
| 盘级重读（重读有坏槽的盘的两槽） | 无 | 与两边都不合 |

所以这一条打中不是只打实现：把代码改成槽级会撞第 397 行那一整句用户定案与先例，改条款只动一句 2026-11-07 写回的话（`.claude/kb/decisions/22-单元原子性怎么合成.md:185` 现在写「第一遍判据为真时整个读阶段重读一次（不只重读那一槽），只按重读那一遍判」）。r1 选的这一边冲突少，判得对。

### 4. 要补的限定（不翻判决）

- r1 判决写「用户定的原句…没说只重读那一槽」只对第 46 行成立；第 45 行（乙直拒）说了，第 46 行的「仍是」字面也偏槽级。判决里应写成：用户选的是整族「乙族：重读仍坏就拒可写」与「乙入判不出」，两句都不指粒度，粒度由 N-配置那一整句（第 397 行）带来。
- 「无岔路」是主 agent 的判断，不是用户定的；多拒域没量，所以应标「推的，多拒域未量」。
- 什么现象会推翻：用户说明乙入判不出只重读被判坏的槽；或量到一类真实的瞬时读错（好槽重读时读坏）出现得足够多，整阶段多拒影响可用性；两者之一出现，改成槽级代码，同时改第 397 行与 F1 乙先例的口径，不止改一处。

## 五、C973 第 0 代根判据：第二份样本

r1 第四节写：判据「没打中」只有攻方一次抽样加主 agent 复跑它手抄的模型，下一次代码轮再攻，算第二份样本。这一节是辩方腿的第二份样本，两条路：手推（读代码，下面第 1–4 条）与独立模型（第一节，待主 agent 跑）。

### 1. 手推：判据只在一个很窄的窗口里起作用，窗口里它必触发

- 落点公式：txg 落在区域「region: checkpoint_txg.0 % ROOT_RING_REGIONS,」（`crates/singlefs-core/src/root_ring.rs:172`），槽「slot: (checkpoint_txg.0 / ROOT_RING_REGIONS) % slots_per_region.count(),」（`crates/singlefs-core/src/root_ring.rs:173`）。txg 取 0 .. 3S−1 时 (txg mod 3, (txg div 3) mod S) 是一一对应，所以任意 3S 个连续 txg 每槽恰好落一个，S 不必是 2 的幂（S 只出现在 `% S` 里，判据没有按位运算）。
- 第 0 代根种在三个区域的槽 0：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」（`crates/singlefs-core/src/make_filesystem.rs:817`）。发布路径第一次写到槽 (r,0)：r = 1、2 时是 txg 1、2，r = 0 时是 txg 3S。三个数都不超过 3S，所以判据里 take 的上限「.take(ring_length_in_slots)」（`crates/singlefs-core/src/instance_table_sweep.rs:612`）截不掉要紧的那个 txg；最大 txg 越过一圈以上时只要范围里含这一个，`any` 就已经为真，环绕不改答案。
- 清扫走得到判据的窗口：读根环遇到全零槽整轮不删：「return Err(NoRowDeletedThisRound::RootRingSlotIsUnreadableOrAllZero { ring_slot });」（`crates/singlefs-core/src/instance_table_sweep.rs:574`）；mkfs 把未种根的槽清零：「并把三个根环区域整段写 0（含未种根的那 3 × (S−1) 个槽）」（`.claude/kb/decisions/22-单元原子性怎么合成.md:183`）。槽 (r,s) 第一次被发布写到的 txg 是 r + 3s，最大的是 3S−1（槽 (2,S−1)），所以清扫要到发布了 3S−1 个 txg 之后才走得到判据。下面假设发布的 txg 连续、没有丢写（跳号与丢写同形：落点 txg 没发布，第 0 代根留得更久，只影响活性）。没有丢写时，槽 (1,0)、(2,0) 在 txg 3S+1、3S+2 之前存着 txg 1、2 的根，槽 (0,0) 到 txg 3S 才被覆盖：第 0 代根在这个世界里只在 T = 3S−1 这一个点上还存在（T 是已发布的 txg 数）。
- 所以没有丢写的单故障世界里，错位读能拿到第 0 代根的只有 T = 3S−1，被换的是槽 (1,0)、(2,0) 里 txg 1、2 的真根，这一遍读出的最大 txg 是 3S−1，比 1、2 大 3S−3 以上，判据触发。这一条与攻方抽样的结论一致，但路径是我从代码推的，不共用它的手抄模型（推的）。

### 2. 条款上界式对得上

`.claude/kb/checks-owed.md` C973 现在写「第 0 代之后至多一圈 3S 个 txg 里落到这一槽的那一个」，对：txg 1 .. 3S 是 3S 个连续 txg，每槽恰好一个（第 1 条第一项）。

### 3. 调读者同一个落点核

清扫、恢复、挂载都调同一个函数 `root_ring::root_sits_in_the_ring_slot_the_formula_gives`：清扫 `crates/singlefs-core/src/instance_table_sweep.rs:584`，恢复 `crates/singlefs-core/src/recovery.rs:2010`，挂载 `crates/singlefs-core/src/mount.rs:3046`，定义 `crates/singlefs-core/src/root_ring.rs:183`。成立。顺带记一笔：读者那几处用同一个落点核，同样把第 0 代根认在三个槽 0 的任一槽，清扫另加的第 0 代根判据读者那几处没有；读者一侧错位读回第 0 代根会怎样，不在本轮范围，推不出。

### 4. 上界 `..=` 与 `..` 等价

差别只在 txg = 这一遍最大 txg 恰好落在这一槽：这一槽读到的是第 0 代根，它的真根（若有）不在读数里，别的槽里没有 txg 等于这一槽的落点 txg 的根（一个 txg 只落一个槽），所以最大 txg 不会等于这一槽的落点 txg。上界取不取等号，在可达输入上结果相同。模型 `exclusive` 变异应仍 `violations=0`（待主 agent 跑）。登记变异行里有「C973 清扫判第 0 代根时往上只走环长少一个 txg」（`crates/mutations.tsv:6995`）咬 take 的上限，没有咬 `..=` 的行，与 r1 判「不进变异表」一致。

### 5. 零轮：丢写加错位读叠在一起，判据放过一格

- 构造（手推，模型 `l2_unflagged_newest` 待跑）：说谎的设备吞掉 txg 1 的根槽写（槽 (1,0) 留着第 0 代根），发布到 T = 3S 为止；槽 (0,0) 刚被 txg 3S 写成最新根；一次错位读把 (0,0) 换成槽 (1,0) 的第 0 代根。这一遍读出的最大 txg 是 3S−1（槽 (2,S−1)），范围 1 .. 3S−1 里没有落到 (0,0) 的 txg，判据不触发，(0,0) 的真根是环里最新的一条。
- 后果条件：删行的筛是「.filter(|row| !instances_with_a_root_in_the_ring.contains(&row.instance))」（`crates/singlefs-core/src/instance_table_sweep.rs:367`），只看环里读出的根。若这个实例在环里只有 txg 3S 这一条根，行被删；这个实例是现行实例本身（它的新根就是最新的）。要凑齐：两处故障（丢写加错位读）、T 恰好等于 3S、实例在环里只有这一条根。
- 与 r1 的关系：r1 只把「丢了写进槽 0 的根槽写」单独写进射程（`.claude/kb/checks-owed.md:613` 「丢了写进槽 0 的根槽写（说谎的设备吞掉 txg 1 那一次）时，清扫每轮都停，最多停一圈（3S 次发布），只少删不多删」）；单次丢写本身的白停数是 3S，对得上（模型 `l1_longest_false_stop_in_publishes` = `three_s`，待跑）。叠上错位读不是「只少删」。这超出单故障承诺，我标零轮、推的；不翻 r1 对单故障的判决，由主 agent 定是写进射程还是把现行实例的号并进「环里有根」的集合。

## 六、八格检查单

| 格 | 回 | 依据 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 实现员那一句：「无。产品代码（`crates/singlefs-core/src/`）里没有新加 `todo!`、`unreachable!`、`assert!`，也没有条款没定的错误成员。」（`~/.local/share/singlefs/work/m3-close-b20-code-r1-fixes/result.md:103`）。我对暂存区里 `recovery.rs`、`mount.rs` 的新加行逐行 grep `todo!` / `unreachable!` / `assert!` / `.expect(` / `unwrap(`：0 处；新成员 `WitnessedCounterComparison::SomeSlotFailedSelfVerificationAndNoDeviceSelfVerifiedBothSlots` 是见证读的比法，不是挂载错误成员，`MountError` 没动。补一笔：`SystemConfigurationSlotReadsRecordedByTheCheck` 的 `note_that_the_bytes_read_fail_self_verification` 里已有的 `.expect(…刚经这一层读过…)`（`crates/singlefs-core/src/mount.rs:6719` 附近）多了第二个调用点（`newer_publish_witness`），理由仍成立：每块盘新建一个记录层，`system_configuration_slots_of_one_device_read_once` 读完一槽当场报自证不过，同落点同长度，`.rev().find` 必命中 |
| 类型与名字 | 成立（带两处轻注） | 新字段 `u64` 计数，与同结构 `slots_reporting_a_read_error: u64` 同型（`crates/singlefs-core/src/recovery.rs:5124`），比较只写 `> 0`；暂存区新加行 grep `_ =>` 0 处，新成员在 `witnesses_a_publish_newer_than_the_selected_version` 里列名匹配（与相邻一支同为 `true`，没写通配）。轻注 1：新加注释里写「fsid」（10 行），与登记行、全仓同写法，命名里没有缩写；轻注 2：验证包 `mount_transparency.rs` 乙那一判带一处 `.expect("一块盘的系统配置槽数装得进 u64")`，相邻 R2 判官有同样一句，理由成立，不在 `crates/singlefs-core/src/` 内 |
| 不兼容旧代码与旧格式 | 成立 | 暂存区四份文件（`recovery.rs`、`mount.rs`、`mount_transparency.rs`、新用例）新加行 grep `legacy` / `compat` / `兼容` / `迁移` / `旧版` / `旧格式` / `旧布局`：0 处；没有版本号常量；新字段是普通计数 |
| 不新建判定库 | 成立 | 新用例的盘是内存里的稀疏盘与 `MemoryPool`，grep `std::fs` / `File::` / `rocksdb` / `sqlite` / `.tsv`：0 处；变异行进的是已有的 `crates/mutations.tsv`；我的模型只往标准输出与 `--out` 文件写文本，不存判定 |
| 所有任务多机可用 | 成立 | 新用例 grep `std::env` / 主机名 / `/home/` / `/tmp` / `NFS_SHARED`：0 处，不读本机状态；模型与 `rerun.sh` 只用 `$MODEL`、`$WORK`，路径落在工作根下 |
| 测试优先多线程 | 成立（不用切片） | 三条新用例（`:1093`、`:1125`、`:1154` 起）各建自己的盘面、挂一次，grep `static` / `Mutex` / `thread`：0 处，互不共享状态；cargo 默认并行跑同一目标里的各函数，单场景单挂载里没有可再切的 |
| 带 `--item` 的命令与项 | 部分成立，其余推不出 | 变异行确为 6 行，第 7064–7069 行。证红 10241 的日志我读了：点名 3 条、抓到 3 条，是第 7064、7065、7069 行；第 7066、7067、7068 行（计数 `+= 0`、判官两行）等 10312，此刻没有结局，推不出。重读粒度那一行（第 7069 行）咬的是「重读那一遍的乙入判不出形不拒」，不咬「整阶段改成槽级」；后一种由用例本身钉（`:1154` 的注释），变异表里没有对应行，不算缺口，用例会红 |
| 新崩溃流多罩了什么 | 成立 | 三条用例是单盘面单挂载，没有新加崩溃点重放流、崩溃枚举用例或 `--item` 项；`ReadFaultsInTheCrashState` 只是把一个镜像加读故障交给判官的载体，没有枚举崩溃点 |

## 七、要主 agent 跑的命令

只有一条（模型复跑）。我没有往任务服务交任务。

1. 命令原样：

```
task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r2-defense/model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs/research/prompts/m3-close-b20-code-r1-snapshot --work /home/fy5090/.local/share/singlefs/work/m3-close-b20-code-r2-defense/rerun-1
```

2. 在哪个目录起：仓根 `/home/fy5090/code/singlefs`。`--frozen` 只是模板要求给一个存在的目录，模型不读它。`--work` 要给没用过的新目录。
3. 看哪几行判结果：`TASK_SERVER_RESULT` 那一行 `type=succeeded` 且 `exit=0`；`task-server/task log <号>` 里依次出现四个 `violations=` 行：第一行（正常跑）`violations=0`，第二行（`never`）与第三行（`take-short`）非 0，第四行（`exclusive`）`violations=0`，最后一行 `name=rerun_roots …`；出现 `✗` 行就是弄坏开关没红或等价变异红了。再读 `rerun-1/out/model.out` 每个 S 的一行：`p1_all=0`、`p1_reachable=0`、`p2=0`、`p4=0`；`l1_longest_false_stop_in_publishes` 等于同行的 `three_s`；`l2_unflagged_newest` 不小于 1（第五节第 5 条的预测）。`p1_cases` 应大于 0（否则 P1 是空断言）。

等这条结果才能往下做的：第五节第 1、4、5 条里标「待主 agent 跑」的读数；在它跑出之前都是「推的，待主 agent 跑」。结果出来后如要我接着读输出，新派同立场的腿，派发提示写「承接：`research/prompts/m3-close-b20-code-r2-sonnet-output.md`」并给任务号。

另：`--item` 复判 10312 此刻 running，其结局决定第六节第 7 格里第 7066–7068 行是否抓到，不是我这里的命令。

## 八、结论

- 三处打中都够得着、改法修到了：别池槽（第三节第 1 条）与拒因注释（第 3 条）无保留；重读粒度（第四节）判决站得住，要补两处限定（第 46 行「仍是」的字面、多拒域未量），不翻判决。
- 「重读粒度改条款不改代码」不只打中实现：改成槽级会撞 N-配置那一整句用户定案与同文件先例；打中对各读法一视同仁，r1 选的这一边冲突最少。
- C973 判据：单故障下第二份样本也没打中（手推；模型待跑）；叠加故障（丢写加错位读，T 恰为 3S）找到一格判据放过、现行实例的行会被删，零轮、超出单故障承诺，由主 agent 定写进射程还是并现行实例号进「环里有根」。
- 八格：七格成立，「带 --item 的命令与项」一格部分成立、三行等 10312。
- 什么现象会推翻各条：别池槽——有一块盘两槽都自证过时新用例阳性对照红，或 7064、7065 的变异行抓不到；重读粒度——用户说明只重读被判坏的槽；C973 单故障——模型 `p1_all` 非 0；C973 叠加格——模型 `l2_unflagged_newest` 为 0；八格——对应 grep 在别的新加行里命中。

## 没做什么

- 没往统一任务服务交任务；没跑模型（只做了 `python3 -m py_compile` 与 `bash -n`），模型读数全是「推的，待主 agent 跑」；判决要引的数没复跑，命令在第七节。
- 没读禁读清单里的攻方报告；没改仓里任何文件（只写了本报告与草稿目录里的模型）。
- 不替主 agent撤销或维持判决，只交复核结果；没有核合入后验证 10243 的 218 条，也没核 clippy / build（引的是主 agent 给的读数）。
- 没有造「盘 1 一槽布局位不认识」那一形的手造盘面，也没逐字段核判官解析与核心层 `parse_slot` 在几何字段不一致的别池槽上是否同拒。
- 没有评攻方面的内容：只读挂载与离线修复路径、透明判官择系统配置不按 fsid 筛、理想模型与产品是否共用错误前提，这些归攻方腿，这里只在第三节顺带记了一两处读到的事实。
- 清理：草稿目录 `~/.local/share/singlefs/work/m3-close-b20-code-r2-defense` 现 40K，里面只有模型两份文件，没有仓副本和编译目录，留给主 agent 跑；用完由主 agent 在判决写完后删。
