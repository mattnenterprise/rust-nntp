#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Capability {
    VERSION_1,
    VERSION_2,
    VERSION(String),
    AUTHINFO(String),
    HDR,
    LIST(Vec<String>),
    ACTIVE,
    COUNTS,
    DISTRIBUTIONS,
    HEADERS,
    MODERATORS,
    MOTD,
    NEWSGROUPS,
    SUBSCRIPTIONS,
    NEWNEWS,
    OVER,
    POST,
    READER,
    SASL(Vec<String>),
    STARTTLS,
    OTHER(Vec<String>),
    MODE_READER,
    XHDR,
    XOVER,
    XZVER,
    XZHDR,
    XFEATURE_COMPRESS(Vec<Compression>),
}

impl From<&str> for Capability {
    fn from(incoming: &str) -> Self {
        let parts: Vec<&str> = incoming.split(' ').collect();
        if parts.len() == 1 {
            match parts[0] {
                "NEWNEWS" => Capability::NEWNEWS,
                "OVER" => Capability::OVER,
                "POST" => Capability::POST,
                "READER" => Capability::READER,
                "MODE-READER" => Capability::MODE_READER,
                "XHDR" => Capability::XHDR,
                "XOVER" => Capability::XOVER,
                "XZVER" => Capability::XZVER,
                "XZHDR" => Capability::XZHDR,
                "HDR" => Capability::HDR,
                _ => Capability::OTHER(parts.iter().map(|&x| x.to_owned()).collect()),
            }
        } else if parts.len() == 2 {
            match (parts[0], parts[1]) {
                ("VERSION", "1") => Capability::VERSION_1,
                ("VERSION", "2") => Capability::VERSION_2,
                ("VERSION", version) => Capability::VERSION(version.into()),
                ("AUTHINFO", auth) => Capability::AUTHINFO(auth.into()),
                (cmd, arg) => Capability::OTHER(vec![cmd.into(), arg.into()]),
            }
        } else if parts[0] == "LIST" {
            let rest = &parts[1..];
            Capability::LIST(rest.iter().map(|&x| x.to_owned()).collect())
        } else if parts[0] == "XFEATURE-COMPRESS" {
            let rest = &parts[1..];
            Capability::XFEATURE_COMPRESS(rest.iter().map(|&x| x.into()).collect())
        } else {
            Capability::OTHER(parts.iter().map(|&x| x.to_owned()).collect())
        }

        //        match incoming {
        //            "VERSION" => Capabilities::VERSION 2
        //                "IMPLEMENTATION" => Capabilities::IMPLEMENTATION INN 2.5.4
        //                "AUTHINFO" => Capabilities::AUTHINFO SASL
        //            "HDR" => Capabilities::HDR
        //            "LIST" => Capabilities::LIST
        //            "ACTIVE" => Capabilities::ACTIVE
        //            "COUNTS" => Capabilities::COUNTS
        //            "DISTRIBUTIONS" => Capabilities::DISTRIBUTIONS
        //            "HEADERS" => Capabilities::HEADERS
        //            "MODERATORS" => Capabilities::MODERATORS
        //            "MOTD" => Capabilities::MOTD
        //            "NEWSGROUPS" => Capabilities::NEWSGROUPS
        //            "SUBSCRIPTIONS" => Capabilities::SUBSCRIPTIONS
        //            "NEWNEWS" => Capabilities::NEWNEWS
        //            "OVER" => Capabilities::OVER
        //            "POST" => Capabilities::POST
        //            "READER" => Capabilities::READER
        //            "SASL" => Capabilities::SASL DIGEST-MD5 NTLM CRAM-MD5
        //            "STARTTLS" => Capabilities::STARTTLS
        //            _ => Capability::Other
        //        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Compression {
    GZIP,
    TERMINATOR,
    OTHER(String),
}

impl From<&str> for Compression {
    fn from(incoming: &str) -> Self {
        match incoming {
            "GZIP" => Compression::GZIP,
            "TERMINATOR" => Compression::TERMINATOR,
            other => Compression::OTHER(other.to_owned()),
        }
    }
}
