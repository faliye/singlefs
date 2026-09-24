# E160 prior-art：公开负载特征里，随机小读占多少

写于 2026-09-24 09:47 JST（写作时 UTC 2026-09-24 00:47）。全部内容来自外部文献与外部工具，
**未在本项目验证**，不许当结论用，只能当线索（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`
「别的项目怎么做，是线索不是证据」）。

## 查法

本机没有 SNIA IOTTA / MSR Cambridge / FIU / Alibaba / Tencent 的 trace 文件或论文镜像
（`find /home/fy5090/code/fs-refs -iname '*fio*'` 与目录列表只有 zfs 自带的 `.fio` 作业文件和
Linux 内核文档，没有存储负载特征论文）；这些引用一律走 WebFetch / WebSearch 现查，写 URL 与取得日期。

fio 本身在本机已装（`fio --version` → `fio-3.36`，Ubuntu 包 `fio 3.36-1ubuntu0.1`），
`/usr/share/doc/fio/HOWTO.rst.gz` 是这一版自带的文档，本机可查；`examples/` 目录不在这个
Debian 包里（`dpkg -L fio | grep -i example` 空），常见作业文件从 fio 上游 GitHub 现拉。

论文 PDF 用 WebFetch 抓不出文字层（几篇都报「binary/compressed，读不出」）时，改用 Bash 里的
Python：正则取出 PDF 的 `stream…endstream` 段、`zlib.decompress`，再从解压出的内容流里用正则
抠 `Tj` / `TJ` 操作数里的括号字符串拼成文本。这条路子在下面四篇论文（事实 3、5 与「查了没查到」里的两篇）上都验证过能拿到可读文字
（贴的引文都是这样抠出来的原文，行内数字与百分号都核对过上下文没有被这条正则误吞）。

## 事实表

| # | 事实 | 出处与日期 | 口径 | 与本工程的差异 |
|---|---|---|---|---|
| 1 | fio 上游 `examples/iometer-file-access-server.fio`：`bssplit=512/10:1k/5:2k/5:4k/60:8k/2:16k/4:32k/4:64k/10`，`rw=randrw`，`rwmixread=80`，未设 `percentage_random`（HOWTO：「This defaults to 100%, in which case the workload is fully random」）| `https://raw.githubusercontent.com/axboe/fio/master/examples/iometer-file-access-server.fio`，本机 `curl` 于 2026-09-24 取得（exit=0，内容见下方逐行抄录）；`percentage_random` 定义抄自本机 `/usr/share/doc/fio/HOWTO.rst.gz`（fio 3.36 版随包文档） | **定义值，不是实测**：这是一份 fio 作业文件的静态定义，逐字段可核算，不是某次真机跑出来的分布 | fio 的 `bssplit` 按次数（每个 I/O 一次抽样）分配大小权重，不区分「这次 I/O 属于哪次用户写事务」；本工程的口径按 fsync/事务分组（D25 已定项 5：块数口径），两者的「一次」指的不是同一个单位 |
| 2 | 由事实 1 的定义精确算出（本报告现算，非引用）：≤16 KiB 的 I/O 占**全部请求数的 86%**（512B10+1k5+2k5+4k60+8k2+16k4=86，因为 `bssplit` 的百分比本身就按次数分配，且读写共用同一份 `bssplit`，所以读、写、读+写三个口径下这个 86% 都成立）；按字节权重算，≤16 KiB 的请求占**总字节数的 30.7%**（512×10+1024×5+2048×5+4096×60+8192×2+16384×4 = 348160；分母 512×10+…+65536×10 = 1134592；348160/1134592 ≈ 30.69%，读、写、读+写三个口径下同样的比例，因为两者共用同一个 `bssplit`，没有分读写两套权重）| 同上；本报告的算术逐项写在本行「事实」一列；算术用的那份作业文件原样存在 `research/prompts/e160-prior-art-sources/iometer-file-access-server.fio` | **按 I/O 次数、按读字节、按总用户字节三种口径都能给**，因为这是一份完整定义（不是采样出来的边际分布），且这份作业文件本身 100% 随机（`percentage_random` 未覆盖，默认值 100） | 这份 job file 模拟的是「IOMeter File Server」这一种合成负载（多用户共享文件服务器，读写混合、大小混合、全随机），不对应任何真实应用轨迹；本工程要问的是自己的 fsync 触发写在真实调用模式下的大小分布，这份定义值不能替代 |
| 3 | Kavalanekar 等（Microsoft）IISWC 2008《Characterization of storage workload traces from production Windows Servers》Table I：14 组微软生产服务器块层轨迹（LM-TFE / LM-TBE / DAP-DS / DAP-PS / Exch-5 / Exch-24 / MSN-CFS / MSN-BEFS / WBS / DTRS / RAD-AS / RAD-BE / TPC-C / TPC-E）。Avg Req Size 一行：TPC-C 8.53 KB、TPC-E 8.38 KB、WBS 27.44 KB、DTRS 25.05 KB；Req Size Modes 一行：TPC-C 与 TPC-E 众数都是 8 KB，WBS 众数 4 KB 与 32 KB，DTRS 众数 4 KB 与 64 KB；`% Seq IOs Initiated`（原文定义：「includes only those uninterrupted by nonsequential requests」）一行：TPC-C 2.10%、TPC-E 1.55%、WBS 7.79%、DTRS 21.80% | `https://iiswc.org/iiswc2008/Papers/012.pdf`，WebFetch 于 2026-09-24 取得 PDF 二进制、本机脚本解压抽字得到可读文字后核对 | **块层**，按 I/O 次数（Avg Req Size 与 Modes 是大小的边际分布，`% Seq IOs Initiated` 是「这次 I/O 是否紧接上一次同盘请求」的按次数占比）；**没有字节口径**，论文没给「≤某大小的请求占多少字节」这类统计，也没有把「小」和「随机」交叉在一起报的数 | 这批轨迹（LM/DAP/Exch/MSN/WBS/DTRS/RAD/TPC）**不是** MSR Cambridge 轨迹，是微软另一批生产服务器轨迹，两者都挂在 SNIA IOTTA 但是两个不同的数据集，本报告不把两者混为一谈；差异：这些是应用层落到块层之后的混合流量（一台机器上多个服务共享一个卷），本工程要问的是单一文件系统自己的 fsync 写模式，中间隔着一层调度器与文件系统自己的合并/预读 |
| 4 | 同一篇（Kavalanekar 2008）正文叙述：build server（WBS）「Over 3.5 million requests (29% of the total) are for 4 KB … split evenly between reads and writes」；DTRS「has 42% of its requests accessing 4 KB」；WBS 的短距离空间局部性分析：「4% of write requests and 13% of read requests … are exactly sequential (zero offset)」；DTRS「contains 20% read and 32% write sequentiality」 | 同上出处 | **块层，按 I/O 次数**；「4% / 13% / 20% / 32%」用的是「与上一次请求偏移量差为零」这一种更严格的顺序定义，与 Table I 的 `% Seq IOs Initiated`（容许非零但连续）不是同一个口径，两个数字不能互相替换 | 差异同上一行；另外这四个百分比是「恰好顺序」而非「随机」的直接对应量，本报告只把它们当「顺序占比」的参照，不倒算成「随机占比」（100% 减法需要口径完全一致，这里两套顺序定义已经不一致，减法会算错） |
| 5 | 《An In-Depth Comparative Analysis of Cloud Block Storage Workloads: Findings and Implications》（后发表于 ACM Transactions on Storage）比较 AliCloud（作者自采，Alibaba Cloud 生产块存储，1,000 卷，31 天）、TencentCloud（引自 Zhang 等，Tencent 云块存储，4,995 卷，约 9 天）、MSRC（即 MSR Cambridge 轨迹，36 卷，7 天，SNIA IOTTA `http://iotta.snia.org/traces/388`）三套块层轨迹。Table 2 / Finding A.2：「75th percentiles of read/write sizes」（对全体请求按大小做累计分布，取 75 分位）——AliCloud 12 KiB/16 KiB，TencentCloud 32 KiB/12 KiB，MSRC 64 KiB/20 KiB（原文：「in AliCloud, 75% of reads and writes are no larger than 12KiB and 16KiB, respectively … In MSRC, 75% of reads and writes are no larger than 64KiB and 20KiB, respectively」）| `https://arxiv.org/pdf/2203.10766`，WebFetch 于 2026-09-24 取得 PDF 二进制、本机脚本解压抽字后核对；论文自称与 MSRC 原始来源为 SNIA IOTTA 388 号轨迹，取自条目 `[30] Microsoft. MSR Cambridge Traces. http://iotta.snia.org/traces/388, 2022` | **块层，按 I/O 次数**，是「对全体请求（三套轨迹各自内部）按大小排序取 75 分位」——**只说明「75% 的请求不超过这个数」，没有给出「≤16 KiB 这一刀」正好落在哪个百分位**，也没有按字节加权 | AliCloud/TencentCloud 是多租户云块存储卷（一个卷背后是别的用户的虚拟磁盘），MSRC 是企业数据中心卷；三者都在块设备层，看不到文件系统内部的 fsync 边界，与本工程按「一次 fsync 触达几个叶子」分类的口径不是同一层 |
| 6 | 同一篇 Finding B.9（随机度）：定义「random 请求」＝当前请求偏移量与最近 32 次请求偏移量的最小距离超过一个阈值（论文取 128 KiB，「the read-ahead length of the surveyed drives」），按卷统计 `randomness ratio`＝随机请求数 / 总请求数。原文：「Half of the volumes have at least 33.5%, 42.1%, and 29.4% of random I/Os in AliCloud, TencentCloud, and MSRC, respectively」；「all volumes in MSRC have less than 46% of random requests, while 20.4% and 35.8% of volumes in AliCloud and TencentCloud have more than 50% of random requests, respectively」| 同上出处 | **块层，按 I/O 次数，按卷取中位数/分布**（不是全体请求合并后的单一比例）；随机的判据是「离最近 32 次请求的最小距离 > 128 KiB」，**门槛是 128 KiB 不是本工程要问的 16 KiB**，两个阈值不能互换 | 差异同上一行；另外论文明说「small 和 random 这两条只是并列观察，没有交叉统计」（原文：「Combining with the observation that small-size I/O requests dominate … we see that random and small I/Os are common in all three traces」是定性并列，不是算出的联合分布），**本报告据此不把 A.2 的大小分位数和 B.9 的随机度相乘或相减去凑「随机小读」这一个数**——论文自己没算，本报告也不替它算 |

