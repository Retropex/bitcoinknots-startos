use std::convert::TryFrom;
use std::env::var;
use std::error::Error;
use std::os::unix::prelude::ExitStatusExt;
use std::sync::Arc;
use std::time::Duration;
use std::{borrow::Cow, sync::Mutex};
use std::{fs, io::Write, path::Path};

use btc_rpc_proxy::{Peers, RpcClient, TorState};
use env_logger::Env;
use heck::TitleCase;
use linear_map::LinearMap;
use nix::sys::signal::Signal;
use serde_yaml::{Mapping, Value};
use tmpl::TemplatingReader;

lazy_static::lazy_static! {
    static ref CHILD_PID: Mutex<Option<u32>> = Mutex::new(None);
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ChainInfo {
    blocks: usize,
    headers: usize,
    verificationprogress: f64,
    size_on_disk: u64,
    #[serde(default)]
    pruneheight: usize,
    #[serde(default)]
    softforks: LinearMap<String, SoftFork>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct NetworkInfo {
    connections: usize,
    connections_in: usize,
    connections_out: usize,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
pub enum SoftFork {
    #[serde(rename = "buried")]
    Buried { active: bool, height: usize },
    #[serde(rename = "bip9")]
    Bip9 { active: bool, bip9: Bip9 },
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "status")]
pub enum Bip9 {
    #[serde(rename = "defined")]
    Defined {
        start_time: u64,
        timeout: u64,
        since: usize,
    },
    #[serde(rename = "started")]
    Started {
        bit: usize,
        start_time: u64,
        timeout: u64,
        since: usize,
        statistics: Bip9Stats,
    },
    #[serde(rename = "locked_in")]
    LockedIn {
        start_time: u64,
        timeout: u64,
        since: usize,
    },
    #[serde(rename = "active")]
    Active {
        start_time: u64,
        timeout: u64,
        since: usize,
    },
    #[serde(rename = "failed")]
    Failed {
        start_time: u64,
        timeout: u64,
        since: usize,
    },
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Bip9Stats {
    pub period: usize,
    pub threshold: usize,
    pub elapsed: usize,
    pub count: usize,
    pub possible: bool,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Stats {
    version: u8,
    data: LinearMap<Cow<'static, str>, Stat>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct Stat {
    #[serde(rename = "type")]
    value_type: &'static str,
    value: String,
    description: Option<Cow<'static, str>>,
    copyable: bool,
    qr: bool,
    masked: bool,
}

fn sidecar(config: &Mapping, addr: &str) -> Result<(), Box<dyn Error>> {
    let mut stats = LinearMap::new();
    if let (Some(user), Some(pass)) = (
        config
            .get(&Value::String("rpc".to_owned()))
            .and_then(|v| v.get(&Value::String("username".to_owned())))
            .and_then(|v| v.as_str()),
        config
            .get(&Value::String("rpc".to_owned()))
            .and_then(|v| v.get(&Value::String("password".to_owned())))
            .and_then(|v| v.as_str()),
    ) {
        stats.insert(
            Cow::from("Tor Quick Connect"),
            Stat {
                value_type: "string",
                value: format!("btcstandup://{}:{}@{}:8332", user, pass, addr),
                description: Some(Cow::from("Bitcoin-Standup Tor Quick Connect URL")),
                copyable: true,
                qr: true,
                masked: true,
            },
        );
        let addr_local = format!("{}local", addr.strip_suffix("onion").unwrap());
        stats.insert(
            Cow::from("LAN Quick Connect"),
            Stat {
                value_type: "string",
                value: format!("btcstandup://{}:{}@{}:8332", user, pass, addr_local),
                description: Some(Cow::from("Bitcoin-Standup LAN Quick Connect URL")),
                copyable: true,
                qr: true,
                masked: true,
            },
        );
        stats.insert(
            Cow::from("RPC Username"),
            Stat {
                value_type: "string",
                value: format!("{}", user),
                description: Some(Cow::from("Bitcoin RPC Username")),
                copyable: true,
                masked: false,
                qr: false,
            },
        );
        stats.insert(
            Cow::from("RPC Password"),
            Stat {
                value_type: "string",
                value: format!("{}", pass),
                description: Some(Cow::from("Bitcoin RPC Password")),
                copyable: true,
                masked: true,
                qr: false,
            },
        );
    }
    // New section to calculate halving progress
    let blockchain_info = std::process::Command::new("bitcoin-cli")
    .arg("-conf=/root/.bitcoin/bitcoin.conf")
    .arg("getblockchaininfo")
    .output()?;

    if blockchain_info.status.success() {
        let blockchain_data: serde_json::Value = serde_json::from_slice(&blockchain_info.stdout)?;
        let current_height = blockchain_data["blocks"].as_u64().unwrap_or(0);

        // Calculate the last halving block and progress towards the next halving
        let halving_interval = 210_000;
        let last_halving_block = (current_height / halving_interval) * halving_interval;
        let blocks_since_last_halving = current_height - last_halving_block;
        let progress_to_next_halving = (blocks_since_last_halving as f64 / halving_interval as f64) * 100.0;

        stats.insert(
            Cow::from("Progress to Next Halving"),
            Stat {
                value_type: "string",
                value: format!("{:.2}%", progress_to_next_halving),
                description: Some(Cow::from("Percentage of blocks completed until the next Bitcoin halving")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );
    }


    // New section to fetch mempool statistics
    let mempool_info = std::process::Command::new("bitcoin-cli")
        .arg("-conf=/root/.bitcoin/bitcoin.conf")
        .arg("getmempoolinfo")
        .output()?;

    if mempool_info.status.success() {
        let mempool_data: serde_json::Value = serde_json::from_slice(&mempool_info.stdout)?;

        let max_mempool = mempool_data["maxmempool"].as_u64().unwrap_or(0) as f64 / 1024_f64.powf(2.0); // Convert bytes to MB
        let mempool_usage = mempool_data["usage"].as_u64().unwrap_or(0) as f64 / 1024_f64.powf(2.0); // Convert bytes to MB
        let mempool_percent = if max_mempool > 0.0 {
            (mempool_usage / max_mempool) * 100.0
        } else {
            0.0
        };
        let tx_count = mempool_data["size"].as_u64().unwrap_or(0); // Number of transactions

        stats.insert(
            Cow::from("Max Mempool Size"),
            Stat {
                value_type: "string",
                value: format!("{:.2} MB", max_mempool),
                description: Some(Cow::from("Maximum memory pool size")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );

        stats.insert(
            Cow::from("Current Mempool Usage"),
            Stat {
                value_type: "string",
                value: format!("{:.2} MB ({:.2}%)", mempool_usage, mempool_percent),
                description: Some(Cow::from("Current memory pool usage as a percentage of max size")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );

        stats.insert(
            Cow::from("Mempool Transaction Count"),
            Stat {
                value_type: "string",
                value: format!("{}", tx_count),
                description: Some(Cow::from("Current number of transactions in the mempool")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );
    } else {
        eprintln!(
            "Error retrieving mempool info: {}",
            std::str::from_utf8(&mempool_info.stderr).unwrap_or("UNKNOWN ERROR")
        );
    }

    // Existing code for blockchain and network info retrieval continues here...
    let info_res = std::process::Command::new("bitcoin-cli")
        .arg("-conf=/root/.bitcoin/bitcoin.conf")
        .arg("getblockchaininfo")
        .output()?;
    if info_res.status.success() {
        let info: ChainInfo = serde_json::from_slice(&info_res.stdout)?;
        stats.insert(
            Cow::from("Block Height"),
            Stat {
                value_type: "string",
                value: format!("{}", info.headers),
                description: Some(Cow::from("The current block height for the network")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );
        stats.insert(
            Cow::from("Synced Block Height"),
            Stat {
                value_type: "string",
                value: format!("{}", info.blocks),
                description: Some(Cow::from("The number of blocks the node has verified")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );
        stats.insert(
            Cow::from("Sync Progress"),
            Stat {
                value_type: "string",
                value: if info.blocks < info.headers {
                    format!("{:.2}%", 100.0 * info.verificationprogress)
                } else {
                    "100%".to_owned()
                },
                description: Some(Cow::from(
                    "The percentage of the blockchain that has been verified",
                )),
                copyable: false,
                qr: false,
                masked: false,
            },
        );
        for (sf_name, sf_data) in info.softforks {
            let sf_name_pretty = sf_name.to_title_case();
            let status_desc = Some(Cow::from(format!(
                "The Bip9 deployment status for {}",
                sf_name_pretty
            )));
            let start_desc = Some(Cow::from(format!(
                "The start time (UTC) of the Bip9 signaling period for {}",
                sf_name_pretty
            )));
            let timeout_desc = Some(Cow::from(format!(
                "The timeout time (UTC) of the Bip9 signaling period for {}",
                sf_name_pretty
            )));
            match sf_data {
                SoftFork::Buried {
                    active: _,
                    height: _,
                } => continue,
                SoftFork::Bip9 { bip9, active: _ } => {
                    let (status, start, end, _since) = match bip9 {
                        Bip9::Defined {
                            start_time,
                            timeout,
                            since,
                        } => {
                            let start_time_pretty = human_readable_timestamp(start_time);
                            let end_time_pretty = human_readable_timestamp(timeout);
                            ("Defined", start_time_pretty, end_time_pretty, since)
                        }
                        Bip9::Started {
                            start_time,
                            timeout,
                            since,
                            bit: _,
                            statistics: _,
                        } => {
                            let start_time_pretty = human_readable_timestamp(start_time);
                            let end_time_pretty = human_readable_timestamp(timeout);
                            ("Started", start_time_pretty, end_time_pretty, since)
                        }
                        Bip9::LockedIn {
                            start_time,
                            timeout,
                            since,
                        } => {
                            let start_time_pretty = human_readable_timestamp(start_time);
                            let end_time_pretty = human_readable_timestamp(timeout);
                            ("Locked In", start_time_pretty, end_time_pretty, since)
                        }
                        Bip9::Active {
                            start_time,
                            timeout,
                            since,
                        } => {
                            // stop showing soft fork info when it's been active for ~12 weeks
                            if info.blocks >= since + 12096 {
                                continue;
                            }
                            let start_time_pretty = human_readable_timestamp(start_time);
                            let end_time_pretty = human_readable_timestamp(timeout);
                            ("Active", start_time_pretty, end_time_pretty, since)
                        }
                        Bip9::Failed {
                            start_time,
                            timeout,
                            since,
                        } => {
                            let start_time_pretty = human_readable_timestamp(start_time);
                            let end_time_pretty = human_readable_timestamp(timeout);
                            ("Active", start_time_pretty, end_time_pretty, since)
                        }
                    };
                    stats.insert(
                        Cow::from(format!("{} Status", sf_name_pretty)),
                        Stat {
                            value_type: "string",
                            value: status.to_owned(),
                            description: status_desc,
                            copyable: false,
                            qr: false,
                            masked: false,
                        },
                    );
                    stats.insert(
                        Cow::from(format!("{} Start Time", sf_name_pretty)),
                        Stat {
                            value_type: "string",
                            value: start,
                            description: start_desc,
                            copyable: false,
                            qr: false,
                            masked: false,
                        },
                    );
                    stats.insert(
                        Cow::from(format!("{} Timeout", sf_name_pretty)),
                        Stat {
                            value_type: "string",
                            value: end,
                            description: timeout_desc,
                            copyable: false,
                            qr: false,
                            masked: false,
                        },
                    );
                    if let Bip9::Started {
                        statistics,
                        start_time: _,
                        timeout: _,
                        since: _,
                        bit: _,
                    } = bip9
                    {
                        stats.insert(
                            Cow::from(format!("{} Signal Percentage", sf_name_pretty)),
                            Stat {
                                value_type: "string",
                                value: format!(
                                    "{:.2}%",
                                    100.0 * (statistics.count as f64) / (statistics.elapsed as f64)
                                ),
                                description: Some(Cow::from(format!("Percentage of the blocks in the current signaling window that are signaling for the activation of {}", sf_name_pretty))),
                                copyable: false,
                                qr: false,
                                masked: false,
                            },
                        );
                    }
                }
            }
        }
        stats.insert(
            Cow::from("Disk Usage"),
            Stat {
                value_type: "string",
                value: format!("{:.2} GiB", info.size_on_disk as f64 / 1024_f64.powf(3_f64)),
                description: Some(Cow::from("The blockchain size on disk")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );
        if info.pruneheight > 0 {
            stats.insert(
                Cow::from("Prune Height"),
                Stat {
                    value_type: "string",
                    value: format!("{}", info.pruneheight),
                    description: Some(Cow::from(
                        "The number of blocks that have been deleted from disk",
                    )),
                    copyable: false,
                    qr: false,
                    masked: false,
                },
            );
        }
    } else if info_res.status.code() == Some(28) {
        return Ok(());
    } else {
        eprintln!(
            "Error updating blockchain info: {}",
            std::str::from_utf8(&info_res.stderr).unwrap_or("UNKNOWN ERROR")
        );
    }
    let info_res = std::process::Command::new("bitcoin-cli")
        .arg("-conf=/root/.bitcoin/bitcoin.conf")
        .arg("getnetworkinfo")
        .output()?;
    if info_res.status.success() {
        let info: NetworkInfo = serde_json::from_slice(&info_res.stdout)?;
        stats.insert(
            Cow::from("Connections"),
            Stat {
                value_type: "string",
                value: format!("{} ({} in / {} out)", info.connections, info.connections_in, info.connections_out),
                description: Some(Cow::from("The number of peers connected (inbound and outbound)")),
                copyable: false,
                qr: false,
                masked: false,
            },
        );
    } else if info_res.status.code() == Some(28) {
        return Ok(());
    } else {
        eprintln!(
            "Error updating network info: {}",
            std::str::from_utf8(&info_res.stderr).unwrap_or("UNKNOWN ERROR")
        );
    }
    serde_yaml::to_writer(
        std::fs::File::create("/root/.bitcoin/start9/.stats.yaml.tmp")?,
        &Stats {
            version: 2,
            data: stats,
        },
    )?;
    std::fs::rename(
        "/root/.bitcoin/start9/.stats.yaml.tmp",
        "/root/.bitcoin/start9/stats.yaml",
    )?;
    Ok(())
}

fn inner_main(reindex: bool) -> Result<(), Box<dyn Error>> {
    while !Path::new("/root/.bitcoin/start9/config.yaml").exists() {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    let config: Mapping =
        serde_yaml::from_reader(std::fs::File::open("/root/.bitcoin/start9/config.yaml")?)?;
    let sidecar_poll_interval = std::time::Duration::from_secs(5);
    let peer_addr = var("PEER_TOR_ADDRESS")?;
    let rpc_addr = var("RPC_TOR_ADDRESS")?;
    let mut btc_args = vec![
        format!("-onion={}:9050", var("EMBASSY_IP")?),
        format!("-externalip={}", peer_addr),
        "-datadir=/root/.bitcoin".to_owned(),
        "-conf=/root/.bitcoin/bitcoin.conf".to_owned(),
    ];
    if config
        .get(&Value::String("advanced".to_owned()))
        .and_then(|v| v.as_mapping())
        .and_then(|v| v.get(&Value::String("peers".to_owned())))
        .and_then(|v| v.as_mapping())
        .and_then(|v| v.get(&Value::String("onlyonion".to_owned())))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        btc_args.push(format!("-proxy={}:9050", var("EMBASSY_IP")?));
    }
    {
        // disable chain data backup
        let mut f = std::fs::File::create("/root/.bitcoin/.backupignore")?;
        writeln!(f, "blocks/")?;
        writeln!(f, "chainstate/")?;
        writeln!(f, "indexes/")?;
        writeln!(f, "testnet3/")?;
        f.flush()?;
    }
    if reindex {
        btc_args.push("-reindex".to_owned());
    }

    std::io::copy(
        &mut TemplatingReader::new(
            std::fs::File::open("/mnt/assets/bitcoin.conf.template")?,
            &config,
            &"{{var}}".parse()?,
            b'%',
        ),
        &mut std::fs::File::create("/root/.bitcoin/bitcoin.conf")?,
    )?;
    let mut child = std::process::Command::new("bitcoind")
        .args(btc_args)
        .spawn()?;
    if reindex {
        match fs::remove_file("/root/.bitcoin/requires.reindex") {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            a => a?,
        }
    }
    let raw_child = child.id();
    *CHILD_PID.lock().unwrap() = Some(raw_child);
    let pruned = {
        config[&Value::from("advanced")][&Value::from("pruning")][&Value::from("mode")]
            == "automatic"
    };
    let _proxy = if pruned {
        let state = Arc::new(btc_rpc_proxy::State {
            rpc_client: RpcClient::new("http://127.0.0.1:18332/".parse().unwrap()),
            tor: Some(TorState {
                proxy: format!("{}:9050", var("EMBASSY_IP")?).parse()?,
                only: config[&Value::from("advanced")][&Value::from("peers")]
                    [&Value::from("onlyonion")]
                    .as_bool()
                    .unwrap(),
            }),
            peer_timeout: Duration::from_secs(30),
            peers: tokio::sync::RwLock::new(Arc::new(Peers::new())),
            max_peer_age: Duration::from_secs(300),
            max_peer_concurrency: Some(1),
        });
        Some(std::thread::spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(btc_rpc_proxy::main(state, ([0, 0, 0, 0], 8332).into()))
                .unwrap();
        }))
    } else {
        None
    };
    let _sidecar_handle = std::thread::spawn(move || loop {
        sidecar(&config, &rpc_addr)
            .err()
            .map(|e| eprintln!("ERROR IN SIDECAR: {}", e));
        std::thread::sleep(sidecar_poll_interval);
    });
    let child_res = child.wait()?;
    let code = if let Some(code) = child_res.code() {
        code
    } else if let Some(signal) = child_res.signal() {
        eprintln!(
            "PROCESS TERMINATED BY {}",
            Signal::try_from(signal)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| "UNKNOWN SIGNAL".to_owned())
        );
        128 + signal
    } else {
        1
    };

    std::process::exit(code)
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    let reindex = Path::new("/root/.bitcoin/requires.reindex").exists();
    ctrlc::set_handler(move || {
        if let Some(raw_child) = *CHILD_PID.lock().unwrap() {
            use nix::{
                sys::signal::{kill, SIGTERM},
                unistd::Pid,
            };
            kill(Pid::from_raw(raw_child as i32), SIGTERM).unwrap();
        } else {
            std::process::exit(143)
        }
    })?;
    inner_main(reindex)
}

fn human_readable_timestamp(unix_time: u64) -> String {
    chrono::DateTime::<chrono::Utc>::from(
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(unix_time),
    )
    .format("%m/%d/%Y @ %H:%M:%S")
    .to_string()
}
