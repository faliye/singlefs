#!/usr/bin/env python3
"""把 model.py 拷进草稿目录逐条改坏、跑 --selftest，每条都必须非 0 退出（证明 selftest 钉的数会红）。
用法：python3 mutate.py [草稿目录，默认 /tmp/claude-1000/alloc-basis-r1-opus/mut]"""
import pathlib
import shutil
import subprocess
import sys

MUTATIONS = [
    ("M1 记账按持久之前的环算（post 退回 pre）", "        ring_after[root.position] = root\n", ""),
    ("M2 第一个事务走通用释放路径（m2 在 A 里被释放）", 'releases = [] if label == "A" else ', "releases = "),
    ("M3 窄读法不减有效根引用的槽", "        return slots_of(abandoned) - slots_of(valid)\n", "        return slots_of(abandoned)\n"),
    ("M4 F_生效取各盘最大值的最大值", "    return min(per_device_maximum)", "    return max(per_device_maximum)"),
    ("M5 checker 并集不按 (槽, 跨度) 去重", "        return sum(span for _slot, span in keys)",
     "        return sum(span for _slot, span in keys) + len(root_list)"),
    ("M6 回退根的 F 一律照抄 R_old", 'new_floor = selected.rollback_floor if self.settings.rollback_root_floor == "selected_root" else floor_before',
     "new_floor = selected.rollback_floor"),
    ("M7 实例表有效性判反（T ≤ Ti 写成 T < Ti）", "            return root.txg <= row_txg", "            return root.txg < row_txg"),
]


def main():
    draft = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "/tmp/claude-1000/alloc-basis-r1-opus/mut")
    draft.mkdir(parents=True, exist_ok=True)
    source = pathlib.Path(__file__).with_name("model.py").read_text()
    all_red = True
    for name, old, new in MUTATIONS:
        count = source.count(old)
        if count != 1:
            print(f"✗ {name}：锚点命中 {count} 次，这条变异没跑")
            print("→ model.py 改过了：把锚点改成在新源码里恰好命中一次的串")
            all_red = False
            continue
        target = draft / "model.py"
        target.write_text(source.replace(old, new))
        result = subprocess.run([sys.executable, str(target), "--selftest"], capture_output=True, text=True, cwd=draft)
        first_line = (result.stdout + result.stderr).strip().splitlines()[0] if (result.stdout + result.stderr).strip() else ""
        print(f"{'✓ 红' if result.returncode != 0 else '✗ 没红'}  {name}  退出码 {result.returncode}  {first_line[:90]}")
        all_red = all_red and result.returncode != 0
    shutil.rmtree(draft)
    print(f"共 {len(MUTATIONS)} 条变异，{'全部被抓' if all_red else '有没被抓或没跑的'}")
    return 0 if all_red else 1


if __name__ == "__main__":
    sys.exit(main())
