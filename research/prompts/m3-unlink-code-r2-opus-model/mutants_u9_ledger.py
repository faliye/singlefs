import sys
orig=open('/tmp/claude-1000/m3-unlink-r2-attack/mount.rs.orig').read()
which=sys.argv[1]
s=orig
if which=='MU1':
    a='''            .map(|version| version.writes().clone())
            .chain(
                space_admission
                    .floor_raises()
                    .iter()
                    .flat_map(|raised| raised.publishes.iter())
                    .map(|publish| publish.writes.clone()),
            )
            .collect();
    let DeletionQueueContinuation {'''
    b='''            .map(|version| version.writes().clone())
            .collect();
    let DeletionQueueContinuation {'''
elif which=='MU2':
    a='''            .steps_published
            .iter()
            .chain(
                self.floor_raises
                    .iter()
                    .flat_map(|raised| raised.publishes.iter()),
            )
            .collect();
        persisted.sort_by_key'''
    b='''            .steps_published
            .iter()
            .collect();
        persisted.sort_by_key'''
elif which=='MU3':
    a='''        persisted.sort_by_key(|publish| publish.root.checkpoint_txg);
'''
    b=''
elif which=='orig':
    a=b=''
else:
    raise SystemExit('which')
if a:
    assert s.count(a)==1, which
    s=s.replace(a,b,1)
open('/tmp/claude-1000/m3-unlink-r2-attack/tree/crates/singlefs-core/src/mount.rs','w').write(s)
print('applied',which)
