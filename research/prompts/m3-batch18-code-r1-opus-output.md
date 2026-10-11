# 删除批第 18 批代码轮第一轮：攻方腿（Opus）报告（2026-10-05）

攻击面：正文第三节 K2，重点 N1–N3（池级 checker 删除态判法）与 N8（GPU 判器内核对 P1 的 CPU 判法），另带 N5、N9。被判对象是冻结副本 `/tmp/claude-1000/m3-batch18-code-r1/tree/`；我在它的三份拷贝上跑原型（原样、改法 A、注入坏实现 C），只跑 harness 档，没起 checker 档的测试目标与 bin。

## 复跑

模型目录 `research/prompts/m3-batch18-code-r1-opus-model/`。在仓根执行，工作目录要空；每一步一条前台命令跑得完（单步最多约 3 分钟），`all` 依次跑全部（约 50 分钟，主 agent 放后台用）：

```
W=<一个空目录>
export FROZEN_TREE=/tmp/claude-1000/m3-batch18-code-r1/tree
R=research/prompts/m3-batch18-code-r1-opus-model/rerun.sh
bash $R $W prepare      # 核冻结树 19 份 sha256，拷三份树、打 workspace / fixA / mutC 补丁、放进原型 crate
bash $R $W build        # 三份各自 cargo build --release -p m3b18-opus-proto（经内存包装、capped.sh 10）
bash $R $W probe; bash $R $W badimages; bash $R $W gpugap; bash $R $W n9; bash $R $W diskloss
bash $R $W fixA-badimages; bash $R $W fixA-tests; bash $R $W mutC-sweep; bash $R $W mutC-crash
bash $R $W sweep 0 1500; bash $R $W sweep 1500 999999
bash $R $W fixA-sweep 0 1500; bash $R $W fixA-sweep 1500 999999
for f in 0 1650 3300 4950 6600 8250; do bash $R $W sweepx $f $((f+1650)); done
for f in 0 9 18 27 36 45 54 63; do for p in 0 1; do bash $R $W crash $f $((f+9)) $p; done; done
bash $R $W summary      # 摘要与 expected-summary.txt 逐行比，打 SUMMARY_MATCHES lines=58
```

交回之前在空目录 `/tmp/claude-1000/m3-batch18-code-r1-opus/rerun-fresh3/` 里从 prepare 起逐步跑完一遍（模型定稿之后的第三遍；前两遍之后各加过一步，加完都从头重跑），末行原样：`SUMMARY_MATCHES lines=58`。`out/` 里放的就是这一遍的原始输出。`summarize.py` 把各步输出并成与挂钟、线程数、分批方式无关的摘要（每段历史的结果行取 sha256 前 16 位）。

模型目录每个文件的 sha256（`find … -type f | sort | xargs sha256sum` 原样）：

