use std::collections::HashMap;

use robbb_db::{fetch::Fetch, fetch_field::FetchField};
use robbb_util::embeds;
use serenity::builder::CreateEmbedAuthor;

use super::*;

/// Fetch a users system information.
#[poise::command(slash_command, guild_only, prefix_command, rename = "fetch")]
pub async fn fetch(
    ctx: Ctx<'_>,
    #[description = "The user"] user: Option<Member>,
    #[description = "The specific field you care about"] field: Option<FetchField>,
) -> Res<()> {
    let db = ctx.get_db();
    let user = member_or_self(ctx, user).await?;
    ctx.defer().await?;

    // Query the database
    let fetch_info: Fetch = db.get_fetch(user.user.id).await?.unwrap_or_else(|| Fetch {
        user: user.user.id,
        info: HashMap::new(),
        create_date: None,
    });

    let create_date = fetch_info.create_date;
    let fetch_data: Vec<(FetchField, String)> = fetch_info.get_values_ordered();
    let color = user.colour(ctx.serenity_context());

    match field {
        // Handle fetching a single field
        Some(desired_field) => {
            let (field_name, value) = fetch_data
                .into_iter()
                .find(|(k, _)| k == &desired_field)
                .user_error("Failed to get that value. Maybe the user hasn't set it?")?;
            let mut embed = embeds::base_embed(ctx.serenity_context())
                .await
                .author(CreateEmbedAuthor::new(user.user.tag()).icon_url(user.user.face()))
                .title(format!("{}'s {}", user.user.name, field_name))
                .color_opt(color);
            if let Some(date) = create_date {
                embed = embed.timestamp(date);
            }
            if desired_field == FetchField::Image {
                embed = embed.image(value);
            } else if let Some(value) = format_fetch_field_value(&field_name, value) {
                embed = embed.description(value);
            } else {
                embed = embed.description("Not set");
            }
            ctx.reply_embed(embed).await?;
        }

        // Handle fetching all fields
        None => {
            let mut embed = embeds::base_embed(ctx.serenity_context()).await;
            embed = embed.author_user(&user.user);
            embed = embed.color_opt(color);
            if let Some(date) = create_date {
                embed = embed.timestamp(date);
            }

            for (key, value) in fetch_data {
                if key == FetchField::Image {
                    embed = embed.image(value);
                } else {
                    if key == FetchField::Distro {
                        if let Some(url) = find_distro_image(&value) {
                            embed = embed.thumbnail(url);
                        }
                    }
                    embed = embed.field_opt(
                        key.to_string(),
                        format_fetch_field_value(&key, value),
                        true,
                    );
                }
            }
            ctx.reply_embed(embed).await?;
        }
    }

    Ok(())
}
