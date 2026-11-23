use anchor_lang::prelude::*;

// Se actualiza automaticamente al compilar en Solana Playground.
declare_id!("Chw3CkaxkRafhHTeJ85DQQJm4frpFwdhqCVtbkdZGtpF");

const MAX_ID_LEN: usize = 32;

#[program]
pub mod aleman_iuslex_ai {
    use super::*;

    // 1. CREAR EXPEDIENTE
    pub fn initialize_case(ctx: Context<InitializeCase>, case_id: String) -> Result<()> {
        require!(
            !case_id.is_empty() && case_id.len() <= MAX_ID_LEN,
            AlemanIusLexError::InvalidIdentifier
        );

        let case = &mut ctx.accounts.case_record;
        case.authority = ctx.accounts.authority.key();
        case.case_id = case_id.clone();
        case.created_at = Clock::get()?.unix_timestamp;
        case.document_count = 0;
        case.bump = ctx.bumps.case_record;

        emit!(CaseCreated {
            case_account: case.key(),
            case_id,
            authority: case.authority,
        });
        msg!("AlemanIusLexAI: Expediente creado");
        Ok(())
    }

    // 2. REGISTRAR DOCUMENTO
    pub fn register_document(
        ctx: Context<RegisterDocument>,
        document_id: String,
        document_hash: [u8; 32],
    ) -> Result<()> {
        require!(
            !document_id.is_empty() && document_id.len() <= MAX_ID_LEN,
            AlemanIusLexError::InvalidIdentifier
        );

        let now = Clock::get()?.unix_timestamp;
        let document = &mut ctx.accounts.document_record;
        document.case_account = ctx.accounts.case_record.key();
        document.authority = ctx.accounts.authority.key();
        document.document_id = document_id.clone();
        document.sha256 = document_hash;
        document.version = 1;
        document.created_at = now;
        document.updated_at = now;
        document.bump = ctx.bumps.document_record;

        let case = &mut ctx.accounts.case_record;
        case.document_count = case
            .document_count
            .checked_add(1)
            .ok_or(AlemanIusLexError::ArithmeticOverflow)?;

        emit!(DocumentRegistered {
            document_account: document.key(),
            case_account: document.case_account,
            document_id,
            document_hash,
            version: 1,
        });
        msg!("AlemanIusLexAI: Documento registrado");
        Ok(())
    }

    // 3. ACTUALIZAR DOCUMENTO
    pub fn update_document(ctx: Context<UpdateDocument>, new_hash: [u8; 32]) -> Result<()> {
        let document = &mut ctx.accounts.document_record;
        require!(
            document.sha256 != new_hash,
            AlemanIusLexError::SameDocumentHash
        );

        document.sha256 = new_hash;
        document.version = document
            .version
            .checked_add(1)
            .ok_or(AlemanIusLexError::ArithmeticOverflow)?;
        document.updated_at = Clock::get()?.unix_timestamp;

        emit!(DocumentUpdated {
            document_account: document.key(),
            document_id: document.document_id.clone(),
            new_hash,
            version: document.version,
        });
        msg!("AlemanIusLexAI: Version documental actualizada");
        Ok(())
    }

    // 4. VERIFICAR INTEGRIDAD
    pub fn verify_document(ctx: Context<VerifyDocument>, candidate_hash: [u8; 32]) -> Result<()> {
        let document = &ctx.accounts.document_record;
        let matches = document.sha256 == candidate_hash;

        emit!(DocumentVerification {
            document_account: document.key(),
            document_id: document.document_id.clone(),
            matches,
            stored_hash: document.sha256,
            candidate_hash,
            verified_at: Clock::get()?.unix_timestamp,
        });

        if matches {
            msg!("AlemanIusLexAI: Integridad verificada");
        } else {
            msg!("AlemanIusLexAI: Documento modificado o diferente");
        }
        Ok(())
    }

    // 5. CERRAR DOCUMENTO (la renta vuelve a la autoridad)
    pub fn close_document(ctx: Context<CloseDocument>) -> Result<()> {
        let case = &mut ctx.accounts.case_record;
        case.document_count = case
            .document_count
            .checked_sub(1)
            .ok_or(AlemanIusLexError::ArithmeticOverflow)?;

        msg!("AlemanIusLexAI: Documento cerrado");
        Ok(())
    }

    // 6. CERRAR EXPEDIENTE (solo si no tiene documentos)
    pub fn close_case(ctx: Context<CloseCase>) -> Result<()> {
        require!(
            ctx.accounts.case_record.document_count == 0,
            AlemanIusLexError::CaseHasDocuments
        );

        msg!("AlemanIusLexAI: Expediente cerrado");
        Ok(())
    }
}

