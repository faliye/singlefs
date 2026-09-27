# E163 跑前登记：GPU多卡算单元校验和

写于 2026-09-27 13:47 JST，装置写之前。

## 一、问题

英文名：gpu_multicard_crc32c

主 agent 给的问题（逐字）：一批单元内容（每个 32768 字节）的 CRC32C 能不能在单机多卡、双机多卡上算出来，合并后与 CPU 参照逐个相同；比对本身会不会红。只问能不能跑、能不能用。

读法写死：
- 「单元内容的 CRC32C」= 对整 32768 字节求 CRC-32C，不置零任何字段；即 checker 判 I-2.1 时对被引单元整份求的那个量（`crates/singlefs-checker/src/image.rs:439`），不是头里 32 字节宽校验和那种「字段按 0 参与」的口径。
- 「卡」= wgpu 在 Vulkan 后端枚举出的、`vendor == 0x10DE` 且 `device_type == DiscreteGpu` 的适配器；软件适配器（llvmpipe 一类）与 CPU 类型不算卡。「起得来上下文」= 那张卡上 `request_device` 成功且第一块缓冲区分配成功。
- 「逐个相同」按全局单元号一一比，分母是派出的单元数 N，不按收到的条数算。
- 不停本地模型；起不来的卡报 wgpu 原样错误，不换配置重试。只问能不能，不设速度门槛，用时只报不判。

岔路单 `research/prompts/m2-gpu-multicard-r1-forks.md`（逐行抄）：

| # | 问题 | 候选（各自的定义） | 翻面观测 | 够判条件 | 状态 |
|---|---|---|---|---|---|
| M1 | 单机多卡：一批单元的校验和能不能切开分到本机几张卡上算、合并后与 CPU 逐个相同 | **能**：每张卡各算一片，合并结果与 CPU 参照逐个相同。**不能** | 任何一个单元 GPU 算出的值与 CPU 参照不同；或某张卡起不来计算上下文 | 本机每张能起上下文的卡各跑一片，报每张卡的单元数、不一致数、用时；起不来的卡报原因 | 开着 |
| M2 | 双机多卡：同一批切到两台机器的卡上，结果收回本机合并后与 CPU 逐个相同 | **能**：另一台机器跑拷过去的同一个二进制、算它那几片、结果文件送回本机。**不能** | 同 M1；或另一台跑不起拷过去的二进制；或送回的结果文件不完整（条数与派出的不符） | 两台机器每张能起上下文的卡各跑一片，报每片的单元数、不一致数、用时、传输用时 | 开着 |
| M3 | 比对本身会不会红 | 往输入里改一个单元的一个位、或让一张卡的结果少一条 | GPU 与 CPU 仍逐个相同、或合并时没报少了一条 | 两种故障各注入一次，报比对是不是报出来 | 开着 |

## 二、被测条款与它引的定义

D18 已定项 17 那一行是 D13 已定项 5「参数钉死」所指的参数出处（`crates/singlefs-core/src/checksum.rs:1-2` 链回它）。

**出处 `.claude/kb/milestone/03-third-txn.md:49-54`（整段抄，未转述）**

