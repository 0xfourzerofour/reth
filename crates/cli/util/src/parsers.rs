use alloy_eips::BlockHashOrNumber;
use alloy_primitives::B256;
use reth_fs_util::FsPathError;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs},
    path::Path,
    str::FromStr,
    time::Duration,
};

/// Helper to parse a [Duration] from seconds
pub fn parse_duration_from_secs(arg: &str) -> eyre::Result<Duration, std::num::ParseIntError> {
    let seconds = arg.parse()?;
    Ok(Duration::from_secs(seconds))
}

/// Helper to parse a [Duration] from seconds if it's a number or milliseconds if the input contains
/// a `ms` suffix:
///  * `5ms` -> 5 milliseconds
///  * `5` -> 5 seconds
///  * `5s` -> 5 seconds
pub fn parse_duration_from_secs_or_ms(
    arg: &str,
) -> eyre::Result<Duration, std::num::ParseIntError> {
    if arg.ends_with("ms") {
        arg.trim_end_matches("ms").parse().map(Duration::from_millis)
    } else if arg.ends_with('s') {
        arg.trim_end_matches('s').parse().map(Duration::from_secs)
    } else {
        arg.parse().map(Duration::from_secs)
    }
}

/// Parse [`BlockHashOrNumber`]
pub fn hash_or_num_value_parser(value: &str) -> eyre::Result<BlockHashOrNumber, eyre::Error> {
    match B256::from_str(value) {
        Ok(hash) => Ok(BlockHashOrNumber::Hash(hash)),
        Err(_) => Ok(BlockHashOrNumber::Number(value.parse()?)),
    }
}

/// Error thrown while parsing a socket address.
#[derive(thiserror::Error, Debug)]
pub enum SocketAddressParsingError {
    /// Failed to convert the string into a socket addr
    #[error("could not parse socket address: {0}")]
    Io(#[from] std::io::Error),
    /// Input must not be empty
    #[error("cannot parse socket address from empty string")]
    Empty,
    /// Failed to parse the address
    #[error("could not parse socket address from {0}")]
    Parse(String),
    /// Failed to parse port
    #[error("could not parse port: {0}")]
    Port(#[from] std::num::ParseIntError),
}

/// Parse a [`SocketAddr`] from a `str`.
///
/// The following formats are checked:
///
/// - If the value can be parsed as a `u16` or starts with `:` it is considered a port, and the
///   hostname is set to `localhost`.
/// - If the value contains `:` it is assumed to be the format `<host>:<port>`
/// - Otherwise it is assumed to be a hostname
///
/// An error is returned if the value is empty.
pub fn parse_socket_address(value: &str) -> eyre::Result<SocketAddr, SocketAddressParsingError> {
    if value.is_empty() {
        return Err(SocketAddressParsingError::Empty)
    }

    if let Some(port) = value.strip_prefix(':').or_else(|| value.strip_prefix("localhost:")) {
        let port: u16 = port.parse()?;
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
    }
    if let Ok(port) = value.parse() {
        return Ok(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port))
    }
    value
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| SocketAddressParsingError::Parse(value.to_string()))
}

/// Wrapper around [`reth_fs_util::read_json_file`] which can be used as a clap value parser.
pub fn read_json_from_file<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, FsPathError> {
    reth_fs_util::read_json_file(Path::new(path))
}

/// Parses an ether value from a string.
///
/// The amount in eth like "1.05" will be interpreted in wei (1.05 * 1e18).
/// Supports both decimal and integer inputs.
///
/// # Examples
/// - "1.05" -> 1.05 ETH = 1.05 * 10^18 wei
/// - "2" -> 2 ETH = 2 * 10^18 wei
pub fn parse_ether_value(value: &str) -> eyre::Result<u128> {
    let eth = value.parse::<f64>()?;
    if eth.is_sign_negative() {
        return Err(eyre::eyre!("Ether value cannot be negative"))
    }
    let wei = eth * 1e18;
    Ok(wei as u128)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn parse_socket_addresses() {
        for value in ["localhost:9000", ":9000", "9000"] {
            let socket_addr = parse_socket_address(value)
                .unwrap_or_else(|_| panic!("could not parse socket address: {value}"));

            assert!(socket_addr.ip().is_loopback());
            assert_eq!(socket_addr.port(), 9000);
        }
    }

    #[test]
    fn parse_socket_address_random() {
        let port: u16 = rand::rng().random();

        for value in [format!("localhost:{port}"), format!(":{port}"), port.to_string()] {
            let socket_addr = parse_socket_address(&value)
                .unwrap_or_else(|_| panic!("could not parse socket address: {value}"));

            assert!(socket_addr.ip().is_loopback());
            assert_eq!(socket_addr.port(), port);
        }
    }

    #[test]
    fn parse_ms_or_seconds() {
        let ms = parse_duration_from_secs_or_ms("5ms").unwrap();
        assert_eq!(ms, Duration::from_millis(5));

        let seconds = parse_duration_from_secs_or_ms("5").unwrap();
        assert_eq!(seconds, Duration::from_secs(5));

        let seconds = parse_duration_from_secs_or_ms("5s").unwrap();
        assert_eq!(seconds, Duration::from_secs(5));

        assert!(parse_duration_from_secs_or_ms("5ns").is_err());
    }

    #[test]
    fn parse_ether_values() {
        // Test basic decimal value
        let wei = parse_ether_value("1.05").unwrap();
        assert_eq!(wei, 1_050_000_000_000_000_000u128);

        // Test integer value
        let wei = parse_ether_value("2").unwrap();
        assert_eq!(wei, 2_000_000_000_000_000_000u128);

        // Test zero
        let wei = parse_ether_value("0").unwrap();
        assert_eq!(wei, 0);

        // Test negative value fails
        assert!(parse_ether_value("-1").is_err());

        // Test invalid input fails
        assert!(parse_ether_value("abc").is_err());
    }
}
