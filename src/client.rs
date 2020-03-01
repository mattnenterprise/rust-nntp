use crate::prelude::*;

use std::io::{Read, Write};

use native_tls::TlsStream;
use std::net::TcpStream;

use super::capabilities::Capability;
use super::response::Response;
use super::stream::Stream;

use flate2::{Decompress, FlushDecompress};

const LIST: &str = "LIST\r\n";
const CAPABILITIES: &str = "CAPABILITIES\r\n";
//const ARTICLE: &'static [u8; 9] = b"ARTICLE\r\n";
//const BODY: &'static [u8; 6] = b"BODY\r\n";
//const DATE: &'static [u8; 6] = b"DATE\r\n";
const HEAD: &str = "HEAD\r\n";
const LAST: &str = "LAST\r\n";
const QUIT: &str = "QUIT\r\n";
//const HELP: &'static [u8; 6] = b"HELP\r\n";
const NEXT: &str = "NEXT\r\n";
//const POST: &'static [u8; 6] = b"POST\r\n";
//const STAT: &'static [u8; 6] = b"STAT\r\n";
//const ARTICLE_END : &'static [u8; 3] = b".\r\n";

macro_rules! simple_command_and_check_code {
    ($fnname:ident, $command:expr, $code:expr) => {
        pub fn $fnname(&mut self) -> Result<String, NNTPError> {
            self.stream.write_all($command)?;

            let response_line = self.read_response_line();

            let _line: Result<String, NNTPError> = match response_line {
                Ok(resp) => {
                    if !resp.starts_with($code) {
                        println!("expected {}, got {}", $code, &resp[0..3])
                    }
                    Ok(resp)
                }
                Err(e) => {
                    panic!("got {:?}", e);
                }
            };

            let rest = self.stream.read_to_terminal().unwrap();
            let rest = String::from_utf8(rest)?;
            Ok(rest)
        }
    };
}

#[allow(unused_macros)]
macro_rules! utf8_plz {
    ($rest:expr) => {
        std::str::from_utf8(&$rest[..]).unwrap_or("bad utf8 buddy".into())
    };
}

pub struct Client<W: Read + Write> {
    pub stream: Stream<W>,
    pub capabilities: Option<Vec<Capability>>,
}

impl<W: Read + Write> Client<W> {
    pub fn new(stream: Stream<W>) -> Client<W> {
        Client {
            stream,
            capabilities: None,
        }
    }

    pub fn flush(&mut self) -> Result<(), NNTPError> {
        self.stream.flush()
    }

    #[inline]
    pub fn read_response_line(&mut self) -> Result<String, NNTPError> {
        self.stream.read_response_line()
    }

    /// Ask for CAPABILITIES and use response to replace this client's capabilities map
    #[allow(unreachable_code)]
    pub fn discovery_capabilities(&mut self) -> Result<(), NNTPError> {
        self.stream.write_all(CAPABILITIES)?;
        //        let mut response = self.read_response()?;
        let response_line = self.read_response_line()?;
        assert_eq!(&response_line[0..3], "101");

        let body = self.stream.read_to_terminal()?;

        let rest = if self.stream.gzip() {
            let mut decompressor = Decompress::new(true);

            let mut decompress_buffer = Vec::with_capacity(10 * body.len());
            let _flat_response = decompressor
                .decompress_vec(&body[..], &mut decompress_buffer, FlushDecompress::None)
                .expect("hello deflation");

            debug!("total out: {}", decompressor.total_out());

            decompress_buffer.truncate(decompressor.total_out() as usize);
            String::from_utf8(decompress_buffer).expect("valid utf8 for gzipped capabilities")
        } else {
            String::from_utf8(body).expect("valid utf8 for capabilities")
        };

        let caps: Vec<Capability> = rest.lines().map(|x| x.into()).collect();

        self.capabilities.replace(caps);

        Ok(())
    }