### 事实 1 引用的原文（逐行抄录，2026-09-24 本机 curl 取得）

```
# This job file tries to mimic the Intel IOMeter File Server Access Pattern
[global]
description=Emulation of Intel IOmeter File Server Access Pattern

[iometer]
bssplit=512/10:1k/5:2k/5:4k/60:8k/2:16k/4:32k/4:64k/10
rw=randrw
rwmixread=80
direct=1
size=4g
ioengine=libaio
# IOMeter defines the server loads as the following:
# iodepth=1	Linear
# iodepth=4	Very Light
# iodepth=8	Light
# iodepth=64	Moderate
# iodepth=256	Heavy
iodepth=64
```

## 查了没查到

| 查了什么 | 怎么查 | 结果 |
|---|---|---|
| FIU（Florida International University）trace 集（Mail / Web-VM / Homes）的 I/O 大小与随机度分布 | WebFetch `http://iotta.snia.org/traces/block-io/391`（SNIA IOTTA 上 FIU IODedup 轨迹页面），2026-09-24：页面只给每份 trace 文件的记录数、文件体积、采集时段（Homes 20 天 1800 万条 2.12 GB；Mail 01–09 各 29 天 4200 万–5100 万条），**没有** I/O 大小或随机/顺序统计 |
| Koller & Rangaswami，FAST'10《I/O Deduplication: Utilizing Content Similarity to Improve I/O Performance》（引入并使用了这三份 FIU trace） | 本机 `curl` 取 `https://www.usenix.org/legacy/event/fast10/tech/full_papers/koller.pdf`（2026-09-24，14 页），同一套解压抽字脚本处理全文；全文关键词计数：`random` 0 次、`sequential` 2 次（两次都在相关工作段落讨论别的系统，不是这三份 trace 自己的统计）、`request size` 0 次 | 论文的 Table 1 只报「静态相似度 / 工作集相似度」（读写字节数、按扇区去重、按内容去重的比例），**不含** I/O 大小分布或随机/顺序占比，这个问题在这篇论文里查不到 |
| Narayanan、Donnelly、Rowstron，FAST'08《Write Off-Loading: Practical Power Management for Enterprise Storage》（MSR Cambridge 轨迹的原始来源论文，36 卷一周） | 本机 `curl` 取 `https://www.usenix.org/legacy/events/fast08/tech/full_papers/narayanan/narayanan.pdf`（2026-09-24，15 页），同一套脚本抽字；全文关键词计数：`random` 0 次、`sequential` 0 次、`request size`/`size of` 0 次，`KB` 命中的 11 处逐条看过，全部是网络带宽（GB/Mbps）或磁盘容量，没有一处是 I/O 请求大小 | 这篇论文的主题是空闲期与省电，**完全没有**报告 I/O 大小或随机度分布；MSR Cambridge 的大小/随机度数字目前只在事实 5、6（后来那篇比较研究论文的重新分析）里查到，原始论文本身给不出来 |
| Tarasov 等《A Nine Year Study of File System and Storage Benchmarking》（文件系统层工作负载研究） | 只跑了 WebSearch，未取全文 | 搜索摘要显示这篇是对**已发表论文用的 benchmark 参数**做的元调查（这些论文的实验设了多大的 I/O、多少顺序比例这类「别人怎么配置基准测试」），不是对某一份真实 trace 的大小/随机度实测；口径对不上本次要问的「负载特征」，**没有再深入取全文**，不引用它的任何数字 |

