use futures01::prelude::*;
use radicle_registry_client::{
    sr25519, Client, ClientT, CryptoPair as _, RegisterProjectParams, H256,
};
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
    fn author_key_pair(&self) -> sr25519::Pair {
        sr25519::Pair::from_string(format!("//{}", self.author_key_seed).as_ref(), None).unwrap()
    }
}

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    /// Register a project
    RegisterProject {
        /// Name of the project to register.
        name: Vec<u8>,
        /// Domain of the project to register.
        domain: Vec<u8>,
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
            let client = Client::create_with_executor().unwrap();
            let checkpoint_id = client
                .create_checkpoint(&author_key_pair, project_hash, None)
                .wait()
                .unwrap();
            let project_id = (name, domain);
            client
                .register_project(
                    &author_key_pair,
                    RegisterProjectParams {
                        id: project_id.clone(),
                        description: vec![],
                        img_url: vec![],
                        checkpoint_id,
                    },
                )
                .wait()
                .unwrap();
            println!("Registered project with ID {:?}", project_id);
        }
    }
}
