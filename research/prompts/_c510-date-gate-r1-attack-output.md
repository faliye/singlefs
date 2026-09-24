# C510 第一轮 · 云端攻方（Opus）：G3 与 G1「漏」那一半

攻击面：只攻 G3（判别力：红绿样本能不能证明它会红）与 G1 的「漏」那一半（射程漏了什么）。
不碰 G2（判据复用）、G4（落点）、G1 的「误判」那一半。

## 各格判定一览

| 编号 | 攻的是 | 判定 | 现跑 95 号的结果 |
|---|---|---|---|
| A1 | G3：同一行里的第二个日期 | **拦不住**（打中） | 绿，退出码 0 |
| A2 | G1 漏：fixtures 下非 `.md` 文件的正文 | **拦不住**（打中，且仓里现有 12 处活的） | 绿，退出码 0 |
| A3 | G1 漏：`YYYY-MM-DD` 以外的日期形态 | **拦不住**（打中，其中 `YYYY-MM` 月文件是本仓现行约定） | 绿，退出码 0 |
| A4 | G1 漏：落在 `[起点-7, 今天]` 区间内的编造日期 | **拦不住**（打中，但这是判据的固有射程，不是实现缺陷） | 绿，退出码 0 |
| A5 | G3：红绿样本罩不住的变异 | **拦不住**（打中：4 个变异体里 3 个存活） | 自检两格都「判得对」 |
| A6 | G3：子 shell 半路崩掉 | **拦得住**（没打中） | 红，退出码 1 |
| A7 | G1 漏：被 `.gitignore` 挡住的路径 | **漏，但漏得有理** | 见第六节 |
| A8 | G3：`gate.sh --staged` 的临时 worktree | **没打中**（推的，未跑） | 见第七节 |
| A9 | G1 漏：写明天的日期 | **半天拦得住、半天拦不住**（推的，未跑） | 见第七节 |

## 复跑

真仓这一侧（只读，不写仓）：

```
bash .claude/gate.d/95-fixture-claims.sh            # 现在是绿
```

造的实验都在仓的一份副本上跑，副本连 `.git` 一起拷，所以 `project_start_date` 与真仓同为 2026-08-26：

```
rsync -a --exclude target --exclude research/target \
  /home/fy5090/code/singlefs/ /tmp/claude-1000/c510-attack/repo/
cd /tmp/claude-1000/c510-attack/repo && bash .claude/gate.d/95-fixture-claims.sh
```

变异实验用的独立阶段目录（副本里现造，真仓没有）：

```
cd /tmp/claude-1000/c510-attack/repo
mkdir -p .claude/gd95/fixtures
cp .claude/gate.d/95-fixture-claims.sh .claude/gd95/
cp -a .claude/gate.d/fixtures/95-fixture-claims.sh .claude/gd95/fixtures/
bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh "$(pwd)/.claude/gd95"
```

副本上的数只用来演示机理，**不入库**；每一条打中都另给了真仓上的只读复核命令。
本机时钟 UTC，跑这一轮时是 2026-09-23 08:52–09:10 UTC（东京 17:52–18:10 JST）。

## 一、A1（G3，主打）：一行里只有第一个日期被判，第二个直接丢掉

### 绕法

在 `.claude/gate.d/fixtures/` 下任意一份 `.md` 里，**把编造日期写在同一行、排在一个合法日期后面**。

```
| 2 | 乙项 | **已定（2026-09-01）：取甲**，2026-01-01 立项 |
```

同样的字单独占一行就判红，跟在一个合法日期后面就判绿。副本上三次对照（每次只改那一份 `.md`，其余不动）：

| 写法 | 95 号退出码 | 成功句里的日期计数 |
|---|---|---|
| `已定（2026-01-01）：取甲` | 1（红，点名 `攻击样本.md:3`） | —— |
| `实测（2026-09-20）复核，已定（2026-01-01）：取甲` | **0（绿）** | 81 |
| `已定（2026-01-01）：取甲，2026-09-20 复核` | 1（红） | —— |

第二行与第三行的字完全一样，只调换了两个日期的先后。

### 机理：指到许可它的那一句

`.claude/gate.d/95-fixture-claims.sh:156`（`grep -nF` 现查）：

```
    done < <(grep -noE "$date_re" "$body" 2>/dev/null | sort -u -t: -k1,1n)
```

`grep -noE` 的每一行输出是 `行号:日期`，`sort -u -t: -k1,1n` 把**行号**当成唯一的排序键，
`-u` 于是按行号去重——**一行上有几个日期，只留一个**，留的是输入顺序里的第一个。直验：

```
$ printf 'a\n实测（2026-09-20）复核，已定（2026-01-01）：取甲\n' > line.md
$ grep -noE '[0-9]{4}-[0-9]{2}-[0-9]{2}' line.md
2:2026-09-20
2:2026-01-01
$ grep -noE '[0-9]{4}-[0-9]{2}-[0-9]{2}' line.md | sort -u -t: -k1,1n
2:2026-09-20
```

