# research/prompts/ 私有主机与服务信息改写登记（2026-09-28）

**依据**：用户 2026-09-28 定「这些本机的私有服务的信息 不要写入kb等公开的日志中了」；随后对「已经在仓里的冻结证据」选了「当前文件也改，历史不动」。所以这一次改动冻结证据是用户定的：只改工作区里的当前文件，不改写 git 历史，不动 `.git`。

**射程**：`research/prompts/` 下（含子目录）。`research/results/`、`multi-host.env`、kb、`records/`、`.claude/`、`research/scripts/` 不在这一次里；本地模型网关的名字与端口不在射程里。

## 怎么改写

| 类别 | 正文里写成 | 命令、路径、代码片段里写成 |
|---|---|---|
| 本机主机名（内核日志行首的主机名字段） | 本机 | 本机（行的其余字节不动） |
| 第二台的 ssh 别名 | 第二台 | `<第二台>` |
| 两台主机名的共同片段（记下的 grep 模式里） | — | `<主机名片段>` |
| 第二台上的路径 | 第二台上的仓副本目录 | — |
| 本机本地模型服务的 systemd 单元名（带不带 `.service`） | 本机本地模型服务 | `<本机本地模型服务>` |

尖括号只用在命令、路径与代码片段里，标明那里原来是一个具体值、现在是占位；跨机格要用的真实别名在本机仓根 `multi-host.env` 的 `PEER_SSH_HOST`，这份文件不抄它的值。

## 改了哪些

16 份文件，94 处。改写用 `research/scripts/replace-batch.py`：每处先在内存里核「恰好命中登记的次数」，全过了才写盘、回读。

| 文件 | 类别 | 处数 | 改前 sha256 | 改后 sha256 |
|---|---|---|---|---|
| `research/prompts/_defs-m2-closeout-r1-appendix.md` | 本机服务单元名（本地模型服务） | 1 | `c4428acb4c62521b62fe9a617fb8262a933687905ce9865f69f3b34fde5f2d6a` | `29bac9de2468d909b5cdd979a445c7c1647ec5928c7252a064dbff1821461a1f` |
| `research/prompts/_defs-m2-closeout-r1-background.md` | 本机服务单元名（本地模型服务） | 1 | `eb87094840112c10e30d9738010bed9a76ccf123cfbce86f5da28a2645bd27e9` | `23fdb3da138a32eedd394c393d6e2b48cd8997f946d1384879e3e685a6ca9609` |
| `research/prompts/_defs-m2-closeout-r2-appendix.md` | 本机服务单元名（本地模型服务） | 1 | `33a4a1f493cd346a5a0e742e53ffdbd3ced135adb119d30858127b3d42bf3b72` | `f8c9161756d772d639193b24f71185441a7a9e76594364410c0f378a1d4d3a13` |
| `research/prompts/_defs-m2-closeout-r2-background.md` | 本机服务单元名（本地模型服务） | 1 | `b008fc7c2fee248629820af42c69c69de60255a97327dc291462ec565e9da56e` | `e4e110700a55280259fbff044bf98dad09c5b0e0314524e10e0312e364f9c02f` |
| `research/prompts/verification-split-tmp-evidence/old-wording-search.txt` | 本机服务单元名（本地模型服务） | 1 | `69d7c84197a977b19732845ed46d6c10a9d68c417a795d31c8474e34411336d8` | `a9d85a0a48d13ca0dd13590974e0982d14f7e3e1a31146d43fbc40113eadcd22` |
| `research/prompts/e162-preregistration.md` | 第二台 ssh 别名（正文 1、代码片段 7） | 8 | `4bbf0ebcce89a82ecac2fe841b4443d14fb657712e5fba185be8f9e5ae2e9e0f` | `906809a9734cd04bc28f0862a59e8e08c6698ab6d65d2936adaa127697041b56` |
| `research/prompts/e162-s3-s4-designer-report.md` | 第二台 ssh 别名（代码片段） | 2 | `0f4375629fa0b4e9cf61731ca4708c32e091c413f7d713673f484044bd81d792` | `8bff742d16d4c9bf18b080ba6fe20870faa90b01256e8488ab6a58cf86eab015` |
| `research/prompts/_governance-defs-r3-appendix.md` | 本机服务单元名（本地模型服务；抄进来的脚本源码里：注释与打印文字 4、命令与路径 2） | 6 | `b90e54ae137972153b4375fd00f60a92222bba2e03f9e1a349a56b6f0142e06c` | `db227b261ddef48d785ceabf538741745211920735ad199e28ce0222bd94e1f8` |
| `research/prompts/_governance-defs-r3-background.md` | 本机服务单元名（本地模型服务；抄进来的脚本源码里：注释与打印文字 4、命令与路径 2） | 6 | `e258c4966abe2d74886687419f8a901b949e0dfb2a3f581ec4b2d9d38968b3b9` | `d77cc9c55432d862713836afb3c7095211ab2cc66bbf7968033e349036a0a92e` |
| `research/prompts/m2-crash-store-r1-forks.md` | 第二台 ssh 别名（正文） | 1 | `8e5f55b354ee79c6399eee66a8895eb299124b4a0d03dcaec99dfad22ae55c2c` | `496bc97f1b378f480a609e51f1f15167ad6f5cc180fe52f6469ef3c2ac75bfb5` |
| `research/prompts/m2-gpu-multicard-r1-forks.md` | 第二台 ssh 别名（正文） | 1 | `7e80a20cfa300afb88e5173b96b85159f9aae4102f5cf9e1c1a5ff98e0522dc9` | `c1157fad32aa78bd882252edf7d8eccb6c36d276e6cdceecb4245c150e4e2e90` |
| `research/prompts/m2-impl-shard1-implementer-report.md` | 第二台上的路径（仓副本目录） | 1 | `468fc1d0ab4ce9a6c4f471998be497fe4c7867fa56a975c5b02eebcf86c028a5` | `c3bfed00e66cce7a88adfce3f2df35f2f85034322b173b20b0f3cdee74161d4a` |
| `research/prompts/m2-readme-status-report.md` | 两台主机名的共同片段（记下的 grep 模式里） | 1 | `fd6801d7f8b89514992de1e9cc16344fb955a28f12416294d877d3a7066bf5bf` | `7a4913feb08dd35b167227903ee0aac53f58cf79ba3caa8f7a6978c0f3dfdce1` |
| `research/prompts/oom-2026-09-25/kernel-oom.log` | 本机主机名（内核日志行首字段） | 60 | `96affcad0f5eb09c0245a0209159f37325e255998fb9ae85e684b31706ac8e32` | `8d13d2ee7f2b7e11d4517bca69d070379e071823e21d60a2044724a0b93f619c` |
| `research/prompts/oom-2026-09-25/report.md` | 本机服务单元名（本地模型服务；正文 1、代码片段 1） | 2 | `9d5d51b12d20d4021eab2cbd1f33574a37e0d450219c1ea442cfac78fd4f8d4a` | `e872cb3e28fc02b5a8516265b069a34736472ab484ffec942555780ab6a0115e` |
| `research/prompts/oom-2026-09-25/repro-cgroup-kill.log` | 本机主机名（内核日志行首字段） | 1 | `cce60d593a0231c38cf17bf68d2d08de56c3502cc83dc6362416eadb12979b4e` | `d36bdcc664344079997b8342a16f02d5d1fc651f37e316d481226a13ea9a6a30` |

`research/prompts/e162-preregistration.md` 改前的工作区里已有别的会话没提交的改动，改前 sha256 是那一刻工作区里的内容，不是 HEAD 里的。

## 留着没改的

| 文件 | 类别 | 处数 | 为什么留 |
|---|---|---|---|
| `research/prompts/_defs-m2-closeout-r1-diff.md` | git 作者名与作者邮箱（`git log` 输出的 Author 行） | 1 | git 作者名按这一次的射程不改 |
| `research/prompts/_defs-gate54-tiering-r2-appendix.md`、`research/prompts/_defs-gate54-tiering-r2-background.md`、`research/prompts/m2-readme-status-report.md`、`research/prompts/verification-split-r1-local-attack.md`、`research/prompts/verification-split-r1-local-attack-output-s2.md`、`research/prompts/verification-split-r1-local-attack-translation-audit.md` | 公开仓库地址里的 GitHub 账号名 | 6 | 公开的托管账号，与 git 作者名是同一个名字，不是哪台机器上的用户名或路径 |
| `research/prompts/m2-readme-status-report.md` | 记下的 grep 模式里的用户名 | 1 | 那一句是查模板里有没有真实值时用的模式，用户名在那里分不清是 git 作者名还是第二台的用户名，交主 agent 定 |

## 这一次之后对不上的 sha256

别处（`records/`、三方判决、核查员的核对表、开工快照）记过这 16 份文件 sha256 的，从此对不上，原因就是这一次改写：改的只是「改了哪些」那张表里列的那几类值，文件的其余字节没动。

## 第二批（2026-09-28）

**依据**：用户对第一批交回里要看的几件事答「3 改了 4 改了」：射程外的命中，以及第一批没改的「光写软件名」「本机用户名路径」两类，都改。只改工作区里的当前文件，不改写 git 历史。

**射程**：`research/prompts/`（含子目录）与 `records/` 下的全部文件。`research/results/`、`.claude/`（含 kb）、`research/scripts/`、`crates/`、`multi-host.env` 不在这一批；本地模型网关的名字与端口、git 作者名与 GitHub 账号不在射程里。

### 第二批怎么改写

| 类别 | 正文里写成 | 命令、路径、代码片段里写成 |
|---|---|---|
| 本机仓根的绝对路径作前缀、后面跟着仓内路径 | 去掉前缀，只留仓内相对路径 | 同左 |
| 本机仓根的绝对路径本身（后面不跟仓内路径） | 仓根 | `<仓根>`；原来带结尾斜杠的写 `<仓根>/` |
| 本机用户目录下、仓根之外的绝对路径 | 用户目录那一段换成 `~` | 同左 |
| 本地推理软件名（不分大小写） | 本地模型服务 | `<本地模型服务>` |
| 本机本地模型服务的 systemd 单元名（软件名带后缀的那个） | 本机本地模型服务（与第一批同） | `<本机本地模型服务>`（与第一批同） |