```markdown
## 四　崩溃放量的 checker 挪到 GPU：过程落文件，一次算出结果

- **原话**：「数据比较核校验部分，我认为可以转移到gpu中，这个作为可选项。默认不生效，但是本机是可以用的，不然显卡是空闲的。当然这部分需要重映射数据结构，需要论证一下代价。测试期间两台主机所有GPU都可以放空。」2026-09-27 改为必做：「这是崩溃放量的实现 是checker的部分 和core无关。」「这个肯定是要实现的，因为一旦到了亿级别的数据验证。花费的时间和实现维护的时间已经不成正比了。所以过程落文件，GPU一次计算得出结果，这样比较好。中间只走过程，过程中的结果还可以hash判定不变。」同日：「如果收益显著的话 我不介意挪动到 里程碑2 来验证」——先做极小规模的计数实验（问题单 `research/prompts/m3-gpu-dedup-r1-forks.md`），收益显著就提前到里程碑二。需要时 5 张卡都可以整卡用（用户同日定）。
- **仓里已有的**：两台主机共 5 张卡，可用合计约 96 GB：本机 RTX 5090 32 GB、RTX 5060 Ti 16 GB；另一台 RTX 5080 16 GB、RTX 5060 Ti 16 GB × 2（2026-09-27 两边 nvidia-smi 现查）；门禁 94 号要求 checker 与实现只共享常量模块。
- **开工前要先定的**：过程文件的格式（内存装置与虚拟机的崩溃落同一种；落「写序列 + 每个状态的子集掩码」而不是整份镜像，推的），与第一项、第四项一起定；过程里哪些中间结果取 hash、hash 不变就复用判定；GPU 上的 checker 与 CPU 上的 checker 怎么对拍（抽样逐格相同，不同就判红），哪一份算规范；测试期间两台主机的卡都放空，跑之前谁负责把本地模型停下、跑完再起来。

```

**出处 `.claude/kb/decisions/13-验证路线.md:84-100`（整段抄，未转述）**

```markdown
#### 已定项 5：只共享一份从 kb 生成的常量

**定案**：checker 与实现之间只共享一样东西：一份由 kb 的字段表生成的常量模块，生成器从 kb 读、两边都只消费、任何人不许手改（`.claude/singlefs-ai-sop/rules/machine-first.md`：重复要生成，不能手抄）。**其余一律不共享**：地址空间的 newtype 各自声明、格式解析各写一份、校验和各用一份独立实现、遍历与记账代码交集为空（C12（增量语义共用） 由门禁 94 号定的判据，形态是 crate 粒度的依赖闭包交集、不是符号级，「格式解析与常量除外」那条例外就是这一项在用）。

- **判定宽度不许从别处拿**：I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 的判据是挂载时探测的 `physical_block_size`，它不在镜像里。checker 自己探一次，再与系统配置记录的 mkfs 时取值比对（D22（单元原子性怎么合成） 已定项 2「mkfs 按池内最大值划、挂载时逐设备复算比对」的 checker 侧形态）。**探不到时报「声明值，未探测」，不许当成探到的**——文件当设备的镜像属于这一类，崩溃点重放的绝大多数镜像都是文件，所以这条不是边角。异构池里判定宽度是一个集合不是一个数（C17（设备几何当全局参数）），checker 要复算 mkfs 那步取最大值的聚合，不然健康的异构池上会假红。
- **生成器拒绝发射没有 kb 落点的常量**，也只发射标量值：它一旦开始发射「按字段表算出来的偏移函数」，那就是 D13（验证路线） 明令不许共享的格式解析，而不再是常量；判据是发射物里有没有分支与算术。
- **CRC32C 各写一份的前提是参数钉死**（多项式、初值、输入输出反转、异或输出）：参数不钉死，两份独立实现会在健康镜像上给出不同结果，那是假红而不是独立性。

**射程**：管 checker 与实现的 crate 边界；管不到 kb 里就写错的值——共享一份生成常量与两边各抄一份都抓不住那一类，它要一条正交的门禁（C94（登记的格式常量与后来的定案对不上））。实现与它的差距：`crates/singlefs-format` 是手写的，不是生成的（值由门禁 27 号按 kb 里的 `format-const` 标记绑住，只绑标了的那些），生成器没有；`crates/singlefs-checker` 只依赖 `singlefs-format`，CRC-32C 另写了一份按位的；checker 取的是系统配置声明的 `physical_block_size`（`crates/singlefs-checker/src/image.rs` 的 `geometry_of`），「探到 / 声明」两种来源只有类型（`crates/singlefs-checker/src/lib.rs` 的 `DecisionWidth`）、没有接到判定上，I-7.5（根槽按判定宽度对齐） 与 I-8.2（记录头不跨原子单元） 未实现。系统配置里 mkfs 时的 `physical_block_size` 字段在 D22（单元原子性怎么合成） 已定项 9 的几何段（4 字节）。

**依据**：
- 无实验：共享边界是保验证独立性的政策，没有可量的量；它的判别力靠两边各写一份的检查逐条红给人看。
- 三方对抗：材料 `_checker-sharing-vehicle-background.md`，腿的产出 `research/prompts/checker-sharing-vehicle-*`，腿况在 [verification-build.md](../verification-build.md)——正推腿从 D13（验证路线）、C12（增量语义共用） 推出「常量在例外内、其余禁止」；替代反推腿与本地腿各自构造出同一条：kb 里写错的值共享与各抄都抓不住。
- 用户定案，记在变更史。

**欠**：C94（登记的格式常量与后来的定案对不上）；C17（设备几何当全局参数）；C387（常量模块手写，不是从 kb 生成）；C388（checker 的判定宽度不自己探测）。

```

