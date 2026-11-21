#!/usr/bin/env python3
# admission: always 被 e165-replay-mutation-kill-bounds.sh 载入、每一趟照那一趟的根、变异表与判定库现算每个变体，上一次的结论不替这一次作保
# run-condition: none 被 E165 第二段驱动载入的模块，自己没有入口；建副本、编译与起装置的命令在整轮里起，环境要求见驱动文件头
"""E165（恢复重放有界穷举与变异杀伤界）第二段驱动 scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh 的各段：登记表与钉值、载入变异表与选行、建副本与编译、
按项判定（ItemVerdicts）、r2、r3、r4 三套整轮（main_run、main_run_r3、main_run_r4）、--select-rows 与 --measure。
读参数与收尾（main 与退出码）在驱动脚本里，自证在 e165_replay_mutation_kill_bounds_selftest.py。弄坏开关的名单（BREAK_MODES 各表）与比较都在这里，
BREAK 是判法模块 e165_replay_mutation_kill_bounds_variant_judgement.py 的那一份，驱动脚本载入这份模块之后照环境 E165_KILL_BOUNDS_BREAK 填（名字、弄坏哪一处、点名的格在驱动文件头「弄坏开关」）。
文件头的用法、登记与口径都在驱动脚本的文件头。
"""
import atexit
import concurrent.futures
import hashlib
import importlib.util
import json
import os
import re
import shutil
import signal
import subprocess
import sys
import threading
import time

sys.dont_write_bytecode = True
SELF_PATH = os.environ["E165_KILL_BOUNDS_SELF_PATH"]
SCRIPT_DIRECTORY = os.path.dirname(SELF_PATH)
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
from heavy_threads import default_heavy_threads  # noqa: E402
import lib_repository_copy as repository_copy  # noqa: E402   拷副本的范围与副本里起命令的环境（scripts/hosts/lib_repository_copy.py）
sys.path.insert(0, SCRIPT_DIRECTORY)
# 判一个变体的那几个函数在判法模块里（按项判定的 judge: 指它们，文件头「按项判定」）；BREAK 是两边共用的那一份，下面照环境填
from e165_replay_mutation_kill_bounds_variant_judgement import (  # noqa: E402
    BIN_NAME, BREAK, DEVICE_PROFILE_ARGUMENTS, J6_ROWS, J6QN_ROWS, R3_ALL_DOMAINS, R3_BOUNDS_ARGUMENTS, R3_DEVICE_PROFILE_ARGUMENTS,
    R4_ALL_DOMAINS, R4_DEVICE_PROFILE_ARGUMENTS,
    baseline_item_line, baseline_item_line_r3, baseline_item_line_r4, device_bounds_arguments, judge_variant, judge_variant_r3, judge_variant_r4,
    mark, parse_row, result_rows, retained_comparison_text, rerun_command, rerun_command_r3, rerun_command_r4, run_captured, run_segment_two,
    violation_lines_by_place)
DEFAULT_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
RECOVERY_PATH = "crates/singlefs-core/src/recovery.rs"
LIB_PATH = "crates/singlefs-replay-prefix/src/lib.rs"
SOURCE_FILES = (RECOVERY_PATH, LIB_PATH)     # 变异改的文件取变异行第二列，只认这两份（r1 9.1）
TABLE_PATH = "crates/mutations.tsv"
CUSTOM_TABLE_PATH = "scripts/mutation/tables/e165_replay_bounded_enumeration_custom.tsv"
TAGS_TABLE_PATH = "scripts/mutation/tables/e165_replay_bounded_enumeration_tags.tsv"   # r1 9.3：每个变体在两版源码上的判定标签集合与依据
FIXED_COPY_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain", "rust-toolchain.toml", "crates")   # 弄坏开关 fixedcopylist 下照拷的那几样
WORK_DIRECTORY_NAME = "kill-bounds"
# crates/mutations.tsv 的登记值在第一段收尾之前填不出（第一段要往表里加装置的行）：占位串让开工核对一律停在 RH1，填了真值才跑得起来。
TABLE_SHA256_PENDING = "pending-segment1-frozen-root"
# 登记 r2（research/prompts/e165-r2-prereg.md）9.4 第 1 条：两份源文件的 sha256 取第三节记的值（第一段不许动这两份，SH1，冻结的根上应当相同）、
# crates/mutations.tsv 的 sha256 取第一段收尾冻结的根上的值（没填之前是占位串）、9.2 的函数行区间取在工作区上现算的那一张；
# 冻结的根上重算与这里不同，就以冻结的根为准改这里并写进登记第十二节修订（行号不是判据，名字与函数才是）。
# 9.1 的 23 个变异名（登记时的表行号与 r1 登记时的标签集合）同 r1；z_tags 是 r1 9.4 给 Z1–Z4 登记的标签集合（r2 不判标签，Z5 没有登记集合）；
# custom_keys 是 r2 9.3 的 Z1–Z5。变异名取登记时 HEAD:crates/mutations.tsv 的第一列逐字；按名字选行，表行号只在产物里标出登记时在第几行。
REGISTERED = {
    "files_sha256": {
        RECOVERY_PATH: "3d79b99ece9bb7a308c8c503f31f445539d0f8020a12f1c6d7ab05f21f5b6acd",
        LIB_PATH: "e53aeb448cf2b6b63fae4bbc1c225656d5aed9094a2db94992faae5284a6b397",
    },
    "table_sha256": "9d5fd9fc20a555a5a98043afcd45293b1c3bc69a523e1c8d35531b69b0d278f0",
    "functions": [
        (LIB_PATH, "record_ends_its_publish", 1055, 1063),
        (LIB_PATH, "readable_record_of_the_same_publish_follows", 1067, 1182),
        (LIB_PATH, "counter_of_the_last_record_the_root_covers", 1186, 1254),
        (LIB_PATH, "is_above_the_chosen_root", 1257, 1266),
        (LIB_PATH, "maximum_transaction_of", 1269, 1315),
        (LIB_PATH, "judge_the_candidate", 1326, 1491),
        (LIB_PATH, "replay_core", 1497, 1876),
        (RECOVERY_PATH, "judge_back_chain_against_the_previous_record_on_disk", 2965, 2997),
        (RECOVERY_PATH, "replay_journal", 3028, 3178),
    ],
    "names": [
        (23, "步 3：前缀跨实例边界（把别的实例的记录也接上）", ["PD1"]),
        (24, "步 3：链首锚点错一位（接在所选根自己那条记录之后第二条）", ["PD7"]),
        (36, "步 3 三方第一轮：链首没锚点时不看 txg、无条件接上水位之上最小的一条", ["PD6"]),
        (226, "只供测试的开关：关掉点名验证时 verification_passed 照加（两臂在健康镜像上报同一个数，运行时看不出走了哪一条）", ["PD18"]),
        (354, "增补 2 第 61 行（C519 已知丢失）：施加前的点名单元验证改成任一条位置条目验过即过（改法 Z2 被悄悄落地时已知丢失那条先红）", ["PD17"]),
        (355, "增补 2 第 62 行（C533 复现）：前缀跨实例边界，写行那条孤记录进了水位之上", ["PD1"]),
        (406, "并行线一：C376：每条记录都当成它那次发布的末条（发布边界不认，前缀停在发布中间也施加）", ["PD19"]),
        (414, "并行线一：崩溃点重放：发布边界不认（每条记录都当末条）", ["PD19"]),
        (448, "P6 后一半：锚点读不出时不看本次发布内序号，链首接在下一次发布的第二条上（三方第一轮 K4-b）", []),
        (512, "实二十：锚点取读法甲——所选根那次发布读得出的记录里 jsn 最大那条，不看末条标志（替换第 438 行那条锚点已腐化的变异）", ["PD3", "PD4"]),
        (513, "实二十：所选根那次发布带末条标志的多于一条时不停下（条款没写那一格照常接链）", ["PD4"]),
        (514, "实二十：一次发布之内跳号不断链", ["PD13"]),
        (515, "实二十：一个事务的提交标记没出现、下一条换了事务号也照接", ["PD14"]),
        (516, "实二十：末条不带提交标记也整次施加", ["PD15"]),
        (517, "实二十：崩溃点重放 L8：恢复在不带提交标记的记录上就停（一事务一条时的旧写法，跨两条记录的事务永远施加不上）", ["PD15"]),
        (518, "实二十：崩溃点重放 L8：恢复把不带末条标志的记录也当末条（半次发布施加上去）", ["PD20"]),
        (613, "实二二三 b（C539，D23 已定项 4 读者规则）：锚点读得出时下一次发布的首条序号不是 1 照接", ["PD11"]),
        (614, "实二二三 b（C540，D23 已定项 4 读者规则）：带末条标志的那一条之后同一 (实例, txg) 还有记录照字面按标志认边界", ["PD16"]),
        (920, "实审 A2a 第 25 条：链值不等的记录照样进重放前缀", ["PD8"]),
        (921, "实审 A2a 第 25 条：前一条不在盘上也断前缀", ["PD9"]),
        (922, "实审 A2a 第 25 条：链值比较写反（对得上的记录被挡）", ["PD8"]),
        (1003, "实审 A2b 第 36 条：所选根的 txg 改回裸加一", ["PD5"]),
        (1004, "实审 A2b 第 36 条：所选根的 txg 加二才算溢出（边界）", ["PD5", "PD6"]),
    ],
    "z_tags": {"Z1": ["PD2"], "Z2": ["PD2"], "Z3": ["PD17"], "Z4": ["PD2"]},
    "custom_keys": ["Z1", "Z2", "Z3", "Z4", "Z5"],
    # r2 9.4 第 2 条：开头那份基线的逐行对应对的是第一段 r2 档的两份产物（5.6 的名字；日期是第一段跑的那一天，与这里不同就随冻结值一起改）。
    "segment1_products": [
        "research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-legal-family.out",
        "research/results/e165-replay-bounded-enumeration-2026-10-02-r2-segment1b-adversarial-family.out",
    ],
}
# 原文与替换文逐字相同的两对（原登记第九节）：355 与 23、414 与 406，标签表里共用前一行的依据。
SAME_AS = {355: 23, 414: 406}
EXIT_OK, EXIT_INCOMPLETE, EXIT_USAGE, EXIT_ANCHOR, EXIT_HALT_H1, EXIT_VOID_V6, EXIT_VOID_V7, EXIT_NOT_PROFILE_R2 = 0, 1, 2, 3, 4, 5, 6, 7
EXIT_PIN_INCOMPLETE = 8   # 钉红例（文件头「钉红例」）：当场追加或 --check 没过
# 钉红例用的那一份：追加与 --check 的写法在它文件头
PIN_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "e165-pin-red-examples.py")
BREAK_MODES = ("noanchorcheck", "reuseignored", "finishignored", "recorddeferred", "progressnever", "byrownumber", "nodupcheck", "nolocationcheck", "recoveryonly",
               "j6qnfromj6", "noprofile", "noprofilecheck", "zfirstfour",
               "r3itemsmissing", "r3firstredswap", "r3kapinunchecked", "r3noprofile",
               "r4itemsmissing", "r4firstredswap", "r4kgcountsasred", "r4kgpinunchecked", "r4noprofile",
               "r5pinsignored", "r4gateignored", "owntablesplit", "defaultone",
               "pincheckskipped", "retainedmissingred", "itemcountsdropped", "itemlinedropped", "unknownitemunlisted",
               "fixedcopylist", "copyconfigurationunset", "outputoverwritten", "judgeoffthetree", "ledgeroverwritten", "judgewholedriver")
# 按项判定那一层的几项（文件头「弄坏开关」）：复用那六格只把外面开着的这几项带进去
ITEM_VERDICT_BREAK_MODES = ("reuseignored", "finishignored", "recorddeferred", "judgeoffthetree")
# 外面打开时自证的 r2 那几趟原样传进去的四项（文件头「弄坏开关」）
R2_BREAK_MODES = ("j6qnfromj6", "noprofile", "noprofilecheck", "zfirstfour")
# 外面打开时自证的 r3 那几趟原样传进去的四项（登记 r3 9.3 第 6 条的表）；别的趟不带
R3_BREAK_MODES = ("r3itemsmissing", "r3firstredswap", "r3kapinunchecked", "r3noprofile")
# 外面打开时自证的 r4 那几趟原样传进去的五项（登记 r4 9.3 第 6 条的表）；别的趟不带
R4_BREAK_MODES = ("r4itemsmissing", "r4firstredswap", "r4kgcountsasred", "r4kgpinunchecked", "r4noprofile")
# ── r3 档（登记 r3 9.3，第三段）。不给 --profile 是 r2 档，上面那一套一个字不动。
# 取第二段冻结根上的值（第二段报告第九节）：recovery.rs、lib.rs 整份哈希与登记第三节相同；crates/mutations.tsv 是根里那一份（第二段盖上装置变异行之后）；
# 两个函数的区间用 r1 9.2 的取法（fn 行到其后第一个只有 } 的行）在根上现算；names 的第三项是 KA 上 KJ2 的违例数：("pinned", n) 是登记 5.7、第七节钉的值
# （1823 → 18、1824 → 52、1825 → 36、1826 → 18，TQ6），("inferred", n) 是登记 9.1 表里推的、没钉（1822 → 0、1827 → 18）。名字取根里 crates/mutations.tsv 第一列逐字。
R3_REGISTERED = {
    "files_sha256": {
        RECOVERY_PATH: "9337dc1b50eb13afdb346ee0a58360709e3a9f2bba6e5da8948fdda5685edd18",
        LIB_PATH: "40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c",
    },
    "table_sha256": "0a52bf84cd1d811619d24ae55f4f5da2866d80ce7e99d0db255501b6e5c96977",
    "functions": [
        (LIB_PATH, "choose_root_slot", 3977, 4016),
        (RECOVERY_PATH, "choose_root", 1250, 1287),
    ],
    "names": [
        (1822, "择根核平局取后者赢（里程碑三七·4 择根核，原型 m1_tie_last_wins）：择新键完全相同的两个槽取了后面那个，不是读根环次序里前面那个", ("inferred", 0)),
        (1823, "择根核比较不看实例代号（D22 已定项 7 平局实例代号高者赢；原型 m2_ignore_instance）：txg 平局时停在读根环次序里前面那条实例代号低的根", ("pinned", 18)),
        (1824, "择根核取第一个有效者（D22 已定项 24 先逐个验证全部候选、再在有效者中择新；原型 m3_first_valid_wins）：择了一个自证过的槽就不再换", ("pinned", 52)),
        (1825, "择根核 txg 平局时实例代号低者赢（C650 改坏择根序：同 checkpoint_txg 平局取低）", ("pinned", 36)),
        (1826, "择根核以实例代号为主、txg 只破平局（C650 改坏择根序：实例代号为主）", ("pinned", 18)),
        (1827, "择根胶水摊择新键时不带实例代号（核看到的实例代号都是 0，txg 平局时停在读根环次序里前面那条）", ("inferred", 18)),
    ],
    # TV7 的比对对象：第二段的产物（取脚本所在的仓里的 research/results/，不取 --root）
    "segment2_product": "research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out",
}
# 登记第七节「本地择根序加理想重放」表 KG 那八行（出自设计员的 anchors_r4.out，sha256 5a36b72237f84c52e6901ae5affcfc5431fd4c8b123f5b9e476907b749ba1fc7）：
# 每个 (M, U) 格上 KJ1 / KJ1T / KJ2 的违例数，按择根序分列；第二段拿它当每条变体在 KG 上每格的钉值（TQ10，合法域那几行只当本地对照、不钉变体）。
R4_KG_RULE_READINGS = {
    "today": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1822": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1823": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1824": {(1, 0): (0, 0, 8), (1, 1): (0, 0, 117), (1, 2): (468, 468, 351), (1, 3): (1404, 1404, 1053), (2, 0): (0, 0, 10824), (2, 1): (0, 0, 88296), (2, 2): (292302, 292302, 423162), (2, 3): (1765179, 1770093, 1756485)},
    "v1825": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 0), (2, 1): (0, 0, 0), (2, 2): (53154, 53154, 0), (2, 3): (265923, 284607, 0)},
    "v1826": {(1, 0): (0, 0, 0), (1, 1): (0, 0, 0), (1, 2): (117, 117, 0), (1, 3): (351, 351, 0), (2, 0): (0, 0, 36), (2, 1): (0, 0, 630), (2, 2): (58824, 58824, 7236), (2, 3): (309609, 346005, 49680)},
}

