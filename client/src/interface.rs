//! Abstract interfaces for the ledger and related types.
use alloc::prelude::v1::*;
use parity_scale_codec::{Decode, Encode};

pub type ProjectId = [u8; 20];
pub type AccountId = [u8; 20];

#[derive(Decode, Encode, Debug, Ord, Eq, PartialEq, PartialOrd, Clone)]
pub struct Project {
    pub description: String,
    pub name: String,
    pub img_url: String,
    pub members: Vec<AccountId>,
}

/// Public interface of the Radicle Registry
pub trait Registry {
    fn counter_inc(&mut self);

    fn counter_value(&mut self) -> u32;

    fn register_project(&mut self, name: String, description: String, img_url: String)
        -> ProjectId;

    fn get_project(&mut self, project_id: ProjectId) -> Option<Project>;

    fn get_projects(&mut self) -> Vec<Project>;
}
