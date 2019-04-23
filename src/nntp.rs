use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::string::String;
use std::vec::Vec;

/// Commands
const LIST: &'static [u8; 6] = b"LIST\r\n";
const CAPABILITIES: &'static [u8; 14] = b"CAPABILITIES\r\n";
const ARTICLE: &'static [u8; 9] = b"ARTICLE\r\n";
const BODY: &'static [u8; 6] = b"BODY\r\n";
const DATE: &'static [u8; 6] = b"DATE\r\n";
const HEAD: &'static [u8; 6] = b"HEAD\r\n";
const LAST: &'static [u8; 6] = b"LAST\r\n";
const QUIT: &'static [u8; 6] = b"QUIT\r\n";
const HELP: &'static [u8; 6] = b"HELP\r\n";
const CRNL: &'static [u8; 2] = b"\r\n";
const NEXT: &'static [u8; 6] = b"NEXT\r\n";
const POST: &'static [u8; 6] = b"POST\r\n";
const STAT: &'static [u8; 6] = b"STAT\r\n";
const MULTILINE_END: &'static [u8; 3] = b".\r\n";

/// Stream to be used for interfacing with a NNTP server.
pub struct NNTPStream {
    stream: TcpStream,
}

pub struct Article {
    pub headers: HashMap<String, String>,
    pub body: Vec<String>,
}

impl Article {
    pub fn new_article(lines: Vec<String>) -> Article {
        let mut headers = HashMap::new();
        let mut body = Vec::new();
        let mut parsing_headers = true;

        for i in lines.iter() {
            if i.as_bytes() == CRNL {
                parsing_headers = false;
                continue;
            }
            if parsing_headers {
                let mut header = i.splitn(2, ':');
                let chars_to_trim: &[char] = &['\r', '\n'];
                let key = format!("{}", header.nth(0).unwrap().trim_matches(chars_to_trim));
                let value = format!("{}", header.nth(0).unwrap().trim_matches(chars_to_trim));
                headers.insert(key, value);
            } else {
                body.push(i.clone());
            }
        }
        Article {
            headers: headers,
            body: body,
        }
    }
}

pub struct NewsGroup {
    pub name: String,
    pub high: isize,
    pub low: isize,
    pub status: String,
}

impl NewsGroup {
    pub fn new_news_group(group: &str) -> NewsGroup {
        let chars_to_trim: &[char] = &['\r', '\n', ' '];
        let trimmed_group = group.trim_matches(chars_to_trim);
        let split_group: Vec<&str> = trimmed_group.split(' ').collect();
        NewsGroup {
            name: format!("{}", split_group[0]),
            high: FromStr::from_str(split_group[1]).unwrap(),
            low: FromStr::from_str(split_group[2]).unwrap(),
            status: format!("{}", split_group[3]),
        }
    }
}

impl NNTPStream {
    /// Creates an NNTP Stream.
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<NNTPStream> {
        let tcp_stream = TcpStream::connect(addr)?;
        let mut socket = NNTPStream { stream: tcp_stream };

        socket
            .read_response(200)
            .map_err(|_| Error::new(ErrorKind::Other, "Failed to read greeting response"))?;

        Ok(socket)
    }

    /// The article indicated by the current article number in the currently selected newsgroup is selected.
    pub fn article(&mut self) -> Result<Article> {
        self.retrieve_article(&ARTICLE[..])
    }

    /// The article indicated by the article id is selected.
    pub fn article_by_id(&mut self, article_id: &str) -> Result<Article> {
        self.retrieve_article(format!("ARTICLE {}\r\n", article_id).as_bytes())
    }

    /// The article indicated by the article number in the currently selected newsgroup is selected.
    pub fn article_by_number(&mut self, article_number: isize) -> Result<Article> {
        self.retrieve_article(format!("ARTICLE {}\r\n", article_number).as_bytes())
    }

    fn retrieve_article(&mut self, article_command: &[u8]) -> Result<Article> {
        self.stream
            .write_all(article_command)
            .map(|_| Error::new(ErrorKind::Other, "Failed to retrieve article"))?;

        let (_code, _first_line) = self.read_response(220)?;

        self.read_multiline_response()
            .map(|ls| Article::new_article(ls))
    }

