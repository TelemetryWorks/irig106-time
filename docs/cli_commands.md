# CLI Commands

| Command                                    | Description                                             |
|--------------------------------------------|---------------------------------------------------------|
| `ch10time summary <file>`                  | Packet counts, time channels, RTC range, jump detection |
| `ch10time channels <file>`                 | Per-channel time source inventory                       |
| `ch10time jumps <file> [--threshold-ms N]` | Discontinuity detection                                 |
| `ch10time timeline <file> [--limit N]`     | Per-packet RTC + resolved absolute time                 |
| `ch10time csv <file> [--output path]`      | Full timestamp export                                   |
| `ch10time correlate <file> <rtc_hex>`      | Resolve one RTC against all channels                    |
