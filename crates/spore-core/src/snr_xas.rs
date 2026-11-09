//! Bounded EA AudioCore XAS1 RAM decoding.
//!
//! Derived from vgmstream r2168 `src/coding/ea_xas_decoder.c` and
//! `src/layout/blocked_ea_sns.c`, and `src/meta/ea_eaac.c`, copyright Adam Gashlin, Fastelbja, Ronny
//! Elfert, bnnm, Christopher Snowhill, NicknineTheEagle, bxaimc, Thealexbarney,
//! CyberBotX, EdnessP, et al. Used under the ISC-style terms in
//! `re/vgmstream-license.txt`. This is not native-game playback validation.

#[derive(Debug, PartialEq)]
pub struct Decoded {
    pub sample_rate: u32,
    pub channels: usize,
    /// Loop start in samples per channel; loop end is the full sample count.
    pub loop_start: Option<usize>,
    /// Interleaved signed 16-bit PCM, decoded once without repetition.
    pub pcm: Vec<i16>,
}

/// Decode bounded non-looping and looped RAM EA XAS1.
/// Unsupported or malformed payloads are rejected rather than guessed.
pub fn decode(data: &[u8]) -> Result<Decoded, &'static str> {
    let u32be = |p: usize| -> Option<u32> {
        Some(u32::from_be_bytes(data.get(p..p + 4)?.try_into().ok()?))
    };
    let h = u32be(0).ok_or("short header")?;
    let n = u32be(4).ok_or("short header")?;
    let channels = ((h >> 18) & 63) as usize + 1;
    let rate = h & 0x3ffff;
    if h >> 28 != 0 || (h >> 24) & 15 != 4 || channels > 2 || rate == 0 || rate > 200_000 {
        return Err("unsupported version, codec, channels or rate");
    }
    if n >> 30 != 0 {
        return Err("streamed resource unsupported");
    }
    let samples = (n & 0x1fff_ffff) as usize;
    if samples == 0 {
        return Err("empty resource unsupported");
    }
    let looping = (n >> 29) & 1 != 0;
    let loop_start = if looping {
        let start = u32be(8).ok_or("missing loop start")? as usize;
        if start >= samples {
            return Err("unsupported loop start");
        }
        Some(start)
    } else {
        None
    };
    let expected = match loop_start {
        Some(start) if start > 0 => vec![start, samples - start],
        _ => vec![samples],
    };
    let mut pos = if looping { 12usize } else { 8usize };
    let mut blocks = Vec::with_capacity(expected.len());
    for count in expected {
        let block = u32be(pos).ok_or("missing block")?;
        if block >> 24 != 0 && block >> 24 != 0x80 {
            return Err("unsupported block id");
        }
        let block_len = (block & 0xffffff) as usize;
        let block_samples = u32be(pos + 4).ok_or("short block")? as usize;
        let end = pos.checked_add(block_len).ok_or("block extent overflow")?;
        if block_len < 8 || end > data.len() || block_samples != count {
            return Err("unsupported block framing or sample count");
        }
        let frames = count.checked_add(127).ok_or("sample count overflow")? / 128;
        let needed = frames
            .checked_mul(channels)
            .and_then(|x| x.checked_mul(76))
            .ok_or("frame count overflow")?;
        if needed != block_len - 8 {
            return Err("frame extent mismatch");
        }
        blocks.push((pos + 8, count, frames));
        pos = end;
    }
    if pos != data.len() {
        return Err("trailing block data");
    }
    let mut pcm = Vec::with_capacity(samples.checked_mul(channels).ok_or("PCM size overflow")?);
    for (block_start, block_samples, frames) in blocks {
        for frame in 0..frames {
            let mut decoded = [[0i16; 128]; 2];
            for ch in 0..channels {
                let start = block_start + (frame * channels + ch) * 76;
                let bytes = &data[start..start + 76];
                for group in 0..4 {
                    let hdr =
                        u32::from_le_bytes(bytes[group * 4..group * 4 + 4].try_into().unwrap());
                    let coefs = [
                        (0.0f32, 0.0f32),
                        (0.9375, 0.0),
                        (1.796875, -0.8125),
                        (1.53125, -0.859375),
                    ];
                    let (c1, c2) = coefs
                        .get((hdr & 15) as usize)
                        .copied()
                        .ok_or("unsupported predictor")?;
                    let mut h2 = (hdr as u16 & 0xfff0) as i16;
                    let mut h1 = ((hdr >> 16) as u16 & 0xfff0) as i16;
                    let shift = (hdr >> 16) & 15;
                    let out = &mut decoded[ch][group * 32..group * 32 + 32];
                    out[0] = h2;
                    out[1] = h1;
                    for row in 0..15 {
                        let byte = bytes[16 + row * 4 + group];
                        for nibble in 0..2 {
                            let v = if nibble == 0 { byte >> 4 } else { byte & 15 };
                            let delta = ((v as i16) << 12) >> shift;
                            let s = (delta as f32 + h1 as f32 * c1 + h2 as f32 * c2) as i32;
                            let s = s.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
                            out[2 + row * 2 + nibble] = s;
                            h2 = h1;
                            h1 = s;
                        }
                    }
                }
            }
            for i in 0..(block_samples - frame * 128).min(128) {
                for channel in decoded.iter().take(channels) {
                    pcm.push(channel[i]);
                }
            }
        }
    }
    Ok(Decoded {
        sample_rate: rate,
        channels,
        loop_start,
        pcm,
    })
}

