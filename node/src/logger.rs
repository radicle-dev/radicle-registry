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

//! Provides [init] to initialize our custom logger.
use env_logger::fmt::Color;
use std::io::Write as _;

/// Initializes [env_logger] using the `RUST_LOG` environment variables and our custom formatter.
pub fn init() {
    env_logger::Builder::from_default_env()
        .format(format_record)
        .target(env_logger::Target::Stdout)
        .init();
}

fn format_record(
    formatter: &mut env_logger::fmt::Formatter,
    record: &log::Record,
) -> std::io::Result<()> {
    let time = time::OffsetDateTime::now_local();

    let context = format!(
        "{time}.{ms:03} {level:<5} {target}",
        time = time.format("%H:%M:%S"),
        ms = time.millisecond(),
        target = record.target(),
        level = record.level(),
    );

    writeln!(
        formatter,
        "{context}  {msg}",
        context = formatter
            .style()
            // Using black with `set_intense(true)` results in grey output.
            .set_color(Color::Black)
            .set_intense(true)
            .value(context),
        msg = record.args()
    )
}
