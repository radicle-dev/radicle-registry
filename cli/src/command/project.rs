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
#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    List(List),
    Register(Register),
    Show(Show),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        match self {
            Command::List(cmd) => cmd.run(command_context).await,
            Command::Register(cmd) => cmd.run(command_context).await,
            Command::Show(cmd) => cmd.run(command_context).await,
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Show information for a registered project.
pub struct Show {
    /// The name of the project
    project_name: ProjectName,
    /// The org in which the project is registered.
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for Show {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let project = command_context
            .client
            .get_project(self.project_name.clone(), self.org_id.clone())
            .await?
            .ok_or(CommandError::ProjectNotFound {
                project_name: self.project_name.clone(),
                org_id: self.org_id.clone(),
            })?;
        println!("project: {}.{}", project.name, project.org_id);
        println!("checkpoint: {}", project.current_cp);
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// List all projects in the registry
pub struct List {}

#[async_trait::async_trait]
impl CommandT for List {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let project_ids = command_context.client.list_projects().await?;
        println!("PROJECTS ({})", project_ids.len());
        for (name, org) in project_ids {
            println!("{}.{}", name, org)
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Register a project with the given name under the given org.
pub struct Register {
    /// Name of the project to register.
    project_name: ProjectName,
    /// Org under which to register the project.
    org_id: OrgId,
    /// Project state hash. A hex-encoded 32 byte string. Defaults to all zeros.
    project_hash: Option<H256>,
}

#[async_trait::async_trait]
impl CommandT for Register {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let create_checkpoint_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::CreateCheckpoint {
                    project_hash: self.project_hash.unwrap_or_default(),
                    previous_checkpoint_id: None,
                },
                command_context.fee,
            )
            .await?;
        println!("creating checkpoint...");

        let checkpoint_created = create_checkpoint_fut.await?;
        let checkpoint_id = transaction_applied_ok(&checkpoint_created)?;
        println!("checkpoint created in block {}", checkpoint_created.block);

        let register_project_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::RegisterProject {
                    project_name: self.project_name.clone(),
                    org_id: self.org_id.clone(),
                    checkpoint_id,
                    metadata: Bytes128::random(),
                },
                command_context.fee,
            )
            .await?;
        println!("registering project...");
        let project_registered = register_project_fut.await?;
        transaction_applied_ok(&project_registered)?;
        println!(
            "project {}.{} registered in block {}",
            self.project_name, self.org_id, project_registered.block,
        );
        Ok(())
    }
}
