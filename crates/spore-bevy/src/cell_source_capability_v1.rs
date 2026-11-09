//! Controlled source creation -> cache composition; no live provider/cache claim.
use std::collections::BTreeMap;
use spore_core::{
    cell_capability_curve::sample_capability_curve,
    cell_capability_geometry::{derive_cell_capability_geometry, CellCapabilityGeometry, CellCapabilityPart},
    cell_capability_scalar::{derive_cell_capability_scalars, CellCapabilityScalarContext, CellCapabilityScalarGlobals, CellCapabilityScalars, INITIAL_ELECTRIC_RECHARGE_CURVE},
    cell_globals::CellGlobals, cell_part_geometry::{build_cell_part, NativeCellPart},
    creature_data, native_sse::{self,Policy,SseError}, native_x87::Context,
    prop::{self,PropValue},
};
#[derive(Debug)]
pub struct Error(pub String);
impl From<SseError> for Error {fn from(e:SseError)->Self{Self(format!("SSE: {e}"))}}

#[derive(Debug)]
pub struct Capability {pub geometry:CellCapabilityGeometry,pub scalars:CellCapabilityScalars,pub row_count:usize, first_word:u32, rows:Vec<NativeCellPart>}
impl Capability {
    /// Ordered, controlled model outputs, not a live E67A50 cache or actor binding.
    pub fn rows(&self)->&[NativeCellPart] { &self.rows }
    /// Admit an actor's independently supplied post-override triple against this
    /// controlled cache snapshot. A first-word hit does not prove a live cache,
    /// provider, actor, full-triple identity, or snapshot lifetime.
    pub fn feature_snapshot(&self, actor_triple:[u32;3])->Result<FeatureSnapshot<'_>, &'static str> {
        FeatureSnapshot::new(self.first_word, &self.rows, actor_triple)
    }
}

