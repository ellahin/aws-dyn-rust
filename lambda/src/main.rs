use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_route53::types::Change;
use aws_sdk_route53::types::ChangeBatch;
use aws_sdk_route53::types::ResourceRecord;
use aws_sdk_route53::types::ResourceRecordSet;
use aws_sdk_route53::types::RrType;
use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};

use common_data::UpdateRequest;

use serde_json;

use regex::Regex;

use sha2::{Digest, Sha512};

use std::env;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let dynamno_table = match env::var("DATABASE") {
        Ok(e) => e,
        Err(_) => {
            println!("Error no DATABASE ENV");
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let body = event.body();
    let body = std::str::from_utf8(body).expect("invalid utf-8 sequence");

    let req: UpdateRequest = match serde_json::from_str(body) {
        Ok(e) => e,
        Err(e) => {
            println!("Error: {}", e);
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body("Bad Request".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let ip = match event.request_context() {
        lambda_http::request::RequestContext::ApiGatewayV1(c) => c.identity.source_ip,
        lambda_http::request::RequestContext::ApiGatewayV2(c) => c.http.source_ip,
        _ => None,
    };

    let ip = match ip {
        Some(i) => i,
        None => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body("Bad Request".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let config = aws_config::load_defaults(BehaviorVersion::v2024_03_28()).await;
    let client = aws_sdk_dynamodb::Client::new(&config);

    let request = match client
        .get_item()
        .table_name(&dynamno_table)
        .key("key", AttributeValue::S(req.key.clone()))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let item = match request.item() {
        Some(e) => e,
        None => {
            let resp = Response::builder()
                .status(401)
                .header("content-type", "text/html")
                .body("Not authorized".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let item_secret = match item.get("secret") {
        Some(AttributeValue::S(e)) => e,
        None => {
            println!("Error: item secret does not extist in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
        _ => {
            println!("Error: item secret is not a string in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let item_domain = match item.get("domain") {
        Some(AttributeValue::S(e)) => e,
        None => {
            println!("Error: item domain does not extist in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
        _ => {
            println!("Error: item domain is not a string in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let item_zoneid = match item.get("zoneid") {
        Some(AttributeValue::S(e)) => e,
        None => {
            println!("Error: item zoneid does not extist in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
        _ => {
            println!("Error: item zoneid is not a string in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let item_last_set = match item.get("last_set") {
        Some(AttributeValue::S(e)) => e,
        None => {
            println!("Error: item last_set does not extist in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
        _ => {
            println!("Error: item last_set is not a string in item {}", &req.key);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let hash = format!("{:x}", Sha512::digest(&req.secret));

    if &hash != item_secret {
        println!("Hash Doesn't match: {}", hash);
        let resp = Response::builder()
            .status(401)
            .header("content-type", "text/html")
            .body("Not authorized".into())
            .map_err(Box::new)
            .unwrap();
        return Ok(resp);
    }

    if item_last_set == &ip {
        let resp = Response::builder()
            .status(200)
            .header("content-type", "text/html")
            .body("No ip change".into())
            .map_err(Box::new)
            .unwrap();
        return Ok(resp);
    }

    let route_client = aws_sdk_route53::Client::new(&config);

    let record = match ResourceRecord::builder().value(ip.clone()).build() {
        Ok(e) => e,
        Err(e) => {
            println!("Error: build record");
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let ipv4_regx = match Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$") {
        Ok(e) => e,
        Err(e) => {
            println!("Error: building regex");
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let record_type: RrType = match ipv4_regx.is_match(&ip) {
        true => RrType::A,
        false => RrType::Aaaa,
    };

    let record_set = match ResourceRecordSet::builder()
        .name(item_domain)
        .set_type(Some(record_type))
        .resource_records(record)
        .ttl(60)
        .build()
    {
        Ok(e) => e,
        Err(e) => {
            println!("Error: build record set");
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let change = match Change::builder()
        .resource_record_set(record_set)
        .action(aws_sdk_route53::types::ChangeAction::Upsert)
        .build()
    {
        Ok(e) => e,
        Err(e) => {
            println!("Error: build change");
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };
    let change_batch = match ChangeBatch::builder().changes(change).build() {
        Ok(e) => e,
        Err(e) => {
            println!("Error: build change batch");
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    match route_client
        .change_resource_record_sets()
        .set_hosted_zone_id(Some(item_zoneid.clone()))
        .set_change_batch(Some(change_batch))
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => {
            println!("Error: could not change record");
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    match client
        .put_item()
        .table_name(&dynamno_table)
        .item("key", AttributeValue::S(req.key))
        .item("last_set", AttributeValue::S(ip))
        .item("secret", AttributeValue::S(item_secret.to_string()))
        .item("domain", AttributeValue::S(item_domain.to_string()))
        .item("zoneid", AttributeValue::S(item_zoneid.to_string()))
        .send()
        .await
    {
        Ok(_) => (),
        Err(e) => {
            println!("Error: could not change record");
            println!("{:?}", e);
            let resp = Response::builder()
                .status(500)
                .header("content-type", "text/html")
                .body("Internal Server Error".into())
                .map_err(Box::new)
                .unwrap();
            return Ok(resp);
        }
    };

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("Record updated".into())
        .map_err(Box::new)
        .unwrap();
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
