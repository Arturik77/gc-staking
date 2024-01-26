use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

declare_id!("Grn2bKrqK9E82UztT87Ldm26WWzsscPE7isYeZtTyZxb");

#[program]
pub mod gc_staking {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, start_time: u64, end_time: u64) -> Result<()> {
        msg!("Contract: Initialize");

        let stake_pool = &mut ctx.accounts.stake_pool;
        stake_pool.owner = ctx.accounts.owner.key();
        stake_pool.start_time = start_time;
        stake_pool.end_time = end_time;
        stake_pool.token = ctx.accounts.stakable_token.key();

        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        msg!("Contract: Stake");

        let stakeholder = &mut ctx.accounts.stakeholder;
        let clock = Clock::get()?;

        if stakeholder.amount > 0 {
            let reward = (clock.slot - stakeholder.deposit) - stakeholder.reward;
            let cpi_accounts = MintTo {
                mint: ctx.accounts.stakable_token.to_account_info(),
                to: ctx.accounts.user_staking_wallet.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::mint_to(cpi_ctx, reward)?;
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_staking_wallet.to_account_info(),
            to: ctx.accounts.owner_staking_wallet.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)?;

        stakeholder.amount += amount;
        stakeholder.deposit = clock.slot;
        stakeholder.reward = 0;

        Ok(())

    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        msg!("Contract: Unstake");

        let stakeholder = &mut ctx.accounts.stakeholder;
        let clock = Clock::get()?;
        let reward = (clock.slot - stakeholder.deposit) - stakeholder.reward;
        let cpi_accounts = MintTo {
            mint: ctx.accounts.stakable_token.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::mint_to(cpi_ctx, reward)?;

        let cpi_accounts = Transfer {
            from: ctx.accounts.owner_staking_wallet.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, stakeholder.amount)?;

        stakeholder.amount = 0;
        stakeholder.deposit = 0;
        stakeholder.reward = 0;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        msg!("Contract: Claim Reward");

        let stakeholder = &mut ctx.accounts.stakeholder;
        let clock = Clock::get()?;
        let reward = (clock.slot - stakeholder.deposit) - stakeholder.reward;
        let cpi_accounts = MintTo {
            mint: ctx.accounts.stakable_token.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::mint_to(cpi_ctx, reward)?;
        stakeholder.reward += reward;
        
        Ok(())
    }

}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(init, payer = owner, space = 8 + PoolInfo::LEN)]
    pub stake_pool: Account<'info, PoolInfo>,
    #[account(mut)]
    pub stakable_token: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub owner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK:
    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(init, payer = user, space = 8 + UserInfo::LEN)]
    pub stakeholder: Account<'info, UserInfo>,
    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub owner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub stakable_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    /// CHECK:
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub stakeholder: Account<'info, UserInfo>,
    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub owner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub stakable_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    /// CHECK:
    #[account(mut)]
    pub user: AccountInfo<'info>,
    /// CHECK:
    #[account(mut)]
    pub owner: AccountInfo<'info>,
    #[account(mut)]
    pub stakeholder: Account<'info, UserInfo>,
    #[account(mut)]
    pub user_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub owner_staking_wallet: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub stakable_token: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
pub struct PoolInfo {
    pub owner: Pubkey,
    pub start_time: u64,
    pub end_time: u64,
    pub token: Pubkey,
}

#[account]
pub struct UserInfo {
    pub amount: u64,
    pub reward: u64,
    pub deposit: u64,
}

impl UserInfo {
    pub const LEN: usize = 8 + 8 + 8;
}

impl PoolInfo {
    pub const LEN: usize = 32 + 8 + 8 + 32;
}