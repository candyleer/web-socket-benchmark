mod bench;

use std::time::{Duration, Instant};

const HELP: &str = "Please run with `--release` flag for accurate results.
Example:
    cargo run --release
    cargo run -r -- -C codegen-units=1 -C opt-level=3
";

fn capacity(msg_len: usize, iter: usize) -> usize {
    (msg_len + 14) * iter
}

fn make_payload(size: usize) -> String {
    "x".repeat(size)
}

struct BenchResult {
    name: &'static str,
    send: Duration,
    echo: Duration,
    recv: Duration,
    total: Duration,
}

fn main() {
    if cfg!(debug_assertions) {
        println!("{HELP}");
    }

    let small = "Hello, World!\n"; // 14 bytes
    let p100 = make_payload(100);
    let p200 = make_payload(200);
    let p500 = make_payload(500);
    let p800 = make_payload(800);
    let large = make_payload(1024); // 1 KB
    let p2k = make_payload(2048); // 2 KB
    let xlarge = make_payload(10240); // 10 KB

    let payloads: &[(&str, &str, usize)] = &[
        ("14B", small, 100_000),
        ("100B", &p100, 100_000),
        ("200B", &p200, 100_000),
        ("500B", &p500, 100_000),
        ("800B", &p800, 100_000),
        ("1KB", &large, 100_000),
        ("2KB", &p2k, 100_000),
        ("10KB", &xlarge, 50_000),
    ];

    // Warm-up: run each benchmark once with small payload to
    // stabilize CPU frequency, fill caches, and prime allocator
    println!("Warming up...");
    bench::block_on(async {
        let wm = small;
        let wi = 10_000;
        let wc = capacity(wm.len(), wi);
        let _ = fastwebsockets_benchmark::run(wm, wi, wc).await;
        let _ = soketto_benchmark::run(wm, wi, wc).await;
        let _ = tokio_tungstenite_benchmark::run(wm, wi, wc).await;
        let _ = web_socket_benchmark::run(wm, wi, wc).await;
        let _ = sockudo_ws_benchmark::run(wm, wi, wc).await;
        let _ = tokio_websockets_benchmark::run(wm, wi, wc).await;
        let _ = wtx_benchmark::run(wm, wi, wc).await;
    });
    println!("Warm-up done.\n");

    for &(label, msg, iter) in payloads {
        let cap = capacity(msg.len(), iter);
        println!(
            "\n\
             ╔══════════════════════════════════════════════════╗\n\
             ║  Payload: {label:<6} ({:>5} bytes) x {iter} msgs  ║\n\
             ╚══════════════════════════════════════════════════╝",
            msg.len()
        );

        let mut results = Vec::new();

        bench::block_on(async {
            results.push(
                fastwebsockets_benchmark::run(msg, iter, cap)
                    .await
                    .unwrap(),
            );
            results.push(
                soketto_benchmark::run(msg, iter, cap)
                    .await
                    .unwrap(),
            );
            results.push(
                tokio_tungstenite_benchmark::run(msg, iter, cap)
                    .await
                    .unwrap(),
            );
            results.push(
                web_socket_benchmark::run(msg, iter, cap)
                    .await
                    .unwrap(),
            );
            results.push(
                sockudo_ws_benchmark::run(msg, iter, cap)
                    .await
                    .unwrap(),
            );
            results.push(
                tokio_websockets_benchmark::run(msg, iter, cap)
                    .await
                    .unwrap(),
            );
            results.push(
                wtx_benchmark::run(msg, iter, cap).await.unwrap(),
            );
        });

        // Sort by total time
        results.sort_by_key(|r| r.total);

        println!("\n{:<20} {:>10} {:>10} {:>10} {:>10}",
            "Library", "Send", "Echo", "Recv", "Total");
        println!("{}", "-".repeat(62));
        for r in &results {
            println!(
                "{:<20} {:>10.2?} {:>10.2?} {:>10.2?} {:>10.2?}",
                r.name, r.send, r.echo, r.recv, r.total
            );
        }
    }
}

