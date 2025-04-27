# Defaults
This API adds it's integral data to the database to be working properly.
All of this includes:

##### Added on startup:

###### Permissions
+ **authoric:permissions:get** - permission to retrieve the permission list from the database
+ **authoric:permissions:post** - permission to post new permission to the database
+ **authoric:permissions:delete** - permission to delete a permission from the database
+ **authoric:groups:get** - permission to retrieve the groups list from the database
+ **authoric:groups:post** - permission to post new group to the database
+ **authoric:groups:delete** - permission to delete a group from the database
+ **authoric:groups:update** - permission to grant/revoke permissions to groups
+ **authoric:users:update** - permission to grant/revoke groups to users

###### Groups
+ **root** - the most privileged group, having to permissions to do everything. Caution: do not grant this group to any untrusted user as it can result in damages done to your system. Instead, create their own group fitting their needs.

#####  Added on action:

###### Permissions:
+ **authoric:event:use:{id}** - created and granted to root and user when user creates a new event (number stands for the event's id).
