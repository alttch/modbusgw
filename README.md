# modbusgw

Simple TCP&lt;->RTU Modbus Gateway Server

## Usage

Download binary from releases and enjoy

```
-p,--port PORT        serial port device (REQUIRED)
-l,--listen LISTEN    host:port to listen
-b,--baud-rate BAUD_RATE
                    serial port baud rate
--char-size CHAR_SIZE serial port char size
--parity PARITY       serial port parity
--stop-bits STOP_BITS serial port stop bits
--timeout TIMEOUT     serial port timeout
```

## Building from sources

```
cargo build --release
```
