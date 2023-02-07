use std::io::Write;
use ureq::serde_json::Value;

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) discord/1.0.9010 Chrome/91.0.4472.164 Electron/13.6.6 Safari/537.36";
const ENDPOINT_URL: &str = "https://discord.com/api/v9/users/@me";

fn main() -> Result<(), token_extractor::ExtractDiscordTokenError> {
    let token = token_extractor::get_discord_token()?;

    let req = ureq::get(ENDPOINT_URL)
        .set("User-Agent", USER_AGENT)
        .set("Authorization", &token);
    if let Some(json_response) = req
        .call()
        .ok()
        .and_then(|resp| resp.into_json::<Value>().ok())
    {
        let username_val = json_response
            .get("username")
            .and_then(|value| value.as_str());
        let discriminator_val = json_response
            .get("discriminator")
            .and_then(|value| value.as_str());
        if let (Some(username), Some(discriminator)) = (username_val, discriminator_val) {
            println!("Username: {username}#{discriminator}");
        }

        if let Some(email) = json_response.get("email").and_then(|value| value.as_str()) {
            println!("Email: {email}");
        }
    }

    println!("Session token: {token}");

    println!();
    print!("Press enter to close...");
    let _ = std::io::stdout().flush();
    std::io::stdin().lines().next();

    Ok(())
}
