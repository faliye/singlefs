# gpu-tests-placement-design-r2 攻方腿（opus）

日期 2026-11-19。读的是主工作区此刻的树（派发没给冻结副本）。没写模型：模型目录 `research/prompts/gpu-tests-placement-design-r2-opus-model/` 空着，没有 `rerun.sh`；要主 agent 跑的只有「要主 agent 跑的命令」一节的三条（编 `unit-graph`、两次 `closure`），读数都标「推的，待主 agent 跑」。禁读清单里的文件一份没读。

## 各格判定一览

| 攻击面 | 判据格 | 判定 | 一句话 |
|---|---|---|---|
| 一 | 第 1 格 两侧录入键怎么证 | 打中（判据本身写错，不分辨臂） | 录入键里有两段按定义各侧自己算（录入函数单元指纹、参照一侧判法单元指纹），同一份输入两侧同一把键做不到；硬要相同，只有让产品一侧借验证包那一组的指纹，键就过窄 |
| 一 | 同上 | 打中 | 只钉黄金值拦不住一侧改格式、连自己的黄金值一起改；门禁格两侧各算再比，比到的是注入同一把指纹之后的算法，比不到各侧生产路径喂的是哪一把 |
| 一 | 同上 | 打中 | 「写进库的值逐字节相同」在 `19` 表上同一侧隔一天都不成立（值里带判定日期）；写进同一个键空间就要照抄布局号 6，一侧升布局时先开库的那一侧决定整库能不能开 |
| 二 | 第 3 格 实验包的接法 | ⑧ 本身没被绕过；理由那一句落空，打中 | 产品依赖实验包会被 ⑧ 判红；但 E202 的对拍两边可以都是产品代码，⑧ 只认两个包名，看不见。这是测试包今天就有的盲区，搬过去照样带着 |
| 二 | 同上 | 打中（正文漏看） | `bin:` 目标的登记有两组：E185 一组、E161 一个观察者。验证包引擎按「设点文件 + 观察者名」查自己编进去的那张表，E161 搬走之后，要么验证包登记表点名实验包里的 fn（反向耦合），要么改引擎接口 |
| 二 | 同上 | 推的，待主 agent 跑 | 包一换，闭包能看见的包就变了（测试包看得见验证包，产品包看不见），键的成员可能变 |
| 二 | 同上 | 打中（正文与第一轮判决都漏了） | D13 已定项 15 写明「装置二进制的内联单测也在里面」：按条那一路只跑 `-p singlefs-verification`，装置搬走后这批内联单测掉出门禁 |
| 三 | 第 5 格 共用模块 | 打中 | e184 经 `#[path]` 一层层引进 5 份 `tests/` 模块，不是正文写的 4 份；另有一份两个 bin 共用的 `src/bin/` 子目录模块 |
| 三 | 同上 | 打中 | 乙等于抄两份：E170 的停机条款 S1 拿自己抄的那段去和 `build_pool` 比，乙之后成了副本比副本 |
| 三 | 同上 | 打中（判据写错） | 「换了 340 个测试的键」不是代价（D13 已定项 17「改键让旧库作废不算代价」）；甲、丙都会换键，只有乙不换 |
| 四 | 第 6 格 E202 | 打中 | E202 带着 131 条 `#[test]`，其中有真卡上产品 GPU 路径的对拍，经判定库进 product-gpu-real-card-tests 那一格：实验包装进它就是测试包的形状，与取卡那一层怎么选无关 |
| 五 | 第一轮判决的推论 | 打中两条 | 「实验包只装装置二进制」照今天的 E202 不成立；「撤的理由是两处特例键、不是代码各写一份」与 11-11 记录第 126、130、134、141 行对不上 |
| 五 | 同上 | 打中 | 判决第 24 行「函数体三行逐字相同，坐实」不对：头一行形参名不同；仓里另有第三份 |
| 六 | 正文「实现今天的样子」 | 7 处对不上 | 见第六节逐条 |

「打中」照 `.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」的四问，各节末尾逐条答。

## 一、第 1 格：两侧录入键的证法

### 1.1 同一份输入、同一把键，照定义做不到（不分辨臂）

录入键是两样拼起来取 SHA-256：「singlefs recording key\nrecording_units={}\ninputs={}\n」（`crates/singlefs-verification/src/crash_library.rs:1789`）。头一样是录入函数单元那一组的指纹，取自验证包库里编进去的那一把：「&judging_units_key(&JudgingUnits::Recording, toolchain_of_this_process())」（`crates/singlefs-verification/src/crash_library.rs:1773`）。那一组登记在验证包 `lib` 目标上：「group_name: "recording",」（`crates/singlefs-verification/src/judging_units_entries.rs:122`），入口是 `crash_facts`、`crash_judge_tables`、`RecordedFlow::to_bytes` 这些（同文件 125–132 行），全在验证包里。

参照一侧要上卡的对拍，输入摘要里还拼了参照一侧那一组的指纹：「singlefs pairing input with the reference\nreference_units={}\ninput={}\n」（`crates/singlefs-verification/src/crash_library.rs:2400`）。

两段都是「各自那一份代码的指纹」。产品 GPU 包照 F1 自己写存取代码之后：

- 录入函数单元指纹：产品一侧的录入函数是另一份代码，指纹不同。今天测试包取的是验证包那一把：「&judging_units_key(&JudgingUnits::Recording, toolchain_of_this_process())」（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:314`）。产品包不能依赖验证包，这一把就拿不到。
- 参照一侧判法单元指纹：由产品包自己的构建脚本算，入口是产品一侧的函数。

所以同一份输入在两侧算出的录入键必然不同。要让它相同，只剩一条路：产品的构建脚本经 `singlefs-unit-graph` 去收验证包里 `recording` 那一组入口（`load_workspace` 装的是整个工作区，`crates/singlefs-background-gpu-tests/build.rs:124`，技术上收得到）。可这样一来，产品一侧自己的录入代码改了键不变，验证包的事实表抽取改了键反倒跟着变——这正是 D13 已定项 17 点名的违例：「键过窄（引用的常量、类型、宏漏收）、库放在会丢的位置、改名丢库、两份库重复判同一批，同样算违反。」（`.claude/kb/decisions/13-验证路线.md:436`）。

