# c245-r3 本地攻方腿 译文核对表

英文项 / 原文文件:行 / 首稿缺的 / 定稿。逐句核对英文提示 `research/prompts/c245-r3-local-attack.md` 与 kb 原文；多出来的限定词、括注单列一行并写明为什么加。

| 英文项 | 原文文件:行 | 首稿缺的 | 定稿 |
|---|---|---|---|
| P1 | decisions/16-发布语义.md:179 | 无 | 「恒为」译成 always，「等...才」译成 does not return until，完整 |
| P2 | decisions/16-发布语义.md:179 | 无（省去「进层0枚举」「段序列因此唯一」两个独立分句，与本轮算术无关，未改已译分句的限定词） | 只译「更新时点」与「每个checkpoint一次」「最后一步」三点 |
| P3 | decisions/16-发布语义.md:184 | 无（省去「比D25推导的两个多两个」「知情接受」两个独立分句，理由同上） | 两道FLUSH+根槽FUA+一道屏障=四个序点，逐字 |
| P4 | decisions/16-发布语义.md:156 | 无 | 「把checkpoint_txg加一」「fsync触发的发布也是发布」「没有小发布不记号的例外」三处限定词都在 |
| P5 | decisions/23-journal的角色与格式.md:123 | 无 | 「同一个频率」「同样要写」两处限定词都在 |
| S1 | layout/01-first-txn.md:403 | 无，但加了推导出的计数 | 加字：t1–t12 标 12 个单元、t13/t14/t15 各标 1，原文只写标号范围没有写「12」「1」这几个数；根据同一行的标号区间与单数标号机械数出，不改变原文的 24/2/1/2 分段合计 |
| S2 | layout/01-first-txn.md:403 | 无，但加了推导出的计数 | 加字：kind 串里 root_record_fua 后补 times 1，原串对单数项不写倍数；省去「16777223个崩溃状态」，与本轮算术无关 |
| S3 | layout/01-first-txn.md:385 | 无 | 「至少w=2列」译 at least two disks，「两盘各一份」译 one copy on each of two disks，完整 |
| S4 | layout/01-first-txn.md:386 | 无，但加了引导语 | 补的「for background only, not part of this publish」是加的连接语，原文括注本身没有这句提示语，加是为了不让模型把取号、暖机那两次写误算进这次发布的代价里 |
| C1 | decisions/22-单元原子性怎么合成.md:190 | 无 | 「46行」「合计489字节」「住4096字节」「槽内余3607」四处数字都在，省去「（已定项2）」这个指针 |
| C2 | decisions/22-单元原子性怎么合成.md:197 | 无 | 五个子字段的字节数逐一对上 |
| C3 | decisions/22-单元原子性怎么合成.md:199 | 无 | 「跨过之后一次写可能落一半」「覆盖整槽4096含补齐」「校验和过且世代号最大」「自动回退」「可检测可恢复」五处限定词都在 |
| E1 | experiments/23-journal几何.md:15 | 首稿把「jbd2」「原地覆盖」两个限定词漏了 | 定稿补回「住journal superblock、原地覆盖、FUA」；省去「jbd2」这个类比名字，不影响算术，只是命名 |
| E2 | experiments/23-journal几何.md:16 | 首稿漏了「XFS」这个类比名字 | 省去「XFS」类比名字，理由同 E1；「内联在每条记录头的tail_lsn」译全 |
| E3 | experiments/23-journal几何.md:18 | 无，但加了 hedge 词 | 加字：steady-state overhead is stated as 0.105 percent 里的 stated as 是加的，原文没有这层「据称」的语气，加是为了不让模型把这句話当成待验证的算术前提去重算，而是当成 E23 报告自己的说法；省去「换来空闲崩溃后重放量归零」，与 E1 的 0 那一格重复 |
| E4 | experiments/23-journal几何.md:22 | 无，但加了推导括注 | 加字：(two durability points per fsync, over 20000 fsyncs) 是从「40000次持久点」反推出的算式，原文没有这个括注，加是为了不让模型误把 40000 当成独立给定的数、看不出它是 2×20000 |
| E5 | experiments/23-journal几何.md:95-96 | 首稿漏了「不碰设备」 | 定稿补回 does not touch the device |
