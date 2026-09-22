# c468-evidence 阶段同步 逐行判定（F4）

- 分组：F4（195 行，来自 `/tmp/claude-1000/c468-evidence-sweep/candidates.tsv`）
- F4 事实（`/tmp/claude-1000/c468-evidence-sweep/facts.tsv`）：`research/scripts/test-environment-check.py` 的 `run_clean` 汇总行（预演「没动任何东西」与清完「clean 汇总」两处）补报「不碰 N 项，其中 M 项只是太新（不到 N 分钟没改），这一次清不掉、下一次门禁会判它们红：逐个点名文件名」；检索词 `残留`
- 结束那一版：暂存区树 `03f2d85185b0d38873d47d6f569276de5c5ca77d`（基准 `d6a9676`），读法一律 `git show 03f2d85185b0d38873d47d6f569276de5c5ca77d:路径`，不读工作区
- 195 行的判定分布：不相干 192 行、事件句不改 3 行（`.claude/kb/checks-owed.md:435` C448、`.claude/kb/checks-owed.md:442` C396、`records/2026-09-19-里程碑二遗留收拢.md:150`）
- 反向核查（F4 那件事实有没有被登记）加一行 M1，判定「要补」

## 逐行判定

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F4 | .claude/kb/checks-owed.md:46 | 不相干 | 这一行说的是欠账C42（残留记录冒充合法前缀），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「&#124; C42 &#124; 残留记录冒充合法前缀 &#124;」是C42正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:47 | 不相干 | 这一行说的是欠账C43（被点名的产物还在不在没人判），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「（上一条时间线的残留） 与 E33（两条」是C43正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:52 | 不相干 | 这一行说的是欠账C49（全称断言只在抽样点验过），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「反向链挡不挡得住残留记录） 的「2 /」是C49正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:57 | 不相干 | 这一行说的是欠账C54（摊销结构没进稳态就取数），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「*不许用事后的「残留净增」当判据**——」是C54正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:142 | 不相干 | 这一行说的是欠账C152（收窄后未发布号重发的扫描重建复核），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「6 原文发现的残留疑点，不是攻击腿打的」是C152正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:251 | 不相干 | 这一行说的是欠账C283（准入失败时不先推发布就报 ENOSPC），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「保留池与推空最坏残留 5 + (N −」是C283正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:335 | 不相干 | 这一行说的是欠账C380（环上有洞时释放代判不出唯一值），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「写过的 txg（残留记录那条层 0 流的」是C380正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:399 | 不相干 | 这一行说的是欠账C451（开新测试周期时旧镜像没人删），与 test-environment-check.py 的 clean 汇总行改动（补报「不碰 N 项，其中 M 项只是太新」）是两件不相干的事，原话「结果还是旧盘面的残留 &#124; 开新周期那一刻」是C451正文自己讲的「残留」，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:435 | 事件句不改 | 这一行是 checks-owed.md 已还清表里 C448（门禁自己留下的镜像让干净检查必红）2026-09-21 那次修 `--produced-after` 分界机制的记录，原话「脚本自带的 113 条断言全过」是那次改动自己的回归自证证据（那天验的是既有断言没被新逻辑破坏），不是「今天脚本共有多少条自检」这句现状话；clean 汇总行新增「不碰 N 项」不改 `--produced-after` 分界机制本身，不受这件事实影响。 |
| F4 | .claude/kb/checks-owed.md:442 | 事件句不改 | 这一行是 checks-owed.md 已还清表里 C396（环境检查脚本没接进里程碑出口与收尾）2026-09-20 接上门禁 77 号那次的记录，原话「红样本只差「临时目录残留」那一类」说的是 89 号判的门禁 77 号红绿夹具，clean 汇总行新增的「不碰 N 项，其中 M 项只是太新」是追加在同一行末尾的新句子，不改门禁 77 号读「判红 N 类」的解析逻辑、也不改这批红绿样本；「/tmp 里 34 个当天跑测试留下的镜像…clean --yes 清掉之后 9 类全绿」是 2026-09-20 那天的现查结果，是那一次发生的事，不是今天的状态，不受这件事实影响。 |
| F4 | .claude/kb/decisions/04-校验和位置.md:54 | 不相干 | 这一行是 04-校验和位置.md 里讨论 journal / 格式设计的决策正文，原话「读那个物理位置的残留（别人的死数据，无语」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/08-核心索引结构.md:199 | 不相干 | 这一行是 08-核心索引结构.md 里讨论 journal / 格式设计的决策正文，原话「择新。⚠️ **残留的疑点**：讲统计量」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/08-核心索引结构.md:247 | 不相干 | 这一行是 08-核心索引结构.md 里讨论 D8（核心索引结构） 已定项 6 的决策正文，原话「不认识统计量标签静默忽略」说的是不认识统计量标签时挂载怎么处理（与 C71 那道闸的射程），跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/13-验证路线.md:34 | 不相干 | 这一行是 13-验证路线.md 里讨论 journal / 格式设计的决策正文，原话「、上一条时间线的残留、槽数与设备数；崩溃」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/16-发布语义.md:39 | 不相干 | 这一行是 16-发布语义.md 里讨论 journal / 格式设计的决策正文，原话「池 − 推空最坏残留 − 滞后量；保留池」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/16-发布语义.md:52 | 不相干 | 这一行是 16-发布语义.md 里讨论 journal / 格式设计的决策正文，原话「= 保留池 + 残留 = 15 + 13」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/16-发布语义.md:258 | 不相干 | 这一行是 16-发布语义.md 里讨论 journal / 格式设计的决策正文，原话「欠**：C42（残留记录冒充合法前缀）（」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/18-块里携带什么信息.md:476 | 不相干 | 这一行是 18-块里携带什么信息.md 里讨论 journal / 格式设计的决策正文，原话「后上一条时间线的残留块会冒充合法块（f2」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:215 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「⇒ 断号之后的残留记录不会消失、只会等」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:224 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「（上一条时间线的残留）：定长环 + 断号」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:226 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「反向链挡不挡得住残留记录）：开链的全部格」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:230 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「欠**：C42（残留记录冒充合法前缀）；」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:238 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「有代号时的样子（残留全数重放）。**这与」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:246 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「滚一格就逐格退回残留全数重放。」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:263 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「实例代号的恢复在残留与新写两条判据上都是」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:266 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「欠**：C42（残留记录冒充合法前缀）。」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:518 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「*上一条时间线的残留**：恢复之后接着写」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:522 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「反例，不需要量；残留那一格的实验在已定项」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/decisions/23-journal的角色与格式.md:524 | 不相干 | 这一行是 23-journal的角色与格式.md 里讨论 journal / 格式设计的决策正文，原话「欠**：C42（残留记录冒充合法前缀）。」说的是日志槽位、时间线或记账语义里的「残留」，跟 test-environment-check.py 的 clean 汇总行怎么报数是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments.md:52 | 不相干 | 这一行来自 experiments.md，原话「（上一条时间线的残留） &#124; 已跑（08-」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments.md:56 | 不相干 | 这一行来自 experiments.md，原话「与序号解耦解不掉残留；打散对齐的是记录长」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments.md:59 | 不相干 | 这一行来自 experiments.md，原话「反向链挡不挡得住残留记录） &#124; 已跑（0」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments.md:158 | 不相干 | 这一行来自 experiments.md，原话「df 扣掉推空残留后两臂都零假性 EN」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/138-按盘回退下界与推空的空间要求.md:6 | 不相干 | 这一行来自 138-按盘回退下界与推空的空间要求.md，原话「保留池与推空最坏残留）下，D3（空间分配」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/138-按盘回退下界与推空的空间要求.md:41 | 不相干 | 这一行来自 138-按盘回退下界与推空的空间要求.md，原话「布开销。推空最坏残留：A = 5 + (」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/138-按盘回退下界与推空的空间要求.md:42 | 不相干 | 这一行来自 138-按盘回退下界与推空的空间要求.md，原话「池 − 推空最坏残留；`df_raw`」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/138-按盘回退下界与推空的空间要求.md:83 | 不相干 | 这一行来自 138-按盘回退下界与推空的空间要求.md，原话「；准入改成按最坏残留收费（可分配取「可再」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/138-按盘回退下界与推空的空间要求.md:102 | 不相干 | 这一行来自 138-按盘回退下界与推空的空间要求.md，原话「池 + 推空最坏残留，是 `df` 比」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/138-按盘回退下界与推空的空间要求.md:129 | 不相干 | 这一行来自 138-按盘回退下界与推空的空间要求.md，原话「保留池与推空最坏残留的 `df` 口径逐」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:4 | 不相干 | 这一行来自 139-按盘回退下界的收严形态.md，原话「（准入按最坏残留收费、界与保留池给」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:36 | 不相干 | 这一行来自 139-按盘回退下界的收严形态.md，原话「根；准入不按最坏残留收费 &#124; 对照：第三」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:40 | 不相干 | 这一行来自 139-按盘回退下界的收严形态.md，原话「+ 准入按最坏残留收费 + B_R =」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:41 | 不相干 | 这一行来自 139-按盘回退下界的收严形态.md，原话「7 c_max、残留 5 + 6 c_m」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:96 | 不相干 | 这一行来自 139-按盘回退下界的收严形态.md，原话「池 + 推空最坏残留，是这条臂的 `df」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:98 | 不相干 | 这一行来自 139-按盘回退下界的收严形态.md，原话「= 保留池 + 残留 = 15 + 19」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/139-按盘回退下界的收严形态.md:99 | 不相干 | 这一行来自 139-按盘回退下界的收严形态.md，原话「少，是准入按最坏残留收费之后盘紧得更早、」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:1 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「上一条时间线的残留 —— 已跑（202」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:3 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「一条时间线**的残留记录接上去重放。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:23 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「「上一条时间线的残留记录」** &#124; 20」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:66 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「**最坏 = 残留全数** &#124; 0 &#124;」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:69 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「**物理作废**残留槽 &#124; **0**」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:71 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「fix` 重放的残留条数**：」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:73 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「&#124; 残留条数 \ 恢复后新写」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:80 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「max(0, 残留条数 − (新写条数」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:81 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「每写一条盖掉一条残留，**盖不完的那些照」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:92 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「（残留记录满足它）；在飞记」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:93 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「而残留记录的校验和**是好」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:98 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「（上一条时间线的残留）实测两条判据全过」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:99 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「恢复时物理作废残留槽 &#124; 不要 &#124; *」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:105 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「（上一条时间线的残留）的判决 &#124;」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:122 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「生在新时间线写满残留段之前」。**」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:125 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「（上一条时间线的残留）没有量这个窗口在真」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:137 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「者独立推进**，残留与「该位置期望的下一」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:138 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「（上一条时间线的残留）测到的「必然重放」」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:143 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「持久之后前移、而残留按定义**从未被应用」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:144 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「l ≤ min(残留 jsn)` 恒成立」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:145 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「被重放的残留条数与固定 tail」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:146 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「（上一条时间线的残留）只要「不同输入给不」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:149 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「（上一条时间线的残留）自己踩过两次「检查」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:164 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「：只靠断号即止，残留会被下一次恢复接上去」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/32-断号即止会不会重放上一条时间线.md:165 | 不相干 | 这一行来自 32-断号即止会不会重放上一条时间线.md，原话「缀判定单独挡不住残留；E32（上一条时间」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/33-两条钉块规则扣的不是同一批.md:6 | 不相干 | 这一行来自 33-两条钉块规则扣的不是同一批.md，原话「（上一条时间线的残留）**：2026-0」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/33-两条钉块规则扣的不是同一批.md:10 | 不相干 | 这一行来自 33-两条钉块规则扣的不是同一批.md，原话「（上一条时间线的残留） 相同：」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:3 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「「上一条时间线的残留被重放」。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:5 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「（上一条时间线的残留） 欠的那一维」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:7 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「（上一条时间线的残留） 的失败条款要求「」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:14 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「置**仍然**是残留序列的下一条。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:19 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「后续残留就整体错位。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:35 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「（上一条时间线的残留） 测出的那条式子」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:36 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「`max(0, 残留 − (新写 − 1」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:42 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「新记录长度 &#124; 残留 3 / 新写 1」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:50 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「（上一条时间线的残留） 的式子」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:55 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「占 1 位 ⇒ 残留整体错位 ⇒ 全部挡」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:57 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「（7 条残留全回来）。⇒ **「」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:60 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「**，于是它盖掉残留（盖掉 `新写 −」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:61 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「把新记录写到**残留之后**，一条也盖不」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:64 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「，方向相反**：残留不重放了，但新写的记」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/36-槽位映射那一维.md:72 | 不相干 | 这一行来自 36-槽位映射那一维.md，原话「、恢复时物理作废残留槽（要求原地覆写，z」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/37-日志实例代号.md:11 | 不相干 | 这一行来自 37-日志实例代号.md，原话「恢复时物理作废残留槽 &#124; 已测，全过，」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/37-日志实例代号.md:18 | 不相干 | 这一行来自 37-日志实例代号.md，原话「残留记录带的是旧代号 ⇒」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/37-日志实例代号.md:36 | 不相干 | 这一行来自 37-日志实例代号.md，原话「（上一条时间线的残留） 测出的式子 `m」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/37-日志实例代号.md:41 | 不相干 | 这一行来自 37-日志实例代号.md，原话「&#124; 臂 &#124; 重放残留（残留 7 / 新写」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/37-日志实例代号.md:70 | 不相干 | 这一行来自 37-日志实例代号.md，原话「（上一条时间线的残留） 已测过另一种）。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:1 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录 —— 已跑（2」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:4 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「把上一条时间线的残留记录挡在合法前缀之外」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:6 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「（上一条时间线的残留） 实测过那个漏洞成」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:7 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「轮字节一致）：残留记录校验和完好、`j」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:9 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）测的是补丁，不」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:13 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「（上一条时间线的残留） 那套模型上加一条」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:21 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「（上一条时间线的残留） 那个非零值——」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:22 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「是模型根本没造出残留。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:25 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「言**：造出来的残留记录条数要由构造直接」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:26 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「可能只是因为一条残留都没造出来。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:30 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「（上一条时间线的残留） 没覆盖的那两个组」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:31 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）要把四个都跑到」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:35 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「的 16 个格子残留全 0，且不误杀」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:43 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「= 6` —— 残留确实造出来了 &#124;」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:48 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「换一个种子，判「残留有没有骗过反向链」）」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:78 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）只取了点名 1」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:79 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）自己的常量（`」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:102 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）的模型独立重写」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:109 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「按「造了 8 条残留就该重放 8 条」写」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:110 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「格加其后 2 条残留」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:111 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「⇒ **活下来的残留 = `stale」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:112 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「回的话新记录写在残留后面，顺序扫描根本走」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:113 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「3`）而不是「残留被重放」。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:125 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）变异表里的 `」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:129 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「洞、其后 8 条残留、新时间线续写 3」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:137 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）里——」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:144 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）不回答它多久发」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:146 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「撞上第一条活着的残留记录里存的 prev」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:147 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「反向链挡不挡得住残留记录）没覆盖。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:153 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「（上一条时间线的残留） 的模型已在仓里，」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/39-反向链挡不挡得住残留记录.md:159 | 不相干 | 这一行来自 39-反向链挡不挡得住残留记录.md，原话「：开链的全部格子残留为 0 且不误杀，误」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/49-反向链宽度32还是64.md:44 | 不相干 | 这一行来自 49-反向链宽度32还是64.md，原话「反向链挡不挡得住残留记录） 已被复算坐实」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/49-反向链宽度32还是64.md:79 | 不相干 | 这一行来自 49-反向链宽度32还是64.md，原话「反向链挡不挡得住残留记录） 用的（84」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:17 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「故的接缝数 + 残留在被覆盖前被扫到的次」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:27 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「**残留能被扫到几次**也要」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:38 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「&#124; 4 &#124; 残留存活时间 = 一圈时」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:44 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「的不同源的记录（残留）⇒ 不同源计数必须」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:45 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「日志（无空洞、无残留）⇒ 不同源计数必须」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:74 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「100 条 + 残留 1 / 6 / 2」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:76 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「⇒ **残留接多长，跨时间线的比」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:79 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「一圈时长是秒级，残留活不到下一次核对」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:87 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「，最大的那个环上残留被扫到的期望次数是」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:88 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「⁻⁴**——**残留在被环绕覆盖之前，核」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/51-反向链的碰撞机会有多少次.md:97 | 不相干 | 这一行来自 51-反向链的碰撞机会有多少次.md，原话「：只算接缝 + 残留被扫到的次数 &#124; *」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/53-丢一整块盘之后根环还挂不挂得上.md:93 | 不相干 | 这一行来自 53-丢一整块盘之后根环还挂不挂得上.md，原话「次 mkfs 的残留）」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/56-消息缓冲的收益vsε.md:166 | 不相干 | 这一行来自 56-消息缓冲的收益vsε.md，原话「*不许用事后的「残留净增」当判据**：那」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/56-消息缓冲的收益vsε.md:168 | 不相干 | 这一行来自 56-消息缓冲的收益vsε.md，原话「、ε=0.9 上残留从 41,445 跳」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/56-消息缓冲的收益vsε.md:264 | 不相干 | 这一行来自 56-消息缓冲的收益vsε.md，原话「消息数 + 缓冲残留 == 操作数`，破」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/56-消息缓冲的收益vsε.md:265 | 不相干 | 这一行来自 56-消息缓冲的收益vsε.md，原话「ll` 边推边数残留 ⇒ 漏掉「上层排空」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/56-消息缓冲的收益vsε.md:340 | 不相干 | 这一行来自 56-消息缓冲的收益vsε.md，原话「计费的预热相与数残留那两段**算进了块层」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/76-载荷校验和的判别力.md:35 | 不相干 | 这一行来自 76-载荷校验和的判别力.md，原话「了一半」建成**残留字节**，不是短缓冲」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/76-载荷校验和的判别力.md:39 | 不相干 | 这一行来自 76-载荷校验和的判别力.md，原话「截是**上一圈的残留字节**，不是空洞。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/76-载荷校验和的判别力.md:78 | 不相干 | 这一行来自 76-载荷校验和的判别力.md，原话「2. **残留字节恰好等于真实载荷」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/76-载荷校验和的判别力.md:79 | 不相干 | 这一行来自 76-载荷校验和的判别力.md，原话「三个残留取值（0x00 /」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/76-载荷校验和的判别力.md:92 | 不相干 | 这一行来自 76-载荷校验和的判别力.md，原话「767 字节；残留字节 0xEE。」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/experiments/78-重放的起点.md:53 | 不相干 | 这一行来自 78-重放的起点.md，原话「残留记录冒充前缀那一类（」是该实验页里讨论的「残留」（journal 记录、块、消息缓冲或校验和层面的残留），与 test-environment-check.py 的 clean 汇总行新增点名太新项是两回事，不受这件事实影响。 |
| F4 | .claude/kb/invariants.md:14 | 不相干 | 这一行是 invariants.md 里的不变量 / checker 判定条目，原话「务（释放、重用、残留）、journal」说的是镜像侧「残留」（撕裂事务垃圾或被抛弃时间线的残留、或第二个事务才要补的释放重用残留检查），跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/invariants.md:25 | 不相干 | 这一行是 invariants.md 里的不变量 / checker 判定条目，原话「或被抛弃时间线的残留**——这是扫描方向」说的是镜像侧「残留」（撕裂事务垃圾或被抛弃时间线的残留、或第二个事务才要补的释放重用残留检查），跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/invariants.md:131 | 不相干 | 这一行是 invariants.md 里的不变量 / checker 判定条目，原话「6-09-18 残留记录那条层 0 流的」说的是镜像侧「残留」（撕裂事务垃圾或被抛弃时间线的残留、或第二个事务才要补的释放重用残留检查），跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/milestone/01-first-txn.md:231 | 不相干 | 这一行来自 01-first-txn.md，原话「/ 重用 / 残留三条要等发布 B 与」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:10 | 不相干 | 这一行来自 02-second-txn.md，原话「/ 重用 / 残留三条要等发布 B 与」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:11 | 不相干 | 这一行来自 02-second-txn.md，原话「务（释放、重用、残留）、journal」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:67 | 不相干 | 这一行来自 02-second-txn.md，原话「置（往环里种一条残留记录）；」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:71 | 不相干 | 这一行来自 02-second-txn.md，原话「建基线（C42（残留记录冒充合法前缀）」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:73 | 不相干 | 这一行来自 02-second-txn.md，原话「（上一条时间线的残留）、C42（残留记录」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:80 | 不相干 | 这一行来自 02-second-txn.md，原话「- 预置的残留记录：属于被抛弃实例」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:86 | 不相干 | 这一行来自 02-second-txn.md，原话「条款）；C42（残留记录冒充合法前缀）。」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:222 | 不相干 | 这一行来自 02-second-txn.md，原话「格的故障注入）；残留记录与陈旧 tail」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:228 | 不相干 | 这一行来自 02-second-txn.md，原话「坏单元（已有）；残留记录（步 0 预置」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:257 | 不相干 | 这一行来自 02-second-txn.md，原话「机器上不许留测试残留、宿主盘不许有异常，」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:353 | 不相干 | 这一行来自 02-second-txn.md，原话「il）；C42（残留记录冒充合法前缀）」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:354 | 不相干 | 这一行来自 02-second-txn.md，原话「数据）、C42（残留记录冒充合法前缀）」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/milestone/02-second-txn.md:355 | 不相干 | 这一行来自 02-second-txn.md，原话「任何根引用它们，残留记录那条流 12 个」说的是层 0 崩溃点重放里残留记录 / 残留测试这一类验证进度，跟 test-environment-check.py 的 clean 汇总行改动是两件事，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:147 | 不相干 | 这一行来自 verification-build.md，原话「录 &#124; C42（残留记录冒充合法前缀）；」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:160 | 不相干 | 这一行来自 verification-build.md，原话「账变更；C42（残留记录冒充合法前缀）还」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:161 | 不相干 | 这一行来自 verification-build.md，原话「写 → 再崩」与残留记录的反例还没有，现」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:162 | 不相干 | 这一行来自 verification-build.md，原话「「上一条时间线的残留」不会自己出现，ha」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:163 | 不相干 | 这一行来自 verification-build.md，原话「跑，释放、重用、残留三类要等到发布 B」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:179 | 不相干 | 这一行来自 verification-build.md，原话「反向链、时间线残留、槽位映射、实例代号」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:248 | 不相干 | 这一行来自 verification-build.md，原话「配）与 C42（残留记录冒充合法前缀）无」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/verification-build.md:253 | 不相干 | 这一行来自 verification-build.md，原话「复、基镜像可预置残留；checker 自」说的是崩溃点重放里残留记录那条模型或用例，跟 clean 汇总行怎么报数无关，不受这件事实影响。 |
| F4 | .claude/kb/vm-harness.md:58 | 不相干 | 这一行来自 vm-harness.md，原话「em'   # 残留虚机」是人工核对残留虚机 / 工作目录 / loop 设备的命令注释，跟 test-environment-check.py 的 clean 汇总行自动报数是两条不同的路，不受这件事实影响。 |
| F4 | .claude/kb/vm-harness.md:59 | 不相干 | 这一行来自 vm-harness.md，原话「# 残留工作目录」是人工核对残留虚机 / 工作目录 / loop 设备的命令注释，跟 test-environment-check.py 的 clean 汇总行自动报数是两条不同的路，不受这件事实影响。 |
| F4 | .claude/kb/vm-harness.md:60 | 不相干 | 这一行来自 vm-harness.md，原话「# 残留 loop」是人工核对残留虚机 / 工作目录 / loop 设备的命令注释，跟 test-environment-check.py 的 clean 汇总行自动报数是两条不同的路，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:44 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「上一条时间线的残留（D23（journ」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:47 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「n` 决定 ⇒ 残留记录不会消失、只会等」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:48 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「，**那正好接在残留序列前面**。」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:50 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「（上一条时间线的残留）`）：`重放的残留」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:57 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「md` C42（残留记录冒充合法前缀）。」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:89 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「&#124; 残留记录冒充合法前缀 &#124;」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:105 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「（上一条时间线的残留） 的第三条出路，不」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:282 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「（上一条时间线的残留）` 第一版把空洞建」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:289 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「（上一条时间线的残留）` 与 `E33（」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:300 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「（上一条时间线的残留）` 完全一致；」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:323 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「而把长度改回等长残留全数回来」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:326 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「（上一条时间线的残留） 的式子，两份独立」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-29-组合对攻轮.md:364 | 不相干 | 这一行来自 2026-08-29-组合对攻轮.md，原话「（上一条时间线的残留）` 的失败条款要求」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-08-30-实验回填与两轮攻击.md:22 | 不相干 | 这一行来自 2026-08-30-实验回填与两轮攻击.md，原话「反向链挡不挡得住残留记录） &#124; D23（」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-09-03-验证三件套落地调研.md:61 | 不相干 | 这一行来自 2026-09-03-验证三件套落地调研.md，原话「且基镜像要能预置残留记录。落地顺序照此改」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-09-06-树ID水位臂比较重做.md:27 | 不相干 | 这一行来自 2026-09-06-树ID水位臂比较重做.md，原话「或被抛弃时间线的残留**」。」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-09-10-D6项2与E130.md:74 | 不相干 | 这一行来自 2026-09-10-D6项2与E130.md，原话「残留落 C257。」说的是 2026-08、09 月那几轮实验对抗里 journal 残留记录的讨论，与 test-environment-check.py 的 clean 汇总行改动无关，不受这件事实影响。 |
| F4 | records/2026-09-19-里程碑二遗留收拢.md:150 | 事件句不改 | 这一行是 records/2026-09-19-里程碑二遗留收拢.md 里用户当天弹窗问答的记录，原话「机器上没有残留的 loop / device-mapper 设备、没有挂着的测试镜像」是 2026-09-19 16:4x UTC 那次现查的结果，后面「新写一个环境检查脚本…自检 113 条」说的是刚写好 test-environment-check.py 时的状态，都是那一次发生的事；这件事实改的是把既有的「太新」判断结果写进 clean 汇总行末尾，不改判断本身，这句历史记录不受影响。 |
| M1 | .claude/kb/checks-owed.md | 要补 | 这一阶段做成的事第 4 条（test-environment-check.py 的 run_clean 汇总行补报「不碰 N 项，其中 M 项只是太新」、自检新增两条断言）在 checks-owed.md 里没有任何一处登记：全文档 grep「太新」「kept_summary」「不碰.*项」「run_clean」均 0 命中，diff 里 checks-owed.md 这一次只新增了 C468（挪进已还清）与新立 C470，都不是这一条；C448、C396 两条与它同一个脚本相关的旧条目也都还停在各自原来的正文，没有一行被更新成指到这次的补报改动。建议：仿 C448/C396 的写法在「已还清」表里新增一条（或补进 C396 正文），写清「run_clean 汇总行现在会点名太新项，避免拿 tail -1 读输出的人漏看下一次门禁会红在哪」，并带判别力自证（脚本里已有的两条新增断言）。