另外，两侧根本没有一份共同录入的输入：验证包录的是崩溃流、事实表、判表，以及它自己的两组对拍；产品录的是 GPU 核的对拍输入。「同一份输入」只能由证法自己造。

四问：
- 分不分辨臂：不分辨。正文给的两种证法（黄金值、门禁格）和任何别的证法，碰到这一条都一样做不到。
- 系统看不看得到：看得到。两段指纹都写在键的原文里，是定义的一部分，不是观测不到。
- 字面满足哪一句：正文第 1 格「同一份输入两侧写进库的字节怎么证逐字节相同」，加上 11-11 记录里的那一句：「照这句判：包之间可以互不依赖、存取的代码可以各写各的，但同一份输入算出的键与写进库的值要逐字节相同」（`records/2026-11-11-GPU两段与按需领卡核查.md:141`）。
- 改法碰不碰得到：两种候选证法都碰不到，病根在判据。判据能做到的最强版本是：给定同一组组成值（录入单元指纹、参照单元指纹、输入字节），两侧拼出的键字节相同。这是「算法逐字节相同」，不是「同一份输入键相同」。另要一条：每侧的键都盖住自己那一份代码（见 1.3）。这是我的改法：只在读码上推过，被攻过零轮。

### 1.2 只钉黄金值：拦得住什么，拦不住什么

拦得住：一侧在自己的黄金值没改的前提下误改了拼法。

拦不住：
1. 一侧连黄金值一起改。验证包改了 `crash_library.rs:1789` 那一行的拼法（比如多拼一段），同时把验证包测试里的黄金值改成新值；产品包测试钉的还是旧值，照样绿。没有哪一格同时读两侧，所以没人发现。仓里今天没有钉过这两个拼法的黄金值：拼法串只在 `crash_library.rs` 里出现一次（1789、1801、2400 三行，`grep -rn` 在 `crates/` 下只命中这一份）。
2. 黄金值放在两侧之外、两侧都读的同一份数据文件里，才拦得住第 1 条。但一份测试 `include_str!` 读另一处目录的文件，就落进前提第 6 条与 ⑨ 要判的那一类。不用 `include_str!`、改成运行时读文件，算不算「跨包引」，前提第 6 条没写，交主 agent 定。
3. 只钉 `Ok` 那一支。拒算有两支各自的拼法：录入单元拒算拼「recording_units=refused」（`crates/singlefs-verification/src/crash_library.rs:1801`）；参照一侧拒算时整个绕开带参照的那层摘要，只拿输入本身的摘要（`crates/singlefs-verification/src/crash_library.rs:2461-2463`），同时置 `skips_the_recording_on_a_hit: false`。产品一侧在参照拒算时如果照旧套一层占位的参照摘要，拼出的字节就不同；只钉 `Ok` 的黄金值看不出来。三支要各钉一条。

### 1.3 门禁格两侧各算再比：拦得住什么，拦不住什么

拦得住：两侧的拼法函数在门禁喂进去的那批组成值上逐字节不同（包括一侧改了、另一侧没跟）。

拦不住：
1. 门禁只能比「注入同一组指纹之后的拼法」（相当于 `recording_key_of_the_inputs_under`，`crash_library.rs:1781`），因为两侧真跑时的指纹按 1.1 必然不同。各侧生产路径实际喂进去的是哪一把指纹，它比不到。产品一侧录入键用错了组（比如拿 GPU 一侧那一组当录入单元，或参照组漏收了编码函数），门禁照绿，键却盖不住该盖的代码。这就是「两侧一起错」的样子：拼法两边一致，喂进去的东西两边都不对。能发现它的只有「改一个该进键的单元，键必须变」那一类检查（`crates/singlefs-unit-graph/tests/unit_graph_fixture_keys_change_exactly_where_the_rules_say.rs` 是现成的形态），正文的两种证法里都没有。
2. 产品包的源码不能有给门禁用的二进制（`src/bin/` 是产品），产品一侧只能在 `tests/` 里写一条打印键的测试，由门禁起。这条测试的单元键复用、记红撤绿也要照 D13 已定项 17 的规矩配，正文没提。
3. 这两种证法都只管键。清单的值（`ItemManifest::to_bytes`，魔数「const ITEM_MANIFEST_MAGIC: &[u8; 8] = b"SFITEM01";」（`crates/singlefs-verification/src/crash_library.rs:1819`）、种类串「kind: "input_pairing_with_the_reference".to_string(),」（`crates/singlefs-verification/src/crash_library.rs:2498`）、清单里两样的名字 `input` / `reference`（同文件 2264、2381 行））都要产品一侧照抄成字面量。一个没钉，两侧写进 `10` 表的值就不同。

### 1.4 「写进库的值逐字节相同」与「算不算另开存放处」

- `19` 表（对拍结论）的值带判定日期与跑法：结论入库时写「circumstances: JudgementCircumstances::of_this_process_now(),」（`crates/singlefs-verification/src/crash_library.rs:2645`），里面是「judged_date: tokyo_date_text_of(std::time::SystemTime::now()),」（`crates/singlefs-verification/src/verdict_store.rs:680`）。同一侧、同一份输入隔一天写，值都不同，逐字节相同只能限在键和不带出处的那几样上。正文第 1 格要写明比哪几张表、比哪几个字段。
- 产品一侧写进哪个键空间，结果是两种：
  - 写进崩溃判定库那一个键空间，键「原样（表标签开头）」：「（表标签开头）；别家（Python 三家）的键以 ASCII 族名起头，与表标签字节不撞。」（`crates/singlefs-verification/src/verdict_store.rs:2773`）。这算不上新开存放处，但要照布局行的规矩办。验证包开库时，布局行不在就写下这一版的号：「let layout_bytes = LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION.to_be_bytes().to_vec();」（`crates/singlefs-verification/src/verdict_store.rs:6524`），号是「const LIBRARY_LAYOUT_NUMBER_OF_THIS_VERSION: u32 = 6;」（`crates/singlefs-verification/src/verdict_store.rs:1655`）；在而不是这一版的，整库拒开（同文件 1684–1691 行）。产品一侧要么照抄常量 6、照写照核，要么不管布局行。照抄的话：验证包把布局升到 7、按「不兼容旧东西」换新库，新库里要是产品测试先开、写下 6，验证包随后开库就整库拒，整道崩溃点重放停摆。不管的话：验证包换了布局，产品一侧照旧往新库里写旧布局的行，验证包认得开、却读到旧格式。两种都是「一侧漂移、另一侧当场才炸，或者根本不炸」，正文的两种证法都没把布局号放进比对。
  - 另起一个族前缀（照 Python 三家那样以 ASCII 族名起头）：两侧的行永远不在同一处，两侧键值逐字节相同这一条要求就没有对象了，只剩 1.1 说的「算法相同」；这和 `.claude/agent-common.md` 里那一句「判定一律经 prune-server 客户端按各族的键前缀写进它自己那一份 RocksDB」（`.claude/agent-common.md:29`）相容，不算新开存放处。
