use bufstream::BufStream;
use prettytable::Table;

use native_tls::TlsStream;
use std::io::{BufRead, Error, ErrorKind, Read, Result, Write};
use std::net::TcpStream;
use std::str::FromStr;
use std::string::String;
use std::vec::Vec;

/// Commands
const LIST: &[u8; 6] = b"LIST\r\n";
const CAPABILITIES: &[u8; 14] = b"CAPABILITIES\r\n";
const ARTICLE: &[u8; 9] = b"ARTICLE\r\n";
const BODY: &[u8; 6] = b"BODY\r\n";
const DATE: &[u8; 6] = b"DATE\r\n";
const HEAD: &[u8; 6] = b"HEAD\r\n";
const LAST: &[u8; 6] = b"LAST\r\n";
const QUIT: &[u8; 6] = b"QUIT\r\n";
const HELP: &[u8; 6] = b"HELP\r\n";
const NEXT: &[u8; 6] = b"NEXT\r\n";
const POST: &[u8; 6] = b"POST\r\n";
const STAT: &[u8; 6] = b"STAT\r\n";
const ARTICLE_END: &[u8; 3] = b".\r\n";

pub struct Article {
    pub buf: Vec<u8>,
}

impl<'a> Article {
    pub fn parse(&'a self) -> Result<ParsedArticle<'a>> {
        ParsedArticle::from_buffer(&self.buf[..])
    }
}

pub struct ParsedArticle<'a> {
    pub headers: ParsedHeaders<'a>,
    pub body: &'a [u8],
}

impl<'a> ParsedArticle<'a> {
    pub fn from_buffer(buf: &[u8]) -> Result<ParsedArticle> {
        let (headers, buf) = ParsedHeaders::from_buffer(buf)?;

        Ok(ParsedArticle { headers, body: buf })
    }
}

pub struct Headers {
    buf: Vec<u8>,
}

impl Headers {
    pub fn size(&self) -> usize {
        self.buf.len()
    }

    pub fn parse(&self) -> Result<ParsedHeaders> {
        ParsedHeaders::from_buffer(&self.buf[..]).map(|(h, _)| h)
    }
}

pub struct ParsedHeaders<'a> {
    pub code: isize,
    pub message: &'a str,
    pub headers: Vec<(&'a str, &'a str)>,
}

impl<'a> std::fmt::Debug for ParsedHeaders<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "code: {}, message: {}", self.code, self.message)?;
        let mut table = Table::new();
        for (k, v) in self.headers.iter() {
            if v.len() < 50 {
                table.add_row(row![k, v]);
            } else {
                table.add_row(row![k, format!("{}...", &v[0..50])]);
            }
        }
        table.printstd();
        Ok(())
    }
}

impl<'a> ParsedHeaders<'a> {
    pub fn from_buffer(buf: &[u8]) -> Result<(ParsedHeaders, &[u8])> {
        let buf = &buf[..];
        let mut headers: Vec<(&str, &str)> = Vec::with_capacity(15);

        // snag response line
        let ((code, message), mut buf) = match ParsedHeaders::consume_line(buf) {
            (line, Some(rest)) => (ParsedHeaders::parse_response(line)?, rest),
            (_, None) => return Err(Error::new(ErrorKind::Other, "failed to consume a line")),
        };

        while let (line, Some(rest)) = ParsedHeaders::consume_line(buf) {
            buf = rest;

            if line.is_empty() {
                break;
            }

            if let Some(pos) = line.iter().position(|&x| x == b':') {
                headers.push((
                    std::str::from_utf8(&line[0..pos]).expect("header key is not valid UTF8"),
                    std::str::from_utf8(&line[pos + 2..]).expect("header value is not valid UTF8"),
                ));
            }
        }

        Ok((
            ParsedHeaders {
                headers,
                code,
                message,
            },
            buf,
        ))
    }

    fn consume_line(buffer: &[u8]) -> (&[u8], Option<&[u8]>) {
        let mut windows = buffer.windows(2).enumerate();
        let found = windows.find(|(_window_index, search)| search == b"\r\n");
        match found {
            Some((offset, _slice)) => {
                let line = &buffer[0..offset];
                if offset > buffer.len() {
                    (line, None)
                } else {
                    (line, Some(&buffer[offset + 2..]))
                }
            }
            _ => (buffer, None),
        }
    }

    fn parse_response(response: &[u8]) -> Result<(isize, &str)> {
        let (code, message) = match response.iter().position(|&x| x == b' ') {
            Some(pos) => response.split_at(pos),
            None => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "could not find a space in the response line",
                ))
            }
        };

        let code: isize = FromStr::from_str(std::str::from_utf8(code).expect("bad UTF8 for code"))
            .map_err(|_| Error::new(ErrorKind::Other, "failed to parse response code"))?;
        Ok((
            code,
            std::str::from_utf8(&message[1..]).expect("message is not valid UTF8"),
        ))
    }
}

