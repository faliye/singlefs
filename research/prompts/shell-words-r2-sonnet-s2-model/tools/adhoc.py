import sys; sys.dont_write_bytecode = True
import sys, json, os
sys.path.insert(0,os.path.dirname(os.path.abspath(__file__)))
import run
def go(ident, cmd, cwd='copy', bg=False):
    c=dict(id='adhoc',ident=ident,cmd=cmd,h=0,d=0,cites=[],opt=dict(cwd=cwd,bg=bg))
    r=run.one(c); print(ident,cwd,repr(cmd[:100]),'H',r['rc_h'],'D',r['rc_d'],'|',r['err_h'][:200] if r['rc_h'] else '', '|', r['err_d'][:200] if r['rc_d'] else '')
if __name__=='__main__':
    go(sys.argv[1], sys.argv[2], sys.argv[3] if len(sys.argv)>3 else 'copy')
