use clap::Parser;
use saros_dlmm_sdk::amms::test_harness::take_snapshot;

#[derive(Parser, Debug)]
pub struct ConfigOverride {
    #[clap(long)]
    pub rpc_url: Option<String>,
}

#[derive(Parser, Debug)]
pub enum Command {
    /// Snapshot a single amm for test harness testing
    SnapshotAmm {
        #[clap(long)]
        amm_id: Option<String>,
        /// Expand an extra option to the snapshot directory (e.g. <amm-id><option>)
        #[clap(long)]
        option: Option<String>,
        /// Overwrite the output snapshot if it exists
        #[clap(short, long)]
        force: bool,
    },
}

#[derive(Parser, Debug)]
pub struct Cli {
    #[clap(flatten)]
    pub config_override: ConfigOverride,
    #[clap(subcommand)]
    pub command: Command,
}

#[tokio::main]
async fn main() {
    let Cli {
        config_override,
        command,
    } = Cli::parse();

    let pool_list = saros_config::POOL_LISTS;
    let overrided_rpc_url = saros_config::RPC_URL.to_string();

    match command {
        Command::SnapshotAmm { option, force, .. } => {
            for (i, pool) in pool_list.iter().enumerate() {
                let amm_id = pool.to_string();

                println!("📸 Snapshotting amm {amm_id} ...");

                // take_snapshot(rpc_url: String, amm_id: String, option: Option<String>, force: bool)
                take_snapshot(
                    overrided_rpc_url.clone(),
                    amm_id.clone(),
                    option.clone(),
                    force,
                )
                .await
                .unwrap();
            }
        }
    }
}
