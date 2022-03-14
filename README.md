# Silk Pay
## Design choices
### Admin 
* Admin can change the fee.
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

## References
1. Silk Pay description: https://github.com/securesecrets/ShadeGrants/issues/1
2. Secret contracts guide: https://github.com/enigmampc/secret-contracts-guide
