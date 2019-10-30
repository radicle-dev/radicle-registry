use radicle_registry_client::{ed25519, Pair as _, RegisterProjectParams, SyncClient};
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
pub struct Args {
    #[structopt(long, default_value = "Alice")]
    /// The key pair that is used to sign transaction is generated from this seed.
    author_key_seed: String,
    #[structopt(subcommand)]
    command: Command,
}

impl Args {
    /// Return the key pair generated from [Args::author_key_seed].
    fn author_key_pair(&self) -> ed25519::Pair {
        ed25519::Pair::from_string(format!("//{}", self.author_key_seed).as_ref(), None).unwrap()
    }
}

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    /// Register a project
    RegisterProject {
        /// Name of the project to register.
        name: String,
    },
}

fn main() {
    pretty_env_logger::init();
    run(Args::from_args());
}

fn run(args: Args) {
    let author_key_pair = args.author_key_pair();

    match args.command {
        Command::RegisterProject { name } => {
            let client = SyncClient::create().unwrap();
            let project_id = client
                .register_project(
                    &author_key_pair,
                    RegisterProjectParams {
                        name,
                        description: format!(""),
                        img_url: format!(""),
                    },
                )
                .unwrap();
            println!("Registered project with ID {:#x}", project_id)
        }
    }
}