/// Decode the bounded XAS1 gigasample form using explicitly supplied SNR and SNS.
/// This does not resolve package overrides or establish that the pair belongs together.
pub fn decode_gigasample(snr: &[u8], sns: &[u8]) -> Result<Decoded, &'static str> {
    const MAX_INPUT: usize = 32 * 1024 * 1024;
    const MAX_PCM_BYTES: usize = 128 * 1024 * 1024;
    const MAX_BLOCKS: usize = 100_000;
    if snr.len() > MAX_INPUT || sns.len() > MAX_INPUT {
        return Err("resource exceeds input bound");
    }
    let read = |data: &[u8], pos: usize| -> Option<u32> {
        Some(u32::from_be_bytes(
            data.get(pos..pos.checked_add(4)?)?.try_into().ok()?,
        ))
    };
    let h = read(snr, 0).ok_or("short header")?;
    let n = read(snr, 4).ok_or("short header")?;
    let channels = ((h >> 18) & 63) as usize + 1;
    let rate = h & 0x3ffff;
    let samples = (n & 0x1fff_ffff) as usize;
    if h >> 28 != 0
        || (h >> 24) & 15 != 4
        || channels > 2
        || rate == 0
        || rate > 200_000
        || n >> 30 != 2
        || samples == 0
    {
        return Err("unsupported gigasample header");
    }
    let looped = (n >> 29) & 1 != 0;
    let loop_start = if looped {
        if read(snr, 8) != Some(0) {
            return Err("unsupported gigasample loop start");
        }
        Some(0)
    } else {
        None
    };
    let prefetch_at = if looped { 12 } else { 8 };
    let prefetch = read(snr, prefetch_at).ok_or("missing prefetch count")? as usize;
    if prefetch == 0 || prefetch >= samples {
        return Err("unsupported prefetch count");
    }
    let pcm_size = samples
        .checked_mul(channels)
        .and_then(|v| v.checked_mul(2))
        .ok_or("PCM size overflow")?;
    if pcm_size > MAX_PCM_BYTES {
        return Err("PCM exceeds output bound");
    }
    let mut blocks: Vec<(&[u8], usize)> = Vec::new();
    let check = |data: &[u8], pos: usize| -> Result<(usize, usize, u32, usize), &'static str> {
        let word = read(data, pos).ok_or("missing block")?;
        let len = (word & 0xffffff) as usize;
        let count = read(data, pos + 4).ok_or("short block")? as usize;
        let end = pos.checked_add(len).ok_or("block extent overflow")?;
        if len < 8 || end > data.len() || count == 0 {
            return Err("invalid block extent");
        }
        let needed = count
            .checked_add(127)
            .and_then(|v| v.checked_div(128))
            .and_then(|v| v.checked_mul(channels))
            .and_then(|v| v.checked_mul(76))
            .ok_or("frame size overflow")?;
        let audio_end = pos
            .checked_add(8)
            .and_then(|v| v.checked_add(needed))
            .ok_or("frame extent overflow")?;
        if audio_end > end {
            return Err("frame extent mismatch");
        }
        let tail = &data[audio_end..end];
        if !tail.is_empty()
            && !(word >> 24 == 0x80
                && end == data.len()
                && tail.len() == 40
                && tail.iter().all(|x| *x == 0))
        {
            return Err("unsupported terminal padding");
        }
        Ok((end, count, word >> 24, audio_end))
    };
    let pre_pos = prefetch_at + 4;
    let (end, count, flag, audio_end) = check(snr, pre_pos)?;
    if end != snr.len() || count != prefetch || flag != 0 {
        return Err("unsupported prefetch block");
    }
    blocks.push((&snr[pre_pos..audio_end], count));
    let mut pos = 0;
    let mut total = prefetch;
    while pos < sns.len() {
        if blocks.len() >= MAX_BLOCKS {
            return Err("block count exceeds bound");
        }
        let (end, count, flag, audio_end) = check(sns, pos)?;
        if flag != if end == sns.len() { 0x80 } else { 0 } {
            return Err("unsupported stream block flag");
        }
        total = total.checked_add(count).ok_or("sample count overflow")?;
        if total > samples {
            return Err("stream sample count exceeds header");
        }
        blocks.push((&sns[pos..audio_end], count));
        pos = end;
    }
    if total != samples || blocks.len() < 2 {
        return Err("stream sample count mismatch");
    }
    let mut pcm = Vec::with_capacity(samples * channels);
    for (block, count) in blocks {
        let mut one = Vec::with_capacity(8 + block.len());
        one.extend_from_slice(&h.to_be_bytes());
        one.extend_from_slice(&(count as u32).to_be_bytes());
        let block_header = u32::from_be_bytes(block[..4].try_into().unwrap());
        one.extend_from_slice(&((block_header & 0xff00_0000) | block.len() as u32).to_be_bytes());
        one.extend_from_slice(&block[4..]);
        pcm.extend(decode(&one)?.pcm);
    }
    Ok(Decoded {
        sample_rate: rate,
        channels,
        loop_start,
        pcm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silent_frame_and_partial_frame() {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0400ac44u32.to_be_bytes());
        d.extend_from_slice(&17u32.to_be_bytes());
        d.extend_from_slice(&84u32.to_be_bytes());
        d.extend_from_slice(&17u32.to_be_bytes());
        d.extend_from_slice(&[0; 76]);
        let output = decode(&d).unwrap();
        assert_eq!(
            (output.sample_rate, output.channels, output.pcm.len()),
            (44100, 1, 17)
        );
        assert!(output.pcm.iter().all(|x| *x == 0));
        d.pop();
        assert!(decode(&d).is_err());
    }
    #[test]
    fn rejects_unsupported_stream_and_loop_start() {
        let mut d = [0u8; 16];
        d[..4].copy_from_slice(&0x0400ac44u32.to_be_bytes());
        d[4..8].copy_from_slice(&0x40000000u32.to_be_bytes());
        assert_eq!(decode(&d), Err("streamed resource unsupported"));
        d[4..8].copy_from_slice(&0x20000001u32.to_be_bytes());
        d[8..12].copy_from_slice(&1u32.to_be_bytes());
        assert_eq!(decode(&d), Err("unsupported loop start"));
    }
    #[test]
    fn stereo_predictor_and_clipping() {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0404ac44u32.to_be_bytes());
        d.extend_from_slice(&4u32.to_be_bytes());
        d.extend_from_slice(&160u32.to_be_bytes());
        d.extend_from_slice(&4u32.to_be_bytes());
        let mut left = [0u8; 76];
        // predictor 1, history samples 32752, shift 0, positive nibbles clip.
        left[..4].copy_from_slice(&0x7ff07ff1u32.to_le_bytes());
        left[16] = 0x77;
        let mut right = [0u8; 76];
        right[..4].copy_from_slice(&0x00100020u32.to_le_bytes());
        right[16] = 0xf0;
        d.extend_from_slice(&left);
        d.extend_from_slice(&right);
        let out = decode(&d).unwrap();
        assert_eq!(out.pcm, vec![32752, 32, 32752, 16, 32767, -4096, 32767, 0]);
        d[16] = 4;
        assert_eq!(decode(&d), Err("unsupported predictor"));
    }

    #[test]
    fn two_block_loop_is_decoded_once_and_validated() {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0400ac44u32.to_be_bytes());
        d.extend_from_slice(&0x20000005u32.to_be_bytes());
        d.extend_from_slice(&2u32.to_be_bytes());
        for (count, first) in [(2u32, 0x00100020u32), (3u32, 0x00300040u32)] {
            d.extend_from_slice(&84u32.to_be_bytes());
            d.extend_from_slice(&count.to_be_bytes());
            let mut frame = [0u8; 76];
            frame[..4].copy_from_slice(&first.to_le_bytes());
            d.extend_from_slice(&frame);
        }
        let out = decode(&d).unwrap();
        assert_eq!(out.loop_start, Some(2));
        assert_eq!(out.pcm, vec![32, 16, 64, 48, 0]);
        d[19] = 3;
        assert!(decode(&d).is_err());
    }
    #[test]
    fn zero_start_loop_is_one_block_and_preserves_metadata() {
        let mut d = Vec::new();
        d.extend_from_slice(&0x0400ac44u32.to_be_bytes());
        d.extend_from_slice(&0x20000002u32.to_be_bytes());
        d.extend_from_slice(&0u32.to_be_bytes());
        d.extend_from_slice(&84u32.to_be_bytes());
        d.extend_from_slice(&2u32.to_be_bytes());
        let mut frame = [0u8; 76];
        frame[..4].copy_from_slice(&0x00100020u32.to_le_bytes());
        d.extend_from_slice(&frame);
        let out = decode(&d).unwrap();
        assert_eq!(out.loop_start, Some(0));
        assert_eq!(out.pcm, vec![32, 16]);
        d.extend_from_slice(&[0; 8]);
        assert_eq!(decode(&d), Err("trailing block data"));
    }
    #[test]
    fn bounded_gigasample_joins_prefetch_and_stream() {
        let mut snr = Vec::new();
        snr.extend_from_slice(&0x0400ac44u32.to_be_bytes());
        snr.extend_from_slice(&0x80000002u32.to_be_bytes());
        snr.extend_from_slice(&1u32.to_be_bytes());
        snr.extend_from_slice(&84u32.to_be_bytes());
        snr.extend_from_slice(&1u32.to_be_bytes());
        snr.extend_from_slice(&[0; 76]);
        let mut sns = Vec::new();
        sns.extend_from_slice(&0x80000054u32.to_be_bytes());
        sns.extend_from_slice(&1u32.to_be_bytes());
        sns.extend_from_slice(&[0; 76]);
        let result = decode_gigasample(&snr, &sns).unwrap();
        assert_eq!(result.pcm, vec![0, 0]);
        assert_eq!(result.loop_start, None);
        let mut padded = sns.clone();
        padded[3] += 40;
        padded.extend_from_slice(&[0; 40]);
        assert_eq!(decode_gigasample(&snr, &padded).unwrap().pcm, vec![0, 0]);
        *padded.last_mut().unwrap() = 1;
        assert!(decode_gigasample(&snr, &padded).is_err());
        sns[0] = 0;
        assert!(decode_gigasample(&snr, &sns).is_err());
        sns[0] = 0x80;
        snr[7] = 3;
        assert!(decode_gigasample(&snr, &sns).is_err());
    }
}
