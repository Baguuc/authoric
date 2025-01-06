# Routes
Here all the routes of the API are listed.

## Permissions

### GET /permissions
Retrieves permissions from the database

Requirements:
+ User have to have the "permissions:get" permission

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ order_in - Optional, order in which the data should be returned in (default: desc)
+ page - Optional, the data is returned in pages, max 10 entries per page. This parameter indicates which page to fetch.

### POST /permissions
Creates a permission

Errors:
Fails when a permission with the same name already exist.

Requirements:
+ User have to have the "permissions:post" permission

JSON Content:
+ name - Required, permission name
+ description - Required, permission description

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)

### DELETE /permissions/{name}
Deletes a permission

Errors:
Fails when a permission with the specified name do not exist.

Requirements:
+ User have to have the "permissions:delete" permission

Query Parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)

## Groups

### GET /groups
Retrieves groups from the database

Requirements:
+ User have to have the "groups:get" permission

Query parameters
+ session_token - Required, token of login session retrieved from POST /user route
+ order_in - Optional, order in which the data should be returned in (default: descending)
+ page - Optional, the data is returned in pages, max 10 entries per page. This parameter indicates which page to fetch.

### POST /groups
Creates a group

Errors:
Fails when a group with the same name is already created.

Requirements:
+ User have to have the "groups:post" permission

JSON Content:
+ name - Required, groups name
+ description - Required, groups description

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)


### DELETE /groups/{name}
Deletes a group

Errors:
Fails when a group with the specified name do not exist.

Requirements:
+ User have to have the "groups:delete" permission

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)

### POST /groups/{name}/{permission_name}
Grants group a permission

Errors:
Fails when a mentioned group or permission do not exist.

Requirements:
+ User have to have the "groups:update" permission

### DELETE /groups/{name}/{permission_name}
Revokes a permission from group

Errors:
Fails when a mentioned group do not exist or group do not have mentioned permission.

Requirements:
+ User have to have the "groups:update" permission

## User

### POST /users
Creates a new user

Errors:
Fails when the user is already created.

JSON Content:
+ login - Required, represents the login to assign
+ password - Required, represents the password to assign
+ details - Optional, additional details in json format that will be associated with a user (default: empty json object ("{}"))

### DELETE /users/{user}
Deletes a user's account from the current login session, removing all of it's sessions

Path parameters:
+ login - the login of the user to delete

Requirements:
+ user have to have "users:delete:{user}" permission

Query parameters
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)

### GET /user
Get the currently logged in user data.

Errors:
+ When the session do not exist

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route

### POST /user
Logs in to a user account, creating a new session

Errors:
Fails when the login and password do not match these in the database

Query parameters:
+ login - Required, represents the login to assign
+ password - Required, represents the password to assign

## DELETE /user
Logs out from the user's account, deleting the session

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)

### POST /users/{name}/{group_name}
Grants user a group

Errors:
Fails when a mentioned user or group do not exist.

Requirements:
+ User have to have the "users:update" permission

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)

### DELETE /users/{name}/{group_name}
Revokes a group from user

Errors:
Fails when a mentioned user do not exist or user do not have mentioned group.

Requirements:
+ User have to have the "users:update" permission

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route
+ auto_commit - Optional, determines if the event should be created or should the operation be commited (default: true)

## Event

### POST /events/{id}
Commits an event with specified ID.

Requirements:
+ User have to have the "events:use:{id}" permission (it's granted to him and the default admin group while creating the event)

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route

### DELETE /events/{id}
Cancels an event with specified ID.

Requirements:
+ User have to have the "events:use:{id}" permission (it's granted to him and the default admin group while creating the event)

Query parameters:
+ session_token - Required, token of login session retrieved from POST /user route