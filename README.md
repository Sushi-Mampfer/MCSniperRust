## How it works
Four accounts send a request to the Mojang API to check if the name is available. If it is, another account claims it. This circumvents the strict rate limit on the claiming endpoint by utilizing the lower rate limit on the checking endpoint. Even with only one account MCSniperRust is as fast as MCSniperGo with over 200(assuming a droptime of over 24h).

## Usage
- Download and run the latest release
- Add one account into the `Claim Account` field, the format is `email:password` or `bearer token`
- Add some accounts into the `Checker Accounts` field, the format is `email:password` or `bearer token`
- Optionally add proxies to the `Proxies` field, one for every 3-4 checker accounts is recommended, you can use more if you want faster auth, the format is `protocol://user:pass@ip:port` or `protocol://ip:port`
- Enter your desired name in the "Name" field
- Press the "Start" button

## The best part
The checker accounts don't need to own minecraft!

## How to get bearer tokens
Check out [this site](https://kqzz.github.io/mc-bearer-token/)

## TODO
- [ ] Add a custom icon
- [ ] Error handeling for non working proxies
- [ ] OAuth2 authentication

## Ratelimits
> [!IMPORTANT]
> The ratelimits are exaggerated by a bit to prevent hitting them because of lag. There could exist more ratelimits when combining one account with multiple IPs. But I don't have the time nor the resources to test this currently.
#### IP
The ratelimit per IP is `20 requests every 60 seconds`.  
There seems to be no rule how these requests need to be spaced out.

#### Account
The ratelimit per account is `30 requests every 300 seconds`.  
And there is one that doesn't allow you to spam more than 20, but the time to reset seems very low, it's way lower than the time it'll take with spaced out requests.

## Credits
- [MCSniperGo](https://github.com/Kqzz/MCsniperGO), for the idea
- [This fork](https://github.com/impliedgg/MCsniperGO) of MCSniperGo, for the auth
- ATTACH, from the MCSniperGo/Forge Sniping dc, for general information
- [The Mojang API docs](https://mojang-api-docs.gapple.pw/), for information about the API

## Bugs
There are probably some bugs, if you find any please make an issue or even better a pull request.

## Demo

I can't show it changing the name, because I only have 2 accounts and both have decent names.