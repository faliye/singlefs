import sys, pathlib
path, old, new = sys.argv[1], sys.argv[2].encode().decode('unicode_escape').encode('latin1').decode('utf-8'), sys.argv[3].encode().decode('unicode_escape').encode('latin1').decode('utf-8')
text = pathlib.Path(path).read_text()
count = text.count(old)
if count != 1:
    sys.exit(f"hit {count} times")
pathlib.Path(path).write_text(text.replace(old, new))
print("applied", path)
