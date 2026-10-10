#!/usr/bin/env python3
"""J3 #8：把 69 条不变量的期望评估数（按 new_pool_file_creation_stream.rs 的分类表算）与 2026-10-04 主工作区 checker 档日志 CHECKER 行逐条比。
用法：check_pin8.py <树根> <日志> [<日志里 CHECKER 行号>]
"""
import re, sys
tree, log = sys.argv[1], sys.argv[2]
src = open(f"{tree}/crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs", encoding="utf-8").read()
def lst(name):
    m = re.search(r"let %s\s*=\s*\[(.*?)\];" % name, src, re.S)
    return re.findall(r'"(I-[0-9.]+)"|(MAPPING_KEY_MATCHES_THE_UNIT_HEADER)', m.group(1))
def names(name):
    return [("I-1.11" if (b and not a) else (a or b)) for a, b in lst(name)]  # MAPPING_KEY_MATCHES_THE_UNIT_HEADER 的值是 I-1.11（grep 核过）
cats = {
 "never": names("never_comparable_on_this_stream"),
 "newroot": names("only_under_the_new_root"),
 "gen": names("under_the_new_root_or_through_the_record_the_next_mount_applies_first"),
 "rec": names("judged_once_a_record_is_on_disk"),
 "rec_root": names("judged_once_a_record_and_the_root_of_its_publish_are_on_disk"),
 "rec_file": names("judged_once_a_record_of_the_file_publish_is_on_disk"),
 "two": names("judged_once_a_device_holds_two_records_of_one_instance"),
 "replay": names("judged_only_when_the_replay_applies_a_record"),
 "mkfs1": names("judged_unless_every_device_has_one_choosable_slot_written_by_mkfs"),
}
def const(n): return int(re.search(r"const %s: u64 = (\d+);" % n, src).group(1))
every = 46
C = dict(root=const("ROOT_PERSISTED_STATES"), empty=const("STATES_WITH_AN_EMPTY_JOURNAL_RING"),
 w1=const("STATES_WITH_ONLY_RECORDS_OF_THE_FIRST_WARM_UP_BEFORE_ITS_ROOT"),
 f=const("STATES_WITH_A_RECORD_OF_THE_FILE_PUBLISH_BUT_NOT_ITS_ROOT"),
 two=const("STATES_BEFORE_A_DEVICE_HOLDS_TWO_RECORDS_OF_ONE_INSTANCE"),
 rp=const("STATES_WHERE_THE_REPLAY_APPLIES_A_RECORD"),
 m=const("STATES_WHERE_EVERY_DEVICE_HAS_ONE_CHOOSABLE_SLOT_WRITTEN_BY_MKFS"))
line = [l for l in open(log, encoding="utf-8", errors="replace") if l.startswith("CHECKER ")][0]
got = {k: (int(a), int(b)) for k, a, b in re.findall(r"(I-[0-9.]+|MAPPING_KEY_MATCHES_THE_UNIT_HEADER)=(\d+)/(\d+)", line)}
# 注意：MAPPING 常量在日志里的名字可能不同，下面按「名字不在 got 里」报出来
exp = {}
def expected(n, gen_states):
    if n in cats["never"]: return 0
    if n in cats["newroot"]: return C["root"]
    if n in cats["gen"]: return gen_states
    if n in cats["rec"]: return every - C["empty"]
    if n in cats["rec_root"]: return every - C["empty"] - C["w1"]
    if n in cats["rec_file"]: return C["root"] + C["f"]
    if n in cats["two"]: return every - C["two"]
    if n in cats["replay"]: return C["rp"]
    if n in cats["mkfs1"]: return every - C["m"]
    return every
gen_states = got["I-3.10"][0]  # 由调用方逐状态现算，这里取日志值
bad = 0
for n, (ev, vio) in got.items():
    e = expected(n, gen_states)
    if (ev != e) or vio != 0:
        bad += 1; print("DIFF", n, "log", ev, vio, "expected", e)
print("invariants_in_log", len(got), "categories", {k: len(v) for k, v in cats.items()}, "consts", C)
print("mismatches", bad)
