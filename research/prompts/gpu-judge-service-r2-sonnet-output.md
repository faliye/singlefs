# gpu-judge-service-r2：辩方腿报告（2026-11-17）

被复核的判决：`research/prompts/gpu-judge-service-r1-main-verification.md`「二、判决」表与「三、改法」六条。方案按 `crates/` 今天的实现谈；行号都是主工作区此刻的文件（`grep -nF` 逐句现查过）。本报告没写模型，没有复跑命令与 sha256；读到的数都是只读命令读出来的。

一处要交代：我为了找一个函数名的出处，在仓根跑了一次带 `research/` 的递归 grep，输出里带出了禁读清单里 `gpu-judge-service-r1-opus-output.md` 的一行。我没有读那份文件，那一行也没有进任何结论；下面凡是涉及「别的包已经在算键」的，都是我另从 `crates/` 里读到的。

## 一、总表

「判」一列：格 2、3、10 答「够得着吗」，格 4、7、8 答「判对 / 判宽 / 判窄」。「同样打中替代方案吗」只对打中的三格问。

| 判决格 | 辩方判定 | 同样打中替代方案吗 | 一句话依据 |
|---|---|---|---|
| 格 2 认领与不重判（判了打中） | 够得着一半：打中的是认领接口今天的行为，不是今天的判块路径；判宽 | 否：认领键用一把不是数据行的键就绕开第一处；按块号静态分片两处都不碰 | 今天的判块路径一次认领都不发，认领只有判定向量编号登记一处在用 |
| 格 3 版本对账（判了打中） | 「服务拆不开」够不着；「红整份换跨判法」够得着但是今天本来的设计、不是服务引进的；先决条件能回答一半 | 「换」的那条同样打中所有设计；「拆不开」不打中任何一种 | 两组键并成一把的函数现成、两组各自取得出，且已有别的包照同一套算键的先例 |
| 格 10 不换判据（判了打中两处线索） | 第一处（旋钮不在打包判表里）事实对，但只打中「清单行只带判表摘要」这一种形状；第二处（对账口径与待判口径不同）够得着今天的代码 | 第一处否（行里带录入表的键、服务读录入表就没有）；第二处同样打中任何把「判完」压成一位的设计 | 旋钮跟着录进库的那张表走；今天的代码里「判过」就有两个口径 |
| 格 4 批大小（判了成立） | 判对，但判宽了一点 | | 式子与今天的代码一致；漏了一条封顶与时间上限，漏了「式子所在的函数在判器组键里」 |
| 格 7 与统一任务服务的分工（判了成立） | 判宽：不只一处推不出 | | 清场不杀进程、不认识常驻服务；任务服务的要卡认法与记账都按任务来 |
| 格 8 多机（判了成立） | 判对，补两处没写的 | | 队列放在判定库服务里站得住；每台一个进程的控制命令与库里状态翻转的写量没覆盖 |

## 二、改法六条的辩护一览

| 改法 | 判 | 要点 |
|---|---|---|
| 1 待判清单 | 站得住，但「判完」不能是一位；缺两个状态 | 「判完」要定义成今天 `is_stored_for_reuse` 的函数，否则换了判据；状态里缺「判不了」与「卡 OOM 交回」 |
| 2 写结论那一帧里核认领、存块比较后交换 | 站得住，是根上的修法 | 四个落笔条件：核的位置在去重之后、核的是「别人持着且租期没到」不是「必须持着」、比较的对象是写的一方比过的那一份、一份结论多行要选一行当守卫 |
| 3 版本拆开核、判它的二进制指纹进比较 | 站得住，但「进比较」只能是准入检查，不能走块属性相等 | 块属性（在哪判的）今天明文不参与相同与否，改成相等会翻转既有规则 |
| 4 旋钮进清单行 | 结果对、形状不好 | 旋钮已经在录进库的那张表里；行里再抄一份就有两处能不一致，应让行指向录入表并让服务用块键核 |
| 5 强制现判换做法 | 轮次加凭据行站得住；CPU 参照复判改的是判据；有第三种 | 今天的哨兵判据本来就是「这个进程交来存过没有」，凭据行是同一个判据搬到库里 |
| 6 批大小的式子 | 式子站得住，但「改式子」本身会换判器组键 | 批宽相关的函数与常量今天就在 GPU 判器组的闭包里（编出来的表为证），改一个字全库换版本；这是六条里唯一一条在判器组键上越界的 |

后面各节逐格给文件:行。

## 三、格 2 认领与不重判

