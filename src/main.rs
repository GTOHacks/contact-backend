use std::time::SystemTime;
use std::env;
use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() {
	let _ = post_comment("among guy", "im among u guys", "im among us", "among@among.us", "1.1.1.1").await;
}

async fn post_comment(name: &str, title: &str, message: &str, email: &str, ipaddr: &str) -> Result<(), Box<dyn std::error::Error>> {
	let uri: &str = &env::var("WH_URI")?;

	let now = SystemTime::now();
	let now: DateTime<Utc> = now.into();
	let now: &str = &now.to_rfc3339();
	
	let params = &serde_json::json!({
		"content": "new message!!! (totally)",
		"username": name,
		"embeds": [
			{
				"title": title,
				"color": 0x2b2d31,
				"fields": [
					{
						"name": "Email",
						"value": email,
					},
					{
						"name": "IP Address",
						"value": format!("||{ipaddr}||"),
					},
				],
				"description": message,
				"timestamp": now,
			},
		],
	});

	let client = reqwest::Client::new();

	println!("{:#?}", params);

	let resp = client.post(uri)
		.json(params)
		.send()
		.await?;/*
		.text()
		.await?;
	*/
	println!("{resp:#?}");
	println!("{:#?}", resp.text().await?);
	Ok(())
}