**出处 `.claude/kb/decisions/18-块里携带什么信息.md:427-427`（整段抄，未转述）**

```markdown
**定案**：32 字节头校验和字段里放 CRC32C 4 字节 + 28 字节零；根记录自证校验和、系统配置整槽校验和、journal `header_csum` 同口径。自证结构（根记录、系统配置槽、journal 记录头）的校验和覆盖整个槽含补齐，校验和字段自身按零参与；补齐恒为 0。**写死的参数**：CRC32C = CRC-32C，Castagnoli 多项式 0x1EDC6F41，反射实现（输入输出都反转），初值 0xFFFFFFFF，输出取反；结果按小端写进 32 字节字段的**前 4 字节**，其余 28 字节恒 0，读者遇到非 0 一律判该结构损坏。全仓每一处 CRC32C（头校验和、载荷 CRC、位置条目里的 4 字节、自证结构的校验和）同一套参数，别处一律链回这里。**校验和算法标识进系统配置**：自举头段加 1 字节「校验和算法标识」，登记表 0 = 无效、1 = CRC-32C，第一版写 1（D22（单元原子性怎么合成） 已定项 9）；挂载时按这个字节选算法，与单元头里那 4 字节校验和的口径同一处登记。改第一个事务的字节：**是**——每一个校验和字段的值与那 4 字节在字段里的位置。
```

## 三、实现今天的样子

- CRC-32C 两份：`crates/singlefs-core/src/checksum.rs:8`（反射多项式 0x82F6_3B78）、`:43-63`（slicing-by-8，`:45` 初值 `!0u32`、`:62` 输出取反）、`:97-98`（断言 "123456789" → 0xE306_9283、空串 → 0）；`crates/singlefs-checker/src/lib.rs:38-52`（按位，`:39` 多项式、`:40` 初值、`:51` 取反）、`:57`（一张 256 项的表）、`:733`（同一公开校验值）。
- 整单元校验：`crates/singlefs-checker/src/image.rs:437-439`（读出的整份单元求 CRC-32C 与位置条目比）；位置条目那 4 字节在加密关时是整单元 CRC-32C（`crates/singlefs-core/src/pointer.rs:13`）；单元大小 `crates/singlefs-format/src/lib.rs:20` `DATA_UNIT_BYTES = 32768`。
- GPU：`grep -rn -i 'wgpu\|vulkan\|cuda' crates/ research/e7-index-bench/src research/e7-index-bench/Cargo.toml | wc -l` → 0。`crates/` 里没有 GPU 路径。

## 四、跑之前已经存在的数

