//! Explicit source SNR inspection playback, not a retail event or selection.
use bevy::{audio::Source,prelude::*,reflect::TypePath};
use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
use sha2::{Digest,Sha256};
use std::{io::Read,path::Path,sync::Arc,time::Duration};
#[path = "cell_event_sample_v1.rs"]
mod cell_event_sample_v1;

#[derive(Asset,TypePath,Clone)]
pub struct Preview { pcm:Arc<[i16]>, channels:u16, rate:u32 }
pub struct PreviewDecoder { pcm:Arc<[i16]>, channels:u16, rate:u32, position:usize }
impl Iterator for PreviewDecoder {
    type Item=f32;
    fn next(&mut self)->Option<f32> {let v=*self.pcm.get(self.position)?;self.position+=1;Some(f32::from(v)/32768.0)}
}
impl Source for PreviewDecoder {
    fn current_frame_len(&self)->Option<usize>{Some(self.pcm.len().saturating_sub(self.position))}
    fn channels(&self)->u16{self.channels}
    fn sample_rate(&self)->u32{self.rate}
    fn total_duration(&self)->Option<Duration>{Some(Duration::from_secs_f64(self.pcm.len() as f64 / f64::from(self.channels) / f64::from(self.rate)))}
}
impl Decodable for Preview {
    type DecoderItem=f32;type Decoder=PreviewDecoder;
    fn decoder(&self)->Self::Decoder{PreviewDecoder{pcm:self.pcm.clone(),channels:self.channels,rate:self.rate,position:0}}
}

/// Explicit package/key/hash selection; package stability while the native
/// index reopens its path is a caller admission. No overlay resolution implied.
pub fn load(package:&Path,scoped:u64,expected:&str)->Result<Preview,String> {
    if expected.len()!=64 || !expected.bytes().all(|b|b.is_ascii_hexdigit()){return Err("source audio SHA-256 must be 64 hex digits".into())}
    let mut f=std::fs::File::open(package).map_err(|e|e.to_string())?;let mut h=[0;dbpf::HEADER_SIZE];f.read_exact(&mut h).map_err(|e|e.to_string())?;
    let h=dbpf::DbpfHeader::parse(&h).map_err(|e|e.to_string())?;
    if h.index_size>64*1024*1024||h.index_entry_count>2_000_000{return Err("source audio package index exceeds bounds".into())}
    let idx=NativePackageIndex::open(package).map_err(|e|e.to_string())?;
    let key=NativeResourceKey::from_scoped_instance(0x01a527db,scoped);
    let e=idx.index().find_unique(key).map_err(|e|e.to_string())?.ok_or("source audio key not found")?;
    if e.stored_size.max(e.decompressed_size)>32*1024*1024{return Err("source audio payload exceeds bounds".into())}
    let data=idx.read(e).map_err(|e|e.to_string())?;
    if format!("{:x}",Sha256::digest(&data))!=expected.to_ascii_lowercase(){return Err("source audio payload SHA-256 mismatch".into())}
    let audio=match data.first().map(|x|x&15){Some(2)=>spore_core::snr_pcm16::decode(&data),Some(4)=>spore_core::snr_xas::decode(&data),_=>return Err("unsupported source audio codec".into())}.map_err(str::to_string)?;
    if audio.loop_start.is_some(){return Err("looping source audio preview unsupported".into())}
    let channels=u16::try_from(audio.channels).map_err(|_|"invalid audio channels")?;
    if channels==0||audio.sample_rate==0||audio.pcm.is_empty()||audio.pcm.len()%usize::from(channels)!=0{return Err("invalid decoded audio".into())}
    println!("CELL_SOURCE_AUDIO_PREVIEW package={} key={key} sha256={} source_selection=explicit decoded=true playback=unverified gameplay_routing=unverified",package.display(),expected.to_ascii_lowercase());
    Ok(Preview{pcm:audio.pcm.into(),channels,rate:audio.sample_rate})
}

/// Explicit event row and SNR package, with no inferred event dispatch or RNG.
/// The caller admits stable paths, a complete unchanged loader read, and getter identity.
pub fn load_event(event_package:&Path,event_key:u64,event_sha:&str,row:usize,snr_package:&Path,snr_sha:&str)->Result<Preview,String> {
    if event_key >> 32 != 0x021407ee { return Err("unsupported source event scope".into()); }
    if event_sha.len()!=64 || !event_sha.bytes().all(|b|b.is_ascii_hexdigit()) {return Err("source event SHA-256 must be 64 hex digits".into())}
    let mut f=std::fs::File::open(event_package).map_err(|e|e.to_string())?;
    let mut header=[0;dbpf::HEADER_SIZE]; f.read_exact(&mut header).map_err(|e|e.to_string())?;
    let header=dbpf::DbpfHeader::parse(&header).map_err(|e|e.to_string())?;
    if header.index_size>64*1024*1024 || header.index_entry_count>2_000_000 {return Err("source event package index exceeds bounds".into())}
    let idx=NativePackageIndex::open(event_package).map_err(|e|e.to_string())?;
    let key=NativeResourceKey::from_scoped_instance(0x02b9f662,event_key);
    let entry=idx.index().find_unique(key).map_err(|e|e.to_string())?.ok_or("source event key not found")?;
    if entry.stored_size.max(entry.decompressed_size)>4*1024*1024 {return Err("source event payload exceeds bounds".into())}
    let event=idx.read(entry).map_err(|e|e.to_string())?;
    if format!("{:x}",Sha256::digest(&event))!=event_sha.to_ascii_lowercase() {return Err("source event payload SHA-256 mismatch".into())}
    let selected=cell_event_sample_v1::select(&event,row).map_err(|e|e.to_string())?;
    println!("CELL_SOURCE_EVENT_PREVIEW package={} key={key} sha256={} row={row} selected_snr={selected:08X} selection=explicit loader=admitted playback=unverified",event_package.display(),event_sha.to_ascii_lowercase());
    load(snr_package,0x021407ee00000000 | u64::from(selected),snr_sha)
}

