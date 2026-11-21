#!/usr/bin/env python3
# admission: always 判的是这一次给的拆前、拆后两棵树此刻的源码、变异表与登记表，上一次的结论不替这一次作保
# run-condition: none 只读两棵树里的 .rs、.wgsl、Cargo.toml、变异表与门禁登记表，经 lib_function_units 切单元，除 python3 外没有环境要求；不编译，不起别的进程（自证的开关轮只再起它自己的 --selftest）
# gate-similar: lib_function_units.py 单元的切法、去注释的词法与指纹照用它那一份（import RustFile、WgslFile、discover_rust_crates），不另切一遍；它按入口收闭包、算一把键，这里只拿两棵树的单元多重集对比，并把可见性修饰从指纹里剥掉
# gate-similar: crates-mutation-rows.py 读 crates/mutations.tsv 的六段与 \n 转义写法同它（原文里 \n 表示换行）；它按表逐行改坏源码、跑测试，这里只把被拆文件那几行的文件列改到锚点现在所在的文件、不改别的段，也不跑变异
# gate-similar: patch-of-the-current-tree.py 它按此刻树筛补丁里的变异行、整行逐字还在的才留，不改文件列；这里在拆分补丁合入前把锚点落进子模块的行改到新文件
# gate-similar: rust-name-collisions.py 同样经 lib_function_units 的 Rust 单元判一棵树，判的是同一个编译目标里的撞名；这里比两棵树的单元多重集，不比名字撞不撞
"""crates 大文件拆分用的三样机械检查（拆分只挪不改：库模块 foo.rs 留作根、拆出的放 foo/<名>.rs，tests/x.rs 与 src/bin/x.rs 变成 x/main.rs 加同目录子模块；
条目正文逐字不变，只许加模块声明、use、放宽可见性与 rustfmt 的重新排版）。

用法：
    crates-split-pure-move.py pure-move --before <树> --after <树> [--crate <包目录>]… [--strict-use] [--list-visibility]
    crates-split-pure-move.py remap-mutations --after <树> [--before <树>] [--split <旧路径>]… [--table <变异表>] [--test-renames <改名表>|<包>:<目标>=<改名表>]… --out <路径>
    crates-split-pure-move.py registries [--root <树>] [--before <树> --after <树>] [--split <旧路径>]… [--test-renames <改名表>|<包>:<目标>=<改名表>]…
    crates-split-pure-move.py --selftest [--cell-workers <路数>]     # 自证，不起 cargo；CRATES_SPLIT_PURE_MOVE_BREAK=<开关> 时红的格必须与点名的逐格相同

pure-move 纯挪动检查：比两棵树里每个包的单元多重集（单元的切法、去注释与指纹照 lib_function_units.py，Rust 与 WGSL 都比；--crate 收窄到点名的包目录）。
  同一个单元 = （种类，名字，剥掉可见性之后的指纹）相同；可见性修饰是 pub、pub(crate)、pub(super)、pub(self)、pub(in …)；名字取单元在文件里的嵌套路径、
  去掉行内模块那一截前缀，名字里的行号折掉（use、条目级宏调用、extern 块挪了位置行号会变）。比的是多重集：同一个键拆前几个、拆后就要几个。
  不算差异的（单列）：拆后多出的 mod 声明与 use 条目（脚手架）；拆一个 impl 块成几个时多出来的、与拆前某个 impl 头逐字相同的 impl 头（脚手架）；
  拆后新加的行内 mod 头（`mod tests {` 这种，每个子模块各带一个时同名头的总数会变；不含文件开头内属性那个 <module attributes>，那个多出来照判红）（脚手架；
  它与新加的 mod 声明按所在的被拆文件族各数几个、逐族列出：族根是拆后文件自己在拆前就有的那份，或往上找到的目录同名的 <目录>.rs、<目录>/main.rs、mod.rs、lib.rs）；
  行内 mod 挪成同名的文件模块（外属性、mod 名同，拆后是 `mod 名;`）；拆前有、拆后没有的 use（删掉用不上的 use 或改写它，编不编得过归编译；--strict-use 按差异判）；
  只差 rustfmt 排版的单元（逐个列出、计数、不判红）：去掉可见性之后，词法单元序列只差两处——紧挨 )、]、}、> 之前的尾逗号（( 前面不是调用的单元素元组 (x,) 的逗号不算，
  去掉它是另一个类型或值），与闭包体、match 臂体外面只包一个表达式（花括号里不空、没有不在括号里面的分号）的那一对花括号（match 臂 `=> { e }` 与 `=> e,` 同；见 lib_crates_split_layout.py）。
  拆前拆后逐字节相同的文件里的单元不进配对：键相同的单元按多重集配、配不上的余数落在哪份文件是配对次序定的，让它们参与会把差异记到没碰过的文件上；它们照数进「完全相同」。
  差异（逐个列出：包、种类与限定名、拆前文件:行、拆后文件:行，退 1）：拆前有、拆后没有；指纹变了（同种类同名、剥掉可见性之后指纹不同）；拆后多出的别的单元；
  行内模块头的外属性在拆后丢了；包只在一边的树里；切不开的写法（拒算条目）拆前拆后对不上。
  另报四个数：只因可见性变了指纹的单元个数、只差 rustfmt 排版的单元个数（crates/singlefs-unit-graph/src/closure.rs 的 reuse_key 由闭包里各单元的（种类，指纹）多重集等组成，
  含这两类单元的判定会作废；两类并列报，又改可见性又改排版的算只差 rustfmt 排版）、脚手架条目个数、完全相同的单元个数；成功句带比了多少个包、多少个单元。
  一份文件的单元切不开（括号不配平）算一处拒算条目，两边对不上就红。

remap-mutations 变异行重归：读变异表（缺省拆前的树里的 crates/mutations.tsv；每行六段，制表符分隔：变异名、文件、原文、替换文、cargo test 的参数、必须红的测试名；
  原文与替换文里的 \n 表示换行），对文件列是被拆文件的行，在拆后那一族文件里数原文：恰好一份文件里恰好命中一次的，把文件列改成那份文件；
  锚点仍在根文件里命中一次的不改（单数）；0 次命中、几份文件里都有、一份文件里命中不止一次的逐行列出（变异名、原文前 60 字、原因）不改。
  被拆文件：--split 给（可几次），或不给、由 --before 与 --after 两棵树认：拆前的 .rs 在拆后同名目录下多出了拆前没有的 .rs，或根挪成 <名>/main.rs；
  crate 根与 mod.rs（lib.rs、main.rs、mod.rs）拆出的是同目录的 <名>.rs，要被它自己用 mod <名>; 声明了才认。
  拆后那一族文件 = 根文件（还在的话）加它目录下新出现的 .rs（给了 --before 才区分新旧，没给就收目录下全部 .rs）。
  整份改后的表写到 --out（不就地写；与变异表同一份拒），只改被拆文件那几行的第 2 段（给了 --test-renames 的，再加第 5、6 段里的测试名，见下），别的段、别的行、行序、结尾换行一个字节不动；
  报改了几行、没改几行（锚点仍在原文件的、列出的各几行）。
  --test-renames <改名表> 或 <包>:<目标>=<改名表>（可给几次取并集）：拆分把 mod tests 里的用例挪进子模块，测试全名（库里的模块路径，例 transaction::tests::x）跟着变；改名表每行「旧全名<TAB>新全名」，
  空行与 # 起头的行跳过，同一个写法（同一个包与目标，或都不带目标）里同一个旧全名对到两个不同的新全名退 2。同一个全名可以在几个测试目标里各有一条（集成测试各目标是各自的二进制，库与二进制里的单测也是），
  改名只属其中一个目标时写成 <包>:<目标>=<改名表>，目标三种：lib（库的单测）、test:<名>（集成测试，<名> 是 cargo test --test <名> 里那个名字）、bin:<名>（二进制里的单测）；
  等号前面有 / 的当不带目标的路径，包或目标写不对退 2。变异行的目标从第 5 段 -- 之前认：-p <包> 定包，--lib、--test <名>、--bin <名> 定目标，各写且只写一个，认不出的列出。
  第 5 段 -- 之后不是选项的词是筛选词（cargo 不带 --exact 按子串筛，带 --exact 按整名相等筛）：
    哪些改名管这一行：按目标写的表，包与目标都与这一行的目标相同的才管；这一行的目标认不出而点名了表里的旧全名（第 6 段，或第 5 段带 --exact 的筛选词）的，列出不改。
    不带目标的表，旧全名在变异表所有点名它的行里只落在一个目标（认不出目标的行各算一个目标）的，照名字换；落在几个目标的，点名它的每一行列出不改。
    换名：第 6 段（必须红的测试名）与带 --exact 的筛选词，整名等于某个管这一行的旧全名的换成新全名，只换整名相等的；不带 --exact 的筛选词一概不改。
    列出（只报不改，变异名在行首，退 1）：不带 --exact 的筛选词，管这一行的（含落在几个目标而没换的不带目标的）旧全名拆前筛得到（筛选词是它的子串）、拆后对应的新全名筛不到的（列筛选词与筛不到的新全名）；
    第 6 段换名之后，第 5 段的筛选在拆后筛不到它的；第 5 段 -- 之后有 --exact 以外的选项的（第 5 段不判；这一行没有管它的旧全名时不列）。拆前拆后同样筛得到的不动、不列出。
  给了 --test-renames 而没认出被拆文件（没给 --before 与 --split）的，只改测试名；成功句与失败句都报「照 N 条测试改名，第 5、6 段改了几行，列出几行」。

registries 登记表扫描：列出 .claude/gate.d/ 顶层 *.tsv（样本档在 fixtures/ 下、不在顶层）里点名被拆文件旧路径的行（文件:行与整行），只报不改；
  点名 = 整条路径出现、前后不紧挨着路径字符（x/ledger.rs.bak、xledger.rs 都不算）。--root 缺省取 --before，再缺省取 --after。
  给了 --test-renames 的，另列出点名改名表旧全名的行（文件:行、整行、旧全名改成什么），点名 = 旧全名整串出现、前面不紧挨标识符字符或 ::、后面不紧挨标识符字符；可以不给被拆文件。
  不带目标的表的旧全名都列；按目标写的表的旧全名，行的目标（取整行里 -- 之前那一段，同变异行的认法）与表一致的列，目标认不出的也列（注明目标认不出），认得出而不一致的不列。

退出码：0 纯挪动 / 全部归位、测试筛选都对 / 登记表里没有点名的行；1 有差异、有变异行没改、有测试筛选列出、登记表里有点名的行；2 用法错（含改名表写错、<包>:<目标> 写错）；3 算不出（树或文件读不了、Cargo.toml 写坏）；
77 本次未跑（没有可比的包、没认出被拆的文件又没给改名表、没有登记表目录）。

--cell-workers 是各格并行的路数（正整数），缺省取每台默认重活线程数；开关轮外层并行度永远取每台默认重活线程数，起每个开关的子进程时交 --cell-workers 1，总并发不超过外层并行度。

弄坏开关（只给自证用：只在 --selftest 里认，平常跑设了退 2；--selftest 带开关时各格照跑、不起开关轮，红的格与点名的逐格相同（多红、少红都不对）；单跑一条：
env CRATES_SPLIT_PURE_MOVE_BREAK=<开关> python3 scripts/gate/crates-split-pure-move.py --selftest；开关名与点名的格登记在 lib_crates_split_common.py 的 BREAK_CELLS；
点名多于一格的，多出的是开关弄坏的那一处连带波及的格）：
  CRATES_SPLIT_PURE_MOVE_BREAK=visibility-not-stripped  指纹不剥可见性修饰
      「✗ 自证 visibility-relaxed-green：」
      「✗ 自证 scaffold-green：」
      「✗ 自证 inline-module-to-file-green：」
      「✗ 自证 visibility-counts：」
      「✗ 自证 removed-use-green：」
      「✗ 自证 exit-codes：」
      「✗ 自证 rustfmt-only-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=changed-ignored  指纹变了的单元不报
      「✗ 自证 changed-body-red：」
      「✗ 自证 crate-filter：」
      「✗ 自证 wgsl-changed-red：」
      「✗ 自证 exit-codes：」
      「✗ 自证 rustfmt-real-changes-red：」
      「✗ 自证 unchanged-files-silent：」
  CRATES_SPLIT_PURE_MOVE_BREAK=missing-ignored  拆前有、拆后没有的单元不报
      「✗ 自证 missing-unit-red：」
      「✗ 自证 duplicate-dropped-red：」
      「✗ 自证 duplicate-twin-unchanged-red：」
      「✗ 自证 removed-use-strict-red：」
      「✗ 自证 inline-module-attribute-dropped-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=extra-ignored  拆后多出的别的单元不报
      「✗ 自证 extra-unit-red：」
      「✗ 自证 module-attributes-head-added-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=scaffold-counted-as-extra  新加的 mod 与 use 也当多出的单元报
      「✗ 自证 visibility-relaxed-green：」
      「✗ 自证 scaffold-green：」
      「✗ 自证 inline-module-to-file-green：」
      「✗ 自证 visibility-counts：」
      「✗ 自证 removed-use-green：」
      「✗ 自证 split-impl-green：」
      「✗ 自证 exit-codes：」
      「✗ 自证 unchanged-files-silent：」
      「✗ 自证 new-mod-heads-scaffold：」
  CRATES_SPLIT_PURE_MOVE_BREAK=module-scope-kept-in-names  名字里保留行内模块前缀
      「✗ 自证 visibility-relaxed-green：」
      「✗ 自证 scaffold-green：」
      「✗ 自证 inline-module-to-file-green：」
      「✗ 自证 visibility-counts：」
      「✗ 自证 changed-body-red：」
      「✗ 自证 removed-use-green：」
      「✗ 自证 exit-codes：」
  CRATES_SPLIT_PURE_MOVE_BREAK=duplicates-collapsed  同一个键拆前几个都当一个比（不按多重集）
      「✗ 自证 duplicate-dropped-red：」
      「✗ 自证 split-impl-green：」
      「✗ 自证 new-mod-heads-scaffold：」
  CRATES_SPLIT_PURE_MOVE_BREAK=strict-use-not-red  --strict-use 不生效
      「✗ 自证 removed-use-strict-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=visibility-counted-as-identical  只因可见性变了的单元算进完全相同
      「✗ 自证 visibility-counts：」
      「✗ 自证 rustfmt-only-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=split-impl-header-extra  拆 impl 块多出的 impl 头也当多出的单元报
      「✗ 自证 split-impl-green：」
  CRATES_SPLIT_PURE_MOVE_BREAK=inline-module-attributes-unchecked  行内模块挪成文件模块不核外属性
      「✗ 自证 inline-module-attribute-dropped-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=line-numbers-in-names  名字里的行号不折掉
      「✗ 自证 visibility-relaxed-green：」
      「✗ 自证 scaffold-green：」
      「✗ 自证 inline-module-to-file-green：」
      「✗ 自证 visibility-counts：」
      「✗ 自证 removed-use-green：」
      「✗ 自证 macro-call-moved-green：」
      「✗ 自证 exit-codes：」
  CRATES_SPLIT_PURE_MOVE_BREAK=crate-filter-ignored  --crate 不收窄
      「✗ 自证 crate-filter：」
  CRATES_SPLIT_PURE_MOVE_BREAK=lone-crate-ignored  包只在一棵树里不报
      「✗ 自证 crate-missing-after-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=wgsl-skipped  不比 .wgsl
      「✗ 自证 wgsl-changed-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=red-exits-zero  有差异、登记表有点名的行也退 0
      「✗ 自证 changed-body-red：」
      「✗ 自证 missing-unit-red：」
      「✗ 自证 extra-unit-red：」
      「✗ 自证 duplicate-dropped-red：」
      「✗ 自证 duplicate-twin-unchanged-red：」
      「✗ 自证 removed-use-strict-red：」
      「✗ 自证 inline-module-attribute-dropped-red：」
      「✗ 自证 crate-filter：」
      「✗ 自证 crate-missing-after-red：」
      「✗ 自证 wgsl-changed-red：」
      「✗ 自证 unknown-item-changed-red：」
      「✗ 自证 exit-codes：」
      「✗ 自证 rustfmt-real-changes-red：」
      「✗ 自证 unchanged-files-silent：」
      「✗ 自证 module-attributes-head-added-red：」
      「✗ 自证 registry-path-boundary：」
      「✗ 自证 registry-test-names：」
      「✗ 自证 target-renames-registry：」
  CRATES_SPLIT_PURE_MOVE_BREAK=refusals-unchecked  拒算条目不比
      「✗ 自证 unknown-item-changed-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=ambiguous-first-hit-taken  锚点在几份文件里都有时取头一份
      「✗ 自证 anchor-in-two-files-listed：」
      「✗ 自证 anchor-staying-in-root-unchanged：」
      「✗ 自证 remap-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=repeated-hit-in-one-file-accepted  锚点在一份文件里命中几次也算
      「✗ 自证 anchor-twice-in-one-file-listed：」
      「✗ 自证 anchor-staying-in-root-unchanged：」
      「✗ 自证 remap-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=zero-hit-dropped  0 次命中的行不列出
      「✗ 自证 anchor-nowhere-listed：」
      「✗ 自证 remap-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=newline-escape-undecoded  锚点里的 \n 不换成换行
      「✗ 自证 multi-line-anchor-changed：」
      「✗ 自证 only-the-file-column-changes：」
      「✗ 自证 remap-counts：」
      「✗ 自证 remap-exit-codes：」
  CRATES_SPLIT_PURE_MOVE_BREAK=unrelated-files-searched  在整棵树里找锚点，不只在拆后那一族
      「✗ 自证 anchor-nowhere-listed：」
      「✗ 自证 anchor-only-in-split-family：」
      「✗ 自证 only-the-file-column-changes：」
      「✗ 自证 remap-counts：」
      「✗ 自证 crate-root-siblings-searched：」
  CRATES_SPLIT_PURE_MOVE_BREAK=family-root-excluded  拆后那一族不含根文件
      「✗ 自证 anchor-in-two-files-listed：」
      「✗ 自证 anchor-twice-in-one-file-listed：」
      「✗ 自证 anchor-staying-in-root-unchanged：」
      「✗ 自证 only-the-file-column-changes：」
      「✗ 自证 remap-counts：」
      「✗ 自证 remap-exit-codes：」
  CRATES_SPLIT_PURE_MOVE_BREAK=main-root-undetected  认被拆文件时不认 <名>/main.rs
      「✗ 自证 root-moved-to-main-detected：」
  CRATES_SPLIT_PURE_MOVE_BREAK=new-submodules-undetected  认被拆文件时只认 main.rs、不认新出的子模块
      「✗ 自证 split-detected-from-trees：」
      「✗ 自证 crate-root-siblings-searched：」
  CRATES_SPLIT_PURE_MOVE_BREAK=crate-root-siblings-unsearched  crate 根拆出的同目录模块不进族
      「✗ 自证 crate-root-siblings-searched：」
  CRATES_SPLIT_PURE_MOVE_BREAK=existing-files-searched  族里把拆前就有的文件也收进来
      「✗ 自证 crate-root-siblings-searched：」
  CRATES_SPLIT_PURE_MOVE_BREAK=fields-reserialized  改过的行把原文与替换文的 \n 展成换行再写
      「✗ 自证 only-the-file-column-changes：」
      「✗ 自证 row-order-kept：」
  CRATES_SPLIT_PURE_MOVE_BREAK=row-order-sorted  改后的表按行排序
      「✗ 自证 only-the-file-column-changes：」
      「✗ 自证 row-order-kept：」
      「✗ 自证 test-columns-only-change：」
  CRATES_SPLIT_PURE_MOVE_BREAK=unresolved-exit-zero  有行没改也退 0
      「✗ 自证 remap-exit-codes：」
      「✗ 自证 split-detected-from-trees：」
  CRATES_SPLIT_PURE_MOVE_BREAK=unchanged-counted-as-changed  锚点仍在原文件的行算进「改了」
      「✗ 自证 anchor-staying-in-root-unchanged：」
      「✗ 自证 remap-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=registry-subdirectories-scanned  登记表扫描连子目录（样本档）也扫
      「✗ 自证 registry-fixtures-excluded：」
  CRATES_SPLIT_PURE_MOVE_BREAK=registry-path-boundary-ignored  路径前后紧挨着路径字符也算点名
      「✗ 自证 registry-path-boundary：」
  CRATES_SPLIT_PURE_MOVE_BREAK=registry-non-tsv-scanned  登记表扫描连不是 .tsv 的文件也扫
      「✗ 自证 registry-only-tsv：」
  CRATES_SPLIT_PURE_MOVE_BREAK=sixth-column-not-renamed  第 6 段的测试名不照改名表换（也不核换名后的第 5 段）
      「✗ 自证 test-names-whole-replaced：」
      「✗ 自证 renamed-test-unselected-listed：」
      「✗ 自证 test-columns-only-change：」
      「✗ 自证 test-remap-counts：」
      「✗ 自证 renames-without-split：」
      「✗ 自证 renames-and-file-column-together：」
      「✗ 自证 target-renames-by-target：」
      「✗ 自证 untargeted-renames-ambiguous-listed：」
      「✗ 自证 unrecognized-target-listed：」
  CRATES_SPLIT_PURE_MOVE_BREAK=exact-filter-not-renamed  带 --exact 的筛选词不照改名表换
      「✗ 自证 test-names-whole-replaced：」
      「✗ 自证 test-remap-counts：」
      「✗ 自证 test-remap-exit-codes：」
      「✗ 自证 renames-without-split：」
      「✗ 自证 renames-and-file-column-together：」
      「✗ 自证 target-renames-by-target：」
      「✗ 自证 untargeted-renames-ambiguous-listed：」
      「✗ 自证 unrecognized-target-listed：」
  CRATES_SPLIT_PURE_MOVE_BREAK=renames-by-substring  测试名里含旧全名就换，不只换整名相等的
      「✗ 自证 test-names-whole-replaced：」
  CRATES_SPLIT_PURE_MOVE_BREAK=substring-filter-renamed  不带 --exact 的筛选词整名等于旧全名时也换
      「✗ 自证 substring-filter-lost-listed：」
  CRATES_SPLIT_PURE_MOVE_BREAK=lost-filter-not-listed  拆前筛得到、拆后筛不到的筛选词不列出
      「✗ 自证 substring-filter-lost-listed：」
      「✗ 自证 test-remap-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=kept-filter-listed  拆后仍筛得到的筛选词也列出
      「✗ 自证 substring-filter-kept-untouched：」
      「✗ 自证 test-remap-counts：」
      「✗ 自证 test-remap-exit-codes：」
  CRATES_SPLIT_PURE_MOVE_BREAK=renamed-test-selection-unchecked  第 6 段换名之后不核第 5 段还筛不筛得到它
      「✗ 自证 renamed-test-unselected-listed：」
      「✗ 自证 test-remap-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=unknown-option-ignored  第 5 段 -- 之后 --exact 以外的选项不列出、照常判筛选词
      「✗ 自证 unknown-option-listed：」
  CRATES_SPLIT_PURE_MOVE_BREAK=columns-reserialized  改过测试名的第 5 段把词间空白折成单个空格再写
      「✗ 自证 test-names-whole-replaced：」
  CRATES_SPLIT_PURE_MOVE_BREAK=listed-tests-exit-zero  只有测试筛选列出时也退 0
      「✗ 自证 test-remap-exit-codes：」
      「✗ 自证 untargeted-renames-ambiguous-listed：」
      「✗ 自证 unrecognized-target-listed：」
  CRATES_SPLIT_PURE_MOVE_BREAK=renames-alone-refused  给了改名表而没认出被拆文件也退 77
      「✗ 自证 renames-without-split：」
  CRATES_SPLIT_PURE_MOVE_BREAK=renames-union-last-only  --test-renames 给几份只读最后一份
      「✗ 自证 renames-table-read：」
      「✗ 自证 target-renames-table-read：」
  CRATES_SPLIT_PURE_MOVE_BREAK=conflicting-renames-accepted  同一个包与目标里同一个旧全名对到两个新全名不拒
      「✗ 自证 renames-table-read：」
      「✗ 自证 target-renames-table-read：」
  CRATES_SPLIT_PURE_MOVE_BREAK=malformed-rename-line-skipped  改名表里不是两段、有空段的行跳过不拒
      「✗ 自证 renames-table-read：」
  CRATES_SPLIT_PURE_MOVE_BREAK=target-ignored  按目标写的表只比包、不比目标
      「✗ 自证 target-renames-by-target：」
      「✗ 自证 target-renames-registry：」
  CRATES_SPLIT_PURE_MOVE_BREAK=package-ignored  按目标写的表只比目标、不比包
      「✗ 自证 target-renames-by-target：」
      「✗ 自证 target-renames-registry：」
  CRATES_SPLIT_PURE_MOVE_BREAK=test-bin-flags-unparsed  第 5 段（与登记表的行）里的 --test、--bin 不认，目标只认得出 --lib
      「✗ 自证 target-renames-by-target：」
      「✗ 自证 unrecognized-target-listed：」
      「✗ 自证 target-renames-registry：」
  CRATES_SPLIT_PURE_MOVE_BREAK=several-targets-first-taken  第 5 段写了几个包或几个目标时取头一个，不当认不出
      「✗ 自证 unrecognized-target-listed：」
  CRATES_SPLIT_PURE_MOVE_BREAK=unrecognized-target-renamed  目标认不出的行也照按目标写的表换名（登记表里也不注明目标认不出）
      「✗ 自证 unrecognized-target-listed：」
      「✗ 自证 target-renames-registry：」
  CRATES_SPLIT_PURE_MOVE_BREAK=unrecognized-target-not-listed  目标认不出的行不列出（登记表里也不列）
      「✗ 自证 unrecognized-target-listed：」
      「✗ 自证 target-renames-registry：」
  CRATES_SPLIT_PURE_MOVE_BREAK=loose-ambiguity-ignored  不带目标的表的旧全名落在几个目标里时也照名字换
      「✗ 自证 untargeted-renames-ambiguous-listed：」
  CRATES_SPLIT_PURE_MOVE_BREAK=malformed-target-accepted  <包>:<目标>=<表> 的包或目标写不对时当路径读，不拒
      「✗ 自证 target-renames-table-read：」
  CRATES_SPLIT_PURE_MOVE_BREAK=renames-conflict-across-targets  不同目标里同一个旧全名对到不同的新全名也拒
      「✗ 自证 target-renames-table-read：」
  CRATES_SPLIT_PURE_MOVE_BREAK=registry-test-names-not-scanned  登记表扫描不找改名表的旧全名
      「✗ 自证 registry-test-names：」
      「✗ 自证 target-renames-registry：」
  CRATES_SPLIT_PURE_MOVE_BREAK=registry-test-name-boundary-ignored  旧全名前后紧挨着标识符字符或 :: 也算点名
      「✗ 自证 registry-test-names：」
  CRATES_SPLIT_PURE_MOVE_BREAK=trailing-comma-kept  排版抹平时不去尾逗号
      「✗ 自证 rustfmt-normal-form：」
      「✗ 自证 rustfmt-only-green：」
      「✗ 自证 rustfmt-only-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=single-expression-braces-kept  排版抹平时不去闭包体与 match 臂体外面的花括号
      「✗ 自证 rustfmt-normal-form：」
      「✗ 自证 rustfmt-only-green：」
  CRATES_SPLIT_PURE_MOVE_BREAK=arm-comma-not-added  去掉 match 臂体的花括号之后臂尾不补逗号
      「✗ 自证 rustfmt-normal-form：」
      「✗ 自证 rustfmt-only-green：」
  CRATES_SPLIT_PURE_MOVE_BREAK=single-element-tuple-comma-stripped  单元素元组 (x,) 的逗号也当尾逗号去掉
      「✗ 自证 rustfmt-normal-form：」
      「✗ 自证 rustfmt-real-changes-red：」
  CRATES_SPLIT_PURE_MOVE_BREAK=statement-braces-dropped  花括号里有分号（语句块）的闭包体、match 臂体也去花括号
      「✗ 自证 rustfmt-normal-form：」
  CRATES_SPLIT_PURE_MOVE_BREAK=layout-not-paired  只差 rustfmt 排版的单元不配对，落回「指纹变了」判红
      「✗ 自证 rustfmt-only-green：」
      「✗ 自证 rustfmt-only-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=layout-counted-as-identical  只差 rustfmt 排版的单元算进完全相同、不单列
      「✗ 自证 rustfmt-only-green：」
      「✗ 自证 rustfmt-only-counts：」
  CRATES_SPLIT_PURE_MOVE_BREAK=unchanged-files-compared  拆前拆后逐字节相同的文件里的单元也进配对
      「✗ 自证 unchanged-files-silent：」
      「✗ 自证 new-mod-heads-scaffold：」
  CRATES_SPLIT_PURE_MOVE_BREAK=mod-head-extra-red  拆后新加的行内 mod 头当拆后多出的单元报
      「✗ 自证 unchanged-files-silent：」
      「✗ 自证 new-mod-heads-scaffold：」
  CRATES_SPLIT_PURE_MOVE_BREAK=mod-head-family-by-own-file  新加的 mod 声明与行内 mod 头记在它们自己所在的文件上，不归到被拆文件的族
      「✗ 自证 new-mod-heads-scaffold：」
  CRATES_SPLIT_PURE_MOVE_BREAK=module-attributes-head-scaffolded  拆后多出的文件开头内属性（<module attributes>）也当脚手架
      「✗ 自证 module-attributes-head-added-red：」
"""
import os
import sys

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, "scripts", "hosts"))
sys.path.insert(0, SCRIPT_DIRECTORY)
from preflight import preflight  # noqa: E402
import heavy_threads  # noqa: E402
import lib_crates_split_cli as cli  # noqa: E402
import lib_crates_split_common as common  # noqa: E402
import lib_crates_split_selftest as selftest  # noqa: E402

