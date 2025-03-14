use poise::serenity_prelude as serenity;
use reqwest;
use scraper::{Html, Selector};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Return the wiki link for the search query
#[poise::command(slash_command)]
async fn wiki(
    ctx: Context<'_>,
    #[description = "Search query"] query: Option<String>,
) -> Result<(), Error> {
    let base_url = "https://wiki.warframe.com/w/";
    if let Some(query) = query {
        let words: Vec<String> = query
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect();
        
        let search_url = format!("{}{}", base_url, words.join("_"));
        
        let response = reqwest::get(search_url.clone()).await?.text().await?;
        
        let description = Html::parse_document(&response).select(&Selector::parse("meta[property=\"og:description\"]").unwrap())
            .next()
            .and_then(|n| n.value().attr("content"))
            .unwrap_or("No description found")
            .to_string();

        let image = Html::parse_document(&response).select(&Selector::parse("meta[property=\"og:image\"]").unwrap())
            .next()
            .and_then(|n| n.value().attr("content"))
            .map(|s| s.to_string());
        
        let embed = serenity::CreateEmbed::default()
            .title(words.join(" "))
            .url(search_url)
            .description(description)
            .thumbnail(image.unwrap_or_default());

        let builder = poise::CreateReply::default()
            .embed(embed);

        ctx.send(builder).await?;

    }
    Ok(())
}


#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![wiki()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}