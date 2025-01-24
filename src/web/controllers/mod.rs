pub mod permissions;
pub mod groups;

pub use self::{
    permissions::{
        list::controller as ListPermissionsController,
        insert::controller as InsertPermissionController,
        delete::controller as DeletePermissionController
    },
    groups::{
        list::controller as ListGroupsController,
        insert::controller as InsertGroupController,
        delete::controller as DeleteGroupController
    }
};
