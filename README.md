rust-nntp
================
NNTP Client for Rust


[![Build Status](https://travis-ci.org/mattnenterprise/rust-imap.svg)](https://travis-ci.org/mattnenterprise/rust-imap)
[![crates.io](http://meritbadge.herokuapp.com/nntp)](https://crates.io/crates/nntp)

### Usage
```rust
extern crate nntp;

use nntp::{Article, NNTPStream};

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
		Ok(groups) => {
			for group in groups.iter() {
				println!("Name: {}, High: {}, Low: {}, Status: {}", group.name, group.high, group.low, group.status)
			}
		},
		Err(e) => panic!(e)
	};

	match nntp_stream.group("comp.sys.raspberry-pi") {
		Ok(_) => (),
		Err(e) => panic!(e)
	}

	match nntp_stream.article_by_number(6187) {
		Ok(Article{headers, body}) => {
			for (key, value) in headers.iter() {
				println!("{}: {}", key, value)
			}
			for line in body.iter() {
				print!("{}", line)
			}
		},
		Err(e) => panic!(e)
	}

	match nntp_stream.article_by_id("<cakj55F1dofU5@mid.individual.net>") {
		Ok(Article{headers, body}) => {
			for (key, value) in headers.iter() {
				println!("{}: {}", key, value)
			}
			for line in body.iter() {
				print!("{}", line)
			}
		},
		Err(e) => panic!(e)
	}

	let _ = nntp_stream.quit();
}
```

### License

MIT
## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