```
eb0394bfe117936c08005da9f391a2a3e325e8a85d9e60a26da89d2b9a4ef292  research/prompts/m3-batch18-code-r1-opus-model/expected-summary.txt
8f7c4f4492a1a9e99cb7644ddf2e6fe432a9108e44737efdd118c13d2008650c  research/prompts/m3-batch18-code-r1-opus-model/fixA.patch
16ff69cd9364d9fda5cdcb601a8f167f8a6205281a0b37c84a1d4a6f9e43bc02  research/prompts/m3-batch18-code-r1-opus-model/mutC.patch
abe052f58ec8cdb420069862f389d27531fe14eff0ee8d95d3598c8aaecb5dbe  research/prompts/m3-batch18-code-r1-opus-model/out/badimages.out
6213605a0ecbd79c5edd143763613d7b655a823f1d4c5b06c953fe969da940b9  research/prompts/m3-batch18-code-r1-opus-model/out/crash-0-p0.out
cf9659aac5d162836ab53f1d62babc595052dbba75f597360cd49bf5ebe8abd7  research/prompts/m3-batch18-code-r1-opus-model/out/crash-0-p1.out
b43050cec25d7f7ded742e06e79a2bdab279792c5ba40cadd18fbfc0e3d62f47  research/prompts/m3-batch18-code-r1-opus-model/out/crash-18-p0.out
f67ca99b339117c7f6715c5f66489d88fc4943a6f194724518f0c199d040cd02  research/prompts/m3-batch18-code-r1-opus-model/out/crash-18-p1.out
312c5e4e49cfdf1e4cfedb1309b294acec1948bec59bfa3e32576225706a5446  research/prompts/m3-batch18-code-r1-opus-model/out/crash-27-p0.out
5455edd13b9ca49183cb818f883f634dfdcff1cb369463fc38e9f6f4694883da  research/prompts/m3-batch18-code-r1-opus-model/out/crash-27-p1.out
2b7261f7d25089be0b2d394bce54419868cccf6db574e97e22842bbfed86942f  research/prompts/m3-batch18-code-r1-opus-model/out/crash-36-p0.out
ac538452e66f377414aa2cf88587724466c71cf3454b7d418e698e1fec824ae1  research/prompts/m3-batch18-code-r1-opus-model/out/crash-36-p1.out
aac477f03e30a79eb02b1d0191f3e6eb82496a78f04d6b3c7f20fe5d14f0d28c  research/prompts/m3-batch18-code-r1-opus-model/out/crash-45-p0.out
599c58d3d8807f63ebb8ffc5ce6420e04d51f0b2311936b1501fa88b93d5a121  research/prompts/m3-batch18-code-r1-opus-model/out/crash-45-p1.out
63c3b3db5573d8547e1d6c17c7f67ccb4bc137214f95f1fe7319f161bde44e6f  research/prompts/m3-batch18-code-r1-opus-model/out/crash-54-p0.out
323f8994e6cd89a09b7263ea3232807bde6eb0e4168f707b9aa80397b46502dc  research/prompts/m3-batch18-code-r1-opus-model/out/crash-54-p1.out
569a917a556777764904d5e8e0ce50242484f9a0b8425b719c3a875ce930496a  research/prompts/m3-batch18-code-r1-opus-model/out/crash-63-p0.out
0ce8d8e11570f50ffc527969574ca0cc3aa0d8b1ba2751222c575dcc35927cc1  research/prompts/m3-batch18-code-r1-opus-model/out/crash-63-p1.out
a12d942fc6eb703f1f511058b44a76ce24281379e94976f702098a70df5e9ff4  research/prompts/m3-batch18-code-r1-opus-model/out/crash-9-p0.out
024bd00a463209812d6366b9fae5ede53aeb55ccbefcde19adb7cf68341e7b5d  research/prompts/m3-batch18-code-r1-opus-model/out/crash-9-p1.out
13fd617c4bc5ed9d703a62e68120f009e4a8312e90dfe4e9e8af88f246eed286  research/prompts/m3-batch18-code-r1-opus-model/out/diskloss.out
1d2efe1cd0e0fdd1f7897f15215d5b0ec2f456d009bc0c796549dd0f9b4e192a  research/prompts/m3-batch18-code-r1-opus-model/out/fixA-badimages.out
6e526ba01470f21396baea51d4efd356ce341c892274a5fea236def0e70d78cc  research/prompts/m3-batch18-code-r1-opus-model/out/fixA-sweep-0.out
9d54bba8b716a762c27b37cb11da2bdae48740ec8d6d1d80e834a5e7cab73837  research/prompts/m3-batch18-code-r1-opus-model/out/fixA-sweep-1500.out
6872a76527deb164377498d1926eee43809d1d44c59340067a4db532379373a2  research/prompts/m3-batch18-code-r1-opus-model/out/fixA-tests.out
78f55e0dc2487e71bb46da4a40f723e6188034633f5097eddf773dc6cbbfab0c  research/prompts/m3-batch18-code-r1-opus-model/out/gpugap.out
353e0c9e2c2d4ce387bfe0358e2c3734552e7b5256b8c841195a828ef643690e  research/prompts/m3-batch18-code-r1-opus-model/out/mutC-crash.out
e7cfaea310a46d5c9fb7a14f76d78d1d546a58bb483669725eace85b8f42b53a  research/prompts/m3-batch18-code-r1-opus-model/out/mutC-sweep.out
3a9e4035fa54ea7bb91a9bcc496bcbd95ee7e6180faad18f16871ecd20941f79  research/prompts/m3-batch18-code-r1-opus-model/out/n9.out
3d03517a972e2c0e787ade512bc7756eaf277465b8a8be3bf1d7c5afa8fc9542  research/prompts/m3-batch18-code-r1-opus-model/out/probe.out
eb0394bfe117936c08005da9f391a2a3e325e8a85d9e60a26da89d2b9a4ef292  research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt
015bb9d97cb7c1d920c7b083a3126761dd51fe342868abb17716c0b5a92afc72  research/prompts/m3-batch18-code-r1-opus-model/out/sweep-0.out
bb7b392a310bba398215a2c57e2ae9b3995ff20e97693549ed9e95d2ad4ef636  research/prompts/m3-batch18-code-r1-opus-model/out/sweep-1500.out
c3c4218b2d2a5de8485de3b115e726eddaac827bea0f958d7221f10f2fdcf49d  research/prompts/m3-batch18-code-r1-opus-model/out/sweepx-0.out
4ed4f7df2f6c17d2721edc9d7e0068b6d818637d139d07a7d8863b5db4172a5c  research/prompts/m3-batch18-code-r1-opus-model/out/sweepx-1650.out
8237b9ce0dff05651ed65ff52dc12114c87f08e488101e7f54fa1d1e96b8da27  research/prompts/m3-batch18-code-r1-opus-model/out/sweepx-3300.out
6869b9adc83fb92f65a4a82742d7c6a9fe4ff94760c9ac8efdad0106953cdc39  research/prompts/m3-batch18-code-r1-opus-model/out/sweepx-4950.out
83379047f7f6bf4783dc4b08e4790e8802c43eeaeb94c95d736c9ac8692e5917  research/prompts/m3-batch18-code-r1-opus-model/out/sweepx-6600.out
d74ec50d38e2d9792b799efe44ecb66b8bff5b65e7f7ec5d48e77a58dc523b3d  research/prompts/m3-batch18-code-r1-opus-model/out/sweepx-8250.out
c05900249c65193b296f0faf134722f5cc53a86c5be2ba3e5ba07715c76defb5  research/prompts/m3-batch18-code-r1-opus-model/proto/Cargo.toml
b2fe67e17ad0a02513c121e775fdcdd28e2c55fbadfb9ef0d6540204d05ce43a  research/prompts/m3-batch18-code-r1-opus-model/proto/src/badimages.rs
1c29f5b35147b636842ec10a743e841139f09891e66ce0bd9c4c9350df3955a1  research/prompts/m3-batch18-code-r1-opus-model/proto/src/crashpoints.rs
c6d2023e980ff80b2d6bfd618b642cc366bfdcec6138380bd074b03884a80ae4  research/prompts/m3-batch18-code-r1-opus-model/proto/src/diskloss.rs
4b7a688135242db3c82997c4aa4147b1d022c678b1811bbed072769783d0da2a  research/prompts/m3-batch18-code-r1-opus-model/proto/src/gpugap.rs
f9de26d5d54bff8baf43af610b8475d2cf08ce89618d600802a80fa29414645d  research/prompts/m3-batch18-code-r1-opus-model/proto/src/judge.rs
b25c1abf275964d8ea1546e0c493ee1a4dd402a3a7cd35268b3fbb35647aa526  research/prompts/m3-batch18-code-r1-opus-model/proto/src/main.rs
db53bb0f7f6178d1351d9c5448e398664085159e0cc79d2a72285527cce43a66  research/prompts/m3-batch18-code-r1-opus-model/proto/src/n9.rs
b2655893a3e064e24db324228faaf6ae106ca2fec359df4f7e39682ccf21d085  research/prompts/m3-batch18-code-r1-opus-model/proto/src/pool.rs
3fcf9022eb9746b69db0e764259339fd51cce4c1f6df1cba4e6034bfb6075e15  research/prompts/m3-batch18-code-r1-opus-model/proto/src/probe.rs
a2d005484a15b47037473920ae544f711c74dc5eca96e53e25a9c41eae4ef30b  research/prompts/m3-batch18-code-r1-opus-model/proto/src/sweep.rs
115d7200d263dfd567c27a9c7fd892c81491e497193cb60cc20ac9c49705f1e1  research/prompts/m3-batch18-code-r1-opus-model/rerun.sh
cfb82c6905c4c70e3e54973e283beb6eb1037e4dfee0c60913000a5200f6dcca  research/prompts/m3-batch18-code-r1-opus-model/summarize.py
c87d2194b2000552fef6e745dd6be2d61f7d36baabc453cfd9c8fe95179e77a2  research/prompts/m3-batch18-code-r1-opus-model/workspace.patch
```