对照：文件名那一维用的是 `sort -u`（整行去重，`:143`），没有这个毛病——两维不对称。

### 这不是个刁钻形状，是 kb 行的主流形状

真仓只读现查（不改任何文件）：

```
$ git ls-files -z '.claude/gate.d/fixtures/*.md' | while IFS= read -r -d '' f; do \
    grep -nE '[0-9]{4}-[0-9]{2}-[0-9]{2}.*[0-9]{4}-[0-9]{2}-[0-9]{2}' "$f" | sed "s|^|$f:|"; done | wc -l
3
$ git grep -cE '[0-9]{4}-[0-9]{2}-[0-9]{2}.*[0-9]{4}-[0-9]{2}-[0-9]{2}' -- '*.md' \
    | awk -F: '{s+=$2} END{print s" 行，分布在 "NR" 份文件"}'
1083 行，分布在 115 份文件
```

即：全仓 `.md` 里有 1083 行一行带两个以上日期，`fixtures/` 下的 `.md` 里现在就有 3 行。
今天这一条**实际判的日期数比它声称的少 3 个**：

```
$ git ls-files -z --cached --others --exclude-standard '.claude/gate.d/fixtures/*.md' | sort -z -u > bodies.z
$ tot=0; chk=0; while IFS= read -r -d '' f; do \
    a=$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' "$f" | wc -l); \
    b=$(grep -noE '[0-9]{4}-[0-9]{2}-[0-9]{2}' "$f" | sort -u -t: -k1,1n | wc -l); \
    tot=$((tot+a)); chk=$((chk+b)); done < bodies.z; echo "$tot $chk"
83 80
```

成功句报的「80 个日期」不是 fixtures 正文里的日期个数（83 个），是**带日期的行数**。

⚠️ **这 3 个被丢掉的今天没有藏东西**：现查那 3 行（都在 `31-blocking-verdict.sh` 的两份样本里）
一行上的两个日期是**同一个** `2026-09-01`，合法且重复，丢掉不影响判定。
把 `sort -u -t: -k1,1n` 换成 `sort -u` 之后，本仓的成功句计数一个都不变（实测仍是「80 个日期」），
因为整行去重照样把这 3 个重复项合掉。⇒ **A1 是一个已经张着口、但今天还没被走过的口子**，
不是一处正在漏的数据；它的分量在「走一遍要花的力气」，不在「已经漏了多少」。

### 最难看的一格：拿仓里现有的那 3 行当载体

副本上把 `.claude/gate.d/fixtures/31-blocking-verdict.sh/green/.claude/kb/decisions/312-阻塞判定样本乙.md`
第 13 行**第二个** `2026-09-01` 改成 `2026-01-01`（第一个不动），95 号原样输出：

```
  ✓ 阶段头部声称的判别力样本都在（72 个阶段里 19 个声称有样本），.claude/gate.d/fixtures 下没有孤儿目录，也没有缺 expect 的空壳
  ✓ 日期都可能是真的（文件名：1668 个文件里 90 条带日期；样本正文：173 份 .md 里 80 个日期；下界 2026-08-26 往前宽 7 天）
RC=0
```

计数一个没变（还是 80），因为那一行本来就只贡献 1。

### 四句判

- **分不分辨臂**：分辨。同一份文件、同一句话，只换两个日期的先后，一个红一个绿。
- **被判的系统当时看不看得到**：看得到。`grep -noE` 已经把两个日期都吐出来了，是 `sort -u` 在下一跳丢的。
- **满足判据字面的哪一个分句**：第五节第一条「一条**具体可复现**的绕法（给得出文件名或正文写法，且现跑 95 号确实绿）⇒ G3 判「拦不住」，当场补」。
- **跑前条款给的每个改法在这一格上还中不中**：正文那一维「只扫 fixtures 不扫真 kb」这条限制与这一格无关——绕法发生在射程**之内**，把射程扩到全仓也照样绕得过去。

## 二、A2（G1 漏）：fixtures 下的非 `.md` 文件，正文一个字都不扫

### 绕法

把编造日期写进 `.claude/gate.d/fixtures/<阶段>/{red,green}/` 下任何一个**不叫 `.md`** 的文件。
副本上同一串字换六种载体，只有 `.md` 那一种红：

| 载体 | 内容 | 95 号退出码 |
|---|---|---|
| `攻击样本.md` | `已定（2026-01-01）：取甲` | 1（红） |
| `攻击样本.txt` | 同上 | **0（绿）** |
| `攻击样本-expect` | `want=已定（2026-01-01）` | **0（绿）** |
| `攻击样本.sh` | `printf "已定（2026-01-01）：取甲\n" > x.md` | **0（绿）** |
| `攻击样本.md.bak` | `已定（2026-01-01）` | **0（绿）** |
| `攻击样本.MD` | `已定（2026-01-01）` | **0（绿）** |

许可它的那一句是 `.claude/gate.d/95-fixture-claims.sh:157`（`grep -nF` 现查）：

