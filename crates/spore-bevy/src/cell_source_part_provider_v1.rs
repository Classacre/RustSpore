//! Explicit single-package Cell part PROP/Parent provider.
//!
//! The caller chooses and authenticates the package. This checked provider
//! does not implement native mount priority, fallback, cached manager state or
//! callback behavior. It rejects missing or ambiguous parents, cycles and
//! unsupported framing instead of inventing a class.
use dbpf::native_index::{NativePackageIndex, NativeResourceKey};
use sha2::{Digest, Sha256};
use spore_core::{cell_part_classification::{classify_cell_part, CellPartPropertyList},
    prop::{self, PropValue}};
use std::collections::BTreeSet;

const PROP_TYPE:u32=0x00b1b104;
const MAX_DEPTH:usize=64;
#[derive(Debug)]
pub struct SourceNode {pub key:NativeResourceKey,pub sha256:String,pub identifiers:Vec<u32>,pub parent:Option<NativeResourceKey>}
#[derive(Debug)]
pub struct Classification {pub class:u32,pub chain:Vec<SourceNode>}

fn parent_key(current:NativeResourceKey, low:u32, scope:u32)->NativeResourceKey {
    NativeResourceKey {instance_id:low,type_id:PROP_TYPE,group_id:if scope==0 {current.group_id} else {scope}}
}

fn parse_node(key:NativeResourceKey, raw:&[u8])->Result<SourceNode,String> {
    if raw.len()<4{return Err(format!("invalid property extent {key}"))}
    let declared=u32::from_be_bytes(raw[..4].try_into().unwrap()) as usize;
    let decoded=prop::parse(raw).map_err(|e|format!("property {key}: {e}"))?;
    let ids:Vec<_>=decoded.items.iter().map(|item|item.id).collect();
    if !decoded.exact || decoded.consumed!=raw.len() || decoded.items.len()!=declared || decoded.items.iter().any(|item|item.anonymous) || !ids.windows(2).all(|w|w[0]<w[1]) {
        return Err(format!("unsupported named property framing {key}"));
    }
    let at=ids.partition_point(|id|*id<prop::idents::PARENT);
    let parent=if ids.get(at)==Some(&prop::idents::PARENT){
        let item=&decoded.items[at];
        if item.prop_type!=prop::types::KEY{return Err(format!("invalid Parent type {key}"))}
        let Some(PropValue::Key(low,_ignored_type,scope))=item.values.first() else{return Err(format!("invalid Parent value {key}"))};
        Some(parent_key(key,*low,*scope))
    }else{None};
    Ok(SourceNode{key,sha256:format!("{:x}",Sha256::digest(raw)),identifiers:ids,parent})
}

fn resolve<F>(root:NativeResourceKey,mut fetch:F)->Result<Classification,String>
where F:FnMut(NativeResourceKey)->Result<SourceNode,String> {
    if root.type_id!=PROP_TYPE{return Err("root is not PROP".into())}
    let mut key=root;let mut visited=BTreeSet::new();let mut chain=Vec::new();
    loop {
        if chain.len()>=MAX_DEPTH{return Err("property Parent depth limit".into())}
        if !visited.insert(key){return Err(format!("property Parent cycle at {key}"))}
        let node=fetch(key)?;
        if node.key!=key{return Err("provider returned a different property key".into())}
        let parent=node.parent;chain.push(node);
        match parent {Some(next)=>key=next,None=>break}
    }
    let lists:Vec<_>=chain.iter().enumerate().map(|(i,node)|CellPartPropertyList{
        identifiers:&node.identifiers,parent:node.parent.map(|_|i+1)
    }).collect();
    let class=classify_cell_part(&lists,Some(0)).map_err(|e|e.to_string())?;
    Ok(Classification{class,chain})
}

/// Reads only the explicitly supplied package. Missing Parent entries do not
/// become native missing-list/class-one results: that would assume fallback.
pub fn classify(index:&NativePackageIndex,root:NativeResourceKey)->Result<Classification,String>{
    resolve(root,|key|{
        let entry=index.index().find_unique(key).map_err(|e|e.to_string())?.ok_or_else(||format!("unresolved property {key} in {}",index.path().display()))?;
        if entry.decompressed_size>16*1024*1024 || entry.stored_size>16*1024*1024{return Err(format!("oversized property {key}"))}
        let raw=index.read(entry).map_err(|e|e.to_string())?;
        if raw.len()!=entry.decompressed_size as usize{return Err(format!("invalid property extent {key}"))}
        parse_node(key,&raw)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(id:u32,scope:u32)->NativeResourceKey {NativeResourceKey{instance_id:id,type_id:PROP_TYPE,group_id:scope}}
    fn node(k:NativeResourceKey,p:Option<NativeResourceKey>)->SourceNode {SourceNode{key:k,sha256:String::new(),identifiers:vec![],parent:p}}
    #[test] fn parent_scope_and_error_boundaries() {
        let a=key(1,7);let b=key(2,7);
        assert_eq!(parent_key(a,2,0),b);
        assert_eq!(parent_key(a,2,9),key(2,9));
        let cycle=resolve(a,|k|Ok(node(k,Some(if k==a {b}else{a})))).unwrap_err();
        assert!(cycle.contains("cycle"));
        let missing=resolve(a,|k|if k==a {Ok(node(k,Some(b)))} else {Err("missing parent".into())}).unwrap_err();
        assert_eq!(missing,"missing parent");
        let duplicate=resolve(a,|_|Err("duplicate key".into())).unwrap_err();
        assert_eq!(duplicate,"duplicate key");
        assert!(parse_node(a,&[0,0,0,1]).is_err());
    }
}