    /// Retrieves the body of the current article number in the currently selected newsgroup.
    pub fn body(&mut self) -> Result<Vec<String>> {
        self.retrieve_body(BODY)
    }

    /// Retrieves the body of the article id.
    pub fn body_by_id(&mut self, article_id: &str) -> Result<Vec<String>> {
        self.retrieve_body(format!("BODY {}\r\n", article_id).as_bytes())
    }

    /// Retrieves the body of the article number in the currently selected newsgroup.
    pub fn body_by_number(&mut self, article_number: isize) -> Result<Vec<String>> {
        self.retrieve_body(format!("BODY {}\r\n", article_number).as_bytes())
    }

    fn retrieve_body(&mut self, body_command: &[u8]) -> Result<Vec<String>> {
        self.stream.write_all(body_command)?;

        let (_code, _first_line) = self.read_response(222)?;

        self.read_multiline_response()
    }

    /// Gives the list of capabilities that the server has.
    pub fn capabilities(&mut self) -> Result<Vec<String>> {
        self.stream.write_all(CAPABILITIES)?;

        let (_code, _first_line) = self.read_response(101)?;

        self.read_multiline_response()
    }

    /// Retrieves the date as the server sees the date.
    pub fn date(&mut self) -> Result<String> {
        self.stream.write_all(DATE)?;

        self.read_response(111).map(|(_, body)| body)
    }

    /// Retrieves the headers of the current article number in the currently selected newsgroup.
    pub fn head(&mut self) -> Result<Vec<String>> {
        self.retrieve_head(HEAD)
    }

    /// Retrieves the headers of the article id.
    pub fn head_by_id(&mut self, article_id: &str) -> Result<Vec<String>> {
        self.retrieve_head(format!("HEAD {}\r\n", article_id).as_bytes())
    }

    /// Retrieves the headers of the article number in the currently selected newsgroup.
    pub fn head_by_number(&mut self, article_number: isize) -> Result<Vec<String>> {
        self.retrieve_head(format!("HEAD {}\r\n", article_number).as_bytes())
    }

    fn retrieve_head(&mut self, head_command: &[u8]) -> Result<Vec<String>> {
        self.stream.write_all(head_command)?;

        let (_code, _first_line) = self.read_response(221)?;

        self.read_multiline_response()
    }

    /// Moves the currently selected article number back one
    pub fn last(&mut self) -> Result<String> {
        self.stream.write_all(LAST)?;

        self.read_response(223).map(|(_, message)| message)
    }

    /// Lists all of the newgroups on the server.
    pub fn list(&mut self) -> Result<Vec<NewsGroup>> {
        self.stream.write_all(LIST)?;

        let (_code, _first_line) = self.read_response(215)?;

        match self.read_multiline_response() {
            Ok(lines) => {
                let lines: Vec<NewsGroup> = lines
                    .iter()
                    .map(|ref mut x| NewsGroup::new_news_group(*x))
                    .collect();
                return Ok(lines);
            }
            Err(e) => Err(e),
        }
    }

    /// Selects a newsgroup
    pub fn group(&mut self, group: &str) -> Result<()> {
        self.stream
            .write_all(format!("GROUP {}\r\n", group).as_bytes())?;

        self.read_response(211).map(|_| ())
    }

    /// Show the help command given on the server.
    pub fn help(&mut self) -> Result<Vec<String>> {
        self.stream.write_all(HELP)?;

        let (_code, _first_line) = self.read_response(100)?;

        self.read_multiline_response()
    }

    /// Quits the current session.
    pub fn quit(&mut self) -> Result<()> {
        match self.stream.write_all(QUIT) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        match self.read_response(205) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Retrieves a list of newsgroups since the date and time given.
    pub fn newgroups(&mut self, date: &str, time: &str, use_gmt: bool) -> Result<Vec<String>> {
        let newgroups_command = if use_gmt {
            format!("NEWSGROUP {} {} GMT\r\n", date, time)
        } else {
            format!("NEWSGROUP {} {}\r\n", date, time)
        };

        match self.stream.write_fmt(format_args!("{}", newgroups_command)) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        match self.read_response(231) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }

        self.read_multiline_response()
    }