```
  done < <(git ls-files -z --cached --others --exclude-standard '.claude/gate.d/fixtures/*.md' | sort -z -u)
```

以及头部 `:10` 自己写的射程：

```
#   ④ 日期不许是编的——全仓**文件名**里的日期，以及 `fixtures/` 下 `.md` **正文**里的日期，
```

`fixtures/` 下非 `.md` 的文件不是少数派——真仓现查：

```
$ git ls-files '.claude/gate.d/fixtures/*' | sed 's|.*/||' | grep -v '\.md$' | sort | uniq -c | sort -rn | head -5
    123 expect
     32 setup.sh
     17 lib.rs
     10 sample.rs
      8 Cargo.toml
```

### 不用我造：仓里现在就有 12 处活的，而 95 号是绿的

真仓只读现查（不改任何文件）：

```
$ source .claude/singlefs-ai-sop/scripts/lib.sh
$ sd=$(git log --reverse --format=%ad --date=short | head -1)   # 2026-08-26
$ git ls-files -z '.claude/gate.d/fixtures/*' | while IFS= read -r -d '' f; do
    case "$f" in *.md) continue;; esac
    grep -HnoE '[0-9]{4}-[0-9]{2}-[0-9]{2}' "$f"; done | sort -u |
  while IFS= read -r l; do d="${l##*:}"
    if why="$(date_out_of_range "$d" "$sd")"; then echo "$l  <= $why"; fi; done
.claude/gate.d/fixtures/60-stale-open-items.sh/green/setup.sh:10:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/60-stale-open-items.sh/green/setup.sh:7:2026-01-01  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/60-stale-open-items.sh/green/setup.sh:8:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/60-stale-open-items.sh/green/setup.sh:9:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/60-stale-open-items.sh/red/setup.sh:12:2026-01-03  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/60-stale-open-items.sh/red/setup.sh:7:2026-01-01  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/60-stale-open-items.sh/red/setup.sh:8:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/60-stale-open-items.sh/red/setup.sh:9:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/61-settled-same-file.sh/green/setup.sh:6:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/61-settled-same-file.sh/green/setup.sh:7:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/61-settled-same-file.sh/red/setup.sh:7:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/91-archive-past-rounds.sh/red/setup.sh:9:2026-01-01  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/95-fixture-claims.sh/red/expect:9:2026-01-01  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/95-fixture-claims.sh/red/setup.sh:10:2026-01-01  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/95-fixture-claims.sh/red/setup.sh:16:2026-01-01  <= 早于这个仓第一个提交（2026-08-26）7 天以上
.claude/gate.d/fixtures/95-fixture-claims.sh/red/setup.sh:18:2026-01-02  <= 早于这个仓第一个提交（2026-08-26）7 天以上
```

16 处里 4 处是 95 号自己的红样本（C441 不许摆进仓，这 4 处是设计如此），**剩下 12 处是真漏的**。
这 12 处全部已提交进 HEAD、工作区未改（`git status --porcelain -- <路径>` 一行不输出，
`git show HEAD:<路径> | grep -c '2026-01-0'` 分别得 4、2、1），而同一轮把 8 个文件名与 32 处正文都改了。

其中 `60-stale-open-items.sh/green/setup.sh:7` 的原文是：

```
GIT_COMMITTER_DATE="2026-01-01T00:00:00" GIT_AUTHOR_DATE="2026-01-01T00:00:00" sh -c 'git add -A && git commit -qm base'
```

`:8` 是 `已定（2026-01-02）` ——**正是 95 号头部第 20 行点名要管的那一种写法**：

```
# `已定（日期）`、`—— 已跑（日期）` 这类装成本仓事实的写法。
```

### 与 checks-owed.md C510 已登记那一笔的关系（要紧）

`.claude/kb/checks-owed.md:454`（`grep -nF '运行时才生成的编造日期 95 号扫不到'` 现查命中这一行）
已定项里写着「② 运行时才生成的编造日期 95 号扫不到（`fixtures/60-stale-open-items.sh/*/setup.sh` 的
`GIT_COMMITTER_DATE="2026-01-01"`、`fixtures/91-archive-past-rounds.sh/red/setup.sh` 造的
`e1-old-round-2026-01-01.out`、`research/scripts/stale-candidates.py` 自检里的 `records/2026-01-01-log.md`）」。

两点分歧：

1. **病因写错了。** 60 号那几处**不是运行时才生成的**：`2026-01-01` 是一串静止的字，躺在一个被 git 跟踪的文件里，
   95 号扫不到它的唯一原因是那个文件不叫 `.md`（A2 的六格对照表就是这一条的判别装置：`.txt` 绿、`.md` 红）。
   病因写成「运行时生成」会把修法引到「没法修」那一侧；按真病因修只要改 `:157` 的 pathspec。
   真正「运行时才生成」的只有 `91-archive-past-rounds.sh/red/setup.sh:9` 造的那个 `.out` 文件名。
2. **清单漏了一个对象。** `61-settled-same-file.sh` 的三处（green 2 处、red 1 处）不在 C510 ② 的清单里，
   也不在别处。它是被这一轮新装的检查放过、且没有任何地方登记在案的活对象。

