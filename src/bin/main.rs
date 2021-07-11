use futures::StreamExt;
use quinn::ReadError;
use quinn_networking::{client, server};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let (mut incoming, server_cert) = server(server_addr)?;

    // Server
    tokio::spawn(async move {
        let incoming_conn = incoming.next().await.unwrap();
        let mut new_conn = incoming_conn.await.unwrap();
        println!(
            "[server] connection accepted: addr={}",
            new_conn.connection.remote_address(),
        );
        
        let (mut send, mut recv) = new_conn.bi_streams.next().await.unwrap().unwrap();

        send.write_all(b"test").await.unwrap();

        let mut buffer = vec![];
        loop {
            match recv.read(&mut buffer).await {
                Ok(Some(received)) => {
                    println!("[server] received {} bytes", received);
                    match &buffer[..] {
                        b"ping" => {
                            println!("[server] received \"ping\" sending \"pong\"");
                            send.write_all(b"pong").await.unwrap();
                        }
                        b"marco" => {
                            println!("[server] received \"marco\" sending \"polo\"");
                            send.write_all(b"polo").await.unwrap();
                        }
                        it => println!("[server] Unknown message {:?}", it),
                    }
                }
                Ok(None) => {
                    println!("[server] no data received");
                }
                Err(ReadError::ConnectionClosed(_)) => { 
                    println!("connection closed");
                    break;
                }
                Err(e) => {
                    println!("an error occurred: {:?}", e);
                    break;
                }
            }
        }


        println!("[server] closing");
        send.finish().await.unwrap();
    });

    // Client
    let endpoint = client("0.0.0.0:0".parse().unwrap(), &[&server_cert])?;
    let quinn::NewConnection {
        connection,
        ..
    } = endpoint
        .connect(&server_addr, "localhost")
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());

    let (mut send, mut recv) = connection.open_bi().await.unwrap();

    println!("[client] sending \"ping\"");
    send.write_all(b"ping").await?;
    println!("[client] sending \"marco\"");
    send.write_all(b"marco").await?;
    send.finish().await?;

    let mut buffer = vec![];
    if let Ok(Some(received)) = recv.read(&mut buffer).await {
        println!("[client] received {} bytes", received);
        println!("[client] received message {}", std::str::from_utf8(&buffer)?);
    } else {
        eprintln!("[client] unable to receive message from server")
    }
    if let Ok(Some(received)) = recv.read(&mut buffer).await {
        println!("[client] received {} bytes", received);
        println!("[client] received message {}", std::str::from_utf8(&buffer)?);
    }
    else {
        eprintln!("[client] unable to receive message from server")
    }

    endpoint.wait_idle().await;

    Ok(())
}
