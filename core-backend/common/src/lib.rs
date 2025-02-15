// This file is part of Gear.

// Copyright (C) 2021-2022 Gear Technologies Inc.
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

//! Crate provides support for wasm runtime.

#![no_std]

extern crate alloc;

pub mod funcs;

use alloc::{
    borrow::Cow,
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};
use gear_core::{
    env::Ext,
    gas::GasAmount,
    ids::{CodeId, MessageId, ProgramId},
    memory::{PageBuf, PageNumber},
    message::{ContextStore, Dispatch},
};

pub const EXIT_TRAP_STR: &str = "exit";
pub const LEAVE_TRAP_STR: &str = "leave";
pub const WAIT_TRAP_STR: &str = "wait";
pub const GAS_ALLOWANCE_STR: &str = "allowance";

pub enum TerminationReason<'a> {
    Exit(ProgramId),
    Leave,
    Success,
    Trap {
        explanation: Option<&'static str>,
        description: Option<Cow<'a, str>>,
    },
    Wait,
    GasAllowanceExceed,
}

pub struct ExtInfo {
    pub gas_amount: GasAmount,
    pub pages: BTreeSet<PageNumber>,
    pub accessed_pages: BTreeMap<PageNumber, Vec<u8>>,
    pub generated_dispatches: Vec<Dispatch>,
    pub awakening: Vec<MessageId>,
    pub program_candidates_data: BTreeMap<CodeId, Vec<(ProgramId, MessageId)>>,
    pub context_store: ContextStore,
    pub trap_explanation: Option<&'static str>,
    pub exit_argument: Option<ProgramId>,
}

pub trait IntoExtInfo {
    fn into_ext_info<F: FnMut(usize, &mut [u8]) -> Result<(), &'static str>>(
        self,
        get_page_data: F,
    ) -> Result<ExtInfo, (&'static str, GasAmount)>;
    fn into_gas_amount(self) -> GasAmount;
}

pub struct BackendReport<'a> {
    pub termination: TerminationReason<'a>,
    pub wasm_memory_addr: u64,
    pub info: ExtInfo,
}

#[derive(Debug)]
pub struct BackendError<'a> {
    pub gas_amount: GasAmount,
    pub reason: &'static str,
    pub description: Option<Cow<'a, str>>,
}

pub trait Environment<E: Ext + IntoExtInfo + 'static>: Sized {
    /// Creates new external environment to execute wasm binary:
    /// 1) instatiates wasm binary.
    /// 2) creates wasm memory with filled data (execption if lazy pages enabled).
    /// 3) instatiate external funcs for wasm module.
    fn new(
        ext: E,
        binary: &[u8],
        memory_pages: &BTreeMap<PageNumber, Option<Box<PageBuf>>>,
        mem_size: u32,
    ) -> Result<Self, BackendError<'static>>;

    /// Returns addr to the stack end if it can be identified
    fn get_stack_mem_end(&mut self) -> Option<i32>;

    /// Returns host address of wasm memory buffer. Needed for lazy-pages
    fn get_wasm_memory_begin_addr(&mut self) -> u64;

    /// Run setuped instance starting at `entry_point` - wasm export function name.
    /// - IMPORTANT: env is in inconsistent state after execution.
    fn execute(&mut self, entry_point: &str) -> Result<BackendReport, BackendError>;

    /// Unset env ext and returns gas amount.
    fn drop_env(&mut self) -> GasAmount;
}