- 第七节 B 类锚点的八个值是本登记用一行 python 按位实现现算的（命令见第十三节），其中 "123456789"、RFC 3720 B.4 四个 32 字节向量与公认值一致；32768×00、32768×FF 两个只有这一次算。它们是锚点，不回答 M1–M3 任何一行。
- 环境数（显存余量约 500 MiB / 140 MiB、卡的型号与张数）来自派发提示与岔路单开头，用来定规模（第五节），不进判据。
- grep 读到的无关数：`.claude/kb/decisions-history/2026-09.md:2117` 的 E6（加密算法选型） 吞吐 17.4 GB/s、244.74 MiB/s；`.claude/kb/checks-owed.md:70` 的 71 字节 / 0.22%；`.claude/kb/milestone/03-third-txn.md:51` 的各卡显存。都不是 CRC 相等性的数，对判据无影响。
- 仓里没有 GPU 算 CRC 的实验（第三节 grep 零命中；`research/prompts/` 下 gpu 相关只有两份岔路单，本登记只读了 m2 那份）。

## 五、臂、阳性对照、真实基线

**装置写在** `research/e7-index-bench/src/bin/e163_gpu_multicard_crc32c.rs`（bin 名 `e163-gpu-multicard-crc32c`），独立手写模型，不是入库装置，不碰 `crates/`。依赖 `wgpu = "=30.0.1"`（`default-features = false`，开 `vulkan`、`wgsl`、`std`；编不过可加最少的特性，写进产物头，不算改判据）与 `pollster = "=1.0.1"`，都设 `optional`，挂在不默认打开的特性 `e163-gpu` 上、bin 写 `required-features = ["e163-gpu"]`，照 `research/e7-index-bench/Cargo.toml:452-471` E162 的写法，门禁 15 号不编它。编：`cargo build --release -p e7-index-bench --features e163-gpu --bin e163-gpu-multicard-crc32c`，只在本机编一次。

- **臂 C（CPU 参照）**：装置里自写按位 CRC-32C（不查表、不从 `crates/` 引、不与臂 G 共用函数或表），参数照 D18 已定项 17。
- **臂 G（每张卡一条）**：WGSL 计算着色器，每个调用算一个单元；表（若用）在着色器里自己生成，主机不传表。输入按小端打进 `array<u32>`，着色器按 `(word >> 8*(k%4)) & 0xFF` 取第 k 字节，支持单元长度 0–32768（锚点要短输入）。每条结果写 `(crc, gid)`，`gid` 取内建 `global_invocation_id.x`；派发前结果缓冲区整片填哨兵 `0xFFFFFFFF`。每次派发 128 单元（4 MiB），一块输入缓冲区复用，压低显存。
- **对面那条臂（「不能」）照支持它的人认的样子**：没有 nvcc / nvrtc，只能走 Vulkan 着色器，整数移位、字节序、驱动编译器可能与 CPU 不一致；显存被本地模型占着，上下文或缓冲区起不来；另一台没有 Rust，拷过去的二进制可能缺 Vulkan loader 或驱动 ABI 不合；scp 往返可能丢或截断。所以：每张枚举到的 NVIDIA 卡都试，不挑起得来的；另一台跑的是同一文件（两端 sha256 相同），不在那边重编；不装包、不停本地模型；软件适配器不算卡。
- **批**：N = 2560 单元 × 32768 字节，种子 `0xE163_0927_0000_0001`；单元 i 的内容 = splitmix64 以 `种子 ^ i·0x9E37_79B9_7F4A_7C15` 为初态的输出按小端铺满 32768 字节。本机生成，按片写成输入文件（头：魔数、首单元号、单元数、单元字节）。
- **跑 R1（M1）**：本机两张卡各 1280 单元（片 0 = 单元 0–1279，片 1 = 1280–2559），按 wgpu 枚举 NVIDIA 卡的次序派。
- **跑 R2（M2）**：五片各 512 单元，片 0–1 本机两张卡，片 2–4 另一台三张卡，按各机枚举次序派；每片报 AdapterInfo 全部字段（API 给得出 PCI 总线号就报）。
- **跑 F1、F2（M3）**：在 R2 的五片配置上各跑一次，注入见第六节。另一台跑不起时改在 R1 的两片配置上跑：F1 注入全局单元 100（字节 4099 位 1）与 1381（字节 8198 位 2），F2 让片 1 少 1480。
- **阳性对照（每条臂都跑）**：PC-A 每次跑（R1、R2、F1、F2）开头，臂 C 与每张卡都算第七节 B 类八个锚点；PC-B 即 F1（每张卡的片里各翻一位）与 F2（一张卡少一条）。
- **真实基线**：项目今天用 CPU 上的 checker 算整单元 CRC-32C（`crates/singlefs-checker/src/image.rs:439`）；本实验不调它（不共用代码），以臂 C 为 CPU 基线，它的用时附带报、不判。
- **传输**：本机 → 另一台：二进制、片 2–4 输入文件；另一台 → 本机：片 2–4 结果文件与另一台的运行日志。每个文件两端各 `sha256sum`，逐个比；结果文件条数在合并时与派出数比。传输用时 = 本机驱动命令里每次 scp 前后各取一次 `date +%s.%N` 的差（scp 整条命令的挂钟）。
- **计时**：每片 GPU 用时由装置在被测进程里用 `std::time::Instant` 自己计（建设备、上传加算加回读、合计三段），写进结果文件头行；另一台的由另一台上的进程自己计。不在转发输出的循环里打时间戳。
- **运行**：本机经 `bash research/scripts/run-with-memory-cap.sh <上限> …`，上限取派发给的、没给取 `research/scripts/replay.sh:26` 的默认 8G；另一台没有仓，经 `systemd-run --user --scope -p MemoryMax=<同一上限> -p MemorySwapMax=0 …`，起不来就报原因交主 agent，不裸跑。驱动两机的命令原样落进产物。
- **实现量**：一个 bin（生成器、按位参照、着色器、枚举分片、结果文件、合并比对、`--inject-flip` / `--inject-drop` 两个注入开关）加两机驱动命令；一段做完，不分段。

