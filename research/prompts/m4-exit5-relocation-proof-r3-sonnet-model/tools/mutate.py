import sys
src, dst, old, new = sys.argv[1:5]
t = open(src).read()
assert t.count(old) == 1, (t.count(old), old)
open(dst, 'w').write(t.replace(old, new))