### 四句判

- **分不分辨臂**：分辨。同一串字，`.md` 红、非 `.md` 绿。
- **被判的系统当时看不看得到**：看不到——`:157` 的 pathspec 先把文件挡在外面，判据函数根本没被调到。
- **满足判据字面的哪一个分句**：第五节第一条（具体绕法 ⇒ G3「拦不住」）与第二条（现存于仓里的对象 ⇒ 射程要改）都满足。
- **跑前条款给的每个改法在这一格上还中不中**：第六节「不许把射程放到真 kb 正文」这条禁令不挡这个改法——
  这里要扩的是 `fixtures/` 里的文件类型，不是 `fixtures/` 以外的目录，两条不冲突。

## 三、A3（G1 漏）：`YYYY-MM-DD` 之外的日期形态，一个都不认

许可它的那一句是 `.claude/gate.d/95-fixture-claims.sh:131`（`grep -nF` 现查）：

```
  date_re='[0-9]{4}-[0-9]{2}-[0-9]{2}'
```

副本上逐格试（文件名那一维放在 `records/`，正文那一维放在一份 fixtures `.md` 里）：

| 写法 | 维 | 95 号退出码 | 本仓有没有这种约定 |
|---|---|---|---|
| `records/2026-01-01-总审核.md` | 文件名 | 1（红，对照组） | 有，这是现行命名 |
| `records/20260101-总审核.md` | 文件名 | **0（绿）** | 无 |
| `records/2026_01_01-总审核.md` | 文件名 | **0（绿）** | 无 |
| `records/2026.01.01-总审核.md` | 文件名 | **0（绿）** | 无 |
| `records/2026-1-1-总审核.md` | 文件名 | **0（绿）** | 无（但少敲两个 0 就到） |
| `records/2026-01-总审核.md` | 文件名 | **0（绿）** | **有**，见下 |
| `已定（2026年1月1日）` | 正文 | **0（绿）** | 无 |
| `已定（2026/01/01）` | 正文 | **0（绿）** | 无 |
| `已定（Jan 1, 2026）` | 正文 | **0（绿）** | 无 |
| `已定（2026-01​-01）`（中间夹一个 U+200B） | 正文 | **0（绿）** | 无 |

**十个里九个是「本仓不会这么写」，不算打中**（按跑前判据「现存于仓里或将来必然出现」这条尺子量）。
真正打中的是 `YYYY-MM` 这一格：

```
$ git ls-files '.claude/kb/decisions-history/*'
.claude/kb/decisions-history/2026-08.md
.claude/kb/decisions-history/2026-09.md
```

`decisions-history/<年>-<月>.md` 是本仓现行的月度归档约定（门禁 48 号 `48-history-month-file.sh` 管它）。
一份 `decisions-history/2026-01.md`（本仓 2026-01 什么都没发生）落在 95 号第 ④ 条的射程里
（文件名那一维扫全仓），而 `date_re` 认不出它。别的九种形态本仓没有先例，
我把它们列在这里只为说清 `date_re` 的边界，不主张为它们收严——为不存在的形态加模式，
换来的是 A1 那种「规则越写越多、漏的那一跳还在原地」。

真仓只读复核（不写文件）：这三条 `git grep` 命中都是 0 行，所以「本仓没有先例」是数出来的，不是印象：

```
git grep -n -E '20[0-9]{2} ?年 ?[0-9]{1,2} ?月' -- .
git grep -n -E '(^|[^0-9])20260[0-9]{3}([^0-9]|$)' -- .
git grep -n -E '20[0-9]{2}[_.][0-9]{2}[_.][0-9]{2}' -- .
```

## 四、A4（G1 漏）：区间之内的编造日期，这条检查按定义看不见

副本实测：

| 写法 | 95 号退出码 |
|---|---|
| `records/2026-08-18-总审核.md` | 1（红，比下界早一天） |
| `records/2026-08-19-总审核.md` | **0（绿，正好落在下界上）** |
| `records/2026-08-20-总审核.md` + 正文 `已定（2026-08-20）` | **0（绿）** |
| `records/2026-09-10-总审核.md` + 正文 `已定（2026-09-10）` | **0（绿）** |
| `records/2026-09-23-总审核.md`（今天） | **0（绿）** |

许可它的两句在 `.claude/singlefs-ai-sop/scripts/lib.sh`（`grep -nF` 现查）：

```
113:DATE_GRACE_DAYS=7
129:latest_today() { TZ=UTC-14 date +%F; }
```

也就是说：**`[2026-08-19, 2026-09-23]` 这 36 天里的任何一天，写上去都是绿的**，窗口每天还长一天。
起因那个 `2026-01-01` 被抓住，只是因为造它的人挑了一个懒惰的占位值；
把占位值从 `2026-01-01` 换成 `2026-09-01`，这一条永远不会红。

