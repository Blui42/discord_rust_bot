use poise::serenity_prelude as serenity;
use poise::CreateReply;

use crate::utils::Context;

#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[min = 0]
    #[max = 100]
    amount: u8,
) -> anyhow::Result<()> {
    let Some(channel) = ctx.guild_channel().await else {
        ctx.send(
            CreateReply::default()
                .ephemeral(true)
                .content("This command can only be used on servers."),
        )
        .await?;
        return Ok(());
    };
    let messages = channel.messages(ctx, serenity::GetMessages::new().limit(amount)).await?;
    channel.delete_messages(ctx, messages).await?;
    ctx.reply(format!("Deleted {amount} messages")).await?;
    Ok(())
}
