# E162 跑前修订（S3 跨机、S4 掉电）设计员报告（2026-09-27 JST）

## 结论

- 登记：`/home/fy5090/code/singlefs/research/prompts/e162-preregistration.md`，实验号 E162（没占新号，简称沿用），修订写在「十二、修订」里、第十三节标题之前，新增第 433–1037 行一段「补登：S3 跨机与 S4 掉电」（补 1–补 13），前面各节与第十三节一个字没改。写完 sha256 `bb1c378f9a6b8529cd64be07fd4ac2dd6c45bbf818953fb704e4e8d57ca94a30`，1 221 行。
- 写法：正文先排他写进草稿目录 `chunk1.md`…`chunk13.md`，再用 `research/scripts/replace-once.py` 逐段插到第十三节标题前；每段插之前核文件 sha256 等于上一段插完时的值（没有别人同时改）、插之后核「文件 = 插之前那一份 + 这一段」，13 段全过（插入脚本 `insert-chunks.py` 的输出：每段 exit 0、`ok`）。之后又用 replace-once 改了五处本段内的字（更正整条引、四处「第八节」改「补 8」）并在命令表里记了这一笔。
- 回读：从登记里抽出补 13 的锚点脚本重跑，输出与登记里的输出段逐字节相同（`cmp` 过，sha256 `25f9617c…0053ee` / `99d697a1…51e5ee1`）；本段各表列数逐行核过，全对得上；问题单 S3、S4 两行在登记里逐字出现（`grep -cxF` 各 1）；没有撇号类角标。
- 问题单：`research/prompts/m2-crash-store-r1-forks.md` sha256 `8e5f55b3…ae55c2c`（主 agent 改成 17:4x 之后那一版）。第二节被测条款重抽之后与原第二节逐字节相同，没变。

## 登记里写死了什么（一句一项，细节在登记里）

- 装置位置：S1 那份源文件在被改，S3、S4 另起三个 bin——`e162_verdict_store_sender.rs`（无存储依赖，musl 静态，回环与跨机两种接法加中继）、`e162_verdict_store_network.rs`、`e162_verdict_store_power_cut.rs`（都挂 `e162-block-stores`）；两张新变异表。都在 `research/e7-index-bench`，不碰 `crates/`。
- S3 回环：原 6.3 那套不动，补一个反向取样点 S3-W256（原 8.2 的 S3 两个取样点同向）与分窗轨迹 Q3k。
- S3 跨机：B = `faliye-jplife`，只走 ssh stdio（A 上库进程起 `ssh -o BatchMode=yes faliye-jplife <中继>`），中断 = 库进程 SIGKILL 自己的 ssh 客户端；格 S3X-T / S3X-C / W1 / W256 / bs10 / busy；对照 PC3-drop-X、PC3-torn-X、PC3-rate-X（按 100 ms 压节奏送 200 块，钉在 [9.0, 10.051] 块/秒）；B 上只放一个静态二进制在 `/tmp/e162-s3x-<pid>/`，收尾三步（按自报 pid 停进程、核路径后 `rm -rf`、`test ! -e` 回读）。
- S4 断电模型：QEMU blklogwrites 设备侧日志重放；断电点 k 的 U-drop 盘面（最后一个 FLUSH 之后的写全丢）是判定读法，U-keep / U-rand / U-tear 与 S4-bs10 是取样点；「已确认」用来宾在 vda2 上的 O_DIRECT 标记扇区钉到日志位置（只会少算、不会多算）。两次开机：写盘（blklogwrites）与验盘（日志放进附加根，来宾在 tmpfs 上重放、loop 挂 ext4、子进程开库逐块核）。
- 断电点：按 D13 已定项 9（`13-验证路线.md:178`「分桶而不是均匀随机」）分三桶取 200 点（普通提交里 / 长窗提交里 60 / 两次提交之间 40），另逐边界枚举 8 块提交（至多 512 点）；为什么够写在补 5.2（检出力 B6）。
- S4 阳性对照：PC4-C（模型级，精确计数 100 / 0 / 47）、PC4-R（整盘重放与 disk0.img 同 sha256）、PC4-L（R-None、K-nosync、F-nofsync：不 fsync 必丢，U-keep 下 20 点 ≥ 10）、PC4-NB（`barrier=0`，U-drop 下 20 点 ≥ 10）、PC4-FD（F 只拿掉目录 fsync）、PC4-S/P/O（钉绝对值）。
- 重型测试：第四段执行员只写装置、驱动（装置的 `s4-drive` 角色，环境变量 `SINGLEFS_HEAVY_TESTS` 不是 `user-request`/`commit` 就拒跑）、单测与变异表，给 `--dry-run`；第五段由主 agent 带 `SINGLEFS_HEAVY_TESTS=user-request` 起，整条命令在补 12。
- 分段：第三段 S3（回环；跨机要派发提示带着补 5.3 前提格的现查结果才开）、第四段 S4 装置、第五段 S4 断电（主 agent）、第六段 S4 判定；三、四段可并行（只共改 `Cargo.toml` 末尾，别同时改）。

