use std::collections::HashMap;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;
extern crate tokio;

use nntp::capabilities::{Capability, Compression};

use elasticsearch::http::request::JsonBody;
use elasticsearch::BulkParts;
use nntp::prelude::*;
use pretty_bytes::converter::convert;
use serde_json::Value;
use std::time::Instant;

#[tokio::main]
pub async fn main() -> Result<(), NNTPError> {
    env_logger::init();
    let elastic_client = elasticsearch::Elasticsearch::default();

    let mut client = Client::connect_tls("us.newsgroupdirect.com", 563, 32 * 1024)?;
    //    let mut client = Client::connect("nntp.aioe.org", 119)?;

    let response = client.read_response_line()?;
    assert!(response.starts_with("200"));

    let auth = true;
    if auth {
        use std::env;
        let envmap: HashMap<String, String> = env::vars().collect();
        client.authinfo_user(envmap.get("NEWSGROUP_USER").expect("newsgroup user"))?;
        client.authinfo_pass(envmap.get("NEWSGROUP_PASS").expect("newsgroup pass"))?;
    }

    //    let groups = client.list()?;
    //    info!("{:#?}", groups);

    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    //    let group_response = client.group("comp.sys.raspberry-pi")?;
    let group_name = "alt.binaries.frogs";
    let group_response = client.group(group_name)?;
    let (_num_articles, low_water_mark, high_water_mark) = group_response.parse_group_stats()?;
    info!(
        "num articles: {}, low water mark: {}, high water mark: {}",
        _num_articles, low_water_mark, high_water_mark
    );

    if client.can(Capability::XFEATURE_COMPRESS(vec![Compression::GZIP])) && false {
        let compression = client.xfeature_compress_gzip().expect("compression");
        println!("compression: {:#?}", compression);
    }

    debug!("gzip: {}", client.stream.gzip());
    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    let chunk_size = 2000;
    use itertools::Itertools;
    let chunks = &(low_water_mark..high_water_mark).chunks(chunk_size);

    let mut before_bytes: usize = client.stream.bytes_read();

    for chunk in chunks {
        let mut iter = chunk.into_iter();
        let first = iter.next().unwrap();
        let last = iter.last().unwrap();
        info!("{:?}-{:?}", first, last);

        let inst = std::time::Instant::now();
        for id in first..=last {
            client.head_by_id_pipeline_write(id)?;
        }
        client.flush().unwrap();
        info!("writing out all HEAD statements {:?}", inst.elapsed());

        let inst = Instant::now();
        let mut docs: Vec<Value> = Vec::with_capacity(chunk_size * 2);

        for id in first..=last {
            let res = client.head_by_id_read_pipeline()?;

            let headers = match res.headers() {
                Some(h) => h,
                None => {
                    info!("skipping a doc for {}", id);
                    continue;
                }
            };
            let json_value = serde_json::to_value(
                headers
                    .0
                    .iter()
                    .map(|(k, v)| {
                        (
                            std::str::from_utf8(k).unwrap(),
                            v.map(|x| std::str::from_utf8(x).unwrap()),
                        )
                    })
                    .collect::<HashMap<_, _>>(),
            );
            let doc = match json_value {
                Err(e) => panic!(
                    "could not serialize {:#?}\n{:?}",
                    res.headers().unwrap().0,
                    e
                ),
                Ok(d) => d,
            };

            let doc_id = format!("{}-{}", group_name, id);
            docs.push(json!({"index": {"_id": doc_id}}).into());
            docs.push(doc);
        }

        info!(
            "reading all HEAD responses took {:?} ({} read)",
            inst.elapsed(),
            convert((client.stream.bytes_read() - before_bytes) as f64)
        );
        before_bytes = client.stream.bytes_read();

        let inst = std::time::Instant::now();
        let res = elastic_client
            .bulk(BulkParts::Index("index-00001"))
            .body(docs.into_iter().map(|x| JsonBody::new(x)).collect())
            .send()
            .await;
        if res.is_err() {
            error!("bulk index failed {}", res.err().unwrap())
        } else {
            let response = res.unwrap();
            let body = response.read_body::<Value>().await.unwrap();
            if body["status"].as_u64() == Some(400) {
                panic!("elasticsearch is unhappy\n{:#?}", body);
            }

            let has_errors = body["errors"].as_bool();
            if has_errors == Some(true) {
                error!("encountered errors while indexing");
            } else {
                error!("no errors, whew");
            }
        }
        info!("bulk index done {:?}", inst.elapsed());
    }

    client.quit().map(|_| ())
}
