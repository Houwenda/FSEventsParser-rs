# FSEventsParser-rs
Yet another fseventsd log parser for forensics.

## Usage
Currently only supports JSON output format.

```bash
% ./fsevents_parser_rs -h
fsevents_parser_rs 0.1.0

USAGE:
    fsevents_parser_rs [OPTIONS]

OPTIONS:
    -f, --format <FORMAT>              [default: json] [possible values: json, csv, sqlite]
    -h, --help                         Print help information
    -i, --input-path <INPUT_PATH>      [default: /System/Volumes/Data/.fseventsd]
    -o, --output-path <OUTPUT_PATH>    [default: ./output.json]
    -V, --version                      Print version information
```

```bash
% sudo ./fsevents_parser_rs
found 21 archives in /System/Volumes/Data/.fseventsd
---------- 0000000000089492 ----------
page count: 2
entry count: 2049
entry count: 681
......
......
---------- 000000000004c323 ----------
page count: 2
entry count: 1850
entry count: 880
% cat ./output.json | tail -n 3
{"path":"private/var/log/DiagnosticMessages/StoreData\u0000","id":308039,"flags":"FSE_CONTENT_MODIFIED | FSE_IS_FILE","create_ts":1664093703,"modiy_ts":1664093703,"source":"000000000004c323"}
{"path":"private/var/log/system.log\u0000","id":308036,"flags":"FSE_CONTENT_MODIFIED | FSE_IS_FILE","create_ts":1664093703,"modiy_ts":1664093703,"source":"000000000004c323"}
{"path":"private/var/root/Library/Logs/Bluetooth/bluetoothd-hci-latest.pklg\u0000","id":309733,"flags":"FSE_CONTENT_MODIFIED | FSE_IS_FILE","create_ts":1664093703,"modiy_ts":1664093703,"source":"000000000004c323"}
```

## References
[FSEventsParser](https://github.com/dlcowen/FSEventsParser)

[MacOS File System Events Disk Log Stream format](https://github.com/libyal/dtformats/blob/main/documentation/MacOS%20File%20System%20Events%20Disk%20Log%20Stream%20format.asciidoc)