- 一次性库：产品测试里不上卡的那几条今天跑在验证包的 `DisposableLibrary` 上（键空间头字节「pub const DISPOSABLE_LIBRARY_KEY_SPACE_LEADING_BYTE: u8 = 0xE0;」（`crates/singlefs-verification/src/verdict_store.rs:2778`））。产品一侧自写，还得再抄一份名字空间的做法，头字节、标签、进程号、时刻、序号的拼法都得抄对，两份撞了会互相看见行。正文第 2 格没把这一样列进产品要写的最小一组里。

### 1.5 这一节没打中的形状

- `1C` 表（对拍输入）：键是输入字节的 SHA-256，值就是输入字节本身（`verdict_store.rs:4472-4478` 的 `store_pairing_input`）；大值分段用的是 `prune_server::segmented_values`（`verdict_store.rs` 第 2767 行起那一段注释），在允许共用的 `prune-server` 里。这一张表两侧不会不同，用不着证。
- `19` 表的键（输入摘要 + 两组判法单元指纹）：两组指纹都由 `singlefs-unit-graph` 算（允许共用），算法本来就是同一份；键在哪侧算出来都由同一组入口定，打不出两侧不同的反例。

## 二、第 3 格：实验包的接法

### 2.1 ⑧ 本身没被绕过（没打中）

⑧ 判的是「两边的传递闭包（dev-dependencies 也算）里都没有对方」（`.claude/gate.d/checker-independence-and-sync.sh:38`）。产品 GPU 包只要依赖实验包（`[dev-dependencies]` 也一样），产品的闭包里就有了验证包，⑧ 当场红。所以「产品代码经实验包间接用到验证包的代码」在 Cargo 依赖这一层走不通。试过的形状：产品 `tests/` 经 dev 依赖引实验包的库来复用取卡模块，⑧ 判红；实验包 `src/lib.rs` 转出验证包的类型给产品用，同样要产品依赖实验包，⑧ 判红。

### 2.2 ⑧ 的理由那一句，在实验包里没有东西兜（打中）

⑧ 的理由是：「⑧ 同一个理由落在 GPU 上：验证包的 GPU 判器与产品 GPU 包共用一段代码时，那段写错会让判器与被判的产品路径一起错，GPU 对拍照样全绿。」（`.claude/gate.d/checker-independence-and-sync.sh:50`）。⑧ 认的只是两个包名。实验包两边都能依赖，里面的对拍把结论写进判定库 `19` 表、按键复用；这样的对拍，参照一侧与被判一侧用不用同一段产品代码，没有任何一格判。

