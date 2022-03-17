// This file is part of Gear.

// Copyright (C) 2021 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use gear_core::program::ProgramId;
use serde::{Deserialize, Deserializer, Serialize};
use sp_core::{crypto::Ss58Codec, hexdisplay::AsBytesRef, sr25519::Public, H256};
use sp_keyring::sr25519::Keyring;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Address {
    value: String,
}

impl Default for Address {
    fn default() -> Self {
        Self {
            value: "alice".to_string(),
        }
    }
}

impl Address {
    pub fn to_program_id(&self) -> ProgramId {
        if let Ok(h256) = H256::from_str(&self.value) {
            return ProgramId::from(h256.as_bytes());
        }
        if let Ok(key) = Public::from_ss58check(&self.value) {
            return ProgramId::from_slice(key.as_bytes_ref());
        }
        if let Ok(id) = self.value.parse::<u64>() {
            return ProgramId::from(id);
        }
        ProgramId::from_slice(Keyring::from_str(&self.value)
            .expect(format!("Failed to determine representation of program id {:?}", self.value).as_str())
            .as_bytes_ref()
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum OldAddress {
    #[serde(rename = "account")]
    Account(String),
    #[serde(rename = "id")]
    ProgramId(u64),
    #[serde(rename = "ss58")]
    SS58(String),
    #[serde(rename = "h256")]
    H256(H256),
}

impl Default for OldAddress {
    fn default() -> Self {
        Self::Account("alice".to_string())
    }
}

impl OldAddress {
    pub fn to_program_id(&self) -> ProgramId {
        match self {
            Self::Account(s) => ProgramId::from_slice(
                Keyring::from_str(s)
                    .expect("No account in Keyring")
                    .to_h256_public()
                    .as_bytes(),
            ),
            Self::ProgramId(id) => ProgramId::from(*id),
            Self::SS58(s) => ProgramId::from_slice(
                Public::from_ss58check(s)
                    .expect("Failed to decode ss58")
                    .as_bytes_ref(),
            ),
            Self::H256(id) => ProgramId::from(id.as_bytes()),
        }
    }

    pub fn to_new(self) -> Address {
        Address {
            value: match self {
                Self::Account(s) => s,
                Self::ProgramId(id) => id.to_string(),
                Self::SS58(s) => s,
                Self::H256(h256) => h256.to_string(),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum UntaggedAddress {
    Integer(u64),
    Address(Address),
}

impl From<UntaggedAddress> for Address {
    fn from(a: UntaggedAddress) -> Self {
        match a {
            UntaggedAddress::Address(s) => s,
            UntaggedAddress::Integer(n) => Address {
                value: n.to_string(),
            },
        }
    }
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Address, D::Error> {
    UntaggedAddress::deserialize(deserializer).map(|a| a.into())
}
