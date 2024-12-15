use std::{error::Error, io::{self, read_to_string}, str::FromStr};

use clap::{
    Args,
    Subcommand
};
use colored::Colorize;
use futures::executor::block_on;
use sqlx::PgConnection;

use crate::{config::CauthConfig, models::{event::Event, group::Group, permission::{Permission, PermissionRetrieveError}, user::User}, util::io::input};


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
    pub fn run(self, config: CauthConfig) {
        let _ = match self.action {
            AdminAction::Create(cmd) => cmd.run(config),
            AdminAction::Inspect(cmd) => cmd.run(config),
            AdminAction::Grant(cmd) => cmd.run(config),
            AdminAction::Revoke(cmd) => cmd.run(config)
        };
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

impl AdminCreateCommand {
    pub fn run(self, config: CauthConfig) {
        match self.entity_type {
            AdminCreateEntityType::Permission => {
                let _ = block_on(Self::create_permission(config));
            },
            AdminCreateEntityType::Group => {
                let _ = block_on(Self::create_group(config));
            }
        }
    }

    async fn create_permission(config: CauthConfig) {
        let name = input(format!("{} Enter the name of the permission: ", "+".green())).unwrap();
        let description = input(format!("{} Enter the description of the permission: ", "+".green())).unwrap();

        let mut executor = block_on(config.db_conn.acquire()).unwrap();
        match Permission::insert(&mut executor, &name, &description).await {
            Ok(_) => (),
            Err(_) => println!("{}", "This permission already exist".red())
        };
    }

    async fn create_group(config: CauthConfig) {
        let mut executor = block_on(config.db_conn.acquire()).unwrap();

        let name = input(format!("{} Enter the name of the group: ", "+".green())).unwrap();
        let description = input(format!("{} Enter the description of the group: ", "+".green())).unwrap();
        println!("{} Enter the permission names of the group (empty to stop): ", "+".green());

        let mut permissions: Vec<String> = vec![];

        while let Ok(permission_name) = input(format!("  {} Enter the name of the permission: ", "+".green())) {
            match Permission::retrieve(&mut executor, &permission_name).await {
                Ok(_) => {
                    permissions.push(permission_name);
                },
                Err(_) => {
                    println!("{}", "  This permission do not exist".red())
                }
            }
        }

        match Group::insert(&mut executor, name, description, permissions).await {
            Ok(_) => (),
            Err(_) => println!("{}", "This group already exist".red())
        };
    }
}



#[derive(Debug, Args)]
pub struct AdminInspectCommand {
    #[clap(subcommand)]
    pub entity_type: AdminInspectEntityType
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

impl AdminInspectCommand {
    pub fn run(self, config: CauthConfig) {
        match self.entity_type {
            AdminInspectEntityType::Permission(id) => {
                let mut executor = block_on(config.db_conn.acquire()).unwrap();
                let permission = match block_on(Permission::retrieve(&mut executor, &id.id)) {
                    Ok(permission) => permission,
                    Err(_) => {
                        println!("{}", format!("Permission \"{}\" not found.", id.id).red());
                        return;
                    }
                };

                println!("{}", permission.to_string());
            },
            AdminInspectEntityType::Group(id) => {
                let mut executor = block_on(config.db_conn.acquire()).unwrap();
                let group = match block_on(Group::retrieve(&mut executor, &id.id)) {
                    Ok(group) => group,
                    Err(_) => {
                        println!("{}", format!("Group \"{}\" not found.", id.id).red());
                        return;
                    }
                };

                println!("{}", group.to_string());
            },
            AdminInspectEntityType::User(id) => {
                let mut executor = block_on(config.db_conn.acquire()).unwrap();
                let user = match block_on(User::retrieve(&mut executor, &id.id)) {
                    Ok(user) => user,
                    Err(_) => {
                        println!("{}", format!("User \"{}\" not found.", id.id).red());
                        return;
                    }
                };

                println!("{}", user.to_string());
            },
            AdminInspectEntityType::Event(id) => {
                let mut executor = block_on(config.db_conn.acquire()).unwrap();
                let event = match block_on(Event::retrieve(&mut executor, id.id)) {
                    Ok(event) => event,
                    Err(_) => {
                        println!("{}", format!("Event \"{}\" not found.", id.id).red());
                        return;
                    }
                };

                println!("{}", event.to_string());
            }
        };
    }
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

impl AdminGrantCommand {
    pub fn run(self, config: CauthConfig) {
        
    }
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

impl AdminRevokeCommand {
    pub fn run(self, config: CauthConfig) {
        
    }
}