# ── r4 档（登记 r4 9.3，第二段）。不给 --profile 是 r2 档、--profile r3 是 r3 档，上面两套一个字不动。
# 取第一段根上的值（登记 r4 修订 2）：recovery.rs、lib.rs 整份哈希与登记第三节相同；crates/mutations.tsv 是根里那一份（盖上装置变异行之后）；
# 两个函数的区间用 r1 9.2 的取法（fn 行到其后第一个只有 } 的行）在根上现算；names 的第一项是今天主工作区里的表行号（只用来给变体起名与在产物里标出来），
# 第三项是 KA 上 KJ2 的违例数：("pinned", n) 是登记第七节钉的值（v1823 → 18、v1824 → 52、v1825 → 36、v1826 → 18，TQ10），("inferred", n) 是登记 9.1 表里推的、没钉
# （v1822 → 0、v1827 → 18）；labels 是各行在登记里的变体名（v1822–v1827，r3 的编号当名字用，同一次序）。名字取根里 crates/mutations.tsv 第一列逐字。
R4_REGISTERED = {
    "files_sha256": {
        RECOVERY_PATH: "264664af7a694fbda1295ac450c3d9c2198336d002b3c288b0f797731350e625",
        LIB_PATH: "40cd99b58242a5e522a9e302fa83ceaed17cdd5c2ea8e8811e0e223a4649f08c",
    },
    "table_sha256": "db2981d0d6fccb1a65bdf701897d4475302ec1ba8cd87501fa1374d191bd9349",
    "functions": [
        (LIB_PATH, "choose_root_slot", 3977, 4016),
        (RECOVERY_PATH, "choose_root", 1255, 1292),
    ],
    "names": [
        (1821, "择根核平局取后者赢（里程碑三七·4 择根核，原型 m1_tie_last_wins）：择新键完全相同的两个槽取了后面那个，不是读根环次序里前面那个", ("inferred", 0)),
        (1822, "择根核比较不看实例代号（D22 已定项 7 平局实例代号高者赢；原型 m2_ignore_instance）：txg 平局时停在读根环次序里前面那条实例代号低的根", ("pinned", 18)),
        (1823, "择根核取第一个有效者（D22 已定项 24 先逐个验证全部候选、再在有效者中择新；原型 m3_first_valid_wins）：择了一个自证过的槽就不再换", ("pinned", 52)),
        (1824, "择根核 txg 平局时实例代号低者赢（C650 改坏择根序：同 checkpoint_txg 平局取低）", ("pinned", 36)),
        (1825, "择根核以实例代号为主、txg 只破平局（C650 改坏择根序：实例代号为主）", ("pinned", 18)),
        (1826, "择根胶水摊择新键时不带实例代号（核看到的实例代号都是 0，txg 平局时停在读根环次序里前面那条）", ("inferred", 18)),
    ],
    "labels": ["v1822", "v1823", "v1824", "v1825", "v1826", "v1827"],
    "kg_columns": None,   # 下面一行填进来（自证里换成假的）
    # TV7 的比对对象：第一段的产物（取脚本所在的仓里的 research/results/，不取 --root）；两遍的 E7RESULT 逐字节相同，取最后一次源码改动之后那一遍
    "segment1_product": "research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out",
}
R4_REGISTERED["kg_columns"] = {rule: {f"{m},{u}": list(reading) for (m, u), reading in cells.items()} for rule, cells in R4_KG_RULE_READINGS.items()}

# ── 第 5 次重跑的钉值（登记 research/prompts/e165-r5-prereg.md 9.3 第 1、2 条，第二段）：--pins r5 选它；不给 --pins 时三个模式照旧取上面三套，一个字不改。
# 都是第一段根（/tmp/claude-1000/e165-r5-segment1/root，清单 root-manifest-at-start.sha256 的 sha256 95237ecf…）上现算的：两份源文件与 crates/mutations.tsv 的整份 sha256，
# 函数区间照 r1 9.2 的取法（fn 行到其后第一个只有 } 的行）；names 的第一项换成根里 crates/mutations.tsv 的表行号（只用来起名），名字、标签、KA 钉值、labels、KG 钉值、custom_keys 不动。
R5_FILES_SHA256 = {
    RECOVERY_PATH: "4454f6d3eab170121ccb92778e7d4525cdaa6a10a6c6bf16d7c213d0edc22632",
    LIB_PATH: "9966f42f396d22c2a90bdd35864557edd7ce45285be3d6b796ba9f8d18aa2fa4",
}
R5_TABLE_SHA256 = "bc2627f85100b155c14b2d7720488e2a49c7a87a20a45325343a22870942249d"
R5_R2_TABLE_ROWS = [23, 24, 36, 224, 347, 348, 399, 407, 441, 505, 506, 507, 508, 509, 510, 511, 604, 605, 908, 909, 910, 990, 991]
R5_ROOT_CHOICE_TABLE_ROWS = [1810, 1811, 1812, 1813, 1814, 1815]
R5_ROOT_CHOICE_FUNCTIONS = [(LIB_PATH, "choose_root_slot", 3980, 4019), (RECOVERY_PATH, "choose_root", 1268, 1305)]
R5_PRODUCT_PREFIX = "research/results/e165-replay-bounded-enumeration-2026-10-04-r5-"
R5_PINS = {
    "r2": dict(REGISTERED, files_sha256=R5_FILES_SHA256, table_sha256=R5_TABLE_SHA256,
               functions=[
                   (LIB_PATH, "record_ends_its_publish", 3101, 3109),
                   (LIB_PATH, "readable_record_of_the_same_publish_follows", 3113, 3228),
                   (LIB_PATH, "counter_of_the_last_record_the_root_covers", 3232, 3300),
                   (LIB_PATH, "is_above_the_chosen_root", 3303, 3312),
                   (LIB_PATH, "maximum_transaction_of", 3315, 3361),
                   (LIB_PATH, "judge_the_candidate", 3372, 3537),
                   (LIB_PATH, "replay_core", 3543, 3975),
                   (RECOVERY_PATH, "judge_back_chain_against_the_previous_record_on_disk", 3404, 3436),
                   (RECOVERY_PATH, "replay_journal", 3467, 3614),
               ],
               names=[(row, name, tags) for row, (_registered_row, name, tags) in zip(R5_R2_TABLE_ROWS, REGISTERED["names"])],
               # V7 的比对对象：第一段重跑的 r2 档合法历史、对抗两份（登记第十一节末尾比对对象表第 4、5 行的重跑）
               segment1_products=[R5_PRODUCT_PREFIX + "segment1-r2-legal-family.out", R5_PRODUCT_PREFIX + "segment1-r2-adversarial-family.out"]),
    "r3": dict(R3_REGISTERED, files_sha256=R5_FILES_SHA256, table_sha256=R5_TABLE_SHA256, functions=list(R5_ROOT_CHOICE_FUNCTIONS),
               names=[(row, name, pin) for row, (_registered_row, name, pin) in zip(R5_ROOT_CHOICE_TABLE_ROWS, R3_REGISTERED["names"])],
               # TV7 的比对对象：第一段重跑的 r3 择根那一份（比对对象表第 15 行的重跑）
               segment2_product=R5_PRODUCT_PREFIX + "segment1-r3-root-choice.out"),
    "r4": dict(R4_REGISTERED, files_sha256=R5_FILES_SHA256, table_sha256=R5_TABLE_SHA256, functions=list(R5_ROOT_CHOICE_FUNCTIONS),
               names=[(row, name, pin) for row, (_registered_row, name, pin) in zip(R5_ROOT_CHOICE_TABLE_ROWS, R4_REGISTERED["names"])],
               # TV7 的比对对象：第一段重跑的 r4 带故障那一份（比对对象表第 16 行的重跑）
               segment1_product=R5_PRODUCT_PREFIX + "segment1-r4-root-choice-faults.out"),
}
# 基线 family=root_choice_faults 的 name=config 行要带的 R2 闸字段与值：5-甲 的装置在 --gate r2 时实际写的（产物 R5_GATE_R2_PINS["segment1_product"] 第一行的 witness_gate=…）
R4_GATE_R2_CONFIG_FIELD = ("witness_gate", "r2_some_slot_reports_a_read_error_and_no_device_self_verified_both_slots")
# 5-乙 跑在 5-甲 的根上（/tmp/claude-1000/e165-r5-segment3/root5：第一段根盖上 5-甲 改过的装置与 crates/mutations.tsv，登记修1.十一「根、清单、在哪编」）：
# 变异表 sha256 是那份根上的 crates/mutations.tsv 的；TV7 的比对对象是 5-甲 的产物（取脚本所在的仓的 research/results/，同别的几套是路径名）。
# 两份源文件与两个函数区间仍是第一段根上的（5-甲 不改它们；开工核对时对着根现判，对不上停在 RH1）。
R5_GATE_R2_TABLE_SHA256 = "c9a03eb4af85a8e4c7206dfa3609593390516b80ca52be7090aced5696e84046"
R5_GATE_R2_PINS = dict(R5_PINS["r4"], table_sha256=R5_GATE_R2_TABLE_SHA256, segment1_product=R5_PRODUCT_PREFIX + "segment5-r2-gate-root-choice-faults.out")


