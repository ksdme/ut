use crate::{
    args::StringInput,
    tool::{Output, Tool},
};
use anyhow::{Result, bail};
use clap::Parser;
use serde_json::json;
use std::net::Ipv4Addr;

#[derive(Parser, Debug)]
#[command(name = "ip", about = "IP address utilities")]
pub struct IPTool {
    #[command(subcommand)]
    command: IPCommand,
}

#[derive(Parser, Debug)]
pub enum IPCommand {
    /// CIDR related utilities
    CIDR {
        #[command(subcommand)]
        command: CIDRCommand,
    },
}

#[derive(Parser, Debug)]
pub enum CIDRCommand {
    /// Show information about a CIDR block
    Describe {
        /// CIDR notation (e.g. 192.168.1.0/24)
        notation: StringInput,
    },
}

impl Tool for IPTool {
    fn cli() -> clap::Command {
        <Self as clap::CommandFactory>::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        match &self.command {
            IPCommand::CIDR { command } => match command {
                CIDRCommand::Describe { notation } => {
                    Ok(Some(Output::JsonValue(cidr_info(notation.as_ref())?)))
                }
            },
        }
    }
}

fn cidr_info(notation: &str) -> Result<serde_json::Value> {
    let parts: Vec<&str> = notation.split('/').collect();
    if parts.len() != 2 {
        bail!("Invalid CIDR notation. Expected format: IP/prefix (e.g., 192.168.1.0/24)");
    }

    let ip: Ipv4Addr = parts[0]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid IP address: {}", parts[0]))?;

    let prefix: u8 = parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid prefix length: {}", parts[1]))?;

    if prefix > 32 {
        bail!("Prefix length must be between 0 and 32, got: {}", prefix);
    }

    let ip_u32: u32 = ip.into();
    let netmask: u32 = if prefix == 0 {
        0
    } else {
        !0u32 << (32 - prefix)
    };
    let network: u32 = ip_u32 & netmask;
    let wildcard: u32 = !netmask;
    let broadcast: u32 = network | wildcard;

    let (first_host, last_host, total_hosts): (u32, u32, u64) = match prefix {
        32 => (ip_u32, ip_u32, 1),
        31 => (network, broadcast, 2),
        _ => (network + 1, broadcast - 1, (1u64 << (32 - prefix)) - 2),
    };

    Ok(json!({
        "address": Ipv4Addr::from(ip_u32).to_string(),
        "address_decimal": ip_u32,
        "address_hex": ip_to_hex(ip_u32),
        "network": Ipv4Addr::from(network).to_string(),
        "network_decimal": network,
        "network_hex": ip_to_hex(network),
        "broadcast": Ipv4Addr::from(broadcast).to_string(),
        "broadcast_decimal": broadcast,
        "broadcast_hex": ip_to_hex(broadcast),
        "first_host": Ipv4Addr::from(first_host).to_string(),
        "last_host": Ipv4Addr::from(last_host).to_string(),
        "total_hosts": total_hosts,
        "prefix": prefix,
        "netmask": Ipv4Addr::from(netmask).to_string(),
        "netmask_hex": ip_to_hex(netmask),
        "wildcard": Ipv4Addr::from(wildcard).to_string(),
        "wildcard_hex": ip_to_hex(wildcard),
    }))
}

fn ip_to_hex(ip: u32) -> String {
    let bytes = ip.to_be_bytes();
    format!(
        "{:02X}.{:02X}.{:02X}.{:02X}",
        bytes[0], bytes[1], bytes[2], bytes[3]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_cidr(input: &str) -> serde_json::Value {
        let tool = IPTool {
            command: IPCommand::CIDR {
                command: CIDRCommand::Describe {
                    notation: StringInput(input.to_string()),
                },
            },
        };
        match tool.execute().unwrap().unwrap() {
            Output::JsonValue(v) => v,
            _ => panic!("Expected JsonValue"),
        }
    }

    #[test]
    fn test_class_c_network() {
        let result = run_cidr("192.168.1.100/24");
        assert_eq!(result["address"], "192.168.1.100");
        assert_eq!(result["address_decimal"], 3232235876u64);
        assert_eq!(result["address_hex"], "C0.A8.01.64");
        assert_eq!(result["network"], "192.168.1.0");
        assert_eq!(result["broadcast"], "192.168.1.255");
        assert_eq!(result["first_host"], "192.168.1.1");
        assert_eq!(result["last_host"], "192.168.1.254");
        assert_eq!(result["total_hosts"], 254);
        assert_eq!(result["prefix"], 24);
        assert_eq!(result["netmask"], "255.255.255.0");
        assert_eq!(result["netmask_hex"], "FF.FF.FF.00");
        assert_eq!(result["wildcard"], "0.0.0.255");
        assert_eq!(result["wildcard_hex"], "00.00.00.FF");
    }

    #[test]
    fn test_single_host() {
        let result = run_cidr("10.0.0.1/32");
        assert_eq!(result["network"], "10.0.0.1");
        assert_eq!(result["broadcast"], "10.0.0.1");
        assert_eq!(result["first_host"], "10.0.0.1");
        assert_eq!(result["last_host"], "10.0.0.1");
        assert_eq!(result["total_hosts"], 1);
        assert_eq!(result["netmask"], "255.255.255.255");
        assert_eq!(result["wildcard"], "0.0.0.0");
    }

    #[test]
    fn test_point_to_point() {
        let result = run_cidr("10.0.0.0/31");
        assert_eq!(result["network"], "10.0.0.0");
        assert_eq!(result["broadcast"], "10.0.0.1");
        assert_eq!(result["first_host"], "10.0.0.0");
        assert_eq!(result["last_host"], "10.0.0.1");
        assert_eq!(result["total_hosts"], 2);
    }

    #[test]
    fn test_class_a_network() {
        let result = run_cidr("10.0.0.0/8");
        assert_eq!(result["network"], "10.0.0.0");
        assert_eq!(result["broadcast"], "10.255.255.255");
        assert_eq!(result["netmask"], "255.0.0.0");
        assert_eq!(result["wildcard"], "0.255.255.255");
        assert_eq!(result["total_hosts"], 16777214u64);
    }

    #[test]
    fn test_all_networks() {
        let result = run_cidr("0.0.0.0/0");
        assert_eq!(result["network"], "0.0.0.0");
        assert_eq!(result["broadcast"], "255.255.255.255");
        assert_eq!(result["netmask"], "0.0.0.0");
        assert_eq!(result["wildcard"], "255.255.255.255");
        assert_eq!(result["total_hosts"], 4294967294u64);
    }

    #[test]
    fn test_invalid_cidr_no_prefix() {
        let tool = IPTool {
            command: IPCommand::CIDR {
                command: CIDRCommand::Describe {
                    notation: StringInput("192.168.1.0".to_owned()),
                },
            },
        };
        assert!(tool.execute().is_err());
    }

    #[test]
    fn test_invalid_prefix_too_large() {
        let tool = IPTool {
            command: IPCommand::CIDR {
                command: CIDRCommand::Describe {
                    notation: StringInput("192.168.1.0/33".to_owned()),
                },
            },
        };
        assert!(tool.execute().is_err());
    }

    #[test]
    fn test_invalid_ip() {
        let tool = IPTool {
            command: IPCommand::CIDR {
                command: CIDRCommand::Describe {
                    notation: StringInput("256.168.1.0/24".to_owned()),
                },
            },
        };
        assert!(tool.execute().is_err());
    }
}
