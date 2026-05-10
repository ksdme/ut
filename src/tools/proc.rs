//! Process and port inspection utilities.
//!
//! Provides cross-platform commands to list running processes, look up a
//! process by PID or name, identify which process owns a given TCP port, and
//! kill a process by port or PID.
//!
//! Port-to-process lookup is implemented per platform:
//! - **Linux**: parses `/proc/net/tcp` and `/proc/<pid>/fd/` directly.
//! - **macOS**: invokes the system `lsof` utility.
//! - **Windows**: invokes the system `netstat` utility.
//! - **Other**: returns an unsupported error.

use crate::tool::{Output, Tool};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use sysinfo::{Pid, Signal, System};

#[derive(Parser, Debug)]
#[command(
    name = "proc",
    about = "Query running processes and open TCP ports",
    long_about = "Inspect running processes and TCP port ownership.\n\n\
                  Port-to-process mapping notes:\n  \
                  - Linux:   reads /proc/net/tcp directly (no extra tools needed)\n  \
                  - macOS:   requires `lsof` (pre-installed on all macOS systems)\n  \
                  - Windows: requires `netstat` (pre-installed on all Windows systems)\n\n  \
                  Viewing or killing processes owned by other users may require elevated privileges.\n\n\
                  Examples:\n  \
                  ut proc list\n  \
                  ut proc list --name node\n  \
                  ut proc name nginx\n  \
                  ut proc pid 1234\n  \
                  ut proc port 3000\n  \
                  ut proc ports\n  \
                  ut proc kill --port 3000\n  \
                  ut proc kill --pid 1234 --force"
)]
pub struct ProcTool {
    #[command(subcommand)]
    command: ProcCommand,
}

#[derive(Subcommand, Debug)]
enum ProcCommand {
    /// List all running processes, optionally filtered by name
    List {
        /// Filter by process name (case-insensitive substring match)
        #[arg(long, short = 'n')]
        name: Option<String>,

        /// Include the full command line arguments for each process
        ///
        /// On Windows, reading command line arguments of other users'
        /// processes may require elevated privileges.
        #[arg(long, short = 'c')]
        cmd: bool,
    },
    /// Find all processes whose name contains the given string
    Name {
        /// Process name to search for (case-insensitive substring match)
        name: String,

        /// Include the full command line arguments for each process
        ///
        /// On Windows, reading command line arguments of other users'
        /// processes may require elevated privileges.
        #[arg(long, short = 'c')]
        cmd: bool,
    },
    /// Show full details for a specific process ID
    Pid {
        /// The numeric process ID to look up
        pid: u32,
    },
    /// Find which process(es) are listening on a given TCP port
    ///
    /// May require elevated privileges to see processes owned by other users.
    Port {
        /// TCP port number to look up
        port: u16,
    },
    /// List all TCP listening ports with their owning processes
    ///
    /// May require elevated privileges to see processes owned by other users.
    Ports,
    /// Kill the process listening on a port, or a process by PID
    ///
    /// On Unix, sends SIGTERM by default (graceful shutdown) or SIGKILL with
    /// --force (immediate termination). On Windows, both behave as a hard kill
    /// since the OS has no SIGTERM equivalent.
    ///
    /// May require elevated privileges to kill processes owned by other users.
    Kill {
        /// Port whose owning process should be killed (mutually exclusive with --pid)
        #[arg(long, conflicts_with = "pid", required_unless_present = "pid")]
        port: Option<u16>,

        /// Process ID to kill (mutually exclusive with --port)
        #[arg(long, conflicts_with = "port", required_unless_present = "port")]
        pid: Option<u32>,

        /// Send SIGKILL instead of SIGTERM on Unix (always a hard kill on Windows)
        #[arg(long, short = 'f')]
        force: bool,
    },
}

impl Tool for ProcTool {
    fn cli() -> clap::Command {
        <Self as clap::CommandFactory>::command()
    }

    fn execute(&self) -> Result<Option<Output>> {
        match &self.command {
            ProcCommand::List { name, cmd } => {
                let processes = list_processes(name.as_deref(), *cmd)?;
                Ok(Some(Output::Table(json!(processes))))
            }
            ProcCommand::Name { name, cmd } => {
                let processes = list_processes(Some(name.as_str()), *cmd)?;
                Ok(Some(Output::Table(json!(processes))))
            }
            ProcCommand::Pid { pid } => {
                let info = process_by_pid(*pid)?;
                Ok(Some(Output::JsonValue(info)))
            }
            ProcCommand::Port { port } => {
                let entries = find_port_owners(*port)?;
                Ok(Some(Output::JsonValue(json!(entries))))
            }
            ProcCommand::Ports => {
                let entries = all_listening_ports()?;
                Ok(Some(Output::Table(json!(entries))))
            }
            ProcCommand::Kill { port, pid, force } => {
                let result = kill_process(port.as_ref(), pid.as_ref(), *force)?;
                Ok(Some(Output::JsonValue(result)))
            }
        }
    }
}