pub fn start(mut commands:Commands,mut assets:ResMut<Assets<Preview>>,preview:Res<PendingPreview>){
    commands.spawn((AudioPlayer(assets.add(preview.0.clone())),PlaybackSettings::DESPAWN));
    commands.remove_resource::<PendingPreview>();
}
#[derive(Resource)] pub struct PendingPreview(pub Preview);

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn preview_decoder_preserves_interleaving_and_finite_duration() {
        let p=Preview{pcm:vec![i16::MIN,0,i16::MAX,16384].into(),channels:2,rate:2};let mut d=p.decoder();
        assert_eq!(d.channels(),2);assert_eq!(d.sample_rate(),2);assert_eq!(d.total_duration(),Some(Duration::from_secs(1)));
        assert_eq!(d.next(),Some(-1.0));assert_eq!(d.current_frame_len(),Some(3));assert_eq!(d.next(),Some(0.0));
        assert_eq!(d.next(),Some(32767.0/32768.0));assert_eq!(d.next(),Some(0.5));assert_eq!(d.next(),None);
    }
    #[test] fn startup_queues_one_player_and_releases_pending_resource() {
        let mut app=App::new();app.init_resource::<Assets<Preview>>();
        app.insert_resource(PendingPreview(Preview{pcm:vec![0,1].into(),channels:1,rate:2}));
        app.add_systems(Startup,start);app.update();
        assert!(!app.world().contains_resource::<PendingPreview>());
        let world=app.world_mut();let mut query=world.query::<(&AudioPlayer<Preview>,&PlaybackSettings)>();
        let mut players=query.iter(world);let (player,settings)=players.next().unwrap();
        assert!(matches!(settings.mode,bevy::audio::PlaybackMode::Despawn));
        let handle=player.0.clone();assert!(players.next().is_none());
        let asset=world.resource::<Assets<Preview>>().get(&handle).unwrap();
        assert_eq!((&*asset.pcm,asset.channels,asset.rate),(&[0,1][..],1,2));
    }
    #[test]
    #[ignore = "requires explicit installed event and SNR packages; no audio device"]
    fn explicit_event_row_loads_selected_source_audio() {
        let event=std::env::var_os("RUSTSPORE_EVENT_PACKAGE").expect("set RUSTSPORE_EVENT_PACKAGE");
        let snr=std::env::var_os("RUSTSPORE_AUDIO_PACKAGE").expect("set RUSTSPORE_AUDIO_PACKAGE");
        let event=Path::new(&event); let snr=Path::new(&snr);
        let esha="359d9b975aca87c70e0d4414199893c9819c8a4b4390dc43ff4666838f0a0c12";
        let ssha="f68c6170fcd374eabbc66d7c9d82e10b421cc052570647478ea4f3cc3670afd4";
        let p=load_event(event,0x021407ee34c2ff0b,esha,0,snr,ssha).unwrap();
        assert_eq!((p.rate,p.channels,p.pcm.len()),(44100,1,33165));
        let bytes=p.pcm.iter().flat_map(|x|x.to_le_bytes()).collect::<Vec<_>>();
        assert_eq!(format!("{:x}",Sha256::digest(bytes)),"386ad72e41360d41d7db698d79646015d63a734281820b747a64691dece50757");
        assert!(load_event(event,0x021407ee34c2ff0b,&"0".repeat(64),0,snr,ssha).is_err());
        assert!(load_event(event,0x021407ee34c2ff0b,esha,usize::MAX,snr,ssha).is_err());
        assert!(load_event(event,0x021407ee34c2ff0b,esha,0,snr,&"0".repeat(64)).is_err());
    }
    #[test]
    #[ignore = "requires an explicitly selected installed source package"]
    fn explicitly_selected_xas_preview_matches_retained_pcm_digest() {
        let path=std::env::var_os("RUSTSPORE_AUDIO_PACKAGE").expect("explicit RUSTSPORE_AUDIO_PACKAGE required");
        let p=load(Path::new(&path),0x021407eeb000b7de,"7c976ebafbaf87da51e058d31173cfbfd9052f0c008aeaf62726f23523130830").unwrap();
        assert_eq!((p.rate,p.channels,p.pcm.len()),(44100,1,14043));
        let bytes=p.pcm.iter().flat_map(|x|x.to_le_bytes()).collect::<Vec<_>>();
        assert_eq!(format!("{:x}",Sha256::digest(&bytes)),"650ee6371dc3630ac7fc81404ef481203ea70cdba6fa727df7045d212cd9a879");
        assert!(load(Path::new(&path),0x021407eeb000b7de,&"0".repeat(64)).is_err());
    }
}

