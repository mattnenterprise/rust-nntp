extern crate nntp;
#[macro_use]
extern crate prettytable;

use nntp::{NNTPStream, ParsedArticle};
use prettytable::Table;

fn main() -> Result<(), std::io::Error> {
    let buf_stream = nntp::tls_buf_stream("nntp.aioe.org", 119)?;
    let mut nntp_stream = NNTPStream::connect(buf_stream)?;

    let lines = nntp_stream.capabilities()?;
    for line in lines.iter() {
        print!("{}", line);
    }

    let groups = nntp_stream.list()?;
    for group in groups.iter() {
        println!(
            "Name: {}, High: {}, Low: {}, Status: {}",
            group.name, group.high, group.low, group.status
        )
    }

    nntp_stream.group("comp.sys.raspberry-pi")?;

    let article = nntp_stream.article_by_number(20000)?;
    let ParsedArticle { headers, body, .. } = article.parse()?;
    let mut table = Table::new();
    for (key, value) in headers.headers.iter() {
        table.add_row(row![key, value]);
    }
    table.printstd();

    println!("==== BODY ====");
    println!("{}", std::str::from_utf8(body).expect("valid utf8"));

    let article = nntp_stream.article_by_id("<a55pbedl7rf6sr0h1d9bf37q5qpj0rgn5j@4ax.com>")?;
    let ParsedArticle { headers, body, .. } = article.parse()?;
    let mut table = Table::new();
    for (key, value) in headers.headers.iter() {
        table.add_row(row![key, value]);
    }
    table.printstd();

    println!("==== BODY ====");
    println!("{}", std::str::from_utf8(body).expect("valid utf8"));

    println!("COMMAND: quit");
    nntp_stream.quit()
}
