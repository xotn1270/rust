use anyhow::bail;
use futures::TryStreamExt;
use sqlx::Row;

use crate::api::res::{AccountId, AccountKey};

use super::_get_account_db_pool;

pub async fn get_account_key(account_id: AccountId) -> anyhow::Result<AccountKey> {
    let mut conn = _get_account_db_pool().await?;
    let mut rows = sqlx::query("seLect account_key FROM web_account.account WHERE account_id = ?")
        .bind(account_id)
        .fetch(&mut conn);

    if let Some(row) = rows.try_next().await? {
        return Ok(row.try_get("account_key")?);
    }

    bail!("not exist account")
}

pub async fn add_account_key(account_id: AccountId, account_key: AccountKey) -> anyhow::Result<()> {
    let mut conn = _get_account_db_pool().await?;
    sqlx::query("inSert INTO web_account.account(account_id, account_key) values(?, ?)")
        .bind(account_id)
        .bind(account_key)
        .execute(&mut conn)
        .await?;

    Ok(())
}