## 六、报哪些量与各自的判据

门槛 0 与 2560 不是臂 G 的构造推得出的：G 与 C 不共用代码，相等只能靠算出来；覆盖数靠哨兵与 `gid` 判，不靠派出数。每个量各报各的判定，不合取。

| 量 | 怎么算 | 门槛 | 岔路行与翻面取值 |
|---|---|---|---|
| Q1 R1 本机卡起不起得来 | 每张枚举到的 NVIDIA 独显：`request_device` 与首块缓冲区分配成败；失败报 wgpu 原样错误，归类（枚举不到 / 建设备失败 / 分配失败） | 起得来的张数 = 本机 `nvidia-smi -L` 行数 | M1：少于 ⇒ 翻「不能」；原因是显存的注明，要不要停模型重跑由主 agent 问用户 |
| Q2 R1 每片不一致数 | 按全局单元号比 G 与 C，不同的个数 | 每片 0 | M1：任一片 ≥ 1 ⇒ 翻「不能」 |
| Q3 R1 合并覆盖数 | 合并后有值、单元号不重复、`gid` 对得上的个数 | 2560 | M1：< 2560 ⇒ 翻「不能」 |
| Q4 另一台跑不跑得起拷过去的二进制 | 另一台上 `--probe`（只列适配器）的退出码、stderr、列出的 NVIDIA 卡数 | 退 0，卡数 = 那台 `nvidia-smi -L` 行数 | M2：非 0、缺 `libvulkan.so.1`、或卡数少 ⇒ 翻「不能」 |
| Q5 R2 每片不一致数 | 同 Q2，五片 | 每片 0 | M2：任一片 ≥ 1 ⇒ 翻「不能」 |
| Q6 R2 往返文件完整 | 两端 sha256 相同的文件数 / 往返文件总数；片 2–4 结果条数 | 全相同；条数各 512 | M2：任一不同或条数 ≠ 512 ⇒ 翻「不能」 |
| Q7 R2 合并覆盖数 | 同 Q3 | 2560 | M2：< 2560 ⇒ 翻「不能」 |
| Q8 F1 报出的不一致单元号 | 片 s（0–4）只在上传副本里翻全局单元 512·s+100+s（100、613、1126、1639、2152）第 4099·(s+1) 字节的第 s+1 位；输入文件与 sha256 不动 | 报出集合含注入的五个 | M3：漏报任一个 ⇒ 翻「不会红」；注入之外多报的计进 Q5 |
| Q9 F2 报缺的单元号 | 片 3 写结果文件时略去单元 1736（`--inject-drop 1736`），另一台报的条数与 sha256 照写出的文件算 | 合并报「缺 1736」且只缺这一个 | M3：没报缺或报错号 ⇒ 翻「不会红」 |
| Q10 每片单元数、各段用时、每次 scp 用时、臂 C 用时 | 第五节「计时」「传输」 | 只报不判 | 岔路单够判条件要报，不翻任何一行 |