// ── sysinfo helpers ────────────────────────────────────────────────────────

fn new_system() -> System {
    System::new_all()
}

/// Full process details — used by the `pid` subcommand.
fn process_to_json(pid: u32, proc: &sysinfo::Process) -> Value {
    json!({
        "pid": pid,
        "name": proc.name().to_string_lossy(),
        "exe": proc.exe().map(|p| p.display().to_string()),
        "status": format!("{:?}", proc.status()),
        "memory_bytes": proc.memory(),
        "cpu_usage": proc.cpu_usage(),
    })
}

/// Compact process row — used by the `list` subcommand table.
/// When `show_cmd` is true, a `cmd` column with the full command line is appended.
fn process_row_json(pid: u32, proc: &sysinfo::Process, show_cmd: bool) -> Value {
    let mut row = json!({
        "pid": pid,
        "name": proc.name().to_string_lossy(),
        "status": format!("{:?}", proc.status()),
        "memory_bytes": proc.memory(),
        "cpu_usage": proc.cpu_usage(),
    });

    if show_cmd {
        let cmd = proc
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        row["cmd"] = json!(cmd);
    }

    row
}

fn list_processes(name_filter: Option<&str>, show_cmd: bool) -> Result<Vec<Value>> {
    let sys = new_system();
    let mut procs: Vec<Value> = sys
        .processes()
        .iter()
        .filter(|(_, p)| {
            name_filter.map_or(true, |filter| {
                p.name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(&filter.to_lowercase())
            })
        })
        .map(|(pid, p)| process_row_json(pid.as_u32(), p, show_cmd))
        .collect();
    procs.sort_by_key(|p| p["pid"].as_u64().unwrap_or(0));
    Ok(procs)
}

fn process_by_pid(pid: u32) -> Result<Value> {
    let sys = new_system();
    let spid = Pid::from_u32(pid);
    sys.process(spid)
        .map(|p| process_to_json(pid, p))
        .with_context(|| format!("No process found with PID {pid}"))
}

// ── Port-to-process mapping ────────────────────────────────────────────────

struct ListeningSocket {
    port: u16,
    pid: u32,
}

/// Returns all processes (deduplicated by pid) listening on `port`.
fn find_port_owners(port: u16) -> Result<Vec<Value>> {
    let raw = listening_sockets()?;
    let sys = new_system();

    let mut seen_pids = std::collections::HashSet::new();
    let entries: Vec<Value> = raw
        .iter()
        .filter(|s| s.port == port && seen_pids.insert(s.pid))
        .map(|s| {
            let spid = Pid::from_u32(s.pid);
            let proc: Option<&sysinfo::Process> = sys.process(spid);
            json!({
                "port": s.port,
                "pid": s.pid,
                "name": proc.map(|p| p.name().to_string_lossy().into_owned()),
                "exe": proc.and_then(|p| p.exe()).map(|e| e.display().to_string()),
            })
        })
        .collect();

    if entries.is_empty() {
        anyhow::bail!("No process found listening on port {port}");
    }
    Ok(entries)
}

/// Returns all listening TCP sockets, deduplicated by (port, pid).
fn all_listening_ports() -> Result<Vec<Value>> {
    let raw = listening_sockets()?;
    let sys = new_system();

    let mut seen = std::collections::HashSet::new();
    let mut entries: Vec<Value> = raw
        .iter()
        .filter(|s| seen.insert((s.port, s.pid)))
        .map(|s| {
            let spid = Pid::from_u32(s.pid);
            let proc: Option<&sysinfo::Process> = sys.process(spid);
            json!({
                "port": s.port,
                "pid": s.pid,
                "name": proc.map(|p| p.name().to_string_lossy().into_owned()),
            })
        })
        .collect();
    entries.sort_by_key(|e| e["port"].as_u64().unwrap_or(0));
    Ok(entries)
}

// ── Kill ───────────────────────────────────────────────────────────────────