- 仓根本身那 136 处都在命令、日志、JSON 与反引号里的路径上，所以都写成了 `<仓根>`，正文写法「仓根」这一批一处都没用上。
- 两处为了不写成同义反复，改了软件名旁边的字：`_governance-defs-r3-appendix.md`、`_governance-defs-r3-background.md` 各一处「本地模型服务（<软件名> 与 ray）」写成「本地模型服务（服务本体与 ray）」；`records/2026-09-16-subagent拆分提案.md` 一处「本地模型服务 <单元名> 的 ray 进程」写成「本机本地模型服务的 ray 进程」。
- 抄进来的 python 源码里以软件名为名的变量（两份各 2 行、4 处）按代码片段写成 `<本地模型服务>`，这几行从此不是合法的 python。

### 第二批改了哪些

160 份文件，450 处（本机仓根前缀 254、仓根本身 136、用户目录 39、软件名与服务单元名 21）。改写用 `research/scripts/replace-batch.py`，分两批跑：`records/2026-09-16-subagent拆分提案.md` 与 `records/2026-09-24-里程碑二收尾调度.md` 两份（主 agent 会往里插行）单独一批 8 条，其余 158 份一批 186 条；每批都在同一条命令里先拿改前 sha256 做 `sha256sum -c`、对得上才跑，跑完回读一致。改后 160 份的 sha256 与事先按位置算出的目标逐份相同。

类别列：「仓内路径」是去掉本机仓根前缀，「仓根」是换成 `<仓根>`，「用户目录」是换成 `~` 起头，「软件名」「服务单元名」括号里分正文与代码。

改前 sha256 是那一刻工作区里的内容：128 份与 HEAD 相同；23 份没进 git（别的会话写的，还没提交）；`_defs-m2-closeout-r2-appendix.md`、`_defs-m2-closeout-r2-background.md`、`_governance-defs-r3-appendix.md`、`_governance-defs-r3-background.md`、`e162-preregistration.md`、`e162-s3-s4-designer-report.md`、`m2-readme-status-report.md` 7 份带着第一批的改写（它们的改前 sha256 就是第一批表里的改后 sha256）；`records/2026-09-16-subagent拆分提案.md` 与 `records/2026-09-24-里程碑二收尾调度.md` 带着别的会话没提交的改动。

`research/prompts/` 顶层（87 份）：

