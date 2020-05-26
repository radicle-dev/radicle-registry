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

use sc_service::{config::Configuration, Properties};
use std::convert::{TryFrom, TryInto};

/// Configuration of PoW algorithm, can be stored as chain spec property
#[derive(serde::Deserialize, serde::Serialize)]
pub enum Config {
    Dummy,
    Blake3,
}

impl Config {
    const PROPERTY_KEY: &'static str = "pow_alg";
}

impl<'a> TryFrom<&'a Configuration> for Config {
    type Error = &'static str;

    fn try_from(config: &'a Configuration) -> Result<Self, Self::Error> {
        config.chain_spec.as_ref().properties().try_into()
    }
}

impl TryFrom<Properties> for Config {
    type Error = &'static str;

    fn try_from(mut properties: Properties) -> Result<Self, Self::Error> {
        let pow_alg_str = properties
            .remove(Config::PROPERTY_KEY)
            .ok_or("properies do not contain PoW algorithm")?;
        serde_json::from_value(pow_alg_str).map_err(|_| "PoW algorithm property malformed")
    }
}

impl TryFrom<Config> for Properties {
    type Error = &'static str;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let key = Config::PROPERTY_KEY.to_string();
        let value = serde_json::to_value(config)
            .map_err(|_| "failed to serialize PoW algorithm into a property")?;
        let mut map = Properties::with_capacity(1);
        map.insert(key, value);
        Ok(map)
    }
}