fn kill_process(port: Option<&u16>, pid: Option<&u32>, force: bool) -> Result<Value> {
    let target_pid = match (port, pid) {
        (Some(&p), _) => listening_sockets()?
            .into_iter()
            .find(|s| s.port == p)
            .map(|s| s.pid)
            .with_context(|| format!("No process found listening on port {p}"))?,
        (_, Some(&p)) => p,
        _ => unreachable!("clap enforces --port or --pid"),
    };

    let sys = new_system();
    let spid = Pid::from_u32(target_pid);
    let proc = sys
        .process(spid)
        .with_context(|| format!("No process found with PID {target_pid}"))?;

    let name = proc.name().to_string_lossy().into_owned();

    let (success, signal) = if force {
        (proc.kill(), "SIGKILL")
    } else {
        match proc.kill_with(Signal::Term) {
            Some(ok) => (ok, "SIGTERM"),
            None => (proc.kill(), "SIGKILL"),
        }
    };

    if !success {
        anyhow::bail!(
            "Failed to send {signal} to process {target_pid} ({name}). \
             You may need elevated privileges (try running with sudo)."
        );
    }

    Ok(json!({
        "pid": target_pid,
        "name": name,
        "signal": signal,
    }))
}

// ── Platform dispatch ──────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn listening_sockets() -> Result<Vec<ListeningSocket>> {
    linux_listening_sockets()
}

#[cfg(target_os = "macos")]
fn listening_sockets() -> Result<Vec<ListeningSocket>> {
    lsof_listening_sockets()
}

#[cfg(target_os = "windows")]
fn listening_sockets() -> Result<Vec<ListeningSocket>> {
    netstat_listening_sockets()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
fn listening_sockets() -> Result<Vec<ListeningSocket>> {
    anyhow::bail!("Port-to-process lookup is not supported on this platform")
}

// ── Linux: /proc/net/tcp ───────────────────────────────────────────────────

#[cfg(target_os = "linux")]
fn linux_listening_sockets() -> Result<Vec<ListeningSocket>> {
    let inode_pid = linux_build_inode_pid_map();
    let mut sockets = Vec::new();

    for path in ["/proc/net/tcp", "/proc/net/tcp6"] {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for line in content.lines().skip(1) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 10 {
                continue;
            }
            // State field: "0A" = TCP_LISTEN
            if fields[3] != "0A" {
                continue;
            }
            let port = linux_parse_hex_port(fields[1]);
            let inode: u64 = fields[9].parse().unwrap_or(0);
            if let Some(&pid) = inode_pid.get(&inode) {
                sockets.push(ListeningSocket { port, pid });
            }
        }
    }

    Ok(sockets)
}

/// Parses the port from a `/proc/net/tcp` local address field such as
/// `"0100007F:1F40"`. The port is encoded as big-endian hex.
#[cfg(target_os = "linux")]
fn linux_parse_hex_port(addr: &str) -> u16 {
    addr.split(':')
        .nth(1)
        .and_then(|p| u16::from_str_radix(p, 16).ok())
        .unwrap_or(0)
}

/// Builds a `socket inode → PID` map by scanning `/proc/<pid>/fd/` symlinks.
/// Entries for processes we lack permission to read are silently skipped.
#[cfg(target_os = "linux")]
fn linux_build_inode_pid_map() -> std::collections::HashMap<u64, u32> {
    let mut map = std::collections::HashMap::new();
    let Ok(proc_dir) = std::fs::read_dir("/proc") else {
        return map;
    };
    for entry in proc_dir.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let Ok(pid) = name_str.parse::<u32>() else {
            continue;
        };
        let Ok(fd_dir) = std::fs::read_dir(format!("/proc/{pid}/fd")) else {
            continue;
        };
        for fd in fd_dir.flatten() {
            let Ok(target) = std::fs::read_link(fd.path()) else {
                continue;
            };
            let t = target.to_string_lossy();
            if let Some(inode_str) = t
                .strip_prefix("socket:[")
                .and_then(|s| s.strip_suffix(']'))
            {
                if let Ok(inode) = inode_str.parse::<u64>() {
                    map.insert(inode, pid);
                }
            }
        }
    }
    map
}

// ── macOS: lsof ───────────────────────────────────────────────────────────

/// Parses `lsof -nP -iTCP -sTCP:LISTEN` output to get listening sockets.
///
/// Output columns: COMMAND PID USER FD TYPE DEVICE SIZE/OFF NODE NAME
/// The NAME field contains the address:port, e.g. `*:3000` or `[::1]:8080`.
#[cfg(target_os = "macos")]
fn lsof_listening_sockets() -> Result<Vec<ListeningSocket>> {
    let output = std::process::Command::new("lsof")
        .args(["-nP", "-iTCP", "-sTCP:LISTEN"])
        .output()
        .context("`lsof` is required for port lookup on macOS but could not be executed")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sockets = Vec::new();

    for line in stdout.lines().skip(1) {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 9 {
            continue;
        }
        let Ok(pid) = fields[1].parse::<u32>() else {
            continue;
        };
        // NAME field: "*:3000", "127.0.0.1:3000", or "[::1]:3000"
        // rsplit on ':' gives the port as the rightmost segment.
        let port = fields[8]
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(0);
        if port > 0 {
            sockets.push(ListeningSocket { port, pid });
        }
    }

    Ok(sockets)
}

