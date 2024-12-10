use std::{error::Error, str::FromStr};

use clap::{
    Args,
    Subcommand
};


#[derive(Debug, Args)]
pub struct AdminCommand {
    #[clap(subcommand)]
    pub action: AdminAction
}

#[derive(Debug, Subcommand)]
pub enum AdminAction {
    Create(AdminCreateCommand),
    Inspect(AdminInspectCommand),
    Grant(AdminGrantCommand),
    Revoke(AdminRevokeCommand)
}

impl AdminCommand {
    pub fn run(self) {
        todo!()
    }
}



#[derive(Debug, Args)]
pub struct AdminCreateCommand {
    #[clap(subcommand)]
    pub entity_type: AdminCreateEntityType
}

#[derive(Debug, Subcommand)]
pub enum AdminCreateEntityType {
    Permission,
    Group
}


#[derive(Debug, Args)]
pub struct AdminInspectCommand {
    #[clap(subcommand)]
    pub action: AdminInspectEntityType
}

#[derive(Debug, Subcommand)]
pub enum AdminInspectEntityType {
    Permission(AdminInspectStringIDCommand),
    Group(AdminInspectStringIDCommand),
    User(AdminInspectStringIDCommand),
    Event(AdminInspectIntegerIDCommand)
}



#[derive(Debug, Args)]
pub struct AdminInspectStringIDCommand {
    pub id: String
}

#[derive(Debug, Args)]
pub struct AdminInspectIntegerIDCommand {
    pub id: i64
}



#[derive(Debug, Args)]
pub struct AdminGrantCommand {
    #[clap(subcommand)]
    pub entity_type: AdminGrantCommandEntityType,
    pub to: String,
    pub value: String
}

#[derive(Debug, Subcommand)]
pub enum AdminGrantCommandEntityType {
    Group(AdminGrantCommandData),
    User(AdminGrantCommandData)
}

#[derive(Debug, Args)]
pub struct AdminGrantCommandData {
    to: String,
    value: String
}



#[derive(Debug, Args)]
pub struct AdminRevokeCommand {
    #[clap(subcommand)]
    pub entity_type: AdminGrantCommandEntityType,
    pub to: String,
    pub value: String
}


#[derive(Debug, Subcommand)]
pub enum AdminRevokeCommandEntityType {
    Group(AdminGrantCommandData),
    User(AdminGrantCommandData)
}


#[derive(Debug, Args)]
pub struct AdminRevokeCommandData {
    to: String,
    value: String
}