#[derive(Debug)]
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
            name: split_group[0].to_string(),
            high: FromStr::from_str(split_group[1]).unwrap(),
            low: FromStr::from_str(split_group[2]).unwrap(),
            status: split_group[3].to_string(),
        }
    }
}

/// Stream to be used for interfacing with a NNTP server.
pub struct NNTPStream<W: Read + Write> {
    stream: BufStream<W>,
}

/// Response owns the blob returned by the server,
/// including unparsed response, headers, body
#[allow(dead_code)]
pub struct NNTPMessage {
    buf: Vec<u8>,
}

impl NNTPMessage {
    #[allow(clippy::type_complexity)]
    pub fn parse(&self) -> (isize, &[u8], Option<&[u8]>, Option<&[u8]>) {
        unimplemented!("bang")
    }
}

pub fn tls_buf_stream(hostname: &str, port: u16) -> Result<BufStream<TlsStream<TcpStream>>> {
    let tcp_stream = std::net::TcpStream::connect((hostname, port))?;

    let connector = native_tls::TlsConnector::new().unwrap();
    let stream = connector
        .connect(hostname, tcp_stream)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "tls failed"))?;
    Ok(BufStream::new(stream))
}

impl<W: Read + Write> NNTPStream<W> {
    /// Creates an NNTP Stream.
    pub fn connect(bufsock: BufStream<W>) -> Result<NNTPStream<W>> {
        let mut socket = NNTPStream { stream: bufsock };

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
        self.write_all(article_command)
            .map(|_| Error::new(ErrorKind::Other, "Failed to retrieve article"))?;

        let buf = self.read_article_buffer()?;

        let article = Article { buf };

        article.parse().expect("parse article");

        Ok(article)
    }

