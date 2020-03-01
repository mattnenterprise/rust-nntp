pub struct Headers<'a>(pub std::collections::HashMap<&'a [u8], Option<&'a [u8]>>);

impl<'a> From<&'a str> for Headers<'a> {
    fn from(headers: &'a str) -> Self {
        let map = parse_multiline_header(headers.as_bytes())
            .iter()
            .map(|x| {
                let mut parts = x.splitn(2, |x| *x == b':');
                let key = parts.next().unwrap();
                let value = parts.next().map(|x| {
                    let mut start = 0;
                    let mut end = x.len();

                    if !x.is_empty() && x[0] == b' ' {
                        start = 1;
                    }

                    if x.len() >= 2 && x[x.len() - 2..x.len()] == b"\r\n"[..] {
                        end = x.len() - 2;
                    }

                    &x[start..end]
                });
                (key, value)
            })
            .collect();
        Headers(map)
    }
}

impl<'a> std::fmt::Debug for Headers<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Headers")
            .field(
                "hashmap",
                &self
                    .0
                    .iter()
                    .map(|(k, v)| {
                        (
                            std::str::from_utf8(k).unwrap(),
                            v.map(|v| std::str::from_utf8(v).unwrap()),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

fn parse_multiline_header(input: &[u8]) -> Vec<&[u8]> {
    let mut input = input;
    let mut parts = vec![];
    let start = 0;
    let mut end = 0;
    loop {
        let mut iter = input.windows(2);

        loop {
            let pos = iter.position(|x| x == &b"\r\n"[..]);

            if pos.is_none() {
                //                println!("end of input");
                end = input.len();
                break;
            }

            end += *pos.as_ref().unwrap() + 2;
            if end >= input.len() {
                break;
            } else if input[end] == b' ' || input[end] == b'\t' {
                //                println!("hit a CRLF+space... {:?}", end);
                end -= 1;
                continue;
            } else {
                //                println!("not a space... {:?}", end);
                break;
            }
        }

        parts.push(&input[start..end]);
        //        println!(
        //            "pushed: ```{:?}```",
        //            std::str::from_utf8(&input[start..end]).unwrap()
        //        );

        if end >= input.len() {
            break;
        }
        input = &input[end..];
        end = 0;
    }

    parts
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_without_nom() {
        let multiline = &b"Hi there.\r\n Mr. Bear\r\n is my friendo.\r\nOther line :)"[..];

        let parsed = super::parse_multiline_header(multiline);
        assert_eq!(
            vec![
                &b"Hi there.\r\n Mr. Bear\r\n is my friendo.\r\n"[..],
                &b"Other line :)"[..]
            ],
            parsed
        );
    }

    #[test]
    fn test_big_parse() {
        let multiline = &b"Path: aioe.org!feeder3.feed.usenet.farm!feed.usenet.farm!newsfeed.xs4all.nl!newsfeed9.news.xs4all.nl!nzpost1.xs4all.net!not-for-mail\r\nSubject: Re: Solved: Laser-Faxmachine / All-in-one?\r\nNewsgroups: alt.os.linux.ubuntu,comp.sys.raspberry-pi,alt.os.linux.mageia\r\nReferences: <qioo0v$lie$1@dont-email.me>\r\nFrom: \"Dirk T. Verbeek\" <dverbeek@xs4all.nl>\r\nDate: Sun, 11 Aug 2019 13:36:41 +0200\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:52.0) Gecko/20100101\r\n Thunderbird/52.9.1\r\nMIME-Version: 1.0\r\nIn-Reply-To: <qioo0v$lie$1@dont-email.me>\r\nContent-Type: text/plain; charset=utf-8; format=flowed\r\nContent-Language: en-GB\r\nContent-Transfer-Encoding: 7bit\r\nLines: 62\r\nMessage-ID: <5d4ffdc9$0$10265$e4fe514c@news.xs4all.nl>\r\nNNTP-Posting-Host: 4963421c.news.xs4all.nl\r\nX-Trace: G=i0ayF1tM,C=U2FsdGVkX1+DsoyoY05ZJsZYYYtl7tjl1Ktvu66L0dicaXocNxRFHestw3/Rq/3cOBeQR9fRj41nlBD/NXqHmvTShGXpghsdPmzj7sWzq0Q=\r\nX-Complaints-To: abuse@xs4all.nl\r\nXref: aioe.org alt.os.linux.ubuntu:203955 comp.sys.raspberry-pi:20994 alt.os.linux.mageia:21467"[..];

        let parsed = super::parse_multiline_header(multiline);
        let expected =             vec![
            &b"Path: aioe.org!feeder3.feed.usenet.farm!feed.usenet.farm!newsfeed.xs4all.nl!newsfeed9.news.xs4all.nl!nzpost1.xs4all.net!not-for-mail\r\n"[..],
            &b"Subject: Re: Solved: Laser-Faxmachine / All-in-one?\r\n"[..],
            &b"Newsgroups: alt.os.linux.ubuntu,comp.sys.raspberry-pi,alt.os.linux.mageia\r\n"[..],
            &b"References: <qioo0v$lie$1@dont-email.me>\r\n"[..],
            &b"From: \"Dirk T. Verbeek\" <dverbeek@xs4all.nl>\r\n"[..],
            &b"Date: Sun, 11 Aug 2019 13:36:41 +0200\r\n"[..],
            &b"User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:52.0) Gecko/20100101\r\n Thunderbird/52.9.1\r\n"[..],
            &b"MIME-Version: 1.0\r\n"[..],
            &b"In-Reply-To: <qioo0v$lie$1@dont-email.me>\r\n"[..],
            &b"Content-Type: text/plain; charset=utf-8; format=flowed\r\n"[..],
            &b"Content-Language: en-GB\r\n"[..],
            &b"Content-Transfer-Encoding: 7bit\r\n"[..],
            &b"Lines: 62\r\n"[..],
            &b"Message-ID: <5d4ffdc9$0$10265$e4fe514c@news.xs4all.nl>\r\n"[..],
            &b"NNTP-Posting-Host: 4963421c.news.xs4all.nl\r\n"[..],
            &b"X-Trace: G=i0ayF1tM,C=U2FsdGVkX1+DsoyoY05ZJsZYYYtl7tjl1Ktvu66L0dicaXocNxRFHestw3/Rq/3cOBeQR9fRj41nlBD/NXqHmvTShGXpghsdPmzj7sWzq0Q=\r\n"[..],
            &b"X-Complaints-To: abuse@xs4all.nl\r\n"[..],
            &b"Xref: aioe.org alt.os.linux.ubuntu:203955 comp.sys.raspberry-pi:20994 alt.os.linux.mageia:21467"[..]
        ];
        if expected != parsed {
            println!(
                "{:#?}",
                //                expected
                //                    .iter()
                //                    .map(|x| std::str::from_utf8(x).unwrap())
                //                    .collect::<Vec<_>>(),
                parsed
                    .iter()
                    .map(|x| std::str::from_utf8(x).unwrap())
                    .collect::<Vec<_>>()
            )
        };
        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_another_big_header() {
        let multiline = &b"Path: aioe.org!eternal-september.org!feeder.eternal-september.org!reader01.eternal-september.org!.POSTED!not-for-mail\r\nFrom: The Natural Philosopher <tnp@invalid.invalid>\r\nNewsgroups: comp.sys.raspberry-pi\r\nSubject: Re: Headphone vs HDMI audio\r\nDate: Fri, 23 Aug 2019 08:21:56 +0100\r\nOrganization: A little, after lunch\r\nLines: 21\r\nMessage-ID: <qjo46k$d94$2@dont-email.me>\r\nReferences: <qjmksb$1fl$1@news.albasani.net>\r\n <gs84iqFklmrU1@mid.individual.net> <qjn8r3$ir9$1@gioia.aioe.org>\r\n <gs8npbFom1aU1@mid.individual.net>\r\nMime-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8; format=flowed\r\nContent-Transfer-Encoding: 8bit\r\nInjection-Date: Fri, 23 Aug 2019 07:21:56 -0000 (UTC)\r\nInjection-Info: reader02.eternal-september.org; posting-host=\"63ea02abf82dba898bb401b78fd953d3\";\r\n\tlogging-data=\"13604\"; mail-complaints-to=\"abuse@eternal-september.org\";\tposting-account=\"U2FsdGVkX19l6YYNyL8WhWYGEz3ytkPKhO9QzyfLpTw=\"\r\nUser-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101\r\n Thunderbird/60.6.1\r\nCancel-Lock: sha1:XFhkMKtZ9Uu1TqajuskdJtWauIg=\r\nIn-Reply-To: <gs8npbFom1aU1@mid.individual.net>\r\nContent-Language: en-GB\r\nXref: aioe.org comp.sys.raspberry-pi:21083"[..];
        let parsed = super::parse_multiline_header(multiline);

        let expected = vec![&b"Path: aioe.org!eternal-september.org!feeder.eternal-september.org!reader01.eternal-september.org!.POSTED!not-for-mail\r\n"[..],
                            &b"From: The Natural Philosopher <tnp@invalid.invalid>\r\n"[..], 
                            &b"Newsgroups: comp.sys.raspberry-pi\r\n"[..],
                            &b"Subject: Re: Headphone vs HDMI audio\r\n"[..], 
                            &b"Date: Fri, 23 Aug 2019 08:21:56 +0100\r\n"[..],
                            &b"Organization: A little, after lunch\r\n"[..],
                            &b"Lines: 21\r\n"[..],
                            &b"Message-ID: <qjo46k$d94$2@dont-email.me>\r\n"[..],
                            &b"References: <qjmksb$1fl$1@news.albasani.net>\r\n <gs84iqFklmrU1@mid.individual.net> <qjn8r3$ir9$1@gioia.aioe.org>\r\n <gs8npbFom1aU1@mid.individual.net>\r\n"[..],
                            &b"Mime-Version: 1.0\r\n"[..],
                            &b"Content-Type: text/plain; charset=utf-8; format=flowed\r\n"[..],
                            &b"Content-Transfer-Encoding: 8bit\r\n"[..],
                            &b"Injection-Date: Fri, 23 Aug 2019 07:21:56 -0000 (UTC)\r\n"[..],
                            &b"Injection-Info: reader02.eternal-september.org; posting-host=\"63ea02abf82dba898bb401b78fd953d3\";\r\n\tlogging-data=\"13604\"; mail-complaints-to=\"abuse@eternal-september.org\";\tposting-account=\"U2FsdGVkX19l6YYNyL8WhWYGEz3ytkPKhO9QzyfLpTw=\"\r\n"[..],
                            &b"User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101\r\n Thunderbird/60.6.1\r\n"[..],
                            &b"Cancel-Lock: sha1:XFhkMKtZ9Uu1TqajuskdJtWauIg=\r\n"[..],
                            &b"In-Reply-To: <gs8npbFom1aU1@mid.individual.net>\r\n"[..],
                            &b"Content-Language: en-GB\r\n"[..],
                            &b"Xref: aioe.org comp.sys.raspberry-pi:21083"[..]];

        assert_eq!(expected, parsed);
    }
}
