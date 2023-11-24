# Bnet RPC

Contains the `proto2` files extracted from the 7.3.5.26972 WoW client, with
`method_id`s taken from TC as of the `8.0.1/28153` branch.

The build script generates service traits that implement the basic Rpc expected
of the wow client.

The this crate also contains several helper traits like `BnetRpcService` for
behaviour shared across all the proto services, as well as helper structs/types
like `BnetServiceWrapper` and `BattlenetRpcErrorCode`