    /// Retrieves a list of new news since the date and time given.
    pub fn newnews(
        &mut self,
        wildmat: &str,
        date: &str,
        time: &str,
        use_gmt: bool,
    ) -> Result<Vec<String>> {
        let newnews_command = match use_gmt {
            true => format!("NEWNEWS {} {} {} GMT\r\n", wildmat, date, time),
            false => format!("NEWNEWS {} {} {}\r\n", wildmat, date, time),
        };

        self.stream.write_all(newnews_command.as_bytes())?;

        self.read_response(230)?;

        self.read_multiline_response()
    }

    /// Moves the currently selected article number forward one
    pub fn next(&mut self) -> Result<String> {
        self.stream.write_all(NEXT)?;

        self.read_response(223).map(|(_, message)| message)
    }

    /// Posts a message to the NNTP server.
    pub fn post(&mut self, message: &str) -> Result<()> {
        if !self.is_valid_message(message) {
            return Err(Error::new(
                ErrorKind::Other,
                "Invalid message format. Message must end with \"\r\n.\r\n\"",
            ));
        }

        self.stream.write_all(POST)?;

        self.read_response(340)?;

        self.stream.write_all(message.as_bytes())?;

        self.read_response(240).map(|_| ())
    }

    /// Gets information about the current article.
    pub fn stat(&mut self) -> Result<String> {
        self.retrieve_stat(STAT)
    }

    /// Gets the information about the article id.
    pub fn stat_by_id(&mut self, article_id: &str) -> Result<String> {
        self.retrieve_stat(format!("STAT {}\r\n", article_id).as_bytes())
    }

    /// Gets the information about the article number.
    pub fn stat_by_number(&mut self, article_number: isize) -> Result<String> {
        self.retrieve_stat(format!("STAT {}\r\n", article_number).as_bytes())
    }

    fn retrieve_stat(&mut self, stat_command: &[u8]) -> Result<String> {
        self.stream
            .write_all(stat_command)
            .map_err(|_| Error::new(ErrorKind::Other, "Write Error"))?;

        self.read_response(223).map(|(_, message)| message)
    }

    fn is_valid_message(&mut self, message: &str) -> bool {
        //Carriage return
        let cr = b'\r';
        //Line Feed
        let lf = b'\n';
        //Dot
        let dot = b'.';
        let message_string = message.to_string();
        let message_bytes = message_string.as_bytes();
        let length = message_string.len();

        return length >= 5
            && (message_bytes[length - 1] == lf
                && message_bytes[length - 2] == cr
                && message_bytes[length - 3] == dot
                && message_bytes[length - 4] == lf
                && message_bytes[length - 5] == cr);
    }

    //Retrieve single line response
    fn read_response(&mut self, expected_code: isize) -> Result<(isize, String)> {
        //Carriage return
        let cr: u8 = b'\r';
        //Line Feed
        let lf: u8 = b'\n';
        let mut line_buffer: Vec<u8> = Vec::new();

        while line_buffer.len() < 2
            || (line_buffer[line_buffer.len() - 1] != lf
                && line_buffer[line_buffer.len() - 2] != cr)
        {
            let byte_buffer: &mut [u8] = &mut [0];

            self.stream
                .read(byte_buffer)
                .map_err(|_| Error::new(ErrorKind::Other, "Error reading response"))?;
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
            return Err(Error::new(ErrorKind::Other, "Invalid response"));
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
            while line_buffer.len() < 2
                || (line_buffer[line_buffer.len() - 1] != lf
                    && line_buffer[line_buffer.len() - 2] != cr)
            {
                let byte_buffer: &mut [u8] = &mut [0];
                match self.stream.read(byte_buffer) {
                    Ok(_) => {}
                    Err(_) => println!("Error Reading!"),
                }
                line_buffer.push(byte_buffer[0]);
            }

            match String::from_utf8(line_buffer.clone()) {
                Ok(res) => {
                    if res.as_bytes() == MULTILINE_END {
                        complete = true;
                    } else {
                        response.push(res);
                        line_buffer = Vec::new();
                    }
                }
                Err(_) => return Err(Error::new(ErrorKind::Other, "Error Reading")),
            }
        }
        Ok(response)
    }
}