// ── Windows: netstat ───────────────────────────────────────────────────────

/// Parses `netstat -ano` output to get listening TCP sockets.
///
/// Output columns: Proto  LocalAddress  ForeignAddress  State  PID
#[cfg(target_os = "windows")]
fn netstat_listening_sockets() -> Result<Vec<ListeningSocket>> {
    let output = std::process::Command::new("netstat")
        .args(["-ano"])
        .output()
        .context("`netstat` is required for port lookup on Windows but could not be executed")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut sockets = Vec::new();

    for line in stdout.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        // TCP  0.0.0.0:3000  0.0.0.0:0  LISTENING  1234
        if fields.len() < 5 || fields[0] != "TCP" || fields[3] != "LISTENING" {
            continue;
        }
        let Ok(pid) = fields[4].parse::<u32>() else {
            continue;
        };
        let port = fields[1]
            .rsplit(':')
            .next()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(0);
        if port > 0 {
            sockets.push(ListeningSocket { port, pid });
        }
    }

    Ok(sockets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_processes_non_empty() {
        let procs = list_processes(None, false).unwrap();
        assert!(!procs.is_empty(), "at least one process must be running");
    }

    #[test]
    fn test_list_processes_fields_present() {
        let procs = list_processes(None, false).unwrap();
        for p in &procs {
            assert!(p["pid"].is_number(), "every process entry must have a numeric pid");
            assert!(p["name"].is_string(), "every process entry must have a string name");
            assert!(p["memory_bytes"].is_number(), "every entry must have memory_bytes");
        }
    }

    #[test]
    fn test_list_processes_sorted_by_pid() {
        let procs = list_processes(None, false).unwrap();
        let pids: Vec<u64> = procs.iter().filter_map(|p| p["pid"].as_u64()).collect();
        let mut sorted = pids.clone();
        sorted.sort();
        assert_eq!(pids, sorted, "processes should be sorted by PID");
    }

    #[test]
    fn test_list_processes_name_filter_is_subset() {
        let all = list_processes(None, false).unwrap();
        let filtered = list_processes(Some("zzz_unlikely_process_name_xyz"), false).unwrap();
        assert!(
            filtered.len() <= all.len(),
            "filtered list must not exceed total process count"
        );
    }

    #[test]
    fn test_list_processes_cmd_flag_adds_column() {
        // Test each call independently — comparing counts across two separate
        // system snapshots is a race condition since processes can appear or
        // disappear between calls.
        let with_cmd = list_processes(None, true).unwrap();
        assert!(!with_cmd.is_empty());
        for p in &with_cmd {
            assert!(p["cmd"].is_string(), "every entry must have a cmd string when --cmd is set");
        }

        let without = list_processes(None, false).unwrap();
        assert!(!without.is_empty());
        for p in &without {
            assert!(p.get("cmd").is_none(), "cmd column must be absent without --cmd");
        }
    }

    #[test]
    fn test_process_by_pid_current_process() {
        let pid = std::process::id();
        let result = process_by_pid(pid);
        assert!(result.is_ok(), "should find the current test process by PID");
        let info = result.unwrap();
        assert_eq!(info["pid"], pid);
        assert!(info["name"].is_string());
        assert!(info["exe"].is_string() || info["exe"].is_null());
    }

    #[test]
    fn test_process_by_pid_not_found() {
        let result = process_by_pid(999_999_999);
        assert!(result.is_err(), "should return an error for a non-existent PID");
    }

    #[test]
    fn test_find_port_owners_unknown_port_errors() {
        let result = find_port_owners(9);
        assert!(result.is_err(), "port 9 (discard) should not be in use");
    }

    #[test]
    fn test_all_listening_ports_no_duplicates() {
        let entries = all_listening_ports().unwrap_or_default();
        let mut seen = std::collections::HashSet::new();
        for e in &entries {
            let key = (e["port"].as_u64(), e["pid"].as_u64());
            assert!(seen.insert(key), "ports list must not contain duplicate (port, pid) pairs");
        }
    }
}
