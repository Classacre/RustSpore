//! Opt-in controlled EAL31b WAV preview via an explicitly trusted libmpg123.
//! LGPL-2.1 library is supplied locally; no decoder binary is distributed.
//! This does not establish source event selection, live audio or general EAL3.
#[cfg(not(windows))]
fn main() { eprintln!("this opt-in decoder currently supports Windows only"); std::process::exit(1); }

#[cfg(windows)]
mod win {
    use dbpf::native_index::{NativePackageIndex,NativeResourceKey};
    use libloading::os::windows::{Library,LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR,LOAD_LIBRARY_SEARCH_SYSTEM32};
    use sha2::{Digest,Sha256};
    use std::{env,ffi::{c_char,c_int,c_long,c_void},fs::{self,File,OpenOptions},io::{Read,Write},os::windows::{fs::OpenOptionsExt,io::AsRawHandle},path::Path};
    #[repr(C)] struct FileDispositionInfo { delete_file:u8 }
    #[link(name="kernel32")]
    extern "system" { fn SetFileInformationByHandle(file:*mut c_void,class:c_int,info:*const c_void,size:u32)->c_int; }
    fn delete_on_close(file:&File)->std::io::Result<()> {
        let disposition=FileDispositionInfo{delete_file:1};
        // SAFETY: live exclusive Windows file handle and correctly sized
        // FILE_DISPOSITION_INFO for FileDispositionInfo class (4).
        let ok=unsafe{SetFileInformationByHandle(file.as_raw_handle(),4,&disposition as *const _ as *const c_void,std::mem::size_of::<FileDispositionInfo>() as u32)};
        if ok==0 {Err(std::io::Error::last_os_error())} else {Ok(())}
    }

    /// On a write/sync error request deletion of this exact opened file via its
    /// handle while the exclusive handle is still live. Never delete by path.
    fn write_new<F>(dest:&Path,writer:F)->Result<(),Box<dyn std::error::Error>>
    where F:FnOnce(&mut File)->std::io::Result<()> {
        const GENERIC_WRITE:u32=0x4000_0000;const DELETE:u32=0x0001_0000;
        let mut file=OpenOptions::new().write(true).create_new(true).access_mode(GENERIC_WRITE|DELETE).share_mode(0).open(dest)?;
        if let Err(error)=writer(&mut file).and_then(|_|file.sync_all()) {
            if let Err(cleanup)=delete_on_close(&file) {return Err(format!("WAV write/sync failed: {error}; handle cleanup failed: {cleanup} (partial file may remain)").into())}
            return Err(format!("WAV write/sync failed: {error}; newly created file marked for deletion on close").into());
        }
        Ok(())
    }