// ============================================================
// fastwebsockets
// ============================================================
mod fastwebsockets_benchmark {
    use super::*;
    use fastwebsockets::*;

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Res<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run(
        msg: &str,
        iter: usize,
        cap: usize,
    ) -> Res<BenchResult> {
        let mut stream = bench::Stream::new(cap);
        let total = Instant::now();

        // Send
        stream.role_client();
        let mut ws =
            WebSocket::after_handshake(&mut stream, Role::Client);
        ws.set_auto_pong(true);
        ws.set_writev(true);
        ws.set_auto_close(true);

        let t = Instant::now();
        for _ in 0..iter {
            ws.write_frame(Frame::new(
                true,
                OpCode::Text,
                None,
                msg.as_bytes().into(),
            ))
            .await?;
        }
        ws.write_frame(Frame::new(
            true,
            OpCode::Close,
            None,
            (&[] as &[u8]).into(),
        ))
        .await?;
        let send = t.elapsed();

        // Echo
        stream.role_server();
        let mut ws =
            WebSocket::after_handshake(&mut stream, Role::Server);
        ws.set_auto_apply_mask(true);
        ws.set_auto_pong(true);
        ws.set_writev(true);
        ws.set_auto_close(true);

        let t = Instant::now();
        let mut ws = FragmentCollector::new(ws);
        loop {
            let frame = ws.read_frame().await?;
            match frame.opcode {
                OpCode::Close => break,
                OpCode::Text | OpCode::Binary => {
                    ws.write_frame(frame).await?;
                }
                _ => {}
            }
        }
        let echo = t.elapsed();

        // Recv
        stream.role_client();
        let mut ws =
            WebSocket::after_handshake(&mut stream, Role::Client);
        let t = Instant::now();
        for _ in 0..iter {
            let frame = ws.read_frame().await?;
            assert!(frame.fin);
            assert_eq!(frame.opcode, OpCode::Text);
            assert_eq!(frame.payload, msg.as_bytes());
        }
        assert_eq!(
            ws.read_frame().await.unwrap().opcode,
            OpCode::Close
        );
        let recv = t.elapsed();

        let total = total.elapsed();
        Ok(BenchResult {
            name: "fastwebsockets",
            send,
            echo,
            recv,
            total,
        })
    }
}

// ============================================================
// soketto
// ============================================================
mod soketto_benchmark {
    use super::*;
    use soketto::{handshake::Client, Incoming};

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Res<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run(
        msg: &str,
        iter: usize,
        cap: usize,
    ) -> Res<BenchResult> {
        let mut stream = bench::Stream::new(cap);
        let total = Instant::now();

        // Send
        stream.role_client();
        let (mut ws, _) =
            Client::new(&mut stream, "", "").into_builder().finish();
        let t = Instant::now();
        for _ in 0..iter {
            ws.send_text(msg).await?;
        }
        ws.close().await?;
        let send = t.elapsed();
        drop(ws);

        // Echo
        stream.role_server();
        let (mut tx, mut rx) =
            Client::new(&mut stream, "", "").into_builder().finish();
        let t = Instant::now();
        loop {
            let mut buf = Vec::new();
            match rx.receive(&mut buf).await? {
                Incoming::Data(data) => match data {
                    soketto::Data::Text(len) => {
                        tx.send_text(
                            std::str::from_utf8(&buf[..len])
                                .unwrap(),
                        )
                        .await?
                    }
                    soketto::Data::Binary(len) => {
                        tx.send_binary(&buf[..len]).await?
                    }
                },
                Incoming::Closed(_) => break,
                _ => {}
            }
        }
        let echo = t.elapsed();
        drop(tx);
        drop(rx);

        // Recv
        stream.role_client();
        let (_, mut ws) =
            Client::new(&mut stream, "", "").into_builder().finish();
        let t = Instant::now();
        for _ in 0..iter {
            let mut buf = Vec::new();
            let ty = ws.receive(&mut buf).await.unwrap();
            assert!(matches!(
                ty,
                Incoming::Data(soketto::Data::Text(_))
            ));
            assert_eq!(std::str::from_utf8(&buf), Ok(msg));
        }
        assert!(matches!(
            ws.receive(&mut Vec::new()).await,
            Ok(Incoming::Closed(_))
        ));
        let recv = t.elapsed();

        let total = total.elapsed();
        Ok(BenchResult {
            name: "soketto",
            send,
            echo,
            recv,
            total,
        })
    }
}

