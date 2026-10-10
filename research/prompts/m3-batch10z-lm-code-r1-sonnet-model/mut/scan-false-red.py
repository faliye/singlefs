# 行为不变的改写喂扫源码模型：B1 空白、B2 局部变量改名、B3 .as_deref() 的等价写法。B2、B3 另用 cargo check 证明编得过（logs/scan-false-red.txt 末两行）。
import re, subprocess, os
W=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1-sonnet')
T=os.path.expanduser('~/.local/share/singlefs/work/m3-batch10z-lm-code-r1/tree')
f=T+'/crates/singlefs-checker-tier/src/gpu_unit_checks.rs'
src=open(f,encoding='utf-8').read()
sig='fn take_in_the_devices_found_idle(&self) {'
i=src.index(sig); j=src.index('\n            }\n',i)
body=src[i:j]
new=body.replace('started','kept_device')
new=new.replace('.kept_device.kept_device_now()','.started.started_now()').replace('.kept_device.','.started.')
cases=[('B2 重命名局部变量 started -> kept_device（行为不变）',src[:i]+new+src[j:]),
       ('B1 闭包参数后多一个空格（空白，应当仍绿）',src[:i]+body.replace('|started: &GpuCard|','|started: &GpuCard |')+src[j:]),
       ('B3 .as_deref() 换成 .as_ref().map(|device| &**device)（行为不变）',src[:i]+body.replace('.as_deref()','.as_ref().map(|device| &**device)')+src[j:])]
for name,text in cases:
    p=W+'/scanmut/b.rs'; open(p,'w',encoding='utf-8').write(text)
    print(name); print(subprocess.run([W+'/target-scan/release/scan-model','gpu',p],capture_output=True,text=True).stdout.strip()[:200])
