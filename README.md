### Candidates

- [fastwebsockets](https://github.com/denoland/fastwebsockets)
- [soketto](https://github.com/paritytech/soketto)
- [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite)
- [web-socket](https://github.com/nurmohammed840/websocket.rs)
- [sockudo-ws](https://github.com/sockudo/sockudo-ws)

### Run benchmark

```bash
cargo r -r
```

### Results

#### AWS Graviton4 4 Core
```
╔══════════════════════════════════════════════════╗
║  Payload: 14B    (   14 bytes) x 100000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
sockudo-ws               3.46ms     8.87ms     6.27ms    18.61ms
web-socket               4.10ms    11.71ms     6.49ms    22.30ms
fastwebsockets           7.17ms    18.26ms    10.01ms    35.43ms
tokio-websockets        13.41ms    16.49ms     9.05ms    38.95ms
wtx                      7.30ms    25.84ms    12.56ms    45.91ms
tokio-tungstenite       11.42ms    21.71ms    14.28ms    47.41ms
soketto                 12.75ms    35.85ms    17.98ms    66.58ms

╔══════════════════════════════════════════════════╗
║  Payload: 100B   (  100 bytes) x 100000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
sockudo-ws               9.05ms    15.30ms     8.09ms    32.96ms
web-socket              11.58ms    19.24ms     6.89ms    37.71ms
fastwebsockets          11.05ms    23.72ms    12.54ms    47.31ms
tokio-websockets        15.34ms    20.67ms    11.53ms    47.55ms
wtx                      5.88ms    27.93ms    16.79ms    50.60ms
tokio-tungstenite       13.71ms    24.88ms    14.83ms    53.43ms
soketto                 24.98ms    57.42ms    27.67ms   110.07ms

╔══════════════════════════════════════════════════╗
║  Payload: 200B   (  200 bytes) x 100000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
fastwebsockets           7.88ms    26.43ms    13.29ms    47.60ms
sockudo-ws              14.92ms    23.91ms     8.94ms    50.46ms
tokio-websockets        18.17ms    21.94ms    10.90ms    51.02ms
wtx                     10.67ms    33.50ms    17.28ms    61.46ms
tokio-tungstenite       16.38ms    29.03ms    16.66ms    62.07ms
web-socket              21.19ms    33.01ms    11.48ms    65.68ms
soketto                 39.16ms    85.21ms    43.31ms   167.68ms

╔══════════════════════════════════════════════════╗
║  Payload: 500B   (  500 bytes) x 100000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
tokio-websockets        26.62ms    32.45ms    13.11ms    72.19ms
fastwebsockets          20.02ms    36.64ms    17.40ms    74.06ms
wtx                     19.10ms    42.12ms    19.38ms    80.60ms
tokio-tungstenite       24.17ms    38.96ms    19.53ms    82.67ms
sockudo-ws              29.95ms    41.41ms    12.78ms    90.84ms
web-socket              48.55ms    64.13ms    13.46ms   126.13ms
soketto                 81.30ms   161.45ms    79.15ms   321.90ms

╔══════════════════════════════════════════════════╗
║  Payload: 800B   (  800 bytes) x 100000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
tokio-websockets        34.81ms    40.43ms    14.69ms    89.93ms
fastwebsockets          27.70ms    45.57ms    17.97ms    91.25ms
wtx                     26.87ms    48.59ms    19.95ms    95.41ms
tokio-tungstenite       31.33ms    48.23ms    22.75ms   102.31ms
sockudo-ws              42.13ms    54.68ms    13.58ms   120.67ms
web-socket              74.57ms    95.70ms    16.53ms   186.80ms
soketto                120.64ms   236.83ms   114.74ms   472.21ms

╔══════════════════════════════════════════════════╗
║  Payload: 1KB    ( 1024 bytes) x 100000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
fastwebsockets          33.68ms    50.04ms    18.83ms   102.56ms
tokio-websockets        40.56ms    46.10ms    16.03ms   102.69ms
wtx                     30.95ms    54.65ms    21.41ms   107.01ms
tokio-tungstenite       36.45ms    54.38ms    25.52ms   116.35ms
sockudo-ws              51.96ms    65.26ms    15.17ms   145.26ms
web-socket              97.95ms   117.31ms    18.92ms   234.18ms
soketto                150.83ms   292.53ms   141.41ms   584.78ms

╔══════════════════════════════════════════════════╗
║  Payload: 2KB    ( 2048 bytes) x 100000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
fastwebsockets          59.22ms    73.90ms    27.24ms   160.37ms
tokio-websockets        69.45ms    74.84ms    25.20ms   169.50ms
wtx                     58.87ms    90.20ms    26.34ms   175.41ms
tokio-tungstenite       63.92ms    85.62ms    36.38ms   185.93ms
sockudo-ws              97.91ms   116.59ms    23.11ms   261.94ms
web-socket             188.67ms   221.96ms    28.42ms   439.05ms
soketto                287.72ms   552.49ms   269.34ms      1.11s

╔══════════════════════════════════════════════════╗
║  Payload: 10KB   (10240 bytes) x 50000 msgs  ║
╚══════════════════════════════════════════════════╝

Library                    Send       Echo       Recv      Total
--------------------------------------------------------------
fastwebsockets         124.14ms   149.71ms    53.06ms   326.92ms
tokio-websockets       130.68ms   154.05ms    51.70ms   336.43ms
wtx                    126.43ms   162.28ms    49.18ms   337.89ms
tokio-tungstenite      134.93ms   174.90ms    62.42ms   372.26ms
sockudo-ws             232.55ms   256.77ms    50.82ms   594.00ms
web-socket             451.24ms   519.63ms    57.72ms      1.03s
soketto                692.86ms      1.31s   644.12ms      2.65s
```
