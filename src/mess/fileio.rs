pub fn make_wav_header(num_channels: u16, sample_rate: u32, num_frames: u32) -> [u8; 44] {
    let bits_per_sample = 32u16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample / 8u16) as u32;
    let block_align = num_channels * (bits_per_sample / 8);
    let data_chunk_size = num_frames * block_align as u32;
    let file_size = 36 + data_chunk_size;

    let mut header = [0u8; 44];

    // RIFF chunk descriptor
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&(file_size).to_le_bytes());     // Chunk size
    header[8..12].copy_from_slice(b"WAVE");

    // fmt sub-chunk
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&(16u32).to_le_bytes());        // Subchunk1Size (16 for PCM)
    header[20..22].copy_from_slice(&(3u16).to_le_bytes());         // AudioFormat (3 = IEEE float)
    header[22..24].copy_from_slice(&(num_channels).to_le_bytes()); // NumChannels
    header[24..28].copy_from_slice(&(sample_rate).to_le_bytes());  // SampleRate
    header[28..32].copy_from_slice(&(byte_rate).to_le_bytes());    // ByteRate
    header[32..34].copy_from_slice(&(block_align).to_le_bytes());  // BlockAlign
    header[34..36].copy_from_slice(&(bits_per_sample).to_le_bytes()); // BitsPerSample

    // data sub-chunk
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&(data_chunk_size).to_le_bytes());

    header
}