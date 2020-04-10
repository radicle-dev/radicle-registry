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

//! Define the commands supported by the CLI related to Projects.

use super::*;

/// Project related commands
#[derive(StructOpt, Clone)]
pub enum Command {
    /// List all projects in the registry
    List(List),
    /// Register a project with the given name under the given org.
    Register(Register),
    /// Show information for a registered project.
    Show(Show),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            Command::List(cmd) => cmd.run().await,
            Command::Register(cmd) => cmd.run().await,
            Command::Show(cmd) => cmd.run().await,
        }
    }
}

#[derive(StructOpt, Clone)]
pub struct Show {
    /// The name of the project
    project_name: ProjectName,

    /// The org in which the project is registered.
    org_id: OrgId,

    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for Show {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;

        let project = client
            .get_project(self.project_name.clone(), self.org_id.clone())
            .await?
            .ok_or(CommandError::ProjectNotFound {
                project_name: self.project_name.clone(),
                org_id: self.org_id.clone(),
            })?;
        println!("Project: {}.{}", project.name, project.org_id);
        println!("Checkpoint: {}", project.current_cp);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct List {
    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for List {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let project_ids = client.list_projects().await?;
        println!("PROJECTS ({})", project_ids.len());
        for (name, org) in project_ids {
            println!("{}.{}", name, org)
        }
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Register {
    /// Name of the project to register.
    project_name: ProjectName,
    /// Org under which to register the project.
    org_id: OrgId,
    /// Project state hash. A hex-encoded 32 byte string. Defaults to all zeros.
    project_hash: Option<H256>,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for Register {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let create_checkpoint_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::CreateCheckpoint {
                    project_hash: self.project_hash.unwrap_or_default(),
                    previous_checkpoint_id: None,
                },
                self.tx_options.fee,
            )
            .await?;
        println!("Creating checkpoint...");

        let checkpoint_created = create_checkpoint_fut.await?;
        let checkpoint_id = transaction_applied_ok(&checkpoint_created)?;
        println!("✓ Checkpoint created in block {}", checkpoint_created.block);

        let register_project_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::RegisterProject {
                    project_name: self.project_name.clone(),
                    org_id: self.org_id.clone(),
                    checkpoint_id,
                    metadata: Bytes128::random(),
                },
                self.tx_options.fee,
            )
            .await?;
        println!("Registering project...");
        let project_registered = register_project_fut.await?;
        transaction_applied_ok(&project_registered)?;
        println!(
            "✓ Project {}.{} registered in block {}",
            self.project_name, self.org_id, project_registered.block,
        );
        Ok(())
    }
}
