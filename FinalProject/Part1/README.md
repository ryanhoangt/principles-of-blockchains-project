# Part 1: Creating price feeds

To reflect the value of other assets, we need to first obtain price feeds before minting synthetic assets. While everyone can get the latest price of a stock on NASDAQ and put it on the chain, data consumers may not want to trust any single data provider.

**Data oracles** provide a decentralized and trustworthy way for blockchains to access external data sources. Resources external to the blockchain are considered "off-chain" while data stored on the blockchain is considered on-chain. Oracle is an additional piece of infrastructure to bridge the two environments.

In this part, we will use one of the most popular oracle solutions, [Chainlink](https://docs.chain.link/), to create price feeds for our synthetic tokens.

## Testnet and wallet

To access the service provided by chainlink in the easiest way, we will use a public blockchain to deploy our smart contracts. In part 0, remember that when deploying contracts, some accounts are automatically generated, each with 100 ETH. These accounts belong to a private testnet blockchain where we can test our applications locally but can not interact with other contracts online. In this part, we need to create some public accounts in **Goerli Testnet** and use **Metamask** to manage them.

1. Install [MetaMask](https://chrome.google.com/webstore/detail/metamask/nkbihfbeogaeaoehlefnkodbefgpgknn) on Chrome, follow the instructions on the app to create a new wallet. After entering the correct phrases, a new account will be created automatically. You can create any number of accounts by clicking the upper right icon and _Create Account_.
2. Switch to Goerli Testnet: click the _Ethereum Mainnet_ at the top right corner of the wallet page and turn on the testnet list by setting _Show/hide test networks_. Switch the network to _Goerli Test Network_.
3. Get some free ETH: go to a [faucet](https://faucets.chain.link/) and enter your address, you will get 0.1 ETH for testing.
4. Open [Remix](https://remix.ethereum.org/) in your web browser, in the _Deploy & run transactions_ tab, set the environment to _Injected Web3_. This will launch a popup page to connect with your wallet.

## Price feed interface

We have provided the interface of the price feed smart contract in `interfaces/IPriceFeed.sol`, you need to implement your `PriceFeed.sol` and deploy one instance for each synthetic asset to provide their prices with respect to USD. You can refer to [this tutorial](https://docs.chain.link/docs/get-the-latest-price/) for help. The proxy addresses of each asset in Goerli are provided below:

```
LINK / USD (instead of BNB / USD): 0x48731cF7e84dc94C5f84577882c14Be11a5B7456
JPY / USD (instead of TSLA / USD): 0x982B232303af1EFfB49939b81AD6866B2E4eeD0B
```

1. There is only one function defined in the interface, you are required to implement it to provide the requested information. You can design other parts of the contract as you like.
2. Deploy the price feed contract for each asset, test the interface and copy their addresses. Once the deployment transactions are confirmed, you are able to find the deployed contracts in [etherscan](https://goerli.etherscan.io/) with https://goerli.etherscan.io/address/{:your_contract_address}.

## PriceFeed Deployment Address

```
LINK / USD PriceFeed Contract: 0xf16890Cef15e8789DB302530494f110c4555a750
JPY  / USD PriceFeed Contract: 0xd23c39dDf5c04C7dC65f935e588dbE7cD110343C
```