**反证（「没有任何现役实现／来源这样统计」这一类）**：本次要查的是负载特征的**事实**，不是「有没有实现走某条路」，所以没有天然对应「反证」的命题。为了不漏掉这一节要求，专门问了一遍「有没有任何一份公开来源，把『大小 ≤ 16 KiB』与『非顺序』做成一个联合统计量（而不是各自的边际分布）」——查了 fio 的 HOWTO/示例、Kavalanekar 2008、以及事实 5/6 这篇三方比较论文，**三者都只给边际分布（大小的分布、随机度的分布），没有一份给联合分布**；事实 6 那篇论文原文自己也只是定性并列（见事实 6 最后一句），这算是查到的最接近「反证」的一条：**在这四份查过的公开来源里，没有一份把「随机」和「小」交叉统计成一个数**，但只查了这四家，不能说全领域都没有。

## 五档映射（`.claude/kb/decisions/25-目标负载优先级.md`「#### 已定项 1」）

D25（目标负载优先级）已定项 1 那张表的五档，轴是**一次 fsync 触达的叶子数**与**这些叶子落在几条互不相交的脊柱上**（已定项 1 原文：「目标负载的两个值——一次 fsync 带 8 个叶子，落在 1 条共享脊柱上」；D25 正文另一处对这两个量的说法是「目标负载的两个可量的值（一次 fsync 带几个叶子、那些叶子落在几条互不相交的脊柱上）」）。

