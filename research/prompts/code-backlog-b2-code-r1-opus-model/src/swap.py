"""Order-swap mutant of slots_of_a_placement_to_isolate_now: dedup before exemption (scratch copies only).
Usage: python3 -I swap.py <mount.rs>"""
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
old = ".filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))"
new = ".filter(|covered| isolated.insert(*covered) && !exempt.contains(covered))"
if text.count(old) != 1:
    sys.exit(f"anchor hit {text.count(old)} times")
open(path, "w", encoding="utf-8").write(text.replace(old, new))
print("swapped", path)