## 各格判定一览

| 格 | 攻法 | 判定 | 一句话 |
|---|---|---|---|
| K2 误红（N1–N3，合法删除中间态） | 用户决定的几步放开扫：文件单元数 6 种、批大小 1/2/144、unlink 对象与次序 5 种、续做在进程里 / 会话里 / 挂载里 / 卸载再挂 / 停在第 k 批再挂 / 管理员回退到第 k 版再挂、调用方时间 3 种、盘 4 / 8 GiB、删完再建再删；另 72 段历史逐个操作前缀崩、崩溃镜像与挂载续做之后各判一遍 | 没打中 | 12456 段走得到的历史、102348 份已发布镜像、11832 个崩溃点与挂载续做之后的镜像，五条与 I-9.6 一处都没红；同一装置在注入坏实现 C 时红（会红） |
| K2 漏判 N1：树表里没有指着这棵 inode 树的 `deleted_inodes` 条目 | 最新版或中间版树表里 `deleted_inodes` 条目种类码 9 改 13、或整条拿掉 | **打中**（I-9.5 报成立） | 1 号 nlink = 0、不在任何队列里，I-9.5 报 H，只红 I-9.16；同一处换成头 ID 配不上（L1d）就报判不了。判法只从 `deleted_inodes` 一侧去配，反方向不配、不记判不了 |
| K2 漏判 N2：同上的版本上 I-4.7、I-4.6 | L1a/L1b/L2；L1c（只改中间版） | **打中**（I-4.7 在 L1c 报成立）；L1a/L1b/L2 报不适用、理由与镜像不符 | L1c 上 I-4.7 报 H（I-4.6 在 L1c 上字面不违反，那一版 size 与单元数对得上）；L2 上 I-4.6 报 N，理由是「每一版 inode 树里都没有 nlink = 0 的记录」，镜像里明明有；「只改中间版、size 也改错」那一形 I-4.6 会报 H 是推的，没造 |
| K2 漏判 N3：读不全夹在中间、头 ID 配错 | L1d、正文已有那一形 | 没打中 | 照共用判法记判不了，有理由 |
| K2「只红自己」：每份坏镜像换造法再造 | V1–V9（9 份，见第三节） | 没打中 | 9 份都只红目标那一条；V9（降水位）是从 I-9.6 前一半红的，墓碑那一半没隔离出来 |
| K2 N8：GPU 内核与 CPU 判法逐支 | 读代码逐支对 | 没打中 | 没找到分叉；上面 N1 那一形 GPU 同样跳过（两边一致地漏） |
| K2 N8：只比违例位那 6 条漏了什么 | CPU 算出对拍流的切段与没展开那几段的状态 | **打中**（覆盖缺口，不是判法分叉） | 展开上限 8 下比了 141 个状态，全量 1174405257 个；四个发布段（24/30/26/24 个写）一个状态都没比；6 条丢掉的是整个第 0 字与恢复字，不只 C712 那一格 |
| K2 N5：I-9.6 墓碑那一半「已发布」 | 读代码与条款对 | 没打中 | 谓词与 I-1.2 同一份；管理员回退的墓碑树随现行那一版，崩溃恢复抛弃的那条线不在所选根的墓碑树里 |
| K2 N9：`--device-bytes` 非 4 GiB | 照 bin 的同一条库路径跑 3、4+64K、6、8、16、64 GiB、4 GiB+256 B、1 GiB、0 | 没打中 | 29 个区域偏移与长度不变，随盘大小变的恒是同 9 个；1 GiB 与 0 被 mkfs 拒。bin 文档的退出码一句有出入（推的，见第五节） |

### 打中之后的四问（N1 / N2 那一格，L1）

1. **分不分辨臂**：这一轮跑前条款没有臂（「打中：主 agent 现查坐实，改法进下一批；改法算零轮，进下一批的代码轮。」，`research/prompts/_m3-batch18-code-r1-body.md:73`）。打中的是实现偏离共用判法的一处，不是几条臂共用的前提，「打中不分辨臂」那一形不适用。
2. **被判的系统看不看得到判别它的东西**：看得到。`judge_deletion_queue_of_a_version` 手里握着这一版整张树表的条目；它只从 `deleted_inodes` 那一侧去配（`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker/src/walk.rs:8322`：「for deleted_inodes_entry in tree_table_entries」），没有指着这棵 inode 树的条目时循环一次都不进，什么都不记。反方向配不上的那一形（头 ID 错，L1d）它照记判不了（同一文件 8347 行：「Err(DeletionScanGap::DeletedInodesHeadIdentifierNamesNoSingleInodeTree)」），可见判据要的信息就在它眼前。
3. **满足的是判据字面哪一个分句**：漏判判据是「漏判判据：镜像违反了 `invariants.md` 那一行的字面，checker 报成立或不适用。」（`research/prompts/_m3-batch18-code-r1-body.md:75`）。
   - L1a、L1b、L1c 的 I-9.5，L1c 的 I-4.7：报**成立**，命中「报成立」分句（L1c 的 I-4.6 也报成立，但那一版 size 与单元数对得上，字面不违反，不算）。I-9.5 ② 字面是「inode 树里 nlink = 0 的每条记录，号都在同一棵树的 `deleted_inodes` 里」（`.claude/kb/invariants.md:292`），I-4.7 字面是「两者皆非的中间态不存在」（`.claude/kb/invariants.md:183`），这几份镜像上 1 号 nlink = 0、还有数据单元、号不在任何队列里。而且每一版都有 `deleted_inodes` 条目是条款写死的（「在第一个文件版本那次发布零根注册」，`.claude/kb/decisions/08-核心索引结构.md:466`），所以这不是「这一版没有这棵树」的不判。
   - L1a、L1b、L2 上 I-4.7、I-4.6 报**不适用**：字面上命中「报不适用」分句，但它与共用判法冲突——共用判法明写「配不上的版记判不了」（`.claude/kb/invariants.md:292`），判不了在没有违例时就报不适用。照这条，L1d 的「不适用」是设计。所以这几格我不算打中，记成「判据字面与共用判法打架」交主 agent（第五节限度第 3 条）。另有一处是实打实的：L2 报不适用时带的理由与镜像不符（理由串在 `walk.rs:8253`：「走读到的每一版 inode 树里都没有 nlink = 0 的记录」），而 `.claude/kb/invariants.md:6` 要的是「不适用（必须带理由，不许静默算成立）」——理由写错等于没写对。
