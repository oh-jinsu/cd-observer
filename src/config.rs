use std::error::Error;
use std::env::var;

pub trait Config {
  fn get_port(&self) -> &str;
  fn get_aws_s3_bucket_name(&self) -> &str;
  fn get_aws_access_key_id(&self) -> &str;
  fn get_secret_access_key(&self) -> &str;
  fn get_aws_region(&self) -> &str;
}

struct ConfigImpl {
  port: String,
  aws_s3_bucket_name: String,
  aws_access_key_id: String,
  aws_secret_access_key: String,
  aws_region: String, 
}

impl Config for ConfigImpl {
  fn get_port(&self) ->  &str { &self.port }
  fn get_aws_s3_bucket_name(&self) -> &str { &self.aws_s3_bucket_name }
  fn get_aws_access_key_id(&self) -> &str { &self.aws_access_key_id }
  fn get_secret_access_key(&self) -> &str { &self.aws_secret_access_key }
  fn get_aws_region(&self) -> &str { &self.aws_region }
}

pub fn load() -> Result<Box<dyn Config>, Box<dyn Error>> {
  let port = var("PORT")?;

  let aws_s3_bucket_name = var("AWS_S3_BUCKET_NAME")?;

  let aws_access_key_id = var("AWS_ACCESS_KEY_ID")?;

  let aws_secret_access_key = var("AWS_SECRET_ACCESS_KEY")?;

  let aws_region = var("AWS_REGION")?;

  return Ok(Box::new(ConfigImpl {
    port,
    aws_s3_bucket_name,
    aws_access_key_id,
    aws_secret_access_key,
    aws_region,
  }));
}