/// Borrowed, immutable feature-loop input; indices remain native row indices.
#[derive(Clone,Copy)]
pub struct FeatureSnapshot<'a> { key:u32, rows: &'a [NativeCellPart] }
impl<'a> FeatureSnapshot<'a> {
    pub(crate) fn new(cache_key:u32, rows:&'a [NativeCellPart], actor_triple:[u32;3])->Result<Self, &'static str> {
        if actor_triple[0] == 0 { return Err("zero actor key does not admit feature cache"); }
        if actor_triple[0] != cache_key { return Err("actor/cache first-word mismatch"); }
        Ok(Self { key:cache_key, rows })
    }
    pub(crate) fn matches_actor(&self, actor_triple:[u32;3])->bool { actor_triple[0] != 0 && actor_triple[0] == self.key }
    /// E6ADE0 skips class 1, but its feature index still counts skipped rows.
    /// Geometry, contact acceptance and callback effects remain unmodeled.
    pub fn candidates(&self)->impl Iterator<Item=(usize, &'a NativeCellPart)> + '_ {
        self.rows.iter().enumerate().filter(|(_, row)| row.class != 1)
    }
}

/// All input bytes and parent classifications must already be authenticated by
/// the caller. The zero empty-center and initialized electric ID are explicitly
/// controlled values, not observed live state. FP scope is restored on error.
pub fn build(creation:&[u8],first_word:u32,classes:&[u32],globals:CellGlobals<'_>,curve:&[u8])->Result<Capability,Error>{
    let creature=creature_data::parse(creation).map_err(|e|Error(format!("creation: {e}")))?;
    if !creature.exact || creature.consumed!=creation.len(){return Err(Error("creation framing".into()))}
    creature.validate_part_ranges().map_err(|e|Error(format!("creation ranges: {e}")))?;
    let count=creature.parts.len().min(64);
    if classes.len()!=count{return Err(Error("class count mismatch".into()))}
    let tree=prop::parse(curve).map_err(|e|Error(format!("curve PROP: {e}")))?;
    if !tree.exact || tree.consumed!=curve.len() || tree.items.len()!=18{return Err(Error("curve framing".into()))}
    let mut curves=BTreeMap::new();let mut total=0usize;
    for item in tree.items {
        if item.anonymous || item.prop_type!=0x30 || item.specifier!=0x9c{return Err(Error("unsupported curve type".into()))}
        let mut points=Vec::new();
        for value in item.values {match value {PropValue::Vector2(x,y)=>points.push([x,y]),_=>return Err(Error("unsupported curve point".into()))}}
        total+=points.len();if curves.insert(item.id,points).is_some(){return Err(Error("duplicate curve".into()))}
    }
    if total!=104{return Err(Error("unexpected curve count".into()))}
    for id in [0x77540c34,0x2c574746,0xbbab1bf5,0x2ef68471,0xe5015d0c,INITIAL_ELECTRIC_RECHARGE_CURVE,0x03ab589e,0xaee2835e,0x92baae19] {
        if !curves.contains_key(&id){return Err(Error(format!("unresolved curve {id:08X}")))}
    }
    let context=Context::from_control_word(0x027f).map_err(|e|Error(format!("x87: {e:?}")))?;
    native_sse::scoped(Policy::Mxcsr1f80,||{
        let mut rows=Vec::with_capacity(count);
        for (i,&class) in classes.iter().enumerate(){
            let view=creature.part_view(i).ok_or_else(||Error("missing part row".into()))?;
            let row:&[u8;140]=view.raw_row.try_into().map_err(|_|Error("part row size".into()))?;
            let part=build_cell_part(row,class,first_word).map_err(|e|Error(format!("part {i}: {e}")))?;
            rows.push(part);
        }
        let parts:Vec<CellCapabilityPart>=rows.iter().copied().map(CellCapabilityPart::from).collect();
        let geometry=derive_cell_capability_geometry(&parts,[0.;3],context).map_err(|e|Error(format!("geometry: {e}")))?;
        let g=CellCapabilityScalarGlobals{cilia_speed_as_jet:globals.cilia_speed_as_jet(),flagella_speed_as_jet:globals.flagella_speed_as_jet(),flagella_speed_as_cilia:globals.flagella_speed_as_cilia(),cilia_speed_as_flagella:globals.cilia_speed_as_flagella(),no_part_speed:globals.no_part_speed()};
        let scalars=derive_cell_capability_scalars(&geometry.class_counts,g,INITIAL_ELECTRIC_RECHARGE_CURVE,
            CellCapabilityScalarContext{x87:context,sse:Policy::Mxcsr1f80},
            |id,x,c| sample_capability_curve(Some(curves.get(&id).expect("all source curve IDs checked").as_slice()),x,c))
            .map_err(|e|Error(format!("scalars: {e}")))?;
        Ok(Capability{geometry,scalars,row_count:count,first_word,rows})
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn admitted_feature_rows_preserve_indices_and_first_word_aliasing() {
        let row = |class| NativeCellPart { word_06_is_ffff:false, class, center:[0.;3], matrix_bits:[0;9], radius:1., tail_bits:[0;3] };
        let rows=[row(1),row(2),row(1),row(9)];
        assert!(FeatureSnapshot::new(7,&rows,[0,0,0]).is_err());
        assert!(FeatureSnapshot::new(7,&rows,[8,0,0]).is_err());
        let snapshot=FeatureSnapshot::new(7,&rows,[7,99,100]).unwrap();
        assert_eq!(snapshot.candidates().map(|(i,r)|(i,r.class)).collect::<Vec<_>>(),vec![(1,2),(3,9)]);
    }
    #[test]
    fn malformed_creation_is_rejected_before_curve_or_geometry() {
        let bytes=[0u8;spore_core::cell_globals::GLOBALS_SIZE];
        let globals=spore_core::cell_globals::parse(&bytes).unwrap();
        assert!(build(&[],1,&[],globals,&[]).unwrap_err().0.contains("creation"));
    }
    #[test]
    #[ignore = "requires exact installed packages and executable; controlled inspection only"]
    fn installed_ordered_rows_retain_complete_modeled_fields() {
        use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
        use std::path::Path;
        let install=std::env::var_os("RUSTSPORE_INSTALL").expect("set RUSTSPORE_INSTALL");
        let staged=crate::cell_source_player_v1::inspect(Path::new(&install),123,0).unwrap();
        let index=NativePackageIndex::open(&Path::new(&install).join("Data/Spore_Content.package")).unwrap();
        let key=NativeResourceKey{instance_id:staged.triple[0],type_id:0x0f43029a,group_id:staged.triple[2]};
        let entry=index.index().find_unique(key).unwrap().unwrap();
        let bytes=index.read(entry).unwrap();
        let creation=creature_data::parse(&bytes).unwrap();
        let rows=staged.capability.rows();
        let snapshot=staged.capability.feature_snapshot(staged.triple).unwrap();
        let expected:Vec<_>=rows.iter().enumerate().filter(|(_, row)| row.class != 1).map(|(i,_)|i).collect();
        assert_eq!(snapshot.candidates().map(|(i,_)|i).collect::<Vec<_>>(),expected);
        assert_eq!(rows.len(),staged.capability.row_count);
        assert_eq!(rows.len(),staged.parts.len());
        assert!(!rows.is_empty());
        for (i,actual) in rows.iter().enumerate() {
            let (source_index,class)=&staged.parts[i];
            assert_eq!(*source_index,i);
            let input:&[u8;140]=creation.part_view(i).unwrap().raw_row.try_into().unwrap();
            let expected=build_cell_part(input,class.class,staged.triple[0]).unwrap();
            let mut observed_bytes=[0xa5;72]; let mut expected_bytes=[0xa5;72];
            actual.write_native(&mut observed_bytes); expected.write_native(&mut expected_bytes);
            assert_eq!(observed_bytes,expected_bytes,"row {i}");
        }
    }
}