4. **跑前条款给的每个改法在打中的格上还中不中**：跑前条款没点名改法（只写改法进下一批）。我给的改法 A（第三节末，**只在我的模型上量过、被攻过零轮**）把 L1a–L1c、L2 都改成判不了、带对的理由，「报成立」那一分句不再中；但「报不适用」分句在改法 A 之后照中（改法 A 正是让它报不适用），对那一分句不起作用。要让它红得走改法 B（推的，没实现）。

## 一、合法删除中间态上误不误红（N1–N3）：没打中

误红判据照正文：「误红判据：一个从入口走得到的已发布状态上，五条或 I-9.6 有一条报不成立，就是误红；读不全报部分没判不算误红。」（`research/prompts/_m3-batch18-code-r1-body.md:74`）。

**已发布状态那一组（`sweep`、`sweepx`）**。只固定前缀：mkfs → 取号 → 暖机 → 新池新建文件（内存稀疏盘、`allocator_after_make_filesystem` 那一条产品路径）。之后放开扫：
- 文件形状 6 种：不再写（3000 字节、1 个单元）、顺序写 0 字节、1 个满净荷、2.5 个净荷（3 单元）、3 个满净荷、5 个净荷 + 1 字节（6 单元）；
- unlink 前建 0 或 2 个空 inode；unlink 对象与次序 5 种（[1]、[2]、[2,1]、[1,3]，没建 inode 时只有 [1]）；
- 批大小 1、2、144（产品）；
- 续做与发布方式 12 种：进程里直接调、会话 `UserChange::Unlink`、意图之后不卸载重挂续做、意图之后正常卸载再挂续做（H2 那一形）、进程里删完后管理员回退到删除串第 0–3 版再挂续做、进程里删到第 0–3 批停下再挂续做；
- 意图的调用方时间 3 种（与记录同、早一天、晚一天）；每盘 4 GiB 与 8 GiB；删完之后会话里再建 2 个 inode、删其中最大号、再建 1 个（做与不做）。

整张叉积 12960 段（`sweep` 3060 段 + `sweepx` 补扫的 9900 段），每次发布、每次用户改动、每次挂载之后的整池镜像都跑池级 checker：

- `research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:1`：「SWEEP histories=3060 ran=2976 not_reachable=84 images_judged=24268 red_histories=0」
- `research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:55`：「SWEEPX histories=9900 ran=9480 not_reachable=420 images_judged=78080 red_histories=0」
- `research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:2`：「'I-4.6=H': 16240, 'I-4.6=N': 8028, 'I-4.7=H': 16240, 'I-4.7=N': 8028, 'I-9.5=H': 24268, 'I-9.6=H': 24268」

走不到的 504 段全是「回退目标越界」（删除串不到 k 版，例如 1 个单元的文件没有第 2 版可回退）。I-4.6、I-4.7 报 N 的那些是没有删除中对象的版本（照条款报不适用带理由），I-9.5、I-3.4、I-9.6 在每一份镜像上都真判过（H）。

**崩溃点那一组（`crash`）**。72 段历史（6 种形状 ×（不建 inode 删 1 号 / 建 2 个先删 2 再删 1）× 批大小 1、2、144 × 续做在进程里 / 意图之后卸载再挂续做），从意图那次发布的第一步起，每个操作前缀当一个崩溃镜像（前缀里的写全落、之后一个都没落）：先判崩溃镜像，再可写挂载续做走空、判续做之后的镜像。

- `research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:3`：「CRASH histories=72 prefixes=11832 crash_image_red=0 mount_refused=0 after_mount_red=0」

**会红**：同一装置在冻结树的拷贝上把 `crates/singlefs-core/src/unlink.rs` 里批之后的 size 多加 1 字节（`mutC.patch`，那一行原文在 `/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-core/src/unlink.rs:128`：「size_after_the_batch_in_bytes: data_units_kept」）：

- `research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:52`：「MUTC_SWEEP histories=420 ran=374 not_reachable=46 images_judged=3154 red_histories=188 first_red_invariants={'I-4.6': 188}」
- `research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:54`：「MUTC_CRASH histories=2 prefixes=172 crash_image_red=59 mount_refused=56 after_mount_red=90」

没红的 186 段是批大小 144 或只删空 inode 的历史，走不到「不是最后一批」那一步。

