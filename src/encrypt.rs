use hmac::{ Hmac, Mac };
use sha2::{ Sha256, digest::CtOutput };

pub fn hmac_sha256(key: &[u8], message: &str) -> CtOutput<Hmac<Sha256>> {
  type HmacSha256 = Hmac<Sha256>;
  
  let mut mac = HmacSha256::new_from_slice(key).unwrap();

  mac.update(message.as_bytes());

  return mac.finalize();
}