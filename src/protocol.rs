/// Razer HID report: always 90 bytes.
///
/// Layout (from usbmon captures of other Razer devices — headset specifics TBD):
///   [0]     Report ID       (0x00)
///   [1]     Status          (0x00 = new cmd, 0x02 = busy, 0x01 = ok, 0x03 = fail, 0x04 = timeout)
///   [2]     Transaction ID  (arbitrary; echo'd back in response)
///   [3..4]  Remaining pkts  (0x00 0x00 for single-packet)
///   [5]     Protocol type   (0x00)
///   [6]     Data size       (number of meaningful argument bytes)
///   [7]     Command class
///   [8]     Command ID
///   [9..87] Arguments
///   [88]    CRC             (XOR of bytes [2..87])
///   [89]    Reserved        (0x00)
pub const REPORT_LEN: usize = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Report([u8; REPORT_LEN]);

impl Report {
    pub fn new(transaction_id: u8, class: u8, id: u8, data: &[u8]) -> Self {
        assert!(data.len() <= 79, "argument data exceeds report capacity");

        let mut buf = [0u8; REPORT_LEN];
        buf[0] = 0x00; // report ID
        buf[2] = transaction_id;
        buf[6] = data.len() as u8;
        buf[7] = class;
        buf[8] = id;
        buf[9..9 + data.len()].copy_from_slice(data);
        buf[88] = crc(&buf);
        Self(buf)
    }

    pub fn from_bytes(buf: [u8; REPORT_LEN]) -> Self {
        Self(buf)
    }

    pub fn as_bytes(&self) -> &[u8; REPORT_LEN] {
        &self.0
    }

    pub fn status(&self) -> ResponseStatus {
        ResponseStatus::from(self.0[1])
    }

    pub fn data(&self) -> &[u8] {
        let len = self.0[6] as usize;
        &self.0[9..9 + len.min(79)]
    }
}

fn crc(buf: &[u8; REPORT_LEN]) -> u8 {
    buf[2..88].iter().fold(0u8, |acc, &b| acc ^ b)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStatus {
    Ok,
    Busy,
    Fail,
    Timeout,
    Unknown(u8),
}

impl From<u8> for ResponseStatus {
    fn from(b: u8) -> Self {
        match b {
            0x01 => Self::Ok,
            0x02 => Self::Busy,
            0x03 => Self::Fail,
            0x04 => Self::Timeout,
            other => Self::Unknown(other),
        }
    }
}

// ---------------------------------------------------------------------------
// Known commands (placeholders until Windows capture fills these in)
// ---------------------------------------------------------------------------

/// Command class / ID pairs, named for what they likely are.
/// Bytes marked TODO need to be confirmed from pcap.
pub mod cmd {
    /// Sidetone level (0x00–0x64).
    /// Class/ID: TODO
    pub const SIDETONE_CLASS: u8 = 0x00; // TODO
    pub const SIDETONE_ID: u8 = 0x00; // TODO

    /// Battery level query.
    /// Class/ID: TODO
    pub const BATTERY_CLASS: u8 = 0x00; // TODO
    pub const BATTERY_ID: u8 = 0x00; // TODO

    /// Mic monitoring level (0x00–0x64).
    /// Class/ID: TODO
    pub const MIC_MONITOR_CLASS: u8 = 0x00; // TODO
    pub const MIC_MONITOR_ID: u8 = 0x00; // TODO
}
