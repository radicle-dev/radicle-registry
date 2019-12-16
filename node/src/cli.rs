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

use crate::chain_spec;
use crate::service;
use aura_primitives::sr25519::AuthorityPair as AuraPair;
use futures::{
    channel::oneshot,
    compat::Future01CompatExt,
    future::{select, Map},
    FutureExt, TryFutureExt,
};
use log::info;
pub use sc_cli::{error, IntoExit, VersionInfo};
use sc_cli::{informant, parse_and_prepare, NoCustom, ParseAndPrepare};
use sc_service::{AbstractService, Configuration, Roles as ServiceRoles};
use std::cell::RefCell;
use tokio::runtime::Runtime;

/// Parse command line arguments into service configuration.
pub fn run<I, T, E>(args: I, exit: E, version: VersionInfo) -> error::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
    E: IntoExit,
{
    type Config<T> = Configuration<(), T>;
    match parse_and_prepare::<NoCustom, NoCustom, _>(&version, "radicle-registry", args) {
        ParseAndPrepare::Run(cmd) => cmd.run(
            load_spec,
            exit,
            |exit, _cli_args, _custom_args, config: Config<_>| {
                info!("{}", version.name);
                info!("  version {}", config.full_version());
                info!("  by {}, 2017, 2018", version.author);
                info!("Chain specification: {}", config.chain_spec.name());
                info!("Node name: {}", config.name);
                info!("Roles: {:?}", config.roles);
                let runtime = Runtime::new().map_err(|e| format!("{:?}", e))?;
                match config.roles {
                    ServiceRoles::LIGHT => run_until_exit(
                        runtime,
                        service::new_light(config).map_err(|e| format!("{:?}", e))?,
                        exit,
                    ),
                    _ => run_until_exit(
                        runtime,
                        service::new_full(config).map_err(|e| format!("{:?}", e))?,
                        exit,
                    ),
                }
                .map_err(|e| format!("{:?}", e))
            },
        ),
        ParseAndPrepare::BuildSpec(cmd) => cmd.run::<NoCustom, _, _, _>(load_spec),
        ParseAndPrepare::ExportBlocks(cmd) => cmd.run_with_builder(
            |config: Config<_>| Ok(new_full_start!(config).0),
            load_spec,
            exit,
        ),
        ParseAndPrepare::ImportBlocks(cmd) => cmd.run_with_builder(
            |config: Config<_>| Ok(new_full_start!(config).0),
            load_spec,
            exit,
        ),
        ParseAndPrepare::CheckBlock(cmd) => cmd.run_with_builder(
            |config: Config<_>| Ok(new_full_start!(config).0),
            load_spec,
            exit,
        ),
        ParseAndPrepare::PurgeChain(cmd) => cmd.run(load_spec),
        ParseAndPrepare::RevertChain(cmd) => {
            cmd.run_with_builder(|config: Config<_>| Ok(new_full_start!(config).0), load_spec)
        }
        ParseAndPrepare::CustomCommand(_) => Ok(()),
    }?;

    Ok(())
}

fn load_spec(id: &str) -> Result<Option<chain_spec::ChainSpec>, String> {
    match chain_spec::from_id(id) {
        Some(spec) => Ok(Some(spec)),
        _ => Err(format!(
            "Unknown chain spec \"{}\". You must run the node with --dev",
            id
        )),
    }
}

fn run_until_exit<T, E>(mut runtime: Runtime, service: T, e: E) -> error::Result<()>
where
    T: AbstractService,
    E: IntoExit,
{
    let (exit_send, exit) = oneshot::channel();

    let informant = informant::build(&service);

    let future = select(exit, informant).map(|_| Ok(())).compat();

    runtime.executor().spawn(future);

    // we eagerly drop the service so that the internal exit future is fired,
    // but we need to keep holding a reference to the global telemetry guard
    let _telemetry = service.telemetry();

    let service_res = {
        let exit = e.into_exit();
        let service = service.map_err(error::Error::Service).compat();
        let select = select(service, exit).map(|_| Ok(())).compat();
        runtime.block_on(select)
    };

    let _ = exit_send.send(());

    // TODO [andre]: timeout this future #1318

    use futures01::Future;

    let _ = runtime.shutdown_on_idle().wait();

    service_res
}

// handles ctrl-c
pub struct Exit;
impl IntoExit for Exit {
    #[allow(clippy::type_complexity)]
    type Exit = Map<oneshot::Receiver<()>, fn(Result<(), oneshot::Canceled>) -> ()>;
    fn into_exit(self) -> Self::Exit {
        // can't use signal directly here because CtrlC takes only `Fn`.
        let (exit_send, exit) = oneshot::channel();

        let exit_send_cell = RefCell::new(Some(exit_send));
        ctrlc::set_handler(move || {
            let exit_send = exit_send_cell
                .try_borrow_mut()
                .expect("signal handler not reentrant; qed")
                .take();
            if let Some(exit_send) = exit_send {
                exit_send.send(()).expect("Error sending exit notification");
            }
        })
        .expect("Error setting Ctrl-C handler");

        exit.map(drop)
    }
}
