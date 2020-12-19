#![crate_name = "nntp"]
#![crate_type = "lib"]

use std::string::String;
use std::io::{Read, Result, Error, ErrorKind, Write};
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::vec::Vec;
use std::collections::HashMap;
use std::str::FromStr;

/// Stream to be used for interfacing with a NNTP server.
pub struct NNTPStream {
	stream: TcpStream,
}

pub struct Article {
	pub headers: HashMap<String, String>,
	pub body: Vec<String>
}

impl Article {
	pub fn new_article(lines: Vec<String>) -> Article {
		let mut headers = HashMap::new();
		let mut body = Vec::new();
		let mut parsing_headers = true;

		for i in lines.iter() {
			if i == &"\r\n".to_string() {
				parsing_headers = false;
				continue;
			}
			if parsing_headers {
				let mut header = i.splitn(2, ':');
				let chars_to_trim: &[char] = &['\r', '\n'];
				let key = header.next().unwrap().trim_matches(chars_to_trim).to_string();
				let value = header.next().unwrap().trim_matches(chars_to_trim).to_string();
				headers.insert(key, value);
			} else {
				body.push(i.clone());
			}

		}
		Article {headers, body}
	}
}

pub struct NewsGroup {
	pub name: String ,
	pub high: isize,
	pub low: isize,
	pub status: String
}

impl NewsGroup {
	pub fn new_news_group(group: &str) -> NewsGroup {
		let chars_to_trim: &[char] = &['\r', '\n', ' '];
		let trimmed_group = group.trim_matches(chars_to_trim);
		let split_group: Vec<&str> = trimmed_group.split(' ').collect();
		NewsGroup{name: split_group[0].to_string(), high: FromStr::from_str(split_group[1]).unwrap(), low: FromStr::from_str(split_group[2]).unwrap(), status: split_group[3].to_string()}
	}
}

impl NNTPStream {

	/// Creates an NNTP Stream.
	pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<NNTPStream> {
		let tcp_stream = TcpStream::connect(addr)?;
		let mut socket = NNTPStream { stream: tcp_stream };

		match socket.read_response(200) {
			Ok(_) => (),
			Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to read greeting response"))
		}

		Ok(socket)
	}

	/// The article indicated by the current article number in the currently selected newsgroup is selected.
	pub fn article(&mut self) -> Result<Article> {
		self.retrieve_article(&"ARTICLE\r\n".to_string())
	}

	/// The article indicated by the article id is selected.
	pub fn article_by_id(&mut self, article_id: &str) -> Result<Article> {
		self.retrieve_article(&format!("ARTICLE {}\r\n", article_id))
	}

	/// The article indicated by the article number in the currently selected newsgroup is selected.
	pub fn article_by_number(&mut self, article_number: isize) -> Result<Article> {
		self.retrieve_article(&format!("ARTICLE {}\r\n", article_number))
	}

	fn retrieve_article(&mut self, article_command: &str) -> Result<Article> {
		match self.stream.write_fmt(format_args!("{}", article_command)) {
			Ok(_) => (),
			Err(_) => return Err(Error::new(ErrorKind::Other, "Failed to retreive atricle"))
		}

		match self.read_response(220) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_multiline_response() {
			Ok(lines) => {
				Ok(Article::new_article(lines))
			}
			Err(e) => Err(e)
		}
	}

	/// Retrieves the body of the current article number in the currently selected newsgroup.
	pub fn body(&mut self) -> Result<Vec<String>> {
		self.retrieve_body(&"BODY\r\n".to_string())
	}

	/// Retrieves the body of the article id.
	pub fn body_by_id(&mut self, article_id: &str) -> Result<Vec<String>> {
		self.retrieve_body(&format!("BODY {}\r\n", article_id))
	}

	/// Retrieves the body of the article number in the currently selected newsgroup.
	pub fn body_by_number(&mut self, article_number: isize) -> Result<Vec<String>> {
		self.retrieve_body(&format!("BODY {}\r\n", article_number))
	}

