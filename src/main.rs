use clap::{Arg, Command};
use colored::*;
use futures::executor::block_on;
use libquartz::{encryption, keytools, msgservices};
use std::{env, fs, io::Read, path};


struct ServerData {
    names: Vec<String>,
    urls: Vec<String>,
}

#[tokio::main]
async fn main() {
    let _matches = Command::new("Quartz CLI Messenger")
        .subcommand_required(true)
        .version("0.1")
        .author("Demir Yerli <mrquantumoff@protonmail.com>")
        .arg(
            Arg::new("debug")
                .long("debug")
                .help("Enable debug output")
                .takes_value(false),
        )
        .subcommand(
            Command::new("get")
                .subcommand_required(false)
                .about("Get Messages")
                .arg(
                    Arg::new("index")
                        .short('i')
                        .long("index")
                        .help("Server index")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("as")
                        .short('a')
                        .long("as")
                        .help("As contact name")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("send")
                .subcommand_required(false)
                .about("Send Message")
                .arg(
                    Arg::new("index")
                        .short('i')
                        .long("index")
                        .help("Server index")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::new("message")
                        .short('m')
                        .long("message")
                        .takes_value(true)
                        .help("Message content")
                        .required(true),
                )
                .arg(
                    Arg::new("to")
                        .short('t')
                        .long("to")
                        .takes_value(true)
                        .help("Recipient")
                        .required(true),
                )
                .arg(
                    Arg::new("from")
                        .short('f')
                        .long("from")
                        .help("Sender")
                        .takes_value(true)
                        .required(true),
                ),
        )
        .about("Libquartz based messenger")
        .get_matches();

    let _key = keytools::get_default_key();

    let _debug = _matches.is_present("debug");
    if let Some(subc) = _matches.subcommand_matches("get") {
        let _index = subc.value_of("index").unwrap();
        let _name = subc.value_of("as").unwrap();
        get_msgs(_name, _index, &_key);
    }
    if let Some(subc) = _matches.subcommand_matches("send") {
        let _index = subc.value_of("index").unwrap().parse::<usize>();
        let _message = subc.value_of("message").unwrap();
        let _to = subc.value_of("to").unwrap();
        let _from = subc.value_of("from").unwrap();
        
        match _index {
            Ok(index) => send_msg(_from, _to, index, &_key, _message),
            Err(_) => {
                println!("{}", "Invalid argument \"index\"".bright_red());
                std::process::exit(1);
            }
        }
    }
}


fn send_msg(_from: &str, _to: &str, _server: usize, _key: &str, _message: &str) {
    let servers = get_servers();
    if servers.urls.is_empty() || _server >= servers.urls.len() {
        println!("{}","Server out of range".bright_red());
        std::process::exit(1);
    }
    println!("{}{}{}{}{}{}{}{}", "Sending message \"".bright_blue(), _message, "\" via server ".bright_blue(), servers.names[_server], " to ".bright_blue(), _to, " as ".bright_blue(), _from);

    let resp = block_on(msgservices::send_msg(
        servers.urls[_server].to_string()+"/incoming",
        _message.to_string(),
        _key.to_string(),
        _to.to_string(),
        _from.to_string(),
    ));
    match resp {
        true => {
            println!("{}", "Message sent".bright_green());
        }
        false => {
            println!("{}", "Message sending failed".bright_red());
            std::process::exit(1);
        }
    }
}


fn get_msgs(_name: &str, _index: &str, _key: &str) {
    let servers = get_servers();

    let index = _index.parse::<usize>();

    match index {
        Ok(i) => {
            if servers.urls.len() - 1 < i || servers.urls.is_empty() {
                println!("{}", servers.urls.len());
                println!("{}", "Server index out of range".bright_red());
                std::process::exit(1);
            } else {
                let name = servers.names[i].clone();
                println!("{}{}","Server name: ".bright_blue(), name.bright_blue());
                let url = &servers.urls[i];
                let url = url.to_string() + "/messages";
                let contact = encryption::encrypt_string(_key.to_string(), _name.to_string());
                let resp = block_on(msgservices::get_msgs_encrypted(url, contact));
                let msgs = msgservices::decrypt_msgs(resp, _key.to_string());
                for msg in &msgs.messages {
                    let index = &msgs.messages.iter().position(|x| x == msg).unwrap();
                    let index = index.to_string().parse::<usize>().unwrap();
                    println!(
                        "{}{}{}{}{}",
                        "\n--------------------------\n".bright_blue(),
                        msg.trim(),
                        "\n--------------------------\nfrom ".bright_blue(),
                        msgs.senders[index].trim(),
                        "\n \n"
                    );
                }
            }
        }
        Err(_) => {
            println!("{}", "Error: index is not a number".bright_red());
            std::process::exit(1);
        }
    }
}

fn get_servers() -> ServerData {
    create_config();
    #[allow(deprecated)]
    let home = env::home_dir().unwrap();
    let srvpath = path::Path::new(&home)
        .join(".config")
        .join("libquartz")
        .join("servers");
    if fs::metadata(&srvpath).is_err() {
        fs::create_dir_all(&srvpath).unwrap();
    }
    let mut servernames: Vec<String> = Vec::new();
    let mut serverurls: Vec<String> = Vec::new();
    for entry in fs::read_dir(&srvpath).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let filename = path.file_name().unwrap().to_str().unwrap();
            servernames.push(filename.to_string());
            let mut file = fs::File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            serverurls.push(contents.to_string());
        }
    }
    ServerData {
        names: servernames,
        urls: serverurls,
    }
}

fn create_config() {
    #[allow(deprecated)]
    let home = env::home_dir().unwrap();
    // Join paths
    let cfgpath = path::Path::new(&home).join(".config");
    // Check if the path exists
    if !cfgpath.exists() {
        // Create the path
        fs::create_dir(&cfgpath).expect("Could not create config directory");
    }
    // Join paths
    let libquartzpath = cfgpath.join("libquartz");
    // Check if the path exists
    if !libquartzpath.exists() {
        // Create the path
        fs::create_dir(&libquartzpath).expect("Could not create libquartz directory");
    }
    // Join paths
    let keyspath = libquartzpath.join("keys");
    // Check if the path exists
    if !keyspath.exists() {
        // Create the path
        fs::create_dir(&keyspath).expect("Could not create keys directory");
    }
}
