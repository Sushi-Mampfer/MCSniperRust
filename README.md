## How it works
4 accounts send a request to the mojang api to check if the name is available. If it is, another account claims it. This circumvents the strict rate limit on the claiming endpoint by utilizing the lower rate limit on the checking endpoint. With four accounts, MCSniperRust is able to claim every name within four seconds of it dropping. To achieve the same performance with MCSniperGo you'd need over 500 accounts.

## Usage
- [Install Tauri](https://tauri.app/start/) and first its [prerequisites](https://tauri.app/start/prerequisites/).
- Download or clone the MCSniperRust repository.
- Go to the root folder (where the `README.md` is located).
- Run `cargo tauri dev`.
- Add at least four accounts into the "Checker Accounts" field, the format is `email:password`.
- Add one Account into the "Claim Account" field, the format is `email:password`.
- Enter your desired name in the "Name" field.
- Press the "Start" button.

## TODO
- [ ] Add a custom icon
- [ ] Compiled releases
- [ ] Support for more than four accounts
- [ ] Proxy Support

## Troubleshooting and Support 
If you have any problems, please message `sushimampfer` on discord.