// ============================================================
// tokio-tungstenite
// ============================================================
mod tokio_tungstenite_benchmark {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{
        tungstenite::{protocol::Role, Message},
        WebSocketStream,
    };

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Res<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run(
        msg: &str,
        iter: usize,
        cap: usize,
    ) -> Res<BenchResult> {
        let mut stream = bench::Stream::new(cap);
        let total = Instant::now();

        // Send
        stream.role_client();
        let mut ws = WebSocketStream::from_raw_socket(
            &mut stream,
            Role::Client,
            None,
        )
        .await;
        let t = Instant::now();
        for _ in 0..iter {
            ws.feed(Message::Text(msg.to_owned().into()))
                .await?;
        }
        ws.close(None).await?;
        let send = t.elapsed();

        // Echo
        stream.role_server();
        let mut ws = WebSocketStream::from_raw_socket(
            &mut stream,
            Role::Server,
            None,
        )
        .await;
        let t = Instant::now();
        while let Some(m) = ws.next().await {
            let m = m?;
            if m.is_text() || m.is_binary() {
                ws.feed(m).await?;
            }
        }
        let echo = t.elapsed();

        // Recv
        stream.role_client();
        let mut ws = WebSocketStream::from_raw_socket(
            &mut stream,
            Role::Client,
            None,
        )
        .await;
        let t = Instant::now();
        for _ in 0..iter {
            match ws.next().await.unwrap()? {
                Message::Text(data) => assert_eq!(data, msg),
                _ => unimplemented!(),
            }
        }
        assert!(matches!(
            ws.next().await,
            Some(Ok(Message::Close(..)))
        ));
        let recv = t.elapsed();

        let total = total.elapsed();
        Ok(BenchResult {
            name: "tokio-tungstenite",
            send,
            echo,
            recv,
            total,
        })
    }
}

// ============================================================
// web-socket
// ============================================================
mod web_socket_benchmark {
    use super::*;
    use tokio::io::AsyncWrite;
    use web_socket::*;

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Res<T, E = DynErr> = std::result::Result<T, E>;

    async fn send_msg<IO>(
        ws: &mut WebSocket<IO>,
        ty: MessageType,
        buf: &[u8],
    ) -> std::io::Result<()>
    where
        IO: Unpin + AsyncWrite,
    {
        match ty {
            MessageType::Text => {
                ws.send(std::str::from_utf8(buf).unwrap()).await
            }
            MessageType::Binary => ws.send(buf).await,
        }
    }

    pub async fn run(
        msg: &str,
        iter: usize,
        cap: usize,
    ) -> Res<BenchResult> {
        let mut stream = bench::Stream::new(cap);
        let total = Instant::now();

        // Send
        stream.role_client();
        let mut ws = WebSocket::client(&mut stream);
        let t = Instant::now();
        for _ in 0..iter {
            ws.send(msg).await?;
        }
        ws.close(()).await?;
        let send = t.elapsed();

        // Echo
        stream.role_server();
        let mut ws = WebSocket::server(&mut stream);
        let t = Instant::now();
        let mut buf = Vec::new();
        loop {
            match ws.recv_event().await? {
                Event::Data { data, ty } => match ty {
                    DataType::Stream(stream) => {
                        buf.extend_from_slice(&data);
                        if let Stream::End(ty) = stream {
                            send_msg(&mut ws, ty, &buf).await?;
                            buf.clear();
                        }
                    }
                    DataType::Complete(ty) => {
                        send_msg(&mut ws, ty, &data).await?
                    }
                },
                Event::Pong(..) => {}
                Event::Ping(data) => {
                    ws.send_ping(data).await?
                }
                Event::Error(..) | Event::Close { .. } => {
                    break ws.close(()).await?
                }
            }
        }
        let echo = t.elapsed();

        // Recv
        stream.role_client();
        let mut ws = WebSocket::client(&mut stream);
        let t = Instant::now();
        for _ in 0..iter {
            let Ok(Event::Data { ty, data }) =
                ws.recv_event().await
            else {
                panic!("invalid data")
            };
            assert!(matches!(
                ty,
                DataType::Complete(MessageType::Text)
            ));
            assert_eq!(std::str::from_utf8(&data), Ok(msg));
        }
        assert!(matches!(
            ws.recv_event().await,
            Ok(Event::Close { .. })
        ));
        let recv = t.elapsed();

        let total = total.elapsed();
        Ok(BenchResult {
            name: "web-socket",
            send,
            echo,
            recv,
            total,
        })
    }
}

