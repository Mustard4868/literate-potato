use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Return the wiki link for the search query
#[poise::command(slash_command)]
async fn wiki(
    ctx: Context<'_>,
    #[description = "Search query"] query: Option<String>,
) -> Result<(), Error> {
    let base_string = "https://wiki.warframe.com/w/";
    
    let formatted_text = query.map(|text| {
        text.to_lowercase() // Convert the whole string to lowercase first
            .split_whitespace() // Split at spaces
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<String>>()
            .join("_")
    });

    let response = match formatted_text {
        Some(text) => format!("{}{}", base_string, text),
        None => "No input provided.".to_string(),
    };

    ctx.say(response).await?;
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