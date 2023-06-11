use neovim_lib::{Neovim, NeovimApi};
use open::that;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

fn handle_connection(
    mut stream: TcpStream,
    running: &mut bool,
    nvim: &mut Neovim,
) -> Option<String> {
    let buf_reader = BufReader::new(&mut stream);

    let parser = "function parseParms(str) {    var pieces = str.split(\"&\"), data = {}, i, parts;    // process each query pair\nfor (i = 0; i < pieces.length; i++) {        parts = pieces[i].split(\"=\");        if (parts.length < 2) {            parts.push(\"\");        }        data[decodeURIComponent(parts[0])] = decodeURIComponent(parts[1]);    }    return data;}";
    let fetch = "fetch(`/access_token?access_token=${access_token}`).then((d) => { window.d = d; setTimeout(() => window.close(), 2000); }).catch(e => { console.error(e); window.e = e; });";
    let script = format!("<script>{parser} const access_token = parseParms(window.location.hash.replace(\"#\", \"\"))?.access_token; console.log('token', access_token); {fetch} </script><h1>Success! Window will close automatically.</h1>");
    let length = script.len();

    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| match result {
            Ok(value) => {
                return value;
            }
            _ => {
                return "".to_string();
            }
        })
        .take_while(|line| !line.is_empty())
        .collect();

    // TODO(Buser): Handle if the http_request Vec is empty
    let url_parts: Vec<&str> = http_request[0].split(' ').collect();
    // nvim.command(&format!("echo \"url_parts: {}\"", url_parts[1]))
    //     .unwrap();

    // TODO(Buser): Handle if the url_parts Vec is empty
    if url_parts[1] == "/" {
        let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n{script}");

        match stream.write_all(response.as_bytes()) {
            Ok(_) => {
                // nvim.command(&format!("echo \"Response sent\"")).unwrap();
                return None;
            }
            Err(e) => {
                *running = false;
                nvim.command(&format!("Error writing data to oauth tab: {}", e))
                    .unwrap();
                return None;
            }
        }
    }

    let possible_parsed_url =
        reqwest::Url::parse(format!("http://localhost{}", url_parts[1]).as_str());

    if let Err(parsed_url_error) = possible_parsed_url {
        nvim.command(&format!("echo \"Error parsing url: {}\"", parsed_url_error))
            .unwrap();
        return None;
    }

    let parsed_url = possible_parsed_url.unwrap();

    // Here just in case we want to debug something
    // let path = parsed_url.path();
    let query: std::collections::HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();

    let mut access_token = String::new();
    match query.get("access_token") {
        Some(value) => {
            let string_value = String::from_str(value.as_str());

            if let Err(string_error) = string_value {
                nvim.command(&format!(
                    "echo \"Error converting code value to String: {}\"",
                    string_error
                ))
                .unwrap();
                return None;
            }
            access_token = string_value.unwrap();
            // TODO(Buser): Do something with the token value
        }
        None => {
            // return None;
        }
    }
    let script = "<h1>Success! Window will close automatically.</h1>";
    let length = script.len();
    let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n{script}");

    match stream.write_all(response.as_bytes()) {
        Ok(_) => {
            if access_token.len() > 0 {
                return Option::from(access_token);
            }
            return None;
        }
        Err(e) => {
            *running = false;
            nvim.command(&format!("Error writing data to oauth tab: {}", e))
                .unwrap();
            return None;
        }
    }
}

pub fn local_connect(
    client_id: String,
    oauth_port: String,
    nvim: &mut Neovim,
) -> Result<String, String> {
    let mut running = true;
    let possible_listener = TcpListener::bind(format!("127.0.0.1:{}", 6969));

    if let Err(listener_error) = possible_listener {
        return Err(listener_error.to_string());
    }

    let listener = possible_listener.unwrap();

    let _ = that(format!("https://id.twitch.tv/oauth2/authorize?response_type=token&client_id={}&redirect_uri=http://localhost:{}&scope=chat%3Aread+chat%3Aedit", client_id, oauth_port));

    for possible_stream in listener.incoming() {
        match possible_stream {
            Ok(stream) => {
                let possible_access_token = handle_connection(stream, &mut running, nvim);

                match possible_access_token {
                    Some(access_token) => {
                        return Ok(access_token);
                    }
                    None => {}
                }
            }
            Err(e) => {
                return Err(format!("Stream encountered an unforseen error: {}", e).to_string());
            }
        }

        if !running {
            break;
        };
    }
    return Err("Error broken stream".to_string());
}
