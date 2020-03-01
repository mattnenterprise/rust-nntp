extern crate nntp;
#[allow(unused_imports)]
#[macro_use]
extern crate prettytable;
extern crate bufstream;

#[allow(unused_imports)]
use std::collections::HashMap;

use nntp::{NNTPStream, NewsGroup};
#[allow(unused_imports)]
use prettytable::Table;

use bufstream::BufStream;

fn main() -> Result<(), std::io::Error> {
    let tcp_stream = std::net::TcpStream::connect(("us.newsgroupdirect.com", 563))?;

    let connector = native_tls::TlsConnector::new().unwrap();
    let stream = connector
        .connect("us.newsgroupdirect.com", tcp_stream)
        .map_err(|_x| std::io::Error::new(std::io::ErrorKind::Other, "tls failed"))?;
    let stream = BufStream::new(stream);
    let mut nntp_stream = NNTPStream::connect(stream)?;

    //    let GROUP = "comp.sys.raspberry-pi";
    let group = "alt.binaries.warez";

    use std::env;
    let envmap: HashMap<String, String> = env::vars().collect();
    nntp_stream.authinfo_user(envmap.get("NEWSGROUP_USER").expect("newsgroup user"))?;
    nntp_stream.authinfo_pass(envmap.get("NEWSGROUP_PASS").expect("newsgroup pass"))?;

    let _cap = nntp_stream.capabilities()?;
    // panic!("cap (after AUTH): {:#?}", cap);

    let groups = nntp_stream.list()?;
    let groups_by_name: HashMap<&str, &NewsGroup> =
        groups.iter().map(|x| (&x.name[..], x)).collect();

    let g = *groups_by_name.get(group).unwrap();
    println!("the g: {:#?}", g);

    let mut t = Table::new();
    for (name, _) in groups_by_name.iter() {
        t.add_row(row![name]);
    }
    t.printstd();
    //
    nntp_stream.group(group)?;
    let _article = nntp_stream.article()?;
    let _stat = nntp_stream.stat()?;
    //    let article = nntp_stream.article_by_number(3269684000).unwrap();
    //    let article = article.parse()?;
    //    let mut t = Table::new();
    //    for (k,v) in article.headers.headers.iter() {
    //        t.add_row(row![k,v]);
    //    }
    //    t.printstd();
    //    println!("body size: {}", article.body.len());

    let mut failure_count = 0;

    println!("going through the group");
    while nntp_stream.next().is_ok() {
        match nntp_stream.head() {
            Ok(headers) => {
                println!("parsing headers");
                let parsed = headers.parse()?;
                println!("{:?}", parsed);
                println!("code: {}, message: {}", parsed.code, parsed.message);
                println!("bleep: {:?}", parsed.headers[0])
            }
            Err(e) => {
                failure_count += 1;
                if failure_count > 3 {
                    panic!("too many failures");
                }
                println!("error {}...", e);
                continue;
            }
        }
    }

    let last_response = nntp_stream.next()?;
    println!("response: {}", last_response);
    let _ = nntp_stream.head()?;
    //    panic!("got whatever\n{:#?}", whatever);

    println!("COMMAND: quit");
    nntp_stream.quit()
}