| 文件 | 类别 | 处数 | 改前 sha256 | 改后 sha256 |
|---|---|---|---|---|
| `research/prompts/_c363b-r1-appendix.md` | 用户目录 1 | 1 | `527fd3648ab5caee84df1d896c331ccc40ae120a0039a7c728349236e01f8354` | `e84b40c2ce5a3fe42b731f9d98fc8e3e0703b2d74883731e34082362b0dd9cc3` |
| `research/prompts/_c363b-r1-background.md` | 用户目录 1 | 1 | `b32ba784cc237c6a6469f380cb511415f9ea1bd1b9727cd632c3e27f37357e15` | `b3711dff029f84090433fd1d84ece534efc6c0e1f8ceb114d949b5deaad522ff` |
| `research/prompts/_c510-date-gate-r1-attack-output.md` | 仓根 1 | 1 | `63b62eab949830cd4144c123d2a6a8f92ae4802e7c5725e3e7f15bde9dff52ec` | `63366f20f46995488206a4c8541dd19c19233e7e64fe466e037878dc4b1ad60b` |
| `research/prompts/_defs-gate54-tiering-r1-appendix.md` | 仓内路径 4 | 4 | `fb0bbc1e2f5381171f06c0ecded49d80a2bcb83b35e71ee85c2105f6844ba119` | `a0cdca26c8155521f1fad7d0a8a7e554645281dc33951988a5959df6b387c0e7` |
| `research/prompts/_defs-gate54-tiering-r1-background.md` | 仓内路径 4 | 4 | `74235dfcfb57e3c2e1b1983a38820b99bdef133ae6892b1790827bd959c9d986` | `cbb022590f71d3d5b48c8a27df0a62c4c4a83d8b76e4cce0ae9039fa49fc8db3` |
| `research/prompts/_defs-gate54-tiering-r2-appendix.md` | 仓内路径 7 | 7 | `74a7fb38a1891bf21446049d6caced388c6856b22c25eaad58a34d18e586546e` | `79638c56b827979652efa57f80f216f8352270c92e0d97fe5717b35169d616eb` |
| `research/prompts/_defs-gate54-tiering-r2-background.md` | 仓内路径 7 | 7 | `dbd697547ba2c14cfe4c110718a78094a42daa073771b93e0343989f5003953b` | `04a56432239b0e594a752b02c7d63b5f678fa21f870c972c64c3d18473d5fcfa` |
| `research/prompts/_defs-gatebatch-m2-r1-diff.md` | 仓内路径 1 | 1 | `a84dde5263cda7ff4d261acf19d5cedb8f7c9390b3758b6062b3c61ee25b249b` | `3cf5f9324f8336a3d2c015b4686163a431a42232995ab98cced63c3b183a5cd4` |
| `research/prompts/_defs-m2-closeout-r1-diff.md` | 仓内路径 6 | 6 | `3db9f19ea6fac39431a9f194ea5c287b51b8bdbc062ecd28f7a3912e4744107f` | `cc9e79d0983fe9bbcf1eee0733c07b1244f7aec76fd1236343190e0294ffe637` |
| `research/prompts/_defs-m2-closeout-r2-appendix.md` | 仓内路径 10、仓根 27 | 37 | `f8c9161756d772d639193b24f71185441a7a9e76594364410c0f378a1d4d3a13` | `2a30478213ca6d4fcf05830e2d3283fd5f8082f6496437e8f0c8060e6b1f7ff0` |
| `research/prompts/_defs-m2-closeout-r2-background.md` | 仓内路径 10、仓根 27 | 37 | `e4e110700a55280259fbff044bf98dad09c5b0e0314524e10e0312e364f9c02f` | `3f8a03bb9852822747c7b3ee9c3ff80d238f1e35df2cce1723e581328dabdcd1` |
| `research/prompts/_governance-defs-r3-appendix.md` | 软件名（正文 3、代码 3） | 6 | `db227b261ddef48d785ceabf538741745211920735ad199e28ce0222bd94e1f8` | `08480472fbbe8fcc4facf34cca93790d2002e3ba5bd88018787171b8c0744282` |
| `research/prompts/_governance-defs-r3-background.md` | 软件名（正文 3、代码 3） | 6 | `d77cc9c55432d862713836afb3c7095211ab2cc66bbf7968033e349036a0a92e` | `bf106d7f2d11b1be969c0e261df2d505b184b4d09a57ade2c57d3d1e464a9836` |
| `research/prompts/_m2-final-code-r3-appendix.md` | 用户目录 1 | 1 | `7fe2390ff9195744c2b9b3824c5ba0677501516d886ab5614e564c971b810fe3` | `7917a06ca1654f48c73507ecff7ebdb713ce9726280a52f4d544db1b1e81c7f4` |
| `research/prompts/_m2-final-code-r3-background.md` | 用户目录 1 | 1 | `6aa656526da8bbc25bc3d5900295b483c81cb2c401d53766d018a4b2841b0982` | `42cd23eadf3fab4b6b42239156db66a834a0c6b4c06529f347ccbccd6cea3acd` |
| `research/prompts/_m2-keyspace-r1-appendix.md` | 用户目录 1 | 1 | `801b235ab8171e2486d0a96e6ec45baae71e55faed41dac2fd0d39b396626953` | `8ee3c9d6d83fd20dfa7fdfb80a18178022c1cd8d13a82fe61cfb1e894a358562` |
| `research/prompts/_m2-keyspace-r1-background.md` | 用户目录 1 | 1 | `0403c259f2b9e45c9d773f75d3d51da5c5d2f9fb1fcd719db6bcc3a9ef61cafd` | `7cb9d888c4205d73c7d965a52b3b09b3d9b415939831e3b91c7467c6113f7bd1` |
| `research/prompts/_m2-rollback-forward-r1-appendix.md` | 用户目录 1 | 1 | `d7344f60b3eea2f2692dfd3163f2b365e75562eb5d40af7d4f37323092a7f860` | `ff348646083e099624542f24514fa50a289060e0531aa220776e195a43cc5d38` |
| `research/prompts/_m2-rollback-forward-r1-background.md` | 用户目录 1 | 1 | `c514eb2e1dedb5c7cb0760bcfe934c05f197806e3080a75719ecb130a709b424` | `72c96e355330bb9677e5f87271b0f5d132d0001962c2b47c3a29f8deddb9ad29` |
| `research/prompts/_m2-treesplit-r1-appendix.md` | 用户目录 1 | 1 | `0daceb10d981213e5eff2880cd81a3f8e6d7e9b285dd7d06eb335d5ef4606fec` | `efb863822cd97ef46b19a3c589974af498a6a85ce72d4855f52b94de3d08aa50` |
| `research/prompts/_m2-treesplit-r1-background.md` | 用户目录 1 | 1 | `742a39831a7f68bd511ba143782557292792f9f82695dfa9bca5304f85898022` | `d3d534e11836f145a585972f606b5f4312bf2802bcc59ad3412ddf956097ecc4` |
| `research/prompts/c355-c363-r3-verifier-output.md` | 仓根 1 | 1 | `acba35345c252c706b43ae97db9c9bb8cdc8f87cbb2a1fa035afc567a3ab0169` | `d4adce98d89b1dd06f0119aee8741b2201c0d631a60feab22a6bfb8ef715a226` |
| `research/prompts/checkpoint-trigger-r1-opus-output.md` | 软件名（正文 1） | 1 | `3450fdbab946365026e51cbaa39886daadc2b5148c978cb6c0f575a574a743ad` | `3428ffdab22f3587234c4a621b5e2c4351880fcb1586c69c1fd45a2fcc3f2600` |
| `research/prompts/d13-item4-fua-r1-main-verification.md` | 用户目录 1 | 1 | `bcffbca1e12fe90b4d918fbf12c3f4dc2bcbbe0c1cb9a4f3656290656e355b61` | `3c74bf7f4e54a3852927b6330e151d308e7dcd6d91bc6a1b2d32dc568a85888b` |
| `research/prompts/defs-gate54-tiering-r1-verifier-output.md` | 仓内路径 1 | 1 | `3cf96d830e9fe282797941c59341698382dff60727f9786b2c865c905fdd1e1b` | `a897f534cf88ec1dd12f17d2b9da483165bca278261b5d5f88d1818e244f9fd4` |
| `research/prompts/defs-gatebatch-m2-g3-report.md` | 仓内路径 1 | 1 | `ee2bcbe2338ee27ecee26f68408255bff694d3f45e4f758977d9630ba4a75711` | `eee1e22dddbb65270145874a1304228ad1510536eb32939c2f702403d6b8acf0` |
| `research/prompts/defs-gatebatch-m2-r1-verifier-output.md` | 仓根 2 | 2 | `ae29efe248f8843363c160b75356e81f18049daa2ea3d804be36978ef2dd2bc4` | `48ec608b8e3171e9e244f96cec2da89ef8fd8c54fa444bd1c0dc51d64adbd5f8` |
| `research/prompts/defs-gatebatch-m2-r2-verifier-output.md` | 仓根 1 | 1 | `fb814cdd3fde5be69a8817babaa96bb41b3d965421cc491e386413e976a7451b` | `494130b17425a92a8b22e9e68d986f70ca4e391de563e49f07f576e5feb5393c` |
| `research/prompts/defs-gatebatch-m2-r3-opus-output.md` | 仓根 1 | 1 | `b3fb28ef44a7ba9686f56ee0ef7a414a574e77a4199ed17729a8f96386efacc6` | `f47b45b6564108e347ce52deaed8f7c1c6c28e1fd3904c08eb572be74fc4f764` |
| `research/prompts/defs-gatebatch-m2-r3-verifier-output.md` | 仓根 1 | 1 | `f95fb4899e28567090774cde464a713a557b3dfb3289f78a74efcd931ead1040` | `f3589056ec43e0e6b381801f75794fc241fa4c864d272ce7f1b5f11fe1f2fcd0` |
| `research/prompts/defs-m2-closeout-r1-opus-output.md` | 仓内路径 5、仓根 1 | 6 | `938c66332edc58fe9c1630b72e9dadbac3032d9cbf331b458304fd65f6bd53ea` | `f1fa2520bb012526651e6939d87ea5deb1287eae47779ed888123038434bfed3` |
| `research/prompts/defs-m2-closeout-r2-opus-output.md` | 仓内路径 3、仓根 1 | 4 | `f3dcebfb39aed0d68beeb7ba6d10be6870c3c78729360756ba1cb8a9cb620062` | `da3afa73f457267b67e24d2234319ad9897c74e7c75983fce1c3b6b98c4cb305` |
| `research/prompts/defs-m2-closeout-r3-verifier-output.md` | 仓根 1 | 1 | `b30778bcd2684b5dfc9215ff4b8615ab232ea1eb1dc4ee5536716a2e26c8b103` | `cc28db4aa7bf8121206bc802ab437551b52a2dad9c6f1132c6e52902cb9f33df` |
| `research/prompts/e142-r15-prereg.md` | 用户目录 1 | 1 | `19fe30438a47a2d91fa7ad6a15e7bebef7b3d8934fe4872eacb437b4de3d4e60` | `a43653dfbf06bdd7bb73d0fe47dca5ae5c6c71bc982eea0a944720a58512871b` |
| `research/prompts/e142-r15-s1-runner-report.md` | 仓内路径 1 | 1 | `0f546d4549a1e3f894c6486daa46f32a179e8c2fe7423ef30d21321ec4854afd` | `c200e90ac60b54440ba0ad9274a41d858d8159fea7edf0688c2d1e3d016ea33a` |
| `research/prompts/e142-r15-step1-runner-report.md` | 仓内路径 1 | 1 | `460a358dea0ed46df1f2c4efd8d243474ccb2756306fd08fdb3df5acdfea0325` | `303ea03260310e730fde99fe5022719b41453966612dbe6639dc19e8aae9ae45` |
| `research/prompts/e142-r16-prereg.md` | 用户目录 1 | 1 | `a85267c90b8a760aafab98869f7c30f7bc12004a909f5d08595e4c6cfffe8130` | `3d6ae9c05de2965cf39cfce514c5ecaea7ed7df6c2464d816166c5255adfacb8` |
| `research/prompts/e142-r17-prereg.md` | 用户目录 1 | 1 | `dbecaaabe00e6a7a9382d7c229b76c891e88a9e46435f6fe2eaa3dc49311e68a` | `410b148c91965115b1a783d03ae5948cf9a5e3ae411f19a0ab9e49a75f307129` |
| `research/prompts/e142-r18-prereg.md` | 仓根 4 | 4 | `59b793ab961433134a504bec69154d5767f74a148448617528c9f44154846082` | `32116a552bdb43e6df9bee39f52d756fbf9d77f575e416778460916101492950` |
| `research/prompts/e142-r19-designer-report.md` | 仓内路径 1 | 1 | `8b08e091af04eb43c29314bd58f73fad37b62d7fa10086128eda224a4df610e3` | `fae9456d0ed4aec8b39b8af862d3d9e8df024efc1534a0feb5bf14acb138bd70` |
| `research/prompts/e142-r19-prereg.md` | 仓根 1 | 1 | `a11d97e8ac35200b39f159a4bd5b366f9fa0534ba4f85dcb2e871ea648788524` | `a7fadb5804418b60fca1cb8ae96254f98e2af4aa63c9cda67f600e9b33ddd9d2` |
| `research/prompts/e156-r3-prereg.md` | 用户目录 1 | 1 | `93cd3e87a580a8965c879c5a4f8480cecd705a6ac9ec353bcda4e9e98628a7f1` | `cd71258206f3714ed1df83a87e56a8187f682da0c994df794c8bf2d600a66ece` |
| `research/prompts/e157-preregistration.md` | 仓内路径 1 | 1 | `78ea5c26b6c2c6e8883d40143c713d3ac819f1e88d09d9b7e44a5921da2c600a` | `44ae2eed908badbd9405c50cb454867645e5920aad220851dd7a63920af072a2` |
| `research/prompts/e158-preregistration.md` | 仓内路径 1 | 1 | `ae745675ef93deabfea8aef16605d827d12d1639b6ace68b3a192a8273eaaf19` | `6824eb6c789d63aa64381b1a0574facedd444b21878ed320be092a27a6c0634b` |
| `research/prompts/e158-r3-designer-report.md` | 仓内路径 1 | 1 | `7a97295220f517526d8d41481b95aa6d38653de524983ddeee662c46afb86245` | `cbfed083108f78533863782469d7dc11fab814c32894e5fefbe98cda6b8190e7` |
| `research/prompts/e158-r4-designer-report.md` | 仓内路径 1 | 1 | `6f4a8264b0a770aa668d732e4cf5da159bacdd3a530af6973e8b7685d1e14c7c` | `dbc643a673474731e03078d584b4b5ab11d769864f8bf61643cf479a2b4c104d` |
| `research/prompts/e158-r4-seg2-device-runner-report.md` | 仓内路径 1 | 1 | `52217422f2fb9206bb511ef0d7c6b0bfdf82f9e303f475d071f5e36911f42564` | `e1f0dedaedd7f739bdfe0405da8e333cb8c7d455c06d5b328951e06c639c351e` |
| `research/prompts/e158-r4-seg3-runner-report.md` | 仓内路径 1 | 1 | `f4fd072687fe3a161b7d360e166b3fd72edd63c8adefce354b2e0f0e3078dc9b` | `d1282bc799c658922b977b54bac887a74f1496df4002b9f3a10c2b2b8b20f3b6` |
| `research/prompts/e158-r4-seg5-runner-report.md` | 仓内路径 3 | 3 | `c8ec7604d3014f92f6d30a331ebd03b2cd1132f30466c349a0196c7101dd8870` | `5bd097275ef01abdf2698182543f5b2faf256f287dece10739277d9518e0ffc6` |
| `research/prompts/e159-preregistration.md` | 仓内路径 1 | 1 | `274446ce2dae009ff5380a865f418762ddeb041191a3005acd9eb7c533f489af` | `4056f29f4b93c77b3c70d297d74832a1e45e2f961198b489c622b1452163d84d` |
| `research/prompts/e160-prior-art.md` | 用户目录 1 | 1 | `428e5409f22b994fb01005cb4d6090b230e2715f7a75ddfb3f82a5aee4bef7cb` | `dd40a11c67eba1c42b4811b2e99fa885cf955bca2cc896376f7781fbd30a7916` |
| `research/prompts/e161-preregistration.md` | 用户目录 1 | 1 | `8422d600619b8ec58694ddb5d3d96a6381d780a1827c039c19a87726ca949a41` | `e2ccc43410234f2eb067c64d8783a75a8793035ec3735d0d77da64a2275143b6` |
| `research/prompts/e162-preregistration.md` | 仓内路径 5、仓根 1 | 6 | `906809a9734cd04bc28f0862a59e8e08c6698ab6d65d2936adaa127697041b56` | `dacf742b1485a3485ccb48437fa4119be082da122d79304745b37614bd9593e8` |
| `research/prompts/e162-s3-s4-designer-report.md` | 仓内路径 1 | 1 | `8bff742d16d4c9bf18b080ba6fe20870faa90b01256e8488ab6a58cf86eab015` | `d324deceb11a78c9e13838e41db89c47512d020f193ddf07f1188511d7448b85` |
| `research/prompts/gate-fix-forks-r1-local-attack-handover.md` | 仓内路径 7、用户目录 1 | 8 | `667c80e863c19096266dd0ba10f58ec3bf18cb24913d74675d9e844be6564864` | `e695100d08c976d828ebab824dc94563ead5623bb91ed2dce37d55c8854d1f59` |
| `research/prompts/gate-fix-forks-r2-local-attack.md` | 仓内路径 1 | 1 | `2dfe3f9efb87e12206e35f3b5095d4c080c2e5c172d482c1c5fbc00415d5af87` | `e4bc23e7f4f2660fb95b759d3450a40878413f370c7a54b19e79fd9c0e05f1a0` |
| `research/prompts/gate-fix-forks-r3-sonnet-output.md` | 仓根 8 | 8 | `a7240a19e7da04e14c2e4e519df9adbc91fc49c231d61529aa43f6c6a37f92b3` | `5f3e6be1bdd6f557c7905a06cf52bb17f42b0c42c19e37b043a90f2ee97ca617` |
| `research/prompts/m2-checker-supp-code-r1-opus-output.md` | 仓根 1、软件名（正文 1） | 2 | `d77e1c47d65a58433e41e04a4ed5673b3ee1f75a14d62d6f5268fb553daa2683` | `3b358d08bdf1829c0f36ef84a5b56efff74a14cc2a9bc75695b27c9a6a341dd8` |
| `research/prompts/m2-checker-supp-code-r1-sonnet-output.md` | 仓内路径 1 | 1 | `151b9f509736e19389dfbd3686f9b73d0b8e4817d531fa11b0eee20802bf9fb8` | `1c6e6d0152444aaf9c9fd7cfd9e607c8462af1b743a2fca46031c268108350a0` |
| `research/prompts/m2-checker-supp-code-r1-verifier-output.md` | 仓内路径 1 | 1 | `e18e93475761eddf658a68044b86548522a3dc41aaf3cc55f0f1c38eb115386f` | `ebd11cc84d7c391aca668409d7dcb3d5bfd782d32aeb7799b265c9747985b8ec` |
| `research/prompts/m2-checker2-implementer-report.md` | 仓内路径 1、仓根 2 | 3 | `eea0eb3eb36196b0f12b2913e5110b59e1a9abf19a5e60dab175315afc968de2` | `8d097746198ab4cc0c7fefbfd69e2e49fdd51a8884f02b28b125c5645a6d3c02` |
| `research/prompts/m2-closeout-code-r1-verifier-output.md` | 仓内路径 1 | 1 | `71c2f1c7626bea8b31f560d29d5c9595f8f722a88f217a9b93f2fcd0eb0375e5` | `6fc56dfb735197533d1c60bb92f6118cb889bf2e9d630e43cd7a05564704f85c` |
| `research/prompts/m2-closeout-code-r2-sonnet-output.md` | 仓根 1 | 1 | `bb634470f532bf79da737f2a03ebc5a05d716eb18ef3db6c8dd99471288ec65c` | `9bf7cd8c2e3da8669419c9c0cbb28e3256d9e36579d5c5605796628cfbfb4108` |
| `research/prompts/m2-e26-implementer-report.md` | 仓内路径 2 | 2 | `23055e00b2ac3ec9cbcfeffd47d1c1007d7cefa11f8e839a6e239ebb92f427d3` | `e9266ce67dfd13a0625889ba53dc84daccd0de093e4bb6b54ffd7f62cab531f2` |
| `research/prompts/m2-e27-implementer-report.md` | 仓内路径 4 | 4 | `afb721175d42bc72a5a07fe348bb47d410ec0f911a660e11b909e54e9ac1f154` | `e1babaa56a0edcebbc12f3b2a0966cbb58cd2ec8649805d5480f89b788f4798d` |
| `research/prompts/m2-final-code-r1-opus-output.md` | 仓内路径 1 | 1 | `61970fa36451f58820ebf550216fa09964541c5a0ed6d668724bcd1960916b5c` | `e02cf0b4ab149412b8be753fad8935cb24c83a82f5d55a7f6afd1b0166e9a410` |
| `research/prompts/m2-final-code-r3-sonnet-output.md` | 仓内路径 2 | 2 | `ec02e15f96aec6f45c45646e86544a7e750f709fbd2a251accbf676005af63a0` | `f854dbccf139410521739ef5ee82fe764b341a6363d24d08f2e886864806eb82` |
| `research/prompts/m2-gate69-tmp-evidence-report.md` | 仓内路径 1 | 1 | `a854889ecfbf32866641365142734ce2abbaa96887a58fde134a6c2bcf483f7e` | `5a880586b65cfb1c9bf3214657668ce139709515927816a23370d63dbf51a9fc` |
| `research/prompts/m2-impl-rbf-4a-implementer-report.md` | 仓内路径 2 | 2 | `7b6488dc73c851ea1a9ae56f21b1530fddf430f8187d99ca69ce3554da1920f6` | `3f8687fdeb7daa802dacd08f93afb60e4426297d251b7acac7568e20d25acd8c` |
| `research/prompts/m2-impl-rbf-4c-implementer-report.md` | 仓内路径 2 | 2 | `f53a6dd85099f133a652c8fef038a21b64b929c5169f51789b27e7d039a4faf9` | `e3bc8980b3a4575456695cd2c0104d14479b71f311efc006783d22a681ffaedf` |
| `research/prompts/m2-investigate-three-reds-report.md` | 仓内路径 2 | 2 | `c43e84471abd1774a4a9bf17b98ce9bb8a3666f0b7cbfecbfe1afc0db18f30a7` | `23a9983baef9def5c1f4e73c57fac57fddf863c0d084a324a22cf3a0daa15588` |
| `research/prompts/m2-keyspace-implementer-report.md` | 仓内路径 8 | 8 | `711884bdc2a04b88ddddbe7d56fe9f09a8e768ff19e3833293bacb1b015464dc` | `0ae21d90ffef3c5577b24ecd4f2934b9bae45b187de25df64063ae4e248f769b` |
| `research/prompts/m2-keyspace-r1-sonnet-output.md` | 用户目录 1 | 1 | `b38986eac09431a0414e84944ad02e0388d31c9a2133a3c0042b01ccdf7d3888` | `1692b43e019ef2ab3158999b4d15d3b512ac9e83e312b5e586681f64b7d0da0b` |
| `research/prompts/m2-placement-falsepositive-r1-opus-output.md` | 仓根 1 | 1 | `7b47498fca45c37c6e6d58a429516e731785eda52e3066b61d65d2a8e9f8ad73` | `94fb69f23c09083966bccf5a7f238de352b38aa0ce577995d51bc4e242db5202` |
| `research/prompts/m2-readme-status-report.md` | 软件名（代码 1） | 1 | `7a4913feb08dd35b167227903ee0aac53f58cf79ba3caa8f7a6978c0f3dfdce1` | `8c610aedf837284aff3288f593f499cc72e146c4121cc69d0f98f4a02263799b` |
| `research/prompts/m2-rev-a2c-implementer-report.md` | 仓内路径 1 | 1 | `628c95ec1fe042f34d5502ad3fb8ac6e558eb4dae6d11472ab9aa59b89e94a9e` | `b4d2b9a15c30c0dfc3c248aeea3a2a16a28171a420e40d9ba9fc4b5d877a3d0f` |
| `research/prompts/m2-rev-a3d-implementer-report.md` | 仓根 1 | 1 | `5610c40c0cc8ddbc334cba1248e293ad0be2c195f0cd89500ec28727f2c2021a` | `69d1905890869e430fd6e603a5b112068e348ba0c304759cbe46c8db918937e4` |
| `research/prompts/m2-rev-a4-implementer-report.md` | 仓内路径 2 | 2 | `23d9c1e713e3a091b20a7404f08546dc82deb87314d0a04046fc4bd9c20be0dd` | `7d1a9a278164b01582b11763c8c3fae8b8355b84fe634b797f77caa50b08b4ee` |
| `research/prompts/m2-rev-a4b-implementer-report.md` | 仓内路径 5 | 5 | `ea6732a60d529beb321ed5f26bc8df4137100b972bee77d337aca5eed378f0c7` | `5b5737d1626c5dec4b5ec6cb726f6e881485d38628ba05dc33816f089a130b2b` |
| `research/prompts/m2-rev-a4c-implementer-report.md` | 仓内路径 1 | 1 | `f368e5f73db2ea95e3fd3e0e02f6ae1d762b55dc67939c3e4eb79bd4d5311ef4` | `363bbd8756859db17276f356aa8b7a111949c05e8c85b0664a42440132b2dd32` |
| `research/prompts/m2-rev-b3a-implementer-report.md` | 仓内路径 1 | 1 | `4847112cc6f09eb7c5d587218a5788669381f395eeb0224fb38673934b8b9cda` | `87b29a395bf11b20874040a5524c4bfbe0e2ef6803161846aa0d729a83439902` |
| `research/prompts/m2-rev-b3a3c-implementer-report.md` | 仓内路径 1 | 1 | `4103eff91afb85c0e471eef126f6a2d32f4e88985653a09710bbac4b39d5a925` | `28c5ee8c08685205724fc5fc1c710f2d1704313dbb95125461ff818fa1a566ae` |
| `research/prompts/m2-rev-i111-implementer-report.md` | 仓内路径 1 | 1 | `ee4a886cd282350d56eae58e140de94955badeb5d8a8ac9f9ebacf8c79afcc41` | `d732d7f072a3497da1093d97b13b2960610706e9f44cd986cd88a7dc7c540ceb` |
| `research/prompts/m2-safety-r3-sonnet-output.md` | 仓根 1 | 1 | `7d5f82dc0d53b6388ee2e526f6955c4595b2de3202ba2084db62d6aaeaf23f29` | `cced975cdd3a3518195b423df12ef9b40b0b5d2335ec4d2d304d38bf2697d4ee` |
| `research/prompts/m2-safety-r3-verifier-output.md` | 仓内路径 1、仓根 2 | 3 | `1e12defc3a0f32fe5d3f8edc58f31a1eb766a4f42b92328a6cc42b3970de6bb6` | `7e416d767e116833b41779a944df1b8ce698eca9fbf570f8d0551c17c4df4179` |
| `research/prompts/m2-supp3-item4-code-r1-opus-output.md` | 仓根 1 | 1 | `b5842ce4427d7ff8e080747557a22ff259f50ce994384afa403e7c2a44f91613` | `3df79d1fa6b15bd7c359deca7e997c2e9dc54bab7ef29a9a4bf3dff757abcd75` |
| `research/prompts/m2-witness-r1-opus-output.md` | 仓内路径 2、仓根 1 | 3 | `cf0272d54d6e22b58cc86942eeed6d36784b6ad756369f962e319275ec7a3651` | `32405560e1d08e0f26b552d04d70cfb8e8fa9904ecdde03b355c54b8bd1878bc` |

