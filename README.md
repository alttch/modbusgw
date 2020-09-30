# modbusgw

Simple TCP&lt;->RTU Modbus Gateway Server

## Usage

Download binary from releases and enjoy

```
  -p,--port PORT        serial port device (REQUIRED)
  -l,--listen LISTEN    host:port to listen (default: 0.0.0.0:5502)
  -b,--baud-rate BAUD_RATE
                        serial port baud rate (default: 9600)
  --char-size CHAR_SIZE serial port char size (default: 8)
  --parity PARITY       serial port parity (default: N)
  --stop-bits STOP_BITS serial port stop bits (default: 1)
  --timeout TIMEOUT     serial port timeout (default: 1s)
  --delay DELAY         delay between frames (default: 0.02s)
```

## Building from sources

```
cargo build --release
```