E202 今天就是这个形状：它的内联真卡对拍是产品 GPU 核对装置自数（GPU 一侧入口 `judged_on_the_cards_that_the_gpu_arm_on_a_real_card_counts_the_same_live_slots_per_segment_as_the_apparatus_on_the_small_pool`，`e202_gpu_cpu_gc_comparison.rs:8920`），同一份文件里还直接引了产品包的 CPU 参照：「use singlefs_background_gpu::region_record_kernels::cpu_reference_classification;」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:84`）。我核过，今天参照一侧的装置自数 `live_slots_of_each_segment_counted_by_the_apparatus`（同文件 8300 行起）没调产品代码，所以今天的代码没有同源。但哪一天有人把参照换成产品那一份，照样全绿、照样入库复用，没有门禁会红。这是测试包今天就有的盲区，搬进实验包原样带过去，第 3 格「两边仍互不依赖」这句话本身补不上它。

四问：分不分辨臂——不分辨，任何同时依赖两边、又做入库对拍的包都一样；看不看得到——看得到，闭包里能看出参照一侧收了哪些产品单元；满足的是 ⑧ 理由那一句，不是 ⑧ 的判法字面；改法——在实验包里给「入库的对拍，参照一侧那一组的闭包不许含被判一侧包的单元」配一格。这是我提的，只在读码上推过，被攻过零轮。

### 2.3 `bin:` 目标的登记有两组，E161 那一组正文漏了（打中）

验证包登记表里目标写 `bin:` 的是两组：E185 的参照组「group_name: "e185_cpu_reference_crcs",」（`crates/singlefs-verification/src/judging_units_entries.rs:146`），与 E161 的观察者「observer_name: "e161_crates_comparison",」（`crates/singlefs-verification/src/judging_units_entries.rs:537`），后者的文件路径写在同文件第 536 行。正文第 3 格只举了 E185、E202。

两组的取法不同，搬走之后要改的东西也不同：

- E185：装置调验证包的 `judging_units_key(&JudgingUnits::PairingComparison { group_name })`（「group_name: super::CPU_REFERENCE_UNITS_GROUP,」（`crates/singlefs-verification/src/bin/e185_gc_batch_wall_clock_split.rs:812`）），取的是验证包构建脚本编进验证包库里的那一把。装置搬进实验包后，可以改成读实验包自己构建脚本写的表（测试包今天就这样做：`judging_units_key_of_the_group_in_the_table_text`，`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:287`），验证包那一行删掉。
- E161：装置不自己取键。它把设点文件路径与观察者名交给验证包的枚举引擎：「"crates/singlefs-implementation-experiments/src/bin/e161_crash_state_dedup_and_time_split/main.rs",」（`crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs:3242`）。引擎按这一对去查验证包自己编进去的那张表（`crates/singlefs-verification/src/crash_amplification.rs:29435` 的 `observer_code_units_of`）。E161 搬进实验包，只有两条路：
  - 甲，验证包登记表照旧登这一组，入口改成实验包里的 fn。这样验证包的构建脚本要经 `singlefs-unit-graph` 收实验包的闭包、盯实验包的文件；实验包改名或删掉这个 fn，验证包自己就编不过：「.unwrap_or_else(|error| panic!("登记表里组 {} 的入口写错了：{error}", request.group_name))」（`crates/singlefs-verification/build.rs:197`）。整个门禁跟着停。这是验证包在键这一层反过来认实验包，Cargo 依赖图上看不出，⑧ 与 ⑨ 都不判。
  - 乙，改验证包枚举引擎的接口，让调用方把观察者那一把版本交进去。正文第 3 格与第 8 格都没把这一处接口改动算进去。

四问：分辨臂——甲、乙两条路各有各的代价；看得到——表就在那里；满足的是第 3 格「搬过来之后入口登记表与算键在哪」那一问；改法——第 3 格要把 E161 的观察者列进来，并在甲、乙之间选一条。

### 2.4 包一换，键的成员可能变（推的，待主 agent 跑）

键里的条件原子、单元集合，都跟「入口在哪个包」有关：
- `singlefs-unit-graph` 收闭包时，跨目标的裸名「跨目标的只认 `pub`、不按模块筛」（`crates/singlefs-unit-graph/src/closure.rs:25`）。测试包看得见验证包库里的全部 `pub` 名字，产品包看不见。今天测试包里参照组、GPU 组的闭包里只要有验证包的单元，搬进产品包之后它们就掉出去，键跟着变。命令 M1、M2 量今天这两组闭包里有几个单元的文件在 `crates/singlefs-verification/` 下。
- 依赖包这一次开了哪些特性，构建脚本只从直接的普通依赖读（`crates/singlefs-background-gpu-tests/Cargo.toml:9-10` 那两行注释）。产品包的构建脚本里，产品自己的特性不再是「依赖包的特性」，而是请求的特性。产品源码今天没有按 `tests-on-a-real-gpu` 编的代码（`crates/singlefs-background-gpu/Cargo.toml:31`），这一项眼下不改键。

键变了照 D13 已定项 17 不算代价。这一节要核的是第一轮判决「回看决策」那一句：「2026-11-19 不受影响：单元键的组成不变，包名与路径不进键；改的是算键的代码住哪个包」（`research/prompts/gpu-tests-placement-design-r1-main-verification.md:84`）。算法确实不变，可「哪些单元进键」会随包变，这句话说宽了。

### 2.5 装置的内联单测会掉出门禁；D13 已定项 15 没人回看（打中）

D13 已定项 15 的定案写死了两件事：验证包里有「实验与真设备装置二进制」；门禁按条那一路是「`cargo test --release -p singlefs-verification --lib --tests`（`--tests` 带上全部 `test = true` 的目标，装置二进制的内联单测也在里面）」（`.claude/kb/decisions/13-验证路线.md:405`）。装置搬进实验包之后：

- 按条那一路点的是 `-p singlefs-verification`，实验包里装置的内联单测不在里面。E185 的复跑一节写着主工作区那一版有 118 条（背景材料 E185 那一页），E202 有 131 条 `#[test]`（`grep -c '#\[test\]'` 量的）。它们今天分别靠验证包那一趟、靠 product-gpu-real-card-tests 那一格起，搬过去之后要有一格明着点实验包，否则就静悄悄地少跑。正文第 3 格没写实验包的测试由门禁哪一格起。
- 验证包清单里那个只为装置设的特性：「apparatus-binaries = []」（`crates/singlefs-verification/Cargo.toml:49`），装置全搬走后就没有用处了，照「不留兼容」要删，门禁与脚本里配合它的 `--no-default-features` 写法也要跟着看。
- D13 已定项 15 是一条已定决策。F2 改了它的定案句，第一轮判决「回看决策」那张表却只列了 D13 已定项 17 与 D26 已定项 13（`research/prompts/gpu-tests-placement-design-r1-main-verification.md:84-85`）。正文「压着这件事的条款」同样只列了已定项 17。

四问：不分辨臂（实验包不管怎么接都要补）；看得到；满足的是第 3 格「它在 ⑧ 下是什么」之外漏掉的那一问；改法——第 3 格加一问「实验包的测试由门禁哪一格起」，回看表补 D13 已定项 15。

## 三、第 5 格：装置用到的验证包 `tests/` 共用模块

### 3.1 e184 经 `#[path]` 引进的是 5 份，不是 4 份；另有一份 `src/bin/` 子目录模块（打中）

e184 自己引四份（`crates/singlefs-verification/src/bin/e184_regular_layer_cost_counts.rs:40-46`），其中 `common_crash_points` 里又套了一层：「#[path = "../common_gpu_cards_of_the_caller/mod.rs"]」（`crates/singlefs-verification/tests/common_crash_points/mod.rs:540`）。所以 e184 编进来的验证包 `tests/` 模块一共 5 份：`common`、`common_crash_point_observers`、`common_crash_points`、`common_snapshot_prefix`、`common_gpu_cards_of_the_caller`（169 行，`wc -l` 量的；它 `use` 验证包的 `gpu_card_configuration`）。

另外，`src/bin/e184_regular_layer_cost_counts_write_dump/mod.rs`（63823 字节）是一份目录模块，e184 主装置与转储 bin 各经 `#[path]` 引它（`e184_regular_layer_cost_counts.rs:52`、`e184_regular_layer_cost_counts_write_dump.rs:27`）。它不跨包，但搬家时要跟两份 bin 一起走，正文「26 个装置」那一句没点到它。

