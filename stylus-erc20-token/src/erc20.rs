use alloc::string::String;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use core::marker::PhantomData;
use stylus_sdk::prelude::*;

// 定义 Erc20 代币的静态参数
pub trait Erc20Params {
    const NAME: &'static str;   // 代币名称
    const SYMBOL: &'static str; // 代币符号
    const DECIMALS: u8;         // 代币小数位数
}

// 定义 ERC20 标准接口 trait
#[public]
pub trait IErc20 {
    fn name(&self) -> String;
    fn symbol(&self) -> String;
    fn decimals(&self) -> u8;
    fn total_supply(&self) -> U256;
    fn balance_of(&self, address: Address) -> U256;
    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error>;
    fn transfer_from(&mut self, from: Address, to: Address, value: U256) -> Result<bool, Erc20Error>;
    fn approve(&mut self, spender: Address, value: U256) -> bool;
    fn allowance(&self, owner: Address, spender: Address) -> U256;
}

// 定义 Solidity 风格的存储结构
sol_storage! {
    pub struct Erc20<T> {
        mapping(address => uint256) balances; // 余额映射
        mapping(address => mapping(address => uint256)) allowances; // 授权映射
        uint256 total_supply; // 总供应量
        PhantomData<T> phantom; // 泛型参数
    }
}

// 定义 Solidity 风格的事件和错误
sol! {
    // 定义 Transfer 事件
    event Transfer(address indexed from, address indexed to, uint256 value);
    // 定义 Approval 事件
    event Approval(address indexed owner, address indexed spender, uint256 value);

    // 定义错误：余额不足
    #[derive(Debug)]
    error InsufficientBalance(address from, uint256 have, uint256 want);
    // 定义错误：授权不足
    #[derive(Debug)]
    error InsufficientAllowance(address owner, address spender, uint256 have, uint256 want);
}

#[derive(SolidityError)]
pub enum Erc20Error {
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

impl<T: Erc20Params> Erc20<T> {
    // 内部转账
    pub fn _transfer(&mut self, from: Address, to: Address, value: U256) -> Result<(), Erc20Error> {
        let mut sender_balance = self.balances.setter(from);

        let old_sender_balance = sender_balance.get();
        if old_sender_balance < value {
            // 余额不足
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance {
                from,
                have: old_sender_balance,
                want: value,
            }));
        }

        sender_balance.set(old_sender_balance - value);
        let mut to_balance = self.balances.setter(to);
        let new_to_balance = to_balance.get() + value;

        to_balance.set(new_to_balance);

        // 记录转账事件到 EVM 日志
        self.vm().log(Transfer { from, to, value });

        Ok(())
    }

    // 铸造代币
    pub fn mint(&mut self, address: Address, value: U256) -> Result<(), Erc20Error>{
        let mut balance = self.balances.setter(address);

        // 为目标地址增加余额
        let new_balance = balance.get() + value;
        balance.set(new_balance);

        // 增加总供应量
        self.total_supply.set(self.total_supply.get() + value);

        // 记录铸造事件到 EVM 日志
        self.vm().log(Transfer {
            from: Address::ZERO,  // 零地址表示铸造
            to: address,
            value,
        });

        Ok(())
    }

    // 销毁代币
    pub fn burn(&mut self, address: Address, value: U256) -> Result<(), Erc20Error>{
        let mut balance = self.balances.setter(address);

        let old_balance = balance.get();
        if old_balance < value {
            return Err(Erc20Error::InsufficientBalance(InsufficientBalance { from:address, have: old_balance, want: value }));
        }

        balance.set(old_balance - value);
        self.total_supply.set(self.total_supply.get() - value);

        // 记录销毁事件到 EVM 日志
        self.vm().log(Transfer {
            from: address,
            to: Address::ZERO,  // 零地址表示销毁
            value,
        });

        Ok(())
    }
}

// 为 Erc20 实现内部方法
impl<T: Erc20Params> Erc20<T> {
    // 获取代币名称
    pub fn get_name() -> String {
        T::NAME.into()
    }

    // 获取代币符号
    pub fn get_symbol() -> String {
        T::SYMBOL.into()
    }

    // 获取代币小数位数
    pub fn get_decimals() -> u8 {
        T::DECIMALS
    }

    // 获取总供应量
    pub fn get_total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    // 查询给定地址的余额
    pub fn get_balance_of(&self, address: Address) -> U256 {
        self.balances.get(address)
    }

    // 转账
    pub fn do_transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        self._transfer(self.vm().msg_sender(), to, value)?;
        Ok(true)
    }

    // 授权转账
    pub fn do_transfer_from(&mut self, from: Address, to: Address, value: U256) -> Result<bool, Erc20Error> {
        // 获取调用者的授权额度
        let sender = self.vm().msg_sender();
        let mut sender_allowances = self.allowances.setter(from);
        let mut allowance = sender_allowances.setter(sender);

        let old_allowance = allowance.get();
        if old_allowance < value {
            // 授权不足
            return Err(Erc20Error::InsufficientAllowance(InsufficientAllowance {
                owner: from,
                spender: sender,
                have: old_allowance,
                want: value
            }));
        }

        // 消耗授权额度
        allowance.set(old_allowance - value);
        self._transfer(from, to, value)?;

        Ok(true)
    }

    // 授权
    pub fn do_approve(&mut self, spender: Address, value: U256) -> bool {
        self.allowances.setter(self.vm().msg_sender()).insert(spender, value);

        self.vm().log(Approval {
            owner: self.vm().msg_sender(),
            spender,
            value,
        });

        true
    }

    // 查询授权额度
    pub fn get_allowance(&self, owner: Address, spender: Address) -> U256 {
        self.allowances.getter(owner).get(spender)
    }
}