这一格**算不算打中要主 agent 判**：它是 `date_out_of_range` 这个判据的固有射程
（它答的问题是「这一天可不可能」，不是「这一天有没有发生」），不是 95 号实现上的缺陷；
但它决定了第 ④ 条能兑现的承诺有多大——头部 `:10` 写的是「日期不许是编的」，
它实际能拦的是「日期不许是**不可能的**」。这两句差得很远，而读门禁输出的人读到的是前一句。
我的判：**按跑前判据这不算「拦不住」的打中**（我没给出一个「现存于仓里」的越界对象），
但 95 号头部与成功句的措辞与它实际做的事不符，算 G3 的一笔账。

## 五、A5（G3 正题）：红绿样本罩不住第 ④ 条的哪几处

装置：副本里另起一个只装 95 号一个阶段的 `gate.d`（`.claude/gd95/`），
拿 `stage-selftest.sh` 跑它的红绿两格；基线两格都「判得对」。然后逐个植入变异体，看样本抓不抓得到。

| 变异体 | 改在哪 | 红绿样本 | 结论 |
|---|---|---|---|
| M1 砍掉「晚于今天」那一支 | `lib.sh:134` 整行换成 `: #` | 两格都「判得对」 | **存活** |
| M9 `DATE_GRACE_DAYS=7` → `200` | `lib.sh:113` | 两格都「判得对」 | **存活** |
| M12 砍掉「没有 COUNT 就判红」那一支 | `95-fixture-claims.sh:183–187` 换成 `elif false; then :` | 两格都「判得对」 | **存活** |
| M13 砍掉 `require_date_arithmetic` | `95-fixture-claims.sh:128` 换成 `: #` | 两格都「判得对」 | **存活** |
| M7 `DATE_GRACE_DAYS=7` → `365` | `lib.sh:113` | red「1 个样本判错」 | 被抓 |

M1 的原样输出（其余三个同形）：

```
  ✓ 95-fixture-claims.sh         red   判得对
  ✓ 95-fixture-claims.sh         green 判得对
  ✓ 有样本的阶段判得都对（2 个样本），0 个阶段仍未自检
```

### 每个存活变异体说明了什么

- **M1：「晚于今天」这半条判据，判别力是零。** 红样本两维用的都是 `2026-01-01`，
  走的都是下界那一支；上界那一支从来没被样本走过。`fixtures/95-fixture-claims.sh/red/expect` 里
  那条 `want=早于这个仓第一个提交` 正是这半边覆盖的全部。
  ⇒ 有人把 `lib.sh` 的上界口径改坏（或上游改坏），本仓这一道不会红。
- **M9：宽限窗的宽度没有被任何样本钉住。** 红样本用的 `2026-01-01` 距沙箱起点 `2026-09-01` 有 243 天，
  所以 `DATE_GRACE_DAYS` 从 7 放到 242 都照样判红、样本照样「判得对」。
  ⇒ 这个数被改成 30、60、200 都没人拦；而它正是这条检查唯一的量纲。
- **M12：「没跑完」那一支没有样本。** 不过它在真仓上的实际后果比看起来轻：
  子 shell 崩掉时 `) > "$date_report" 2>&1 || date_rc=$?` 已经把非零退出码接走了，所以**退出码仍然是 1**。
  副本上把 `lib.sh` 挪走再跑带 M12 的阶段：

  ```
    ✓ 阶段头部声称的判别力样本都在（72 个阶段里 19 个声称有样本），.claude/gate.d/fixtures 下没有孤儿目录，也没有缺 expect 的空壳
    ✓ 日期都可能是真的（文件名： 个文件里  条带日期；样本正文： 份 .md 里  个日期；下界  往前宽 7 天）
  RC=1
  ```

  ⇒ 危害不是放绿，是**成功句在撒谎**：屏幕上写着「✓ 日期都可能是真的」，而四个计数全是空的。
  读汇总的人看到 ✓ 就过去了，退出码要跑 `echo $?` 才看得见。
- **M13：`require_date_arithmetic` 没有样本。** 它防的是 busybox/BSD `date`，
  沙箱里的 `date` 一直是 GNU 的，这一支永远走不到。这一条我不主张补样本
  （造一个假 `date` 放进 PATH 前面是可以做的，但代价大于收益，主 agent 判）。

## 六、A6（G3）：子 shell 半路崩掉——核过了，真的会红

派发提示要我核「没有 COUNT 那一支我写了判红，你去核它真的会红吗」。**会红。**
副本上把 `.claude/singlefs-ai-sop/scripts/lib.sh` 挪走再跑原版 95 号：

```
  ✓ 阶段头部声称的判别力样本都在（72 个阶段里 19 个声称有样本），.claude/gate.d/fixtures 下没有孤儿目录，也没有缺 expect 的空壳
  ✗ 日期这一条没跑完，报告里没有计数行：
      .claude/gate.d/95-fixture-claims.sh: line 127: /tmp/claude-1000/c510-attack/repo/.claude/singlefs-ai-sop/scripts/lib.sh: No such file or directory
      .claude/gate.d/95-fixture-claims.sh: line 128: require_date_arithmetic: command not found
      .claude/gate.d/95-fixture-claims.sh: line 130: project_start_date: command not found
      .claude/gate.d/95-fixture-claims.sh: line 130: SOP_START_DATE: unbound variable
     → 怎么办：上面是子 shell 的原样输出；多半是 lib.sh 取不到或 git 列不出文件，按它说的修。
RC=1
```