## 七、钉绝对值的断言

**A 类，出自被测条款本身**（不符走第十节失败 4）：D18 已定项 17（D13 已定项 5「参数钉死」所指）的四样参数——多项式 0x1EDC6F41（反射 0x82F63B78）、输入输出反射、初值 0xFFFFFFFF、输出取反。臂 C 与着色器源码里这四样逐一对得上，执行员贴源码行。

**B 类，独立算出、命令核过**（臂 C 不符或每张卡都不符 ⇒ 第十一节作废 1；只有部分卡不符走第十节末条）：臂 C 与每张卡在每次跑开头都算这八个。

| 输入 | 长度 | CRC-32C | 出处 |
|---|---|---|---|
| ASCII "123456789" | 9 | 0xE3069283 | 公认校验值；派发提示；第十三节命令复算 |
| 空串 | 0 | 0x00000000 | 第十三节命令 |
| 32 × 0x00 / 32 × 0xFF | 32 | 0x8A9136AA / 0x62A8AB43 | RFC 3720 B.4；第十三节命令复算 |
| 0x00..0x1F 升序 / 0x1F..0x00 降序 | 32 | 0x46DD794E / 0x113FDB5C | RFC 3720 B.4；第十三节命令复算 |
| 32768 × 0x00 / 32768 × 0xFF | 32768 | 0xBC43BAAD / 0x434C2368 | 第十三节命令，只算过这一次 |

另三条（不符 ⇒ 作废 2）：臂 C 的 2560 个值两两不同（真碰撞概率约 7.6e-4，推的）；执行员用第十三节那行 python 的同一函数，从输入文件字节重算单元 0、1279、2559，与臂 C 同值；每张卡的每条结果 `gid` = 片内偏移、没有哨兵残留。

## 八、轨迹与几何敏感性

不适用：没有被停机 / 准入谓词消费的量，每个量是一次跑的计数，没有轮次；主 agent 定这次可行性验证不做几何敏感性。限度照写：结论只在 N = 2560、每次派发 128 单元、单元 32768 字节、本次两机的显存余量下成立，没扫片大小与派发粒度。

## 九、变异

执行员逐条跑，每条必须在写明的取样点上改变输出（判红）；不红照 `.claude/rules/mutation-sampling.md` 分类。都在本机跑，用 R1 规模，V9 用 R2 收回的文件。

- V1 合并比对恒判相同 → F1（Q8 报出 0 个，应为 5 个）。
- V2 合并只遍历收到的记录、不按 0..N 查缺 → F2（Q9 不报缺 1736）。
- V3 着色器用未反射的 0x1EDC6F41 → 每张卡的 B 类锚点（空串以外全红），R1 两片 Q2 近 1280。
- V4 着色器漏最后取反 → 每张卡的 B 类锚点全红（空串 0 → 0xFFFFFFFF）。
- V5 臂 C 与着色器同改成 0x82F63B79 → Q2 仍 0（互比看不见），只有 B 类锚点红：证明绝对值那一条不可少。
- V6 着色器按大端从 u32 取字节 → "123456789"、升序、降序与随机单元红；四个全同字节的锚点不变。
- V7 结果不从设备回读、由主机按原输入算 → F1 漏报（翻位只进上传副本），`gid` 断言红。
- V8 每次只派一半工作组 → 哨兵残留，Q3 < 2560。
- V9 本机收到的片 2 结果文件截掉末行 → Q6 的 sha256 与条数都红。

