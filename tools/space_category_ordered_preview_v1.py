"""Inspect a Space candidate graph in caller-supplied package order.

Index presence is a controlled first-hit approximation, not proof that native
virtual +34 succeeds, that the package is mounted, or that the game selected it.
All paths must stay stable; hashes cannot guarantee an atomic snapshot.
"""
import argparse
import hashlib
import json
import struct
from pathlib import Path
if not __debug__: raise RuntimeError('normal Python required')
from native_archive_index_probe import decode
from space_category_candidate_graph_v1 import read_shape, decompress_bounded, hash_file
ROOT=0x2DB6DAD3
TYPE=0x00B1B104
GROUP=0xCC489C6F

def select_first(indexes, key):
    """First present exact key; duplicates within one package are rejected."""
    for ordinal, rows in enumerate(indexes):
        hits=[row for row in rows if row[:3]==key]
        if len(hits)>1: raise ValueError('duplicate key in selected package')
        if hits: return ordinal, hits[0]
    raise ValueError('candidate missing from supplied packages')

def inspect(packages):
    if not 1<=len(packages)<=8: raise ValueError('requires 1..8 packages')
    resolved=[p.resolve() for p,_ in packages]
    if len(set(resolved))!=len(resolved): raise ValueError('duplicate package path')
    snapshots=[]; indexes=[]
    for (path,expected),absolute in zip(packages,resolved):
        before,size=hash_file(absolute)
        if before.lower()!=expected.lower(): raise ValueError('package pin')
        with absolute.open('rb') as f:
            head=f.read(96)
            if len(head)!=96 or head[:4]!=b'DBPF': raise ValueError('invalid header')
            count,length,offset=(struct.unpack_from('<I',head,x)[0] for x in (36,44,64))
            if count>1_000_000 or length>128*1024**2 or offset+length>size: raise ValueError('index bound')
            f.seek(offset);block=f.read(length)
            if len(block)!=length: raise ValueError('short index')
        flags,rows=decode(block,count);indexes.append(rows)
        snapshots.append({'path':absolute,'sha256':before,'size':size,'index_flags':flags})
    def load(instance):
        ordinal,row=select_first(indexes,(instance,TYPE,GROUP))
        _,_,_,offset,stored,decoded,compression,_,_=row; snap=snapshots[ordinal]
        if not (0<stored<=1024**2 and 0<decoded<=1024**2 and offset+stored<=snap['size']): raise ValueError('resource bound')
        with snap['path'].open('rb') as f:f.seek(offset);raw=f.read(stored)
        if len(raw)!=stored: raise ValueError('short resource')
        if compression==0 and stored==decoded:data,used=raw,stored
        elif compression==0xffff and stored<decoded:data,used=decompress_bounded(raw,decoded)
        else:raise ValueError('unsupported compression')
        if len(data)!=decoded or not 0<=used<=stored:raise ValueError('decoded extent')
        return {'instance':f'{instance:08X}','package_ordinal':ordinal,'payload_sha256':hashlib.sha256(data).hexdigest(),
                'stored':stored,'decoded':decoded,'consumed':used,'unconsumed_hex':raw[used:].hex(),'shape':read_shape(data)}
    root=load(ROOT);words=root['shape']['words_le']
    if root['shape']['kind']!='array' or len(words)!=5 or any(w[1:]!=[0,0] for w in words) or len({w[0] for w in words})!=5:
        raise ValueError('unsupported root')
    children=[load(w[0]) for w in words]
    for snap in snapshots:
        if hash_file(snap['path'])!=(snap['sha256'],snap['size']):raise ValueError('package changed')
    return {'status':'matched_static','scope':'caller-supplied order, exact index first-hit approximation; no native provider, cache, active mount or live galaxy claim',
            'framing_note':'unconsumed bytes reported; original decoder acceptance unverified',
            'packages':[{'ordinal':i,'name':s['path'].name,'sha256':s['sha256'],'index_flags':s['index_flags']} for i,s in enumerate(snapshots)],
            'root':root,'children':children}

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--package',action='append',nargs=2,metavar=('PATH','SHA256'),required=True,help='repeat in explicit desired order')
    p.add_argument('--admit-stable-files',action='store_true',required=True)
    a=p.parse_args();print(json.dumps(inspect([(Path(path),sha) for path,sha in a.package]),indent=2))
if __name__=='__main__':main()
