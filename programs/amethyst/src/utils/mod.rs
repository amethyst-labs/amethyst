pub mod conversions;
pub mod fees;
pub mod oracle;
pub mod price;

pub use conversions::*;
pub use fees::*;
pub use oracle::*;
pub use price::*;

use anchor_lang::prelude::*;

pub fn with_signer_pda<'info, T: ToAccountInfo<'info>>(acc_info: &T) -> AccountInfo<'info> {
    let mut acc_info = acc_info.to_account_info();
    acc_info.is_signer = true;
    acc_info
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    pub fn test_with_signer_pda() -> Result<()> {
        let mut lamports = 0;
        let mut data = vec![];
        let account_info = AccountInfo {
            key: &Pubkey::default(),
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(&mut lamports)),
            data: Rc::new(RefCell::new(&mut data)),
            owner: &Pubkey::default(),
            executable: false,
            rent_epoch: 0,
        };
        let signer_account_info = with_signer_pda(&account_info);
        assert!(signer_account_info.is_signer);
        Ok(())
    }
}