**取样与估时**：
- 已发布那一组没缩：起初只扫了 3060 段（时间与盘大小两轴只在进程里那一形上扫满），按 3060 段 246 秒（10 线程）估整张叉积 12960 段约 17 分钟，不到 40 分钟，于是补扫了其余 9900 段，分 6 批、每批 1650 段、每批实测 94–141 秒。
- 崩溃点那一组缩了：时间只取「与记录同」、盘只取 4 GiB、发布方式只取两种。缩前（6 × 2 × 3 × 12 种发布方式 × 3 种时间 × 2 种盘）2592 段，缩后 72 段。估时：单线程 1 段 57 个前缀 22.8 秒，每个前缀约 0.4 秒（两次 checker、一次可写挂载），平均每段约 164 个前缀；2592 段 × 164 × 0.4 秒 ÷ 9 线程约 5.3 小时，超过 40 分钟。分 16 批（9 段 × 前缀按序号模 2 分两份），每批实测 47–143 秒。
- **层 0 的枚举域没跑**。一段删除历史自己的闭式状态数：`research/prompts/m3-batch18-code-r1-opus-model/out/probe.out:2`：「closed_form_states=1179648114」；每次删除发布的那一段单元写有 20–26 个（同一行 `segment_lengths`），一段就是 2^20 以上。照原型规则「一段历史自己就超过约 10⁶ 的，那段不跑」，这几段历史的层 0 域一段都没跑。上面的崩溃点只是操作前缀，不是段内任意整写子集，不能替代层 0。这一条已交给主 agent（第五节）。

## 二、漏判：没有指着这棵 inode 树的 `deleted_inodes` 条目（N1、N2）：打中

造法（`badimages.rs`，都在发布路径写出的镜像上改一处、重封、校验和沿引用链补到根）：

| 形 | 改法 | I-9.5 | I-4.7 | I-4.6 | 别的红 |
|---|---|---|---|---|---|
| L1a | 意图那一版（最新）树表里 `deleted_inodes` 条目种类码 9 → 13 | H | N | N | I-9.16 |
| L1b | 同上，整条拿掉 | H | N | N | I-9.16 |
| L1c | 只改第一批那一版（中间版），最新根写对 | H | H | H | I-9.16 |
| L1d（对照） | 意图那一版 `deleted_inodes` 头 ID 改成 extent 树的树 ID | N（判不了） | N | N | I-9.27 |
| L2 | L1a 之上把 1 号 size 加一个净荷（要 4 个单元、有 3 个） | H | N | N（理由不符） | I-9.16 |

原样（`research/prompts/m3-batch18-code-r1-opus-model/out/badimages.out`）：
- 第 19 行（L1a）：「literal_red=I-9.5 checker_verdict=H missed=true reason= violated=["I-9.16"]」
- 第 21 行（L1c）：「watched=I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=H,I-4.6=H,I-4.4=N」
- 第 23 行（L2）：「reason=走读到的每一版 inode 树里都没有 nlink = 0 的记录」

L1c 是要害：中间版的树表丢了 `deleted_inodes` 条目，那一版 1 号 nlink = 0、还有 2 个单元、不在任何队列里，I-9.5 与 I-4.7 都报**成立**（成立是别的版本判出来的，这一版一次都没判、也没记判不了）。冻结代码里判法只从 `deleted_inodes` 一侧去配 inode 树，没有反方向。I-9.16 因为树表条目的发号次序红了，所以这张镜像在层 0 里仍是红的，漏的是这几条自己的判定，与「每条只红自己」那一串归因。

I-9.16 在 L1a–L1c、L2 上红的是它第 ④ 句（树表非空时十一种各恰好一条），L1d 上红的是 I-9.27。这是我在副本上把每条违例说明都打出来的一次性核对（`/tmp/claude-1000/m3-batch18-code-r1-opus/oneoff-details.out`，没进模型，交回前删），原样一行：「L1c I-9.16：根（实例 1、txg 6）的树表有 11 条，种类 9 一条都没有（④ 树表非空时进树表的十一种各恰好一条）」。所以这一形在层 0 里不会漏红，漏的是 I-9.5、I-4.7 自己的判定：该报判不了（或红），报了成立。

**改法 A（只在我的模型上量过、被攻过零轮）**：`judge_deletion_queue_of_a_version` 里先反方向配一遍——这一版每一棵 inode 树（种类 2）都要恰好一条 `deleted_inodes` 条目的头 ID 指着它，不是恰好一条就对 I-9.5、I-4.7、I-4.6 记射程里判不了，理由「走读到的某一版树表里头 ID 指着某棵 inode 树的 deleted_inodes 条目不是恰好一条：那一版判不了」（`fixA.patch`，加一个 `DeletionScanGap` 成员与一段 20 行的循环，别处不动）。量的结果：
- 坏镜像：`research/prompts/m3-batch18-code-r1-opus-model/out/fixA-badimages.out:21`（L1c）：「watched=I-9.6=H,I-9.5=N,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N」；L1a、L1b、L2 同样转成 N、理由是上面那一句（第 19、20、23 行）。
- 合法镜像：整套已发布那一组在改法 A 上逐段结果相同，`research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:33`：「FIXA_SWEEP histories=3060 ran=2976 not_reachable=84 images_judged=24268 red_histories=0 first_red_invariants={} history_lines_sha256_16=dac44603ca43672e」，与原样那一份（第 1 行）的逐段摘要 sha256 前 16 位相同。
- 副本上的 harness 用例：`research/prompts/m3-batch18-code-r1-opus-model/out/summary.txt:50`：「FIXA_TESTS test result: ok. 86 passed; 0 failed」，第 51 行「12 passed」（`checker_known_bad_images` 与 `pool_checker_judges_the_deletion_state_of_every_walked_version` 两个目标）。副本上的数不算入库装置上的数。
- GPU：改法 A 只加「判不了」，判不了在 GPU 那一侧本来就不置位，违例位不变（推的，没跑卡）。

改法 B（推的，没实现）：没有配上 `deleted_inodes` 的 inode 树按空队列判，L1a–L1c 就红 I-9.5（② 方向）与 I-4.7。它对「报不适用」那一分句也起作用，但让这一形多红两条，「只红自己」那一串要跟着改。

## 三、「只红自己」：每份坏镜像换一种造法再造：没打中

