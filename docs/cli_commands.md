# CLI Commands

| Command                                    | Description                                             |
|--------------------------------------------|---------------------------------------------------------|
| `irigtime summary <file>`                  | Packet counts, time channels, RTC range, jump detection |
| `irigtime channels <file>`                 | Per-channel time source inventory                       |
| `irigtime jumps <file> [--threshold-ms N]` | Discontinuity detection                                 |
| `irigtime timeline <file> [--limit N]`     | Per-packet RTC + resolved absolute time                 |
| `irigtime csv <file> [--output path]`      | Full timestamp export                                   |
| `irigtime correlate <file> <rtc_hex>`      | Resolve one RTC against all channels                    |
