# Stylus ERC20 Token - 使用说明

## 概述

这是一个基于 Arbitrum Stylus SDK 0.10.0 实现的 ERC20 代币合约，包含完整的 ERC20 标准功能和 Ownable 权限控制。

## 功能特性

### ERC20 标准功能
- ✅ `name()` - 获取代币名称
- ✅ `symbol()` - 获取代币符号
- ✅ `decimals()` - 获取小数位数
- ✅ `totalSupply()` - 获取总供应量
- ✅ `balanceOf(address)` - 查询地址余额
- ✅ `transfer(address, uint256)` - 转账
- ✅ `transferFrom(address, address, uint256)` - 授权转账
- ✅ `approve(address, uint256)` - 授权额度
- ✅ `allowance(address, address)` - 查询授权额度

### Ownable 权限控制
- ✅ `constructor()` - 构造函数，部署者自动成为所有者
- ✅ `owner()` - 获取当前所有者地址
- ✅ `transferOwnership(address)` - 转移所有权（仅所有者）
- ✅ `renounceOwnership()` - 放弃所有权（仅所有者）

### 受保护的铸造/销毁功能
- 🔒 `mint(uint256)` - 铸造代币到调用者地址（仅所有者）
- 🔒 `mintTo(address, uint256)` - 铸造代币到指定地址（仅所有者）
- 🔒 `burn(uint256)` - 销毁调用者的代币（仅所有者）

## 架构设计

### Trait-based 继承模式

合约采用 Stylus SDK 0.10.0 推荐的 trait-based 模式：

```rust
// 1. 定义 IErc20 trait
#[public]
pub trait IErc20 {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    // ... 其他方法
}

// 2. 为主合约实现 trait
#[public]
impl IErc20 for StylusToken {
    // 实现所有 ERC20 方法
}

// 3. 使用 #[implements] 声明继承
#[public]
#[implements(IErc20)]
impl StylusToken {
    // 自定义方法：mint, burn 等
}
```

### 权限控制实现

```rust
// 内部方法：检查调用者是否为所有者
fn only_owner(&self) -> Result<(), OwnableError> {
    let caller = self.vm().msg_sender();
    let current_owner = self.owner.get();

    if caller != current_owner {
        return Err(OwnableError::OwnableUnauthorizedAccount(...));
    }
    Ok(())
}

// 在受保护的方法中使用
pub fn mint(&mut self, value: U256) -> Result<(), TokenError> {
    self.only_owner()?;  // 权限检查
    self.erc20.mint(self.vm().msg_sender(), value)?;
    Ok(())
}
```

## 错误处理

合约定义了统一的错误类型 `TokenError`，包含：

### Ownable 错误
- `OwnableUnauthorizedAccount(address)` - 非所有者尝试调用受保护方法
- `OwnableInvalidOwner(address)` - 无效的所有者地址（零地址）

### ERC20 错误
- `InsufficientBalance(address, uint256, uint256)` - 余额不足
- `InsufficientAllowance(address, address, uint256, uint256)` - 授权额度不足

## 事件

### OwnershipTransferred
```solidity
event OwnershipTransferred(address indexed previous_owner, address indexed new_owner);
```
当所有权转移时触发。

### Transfer
```solidity
event Transfer(address indexed from, address indexed to, uint256 value);
```
当代币转移时触发（包括铸造和销毁）。

### Approval
```solidity
event Approval(address indexed owner, address indexed spender, uint256 value);
```
当授权额度变更时触发。

## 部署流程

### 1. 构建合约
```bash
cargo stylus build
```

### 2. 检查合约
```bash
cargo stylus check
```

### 3. 部署到网络
```bash
# 部署到 Arbitrum Sepolia 测试网
cargo stylus deploy \
  --private-key=<YOUR_PRIVATE_KEY> \
  --endpoint=https://sepolia-rollup.arbitrum.io/rpc
```

### 4. 导出 ABI
```bash
cargo stylus export-abi > abi.json
```

## 使用示例

### 从 Solidity 调用

```solidity
// 导入接口
import "./IStylusToken.sol";

contract Example {
    IStylusToken public token;

    constructor(address tokenAddress) {
        token = IStylusToken(tokenAddress);
    }

    // 查询余额
    function getBalance(address account) public view returns (uint256) {
        return token.balanceOf(account);
    }

    // 转账
    function doTransfer(address to, uint256 amount) public {
        token.transfer(to, amount);
    }

    // 仅所有者可以铸造
    function mintTokens(uint256 amount) public {
        token.mint(amount);  // 如果调用者不是 owner，会 revert
    }
}
```

### 从 JavaScript 调用

```javascript
const { ethers } = require('ethers');

// 连接到合约
const token = new ethers.Contract(
    tokenAddress,
    abi,
    signer
);

// 查询信息
const name = await token.name();
const symbol = await token.symbol();
const decimals = await token.decimals();
const totalSupply = await token.totalSupply();
const balance = await token.balanceOf(userAddress);

// 转账
await token.transfer(recipientAddress, ethers.parseUnits("100", decimals));

// 授权
await token.approve(spenderAddress, ethers.parseUnits("500", decimals));

// 仅所有者：铸造代币
const owner = await token.owner();
if (signerAddress === owner) {
    await token.mint(ethers.parseUnits("1000", decimals));
    await token.mintTo(recipientAddress, ethers.parseUnits("500", decimals));
}

// 仅所有者：转移所有权
await token.transferOwnership(newOwnerAddress);
```

## 安全注意事项

1. **所有者权限**：mint、mintTo、burn 方法只能由所有者调用
2. **零地址检查**：不能将所有权转移给零地址（除非调用 renounceOwnership）
3. **余额检查**：转账和销毁操作会检查余额是否足够
4. **授权检查**：transferFrom 会检查授权额度

## 合约信息

- **SDK 版本**: stylus-sdk 0.10.0
- **合约大小**: ~12.7 KB
- **部署费用**: ~0.000095 ETH (以 20% 缓冲)
- **代币配置**:
  - 名称: "Rust Stylus Token(demo)"
  - 符号: "STY(demo)"
  - 小数位: 2

## 技术亮点

1. ✅ 使用 trait-based 继承模式（符合 SDK 0.10.0 最佳实践）
2. ✅ 统一的错误处理机制
3. ✅ 完整的权限控制系统
4. ✅ 标准的 Solidity 事件和错误
5. ✅ 构造函数自动初始化所有者
6. ✅ 完全兼容 ERC20 标准

## 相关链接

- [Arbitrum Stylus 文档](https://docs.arbitrum.io/stylus/gentle-introduction)
- [Stylus SDK GitHub](https://github.com/OffchainLabs/stylus-sdk-rs)
- [ERC20 标准](https://eips.ethereum.org/EIPS/eip-20)
