use std::{env, io::{self, Write}, net::{SocketAddr, TcpStream}, process};

fn main() -> Result<(), io::Error> {
    let mut args = env::args().skip(1);
    
    let remote_addr = {
        let remote_addr = args.next().unwrap_or_else(|| {
            eprintln!("error: remote node address is missing");
            process::exit(1);
        });

        remote_addr.parse::<SocketAddr>().unwrap_or_else(|_| {
            eprintln!("error: invalid remote node address");
            process::exit(1);
        })
    };

    let data = args.next().unwrap_or_else(|| {
        eprintln!("error: no data provided for dissemination");
        process::exit(1);
    });

    let request_msg = format!("UPDATE_DATA=[{}];", data);
    
    let mut request_stream = TcpStream::connect(remote_addr)?;

    request_stream.write(request_msg.as_bytes())?;

    Ok(())
}