## 十、失败条款

- 失败 1（M1 翻「不能」）：R1 里 Q1、Q2、Q3 任一不过门槛。触发观测：合并报告某张卡那一行是 wgpu 错误串、或某片不一致数 ≥ 1、或覆盖数 < 2560。
- 失败 2（M2 翻「不能」）：Q4、Q5、Q6、Q7 任一不过门槛。触发观测：另一台 `--probe` 非 0 退出或卡数少、某片不一致数 ≥ 1、某个往返文件两端 sha256 不同或结果条数 ≠ 512、覆盖数 < 2560。
- 失败 3（M3 翻「不会红」）：Q8 或 Q9 不过门槛。触发观测：F1 报出集合缺注入的某个号，或 F2 合并没报缺 1736 / 报的号不是 1736。
- 失败 4（条款参数可能错）：臂 C、每张卡、第十三节 python 三方都照 A 类参数算，"123456789" 三方同值却不是 0xE3069283。触发观测：三份输出同一个非 0xE3069283 的值。
- 某张卡锚点不符而同一二进制的另一张卡相符：那张卡计进 Q2 / Q5（按不一致算，走失败 1 / 2），不走作废。

## 十一、作废条款与停机条款

- 作废 1：臂 C 或所有卡同时有 B 类锚点不符（装置写错）→ 那一次跑作废，修装置后重跑。
- 作废 2：臂 C 2560 个值有重复、python 抽算三单元与臂 C 不同、或 `gid` / 哨兵断言不过 → 那一次跑作废。
- 作废 3：某片的适配器不是 `vendor 0x10DE` 的独显（软件适配器顶上）→ 那一片作废，不算「能」。
- 作废 4：另一台跑的二进制与本机编出的 sha256 不同；R1、R2 的产物头里注入列表非空；内存包装退 250–254 → 那一次不算。
- 停机（`.claude/rules/implementation-first.md` 第 4 条）：臂 C 的四样参数与 `crates/singlefs-core/src/checksum.rs:8`、`:45`、`:62`、`crates/singlefs-checker/src/lib.rs:39-40`、`:51` 对不上，或臂 C 在 "123456789"、空串上的值与 `crates/singlefs-core/src/checksum.rs:97-98`、`crates/singlefs-checker/src/lib.rs:733` 断言的不同 → 停，两边都查，不作废也不当结果。
- 够判停机：次序 R1 → R2 → F1 → F2；M1–M3 每行都够判就停，没跑的量逐条标「够判后未跑」。另一台跑不起时 M2 已够判（翻「不能」），F1、F2 改用第五节的本机配置。

## 十二、修订

（留空；装置写之后、产物之前由执行员写，只许收严或补臂。）

## 十三、读过的文件与跑过的命令

