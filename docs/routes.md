# Routes
Here all the routes of the API are listed.


## Permissions

### GET /permissions
Retrieves permissions from the database

Requirements:
+ User have to have the "permissions:get" permission

Query parameters:
+ order_in - Optional, order in which the data should be returned in (default: desc)
+ limit - Optional, describes how much rows should be returned (default: None)
+ with_name - Optional, filters what name the retrieved permission will have (default: retrieves any)


### POST /permissions
Creates a permission

Errors:
Fails when a permission with the same name already exist.

Requirements:
+ User have to have the "permissions:post" permission

JSON Content:
+ name - Required, permission name
+ description - Required, permission description


### DELETE /permissions/{name}
Deletes a permission

Errors:
Fails when a permission with the specified name do not exist.

Requirements:
+ User have to have the "permissions:delete" permission


## Groups

### GET /groups
Retrieves groups from the database

Requirements:
+ User have to have the "groups:get" permission

Query parameters
+ order_in - Optional, order in which the data should be returned in (default: descending)
+ limit - Optional, describes how much rows should be returned (default: None)
+ with_name - Optional, filters what name the retrieved group will have (default: retrieves any)


### POST /groups
Creates a group

Errors:
Fails when a group with the same name is already created.

Requirements:
+ User have to have the "groups:post" permission

JSON Content:
+ name - Required, groups name
+ description - Required, groups description


### DELETE /groups/{name}
Deletes a group

Errors:
Fails when a group with the specified name do not exist.

Requirements:
+ User have to have the "groups:delete" permission


## User

### POST /users
Creates a new user

Errors:
Fails when the user is already created.

JSON Content:
+ login - Required, represents the login to assign
+ password - Required, represents the password to assign


### DELETE /users
Deletes a user's account from the current login session, removing all of it's sessions

Path parameters:
+ login - the login of the user to delete

### POST /user
Logs in to a user account, creating a new session

Errors:
Fails when the login and password do not match these in the database

Query parameters:
+ login - Required, represents the login to assign
+ password - Required, represents the password to assign


### DELETE /user
Logs out from the user's account, deleting the session


## Event

### POST /events/{id}
Commits an event with specified ID.

Requirements:
+ User have to have the "event:use_{id}" permission (it's granted to him and the default admin group while creating the event)


### DELETE /events/{id}
Cancels an event with specified ID.

Requirements:
+ User have to have the "event:use_{id}" permission (it's granted to him and the default admin group while creating the event)