`research/prompts/` 的子目录（68 份）：

| 文件 | 类别 | 处数 | 改前 sha256 | 改后 sha256 |
|---|---|---|---|---|
| `research/prompts/agent-analysis-2026-09-27/scratch/extract.py` | 用户目录 1 | 1 | `95eff5c3525ac037c89d963a93bbef985d3c2eb93e96d56d4349c062e583877c` | `47568041a2ff79dd0190029986cd3acee5f03bf380383b1b400625b0a0439228` |
| `research/prompts/agent-analysis-2026-09-27/scratch/substats.py` | 用户目录 1 | 1 | `34246330921cffa52f21124aebe30fdd12ed016d77b789cdea91cbb0bc9c1d4d` | `ba913b4867100dacefe0b311519a4630c64fe44f2682006ed669ff9e51539450` |
| `research/prompts/c355-c363-r3-opus-model/results/w2_count--mutate-ignore-device-cover.out` | 仓内路径 1 | 1 | `5d95ca30b7dd5c2905bead3228d6fbdb3d460dee31d914ac9d1480ad059e13aa` | `353877739004f37e99d79e8d0120b2ba8f1246a124e3934ee7cd68b9ed2dbfeb` |
| `research/prompts/c355-c363-r3-opus-model/results/w2_count--mutate-split-pays-no-fixed-point.out` | 仓内路径 1 | 1 | `4aa493d024112c7a287cb567c31d7b3c3d161c0ce74da4c2ed3fba3426e8237f` | `2c6aace3bec38fe40f735e57a1c6686093ced932535813c0718d6c9d8ceb12c9` |
| `research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-translation-audit.md` | 仓内路径 1 | 1 | `b682478b01099f855b37cf7571981ca5af1d2445b03744e53f0d36777575ded1` | `41ca6ba7e59a3504c60e628a03ca9a14a24d4cc8cf50f020afc1acae14861371` |
| `research/prompts/defs-closeout-draft-2-tmp-evidence/report.md` | 仓内路径 3 | 3 | `651fee1e009d1f5e1beb5e869308612ba1b322de24d1b64f16921fc4a672608d` | `a9bdaf65d0c4f84c7d3534423a52bc7bc8a288928eee31a5892a40537911371f` |
| `research/prompts/defs-closeout-r1-fixes-tmp-evidence/probe/f14-cases.json` | 仓内路径 4、仓根 25 | 29 | `df2d1ee879012b06618b752c121ae5d713450eb0ab836f42df004e02ed94965a` | `7e3b1605d9a76d690b8835980eb97ece8e35b578e2239136912c27db9b5c863b` |
| `research/prompts/defs-closeout-r1-fixes-tmp-evidence/probe/f14-out.txt` | 仓内路径 3、仓根 1 | 4 | `3109772f30121c10fb56fb8504caa97c76e56a579bd975b35481825cf876dcb4` | `e556365951d60b48321e668b1b8850ca6305bdd7275fe2da36ce3eadf87fd82f` |
| `research/prompts/defs-closeout-r1-fixes-tmp-evidence/report.md` | 仓内路径 3、仓根 1 | 4 | `a066e5cea56928e497b50922e77cc0d876a0a7d5a2c4f3e1e1dca3c5993267b9` | `66c302f71cf18727c0b624f119d40925d9f744e5147f2ad5a91222cd55d148a7` |
| `research/prompts/defs-gate54-tiering-implementer/r1-report.md` | 仓内路径 4 | 4 | `ae261b5a926c2840bbbc6c21b1b8bd93f73fd0094cfa51a74c276a560dba0631` | `d12c19ae40d1076a23a405ea8f184086935cb9eb0674943dc0a0e791912bb30e` |
| `research/prompts/defs-gate54-tiering-implementer/r2-report.md` | 仓内路径 2 | 2 | `0be290733b82098b5a8abc0a10aa4cda9d80682210ac338ff1d208ef64f6a17c` | `d19fcb04ac0a16645c63868ed339c0e646dab8f91e6f17d238495dc734338a3a` |
| `research/prompts/defs-gate54-tiering-implementer/r3-report.md` | 仓内路径 2 | 2 | `ab2167fbdc4ca5e13744879abcced4f5e5d4ee256ee02d93b38835975518f4eb` | `70f5666aa16ee2b1d98a54668ba4cc02549f7e617f624505b42d5003754a1a28` |
| `research/prompts/defs-gate54-tiering-r1-opus-model/lib.sh` | 仓根 1 | 1 | `dd1909c513f18033f44e84e081c4a4e96178cb620b500ceb2512f9619b603054` | `2d099790619fd21761651ec8b6863378a3ababf73f84430e95a48f060aec3fbb` |
| `research/prompts/defs-gate54-tiering-r2-implementer/report.md` | 仓内路径 1 | 1 | `52378256606673cbd37b9dfd3a407a1815b448a882cc34d74fd6b405580a2c9f` | `d69437778c50e10fc6e4f96c52ae9967208d27affd28e45c0d22090d9887e0d3` |
| `research/prompts/defs-gatebatch-m2-r2-opus-model/outputs/fix-k1-hook.log` | 仓根 1 | 1 | `ba9d65988670b68ed857d50fa43dfe66a3a7882f8c37ff0bd1c23ea3076f3e55` | `1abcd943554bde83dea51a92cd10906f1988065c1d0275bacd9860cc756d71c9` |
| `research/prompts/defs-gatebatch-m2-r2-opus-model/outputs/k1-hook.log` | 仓根 1 | 1 | `8e0bd3619f7300327e83545b9442a4b403bf672de5ae8abc9793fa49b616f227` | `eca76a2bc76d6504029da337f2b50b1f909e59b455293efa4f7498348a898c6e` |
| `research/prompts/defs-gatebatch-m2-r3-opus-model/fix-lib-heavy-tests.diff` | 仓内路径 1 | 1 | `6c833e0158c4688a3e5aa7729c081196aad94c29f22f6852815ae7529bd59b6d` | `d5bf92794e3dfe89d54f99b43f600486f26a765efca7bd11bdeb0d6b747da72a` |
| `research/prompts/defs-m2-closeout-r1-opus-model/cases.json` | 仓内路径 1 | 1 | `f1f646615e1ed0a6ee82a0fae78dd9a7c3cc86892b335990c90316dfe7aad0f1` | `5e4dab06f6ffb446b9076acfc822b338265b2f67408767fb1e9533b8c1d38917` |
| `research/prompts/defs-m2-closeout-r1-opus-model/probe-output.txt` | 仓内路径 4、仓根 1 | 5 | `c7e4add2b397336c63cc31fe7b5390212944de97ff754faa960b7fe558b12ad5` | `d8b830430d673d6f58dcfd1bfa3c8f460211ed9f5ced553affd740ae80d025a5` |
| `research/prompts/defs-m2-closeout-r2-opus-model/cases-hooks.json` | 仓内路径 3 | 3 | `9d6eab865f5c60736d2e2d4ce258960af99c9fa7b52f3f98f9fdddef939aa0fb` | `02cf763e54b592d1a6aaec4ddf9ed504a94953c9e72195b60945837765b31c5c` |
| `research/prompts/defs-m2-closeout-r2-opus-model/rerun-output.txt` | 仓内路径 1 | 1 | `d2703c2bfa0fcd8109a79b2652677c5d56d125439824563d81cf763548373699` | `82b3deddbe6841d29c4ba2b8d9ee1b58551dc8588e7ced572a9db33c9fc7d481` |
| `research/prompts/defs-m2-closeout-r3-opus-model/cases-k1.json` | 仓内路径 4 | 4 | `e933b487c4a0fc1da8561f82a9ab8d18493c3e2d5279f7cb830ca048d0d96c85` | `6139855911959f831a9e37009bf9b4255244e2ed257cd01fdb5fa7535415ebc0` |
| `research/prompts/e142-r19-runner-evidence/build-tests-1.log` | 仓内路径 1 | 1 | `108c307b1be0b93cd205a8e51998e6d8c80b931406a3a3eebfbc55634c33f9dc` | `c73309848c798c72f064602c5ac08e8c8a72221a1d796158e8098a1ff1fdf584` |
| `research/prompts/e142-r19-runner-evidence/judge/pc_c.sh` | 仓内路径 1 | 1 | `0392bfdbe14230dadca620a9c19d63c54fe0da589ab2e5b99c2be7d93177cd66` | `fcebbef8f0ec988d136341002adce399caed45524ae7bdf20ae27d2ac796a044` |
| `research/prompts/e142-r19-runner-evidence/make_mutations.py` | 仓内路径 1 | 1 | `dfa01a82974f119deabc046307f683d5660fb68dfe187f416135e24c412ef365` | `167fd7fe1d7804d29fcb3f8146d897f84caddb8e6557217370b1afd579716fc0` |
| `research/prompts/e142-r19-runner-evidence/naming-lint.log` | 仓内路径 1 | 1 | `e971700930a6d7c7e7a2e96cfe867ae17b03ab50bd79575b02a28de0092ec81a` | `a0c4f45c7ba47c4f9c5880f0ee2bda702d0223599c4a00a504530ee379a50416` |
| `research/prompts/e142-r19-runner-evidence/run/arm-n18.exits.txt` | 仓内路径 1 | 1 | `c8af94558e9415bc051f8782a309bac8785c891daefdd998f7c1a208d997ebe6` | `70286979cc2f2363c03dca8fe48b8c7a144aa9f9fda9effcf8af32bc1fdad62e` |
| `research/prompts/e142-r19-runner-evidence/run/driver.sh` | 仓根 1 | 1 | `df7411979bbce8b7c8267d852de94f41e6ada34b196808595fbb4da0ae99efde` | `831f4ef367ec385f3cc3bf9cec0b0431eae96b142f35b494587598f56546d455` |
| `research/prompts/e142-r19-runner-evidence/test-2.log` | 仓内路径 1 | 1 | `65fccabf8bf7f9fd2a815f00d95b4e74a3182d7d85bb36dd346619e4c49bccee` | `b81cb3268e4bd69c07d330f41e97f54e6b8f6cda44c691678575d41d56f153dd` |
| `research/prompts/e162-s4-apparatus-evidence/build1.log` | 仓内路径 1 | 1 | `7202726b312a54b58e706d62c1afc203afb5a0ceb32f0b2036f876181dbec693` | `ad942a5fd9cb23e8d9e4b6efc1c31c90fd368b9dc34236ac22cbd7fff4600434` |
| `research/prompts/e162-s4-apparatus-evidence/build2.log` | 仓内路径 1 | 1 | `bd6a8d13fd79dd2c9d06a48b44cd5fb49071eccf0558016e6dc6394eb0b37ba3` | `90007c8c64857df0bed6c1a56bd03236e67ef42208342ce28f484f6514be091e` |
| `research/prompts/e162-s4-apparatus-evidence/clippy1.log` | 仓内路径 1 | 1 | `2dd6b8963f2abc2a5d384988d8618a87277c22cf5fd29ee2ea657cf4ac18cef9` | `78f55c893d18e562041b72fac62699b00812d9aa625fa09fb3d15f18f876b637` |
| `research/prompts/e162-s4-apparatus-evidence/clippy2.log` | 仓内路径 1 | 1 | `04fdfd067e2226f2750f0438e44138e4dacc8d814219f2982e70ffaad2e0468a` | `58b0562910c598203d1bf7369dd44c79c35d42b0bc2af9e3b549075b09c58c5c` |
| `research/prompts/e162-s4-apparatus-evidence/clippy3.log` | 仓内路径 1 | 1 | `e160ea6fe7f645f515081ad662845ad15516eddaa8285666cdfa32915ce2285a` | `b4fa5acf8ab2dd5eeb56766fd1b8bbf067edce1d5cd1dcd28c5e7ff534f262f5` |
| `research/prompts/e162-s4-apparatus-evidence/clippy4.log` | 仓内路径 1 | 1 | `e7884ae03e96ba94f7ba5db8e54c2de6cb3ba10098ffdb9a05c04b5e2a65c71d` | `d0f617eb65ed4e7919fb53577030df0aba8ee4b7dfe8693b57bc1e5dd342d03c` |
| `research/prompts/e162-s4-apparatus-evidence/clippy5.log` | 仓内路径 1 | 1 | `251b32ba08e79b8ebd95eec49e522f1e52dffcbd0f71ca1d80e28c3fd985695f` | `4cf7c24613f3e2c13a20f5a1da23256fc706f16ada7caf0bb1b586e7018c51a1` |
| `research/prompts/e162-s4-apparatus-evidence/clippy6.log` | 仓内路径 1 | 1 | `e160ea6fe7f645f515081ad662845ad15516eddaa8285666cdfa32915ce2285a` | `b4fa5acf8ab2dd5eeb56766fd1b8bbf067edce1d5cd1dcd28c5e7ff534f262f5` |
| `research/prompts/e162-s4-apparatus-evidence/dry-run.stdout` | 仓内路径 35 | 35 | `8c7adc15359541f8f1b2eab0b9f1ab9ed98243c170ebd978c6ec3a8c1b32080c` | `acaee23f7898dae848cdb1466235b8d73ab04609e12bd691762662202f99ba5d` |
| `research/prompts/e162-s4-apparatus-evidence/test1.log` | 仓内路径 1 | 1 | `aa131eef5dfa2c132e707e9cedf444b6bccca45c64eefe9571f05b6196740e9d` | `b719e2b4bfb4c3d740d7c03a488317cf000f40ab9f81d49d652740a1d52a2654` |
| `research/prompts/e162-s4-apparatus-evidence/test2.log` | 仓内路径 1 | 1 | `43d82bb094f12712bfafd3a7c68c35b026ad494d73a20a51254200631fa905d6` | `bddd1b0b6dede524af4e16efaff82719ca9b23aa3056d695f628e2b583458874` |
| `research/prompts/gate-batch-m2-g3-tmp-evidence/final.sh` | 仓根 1 | 1 | `47cc91a7a20a376b17ea81053e96c0a30e7c669d593a4f87f5a39bb55efb9f0d` | `0d4913764abcccb9405e096e160c2cd01e87ae5e8324de8c720ae5bd92d6b090` |
| `research/prompts/impl-rbf-4a-tmp-evidence/report.md` | 仓内路径 2 | 2 | `7b6488dc73c851ea1a9ae56f21b1530fddf430f8187d99ca69ce3554da1920f6` | `3f8687fdeb7daa802dacd08f93afb60e4426297d251b7acac7568e20d25acd8c` |
| `research/prompts/impl-rbf-4c-tmp-evidence/report.md` | 仓内路径 2 | 2 | `f53a6dd85099f133a652c8fef038a21b64b929c5169f51789b27e7d039a4faf9` | `e3bc8980b3a4575456695cd2c0104d14479b71f311efc006783d22a681ffaedf` |
| `research/prompts/kb-writeback-batch3-tmp-evidence/gate-results.txt` | 用户目录 9 | 9 | `9999fc8198ee4bfd83e6e45f1530838ea8696fc282a4ea1925e1623824f97a33` | `3a614be15ed1a3c0dc1da2fac2b697c1a4e00380bc697cbdcc7b942c1919545a` |
| `research/prompts/m2-closeout-code-r1-opus-model/batch.sh` | 仓内路径 2 | 2 | `9f820f9b5629e7b1aeb7c2b9c2aa2fb1da18c30c7c47261aa415f38a7176e2ca` | `5249c751364d573a00b55634577121d563e25446155f6434c4f5e5911771cfca` |
| `research/prompts/m2-closeout-code-r1-opus-model/rerun.sh` | 仓根 1 | 1 | `b9c198cb8bd47d57228eee98b23e536a570054143c2f51f58513952c6d26634e` | `ebf8bd918d599c73ba5f62111269900e46481d2fd6c975f8412aed5b47a8e783` |
| `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md` | 用户目录 2 | 2 | `805287d6343c01d8aebfd412ab479db45ff4b55a175ea1e369e102fdf7e933b8` | `5ebfc56f9e7bd5381e4553a628d67aef3dd1aafbb7358c2a4e5c6a39dc7239f4` |
| `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/decisions/22-单元原子性怎么合成.md` | 用户目录 1 | 1 | `c49d96e56bfb4cdc67856bad8946ba42c68bf45b7ffe72fe548bd72b12724f9c` | `216c8681f9171a6a8335441ced5aa241df519140b99bfe40956e98bbb0e98b05` |
| `research/prompts/m2-closeout-code-r2-opus-model/batch_sweep.sh` | 仓根 1 | 1 | `5008ed13fd5e9dd784f46d5877abb60ab9cfe2a51fc874f6d99793c9232800bd` | `f01d25cddf59face0428f64c4b64a5509d943d4066690d4b97534c670bd172af` |
| `research/prompts/m2-closeout-code-r2-opus-model/rerun.sh` | 仓根 1 | 1 | `acd0b4af8e0327e4c7af695527aa3d51c8f949fc873d97c4352b9cb7608d587e` | `256a889d246c047317f45bfb03221096e056a30bd6c43219e9f01ca5f2ef7c96` |
| `research/prompts/m2-closeout-code-r2-snapshot/kb-at-start/.claude/kb/decisions/08-核心索引结构.md` | 用户目录 1 | 1 | `62f0815146d72ec03093a2ae7d84307b881e48b384a70e0253ee6e16fefb2ec2` | `54fde65d24c28fca7564e81a6e636ad165b8e1b11622297eff8a2d09cf0b79f6` |
| `research/prompts/m2-closeout-code-r2-snapshot/kb-at-start/.claude/kb/decisions/22-单元原子性怎么合成.md` | 用户目录 1 | 1 | `e8ae46bb065b8dce8b3874e5fc7475affb01c45f7b336eed55d7dd8ad56ac2ff` | `999099f6cdc9fd77a41c4df8f22282e0ad81f69a10095dd585c61385ce159581` |
| `research/prompts/m2-code-review-6c/clippy.log` | 仓内路径 4 | 4 | `a3ddcdd924027d9fcb800fba0506e17f12fdb2e3a02b0083bda7e5b30ce0c792` | `0102f2d3140dcc0a9e614a506c6428d8ef5ee73730e2a31dc1f10163776e81ac` |
| `research/prompts/m2-final-code-r2-opus-model/rerun.sh` | 仓内路径 1 | 1 | `1794dc0f6ed0f5f7e3bbd5488022df51866190991a3e055ff67ce2a22ee6dd9a` | `7a51c68d4a05a7d3776e05cd65b48914ea95e90226b476297c958e137aff8493` |
| `research/prompts/m2-final-code-r3-opus-model/rerun.sh` | 仓内路径 1 | 1 | `3c7c4159099f239a2cbe174a6d4a648431560edf318b20090bbcf0d3ac05f8f5` | `29b4ced00f1e16b28c23ae0457e6cfc921fa975557ae950e18cc9d663207de9c` |
| `research/prompts/m2-final-code-r4-opus-model/rerun.sh` | 仓内路径 1 | 1 | `d8373c15993944883d0849737366a59aed2caafa4c1b58d4208c02bcd650bbc9` | `9daf6a275b5511b0a92e528fbcf68b72f7771fc8f13f8aabc0132f6a46e60699` |
| `research/prompts/m2-impl-r2-fixes-a-evidence/build-patch.sh` | 仓根 1 | 1 | `46a6905c36fe18ffef657c7c3c9ab5fca91e78d0a6355181fbf7166bf03a58b1` | `7de34c25e12cd356fc19419bd77a794af9f556ff7c9d91e13d2e30413cd9d167` |
| `research/prompts/m2-impl-r2-fixes-b-evidence/run-all.sh` | 仓根 1 | 1 | `58e31a958bc36660b47ef7bc3eb0de8e71ebea311aedd0a71e89e45c88c9fdcf` | `07c7585360bc82b5f17c59f2d4a591bf7bfacfe2d8e9ec6e75f093dee5cb62fa` |
| `research/prompts/m2-impl-r2-fixes-b-evidence/sync-from-main.sh` | 仓根 1 | 1 | `fd25e3dc062ac374badaddb2b9ed4753e362a80c161e68e59c5eec8fc3555f3a` | `25793a09d7d7cf7757fc9a2c00e1ceca1adb3942eee0eb0ced73d6c9b47b5d45` |
| `research/prompts/m2-safety-r1-opus-model/rerun.sh` | 仓内路径 1 | 1 | `df4096456cac71a14893be7992155b391e2897de1775fe6c290c482f6b6ce4bf` | `bb4a938378651a5863fe1686bc47a72d739d0689dcfd9b7040cf9a900b08a732` |
| `research/prompts/m2-safety-r1-sonnet-model/rerun.sh` | 仓内路径 1 | 1 | `f7b30d9274642592a53bba18301d1a340c714041eea85cd12c43317835d8c550` | `67ce93b66d45a62976501e6e5e77a67a74c717b2c43b22d2db9961ef1b578d36` |
| `research/prompts/m2-safety-r2-sonnet-model/s4-r2-candidates-run2.log` | 仓内路径 2 | 2 | `3fc4f46d4074357b9cc9805f0b05c261f82705031da6ab90d61e81844fbd5ab5` | `fad6e2765a3b3ec2edf66b047e7b49ee945e0b9214f55fc69e20d9733b305dd4` |
| `research/prompts/m2-safety-r3-sonnet-model/rerun-output.log` | 仓内路径 4、仓根 1 | 5 | `3c4375299678a93abb86b93b2eab7042f9d7828bacb35ba7690106ba6fc677fc` | `44af0c3521cb8205b129087dc520ff565adf3347f6785fac349693324e3eabc1` |
| `research/prompts/m2-supp3-item5-bad-disk/second-campaign.log` | 仓内路径 1 | 1 | `426d867f5f9bd67f613630cf4acefce42f66da2fdef278d9f9c0e00bc75c9030` | `00606a717f3e8e874dd9b9f237da64245f9041be223da89e8b31092cb7a4b353` |
| `research/prompts/process-safety-tmp-evidence/report.md` | 软件名（代码 1） | 1 | `1eac244fdb66596408aa68067d6f6d9d146881966965ff2811bb1e07eebf968e` | `699958718704e915c60388359a457d2e916eb7a1a5b49480d3f3c58e86fce1b1` |
| `research/prompts/sync-local-legs-r1-opus-model/cases1.json` | 仓内路径 12、仓根 4 | 16 | `9020f977cb9c4825f05714ef701c3b2d0d1644fd5283c89e126293757b61b7c4` | `774b9fbf23347b9594fa13c5a63f94e87621277bf58f30b55370273e6c74a9a5` |
| `research/prompts/sync-local-legs-r1-opus-model/hookrun.py` | 仓根 1 | 1 | `34d1581099f05422fcb929f806c83e1ce8eca57ca2e48f1f76ad5c09c8f1aff6` | `9b08253633ce29cc34fb0c272eed21546d6ac4036bb444393a9561e5ffd944bf` |
| `research/prompts/sync-local-legs-r1-opus-model/sim_watch.py` | 仓内路径 1 | 1 | `dbd45347950f87ed17175d00232c6401c7532e78e3f583db7429a2a54c76e3f9` | `4ca36178fdceae4cfa3ed1408526fd598ea7cb07f0f1be710a4090c53ef0c209` |

