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

//! Defines [CommandT] trait, structs for all commands and their [CommandT] implementations.
use futures01::prelude::*;
use radicle_registry_client::*;
use std::str::FromStr;
use structopt::StructOpt;

lazy_static::lazy_static! {
    static ref RAD_DOMAIN: String32 =
        String32::from_str("rad").expect("statically valid");
}

/// Contextual data for running commands. Created from command line options.
pub struct CommandContext {
    pub author_key_pair: ed25519::Pair,
    pub client: Client,
}

/// Every CLI command must implement this trait.
pub trait CommandT {
    fn run(&self, command_context: &CommandContext) -> Result<(), ()>;
}

#[derive(StructOpt, Debug, Clone)]
/// Show information for a registered project in the .rad domain.
pub struct ShowProject {
    project_name: String,
}

impl CommandT for ShowProject {
    fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let project_name: String32 = match self.project_name.parse() {
            Ok(project_name) => project_name,
            Err(error) => {
                println!("Invalid project name: {}", error);
                return Err(());
            }
        };
        let opt_project = command_context
            .client
            .get_project((project_name, RAD_DOMAIN.clone()))
            .wait()
            .unwrap();

        match opt_project {
            None => {
                println!("Project not found");
                Err(())
            }
            Some(project) => {
                println!("project: {}.{}", project.id.0, project.id.1);
                println!("account id: {}", project.account_id);
                println!("checkpoint: {}", project.current_cp);
                println!("members: {:?}", project.members);
                Ok(())
            }
        }
    }
}
#[derive(StructOpt, Debug, Clone)]
/// List all projects in the registry
pub struct ListProjects {}

impl CommandT for ListProjects {
    fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let project_ids = command_context.client.list_projects().wait().unwrap();
        println!("PROJECTS");
        for (name, domain) in project_ids {
            println!("{}.{}", name, domain)
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Register a project under the default "rad" domain.
pub struct RegisterProject {
    /// Name of the project to register.
    name: String32,
    /// Project state hash. A hex-encoded 32 byte string. Defaults to all zeros.
    project_hash: Option<H256>,
}

impl CommandT for RegisterProject {
    fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let client = &command_context.client;

        let create_checkpoint_fut = client
            .sign_and_submit_call(
                &command_context.author_key_pair,
                CreateCheckpointParams {
                    project_hash: self.project_hash.unwrap_or_default(),
                    previous_checkpoint_id: None,
                },
            )
            .wait()
            .unwrap();
        println!("creating checkpoint...");

        let checkpoint_created = create_checkpoint_fut.wait().unwrap();
        let checkpoint_id = checkpoint_created.result.unwrap();
        println!("checkpoint created in block {}", checkpoint_created.block,);

        let domain = String32::from_str("rad").expect("statically valid");
        let project_id = (self.name.clone(), domain.clone());
        let register_project_fut = client
            .sign_and_submit_call(
                &command_context.author_key_pair,
                RegisterProjectParams {
                    id: project_id,
                    description: format!(""),
                    img_url: format!(""),
                    checkpoint_id,
                },
            )
            .wait()
            .unwrap();
        println!("registering project...");
        let project_registered = register_project_fut.wait().unwrap();
        match project_registered.result {
            Ok(()) => {
                println!(
                    "project {}.{} registered in block {}",
                    self.name, domain, project_registered.block,
                );
                Ok(())
            }
            Err(_) => {
                println!("transaction failed in block {}", project_registered.block,);
                Err(())
            }
        }
    }
}
