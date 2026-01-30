// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
mod erc721;
extern crate alloc;

use core::{borrow::BorrowMut};

use crate::erc721::{Erc721, Erc721Error, Erc721Params, IERC721TokenReceiver, InvalidTokenId, NotApproved, NotOwner, ReceiverRefused, TransferToZero};
use alloy_primitives::{Address, U256, FixedBytes};
use stylus_sdk::{abi::Bytes, prelude::*};
use alloy_sol_types::sol;

// 定义 onERC721Received 方法的选择器常量
const ERC721_TOKEN_RECEIVER_ID: u32 = 0x150b7a02;

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
    InvalidTokenId(InvalidTokenId),
    NotOwner(NotOwner),
    NotApproved(NotApproved),
    TransferToZero(TransferToZero),
    ReceiverRefused(ReceiverRefused),
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

// 实现从 Erc721Error 到 TokenError 的转换
impl From<Erc721Error> for TokenError {
    fn from(err: Erc721Error) -> Self {
        match err {
            Erc721Error::InvalidTokenId(e) => TokenError::InvalidTokenId(e),
            Erc721Error::NotOwner(e) => TokenError::NotOwner(e),
            Erc721Error::NotApproved(e) => TokenError::NotApproved(e),
            Erc721Error::TransferToZero(e) => TokenError::TransferToZero(e),
            Erc721Error::ReceiverRefused(e) => TokenError::ReceiverRefused(e),
        }
    }
}


struct StylusNFTParams;
impl Erc721Params for StylusNFTParams {
    const NAME: &'static str = "Stylus Rust NFT(demo)";
    const SYMBOL: &'static str = "Rust(demo)";
    fn token_uri(token_id: U256) -> String {
        format!("{}{}{}", "https://external-magenta-alpaca.myfilebase.com/ipfs/QmY47C6mUFEGPGF5muGTEcSD3MPspCSpT2EGJV8QvQGUnV", token_id, ".json")

    }
}

sol_storage! {
    #[entrypoint]
    struct StylusNFT {
        #[borrow]
        Erc721<StylusNFTParams> erc721;
        address owner;
    }
}

// StylusNFT 的内部方法（非 public）
impl StylusNFT {
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


#[public]
impl StylusNFT {
    // ======= ERC721 标准方法 =======
    pub fn name(&self) -> String {
        StylusNFTParams::NAME.into()
    }

    pub fn symbol(&self) -> String {
        StylusNFTParams::SYMBOL.into()
    }

    // #[selector(name = "balanceOf")]
    pub fn balance_of(&self, owner: Address) -> U256 {
        self.erc721.balance_of(owner)
    }

    // #[selector(name = "ownerOf")]
    pub fn owner_of(&self, token_id: U256) -> Result<Address, Erc721Error> {
        self.erc721.owner_of(token_id)
    }

    // #[selector(name = "transferFrom")]
    pub fn transfer_from(&mut self, from: Address, to: Address, token_id: U256) -> Result<(), Erc721Error> {
        self.erc721.transfer_from(from, to, token_id)
    }

    pub fn approve(&mut self, to: Address, token_id: U256) -> Result<(), Erc721Error> {
        self.erc721.approve(to, token_id)
    }

    // #[selector(name = "setApprovalForAll")]
    pub fn set_approval_for_all(&mut self, to: Address, approved: bool) -> Result<(), Erc721Error> {
        self.erc721.set_approval_for_all(to, approved)
    }

    // #[selector(name = "getApproved")]
    pub fn get_approved(&mut self, token_id: U256) -> Result<Address, Erc721Error> {
        self.erc721.get_approved(token_id)
    }

    // #[selector(name = "isApprovedForAll")]
    pub fn is_approved_for_all(&mut self, owner: Address, operator: Address) -> Result<bool, Erc721Error> {
        self.erc721.is_approved_for_all(owner, operator)
    }

    // ======= Ownable 方法 =======
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


    // ======= NFT 的方法 ==========
    fn token_uri(&self, token_id: U256) -> Result<String, TokenError> {
        let uri = self.erc721.token_uri(token_id)?;
        Ok(uri)
    }

    /// 铸造代币（只有所有者可以调用）
    pub fn mint_to(&mut self, to: Address) -> Result<(), TokenError> {
        self.only_owner()?;
        self.erc721.mint(to)?;
        Ok(())
    }
    // 为调用者铸币
    pub fn mint(&mut self) -> Result<(), TokenError> {
        self.erc721.mint(self.vm().msg_sender())?;
        Ok(())
    }
    /// 销毁代币（只有销毁持有者自己的代币）
    pub fn burn(&mut self, token_id: U256) -> Result<(), TokenError> {
        self.erc721.burn(self.vm().msg_sender(), token_id)?;
        Ok(())
    }

    pub fn total_supply(&mut self) -> Result<U256, TokenError> {
        Ok(self.erc721.total_supply.get())
    }

    // 执行带数据的安全转账
    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from_widh_data(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
        data: Bytes,
    ) -> Result<(), TokenError> {
        // 禁止转账到空地址
        if to.is_zero() {
            return Err(TokenError::TransferToZero(TransferToZero { token_id }));
        }
        self.erc721.borrow_mut().require_authorized_to_spend(from, token_id)?;

        self.erc721.borrow_mut().transfer(token_id, from, to)?;

        // 尝试调用 onERC721Received，如果失败说明不是合约或不支持该接口
        let receiver = IERC721TokenReceiver::new(to);
        let sender = self.vm().msg_sender();
        let call = Call::<true>::new_mutating(self);

        match receiver.on_erc_721_received(self.vm(), call, sender, from, token_id, data) {
            Ok(received) => {
                if u32::from_be_bytes(received.0) != ERC721_TOKEN_RECEIVER_ID {
                    return Err(TokenError::ReceiverRefused(ReceiverRefused {
                        receiver: to,
                        token_id,
                        returned: FixedBytes(received.0)
                    }));
                }
            }
            Err(_) => {
                // 如果调用失败，可能不是合约或不支持接口
                // 简化处理：对于 EOA 地址，调用会失败但这是正常的
            }
        }

        Ok(())
    }

    // 执行不带数据的安全转账
    #[selector(name = "safeTransferFrom")]
    pub fn safe_transfer_from(
        &mut self,
        from: Address,
        to: Address,
        token_id: U256,
    ) -> Result<(), TokenError> {
        self.safe_transfer_from_widh_data(from, to, token_id, Bytes::new())?;
        Ok(())
    }

    // 检查是否支持指定接口
    pub fn supports_interface(interface: FixedBytes<4>) -> Result<bool, TokenError> {
        let interface_slice_array: [u8; 4] = interface.as_slice().try_into().unwrap();

        // 特殊处理 ERC165 标准中的 0xffffffff
        if u32::from_be_bytes(interface_slice_array) == 0xffffffff {
            return Ok(false);
        }

        // 定义支持的接口 ID
        const IERC165: u32 = 0x01ffc9a7;
        const IERC721: u32 = 0x80ac58cd;
        const IERC721_METADATA: u32 = 0x5b5e139f;

        Ok(matches!(
            u32::from_be_bytes(interface_slice_array),
            IERC165 | IERC721 | IERC721_METADATA
        ))
    }
}