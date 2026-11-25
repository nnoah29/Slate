/*
**  _                                              _      ___    ___
** | |                                            | |    |__ \  / _ \
** | |_Created _       _ __   _ __    ___    __ _ | |__     ) || (_) |
** | '_ \ | | | |     | '_ \ | '_ \  / _ \  / _` || '_ \   / /  \__, |
** | |_) || |_| |     | | | || | | || (_) || (_| || | | | / /_    / /
** |_.__/  \__, |     |_| |_||_| |_| \___/  \__,_||_| |_||____|  /_/
**          __/ |     on 2026-11-22.
**         |___/
**
**
** Conversion du PCM 16 bits little-endian du micro en échantillons flottants dans [-1, 1].
*/

/// Décodeur de flux PCM 16 bits qui garde l'octet orphelin d'un paquet pour le suivant.
#[derive(Debug, Default)]
pub struct Pcm16Decoder {
    carry: Option<u8>,
}

impl Pcm16Decoder {
    /// Convertit un paquet d'octets en échantillons ; un octet impair final attend le paquet suivant.
    pub fn add(&mut self, chunk: &[u8]) -> Vec<f32> {
        let mut data = Vec::with_capacity(chunk.len() + 1);
        data.extend(self.carry.take());
        data.extend_from_slice(chunk);
        let pairs = data.chunks_exact(2);
        self.carry = pairs.remainder().first().copied();
        pairs
            .map(|p| f32::from(i16::from_le_bytes([p[0], p[1]])) / 32768.0)
            .collect()
    }
}