`records/`（5 份）：

| 文件 | 类别 | 处数 | 改前 sha256 | 改后 sha256 |
|---|---|---|---|---|
| `records/2026-08-28-原子性与日志定案.md` | 用户目录 1 | 1 | `0eb3cd5d0017eec7864f861a5798b74c200856da770a44ca4c10adaba72b554d` | `f0384087252246fd475f3dd75ac15e3cb38d490a1a2386883486d8ebb7364998` |
| `records/2026-08-29-复跑复核轮.md` | 用户目录 1 | 1 | `8448541221caaa1c2f72cb8ce9e5a8407089c63eaf463366bf62cdf2677f32d1` | `7cb187f8d987dbbd3f616040f65f5d3cce5f2416b2e223b732e2bbfb91265820` |
| `records/2026-08-29-审计轮-外部引用复核与阻塞集.md` | 用户目录 1 | 1 | `c72f60eda3b5b37b51429af3cc40f3654d5239c689330066957945d5bff5c633` | `6dbcadfc0146d81c6489fd8ba5bdda8b5d275c92d0323b6107a12ee3de3fa9f4` |
| `records/2026-09-16-subagent拆分提案.md` | 仓根 2、用户目录 1、软件名（代码 1）、服务单元名（正文 2、代码 1） | 7 | `a75c8a96db9a41b1848feab9fb8d524aaf5a7a66ad970642be7036a1ef23b8dc` | `ec1ae00564bc4f4c2f2706fa8a3eae147ab4376c5857dbb9800fa3c35d25a746` |
| `records/2026-09-24-里程碑二收尾调度.md` | 软件名（正文 1） | 1 | `5e8a169ec111cbb926fb42301968e334cbe7ab34312910c544527ead2252f972` | `646cec3855ef1e2f5361f03daa17b5b04959162f113fb7365320b0d4b43b161f` |

