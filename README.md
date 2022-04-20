<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/securesecrets">
    <img src="images/logo.png" alt="Logo" height="80">
  </a>

  <h3 align="center">Silk Pay</h3>

  <p align="center">
    Transfer funds safely and smoothly.
    <br />
    <br />
    <a href="https://btn.group/secret_network/silk_pay_demo">View Demo</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#setting-up-locally">Setting up locally</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a>
      <ul>
        <li><a href="#init">Init</a></li>
        <li><a href="#queries">Queries</a></li>
        <li><a href="#handle-functions">Handle functions</a></li>
      </ul>
    </li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->
## About The Project

[![Product Name Screen Shot][product-screenshot]](https://btn.group/secret_network/silk_pay_demo)

This is a smart contract for Silk Pay - a privacy-preserving payment application built on Secret Network that
introduces a new sender and receiver confirmation architecture. Using the send request and receive request architecture,
Silk Pay ensures outbound capital both safely and consistently sent to the correct location,
enabling peace of mind and usability for everyday users.

<p align="right">(<a href="#top">back to top</a>)</p>

### Built With

* [Cargo](https://doc.rust-lang.org/cargo/)
* [Rust](https://www.rust-lang.org/)
* [secret-toolkit](https://github.com/scrtlabs/secret-toolkit)

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

To get a local copy up and running follow these simple example steps.

### Prerequisites

* Download and install secretcli: https://docs.scrt.network/cli/install-cli.html
* Setup developer blockchain and Docker: https://docs.scrt.network/dev/developing-secret-contracts.html#personal-secret-network-for-secret-contract-development

### Setting up locally

Do this on the command line (terminal etc) in this folder.

1. Run chain locally and make sure to note your wallet addresses.

```sh
docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v $(pwd):/root/code --name secretdev enigmampc/secret-network-sw-dev
```

2. Access container via separate terminal window

```sh
docker exec -it secretdev /bin/bash

# cd into code folder
cd code
```

3. Store contract

```sh
# Store contracts required for test
secretcli tx compute store snip-20-reference-impl.wasm.gz --from a --gas 3000000 -y --keyring-backend test
secretcli tx compute store sn-silk-pay.wasm.gz --from a --gas 3000000 -y --keyring-backend test

# Get the contract's id
secretcli query compute list-code
```

4. Initiate SNIP-20 contracts and set viewing keys (make sure you substitute the wallet and contract addressses as required)

```sh
# Init SNIP-20 (SSCRT)
CODE_ID=1
INIT='{ "name": "SSCRT", "symbol": "SSCRT", "decimals": 6, "initial_balances": [{ "address": "secret1mmhhzccndqplwp9juj6z3hy0eaqh4pf395e2my", "amount": "1000000000000000000" }, { "address": "secret1pt9psved7z8hygryv7wyyur64rumys9ugj6n9w", "amount": "1000000000000000000" }], "prng_seed": "RG9UaGVSaWdodFRoaW5nLg==", "config": { "public_total_supply": true, "enable_deposit": true, "enable_redeem": true, "enable_mint": false, "enable_burn": false } }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SSCRT" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Set viewing key for SSCRT
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "BUTT2022."}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "BUTT2022."}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Init SHADE (SHD)
INIT='{ "name": "SHADE", "symbol": "SHD", "decimals": 8, "initial_balances": [{ "address": "secret1mmhhzccndqplwp9juj6z3hy0eaqh4pf395e2my", "amount": "2000000000000000000" }, { "address": "secret1pt9psved7z8hygryv7wyyur64rumys9ugj6n9w", "amount": "2000000000000000000" }], "prng_seed": "RG9UaGVSaWdodFRoaW5nLg==", "config": { "public_total_supply": true, "enable_deposit": false, "enable_redeem": false, "enable_mint": false, "enable_burn": false } }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SHADE" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Set viewing key for SHADE
secretcli tx compute execute secret1hqrdl6wstt8qzshwc6mrumpjk9338k0lpsefm3 '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "BUTT2022."}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
secretcli tx compute execute secret1hqrdl6wstt8qzshwc6mrumpjk9338k0lpsefm3 '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "BUTT2022."}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Init SILK (SILK)
INIT='{ "name": "SILK", "symbol": "SILK", "decimals": 8, "initial_balances": [{ "address": "secret1mmhhzccndqplwp9juj6z3hy0eaqh4pf395e2my", "amount": "3000000000000000000" }, { "address": "secret1pt9psved7z8hygryv7wyyur64rumys9ugj6n9w", "amount": "3000000000000000000" }], "prng_seed": "RG9UaGVSaWdodFRoaW5nLg==", "config": { "public_total_supply": true, "enable_deposit": false, "enable_redeem": false, "enable_mint": false, "enable_burn": false } }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SILK" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt

# Set viewing key for SILK
secretcli tx compute execute secret18r5szma8hm93pvx6lwpjwyxruw27e0k57tncfy '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "BUTT2022."}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
secretcli tx compute execute secret18r5szma8hm93pvx6lwpjwyxruw27e0k57tncfy '{"set_viewing_key": {"key": "DoTheRightThing.", "padding": "BUTT2022."}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

5. Initialize Silk Pay (make sure you substitute the wallet and contract addressses as required)

```sh
# Init SILK Pay
CODE_ID=2
INIT='{ "fee": "500000", "shade": {"address": "secret1hqrdl6wstt8qzshwc6mrumpjk9338k0lpsefm3", "contract_hash": "35F5DB2BC5CD56815D10C7A567D6827BECCB8EAF45BC3FA016930C4A8209EA69"}, "sscrt": {"address": "secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg", "contract_hash": "35F5DB2BC5CD56815D10C7A567D6827BECCB8EAF45BC3FA016930C4A8209EA69"}, "treasury_address": "secret1fwulevfv3cs4ec3rzv9cthu97pf6us00rzmdex" }'
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "SILK Pay" -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- USAGE EXAMPLES -->
## Usage

You can decode and encode the msg used in the send functions below via https://www.base64encode.org/

### Init

| Name             | Type           | Description                                       | Optional |
|------------------|----------------|---------------------------------------------------|----------|
| fee              | Uint128        | sscrt fee for using safe send and receive request | no       |
| shade            | SecretContract | to verify user's viewing key to view txs          | no       |
| sscrt            | SecretContract |                                                   | no       |
| treasury_address | HumanAddr      | fee sent here when sender sends payment           | no       |

### Queries

1. Query config

``` sh
secretcli query compute query secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"config": {}}'
```
##### Response
```json
{
  "config": {
    "admin": "HumanAddr",
    "fee": "Uint128",
    "new_admin_nomination": "HumanAddr",
    "shade": "SecretContract",
    "sscrt": "SecretContract",
    "treasury_address": "HumanAddr"
  }
}
```

2. Query user's txs

| Name      | Type      | Description                    | Optional |
|-----------|-----------|--------------------------------|----------|
| address   | HumanAddr | address of user                | no       |
| key       | String    | user's SHD token viewing key   | no       |
| page      | u32       | page number starting from zero | no       |
| page_size | u32       | number of txs per page         | no       |

``` sh
secretcli query compute query secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"txs": {"address": "secret1mmhhzccndqplwp9juj6z3hy0eaqh4pf395e2my", "key": "DoTheRightThing.", "page": 0, "page_size": 50}}'
```
##### Response
```json
{
  "txs": {
    "txs": "Vec<HumanizedTx>",
    "total": "Option<u64>",
  }
}
```

### Handle functions

1. Nominate new admin

* Admin only

| Name    | Type      | Description                     | Optional |
|---------|-----------|---------------------------------|----------|
| address | HumanAddr | address of new admin nomination | no       |

``` sh
# Nominate new admin
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"nominate_new_admin":{"address": "secret1pt9psved7z8hygryv7wyyur64rumys9ugj6n9w"}}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

2. Accept admin nomination

* Can only be called by nominated address and no params required

``` sh
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"accept_new_admin_nomination":{}}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

3. Update fee

* Admin only
* Each tx keeps a track of the fee that was paid, so this can be changed without any concern

| Name | Type    | Description   | Optional |
|------|---------|---------------|----------|
| fee  | Uint128 | new sscrt fee | no       |

``` sh
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"update_fee":{ "fee": "555" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

4. Update treasury address

* Admin only

| Name    | Type      | Description             | Optional |
|---------|-----------|-------------------------|----------|
| address | HumanAddr | address to send fees to | no       |

``` sh
secretcli tx compute execute secret1vjecguu37pmd577339wrdp208ddzymku0apnlw '{"update_treasury_address":{ "address": "secret1fwulevfv3cs4ec3rzv9cthu97pf6us00rzmdex" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

5. Create send request

* via SSCRT
* Sender creates a Safe Send Tx, sends fee in SSCRT, sets details of Tx.
* If token is not registered, it is registered.
* Tx status is 0 (pending address confirmation).

| Name        | Type           | Description         | Optional |
|-------------|----------------|---------------------|----------|
| address     | HumanAddr      | address of receiver | no       |
| description | String         | description for tx  | yes      |
| send_amount | Uint128        | amount to send      | no       |
| token       | SecretContract | token to send       | no       |

``` sh
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"send": { "recipient": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw", "amount": "555", "msg": "eyJjcmVhdGVfc2VuZF9yZXF1ZXN0IjogeyJhZGRyZXNzIjogInNlY3JldDFtbWhoemNjbmRxcGx3cDlqdWo2ejNoeTBlYXFoNHBmMzk1ZTJteSIsICJzZW5kX2Ftb3VudCI6ICI1NTU1NTUiLCAiZGVzY3JpcHRpb24iOiAiYXBvY2FseXB0byIsICJ0b2tlbiI6IHsiYWRkcmVzcyI6ICJzZWNyZXQxOHI1c3ptYThobTkzcHZ4Nmx3cGp3eXhydXcyN2UwazU3dG5jZnkiLCAiY29udHJhY3RfaGFzaCI6ICIzNUY1REIyQkM1Q0Q1NjgxNUQxMEM3QTU2N0Q2ODI3QkVDQ0I4RUFGNDVCQzNGQTAxNjkzMEM0QTgyMDlFQTY5In19fQ==" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

6. Confirm address
* via SSCRT
* Tx status updated to 1 (pending payment).

| Name     | Type | Description                       | Optional |
|----------|------|-----------------------------------|----------|
| position | u32  | position of Tx in user's Tx array | no       |

``` sh
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"send": { "recipient": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw", "amount": "0", "msg": "eyJjb25maXJtX2FkZHJlc3MiOiB7InBvc2l0aW9uIjogMH19" }}' --from a -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

7. Create receive request

* via SSCRT
* Receiver create a Receive Request Tx via SSCRT, sends fee in SSCRT, sets details of Tx.
* If token is not registered, it is registered.
* Tx status is 1 (pending payment).

| Name        | Type           | Description         | Optional |
|-------------|----------------|---------------------|----------|
| address     | HumanAddr      | address of sender   | no       |
| description | String         | description for tx  | yes      |
| send_amount | Uint128        | amount to send      | no       |
| token       | SecretContract | token to send       | no       |

``` sh
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"send": { "recipient": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw", "amount": "555", "msg": "eyJjcmVhdGVfcmVjZWl2ZV9yZXF1ZXN0IjogeyJhZGRyZXNzIjogInNlY3JldDFtbWhoemNjbmRxcGx3cDlqdWo2ejNoeTBlYXFoNHBmMzk1ZTJteSIsICJzZW5kX2Ftb3VudCI6ICI1NTU1NTUiLCAiZGVzY3JpcHRpb24iOiAiYXBvY2FseXB0byByZWNlaXZlIHJlcXVlc3QiLCAidG9rZW4iOiB7ImFkZHJlc3MiOiAic2VjcmV0MThyNXN6bWE4aG05M3B2eDZsd3Bqd3l4cnV3MjdlMGs1N3RuY2Z5IiwgImNvbnRyYWN0X2hhc2giOiAiMzVGNURCMkJDNUNENTY4MTVEMTBDN0E1NjdENjgyN0JFQ0NCOEVBRjQ1QkMzRkEwMTY5MzBDNEE4MjA5RUE2OSJ9fX0=" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

8. Send payment

* If sender sends the correct token and amount, the contract forwards payment to the receiver and sends the fee to the treasury.
* Tx status updated to 3 (paid).

| Name     | Type | Description                       | Optional |
|----------|------|-----------------------------------|----------|
| position | u32  | position of Tx in user's Tx array | no       |

``` sh
secretcli tx compute execute secret18r5szma8hm93pvx6lwpjwyxruw27e0k57tncfy '{"send": { "recipient": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw", "amount": "555555", "msg": "eyJzZW5kX3BheW1lbnQiOiB7InBvc2l0aW9uIjogMH19" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

9. Cancel

* via SSCRT
* Either party can cancel the Tx and the fee is sent back to the creator.
* Tx status updated to 2 (cancelled).

| Name     | Type | Description                       | Optional |
|----------|------|-----------------------------------|----------|
| position | u32  | position of Tx in user's Tx array | no       |

``` sh
secretcli tx compute execute secret18vd8fpwxzck93qlwghaj6arh4p7c5n8978vsyg '{"send": { "recipient": "secret1vjecguu37pmd577339wrdp208ddzymku0apnlw", "amount": "0", "msg": "eyJjYW5jZWwiOiB7InBvc2l0aW9uIjogMX19" }}' --from b -y --keyring-backend test --gas 3000000 --gas-prices=3.0uscrt
```

<p align="right">(<a href="#top">back to top</a>)</p>

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->
[product-screenshot]: images/screenshot.png
