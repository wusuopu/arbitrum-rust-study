一个简单的计数合约程序

## 编译、部署
```bash
cargo stylus build
cargo stylus deploy --endpoint=http://127.0.0.1:8547 --private-key=<YOUR_PRIVATE_KEY>
```

## 调用合约
```bash
cast send --rpc-url 'http://localhost:8547' --private-key=<YOUT_PRIVATE_KEY> <contract_address> "setNumber(uint256)" 1234
cast call --rpc-url 'http://localhost:8547' <contract_address> "number()(uint256)"
```