    type Init=unsafe extern "C" fn()->c_int;
    type Exit=unsafe extern "C" fn();
    type New=unsafe extern "C" fn(*const c_char,*mut c_int)->*mut c_void;
    type Delete=unsafe extern "C" fn(*mut c_void);
    type HandleCall=unsafe extern "C" fn(*mut c_void)->c_int;
    type Format=unsafe extern "C" fn(*mut c_void,c_long,c_int,c_int)->c_int;
    type Feed=unsafe extern "C" fn(*mut c_void,*const u8,usize)->c_int;
    type ReadPcm=unsafe extern "C" fn(*mut c_void,*mut u8,usize,*mut usize)->c_int;
    type GetFormat=unsafe extern "C" fn(*mut c_void,*mut c_long,*mut c_int,*mut c_int)->c_int;
    struct Decoder { _lib:Library, handle:*mut c_void, delete:Delete, exit:Exit, feed:Feed, read:ReadPcm, get:GetFormat }
    impl Drop for Decoder { fn drop(&mut self) { unsafe { (self.delete)(self.handle); (self.exit)(); } } }
    impl Decoder {
        /// The user explicitly selects and trusts the executable library and
        /// its dependencies. This process is single-threaded at this boundary.
        unsafe fn open(path:&Path,rate:u32,channels:usize)->Result<Self,Box<dyn std::error::Error>> {
            let lib=Library::load_with_flags(path,LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR|LOAD_LIBRARY_SEARCH_SYSTEM32)?;
            let init:Init=*lib.get(b"mpg123_init\0")?;let exit:Exit=*lib.get(b"mpg123_exit\0")?;
            let new:New=*lib.get(b"mpg123_new\0")?;let delete:Delete=*lib.get(b"mpg123_delete\0")?;
            let none:HandleCall=*lib.get(b"mpg123_format_none\0")?;let format:Format=*lib.get(b"mpg123_format\0")?;
            let open:HandleCall=*lib.get(b"mpg123_open_feed\0")?;let feed:Feed=*lib.get(b"mpg123_feed\0")?;
            let read:ReadPcm=*lib.get(b"mpg123_read\0")?;let get:GetFormat=*lib.get(b"mpg123_getformat\0")?;
            if init()!=0{return Err("mpg123 initialization failed".into())}
            let mut err=0;let handle=new(std::ptr::null(),&mut err);
            if handle.is_null(){exit();return Err(format!("mpg123_new failed: {err}").into())}
            let decoder=Self{_lib:lib,handle,delete,exit,feed,read,get};
            // MPG123_MONO/STEREO are 1/2; MPG123_ENC_SIGNED_16 is 0xd0.
            if none(handle)!=0 || format(handle,rate as c_long,channels as c_int,0xd0)!=0 || open(handle)!=0 {
                return Err("mpg123 format/feed setup failed".into());
            }
            Ok(decoder)
        }
        fn decode(&self,mpeg:&[u8],rate:u32,channels:usize)->Result<Vec<i16>,Box<dyn std::error::Error>> {
            unsafe {if (self.feed)(self.handle,mpeg.as_ptr(),mpeg.len())!=0{return Err("mpg123 feed failed".into())}}
            let mut bytes=Vec::new();let mut buf=[0u8;32768];let mut configured=false;
            loop {
                let mut done=0usize;let code=unsafe{(self.read)(self.handle,buf.as_mut_ptr(),buf.len(),&mut done)};
                if done>buf.len() || bytes.len().checked_add(done).is_none_or(|n|n>128*1024*1024) {return Err("decoded PCM exceeds bounds".into())}
                if done!=0 && !configured{return Err("PCM before verified format".into())}bytes.extend_from_slice(&buf[..done]);
                match code {
                    -11=>{if configured{return Err("unexpected format change".into())}let(mut r,mut c,mut e)=(0,0,0);
                        if unsafe{(self.get)(self.handle,&mut r,&mut c,&mut e)}!=0 || r!=rate as c_long || c!=channels as c_int || e!=0xd0 {return Err("unexpected mpg123 output format".into())}
                        configured=true;},
                    -10|-12=>break,
                    0 if done>0=>{},
                    0=>return Err("mpg123 made no progress".into()),
                    x=>return Err(format!("mpg123 decode error: {x}").into()),
                }
            }
            if !configured || bytes.len()%2!=0{return Err("invalid mpg123 PCM output".into())}
            // Windows target is little-endian; mpg123 default is native-endian.
            Ok(bytes.chunks_exact(2).map(|x|i16::from_le_bytes([x[0],x[1]])).collect())
        }
    }
    fn digest_file(p:&Path)->Result<String,Box<dyn std::error::Error>> {let mut f=fs::File::open(p)?;let mut h=Sha256::new();let mut b=[0;65536];loop{let n=f.read(&mut b)?;if n==0{break}h.update(&b[..n]);}Ok(format!("{:x}",h.finalize()))}
    fn hex64(s:&str)->Result<String,Box<dyn std::error::Error>> {if s.len()!=64||!s.bytes().all(|b|b.is_ascii_hexdigit()){return Err("expected 64 hex digest".into())}Ok(s.to_ascii_lowercase())}
    pub fn run()->Result<(),Box<dyn std::error::Error>> {
        let a:Vec<_>=env::args_os().skip(1).collect();if a.len()!=7{return Err("usage: <package> <16-hex-scope-instance> <source-sha256> <absolute-trusted-mpg123.dll> <dll-sha256> <raw-start-frames> <new-target.wav>".into())}
        let key=a[1].to_str().ok_or("invalid key")?;if key.len()!=16||!key.bytes().all(|b|b.is_ascii_hexdigit()){return Err("expected 16 hex key".into())}
        let key=NativeResourceKey::from_scoped_instance(0x01a527db,u64::from_str_radix(key,16)?);
        let source_sha=hex64(a[2].to_str().ok_or("invalid digest")?)?;let dll_sha=hex64(a[4].to_str().ok_or("invalid digest")?)?;
        let requested=Path::new(&a[3]);if !requested.is_absolute(){return Err("decoder DLL path must be absolute".into())}let dll=requested.canonicalize()?;
        let start:usize=a[5].to_str().ok_or("invalid offset")?.parse()?;let out=Path::new(&a[6]);
        if !out.extension().is_some_and(|x|x.eq_ignore_ascii_case("wav")){return Err("output must be WAV".into())}
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target").canonicalize()?;
        let parent=out.parent().ok_or("missing output parent")?.canonicalize()?;if !parent.starts_with(root){return Err("output must be under target".into())}
        let dest=parent.join(out.file_name().ok_or("missing output filename")?);if dest.exists(){return Err("output already exists".into())}
        if digest_file(&dll)?!=dll_sha{return Err("decoder DLL hash mismatch".into())}
        let package=Path::new(&a[0]);let mut f=fs::File::open(package)?;let mut h=[0;dbpf::HEADER_SIZE];f.read_exact(&mut h)?;
        let h=dbpf::DbpfHeader::parse(&h)?;if h.index_size>64*1024*1024||h.index_entry_count>2_000_000{return Err("package index exceeds bounds".into())}
        let idx=NativePackageIndex::open(package)?;let e=idx.index().find_unique(key)?.ok_or("SNR not found")?;
        if e.stored_size.max(e.decompressed_size)>32*1024*1024{return Err("SNR exceeds bounds".into())}
        let source=idx.read(e)?;if format!("{:x}",Sha256::digest(&source))!=source_sha{return Err("source payload hash mismatch".into())}
        let (r,inserts)=spore_core::eal31b::reconstruct_with_insertions(&source)?;
        if inserts.len()!=1||inserts[0].granule!=1||inserts[0].offset_samples!=0||inserts[0].unknown!=0||inserts[0].pcm.is_empty(){return Err("unsupported insertion layout".into())}
        let decoder=unsafe{Decoder::open(&dll,r.sample_rate,r.channels)?};let raw=decoder.decode(&r.mpeg,r.sample_rate,r.channels)?;drop(decoder);
        if digest_file(&dll)?!=dll_sha{return Err("decoder DLL changed during run".into())}
        let audio=spore_core::eal31b::assemble_first_insertion(&source,&raw,start)?;
        let bytes:u32=audio.pcm.len().checked_mul(2).ok_or("WAV size overflow")?.try_into()?;
        let align:u16=(audio.channels*2).try_into()?;let byte_rate=audio.sample_rate.checked_mul(align as u32).ok_or("WAV rate overflow")?;
        let mut wav=Vec::with_capacity(bytes as usize+44);wav.extend_from_slice(b"RIFF");wav.extend_from_slice(&bytes.checked_add(36).ok_or("RIFF size overflow")?.to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");wav.extend_from_slice(&16u32.to_le_bytes());wav.extend_from_slice(&1u16.to_le_bytes());wav.extend_from_slice(&(audio.channels as u16).to_le_bytes());wav.extend_from_slice(&audio.sample_rate.to_le_bytes());wav.extend_from_slice(&byte_rate.to_le_bytes());wav.extend_from_slice(&align.to_le_bytes());wav.extend_from_slice(&16u16.to_le_bytes());wav.extend_from_slice(b"data");wav.extend_from_slice(&bytes.to_le_bytes());
        for x in &audio.pcm{wav.extend_from_slice(&x.to_le_bytes())}
        write_new(&dest,|file|file.write_all(&wav))?;
        let wav_sha=format!("{:x}",Sha256::digest(&wav));
        if digest_file(&dest)?!=wav_sha{return Err("output WAV changed after write; file retained".into())}
        println!("controlled preview {key}: {} Hz, {} channels, {} frames; raw alignment {} explicitly supplied; wav_sha256={wav_sha}; no source-event or playback claim",audio.sample_rate,audio.channels,audio.pcm.len()/audio.channels,start);
        Ok(())
    }
    #[cfg(test)] mod tests {
        use super::*;
        #[test] fn failed_write_deletes_new_file_and_does_not_touch_existing() {
            use std::io::{Seek,SeekFrom};
            let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target").canonicalize().unwrap();
            let p=root.join(format!("eal31b-write-failure-{}.tmp",std::process::id()));
            assert!(!p.exists());
            let result=write_new(&p,|f|{f.write_all(b"partial")?;Err(std::io::Error::other("injected"))});
            assert!(result.is_err());assert!(!p.exists());
            let mut existing=OpenOptions::new().read(true).write(true).create_new(true).access_mode(0xc001_0000).share_mode(0).open(&p).unwrap();existing.write_all(b"existing").unwrap();
            assert!(write_new(&p,|f|f.write_all(b"different")).is_err());
            existing.seek(SeekFrom::Start(0)).unwrap();let mut actual=Vec::new();existing.read_to_end(&mut actual).unwrap();
            assert_eq!(actual,b"existing");delete_on_close(&existing).unwrap();drop(existing);assert!(!p.exists());
        }
    }
}
#[cfg(windows)] fn main()->Result<(),Box<dyn std::error::Error>> {win::run()}
