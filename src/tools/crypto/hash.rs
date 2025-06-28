/// Based on https://it-tools.tech/hash-text
use std::io::Read;

use anyhow::Context;
use base64ct::Encoding;
use serde_json::json;

use crate::tool::{Output, Tool};

#[derive(Debug, clap::Parser)]
#[command(about = "Generate a hash from text or stdin")]
pub struct Hash {
    contents: String,

    /// The hash algorithm.
    #[arg(
        short = 'a',
        long = "algo",
        default_value = "sha256",
        ignore_case = true
    )]
    algo: Algorithm,

    /// The encoding to use to represent the hash value.
    #[arg(short = 'd', long = "digest", default_value = "hex")]
    digest: Digest,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Algorithm {
    MD5,
    SHA1,
    SHA224,
    SHA256,
    SHA384,
    SHA512,
    SHA3_224,
    SHA3_256,
    SHA3_384,
    SHA3_512,
    RIPEMD160,
}

#[derive(Debug, Copy, Clone, clap::ValueEnum)]
pub enum Digest {
    Hex,
    Base64,
    Base64URL,
}

impl Tool for Hash {
    fn execute(&self) -> anyhow::Result<Option<Output>> {
        let bytes = if self.contents.trim() == "-" {
            let stdin = std::io::stdin();
            let mut lock = stdin.lock();

            let mut buf: Vec<u8> = Vec::new();
            lock.read_to_end(&mut buf)
                .context("Could not read from stdin")?;

            buf
        } else {
            self.contents.clone().into_bytes()
        };

        Ok(Some(Output::JsonValue(json!(match self.algo {
            Algorithm::MD5 => hash::<md5::Md5>(bytes, self.digest),
            Algorithm::SHA1 => hash::<sha1::Sha1>(bytes, self.digest),
            Algorithm::SHA224 => hash::<sha2::Sha224>(bytes, self.digest),
            Algorithm::SHA256 => hash::<sha2::Sha256>(bytes, self.digest),
            Algorithm::SHA384 => hash::<sha2::Sha384>(bytes, self.digest),
            Algorithm::SHA512 => hash::<sha2::Sha512>(bytes, self.digest),
            Algorithm::SHA3_224 => hash::<sha3::Sha3_224>(bytes, self.digest),
            Algorithm::SHA3_256 => hash::<sha3::Sha3_256>(bytes, self.digest),
            Algorithm::SHA3_384 => hash::<sha3::Sha3_384>(bytes, self.digest),
            Algorithm::SHA3_512 => hash::<sha3::Sha3_512>(bytes, self.digest),
            Algorithm::RIPEMD160 => hash::<ripemd::Ripemd160>(bytes, self.digest),
        }))))
    }
}

fn hash<D: digest::Digest>(contents: Vec<u8>, digest: Digest) -> String {
    let mut hasher = D::new();
    hasher.update(contents);

    let hash = hasher.finalize();
    match digest {
        Digest::Hex => base16ct::lower::encode_string(&hash),
        Digest::Base64 => base64ct::Base64::encode_string(&hash),
        Digest::Base64URL => base64ct::Base64UrlUnpadded::encode_string(&hash),
    }
}
