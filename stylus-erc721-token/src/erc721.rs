use alloc::{string::String, vec, vec::Vec};
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use core::{marker::PhantomData};
use stylus_sdk::{prelude::*};

pub trait Erc721Params {
    const NAME: &'static str;       // 不可变 NFT 名称
    const SYMBOL: &'static str;     // 不可变 NFT 符号
    fn token_uri(token_id: U256) -> String; // token uri
}

#[public]
pub trait IERC721 {
    fn name(&self) -> String;
    fn symbol(&self) -> String;

    fn balance_of(&self, address: Address) -> U256;
    fn owner_of(&self, token_id: U256) -> Result<Address, Erc721Error>;
    // fn safe_transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<(), Erc721Error>;
    // fn safe_transfer_from_widh_data(&mut self, from: Address, to: Address, token_id: U256, data: Bytes) -> Result<(), Erc721Error>;
    fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<(), Erc721Error>;
    fn approve(&mut self, spender: Address, token_id: U256) -> Result<(), Erc721Error>;
    fn set_approval_for_all(&mut self, operator: Address, approved: bool,) -> Result<(), Erc721Error>;
    fn get_approved(&mut self, token_id: U256) -> Result<Address, Erc721Error>;
    fn is_approved_for_all(&mut self, owner: Address, operator: Address) -> Result<bool, Erc721Error>;
}

sol_storage! {
    pub struct Erc721<T> {
        mapping(uint256 => address) owners;                 // NFT到地址的映射
        mapping(address => uint256) balances;               // 余额映射
        mapping(uint256 => address) token_approvals;        // 授权映射
        mapping(address => mapping(address => bool)) operator_approvals; // 操作者授权映射
        uint256 total_supply;                               // 总供应量
        PhantomData<T> phantom;
    }
}

sol!{
    event Transfer(address indexed from, address indexed to, uint256 indexed token_id);
    event Approval(address indexed owner, address indexed approved, uint256 indexed token_id);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

    error InvalidTokenId(uint256 token_id);
    error NotOwner(address from, uint256 token_id, address real_owner);
    error NotApproved(address owner, address spender, uint256 token_id);
    error TransferToZero(uint256 token_id);
    error ReceiverRefused(address receiver, uint256 token_id, bytes4 returned);
}

#[derive(SolidityError)]
pub enum Erc721Error {
    InvalidTokenId(InvalidTokenId),
    NotOwner(NotOwner),
    NotApproved(NotApproved),
    TransferToZero(TransferToZero),
    ReceiverRefused(ReceiverRefused),
}

// 外部接口
sol_interface! {
    // 用于调用实现 IERC721TokenReceiver 的合约的 onERC721Received 方法
    interface IERC721TokenReceiver {
        function onERC721Received(address operator, address from, uint256 token_id, bytes data) external returns (bytes4);
    }
}

// 为 ERC-721 实现内部方法
impl <T: Erc721Params> Erc721<T> {
    pub fn require_authorized_to_spend(&self, from: Address, token_id: U256) -> Result<(), Erc721Error> {
        let owner = self.owner_of(token_id)?;
        let sender = self.vm().msg_sender();
        if from != owner {
            return Err(Erc721Error::NotOwner(NotOwner { from, token_id, real_owner: owner }));
        }

        if sender == owner {
            return Ok(());
        }

        // 检查调用者是否为拥有者的操作者
        if self.operator_approvals.getter(owner).get(sender) {
            return Ok(());
        }

        // 检查调用者是否被授权操作此 token
        if sender == self.token_approvals.get(token_id) {
            return Ok(());
        }

        Err(Erc721Error::NotApproved(NotApproved { owner, spender: sender, token_id }))
    }

    pub fn transfer(&mut self, token_id: U256, from: Address, to: Address) -> Result<(), Erc721Error> {
        let mut owner = self.owners.setter(token_id);
        let previous_owner = owner.get();

        // 验证 from 是否为拥有者
        if previous_owner != from {
            return Err(Erc721Error::NotOwner(NotOwner { from, token_id, real_owner: previous_owner }));
        }

        owner.set(to);

        let mut from_balance = self.balances.setter(from);
        let balance = from_balance.get() - U256::from(1);
        from_balance.set(balance);

        let mut to_balance = self.balances.setter(to);
        let balance = to_balance.get() + U256::from(1);
        to_balance.set(balance);

        // 清除 token 的授权
        self.token_approvals.delete(token_id);

        self.vm().log(Transfer { from, to, token_id });

        Ok(())
    }

    pub fn mint(&mut self, to: Address) -> Result<(), Erc721Error> {
        let new_token_id = self.total_supply.get();

        self.total_supply.set(new_token_id + U256::from(1));

        self.transfer(new_token_id, Address::default(), to)?;

        Ok(())
    }

    pub fn burn(&mut self, from: Address, token_id: U256) -> Result<(), Erc721Error> {
        // 执行转账到零地址
        self.transfer(token_id, from, Address::default())?;
        Ok(())
    }
}

#[public]
impl<T: Erc721Params> Erc721<T> {
    pub fn name() -> Result<String, Erc721Error> {
        Ok(T::NAME.into())
    }

    pub fn symbol() -> Result<String, Erc721Error> {
        Ok(T::SYMBOL.into())
    }

    #[selector(name = "tokenURI")]
    pub fn token_uri(&self, token_id: U256) -> Result<String, Erc721Error> {
        self.owner_of(token_id)?;
        Ok(T::token_uri(token_id))
    }

    pub fn balance_of(&self, owner: Address) -> U256 {
        self.balances.get(owner)
    }

    pub fn owner_of(&self, token_id: U256) -> Result<Address, Erc721Error> {
        let owner = self.owners.get(token_id);
        if owner.is_zero() {
            return Err(Erc721Error::InvalidTokenId(InvalidTokenId { token_id }));
        }
        Ok(owner)
    }

    // 普通转账
    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), Erc721Error> {
        // 禁止转账到空地址
        if to.is_zero() {
            return Err(Erc721Error::TransferToZero(TransferToZero { token_id }));
        }
        self.require_authorized_to_spend(from, token_id)?;
        self.transfer(token_id, from, to)?;
        Ok(())
    }

    // 为指定 token 授权
    pub fn approve(&mut self, to: Address, token_id: U256) -> Result<(), Erc721Error> {
        let owner = self.owner_of(token_id)?;
        let sender = self.vm().msg_sender();
        // 判断调用者权限
        if owner != sender && !self.operator_approvals.getter(owner).get(sender){
            return Err(Erc721Error::NotApproved(NotApproved { owner, token_id, spender: sender }));
        }

        self.token_approvals.insert(token_id, to);
        self.vm().log(Approval {
            approved: to,
            owner,
            token_id,
        });
        Ok(())
    }

    // 批量授权
    pub fn set_approval_for_all(
        &mut self,
        operator: Address,
        approved: bool,
    ) -> Result<(), Erc721Error> {
        let owner = self.vm().msg_sender();
        self.operator_approvals.setter(owner).insert(operator, approved);

        self.vm().log(ApprovalForAll {
            operator,
            approved,
            owner,
        });

        Ok(())
    }

    // 获取 token 的授权地址
    pub fn get_approved(&mut self, token_id: U256) -> Result<Address, Erc721Error> {
        Ok(self.token_approvals.get(token_id))
    }

    pub fn is_approved_for_all(
        &mut self,
        owner: Address,
        operator: Address,
    ) -> Result<bool, Erc721Error> {
        Ok(self.operator_approvals.getter(owner).get(operator))
    }
}