    pub fn can(&self, ask_cap: Capability) -> bool {
        if let Some(ref caps) = self.capabilities {
            if let Capability::XFEATURE_COMPRESS(ref ask_xfeatures) = ask_cap {
                for possible_cap in caps {
                    if let Capability::XFEATURE_COMPRESS(ref xfeatures) = possible_cap {
                        let set: HashSet<_> = xfeatures.iter().collect();
                        let ask_set: HashSet<_> = ask_xfeatures.iter().collect();
                        return ask_set.is_subset(&set);
                    }
                }
                false
            } else {
                caps.contains(&ask_cap)
            }
        } else {
            false
        }
    }

    pub fn authinfo_user(&mut self, user: &str) -> Result<Response, NNTPError> {
        self.stream
            .write_all(&format!("AUTHINFO USER {}\r\n", user)[..])?;

        let response_line = self.read_response_line()?;

        Ok(Response::new(response_line, None))
    }

    pub fn authinfo_pass(&mut self, pass: &str) -> Result<Response, NNTPError> {
        self.stream
            .write_all(&format!("AUTHINFO PASS {}\r\n", pass)[..])?;

        let response = self.read_response_line()?;
        Ok(Response::new(response, None))
    }

    simple_command_and_check_code!(head, HEAD, "205");
    simple_command_and_check_code!(quit, QUIT, "205");
    simple_command_and_check_code!(list, LIST, "215");
    simple_command_and_check_code!(_next, NEXT, "223");
    simple_command_and_check_code!(last, LAST, "205");

    /// Selects a newsgroup
    pub fn group(&mut self, group: &str) -> Result<Response, NNTPError> {
        self.stream.write_all(&format!("GROUP {}\r\n", group)[..])?;

        let response = self.read_response_line()?;

        Ok(Response::new(response, None))
    }

    //    pub fn list(&mut self) -> Result<Response, NNTPError> {
    //        self.stream.write_all("LIST\r\n")?;
    //
    //        let response = self.read_response_line()?;
    //        panic!("response: {}", response);
    //    }

    /// Lists articles in a group, you probably don't want this
    pub fn listgroup(&mut self) -> Result<Response, NNTPError> {
        self.stream.write_all("LISTGROUP\r\n")?;

        let response = self.read_response_line()?;
        info!("listgroup response line `{}`", response);
        let _rest = self.stream.read_to_terminal_noisey()?;
        //        panic!("response: {:#?}/{}", response, rest.len());

        Ok(Response::new(response, None))
    }

    /// Lists articles in a group based on the provided range, you probably don't want this
    pub fn listgroup_range(
        &mut self,
        group: &str,
        thing: std::ops::Range<usize>,
    ) -> Result<Response, NNTPError> {
        let command = format!("LISTGROUP {} {}-{}\r\n", group, thing.start, thing.end);
        self.stream.write_all(&command[..])?;

        let response = self.read_response_line()?;
        println!("got response: {:?}", response);
        let _rest = self.stream.read_to_terminal()?;

        Ok(Response::new(response, None))
    }

    /// Lists articles in a group, you probably don't want this
    pub fn article_by_id(&mut self, id: usize) -> Result<Response, NNTPError> {
        self.article_by_id_pipeline_write(id)?;
        self.article_by_id_pipeline_read()
    }

