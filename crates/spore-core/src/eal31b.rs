//! Bounded EA EAL31b RAM framing and MPEG1 frame reconstruction.
//!
//! Derived from vgmstream r2168 `src/coding/mpeg_custom_utils_ealayer3.c`,
//! `src/meta/ea_eaac.c`, and `src/layout/blocked_ea_sns.c`; copyright Adam
//! Gashlin, Fastelbja, Ronny Elfert, bnnm, Christopher Snowhill, NicknineTheEagle,
//! bxaimc, Thealexbarney, CyberBotX, EdnessP, et al. ISC-style license notice:
//! `re/vgmstream-license.txt`. Literal PCM can be extracted and one narrow
//! layout assembled from caller-supplied decoded samples and alignment. The
//! module does not decode MPEG, derive decoder delay, or implement looping.

#[derive(Debug, PartialEq, Eq)]
pub struct PcmInsertion {
    pub granule: usize,
    pub offset_samples: u16,
    pub samples_per_channel: u16,
    pub unknown: u32,
    /// Byte range of planar big-endian PCM in the original SNR payload.
    pub source_start: usize,
    pub source_end: usize,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Reconstructed {
    pub sample_rate: u32,
    pub channels: usize,
    pub declared_samples: usize,
    /// Free-format MPEG1 Layer III frames; this is not assembled game audio.
    pub mpeg: Vec<u8>,
    pub insertions: Vec<PcmInsertion>,
    pub granule_count: usize,
}

/// PCM values carried by one insertion, in frame-interleaved channel order.
/// This is only the literal insertion data, not assembled MPEG audio.
#[derive(Debug, PartialEq, Eq)]
pub struct DecodedInsertion {
    pub granule: usize,
    pub offset_samples: u16,
    pub unknown: u32,
    pub pcm: Vec<i16>,
}

/// Controlled one-pass assembly from caller-supplied, frame-interleaved MPEG
/// decoder output. The decoder and raw alignment remain caller admissions.
#[derive(Debug, PartialEq, Eq)]
pub struct AssembledPcm { pub sample_rate:u32, pub channels:usize, pub pcm:Vec<i16> }

/// Assemble only a nonempty insertion at granule 1 with source offset 0. The
/// remaining samples are taken from `raw` at an explicitly supplied frame
/// offset; this does not derive decoder latency, load a decoder, or model later
/// insertion placement. This narrow boundary is based on observed fixtures,
/// not a general EAL31b playback implementation.
pub fn assemble_first_insertion(source:&[u8], raw:&[i16], raw_start_frame:usize)->Result<AssembledPcm,&'static str> {
    let (r,insertions)=reconstruct_with_insertions(source)?;
    if insertions.len()!=1 || insertions[0].granule!=1 || insertions[0].offset_samples!=0 || insertions[0].unknown!=0 || insertions[0].pcm.is_empty() {
        return Err("unsupported PCM insertion layout");
    }
    let channels=r.channels;
    if raw.len()%channels!=0 {return Err("raw PCM has incomplete frames")}
    let prefix=insertions[0].pcm.len()/channels;
    let remaining=r.declared_samples.checked_sub(prefix).ok_or("insertion exceeds declared samples")?;
    let total=r.declared_samples.checked_mul(channels).ok_or("PCM output overflow")?;
    if total>32*1024*1024 {return Err("PCM output exceeds bound")}
    let start=raw_start_frame.checked_mul(channels).ok_or("raw offset overflow")?;
    let end=start.checked_add(remaining.checked_mul(channels).ok_or("raw extent overflow")?).ok_or("raw extent overflow")?;
    let tail=raw.get(start..end).ok_or("raw PCM shorter than admitted alignment")?;
    let mut pcm=Vec::with_capacity(total);pcm.extend_from_slice(&insertions[0].pcm);pcm.extend_from_slice(tail);
    Ok(AssembledPcm{sample_rate:r.sample_rate,channels,pcm})
}