这几份模块之间是互相 `use super::common…` 连着的（`crates/singlefs-verification/tests/common_crash_point_observers/mod.rs:14`、`crates/singlefs-verification/tests/common_crash_points/mod.rs:15`），甲挪进 `src/` 要五份一起挪；乙自己写，要连第四套取卡一起写（见 4.2）。

### 3.2 乙就是抄两份，而且让装置的停机条款没了对象（打中）

装置引 `tests/common` 不是图省事。E170 的停机条款 S1 拿自己照抄的那段流去和那一份比：「再拿 `build_pool` 整条流逐条回比（S1）」（`crates/singlefs-verification/src/bin/e170_specification_executor_export.rs:10`）。E161 的说明里写：「新池新建文件流经 `#[path]` 引 `tests/common/mod.rs` 的 `build_pool`」（`crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs:18`）——它量的就是门禁崩溃枚举用例用的那一条流。

乙让实验包自己写一份。S1 就成了「装置抄的一段」比「装置包里另抄的一份」，门禁那条流改了，两份副本照样一致，S1 照绿，装置量的已经不是门禁那条流。这不是「各写各的」，是抄两份。

前提第 2 条只在不公用撞上时撤掉 11-11 那句「不要做第二套或者第三套东西」（原话在 `records/2026-11-11-GPU两段与按需领卡核查.md:126`）。实验包依赖验证包是定案允许的（前提第 3 条），这里撞不上不公用，那句话照管。乙撞它。

四问：分辨臂（只打乙）；看得到；满足的是第 5 格「抄两份而不是各写各的」；改法——乙出局，或者乙写明怎么保住 S1 的回比对象（我没找到不跨包又保得住的写法）。

### 3.3 「换不换 340 个测试的键」不是代价，这一问判不了候选（判据写错）

正文第 5 格问「挪动它会不会换这些测试的键」。D13 已定项 17 明写：「改键让旧库作废不算代价：不做兼容迁移、不为旧行补签，改完重跑。」（`.claude/kb/decisions/13-验证路线.md:441`）。再看各候选：

- 甲（挪进验证包 `src/` 的公开模块）：340 个目标要么照旧 `mod common;` 留一份（两份），要么改成用库里那一份。改了的话，调用处的写法变了，指纹跟着变，凡是闭包够到挪走单元的目标，键都换。
- 丙（验证包库里经同包 `#[path]` 把 `tests/common/mod.rs` 也编进库）：同一份文件两处编。测试目标里的裸名跨目标照同名连库里那一份 `pub` 单元（`closure.rs:25` 那条规则），闭包里多出一份同指纹的单元，键照样换（推的，没量）。
- 乙：不换键，可它撞 3.2。

能不换键的只有乙，乙又出局，所以这一问在候选之间分不出高下；照 D13 已定项 17 它本来也不是代价。真正的代价是 340 个目标的测试函数要整批现判一次的挂钟——这是要量的数，不是「撞不撞」。第 8 格应当把它当一个数来量。

### 3.4 甲会不会扩大验证包判器组的闭包（查了，R-开放这一支没打中）

`tests/common/mod.rs` 里有一处泛型 trait impl：「impl<Inner: singlefs_core::block_device::BlockDevice> singlefs_core::block_device::BlockDevice」（`crates/singlefs-verification/tests/common/mod.rs:1181`）。挪进验证包库之后，`lib` 目标上入口签名带 trait 约束或 `dyn` 的判器组，会照 R-开放把链接集里这个 trait 的全部 impl 收进来。我把 `judging_units_entries.rs` 里目标是 `lib` 的 36 个入口按函数名 `grep -A6` 粗筛了签名（不是逐个读，粗筛会漏多行签名）：带 `dyn` 或 `impl` 的有 6 个，约束的 trait 是 `Fn`、`FnMut`、`Into` 与 `ImageReader`（`crash_judge_conclusions.rs:1380`）。没有一个约束 `BlockDevice`，`tests/common` 里也没有 `ImageReader` 的 impl。所以 R-开放这一支不会让判器组的键变。R-裸名「点不准的退回全连」那一支我没查：62 个公开名字里有 22 个在验证包 `src/` 里有同名的词（`grep -rlw` 量的，含注释），会不会退回全连，要跑 `unit-graph why` 才知道。

## 四、第 6 格：E202 的取卡与对拍那一层

### 4.1 E202 自己就是测试包的形状，与取卡那一层怎么选无关（打中）