这一支有两道锁，任一道都够：`set -u` 让 `$SOP_START_DATE` 未绑定时子 shell 当场死掉（`date_rc` 收到非零），
以及 `:183` 的「没有 COUNT 就判红」。M12 证明第二道锁没有样本罩着，但第一道锁还在。

**我没有找到一条让子 shell 退出 0 且不打印 COUNT 的路。** 试过的形状写在第七节。

## 七、没打中的形状（试过、不成立或没跑）

| 形状 | 结果 | 取样范围 |
|---|---|---|
| 子 shell 退出 0 且不打印 COUNT | **没找到路** | 读完 `:126–159` 全部退出点：只有 `:129` 的 `SKIP` 一支会 `exit 0`，而它自己先打印 `SKIP`；其余非零退出都被 `\|\| date_rc=$?` 接走 |
| 往报告里注入一行 `SKIP`（靠带换行的路径名） | **没用** | `:161` 的 `BAD(NAME\|BODY)` 判在 `:181` 的 `SKIP` 判之前，注入 `SKIP` 挡不住已经打印出来的 BAD 行 |
| `scanned=0` 也报绿（这一条没有「扫到 0 项」的护栏，对比 python 段 `:57` 的 `if not stages`） | **构造不出可达的场景** | 试过把 `ROOT` 指到子目录：`git ls-files` 确实只列子目录，但前三条的 python 段会先因为 `.claude/gate.d` 不存在而判红，整阶段仍是红 |
| 非 git 仓 ⇒ `SKIP` ⇒ ⊘ 且退出 0 | **不算绕法** | `:129` 是显式写的「未跑」分支，屏幕上是 ⊘ 不是 ✓；真仓永远是 git 仓 |
| 被 `.gitignore` 挡住的路径（A7） | **漏，但漏得有理** | `git ls-files -z --others --ignored --exclude-standard --directory` 现查得 14 项，唯一带日期的是 `/.claude/singlefs-ai-sop/`（3 个 `warnings/2026-09-06.md`，合法）；它是另一个包的东西，本仓也改不了它。其余是 `target/`、`__pycache__/`、`.env`、`settings.local.json` |
| 索引里有、工作区没有的文件 | **没跑** | `:156` 的 `grep … 2>/dev/null` 会把「文件不存在」静默吞掉。要造这个现场得 `git rm --cached` 或手删已提交文件，**agent-common 不许我做 git 写操作**，所以这一格我只读到了代码、没有实测 |
| `gate.sh --staged` 的临时 worktree（A8） | **推的，没跑** | 读 `.claude/singlefs-ai-sop/scripts/gate.sh:54–80`：它用 `git worktree add --detach` 建树，所以 `.git` 是文件、`git rev-parse --show-toplevel` 有值、`git ls-files` 正常，`project_start_date` 拿到的是同一段历史。**推断是这一维在 --staged 下照常跑**；要实测得 `git worktree add`，同样是 git 写操作，没做 |
| 写明天的日期（A9） | **半天拦得住、半天拦不住，没实测到绿的那一半** | `latest_today()` 是 `TZ=UTC-14 date +%F`（UTC+14）。跑这一轮时是 UTC 08:52，UTC+14 还在同一天，`records/2026-09-24-总审核.md` 判红（实测）。**UTC 10:00 之后 UTC+14 就进了第二天**，那时写 UTC 的明天会判绿——每天有 14 小时是这样。本机没有 `faketime` / `datefudge`（`command -v` 都空），不许改上游 `lib.sh`，所以这一半我没有跑出来 |
| 把 `date_report` 换成别的文件 / 竞争 `mktemp` | **没试** | 要往 `/tmp` 里抢一个随机名，不是「可达的历史或输入」，是攻击装置本身 |

## 八、我自己提的改法（**只在副本上量过、被攻过零轮**）