    fn read_article_buffer(&mut self) -> Result<Vec<u8>> {
        let mut buffer = vec![0; 2048];
        let mut bytes_read = 0;

        loop {
            match self.stream.read(&mut buffer[bytes_read..]) {
                Ok(0) => panic!("empty read"),
                Ok(bytes) => {
                    bytes_read += bytes;
                    println!("got {} bytes", bytes);
                    println!(
                        "buff: {}",
                        std::str::from_utf8(&buffer[0..bytes_read]).unwrap()
                    );

                    if &buffer[bytes_read - 3..bytes_read] == ARTICLE_END {
                        // Don't pass on the rest of 0'd data, skip the ARTICLE_END
                        buffer.truncate(bytes_read - 3);
                        break;
                    } else if buffer.len() == bytes_read {
                        // we gotta resize this buffer
                        let new_cap = buffer.capacity() * 2;
                        buffer.resize(new_cap, 0);
                        println!("read_article_buffer is resizing!")
                    }
                }
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "trouble reading the whole article",
                    ))
                }
            }
        }

        Ok(buffer)
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
        self.write_all(body_command)?;

        let (_code, _first_line) = self.read_response(222)?;

        self.read_buffered_multiline_response()
    }

    /// Gives the list of capabilities that the server has.
    pub fn capabilities(&mut self) -> Result<Vec<String>> {
        self.write_all(CAPABILITIES)?;

        let (_code, _first_line) = self.read_response(101)?;

        self.read_buffered_multiline_response()
    }

    /// Retrieves the date as the server sees the date.
    pub fn date(&mut self) -> Result<String> {
        self.write_all(DATE)?;

        self.read_response(111).map(|(_, body)| body)
    }

    /// Retrieves the headers of the current article number in the currently selected newsgroup.
    pub fn head(&mut self) -> Result<Headers> {
        self.retrieve_head(HEAD)
    }

    /// Retrieves the headers of the article id.
    pub fn head_by_id(&mut self, article_id: &str) -> Result<Headers> {
        self.retrieve_head(format!("HEAD {}\r\n", article_id).as_bytes())
    }

    /// Retrieves the headers of the article number in the currently selected newsgroup.
    pub fn head_by_number(&mut self, article_number: isize) -> Result<Headers> {
        self.retrieve_head(format!("HEAD {}\r\n", article_number).as_bytes())
    }

    fn retrieve_head(&mut self, head_command: &[u8]) -> Result<Headers> {
        self.write_all(head_command)?;

        self.read_response(100)?;

        let buf = self.read_article_buffer()?;

        Ok(Headers { buf })
    }

    /// Moves the currently selected article number back one
    pub fn last(&mut self) -> Result<String> {
        self.write_all(LAST)?;

        self.read_response(223).map(|(_, message)| message)
    }

    /// Lists all of the newgroups on the server.
    pub fn list(&mut self) -> Result<Vec<NewsGroup>> {
        self.write_all(LIST)?;

        let (_code, _first_line) = self.read_response(215)?;

        match self.read_buffered_multiline_response() {
            Ok(lines) => {
                let lines: Vec<NewsGroup> = lines
                    .iter()
                    .map(|ref mut x| NewsGroup::new_news_group(*x))
                    .collect();
                Ok(lines)
            }
            Err(e) => Err(e),
        }
    }

    /// Selects a newsgroup
    pub fn group(&mut self, group: &str) -> Result<()> {
        self.write_all(format!("GROUP {}\r\n", group).as_bytes())?;

        self.read_response(211).map(|_| ())
    }

    /// Show the help command given on the server.
    pub fn help(&mut self) -> Result<Vec<String>> {
        self.write_all(HELP)?;

        let (_code, _first_line) = self.read_response(100)?;

        self.read_buffered_multiline_response()
    }

    /// Quits the current session.
    pub fn quit(&mut self) -> Result<()> {
        self.write_all(QUIT)?;

        self.read_response(205).map(|_| ())
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

        self.read_buffered_multiline_response()
    }

    /// Retrieves a list of new news since the date and time given.
    pub fn newnews(
        &mut self,
        wildmat: &str,
        date: &str,
        time: &str,
        use_gmt: bool,
    ) -> Result<Vec<String>> {
        let newnews_command = if use_gmt {
            format!("NEWNEWS {} {} {} GMT\r\n", wildmat, date, time)
        } else {
            format!("NEWNEWS {} {} {}\r\n", wildmat, date, time)
        };

        self.write_all(newnews_command.as_bytes())?;

        self.read_response(230)?;

        self.read_buffered_multiline_response()
    }

    #[allow(clippy::should_implement_trait)]
    /// Moves the currently selected article number forward one
    pub fn next(&mut self) -> Result<String> {
        self.write_all(NEXT)?;

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

        self.write_all(POST)?;

        self.read_response(340)?;

        self.write_all(message.as_bytes())?;

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

    pub fn authinfo_user(&mut self, user: &str) -> Result<String> {
        self.write_all(&format!("AUTHINFO USER {}\r\n", user).as_bytes()[..])?;

        self.read_response(381).map(|(_code, message)| message)
    }

    pub fn authinfo_pass(&mut self, pass: &str) -> Result<String> {
        self.write_all(&format!("AUTHINFO PASS {}\r\n", pass).as_bytes()[..])?;

        self.read_response(281).map(|(_code, message)| message)
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.stream.write_all(buf)?;
        self.stream.flush()
    }

    fn retrieve_stat(&mut self, stat_command: &[u8]) -> Result<String> {
        self.write_all(stat_command)?;

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

        length >= 5
            && (message_bytes[length - 1] == lf
                && message_bytes[length - 2] == cr
                && message_bytes[length - 3] == dot
                && message_bytes[length - 4] == lf
                && message_bytes[length - 5] == cr)
    }

    //Retrieve single line response
    fn read_response(&mut self, expected_code: isize) -> Result<(isize, String)> {
        //        println!("reading a new response...");
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

        //        println!("done reading response from socket...");

        let response = String::from_utf8(line_buffer).unwrap();
        let chars_to_trim: &[char] = &['\r', '\n'];
        let trimmed_response = response.trim_matches(chars_to_trim);
        if trimmed_response.len() < 5 || &trimmed_response[3..4] != " " {
            return Err(Error::new(ErrorKind::Other, "Invalid response"));
        }

        let v: Vec<&str> = trimmed_response.splitn(2, ' ').collect();
        let code: isize = FromStr::from_str(v[0]).unwrap();
        let message = v[1];
        if code != expected_code {
            panic!("expected {}, got {}", expected_code, code);
            //            return Err(Error::new(ErrorKind::Other, "Invalid response"));
        }

        Ok((code, message.to_string()))
    }

    fn read_buffered_multiline_response(&mut self) -> Result<Vec<String>> {
        let mut output = Vec::new();

        let mut buf = &mut self.stream;
        let lines_iter = NNTPLines { buf: &mut buf };

        for line in lines_iter {
            match line {
                Ok(l) => output.push(unsafe { String::from_utf8_unchecked(l) }),
                Err(_) => return Err(Error::new(ErrorKind::Other, "problem reading lines")),
            }
        }

        Ok(output)
    }
}

struct NNTPLines<'a, W: Read + Write> {
    buf: &'a mut BufStream<W>,
}

/// A reimplementation of the BufReader::lines method with
/// some added logic for handling NNTP empty line signals
impl<'a, W: Read + Write> Iterator for NNTPLines<'a, W> {
    type Item = Result<Vec<u8>>;

    fn next(&mut self) -> Option<Result<Vec<u8>>> {
        let next = self.buf.lines().next();
        match next {
            Some(Ok(l)) => {
                if &l[..] == "." {
                    None
                } else {
                    Some(Ok(l.as_bytes().to_owned()))
                }
            }
            Some(Err(_)) => Some(Err(Error::new(ErrorKind::Other, "problem reading line"))),
            None => None,
        }
    }
}
