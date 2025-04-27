pub mod subcommands;

use crate::{
    cli::subcommands::{
        admin::AdminCommand, config::ConfigCommand, run::RunCommand,
    },
    config::CauthConfig,
    models::{Group, Permission},
};
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CauthCli {
    #[clap(subcommand)]
    pub action: ActionType,
}

impl CauthCli {
    pub fn run(self) {
        let config = CauthConfig::parse_or_edit();

        let _ = match self.action {
            ActionType::Run(cmd) => cmd.run(config),
            ActionType::Admin(cmd) => cmd.run(config),
            ActionType::Config(cmd) => cmd.run(),
        };
    }
}

#[derive(Debug, Subcommand)]
pub enum ActionType {
    Run(RunCommand),
    Config(ConfigCommand),
    Admin(AdminCommand),
}

pub async fn init_defaults(config: &CauthConfig) {
    let mut tx = config.db_conn.begin().await.unwrap();

    let _ = Permission::insert(
        &mut tx,
        &"authoric:permissions:get".to_string(),
        &"permission to retrieve the permission list from the database".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:permissions:post".to_string(),
        &"permission to post new permission to the database".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:permissions:delete".to_string(),
        &"permission to delete a permission from the database".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:groups:get".to_string(),
        &"permission to retrieve the groups list from the database".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:groups:post".to_string(),
        &"permission to post new group to the database".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:groups:delete".to_string(),
        &"permission to post new group to the database".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:groups:update".to_string(),
        &"permission to grant/revoke permissions to groups".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:users:update".to_string(),
        &"permission to grant/revoke groups to users".to_string(),
    )
    .await;

    let _ = Permission::insert(
        &mut tx,
        &"authoric:users:delete".to_string(),
        &"permission to delete ANY user on the service, use with caution.".to_string(),
    )
    .await;

    let _ = Group::insert(
    &mut tx,
    &"root".to_string(), 
    &"the most privileged group, having to permissions to do everything. Caution: do not grant this group to any untrusted user as it can result in damages done to your system. Instead, create their own group fitting their needs.".to_string(),
    &vec![
      "authoric:permissions:get".to_string(),
      "authoric:permissions:post".to_string(),
      "authoric:permissions:delete".to_string(),
      "authoric:groups:get".to_string(),
      "authoric:groups:post".to_string(),
      "authoric:groups:delete".to_string(),
      "authoric:groups:update".to_string(),
      "authoric:users:update".to_string(),
      "authoric:users:delete".to_string()
    ]
  )
  .await;

    let _ = tx.commit().await;
}