### 第二批留着没改的

两类改写规则的模式（本机用户目录的绝对路径、软件名不分大小写）在 `research/prompts/` 与 `records/` 里改后都是 0 命中。下面这几处带本机用户名，但不是 `/home/<本机用户>/` 起头的绝对路径，两类规则都不罩，没改，交主 agent 定改不改、改成什么：

| 文件 | 类别 | 处数 | 为什么留 |
|---|---|---|---|
| `records/2026-09-24-里程碑二收尾调度.md`（1）、`research/prompts/_defs-gatebatch-m2-r1-diff.md`（2）、`research/prompts/d16-item1-r3-main-verification.md`（1）、`research/prompts/e158-preregistration.md`（1）、`research/prompts/gate-fix-forks-r1-local-attack-handover.md`（4）、`research/prompts/m2-c511-c512-implementer-report.md`（1）、`research/prompts/m2-code-review-6c/00-handover-message.md`（1）、`research/prompts/m2-header311-sweep-report.md`（1）、`research/prompts/m2-witness-r1-verifier-output.md`（1）、`research/prompts/sync-local-legs-r1-opus-output.md`（1）、`research/prompts/verification-split-tmp-evidence/rerun-failed-on-head.log`（19）、`research/prompts/verification-split-tmp-evidence/rerun-failed-on-staged-tree.log`（8） | 会话暂存目录名里编进去的本机仓根路径（`/tmp/claude-1000/-home-<本机用户>-code-singlefs/…`） | 41 | 路径里的 `/` 换成了 `-`、编成一段目录名，不是用户目录的绝对路径 |
| `records/2026-09-17-已分配口径三方与两个实验.md`（1）、`research/prompts/agent-analysis-2026-09-27/scratch/extract.py`（1）、`research/prompts/agent-analysis-2026-09-27/scratch/substats.py`（1）、`research/prompts/gate-fix-forks-r1-local-attack-handover.md`（1） | 会话记录目录名里编进去的本机仓根路径（`~/.claude/projects/-home-<本机用户>-code-singlefs/…`；用户目录那一段这一批已换成 `~`） | 4 | 同上一类，目录名本身不是绝对路径 |
| `research/prompts/defs-gatebatch-m2-r3-sonnet-output.md`（2） | `ls -l` 输出里文件的属主与属组 | 2 | 是用户名，不是路径 |
| `research/prompts/m2-readme-status-report.md`（1） | 记下的 grep 模式里的用户名（第一批「留着没改的」最后一行同一处） | 1 | 同上；同一个模式里的软件名这一批已换成 `<本地模型服务>` |

