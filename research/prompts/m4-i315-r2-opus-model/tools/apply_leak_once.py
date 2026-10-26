"""在 apply_arms.py 打过的副本上再加一个缺陷开关 ATTACK_DEFECT=leak_once：只在 txg 5 那一次发布把第一个没隔离的换下落点在两块盘上都留在已分配。
用法：python3 -I apply_leak_once.py <副本根>（先跑 apply_arms.py）"""
import sys, pathlib
tx = pathlib.Path(sys.argv[1]) / "crates/singlefs-core/src/transaction.rs"
t = tx.read_text()
old = '                "leak_every" => txg.0 % 7 == 3,\n'
if t.count(old) != 1:
    raise SystemExit(f"leak_once: 旧串命中 {t.count(old)} 次")
tx.write_text(t.replace(old, old + '                "leak_once" => txg.0 == 5,\n'))
print("ok")
