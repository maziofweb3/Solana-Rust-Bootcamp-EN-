use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, PartialEq)]
pub enum BankError {
    #[error("Deposit or withdrawal amount must be greater than zero")]
    ZeroAmount,
    #[error("Insufficient funds for withdrawal")]
    InsufficientFunds,
    #[error("Account already initialized")]
    AlreadyInitialized,
    #[error("Account not initialized")]
    NotInitialized,
}

impl From<BankError> for ProgramError {
    fn from(e: BankError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub struct BankAccount {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub lamports: u64,
    pub history: Vec<String>,
}

impl BankAccount {
    pub fn new(owner: Pubkey) -> Self {
        Self {
            is_initialized: true,
            owner,
            lamports: 0,
            history: Vec::new(),
        }
    }

    pub fn deposit(&mut self, amount: u64) -> Result<(), BankError> {
        if amount == 0 {
            return Err(BankError::ZeroAmount);
        }
        self.lamports += amount;
        self.history.push(format!("deposit {amount}"));
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u64) -> Result<(), BankError> {
        if amount == 0 {
            return Err(BankError::ZeroAmount);
        }
        if amount > self.lamports {
            return Err(BankError::InsufficientFunds);
        }
        self.lamports -= amount;
        self.history.push(format!("withdraw {amount}"));
        Ok(())
    }

    pub fn balance_in_sol(&self) -> u64 {
        self.lamports / 1_000_000_000
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq, Clone)]
pub enum BankInstruction {
    Initialize,
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = BankInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        BankInstruction::Initialize => process_initialize(program_id, accounts),
        BankInstruction::Deposit { amount } => process_deposit(program_id, accounts, amount),
        BankInstruction::Withdraw { amount } => process_withdraw(program_id, accounts, amount),
    }
}

fn process_initialize(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bank_account_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !bank_account_info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Ok(existing) = BankAccount::try_from_slice(&bank_account_info.data.borrow()) {
        if existing.is_initialized {
            return Err(BankError::AlreadyInitialized.into());
        }
    }

    let bank_account = BankAccount::new(*owner_info.key);
    bank_account.serialize(&mut &mut bank_account_info.data.borrow_mut()[..])?;
    Ok(())
}

fn process_deposit(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bank_account_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !bank_account_info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut bank_account = BankAccount::try_from_slice(&bank_account_info.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if !bank_account.is_initialized {
        return Err(BankError::NotInitialized.into());
    }

    if bank_account.owner != *owner_info.key {
        return Err(ProgramError::IllegalOwner);
    }

    bank_account.deposit(amount)?;
    bank_account.serialize(&mut &mut bank_account_info.data.borrow_mut()[..])?;
    Ok(())
}

fn process_withdraw(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let bank_account_info = next_account_info(account_info_iter)?;
    let owner_info = next_account_info(account_info_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if !bank_account_info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut bank_account = BankAccount::try_from_slice(&bank_account_info.data.borrow())
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if !bank_account.is_initialized {
        return Err(BankError::NotInitialized.into());
    }

    if bank_account.owner != *owner_info.key {
        return Err(ProgramError::IllegalOwner);
    }

    bank_account.withdraw(amount)?;
    bank_account.serialize(&mut &mut bank_account_info.data.borrow_mut()[..])?;
    Ok(())
}