def default_jobs_and_threads(jobs, threads, environment=os.environ):
    """没给 --jobs 是 1；没给 --threads 是 N ÷ jobs（向下取整，至少 1），N 是 SINGLEFS_THREAD_CAP，没设取登记的重活默认并行度。"""
    if "defaultone" in BREAK:   # 弄坏开关：取 1 × 1
        return jobs or 1, threads or 1
    jobs = jobs or 1
    return jobs, threads or max(1, default_heavy_threads(environment) // jobs)


class Stop(Exception):
    def __init__(self, code, message):
        super().__init__(message)
        self.code = code


def unknown_item(name, names, listing):
    """--item 写了不认识的变体名：列出可点名的变体，退 2（文件头「每项一行与总计」那一条规则）。"""
    named = "" if "unknownitemunlisted" in BREAK else f"；可点名的 {len(names)} 项：{' '.join(names)}"
    return Stop(EXIT_USAGE, f"  ✗ 不认识的变体：{name}{named}\n  → 怎么办：--item 照可点名的名字写；{listing} 逐行列变体与它的表、行号")


ITEM_LINES = {"report": None}   # 这一趟整轮的那一份 ItemLines；文件末尾收尾时打总计


class ItemLines:
    """整轮每项一行与末行总计（文件头「每项一行与总计」）。打的时候由调用方拿着锁（work() 里那把），这里不另加锁。"""

    def __init__(self):
        self.dispatched = self.collected = self.lines = 0
        self.judged = self.red = self.reused = self.failed = 0
        self.dropped = False
        self.started = time.monotonic()
        ITEM_LINES["report"] = self

    def emit(self, name, exit_code, judged, red, reused, seconds):
        counts = "" if "itemcountsdropped" in BREAK else f" judged={judged} red={red} reused={reused}"
        print(f"E165_KILL_BOUNDS_ITEM item={name} exit={exit_code}{counts} seconds={seconds:.1f}", flush=True)
        self.lines += 1
        self.judged, self.red, self.reused, self.failed = self.judged + judged, self.red + red, self.reused + reused, self.failed + (1 if exit_code else 0)

    def reused_item(self, name, line):
        self.emit(name, 0, 0, int(parse_row(line).get("outcome") == "red"), 1, 0.0)

    def dispatch(self, count):
        self.dispatched += count

    def judged_item(self, name, record):
        if "itemlinedropped" in BREAK and not self.dropped and name != "baseline":
            self.dropped = True
            return
        self.collected += 1
        outcome = parse_row(record["lines"][0]).get("outcome") if record["lines"] else None
        self.emit(name, 1 if record["problem"] else 0, 0 if record["problem"] else 1, int(outcome == "red"), 0, record["timing"][2])

    def close(self, code):
        if self.collected != self.dispatched:
            print(f"  ✗ 派出 {self.dispatched} 项、收回 {self.collected} 项：有变体做完了没打出它那一行，这一趟的总计不算数")
            print("  → 怎么办：看上面每项一行少了哪个变体、它的日志在草稿目录 kill-bounds/ 下；修好之后挪走这一趟的产物（有红例账的连同它；--output、--ledger 已经存在的不收），再重跑同一条命令")
            code = code or EXIT_INCOMPLETE
        print(f"E165_KILL_BOUNDS_TOTAL dispatched={self.dispatched} collected={self.collected} items={self.lines} judged={self.judged} red={self.red} "
              f"reused={self.reused} failed={self.failed} seconds={time.monotonic() - self.started:.1f}", flush=True)
        return code


PROGRESS_INTERVAL_SECONDS = 300   # 进度间隔（文件头「进度」）


class Progress:
    """跑着的时候每过一个进度间隔往 stderr 打一行（文件头「进度」）。clock 与 sink 可以换成假的（自证）；
    poll 判过没过间隔：过了打一行并交回它，没过交 None。"""

    def __init__(self, total, interval, clock=time.monotonic, sink=None):
        self.total, self.interval, self.clock = total, interval, clock
        self.sink = sink if sink is not None else sys.stderr
        self.lock = threading.Lock()
        self.in_flight = {}
        self.done = 0
        self.started = self.last_line = clock()
        self.stopped = threading.Event()
        self.thread = None

    def phase(self, name, text):
        with self.lock:
            now = self.clock()
            entry = self.in_flight.setdefault(name, {"since": now})
            entry.update(text=text, phase_since=now)

    def finished(self, name, counted=True):
        with self.lock:
            self.in_flight.pop(name, None)
            self.done += 1 if counted else 0

    def due(self, now):
        return "progressnever" not in BREAK and now - self.last_line >= self.interval

    def poll(self):
        now = self.clock()
        if not self.due(now):
            return None
        with self.lock:
            doing = "、".join(f"{name}（{entry['text']}，这一步 {now - entry['phase_since']:.0f} 秒，这一项共 {now - entry['since']:.0f} 秒）"
                             for name, entry in self.in_flight.items()) or "无"
            line = f"E165_KILL_BOUNDS_PROGRESS 挂钟 {now - self.started:.0f} 秒：变体做完 {self.done}/{self.total} 个；在做 {len(self.in_flight)} 项：{doing}"
        self.last_line = now
        print(line, file=self.sink, flush=True)
        return line

    def start(self):
        def loop():
            while not self.stopped.wait(max(0.01, min(self.interval / 4, 5.0))):
                self.poll()
        self.thread = threading.Thread(target=loop, name="e165-kill-bounds-progress", daemon=True)
        self.thread.start()

    def stop(self):
        self.stopped.set()
        if self.thread is not None:
            self.thread.join()


def progress_interval(setup):
    given = os.environ.get("E165_KILL_BOUNDS_PROGRESS_INTERVAL")
    return float(given) if given and setup.selftest else PROGRESS_INTERVAL_SECONDS


def sha256_of_file(path):
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def unescape(text):
    return text.replace("\\n", "\n")


class Setup:
    """这一次跑的仓根与登记值；自证时换成假的仓与假的登记值。"""

    def __init__(self, root, selftest, profile="r2", pins=None, gate=None):
        self.root = root
        self.selftest = selftest
        self.profile = profile
        # --pins r5（登记 r5 9.3 第 1、2 条）；弄坏开关 r5pinsignored：给了也照样用原来那套
        self.pins = pins
        self.gate = gate
        uses_r5_pins = pins == "r5" and "r5pinsignored" not in BREAK
        if uses_r5_pins:
            self.registered = dict(R5_GATE_R2_PINS if gate == "r2" else R5_PINS[profile])
        else:
            self.registered = dict(REGISTERED if profile == "r2" else R3_REGISTERED if profile == "r3" else R4_REGISTERED)
        if selftest:
            name = "selftest-registered" if profile == "r2" else "selftest-registered-r3" if profile == "r3" else "selftest-registered-r4"
            if uses_r5_pins:
                name += "-r5-gate-r2" if gate == "r2" else "-r5"
            with open(os.path.join(root, name + ".json"), encoding="utf-8") as handle:
                self.registered = json.load(handle)
        self.cargo = [os.path.join(root, "fake-cargo")] if selftest else ["cargo"]
        # r3 模式的 --root 是第二段冻结根（只有 Cargo.toml、Cargo.lock、.cargo、crates/）：--measure 用的内存包装与第二段产物取脚本所在的仓（自证里取假仓）。
        self.script_root = root if selftest else DEFAULT_ROOT
        # r2 模式原来的冻结根自己带 research/（自造表、标签表、--measure 用的内存包装、第一段产物都从根里读）；--pins r5 的根是第一段根，只有 crates/ 那几样，
        # 这几样改取脚本所在的仓（登记 r5 第十一节「根、清单、在哪编」：驱动的自造表与 scripts 取脚本所在的仓）。
        self.research_root = self.script_root if pins == "r5" else root
        self.memory_cap_runner = os.path.join(self.research_root if profile == "r2" else self.script_root, "scripts/hosts/run-with-memory-cap.sh")
        # 这一趟本次未跑的比对（留存产物不在，retained_product_not_run 记），收尾那句逐项列出
        self.not_run = []

    def bounds_arguments(self):
        """每次整份跑装置的界参数（判法模块的 device_bounds_arguments）。"""
        return device_bounds_arguments(self)

    def pins_arguments(self):
        """复跑命令里要带的 --pins / --gate（照这一次给的原样写）。"""
        return (["--pins", self.pins] if self.pins else []) + (["--gate", self.gate] if self.gate else [])

    def command_environment(self):
        """副本里跑编出来的装置的环境：多机配置取脚本所在的仓那一份（environment_in_the_copy）。判法模块起装置时经它取，拷副本那一层不进判法闭包。"""
        return environment_in_the_copy(self.script_root)


DEVICE_SOURCE_PATH = "crates/singlefs-verification/src/bin/e165_replay_bounded_enumeration.rs"


def pins_config_text(setup):
    """给了 --pins r5 / --gate r2 时产物 config 行多带的字段（登记 r5 修1.九乙第 3 条：config 行带上根里装置的 sha256）；都不给时是空串，原来三套的 config 行一个字不变。"""
    text = ""
    if setup.pins:
        text += f" pins={setup.pins} pins_registration=research/prompts/e165-r5-prereg.md"
    if setup.gate:
        device = os.path.join(setup.root, DEVICE_SOURCE_PATH)
        text += f" gate={setup.gate} device_sha256={sha256_of_file(device) if os.path.exists(device) else 'missing'}"
    return text


def read_source(setup, path):
    with open(os.path.join(setup.root, path), encoding="utf-8") as handle:
        return handle.read()


_CRATES_MUTATION_ROWS = []


def mutation_table_rows(setup):
    """crates/mutations.tsv 里成形的行：[(表行号, 各段原样)]。切表与成形照 scripts/mutation/crates-mutation-rows.py 的 read_table
    （门禁 crates-mutation-replay 与 prove-red.sh 同一份）；弄坏开关 owntablesplit 下自己切（四段以上就收）。"""
    if "owntablesplit" in BREAK:
        rows = []
        for number, line in enumerate(read_source(setup, TABLE_PATH).split("\n"), 1):
            if line.startswith("#") or not line.strip():
                continue
            fields = line.split("\t")
            if len(fields) >= 4:
                rows.append((number, fields))
        return rows
    if not _CRATES_MUTATION_ROWS:
        specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_e165_kill_bounds", os.path.join(SCRIPT_DIRECTORY, "..", "mutation", "crates-mutation-rows.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _CRATES_MUTATION_ROWS.append(module)
    rows, _malformed = _CRATES_MUTATION_ROWS[0].read_table(os.path.join(setup.root, TABLE_PATH))
    return [(row.line_number, list(row.fields)) for row in rows]


def function_ranges(setup):
    """r1 9.2：函数的行区间 = 从 `fn 名字`（或 `pub fn 名字`）那一行到其后第一行只有 `}` 的那一行。
    返回 [(文件, 名字, 起, 止)]；找不到、不唯一或没有收尾行时起止记 None。"""
    texts, ranges = {}, []
    for path, name, _first, _last in setup.registered["functions"]:
        lines = texts.setdefault(path, read_source(setup, path).split("\n"))
        starts = [number for number, line in enumerate(lines, 1) if re.match(r"^(pub )?fn " + re.escape(name) + r"\b", line)]
        if len(starts) != 1:
            ranges.append((path, name, None, None))
            continue
        end = next((number for number in range(starts[0], len(lines) + 1) if lines[number - 1] == "}"), None)
        ranges.append((path, name, starts[0], end))
    return ranges


def locate_in_functions(ranges, path, first_line, last_line):
    """命中处的起行落在 9.2 的某个函数里就取它，否则末行落在某个函数里取它；都不在交 None。"""
    for line in (first_line, last_line):
        for range_path, name, start, end in ranges:
            if range_path == path and start is not None and start <= line <= end:
                return name, start, end
    return None


def tag_text(tags):
    return ",".join(sorted(tags, key=lambda tag: int(tag[2:]))) or "empty"


def compare_tag_sets(registered, today):
    """r1 9.3：两个集合相等 ⇔ 对得上同一个判定。返回 (相等, 今天多出来的, 今天少了的)。"""
    registered, today = set(registered), set(today)
    return registered == today, sorted(today - registered), sorted(registered - today)


def load_tag_table(setup):
    """scripts/mutation/tables/e165_replay_bounded_enumeration_tags.tsv：键 → 版本 → [(标签们, 在不在, 依据)]；集合 = 标 yes 的标签。"""
    table = {}
    with open(os.path.join(setup.research_root, TAGS_TABLE_PATH), encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            if line.startswith("#") or not line.strip():
                continue
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 5 or fields[1] not in ("old", "today") or fields[3] not in ("yes", "no", "none"):
                raise Stop(EXIT_HALT_H1, f"  ✗ {TAGS_TABLE_PATH} 第 {line_number} 行不是五段、或版本不是 old / today、或第四段不是 yes / no / none\n"
                                        "  → 怎么办：照表头改；这一次什么都没跑")
            table.setdefault(fields[0], {}).setdefault(fields[1], []).append((fields[2], fields[3], fields[4], line_number))
    return table


def tags_of(table, key, version):
    """集合：第四段为 yes 的行的标签（一行可以写几个，逗号连）；空集的行第四段写 none。键或版本缺了交 None。"""
    entries = table.get(key, {}).get(version)
    if entries is None:
        return None
    tags = set()
    for tag, member, _text, _line in entries:
        if member == "yes":
            tags.update(tag.split(","))
    return tags


def select_by_name(setup, ranges, texts):
    """RQ4（r1 9.1）：登记的每个变异名在今天的 crates/mutations.tsv 里按第一列逐字找；那一行要满足四条：恰 1 行、
    第二列是 recovery.rs 或 lib.rs、原文在那份文件里命中 1 次、命中处的起行或末行落在 9.2 的函数里。不按行号选。"""
    table_lines = read_source(setup, TABLE_PATH).split("\n")   # 只给弄坏开关 byrownumber 按登记时的行号取行
    by_name = {}
    for number, fields in mutation_table_rows(setup):
        by_name.setdefault(fields[0], []).append((number, fields))
    selected = []
    for registered_row, name, registered_tags in setup.registered["names"]:
        entry = {"registered_row": registered_row, "name": name, "registered_tags": registered_tags, "failure": None, "today_row": None,
                 "file": None, "hits": None, "first": None, "last": None, "function": None, "function_lines": None, "old": None, "new": None}
        candidates = by_name.get(name, [])
        if "byrownumber" in BREAK:      # 弄坏开关：按登记时的行号取行，不看名字
            candidates = [(registered_row, table_lines[registered_row - 1].split("\t"))] if registered_row <= len(table_lines) else []
        entry["rows_with_this_name"] = len(candidates)
        if not candidates:
            entry["failure"] = "no_row_with_this_name"
        elif len(candidates) > 1 and "nodupcheck" not in BREAK:
            entry["failure"] = "more_than_one_row_with_this_name"
        if entry["failure"] is None:
            number, fields = candidates[0]
            entry.update(today_row=number, file=fields[1], old=unescape(fields[2]), new=unescape(fields[3]), raw_old=fields[2], raw_new=fields[3])
            if fields[1] not in SOURCE_FILES:
                entry["failure"] = "file_column_is_not_a_registered_source"
            else:
                entry["hits"] = texts[fields[1]].count(entry["old"])
                if entry["hits"] != 1:
                    entry["failure"] = f"anchor_hits_{entry['hits']}"
                else:
                    first = texts[fields[1]].count("\n", 0, texts[fields[1]].index(entry["old"])) + 1
                    last = first + entry["old"].count("\n")
                    entry.update(first=first, last=last)
                    located = locate_in_functions(ranges, fields[1], first, last)
                    if located is None and "nolocationcheck" not in BREAK:
                        entry["failure"] = "outside_the_registered_functions"
                    elif located is not None:
                        entry["function"], entry["function_lines"] = located[0], (located[1], located[2])
        selected.append(entry)
    return selected


def custom_rows(setup, texts):
    """自造表每行：四段、文件列是 recovery.rs 或 lib.rs；原文命中次数与所在函数一并交回。"""
    rows = []
    with open(os.path.join(setup.research_root, CUSTOM_TABLE_PATH), encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            if line.startswith("#") or not line.strip():
                continue
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 4 or fields[1] not in SOURCE_FILES:
                raise Stop(EXIT_USAGE, f"  ✗ {CUSTOM_TABLE_PATH} 第 {line_number} 行不是四段、或文件列不是 {' 或 '.join(SOURCE_FILES)}\n"
                                       "  → 怎么办：自造表每行写 变异名、文件、原文、替换文 四段，制表符分隔")
            old = unescape(fields[2])
            hits = texts[fields[1]].count(old)
            first = texts[fields[1]].count("\n", 0, texts[fields[1]].index(old)) + 1 if hits == 1 else None
            rows.append({"line": line_number, "title": fields[0], "key": fields[0].split()[0], "file": fields[1], "old": old,
                         "new": unescape(fields[3]), "hits": hits, "first": first, "last": first + old.count("\n") if first else None})
            if "zfirstfour" in BREAK and len(rows) == 4:   # 弄坏开关：只读前四行
                break
    return rows


def check_the_custom_keys(setup, items):
    """r2 9.3、9.4 第 1 条：自造表的 Z 行名字（依次）要等于登记的 custom_keys（Z1–Z5）；少一行、多一行、换了名都停在 RH1。"""
    keys = [item["name"] for item in items if item["table"] == CUSTOM_TABLE_PATH]
    expected = list(setup.registered.get("custom_keys", []))
    if keys != expected:
        raise Stop(EXIT_HALT_H1, f"  ✗ 停机条款 RH1：{CUSTOM_TABLE_PATH} 的 Z 行是 {'、'.join(keys) or '（一行都没有）'}，登记的是 {'、'.join(expected)}\n"
                                 "  → 怎么办：自造表照登记 r2 9.3（Z1–Z4 不动，加 Z5）；这一次一条变异都没跑")


def load_items(setup):
    """变体清单：基线、照 r1 9.1 按变异名选出的行（原文、替换文与文件都逐字相同的并成一个变体）、自造表的每一行。
    选不到的名字（RQ4 有一条不满足）是停机：退 4、一条都不跑，细节交 --select-rows 的产物。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    failed = [entry for entry in selection if entry["failure"] is not None]
    if failed:
        raise Stop(EXIT_HALT_H1, "\n".join(f"  ✗ 停机（r1 RF2 / RQ4）：「{entry['name'][:40]}」今天选不到：{entry['failure']}" for entry in failed)
                   + "\n  → 怎么办：登记里的判定在今天的表与源码里选不到；交回主 agent，不接着跑（--select-rows 写逐行结果）")
    items = [{"name": "baseline", "table": "none", "rows": [], "titles": ["基线（不改）"], "old": None, "new": None, "file": None}]
    by_replacement = {}
    for entry in sorted(selection, key=lambda candidate: candidate["today_row"]):
        key = (entry["file"], entry["raw_old"], entry["raw_new"])
        if key in by_replacement:
            by_replacement[key]["rows"].append(entry["today_row"])
            by_replacement[key]["titles"].append(entry["name"])
            continue
        item = {"name": f"tsv-{entry['today_row']}", "table": TABLE_PATH, "rows": [entry["today_row"]], "titles": [entry["name"]],
                "file": entry["file"], "old": entry["old"], "new": entry["new"]}
        by_replacement[key] = item
        items.append(item)
    for row in custom_rows(setup, texts):
        items.append({"name": row["key"], "table": CUSTOM_TABLE_PATH, "rows": [row["line"]], "titles": [row["title"]],
                      "file": row["file"], "old": row["old"], "new": row["new"]})
    return items, [entry["today_row"] for entry in selection], texts


def source_hashes(setup):
    return {path: sha256_of_file(os.path.join(setup.root, path)) for path in SOURCE_FILES}


def check_the_registered_inputs(setup):
    """停机条款 RH1：两份源文件与变异表的 sha256、9.2 的函数行区间都要等于登记 r1 的值。"""
    problems = []
    for path, actual in source_hashes(setup).items():
        if actual != setup.registered["files_sha256"][path]:
            problems.append(f"{path} 的 sha256 是 {actual}，登记的是 {setup.registered['files_sha256'][path]}")
    expected_table = setup.registered.get("table_sha256")
    if expected_table == TABLE_SHA256_PENDING:
        problems.append(f"{TABLE_PATH} 的登记值还是占位串（{TABLE_SHA256_PENDING}）：r2 9.4 第 1 条要第一段收尾冻结的根上的值，填进 REGISTERED 再跑")
    elif expected_table and sha256_of_file(os.path.join(setup.root, TABLE_PATH)) != expected_table:
        problems.append(f"{TABLE_PATH} 的 sha256 不等于登记的 {expected_table}")
    registered_ranges = [tuple(entry) for entry in setup.registered["functions"]]
    for registered_entry, actual_entry in zip(registered_ranges, function_ranges(setup)):
        if tuple(registered_entry) != tuple(actual_entry):
            problems.append(f"9.2 的函数 {actual_entry[1]} 在 {actual_entry[0]} 里今天的行区间是 {actual_entry[2]}–{actual_entry[3]}，登记的是 {registered_entry[2]}–{registered_entry[3]}")
    if problems:
        raise Stop(EXIT_HALT_H1, "\n".join(f"  ✗ 停机条款 RH1：{problem}" for problem in problems)
                   + "\n  → 怎么办：源码或变异表变了，9.1 的位置、9.2 的行区间都要重核；交回主 agent，不接着跑")


def check_the_anchors(items, texts):
    rotten = [item for item in items if item["old"] is not None and texts[item["file"]].count(item["old"]) != 1]
    if rotten and "noanchorcheck" not in BREAK:
        lines = [f"  ✗ 锚点腐化：{item['name']}（{item['table']} 第 {item['rows'][0]} 行「{item['titles'][0]}」）的原文在 {item['file']} 里命中 "
                 f"{texts[item['file']].count(item['old'])} 次，不是 1 次" for item in rotten]
        raise Stop(EXIT_ANCHOR, "\n".join(lines) + "\n  → 怎么办：把那一行的原文改到今天源码里逐字存在、只出现一次的那一段；这一次一条变异都没跑")


def check_the_profile_rows(baseline):
    """r2 9.4 第 6 条：基线每个有 J6 行的 (域, N) 都要有 J6Qn 两行；一个有 J6 行的 (域, N) 都没有、或有一处缺 J6Qn 行，就是装置没带 --profile r2。
    弄坏开关 noprofilecheck：不核。"""
    if "noprofilecheck" in BREAK:
        return
    places, order = violation_lines_by_place(baseline["rows"])
    with_j6 = [place for place in order if any(criterion in places[place] for criterion in J6_ROWS)]
    lacking = [place for place in with_j6 if any(criterion not in places[place] for criterion in J6QN_ROWS)]
    if lacking or not with_j6:
        first = f"{lacking[0][0]}:{lacking[0][1]}" if lacking else "（基线一行 J6 都没有）"
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --profile r2：基线有 J6 行的 {len(with_j6)} 个 (域, N) 里 {len(lacking)} 个缺 J6Qn_ 行，第一个 {first}\n"
                                        "  → 怎么办：装置要认 --profile r2（r2 登记 5.1）、驱动调它时要带上；这一次变体一个都没编")


# ── 一个变体

COPY_ENTRIES_BY_ROOT = {}            # 每棵根只算一次闭包：一趟里根不变；几个变体并行建副本，用锁串起来
COPY_ENTRIES_LOCK = threading.Lock()


def copy_root_entries(root, destination):
    """把 root 照 scripts/hosts/lib_repository_copy.py 在它上面现算的共用清单与闭包拷进 destination（不在就建；文件头「做法」第 4 步）。
    闭包算不成、拷不成抛 Stop（退 1，点名哪一处）。弄坏开关 fixedcopylist：只拷 FIXED_COPY_ENTRIES。"""
    if "fixedcopylist" in BREAK:
        entries = FIXED_COPY_ENTRIES
    else:
        with COPY_ENTRIES_LOCK:
            if root not in COPY_ENTRIES_BY_ROOT:
                try:
                    COPY_ENTRIES_BY_ROOT[root] = repository_copy.copy_closure(root).entries
                except ValueError as error:   # ClosureRefused 是它的子类
                    raise Stop(EXIT_INCOMPLETE, f"  ✗ 副本的范围在 {root} 上算不成：{error}\n"
                                                "  → 怎么办：照 scripts/hosts/lib_repository_copy.py 文件头「闭包」修好那一处（根里缺编译期读的文件就先补进根）") from error
            entries = COPY_ENTRIES_BY_ROOT[root]
    os.makedirs(destination)
    try:
        repository_copy.copy_entries(root, destination, entries)
    except OSError as error:
        raise Stop(EXIT_INCOMPLETE, f"  ✗ 副本 {destination} 没拷成：{error}\n  → 怎么办：看 rsync 的报错（盘满、权限）") from error


def environment_in_the_copy(script_root, environment=None):
    """副本里起命令（编、跑副本里编出来的装置）的环境：多机配置取脚本所在的仓那一份（scripts/hosts/lib_repository_copy.py 的
    environment_for_commands_in_the_copy；--root 给的根与副本都不在 git 里，照它们的位置找不到配置）。弄坏开关 copyconfigurationunset：照原样交。"""
    if "copyconfigurationunset" in BREAK:
        return dict(os.environ if environment is None else environment)
    return repository_copy.environment_for_commands_in_the_copy(script_root, environment)


def prepare_and_build(setup, item, directory, threads, wrap_build_for_measure=False):
    """建副本、另存原件、改坏、编。返回 (二进制路径或 None, 说明, 编译挂钟秒)。"""
    if os.path.exists(directory):
        shutil.rmtree(directory)
    tree = os.path.join(directory, "tree")
    copy_root_entries(setup.root, tree)
    for path in SOURCE_FILES:
        if sha256_of_file(os.path.join(tree, path)) != setup.registered["files_sha256"][path]:
            raise Stop(EXIT_HALT_H1, f"  ✗ 停机条款 RH1：副本里的 {path} 的 sha256 不等于登记值\n  → 怎么办：仓里的源文件在跑的时候被改了；交回主 agent")
    target = target_of(item)
    original = os.path.join(directory, "source.original")
    if target is not None:
        shutil.copy2(os.path.join(tree, target), original)
    if item["old"] is not None:
        text = open(original, encoding="utf-8").read()
        if text.count(item["old"]) == 1:
            text = text.replace(item["old"], item["new"])
        elif "noanchorcheck" not in BREAK:
            raise Stop(EXIT_ANCHOR, f"  ✗ 锚点腐化：{item['name']} 的原文在副本里的 {target} 命中 {text.count(item['old'])} 次\n  → 怎么办：改那一行的原文")
        with open(os.path.join(tree, target), "w", encoding="utf-8") as handle:
            handle.write(text)
    environment = environment_in_the_copy(setup.script_root)
    environment["CARGO_TARGET_DIR"] = os.path.join(directory, "target")
    environment["CARGO_BUILD_JOBS"] = str(threads)
    command = ["nice", "-n", "19"] + setup.cargo + ["build", "--release", "--offline", "-p", "singlefs-verification", "--bin", BIN_NAME]
    if wrap_build_for_measure:
        environment["RUN_WITH_MEMORY_CAP_KEY"] = "e165-kill-bounds measure build"
        command = ["bash", setup.memory_cap_runner, "8G"] + command
    started = time.monotonic()
    completed = run_captured(command, environment, cwd=tree, log_path=os.path.join(directory, "build.log"))
    elapsed = time.monotonic() - started
    binary = os.path.join(directory, "target", "release", BIN_NAME)
    if completed.returncode != 0 or not os.path.exists(binary):
        error_lines = [line for line in completed.stderr.split("\n") if line.startswith("error")]
        return None, (error_lines[0] if error_lines else f"cargo build 退出码 {completed.returncode}"), elapsed
    return binary, "", elapsed


def target_of(item):
    """这个变体改副本里的哪一份源文件：变异行第二列（基线不改）。弄坏开关 recoveryonly 只改 recovery.rs、不看第二列。"""
    if item.get("old") is None:
        return None
    return RECOVERY_PATH if "recoveryonly" in BREAK else item.get("file")


def restore_and_remove(setup, directory, keep, target=None):
    """被改的那一份原件拷回副本，两份源文件的 sha256 都要等于登记值；不留的删掉整份。"""
    tree = os.path.join(directory, "tree")
    original = os.path.join(directory, "source.original")
    restored = False
    if os.path.isdir(tree):
        if target is not None and os.path.exists(original):
            shutil.copy2(original, os.path.join(tree, target))
        restored = all(os.path.exists(os.path.join(tree, path)) and sha256_of_file(os.path.join(tree, path)) == setup.registered["files_sha256"][path]
                       for path in SOURCE_FILES)
    if not keep:
        shutil.rmtree(directory, ignore_errors=True)
    return restored


def judge_built_variant(setup, item, baseline, options, judge):
    """做一个变体：在这里建副本、改坏、编（prepare_and_build），起装置、比、出产物行与红例账行交判法模块的 judge（judge_variant、judge_variant_r3、
    judge_variant_r4；判完照交进去的 restore 还原并删掉副本）。r3、r4 选行没满足的（item["failure"]，登记 TH5）不建副本、不编，直接交 judge 记 not_compared。
    返回 {lines: 产物行, ledger: 红例账行, problem: 没得出结果的原因或 None, timing: (编译秒, 跑秒, 这一项的秒)}。"""
    if item.get("failure") is not None:
        judged = judge(setup, item, baseline, options, None, None, None, None)
        timing = (0.0, 0.0, 0.0)
    else:
        directory = os.path.join(options["work"], item["name"])
        started = time.monotonic()
        mark(options, item["name"], "建副本、改坏、编译")
        binary, build_problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"])
        judged = judge(setup, item, baseline, options, binary, build_problem, directory,
                       lambda: restore_and_remove(setup, directory, keep=False, target=target_of(item)))
        timing = (build_seconds, judged["run_seconds"], time.monotonic() - started)
    return {"lines": judged["lines"], "ledger": judged["ledger"], "problem": judged["problem"], "timing": timing}


def process_item(setup, item, baseline, options):
    """不给 --profile 那一套做一个变体（判法模块的 judge_variant）。"""
    return judge_built_variant(setup, item, baseline, options, judge_variant)


def build_and_run_baseline(setup, options, name):
    item = {"name": name, "old": None, "new": None}
    directory = os.path.join(options["work"], name)
    label = "开头基线" if name == "baseline" else "收尾基线"
    mark(options, label, "建副本、编译")
    binary, problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"] * options["jobs"])
    if binary is None:
        raise Stop(EXIT_INCOMPLETE, f"  ✗ 基线编不过：{problem}（日志 {directory}/build.log）\n  → 怎么办：先让仓里这个 bin 编得过")
    mark(options, label, "跑 " + " ".join(setup.bounds_arguments()))
    completed, run_seconds = run_segment_two(setup, binary, options["threads"] * options["jobs"], directory)
    if options.get("progress") is not None:
        options["progress"].finished(label, counted=False)
    rows = result_rows(completed.stdout)
    if completed.returncode != 0 or not rows or not rows[-1].startswith("E7RESULT name=done "):
        raise Stop(EXIT_INCOMPLETE, f"  ✗ 基线没跑完或退出码不是 0（{completed.returncode}；日志 {directory}/run.log）\n  → 怎么办：先单跑基线看（撞内存上限的由 task-server 放大重排整条命令，不在这里判）")
    print(f"  基线 {name}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，{len(rows)} 行", file=sys.stderr)
    return {"binary": binary, "rows": rows, "row_count": len(rows), "exit": completed.returncode, "directory": directory}


def retained_product_not_run(setup, clause, paths):
    """比对的对象是 research/ 下的留存产物（可以整个删掉）：有不在的，这一项比对本次未跑——逐个点名不在的产物，记进 setup.not_run、打一行「本次未跑」，
    环境里有 GATE_NOT_RUN_FILE 的另往那份文件追加一行；不当绿、不判红，交 True（调用方不比、比对那一栏写 not_run）。都在交 False（照旧比）。
    弄坏开关 retainedmissingred 下一律交 False（不在就判红或读不到）。"""
    absent = [path for path in paths if not os.path.exists(path)]
    if not absent or "retainedmissingred" in BREAK:
        return False
    for path in absent:
        line = f"作废条款 {clause} 的比对：留存产物 {path} 不在"
        setup.not_run.append(line)
        print(f"  ! 本次未跑：{line}（留存产物可以整个删掉；在时照比，不在不当绿、不判红）")
        not_run_file = os.environ.get("GATE_NOT_RUN_FILE")
        if not_run_file:
            with open(not_run_file, "a", encoding="utf-8") as handle:
                handle.write(f"e165-replay-mutation-kill-bounds：{line}\n")
    return True


def check_against_segment_one(setup, baseline):
    """作废条款 V7：基线每个 (域, N) 的行必须逐字出现在第一段的产物里。第一段产物有不在的，这一项本次未跑（retained_product_not_run）。"""
    if retained_product_not_run(setup, "V7", [os.path.join(setup.research_root, product) for product in setup.registered["segment1_products"]]):
        baseline["verbatim_rows"] = "not_run"
        return
    known = set()
    for product in setup.registered["segment1_products"]:
        with open(os.path.join(setup.research_root, product), encoding="utf-8") as handle:
            known.update(line.rstrip("\n") for line in handle)
    place_rows = [row for row in baseline["rows"] if " domain=" in row and " n=" in row and " name=verdict " not in row]
    absent = [row for row in place_rows if row not in known]
    if absent or not place_rows:
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 V7：基线有 {len(absent)} 行（共 {len(place_rows)} 行）不在第一段的产物里，第一行：{absent[0][:200] if absent else '一行都没有'}\n"
                                 "  → 怎么办：第一段的产物与今天的 bin 对不上，先复跑第一段")
    baseline["verbatim_rows"] = len(place_rows)


# ── r3 档（登记 r3 9.3，第三段）：整份装置输出的读法在判法模块里，这里只留 TV7 比对要的那一个。

def r3_normalised(rows):
    """TV7 比对用：E7RESULT 行去掉耗时字段（登记第六节「elapsed_seconds、E7TIMING 行除外」；E7TIMING 行本来不是 E7RESULT 行）。"""
    return [re.sub(r" elapsed_seconds=\S+", "", row) for row in rows]


def check_the_profile_rows_r4(baseline):
    """登记 r4 9.3 第 6 条第五格：基线要有五个合法域与 KG 各一条 criteria=root_choice_faults 的判决行、KA 一条 criteria=root_choice 的判决行；缺了就是装置没带
    --profile r4，退 7、变体一个不编。（不带 --profile r4 时装置自己退 2，走不到这里，这一格防的是装置不认这一族却照旧跑出别的行。）"""
    found = {parse_row(row).get("domain"): parse_row(row).get("criteria") for row in baseline["rows"] if " name=verdict " in row and " domain=" in row}
    lacking = [domain for domain in R4_ALL_DOMAINS if found.get(domain) != ("root_choice" if domain == "KA" else "root_choice_faults")]
    if lacking:
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --profile r4：基线缺 criteria=root_choice_faults 的判决行，缺 {'、'.join(lacking)} 域\n"
                                        "  → 怎么办：装置要认 --profile r4 --family root_choice_faults（r4 登记 5.1）、驱动调它时要带上；这一次变体一个都没编")


def check_the_gate_rows_r4(setup, baseline):
    """登记 r5 修1.九乙第 4 条：给了 --gate r2 时，基线 family=root_choice_faults 的 name=config 行要带 R2 的闸字段（R4_GATE_R2_CONFIG_FIELD：witness_gate=r2_…）；缺了就是装置没收到
    或不认 --gate r2，退 7、变体一个不编。不给 --gate 时不判。"""
    if setup.gate != "r2":
        return
    field, value = R4_GATE_R2_CONFIG_FIELD
    configs = [parse_row(row) for row in baseline["rows"] if " name=config " in row and " family=root_choice_faults" in row]
    if not configs or any(config.get(field) != value for config in configs):
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --gate r2：基线 family=root_choice_faults 的 name=config 行 {len(configs)} 行，缺 {field}={value}\n"
                                        f"  → 怎么办：装置要认 --profile r4 --family root_choice_faults --gate r2 并在 config 行写 {field}={value}（5-甲），驱动调它时要带上；这一次变体一个都没编")


def check_against_segment_one_product(setup, baseline):
    """作废条款 TV7（r4 9.3 第 3 条与第六节末段）：开头基线的 E7RESULT 行必须与第一段的产物逐行相同（同一份根、同一份源码编出来的）；
    一行都没有按 TV8 不当结果；产物不在，这一项本次未跑（retained_product_not_run）。"""
    path = os.path.join(setup.script_root, setup.registered["segment1_product"])
    if retained_product_not_run(setup, "TV7", [path]):
        baseline["verbatim_rows"] = "not_run"
        return
    if not os.path.exists(path):
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：第一段产物读不到：{path}\n  → 怎么办：产物在脚本所在的仓的 research/results/ 下；这一次不当结果")
    with open(path, encoding="utf-8") as handle:
        product = r3_normalised(result_rows(handle.read()))
    mine = r3_normalised(baseline["rows"])
    if not product or not mine:
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：比对读到 0 行（第一段产物 {len(product)} 行、基线 {len(mine)} 行）\n  → 怎么办：先看两份文件各自有没有 E7RESULT 行")
    if product != mine:
        first = next((index for index, (left, right) in enumerate(zip(product, mine)) if left != right), min(len(product), len(mine)))
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV7：开头基线 {len(mine)} 行与第一段产物 {len(product)} 行不逐行相同，第一处不同在第 {first + 1} 行\n"
                                 f"     产物：{(product[first] if first < len(product) else '（没有）')[:240]}\n     基线：{(mine[first] if first < len(mine) else '（没有）')[:240]}\n"
                                 "  → 怎么办：第一段的产物与今天根上编出的装置对不上；这一次不当结果，先查是谁换了装置或根")
    baseline["verbatim_rows"] = len(mine)


def load_items_r4(setup):
    """r4 的变体清单：基线、9.1 那六个名字按名字选出的行（按今天的表行号升序，起名 tsv-<行号>）。四条里有一条不满足的行不停机：
    记成 failure、那一行记 outcome=not_compared（登记 TH5），别的行照做。弄坏开关 r4itemsmissing：名单少最后一条。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    labels = dict(zip((row for row, _name, _pin in setup.registered["names"]), setup.registered["labels"]))
    if "r4itemsmissing" in BREAK:
        selection = selection[:-1]
    items = [{"name": "baseline", "table": "none", "rows": [], "titles": ["基线（不改）"], "old": None, "new": None, "file": None, "failure": None}]
    for entry in sorted(selection, key=lambda candidate: (candidate["today_row"] is None, candidate["today_row"] or 0, candidate["registered_row"])):
        name = f"tsv-{entry['today_row']}" if entry["today_row"] is not None else f"tsv-registered-{entry['registered_row']}"
        items.append({"name": name, "table": TABLE_PATH, "rows": [entry["today_row"] if entry["today_row"] is not None else entry["registered_row"]],
                      "titles": [entry["name"]], "file": entry["file"], "old": entry["old"], "new": entry["new"], "failure": entry["failure"],
                      "pin": list(entry["registered_tags"]), "registered_row": entry["registered_row"], "function": entry["function"],
                      "label": labels[entry["registered_row"]]})
    return items, texts


def process_item_r4(setup, item, baseline, options):
    """--profile r4 那一套做一个变体（判法模块的 judge_variant_r4）。"""
    return judge_built_variant(setup, item, baseline, options, judge_variant_r4)


# ── 结果文件

def recorded_items(path):
    """产物里有完整一行（complete=yes 收尾）的变体 → 它的行：--recheck 拿来比的那一份产物，与整轮收尾读回这一趟自己写的产物。"""
    recorded = {}
    if not os.path.exists(path):
        return recorded
    with open(path, encoding="utf-8") as handle:
        content = handle.read()
    for line in content.split("\n"):
        if line.startswith("E7RESULT name=kill_bound ") and line.endswith(" complete=yes"):
            recorded.setdefault(parse_row(line)["item"], []).append(line)
    return recorded


def start_file(path, lines):
    """这一趟的产物（或红例账）从头写，先写开头那几行（config 行、账头）：--output 与 --ledger 已经存在的在开跑之前就拒了（refuse_existing_output；
    弄坏开关 outputoverwritten、ledgeroverwritten 下照收，这里整份换掉那一份）；产物与账都不是复用来源（文件头「按项判定」）。"""
    with open(path, "w", encoding="utf-8") as handle:
        for line in lines:
            handle.write(line + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def refuse_existing_output(setup, path, shown, ledger=None, ledger_shown=None):
    """三套整轮的 --output、不给 --profile 那一套的 --ledger 指到已经存在的文件：退 2、不编不跑、不写产物与账，那一份一个字不动
    （--recheck 读已有的产物与账，不走这里）。弄坏开关 outputoverwritten 下 --output 照收、ledgeroverwritten 下 --ledger 照收（start_file 整份换掉那一份）。
    只供自证：SET_OUTPUT_ASIDE_VARIABLE 设成 1（且是自证起的那一趟）时，先把已经存在的产物与账挪成 <名>.previous，再判（同一条命令重交几趟的那几格）。"""
    if setup.selftest and os.environ.get(SET_OUTPUT_ASIDE_VARIABLE) == "1":
        for existing in (path, ledger):
            if existing is not None and os.path.exists(existing):
                os.replace(existing, existing + ".previous")
    for option, existing, existing_shown, broken in (("--output", path, shown, "outputoverwritten"), ("--ledger", ledger, ledger_shown, "ledgeroverwritten")):
        if existing is not None and os.path.exists(existing) and broken not in BREAK:
            raise Stop(EXIT_USAGE, f"  ✗ {option} {existing_shown} 已经存在：这一趟不编不跑、不写产物，那一份没动\n  → 怎么办：换一个路径，或先挪走那一份")


def append_lines(path, lines):
    with open(path, "a", encoding="utf-8") as handle:
        for line in lines:
            handle.write(line + "\n")
        handle.flush()
        os.fsync(handle.fileno())


# ── 按项判定（文件头「按项判定」）：每个变体一项，经 long_run_item_verdicts.py 的 plan / record / finish 存进 prune-server 的 apparatus-items 族

ITEM_VERDICTS_SCRIPT = os.path.join(SCRIPT_DIRECTORY, "long_run_item_verdicts.py")
SCRIPT_KEY = "scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh"   # 按项判定 plan 与复用格认的脚本名（弄坏开关 judgewholedriver 下判法也写它）
LIBRARY_KIND = BIN_NAME                                                            # 判定库的种类：装置的 bin 名，三套整轮共用，档写在 line: 里
APPARATUS_ENTRY = DEVICE_SOURCE_PATH + "::main"
ITEM_STATE_DIRECTORY_NAME = "item-verdicts"   # 这一趟的状态目录（计划、复用项取回的值、交给 record 的值），在 <草稿目录>/kill-bounds/ 下，跑完删
RERUN_PLACEHOLDER = 'rerun="<复跑命令>"'      # 存进库的产物行里复跑命令那一段换成它，取回时照这一趟的 --output、--ledger 填回
# 改坏那一处碰到模块集合的变体拒算：改坏之后的模块集合与没改坏的闭包不同，变异行原文进键罩不住
MODULE_SET_TOKENS = ("mod ", "#[path", "include_str!", "include_bytes!", "include!")
# 产物行与红例账行里写到的这一项的字段：都进这一项的键（line:item=）
ITEM_KEY_FIELDS = ("name", "table", "rows", "titles", "file", "old", "new", "failure", "registered_row", "label", "pin", "function")
EXIT_LIBRARY_UNAVAILABLE = 75
KILL_AFTER_VARIABLE = "E165_KILL_BOUNDS_SELFTEST_KILL_AFTER"   # 只供自证：判完这一项、record 之后整个进程被 SIGKILL（文件头「按项判定」）
SET_OUTPUT_ASIDE_VARIABLE = "E165_KILL_BOUNDS_SELFTEST_SET_OUTPUT_ASIDE"   # 只供自证：已经存在的 --output 与 --ledger 先挪走再判（refuse_existing_output）
# 自证起的那几趟的判法（文件头「按项判定」）：自证把这份文件写进假树，judge: 指它、不带 judge-root:，判法闭包在假树上算
SELFTEST_JUDGE_FILE = "scripts/experiment-apparatus/e165_kill_bounds_selftest_judge.py"
SELFTEST_JUDGE_ENTRY = SELFTEST_JUDGE_FILE + "::judge_variant"
SELFTEST_JUDGE_RESULT = '    return "red when a criterion differs from the baseline"\n'   # 自证「改了假树里的判法」那一格改的就是这一行
SELFTEST_JUDGE_TEXT = ('"""E165 第二段驱动自证的假判法：自证起的那几趟，按项判定的 judge: 指这里（判法闭包在假树上算，不在脚本所在的仓上收整份驱动）。"""\n\n\n'
                       "def judge_variant():\n" + SELFTEST_JUDGE_RESULT)


# 真跑的按项判定请求里的判法（文件头「按项判定」）：判法模块里这一套判一个变体的函数与写基线那一行的函数，judge: 逐个指它们
JUDGE_MODULE_KEY = "scripts/experiment-apparatus/e165_replay_mutation_kill_bounds_variant_judgement.py"
JUDGE_FUNCTIONS = {"r2": (judge_variant, baseline_item_line), "r3": (judge_variant_r3, baseline_item_line_r3), "r4": (judge_variant_r4, baseline_item_line_r4)}


def judge_tokens(setup):
    """按项判定请求里的判法：真跑是判法模块里这一套的 JUDGE_FUNCTIONS，判法闭包在脚本所在的仓上算（judge-root:）；
    自证起的那几趟换成假树里的 SELFTEST_JUDGE_ENTRY、不带 judge-root:。
    弄坏开关 judgewholedriver：真跑的判法写整份驱动（judge:<本脚本>）；judgeoffthetree：自证的判法另带 judge-root:..，
    闭包换到假树的上一层去算（那里没有假判法，每一项拒算）。"""
    if not setup.selftest:
        if "judgewholedriver" in BREAK:
            return [f"judge:{SCRIPT_KEY}", f"judge-root:{DEFAULT_ROOT}"]
        return [f"judge:{JUDGE_MODULE_KEY}::{function.__name__}" for function in JUDGE_FUNCTIONS[setup.profile]] + [f"judge-root:{DEFAULT_ROOT}"]
    return [f"judge:{SELFTEST_JUDGE_ENTRY}"] + (["judge-root:.."] if "judgeoffthetree" in BREAK else [])


def touched_module_set_tokens(item):
    text = (item.get("old") or "") + (item.get("new") or "")
    return [token.strip() for token in MODULE_SET_TOKENS if token in text]


def item_verdicts_command(arguments):
    """起 long_run_item_verdicts.py 的一条子命令（plan、record、finish、自证用的几条）：交 (退出码, 合起来的输出)。"""
    completed = subprocess.run([sys.executable, ITEM_VERDICTS_SCRIPT, *arguments], stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    return completed.returncode, completed.stdout


def item_verdicts_failure(step, code, output):
    """按项判定那一步没成：75（prune-server 不可用）与 2（用法错）照交，别的按 1 交。"""
    return Stop(code if code in (EXIT_LIBRARY_UNAVAILABLE, EXIT_USAGE) else EXIT_INCOMPLETE,
                f"{output.rstrip()}\n  ✗ 按项判定的 {step} 没成（long_run_item_verdicts.py 退 {code}）\n"
                "  → 怎么办：退 75 是 prune-server 不可用：python3 prune-server/control check 看服务，修好之后挪走这一趟写了一半的产物（有红例账的连同它；--output、--ledger 已经存在的不收）、重交同一条命令（报给主 agent）；别的照上面它打的出路改")


def item_requests_text(setup, planned, retained_paths):
    """请求正文（long_run_item_verdicts.py 文件头「请求文件」）：每项一行，基线之外的每项 @after 基线；留存产物经 @input 进基线的键。"""
    registered = hashlib.sha256(json.dumps(setup.registered, ensure_ascii=False, sort_keys=True).encode("utf-8")).hexdigest()
    common = [APPARATUS_ENTRY, "profile:release", *judge_tokens(setup),
              f"line:mode=profile {setup.profile} pins {setup.pins or 'none'} gate {setup.gate or 'none'} bounds {' '.join(setup.bounds_arguments())}",
              f"line:registered={registered}"]
    rows = []
    for item in planned:
        described = json.dumps({field: item.get(field) for field in ITEM_KEY_FIELDS}, ensure_ascii=False, sort_keys=True)
        rows.append("\t".join([item["name"], *common, "line:item=" + described]))
        if item["name"] == "baseline":
            rows += [f"@input\tbaseline\t{os.path.basename(path)}\t{path}" for path in retained_paths]
        else:
            rows.append(f"@after\t{item['name']}\tbaseline")
    return "".join(row + "\n" for row in rows)


class ItemVerdicts:
    """一趟整轮的按项判定（文件头「按项判定」）：构造时 plan（算键、查库），库里复用的项取回它的产物行与红例账行；现判的项判完 record，收尾 finish。
    基线不管点没点名都在计划里（变体 @after 它）。改坏碰到模块集合的变体不进计划：照跑、不复用、不入库。"""

    def __init__(self, setup, options, items, wanted, rerun_of, retained_paths):
        self.setup, self.rerun_of = setup, rerun_of
        self.by_name = {item["name"]: item for item in items}
        self.wanted_names = [item["name"] for item in wanted]
        self.directory = os.path.join(options["work"], ITEM_STATE_DIRECTORY_NAME)
        shutil.rmtree(self.directory, ignore_errors=True)
        os.makedirs(os.path.join(self.directory, "values"))
        atexit.register(shutil.rmtree, self.directory, True)
        self.state = os.path.join(self.directory, "plan")
        self.module_set = {item["name"]: touched_module_set_tokens(item) for item in wanted if touched_module_set_tokens(item)}
        planned = [item for item in items if item["name"] == "baseline" or (item["name"] in self.wanted_names and item["name"] not in self.module_set)]
        requests = os.path.join(self.directory, "requests.tsv")
        with open(requests, "w", encoding="utf-8") as handle:
            handle.write(item_requests_text(setup, planned, retained_paths))
        code, output = item_verdicts_command(["plan", self.state, "--script", SCRIPT_KEY, "--kind", LIBRARY_KIND, "--root", setup.root, "--requests", requests])
        print(output, end="", flush=True)
        if code != 0:
            raise item_verdicts_failure("plan", code, output)
        self.decisions = {}
        with open(os.path.join(self.state, "decisions.tsv"), encoding="utf-8") as handle:
            for line in handle:
                name, _tab, decision = line.rstrip("\n").partition("\t")
                self.decisions[name] = decision
        self.reused = {}
        if "reuseignored" not in BREAK:   # 弄坏开关：plan 答 reuse 的也照现判
            self.reused = {name: self.reused_record(name) for name, decision in self.decisions.items() if decision == "reuse"}
        self.to_run = [item for item in wanted if item["name"] not in self.reused]
        # 没点名的基线库里没有可复用的（要现判、拒算）：照跑、照记，强制现判抽中它时也是这一支
        self.baseline_pending = "baseline" not in self.reused and "baseline" not in self.wanted_names
        self.deferred = []
        self.summary = "取不出"

    def rerun_text(self, name):
        return f'rerun="{self.rerun_of(self.by_name[name])}"'

    def reused_record(self, name):
        """库里复用的一项：取回的值是 {"lines", "ledger"}，复跑命令那一段照这一趟填回。"""
        try:
            with open(os.path.join(self.state, "outputs", name + ".out"), encoding="utf-8") as handle:
                value = json.load(handle)
            lines, ledger = list(value["lines"]), list(value["ledger"])
        except (OSError, ValueError, KeyError, TypeError) as error:
            raise Stop(EXIT_INCOMPLETE, f"  ✗ 判定库里 {name} 那一项的值读不出产物行与红例账行（{error}）\n"
                                        "  → 怎么办：这把键的值不是这份驱动写的那一种；查是谁往 apparatus-items 族这个种类里写了别的东西，删掉那一条再起同一条命令") from error
        return {"lines": [line.replace(RERUN_PLACEHOLDER, self.rerun_text(name)) for line in lines], "ledger": ledger, "problem": None, "timing": (0.0, 0.0, 0.0)}

    def value_text(self, name, record):
        """存进库的值：产物行（复跑命令那一段换成占位）与红例账行，键排序的 JSON，同一份结果写出来逐字节相同。"""
        rerun = self.rerun_text(name)
        lines = []
        for line in record["lines"]:
            if line.count(rerun) != 1:
                raise Stop(EXIT_INCOMPLETE, f"  ✗ {name} 的产物行里复跑命令那一段不是恰好一处，存不进判定库：{line[:200]}\n  → 怎么办：看判法模块里这一套的 rerun_command 与 judge_variant 的 tail")
            lines.append(line.replace(rerun, RERUN_PLACEHOLDER))
        return json.dumps({"ledger": record["ledger"], "lines": lines}, ensure_ascii=False, sort_keys=True) + "\n"

    def report_reused(self, item_lines):
        """库里复用的点名项开跑前各打一行（文件头「每项一行与总计」）。"""
        for name in self.wanted_names:
            if name in self.reused:
                item_lines.reused_item(name, self.reused[name]["lines"][0])

    def record(self, name, record):
        """现判或拒算的项判完当场写进库：得出结果的判绿，驱动没得出结果的（record 的 problem）记红。复用的项不调；模块集合拒算的不入库。"""
        if name in self.module_set or name not in self.decisions:
            return
        if "recorddeferred" in BREAK:   # 弄坏开关：判完不当场写，finish 之前一并写
            self.deferred.append((name, record))
        else:
            self.write(name, record)
        if self.setup.selftest and os.environ.get(KILL_AFTER_VARIABLE) == name:
            os.kill(os.getpid(), signal.SIGKILL)

    def write(self, name, record):
        red = record["problem"] is not None
        value = os.path.join(self.directory, "values", name + ".json")
        with open(value, "w", encoding="utf-8") as handle:
            handle.write("" if red else self.value_text(name, record))
        code, output = item_verdicts_command(["record", self.state, name, "1" if red else "0", value] + (["--red"] if red else []))
        if code != 0:
            raise item_verdicts_failure(f"record（项 {name}）", code, output)

    def baseline_ran(self, baseline_line):
        """开头基线跑完：库里复用的基线要与这一次的逐字相同（键相同而基线变了，退 5）；没点名、库里又没有可复用的基线，当场记。"""
        stored = self.reused.get("baseline")
        if stored is not None:
            if stored["lines"][0] != baseline_line:
                raise Stop(EXIT_VOID_V6, f"  ✗ 这一次的基线与判定库里同一把键的基线不同\n     库里的：{stored['lines'][0][:240]}\n     这次：{baseline_line[:240]}\n"
                                         "  → 怎么办：键相同而基线变了，判法或装置读了键之外的输入；这一趟变体一个都没判，交主 agent 查")
        elif self.baseline_pending:
            self.record("baseline", {"lines": [baseline_line], "ledger": [], "problem": None})

    def finish(self):
        """收尾：写 recorddeferred 压下的那几项、起 finish、报模块集合拒算的项；交 True（没红）或 False（复用哨兵红、键中途变了：产物不收尾）。"""
        for name, record in self.deferred:
            self.write(name, record)
        code, output = item_verdicts_command(["finish", self.state])
        print(output, end="", flush=True)
        if code > 1:
            raise item_verdicts_failure("finish", code, output)
        for name, tokens in self.module_set.items():
            print(f"  · {name}：拒算（改坏那一处碰到模块集合：{'、'.join(tokens)}；照跑、不复用、不入库）")
        found = re.findall(r"^LONG_RUN_ITEMS .* 现判 (\d+)、复用 (\d+)、拒算 (\d+)$", output, re.M)
        if found:
            judged, reused, refused = (int(value) for value in found[-1])
            self.summary = f"现判 {judged}、复用 {reused}、拒算 {refused + len(self.module_set)}"
        if code == 1 and "finishignored" not in BREAK:   # 弄坏开关：finish 的红不认
            print(f"  ✗ 按项判定收尾判红（复用哨兵或键在这一趟中途变了，见上面 ✗ 那几行；{self.summary}）：这一趟的产物不收尾")
            print("  → 怎么办：照上面那几行的出路办，挪走这一趟没收尾的产物（有红例账的连同它；--output、--ledger 已经存在的不收），再起同一条命令")
            return False
        return True


def opening_baseline(setup, options, verdicts, to_run, run_checks, line_of, retained_absent):
    """开头基线（文件头「做法」第 3 步）：这一趟有项要跑才编、才跑、才核（库里复用的项不要基线）；交 (基线, 基线那一行)。
    一项都不跑时交一份替身、不编不跑：收尾行要的基线行数取库里基线那一行的 result_rows，留存产物在不在照现查（不在的照报本次未跑）。"""
    if to_run or (verdicts is not None and verdicts.baseline_pending):
        baseline = build_and_run_baseline(setup, options, "baseline")
        run_checks(baseline)
        baseline_line = line_of(baseline)
        if verdicts is not None:
            verdicts.baseline_ran(baseline_line)
        return baseline, baseline_line
    stored = verdicts.reused.get("baseline")
    row_count = int(parse_row(stored["lines"][0]).get("result_rows", "0")) if stored is not None else None
    return {"binary": None, "rows": None, "row_count": row_count, "verbatim_rows": "not_run" if retained_absent() else 0}, None


def judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process, item_lines, write_product):
    """to_run 里的项按 --jobs 并行做（基线那一项就是开头基线那一行）：每项做完当场打它那一行、记进判定库；write_product 时照变体的次序落进这一趟的产物
    （库里复用的项拿库里的行；没得出结果的那一项与它后面的不落，等下一趟）。交 (results, problems)；有项抛 Stop 的，别的项做完之后抛先抛的那一个。"""
    results, problems = {}, []
    lock = threading.Lock()
    flushed = set()

    def flush_in_order():
        for item in items:
            name = item["name"]
            if name in flushed or item not in wanted:
                continue
            record = results.get(name) or (verdicts.reused.get(name) if verdicts is not None else None)
            if record is None or record.get("problem"):
                return
            append_lines(options["output"], record["lines"])
            if record["ledger"]:
                append_lines(options["ledger"], record["ledger"])
            flushed.add(name)

    def work(item):
        if item["name"] == "baseline":
            record = {"lines": [baseline_line], "ledger": [], "problem": None, "timing": (0.0, 0.0, 0.0)}
        else:
            record = process(setup, item, baseline, options)
        with lock:
            results[item["name"]] = record
            options["progress"].finished(item["name"])
            item_lines.judged_item(item["name"], record)
            build_seconds, run_seconds, total_seconds = record["timing"]
            print(f"  {item['name']}：编译 {build_seconds:.1f} 秒，跑 {run_seconds:.1f} 秒，合计 {total_seconds:.1f} 秒"
                  + (f"；没得出结果：{record['problem']}" if record["problem"] else ""), file=sys.stderr)
            with open(os.path.join(options["work"], "progress.log"), "a", encoding="utf-8") as handle:
                handle.write(f"{item['name']}\tbuild={build_seconds:.1f}\trun={run_seconds:.1f}\ttotal={total_seconds:.1f}\tproblem={record['problem']}\n")
            if record["problem"]:
                problems.append(f"{item['name']}：{record['problem']}")
            if verdicts is not None:
                verdicts.record(item["name"], record)
            if write_product and not record["problem"]:
                flush_in_order()

    halted = None
    with concurrent.futures.ThreadPoolExecutor(max_workers=options["jobs"]) as pool:
        item_lines.dispatch(len(to_run))
        futures = [pool.submit(work, item) for item in to_run]
        for future in concurrent.futures.as_completed(futures):
            try:
                future.result()
            except Stop as stop:
                halted = halted or stop
    if halted is not None:
        raise halted
    if write_product:
        with lock:
            flush_in_order()
    return results, problems


LEDGER_HEADER = ("# E165 第二段红例账：每个红的变体在每个红了的域首红的 N 上一个反例状态。制表符分隔，十三列：变体、表行、域、N、状态在枚举次序里的序号、"
                 "怎么不同（counters=加了一的量不同 / outcome_only=只有结局不同）、不同的量（+ 是变体多加的、- 是变体少加的）、"
                 "原实现单条重放与逐态行相同、变体单条重放与逐态行相同、是不是这个变体首红的那一个、单条重放的命令、原实现的逐态行、变体的逐态行")


def check_the_profile_rows_r3(baseline):
    """登记 r3 9.3 第 6 条第四格：基线要有每个域一条 criteria=root_choice 的判决行；缺了就是装置没带 --profile r3，退 7、变体一个不编。
    （r3 档才有 --family root_choice；不带 --profile r3 时装置自己退 2，走不到这里，这一格防的是装置不认这一族却照旧跑出别的行。）"""
    found = {parse_row(row).get("domain") for row in baseline["rows"] if " name=verdict " in row and " criteria=root_choice" in row and " domain=" in row}
    lacking = [domain for domain in R3_ALL_DOMAINS if domain not in found]
    if lacking:
        raise Stop(EXIT_NOT_PROFILE_R2, f"  ✗ 装置没带 --profile r3：基线缺 criteria=root_choice 的判决行，缺 {'、'.join(lacking)} 域\n"
                                        "  → 怎么办：装置要认 --profile r3 --family root_choice（r3 登记 5.1）、驱动调它时要带上；这一次变体一个都没编")


def check_against_segment_two_product(setup, baseline):
    """作废条款 TV7（r3 9.3 第 3 条与第六节末段）：开头基线的 E7RESULT 行必须与第二段产物逐行相同（同一份根、同一份源码编出来的，耗时字段除外）；
    一行都没有按 TV8 不当结果；产物不在，这一项本次未跑（retained_product_not_run）。"""
    path = os.path.join(setup.script_root, setup.registered["segment2_product"])
    if retained_product_not_run(setup, "TV7", [path]):
        baseline["verbatim_rows"] = "not_run"
        return
    if not os.path.exists(path):
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：第二段产物读不到：{path}\n  → 怎么办：产物在脚本所在的仓的 research/results/ 下；这一次不当结果")
    with open(path, encoding="utf-8") as handle:
        product = r3_normalised(result_rows(handle.read()))
    mine = r3_normalised(baseline["rows"])
    if not product or not mine:
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV8：比对读到 0 行（第二段产物 {len(product)} 行、基线 {len(mine)} 行）\n  → 怎么办：先看两份文件各自有没有 E7RESULT 行")
    if product != mine:
        first = next((index for index, (left, right) in enumerate(zip(product, mine)) if left != right), min(len(product), len(mine)))
        raise Stop(EXIT_VOID_V7, f"  ✗ 作废条款 TV7：开头基线 {len(mine)} 行与第二段产物 {len(product)} 行不逐行相同，第一处不同在第 {first + 1} 行\n"
                                 f"     产物：{(product[first] if first < len(product) else '（没有）')[:240]}\n     基线：{(mine[first] if first < len(mine) else '（没有）')[:240]}\n"
                                 "  → 怎么办：第二段的产物与今天冻结根上编出的装置对不上；这一次不当结果，先查是谁换了装置或根")
    baseline["verbatim_rows"] = len(mine)


def load_items_r3(setup):
    """r3 的变体清单：基线、9.1 那六个名字按名字选出的行（按 tsv 行号升序，起名 tsv-<行号>）。四条里有一条不满足的行不停机：
    记成 failure、那一行记 outcome=not_compared（登记 TH5），别的行照做。弄坏开关 r3itemsmissing：名单少最后一条。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    if "r3itemsmissing" in BREAK:
        selection = selection[:-1]
    items = [{"name": "baseline", "table": "none", "rows": [], "titles": ["基线（不改）"], "old": None, "new": None, "file": None, "failure": None}]
    for entry in sorted(selection, key=lambda candidate: (candidate["today_row"] is None, candidate["today_row"] or 0, candidate["registered_row"])):
        name = f"tsv-{entry['today_row']}" if entry["today_row"] is not None else f"tsv-registered-{entry['registered_row']}"
        items.append({"name": name, "table": TABLE_PATH, "rows": [entry["today_row"] if entry["today_row"] is not None else entry["registered_row"]],
                      "titles": [entry["name"]], "file": entry["file"], "old": entry["old"], "new": entry["new"], "failure": entry["failure"],
                      "pin": list(entry["registered_tags"]), "registered_row": entry["registered_row"], "function": entry["function"]})
    return items, texts


def process_item_r3(setup, item, baseline, options):
    """--profile r3 那一套做一个变体（判法模块的 judge_variant_r3）。"""
    return judge_built_variant(setup, item, baseline, options, judge_variant_r3)


def not_run_suffix(setup):
    """收尾那句后面接的：这一趟本次未跑的比对逐项列出（留存产物不在）；没有交空串。"""
    return f"；本次未跑 {len(setup.not_run)} 项：{'；'.join(setup.not_run)}" if setup.not_run else ""


PIN_LINE_LIMIT = 40   # 钉红例那一步没过时，原样转出它的输出最多几行


def pin_red_examples(setup, options):
    """钉红例（文件头「钉红例」）：这一对账（options["ledger"]）与 kill-bounds 产物（options["output"]）写完之后当场钉进钉表，再核钉全：
    先照 e165-pin-red-examples.py 的追加用法（--kill-bounds <产物> <账>），再 --check 显式给这一对；钉表与变异表取脚本所在的仓（setup.script_root）。
    有一步退非 0 就 raise Stop(EXIT_PIN_INCOMPLETE)，原样转出它的输出；都过交成功那句要报的「钉进 N 行、核了 M 行」。
    只供自证：E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND 设了（且是自证起的那一趟）就不追加，造「账写完了、钉表没钉」。
    弄坏开关 pincheckskipped 下不跑 --check（当场不核钉全）。"""
    ledger, product = options["ledger"], options["output"]
    steps = []
    if not (setup.selftest and os.environ.get("E165_KILL_BOUNDS_SELFTEST_SKIP_PIN_APPEND")):
        steps.append(("追加", ["--kill-bounds", product, ledger]))
    if "pincheckskipped" not in BREAK:
        steps.append(("核钉全（--check）", ["--check", "--kill-bounds", product, ledger]))
    said = {}
    for label, arguments in steps:
        command = [sys.executable, PIN_SCRIPT, "--root", setup.script_root, *arguments]
        environment = dict(os.environ)
        environment.pop("E165_PIN_RED_EXAMPLES_BREAK", None)   # 那一份的弄坏开关只给它自己的自证用，不从这里带进去
        completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, env=environment)
        lines = [line for line in completed.stdout.split("\n") if line.strip()]
        if completed.returncode != 0:
            shown = "\n".join("     " + line for line in lines[-PIN_LINE_LIMIT:])
            raise Stop(EXIT_PIN_INCOMPLETE, f"  ✗ 钉红例：{label}没过（{' '.join(command[1:])} 退 {completed.returncode}），这一趟的红例没钉全：\n{shown}\n"
                                            "  → 怎么办：照上面它打的出路办（同档冲突、按名字找不到变异、配对不上，写法在 scripts/experiment-apparatus/e165-pin-red-examples.py 文件头），"
                                            "改好之后挪走这一趟的产物与账（--output、--ledger 已经存在的不收）、重跑同一条命令：判过的变体从判定库复用、不重跑，产物与账照样从头写、照样钉")
        said[label] = next((line for line in reversed(lines) if line.lstrip().startswith("✓")), lines[-1] if lines else "")
    appended = said.get("追加", "")
    pinned_match = re.search(r"钉进 (\d+) 行", appended)
    pinned = pinned_match.group(1) if pinned_match else ("0" if "没有要新钉的" in appended else "不追加（自证）" if not appended else "取不出")
    checked_line = said.get("核钉全（--check）")
    checked_match = re.search(r"份红例账 (\d+) 行都钉在", checked_line or "")
    checked = checked_match.group(1) if checked_match else ("没核（弄坏开关 pincheckskipped）" if checked_line is None else "取不出")
    return f"钉红例：钉进 {pinned} 行、核了 {checked} 行（账 {options.get('ledger_shown', ledger)} 配产物 {options.get('output_shown', product)}）"


def main_run(setup, options, item_names, recheck):
    check_the_registered_inputs(setup)
    items, selected_rows, texts = load_items(setup)
    check_the_anchors(items, texts)
    check_the_custom_keys(setup, items)
    by_name = {item["name"]: item for item in items}
    for name in item_names:
        if name not in by_name:
            raise unknown_item(name, list(by_name), "--list-items")
    wanted = [by_name[name] for name in item_names] if item_names else items
    retained = [os.path.join(setup.research_root, product) for product in setup.registered["segment1_products"]]
    # --recheck 只拿新跑的行与给的那一份产物比，不读不写判定库、不写产物（文件头「用法」）
    recorded = recorded_items(options["output"]) if recheck else {}
    verdicts = None if recheck else ItemVerdicts(setup, options, items, wanted, lambda item: rerun_command(item, options), retained)
    to_run = list(wanted) if recheck else verdicts.to_run
    status_at_start = subprocess.run(["git", "-C", setup.root, "status", "--porcelain", "crates/"], stdout=subprocess.PIPE, text=True).stdout
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；判定库里复用 {len(wanted) - len(to_run)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")
    item_lines = ItemLines()   # 文件头「每项一行与总计」：库里复用的那几项开跑前各打一行
    if verdicts is not None:
        verdicts.report_reused(item_lines)

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()   # 守护线程：收尾基线跑完就停；中途 Stop 退出的，进程退出时随之结束

    def run_checks(baseline):
        check_the_profile_rows(baseline)
        check_against_segment_one(setup, baseline)

    baseline, baseline_line = opening_baseline(setup, options, verdicts, to_run, run_checks,
                                               lambda baseline: baseline_item_line(by_name["baseline"], baseline, options),
                                               lambda: retained_product_not_run(setup, "V7", retained))
    if not recheck:
        start_file(options["output"], [
            f"E7RESULT name=config experiment=E165 segment=2 registration=research/prompts/e165-r2-prereg.md "
            f"device_arguments={'+'.join([] if 'noprofile' in BREAK else DEVICE_PROFILE_ARGUMENTS) or 'none'} "
            f"bounds=segment2 items={len(items)} table_rows={len(selected_rows)} "
            f"custom_rows={sum(1 for item in items if item['table'] == CUSTOM_TABLE_PATH)} recovery_sha256={sha_at_start[RECOVERY_PATH]} "
            f"lib_sha256={sha_at_start[LIB_PATH]} widening=none{pins_config_text(setup)}"])
        start_file(options["ledger"], [LEDGER_HEADER])

    started = time.monotonic()
    results, problems = judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process_item, item_lines, not recheck)
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；仓里的 crates/ 没变。开头基线没跑的（一项都不用现判）收尾基线也不跑，只核仓里的 crates/。
    if baseline["binary"] is not None:
        baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
        options["progress"].stop()
        same_baseline = baseline_at_end["rows"] == baseline["rows"]
        restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
        restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    else:
        options["progress"].stop()
        same_baseline = restored_start = restored_end = True
    status_at_end = subprocess.run(["git", "-C", setup.root, "status", "--porcelain", "crates/"], stdout=subprocess.PIPE, text=True).stdout
    sha_at_end = source_hashes(setup)
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if (not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"]
            or status_at_start != status_at_end or not (restored_start and restored_end)):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 V6：收尾基线与开头逐字相同={same_baseline}；仓里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… 结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；"
                                 f"`git status --porcelain crates/` 前后相同={status_at_start == status_at_end}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了 crates/，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同" if baseline["binary"] is not None else "  这一趟没有要现判的项，开头与收尾的基线都没编没跑；仓里 recovery.rs、lib.rs 与 crates/ 前后相同")

    if recheck:
        different = 0
        for item in wanted:
            record = results.get(item["name"])
            fresh = record["lines"] if record else []
            same = fresh == recorded.get(item["name"], [])
            different += 0 if same else 1
            print(f"  {'✓' if same else '✗'} {item['name']}：新跑的 {len(fresh)} 行与产物里的 {len(recorded.get(item['name'], []))} 行{'逐字相同' if same else '不同'}")
            if not same:
                for line in fresh:
                    print(f"     新：{line}")
        if different:
            print("  → 怎么办：bin、recovery.rs 或变异表变过；看上面新跑的行与产物里那一行差在哪")
        return EXIT_INCOMPLETE if different or problems else EXIT_OK

    if not verdicts.finish():
        return EXIT_INCOMPLETE
    recorded = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，挪走这一趟的产物与红例账（--output、--ledger 已经存在的不收），再重跑同一条命令（判过的从判定库复用，只现判没判绿的）")
    if missing:
        print(f"  这一趟的产物里还有 {len(missing)} 个变体没有行：{' '.join(missing)}；挪走这一趟的产物与红例账（--output、--ledger 已经存在的不收）、不带 --item 重跑同一条命令补齐（判过的从判定库复用）；按项判定 {verdicts.summary}")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes = {}
    for item in items:
        if item["name"] != "baseline":
            outcome = parse_row(recorded[item["name"]][0])["outcome"]
            outcomes[outcome] = outcomes.get(outcome, 0) + 1
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={baseline['row_count']} "
        f"repository_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} repository_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"repository_lib_sha256_at_start={sha_at_start[LIB_PATH]} repository_lib_sha256_at_end={sha_at_end[LIB_PATH]} crates_git_status_unchanged=true copies_restored=true",
        f"E7RESULT name=verdict scope=segment2 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_implementation_anchor={outcomes.get('red_only_by_implementation_anchor', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} ledger_sha256={sha256_of_file(options['ledger'])}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"])
    pin_text = pin_red_examples(setup, options)
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}；{pin_text}；按项判定 {verdicts.summary}{not_run_suffix(setup)}")
    return EXIT_OK


def main_run_r3(setup, options, item_names):
    """r3 模式整轮（登记 r3 9.3）。做法同 main_run：开工核对、开头基线、每个变体一份副本、收尾基线；差在：基线要带 criteria=root_choice 判决行（退 7）、
    要与第二段产物逐行相同（退 6）、不落红例账、选行不满足的记 not_compared、KA 钉值不符整轮退 1（TF8）。"""
    check_the_registered_inputs(setup)
    items, texts = load_items_r3(setup)
    check_the_anchors([item for item in items if item["failure"] is None], texts)
    by_name = {item["name"]: item for item in items}
    for name in item_names:
        if name not in by_name:
            raise unknown_item(name, list(by_name), "--profile r3 --list-items")
    wanted = [by_name[name] for name in item_names] if item_names else items
    retained = os.path.join(setup.script_root, setup.registered["segment2_product"])
    verdicts = ItemVerdicts(setup, options, items, wanted, lambda item: rerun_command_r3(item, options), [retained])
    to_run = verdicts.to_run
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；判定库里复用 {len(wanted) - len(to_run)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")
    item_lines = ItemLines()   # 文件头「每项一行与总计」：库里复用的那几项开跑前各打一行
    verdicts.report_reused(item_lines)

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()

    def run_checks(baseline):
        check_the_profile_rows_r3(baseline)
        check_against_segment_two_product(setup, baseline)

    baseline, baseline_line = opening_baseline(setup, options, verdicts, to_run, run_checks,
                                               lambda baseline: baseline_item_line_r3(by_name["baseline"], baseline, options),
                                               lambda: retained_product_not_run(setup, "TV7", [retained]))
    start_file(options["output"], [
        f"E7RESULT name=config experiment=E165 segment=3 profile=r3 registration=research/prompts/e165-r3-prereg.md "
        f"device_arguments={'+'.join(([] if 'r3noprofile' in BREAK else R3_DEVICE_PROFILE_ARGUMENTS) + R3_BOUNDS_ARGUMENTS) or 'none'} "
        f"items={len(items)} table_rows={len(items) - 1} recovery_sha256={sha_at_start[RECOVERY_PATH]} lib_sha256={sha_at_start[LIB_PATH]} "
        f"table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} widening=none{pins_config_text(setup)}"])

    started = time.monotonic()
    _results, problems = judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process_item_r3, item_lines, True)
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；两份源文件与变异表的 sha256 开头结尾相同（冻结根不是 git 仓，不看 git status）。
    # 开头基线没跑的（一项都不用现判）收尾基线也不跑，只核根里的两份源文件与变异表。
    if baseline["binary"] is not None:
        baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
        options["progress"].stop()
        same_baseline = baseline_at_end["rows"] == baseline["rows"]
        restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
        restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    else:
        options["progress"].stop()
        same_baseline = restored_start = restored_end = True
    sha_at_end = source_hashes(setup)
    table_unchanged = sha256_of_file(os.path.join(setup.root, TABLE_PATH)) == setup.registered["table_sha256"] if setup.registered.get("table_sha256") else True
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"] or not table_unchanged or not (restored_start and restored_end):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 TV9：收尾基线与开头逐字相同={same_baseline}；根里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… "
                                 f"结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；变异表没变={table_unchanged}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了根，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同" if baseline["binary"] is not None else "  这一趟没有要现判的项，开头与收尾的基线都没编没跑；根里 recovery.rs、lib.rs 与变异表前后相同")

    if not verdicts.finish():
        return EXIT_INCOMPLETE
    recorded = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，挪走这一趟的产物（--output 已经存在的不收），再重跑同一条命令（判过的从判定库复用，只现判没判绿的）")
    if missing:
        print(f"  这一趟的产物里还有 {len(missing)} 个变体没有行：{' '.join(missing)}；挪走这一趟的产物（--output 已经存在的不收）、不带 --item 重跑同一条命令补齐（判过的从判定库复用）；按项判定 {verdicts.summary}")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes, pin_mismatches = {}, []
    for item in items:
        if item["name"] != "baseline":
            fields = parse_row(recorded[item["name"]][0])
            outcomes[fields["outcome"]] = outcomes.get(fields["outcome"], 0) + 1
            if fields.get("ka_pin") == "mismatch":
                pin_mismatches.append(item["name"])
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={baseline['row_count']} "
        f"root_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} root_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"root_lib_sha256_at_start={sha_at_start[LIB_PATH]} root_lib_sha256_at_end={sha_at_end[LIB_PATH]} copies_restored=true",
        f"E7RESULT name=verdict scope=segment3 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_kj0_or_kj3={outcomes.get('red_only_by_kj0_or_kj3', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} not_compared={outcomes.get('not_compared', 0)} "
        f"ka_pin_mismatches={len(pin_mismatches)} baseline_equals_segment2_product={retained_comparison_text(baseline, 'true')}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"])
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}；按项判定 {verdicts.summary}{not_run_suffix(setup)}")
    if pin_mismatches:
        print(f"  ✗ KA 钉值对不上（TF8）：{' '.join(pin_mismatches)}；这几个变体的 K 族结果不能读成「没有取样点」，交主 agent；别的变体照判")
        print("     → 怎么办：把这几个变体的 KA 读数与跑前登记 TF8 钉的值并排交主 agent，由主 agent 判是登记钉错还是装置算错")
    if outcomes.get("not_compared", 0):
        print("  ✗ 有变体选行不满足（TH5）记了 not_compared，交主 agent；别的变体照判")
    return EXIT_INCOMPLETE if pin_mismatches or outcomes.get("not_compared", 0) else EXIT_OK


def main_run_r4(setup, options, item_names):
    """r4 模式整轮（登记 r4 9.3）。做法同 main_run_r3：开工核对、开头基线、每个变体一份副本、收尾基线；差在：基线要带 criteria=root_choice_faults 判决行（退 7）、
    要与第一段产物逐行相同（退 6）、不落红例账、选行不满足的记 not_compared、KA 或 KG 钉值不符整轮退 1（TF7）。"""
    check_the_registered_inputs(setup)
    items, texts = load_items_r4(setup)
    check_the_anchors([item for item in items if item["failure"] is None], texts)
    by_name = {item["name"]: item for item in items}
    for name in item_names:
        if name not in by_name:
            raise unknown_item(name, list(by_name), "--profile r4 --list-items")
    wanted = [by_name[name] for name in item_names] if item_names else items
    retained = os.path.join(setup.script_root, setup.registered["segment1_product"])
    verdicts = ItemVerdicts(setup, options, items, wanted, lambda item: rerun_command_r4(item, options), [retained])
    to_run = verdicts.to_run
    sha_at_start = source_hashes(setup)
    print(f"  变体 {len(items)} 个；点名 {len(wanted)} 个；判定库里复用 {len(wanted) - len(to_run)} 个；"
          f"这一次要跑 {len(to_run)} 个：{' '.join(item['name'] for item in to_run) or '无'}")
    item_lines = ItemLines()   # 文件头「每项一行与总计」：库里复用的那几项开跑前各打一行
    verdicts.report_reused(item_lines)

    options["progress"] = Progress(len(to_run), progress_interval(setup))
    options["progress"].start()

    def run_checks(baseline):
        check_the_profile_rows_r4(baseline)
        check_the_gate_rows_r4(setup, baseline)
        check_against_segment_one_product(setup, baseline)

    baseline, baseline_line = opening_baseline(setup, options, verdicts, to_run, run_checks,
                                               lambda baseline: baseline_item_line_r4(setup, by_name["baseline"], baseline, options),
                                               lambda: retained_product_not_run(setup, "TV7", [retained]))
    start_file(options["output"], [
        f"E7RESULT name=config experiment=E165 segment=2 profile=r4 registration=research/prompts/e165-r4-prereg.md "
        f"device_arguments={'+'.join(([] if 'r4noprofile' in BREAK else R4_DEVICE_PROFILE_ARGUMENTS) + setup.bounds_arguments()) or 'none'} "
        f"items={len(items)} table_rows={len(items) - 1} recovery_sha256={sha_at_start[RECOVERY_PATH]} lib_sha256={sha_at_start[LIB_PATH]} "
        f"table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} widening=none{pins_config_text(setup)}"])

    started = time.monotonic()
    _results, problems = judge_items_in_parallel(setup, options, items, wanted, to_run, verdicts, baseline, baseline_line, process_item_r4, item_lines, True)
    variants_seconds = time.monotonic() - started

    # 收尾：重建一份不改的副本、重编重跑基线，必须与开头逐字相同；两份源文件与变异表的 sha256 开头结尾相同（冻结根不是 git 仓，不看 git status）。
    # 开头基线没跑的（一项都不用现判）收尾基线也不跑，只核根里的两份源文件与变异表。
    if baseline["binary"] is not None:
        baseline_at_end = build_and_run_baseline(setup, options, "baseline-at-end")
        options["progress"].stop()
        same_baseline = baseline_at_end["rows"] == baseline["rows"]
        restored_start = restore_and_remove(setup, baseline["directory"], keep=False)
        restored_end = restore_and_remove(setup, baseline_at_end["directory"], keep=False)
    else:
        options["progress"].stop()
        same_baseline = restored_start = restored_end = True
    sha_at_end = source_hashes(setup)
    table_unchanged = sha256_of_file(os.path.join(setup.root, TABLE_PATH)) == setup.registered["table_sha256"] if setup.registered.get("table_sha256") else True
    print(f"  这一次 {len(to_run)} 个变体，挂钟 {variants_seconds:.1f} 秒（不含开头与收尾的基线）", file=sys.stderr)
    if not same_baseline or sha_at_start != sha_at_end or sha_at_end != setup.registered["files_sha256"] or not table_unchanged or not (restored_start and restored_end):
        raise Stop(EXIT_VOID_V6, f"  ✗ 作废条款 TV9：收尾基线与开头逐字相同={same_baseline}；根里 recovery.rs 与 lib.rs 的 sha256 开头 {sha_at_start[RECOVERY_PATH][:16]}…/{sha_at_start[LIB_PATH][:16]}… "
                                 f"结尾 {sha_at_end[RECOVERY_PATH][:16]}…/{sha_at_end[LIB_PATH][:16]}…；变异表没变={table_unchanged}；基线副本还原={restored_start and restored_end}\n"
                                 "  → 怎么办：这一次落的行不算数；查是谁在跑的时候改了根，换一个 --output 重跑")
    print("已还原，基线与开头逐字相同" if baseline["binary"] is not None else "  这一趟没有要现判的项，开头与收尾的基线都没编没跑；根里 recovery.rs、lib.rs 与变异表前后相同")

    if not verdicts.finish():
        return EXIT_INCOMPLETE
    recorded = recorded_items(options["output"])
    missing = [item["name"] for item in items if item["name"] not in recorded]
    for problem in problems:
        print(f"  ✗ {problem}")
    if problems:
        print("  → 怎么办：没得出结果的那几个照上面各自的原因处理，挪走这一趟的产物（--output 已经存在的不收），再重跑同一条命令（判过的从判定库复用，只现判没判绿的）")
    if missing:
        print(f"  这一趟的产物里还有 {len(missing)} 个变体没有行：{' '.join(missing)}；挪走这一趟的产物（--output 已经存在的不收）、不带 --item 重跑同一条命令补齐（判过的从判定库复用）；按项判定 {verdicts.summary}")
        return EXIT_INCOMPLETE if problems else EXIT_OK
    outcomes, pin_mismatches, kg_mismatches = {}, [], []
    for item in items:
        if item["name"] != "baseline":
            fields = parse_row(recorded[item["name"]][0])
            outcomes[fields["outcome"]] = outcomes.get(fields["outcome"], 0) + 1
            if fields.get("ka_pin") == "mismatch":
                pin_mismatches.append(item["name"])
            if fields.get("kg_pin") == "mismatch":
                kg_mismatches.append(item["name"])
    closing = [
        f"E7RESULT name=restored text=已还原，基线与开头逐字相同 baseline_result_rows={baseline['row_count']} "
        f"root_recovery_sha256_at_start={sha_at_start[RECOVERY_PATH]} root_recovery_sha256_at_end={sha_at_end[RECOVERY_PATH]} "
        f"root_lib_sha256_at_start={sha_at_start[LIB_PATH]} root_lib_sha256_at_end={sha_at_end[LIB_PATH]} copies_restored=true",
        f"E7RESULT name=verdict scope=segment2 variants={len(items)} variants_excluding_baseline={len(items) - 1} "
        f"red={outcomes.get('red', 0)} red_only_by_kj0_or_kj3={outcomes.get('red_only_by_kj0_or_kj3', 0)} "
        f"not_red_within_bounds={outcomes.get('not_red_within_bounds', 0)} invalid={outcomes.get('invalid', 0)} "
        f"run_did_not_finish={outcomes.get('run_did_not_finish', 0)} not_compared={outcomes.get('not_compared', 0)} "
        f"ka_pin_mismatches={len(pin_mismatches)} kg_pin_mismatches={len(kg_mismatches)} baseline_equals_segment1_product={retained_comparison_text(baseline, 'true')}{options.get('gate_field', '')}",
    ]
    total = len(result_rows(open(options["output"], encoding="utf-8").read())) + len(closing) + 1
    append_lines(options["output"], closing + [f"E7RESULT name=done emitted={total}"])
    print(f"  ✓ {len(items)} 个变体都有行，收尾三行已写：{options['output']}；按项判定 {verdicts.summary}{not_run_suffix(setup)}")
    if pin_mismatches:
        print(f"  ✗ KA 钉值对不上（TF7）：{' '.join(pin_mismatches)}；这几个变体的合法域结果不能读成「没有取样点」，交主 agent；别的变体照判")
        print("     → 怎么办：把这几个变体的 KA 读数与跑前登记 TF7 钉的值并排交主 agent，由主 agent 判是登记钉错还是装置算错")
    if kg_mismatches:
        print(f"  ✗ KG 钉值对不上（TF7）：{' '.join(kg_mismatches)}；这几个变体在 KG 上的落点不是登记写的那一种择根序加理想重放，交主 agent；别的变体照判")
        print("     → 怎么办：把这几个变体在 KG 上的落点与跑前登记 TF7 写的择根序并排交主 agent，由主 agent 判是登记写错还是装置算错")
    if outcomes.get("not_compared", 0):
        print("  ✗ 有变体选行不满足（TH5）记了 not_compared，交主 agent；别的变体照判")
    return EXIT_INCOMPLETE if pin_mismatches or kg_mismatches or outcomes.get("not_compared", 0) else EXIT_OK


def measure(setup, options, name):
    check_the_registered_inputs(setup)
    items, selected_rows, texts = load_items(setup)
    check_the_anchors(items, texts)
    item = next((item for item in items if item["name"] == name), None)
    if item is None:
        raise unknown_item(name, [item["name"] for item in items], "--list-items")
    directory = os.path.join(options["work"], "measure-" + name)
    binary, problem, build_seconds = prepare_and_build(setup, item, directory, options["threads"], wrap_build_for_measure=True)
    peak = "没读到"
    # 峰值表的位置与读法只在 scripts/hosts/memory_peaks_lib.py（共享数据根下一份、只认本台的行；RUN_WITH_MEMORY_CAP_PEAKS 给了照它）
    sys.path.insert(0, os.path.join(DEFAULT_ROOT, "scripts", "hosts"))
    import memory_peaks_lib
    try:
        peaks_path = os.environ.get("RUN_WITH_MEMORY_CAP_PEAKS") or memory_peaks_lib.default_peak_table()
        row = memory_peaks_lib.read_peak_rows(peaks_path).get(memory_peaks_lib.command_key([], "e165-kill-bounds measure build"))
        if row is not None:
            peak = f"{row[0] / (1 << 20):.0f} MiB"
    except (OSError, ValueError) as error:   # PeakTableUnavailable 与 TableRowRefused 都是 ValueError
        peak = f"没读到（{error}）"
    print(f"量到：变体 {name}，{options['threads']} 线程冷编译（从改 recovery.rs 到编完 bin）挂钟 {build_seconds:.1f} 秒，"
          f"内存峰值（回收不掉的用量的峰值，不含页缓存）{peak}；编译{'过了' if binary else '没过：' + problem}")
    if binary:
        completed, run_seconds = run_segment_two(setup, binary, options["threads"], directory)
        print(f"量到：跑 --bounds segment2（{options['threads']} 线程）挂钟 {run_seconds:.1f} 秒，退出码 {completed.returncode}，E7RESULT {len(result_rows(completed.stdout))} 行")
    size = subprocess.run(["du", "-sh", directory], stdout=subprocess.PIPE, text=True).stdout.split("\t")[0]
    print(f"量到：这一份副本加编译目录 {size}")
    restore_and_remove(setup, directory, keep=False, target=target_of(item))
    return EXIT_OK


def clean_value(text):
    return str(text).replace(" ", "_").replace("=", "＝")


def build_row_selection(setup, output_shown):
    """r1 RQ4 到 RQ7：产物行与总判。返回 (行, 23 行都选到且 Z 四行都对得上)。"""
    texts = {path: read_source(setup, path) for path in SOURCE_FILES}
    ranges = function_ranges(setup)
    selection = select_by_name(setup, ranges, texts)
    customs = custom_rows(setup, texts)
    tag_table = load_tag_table(setup)
    hashes = source_hashes(setup)
    lines = [f"E7RESULT name=config experiment=E165 segment=2 part=row_selection registration=research/prompts/e165-r2-prereg.md "
             f"recovery_sha256={hashes[RECOVERY_PATH]} lib_sha256={hashes[LIB_PATH]} table_sha256={sha256_of_file(os.path.join(setup.root, TABLE_PATH))} "
             f"registered_names={len(setup.registered['names'])} custom_rows={len(customs)} function_ranges={len(ranges)} "
             f"rerun=\"bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --select-rows --output {output_shown}\""]
    for (path, name, start, end), registered_entry in zip(ranges, setup.registered["functions"]):
        lines.append(f"E7RESULT name=function_range file={path} function={name} first={start} last={end} registered_first={registered_entry[2]} registered_last={registered_entry[3]}")
    item_of = {}
    for entry in sorted((candidate for candidate in selection if candidate["today_row"] is not None), key=lambda candidate: candidate["today_row"]):
        item_of.setdefault((entry["file"], entry["raw_old"], entry["raw_new"]), f"tsv-{entry['today_row']}")
    totals = {"selected": 0, "same": 0, "differs": [], "extra": [], "missing": [], "rh4": [], "same_method_equal": 0, "not_selected": []}
    for entry in selection:
        key = f"r{SAME_AS.get(entry['registered_row'], entry['registered_row'])}"
        old_tags, today_tags = tags_of(tag_table, key, "old"), tags_of(tag_table, key, "today")
        if old_tags is None or today_tags is None:
            raise Stop(EXIT_HALT_H1, f"  ✗ {TAGS_TABLE_PATH} 缺键 {key} 的 old 或 today 行\n  → 怎么办：9.3 要求每一行逐版写集合；这一次什么都没跑")
        equal, extra, missing = compare_tag_sets(entry["registered_tags"], today_tags)
        independent_equal = old_tags == set(entry["registered_tags"])
        selected = entry["failure"] is None
        totals["selected"] += 1 if selected else 0
        if not selected:
            totals["not_selected"].append(str(entry["registered_row"]))
        if equal:
            totals["same"] += 1
        else:
            totals["differs"].append(str(entry["registered_row"]))
            if extra:
                totals["extra"].append(str(entry["registered_row"]))
            if missing:
                totals["missing"].append(str(entry["registered_row"]))
        if not independent_equal:
            totals["rh4"].append(str(entry["registered_row"]))
        totals["same_method_equal"] += 1 if old_tags == today_tags else 0
        anchor = f"{entry['first']}-{entry['last']}" if entry["first"] is not None else "none"
        function_lines = f"{entry['function_lines'][0]}-{entry['function_lines'][1]}" if entry["function_lines"] else "none"
        lines.append(
            f"E7RESULT name=row_selection row_kind=table item={item_of.get((entry['file'], entry.get('raw_old'), entry.get('raw_new')), 'none') if selected else 'none'} "
            f"mutation_name={clean_value(entry['name'])} registered_table_row={entry['registered_row']} today_table_row={entry['today_row'] if entry['today_row'] else 'none'} "
            f"rows_with_this_name={entry['rows_with_this_name']} file={entry['file'] or 'none'} anchor_hits={entry['hits'] if entry['hits'] is not None else 'none'} "
            f"anchor_lines={anchor} function={entry['function'] or 'none'} function_lines={function_lines} selected={str(selected).lower()} "
            f"failed_condition={entry['failure'] or 'none'} tag_key={key} registered_tags={tag_text(entry['registered_tags'])} "
            f"registered_version_tags_by_the_same_method={tag_text(old_tags)} today_tags={tag_text(today_tags)} today_equals_registered={str(equal).lower()} "
            f"today_extra_tags={tag_text(extra) if extra else 'none'} today_missing_tags={tag_text(missing) if missing else 'none'} "
            f"registered_version_by_the_same_method_equals_registered={str(independent_equal).lower()} rh4={str(not independent_equal).lower()} "
            f"registered_version_equals_today_by_the_same_method={str(old_tags == today_tags).lower()}")
    z_ok = 0
    for row in customs:
        key = row["key"]
        located = locate_in_functions(ranges, row["file"], row["first"], row["last"]) if row["first"] else None
        if key not in setup.registered["z_tags"]:
            # r2 9.1：标签不判；没有登记标签集合的 Z 行（Z5）只按原文命中 1 次判选没选到，标签几列写 not_judged_in_r2。
            failure = "none" if row["hits"] == 1 else f"anchor_hits_{row['hits']}"
            z_ok += 1 if failure == "none" else 0
            lines.append(
                f"E7RESULT name=row_selection row_kind=custom item={key} mutation_name={clean_value(row['title'])} registered_table_row=custom_line_{row['line']} "
                f"today_table_row=custom_line_{row['line']} rows_with_this_name=1 file={row['file']} anchor_hits={row['hits']} "
                f"anchor_lines={str(row['first']) + '-' + str(row['last']) if row['first'] else 'none'} function={located[0] if located else 'none'} "
                f"function_lines={str(located[1]) + '-' + str(located[2]) if located else 'none'} selected={str(failure == 'none').lower()} failed_condition={failure} "
                f"tag_key={key} registered_tags=not_judged_in_r2 today_tags=not_judged_in_r2")
            continue
        registered = setup.registered["z_tags"][key]
        old_tags, today_tags = tags_of(tag_table, key, "old"), tags_of(tag_table, key, "today")
        if old_tags is None or today_tags is None:
            raise Stop(EXIT_HALT_H1, f"  ✗ {TAGS_TABLE_PATH} 缺键 {key} 的 old 或 today 行\n  → 怎么办：9.3 要求每一行逐版写集合；这一次什么都没跑")
        equal, extra, missing = compare_tag_sets(registered, today_tags)
        failure = "none" if row["hits"] == 1 and equal else (f"anchor_hits_{row['hits']}" if row["hits"] != 1 else "today_tags_differ_from_9_4")
        z_ok += 1 if failure == "none" else 0
        independent_equal = old_tags == set(registered)
        lines.append(
            f"E7RESULT name=row_selection row_kind=custom item={key} mutation_name={clean_value(row['title'])} registered_table_row=custom_line_{row['line']} "
            f"today_table_row=custom_line_{row['line']} rows_with_this_name=1 file={row['file']} anchor_hits={row['hits']} "
            f"anchor_lines={str(row['first']) + '-' + str(row['last']) if row['first'] else 'none'} function={located[0] if located else 'none'} "
            f"function_lines={str(located[1]) + '-' + str(located[2]) if located else 'none'} selected={str(failure == 'none').lower()} failed_condition={failure} "
            f"tag_key={key} registered_tags={tag_text(registered)} registered_version_tags_by_the_same_method={tag_text(old_tags)} today_tags={tag_text(today_tags)} "
            f"today_equals_registered={str(equal).lower()} today_extra_tags={tag_text(extra) if extra else 'none'} today_missing_tags={tag_text(missing) if missing else 'none'} "
            f"registered_version_by_the_same_method_equals_registered={str(independent_equal).lower()} rh4={str(not independent_equal).lower()} "
            f"registered_version_equals_today_by_the_same_method={str(old_tags == today_tags).lower()}")
    for key, versions in tag_table.items():
        for version in ("old", "today"):
            for tag, member, text, line_number in versions.get(version, []):
                lines.append(f"E7RESULT name=tag_judgement key={key} version={version} tags={tag} member={member} table_line={line_number} text={clean_value(text)}")
    # RQ7（附带）：9.2 的函数里还有哪些 crates/mutations.tsv 行不在 23 个名字之中；只列，不判标签。
    registered_names = {name for _row, name, _tags in setup.registered["names"]}
    additional = 0
    for number, fields in mutation_table_rows(setup):
        if fields[1] not in SOURCE_FILES or fields[0] in registered_names:
            continue
        old = unescape(fields[2])
        if texts[fields[1]].count(old) != 1:
            continue
        first = texts[fields[1]].count("\n", 0, texts[fields[1]].index(old)) + 1
        located = locate_in_functions(ranges, fields[1], first, first + old.count("\n"))
        if located is None:
            continue
        additional += 1
        lines.append(f"E7RESULT name=additional_row_in_the_function_ranges mutation_name={clean_value(fields[0])} today_table_row={number} file={fields[1]} "
                     f"anchor_lines={first}-{first + old.count(chr(10))} function={located[0]} tags=not_judged_enough_to_judge_without_it")
    all_ok = not totals["not_selected"] and z_ok == len(customs)
    lines.append(
        f"E7RESULT name=verdict scope=row_selection registered_names={len(selection)} selected={totals['selected']} not_selected={len(totals['not_selected'])} "
        f"not_selected_rows={'+'.join(totals['not_selected']) or 'none'} custom_rows={len(customs)} custom_ok={z_ok} "
        f"today_equals_registered={totals['same']} today_differs_from_registered={len(totals['differs'])} differing_rows={'+'.join(totals['differs']) or 'none'} "
        f"differing_rows_with_extra_tags={'+'.join(totals['extra']) or 'none'} differing_rows_with_missing_tags={'+'.join(totals['missing']) or 'none'} "
        f"registered_version_by_the_same_method_differs_from_registered={len(totals['rh4'])} rh4_rows={'+'.join(totals['rh4']) or 'none'} "
        f"registered_version_equals_today_by_the_same_method={totals['same_method_equal']} additional_rows_in_the_function_ranges={additional}")
    lines.append(f"E7RESULT name=done emitted={len(lines) + 1}")
    return lines, all_ok


def select_rows_action(setup, output, output_shown):
    if os.path.exists(output):
        raise Stop(EXIT_USAGE, f"  ✗ {output_shown} 已经存在\n  → 怎么办：research/results 下已有的产物不覆盖；换一个新文件名")
    check_the_registered_inputs(setup)
    lines, all_ok = build_row_selection(setup, output_shown)
    with open(output, "x", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    print(f"  ✓ 写了 {len(lines)} 行：{output_shown}；23 个名字与自造 Z 行{'都选到、有登记集合的 Z 行标签对得上' if all_ok else '里有一行选不到（RQ4 / RQ6），看 name=verdict 与各行 failed_condition'}")
    return EXIT_OK if all_ok else EXIT_INCOMPLETE
