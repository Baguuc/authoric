# Auth CLI

## Base commands
+ ### run - run the API server
```bash
  cauth run
```
+ ### daemon - run the API server in background
```bash
  cauth daemon
```

## Config commands
+ ### edit - edit the config in the editor
```bash
  cauth config edit
```

## Admin commands
+ ### create - create a permission or group
```bash
  cauth admin create <permission|group>
  
  # Example
  cauth admin create permission
  cauth admin create group
```
+ ### inspect - see the data of a permission, group, user or event
```bash
  cauth admin inspect <permission|group|user|event> <permission_name|group_name|user_login|event_id>
  
  # Example
  cauth admin inspect permission self-content:manage
  cauth admin inspect user user123
```
+ ### grant - grant a permission to a group or group to a user
```bash
  cauth admin grant <user|group> <user_login|group_name> <group_name|permission_name>
  
  # Example
  cauth admin grant user user123 regular_plan_user
  cauth admin grant group moderator all-content:manage
```
+ ### revoke - revoke a permission from a group or group from a user
```bash
  cauth admin revoke <user|group> <user_login|group_name> <group_name|permission_name>
  
  # Example
  cauth admin revoke user user123 better_plan_user
  cauth admin revoke group member self-content:manage
```