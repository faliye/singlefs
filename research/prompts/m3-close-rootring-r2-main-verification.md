# m3-close-rootring-r2 判决（第二轮，2026-10-20）

正文 `research/prompts/_m3-close-rootring-r2-body.md`，背景材料 `research/prompts/_m3-close-rootring-r2-background.md`（sha256 daa0caf397381dadcd682a4c67f6610d15aa6de53fc7894879d6fcb920ca7e39），开工快照 `research/prompts/m3-close-rootring-r2-snapshot/kb-sha256.txt`。第一轮判决 `research/prompts/m3-close-rootring-r1-main-verification.md`。

两条腿：
- 正推（Sonnet）：`research/prompts/m3-close-rootring-r2-sonnet-output.md`，sha256 53ff6bf39801fd2ae993854b634ebc350f7eeeab5095d429b2850037b12b0983。模型 `research/prompts/m3-close-rootring-r2-sonnet-model/`。
- 攻方（Opus）：`research/prompts/m3-close-rootring-r2-opus-output.md`，sha256 3c08afdcae7408d0875d4aa80ceeeaf360c078aa2685b8331a59ad5d0390de65。模型 `research/prompts/m3-close-rootring-r2-opus-model/`（攻补丁前的 `crates/`）。

## 一、核查

1. 引文：`cite-check.py <两份报告> --root . --background <背景材料>` 末行「✓ 核了 16 处引文、104 处裸行号，对上 16 处，没判 104 处（其中裸行号没有引文可比 104 处；逐处列在上面）」。
2. 快照：`sha256sum -c` 全部 OK。
3. 攻方复跑：它的开工拷贝拷进 `~/.local/share/singlefs/work/m3-close-rootring-r2-main-checks/frozen-opus`，经 4G 包装跑 `tools/rerun.sh`，退 0：

```
name=rerun_compare file=same.out same=yes lines=27
name=rerun_compare file=unify.out same=yes lines=183
```

4. 正推复跑：经 16G 包装排队中（共用内存额度被长跑占满），这一份判决写成时还没跑出；正推「量过」的数（Y1 122 张镜像 2094 个根、Y2、Y3 那几格）下面标「正推副本，主 agent 未复跑」，复跑出来之后补进这一节。
   补（2026-10-20，排队 14800 秒起跑）：开工拷贝拷进 `~/.local/share/singlefs/work/m3-close-rootring-r2-main-checks/frozen-sonnet`，跑它的 `tools/rerun.sh`：`y3.out same=yes lines=58`、`derive_y3.out same=yes lines=23`；`y1.out`、`y2.out` 报 `same=no`，逐行比只差各自第一行——那一行在原始日志里逐字在，跟在 libtest 的「test <名> ... 」后面同一行，复跑脚本按行首取没取到；去掉那段前缀补回之后两份与模型 `out/` 逐行相同。正推的数由此核上，「主 agent 未复跑」那几处标注作废。

## 二、逐格判定

