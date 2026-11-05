# 补充：H3 的一行写法与换行写法（bash 读完一整行才执行，同一行里刚定义的别名不生效）；以及两个我自己加的对照
HK = "heavy-test-guard.sh"
RAW = [
 ("H3-一行-红?", "shopt -s expand_aliases; alias t='cargo test --all'; t"),
 ("H3-换行-红", "shopt -s expand_aliases\nalias t='cargo test --all'\nt"),
 ("H3-换行-绿", "shopt -s expand_aliases\nalias t='ls'\nt"),
 ("H3-一行-无shopt", "alias t='cargo test --all'; t"),
 ("H3-换行-无shopt", "alias t='cargo test --all'\nt"),
 ("H3-eval-红", "shopt -s expand_aliases; alias t='cargo test --all'; eval t"),
 ("H3-bash-c-红", "bash -c \"shopt -s expand_aliases; alias t='cargo test --all'; t\""),
 ("A2-换行形", "f(){ bash -c true; }\nBASH_ENV=e.sh f"),
 ("H5-&&-不起", "cd missing-dir && bash heavy.sh"),
 ("H5-||-不起", "cd missing-dir || exit 1; bash heavy.sh"),
 ("H5-分号-起", "cd missing-dir; bash heavy.sh"),
]
CASES = [(lab, HK, ag, cmd) for lab, cmd in RAW for ag in ("main", "implementation-writer")]
