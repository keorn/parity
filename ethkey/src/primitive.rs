// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use std::ops::{Deref, DerefMut};
use std::{fmt, cmp, hash};
use std::str::FromStr;
use rustc_serialize::hex::{ToHex, FromHex};
use Error;

macro_rules! impl_primitive {
	($name: ident, $size: expr, $err: expr) => {

		#[repr(C)]
		#[derive(Eq)]
		pub struct $name([u8; $size]);

		impl fmt::Debug for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
				write!(f, "{}", self.to_hex())
			}
		}

		impl fmt::Display for $name {
			fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
				write!(f, "{}", self.to_hex())
			}
		}

		impl FromStr for $name {
			type Err = Error;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				match s.from_hex() {
					Ok(ref hex) if hex.len() == $size => {
						let mut res = $name::default();
						res.copy_from_slice(hex);
						Ok(res)
					},
					_ => Err($err)
				}
			}
		}

		impl PartialEq for $name {
			fn eq(&self, other: &Self) -> bool {
				let self_ref: &[u8] = &self.0;
				let other_ref: &[u8] = &other.0;
				self_ref == other_ref
			}
		}

		impl PartialOrd for $name {
			fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
				let self_ref: &[u8] = &self.0;
				let other_ref: &[u8] = &other.0;
				self_ref.partial_cmp(other_ref)
			}
		}

		impl Ord for $name {
			fn cmp(&self, other: &Self) -> cmp::Ordering {
				let self_ref: &[u8] = &self.0;
				let other_ref: &[u8] = &other.0;
				self_ref.cmp(other_ref)
			}
		}

		impl Clone for $name {
			fn clone(&self) -> Self {
				let mut res = Self::default();
				res.copy_from_slice(&self.0);
				res
			}
		}

		impl Default for $name {
			fn default() -> Self {
				$name([0u8; $size])
			}
		}

		impl From<[u8; $size]> for $name {
			fn from(s: [u8; $size]) -> Self {
				$name(s)
			}
		}

		impl Into<[u8; $size]> for $name {
			fn into(self) -> [u8; $size] {
				self.0
			}
		}

		impl hash::Hash for $name {
			fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
				let self_ref: &[u8] = &self.0;
				self_ref.hash(state)
			}
		}

		impl Deref for $name {
			type Target = [u8; $size];

			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl DerefMut for $name {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.0
			}
		}
	}
}

impl_primitive!(Address, 20, Error::InvalidAddress);
impl_primitive!(Secret, 32, Error::InvalidSecret);
impl_primitive!(Message, 32, Error::InvalidMessage);
impl_primitive!(Public, 64, Error::InvalidPublic);

#[cfg(test)]
mod tests {

}
