use crate::{Error, Result, hls::SegmentFormat};
use bytes::Bytes;
use std::io::Cursor;

use super::DecryptionKey;

/// Supported decryption methods for /segment endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentDecryptMethod {
    /// SAMPLE-AES for MPEG-TS/AAC.
    SampleAes,
    /// SAMPLE-AES-CTR (typically fMP4).
    SampleAesCtr,
    /// Common Encryption (fMP4).
    Cenc,
}

impl SegmentDecryptMethod {
    /// Parse from query parameter string.
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "ssa" => Ok(Self::SampleAes),
            "ssa-ctr" => Ok(Self::SampleAesCtr),
            "cenc" => Ok(Self::Cenc),
            other => Err(Error::UnsupportedMethod(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SampleAes => "ssa",
            Self::SampleAesCtr => "ssa-ctr",
            Self::Cenc => "cenc",
        }
    }
}

/// Decryptor that wraps iori-ssa and mp4decrypt.
pub struct SegmentDecryptor {
    method: SegmentDecryptMethod,
    key: DecryptionKey,
    iv: [u8; 16],
}

impl SegmentDecryptor {
    pub fn new(method: SegmentDecryptMethod, key: DecryptionKey, iv: [u8; 16]) -> Self {
        Self { method, key, iv }
    }

    /// Decrypt segment data.
    ///
    /// For SAMPLE-AES (MPEG-TS/AAC): uses iori-ssa.
    /// For CENC (fMP4): uses mp4decrypt.
    pub async fn decrypt(
        &self,
        data: Bytes,
        init_segment: Option<Bytes>,
        init_decrypted: Option<Bytes>,
        format: SegmentFormat,
    ) -> Result<Bytes> {
        match (&self.method, format) {
            (SegmentDecryptMethod::SampleAes, SegmentFormat::MpegTS) => {
                self.decrypt_ssa_ts(data).await
            }
            (SegmentDecryptMethod::SampleAes, SegmentFormat::Aac) => {
                self.decrypt_ssa_aac(data).await
            }
            (
                SegmentDecryptMethod::SampleAesCtr
                | SegmentDecryptMethod::Cenc
                | SegmentDecryptMethod::SampleAes,
                SegmentFormat::Mp4,
            ) => self.decrypt_cenc(data, init_segment, init_decrypted).await,
            _ => Err(Error::UnsupportedCombination {
                method: self.method.as_str().to_string(),
                format: format.as_str().to_string(),
            }),
        }
    }

    /// Decrypt just an init segment (used to pre-compute the decrypted header for caching).
    pub fn decrypt_init(&self, init: &Bytes) -> Result<Bytes> {
        let keys = self.key.to_mp4decrypt_keys()?;
        let decryptor = mp4decrypt::Ap4CencDecryptingProcessor::new()
            .keys(&keys)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?
            .build()
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;
        decryptor
            .decrypt(init.as_ref(), None)
            .map(Bytes::from)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))
    }

    async fn decrypt_ssa_ts(&self, data: Bytes) -> Result<Bytes> {
        let key = *self.key.require_single()?;
        let iv = self.iv;

        // Use iori-ssa for MPEG-TS SAMPLE-AES decryption
        // iori-ssa uses Read/Write interface
        let input = Cursor::new(data.as_ref());
        let mut output = Vec::new();

        iori_ssa::decrypt_mpegts(input, &mut output, key, iv)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        Ok(Bytes::from(output))
    }

    async fn decrypt_ssa_aac(&self, data: Bytes) -> Result<Bytes> {
        let key = *self.key.require_single()?;
        let iv = self.iv;

        // Use iori-ssa for AAC SAMPLE-AES decryption
        // Note: iori-ssa uses the general `decrypt` function for AAC
        let input = Cursor::new(data.as_ref());
        let mut output = Vec::new();

        iori_ssa::decrypt(input, &mut output, key, iv)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        Ok(Bytes::from(output))
    }

    async fn decrypt_cenc(
        &self,
        data: Bytes,
        init_segment: Option<Bytes>,
        init_decrypted: Option<Bytes>,
    ) -> Result<Bytes> {
        let keys = self.key.to_mp4decrypt_keys()?;

        // Concatenate init + data if init provided
        let full_data = match &init_segment {
            Some(init) => [init.as_ref(), data.as_ref()].concat(),
            None => data.to_vec(),
        };

        // Build decryptor with keys using new builder API
        let decryptor = mp4decrypt::Ap4CencDecryptingProcessor::new()
            .keys(&keys)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?
            .build()
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        let decrypted = decryptor
            .decrypt(&full_data, None)
            .map_err(|e| Error::DecryptionFailed(e.to_string()))?;

        // Strip the prepended init segment from the output if it is unchanged
        let result = match &init_decrypted {
            Some(init) if decrypted.starts_with(init.as_ref()) => decrypted[init.len()..].to_vec(),
            _ => decrypted,
        };

        Ok(Bytes::from(result))
    }
}