| 改法 | 修哪一格 | 量过 / 推的 | 量到什么 |
|---|---|---|---|
| F1：`:156` 的 `sort -u -t: -k1,1n` → `sort -u` | A1 | **量过**（副本） | 注入「一行先真后假」的写法后判红并点名 `攻.md:3`；本仓成功句计数不变（仍是「173 份 .md 里 80 个日期」）；`stage-selftest` 红绿两格仍「判得对」 |
| F2a：`:157` 的 pathspec `'.claude/gate.d/fixtures/*.md'` → `'.claude/gate.d/fixtures/'` | A2 | **量过**（副本） | 判红，16 处，退出码 1——其中 4 处是 95 号自己的红样本，C441 不许把它们搬走 |
| F2b：F2a 再加 `':(exclude).claude/gate.d/fixtures/95-fixture-claims.sh/'` | A2 | **量过**（副本） | 判红，**正好 12 处**，与第二节数出来的 12 处活对象逐条对上 |
| F3：把 `date_re` 加一条 `YYYY-MM`（只用在文件名那一维） | A3 | **推的**，没实现没跑 | ⚠️ 直接把 `2026-01` 当 `2026-01-01` 比会**误判现存的 `decisions-history/2026-08.md`**（2026-08-01 早于下界 2026-08-19）。要判月粒度得问「这个月里有没有一天在区间内」，那是另一个函数，不是 `date_out_of_range` |
| F4：红样本加一个未来日期的文件名，`expect` 加 `want=晚于今天` | M1 | **量过**（副本） | 在 `red/setup.sh` 末尾加 `records/2099-01-01-样本.md`、`expect` 的 `want=1 个文件名…` 改成 `3 个`，基线两格仍「判得对」；再植入 M1（砍掉上界）时红样本报「输出里找不到『晚于今天』」，**变异被抓** |
| F5：红样本加「起点往前第 8 天」、绿样本加「起点往前第 7 天」 | M9 | **量过**（副本） | 红加 `records/2026-08-24-样本.md`（`want=2026-08-24`），绿加 `records/2026-08-25-样本.md`（`want=1 条带日期` 改成 `2 条`）。基线两格「判得对」；`DATE_GRACE_DAYS` 改成 8 → 红样本报「找不到『2026-08-24』」，改成 6 → 绿样本报「找不到『2 条带日期』」，**两个方向都被钉住** |
| F6：M13（`require_date_arithmetic` 没样本） | M13 | **不主张改** | 要造一个假 `date` 放进 PATH 前面，代价大于收益 |

⚠️ **F2b 自带一笔新账**：用 `:(exclude)` 把 95 号自己的样本目录整个排除，等于在这条检查的射程里
挖了一个洞——以后往 `fixtures/95-fixture-claims.sh/` 里写的任何编造日期都不会被看见。
这是 C441（全仓清扫工具会吃掉自己的判别力样本）在「排除」这一侧的对偶，本轮没有解。
另一条出路是把 95 号红样本那 4 处日期也改成由 `setup.sh` 用变量拼出来（`bad_date="2026-01-01"` 也会被扫到，
得拼成 `bad_date="2026-01-""01"` 这类，读起来更糟）。**两条路都没实测，交主 agent 判。**

## 九、这条腿自己的限度

- 副本上的数**不入库**：`/tmp/claude-1000/c510-attack/repo` 是 `rsync -a` 的一份拷贝（连 `.git`），
  所有变异、注入、改法都只在那里跑过；真仓这一侧我只跑了只读的 `bash .claude/gate.d/95-fixture-claims.sh`
  与若干 `git ls-files` / `git grep` / `git show`，一个文件都没往仓里写（除这份报告）。
- **没做 git 写操作**，所以两格没实测：`--staged` 的临时 worktree（A8）、索引里有工作区没有的文件。
- **F1–F5 全部被攻过零轮**，按 `.claude/rules/three-way-inference.md`「攻方腿自己提的收严，只在它自己的模型上量过，算「没被攻过」」办。
- 我没碰 G2、G4，也没碰 G1 的「误判」那一半——**F2a/F2b、F3 都会改变误判面，那一侧的判归本地攻方腿与主 agent**。
- A4（区间内的编造日期）我判「不算打中」，是按跑前判据的字面判的；如果主 agent 认为「头部写着『日期不许是编的』
  而它实际只能拦『不可能的日期』」本身就是 G3 的一个失败，那 A4 要从「没打中」挪到「打中」。
- 第二节对 `checks-owed.md:454` C510 的引述是**节选**，整行见附录。

## 附录：本报告引到的 kb 条目（整行，未摘句）

`.claude/kb/checks-owed.md:392`（C441）与 `:454`（C510），用 `awk NR==<行号>` 原样取出：