## 待主 agent 认的项

1. 另起三个 bin 与它们的英文名（补 5 开头）：这是对 1.3「同一个 bin 按角色起」的改动，理由是 S1 源文件正被另一个执行员改。
2. S4 跑哪几条臂（派发时按第一段交回点名，出局的标「够判后未跑」）；内核用 OEM（`vm-kernel.sh --release`，要口令）还是 lockdep。
3. 补 5.3 前提格（B 的架构、nproc、内存、有没有 Rust、/tmp、负载、链路；S4 的宿主内存 ≥ 16 GiB、盘 ≥ 40 GiB）由主 agent 现查，结果放进第三段③的派发提示；B 的清场（`QUIESCE_*`）停不停由你定。
4. S3X-busy 会让 B 的全部逻辑 CPU 空转一趟（S3X-T 那么长），认不认。
5. 跨机格的吞吐对照换成钉绝对值的 PC3-rate-X（不照搬回环的「睡 5 ms、不睡那遍 > 200 块/秒」），理由在补 5.1。
6. F12：某条臂只在 U-rand / U-tear / U-keep / bs10 上翻时算不算出局，登记留给你定，没替你判。
7. 补 1 里整条引了你的更正（含「18:1x」四个字作为被更正的那个时刻）；用户定案的时刻处处写的是 17:4x。要连引文里也不出现「18:1x」就删那一行。
8. 门禁 86 号：这次没占新号；E162 实验页在禁读清单里、我没看它在不在，86 号红不红不知道。

## 什么现象会推翻这份设计里的前提

- PC4-C 对不上（例如 fdatasync 之后日志里没有 FLUSH 条目、来宾 `write_cache` 不是 `write back`）→ 段模型套不到这台虚拟盘上，S4 全作废（V17、V23），要回来重想断电模型。
- PC4-R 过不了（QEMU 关机时没更新日志系统配置里的条目数之类）→ H9，解析与 `device_log.rs` 两边查。
- ext4 6.17 不认 `barrier=0` 或认了照样发 FLUSH → PC4-NB 记「不成立」，写缓存那一半只剩模型级的 PC4-C 撑。
- 验盘附加根装不下日志（1 GB 量级）或 12 GiB 来宾内存不够 → H10，要改 vm-bench 或换重放方式（不在执行员写范围）。

## 读过的文件、跑过的命令

整张清单在登记补 12（第 802–883 行：「读过的文件」「跑过的命令」两段）。没读：`research/results/` 任何文件、`.claude/kb/experiments/` 下任何页、实验索引与实验变更史、S1 那份源文件与它的变异表内容、E161 任何东西；没有 ssh 到 B。

## 没做什么

- 没写代码、没编译、没跑装置、没起 QEMU（重型测试：不跑）、没 ssh 到 B。
- 没核的（写成前提或作废 / 停机条款交给跑的时候）：QEMU 8.2.2 的 blklogwrites 什么时候更新日志系统配置里的条目数；ext4 6.17 对 `barrier=0` 的处理；来宾 `fua` 是否为 0；redb 4.3.0 / rocksdb 0.25.0 在来宾里的动态链接能不能搭起来；B 能不能跑 musl 静态二进制；K 的 memtable 落盘次数（B9 是推的）。
- 没跑门禁：`stage-owners.tsv` 里没有登记给 experiment-designer 的阶段（`awk` 输出为空）。
- 登记之后要改，由主 agent 按 `evidence-discipline.md`「臂的定义也在「跑前写死」之列」那三步办。
- 草稿目录 `/tmp/claude-1000/e162-s3-s4-designer/` 里留着 chunk、锚点脚本与输出、插入脚本、第二节重抽件；没有编译目录、工作树或仓副本。