// ============================================================
// sockudo-ws
// ============================================================
mod sockudo_ws_benchmark {
    use super::*;
    use bytes::Bytes;
    use futures_util::{SinkExt, StreamExt};
    use sockudo_ws::{protocol::Message, Config, WebSocketStream};

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Res<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run(
        msg: &str,
        iter: usize,
        cap: usize,
    ) -> Res<BenchResult> {
        let msg_bytes = Bytes::from(msg.as_bytes().to_vec());
        let config = Config {
            auto_ping: false,
            idle_timeout: 0,
            ..Config::default()
        };

        let mut stream = bench::Stream::new(cap);
        let total = Instant::now();

        // Send
        stream.role_client();
        let mut ws =
            WebSocketStream::client(&mut stream, config.clone());
        let t = Instant::now();
        for _ in 0..iter {
            ws.feed(Message::Text(msg_bytes.clone())).await?;
        }
        ws.send(Message::Close(None)).await?;
        let send = t.elapsed();
        drop(ws);

        // Echo
        stream.role_server();
        let mut ws =
            WebSocketStream::server(&mut stream, config.clone());
        let t = Instant::now();
        while let Some(result) = ws.next().await {
            match result? {
                msg @ Message::Text(_)
                | msg @ Message::Binary(_) => {
                    ws.feed(msg).await?;
                }
                Message::Close(_) => {
                    SinkExt::flush(&mut ws).await?;
                    break;
                }
                _ => {}
            }
        }
        let echo = t.elapsed();
        drop(ws);

        // Recv
        stream.role_client();
        let mut ws =
            WebSocketStream::client(&mut stream, config);
        let t = Instant::now();
        for _ in 0..iter {
            match ws.next().await.unwrap()? {
                Message::Text(data) => {
                    assert_eq!(data.as_ref(), msg.as_bytes())
                }
                _ => unimplemented!(),
            }
        }
        let recv = t.elapsed();

        let total = total.elapsed();
        Ok(BenchResult {
            name: "sockudo-ws",
            send,
            echo,
            recv,
            total,
        })
    }
}

// ============================================================
// tokio-websockets
// ============================================================
mod tokio_websockets_benchmark {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio_websockets::Message;

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Res<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run(
        msg: &str,
        iter: usize,
        cap: usize,
    ) -> Res<BenchResult> {
        let msg_owned = msg.to_owned();
        let mut stream = bench::Stream::new(cap);
        let total = Instant::now();

        // Send (client role — masks frames)
        stream.role_client();
        let mut ws = tokio_websockets::ClientBuilder::new()
            .take_over(&mut stream);
        let t = Instant::now();
        for _ in 0..iter {
            ws.send(Message::text(msg_owned.clone())).await?;
        }
        ws.send(Message::close(None, "")).await?;
        SinkExt::flush(&mut ws).await?;
        let send = t.elapsed();
        drop(ws);

        // Echo (server role — reads masked, writes unmasked)
        stream.role_server();
        let mut ws = tokio_websockets::ServerBuilder::new()
            .serve(&mut stream);
        let t = Instant::now();
        while let Some(m) = ws.next().await {
            let m = m?;
            if m.is_text() || m.is_binary() {
                ws.feed(m).await?;
            }
        }
        let echo = t.elapsed();
        drop(ws);

        // Recv (client role — reads unmasked frames)
        stream.role_client();
        let mut ws = tokio_websockets::ClientBuilder::new()
            .take_over(&mut stream);
        let t = Instant::now();
        for _ in 0..iter {
            let m = ws.next().await.unwrap()?;
            assert!(m.is_text());
            assert_eq!(m.as_text().unwrap(), msg);
        }
        let m = ws.next().await.unwrap()?;
        assert!(m.is_close());
        let recv = t.elapsed();

        let total = total.elapsed();
        Ok(BenchResult {
            name: "tokio-websockets",
            send,
            echo,
            recv,
            total,
        })
    }
}

// ============================================================
// wtx
// ============================================================
mod wtx_benchmark {
    use super::*;
    use std::io;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use wtx::{
        collection::Vector,
        rng::Xorshift64,
        web_socket::{
            Frame, OpCode, WebSocketBuffer, WebSocketOwned,
            WebSocketPayloadOrigin,
        },
    };

    impl wtx::stream::StreamReader for bench::Stream {
        async fn read(
            &mut self,
            bytes: &mut [u8],
        ) -> wtx::Result<usize> {
            Ok(AsyncReadExt::read(self, bytes)
                .await
                .map_err(|e: io::Error| wtx::Error::IoError(e))?)
        }
    }

