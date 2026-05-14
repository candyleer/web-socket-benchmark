### Candidates

- [fastwebsockets](https://github.com/denoland/fastwebsockets)
- [soketto](https://github.com/paritytech/soketto)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- [web-socket](https://github.com/nurmohammed840/websocket.rs)

### Run benchmark

```bash
cargo r -r
```

### Results

#### AWS Graviton4 4 Core
```
fastwebsockets (send):  7.857521ms
fastwebsockets (echo):  18.123787ms
fastwebsockets (recv):  10.046021ms
fastwebsockets:         36.032242ms


soketto (send):  12.81325ms
soketto (echo):  36.085508ms
soketto (recv):  18.214928ms
soketto:         67.117091ms


tokio_tungstenite (send):  11.172872ms
tokio_tungstenite (echo):  23.055114ms
tokio_tungstenite (recv):  15.536063ms
tokio_tungstenite:         49.775165ms


web-socket (send):  4.210412ms
web-socket (echo):  11.918639ms
web-socket (recv):  6.013216ms
web-socket:         22.142436ms


sockudo-ws (send):  2.022251ms
sockudo-ws (echo):  9.086881ms
sockudo-ws (recv):  6.220709ms
sockudo-ws:         17.333161ms
```
