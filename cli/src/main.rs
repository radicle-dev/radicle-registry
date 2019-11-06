use radicle_registry_client::{ed25519, CryptoPair as _, RegisterProjectParams, SyncClient, H256};
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
        /// Domain of the project to register.
        domain: String,
        project_hash: H256,
    },
}

fn main() {
    pretty_env_logger::init();
    run(Args::from_args());
}

fn run(args: Args) {
    let author_key_pair = args.author_key_pair();

    match args.command {
        Command::RegisterProject {
            name,
            domain,
            project_hash,
        } => {
            let client = SyncClient::create().unwrap();
            let checkpoint_id = client
                .create_checkpoint(&author_key_pair, project_hash, None)
                .unwrap();
            let project_id = (name, domain);
            client
                .register_project(
                    &author_key_pair,
                    RegisterProjectParams {
                        id: project_id.clone(),
                        description: format!(""),
                        img_url: format!(""),
                        checkpoint_id,
                    },
                )
                .unwrap();
            println!("Registered project with ID {:?}", project_id)
        }
    }
}
