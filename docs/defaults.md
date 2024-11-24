# Defaults
This API adds it's integral data to the database to be working properly.
All of this includes:

##### Added on startup:

###### Permissions
+ **permissions:get** - permission to retrieve the permission list from the database
+ **permissions:post** - permission to post new permission to the database
+ **permissions:delete** - permission to delete a permission from the database
+ **groups:get** - permission to retrieve the groups list from the database
+ **groups:post** - permission to post new group to the database
+ **groups:delete** - permission to delete a group from the database

###### Groups
+ **root** - the most privileged group, having to permissions to do everything. Caution: do not grant this group to any untrusted user as it can result in damages done to your system. Instead, create their own group fitting their needs.

#####  Added on action:

###### Permissions:
+ **event:use_{number}** - created and granted to root and user when user creates a new event (number stands for the event's id).
