use std::{error::Error, io::{self, read_to_string}, str::FromStr};

use clap::{
    Args,
    Subcommand
};
use colored::Colorize;
use futures::executor::block_on;
use sqlx::PgConnection;

use crate::{config::CauthConfig, models::{event::Event, group::{Group, GroupGrantError, GroupRevokeError}, permission::{Permission, PermissionRetrieveError}, user::{User, UserGrantError, UserRevokeError}}, util::io::input};


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

        let mut executor = config.db_conn.acquire().await.unwrap();
        match Permission::insert(&mut executor, &name, &description).await {
            Ok(_) => (),
            Err(_) => println!("{}", "This permission already exist".red())
        };
    }

    async fn create_group(config: CauthConfig) {
        let mut executor = config.db_conn.acquire().await.unwrap();

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
        match self.entity_type {
            AdminGrantCommandEntityType::Group(data) => {
                let _ = match block_on(Self::grant_group_permission(config, &data)) {
                    Ok(_) => println!(
                        "{}",
                        format!("Successfully granted permission {} to group {}.", data.value, data.to)
                            .green()
                    ),
                    Err(err) => println!(
                        "{}",
                        format!("Error while granting permission {} to group {}.\n{}", data.value, data.to, err.to_string())
                            .green()
                    )
                };
            }
            AdminGrantCommandEntityType::User(data) => {
                let _ = match block_on(Self::grant_user_group(config, &data)) {
                    Ok(_) => println!(
                        "{}",
                        format!("Successfully granted group {} to user {}.", data.value, data.to)
                            .green()
                    ),
                    Err(err) => println!(
                        "{}",
                        format!("Error while granting group {} to user {}.\n{}", data.value, data.to, err.to_string())
                            .green()
                    )
                };
            }
        }
    }

    pub async fn grant_group_permission(config: CauthConfig, data: &AdminGrantCommandData) -> Result<(), GroupGrantError> {
        let mut executor = config.db_conn.acquire().await.unwrap();
        Group::grant_permission(&mut executor, &data.to, &data.value).await?;

        return Ok(());
    }

    pub async fn grant_user_group(config: CauthConfig, data: &AdminGrantCommandData) -> Result<(), UserGrantError> {
        let mut executor = config.db_conn.acquire().await.unwrap();
        User::grant_group(&mut executor, &data.to, &data.value).await?;

        return Ok(());
    }
}



#[derive(Debug, Args)]
pub struct AdminRevokeCommand {
    #[clap(subcommand)]
    pub entity_type: AdminRevokeCommandEntityType,
    pub to: String,
    pub value: String
}


#[derive(Debug, Subcommand)]
pub enum AdminRevokeCommandEntityType {
    Group(AdminRevokeCommandData),
    User(AdminRevokeCommandData)
}


#[derive(Debug, Args)]
pub struct AdminRevokeCommandData {
    to: String,
    value: String
}

impl AdminRevokeCommand {
    pub fn run(self, config: CauthConfig) {
        match self.entity_type {
            AdminRevokeCommandEntityType::Group(data) => {
                let _ = match block_on(Self::revoke_group_permission(config, &data)) {
                    Ok(_) => println!(
                        "{}",
                        format!("Successfully revoked permission {} from group {}.", data.value, data.to)
                            .green()
                    ),
                    Err(err) => println!(
                        "{}",
                        format!("Error while revoking permission {} from group {}.\n{}", data.value, data.to, err.to_string())
                            .green()
                    )
                };
            }
            AdminRevokeCommandEntityType::User(data) => {
                let _ = match block_on(Self::revoke_user_group(config, &data)) {
                    Ok(_) => println!(
                        "{}",
                        format!("Successfully revoked group {} from user {}.", data.value, data.to)
                            .green()
                    ),
                    Err(err) => println!(
                        "{}",
                        format!("Error while revoking group {} from user {}.\n{}", data.value, data.to, err.to_string())
                            .green()
                    )
                };
            }
        }
    }

    pub async fn revoke_group_permission(config: CauthConfig, data: &AdminRevokeCommandData) -> Result<(), GroupRevokeError> {
        let mut executor = config.db_conn.acquire().await.unwrap();
        Group::revoke_permission(&mut executor, &data.to, &data.value).await?;

        return Ok(());
    }

    pub async fn revoke_user_group(config: CauthConfig, data: &AdminRevokeCommandData) -> Result<(), UserRevokeError> {
        let mut executor = config.db_conn.acquire().await.unwrap();
        User::revoke_group(&mut executor, &data.to, &data.value).await?;

        return Ok(());
    }
}
