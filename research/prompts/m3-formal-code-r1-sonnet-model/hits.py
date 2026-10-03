import sys
root=sys.argv[1]
rows=open(root+'/crates/mutations.tsv',encoding='utf-8').read().split('\n')
for n in (173,174,177):
    f=rows[n-1].split('\t')
    orig=f[2].replace('\\n','\n')          # 表头约定：原文里的 \n 表示换行
    text=open(root+'/'+f[1],encoding='utf-8').read()
    raw=text.count(f[2])                    # 不换行、按字面反斜杠 n 数（实现员的数法可能是这样）
    print('line',n,'file',f[1],'hits_after_newline_conversion=',text.count(orig),'hits_raw_backslash_n=',raw)