读过（行号区间；grep 命中行也列）：`.claude/agent-common.md:1-97`；`research/prompts/m2-gpu-multicard-r1-forks.md:1-11`；`.claude/singlefs-ai-sop/rules/test-discipline.md:44-136`；`.claude/rules/three-way-inference.md:145-148`；`.claude/singlefs-ai-sop/rules/evidence-discipline.md:119-148`；`.claude/rules/mutation-sampling.md:34-55`；`.claude/rules/implementation-first.md:1-20`；`.claude/kb/milestone/03-third-txn.md:1-60`；`.claude/kb/decisions/13-验证路线.md:1-23,34,73,84-100`；`.claude/kb/decisions/18-块里携带什么信息.md:167,204,413,425-432`；`.claude/kb/checks-owed.md:70`；`.claude/kb/decisions-history/2026-09.md:305,445,817,2117,7672,7925,14045,14047,14049,14052,14058,14059`；`.claude/kb/vm-harness.md:149,176`；`crates/singlefs-core/src/checksum.rs:1-64,94-98`；`crates/singlefs-core/src/journal.rs:3,16,174,255,332,408`；`crates/singlefs-core/src/pointer.rs:13`；`crates/singlefs-checker/src/lib.rs:2,36-70,89,348,634,639,674,732-733`；`crates/singlefs-checker/src/image.rs:425-445`；`crates/singlefs-format/src/lib.rs:19-20,25,28-29,109,118,353,358`；`research/e7-index-bench/Cargo.toml:1-84,448-471`；`research/e7-index-bench/src/bin/e144_header_checksum_cost.rs:311,313,336`；`research/e7-index-bench/src/bin/e162_crash_verdict_block_store.rs:195,207,229`；`.claude/gate.d/15-research-build.sh:3,12,26-29,32,34,43-44,55,64,69`；`research/scripts/quote-kb.py:1-40`；`research/scripts/claim-experiment.sh:1-60`；`research/scripts/run-with-memory-cap.sh:1-45`；`research/scripts/replay.sh:25-26,406`。没读 `.claude/kb/experiments/`、`research/results/`、`research/prompts/m3-gpu-dedup-r1-forks.md`。

跑过（仓根下，原样）：
- `bash research/scripts/claim-experiment.sh --next` → `E163`；`bash research/scripts/claim-experiment.sh E163 'GPU多卡算单元校验和'`
- `grep -rn -i 'crc32c\|crc-32c\|0x82F63B78\|0x1EDC6F41\|Castagnoli' crates/ --include=*.rs`；`grep -rn -i 'wgpu\|vulkan\|cuda' crates/ research/e7-index-bench/src research/e7-index-bench/Cargo.toml | wc -l` → `0`
- `grep -rn '备料' .claude/kb --exclude-dir=experiments --exclude-dir=results --exclude-dir=prompts --exclude=experiments.md --exclude=experiments-history.md`
- `grep -rli 'crc32c\|castagnoli' research/e7-index-bench/src/bin/`；`ls research/prompts | grep -i gpu`
- `cargo search wgpu --limit 3` → `wgpu = "30.0.1"`；`cargo search pollster --limit 2` → `pollster = "1.0.1"`；`cargo info wgpu@30.0.1` → `rust-version: 1.87.0`（本机 `rustc 1.98.0`；这一条把 wgpu 源码包下进了 `~/.cargo/registry` 缓存）
- `python3 research/scripts/quote-kb.py <草稿>/quote.md '<milestone 03>@## 四…' '<D13>@#### 已定项 5…' '<D18>:427-427'` → 3 段整抄、回读一致；追加后 `tail -n 38` 与草稿 `cmp` 一致
- B 类锚点（第七节）用的命令，输出八行依次为 0xE3069283、0x00000000、0x8A9136AA、0x62A8AB43、0x46DD794E、0x113FDB5C、0xBC43BAAD、0x434C2368：

```
python3 -c 'exec("def c(d):\n r=0xFFFFFFFF\n for b in d:\n  r^=b\n  for _ in range(8): r=(r>>1)^0x82F63B78 if r&1 else r>>1\n return r^0xFFFFFFFF"); [print("%-10s 0x%08X"%(n,c(d))) for n,d in [("123456789",b"123456789"),("empty",b""),("32x00",bytes(32)),("32xFF",b"\xff"*32),("00..1F",bytes(range(32))),("1F..00",bytes(range(31,-1,-1))),("32768x00",bytes(32768)),("32768xFF",b"\xff"*32768)]]'
```
