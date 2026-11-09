//! Bounded EA AudioCore PCM16BE non-looping RAM decoder.
//!
//! Header and SNS block framing follow vgmstream r2168 `src/meta/ea_eaac.c`
//! and `src/layout/blocked_ea_sns.c`; its ISC-style notice is in
//! `re/vgmstream-license.txt`. No runtime playback behavior is implied.
use crate::snr_xas::Decoded;

pub fn decode(data: &[u8]) -> Result<Decoded, &'static str> {
    let word = |p: usize| -> Option<u32> {
        Some(u32::from_be_bytes(data.get(p..p + 4)?.try_into().ok()?))
    };
    let h = word(0).ok_or("short header")?;
    let n = word(4).ok_or("short header")?;
    let channels = ((h >> 18) & 63) as usize + 1;
    let rate = h & 0x3ffff;
    if h >> 28 != 0 || (h >> 24) & 15 != 2 || channels > 2 || rate == 0 || rate > 200_000 {
        return Err("unsupported version, codec, channels or rate");
    }
    if n >> 29 != 0 {
        return Err("streamed or looped resource unsupported");
    }
    let samples = (n & 0x1fff_ffff) as usize;
    if samples == 0 {
        return Err("empty resource unsupported");
    }
    let block = word(8).ok_or("missing block")?;
    if block >> 24 != 0 && block >> 24 != 0x80 {
        return Err("unsupported block id");
    }
    let size = (block & 0xffffff) as usize;
    let block_samples = word(12).ok_or("short block")? as usize;
    let bytes = samples
        .checked_mul(channels)
        .and_then(|v| v.checked_mul(2))
        .ok_or("size overflow")?;
    if block_samples != samples
        || size.checked_sub(8) != Some(bytes)
        || 8usize.checked_add(size) != Some(data.len())
    {
        return Err("unsupported block framing or sample count");
    }
    let pcm = data[16..]
        .chunks_exact(2)
        .map(|x| i16::from_be_bytes([x[0], x[1]]))
        .collect();
    Ok(Decoded {
        sample_rate: rate,
        channels,
        loop_start: None,
        pcm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stereo_big_endian_and_framing() {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0204ac44u32.to_be_bytes());
        d.extend_from_slice(&2u32.to_be_bytes());
        d.extend_from_slice(&16u32.to_be_bytes());
        d.extend_from_slice(&2u32.to_be_bytes());
        d.extend_from_slice(&[0x7f, 0xff, 0x80, 0x00, 0xff, 0xff, 0x00, 0x01]);
        let result = decode(&d).unwrap();
        assert_eq!(result.pcm, vec![32767, -32768, -1, 1]);
        assert_eq!(
            (result.channels, result.sample_rate, result.loop_start),
            (2, 44100, None)
        );
        d.pop();
        assert!(decode(&d).is_err());
    }
}