第一批「留着没改的」表里 git 作者名与 GitHub 账号那 7 处照旧，不在这一批射程里。

### 第二批之后对不上的 sha256 与别的连带

这一批改的 160 份里，有 51 份的改前 sha256 记在 41 份别的文件里（模型目录的 `SHA256SUMS`、快照的 `kb-sha256.txt`、腿与核查员的报告、第一批「改了哪些」表的改后列），从此对不上，原因就是这一次改写：改的只是「第二批改了哪些」那几张表里列的那几类值，文件的其余字节没动。逐份（按记旧值的文件列；是按改前 sha256 全文 grep 找到的，只记了前缀、或记在 `research/results/`、`records/`、`.claude/` 之外的没找）：

| 记着旧 sha256 的文件 | 它记的、这一批改过的文件 |
|---|---|
| `research/prompts/c355-c363-r3-opus-output.md` | `research/prompts/c355-c363-r3-opus-model/results/w2_count--mutate-ignore-device-cover.out`、`research/prompts/c355-c363-r3-opus-model/results/w2_count--mutate-split-pays-no-fixed-point.out` |
| `research/prompts/_c510-date-gate-r1-local-attack-output.md` | `research/prompts/c510-r1-main-checks/c510-date-gate-r1-local-attack-translation-audit.md` |
| `research/prompts/defs-gate54-tiering-r1-main-verification.md` | `research/prompts/defs-gate54-tiering-r1-verifier-output.md` |
| `research/prompts/defs-gate54-tiering-r1-opus-output.md` | `research/prompts/defs-gate54-tiering-r1-opus-model/lib.sh` |
| `research/prompts/_defs-gate54-tiering-r2-appendix.md` | `research/prompts/defs-gate54-tiering-r1-verifier-output.md` |
| `research/prompts/_defs-gate54-tiering-r2-background.md` | `research/prompts/defs-gate54-tiering-r1-verifier-output.md` |
| `research/prompts/defs-gatebatch-m2-r2-opus-model/SHA256SUMS` | `research/prompts/defs-gatebatch-m2-r2-opus-model/outputs/fix-k1-hook.log`、`research/prompts/defs-gatebatch-m2-r2-opus-model/outputs/k1-hook.log` |
| `research/prompts/defs-gatebatch-m2-r3-opus-model/SHA256SUMS` | `research/prompts/defs-gatebatch-m2-r3-opus-model/fix-lib-heavy-tests.diff` |
| `research/prompts/defs-gatebatch-m2-r3-opus-output.md` | `research/prompts/defs-gatebatch-m2-r3-opus-model/fix-lib-heavy-tests.diff` |
| `research/prompts/defs-m2-closeout-r1-opus-model/SHA256SUMS` | `research/prompts/defs-m2-closeout-r1-opus-model/cases.json`、`research/prompts/defs-m2-closeout-r1-opus-model/probe-output.txt` |
| `research/prompts/defs-m2-closeout-r1-opus-output.md` | `research/prompts/defs-m2-closeout-r1-opus-model/cases.json` |
| `research/prompts/defs-m2-closeout-r1-verifier-output.md` | `research/prompts/defs-m2-closeout-r1-opus-output.md` |
| `research/prompts/defs-m2-closeout-r2-opus-model/SHA256SUMS` | `research/prompts/defs-m2-closeout-r2-opus-model/cases-hooks.json`、`research/prompts/defs-m2-closeout-r2-opus-model/rerun-output.txt` |
| `research/prompts/defs-m2-closeout-r2-opus-output.md` | `research/prompts/defs-m2-closeout-r2-opus-model/cases-hooks.json`、`research/prompts/defs-m2-closeout-r2-opus-model/rerun-output.txt` |
| `research/prompts/defs-m2-closeout-r2-verifier-output.md` | `research/prompts/defs-m2-closeout-r2-opus-output.md` |
| `research/prompts/defs-m2-closeout-r3-opus-model/SHA256SUMS` | `research/prompts/defs-m2-closeout-r3-opus-model/cases-k1.json` |
| `research/prompts/defs-m2-closeout-r3-opus-output.md` | `research/prompts/defs-m2-closeout-r3-opus-model/cases-k1.json` |
| `research/prompts/gate-fix-forks-r3-verifier-output.md` | `research/prompts/gate-fix-forks-r3-sonnet-output.md` |
| `research/prompts/m2-closeout-code-r1-opus-output.md` | `research/prompts/m2-closeout-code-r1-opus-model/batch.sh`、`research/prompts/m2-closeout-code-r1-opus-model/rerun.sh` |
| `research/prompts/m2-closeout-code-r1-snapshot/kb-sha256.txt` | `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md`、`research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/decisions/22-单元原子性怎么合成.md` |
| `research/prompts/m2-closeout-code-r2-opus-output.md` | `research/prompts/m2-closeout-code-r2-opus-model/batch_sweep.sh`、`research/prompts/m2-closeout-code-r2-opus-model/rerun.sh` |
| `research/prompts/m2-closeout-code-r2-snapshot/kb-sha256.txt` | `research/prompts/m2-closeout-code-r2-snapshot/kb-at-start/.claude/kb/decisions/08-核心索引结构.md`、`research/prompts/m2-closeout-code-r2-snapshot/kb-at-start/.claude/kb/decisions/22-单元原子性怎么合成.md` |
| `research/prompts/m2-code-review-6c/SHA256SUMS` | `research/prompts/m2-code-review-6c/clippy.log` |
| `research/prompts/m2-final-code-r2-opus-model/SHA256SUMS` | `research/prompts/m2-final-code-r2-opus-model/rerun.sh` |
| `research/prompts/m2-final-code-r2-opus-output.md` | `research/prompts/m2-final-code-r2-opus-model/rerun.sh` |
| `research/prompts/m2-final-code-r3-opus-model/SHA256SUMS` | `research/prompts/m2-final-code-r3-opus-model/rerun.sh` |
| `research/prompts/m2-final-code-r3-opus-output.md` | `research/prompts/m2-final-code-r3-opus-model/rerun.sh` |
| `research/prompts/m2-final-code-r4-opus-model/SHA256SUMS` | `research/prompts/m2-final-code-r4-opus-model/rerun.sh` |
| `research/prompts/m2-final-code-r4-opus-output.md` | `research/prompts/m2-final-code-r4-opus-model/rerun.sh` |
| `research/prompts/m2-gate69-tmp-evidence-report.md` | `research/prompts/defs-closeout-draft-2-tmp-evidence/report.md`、`research/prompts/defs-closeout-r1-fixes-tmp-evidence/probe/f14-cases.json`、`research/prompts/defs-closeout-r1-fixes-tmp-evidence/probe/f14-out.txt`、`research/prompts/defs-closeout-r1-fixes-tmp-evidence/report.md`、`research/prompts/impl-rbf-4a-tmp-evidence/report.md`、`research/prompts/impl-rbf-4c-tmp-evidence/report.md`、`research/prompts/m2-impl-rbf-4a-implementer-report.md`、`research/prompts/m2-impl-rbf-4c-implementer-report.md` |
| `research/prompts/m2-kb-writeback-batch2-scribe-report.md` | `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/checks-owed.md` |
| `research/prompts/m2-kb-writeback-batch5-scribe-report.md` | `research/prompts/m2-closeout-code-r1-snapshot/kb-at-start/.claude/kb/decisions/22-单元原子性怎么合成.md` |
| `research/prompts/m2-kb-writeback-batch7a-scribe-report.md` | `research/prompts/m2-closeout-code-r2-snapshot/kb-at-start/.claude/kb/decisions/22-单元原子性怎么合成.md` |
| `research/prompts/m2-safety-r1-opus-model/SHA256SUMS` | `research/prompts/m2-safety-r1-opus-model/rerun.sh` |
| `research/prompts/m2-safety-r1-opus-output.md` | `research/prompts/m2-safety-r1-opus-model/rerun.sh` |
| `research/prompts/m2-safety-r1-sonnet-model/SHA256SUMS` | `research/prompts/m2-safety-r1-sonnet-model/rerun.sh` |
| `research/prompts/m2-safety-r3-sonnet-model/SHA256SUMS` | `research/prompts/m2-safety-r3-sonnet-model/rerun-output.log` |
| `research/prompts/m2-safety-r3-verifier-output.md` | `research/prompts/m2-safety-r3-sonnet-output.md` |
| `research/prompts/m2-supp3-item4-code-r1-verifier-output.md` | `research/prompts/m2-supp3-item4-code-r1-opus-output.md` |
| `research/prompts/private-info-redaction-2026-09-28.md`（第一批「改了哪些」表的改后列） | `research/prompts/_defs-m2-closeout-r2-appendix.md`、`research/prompts/_defs-m2-closeout-r2-background.md`、`research/prompts/_governance-defs-r3-appendix.md`、`research/prompts/_governance-defs-r3-background.md`、`research/prompts/e162-preregistration.md`、`research/prompts/e162-s3-s4-designer-report.md`、`research/prompts/m2-readme-status-report.md` |
| `research/prompts/sync-local-legs-r1-opus-output.md` | `research/prompts/sync-local-legs-r1-opus-model/cases1.json`、`research/prompts/sync-local-legs-r1-opus-model/hookrun.py`、`research/prompts/sync-local-legs-r1-opus-model/sim_watch.py` |

- `research/prompts/e142-r19-prereg.md` 登记在 `.claude/gate.d/stage-inputs.tsv` 第 34 行（键 E142）的输入路径里，这一批改了它一处仓根，E142 的输入指纹因此变了。
- `research/prompts/e162-s4-apparatus-evidence/dry-run.stdout` 有 35 行、`research/prompts/` 里另有 1 行，原样抄自 `research/results/` 的产物（`e162-verdict-store-power-cut-2026-09-27-dry-run.out`、`e159-fsync-wait-group-commit-2026-09-24-smoke.out`）；这一批改了抄本里的路径，产物归另一批，两边在那一批按同一套规则改完之前逐字对不上。
- 抄进来的脚本与命令（`*-model/rerun.sh`、`batch.sh`、`cases*.json` 这类）里的仓根换成了 `<仓根>`，不能照原样再跑；`.diff` 文件头里的路径去掉了仓根前缀，打补丁的 `-p` 层数要按新路径算。
