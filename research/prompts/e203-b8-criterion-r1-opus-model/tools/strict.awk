{ n=split($0,seg," [|] "); head=seg[1]; for(i=2;i<=n;i++){split(seg[i],t," "); split(t[1],hh,":"); d=hh[1]; t1=hh[2]; split(t1,f,"="); split(t[2],p,"="); split(t[3],m,"="); split(t[4],w,"="); F[d]=f[2];P[d]=p[2];M[d]=m[2];W[d]=w[2]}
 for(d in F){ if(d=="raw")continue; c++; if(F[d]+0>F["raw"]+0.0005){up++; print head,d,"F",F[d],"raw",F["raw"]} c++; if(P[d]+0>P["raw"]+0.0005){up++; print head,d,"P",P[d],"raw",P["raw"]} c++; if(M[d]+0>M["raw"]+0.0005){up++; print head,d,"M",M[d],"raw",M["raw"]} c++; if(W[d]+0>W["raw"]+0.0005){up++;print head,d,"w",W[d],"raw",W["raw"]} }
}
END{print "cells="c,"ratio_alpha_above_raw="up+0}
