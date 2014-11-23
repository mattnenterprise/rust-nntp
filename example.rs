extern crate nntp;

use nntp::{NNTPStream};

fn main() {
	let mut nntp_stream = match NNTPStream::connect("nntp.aioe.org", 119) {
		Ok(stream) => stream,
		Err(e) => panic!("{}", e)
	};

	match nntp_stream.capabilities() {
		Ok(lines) => {
			for line in lines.iter() {
				print!("{}", line);
			}
		},
		Err(e) => panic!(e)
	}

	match nntp_stream.list() {
		Ok(lines) => {
			for line in lines.iter() {
				print!("{}", line);
			}
		},
		Err(e) => panic!(e)
	};

	match nntp_stream.group("comp.sys.raspberry-pi") {
		Ok(_) => (),
		Err(e) => panic!(e)
	}

	match nntp_stream.article_by_number(6000) {
		Ok(lines) => {
			for line in lines.iter() {
				print!("{}", line);
			}
		},
		Err(e) => panic!(e)
	}

	match nntp_stream.article_by_id("<E2w*P06cv@news.chiark.greenend.org.uk>") {
		Ok(lines) => {
			for line in lines.iter() {
				print!("{}", line);
			}
		},
		Err(e) => panic!(e)
	}	

	let _ = nntp_stream.quit();
}