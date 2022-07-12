use std::fs::File;
use regex::Regex;
use reqwest::{ header, blocking::Client, StatusCode };
use chrono::*;
use sha2::{ Sha256, Digest };

use delivery::config;
use delivery::listener;
use delivery::encrypt::hmac_sha256;

fn main() {
    let config = match config::load() {
        Ok(x) => x,
        Err(e) => {
            return println!("{}", e);
        }   
    };

    let address = format!("localhost:{}", config.get_port());

    let mut listener = listener::new();

    listener.add(handle_connection(config));

    if let Err(e) = listener.listen(&address) {
        println!("{}", e);
    }
}


fn handle_connection(config: Box<dyn config::Config>) -> listener::Handler {
    Box::new(move |request| {
        let preffix = r":?^GET\s/sync/";
  
        let suffix = r"/?\sHTTP/1.1";
    
        let validator = Regex::new(format!(r"{}{}{}", preffix, r"[a-zA-Z0-9]+", suffix).as_str())?;
    
        let response = match validator.find(request) {
            Some(x) => { 
                let replacer = Regex::new(format!(r"({}|{})", preffix, suffix).as_str())?;
    
                let id = replacer.replace_all(x.as_str(), "");
    
                let now = Utc::now();
    
                let iso_date_string = format!("{}{:0>2}{:0>2}T{:0>2}{:0>2}{:0>2}Z", now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
                
                let service = "s3";
                
                let datetime = format!("{}{:0>2}{:0>2}", now.year(), now.month(), now.day());
                
                let scope = format!("{}/{}/{}/aws4_request",  datetime, config.get_aws_region(), service);
    
                let method = "GET";
    
                let canonical_uri = format!("/{}/build.zip", id);
    
                let canonical_query_string = "";
    
                let sha256_for_empty = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    
                let canonical_headers = format!("host:{}.{}.{}.amazonaws.com\nx-amz-content-sha256:{}\nx-amz-date:{}\n", config.get_aws_s3_bucket_name(), service, config.get_aws_region(), sha256_for_empty, iso_date_string);
    
                let signed_headers = "host;x-amz-content-sha256;x-amz-date";
    
                let canonical_request = format!("{}\n{}\n{}\n{}\n{}\n{}", method, canonical_uri, canonical_query_string, canonical_headers, signed_headers, sha256_for_empty);
    
                let mut hasher = Sha256::new();
    
                hasher.update(canonical_request.as_bytes());
    
                let signing_method = "AWS4-HMAC-SHA256";
    
                let string_to_sign = format!("{}\n{}\n{}\n{:x}", signing_method, iso_date_string, scope, hasher.finalize());
    
                let date_key = hmac_sha256(&format!("AWS4{}", config.get_secret_access_key()).into_bytes(), &datetime);
    
                let date_region_key = hmac_sha256(&date_key.into_bytes()[..], config.get_aws_region());
    
                let date_region_service_key = hmac_sha256(&date_region_key.into_bytes()[..], service);
    
                let signing_key = hmac_sha256(&date_region_service_key.into_bytes()[..], "aws4_request");
    
                let signature = hex::encode(hmac_sha256(&signing_key.into_bytes()[..], &string_to_sign).into_bytes());
    
                let credential = format!("{}/{}", config.get_aws_access_key_id(), scope);
    
                let authorization = format!("{} Credential={},SignedHeaders={},Signature={}", signing_method, credential, signed_headers, signature);
    
                let client = Client::new();
    
                match client
                    .get(format!("https://{}.{}.{}.amazonaws.com/{}/build.zip", config.get_aws_s3_bucket_name(), service, config.get_aws_region(), id))
                    .header(header::AUTHORIZATION, &authorization)
                    .header("x-amz-content-sha256", sha256_for_empty)
                    .header("x-amz-date", iso_date_string)
                    .send() {
                        Ok(mut response) => match response.status() {
                            StatusCode::OK => match File::create("build.zip") {
                                Ok(mut file) => match response.copy_to(&mut file) {
                                    Ok(_) => "HTTP/1.1 200 OK\r\n\r\n",
                                    Err(e) => {
                                        println!("{}", e);
    
                                        "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n"
                                    }
                                }
                                Err(e) => {
                                    println!("{}", e);
    
                                    "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n"
                                }
                            }
                            s => {
                                println!("{}", s);
    
                                "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n"
                            }
                        }
                        Err(e) => {
                            println!("{}", e);
    
                            "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n"
                        }
                    }
            },
            None => "HTTP/1.1 404 NOT FOUND\r\n\r\n"
        };
    
        return Ok(String::from(response))
    })
  }