pub mod header;
pub mod message;
pub mod question;
pub mod record;

pub use header::DnsHeader;
pub use message::{DnsMessage, RequestCase};
pub use question::Question;