| 形 | 换的造法 | 只红 | 原样（`research/prompts/m3-batch18-code-r1-opus-model/out/badimages.out` 行） |
|---|---|---|---|
| V1 | I-9.5 ①：改在第一批之后那一版（最新），队列 {1} 改 {1,7} | I-9.5 | 第 1 行「only_red_itself=true violated=["I-9.5"]」 |
| V2 | I-9.5 ①：队列里加 2 号，2 号活着、nlink 1（C708 还差的那一形） | I-9.5 | 第 3 行同上 |
| V3 | I-9.5 ②：只在意图那一版把空 inode 2 的 nlink 改 0，最新根写对 | I-9.5 | 第 5 行同上 |
| V4 | I-4.7：三个单元（有下段）的文件拿掉记录、单元留着 | I-4.7 | 第 7 行「only_red_itself=true violated=["I-4.7"]」 |
| V5 | I-4.6：意图那一版 size 2.5 净荷改 1.5 净荷（改小，不是改大） | I-4.6 | 第 9 行「only_red_itself=true violated=["I-4.6"]」 |
| V6 | I-4.6：意图那一版 size 改 0、单元 3 个 | I-4.6 | 第 11 行同上 |
| V7 | I-3.4：标签 4 非 0 改在意图那一版（最新） | I-3.4 | 第 13 行「only_red_itself=true violated=["I-3.4"]」 |
| V8 | I-3.4：只在第一批那一版拿掉标签 4 那一行，最新根写对 | I-3.4 | 第 15 行同上 |
| V9 | I-9.6：删完之后记账 inode 号水位 2 改 1 | I-9.6 | 第 17 行「only_red_itself=true violated=["I-9.6"]」 |

V9 只红 I-9.6，但红的是前一半（第 18 行：「I-9.6：记账里的 inode 号水位 1 不大于 inode 树内最大 key 1」），墓碑那一半没被单独隔离出来：最新那一版 inode 树已经是零条目根，「树内最大 key 1」应是主走读走候选集里还带着 1 号记录的旧版时数进来的（推的，没逐行核主走读）。想单独打墓碑那一半，要墓碑对象 ID 大于候选集里每一版的最大 key——正文已有那份（墓碑对象 ID 抬到水位）就是这一形，我没再造第二种。

## 四、N8：GPU 判器内核对 P1 的 CPU 判法