// ==========================================
// CUENTAS: EXPEDIENTES
// ==========================================

#[derive(Accounts)]
#[instruction(case_id: String)]
pub struct InitializeCase<'info> {
    #[account(
        init,
        payer = authority,
        space = CaseRecord::SPACE,
        seeds = [b"case", authority.key().as_ref(), case_id.as_bytes()],
        bump
    )]
    pub case_record: Account<'info, CaseRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseCase<'info> {
    #[account(
        mut,
        close = authority,
        seeds = [b"case", authority.key().as_ref(), case_record.case_id.as_bytes()],
        bump = case_record.bump,
        has_one = authority
    )]
    pub case_record: Account<'info, CaseRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct CaseRecord {
    pub authority: Pubkey,
    pub case_id: String,
    pub created_at: i64,
    pub document_count: u32,
    pub bump: u8,
}

impl CaseRecord {
    pub const SPACE: usize = 8 + 32 + (4 + MAX_ID_LEN) + 8 + 4 + 1;
}

// ==========================================
// CUENTAS: DOCUMENTOS
// ==========================================

#[derive(Accounts)]
#[instruction(document_id: String)]
pub struct RegisterDocument<'info> {
    #[account(
        mut,
        seeds = [b"case", authority.key().as_ref(), case_record.case_id.as_bytes()],
        bump = case_record.bump,
        has_one = authority
    )]
    pub case_record: Account<'info, CaseRecord>,

    #[account(
        init,
        payer = authority,
        space = DocumentRecord::SPACE,
        seeds = [b"document", case_record.key().as_ref(), document_id.as_bytes()],
        bump
    )]
    pub document_record: Account<'info, DocumentRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateDocument<'info> {
    #[account(
        seeds = [b"case", authority.key().as_ref(), case_record.case_id.as_bytes()],
        bump = case_record.bump,
        has_one = authority
    )]
    pub case_record: Account<'info, CaseRecord>,

    #[account(
        mut,
        seeds = [b"document", case_record.key().as_ref(), document_record.document_id.as_bytes()],
        bump = document_record.bump,
        has_one = authority,
        constraint = document_record.case_account == case_record.key()
    )]
    pub document_record: Account<'info, DocumentRecord>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyDocument<'info> {
    pub document_record: Account<'info, DocumentRecord>,
}

#[derive(Accounts)]
pub struct CloseDocument<'info> {
    #[account(
        mut,
        seeds = [b"case", authority.key().as_ref(), case_record.case_id.as_bytes()],
        bump = case_record.bump,
        has_one = authority
    )]
    pub case_record: Account<'info, CaseRecord>,

    #[account(
        mut,
        close = authority,
        seeds = [b"document", case_record.key().as_ref(), document_record.document_id.as_bytes()],
        bump = document_record.bump,
        has_one = authority,
        constraint = document_record.case_account == case_record.key()
    )]
    pub document_record: Account<'info, DocumentRecord>,

    #[account(mut)]
    pub authority: Signer<'info>,
}

#[account]
pub struct DocumentRecord {
    pub case_account: Pubkey,
    pub authority: Pubkey,
    pub document_id: String,
    pub sha256: [u8; 32],
    pub version: u32,
    pub created_at: i64,
    pub updated_at: i64,
    pub bump: u8,
}

impl DocumentRecord {
    pub const SPACE: usize = 8 + 32 + 32 + (4 + MAX_ID_LEN) + 32 + 4 + 8 + 8 + 1;
}

// ==========================================
// EVENTOS
// ==========================================

#[event]
pub struct CaseCreated {
    pub case_account: Pubkey,
    pub case_id: String,
    pub authority: Pubkey,
}

#[event]
pub struct DocumentRegistered {
    pub document_account: Pubkey,
    pub case_account: Pubkey,
    pub document_id: String,
    pub document_hash: [u8; 32],
    pub version: u32,
}

#[event]
pub struct DocumentUpdated {
    pub document_account: Pubkey,
    pub document_id: String,
    pub new_hash: [u8; 32],
    pub version: u32,
}

#[event]
pub struct DocumentVerification {
    pub document_account: Pubkey,
    pub document_id: String,
    pub matches: bool,
    pub stored_hash: [u8; 32],
    pub candidate_hash: [u8; 32],
    pub verified_at: i64,
}

// ==========================================
// ERRORES
// ==========================================

#[error_code]
pub enum AlemanIusLexError {
    #[msg("Identificador vacio o demasiado largo")]
    InvalidIdentifier,

    #[msg("Desbordamiento numerico")]
    ArithmeticOverflow,

    #[msg("El documento tiene el mismo hash")]
    SameDocumentHash,

    #[msg("El expediente aun tiene documentos")]
    CaseHasDocuments,
}
