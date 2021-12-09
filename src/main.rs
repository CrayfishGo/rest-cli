#[macro_use]
extern crate clap;

#[macro_use]
extern crate anyhow;

use clap::{AppSettings, Parser};
use colored::*;
use mime::Mime;
use reqwest::{Client, Response, Url};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Crayfishgo")]
struct Opts {
    #[clap(subcommand)]
    sub_cmd: SubCommand,
}

#[derive(Debug, Parser)]
enum SubCommand {
    Get(Get),
    Post(Post),
    Put(Put),
    Delete(Delete),
}

#[derive(Debug, Parser)]
pub struct Get {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

#[derive(Debug, Parser)]
pub struct Post {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    body: Vec<KvPair>,
}

#[derive(Debug, Parser)]
struct Put {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    body: Vec<KvPair>,
}

#[derive(Debug, Parser)]
struct Delete {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

fn parse_url(url: &str) -> anyhow::Result<String> {
    let result: Url = url.parse()?;
    Ok(result.into())
}

#[derive(Debug)]
pub struct KvPair {
    key: String,
    value: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_strs = s.split("=");
        let err = || {
            anyhow!(format!(
                "Failed to parse {}, the request body shoud be K/V pair",
                s
            ))
        };
        Ok(Self {
            key: (split_strs.next().ok_or_else(err)?).to_string(),
            value: (split_strs.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_request_body_kv_parir(s: &str) -> anyhow::Result<KvPair> {
    Ok(s.parse()?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);

    let client = Client::new();

    let result = match opts.sub_cmd {
        SubCommand::Get(ref args) => do_get(client, args).await?,
        SubCommand::Post(ref args) => do_post(client, args).await?,
        SubCommand::Put(_) => {}
        SubCommand::Delete(_) => {}
    };
    Ok(result)
}

fn print_status(resp: &Response) {
    let status = format!("{:?},{}", resp.version(), resp.status()).blue();
    println!("{}\n", status)
}

fn print_resp_header(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
    println!("\n")
}

fn print_resp_body(m: Option<Mime>, resp_body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(resp_body).unwrap().cyan())
        }
        _ => {
            println!("{}", resp_body)
        }
    }
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

async fn print_response(resp: Response) -> anyhow::Result<()> {
    print_status(&resp);
    print_resp_header(&resp);
    let mime = get_content_type(&resp);
    let resp_body = resp.text().await?;
    print_resp_body(mime, &resp_body);
    Ok(())
}

async fn do_get(client: Client, args: &Get) -> anyhow::Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok((print_response(resp).await?))
}

async fn do_post(client: Client, args: &Post) -> anyhow::Result<()> {
    let mut req_body = HashMap::new();
    for pair in args.body.iter() {
        req_body.insert(&pair.key, &pair.value);
    }
    let resp = client.post(&args.url).json(&req_body).send().await?;
    Ok((print_response(resp).await?))
}