**逐支对（读代码）：没找到分叉**。对过的支：判哪几版（候选集每条根，`judge_deletion_state_of_the_versions` 在扫描内核里排在 I-5.5 之后，`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:2535`：「judge_deletion_state_of_the_versions();」）；树表读不出、0 条、条目宽不到 200 不判；只从 `deleted_inodes` 条目去配（同一文件 2421 行：「== TREE_KIND_DELETED_INODES) {」）、配到的 inode 树不是恰好一条就不判（2317 行：「if (inode_tree_entry_count != 1u) {」）；队列根层级、key 宽、条目宽；inode 树按条目序深度优先、同号后读到的盖掉先读到的（倒着压栈，弹出次序与 CPU 递归一致）；extent 树按号数单元、全零数据指针不数；I-4.7 只看有单元的号；I-4.6 的 0 与 ⌈size ÷ 净荷⌉；I-3.4 整棵读完才判、缺行与非 0 都红；墓碑那一半先要有水位行、读不全不判、已发布谓词 `w_published` 与 CPU 的 `holds_for` 逐支相同（2508 行：「if (!w_published(container.birth_txg, container.instance, false, vec2<u32>(0u, 0u))) {」，`container.instance` 取的是码 3 头偏移 93，`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1494`：「p.instance = rd32(base, 93u);」，与 CPU 的写序实例代号同一处，`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker/src/walk.rs:175`：「const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;」）。读出来的节点 `base` 指着版本字节表、不是可复用的草稿槽，先读的树表与队列根在后面读别的单元时不会被盖掉。CPU 按树表单元去重只扫一次、GPU 不去重：违例位是并集，结果相同。

上面第二节那一形（没有 `deleted_inodes` 条目）GPU 同样跳过，两边一致地漏；改法 A 只加判不了，GPU 不用跟。

**只比违例位的对拍漏了什么：打中（覆盖缺口，不是判法分叉）**。

1. **展开上限 8 让每个发布段一个状态都不比**。对拍按段展开，段长不超过上限的展开全部真子集，超过的不展开（`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:1872`：「Layer0SegmentExpansion::NotExpanded」），上限缺省是 8（同一文件 74 行：「const DEFAULT_EXPAND_UP_TO: usize = 8;」）。我用 CPU 算了删除线那条流（新池新建文件 → 顺序写三个单元 → 意图 → 批大小 1 的第一批，与对拍的 `record_new_pool_through_the_first_deletion_batch` 同一串发布）的切段：`research/prompts/m3-batch18-code-r1-opus-model/out/gpugap.out:1`：「plan_states_limit8=141 plan_states_full=1174405257 not_expanded_segments=[(7, 24), (11, 30), (15, 26), (19, 24)]」。四个没展开的段就是新池新建文件、顺序写、意图、第一批那四次发布的单元写，删除态扫描的 GPU 代码在「这一次发布写了一半」的状态上一次都没被比过。
2. **CPU 在那几段上的期望**（段内只差一个写的状态，各段之前的段全落）：第 2–5 行，四段共 104 个状态，`cpu_red_states=0`；例如意图那一段（第 4 行）：「states_all_but_one=26 cpu_red_states=0 cpu_watched={"I-9.6=H,I-9.5=H,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N": 26}」。GPU 那一侧在这 104 个状态上该给全 0 的池级 checker 违例位；这是 CPU 算的期望，GPU 没跑。
3. **6 条只比违例位，丢掉的不只是恢复结局**。`compare_the_checker_on_a_deletion_flow`（同一文件 4619 行：「fn compare_the_checker_on_a_deletion_flow(」）只断言 `checker_mismatches` 为空（4635 行：「comparison.checker_mismatches.is_empty(),」），而进 `checker_mismatches` 的条件只有判不了或池级 checker 违例位不同（2024 行：「if undecidable || gpu_checker_bits != *cpu_checker_bits {」）。第 0 字的其余位（两遍 oracle 的七类、根在而记录都不在、恢复自称的状态少单元）与恢复那几个字整块不比。C712 只解释恢复结局对不上（GPU 恢复内核不看 nlink），记录核对器那两位与「不看 journal 那一遍」的 oracle 在删除中间态上有没有别的分叉，这 6 条看不见（推的：我没跑卡，说不出有没有）。
4. **这 8 条新流都走不到的 GPU 分支**（推的，按流的构造读）：每条流只有一次 unlink，墓碑树是层级 1 的根加一个容器，所以墓碑那一半「类型段 0 往下走一层」那一支与已发布谓词滤掉容器那一支（`continue`）没被比过；inode 树都只有一片叶容器，「几片叶、同号后读到的盖掉先读到的」的出栈次序没被比过；回退、挂载续做、两次 unlink 的版本都不在这几条流里。p2 那 11 行变异都钉在这几条流上，只在上面这几种状态里出错的 GPU 改坏抓不到。

## 五、N5、N9

**N5（I-9.6 墓碑那一半的「已发布」）：没打中**。墓碑那一半读的是最新那条根的树表（`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker/src/walk.rs:9293`：「&roots[newest_index].2.record_bytes[ROOT_RECORD_TREE_TABLE_POINTER],」），容器按 I-1.2 那一份已发布谓词滤（同一文件 8592 行：「let is_published = published.holds_for(」）。条款要的是「被抛弃时间线的墓碑不算」（`.claude/kb/invariants.md:293`）：崩溃恢复落到旧根之后，被抛弃那条线写的墓碑容器不在所选根的墓碑树里，自然不算；管理员回退那次发布的墓碑树随现行那一版（`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-core/src/mount.rs:6273`：「墓碑树随现行那一版」），回退掉的那一段时间线上的墓碑留在树里、照算，与「水位取 max、不重发」一致。所选根墓碑树里还能被谓词滤掉的容器，只有写序实例代号高于挂载根、或诞生代号高于实例表行的那种，它被最新根引用就已经是 I-4.2 那一形（推的，没造镜像）。已发布那一组 102348 份镜像上 I-9.6 都判过、成立；回退到删除串各版那几十段历史也在里面。

**N9（`--device-bytes` 非 4 GiB）：没打中**。bin 本身属于 checker 档、我没直接执行；我在原型里走 bin 调的同一个库函数（`run_new_pool_file_creation`，同一组 `e142_parameters(512, 512)`、同样两块稀疏盘），比新池新建文件那一窗的每一个写：
- `research/prompts/m3-batch18-code-r1-opus-model/out/n9.out:4`（8 GiB）：「regions=29 same_offsets_and_lengths_as_4GiB=true regions_with_different_bytes=9」；3、4 GiB + 64 KiB、6、16、64 GiB（第 1–3、5、6 行）也都是 29 个区域、偏移长度不变、同样 9 个区域字节变（两盘各三个 16 KiB 节点、两盘各一条 4096 字节、盘 0 上 512 字节的根槽；推的：记账里的每盘空闲随盘大小变，校验和沿引用链级联上去）。
- 第 7 行（4 GiB + 256 字节，不是扇区整数倍）：「regions_with_different_bytes=0 differing=[]」——容量按槽向下取整，与 4 GiB 一样。
- 第 8、9 行：1 GiB 与 0 被 mkfs 拒（`JournalRingTooLargeForDevice`）。
分配器的每盘大小取盘自己报的（`/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-core/src/make_filesystem.rs:721`：「device.size_in_bytes(),」），bin 只把 `device_bytes` 交给稀疏盘（缺省一支在 `/tmp/claude-1000/m3-batch18-code-r1/tree/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump.rs:96`：「[] => Ok(TEST_IMAGE_DEFAULT_BYTES),」），路上没有写死 4 GiB 的地方。

小出入（推的，没跑 bin）：文件头说「退出码：0 = 正常；1 = 命令行参数不是上面两种之一。」（同一文件 19 行），而 `--device-bytes 0` 这种解得开、mkfs 拒的值走到「在内存盘上跑得通」那一个 `expect`（168 行）panic，退出码是 101，不是 1。只影响给错盘大小时的报法。

## 六、丢哪块盘

已发布那一组里进程内删除的 90 段历史（4 GiB、时间与记录同、删完不再建），每次发布之后的镜像各丢一块盘（整块读出全 0），共 1082 份：删除线五条与 I-9.6 一处都没红（`research/prompts/m3-batch18-code-r1-opus-model/out/diskloss.out` 末行：「DISKLOSS_SUMMARY histories=90 images_judged=1082 histories_with_watched_red=0」）。红的是别的条（I-2.1、I-4.8、I-7.4、I-3.1、I-9.1、I-7.3），那是丢盘本来该红的。

## 七、几个改法各修哪一格

| 改法 | 修的格 | 量过 / 推的 | 原样或依据 |
|---|---|---|---|
| A：判法反方向也配，没有恰好一条 `deleted_inodes` 指着的 inode 树记判不了（`fixA.patch`） | L1a–L1c、L2 的 I-9.5、I-4.7「报成立」与 L2「不适用理由不符」 | 量过（副本） | `research/prompts/m3-batch18-code-r1-opus-model/out/fixA-badimages.out:21`：「watched=I-9.6=H,I-9.5=N,I-3.4=H,I-4.7=N,I-4.6=N,I-4.4=N」 |
| A | 合法镜像上不添误红、不改任何一段的结果 | 量过（副本） | `summary.txt:33` 与第 1 行逐段摘要相同：「history_lines_sha256_16=dac44603ca43672e」 |
| A | 漏判判据「报不适用」那一分句 | 量过：仍中 | 改法 A 把它变成判不了，判不了在没有违例时就是不适用 |
| B：没配上 `deleted_inodes` 的 inode 树按空队列判 | L1 各形红 I-9.5、I-4.7 | 推的 | 没实现；会让这一形多红两条，只红自己那一串要跟着改 |
| C：删除线那几条对拍流在没展开的四段上加「只差一个写」的 104 个状态 | 第四节第 1、2 条的覆盖缺口 | 推的 | CPU 期望已算好（`gpugap.out` 第 2–5 行），GPU 没跑 |
| D：6 条只屏蔽恢复结局对不上的那几位，第 0 字其余位照比 | 第四节第 3 条 | 推的 | 没实现，没跑卡 |

上面「量过」都只在我的模型上量过、被攻过零轮；副本上的数不算入库装置上的数。

## 没打中的形状

- 合法删除中间态（第一节）：文件 0–6 个单元、批大小 1/2/144、unlink 第一个文件 / 空 inode / 先空后文件 / 先文件后空、进程里 / 会话 / 不卸载重挂 / 卸载再挂 / 停在第 0–3 批再挂 / 管理员回退到删除串第 0–3 版再挂、三种调用方时间、4 与 8 GiB、删完再建再删：12960 段（走得到 12456 段）、102348 份已发布镜像，0 红。
- 崩溃点：72 段历史 11832 个操作前缀，崩溃镜像与挂载续做之后的镜像 0 红、挂载 0 次被拒。
- 丢盘：90 段历史 1082 份丢一块盘的镜像，删除线五条与 I-9.6 0 红。
- 只红自己：V1–V9 九种换法，九种都只红目标。
- 读不全夹在中间、头 ID 配错（L1d）：照共用判法报判不了、有理由。
- GPU 内核逐支读：没找到分叉。
- N5 已发布谓词：与条款一致。
- N9 七种盘大小：随盘大小算，没有写死 4 GiB。
- 没造、只按代码推的：extent 树头 ID 配错（推的：记 `ExtentTreeOfTheInodeTreeNotSingle` 判不了）；墓碑树里塞进被抛弃那条线的墓碑容器（推的：谓词滤掉，被最新根引用就是 I-4.2 那一形）。

## 这条腿自己的限度

1. 层 0 的枚举域一段删除历史都没跑：每次删除发布的单元写那一段有 20–26 个写，一段就是 2^20 以上，整段历史闭式 1179648114 个状态，超过原型规则的约 10⁶。第一节的崩溃点只是操作前缀，段内「记录落了而单元缺一个」这类子集一个都没碰，读不全那一支（`deletion_scan_unreadable_verdicts`）在 11832 个崩溃点上一次都没出现过。这几段历史的层 0 是不是全绿，我答不了；要跑的话是 checker 档的重活，交主 agent。
2. 崩溃点那一组缩了取样（时间、盘大小、发布方式），缩法与估时在第一节；已发布那一组没缩。
3. 漏判判据的「报不适用」分句与共用判法「配不上的版记判不了」打架：照判据字面，L1d 这类设计上的判不了也算漏判。这是判据本身要改还是共用判法要改，归主 agent 判（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 里「打中归错了判据」那一形）。我只把「报成立」那几格算打中。
4. GPU 一侧全是读代码与 CPU 期望，没跑卡；「有没有恢复结局之外的分叉」我只说得出对拍看不见，说不出有没有。
5. 我是一次抽样：没打中的那几格照 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 不能单独当「没打中」用。
6. 改法 A、C、D 与我的「会红」注入（`mutC.patch`）都只在我的模型上，被攻过零轮。

## 没做什么

- 没判正推腿那几格（K1、K3），没替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没跑任何 checker 档的东西：没起 `singlefs-checker-tier` 的测试目标、没直接执行它的 bin（E142 导出 bin、`deletion_write_dump`），GPU 一张卡都没用；只把它的库当依赖调了 `layer0_plan_state_count`。层 0 枚举域一段都没跑（限度第 1 条）。要判「删除中间态的层 0 全绿」，得由主 agent 在提交时带 `SINGLEFS_HEAVY_TESTS` 跑 checker 档；GPU 对拍补那 104 个状态（改法 C）同样归它。
- 没造 extent 树头 ID 配错、被抛弃那条线的墓碑容器两形（只推了判定）。
- 没核 V9 里「树内最大 key 1」从哪一版来（推的）。

## 内存包装与负载的记录

开工时 `ps` 看到主 agent 的 E161 可行性档与 E142 变异表、别的会话的 `cargo test`，没有性能测量在跑；内存 slice 常被别的活订满，几次排队。包装自己的结局（这几次的输出都不算结果，改上限或等位之后重跑）：

| 命令（经 `scripts/hosts/run-with-memory-cap.sh`） | 退出码 | 怎么了 |
|---|---|---|
| `3G cargo build --release -p m3b18-opus-proto`（等 220 秒） | 252 | 排不上 |
| `12G target/release/m3b18-opus-proto sweep` | 252 | 排不上；改 4G 重跑 |
| `6G cargo build …`、`4G cargo build …`（改法 A 副本） | 251 | 超过峰值表的 2 倍 |
| `auto cargo build …`（改法 A 副本，新编译目录） | 250 | 撞了 1322M 上限；改 3G 加理由重跑 |
| `3G … crashpoints 18 27`（第 1 份） | 251 | 超过峰值表的 2 倍；改 1G 加理由重跑 |
| `2G … sweep 4125`（补扫） | 252 | 排不上 |
| `1G … sweep 4125`（补扫） | 250 | 撞了 1G 上限；改 3G、分批改成每批 1650 段重跑 |
| `1G … badimages`（一次性核说明） | 251 | 超过峰值表的 2 倍；加理由重跑 |

另有一次全量 sweep 在前台超过 240 秒、被工具挪到后台跑完（退出码 0），之后每批都按估时切到前台跑得完。

## 草稿与副本

草稿目录 `/tmp/claude-1000/m3-batch18-code-r1-opus/` 里建过的三份仓副本、四个编译目录、三遍从头复跑的工作目录与一次性核对的输出，交回前都删了；能复跑的全部东西在模型目录里。