| 格 | 正推 | 攻方 | 判定 |
|---|---|---|---|
| Y1 C743 补第 0 代例外 | 例外 = `checkpoint_txg == 0 且槽号 == 0`，合法历史带例外拒 0、裸公式拒 19（正推副本，主 agent 未复跑）；D22 实例代号行的平局括号与已定项 16 公式句冲突 1 处 | 没打中（推的）：根环洞、跨区域回绕、同 txg 两根都住在自己的公式槽；第一轮说「放行槽 0 读回别区域第 0 代根无害」的理由写错，被错位读回的那份本身在环里、门槛不动；补了取号那一处入口，结局不变 | **站住（被攻过一轮没打中）**。第三轮只复核 D22 那一处冲突与第一轮理由的更正 |
| Y2 C751 同实例的根当作不被抛弃 | 量过（正推副本）：登记那一形只读从 Pm 到 P，`:656` 不变 | 打中（量过，主 agent 复跑对上）：`out/same.out` 原行「`R2SAME shape=cross kind=ow k=5 crash_after_mount_root#2 fault=table+cfg chosen=(2,11) table_readable=false today rec=Ok(1,9) ro=Ok(2,11) rw=Fail \| f2_core=(1,9) f2_replica_off=(1,9) f2_same_instance=(2,10)`…」——见证到的那一版属更旧实例时，新形态退到同实例的 (2,10)，它与所选根共用读不出的实例表，恢复从 `Ok(1,9)` 变成失败；零轮改法「只在见证到的那一版与所选根同实例时才收同实例的根」（按输出推的，没跑） | **打中成立**。交用户在零轮改法与退回「承诺外」之间定 |
| Y3 C755 只统一落到哪一版 | 并集只判退、不判成败，第一轮 100 格挂不上 0、三处同版 16 → 46（正推副本） | 取「打开」或「建上一版」会让恢复或可写由成变败（量过）；取「走读」或「并集」不少挂上，代价只读 34 格、可写 8 格落回上一版（`out/unify.out` 原行「`R2UNIFY_TALLY img=witness_lost trigger=walk rule=same place=ro change=older_ok cells=34`」「`… place=rw change=older_ok cells=8`」，主 agent 复跑对上）；三处交 F2 的记录不是同一份（读码） | **站住，取并集（或走读）**：两腿一致不少挂上。条文要写死「并集只用来判退」，并补「三处交 F2 的记录同一份（触发之后各重扫）」，与 C754 一致。第三轮攻补好的条文 |
| Y4 C773 另记钉槽、checker 现算求等 | 只加字段与求等修不了 C773 那一形（那 44 槽无根引用，推的）；I-3.1 要多并「环里没有任何根引用的已释放槽」，折中写法（被抛弃根引用着的不并）保 C684 红；拟 I-7.18、I-7.19 | 条文层打中四处（推的）：checker 取 F_生效 在合法态误红、要取根自己带的 F；少了「钉着的槽不住本实例写的根」；两类坏实现镜像上看不出；没写 I-3.1 怎么用盘上门槛，照今天按引用并 C773 登记形仍红；零轮补法「I-3.1 只把钉带出来的那一截不看引用并进」 | **打中成立**：两腿一致这一轮的形态不够。合成的零轮形态：根记录加门槛 t 与钉槽 p；checker 取根自己带的 F 现算、要求等于 t（I-7.18）；钉着的槽不得住着本实例写的根（I-7.19，正推与攻方同提）；I-3.1 只把「钉带出来的那一截」并进、被抛弃根引用着的不并（C684 保持「不并」）。交用户认，第三轮攻它 |
| Y5 交互 | Y2 与 Y3 改 F2 两个不同位置，100 格与记录 4 格同实例开关开关结局逐格相同（正推副本） | 「同实例 × 统一」下 `:656` 与 50 格不变（量过）；Y2 那一形统一后三处都落 (2,10)、恢复失败（量过）；Y1 × Y4 求等自洽（推的） | 照收；Y2 改法定了之后第三轮一并攻 |

方法纠错两处照收：第一轮攻方 X4.2 第 1 条（分配器在装记账行之前记根，门槛不会差一截）；第一轮「三处都能落 P」只是 F2 交回的落点，落点上恢复与可写仍失败。

## 三、交用户

- **C751**：同实例那一支被打中。候选：攻方零轮改法「只在见证到的那一版与所选根同实例时才收同实例的根」；或退回「承诺外，并进 F2 的代价」（不改代码）。
- **C773**：合成的零轮形态（第二节 Y4 判定那一格）认不认。

C743、C755 不交用户，第三轮攻补好的条文。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D17（实现分层与第三方管道） 已定项 1 | 备料 | 2026-10-20 不受影响：错位读条文第三轮之后再写 |
| D16（发布语义） 已定项 1 | 备料 | 2026-10-20 不受影响：回收门槛落盘形态待用户认、第三轮攻 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-10-20 不受影响：F2 段条文第三轮之后再写 |
| D22（单元原子性怎么合成） 已定项 7 | 备料 | 2026-10-20 不受影响：实例代号行平局括号与已定项 16 公式句的冲突第三轮复核 |