    impl wtx::stream::StreamWriter for bench::Stream {
        async fn write_all(
            &mut self,
            bytes: &[u8],
        ) -> wtx::Result<()> {
            AsyncWriteExt::write_all(self, bytes)
                .await
                .map_err(|e: io::Error| wtx::Error::IoError(e))?;
            Ok(())
        }

        async fn write_all_vectored(
            &mut self,
            bytes: &[&[u8]],
        ) -> wtx::Result<()> {
            for b in bytes {
                AsyncWriteExt::write_all(self, b)
                    .await
                    .map_err(|e: io::Error| {
                        wtx::Error::IoError(e)
                    })?;
            }
            Ok(())
        }
    }

    type DynErr = Box<dyn std::error::Error + Sync + Send>;
    type Res<T, E = DynErr> = std::result::Result<T, E>;

    pub async fn run(
        msg: &str,
        iter: usize,
        cap: usize,
    ) -> Res<BenchResult> {
        let mut stream = bench::Stream::new(cap);
        let total = Instant::now();

        // Send (client role: IS_CLIENT=true)
        stream.role_client();
        let send;
        {
            let rng = Xorshift64::new(42);
            let wsb = WebSocketBuffer::default();
            let mut ws: WebSocketOwned<
                (),
                Xorshift64,
                &mut bench::Stream,
                true,
            > = wtx::web_socket::WebSocket::new(
                (), false, rng, &mut stream, wsb,
            );
            let t = Instant::now();
            for _ in 0..iter {
                let payload =
                    Vector::from_copyable_slice(msg.as_bytes())
                        .unwrap();
                ws.write_frame(&mut Frame::new_fin(
                    OpCode::Text,
                    payload,
                ))
                .await?;
            }
            ws.write_frame(&mut Frame::new_fin(
                OpCode::Close,
                Vector::new(),
            ))
            .await?;
            send = t.elapsed();
        }

        // Echo (server role: IS_CLIENT=false)
        stream.role_server();
        let echo;
        {
            let rng = Xorshift64::new(42);
            let wsb = WebSocketBuffer::default();
            let mut ws: WebSocketOwned<
                (),
                Xorshift64,
                &mut bench::Stream,
                false,
            > = wtx::web_socket::WebSocket::new(
                (), false, rng, &mut stream, wsb,
            );
            let t = Instant::now();
            let mut read_buf = Vector::new();
            loop {
                let frame = ws
                    .read_frame(
                        &mut read_buf,
                        WebSocketPayloadOrigin::Adaptive,
                    )
                    .await?;
                match frame.op_code() {
                    OpCode::Text | OpCode::Binary => {
                        let op = frame.op_code();
                        let payload =
                            Vector::from_copyable_slice(
                                frame.payload(),
                            )
                            .unwrap();
                        drop(frame);
                        ws.write_frame(&mut Frame::new_fin(
                            op, payload,
                        ))
                        .await?;
                        read_buf.clear();
                    }
                    OpCode::Close => break,
                    _ => {
                        read_buf.clear();
                    }
                }
            }
            echo = t.elapsed();
        }

        // Recv (client role: IS_CLIENT=true)
        stream.role_client();
        let recv;
        {
            let rng = Xorshift64::new(42);
            let wsb = WebSocketBuffer::default();
            let mut ws: WebSocketOwned<
                (),
                Xorshift64,
                &mut bench::Stream,
                true,
            > = wtx::web_socket::WebSocket::new(
                (), false, rng, &mut stream, wsb,
            );
            let t = Instant::now();
            let mut read_buf = Vector::new();
            for _ in 0..iter {
                let frame = ws
                    .read_frame(
                        &mut read_buf,
                        WebSocketPayloadOrigin::Adaptive,
                    )
                    .await?;
                assert_eq!(frame.op_code(), OpCode::Text);
                assert_eq!(
                    frame.payload() as &[u8],
                    msg.as_bytes()
                );
                read_buf.clear();
            }
            let frame = ws
                .read_frame(
                    &mut read_buf,
                    WebSocketPayloadOrigin::Adaptive,
                )
                .await?;
            assert_eq!(frame.op_code(), OpCode::Close);
            recv = t.elapsed();
        }

        let total = total.elapsed();
        Ok(BenchResult {
            name: "wtx",
            send,
            echo,
            recv,
            total,
        })
    }
}