# 弄坏开关的环境变量名；环境变量只在这份入口里读一次（见 __main__），各模块要名字、要开关集合都从入口拿
BREAK_VARIABLE = "CRATES_SPLIT_PURE_MOVE_BREAK"


def worker_count():
    """每台默认重活线程数（读文件、跑自证各格的并行路数）；取不了交 None。"""
    try:
        return int(heavy_threads.default_heavy_threads())
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 取不了每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}")
        return None


def main(arguments, switches):
    """switches 是入口读到的 BREAK_VARIABLE 原文（逗号分隔的开关名，没设是空串）。"""
    workers = worker_count()
    if workers is None:
        return 2
    if arguments[:1] == ["--selftest"]:
        try:
            cell_workers = selftest.parse_selftest_arguments(arguments[1:], BREAK_VARIABLE)
        except common.UsageProblem as problem:
            print(f"  ✗ {problem}")
            print(f"     → 怎么办：{problem.howto}")
            return 2
        requested = [name for name in switches.split(",") if name]
        unknown = [name for name in requested if name not in common.BREAKS]
        if unknown:
            print(f"  ✗ {BREAK_VARIABLE} 里有不认得的开关：{'、'.join(unknown)}")
            print(f"     → 怎么办：认得的开关是 {'、'.join(common.BREAKS)}")
            return 2
        common.broken.update(requested)
        status, cell_count, switch_count = selftest.run_selftest(workers, BREAK_VARIABLE, cell_workers)
        if status == 0:
            print(f"  ✓ --selftest：{cell_count} 格都对（并行 {cell_workers or workers} 路；纯挪动绿红、可见性与脚手架计数、rustfmt 排版与没碰过的文件、变异行重归各分支、测试改名与筛选（含按测试目标分表）、登记表扫描、用法错与退出码、开关轮判法与自证参数）；"
                  f"开关轮 {switch_count} 个开关逐个打开（外层并行 {workers} 路、每路各格只 1 路），各自红的格与点名的逐格相同")
        return status
    status, lines = cli.run_command(arguments, workers)
    for line in lines:
        print(line)  # gate-lint:detail
    return status


def settle_break_switches(arguments, switches):
    """弄坏开关只在 --selftest 里认：平常跑设了就拒，退 2、什么都不做。"""
    if not switches or arguments[:1] == ["--selftest"]:
        return None
    print(f"  ✗ {BREAK_VARIABLE} 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，跑 python3 scripts/gate/crates-split-pure-move.py --selftest（开关轮逐个打开）", file=sys.stderr)
    return 2


if __name__ == "__main__":
    break_switches = os.environ.get(BREAK_VARIABLE, "")
    refused = settle_break_switches(sys.argv[1:], break_switches)
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:], break_switches))
