# canister-upgrader

## Introduction

The upgrader is a canister allows the creation of polls to approve upgrade of other canisters.
This allows registered voters to approve or reject upgrades identified by unique hashes.

## Poll types

Thee different types of polls can be created:
1. `ProjectHash`: a poll to approve a specific canister hash
1. `AddPermission`: a poll to grant permissions to a Principal
1. `RemovePermission`: a poll to remove permissions from a Principal

For each new poll, the creator has to provide the following informations:
- `description`: The description of the poll,
- `poll_type`: The type of poll as discussed above,
- `start_timestamp_secs`: The timestamp in seconds of when the poll opens
- `end_timestamp_secs`: The timestamp in seconds of when the poll closes

## User Permissions

The access to the canister features is restricted by a set of permissions that allow selected Pricipals to operate on the canister.
The available permissions are:

- `Admin`: this permission grants admin rights to the principal. An admin can directy grant or remove permissions to other principals
- `CreateProject`: Allows calling the endpoints to create a project (e.g. evm, bridge, etc.)
- `CreatePoll`: Allows calling the endpoints to create a poll
- `VotePoll`: Allows calling the endpoints to vote in a poll

