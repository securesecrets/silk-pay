# Silk Pay
## Design choices
### Admin 
* Admin can change the fee.
* Admin can change the treasury_address.
* Admin can nominate a new admin.
* Nominated new admin can accept the nomination and replace the current admin.

### Safe Send
* Sender creates a Safe Send Tx via SSCRT, sends fee in SSCRT, sets details of Tx. (Tx status = pending address payment)
* If token is not registered, it is registered.
* Receiver confirms Tx. (Tx status = pending payment)
* Sender sends the correct token and amount and the contract forwards that to the receiver and the contract sends the fee to the treasury. (Tx status = finalized)

### Receive Request
* Receiver create a Receive Request Tx via SSCRT, sends fee in SSCRT, sets details of Tx. (Tx status = pending payment)
* If token is not registered, it is registered.
* Sender sends the correct token and amount and the contract forwards that to the receiver and the contract sends the fee to the treasury. (Tx status = finalized)

### Cancelling
* Either party can cancel the Tx and the fee is sent back to the creator. (Tx status = cancelled)

### Shared Viewing Key
Trialling access for the user via them using their SHADE viewing key. There's so many viewing keys these days so thought it could be a win-win for everyone (user doesn't have to create and store another viewing key, network doesn't have to request with viewing key everytime someone comes to the site or opens their wallet).

## Testing locally examples
```
# Optimize contract
docker run --rm -v $(pwd):/contract --mount type=volume,source=$(basename $(pwd))_cache,target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry enigmampc/secret-contract-optimizer

# Run chain locally
docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v $(pwd):/root/code --name secretdev enigmampc/secret-network-sw-dev

# Access container via separate terminal window
docker exec -it secretdev /bin/bash

# cd into code folder
cd code

# Store contracts required for test
secretcli tx compute store snip-20-reference-impl.wasm.gz --from a --gas 3000000 -y --keyring-backend test
secretcli tx compute store sn-silk-pay.wasm.gz --from a --gas 3000000 -y --keyring-backend test

# Get the contract's id
secretcli query compute list-code

# Init SNIP-20 (SSCRT)
CODE_ID=1
INIT='{ "name": "SSCRT", "symbol": "SSCRT", "decimals": 6, "initial_balances": [{ "address": "secret1u52q5le8tmgejkt5cfsgd0pmldkzxq3eerjp9d", "amount": "1000000000000000000" }, { "address": "secret1u5dv38d8qvf86z3kwyd9xsqd4eqf3juxlxh970", "amount": "1000000000000000000" }], "prng_seed": "RG9UaGVSaWdodFRoaW5nLg==", "config": { "public_total_supply": true, "enable_deposit": true, "enable_redeem": true, "enable_mint": false, "enable_burn": false } }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SSCRT" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Set viewing key for SSCRT
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "ThereWillBeButt."}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "ThereWillBeButt."}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Init SHADE (SHD)
INIT='{ "name": "SHADE", "symbol": "SHD", "decimals": 8, "initial_balances": [{ "address": "secret1u52q5le8tmgejkt5cfsgd0pmldkzxq3eerjp9d", "amount": "2000000000000000000" }, { "address": "secret1u5dv38d8qvf86z3kwyd9xsqd4eqf3juxlxh970", "amount": "2000000000000000000" }], "prng_seed": "RG9UaGVSaWdodFRoaW5nLg==", "config": { "public_total_supply": true, "enable_deposit": false, "enable_redeem": false, "enable_mint": false, "enable_burn": false } }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SHADE" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Set viewing key for SHADE
secretcli tx compute execute secret1hqrdl6wstt8qzshwc6mrumpjk9338k0lpsefm3 '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "ThereWillBeButt."}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
secretcli tx compute execute secret1hqrdl6wstt8qzshwc6mrumpjk9338k0lpsefm3 '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "ThereWillBeButt."}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Init SILK (SILK)
INIT='{ "name": "SILK", "symbol": "SILK", "decimals": 8, "initial_balances": [{ "address": "secret1u52q5le8tmgejkt5cfsgd0pmldkzxq3eerjp9d", "amount": "3000000000000000000" }, { "address": "secret1u5dv38d8qvf86z3kwyd9xsqd4eqf3juxlxh970", "amount": "3000000000000000000" }], "prng_seed": "RG9UaGVSaWdodFRoaW5nLg==", "config": { "public_total_supply": true, "enable_deposit": false, "enable_redeem": false, "enable_mint": false, "enable_burn": false } }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SILK" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Set viewing key for SILK
secretcli tx compute execute secret18r5szma8hm93pvx6lwpjwyxruw27e0k57tncfy '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "ThereWillBeButt."}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
secretcli tx compute execute secret18r5szma8hm93pvx6lwpjwyxruw27e0k57tncfy '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "ThereWillBeButt."}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Init SILK Pay
CODE_ID=2
INIT='{ "fee": "500000", "shade": {"address": "secret1hqrdl6wstt8qzshwc6mrumpjk9338k0lpsefm3", "contract_hash": "35F5DB2BC5CD56815D10C7A567D6827BECCB8EAF45BC3FA016930C4A8209EA69"}, "sscrt": {"address": "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg", "contract_hash": "35F5DB2BC5CD56815D10C7A567D6827BECCB8EAF45BC3FA016930C4A8209EA69"}, "treasury_address": "secret1e9c0ghgmmf64r43tn55e72yd26vqcvqzlcxpv9" }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SILK Pay" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Query config
secretcli query compute query secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"config": {}}'

# Query Txs
secretcli query compute query secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"txs": {"address": "secret1u52q5le8tmgejkt5cfsgd0pmldkzxq3eerjp9d", "key": "DoTheRightThing.", "page": 0, "page_size": 50}}'

# Nominate new admin
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"nominate_new_admin":{"address": "secret1u5dv38d8qvf86z3kwyd9xsqd4eqf3juxlxh970"}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Accept new admin nomination
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"accept_new_admin_nomination":{}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Update fee
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"update_fee":{ "fee": "555" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Update treasury address
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"update_treasury_address":{ "address": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Create send request
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"send": { "recipient": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw", "amount": "555", "msg": "eyJjcmVhdGVfc2VuZF9yZXF1ZXN0IjogeyJhZGRyZXNzIjogInNlY3JldDF1NTJxNWxlOHRtZ2Vqa3Q1Y2ZzZ2QwcG1sZGt6eHEzZWVyanA5ZCIsICJzZW5kX2Ftb3VudCI6ICI1NTU1NTUiLCAiZGVzY3JpcHRpb24iOiAiYXBvY2FseXB0byIsICJ0b2tlbiI6IHsiYWRkcmVzcyI6ICJzZWNyZXQxOHI1c3ptYThobTkzcHZ4Nmx3cGp3eXhydXcyN2UwazU3dG5jZnkiLCAiY29udHJhY3RfaGFzaCI6ICIzNUY1REIyQkM1Q0Q1NjgxNUQxMEM3QTU2N0Q2ODI3QkVDQ0I4RUFGNDVCQzNGQTAxNjkzMEM0QTgyMDlFQTY5In19fQ==" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Confirm address
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"send": { "recipient": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw", "amount": "0", "msg": "eyJjb25maXJtX2FkZHJlc3MiOiB7InBvc2l0aW9uIjogMH19" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

## References
1. Silk Pay description: https://github.com/securesecrets/ShadeGrants/issues/1
2. Secret contracts guide: https://github.com/enigmampc/secret-contracts-guide
