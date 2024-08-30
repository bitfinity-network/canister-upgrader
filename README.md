# canister-upgrader

## Introduction

The upgrader canister allows the creation of polls to approve upgrade of other canisters.
This is achieved by allowing registered voters to approve or reject upgrades identified by unique hashes.

## Poll types

Thee different types of polls can be created:
1. `ProjectHash`: a poll to approve a specific project hash
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

## Manual local Testing

### Prepare the environment

Build the canister wasm:
```bash
./scripts/build.sh
```

(Optional) If you don't have one yet, create a local dfx identity
```bash
dfx identity new --storage-mode=plaintext alice
dfx identity use alice
```

Get current IC identity pricipal
```bash
IDENTITY_PRICIPAL=$(dfx identity get-principal)
echo $IDENTITY_PRICIPAL
```

Start dfx in background
```bash
dfx start --clean --background --artificial-delay 0
```

Install the upgrader-canister and set the current IC identity as administrator
```bash
dfx canister install --mode=install --yes --network=local upgrader_canister --argument="(record { admin = principal \"$IDENTITY_PRICIPAL\" })"
UPGRADER_CANISTER_ID=$(dfx canister id upgrader_canister)
```

Verify that the canister is working, this should return information about the canister build
```bash
dfx canister call $UPGRADER_CANISTER_ID canister_build_data --network local
```

### Create a project, a poll, and vote it

Before a user attemts to create a project or a poll, the administrator should grant him the required permissions.
For simplicity, in this example we use the admin account itself to perform every operation.

Grant the permissions to create a project, create a poll and vote
```bash
dfx canister call $UPGRADER_CANISTER_ID admin_permissions_add --network local "(principal \"$IDENTITY_PRICIPAL\", vec {variant { CreateProject }; variant { CreatePoll }; variant { VotePoll }})"
```

Create a project
```bash
dfx canister call $UPGRADER_CANISTER_ID project_create --network local "(record { key = \"test_project\" ; name = \"test_project_name\"; description = \"test_project_description\" })"
```

Create a poll for the test_project
```bash
dfx canister call $UPGRADER_CANISTER_ID poll_create --network local '(record { description = "A new hash"; end_timestamp_secs = 999_999_999_999 : nat64; poll_type = variant { ProjectHash = record { hash = "hash"; project = "test_project" } }; start_timestamp_secs = 0 : nat64; }, )'
```

this call returns the ID of the new poll, for example, in the following returned data the poll ID is `1`:
```bash
(variant { Ok = 1 : nat64 })
```

Now let's vote by approving for the poll (use `false` to reject instead)
```bash
dfx canister call $UPGRADER_CANISTER_ID poll_vote --network local "($POLL_ID: nat64, true)"
```

We can now verify that the vote is registered
```bash
dfx canister call $UPGRADER_CANISTER_ID poll_get --network local "($POLL_ID: nat64)"
```