E202 这份文件里有 131 条 `#[test]`（`grep -c '#\[test\]'` 量的），其中挂 `tests-on-a-real-gpu` 的真卡对拍，测的是产品 GPU 核本身：「同上一条，GPU 点名一张卡臂换成真卡上产品那一份：五遍每段的活槽数都等于装置自己数的。」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/pools_cards_and_sessions_for_tests.rs:204`）。测试函数在同文件 12385–12406 行，经对拍库两段、结论入库按键复用，由门禁 product-gpu-real-card-tests 那一格起：「起产品 GPU 包的测试包（singlefs-background-gpu-tests）带 tests-on-a-real-gpu 特性才编的真显卡对拍测试」（`.claude/gate.d/crash-point-replay.sh:12`）。GPU 一侧那一组的闭包里有产品的 WGSL 核与开卡代码（`src/gpu_side_units_entries.rs` 那一组的入口）。

E202 照原样搬进实验包，实验包就是「同时依赖两边的一个包里放测试产品包的代码」。这和测试包的定义一字不差：「产品的后台 GPU 包（singlefs-background-gpu）的集成测试：真显卡上的对拍与端到端、CPU 顶替的测试替身那一组，连同它们共用的取卡与造池模块，另有实验装置 E202」（`crates/singlefs-background-gpu-tests/Cargo.toml:6`）。取卡与对拍那一层不管选「各写各的」还是「改用验证包已有的」，都碰不到这一点：病根在 E202 带进去的那几条测试。

要躲开，只有把这几条拆出 E202：做法是改成产品 GPU 包 `tests/` 里的对拍（产品自己的 CPU 参照），或者明写它是装置的阳性对照、不进判定库、不进 product-gpu-real-card-tests 那一格。哪一种都要改 E202 的跑前登记——修订第 9 条那一类，登记里写着「各量 5 次」（`e202_gpu_cpu_gc_comparison.rs:8953`）。正文第 6 格没提这一层。

四问：不分辨第 6 格的两个候选（两个都中）；看得到；满足的是派发给的第 4 条「绕回测试包的形状」，也满足判决第 2 条那句前提（见 5.1）；改法——两个候选都碰不到，只有上面那一步拆分碰得到。

### 4.2 取卡那一层今天已经是三份，「各写各的」就是第四份

今天仓里管「服务交的卡、本机哪几张、领卡」的有三份：
- 验证包库 `gpu_card_configuration`（领卡：`cards_handed_over_or_leased_on_demand`，`crates/singlefs-verification/src/gpu_card_configuration.rs:707`）；
- 测试包 `tests/common_discrete_gpus_of_this_host/mod.rs`（859 行，`wc -l` 量的：交来的卡对上本机的 Vulkan 独显、每张卡的显存上限）；
- 验证包 `tests/common_gpu_cards_of_the_caller/mod.rs`（169 行，e184 经 3.1 那条链引进来）。

F1 之后产品 `tests/` 自己写一份，是定案允许的（不公用）。E202 进了实验包、可以依赖验证包，它要是再「各写各的」一份，就撞上前提第 2 条留下的那句「不要做第二套或者第三套东西」（`records/2026-11-11-GPU两段与按需领卡核查.md:126`）。E202 今天已经直接用验证包的领卡了：「use singlefs_verification::gpu_card_configuration::{」（`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison.rs:143`），调用在同文件 5766 行。它从测试包那份模块里另外用的，是对上本机独显那一段；这一段产品库里有现成的 `adapter_inventory`、`background_gpu::VulkanInstanceOfThisProcess`（E202 第 62–69 行已经在引），测试包那份模块也是在它们上面搭的（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:28`、同文件第 33 行）。所以还有第三条路：E202 用产品库的卡清单，加验证包的领卡，只写把两样接起来的那一层。这条路我没量写出来有多少行，标推的。

### 4.3 E202 跨包读的五份 `Cargo.toml` 搬过去照样跨包

E202 的内联测试里 `include_str!` 读了五份别的包的清单：同文件 13836、13840、13844、13848、13852 行，即 `singlefs-background-gpu`、`singlefs-core`、`singlefs-format`、`singlefs-mount-runtime`、`singlefs-replay-prefix` 的 `Cargo.toml`，13832 行另读自己那份。搬进实验包，这五处还是跨包 `include_str!`，正好落在前提第 6 条、⑨ 要判的那一类上。第 9 格要逐处给判法；第 6 格要说 E202 的这几条测试怎么改。

## 五、第一轮判决里主 agent 自己写的推论

### 5.1 「实验包只装装置二进制，不装产品包的测试，所以不是测试包」（打中）

原文：「按装的是什么判：实验包只装装置二进制，不装产品包的测试，所以不是测试包，攻方这一条不采」（`research/prompts/gpu-tests-placement-design-r1-main-verification.md:34`）。

前提「只装装置二进制、不装产品包的测试」照今天的 E202 不成立，见 4.1：E202 带着产品 GPU 核的真卡对拍测试，入判定库、进 product-gpu-real-card-tests 格。判决的判法（按装的是什么判）本身可以留，可用这个判法判下来，结论正相反——E202 原样搬进去，实验包就装了产品包的测试。要让结论成立，判决得加一条做法：「E202 里测产品 GPU 路径的那几条测试不随它进实验包」。

### 5.2 「攻方说乙就是 11-11 撤过的形状不成立：撤的理由是两处特例键，不是因为代码各写一份」（部分打中）

原文那一句（判决第 33 行）的要点：「撤的理由是两处特例键」（`research/prompts/gpu-tests-placement-design-r1-main-verification.md:33`），据此判攻方说乙「就是 11-11 撤过的形状」不成立。

第 128 行的括号里确实只写了两处特例键。但同一节读全了：
- 撤回是照用户这条定案做的，定案原话有「我们在录制入库部分的逻辑应该是一致的。 不要做第二套或者第三套东西。」（`records/2026-11-11-GPU两段与按需领卡核查.md:126`）；
- 撤完紧接着全仓查另一套录制入库，结论是「Rust 里没有第二套」（`records/2026-11-11-GPU两段与按需领卡核查.md:130`）；
- 新开测试包的接法写的是「对拍调验证包的 `crash_library::two_stages`」（`records/2026-11-11-GPU两段与按需领卡核查.md:134`）；
- 「新开的对拍测试包直接调验证包的 `two_stages` 时键值出自同一份代码，自然相同」（`records/2026-11-11-GPU两段与按需领卡核查.md:141`）。

所以 11-11 撤掉的形状有两条理由：特例键，与「第二套」。判决说「不是因为代码各写一份」，与第 126、130、134、141 行对不上。

这一条不翻定案：前提第 2 条里用户已经说了「不能违背不公用的前提。如果有冲突删掉」，产品一侧各写各的照定案成立。会受影响的是不撞不公用的地方（第 5 格乙、第 6 格 E202 各写各的）：在那里，11-11 的「不要做第二套」照管，判决那句读法会让人以为它不管。

### 5.3 「函数体三行逐字相同，坐实」（打中）

原文：「逐字相同的两个函数：…`pci_bus_identifier_without_the_domain`，函数体三行逐字相同，坐实。」（`research/prompts/gpu-tests-placement-design-r1-main-verification.md:24`）。

头一行不同：测试包那份是「let lowered = pci_bus_address.trim().to_ascii_lowercase();」（`crates/singlefs-background-gpu-tests/tests/common_discrete_gpus_of_this_host/mod.rs:207`），验证包那份是「let lowered = text.trim().to_ascii_lowercase();」（`crates/singlefs-verification/src/gpu_memory_ledger.rs:781`），形参名不同。`diff` 两段三行，第 1 行不同、退 1。单元指纹按词法单元取（`crates/singlefs-unit-graph/src/rust_items.rs:143` 的 `fingerprint_of_tokens`），两份的指纹也不同。

