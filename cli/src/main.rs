// Radicle Registry
// Copyright (C) 2019 Monadic GmbH <radicle@monadic.xyz>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use futures01::prelude::*;
use radicle_registry_client::*;
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
        name: String32,
        /// Domain of the project to register.
        domain: String32,
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
                .submit(
                    &author_key_pair,
                    CreateCheckpointParams {
                        project_hash,
                        previous_checkpoint_id: None,
                    },
                )
                .wait()
                .unwrap()
                .result
                .unwrap();
            let project_id = (name, domain);
            client
                .submit(
                    &author_key_pair,
                    RegisterProjectParams {
                        id: project_id.clone(),
                        description: format!(""),
                        img_url: format!(""),
                        checkpoint_id,
                    },
                )
                .wait()
                .unwrap();
            println!("Registered project with ID {:?}", project_id)
        }
    }
}
