extern crate nntp;

use nntp::{Article, NNTPStream};

fn main() -> Result<(), std::io::Error> {
    let mut nntp_stream = NNTPStream::connect(("nntp.aioe.org", 119))?;

    let lines = nntp_stream.capabilities()?;
    for line in lines.iter() {
        print!("{}", line);
    }

	let groups = nntp_stream.list()?;
	for group in groups.iter() {
		println!("Name: {}, High: {}, Low: {}, Status: {}", group.name, group.high, group.low, group.status)
	}

    nntp_stream.group("comp.sys.raspberry-pi")?;

    let Article { headers, body } = nntp_stream.article_by_number(20000)?;
    for (key, value) in headers.iter() {
        println!("{}: {}", key, value)
    }
    for line in body.iter() {
        println!("{}", line)
    }

    let Article { headers, body } =
        nntp_stream.article_by_id("<a55pbedl7rf6sr0h1d9bf37q5qpj0rgn5j@4ax.com>")?;
    for (key, value) in headers.iter() {
        println!("{}: {}", key, value)
    }
    for line in body.iter() {
        println!("{}", line)
    }

    println!("COMMAND: quit");
    nntp_stream.quit()
}
