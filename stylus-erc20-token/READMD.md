一个兼容 ERC20 的代币合约程序

参考文档： https://docs.arbitrum.io/stylus-by-example/applications/erc20

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
# 铸币 - 只有合约的所有者才有权限调用
cast send --rpc-url $ARB_RPC_URL --private-key $PRIVATE_KEY <contract_address> "mint(uint256)" 9000000

# 查询余额
cast erc20 balance  --rpc-url $ARB_RPC_URL <contract_address> $PUBLIC_KEY

# 转账
cast send --rpc-url $ARB_RPC_URL --private-key $PRIVATE_KEY <contract_address> "transfer(address, uint256)" <to_address> <amount>
```