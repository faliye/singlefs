# selftest-serial: 只有一格，reuse-cells 那六格在同一份临时服务的命名空间里依次跑，前后有先后
"""E171（三oracle合谋测试）整表那几行（scripts/experiment-apparatus/replay.sh 的 E171 第 7、9、10、11 行，驱动 driver_e171_s2_comparator、driver_e171_s4_comparator
与跟着它的扩表、补位两条）里与表无关、几行共用的前置，做成能单独点名、各自一条任务的项；每项一条判定存进判定库（下面「判定库」），几行共用的只做一次，之后从库里取回。

项（scripts/experiment-apparatus/e171-row-shard.sh 的 --list-items 列的是同一份名字；那份入口替这几项起本文件）：
  prerequisites              装置基线两趟（baseline-run1 连同装置的标准输出、baseline-run2）、E170 导出装置的 dump 与 bad-states、已知坏镜像、关 O1 与关 O2 的基线、
                             O3 判官在坏镜像与基线上各一趟、冻结的 O3 执行器在 baseline-run1 上一趟（PREREQUISITE_STEPS，这几步的唯一定义）
  positive-control:O1-对拍、positive-control:O1-崩溃判官、positive-control:O2
                             阳性对照：装置 baseline 带 --positive-control 一趟（control-baseline-<对照>），逐行驱动只跑空变异一趟（rows-controls-<对照>）
  positive-control:O3        O3 阳性对照：逐行驱动只跑空变异，带 prerequisites 里那份 E170 bad-states 导出（先要库里有 prerequisites，没有就退 4、点名它）
每一项的产物放在 <共用输出根>/<项的目录>/（ITEM_DIRECTORIES）下的 dumps/（导出目录）与 scratch/（装置标准输出、判官与执行器的产物），名字与 replay.sh 的驱动的比对器读的相同
（E170 的 dump 导出这里叫 e170-dump，取的时候换成驱动用的名字）。
判定库（.claude/rules/verification.md「判定库只有一份」；算键、查库、入库与复用哨兵都经 scripts/experiment-apparatus/long_run_item_verdicts.py，本地不建库目录）：
  一项一条判定（库里的项名是 LIBRARY_ITEM_NAMES：只许字母、数字、. _ -），存进 prune-server 的 apparatus-items 族、种类 e171_three_oracle_collusion_shared_steps
  （LIBRARY_KIND）：items/<键>.out 是 row_driver.pack_directory 打的项目录（dumps/ 与 scratch/，同一份内容同一串字节），items/<键>.verdict 是判定（比法 timing：
  产物里带耗时）。
  键照 long_run_item_verdicts.py 文件头「请求文件」，由 admission.py 的 compute_unit_key 在 --repo-root 那棵树上算（item_request_lines）：
    入口   装置二进制 e171_three_oracle_collusion 的 main（--device-features gpu 带 feature:singlefs-implementation-experiments/gpu；profile:release）；prerequisites 另一条请求是
           E170 导出装置 e170_specification_executor_export 的 main（同一项的几条请求合成一把键）
    judge  本文件的 make_shared_step；prerequisites 与 positive-control:O3 另加 o3_bad_image_judge.py 的 main，四条阳性对照另加 row_driver.py 的 run_row_on_the_lane
           （judge-root 是本文件所在的仓，只当索引）
    line:  项名、--device-features、冻结 O3 的冻结行摘要（row_driver.o3_freeze_digest）；阳性对照另加对照名与空变异那一行的变异行原文（row_driver.empty_mutation_line_of）
    file:  positive-control:O3 读的 prerequisites 产物：<共用输出根>/prerequisites/dumps/e170-bad-states-copy-s2 里每个文件的内容（@files，路径不进）
  做（run）：plan 答 reuse 的从库里取回、解进项目录（先删那里留的），一步都不起，标准错误打 E171_SHARED_STEP item=<项> taken=yes；答 judge 或拒算的删掉项目录与
    工作目录重做，做完打 taken=no seconds=<秒>，经 record 记：做成的记绿、存项目录，有一步退非 0 的照同一把键记红（盖掉原有的绿）、下一趟现判；
    再起 finish（重算现判过的项的键、判复用哨兵），plan 与 finish 的逐项行与总计行只进标准错误，finish 退非 0 这一项退 1。
  positive-control:O3 做、取之前先照键从库里查 prerequisites（只查不判：不起 plan、不抽哨兵），在就解进 <共用输出根>/prerequisites/（它的产物进 O3 的键），不在退 4。
  共用输出根里已有的项目录不当复用来源：做与取都先删掉，照库里那一份或现做的重写。

跑法（在快照树的根目录起，整条经 task-server/task submit 交；平常经 e171-row-shard.sh 起）：
  python3 shared_steps.py run <项> --root <共用输出根> --repo-root <树根> --device-features gpu|none
  python3 shared_steps.py fetch <项> --root … --repo-root … --device-features …      库里有这一项（键相同、判绿）：解进 <共用输出根>/<项的目录>/、退 0；
                             没有退 4，标准错误打一行 E171_SHARED_STEP_MISSING
  python3 shared_steps.py rows-done --selected-rows <表> --rows-root <输出根>         表里每一行在输出根里都有 row-<序号>/row-result.tsv 退 0，缺的退 4
  python3 shared_steps.py take --root … --repo-root … --device-features … --dumps <导出目录> --scratch <草稿目录> --e170-dump-name <名字>
                             只取（不现判）：replay.sh 的 s2、s4 驱动用（--root 是驱动自己草稿目录下的一个子目录）：SHARED_ITEMS 五项库里都有（键相同、判绿），
                             才逐项解进 --root、把各项的 dumps/、scratch/ 拷进驱动的导出目录与草稿目录；缺的逐项点名（连同单做那一项的命令），退 4，一份都不拷
  python3 shared_steps.py take-rows <row_driver.py 的选行与键的参数…> [--check-only]
                             只取（不现判）逐行驱动的行：row_driver.py 文件头「只取」（参数、输出的几种行、取法都在那里）；库里没有的行逐行点名、退 4，一行都不编不跑；
                             命令行设了 E171_ROW_DRIVER_BREAK 退 2（弄坏开关只在单测与 row_driver.py 的自证里认）
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/experiment-apparatus/e171_three_oracle_collusion/shared_steps.py --selftest
                             自证（下面「自证」）
  PREREQUISITE_STEPS、CONTROL_DEVICE_STEP 是这几步的唯一定义：replay.sh 的 s2、s4、扩表、补位四个驱动只经 take 取它们的产物、自己不做。replay_text 拼的是这一步写成
  replay.sh 那种命令行的样子（只当步名打在日志里）。test_shared_steps.py 核两样：坏镜像那几步（基线、拷一份、E170 导出装置 bad-states 与它的种子、已知坏镜像、
  O3 判官）与 replay.sh 里还在的单列驱动（e171_s2_prepare_bad_states、e171_s2_known_bad、driver_e171_s2_images_o3）逐词相同；四个驱动的比对器读的每一样
  共用前置的产物，都是这里某一步写出的（s2 那一份的 E170 导出叫 e170-dump、s4 的叫 e170-pinned-dump）。
  装置那几步：开了 GPU（--device-features gpu）先 cargo build（不持卡），再分录入、判两段起
  （device_card_lease.record_then_judge_on_a_leased_card，规矩在 device_card_lease.py 文件头「装置分录入、判两段起」）：录入段（cargo run … -- <参数> --stage record）
  不持卡、环境里没有 GPU_CARDS_BY_PRIORITY，标准输出转到标准错误；录入段退 0 才经 device_card_lease.take_card 领一张，判段（同一串参数，--stage judge）带它的
  GPU_CARDS_BY_PRIORITY 起、标准输出照这一步写，跑完 return_card 还掉（判段退非 0 照还）；录入段退非 0 不领卡、判段不起，这一步记录入段的退出码。
  没开 GPU 直接 cargo run、一段起、不领卡。E170 导出装置、O3 判官、执行器不持卡；逐行驱动开了 GPU 时带
  --lease-card-per-device-run，自己只在判段领还。开了 GPU 而环境里没有领卡通道：要现做的项一步都不做，退 78。
  编译目录：环境里有 CARGO_TARGET_DIR（服务照 e171-row-shard.sh 文件头的会编声明给的）就照它编、不删；没有就编进这一项的工作目录 <共用输出根>/work-<项的目录>/target。
  逐行驱动的线道也在这个工作目录下；工作目录与这一趟的状态目录（<共用输出根>/item-verdicts-*）成功、失败都删。
自证（--selftest，一格；只在临时 prune-server 底下跑，客户端 selftest_service_problem 判，不在退 2、一格都不跑；PATH 最前面一份假 cargo 答算键时问的版本，不起真 cargo）：
  reuse-three-questions  long_run_item_verdicts.py reuse-cells 那六格（打 REUSE_SELFTEST script=scripts/experiment-apparatus/e171_three_oracle_collusion/shared_steps.py …）：
      样板树是空的，起法是本文件的 selftest-round <树>（只在临时 prune-server 底下认：prerequisites 与 positive-control:O2 两项在那棵树上一个计划里走一趟
      judge_shared_steps，做一项换成往项目录里写两份固定的文件）
  末行 SHARED_STEPS_SELFTEST cells=1 red=<红几格>
退出码：0 做完或取了现成的；1 有一步退非 0、finish 判红、自证有红；2 用法错、--selftest 与 selftest-round 不在临时服务底下、take-rows 的命令行设了 E171_ROW_DRIVER_BREAK；
  4 缺前置项、库里没有（take、fetch、rows-done 缺项缺行，take-rows 有行库里没有）；
  75 prune-server 连不上（照客户端的码）；78 开了 GPU 而没有领卡通道。
弄坏开关（E171_SHARED_STEPS_BREAK，只在自证里认：test_shared_steps.py 在进程里调 main 时把 BREAK_ACCEPTED_IN_PROCESS 置真，临时 prune-server 底下的 --selftest 与
selftest-round 也认；命令行别处设了退 2）。逐项写开关名与弄坏哪一处，下一行起写打开后红的用例（unittest 失败行「FAIL: <测试名> …」的句首）或自证项：
  E171_SHARED_STEPS_BREAK=reused-item-remade  plan 答 reuse 也照做一遍
      「FAIL: test_a_second_run_takes_the_item_from_the_library_and_starts_nothing」
  E171_SHARED_STEPS_BREAK=prerequisites-unchecked  O3 阳性对照不查库里有没有 prerequisites
      「FAIL: test_the_o3_control_without_prerequisites_names_the_missing_item_and_exits_4」
  E171_SHARED_STEPS_BREAK=card-held-through-the-build  装置那一步先领卡再编
      「FAIL: test_no_card_is_held_while_building_or_recording_and_each_judge_stage_takes_and_returns_one」
      「FAIL: test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage」（编之前领了一回）
  E171_SHARED_STEPS_BREAK=take-ignores-the-library  取的那一步不查库，共用输出根里有项目录就拷
      「FAIL: test_take_refuses_items_the_library_does_not_hold」
  E171_SHARED_STEPS_BREAK=e170-dump-not-renamed  取的时候 E170 的 dump 导出不换成驱动用的名字
      「FAIL: test_take_copies_every_product_under_the_names_the_driver_uses」
  E171_SHARED_STEPS_BREAK=step-drifts-from-replay  bad-states 那一步的种子换一个（与 replay.sh 单列驱动 e171_s2_prepare_bad_states 那一行不同）
      「FAIL: test_the_bad_image_steps_match_the_single_row_drivers_and_feed_every_comparator_input」
  E171_SHARED_STEPS_BREAK=request-without-features  请求里不带 --device-features
      「FAIL: test_the_request_names_the_item_the_features_and_what_each_item_runs」
  E171_SHARED_STEPS_BREAK=failed-item-recorded-green  有一步退非 0 的项照绿记、存项目录
      「FAIL: test_a_failed_step_names_the_step_and_is_judged_again」
  E171_SHARED_STEPS_BREAK=item-finish-ignored  不认 long_run_item_verdicts finish 的退出码
      自证项「✗ reuse-three-questions」（reuse-cells 里 sentinel-rejudged-red、forced-verdict-differs-red、forced-still-reused-red 三格要的退 1 变成退 0）
  E171_SHARED_STEPS_BREAK=card-held-through-the-record-stage  装置那一步编完就领卡，录入段拿着卡串跑
      「FAIL: test_no_card_is_held_while_building_or_recording_and_each_judge_stage_takes_and_returns_one」
      「FAIL: test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage」
      「FAIL: test_the_record_stage_runs_without_the_cards_of_the_caller」「FAIL: test_a_failed_judge_stage_returns_its_card」（录入段拿着领到的卡）
  E171_SHARED_STEPS_BREAK=cards-left-in-the-record-environment  录入段的环境里留着这一项环境里的 GPU_CARDS_BY_PRIORITY
      「FAIL: test_the_record_stage_runs_without_the_cards_of_the_caller」
  E171_SHARED_STEPS_BREAK=card-returned-before-the-judge-stage  领了卡先还、再起判段
      「FAIL: test_no_card_is_held_while_building_or_recording_and_each_judge_stage_takes_and_returns_one」
  E171_SHARED_STEPS_BREAK=judge-stage-after-a-failed-record  录入段退非 0 照领卡、起判段
      「FAIL: test_a_failed_record_stage_takes_no_card_and_starts_no_judge_stage」
  E171_SHARED_STEPS_BREAK=card-kept-after-a-failed-judge  判段退非 0 不还卡
      「FAIL: test_a_failed_judge_stage_returns_its_card」
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time

import device_card_lease
import frozen_o3
import row_driver

HERE = os.path.dirname(os.path.abspath(__file__))
ROW_DRIVER_SCRIPT = os.path.join(HERE, "row_driver.py")
IMAGE_JUDGE_SCRIPT = os.path.join(HERE, "o3_bad_image_judge.py")   # 同 row_driver.py 的 IMAGE_JUDGE_SCRIPT
# 冻结执行器的目录与入口（同 frozen_o3.py 的 FROZEN_DIRECTORY_RELATIVE、EXECUTOR_SCRIPT；test_shared_steps.py 核两处相同）
EXECUTOR_DIRECTORY_RELATIVE = "scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_496"
EXECUTOR_SCRIPT_NAME = "executor_main.py"
# 子进程的标准输出转到标准错误（replay.sh 的 >&2）：给描述符 2，不给 sys.stderr（自证在进程里把 sys.stderr 换成内存里的流，它没有描述符）
STANDARD_ERROR_DESCRIPTOR = 2
RESEARCH_DIRECTORY_RELATIVE = "scripts/experiment-apparatus/e171_three_oracle_collusion"
EXPORTER_BINARY_NAME = "e170_specification_executor_export"
BREAK_VARIABLE = "E171_SHARED_STEPS_BREAK"
# test_shared_steps.py 在进程里调 main 时把它置真，临时 prune-server 底下的 --selftest 与 selftest-round 也置真；别的命令行起的本脚本永远是假
BREAK_ACCEPTED_IN_PROCESS = False
EXIT_STEP_FAILED = 1
EXIT_USAGE = 2
EXIT_MISSING = 4
DUMPS = "dumps"
SCRATCH = "scratch"
E170_DUMP_SHARED_NAME = "e170-dump"   # 共用输出根里 E170 dump 导出的名字；取的时候换成驱动用的（--e170-dump-name）
PREREQUISITES = "prerequisites"
O3_CONTROL_ITEM = "positive-control:O3"
CONTROLS_WITH_A_DEVICE_BASELINE = ("O1-对拍", "O1-崩溃判官", "O2")
SHARED_ITEMS = (PREREQUISITES, *(f"positive-control:{control}" for control in CONTROLS_WITH_A_DEVICE_BASELINE), O3_CONTROL_ITEM)
ITEM_DIRECTORIES = {item: item.replace(":", "-") for item in SHARED_ITEMS}
FEATURE_CHOICES = ("gpu", "none")
# 判定库（文件头「判定库」）：种类、本文件的键名、库里的项名（long_run_item_verdicts 只认字母、数字、. _ -）、各项请求里的入口与判法、
# O3 阳性对照读的那份 prerequisites 导出、这一趟的状态目录（共用输出根下新建，跑完删）
LIBRARY_KIND = "e171_three_oracle_collusion_shared_steps"
SCRIPT_KEY = f"{RESEARCH_DIRECTORY_RELATIVE}/shared_steps.py"
LIBRARY_ITEM_NAMES = {PREREQUISITES: "prerequisites", "positive-control:O1-对拍": "positive-control.O1-model-differential",
                      "positive-control:O1-崩溃判官": "positive-control.O1-crash-judge", "positive-control:O2": "positive-control.O2",
                      O3_CONTROL_ITEM: "positive-control.O3"}
DEVICE_ENTRY = f"crates/{row_driver.DEVICE_PACKAGE}/src/bin/{row_driver.DEVICE_BINARY_NAME}.rs::main"
EXPORTER_ENTRY = f"crates/{row_driver.DEVICE_PACKAGE}/src/bin/{EXPORTER_BINARY_NAME}.rs::main"
ITEM_JUDGE = f"{SCRIPT_KEY}::make_shared_step"
ROW_JUDGE = f"{RESEARCH_DIRECTORY_RELATIVE}/row_driver.py::run_row_on_the_lane"
IMAGE_JUDGE = f"{RESEARCH_DIRECTORY_RELATIVE}/o3_bad_image_judge.py::main"
BAD_STATES_COPY = "e170-bad-states-copy-s2"
ITEM_STATE_PREFIX = "item-verdicts-"
SELFTEST_ROUND_MODE = "selftest-round"
SELFTEST_ITEMS = (PREREQUISITES, "positive-control:O2")
# 词里的占位写成 replay.sh 那种命令行时的变量（replay_text 只拿它当步名）；bad_states 是 E170 bad-states 导出所在的导出目录（驱动里与别的导出同在 $dumps，这里在 prerequisites 那一项下）
SHELL_VARIABLES = {"dumps": "$dumps", "scratch": "$scratch", "control": "$control", "workers": "$o3_workers", "bad_states": "$dumps"}

DEVICE = "device"
EXPORTER = "exporter"
COPY = "copy"
JUDGE = "judge"
EXECUTOR = "executor"
# 每一种步骤写成 replay.sh 那种命令行时的命令词（replay_text 拼步名用；模块级赋值进项键的判法闭包，照原样留着）
SHELL_WORDS = {DEVICE: "e171_device_on_a_leased_card", EXPORTER: "e171_e170_exporter", COPY: "cp -r",
               JUDGE: "e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py",
               EXECUTOR: "PYTHONDONTWRITEBYTECODE=1 python3 executor_main.py"}
TO_STANDARD_ERROR = ">&2"
TO_NOWHERE = ">/dev/null"


@dataclasses.dataclass(frozen=True)
class Step:
    """一步：`kind` 是哪一种（装置、导出装置、拷目录、O3 判官、执行器）；`arguments` 的每个词可以带 {dumps}、{scratch}、{control}、{workers}、{e170_dump}，
    做的时候换成这一项的目录与值；`stdout` 是 TO_STANDARD_ERROR、TO_NOWHERE、写进 {scratch} 下的文件，或空串（拷目录）。"""

    kind: str
    arguments: tuple[str, ...]
    stdout: str


PREREQUISITE_STEPS = (
    Step(DEVICE, ("baseline", "--output-directory", "{dumps}/baseline-run1"), "{scratch}/baseline-run1.stdout"),
    Step(DEVICE, ("baseline", "--output-directory", "{dumps}/baseline-run2"), TO_NOWHERE),
    Step(EXPORTER, ("dump", "--output-directory", "{dumps}/{e170_dump}"), TO_NOWHERE),
    Step(COPY, ("{dumps}/baseline-run1", "{dumps}/e170-bad-states-copy-s2"), ""),
    Step(EXPORTER, ("bad-states", "--dump-directory", "{dumps}/e170-bad-states-copy-s2", "--seed-hex", "85441ae491419411"), TO_NOWHERE),
    Step(DEVICE, ("known-bad-images", "--dump-directory", "{dumps}/e170-bad-states-copy-s2", "--output-directory", "{dumps}/known-bad-s2", "--e170-bad-rows"),
         TO_NOWHERE),
    Step(DEVICE, ("baseline", "--output-directory", "{dumps}/disable-O1", "--disable-oracle", "O1"), TO_NOWHERE),
    Step(DEVICE, ("baseline", "--output-directory", "{dumps}/disable-O2", "--disable-oracle", "O2"), TO_NOWHERE),
    Step(JUDGE, ("judge", "--e170-dumps", "{dumps}/e170-bad-states-copy-s2", "--known-bad-dir", "{dumps}/known-bad-s2", "--output", "{scratch}/images-o3.out"),
         TO_STANDARD_ERROR),
    Step(JUDGE, ("control", "--dumps", "{dumps}/baseline-run1", "--e170-dumps", "{dumps}/e170-bad-states-copy-s2", "--row-name", row_driver.O3_CONTROL_ROW_NAME,
                 "--output", "{scratch}/o3-control-baseline.out"), TO_STANDARD_ERROR),
    Step(EXECUTOR, ("--dumps", "{dumps}/baseline-run1", "--workers", "{workers}", "--output", "{scratch}/o3-baseline.out"), TO_STANDARD_ERROR),
)
CONTROL_DEVICE_STEP = Step(DEVICE, ("baseline", "--output-directory", "{dumps}/control-baseline-{control}", "--positive-control", "{control}"), TO_NOWHERE)
# 逐行驱动跑阳性对照的那几个参数（会改变行的结果、进行的复用键的那几个）
CONTROL_ROW_ARGUMENTS = ("--output-root", "{dumps}/rows-controls-{control}", "--empty-mutation-only", "--positive-control", "{control}")
O3_CONTROL_ROW_ARGUMENTS = ("--output-root", "{dumps}/rows-controls-O3", "--empty-mutation-only", "--positive-control", row_driver.O3_CONTROL,
                            "--o3-control-bad-states", "{bad_states}/e170-bad-states-copy-s2")


def broken(name: str) -> bool:
    return BREAK_ACCEPTED_IN_PROCESS and os.environ.get(BREAK_VARIABLE) == name


def steps_of(item: str) -> tuple[Step, ...]:
    if item != PREREQUISITES:
        return ()
    if not broken("step-drifts-from-replay"):
        return PREREQUISITE_STEPS
    return tuple(dataclasses.replace(step, arguments=tuple("0000000000000000" if word == "85441ae491419411" else word for word in step.arguments))
                 for step in PREREQUISITE_STEPS)


def shell_word(word: str, e170_dump_name: str) -> str:
    """一个词写成 replay.sh 那种命令行时的样子：带目录与变量的照 SHELL_VARIABLES 的变量名写、整个加双引号，别的照原样。"""
    if not any("{" + name + "}" in word for name in SHELL_VARIABLES):
        return word.format(e170_dump=e170_dump_name)
    return '"' + word.format(e170_dump=e170_dump_name, **SHELL_VARIABLES) + '"'


def replay_text(step: Step, e170_dump_name: str) -> str:
    """这一步写成 replay.sh 那种命令行的样子（空白折成一个空格，不带 `&&`、括号与续行），只当步名打在日志里。"""
    words = [SHELL_WORDS[step.kind], *(shell_word(word, e170_dump_name) for word in step.arguments)]
    if step.stdout in (TO_STANDARD_ERROR, TO_NOWHERE):
        words.append(step.stdout)
    elif step.stdout:
        words.append(">" + shell_word(step.stdout, e170_dump_name))
    return " ".join(words)


def item_directory(root: str, item: str) -> str:
    return os.path.join(root, ITEM_DIRECTORIES[item])


def item_request_lines(item: str, repo_root: str, device_features: str, root: str) -> list[str]:
    """一项的判定请求（long_run_item_verdicts.py 文件头「请求文件」，本文件头「判定库」）。"""
    name = LIBRARY_ITEM_NAMES[item]
    program = [*([f"feature:{row_driver.DEVICE_PACKAGE}/gpu"] if device_features == "gpu" else []), "profile:release"]
    judges = [ITEM_JUDGE, *([IMAGE_JUDGE] if item in (PREREQUISITES, O3_CONTROL_ITEM) else []), *([ROW_JUDGE] if item != PREREQUISITES else [])]
    tokens = [name, DEVICE_ENTRY, *program, *(f"judge:{judge}" for judge in judges), f"judge-root:{frozen_o3.OWN_REPOSITORY_ROOT}", f"line:item={name}"]
    if not broken("request-without-features"):
        tokens.append(f"line:device-features={device_features}")
    tokens.append(f"line:o3-freeze={row_driver.o3_freeze_digest(repo_root)}")
    if item != PREREQUISITES:
        tokens += [f"line:control={item.split(':', 1)[1]}",
                   f"line:empty-mutation-line={json.dumps(row_driver.empty_mutation_line_of(repo_root), ensure_ascii=False)}"]
    tokens.append("line:compare=timing")
    lines = ["\t".join(tokens)]
    if item == PREREQUISITES:
        lines.append("\t".join([name, EXPORTER_ENTRY, *program]))
    if item == O3_CONTROL_ITEM:
        lines.append(f"@files\t{name}\t{os.path.join(item_directory(root, PREREQUISITES), DUMPS, BAD_STATES_COPY)}")
    return lines


def requests_text(items: list[str], repo_root: str, device_features: str, root: str) -> str:
    return "".join(line + "\n" for item in items for line in item_request_lines(item, repo_root, device_features, root))


def library_entries(items: list[str], root: str, repo_root: str, device_features: str) -> dict[str, bytes | None]:
    """这几项照此刻的键从库里取那一包（只查不判：不起 plan、不抽哨兵）：.verdict 记着绿、.out 读得回且 sha256 与它记的相同才算有。交 {项: 那一包或 None}。"""
    verdicts = row_driver.item_verdicts_module()
    requests = verdicts.parse_requests(requests_text(items, repo_root, device_features, root), "E171 共用前置的请求")
    if os.environ.get(verdicts.SUBSTITUTION_VARIABLE):
        verdicts.substitute_entries(requests, os.environ[verdicts.SUBSTITUTION_VARIABLE])
    keyed = verdicts.item_keys(repo_root, requests)
    store = verdicts.library_of(LIBRARY_KIND, verdicts.library_directory(repo_root))
    found: dict[str, bytes | None] = {}
    for item in items:
        key = keyed[LIBRARY_ITEM_NAMES[item]][0]
        found[item] = None
        if key is None:
            continue
        raw_verdict, output = store.read_many([verdicts.verdict_relative(key), verdicts.output_relative(key)])
        record = verdicts.parsed_record(raw_verdict, key)
        if record is not None and record.get("verdict") == "green" and output is not None and hashlib.sha256(output).hexdigest() == record.get("output_sha256"):
            found[item] = output
    return found


def place_item(root: str, item: str, packed: bytes) -> None:
    """库里那一包解进 <共用输出根>/<项的目录>/（先删那里留的）。"""
    directory = item_directory(root, item)
    if os.path.isdir(directory):
        shutil.rmtree(directory)
    row_driver.unpack_directory(packed, directory)


def prerequisites_from_the_library(root: str, repo_root: str, device_features: str) -> bool:
    """O3 阳性对照之前（文件头「判定库」）：库里有 prerequisites 就解进共用输出根、交 True；没有交 False。"""
    packed = library_entries([PREREQUISITES], root, repo_root, device_features)[PREREQUISITES]
    if packed is None:
        return False
    place_item(root, PREREQUISITES, packed)
    return True


def remake_command(root: str, item: str) -> str:
    return f"bash scripts/experiment-apparatus/e171-row-shard.sh --root {root} --item {item}"


def stdout_target(step: Step, placeholders: dict[str, str]):
    if step.stdout == TO_STANDARD_ERROR:
        sys.stderr.flush()
        return STANDARD_ERROR_DESCRIPTOR
    if step.stdout == TO_NOWHERE:
        return subprocess.DEVNULL
    return open(step.stdout.format(**placeholders), "w", encoding="utf-8")


def cargo_features(device_features: str) -> list[str]:
    return ["--features", "gpu"] if device_features == "gpu" else []


def run_cargo(command: list[str], repo_root: str, environment: dict[str, str], stdout) -> int:
    return subprocess.run(command, cwd=repo_root, env=environment, stdout=stdout).returncode


def run_device_step(repo_root: str, device_features: str, environment: dict[str, str], arguments: list[str], stdout, step_name: str) -> int:
    """装置那一步（文件头「跑法」）：没开 GPU 直接 cargo run、一段起；开了先编（不持卡），再分录入、判两段起：
    录入段不持卡、标准输出转到标准错误，录入段退 0 才领卡起判段（标准输出进 stdout），判段跑完还卡。领卡没成交 EXIT_STEP_FAILED（判段没起）。"""
    features = cargo_features(device_features)
    run_command = ["cargo", "run", "-q", "--release", "-p", row_driver.DEVICE_PACKAGE, *features, "--bin", row_driver.DEVICE_BINARY_NAME, "--", *arguments]
    if device_features != "gpu":
        return run_cargo(run_command, repo_root, environment, stdout)
    build_command = ["cargo", "build", "-q", "--release", "-p", row_driver.DEVICE_PACKAGE, *features, "--bin", row_driver.DEVICE_BINARY_NAME]
    client = device_card_lease.client_of(repo_root)
    cards_held_through_the_build = None
    if broken("card-held-through-the-build"):
        try:
            cards_held_through_the_build = device_card_lease.take_card(client, repo_root, step_name)
        except device_card_lease.CardLeaseRefused:
            return EXIT_STEP_FAILED
    sys.stderr.flush()
    build_exit = run_cargo(build_command, repo_root, environment, STANDARD_ERROR_DESCRIPTOR)
    if build_exit != 0:
        if cards_held_through_the_build is not None:
            device_card_lease.return_card(client, repo_root, cards_held_through_the_build, step_name)
        return build_exit

    def run_stage(stage: str, cards: str | None) -> int:
        sys.stderr.flush()
        stage_output = STANDARD_ERROR_DESCRIPTOR if stage == device_card_lease.RECORD_STAGE else stdout
        return run_cargo(run_command + device_card_lease.stage_arguments(stage), repo_root,
                         device_card_lease.environment_of_the_stage(environment, stage, cards, broken), stage_output)

    def take() -> str:
        if cards_held_through_the_build is not None:
            return cards_held_through_the_build
        return device_card_lease.take_card(client, repo_root, step_name)

    def give_back(cards: str) -> None:
        device_card_lease.return_card(client, repo_root, cards, step_name)

    try:
        return device_card_lease.record_then_judge_on_a_leased_card(run_stage, take, give_back, broken)
    except device_card_lease.CardLeaseRefused:
        return EXIT_STEP_FAILED


def run_step(step: Step, repo_root: str, device_features: str, environment: dict[str, str], placeholders: dict[str, str]) -> int:
    words = [word.format(**placeholders) for word in step.arguments]
    name = replay_text(step, E170_DUMP_SHARED_NAME)
    if step.kind == COPY:
        shutil.copytree(words[0], words[1], symlinks=True)
        return 0
    stdout = stdout_target(step, placeholders)
    try:
        if step.kind == DEVICE:
            return run_device_step(repo_root, device_features, environment, words, stdout, name)
        if step.kind == EXPORTER:
            command = ["cargo", "run", "-q", "--release", "-p", row_driver.DEVICE_PACKAGE, *cargo_features(device_features), "--bin", EXPORTER_BINARY_NAME, "--", *words]
            return run_cargo(command, repo_root, environment, stdout)
        python_environment = dict(environment, PYTHONDONTWRITEBYTECODE="1")
        if step.kind == JUDGE:
            return subprocess.run(["python3", IMAGE_JUDGE_SCRIPT, *words], cwd=repo_root, env=python_environment, stdout=stdout).returncode
        executor = os.path.join(repo_root, EXECUTOR_DIRECTORY_RELATIVE, EXECUTOR_SCRIPT_NAME)
        return subprocess.run(["python3", executor, *words], cwd=os.path.join(repo_root, EXECUTOR_DIRECTORY_RELATIVE), env=python_environment,
                              stdout=stdout).returncode
    finally:
        if stdout not in (STANDARD_ERROR_DESCRIPTOR, subprocess.DEVNULL):
            stdout.close()


def run_control_row(repo_root: str, device_features: str, environment: dict[str, str], work: str, row_arguments: list[str]) -> int:
    """阳性对照的逐行驱动（只跑空变异）：开了 GPU 带 --lease-card-per-device-run；标准输出转到标准错误。"""
    card_lease = ["--lease-card-per-device-run"] if device_features == "gpu" else []
    sys.stderr.flush()
    return subprocess.run(["python3", ROW_DRIVER_SCRIPT, "--repo-root", repo_root, *card_lease, "--work-root", work, *row_arguments],
                          cwd=repo_root, env=dict(environment, PYTHONDONTWRITEBYTECODE="1"), stdout=STANDARD_ERROR_DESCRIPTOR).returncode


def make_shared_step(item: str, root: str, repo_root: str, features: str) -> int:
    """现做一项（文件头「项」）：删掉项目录与工作目录、照 PREREQUISITE_STEPS 或对照那几步逐条做；交 0 做成、EXIT_STEP_FAILED 有一步退非 0（标准错误点名是哪一步）。"""
    directory = item_directory(root, item)
    prerequisites = item_directory(root, PREREQUISITES)
    work = os.path.join(root, "work-" + ITEM_DIRECTORIES[item])
    for stale in (directory, work):
        if os.path.isdir(stale):
            shutil.rmtree(stale)
    placeholders = {"dumps": os.path.join(directory, DUMPS), "scratch": os.path.join(directory, SCRATCH), "e170_dump": E170_DUMP_SHARED_NAME,
              "workers": str(row_driver.threads_per_lane(None, 1))}
    for path in (placeholders["dumps"], placeholders["scratch"], work):
        os.makedirs(path)
    environment = dict(os.environ)
    environment.setdefault("CARGO_TARGET_DIR", os.path.join(work, "target"))   # 服务照入口的会编声明给了编译目录就照它编（归服务删）
    try:
        if item == PREREQUISITES:
            for step in steps_of(item):
                exit_code = run_step(step, repo_root, features, environment, placeholders)
                if exit_code != 0:
                    print(f"  ✗ E171 共用前置 {item}：「{replay_text(step, E170_DUMP_SHARED_NAME)}」退 {exit_code}，这一项没做完（照同一把键记红）", file=sys.stderr)
                    return EXIT_STEP_FAILED
        elif item == O3_CONTROL_ITEM:
            control_placeholders = dict(placeholders, bad_states=os.path.join(prerequisites, DUMPS))
            exit_code = run_control_row(repo_root, features, environment, work, [word.format(**control_placeholders) for word in O3_CONTROL_ROW_ARGUMENTS])
            if exit_code != 0:
                print(f"  ✗ E171 共用前置 {item}：逐行驱动退 {exit_code}，这一项没做完（照同一把键记红）", file=sys.stderr)
                return EXIT_STEP_FAILED
        else:
            control_placeholders = dict(placeholders, control=item.split(":", 1)[1])
            exit_code = run_step(CONTROL_DEVICE_STEP, repo_root, features, environment, control_placeholders)
            if exit_code == 0:
                exit_code = run_control_row(repo_root, features, environment, work, [word.format(**control_placeholders) for word in CONTROL_ROW_ARGUMENTS])
            if exit_code != 0:
                print(f"  ✗ E171 共用前置 {item}：装置对照基线或逐行驱动退 {exit_code}，这一项没做完（照同一把键记红）", file=sys.stderr)
                return EXIT_STEP_FAILED
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return 0


def print_to_standard_error(line: str) -> None:
    print(line, file=sys.stderr, flush=True)


def judge_shared_steps(items: list[str], root: str, repo_root: str, features: str, item_maker=None, printer=print_to_standard_error) -> int:
    """做几项（文件头「判定库」的「做」）：一个计划里算键、查库；复用的从库里取回，别的现做（item_maker，不给照 make_shared_step）、经 record 记库；末了 finish。
    交退出码：0 都做完或取了现成的；EXIT_MISSING O3 阳性对照之前库里没有 prerequisites；78 开了 GPU 而没有领卡通道；EXIT_STEP_FAILED 有一项没做成、finish 判红。"""
    verdicts = row_driver.item_verdicts_module()
    if O3_CONTROL_ITEM in items and not broken("prerequisites-unchecked") and not prerequisites_from_the_library(root, repo_root, features):
        print(f"  ✗ E171 共用前置 {O3_CONTROL_ITEM}：要先有 {PREREQUISITES}，库里没有这棵树、这组特性下键相同、判绿的那一项；这一项一步都没做", file=sys.stderr)
        print(f"  → 怎么办：先做 {PREREQUISITES}（{remake_command(root, PREREQUISITES)}），再做这一项", file=sys.stderr)
        return EXIT_MISSING
    os.makedirs(root, exist_ok=True)
    state = tempfile.mkdtemp(prefix=ITEM_STATE_PREFIX, dir=root)
    failed = False
    try:
        verdicts.plan(state, SCRIPT_KEY, LIBRARY_KIND, repo_root, requests_text(items, repo_root, features, root), "E171 共用前置的请求", printer)
        planned = verdicts.read_plan(state)["items"]
        for item in items:
            name = LIBRARY_ITEM_NAMES[item]
            key = (planned[name]["key"] or "-")[:12]
            if planned[name]["decision"] == verdicts.REUSE and not broken("reused-item-remade"):
                place_item(root, item, verdicts.reused_output(state, name)[1])
                print(f"E171_SHARED_STEP item={item} taken=yes key={key}", file=sys.stderr, flush=True)
                continue
            if features == "gpu":
                problem = device_card_lease.channel_problem()
                if problem is not None:
                    device_card_lease.report_missing_channel("shared_steps.py", problem, remake_command(root, item))
                    return device_card_lease.EXIT_NO_CHANNEL
            started = time.monotonic()
            exit_code = (item_maker or make_shared_step)(item, root, repo_root, features)
            made = exit_code == 0 or broken("failed-item-recorded-green")
            packed = row_driver.pack_directory(item_directory(root, item)) if made else b""
            verdicts.record(state, name, 0 if made else exit_code, packed, made, printer)
            if exit_code != 0:
                failed = True
                break
            print(f"E171_SHARED_STEP item={item} taken=no key={key} seconds={time.monotonic() - started:.0f}", file=sys.stderr, flush=True)
        finish_code = verdicts.finish(state, printer)
    finally:
        shutil.rmtree(state, ignore_errors=True)
    if failed or (finish_code != 0 and not broken("item-finish-ignored")):
        return EXIT_STEP_FAILED
    return 0


def run_shared_step(arguments: argparse.Namespace) -> int:
    return judge_shared_steps([arguments.item], arguments.root, arguments.repo_root, arguments.device_features)


def fetch(arguments: argparse.Namespace) -> int:
    """库里有这一项（键相同、判绿）就解进共用输出根、退 0；没有退 EXIT_MISSING（文件头「跑法」fetch）。"""
    item, root, repo_root, features = arguments.item, arguments.root, arguments.repo_root, arguments.device_features
    if item == O3_CONTROL_ITEM and not prerequisites_from_the_library(root, repo_root, features):
        print(f"E171_SHARED_STEP_MISSING item={item} 库里没有 {PREREQUISITES}（这一项读它的产物）", file=sys.stderr)
        return EXIT_MISSING
    packed = library_entries([item], root, repo_root, features)[item]
    if packed is None:
        print(f"E171_SHARED_STEP_MISSING item={item} 库里没有这棵树、这组特性下键相同、判绿的这一项", file=sys.stderr)
        return EXIT_MISSING
    place_item(root, item, packed)
    return 0


def check_rows_done(arguments: argparse.Namespace) -> int:
    """表里每一行在输出根里都有做完的行目录（row-result.tsv 只在一行跑完时写）。"""
    numbers = [row.number for row in row_driver.read_selected_rows(arguments.selected_rows)]
    missing = [number for number in numbers if not os.path.isfile(os.path.join(arguments.rows_root, f"row-{number}", "row-result.tsv"))]
    if not missing:
        return 0
    shown = ",".join(str(number) for number in missing[:10]) + ("…" if len(missing) > 10 else "")
    print(f"E171_ROWS_MISSING rows_root={arguments.rows_root} missing={len(missing)} of={len(numbers)} rows={shown}", file=sys.stderr)
    return EXIT_MISSING


def copy_entries(source: str, destination: str, renamed: dict[str, str]) -> None:
    for name in sorted(os.listdir(source)):
        target = os.path.join(destination, renamed.get(name, name))
        if os.path.lexists(target):
            raise FileExistsError(target)
        if os.path.isdir(os.path.join(source, name)):
            shutil.copytree(os.path.join(source, name), target, symlinks=True)
        else:
            shutil.copy2(os.path.join(source, name), target)


def take(arguments: argparse.Namespace) -> int:
    """只取（不现判）：replay.sh 的 s2、s4 驱动取现成的前置（文件头「跑法」take）：五项照键从库里查（prerequisites 先查先解开，O3 阳性对照的键读它的产物），
    都在才逐项解进 --root、拷进驱动的目录；缺一项一份都不拷、一步都不做。"""
    root, repo_root, features = arguments.root, arguments.repo_root, arguments.device_features
    found: dict[str, bytes | None] = {}
    for group in ([PREREQUISITES], [item for item in SHARED_ITEMS if item != PREREQUISITES]):
        for item, packed in library_entries(group, root, repo_root, features).items():
            found[item] = packed
            if packed is not None:
                place_item(root, item, packed)
    missing = [item for item in SHARED_ITEMS if found[item] is None]
    if broken("take-ignores-the-library"):
        missing = [item for item in SHARED_ITEMS if not os.path.isdir(item_directory(root, item))]
    if missing:
        for item in missing:
            print(f"  ✗ E171 共用前置 {item} 库里没有这棵树、这组特性下键相同、判绿的这一项", file=sys.stderr)
        print(f"  → 怎么办：经统一任务服务交分片入口 e171-row-shard.sh 补判缺的那几项（单做一项的起法：{'；'.join(remake_command(root, item) for item in missing)}，"
              f"--root 换成分片入口的共用输出根），判完再取；这一趟一份都没拷、一步都没做", file=sys.stderr)
        return EXIT_MISSING
    renamed = {} if broken("e170-dump-not-renamed") else {E170_DUMP_SHARED_NAME: arguments.e170_dump_name}
    for item in SHARED_ITEMS:
        directory = item_directory(root, item)
        copy_entries(os.path.join(directory, DUMPS), arguments.dumps, renamed)
        copy_entries(os.path.join(directory, SCRATCH), arguments.scratch, {})
    print(f"E171_SHARED_STEPS_TAKEN items={len(SHARED_ITEMS)} root={root}", file=sys.stderr, flush=True)
    return 0


def take_rows(arguments: argparse.Namespace) -> int:
    """只取逐行驱动的行（文件头「跑法」take-rows；取法在 row_driver.py 文件头「只取」）：一行不缺退 0，有缺退 EXIT_MISSING。
    逐行驱动的弄坏开关只在单测与它自己的自证里认：命令行设了 E171_ROW_DRIVER_BREAK 退 EXIT_USAGE、一行都不查。"""
    if os.environ.get(row_driver.BREAK_VARIABLE) and not row_driver.BREAK_ACCEPTED_IN_PROCESS:
        print(f"  ✗ {row_driver.BREAK_VARIABLE} 只在逐行驱动的单测与自证里认，这一趟不是自证；一行都没查", file=sys.stderr)
        print(f"     → 怎么办：去掉 {row_driver.BREAK_VARIABLE} 再起", file=sys.stderr)
        return EXIT_USAGE
    missing = row_driver.take_rows_into_the_output_root(arguments, print_to_standard_error)
    return EXIT_MISSING if missing else 0


# ── 自证（文件头「自证」） ──

def written_item(item: str, root: str, repo_root: str, features: str) -> int:
    """selftest-round 里换掉 make_shared_step 的那一份：往项目录的 dumps/、scratch/ 里各写一份只随项名变的文件。"""
    directory = item_directory(root, item)
    if os.path.isdir(directory):
        shutil.rmtree(directory)
    for part in (DUMPS, SCRATCH):
        os.makedirs(os.path.join(directory, part))
        with open(os.path.join(directory, part, f"{ITEM_DIRECTORIES[item]}.txt"), "w", encoding="utf-8") as handle:
            handle.write(f"{part} of {item}\n")
    return 0


def run_selftest_round(tree_root: str) -> int:
    """selftest-round（自证 reuse-three-questions 的起法）：SELFTEST_ITEMS 两项在 tree_root 上一个计划里走一趟 judge_shared_steps（做一项换成 written_item）。"""
    root = tempfile.mkdtemp(prefix="e171-shared-steps-selftest-round-")
    try:
        return judge_shared_steps(list(SELFTEST_ITEMS), root, tree_root, "none", written_item)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest() -> int:
    """reuse-three-questions：long_run_item_verdicts.py reuse-cells 那六格，样板树是空的，起法是本文件的 selftest-round。"""
    verdicts = row_driver.item_verdicts_module()
    with row_driver.fake_cargo_ahead_on_the_path(), tempfile.TemporaryDirectory(prefix="e171-shared-steps-reuse-cells-") as work:
        template = os.path.join(work, "template")
        os.makedirs(template)
        command = [sys.executable, "-B", os.path.realpath(__file__), SELFTEST_ROUND_MODE, "{tree}"]
        red = verdicts.reuse_cells(SCRIPT_KEY, template, [LIBRARY_ITEM_NAMES[item] for item in SELFTEST_ITEMS], os.path.join(work, "cells"), 0, command)
    if red:
        print("  ✗ reuse-three-questions")
        print(f"     → 怎么办：照文件头「判定库」看 judge_shared_steps、library_entries 与 item_request_lines，再在临时服务底下单独起 {SCRIPT_KEY} 的 --selftest 到它退 0")
    else:
        print("  ✓ reuse-three-questions")
    print(f"SHARED_STEPS_SELFTEST cells=1 red={1 if red else 0}")
    return EXIT_STEP_FAILED if red else 0


def selftest_entry(argument_list: list[str]) -> int:
    """--selftest 与 selftest-round <树>：只在临时 prune-server 底下认（客户端 selftest_service_problem），不在退 2；在就认弄坏开关。"""
    global BREAK_ACCEPTED_IN_PROCESS
    client = row_driver.item_verdicts_module().prune_server_client
    problem = client.selftest_service_problem()
    if problem:
        print(f"  ✗ {argument_list[0]} 只在临时 prune-server 底下的自证里起：{problem}", file=sys.stderr)
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY}", file=sys.stderr)
        return EXIT_USAGE
    BREAK_ACCEPTED_IN_PROCESS = True
    if argument_list == ["--selftest"]:
        return selftest()
    if len(argument_list) == 2 and argument_list[0] == SELFTEST_ROUND_MODE:
        return run_selftest_round(argument_list[1])
    print(f"  ✗ 用法：shared_steps.py --selftest，或 shared_steps.py {SELFTEST_ROUND_MODE} <树>（只给 reuse-cells 起）", file=sys.stderr)
    return EXIT_USAGE


def main(argument_list: list[str]) -> int:
    if argument_list and argument_list[0] in ("--selftest", SELFTEST_ROUND_MODE):
        return selftest_entry(argument_list)
    if os.environ.get(BREAK_VARIABLE) and not BREAK_ACCEPTED_IN_PROCESS:
        print(f"  ✗ {BREAK_VARIABLE} 只在自证里认（test_shared_steps.py 在进程里调 main，临时 prune-server 底下的 --selftest），这一趟不是自证", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑", file=sys.stderr)
        return EXIT_USAGE
    parser = argparse.ArgumentParser(description="E171 整表那几行共用的前置")
    commands = parser.add_subparsers(dest="command", required=True)
    for name in ("run", "fetch"):
        command = commands.add_parser(name)
        command.add_argument("item", choices=SHARED_ITEMS)
    take_command = commands.add_parser("take")
    take_command.add_argument("--dumps", required=True)
    take_command.add_argument("--scratch", required=True)
    take_command.add_argument("--e170-dump-name", required=True)
    for command in (commands.choices["run"], commands.choices["fetch"], take_command):
        command.add_argument("--root", required=True)
        command.add_argument("--repo-root", required=True)
        command.add_argument("--device-features", required=True, choices=FEATURE_CHOICES)
    rows_command = commands.add_parser("rows-done")
    rows_command.add_argument("--selected-rows", required=True)
    rows_command.add_argument("--rows-root", required=True)
    # take-rows 的参数与 row_driver.py 现判那一趟同名（它的 main 在共用前置的判法闭包里，不往那里加只取的开关；row_driver.py 文件头「只取」）
    rows_taken = commands.add_parser("take-rows")
    for name in ("--repo-root", "--work-root", "--output-root"):
        rows_taken.add_argument(name, required=True)
    rows_taken.add_argument("--selected-rows", default=os.path.join(HERE, "bad-implementations-step-8.tsv"))
    rows_taken.add_argument("--rows", default=None)
    rows_taken.add_argument("--row-range", default=None)
    rows_taken.add_argument("--empty-mutation", action="store_true")
    rows_taken.add_argument("--empty-mutation-only", action="store_true")
    rows_taken.add_argument("--positive-control", choices=["O1-对拍", "O1-崩溃判官", "O2", row_driver.O3_CONTROL])
    rows_taken.add_argument("--disable-oracle", choices=["O1", "O2"])
    rows_taken.add_argument("--no-executor", action="store_true")
    rows_taken.add_argument("--o3-control-bad-states")
    rows_taken.add_argument("--snapshot-from")
    rows_taken.add_argument("--check-only", action="store_true", help="只查库、不往 --output-root 写行目录")
    rows_taken.set_defaults(first_slice=False)
    arguments = parser.parse_args(argument_list)
    if arguments.command == "rows-done":
        return check_rows_done(arguments)
    if arguments.command == "take-rows":
        if sum(option is not None and option is not False for option in (arguments.rows, arguments.row_range, arguments.empty_mutation_only)) > 1:
            rows_taken.error("--rows、--row-range、--empty-mutation-only 只给一样")
        if arguments.positive_control == row_driver.O3_CONTROL and not arguments.o3_control_bad_states:
            rows_taken.error("--positive-control O3 要同时给 --o3-control-bad-states")
        if arguments.positive_control == row_driver.O3_CONTROL and arguments.no_executor:
            rows_taken.error("--positive-control O3 与 --no-executor 不能一起")
        if arguments.row_range is not None:
            try:
                row_driver.parse_row_range(arguments.row_range)
            except ValueError as error:
                rows_taken.error(str(error))
        for name in ("repo_root", "work_root", "output_root"):
            setattr(arguments, name, os.path.abspath(getattr(arguments, name)))
        return take_rows(arguments)
    arguments.root = os.path.abspath(arguments.root)
    arguments.repo_root = os.path.abspath(arguments.repo_root)
    if arguments.command == "run":
        return run_shared_step(arguments)
    if arguments.command == "fetch":
        return fetch(arguments)
    arguments.dumps = os.path.abspath(arguments.dumps)
    arguments.scratch = os.path.abspath(arguments.scratch)
    return take(arguments)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