    /// Lists articles in a group, you probably don't want this
    pub fn article_by_id_pipeline_write(&mut self, id: usize) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("ARTICLE {}\r\n", id)[..])
            .map_err(|e| e)
    }

    pub fn article_by_id_pipeline_read(&mut self) -> Result<Response, NNTPError> {
        let response = self.read_response_line()?;

        // If it's not a 220, we shouldn't bother reading the rest
        if !response.starts_with("220") {
            return Ok(Response::new(response, None));
        }

        let header = self.stream.read_to_terminal()?;
        let header = String::from_utf8(header)?;

        Ok(Response::new(response, Some(header)))
    }

    pub fn xfeature_compress_gzip(&mut self) -> Result<Response, NNTPError> {
        self.stream.write_all("XFEATURE COMPRESS GZIP *\r\n")?;

        let response_line = self.read_response_line()?;

        // If it's not a 220, we shouldn't bother reading the rest
        if !response_line.starts_with("220") {
            return Err(NNTPError::UnexpectedCode(response_line[0..2].to_string()));
        }

        self.stream.set_gzip();

        let header = self.stream.read_to_terminal()?;
        let header = String::from_utf8(header)?;

        Ok(Response::new(response_line, Some(header)))
    }

    /// Retrieves the headers of the article id.
    pub fn head_by_id(&mut self, article_id: usize) -> Result<Response, NNTPError> {
        self.head_by_id_pipeline_write(article_id)?;
        self.head_by_id_read_pipeline()
    }

    pub fn head_by_id_pipeline_write(&mut self, article_id: usize) -> Result<(), NNTPError> {
        self.stream
            .write_all_buffered(&format!("HEAD {}\r\n", article_id)[..])
            .map_err(|e| e)
    }

    pub fn head_by_range_pipeline_write(
        &mut self,
        articles: std::ops::Range<usize>,
    ) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("HEAD {}-{}\r\n", articles.start, articles.end)[..])
            .map_err(|e| e)
    }

    pub fn xhdr_by_range_pipeline_write(
        &mut self,
        header_name: String,
        articles: std::ops::Range<usize>,
    ) -> Result<(), NNTPError> {
        let cmd = format!(
            "XHDR {} {}-{}\r\n",
            header_name, articles.start, articles.end
        );
        self.stream.write_all(&cmd[..])
    }

    pub fn xzhdr_by_id_pipeline_write(&mut self, article_id: usize) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("XZHDR {}\r\n", article_id)[..])
    }

    pub fn xzhdr_by_range_pipeline_write(
        &mut self,
        articles: std::ops::Range<usize>,
    ) -> Result<(), NNTPError> {
        self.stream
            .write_all(&format!("XZHDR {}-{}\r\n", articles.start, articles.end)[..])
    }

    pub fn head_by_id_read_pipeline(&mut self) -> Result<Response, NNTPError> {
        let response = self.read_response_line()?;

        // If it's not a 100, we shouldn't bother reading the rest
        if !(response.starts_with("100") || response.starts_with("221")) {
            //            panic!("no me gusta `{}`", response.response_line);
            return Ok(Response::new(response, None));
        }

        let header = self.stream.read_to_terminal_noisey()?;
        let header = String::from_utf8(header)?;

        Ok(Response::new(response, Some(header)))
    }

    pub fn xzhdr_by_id_read_pipeline(&mut self) -> Result<Response, NNTPError> {
        let response = self.read_response_line()?;
        println!("response: {:#?}", response);

        // If it's not a 100, we shouldn't bother reading the rest
        if !(response.starts_with("100") || response.starts_with("221")) {
            //            panic!("no me gusta `{}`", response.response_line);
            return Ok(Response::new(response, None));
        }

        let header = self.stream.read_to_terminal_noisey()?;
        let header = String::from_utf8(header)?;

        Ok(Response::new(response, Some(header)))
    }

    pub fn xhdr_by_id_read_pipeline(&mut self) -> Result<Response, NNTPError> {
        let response = self.read_response_line()?;
        println!("response: {:#?}", response);

        // If it's not a 100, we shouldn't bother reading the rest
        if !(response.starts_with("100") || response.starts_with("221")) {
            //            panic!("no me gusta `{}`", response.response_line);
            return Ok(Response::new(response, None));
        }

        let header = self.stream.read_to_terminal_noisey()?;
        let header = String::from_utf8(header)?;

        Ok(Response::new(response, Some(header)))
    }
}

use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result as FmtResult};

impl Debug for Client<TlsStream<TcpStream>> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Client")
            .field("stream", &self.stream)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl Debug for Client<TcpStream> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_struct("Client")
            .field("stream", &self.stream)
            .field("capabilities", &self.capabilities)
            .finish()
    }
}

impl Client<TcpStream> {
    /// Helper to easily connect to a host
    pub fn connect(host: &str, port: u16) -> Result<Client<TcpStream>, NNTPError> {
        let stream = Stream::connect(host, port)?;

        Ok(Client::new(stream))
    }
}

impl Client<TlsStream<TcpStream>> {
    /// Helper to easily connect to a TLS host
    pub fn connect_tls(
        host: &str,
        port: u16,
        buf_size: usize,
    ) -> Result<Client<TlsStream<TcpStream>>, NNTPError> {
        let stream = Stream::connect_tls(host, port, buf_size)?;

        Ok(Client::new(stream))
    }
}
