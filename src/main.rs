mod proto;

use std::error::Error;

use byteorder::{BigEndian, ByteOrder};
use bytes::BufMut;
use bytes::{Bytes, BytesMut};
use h2::server;
use http::{HeaderMap, HeaderValue};
use prost::Message;
use tokio::net::{TcpListener, TcpStream};
// use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let _ = env_logger::try_init();

    let p = proto::helloworld::HelloReply {
        message: "haha".to_owned(),
    };

    let mut buf: Vec<u8> = [].to_vec();
    p.encode(&mut buf).unwrap();
    let pp = proto::helloworld::HelloReply::decode(&buf[..]).unwrap();
    println!("{:?}", buf);
    println!("{:?}", pp);

    // let pp = Person::decode(&buf[..]).unwrap();

    let listener = TcpListener::bind("0.0.0.0:5928").await?;

    println!("listening on {:?}", listener.local_addr());

    loop {
        if let Ok((socket, _peer_addr)) = listener.accept().await {
            tokio::spawn(async move {
                if let Err(e) = handle(socket).await {
                    println!("  -> err={:?}", e);
                }
            });
        } else {
            println!("=-=-=-=-=-=-==-=-=-=-=-=-=-=-=-");
        }
    }
}

async fn handle(socket: TcpStream) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut connection = server::handshake(socket).await?;
    // let mut builder = h2::server::Builder::default();
    // builder
    //     .initial_window_size(config.initial_stream_window_size)
    //     .initial_connection_window_size(config.initial_conn_window_size)
    //     .max_frame_size(config.max_frame_size);
    // let builder = server::Builder::new()
    // builder
    //     .initial_connection_window_size(1_000_000)
    //     .initial_window_size(1_000_000)
    //     .max_frame_size(1_000_000);
    // builder.adaptive_window = false;
    // let mut connection: server::Connection<TcpStream, Bytes> = builder.handshake(socket).await?;

    // connection.set_initial_window_size(1_000_000).unwrap();
    println!("H2 connection bound");

    while let Some(result) = connection.accept().await {
        let (request, mut respond) = result?;
        // let (mut request, mut respond) = result?;
        // println!("GOT request: {:?}", request);
        // let headers = request.headers();
        // println!("{:?}", headers);

        // let body = request.body_mut();

        let (headers, mut body) = request.into_parts();
        println!("{:?}", headers);
        println!("processing body");
        let mut body_buf = vec![];
        while let Some(chunk) = body.data().await {
            let buf: Bytes = chunk?;

            let _ = body.flow_control().release_capacity(buf.len());
            println!("---{}", body.flow_control().available_capacity());
            body_buf.put(buf);
        }
        println!("parseing body done");
        let compressed_flag = &body_buf[0..1];
        let proto_len = &body_buf[1..5];
        let proto_buf = &body_buf[5..];
        println!(
            "{:?}, {:?}, {:?}",
            compressed_flag,
            proto_len,
            proto_buf.len()
        );

        let flags = BigEndian::read_uint(compressed_flag, 1);
        let len = BigEndian::read_uint(proto_len, 4);
        // let len = u8::from_be_bytes(lenbuf);
        let body_len = proto_buf.len() as u64;
        //     if len != body_len {
        //         println!("len != body_len")
        //     }
        println!(
            "flags {}, len {:?}, proto_body_len: {}",
            flags, len, body_len
        );
        //     // let bodybuf = bodybuf.slice(5..);

        //     // LittleEndian::read_uint(buflen[..], 5);
        //     // usize::from_ne_bytes(buflen[..]);
        //     println!("{:?}", bodybuf.to_vec());
        //     println!("{:?}", len);
        let _pp = proto::helloworld::HelloRequest::decode(proto_buf).unwrap();
        // println!("{:?}", pp);

        let hello_reply = proto::helloworld::HelloReply {
            message: "haha".to_owned(),
        };
        let mut reply_buf: Vec<u8> = [].to_vec();
        hello_reply.encode(&mut reply_buf).unwrap();
        let reply_len = reply_buf.len() as u32;

        let mut len_buf = [0; 4];
        BigEndian::write_u32(&mut len_buf, reply_len);
        println!("{:?}", reply_len);

        let mut reply_body_buf = vec![0];
        reply_body_buf.put(&len_buf[..]);
        reply_body_buf.put(&reply_buf[..]);
        println!("{:?}", reply_body_buf);

        let mut response = http::Response::new(());
        response
            .headers_mut()
            .insert("key", HeaderValue::from_str("src").unwrap());

        let mut send = respond.send_response(response, false)?;

        println!(">>>> sending data");
        send.send_data(reply_body_buf.into(), false)?;
        // send.send_data(Bytes::from_static(b"hello world"), false)?;

        let mut trailers = HeaderMap::new();
        // 'grpc-status': '0',
        //   'grpc-message': 'OK',
        trailers.insert("grpc-status", "0".parse().unwrap());
        trailers.insert("grpc-message", "OK".parse().unwrap());
        send.send_trailers(trailers).unwrap();
    }

    println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~ H2 connection CLOSE !!!!!! ~~~~~~~~~~~");

    Ok(())
}