另外还有第三份，和测试包那份逐字相同：「fn pci_bus_identifier_without_the_domain(pci_bus_address: &str) -> String {」（`scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:488`）。正文第 7 格只问「两份」。

### 5.4 回看决策漏了 D13 已定项 15（打中，见 2.5）

## 六、正文「实现今天的样子」逐句现查

量法：主工作区此刻的树；命令都是只读的 `grep`、`git grep -l`、`wc`、`diff`，在仓根跑，输出存在草稿目录。

### 对不上的（7 处）

1. 「每一份都引验证包自己的模块，2 到 56 处（逐份 `grep -c '(crate|singlefs_verification)::'`）」（`research/prompts/_gpu-tests-placement-design-r2-body.md:29`）。
   - `e197_scan_journal_glue_memory_by_device_count.rs` 里 `singlefs_verification` 出现 0 次，它只用 `singlefs_core`、`singlefs_replay_prefix`、`singlefs_format`（同文件 28–45、185–199 行）。「2」这个下限是它那 2 处 `crate::`：一处在注释里（第 785 行），一处是字符串字面量（第 2966 行）。
   - 方法本身就错：bin 里的 `crate::` 指的是这份 bin 自己，不是验证包库。按 `grep -c 'singlefs_verification::'` 逐份数，是 0 到 51：e197 是 0；`e184_regular_layer_cost_counts_write_dump.rs` 本文件也是 0（它经 `#[path]` 引的目录模块里有 8 处）；e158 是 51，不是 56（另 5 处是 `crate::`）。
   - 对第 4 格的影响：e197 不依赖验证包，进实验包只要依赖核心层，用不着验证包。
2. 「e184 另引 `common_crash_point_observers`、`common_crash_points`、`common_snapshot_prefix`」（`research/prompts/_gpu-tests-placement-design-r2-body.md:29`）：漏了套在 `crates/singlefs-verification/tests/common_crash_points/mod.rs:540` 里的 `common_gpu_cards_of_the_caller`，也漏了两份 bin 共用的 `src/bin/e184_regular_layer_cost_counts_write_dump/mod.rs`（见 3.1）。
3. 不带实验号的 5 份「与点名它们的地方（`git grep -l`）」（`research/prompts/_gpu-tests-placement-design-r2-body.md:30-34`）：清单不全。`git grep -l <名> -- . ':!research' ':!records'`、去掉 `src/bin/` 下的文件本身之后：
   - 五份里四份被 `crates/mutations.tsv` 点名，行数（`grep -c` 量的）：`crash_library_reader` 26、`new_pool_file_creation_on_device` 33、`new_pool_file_creation_device_log_check` 8、`deletion_write_dump` 6。正文五条清单里一条都没列 `mutations.tsv`。
   - `new_pool_file_creation_on_device`、`…_device_log_check` 两份的六个模式与写路，住在验证包库模块 `crates/singlefs-verification/src/on_device_modes.rs` 里（第 1 行「虚机档 `new_pool_file_creation_on_device` 的六个模式」）。在验证包里点名它的，只有这两份 bin 与 `lib.rs`（`grep -rln on_device_modes` 量的）。两份 bin 搬走之后，这个模块在验证包里就没有自己的用者了；第 4 格要说它留下还是跟着搬。
   - 别的漏列：`.claude/hooks/lib_project_code_runs.py:3419`（钩子自证里的一条样本路径）、`crates/singlefs-verification/src/fault_injection.rs:9`、验证包 3 份测试的注释与函数名（`a_swallowed_barrier_is_in_the_recorded_stream_but_not_on_the_device_side.rs`、`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`fragmentation_statistics_follow_the_probed_device_capacity.rs`）、`crates/singlefs-verification/tests/new_pool_file_creation_write_regions.rs`（点名 `new_pool_file_creation_region_bytes`）、`scripts/crash/README.md`、`.claude/gate.d/path-moves.tsv`、`.claude/preflight-exclude`、`.claude/data-name-lint-exclude`，以及 kb 的 E142、E165、E172、E177、里程碑页与 `tooling.md`（三种语言各一份）。
4. 「门禁 `.claude/gate.d/qemu-device-streams.sh` 与它的 12 个样本档目录」（`research/prompts/_gpu-tests-placement-design-r2-body.md:32`）：点名两份真设备 bin 的样本档目录是 11 个（`tier-key-samples-` 起头的那 11 个）。第 12 个是样本根本身，命中的是根上那份 `PROVENANCE.txt`。数法差一个，算对不上。
5. 「E202 从测试包的两个共用模块用到」（`research/prompts/_gpu-tests-placement-design-r2-body.md:36`）那四样，量法是同一行写的 `grep -oE` 计数；第 6 格又写「上一节列了用到的四样」（`research/prompts/_gpu-tests-placement-design-r2-body.md:50`）。`grep -oE` 只数写全路径的那几处，`use` 引进来的一个都没数。E202 第 147–154 行、13350–13353 行用 `use` 另外引了 11 个名字：`handed_over_gpus_or_report_not_run`、`note_the_cards_leased_on_demand_for_this_process`、`VARIABLE_OF_THE_GPUS_HANDED_OVER_BY_THE_TASK_SERVER`、`append_count`、`append_number`、`judgement_of_the_assertions_on_the_cards`、`ComparisonOnTheCards`、`PairingInputBeingRead`、`pairing_concluded_on_the_cards_in_the_library`、`KeysOfAPairingOnTheCards`、`PairingOnTheCardsOutcome`。加上正文那四样，一共 15 个：取卡模块 6 个，对拍模块 9 个。第 6 格「各写各的还是改用验证包已有的」照 4 样估，工作量少估了。
6. 「两份函数体逐字相同」（`research/prompts/_gpu-tests-placement-design-r2-body.md:38`）：不逐字相同，头一行形参名不同（见 5.3）；而且不是两份，是三份（`scripts/experiment-apparatus/e163_gpu_multicard_crc32c.rs:488`）。
7. 「组的入口登记在验证包 `src/judging_units_entries.rs`（读码，没逐组数哪几组的目标是 `bin:`）」（`research/prompts/_gpu-tests-placement-design-r2-body.md:35`）：这一句没说错，是没数。数出来是两组：E185 一组、E161 一个观察者（见 2.3）。E161 不在正文第 3 格的举例里，按「漏看」记一处。

### 对得上的

- 26 个装置二进制、`Cargo.toml` 26 条 `[[bin]]`（`ls src/bin/*.rs | wc -l` 与 `grep -c '^\[\[bin\]\]'` 都是 26；`src/bin/` 下另有一个目录，见 3.1）。
- 经 `#[path]` 引 `tests/common/mod.rs` 的 6 份：e161、e170、e171 两版、e184、e185。
- `tests/common/mod.rs` 1416 行；`mod common;` 的测试目标 340 个（`grep -lE '^\s*mod common;' tests/*.rs | wc -l`）。
- E185 的 `CPU_REFERENCE_UNITS_GROUP` 在第 464 行，取键的调用在 803–815 行（`group_name` 在第 812 行）。
- `prune-server/src/lib.rs` 第 53 行「pub mod reliable_client;」；`crates/singlefs-verification/Cargo.toml:33` 是 `prune-server` 那一行依赖。
- 第 204 行那句注释今天不对：测试包直接依赖验证包（`crates/singlefs-background-gpu-tests/Cargo.toml:15`）。
- 「产品 GPU 包测试要的验证包东西（24 个名字）」：这是转述第一轮攻方的数，我没重数。

## 没打中的形状

- 产品经实验包间接用到验证包：试了产品 `tests/` 经 dev 依赖引实验包的库、实验包库转出验证包类型两种，都要产品依赖实验包，⑧ 第 38 行那条当场红（2.1）。
- `1C` 表与 `19` 表的键：两侧不会不同（1.5）。
- 甲把 `tests/common` 挪进验证包库，经 R-开放扩大判器组闭包：`lib` 目标 36 个入口粗筛，没有约束 `BlockDevice` 的（3.4）。R-裸名那一支没查。
- E202 参照一侧今天与产品同源：查了 `live_slots_of_each_segment_counted_by_the_apparatus`，没调产品代码（2.2）。
- `prune-server/src/lib.rs` 的 `reliable_client`、`crates/singlefs-verification/Cargo.toml:33`、26 个 bin、6 份 `#[path]`、340 个目标、1416 行：都对得上（第六节）。

## 这条腿自己的限度

- 没编、没跑任何东西。2.4（包一换键的成员变不变）与 3.3 里丙会不会换键，都是读码推的，要靠下一节的命令量。
- 第一节「同一份输入两侧同一把键做不到」是照键的拼法推的；用户那句「输出的键值要一样」原意是不是「算法一样」，我判不了，交主 agent 问用户。
- 第六节第 3 条的漏列，是按 `git grep -l` 出的文件清单比的；每份文件里点名的那一行是不是真要跟着改，我没逐行判。
- 3.4 的签名是按函数名 `grep -A6` 粗筛的，多行签名会漏。
- 没读第一轮正推腿的报告；第一轮攻方腿的报告也没读，那里「24 个名字」那一句我没重数。

## 要主 agent 跑的命令

量 2.4 用：今天测试包里参照一侧、GPU 一侧各一组的闭包里，有几个单元的文件在验证包里。有，就说明搬进产品包之后这几组键的成员会变。三条依次跑，第二、三条要第一条的 `build=`。

1. 编 `unit-graph`
   - 命令：`task-server/task submit -- cargo build --release -p singlefs-unit-graph --bin unit-graph`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看什么：末行 `TASK_SERVER_RESULT` 的 `type=succeeded`、`exit=0`；记下 `task wait` 末行的 `build=` 目录，下面两条里的 `<build= 目录>` 照它填。
2. 参照一侧那一组（活槽核的 CPU 参照）的闭包
   - 命令：`task-server/task submit -- <build= 目录>/target/release/unit-graph closure /home/fy5090/code/singlefs singlefs_background_gpu_tests::reference_bytes_of_the_live_slot_pairing_input --target test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference --cfg 'feature="tests-on-a-real-gpu"' --no-default-features --toolchain-of-the-build release --lock-file /home/fy5090/code/singlefs/Cargo.lock --judgment-kind product-behavior`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看什么：`TASK_SERVER_RESULT` 的 `exit=0`（退 3 是闭包里有拒算，照样列单元行，也能读）。数 `task-server/task log <号> --all` 里含 `<TAB>crates/singlefs-verification/` 的单元行：大于 0，2.4 成立；等于 0，参照一侧这一组搬包不会因此变成员。
3. GPU 一侧那一组（活槽核走读那一条用例的持卡判）的闭包
   - 命令：`task-server/task submit -- <build= 目录>/target/release/unit-graph closure /home/fy5090/code/singlefs singlefs_background_gpu_tests::judged_on_the_cards_that_the_kernel_walking_every_record_matches_the_cpu_reference_on_both_unit_areas --target test:live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference --cfg 'feature="tests-on-a-real-gpu"' --no-default-features --toolchain-of-the-build release --lock-file /home/fy5090/code/singlefs/Cargo.lock --judgment-kind product-behavior`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看什么：同第 2 条。我预期大于 0，因为持卡判经测试包共用模块用到了验证包的 `PairingJudged`、`PairingAgreement` 与领卡，这一点是推的。

## 没做什么

- 不判正推、辩方那几格（第 2、4、7、8、9 格），不替主 agent 采纳。
- 没写模型、没往统一任务服务交任务；判决要引的数一个都没复跑，上一节三条交主 agent。
- 2.2、2.5、3.2、4.1 里提的改法（实验包对拍参照同源那一格、实验包测试由门禁哪一格起、乙出局、E202 里产品对拍测试拆出去），都只在读码上推过，被攻过零轮。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/gpu-tests-placement-design-r2-attack/` 里只有两份 `grep` 中间清单（`common-pub-names.txt`、`lib-entries.txt`），没建仓副本、没建编译目录，没有要删的。