**事实 1–6 全部落不进这五档**，理由对六条都一样：这六条事实量的是**块层或作业文件定义里的请求大小分布、随机/顺序占比**，没有一条量得到「一次 fsync 触达几个叶子」或「这些叶子落在几条脊柱上」——这两个量是这个工程自己的索引树/日志几何里的内部结构，block trace 或 fio 作业文件看不到文件系统内部的事务边界，也看不到索引树形状，没有换算关系。**差异不明，只能当线索**用在别处（比如给「随机小读确实在公开负载里普遍存在」这句话找旁证），换不成 D25 五档里任何一档的数。

唯一勉强够得上「至少不矛盾」的一点：事实 3 的 WBS/DTRS（build server，大量小文件创建）与 D25「smallfile 小文件密集」这一档的**名字**（小文件密集）字面对得上，但 D25 那一行「叶/fsync、脊柱、写放大」三列都写着「未测」（依据 D25 正文：「负载表这三列的口径来自 E16 的叶/fsync 模型，而 E114 量的是对象大小与每次发布落多少个小对象，两者不同轴，不许把 E114 的数填进这三格」）——按同一条纪律，事实 3/4 的「4 KB 众数、29%/42% 的请求是 4 KB」同样不是「叶/fsync」这个轴上的数，一样填不进去。

## 没做什么

- 没有论证这些外部数据适不适合本工程，也没有在本项目里验证任何一条（`evidence-discipline.md`：外部实现/数据只能提出假设、指出该测哪条路径，不进正推反推校验）。
- 没有查 Alibaba/Tencent 官方是否还有别的独立发布的 block trace 论文（本次只查到事实 5/6 这一篇由香港中文大学团队做的三方比较研究，它引用的 TencentCloud 数据本身来自另一篇 Zhang 等的论文，那篇原始论文没有单独去查）。
- 没有查 SNIA IOTTA 上除 FIU、MSR Cambridge 之外的其它数据集（Financial1/2、WebSearch1-3 等 UMass/HP 系列 trace）；这些也是常被引用的存储负载特征来源，本次时间只够查上面六条事实，标记为待查而非查过没找到。
- 没有取 Tarasov「Nine Year Study」全文，只看了 WebSearch 摘要就判定它口径不对，没有第二次确认（摘要本身已经说清这是 benchmark 参数调查而非 trace 实测，判定的把握足够，但严格说没有逐字核过全文）。
- 事实 2（fio 定义的算术推导）只由本报告在会话里现算并核对过一遍（Python 脚本跑过一次，结果 86% / 30.69%），没有第二条独立路子复算；这条不是「实测」，是对一份公开定义的确定性算术，按规则不需要走「跑 5 轮」那一套（`test-discipline.md`「单次观测不算数」管的是有随机性的观测，这里没有随机性）。
- 没有把这六条事实换算成本工程可以直接采信的任何百分比：三种口径（按 I/O 次数、按读字节、按全部用户字节）里，事实 3–6（真实 trace）只给得出「按 I/O 次数」这一种口径的边际分布，**给不出按字节的口径**，也没有一条给得出「≤16 KiB 且非顺序」这个联合口径；能三种口径都给的只有事实 1/2（fio 的合成定义），且那是定义值不是实测。