`````
| C441 | 全仓清扫工具会吃掉自己的判别力样本 | 术语清扫按登记表全仓替换时，把门禁 90 号自己的红绿两份样本也扫了：`fixtures/90-term-renames.sh/{red,green}/.claude/kb/term-renames.md` 的**旧名那一列**被换成新名，红绿两份内容变得一模一样，判别力当场归零——而 90 号本身照样报绿，只有阶段自检报「green 期望退出 0，实测 1」时才露出来（2026-09-21 实测）。同一形态对任何「按表替换全仓」的工具都成立：它的样本里必须留着被替换的那一侧 | 判据：一道检查的判别力样本被它自己扫过之后，红样本还红不红。做法是让清扫工具默认把 `fixtures/<自己的阶段名>/` 排除掉，排不掉就在自检里造一次「样本被自己扫过」的场景、要求红样本仍然红 | 无 | 2026-09-21 archive-rename 轮：`.claude/term-rename-exempt` 里那条 `fixtures/90-term-renames.sh/` 的登记 |

| C510 | 编造的日期没人拦 | **「日期不许是编的」这条禁令住在上游**（`.claude/singlefs-ai-sop/scripts/doc-lint.sh` 的 M 段，判据是 `lib.sh` 的 `date_out_of_range`：早于本仓第一个提交宽限 7 天、或晚于最晚时区的今天，都不可能是真发生过的事），**而它只认正文里的两种形态**：历史条目的 `### 日期`、正文里的「实测（日期）」。2026-09-23 实测：门禁样本里 8 个文件名与 32 处正文写着 2026-01-01 / 2026-01-02，比本仓第一个提交（2026-08-26）早大半年；带一个 `records/2026-01-01-临时实测.md` 跑 64 道阶段加 doc-lint，**0 道点名它**。一个编造日期读起来和真日期一模一样，而门禁样本正是后来人照抄形态的样板 | **已还大半（2026-09-23，门禁 95 号第 ④ 条）**：判据 source 上游 `lib.sh` 的 `date_out_of_range`，不另写一份；两维各判各的、各报各的数——文件名扫全仓路径（现查 1651 个文件、90 条带日期），正文只扫 `.claude/gate.d/fixtures/` 下的 `.md`（现查 173 份、80 个日期）。当天把那 8 个文件名与 32 处正文改成各自 fixture 的创建日（样本里要表先后两次事件的用创建日与次日）。判别力在 `fixtures/95-fixture-claims.sh/` 双向证过：red 的 `setup.sh` 在临时仓里现造一个坏文件名与一处坏正文（**不摆进仓里**，摆进来会被这一条自己扫到，C441（全仓清扫工具会吃掉自己的判别力样本）），并先做一个真实日期的提交让 `project_start_date` 有值、下界那一支真的走到；green 两维各放一个合法日期，成功句报「1 条带日期」「1 份 .md 里 1 个日期」，防「扫到 0 项也报绿」。**仍欠三样**：① `.claude/doc-lint-exclude` 里每行要写清它关掉了哪几类检查——只为绕开登记冲突而整份排除，等于把日期、指代、编号简称几条一起关掉，排除项该按检查分格、不按文件分格；② 运行时才生成的编造日期 95 号扫不到（`fixtures/60-stale-open-items.sh/*/setup.sh` 的 `GIT_COMMITTER_DATE="2026-01-01"`、`fixtures/91-archive-past-rounds.sh/red/setup.sh` 造的 `e1-old-round-2026-01-01.out`、`research/scripts/stale-candidates.py` 自检里的 `records/2026-01-01-log.md`）；③ 真 kb 正文里的 `已定（日期）`、`—— 已跑（日期）` 形态**明确不做**，依据见下一格 | **射程不许再放大，这条是实测定的**：2026-09-23 扫全仓正文得 52 处越界，其中 20 处全部合法——`prior-art.md` 9 处（别家项目的真实日期：bcachefs 手册 2026-04-16、`k1024.org/posts/2019/2019-02-08`、Greg KH 2026-07-15 表态）、`decisions-history/2026-09.md` 3 处（RFC 演进史 2015-08-07 → 2016-11-14 → 2017-01-30）、`checks-owed.md` 8 处（这一行自己引 2026-01-01 当例子）。`date_out_of_range` 的下界假设是「本仓的事不可能早于本仓第一个提交」，**这只对说本仓自己事的日期成立**；上游 M 段只认那两种形态不是射程不够，是有意收窄。⚠️ 本条 2026-09-23 早先的版本提议「上游把 M 段射程扩到表格单元格与标题括注」，**当天被这次实测推翻、已撤回** | 2026-09-23 用户看到 `fixtures/81-audit-contradictions.sh/red/records/2026-01-01-总审核.md` 问「我们禁止了这样的命名，为什么还出现」，逐处现查后立 |

`````

取法与逐字节复核：

```
awk 'NR==392' .claude/kb/checks-owed.md
awk 'NR==454' .claude/kb/checks-owed.md
```

## 十、没做什么

- 没往仓里写任何文件，只写了这一份报告。没有 git 写操作，没有提交。
- 没跑 `gate.sh` 全量，没编译 Rust。跑过的只有 `95-fixture-claims.sh`、`stage-selftest.sh`（限于副本里自造的单阶段目录 `.claude/gd95`）与只读的 git 命令，全部 `nice -n 19`。
- 没碰 `.claude/singlefs-ai-sop/` 的真副本（M1/M9 的变异只改副本仓里的那一份，每次跑完立刻从 `lib.orig` 还原并复跑基线确认还原成功）。
- 没攻 G2、G4、G1 的「误判」那一半。
- 草稿与产物都在 `/tmp/claude-1000/c510-attack/`（`repo/` 是仓副本，`lib.orig`、`lib.orig2`、`stage.orig`、`stage.orig2`、`312.orig` 是还原用的原件，`bodies.z`、`nonmd.txt`、`line.md` 是中间结果）。**这些都不入库**：它们是为了演示机理造的一次性装置，真仓这一侧的每条结论另给了只读复核命令。主 agent 要留的话现在还在。
