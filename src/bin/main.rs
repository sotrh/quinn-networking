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

        let mut buffer = (0..16).collect::<Vec<_>>();
        loop {
            match recv.read(&mut buffer).await {
                Ok(Some(received)) => {
                    println!("[server] received {} bytes", received);
                    let message = String::from_utf8_lossy(&buffer[..received]);
                    match &message[..] {
                        "ping" => {
                            println!("[server] received \"ping\" sending \"pong\"");
                            send.write_all(b"pong").await.unwrap();
                        }
                        "marco" => {
                            println!("[server] received \"marco\" sending \"polo\"");
                            send.write_all(b"polo").await.unwrap();
                        }
                        it => println!("[server] Unknown message {:?}", it),
                    }
                }
                Ok(None) => {
                    // println!("[server] no data received");
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
    // send.poll_finish(cx)
    println!("[client] sending \"marco\"");
    send.write_all(b"marco").await?;
    send.finish().await?;

    let mut buffer = (0..16).collect::<Vec<_>>();
    if let Ok(recv_op) = recv.read(&mut buffer).await {
        if let Some(received) = recv_op {
            println!("[client] received {} bytes", received);
            println!("[client] received message {}", std::str::from_utf8(&buffer)?);
        } else {
            println!("[client] unable to receive message");
        }
    } else {
        eprintln!("[client] unable to receive message from server")
    }
    if let Ok(recv_op) = recv.read(&mut buffer).await {
        if let Some(received) = recv_op {
            println!("[client] received {} bytes", received);
            println!("[client] received message {}", std::str::from_utf8(&buffer)?);
        } else {
            println!("[client] unable to receive message");
        }
    }
    else {
        eprintln!("[client] unable to receive message from server")
    }

    endpoint.wait_idle().await;

    Ok(())
}