	fn retrieve_body(&mut self, body_command: &str) -> Result<Vec<String>> {
		match self.stream.write_fmt(format_args!("{}", body_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(222) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		self.read_multiline_response()
	}

	/// Gives the list of capabilities that the server has.
	pub fn capabilities(&mut self) -> Result<Vec<String>> {
		let capabilities_command = "CAPABILITIES\r\n".to_string();

		match self.stream.write_fmt(format_args!("{}", capabilities_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(101) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		self.read_multiline_response()
	}

	/// Retrieves the date as the server sees the date.
	pub fn date(&mut self) -> Result<String> {
		let date_command = "DATE\r\n".to_string();

		match self.stream.write_fmt(format_args!("{}", date_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(111) {
			Ok((_, message)) => Ok(message),
			Err(e) => Err(e)
		}
	}

	/// Retrieves the headers of the current article number in the currently selected newsgroup.
	pub fn head(&mut self) -> Result<Vec<String>> {
		self.retrieve_head(&"HEAD\r\n".to_string())
	}

	/// Retrieves the headers of the article id.
	pub fn head_by_id(&mut self, article_id: &str) -> Result<Vec<String>> {
		self.retrieve_head(&format!("HEAD {}\r\n", article_id))
	}

	/// Retrieves the headers of the article number in the currently selected newsgroup.
	pub fn head_by_number(&mut self, article_number: isize) -> Result<Vec<String>> {
		self.retrieve_head(&format!("HEAD {}\r\n", article_number))
	}

	fn retrieve_head(&mut self, head_command: &str) -> Result<Vec<String>> {
		match self.stream.write_fmt(format_args!("{}", head_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(221) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		self.read_multiline_response()
	}

	/// Moves the currently selected article number back one
	pub fn last(&mut self) -> Result<String> {
		let last_command = "LAST\r\n".to_string();

		match self.stream.write_fmt(format_args!("{}", last_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(223) {
			Ok((_, message)) => Ok(message),
			Err(e) => Err(e)
		}
	}

	/// Lists all of the newgroups on the server.
	pub fn list(&mut self) -> Result<Vec<NewsGroup>> {
		let list_command = "LIST\r\n".to_string();

		match self.stream.write_fmt(format_args!("{}", list_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(215) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_multiline_response() {
			Ok(lines) => {
				let lines: Vec<NewsGroup> = lines.iter().map(|ref mut x| NewsGroup::new_news_group(*x)).collect();
				Ok(lines)
			},
			Err(e) => Err(e)
		}
	}

	/// Selects a newsgroup
	pub fn group(&mut self, group: &str) -> Result<()> {
		let group_command = format!("GROUP {}\r\n", group);

		match self.stream.write_fmt(format_args!("{}", group_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(211) {
			Ok(_) => Ok(()),
			Err(e) => Err(e)
		}
	}

	/// Show the help command given on the server.
	pub fn help(&mut self) -> Result<Vec<String>> {
		let help_command = "HELP\r\n".to_string();

		match self.stream.write_fmt(format_args!("{}", help_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(100) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		self.read_multiline_response()
	}

	/// Quits the current session.
	pub fn quit(&mut self) -> Result<()> {
		let quit_command = "QUIT\r\n".to_string();
		match self.stream.write_fmt(format_args!("{}", quit_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(205) {
			Ok(_) => Ok(()),
			Err(e) => Err(e)
		}
	}

	/// Retrieves a list of newsgroups since the date and time given.
	pub fn newgroups(&mut self, date: &str, time: &str, use_gmt: bool) -> Result<Vec<String>> {
		let newgroups_command = match use_gmt {
			true => format!("NEWSGROUP {} {} GMT\r\n", date, time),
			false => format!("NEWSGROUP {} {}\r\n", date, time)
		};

		match self.stream.write_fmt(format_args!("{}", newgroups_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(231) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		self.read_multiline_response()
	}

	/// Retrieves a list of new news since the date and time given.
	pub fn newnews(&mut self, wildmat: &str, date: &str, time: &str, use_gmt: bool) -> Result<Vec<String>> {
		let newnews_command = match use_gmt {
			true => format!("NEWNEWS {} {} {} GMT\r\n", wildmat, date, time),
			false => format!("NEWNEWS {} {} {}\r\n", wildmat, date, time)
		};

		match self.stream.write_fmt(format_args!("{}", newnews_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(230) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		self.read_multiline_response()
	}

	/// Moves the currently selected article number forward one
	pub fn next(&mut self) -> Result<String> {
		let next_command = "NEXT\r\n".to_string();
		match self.stream.write_fmt(format_args!("{}", next_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(223) {
			Ok((_, message)) => Ok(message),
			Err(e) => Err(e)
		}
	}

	/// Posts a message to the NNTP server.
	pub fn post(&mut self, message: &str) -> Result<()> {
		if !self.is_valid_message(message) {
			return Err(Error::new(ErrorKind::Other, "Invalid message format. Message must end with \"\r\n.\r\n\""));
		}

		let post_command = "POST\r\n".to_string();

		match self.stream.write_fmt(format_args!("{}", post_command)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(340) {
			Ok(_) => (),
			Err(e) => return Err(e)
		};

		match self.stream.write_fmt(format_args!("{}", message)) {
			Ok(_) => (),
			Err(e) => return Err(e)
		}

		match self.read_response(240) {
			Ok(_) => Ok(()),
			Err(e) => Err(e)
		}
	}

	/// Gets information about the current article.
	pub fn stat(&mut self) -> Result<String> {
		self.retrieve_stat(&"STAT\r\n".to_string())
	}

	/// Gets the information about the article id.
	pub fn stat_by_id(&mut self, article_id: &str) -> Result<String> {
		self.retrieve_stat(&format!("STAT {}\r\n", article_id))
	}

	/// Gets the information about the article number.
	pub fn stat_by_number(&mut self, article_number: isize) -> Result<String> {
		self.retrieve_stat(&format!("STAT {}\r\n", article_number))
	}

	fn retrieve_stat(&mut self, stat_command: &str) -> Result<String> {
		match self.stream.write_fmt(format_args!("{}", stat_command)) {
			Ok(_) => (),
			Err(_) => return Err(Error::new(ErrorKind::Other, "Write Error"))
		}

		match self.read_response(223) {
			Ok((_, message)) => Ok(message),
			Err(e) => Err(e)
		}
	}

	fn is_valid_message(&mut self, message: &str) -> bool {
		//Carriage return
		let cr = 0x0d;
		//Line Feed
		let lf = 0x0a;
		//Dot
		let dot = 0x2e;
		let message_string = message.to_string();
		let message_bytes = message_string.as_bytes();
		let length = message_string.len();

		length >= 5 && (message_bytes[length-1] == lf && message_bytes[length-2] == cr &&
			message_bytes[length-3] == dot && message_bytes[length-4] == lf && message_bytes[length-5] == cr)
	}

	//Retrieve single line response
	fn read_response(&mut self, expected_code: isize) -> Result<(isize, String)> {
		//Carriage return
		let cr = 0x0d;
		//Line Feed
		let lf = 0x0a;
		let mut line_buffer: Vec<u8> = Vec::new();

		while line_buffer.len() < 2 || (line_buffer[line_buffer.len()-1] != lf && line_buffer[line_buffer.len()-2] != cr) {
				let byte_buffer: &mut [u8] = &mut [0];
				match self.stream.read(byte_buffer) {
					Ok(_) => {},
					Err(_) => return Err(Error::new(ErrorKind::Other, "Error reading response")),
				}
				line_buffer.push(byte_buffer[0]);
		}

		let response = String::from_utf8(line_buffer).unwrap();
		let chars_to_trim: &[char] = &['\r', '\n'];
		let trimmed_response = response.trim_matches(chars_to_trim);
    	let trimmed_response_vec: Vec<char> = trimmed_response.chars().collect();
    	if trimmed_response_vec.len() < 5 || trimmed_response_vec[3] != ' ' {
    		return Err(Error::new(ErrorKind::Other, "Invalid response"));
    	}

    	let v: Vec<&str> = trimmed_response.splitn(2, ' ').collect();
    	let code: isize = FromStr::from_str(v[0]).unwrap();
    	let message = v[1];
    	if code != expected_code {
    		return Err(Error::new(ErrorKind::Other, "Invalid response"))
    	}
    	Ok((code, message.to_string()))
	}

	fn read_multiline_response(&mut self) -> Result<Vec<String>> {
		let mut response: Vec<String> = Vec::new();
		//Carriage return
		let cr = 0x0d;
		//Line Feed
		let lf = 0x0a;
		let mut line_buffer: Vec<u8> = Vec::new();
		let mut complete = false;

		while !complete {
			while line_buffer.len() < 2 || (line_buffer[line_buffer.len()-1] != lf && line_buffer[line_buffer.len()-2] != cr) {
				let byte_buffer: &mut [u8] = &mut [0];
				match self.stream.read(byte_buffer) {
					Ok(_) => {},
					Err(_) => println!("Error Reading!"),
				}
				line_buffer.push(byte_buffer[0]);
			}

			match String::from_utf8(line_buffer.clone()) {
        		Ok(res) => {
        			if res == *".\r\n".to_string() {
        				complete = true;
        			}
        			else {
          				response.push(res.clone());
            			line_buffer = Vec::new();
            		}
        		},
        		Err(_) => return Err(Error::new(ErrorKind::Other, "Error Reading"))
      		}
		}
		Ok(response)
	}
}
