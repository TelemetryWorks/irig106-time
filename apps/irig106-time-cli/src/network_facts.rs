//! Partial Traces: L3-CLI-105 <- L2-CLI-105 <- L1-CLI-005.
//! Wire observation facts; separate from validity/admission and clock basis.
use serde_json::{Value, json};
use std::io::{self, Write};
#[derive(Clone, Copy, Debug)]
pub(super) struct NetworkFacts(pub(super) u8);
impl NetworkFacts {
    pub(super) fn observed(body: &[u8]) -> Option<Self> {
        body.get(..4).map(|b| Self(b[0]))
    }
    pub(super) fn json(self) -> Value {
        let ntf = self.0 >> 4;
        let ts = self.0 & 15;
        json!({"ntf":ntf,"protocol":match ntf {0=>"NTP",1=>"IEEE1588-2002",2=>"IEEE1588-2008",_=>"reserved"},
            "ts":ts,"status":match ts {0=>"not-valid",1=>"valid",_=>"reserved"}})
    }
    // Packet-observed fields only; no conversion or reference lookup here.
    pub(super) fn write_csv(self, out: &mut dyn Write) -> io::Result<()> {
        let ntf = self.0 >> 4;
        let ts = self.0 & 15;
        let protocol = match ntf {
            0 => "NTP",
            1 => "IEEE1588-2002",
            2 => "IEEE1588-2008",
            _ => "reserved",
        };
        // Closed ASCII spellings contain no CSV separators/quotes/newlines.
        write!(out, ",{ntf},{protocol},{ts},")?;
        match ts {
            0 => out.write_all(b"NotValid"),
            1 => out.write_all(b"Valid"),
            _ => write!(out, "RSVD-{ts}"),
        }
    }
    pub(super) fn annotate(value: String, facts: Option<Self>, scope: &str) -> String {
        let Some(facts) = facts else {
            return value;
        };
        let ntf = facts.0 >> 4;
        let protocol = match ntf {
            0 => "NTP".into(),
            1 => "IEEE1588-2002".into(),
            2 => "IEEE1588-2008".into(),
            _ => format!("Reserved NTF {ntf}"),
        };
        let ts = facts.0 & 15;
        let status = match ts {
            0 => "not-valid",
            1 => "valid",
            _ => "reserved",
        };
        format!("{value} [{scope}: {protocol} TS={ts} {status}]")
    }
}
