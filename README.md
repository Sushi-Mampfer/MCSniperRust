## How it works
Four accounts send a request to the mojang api to check if the name is available. If it is, another account claims it. This circumvents the strict rate limit on the claiming endpoint by utilizing the lower rate limit on the checking endpoint. With four accounts, MCSniperRust is able to claim every name within four seconds of it dropping. To achieve the same performance with MCSniperGo you'd need over 500 accounts.

## Usage
- [Install Tauri](https://tauri.app/start/) and first its [prerequisites](https://tauri.app/start/prerequisites/).
- Download or clone the MCSniperRust repository.
- Go to the root folder (where the `README.md` is located).
- Run `cargo tauri dev`.
- Add at least four accounts into the "Checker Accounts" field, the format is `email:password`.
- Add one Account into the "Claim Account" field, the format is `email:password`.
- Enter your desired name in the "Name" field.
- Press the "Start" button.

## How it looks
![grafik](https://github.com/user-attachments/assets/89b1eb24-9c7a-4f84-a5e1-0dd8ae837ffe)

## TODO
- [ ] Add a custom icon
- [ ] Compiled releases
- [ ] Support for more than four accounts
- [ ] Proxy Support

## Ratelimits
> [!IMPORTANT]
> The ratelimits are exaggerated by a bit to prevent hitting them because of lag. There could exist more ratelimits when combinding one account with multiple ips. But I don't have the time nor the resources to test this currently.
#### IP
The ratelimit per IP is `20 requests every 60 seconds`.  
There seems to be no rule how these requests need to be spaced out.

#### Account
The ratelimit per account is `30 requests every 300 seconds`.  
And there is one that doesn't allow you to spam more than 20, but the time to reset seems very low, it's way lower than the time it'll take with spaced out requests.