/// Reconstruct the supported source and decode its own planar big-endian PCM
/// insertion bytes. Parsing and extraction use the same immutable input; the
/// caller must still resolve decoder delay, insertion placement and sample
/// trimming before this can become complete audio. No MPEG decode is done.
pub fn reconstruct_with_insertions(data: &[u8]) -> Result<(Reconstructed, Vec<DecodedInsertion>), &'static str> {
    let reconstructed = reconstruct(data)?;
    let mut decoded = Vec::with_capacity(reconstructed.insertions.len());
    for insertion in &reconstructed.insertions {
        let count = usize::from(insertion.samples_per_channel);
        let samples = count.checked_mul(reconstructed.channels).ok_or("PCM sample count overflow")?;
        let bytes = data.get(insertion.source_start..insertion.source_end).ok_or("PCM insertion outside input")?;
        if bytes.len() != samples.checked_mul(2).ok_or("PCM byte count overflow")? {
            return Err("PCM insertion extent mismatch");
        }
        let mut pcm = Vec::with_capacity(samples);
        for frame in 0..count {
            for channel in 0..reconstructed.channels {
                let pos = (channel * count + frame) * 2;
                pcm.push(i16::from_be_bytes([bytes[pos],bytes[pos+1]]));
            }
        }
        decoded.push(DecodedInsertion {granule:insertion.granule,offset_samples:insertion.offset_samples,unknown:insertion.unknown,pcm});
    }
    Ok((reconstructed,decoded))
}
struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
}
impl Bits<'_> {
    fn read(&mut self, n: usize) -> Result<u32, &'static str> {
        if n > 32
            || self
                .pos
                .checked_add(n)
                .filter(|x| *x <= self.data.len() * 8)
                .is_none()
        {
            return Err("truncated bits");
        }
        let mut v = 0u32;
        for _ in 0..n {
            v = (v << 1) | u32::from((self.data[self.pos / 8] >> (7 - self.pos % 8)) & 1);
            self.pos += 1;
        }
        Ok(v)
    }
}
struct Writer {
    bytes: Vec<u8>,
    pos: usize,
}
impl Writer {
    fn put(&mut self, n: usize, v: u32) {
        for i in (0..n).rev() {
            if self.pos % 8 == 0 {
                self.bytes.push(0);
            }
            self.bytes[self.pos / 8] |= (((v >> i) & 1) as u8) << (7 - self.pos % 8);
            self.pos += 1;
        }
    }
}
struct Granule {
    ver: u32,
    rate: u32,
    mode: u32,
    ext: u32,
    index: u32,
    channels: usize,
    scfsi: [u32; 2],
    lens: [u32; 2],
    side1: [u32; 2],
    side2: [u32; 2],
    data_bit: usize,
}
fn be32(d: &[u8], p: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        d.get(p..p.checked_add(4)?)?.try_into().ok()?,
    ))
}
/// Parse and reconstruct only one-block non-looping EAL31b RAM MPEG1 streams.
pub fn reconstruct(data: &[u8]) -> Result<Reconstructed, &'static str> {
    let h = be32(data, 0).ok_or("short header")?;
    let n = be32(data, 4).ok_or("short header")?;
    let channels = ((h >> 18) & 63) as usize + 1;
    let sample_rate = h & 0x3ffff;
    let declared_samples = (n & 0x1fff_ffff) as usize;
    if h >> 28 != 0
        || (h >> 24) & 15 != 5
        || channels > 2
        || !matches!(sample_rate, 32000 | 44100 | 48000)
        || n >> 29 != 0
        || declared_samples == 0
    {
        return Err("unsupported EAAC header");
    }
    let block = be32(data, 8).ok_or("missing block")?;
    let block_len = (block & 0xffffff) as usize;
    if !matches!(block >> 24, 0 | 0x80)
        || block_len < 8
        || 8usize.checked_add(block_len) != Some(data.len())
        || be32(data, 12) != Some(declared_samples as u32)
    {
        return Err("unsupported SNS block");
    }
    const MAX_OUTPUT: usize = 32 * 1024 * 1024;
    let frame_size = (144usize * 640_000) / (sample_rate as usize);
    let max_granules = (MAX_OUTPUT / frame_size) * 2;
    let mut pos = 16usize;
    let mut granules = Vec::new();
    let mut insertions = Vec::new();
    while pos < data.len() {
        if granules.len() >= max_granules {
            return Err("reconstructed output exceeds bound");
        }
        let flag = *data.get(pos).ok_or("missing flag")?;
        if !matches!(flag, 0 | 0xee) {
            return Err("unsupported EA frame flag");
        }
        if flag == 0xee && granules.is_empty() {
            return Err("PCM insertion on first granule unsupported");
        }
        pos += 1;
        let mut b = Bits {
            data,
            pos: pos.checked_mul(8).ok_or("bit offset overflow")?,
        };
        let ver = b.read(2)?;
        let rate = b.read(2)?;
        let mode = b.read(2)?;
        let ext = b.read(2)?;
        let index = b.read(1)?;
        let frame_channels = if mode == 3 { 1 } else { 2 };
        let table = [44100, 48000, 32000];
        if ver != 3 || rate > 2 || table[rate as usize] != sample_rate || frame_channels != channels
        {
            return Err("unsupported granule header");
        }
        let mut g = Granule {
            ver,
            rate,
            mode,
            ext,
            index,
            channels,
            scfsi: [0; 2],
            lens: [0; 2],
            side1: [0; 2],
            side2: [0; 2],
            data_bit: 0,
        };
        if index == 1 {
            for i in 0..channels {
                g.scfsi[i] = b.read(4)?;
            }
        }
        for i in 0..channels {
            g.lens[i] = b.read(12)?;
            g.side1[i] = b.read(32)?;
            g.side2[i] = b.read(15)?;
        }
        g.data_bit = b.pos;
        let data_bits = g.lens.iter().map(|x| *x as usize).sum::<usize>();
        let end = b.pos.checked_add(data_bits).ok_or("data extent overflow")?;
        if end > data.len() * 8 {
            return Err("truncated granule data");
        }
        pos = end.checked_add(7).ok_or("alignment overflow")? / 8;
        if flag == 0xee {
            let raw = data
                .get(pos..pos.checked_add(8).ok_or("PCM header overflow")?)
                .ok_or("truncated PCM header")?;
            let offset_samples = u16::from_be_bytes([raw[0], raw[1]]);
            let count = u16::from_be_bytes([raw[2], raw[3]]);
            let unknown = u32::from_be_bytes(raw[4..8].try_into().unwrap());
            let source_start = pos + 8;
            let source_end = source_start
                .checked_add(usize::from(count) * channels * 2)
                .ok_or("PCM extent overflow")?;
            if source_end > data.len() {
                return Err("truncated PCM insertion");
            }
            insertions.push(PcmInsertion {
                granule: granules.len(),
                offset_samples,
                samples_per_channel: count,
                unknown,
                source_start,
                source_end,
            });
            pos = source_end;
        }
        granules.push(g);
    }
    if granules.is_empty() || granules.len() % 2 != 0 {
        return Err("unpaired granules");
    }
    let mut mpeg = Vec::new();
    for pair in granules.chunks_exact(2) {
        let a = &pair[0];
        let b = &pair[1];
        if a.index != 0
            || b.index != 1
            || (a.ver, a.rate, a.mode, a.ext, a.channels)
                != (b.ver, b.rate, b.mode, b.ext, b.channels)
        {
            return Err("incompatible granule pair");
        }
        let mut w = Writer {
            bytes: Vec::new(),
            pos: 0,
        };
        for (n, v) in [
            (11, 0x7ff),
            (2, a.ver),
            (2, 1),
            (1, 1),
            (4, 0),
            (2, a.rate),
            (1, 0),
            (1, 0),
            (2, a.mode),
            (2, a.ext),
            (1, 1),
            (1, 1),
            (2, 0),
        ] {
            w.put(n, v);
        }
        w.put(9, 0);
        w.put(if channels == 1 { 5 } else { 3 }, 0);
        for i in 0..channels {
            w.put(4, b.scfsi[i]);
        }
        for g in [a, b] {
            for i in 0..channels {
                w.put(12, g.lens[i]);
                w.put(32, g.side1[i]);
                w.put(15, g.side2[i]);
            }
        }
        for g in [a, b] {
            let mut bits = Bits {
                data,
                pos: g.data_bit,
            };
            for _ in 0..g.lens.iter().map(|x| *x as usize).sum::<usize>() {
                w.put(1, bits.read(1)?);
            }
        }
        if w.bytes.len() > frame_size {
            return Err("reconstructed MPEG frame overflow");
        }
        w.bytes.resize(frame_size, 0);
        mpeg.extend_from_slice(&w.bytes);
    }
    Ok(Reconstructed {
        sample_rate,
        channels,
        declared_samples,
        mpeg,
        insertions,
        granule_count: granules.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn granule(index: u32, flag: u8) -> Vec<u8> {
        let mut w = Writer {
            bytes: Vec::new(),
            pos: 0,
        };
        for (n, v) in [(2, 3), (2, 0), (2, 1), (2, 0), (1, index)] {
            w.put(n, v)
        }
        if index == 1 {
            w.put(4, 0);
            w.put(4, 0);
        }
        for _ in 0..2 {
            w.put(12, 0);
            w.put(32, 0);
            w.put(15, 0);
        }
        let mut out = vec![flag];
        out.extend(w.bytes);
        if flag == 0xee {
            out.extend_from_slice(&[0, 7, 0, 1, 0, 0, 0, 0, 0, 1, 0xff, 0xff]);
        }
        out
    }
    fn fixture() -> Vec<u8> {
        let mut body = granule(0, 0);
        body.extend(granule(1, 0xee));
        let mut d = Vec::new();
        d.extend_from_slice(&0x0504ac44u32.to_be_bytes());
        d.extend_from_slice(&1152u32.to_be_bytes());
        d.extend_from_slice(&((body.len() + 8) as u32).to_be_bytes());
        d.extend_from_slice(&1152u32.to_be_bytes());
        d.extend(body);
        d
    }
    #[test]
    fn reconstructs_pair_and_identifies_pcm_insertion() {
        let d = fixture();
        let r = reconstruct(&d).unwrap();
        assert_eq!((r.channels, r.sample_rate, r.granule_count), (2, 44100, 2));
        assert_eq!(r.mpeg.len(), 144 * 640000 / 44100);
        assert_eq!(&r.mpeg[..2], &[0xff, 0xfb]);
        assert_eq!(r.insertions.len(), 1);
        assert_eq!(
            (
                r.insertions[0].granule,
                r.insertions[0].offset_samples,
                r.insertions[0].samples_per_channel
            ),
            (1, 7, 1)
        );
        assert_eq!(
            &d[r.insertions[0].source_start..r.insertions[0].source_end],
            &[0, 1, 0xff, 0xff]
        );
    }
    #[test]
    fn extracts_planar_big_endian_samples_without_claiming_placement() {
        let d=fixture(); let (r,pcm)=reconstruct_with_insertions(&d).unwrap();
        assert_eq!(pcm,vec![DecodedInsertion{granule:1,offset_samples:7,unknown:0,pcm:vec![1,-1]}]);
        assert_eq!(r.insertions[0].samples_per_channel,1);
        let mut extended=d.clone();
        let start=r.insertions[0].source_start;
        extended[r.insertions[0].source_start-6..r.insertions[0].source_start-4].copy_from_slice(&2u16.to_be_bytes());
        extended.splice(start..r.insertions[0].source_end,[0,1,0,2,0xff,0xff,0xff,0xfe]);
        let len=(extended.len()-8) as u32;extended[8..12].copy_from_slice(&len.to_be_bytes());
        let (_,pcm)=reconstruct_with_insertions(&extended).unwrap();
        assert_eq!(pcm[0].pcm,vec![1,-1,2,-2]);
    }
    #[test]
    fn zero_length_insertion_is_empty_and_truncated_payload_is_rejected() {
        let mut d=fixture();let original=reconstruct(&d).unwrap();let i=&original.insertions[0];
        d[i.source_start-6..i.source_start-4].copy_from_slice(&0u16.to_be_bytes());
        d.drain(i.source_start..i.source_end);
        let len=(d.len()-8) as u32;d[8..12].copy_from_slice(&len.to_be_bytes());
        let (_,pcm)=reconstruct_with_insertions(&d).unwrap();assert!(pcm[0].pcm.is_empty());
        let mut truncated=fixture();truncated.pop();
        let len=(truncated.len()-8) as u32;truncated[8..12].copy_from_slice(&len.to_be_bytes());
        assert!(reconstruct_with_insertions(&truncated).is_err());
    }
    #[test]
    fn controlled_assembly_checks_alignment_and_preserves_prefix() {
        let mut d=fixture();let r=reconstruct(&d).unwrap();let i=&r.insertions[0];
        d[i.source_start-8..i.source_start-6].copy_from_slice(&0u16.to_be_bytes());
        let raw:Vec<i16>=(0..2308).map(|i|i as i16).collect();
        let a=assemble_first_insertion(&d,&raw,1).unwrap();
        assert_eq!(a.pcm.len(),2304);assert_eq!(&a.pcm[..6],&[1,-1,2,3,4,5]);
        assert_eq!(assemble_first_insertion(&d,&raw[..10],1),Err("raw PCM shorter than admitted alignment"));
        assert_eq!(assemble_first_insertion(&d,&raw[..11],0),Err("raw PCM has incomplete frames"));
        assert_eq!(assemble_first_insertion(&fixture(),&raw,0),Err("unsupported PCM insertion layout"));
        let mut unknown=d.clone();unknown[i.source_start-1]=1;
        assert_eq!(assemble_first_insertion(&unknown,&raw,0),Err("unsupported PCM insertion layout"));
    }
    #[test]
    fn rejects_invalid_framing_and_pair() {
        let mut d = fixture();
        d.pop();
        assert!(reconstruct(&d).is_err());
        let mut d = fixture();
        d[16] = 0x33;
        assert_eq!(reconstruct(&d), Err("unsupported EA frame flag"));
        d[16] = 0xee;
        assert_eq!(
            reconstruct(&d),
            Err("PCM insertion on first granule unsupported")
        );
        let mut body = granule(1, 0);
        body.extend(granule(1, 0));
        let mut d = fixture();
        d.truncate(16);
        d.extend(body);
        let size = (d.len() - 8) as u32;
        d[8..12].copy_from_slice(&size.to_be_bytes());
        assert_eq!(reconstruct(&d), Err("incompatible granule pair"));
    }
    #[test]
    fn rejects_reconstruction_amplification() {
        let mut pair = granule(0, 0);
        pair.extend(granule(1, 0));
        let mut d = fixture();
        d.truncate(16);
        for _ in 0..((32 * 1024 * 1024 / (144 * 640000 / 44100)) + 1) {
            d.extend_from_slice(&pair);
        }
        let size = (d.len() - 8) as u32;
        d[8..12].copy_from_slice(&size.to_be_bytes());
        assert_eq!(reconstruct(&d), Err("reconstructed output exceeds bound"));
    }
}
