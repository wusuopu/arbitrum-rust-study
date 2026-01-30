一个兼容 ERC721 的代币合约程序

参考文档： https://docs.arbitrum.io/stylus-by-example/applications/erc721

创建一个 `.env` 文件
```
export ARB_RPC_URL=https://sepolia-rollup.arbitrum.io/rpc
export PRIVATE_KEY=YOUR_PRIVATE_KEY
export PUBLIC_KEY=YOUR_PUBLIC_KEY
```

## 编译合约

```bash
cargo stylus build
```

## 部署合约

```bash
source .env
cargo stylus deploy --endpoint=$ARB_RPC_URL --private-key=$PRIVATE_KEY
```

## 调用合约

```bash
# 为调用者铸币
cast send --rpc-url $ARB_RPC_URL --private-key $PRIVATE_KEY <contract_address> "mint()"

# 查询余额
cast call --rpc-url $ARB_RPC_URL <contract_address> "balanceOf(address)(uint256)" $PUBLIC_KEY

# 查看对应 token 的拥有者
cast call --rpc-url $ARB_RPC_URL <contract_address> "ownerOf(uint256)(address)" <token_id>
```