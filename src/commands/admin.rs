use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::utils::Context;

/// Delete some messages
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_MESSAGES",
    required_bot_permissions = "MANAGE_MESSAGES"
)]
pub async fn delete(
    ctx: Context<'_>,
    #[min = 0]
    #[max = 100]
    amount: u8,
) -> anyhow::Result<()> {
    let channel = ctx.channel_id();
    let messages = channel.messages(ctx, serenity::GetMessages::new().limit(amount)).await?;
    channel.delete_messages(ctx, messages).await?;
    ctx.send(CreateReply::default().ephemeral(true).content(format!("Deleted {amount} messages")))
        .await?;
    Ok(())
}
