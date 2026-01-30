// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]

extern crate alloc;
mod erc20;

use crate::erc20::{Erc20, Erc20Error, Erc20Params, IErc20, InsufficientBalance, InsufficientAllowance};
use alloc::string::String;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::prelude::*;

// 定义 Ownable 相关的错误和事件
sol! {
    // 所有者转移事件
    event OwnershipTransferred(address indexed previous_owner, address indexed new_owner);

    // 错误：调用者不是所有者
    error OwnableUnauthorizedAccount(address account);

    // 错误：无效的所有者地址
    error OwnableInvalidOwner(address owner);
}

// 定义 Ownable 错误枚举
#[derive(SolidityError)]
pub enum OwnableError {
    OwnableUnauthorizedAccount(OwnableUnauthorizedAccount),
    OwnableInvalidOwner(OwnableInvalidOwner),
}

// 定义统一的错误类型，包含所有可能的错误
#[derive(SolidityError)]
pub enum TokenError {
    OwnableUnauthorizedAccount(OwnableUnauthorizedAccount),
    OwnableInvalidOwner(OwnableInvalidOwner),
    InsufficientBalance(InsufficientBalance),
    InsufficientAllowance(InsufficientAllowance),
}

// 实现从 OwnableError 到 TokenError 的转换
impl From<OwnableError> for TokenError {
    fn from(err: OwnableError) -> Self {
        match err {
            OwnableError::OwnableUnauthorizedAccount(e) => TokenError::OwnableUnauthorizedAccount(e),
            OwnableError::OwnableInvalidOwner(e) => TokenError::OwnableInvalidOwner(e),
        }
    }
}

// 实现从 Erc20Error 到 TokenError 的转换
impl From<Erc20Error> for TokenError {
    fn from(err: Erc20Error) -> Self {
        match err {
            Erc20Error::InsufficientBalance(e) => TokenError::InsufficientBalance(e),
            Erc20Error::InsufficientAllowance(e) => TokenError::InsufficientAllowance(e),
        }
    }
}

struct StylusTokenParams;
impl Erc20Params for StylusTokenParams {
    const NAME: &'static str = "Rust Stylus Token(demo)";
    const SYMBOL: &'static str = "STY(demo)";
    const DECIMALS: u8 = 2;
}

sol_storage! {
    #[entrypoint]
    struct StylusToken {
        #[borrow]
        Erc20<StylusTokenParams> erc20;
        address owner;
    }
}

// StylusToken 的内部方法（非 public）
impl StylusToken {
    /// 检查调用者是否为所有者
    fn only_owner(&self) -> Result<(), OwnableError> {
        let caller = self.vm().msg_sender();
        let current_owner = self.owner.get();

        if caller != current_owner {
            return Err(OwnableError::OwnableUnauthorizedAccount(
                OwnableUnauthorizedAccount { account: caller }
            ));
        }

        Ok(())
    }

    /// 内部方法：转移所有权
    fn _transfer_ownership(&mut self, new_owner: Address) {
        let previous_owner = self.owner.get();
        self.owner.set(new_owner);

        self.vm().log(OwnershipTransferred {
            previous_owner,
            new_owner,
        });
    }
}

// 为 StylusToken 实现 IErc20 trait
#[public]
impl IErc20 for StylusToken {
    fn name(&self) -> String {
        Erc20::<StylusTokenParams>::get_name()
    }

    fn symbol(&self) -> String {
        Erc20::<StylusTokenParams>::get_symbol()
    }

    fn decimals(&self) -> u8 {
        Erc20::<StylusTokenParams>::get_decimals()
    }

    fn total_supply(&self) -> U256 {
        self.erc20.get_total_supply()
    }

    fn balance_of(&self, address: Address) -> U256 {
        self.erc20.get_balance_of(address)
    }

    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, Erc20Error> {
        self.erc20.do_transfer(to, value)
    }

    fn transfer_from(&mut self, from: Address, to: Address, value: U256) -> Result<bool, Erc20Error> {
        self.erc20.do_transfer_from(from, to, value)
    }

    fn approve(&mut self, spender: Address, value: U256) -> bool {
        self.erc20.do_approve(spender, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.get_allowance(owner, spender)
    }
}

// 实现额外的自定义方法
#[public]
#[implements(IErc20)]
impl StylusToken {
    /// 构造函数 - 初始化合约所有者
    #[constructor]
    pub fn constructor(&mut self) -> Result<(), OwnableError> {
        // https://stylus-by-example.org/basic_examples/constructor
        let deployer = self.vm().tx_origin();

        // 检查地址是否有效
        if deployer == Address::ZERO {
            return Err(OwnableError::OwnableInvalidOwner(
                OwnableInvalidOwner { owner: Address::ZERO }
            ));
        }

        self._transfer_ownership(deployer);
        Ok(())
    }

    /// 获取当前所有者地址
    pub fn owner(&self) -> Address {
        self.owner.get()
    }

    /// 转移所有权（只有当前所有者可以调用）
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), OwnableError> {
        self.only_owner()?;

        if new_owner == Address::ZERO {
            return Err(OwnableError::OwnableInvalidOwner(
                OwnableInvalidOwner { owner: Address::ZERO }
            ));
        }

        self._transfer_ownership(new_owner);
        Ok(())
    }

    /// 放弃所有权（将所有者设置为零地址）
    pub fn renounce_ownership(&mut self) -> Result<(), OwnableError> {
        self.only_owner()?;
        self._transfer_ownership(Address::ZERO);
        Ok(())
    }

    /// 铸造代币（只有所有者可以调用）
    pub fn mint(&mut self, value: U256) -> Result<(), TokenError> {
        self.only_owner()?;
        self.erc20.mint(self.vm().msg_sender(), value)?;
        Ok(())
    }

    /// 铸造代币到指定地址（只有所有者可以调用）
    pub fn mint_to(&mut self, to: Address, value: U256) -> Result<(), TokenError> {
        self.only_owner()?;
        self.erc20.mint(to, value)?;
        Ok(())
    }

    /// 销毁代币（只有销毁持有者自己的代币）
    pub fn burn(&mut self, value: U256) -> Result<(), TokenError> {
        self.erc20.burn(self.vm().msg_sender(), value)?;
        Ok(())
    }
}