#!/usr/bin/env python3
"""拿两份编译各跑一次 `attack r3-dump` 的原料比（m3-prune-gpu-r3 云端攻方腿的原型，不入库）。

A1 被验的形态：身份 = <代码文件名>::<验证目标名>::<流程步号>::<哈希串>，重录只看属性「写出的内容哈希」变没变
（「只改 core 只重录内容哈希变了的那一步与它下面的分支」）。两份原料里文件名、目标名、路径号、哈希串（流程定义）逐字相同，
流程步号 = 步.步内第几段.段内各次写的落法。这里对「写出的内容哈希」的三种读法各算一遍：
  final_sectors               这一步写完之后每个被写过的扇区留下的字节（次序、屏障、FUA、种类都不进）
  write_list                  这一步的写按录制次序逐次（盘、偏移、长度、字节）
  write_table_with_barriers   收严的：连种类、FUA 与段边界（屏障）一起
一步连同它前面的每一步（与 mkfs 那份基线）属性都没变，就照 KV 里旧编译的结果用（不重录、不重核）；
数「KV 给出的判定与这份编译真跑出来的不同」的状态（stale），其中真跑是红、KV 给的是绿或根本没有这个身份的（missed_red）。
用法：compare_dumps.py <名字> <旧编译的原料> <新编译的原料>
"""
import re
import sys

CANDIDATES = ["final_sectors", "write_list", "write_table_with_barriers"]


def fields(line):
    return dict(re.findall(r"(\w+)=(\S+)", line))


def load(path):
    base, steps, states = None, [], {}
    for line in open(path, encoding="utf-8"):
        if line.startswith("E7RESULT name=r3_dump_base "):
            base = fields(line)
            base["segments"] = line.split(" segments=", 1)[1].strip()
        elif line.startswith("E7RESULT name=r3_dump_step "):
            steps.append(fields(line))
        elif line.startswith("E7RESULT name=r3_dump_state "):
            record = fields(line)
            red = line.split(" red=", 1)[1].split(" full=", 1)[0]
            states[record["identity"]] = (record["status"], red != "[]", record["full"])
    return base, steps, states


def main():
    name, old_path, new_path = sys.argv[1:4]
    old_base, old_steps, old_states = load(old_path)
    new_base, new_steps, new_states = load(new_path)
    mkfs_same = old_base["final_sectors"] == new_base["final_sectors"]
    segments_same = old_base["segments"] == new_base["segments"]
    verdict_differs = sum(1 for identity in new_states if identity in old_states and old_states[identity][0] != new_states[identity][0])
    full_differs = sum(1 for identity in new_states if identity in old_states and old_states[identity][2] != new_states[identity][2])
    only_new = sum(1 for identity in new_states if identity not in old_states)
    only_old = sum(1 for identity in old_states if identity not in new_states)
    parts = [f"name=r3_attribute_reuse world={name} mkfs_base_same={str(mkfs_same).lower()} segment_lengths_same={str(segments_same).lower()}",
             f"states_old={len(old_states)} states_new={len(new_states)} red_old={sum(red for _, red, _ in old_states.values())} red_new={sum(red for _, red, _ in new_states.values())}",
             f"same_identity_verdict_differs={verdict_differs} same_identity_full_digest_differs={full_differs} identities_only_in_new={only_new} identities_only_in_old={only_old}"]
    stale_by_candidate = {}
    for candidate in CANDIDATES:
        reused_steps = []
        prefix_same = mkfs_same
        for old_step, new_step in zip(old_steps, new_steps):
            prefix_same = prefix_same and old_step[candidate] == new_step[candidate]
            if prefix_same:
                reused_steps.append(int(new_step["step"]))
        stale = missed_red = stale_full = 0
        for identity, (status, red, full) in new_states.items():
            step = int(identity.split(".")[0]) if identity != "end" else len(new_steps) - 1
            if step not in reused_steps:
                continue
            kv = old_states.get(identity)
            stale_full += int(kv is None or kv[2] != full)
            if kv is None or kv[0] != status:
                stale += 1
                if red and (kv is None or not kv[1]):
                    missed_red += 1
        stale_by_candidate[candidate] = stale
        parts.append(f"{candidate}:steps_reused={'/'.join(map(str, reused_steps)) or 'none'}:stale={stale}:stale_full_digest={stale_full}:missed_red={missed_red}")
    changed_steps = [int(new["step"]) for old, new in zip(old_steps, new_steps) if old["write_table_with_barriers"] != new["write_table_with_barriers"]]
    parts.append(f"steps_whose_write_table_changed={'/'.join(map(str, changed_steps)) or 'none'}")
    parts.append(f"must_be_nonzero={stale_by_candidate['write_list']}")
    print("E7RESULT " + " ".join(parts))


if __name__ == "__main__":
    main()