判决的两处读码：写帧释放认领不核持有者、认领回「已有结论」只看键在不在。两处现读都属实，行号略挪：释放在「self.release_claim_on(&mut group.batch, entry.key);」（`prune-server/src/committer.rs:451`），已有结论在「match self.shared.library.contains_key(key.key) {」（`prune-server/src/committer.rs:488`）与「verdicts.push(ClaimVerdict::AlreadyConcluded);」（`prune-server/src/committer.rs:490`）。`contains_key` 查的就是被认领的那把键本身（`prune-server/src/storage.rs:500`），认领不分表（`prune-server/src/storage.rs:296` 的 `claim_key` 文档）。

够得着多少，分四点：

1. 今天的判块路径一次认领都不发。`crates/` 里调认领的只有判定向量编号登记：封装在「let claimed = self.lock_the_reliable_client().claim(」（`crates/singlefs-verification/src/verdict_store.rs:2985`），唯一的调用点是「match authority.claim(&number_key.0)? {」（`crates/singlefs-verification/src/verdict_store.rs:5458`）。块的复用走「pub(crate) fn pending_blocks(」（`crates/singlefs-verification/src/crash_amplification.rs:3727`）里的 `is_stored_for_reuse`，不经认领。所以「红块与强制现判的块认领不到」「租期过后两份互盖」是把块队列架在认领上才有的后果，不是今天代码的行为。判决写「打中」，打中的是认领接口，不是判块。
2. 第一处（已有结论）只打中「被认领的键就是数据行的键」这一种架法。认领的键换成一把永远不当数据行写的租约键，`contains_key` 永远答没有，红块与强制块都认领得到。判决里说正推腿的「认领键改用判过记号」绕得开第一处，这个读法对；这个记号要做到「在 = 可复用」：写结论的同一批里写红时撤记号，才与 `is_stored_for_reuse` 的口径一致（见格 10）。
3. 第二处（写帧不核持有者）今天无害的理由写在代码里：登记编号的认领租期是「const PRUNE_SERVER_NUMBER_CLAIM_LEASE_MILLISECONDS: u32 = 30_000;」（`crates/singlefs-verification/src/verdict_store.rs:2395`），认领到手到写完是毫秒级。拿它当判几分钟的队列租约，这个窗口才真实存在。所以第二处对「长租约队列」成立，对今天的用法不成立。
4. 「租期过后两份都绿而不同、后写盖先写」只在读与写之间被别的客户端插进来时才发生。库里已有一份、两份不同、都不红时，存块直接报错：「fn stored_conclusion_compared_with(」（`crates/singlefs-verification/src/verdict_store.rs:4843`）返回「VerdictStoreError::DifferentConclusionUnderAStoredKeyNotDecidedInTheFirstVersion {」（`crates/singlefs-verification/src/verdict_store.rs:4855`）；要盖，只有读的那一步「let stored = self.read_the_stored_conclusion_block_rereading_after_a_side_row(」（`crates/singlefs-verification/src/verdict_store.rs:4709`）与写的那一步「self.write_batch_keeping_the_kept_out_rows_in_this_process(batch)?;」（`crates/singlefs-verification/src/verdict_store.rs:4688`）之间有第二个写者。这是先查后写的竞态，不是认领的洞；比较后交换补的正是这个。

同样打中所有替代方案吗：不。至少有三种架法不吃第一处，有一种两处都不吃。

| 架法 | 吃第一处 | 吃第二处 | 备注 |
|---|---|---|---|
| 认领键 = 结论块的键 | 是 | 是 | 判决说的那种 |
| 认领键 = 另设的租约键或判过记号 | 否 | 是 | 记号要与「不红才可复用」同口径 |
| 待判行自己带状态，用比较后交换翻状态 | 不适用（不用认领） | 不适用 | 就是改法 2 的后半 |
| 按块号静态分给几方，不商量 | 否 | 否 | 今天的做法，下面第一条 |

今天的做法：块怎么分给几方，「第 k 块归哪一方只由权重表与 k 定，几方各算各的、不用商量。」（`crates/singlefs-verification/src/crash_amplification.rs:1386`）；给定状态表那一路「一张表一块、按块键的输入指纹归一台」（`crates/singlefs-verification/src/crash_amplification.rs:19706`）。这一种从根上没有两个写者抢同一块，所以与认领的两处洞无关；代价是一台挂了它那份没人接（今天一台挂了整趟也停）、权重是静态的。判决没把它列进替代方案；它是「每台一个服务进程、队列放进程里、两台按权重分」，与格 8 并存。

判定：判宽。打中成立的范围是「拿认领当块队列」这一种设计，不是今天的判块路径，也不是所有设计。

会推翻这条的现象：在 `crates/` 里找到一处对块键或结论键调认领的现行代码（我 grep 到的认领调用点只有 `verdict_store.rs` 的上面两处）；或有人给出一个必须跨台动态接手、静态分片做不到的需求（用户原话里没有）。

### 改法 2 的辩护（并入格 2）

站得住，是第二处的根上的修法。落笔时守四条，都有今天的代码为证：

1. 核的位置在去重之后。写帧先查「已生效的请求号」，见到就回「早已生效」直接返回：「let earlier = match group.applied_in_group.get(&request) {」（`prune-server/src/committer.rs:383`），真正入批在「.add_write(&mut group.batch, client_identity, request_number, entries);」（`prune-server/src/committer.rs:448`）。比较后交换放在入批之前、去重之后；放在去重之前，重发一帧已经落下的写会因为键上已经是它自己写的内容而被判「对不上」。
2. 核的是「别人持着且租期没到就拒」，不是「必须持着才许写」。今天所有块与录入的写都不认领，只核持有者的话，这些写全拒。
3. 比较的对象是写的一方比过的那一份。存块的决定（红整份换、相同不写、不同报错）建立在读到的 `stored` 上（`verdict_store.rs:4709`、`verdict_store.rs:4714`），期望值取这一份的摘要才与它同口径。
4. 一份结论是多行（块行、违例行、计数向量、块属性、恢复字，旁表先于结论块写：「先在同一把键下写它，再写结论块」，`crates/singlefs-verification/src/crash_amplification.rs:2068`），守卫放在最后落的那一行（结论块）上，才不破「结论块在，旁表就在」的顺序。

不换判据：判定字与今天逐态相同，改的只是写入的原子性。会推翻的现象：比较后交换让「库里本来相同、重写应当一行不写」的路径多写或报拒（相同路径在 `crates/singlefs-verification/src/verdict_store.rs:4684`）。

## 四、格 3 版本对账

判决的话分三句，逐句答。

### 「结论版本是 CPU 二进制里两组键并成的一把，服务拆不开核」：够不着

- 并成的函数是公开的纯函数：「pub fn judgement_units_key_with_the_gpu_judge(」（`crates/singlefs-verification/src/crash_identity.rs:825`），指纹是「message.extend_from_slice(definition_units.fingerprint());」（`crates/singlefs-verification/src/crash_identity.rs:831`）起头、接上 GPU 判器那一把的指纹后取 SHA-256；组成清单是两组的单元各带组名起头。入参只有两把组键。
- 两把组键各自经同一个入口取，不必成对：「|| gpu_judge_units_of_this_build(toolchain),」（`crates/singlefs-verification/src/crash_amplification.rs:21266`）取 GPU 判器那一把，判定定义那一把另取（`crates/singlefs-verification/src/crash_amplification.rs:21281`）。CPU 路线、GPU 路线同取这两把：「GPU 判的与 CPU 判的都记在这一把下」（`crates/singlefs-verification/src/crash_amplification.rs:21237`）。
- 「别的包的构建脚本照同一套」写下的键表，已有函数按组名取键：「pub fn judging_units_key_of_the_group_in_the_table_text(」（`crates/singlefs-verification/src/crash_identity.rs:792`）。用它的是产品 GPU 包的测试包，它自己的构建脚本也算判器组键（`crates/singlefs-background-gpu-tests/build.rs:1-6`），并且对拍结论本来就按「参照一侧的单元键、GPU 一侧的单元键」两把分开记（`crates/singlefs-background-gpu-tests/tests/common_pairing_input_through_the_library/mod.rs:6-8`）。所以「拆成各组键由服务自己核」不是新发明：行里写判定定义那一把，服务取自己的 GPU 判器那一把，用 `judgement_units_key_with_the_gpu_judge` 并出来比。
- 要补一句：并成的那一把只是哈希，库里没法从它倒推两把。要拆，待判行得带判定定义那一把的指纹。这是改法 3 已经说了的（「拆成服务自己核得了的各组键」），判决表里「服务拆不开」写成了今天的事实，写宽了。

### 「红整份换让一种判法的结论换掉另一种」：够得着，但不是服务引进的

机制现读属实：换不换只看「不同」与「有一份红」：「if !stored.is_the_same_conclusion_as(requested)」（`crates/singlefs-verification/src/verdict_store.rs:4716`）；在哪判的写在块属性里，不参与：「只有在哪判的不同也算相同，留先存的那一份块属性」（`crates/singlefs-verification/src/verdict_store.rs:4693`）、「if stored.judged_on != requested.judged_on {」（`crates/singlefs-verification/src/verdict_store.rs:4870`）。

CPU 判与 GPU 判落在同一把版本键下是今天设计好的（`crash_amplification.rs:21237` 与 `crash_identity.rs:825` 前的文档）；判决说「取决于两份结论是不是落在同一把键下，没核」，答案是：是，本来就是。但今天流水线那一路里，GPU 判不了的状态是「整趟停下、报错，不退回 CPU 判器。」（`crates/singlefs-verification/src/crash_amplification.rs:3820`），没有 CPU 与 GPU 判同一块的路径；两种判法只在不同趟、配置换了开关时才碰同一把键。服务把判块全放到卡上，这个交叠只会更少。所以这一句打中的是「CPU 与 GPU 结论共享版本键、有红整份换」这条今天的规矩，同样打中所有设计，且没有因服务变坏。

### 先决条件「测试二进制与服务二进制编出的键相不相等」：能回答一半

键是库自己的构建脚本在编译时算好、编进库里的（`crates/singlefs-verification/build.rs:1-6` 与 `judging_units_key` 读编进去的表，`crates/singlefs-verification/src/crash_identity.rs:749`）。服务若链接同一个库，取到的是这个库那一次编译的键，与服务自己的 `main` 无关；等不等看库两次编译的输入一不一样：工具链行（含 RUSTFLAGS、profile、目标）、按闭包收窄的 `Cargo.lock`、依赖包被并上的特性（`crates/singlefs-unit-graph/src/entry_groups.rs:4-6`）。

只读的量（两份已有的编译输出，不是同一棵树同一刻的两次编）：

```
A=target/release/build/singlefs-verification-74353dc820e1d6ff/out/judging_units_keys.tsv
B=target/x86_64-unknown-linux-gnu/release/build/singlefs-verification-b7d0b8c60bb6e05d/out/judging_units_keys.tsv
sha256sum $A $B
1275bd77e0e7bb302dfa3631cde21a253df36177fc75317eafbac009e4fc1559  A
a21f1c261968d897555acad8876b88fe86b69ac8563e801d471eef8a5b73e761  B
diff <(sed -n 1,18p $A) <(sed -n 1,18p $B)
11c11
< toolchain	rustflags: 
> toolchain	rustflags: -C target-feature=+crt-static -L /usr/lib/gcc/x86_64-linux-gnu/13 -L /usr/lib/gcc/x86_64-linux-gnu/14 -l static=stdc++ -C link-arg=-fuse-ld=bfd
sed -n 19p $A | cut -f1-4   # group gpu_judge keyed  c408bb62…
sed -n 19p $B | cut -f1-4   # group gpu_judge keyed  bb6025df…
```

两份的 GPU 判器那一组键不同，工具链头里 `rustflags` 一行也不同。但这两份是 11-16 与 11-17 两天的编，源码也变过：逐单元比（`awk -F'\t' '$1=="unit" && $2=="gpu_judge"{print $4"\t"$6}'` 取名与指纹再 `diff`），2886 与 2887 个单元里只有 7 行不同，全在 `crash_judge_dispatch` 的批宽调度（`DispatchWidthSchedule`、`starting_narrow`、新增 `IndivisibleFloorWatch`、`use@` 行号移位）。所以这对数据证明「源码一变，键就变」，不能把键的差异归到 RUSTFLAGS；归因要同一棵树只改 RUSTFLAGS 编两次（要主 agent 跑，见第九节）。

对判决的意义：键不等的后果方向是安全的。待判行里写的并成键与服务自己算的对不上，服务拒绝写、不会写错键；取不到键时今天的路径是照判不入库（占位键，`crates/singlefs-verification/src/crash_amplification.rs:21096`）。最坏是卡在「一直对不上、一直重排」的活性问题，不是静默的错账。服务的 RUSTFLAGS、profile、`Cargo.lock` 与特性取法得与 CPU 一侧的构建同一套；放进同一个工作区（共享根目录的 `Cargo.lock` 与 `[profile]`）比像 `prune-server` 那样自成工作区（`Cargo.toml:3-4` 的注释写明它自己有空的 `[workspace]`）更容易对上。这一点是推的，没量。

### 「内核改了之后旧服务还在跑，有没有一条路径把旧判法的结论记在新键下」

今天没有，靠的是构造而不是检查：判的进程自己算版本键，版本键与块属性里的判器指纹同取自这一个进程：「gpu_judge_units_fingerprint: gpu_judge_units_fingerprint_of_this_process(),」（`crates/singlefs-verification/src/crash_amplification.rs:3800`）与版本键的取法（`crash_amplification.rs:21266`）。库里没有任何读者比块属性里的指纹与版本键（`gpu_judge_units_fingerprint` 在 `verdict_store.rs` 里只有编码、解码与测试构造，行 847、860、865、878）。进程一拆，这个由构造保证的等式就没了：CPU 一侧把新键写进待判行，旧服务照行里写的键存旧判法的结论。所以改法 3 必须有写入准入检查（服务自己算的 GPU 判器那一把与行里写的并成再比，不等就拒写、整行标判不了）。这一条判决写对了。

判定：格 3 判宽一半。「服务拆不开」够不着，「红整份换」打中所有设计、不因服务变坏，「旧服务把旧结论记在新键下」成立且正是改法 3 要堵的。

会推翻的现象：服务按同一套构建输入编出的 GPU 判器那一组键与 CPU 一侧不等（见第九节命令）；或在 `crates/` 里找到一处读块属性里的指纹去比版本键的代码（我只看到构造与序列化）。

### 改法 3 的辩护（并入格 3）

站得住，限定一条：「判它的那份二进制的指纹进比较」只能做成写入准入检查，不能做成存块比较里的块属性相等。今天块属性里的 `judged_on` 明文不参与是不是同一份结论的判断（`verdict_store.rs:4693`、`verdict_store.rs:4870`），且它含卡型与驱动版本；若把指纹放进相等比较，同一结论在 5090 与 5060 Ti 上判出来就从「已存相同」变成「不同」，进「红整份换」或「报错」。CPU 判的块属性里还没有 GPU 判器指纹。做成准入检查（写的一方的键 = 行里的键）就不动这条规矩。

补记（格 3，更正上面「GPU 判不了就整趟停下」那一句的范围）：那一句只对流水线（崩溃放量）那一路成立。给定状态表那一路不同：GPU 判不了的表「照 H6 记一条不一致、判红，这几个状态的结论取 CPU 判器的」（`crates/singlefs-verification/src/crash_amplification.rs:19709`），那一路里一张表上两种判法的结论确实会并存。服务若也接给定状态表，这一点要在待判行里有位置（见改法 1）。

## 五、格 10 不换判据，与改法 4

判决记了两处线索、都说没核。我去读了。

### 线索一「旋钮不在打包判表里，清单只带判表摘要时服务会照产品旋钮判」

事实对，形状窄。

- 打包判表里确实没有旋钮：「pub struct PackedJudgeTables {」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:380`），字段只有 `words` 与 `state_count`（`crash_judge_dispatch.rs:381-382`）。旋钮值也不进判器组键：「旋钮值不进判法单元的键」（`crates/singlefs-verification/src/crash_judge_gpu.rs:1159`）；进键的是旋钮常量的写法，入口登记里 `of_the_judge_given_the_container_capacity_knobs` 是组的入口：「"singlefs_verification::crash_judge_gpu::JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs",」（`crates/singlefs-verification/src/judging_units_entries.rs:58`）。
- 但旋钮今天不是丢的：它跟着给定状态表录进了判定库。「pub struct RecordedGivenStatesTableWithItsColumns {」（`crates/singlefs-verification/src/crash_amplification.rs:6266`）里就有「pub container_capacity_knobs: ContainerCapacityKnobs,」（`crates/singlefs-verification/src/crash_amplification.rs:6268`），写进库的字节头 `SFGSCOL1` 后紧跟三棵树的每片记录数（`crash_amplification.rs:6274`、`crash_amplification.rs:6374-6391`）；文档写明「两栏都跟着表录进判定库、进块键」（`crates/singlefs-verification/src/crash_given_states.rs:116`）。判段读回之后照这一栏编内核：「let kernel_sources = JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs(」（`crates/singlefs-verification/src/crash_amplification.rs:20449`，旋钮取自 `20450`）。
- 流水线（崩溃放量）那一路根本没有旋钮：「崩溃点重放与放量走旧入口」，编的是三棵容器树都没装旋钮的那一份（`crates/singlefs-verification/src/crash_judge_gpu.rs:230`）。旋钮只在给定状态表那一路有。

所以线索一打中的是「待判行只带判表摘要、服务不读录入表」这一种形状。用户定的前提是服务从库里读录入的东西（正文「已定的前提」第 2 条），照这个前提服务读的是录入表本身，旋钮就在里面。打不中「行里带录入表的键」这一类设计，也打不中今天的代码。判宽。

另一件判决与线索都没说到：给定状态表那一路今天的判表是判段现建的（「判表先入库、再读库，GPU 只拿读回的那一份算；打包在锁外」，`crates/singlefs-verification/src/crash_amplification.rs:20501`），不是录入段先打好的。流水线那一路才是录入段打包、写进库（「录入段打包一次、写进库，判段从库里读回」，`crates/singlefs-verification/src/crash_judge_dispatch.rs:378`）。所以给定状态表那一路根本没有「判表摘要」可写进待判行；服务要么自己建表（CPU 活挪进服务），要么 CPU 一侧在录入段先建好打好。这是改法 1、4 要先定的事，判决没提。

### 线索二「对账口径与待判口径不同，服务漏判的红块与强制块会被当成判过」

够得着今天的代码，判对。「判过」今天就有两个口径：

- 复用口径：「pub(crate) fn is_stored_for_reuse(」（`crates/singlefs-verification/src/crash_amplification.rs:1950`）= 不在强制现判表里、结论与逐盘槽数都在、「&& !self.is_red(store, point_index, key)?)」（`crates/singlefs-verification/src/crash_amplification.rs:1960`）。红的不算。
- 续点口径：「pub(crate) fn is_judged_with_the_slots(」（`crates/singlefs-verification/src/crash_amplification.rs:1966`）= 结论在、槽数在，红块也算判过（文档在 `crash_amplification.rs:1949`）。
- 强制现判表与「交来存过」表都是进程内的：「conclusion_blocks_forced_to_be_judged_again: BTreeSet<ConclusionOfABlockUnderAKey>,」（`crates/singlefs-verification/src/verdict_store.rs:3095`）、「conclusion_blocks_offered_in_this_process: Vec<ConclusionOfABlockUnderAKey>,」（`crates/singlefs-verification/src/verdict_store.rs:3091`）。服务是另一个进程，看不到它们。

同样打中任何把「判完」压成一位的设计（包括正推腿的判过记号）。不打中「判完 = 今天的两个函数在库上现算」的设计。判对，范围是「压成一位」。

会推翻的现象：找到今天某个读者只用一个口径就够（续点与复用合一）；或强制现判表在库里有一份落地（我读到的只在 `VerdictStore` 里）。

### 改法 4 的辩护

结果对（服务编内核必须用这张表的旋钮），形状不好。两个理由：

1. 旋钮同时在录入表字节里和块键输入里（`crash_amplification.rs:6440` 注明「录进库的那一栏与块键都照这一种写法」）。待判行再抄一份，就有三处能不一致，其中块键是哈希、倒推不出旋钮。行里改成带「录入表的键」，服务读回录入表、取旋钮、再用同一套算法算块键与行里的块键比，旋钮错了块键对不上，当场拒；比「行里抄一份」强一级，因为错的旋钮在写库之前就被拦下。
2. 每个不同的旋钮组合要编不同的内核源码：管线按三份源码合起来的摘要缓存（`fn kernel_sources_digest(kernel_sources: &JudgeKernelSources) -> [u8; DIGEST_BYTES] {`，`crates/singlefs-verification/src/crash_judge_dispatch.rs:517`），同一张卡上同一套源码只编一次。常驻服务里每多一种旋钮组合多编一套管线、多占卡上固定那一份，今天的 2560 MiB 留量（`crash_judge_dispatch.rs:36`）里算没算这些没量。冷编在慢机上很重（`crash_judge_dispatch.rs:607-611` 的注释说一台机器 48G 编不完展开过大的内核），所以改法 4 要配一条：旋钮组合的种数有限、先量再放行。

不换判据：判定字取决于旋钮值，行里写错才会换。上面第一条就是堵它。

判定（格 10）：线索一判宽，线索二判对。

## 六、格 4 批大小与显存，与改法 6

### 格 4：式子判对，说宽了三处

今天的式子在 `crash_judge_dispatch.rs`：`states_in_flight_within`（`crates/singlefs-verification/src/crash_judge_dispatch.rs:479-492`）取「卡的显存额度减判表减留量」，交给 `states_per_dispatch`（`crash_judge_dispatch.rs:594-605`）除以每态草稿区字节数，再与草稿缓冲条数的封顶取小。留量常量「pub const MEBIBYTES_LEFT_FOR_THE_DRIVER: u64 = 2560;」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:36`）。

我自己代入（算术，推的；每态 `SCRATCH_WORDS × 4 = 8 038 396` 字节，`crates/singlefs-verification/src/crash_judge_gpu.rs:988`，表取 0 作上界）：32G 卡 (32768−2560) MiB ÷ 8 038 396 = 3940 态，16G 卡 1803 态。判决里的 3848 与 1727 是再减了判表以后的数，方向一致、量到的 3843 与 1600–1730 在其下。判对。

说宽了的三处：

1. 批宽不只受显存管。草稿缓冲条数封顶：「states_per_scratch_lane(maximum_storage_buffer_binding_size) * MAXIMUM_SCRATCH_LANES;」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:603`）与「pub const MAXIMUM_SCRATCH_LANES: usize = 32;」（`crates/singlefs-verification/src/crash_judge_dispatch.rs:33`）。今天一条缓冲装 267 态，封顶 8544 态，没压过显存那一条。草稿区按写到的字缩到 322 655 字之后（前提 6 的新量到的数），一条缓冲装 1663 态、封顶 53 216 态，16G 卡的显存上界 11 231 态（我的算术）；封顶还是不压显存，但「压不压」要现算，不是式子里自带的。
2. 批宽还受时间管：一次提交最久 0.5 秒（`crash_judge_dispatch.rs:42`），驱动会掐的线 2 秒（`crash_judge_dispatch.rs:47`），实际一次派活带几态由 `DispatchWidthSchedule` 按最久那次提交加宽或减半（`crash_judge_dispatch.rs:1653`、`crash_judge_dispatch.rs:1678`）。这一条今天对快流不起作用（文件头量到的一次 13 020 态每次 38 毫秒，`crash_judge_dispatch.rs:9-10`），但同卡交替派两条流时一条流的提交会排在另一条后面，加宽与减半的判据吃到对方的等待时间，式子没有这一项。
3. 交替派的多条流，每个判器各开一份草稿缓冲与判表（`scratch_lanes: Vec<wgpu::Buffer>`，`crates/singlefs-verification/src/crash_judge_dispatch.rs:547`；判表 `tables: wgpu::Buffer`，`crash_judge_dispatch.rs:545`）。几条流合计占的是 Σ判表 + Σ草稿区，批宽要从总额度里按条分，「同卡同进程放几条流合计几乎不变」成立的前提是按条分配后每条分到的是 1/条数。判决里「窄的是每条流的批宽」对，但式子要写成按条数除。

不判错的理由：这三点都是式子外面的封顶，量到的数与式子相符。判窄不对，判宽一点。

### 改法 6：式子站得住；改它本身换判器组键，是六条里唯一越界的一条

「批宽不进判器组键」是 `judging_units_entries.rs` 的文档说的：「切法不进结论，不登进来」（`crates/singlefs-verification/src/judging_units_entries.rs:29`）。但组键是按入口算调用闭包、再把闭包里的单元并起来，不是看文档。入口里有「GpuCrashJudge::on_card_compiling_these_kernel_sources」（`judging_units_entries.rs:59`），这个构造函数里就调批宽调度与式子。编出来的表怎么说，读两份已有的编译输出（只读，行号对得上今天的源码）：

```
B=target/x86_64-unknown-linux-gnu/release/build/singlefs-verification-b7d0b8c60bb6e05d/out/judging_units_keys.tsv
awk -F'\t' '$1=="unit" && $2=="gpu_judge" && ($4 ~ /::states_per_dispatch$|::states_in_flight_within$|::states_per_scratch_lane$|MEBIBYTES_LEFT_FOR_THE_DRIVER$|MAXIMUM_SCRATCH_LANES$|::starting_narrow$|LONGEST_DISPATCH_SECONDS$|kernel_sources_digest$/){print $3"\t"$4"\t"$5}' $B
const  ...crash_judge_dispatch::MAXIMUM_SCRATCH_LANES                      crates/singlefs-verification/src/crash_judge_dispatch.rs:33-33
const  ...crash_judge_dispatch::MEBIBYTES_LEFT_FOR_THE_DRIVER              crates/singlefs-verification/src/crash_judge_dispatch.rs:36-36
const  ...crash_judge_dispatch::LONGEST_DISPATCH_SECONDS                   crates/singlefs-verification/src/crash_judge_dispatch.rs:42-42
fn     ...crash_judge_dispatch::states_in_flight_within                    crates/singlefs-verification/src/crash_judge_dispatch.rs:479-492
fn     ...crash_judge_dispatch::kernel_sources_digest                      crates/singlefs-verification/src/crash_judge_dispatch.rs:517-528
fn     ...crash_judge_dispatch::states_per_scratch_lane                    crates/singlefs-verification/src/crash_judge_dispatch.rs:585-591
fn     ...crash_judge_dispatch::states_per_dispatch                        crates/singlefs-verification/src/crash_judge_dispatch.rs:594-605
fn     ...crash_judge_dispatch::<impl DispatchWidthSchedule>::starting_narrow  crates/singlefs-verification/src/crash_judge_dispatch.rs:1679-1693
```

（输出里 `...` 是我省掉的 `singlefs_verification::` 前缀，其余原样；制表符换成了空格。）这八个单元在 GPU 判器那一组的组成清单里，行区间与今天源码里这些函数的位置吻合。唯一被明文排除的是显存记账里的「一次派活带几个状态的封顶」那一个：`states_in_one_dispatch_capped_by_the_state_count`（`crates/singlefs-verification/src/judging_units_entries.rs:884`），不是上面这几个。前面第四节那两份编出来的表也印证：两天之间只改了批宽调度（`DispatchWidthSchedule`、`starting_narrow`、新增 `IndivisibleFloorWatch`），GPU 判器那一组键就变了。

后果：照判决「三、改法」第 6 条改批大小的式子，要改的函数在这一组的闭包里，改一个字，GPU 判器那一组键变，并成的版本键变，全库（CPU 判的与 GPU 判的）的块都换版本、全部现判。常驻服务要按新卡调批宽，每调一次式子重判整库，不能接受。这是「换了不该换的判器组键」。出路是把这几个函数与常量放进「不沿它收」的名单（同 `judging_units_entries.rs:840` 的显存记账那一份做法，加对应的登记与红绿测试），或把式子搬出判器库、由服务算好批宽作参数传进去；两种都要在落笔时一次性换一回键，之后调式子不再换。这是推的，没做实验：证实它要在副本里改一个常量重算组键（第九节）。

同一段里，前提 6 的「草稿区按每条流实际写到的字定」比改式子更重：每态草稿区字数是内核源码里的一个常量，「let declaration = format!("const SCRATCH_END: u32 = {SCRATCH_WORDS}u;");」（`crates/singlefs-verification/src/crash_judge_gpu.rs:1819`），派活里每个状态的草稿区在缓冲的第 k 乘每态字数个字起（`crash_judge_gpu.rs:997`）；字数一变，内核源码与组键都变，而且「桶表整张写满」（背景材料第一轮之后新量到的）意味着只是缩每态长度不行，要重排桶表与记录表的布局；表越过内核容量的状态今天要么整趟停下（流水线，`crash_amplification.rs:3820`）要么判红（给定状态表，`crash_amplification.rs:19709`），容量一变，哪些状态落进「判不了」也变。这条属判决「三、改法」之外，我不替它辩护，只记「它会改判定字，不是批大小的式子」。

判定：改法 6 的式子辩护成立；落笔方式要先解决组键，否则越界。

会推翻的现象：在副本里改 `MEBIBYTES_LEFT_FOR_THE_DRIVER` 一个数、重编重算，GPU 判器那一组键不变（那就是我读表读错了：表里列的单元没参与哈希）。

补记（改法 6，调用边的落点）：构造函数 `on_card_compiling_these_kernel_sources` 在 `crates/singlefs-verification/src/crash_judge_dispatch.rs:989` 起，里面「states_in_flight_within(limit, card.memory_budget_mebibytes(), tables),」在第 1010 行，「dispatch_width_schedule: DispatchWidthSchedule::starting_narrow(」在第 1066 行。这两处是批宽式子与调度进到 GPU 判器组闭包的调用边，不是我猜的名字。

## 七、格 7 与统一任务服务的分工：判宽

判决写「清场、卡坏发现、起停归服务，卡按配置划界」成立，只有「服务自己的 CPU 怎么不被挤」推不出。读码看，还有至少三处没说：

1. 清场的行为是拒，不是杀，所以常驻服务压着卡时，每一次要这张卡的任务清场回读都不过。清场的注释写明：「清场不停、不杀任何进程，也不按模式找进程」（`scripts/crash/gpu-quiesce-run.sh:32`），这一趟要用的卡上有「这一趟要用的卡上有不归白名单、也不在别的活着的登记之下的」计算进程时回读不过、退 78（`scripts/crash/gpu-quiesce-run.sh:32-33`）。常驻服务不在清场命令白名单里，也不在任务服务起的任务登记里。所以「按配置划界」不是可选：要么划出的卡不在 `GPU_CARDS_BY_PRIORITY`（`multi-host.env.example:89`）与 `GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED`（`multi-host.env.example:92`）里，要么让服务在清场看得到的登记下面。这是失败方向安全（拒而不杀）的一点，判决没写出来。
2. 任务服务认「要卡」的名单里，崩溃放量入口是要卡的：「gpu-quiesce-run.sh、crash-amplification*.sh」（`task-server/task_server.py:605`）；按声明「all」要整台空卡（`task-server/task_server.py:618-620` 一带）。后半段拆走之后，崩溃放量的 CPU 一半不该再要卡、不该再清场，这张名单与声明的写法都要改；不改，放量任务仍排队等卡、与常驻服务争。这是「分工」要改的任务服务一侧，不止「卡划界」。
3. 任务服务给卡的记账只认它自己起的任务：「挑优先级最高、此刻没有服务起的任务占着、所在那台又放得下的一张」（`task-server/task_server.py:614`）。常驻服务占着的卡，它不知道；除非卡写在划界之外，它不会自己避开。这与第一点同一件事的两面：一个是清场拒，一个是记账看不见。

判宽的理由：判决把分工的缺口收成「一处」，实际是三处，其中前两处会让一批真卡测试与崩溃放量同时红。

会推翻的现象：常驻服务的进程被当作清场登记里的「活着的登记」认出来（那就是第一点不成立）；或崩溃放量脚本的要卡声明已经只管 GPU 一半（我读到的 605 行名单没分）。

## 八、格 8 多机：判对，补两处

判决：每台有卡的机器一个进程，队列在判定库服务里。

- 位置站得住。判定库只有一份，本机走 unix socket，第二台走 TCP（`prune-server/README.md:3`），两台的任务经它读写同一份库；第二台的服务要连的是本机的判定库，待判行放里面两台都读得到。
- 补一：控制命令不能抄判定库服务那一套。判定库服务的 `control` 是「派到哪台都只管服务那台的单元」（`prune-server/README.md:47`）：只有一个单元，不在那台就 ssh 转去。每台一个服务进程是两个单元，规则要求控制命令派到哪台都跑得起（`.claude/rules/verification.md:65`），所以 `control` 的 `install`、`start`、`stop`、`check` 要对每台各管各的，写法与现有那份不同。判决没写。
- 补二：状态翻转的写量没量。每块 待判、判着、判完 至少三次写，库里写一帧是组提交、同步一次 WAL（「几帧的写合进一个 WriteBatch、只同步一次 WAL」，`prune-server/src/committer.rs:4`）；块数大的流（几万块）每块三次、两台一起，库每秒能吃多少没量。这条放进格 8 的「推不出」，不放进「成立」。
- 补三：判定库那条 TCP「不认证、帧不带校验列」（`prune-server/README.md:86`），只开在有线直连段。常驻服务写的是判定结论，仍沿用这一条前提；若放到别的网段要重议。这是现状，不是缺陷。

判决「没被攻过」不等于没有缺口，但格 8 的方向没错。判对，补两处（补一、补二）。

会推翻的现象：量出状态翻转的写量远低于库的吞吐（补二落空）；或哪个机制让一台的服务死了而另一台自动接手（今天的静态分片没有，第三节的架法 4 也没有）。

## 九、改法 1 与改法 5 的辩护

### 改法 1 待判清单

方向站得住：今天「要判哪些块」是现算的，不是库里一份清单：「/// 这一趟要判的块（这一判法单元指纹下还没有判定块的、归自己的），按点与块序。」（`crates/singlefs-verification/src/crash_amplification.rs:3726`），由 `pending_blocks` 遍历计划的点与块、逐块问 `is_stored_for_reuse`（`crash_amplification.rs:3742`）。服务是另一个进程，进程内的强制现判表与「交来存过」表它看不到（`verdict_store.rs:3091`、`verdict_store.rs:3095`），所以要一张库里的表。

行的每个字段，今天的来源（读码，行号在 `pending_blocks` 一带）：

| 字段 | 今天从哪来 |
|---|---|
| 块在计划里的位置（点号、块内序号区间、全流块序） | `plan.points` 遍历与 `point.blocks()`，全流块序 `block_sequence`（`crash_amplification.rs:3734-3738`） |
| 块键 | `point.block_key(&local)`（`crash_amplification.rs:3742`） |
| 结论种类与版本键 | `judged_by.conclusion_kind()` 与每点一把版本（`ConclusionVersionsOfThePlan::judgement_of_each_point`，`crash_amplification.rs:21242`） |
| 判定定义那一把组键（拆开核要用） | `judging_units_key(&JudgingUnits::JudgementOfTheCrashImages, …)`（`crash_amplification.rs:21281`） |
| 流水线那一路的判表摘要 | 录入段打包写进库（`crates/singlefs-verification/src/crash_judge_dispatch.rs:378`） |
| 给定状态表那一路的录入表键 | 录入表跟旋钮栏一起录进库（`crash_amplification.rs:6266`）；判表在判段现建，没有录入期摘要（`crash_amplification.rs:20501`） |
| 是不是被抽中强制现判 | 进程内（`verdict_store.rs:3095`），要新写进行里 |
| 归哪一台 | 今天由权重与块序定，不用商量（`crash_amplification.rs:1386`），可以直接写进行里 |

两条落笔条件：

1. 「判完」不能是一位。它必须是今天两个函数在库上现算的结果：复用口径（红不算，强制不算）与续点口径（红算）。判决没有改的话，对账会把「服务漏判的红块」认成判过（格 10 的线索二）。做法：待判行只记「谁在判、判到哪」，「这条流判完了」由 CPU 一侧重新跑一遍 `pending_blocks`，为空才算。这样 `crates/` 里的判据没有换，只加了一张谁在判的账。
2. 状态不只三个。流水线那一路卡判不了某个状态时「整趟停下、报错，不退回 CPU 判器」（`crash_amplification.rs:3820`），要有「拒判」这个终态与原因；一张卡报 Out of Memory 时「判一块时报的那一块已经交回队里」（`crash_amplification.rs:3822`），要有判着回待判的转移；给定状态表那一路「GPU 判不了的表」记不一致判红、结论取 CPU 的（`crash_amplification.rs:19709`），要有一条路径让这些行回到 CPU 一侧。这三种今天在进程内的消息枚举里（`CardMessage`，`crash_amplification.rs:3817-3828`），拆成两个进程要变成行状态。

是不是换了判据：只要「判完」按上面第一条定义，没有换；定义成「行里的状态位 = 判完」就换了。

### 改法 5 强制现判与哨兵

先说今天的判据，因为两种做法差在它上面。哨兵判「抽中的块这一趟被现判了没有」只看这个进程交来存过的记录，不看库里的字节：「conclusion_blocks_offered_in_this_process: Vec<ConclusionOfABlockUnderAKey>,」（`crates/singlefs-verification/src/verdict_store.rs:3091`）的文档写明原因是现判出同一份结论时库里一个字节都不动（`verdict_store.rs:3089-3090`），相同时存块直接返回：「VerdictBlockStoring::AlreadyStoredIdentically => return Ok(storing),」（`crates/singlefs-verification/src/verdict_store.rs:4684`）。强制现判那一块判完与库里不同，报「判定不稳」「判法依赖了键之外的输入」（`crates/singlefs-verification/src/crash_amplification.rs:3632`）。所以哨兵管两件事：判了没有（活性），判出来的与存的同不同（键的完备性）。

- 轮次加凭据行：把「判了没有」从进程内搬进库里一行（轮次、块键、判它的指纹，每判一块都写，不论结论是否相同）。判据没变，只是记录的位置变了，服务的写量加一行。拦得住「说现判了其实读库」：凭据行是判的那一步写的，读库的路径不写。拦不住：服务把凭据行写了却没真上卡（它自己撒谎），这要靠它的代码路径可核，不靠凭据。
- CPU 参照复判：让强制块在 CPU 参照判器上也判一次比。拦得住上面的服务撒谎，也比的是结果。但它动「登记了的那几类只走加速路径」那一条（派发提示第 5 点已经问了有没有不动它的第三种），而且判据变成「GPU 与 CPU 一致」，对「判法依赖了键之外的输入」那条哨兵目的是另一回事。
- 第三种，不动那一条：强制块之外，每轮往流里插一块已知答案的合成块（种一个已知违例的状态，块键带轮次，库里不可能已有），收到的结论必须与已知答案一致才算服务真上卡判了。它拦「说现判了其实读库」（库里没有这一块，读不到），不用 CPU 判器；只证明服务这一轮真的在判，不证明某个强制块的键完备。它与凭据行互补：凭据行管「这一块判了没有」，已知答案块管「判器此刻能判对」。这是推的，没实验；要做的话先在假卡上做 A/B（`.claude/rules/three-way-inference.md` 要求零轮改法写进产品代码前先做探针）。

判定：轮次加凭据行站得住；CPU 参照复判改的是判据；第三种存在但只覆盖一半。

会推翻的现象：找到一处读者拿「库里这一块的字节变了」当现判过的依据（我读到的复用哨兵只看进程内交来存过的表）。

## 十、要主 agent 跑的命令

我没写模型，没起任何样本进程，没有探针占内存与时间；上面读到的数都是 `grep`、`awk`、`diff`、`python3` 算术的只读结果。下面三条是要把「推的」变成「量的」的，都不是判决要引的数的复跑，是我提出的新验证；写法没经项目代码闸与范围声明闸验过，交之前请照 `.claude/hooks/project-code-guard.sh` 与 `.claude/hooks/run-scope-guard.sh` 文件头补范围声明。

1. 重放格 6 那八个单元在组闭包里（只读，不经服务）。
   - 命令原样：`awk -F'\t' '$1=="unit" && $2=="gpu_judge" && ($4 ~ /::states_per_dispatch$|::states_in_flight_within$|MEBIBYTES_LEFT_FOR_THE_DRIVER$|MAXIMUM_SCRATCH_LANES$|::starting_narrow$/){print $3"\t"$4"\t"$5}' <最新的 judging_units_keys.tsv>`，`<最新的…>` 取 `target/**/build/singlefs-verification-*/out/` 下按修改时间最新那一份。
   - 在仓根起。
   - 看：五个单元都在、行区间与 `crash_judge_dispatch.rs` 里这些函数此刻的位置对得上，就是批宽函数在 GPU 判器组闭包里；缺任何一个，改法 6 的越界收窄到缺的那几个。

2. 证实改批宽常量会换 GPU 判器组键（A/B，一个数）。
   - 先 `python3 scripts/hosts/minimal-copy.py <仓根> ~/.local/share/singlefs/work/gpu-judge-service-r2-defense-copy --with-test-inputs` 建副本；在副本里把 `crates/singlefs-verification/src/crash_judge_dispatch.rs` 第 36 行的 2560 改成 2561；然后 `task-server/task submit --cwd ~/.local/share/singlefs/work/gpu-judge-service-r2-defense-copy -- cargo build -p singlefs-verification --release`（写法待核）。
   - 在副本根起。
   - 看：副本编出的 `judging_units_keys.tsv` 里 `grep -P '^group\tgpu_judge\t'` 那一行的指纹与原树编出的那一份比；变了，改法 6「改式子换组键」成立；没变，我读表读错了，这一条撤。
   - 用完删副本（交回前我没建，所以这里没有要删的）。

3. 回答格 3 的先决条件，库一级：同一份库源码、同一套 RUSTFLAGS，测试目标与库目标编出的 GPU 判器那一组键与判定定义那一组键相不相等。
   - 命令原样：`task-server/task submit -- cargo build -p singlefs-verification --release`（写法待核）。
   - 命令原样（与上一条同一棵树、同一个 RUSTFLAGS）：`task-server/task submit -- cargo test -p singlefs-verification --release --no-run`（写法待核）。
   - 在仓根起（或同一副本）。
   - 看：两次各自 `OUT_DIR` 下 `judging_units_keys.tsv` 里 `^group\tgpu_judge\t` 与 `^group\tjudgement_of_the_crash_images\t` 两行的指纹是否逐字相同。相同：同库的不同目标键相等，服务只要用同一个库、同一套构建输入就对得上；不同：判决那一句「先决条件」就升成真问题，服务不能拿自己编的库当对账的一方。再把 RUSTFLAGS 换一份重编一遍，看键变不变（归因到 RUSTFLAGS）。

## 十一、没做什么

- 不替主 agent撤销或维持判决，只交复核结果。
- 没往统一任务服务交任务，没复跑判决要引的数（判决引的 3848、1727、3843、1600–1730 这几个我只做了代入的算术，标「推的」；原始量在 `records/2026-11-17-崩溃判器与测试流的效率优化.md`，我没读）。
- 没读两份禁读文件，只是上面开头说的那一行误入输出；没读第一轮两条腿的报告，也就没核它们引的 `crash_amplification.rs` 其余行号，我引的行号都是此刻主工作区的文件。
- 没核崩溃放量入口脚本的文件头是否声明「all」张卡：第七节第二点我只读了任务服务的认法名单（`task_server.py:605`），没有读那份脚本自己对张数的声明；今天的脚本另有按需领卡一套（`scripts/crash/crash-amplification.sh:2171`），拆走之后怎么与服务衔接没读。
- 没量常驻服务占卡时的显存、冷编时间、状态翻转的写量、服务编出的键与 CPU 一侧是否相等；这几条都写成「推的」，命令在上一节。
- 没核第一轮判决里没点到的格（1、5、6、9）；改法 5 只对「说现判了其实读库」答了，没有评估强制现判在库里的具体存法。
- 没评估给定状态表那一路是否真要进服务（它的判表是判段现建、有 CPU 活，见第五节）；这是要用户或主 agent 定的范围，不是我的结论。

## 十二、判定一览（回报用）

- 格 2：判宽；打中的是认领接口今天的行为，不是今天的判块路径，也不打中静态分片与另设租约键的架法。
- 格 3：判宽一半；「服务拆不开」够不着；「红整份换」是今天的设计、打中所有设计；旧服务写旧结论的路径成立。
- 格 10：线索一判宽（只打中行里只带判表摘要这一种形状）；线索二判对（「判过」今天就有两个口径）。
- 格 4：判对，判宽一点（漏了条数封顶、时间上限、多条流按条分额度）。
- 格 7：判宽（清场拒而不杀、任务服务要卡的认法与记账、崩溃放量要卡的名单，三处都要改）。
- 格 8：判对，补控制命令与状态翻转写量两处。
- 改法 1 站得住，判完不能是一位、缺两个状态；2 站得住，四条落笔条件；3 站得住，只能做准入检查；4 结果对形状不好；5 凭据行站得住，CPU 参照复判改判据；6 式子站得住但改它会换判器组键，是六条里唯一越界的。
