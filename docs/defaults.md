# Defaults
This API adds it's integral data to the database to be working properly.
All of this includes:

##### Added on startup:

###### Permissions
+ **cauth:permissions:get** - permission to retrieve the permission list from the database
+ **cauth:permissions:post** - permission to post new permission to the database
+ **cauth:permissions:delete** - permission to delete a permission from the database
+ **cauth:groups:get** - permission to retrieve the groups list from the database
+ **cauth:groups:post** - permission to post new group to the database
+ **cauth:groups:delete** - permission to delete a group from the database
+ **cauth:groups:update** - permission to grant/revoke permissions to groups
+ **cauth:users:update** - permission to grant/revoke groups to users

###### Groups
+ **root** - the most privileged group, having to permissions to do everything. Caution: do not grant this group to any untrusted user as it can result in damages done to your system. Instead, create their own group fitting their needs.

#####  Added on action:

###### Permissions:
+ **cauth:event:use:{id}** - created and granted to root and user when user creates a new event (number stands for the event's id).
